// LOC-46: client C# thật gọi MatchingService.SuggestMatch của geo-engine (Rust) và Dispatch stub trong bộ nhớ.
//
//   dotnet run --project examples/toi-noi-location/dotnet/DispatchStub -c Release -- contract --addr http://127.0.0.1:PORT
//
// Máy chủ phải chạy với đồ thị fixture (toạ độ khu mẫu). Thoát mã 0 nếu mọi kiểm tra PASS, 1 nếu có FAIL.
// Phần A kiểm hợp đồng thông điệp với máy chủ Rust thật. Phần B điều khiển InMemoryDispatch bằng gợi ý thật của máy chủ.
// Đây là kiểm thử hợp đồng và bất biến của stub, không phải kiểm thử đồng thời hay bền vững của Dispatch thật.

using Grpc.Core;
using Grpc.Net.Client;
using ToiNoi.Location.DispatchStub;
using ToiNoi.Location.V1;

if (args.FirstOrDefault() != "contract")
{
    Console.WriteLine("dùng: dotnet run --project .../DispatchStub -- contract --addr http://127.0.0.1:PORT");
    return 2;
}
var addrIdx = Array.IndexOf(args, "--addr");
var address = addrIdx >= 0 && addrIdx + 1 < args.Length ? args[addrIdx + 1] : "http://127.0.0.1:50071";

using var channel = GrpcChannel.ForAddress(address);
var client = new MatchingService.MatchingServiceClient(channel);
int pass = 0, fail = 0;

void Check(string name, bool ok, string detail = "")
{
    Console.WriteLine($"  [{(ok ? "PASS" : "FAIL")}] {name}{(detail.Length > 0 ? $" — {detail}" : "")}");
    if (ok) pass++; else fail++;
}

Console.WriteLine($"client C# (Dispatch stub) gọi {address}");

// Thế giới mẫu trong khu mẫu của đồ thị fixture
InMemoryDispatch World(FakeClock clock, int drivers = 4, int bookings = 3)
{
    var d = new InMemoryDispatch(clock);
    (string, double, double)[] bs = [("BK-001", 10.7705, 106.6915), ("BK-002", 10.7745, 106.7005), ("BK-003", 10.7725, 106.6955)];
    (string, double, double)[] ds = [("DR-001", 10.7710, 106.6920), ("DR-002", 10.7740, 106.7000), ("DR-003", 10.7720, 106.6960), ("DR-004", 10.7695, 106.7010)];
    foreach (var (id, la, ln) in bs.Take(bookings)) d.AddBooking(id, la, ln);
    foreach (var (id, la, ln) in ds.Take(drivers)) d.AddDriver(id, la, ln);
    return d;
}

const long T0 = 1_700_000_000_000;

// ---------------------------------------------------------------- A. hợp đồng thông điệp với Rust
Console.WriteLine("\n== A. hợp đồng SuggestMatch với máy chủ Rust ==");
{
    var clock = new FakeClock(T0);
    var d = World(clock);
    var req = d.BuildRequest("REQ-A1", snapshot: 77);
    var res = await client.SuggestMatchAsync(req, deadline: DateTime.UtcNow.AddSeconds(5));
    Console.WriteLine($"  gợi ý: {string.Join(", ", res.Suggestions.Select(s => $"{s.BookingId}←{s.DriverId} eta {s.EtaMs / 1000.0:F0}s"))}; stats {res.Stats.Algo} {res.Stats.Rows}×{res.Stats.Cols}, ma trận {res.Stats.MatrixUs} µs, giải {res.Stats.SolveUs} µs");
    Check("A1 chất lượng ROUTED, đủ cặp, echo request_id/snapshot/policy",
        res.Quality == RoutingQuality.Routed && res.Suggestions.Count == 3 && res.RequestId == "REQ-A1" && res.SnapshotVersion == 77 && res.PolicyVersion == "v0-eta",
        $"{res.Suggestions.Count} gợi ý, source {res.Source}");
    Check("A2 mỗi booking và mỗi tài xế tối đa một lần; suggestion_id duy nhất",
        res.Suggestions.Select(s => s.BookingId).Distinct().Count() == res.Suggestions.Count
        && res.Suggestions.Select(s => s.DriverId).Distinct().Count() == res.Suggestions.Count
        && res.Suggestions.Select(s => s.SuggestionId).Distinct().Count() == res.Suggestions.Count);
    var sentVersions = req.Drivers.ToDictionary(x => x.DriverId, x => x.StateVersion);
    Check("A3 TTL: valid_until = now + 10 s; driver_state_version khớp bản đã gửi; eta > 0",
        res.Suggestions.All(s => s.ValidUntilMs == T0 + 10_000 && s.DriverStateVersion == sentVersions[s.DriverId] && s.EtaMs > 0 && s.SuggestionId.StartsWith("sg-")));
    var again = await client.SuggestMatchAsync(req);
    Check("A4 cùng request_id, cùng nội dung → đúng cùng gợi ý (idempotent, tất định)", again.Suggestions.SequenceEqual(res.Suggestions));
    var req2 = req.Clone(); req2.RequestId = "REQ-A4b";
    var other = await client.SuggestMatchAsync(req2);
    Check("A5 đổi request_id → cùng cặp nhưng suggestion_id khác",
        other.Suggestions.Select(s => (s.BookingId, s.DriverId)).SequenceEqual(res.Suggestions.Select(s => (s.BookingId, s.DriverId)))
        && other.Suggestions.Zip(res.Suggestions).All(p => p.First.SuggestionId != p.Second.SuggestionId));

    async Task<StatusCode?> Code(SuggestMatchRequest r)
    {
        try { await client.SuggestMatchAsync(r); return null; }
        catch (RpcException e) { return e.StatusCode; }
    }
    var bad1 = req.Clone(); bad1.RequestId = "";
    var bad2 = req.Clone(); bad2.Drivers.Clear();
    var bad3 = req.Clone(); bad3.Pending[0].Pickup = null;
    var bad4 = req.Clone(); bad4.Drivers.Add(bad4.Drivers[0].Clone());
    Check("A6 yêu cầu sai (thiếu request_id / không tài xế / thiếu điểm đón / trùng driver_id) → InvalidArgument",
        await Code(bad1) == StatusCode.InvalidArgument && await Code(bad2) == StatusCode.InvalidArgument
        && await Code(bad3) == StatusCode.InvalidArgument && await Code(bad4) == StatusCode.InvalidArgument);

    var budget0 = req.Clone(); budget0.RequestId = "REQ-A7"; budget0.ComputeBudgetMs = 0;
    var est = await client.SuggestMatchAsync(budget0);
    Check("A7 hết ngân sách tính toán (compute_budget_ms = 0) → ESTIMATED, không lỗi, vẫn là phép gán hợp lệ",
        est.Quality == RoutingQuality.Estimated && est.Source == "straight-line:budget" && est.Suggestions.Count == 3
        && est.Suggestions.Select(s => s.DriverId).Distinct().Count() == 3, $"source {est.Source}, algo {est.Stats.Algo}");

    var hard = req.Clone(); hard.RequestId = "REQ-A8"; hard.Policy = new MatchPolicy { Version = "v1-test", HardPickupS = 1 };
    var none = await client.SuggestMatchAsync(hard);
    Check("A8 giới hạn cứng đón xa 1 s → không gợi ý nào, mọi booking unmatched BEYOND_PICKUP_LIMIT, policy_version được echo",
        none.Suggestions.Count == 0 && none.Unmatched.Count == 3 && none.Unmatched.All(u => u.Reason == UnmatchedReason.BeyondPickupLimit) && none.PolicyVersion == "v1-test");

    var chain = req.Clone(); chain.RequestId = "REQ-A9";
    var baseEta = res.Suggestions.ToDictionary(s => s.BookingId, s => s.EtaMs);
    chain.Pending.Clear(); chain.Pending.Add(req.Pending[0]);
    chain.Drivers.Clear(); chain.Drivers.Add(req.Drivers[0]);
    var one = await client.SuggestMatchAsync(chain);
    chain.RequestId = "REQ-A9b"; chain.Drivers[0].BusyRemainingMs = 60_000;
    var busy = await client.SuggestMatchAsync(chain);
    Check("A9 tài xế đang chở khách (busy_remaining_ms = 60 s) → ETA cộng đúng 60 s",
        one.Suggestions.Count == 1 && busy.Suggestions.Count == 1 && busy.Suggestions[0].EtaMs == one.Suggestions[0].EtaMs + 60_000,
        $"{one.Suggestions[0].EtaMs} ms → {busy.Suggestions[0].EtaMs} ms");
}

// ---------------------------------------------------------------- B. Dispatch stub điều khiển bằng gợi ý thật
Console.WriteLine("\n== B. Dispatch stub (C# quyết định) dùng gợi ý thật của Rust ==");
Console.WriteLine("  TTL gợi ý 10 s (hạn để ÁP DỤNG gợi ý); TTL offer 15 s (hạn tài xế trả lời). Hai TTL độc lập: gợi ý hết hạn thì không tạo được offer, offer đã tạo chỉ theo TTL của offer.");

async Task<SuggestMatchResponse> Suggest(InMemoryDispatch d, string id, string? only = null, ISet<string>? exclude = null)
    => await client.SuggestMatchAsync(d.BuildRequest(id, only, exclude), deadline: DateTime.UtcNow.AddSeconds(5));

// B1 đường vui
{
    var clock = new FakeClock(T0); var d = World(clock);
    var res = await Suggest(d, "REQ-B1");
    var applied = res.Suggestions.Select(s => d.ApplySuggestion(s)).ToList();
    var offers = applied.Where(a => a.Offer is not null).Select(a => a.Offer!).ToList();
    Check("B1a áp dụng 3 gợi ý → 3 offer, mỗi offer có offer_id riêng, hết hạn sau 15 s, tài xế bị lease",
        applied.All(a => a.Outcome == ApplyOutcome.Offered) && offers.Select(o => o.Id).Distinct().Count() == 3
        && offers.All(o => o.ExpiresMs == T0 + 15_000) && d.Drivers.Values.Count(x => x.State == DriverState.Leased) == 3);
    var acc = offers.Select(o => d.Accept(o.Id, $"key-{o.Id}")).ToList();
    Check("B1b accept cả ba → 3 booking Assigned, mỗi booking một tài xế, bất biến §4.5 đúng",
        acc.All(r => r.Outcome == DecisionOutcome.Accepted) && d.Bookings.Values.All(b => b.State == BookingState.Assigned && b.DriverId is not null)
        && d.CheckInvariants().Count == 0, string.Join("; ", d.CheckInvariants()));
}

// B2 tài xế từ chối → gợi ý lại, loại tài xế đó (C# nhớ, Rust không biết)
{
    var clock = new FakeClock(T0); var d = World(clock);
    var res = await Suggest(d, "REQ-B2", only: "BK-001");
    var s1 = res.Suggestions[0];
    var o1 = d.ApplySuggestion(s1).Offer!;
    var rej = d.Reject(o1.Id, "rej-1");
    var res2 = await Suggest(d, "REQ-B2b", only: "BK-001", exclude: d.Bookings["BK-001"].RejectedBy);
    var s2 = res2.Suggestions[0];
    var o2 = d.ApplySuggestion(s2).Offer;
    Check("B2 từ chối → offer cũ Rejected, tài xế cũ rảnh lại (state_version tăng), gợi ý lại chọn tài xế khác, offer mới có offer_id mới",
        rej.Outcome == DecisionOutcome.Rejected && o1.Status == OfferStatus.Rejected && d.Drivers[s1.DriverId].State == DriverState.Idle
        && d.Drivers[s1.DriverId].StateVersion > s1.DriverStateVersion && s2.DriverId != s1.DriverId && o2 is not null && o2.Id != o1.Id && d.CheckInvariants().Count == 0,
        $"{s1.DriverId} từ chối → {s2.DriverId}");
}

// B3 gợi ý cũ: tài xế đổi trạng thái giữa lúc Rust tính và lúc C# áp dụng
{
    var clock = new FakeClock(T0); var d = World(clock);
    var res = await Suggest(d, "REQ-B3");
    var victim = res.Suggestions[0];
    d.MoveDriver(victim.DriverId, 10.7696, 106.6910); // state_version tăng
    var results = res.Suggestions.Select(s => (s, r: d.ApplySuggestion(s))).ToList();
    var vr = results.First(x => x.s.SuggestionId == victim.SuggestionId).r;
    Check("B3 gợi ý cũ (tài xế đã đổi state_version) bị từ chối, không lease, booking vẫn chờ; các gợi ý còn lại vẫn áp dụng được",
        vr.Outcome == ApplyOutcome.StaleDriverVersion && d.Drivers[victim.DriverId].State == DriverState.Idle && d.Bookings[victim.BookingId].State == BookingState.Pending
        && results.Where(x => x.s.SuggestionId != victim.SuggestionId).All(x => x.r.Outcome == ApplyOutcome.Offered) && d.CheckInvariants().Count == 0);
    var res2 = await Suggest(d, "REQ-B3b", only: victim.BookingId);
    Check("B3b gợi ý lại với phiên bản mới của tài xế thì áp dụng được", res2.Suggestions.Count == 1 && d.ApplySuggestion(res2.Suggestions[0]).Outcome == ApplyOutcome.Offered);
}

// B4 hết hạn: gợi ý quá valid_until; phản hồi muộn sau TTL của offer
{
    var clock = new FakeClock(T0); var d = World(clock);
    var res = await Suggest(d, "REQ-B4");
    clock.Advance(10_001);
    var late = d.ApplySuggestion(res.Suggestions[0]);
    Check("B4a áp dụng gợi ý sau valid_until (10 s) → từ chối StaleSuggestionExpired, không offer", late.Outcome == ApplyOutcome.StaleSuggestionExpired && d.Offers.Count == 0);

    var res2 = await Suggest(d, "REQ-B4b", only: "BK-001");
    var s = res2.Suggestions[0];
    var o = d.ApplySuggestion(s).Offer!;
    clock.Advance(15_001); // quá TTL của offer
    var r1 = d.Accept(o.Id, "late-key");
    var r2 = d.Accept(o.Id, "late-key");
    Check("B4b accept muộn (sau TTL offer 15 s) → LateResponse; offer Expired; tài xế nhả về Idle; booking về Pending; không assignment",
        r1.Outcome == DecisionOutcome.LateResponse && o.Status == OfferStatus.Expired && d.Drivers[s.DriverId].State == DriverState.Idle
        && d.Bookings["BK-001"].State == BookingState.Pending && d.Bookings["BK-001"].DriverId is null && d.CheckInvariants().Count == 0);
    Check("B4c gửi lại cùng khoá idempotency → đúng kết quả cũ (LateResponse), không tác dụng phụ", ReferenceEquals(r1, r2));
    var res3 = await Suggest(d, "REQ-B4c", only: "BK-001");
    Check("B4d sau khi nhả, booking được gợi ý và áp dụng lại bình thường", res3.Suggestions.Count == 1 && d.ApplySuggestion(res3.Suggestions[0]).Outcome == ApplyOutcome.Offered);

    // ExpireDue: quét nền thay vì chờ phản hồi
    var clock2 = new FakeClock(T0); var d2 = World(clock2);
    var q = await Suggest(d2, "REQ-B4e", only: "BK-002");
    var o2 = d2.ApplySuggestion(q.Suggestions[0]).Offer!;
    clock2.Advance(15_001);
    Check("B4e quét hết hạn nền (ExpireDue) cũng nhả lease", d2.ExpireDue() == 1 && o2.Status == OfferStatus.Expired && d2.Bookings["BK-002"].State == BookingState.Pending);
}

// B5 idempotency của accept/reject
{
    var clock = new FakeClock(T0); var d = World(clock);
    var res = await Suggest(d, "REQ-B5", only: "BK-003");
    var o = d.ApplySuggestion(res.Suggestions[0]).Offer!;
    var a1 = d.Accept(o.Id, "same-key");
    var a2 = d.Accept(o.Id, "same-key");
    var a3 = d.Accept(o.Id, "other-key");
    var rj = d.Reject(o.Id, "reject-after-accept");
    Check("B5 accept hai lần cùng khoá → cùng kết quả, một assignment; khoá khác → AlreadyDecided; reject sau accept → AlreadyDecided",
        a1.Outcome == DecisionOutcome.Accepted && ReferenceEquals(a1, a2) && a3.Outcome == DecisionOutcome.AlreadyDecided && rj.Outcome == DecisionOutcome.AlreadyDecided
        && d.Offers.Count(x => x.Status == OfferStatus.Accepted) == 1 && d.Bookings["BK-003"].DriverId == o.DriverId && d.CheckInvariants().Count == 0);
    var rejOnce = (a: d.Reject("OF-9999", "k"), b: d.Reject("OF-9999", "k"));
    Check("B5b offer không tồn tại → UnknownOffer, lặp lại cùng khoá cho cùng kết quả", rejOnce.a.Outcome == DecisionOutcome.UnknownOffer && ReferenceEquals(rejOnce.a, rejOnce.b));
}

// B6 hai gợi ý tranh cùng một tài xế
{
    var clock = new FakeClock(T0); var d = World(clock, drivers: 1, bookings: 2);
    var ra = await Suggest(d, "REQ-B6a", only: "BK-001");
    var rb = await Suggest(d, "REQ-B6b", only: "BK-002");
    var sa = ra.Suggestions[0]; var sb = rb.Suggestions[0];
    var first = d.ApplySuggestion(sa);
    var second = d.ApplySuggestion(sb);
    Check("B6 hai gợi ý (hai lời gọi Rust khác nhau) cùng chọn tài xế DR-001 → chỉ một lease, gợi ý thứ hai bị từ chối, booking thứ hai vẫn chờ",
        sa.DriverId == sb.DriverId && first.Outcome == ApplyOutcome.Offered && second.Outcome == ApplyOutcome.DriverNotIdle
        && d.Drivers.Values.Count(x => x.State == DriverState.Leased) == 1 && d.Bookings["BK-002"].State == BookingState.Pending && d.CheckInvariants().Count == 0);
    d.Accept(first.Offer!.Id, "k");
    var third = d.ApplySuggestion(sa);
    Check("B6b gợi ý cho booking đã Assigned → BookingNotPending", third.Outcome == ApplyOutcome.BookingNotPending);
}

Console.WriteLine($"\nKẾT QUẢ: {pass} PASS, {fail} FAIL");
Console.WriteLine(fail == 0 ? "PASS" : "FAIL");
return fail == 0 ? 0 : 1;

// STUB của Dispatch (C#) cho LOC-46: trong bộ nhớ, một tiến trình, không lưu bền, không Redis, không đồng thời thật.
// Mục đích: chứng minh hợp đồng «Rust đề xuất, C# quyết định» và các bất biến của kiến trúc §4.5:
//   - một booking chỉ có một assignment hợp lệ; một tài xế chỉ bị giữ (lease) bởi một offer tại một thời điểm;
//   - offer có offer_id và TTL riêng; accept/reject idempotent theo khoá; phản hồi muộn (sau TTL) bị từ chối;
//   - gợi ý của Rust luôn bị kiểm lại với trạng thái của C# trước khi thành offer (gợi ý cũ bị từ chối).
// Đây KHÔNG phải Dispatch thật: không transaction, không persist assignment, không test tranh chấp đa luồng.

using ToiNoi.Location.V1;

namespace ToiNoi.Location.DispatchStub;

public interface IClock { long NowMs { get; } }

/// Đồng hồ giả để test tất định.
public sealed class FakeClock(long start) : IClock
{
    public long NowMs { get; private set; } = start;
    public void Advance(long ms) => NowMs += ms;
}

public enum BookingState { Pending, Offered, Assigned }
public enum DriverState { Idle, Leased, OnTrip }
public enum OfferStatus { Open, Accepted, Rejected, Expired }

public sealed class Booking(string id, double lat, double lng, long requestedAtMs)
{
    public string Id { get; } = id;
    public double Lat { get; } = lat;
    public double Lng { get; } = lng;
    public long RequestedAtMs { get; } = requestedAtMs;
    public BookingState State { get; set; } = BookingState.Pending;
    public string? DriverId { get; set; }
    /// Tài xế đã từ chối booking này: C# nhớ, Rust không biết.
    public HashSet<string> RejectedBy { get; } = [];
}

public sealed class Driver(string id, double lat, double lng)
{
    public string Id { get; } = id;
    public double Lat { get; set; } = lat;
    public double Lng { get; set; } = lng;
    /// Tăng mỗi lần C# đổi trạng thái hoặc vị trí đáng kể của tài xế. Gợi ý ghi lại giá trị đã dùng.
    public ulong StateVersion { get; set; } = 1;
    public DriverState State { get; set; } = DriverState.Idle;
    public long IdleSinceMs { get; set; }
}

public sealed class Offer(string id, string bookingId, string driverId, string suggestionId, long createdMs, long expiresMs)
{
    public string Id { get; } = id;
    public string BookingId { get; } = bookingId;
    public string DriverId { get; } = driverId;
    public string SuggestionId { get; } = suggestionId;
    public long CreatedMs { get; } = createdMs;
    public long ExpiresMs { get; } = expiresMs;
    public OfferStatus Status { get; set; } = OfferStatus.Open;
}

public enum ApplyOutcome { Offered, StaleSuggestionExpired, StaleDriverVersion, DriverNotIdle, BookingNotPending, UnknownBookingOrDriver }

public sealed record ApplyResult(ApplyOutcome Outcome, Offer? Offer);

public enum DecisionOutcome { Accepted, Rejected, LateResponse, AlreadyDecided, UnknownOffer }

public sealed record DecisionResult(DecisionOutcome Outcome, string Detail);

public sealed class InMemoryDispatch(IClock clock, long offerTtlMs = 15_000)
{
    public Dictionary<string, Booking> Bookings { get; } = [];
    public Dictionary<string, Driver> Drivers { get; } = [];
    public List<Offer> Offers { get; } = [];
    public long OfferTtlMs { get; } = offerTtlMs;
    // idempotency: khoá (thao tác, offer, khoá do client gửi) → kết quả đã trả lần đầu
    private readonly Dictionary<string, DecisionResult> _idem = [];
    private int _offerSeq;

    public Booking AddBooking(string id, double lat, double lng)
    {
        var b = new Booking(id, lat, lng, clock.NowMs);
        Bookings[id] = b;
        return b;
    }

    public Driver AddDriver(string id, double lat, double lng)
    {
        var d = new Driver(id, lat, lng) { IdleSinceMs = clock.NowMs };
        Drivers[id] = d;
        return d;
    }

    public void MoveDriver(string id, double lat, double lng)
    {
        var d = Drivers[id];
        d.Lat = lat;
        d.Lng = lng;
        d.StateVersion++;
    }

    static LatLngE7 E7(double lat, double lng) => new() { LatE7 = (int)Math.Round(lat * 1e7), LngE7 = (int)Math.Round(lng * 1e7) };

    /// Dựng yêu cầu SuggestMatch từ trạng thái của C#: các booking đang chờ (hoặc `only`) và tài xế rảnh (trừ `excludeDrivers`).
    public SuggestMatchRequest BuildRequest(string requestId, string? only = null, ISet<string>? excludeDrivers = null,
        MatchPolicy? policy = null, ulong snapshot = 1)
    {
        var req = new SuggestMatchRequest
        {
            RequestId = requestId,
            CityCode = "HCM",
            NowMs = clock.NowMs,
            SnapshotVersion = snapshot,
            Policy = policy ?? new MatchPolicy(),
        };
        foreach (var b in Bookings.Values.Where(b => b.State == BookingState.Pending && (only is null || b.Id == only)).OrderBy(b => b.Id))
            req.Pending.Add(new PendingRequest { BookingId = b.Id, Pickup = E7(b.Lat, b.Lng), RequestedAtMs = b.RequestedAtMs });
        foreach (var d in Drivers.Values.Where(d => d.State == DriverState.Idle && excludeDrivers?.Contains(d.Id) != true).OrderBy(d => d.Id))
            req.Drivers.Add(new DriverCandidate { DriverId = d.Id, Position = E7(d.Lat, d.Lng), StateVersion = d.StateVersion, IdleSinceMs = d.IdleSinceMs });
        return req;
    }

    /// C# QUYẾT ĐỊNH: kiểm lại gợi ý với trạng thái của chính nó rồi mới lấy lease và tạo offer.
    public ApplyResult ApplySuggestion(Suggestion s)
    {
        if (clock.NowMs > s.ValidUntilMs) return new(ApplyOutcome.StaleSuggestionExpired, null);
        if (!Bookings.TryGetValue(s.BookingId, out var b) || !Drivers.TryGetValue(s.DriverId, out var d))
            return new(ApplyOutcome.UnknownBookingOrDriver, null);
        if (b.State != BookingState.Pending) return new(ApplyOutcome.BookingNotPending, null);
        if (d.State != DriverState.Idle) return new(ApplyOutcome.DriverNotIdle, null);
        if (d.StateVersion != s.DriverStateVersion) return new(ApplyOutcome.StaleDriverVersion, null);
        var o = new Offer($"OF-{++_offerSeq:D4}", b.Id, d.Id, s.SuggestionId, clock.NowMs, clock.NowMs + OfferTtlMs);
        Offers.Add(o);
        d.State = DriverState.Leased; // lease: không ai khác được offer cho tài xế này
        d.StateVersion++;
        b.State = BookingState.Offered;
        return new(ApplyOutcome.Offered, o);
    }

    /// Quét offer hết hạn: nhả lease, booking về hàng chờ.
    public int ExpireDue()
    {
        var n = 0;
        foreach (var o in Offers.Where(o => o.Status == OfferStatus.Open && clock.NowMs > o.ExpiresMs))
        {
            Release(o, OfferStatus.Expired);
            n++;
        }
        return n;
    }

    void Release(Offer o, OfferStatus status)
    {
        o.Status = status;
        var d = Drivers[o.DriverId];
        d.State = DriverState.Idle;
        d.IdleSinceMs = clock.NowMs;
        d.StateVersion++;
        Bookings[o.BookingId].State = BookingState.Pending;
    }

    DecisionResult Idempotent(string op, string offerId, string key, Func<DecisionResult> run)
    {
        var k = $"{op}|{offerId}|{key}";
        if (_idem.TryGetValue(k, out var prev)) return prev; // cùng khoá → đúng kết quả lần đầu, không tác dụng phụ
        return _idem[k] = run();
    }

    public DecisionResult Accept(string offerId, string idempotencyKey) => Idempotent("accept", offerId, idempotencyKey, () =>
    {
        var o = Offers.FirstOrDefault(x => x.Id == offerId);
        if (o is null) return new(DecisionOutcome.UnknownOffer, "offer không tồn tại");
        if (o.Status == OfferStatus.Open && clock.NowMs > o.ExpiresMs)
        {
            Release(o, OfferStatus.Expired); // phản hồi muộn: offer hết hạn, nhả tài xế
            return new(DecisionOutcome.LateResponse, $"offer hết hạn lúc {o.ExpiresMs}, nhận phản hồi lúc {clock.NowMs}");
        }
        if (o.Status == OfferStatus.Expired) return new(DecisionOutcome.LateResponse, "offer đã hết hạn");
        if (o.Status != OfferStatus.Open) return new(DecisionOutcome.AlreadyDecided, $"offer đã {o.Status}");
        o.Status = OfferStatus.Accepted;
        var d = Drivers[o.DriverId];
        d.State = DriverState.OnTrip;
        d.StateVersion++;
        var b = Bookings[o.BookingId];
        b.State = BookingState.Assigned;
        b.DriverId = d.Id;
        return new(DecisionOutcome.Accepted, $"booking {b.Id} ← tài xế {d.Id}");
    });

    public DecisionResult Reject(string offerId, string idempotencyKey) => Idempotent("reject", offerId, idempotencyKey, () =>
    {
        var o = Offers.FirstOrDefault(x => x.Id == offerId);
        if (o is null) return new(DecisionOutcome.UnknownOffer, "offer không tồn tại");
        if (o.Status == OfferStatus.Open && clock.NowMs > o.ExpiresMs)
        {
            Release(o, OfferStatus.Expired);
            return new(DecisionOutcome.LateResponse, "từ chối đến muộn, offer đã hết hạn");
        }
        if (o.Status != OfferStatus.Open) return new(DecisionOutcome.AlreadyDecided, $"offer đã {o.Status}");
        Bookings[o.BookingId].RejectedBy.Add(o.DriverId);
        Release(o, OfferStatus.Rejected);
        return new(DecisionOutcome.Rejected, $"tài xế {o.DriverId} từ chối {o.BookingId}");
    });

    /// Bất biến của §4.5; trả danh sách vi phạm (rỗng = đúng).
    public List<string> CheckInvariants()
    {
        var bad = new List<string>();
        foreach (var g in Offers.Where(o => o.Status is OfferStatus.Open or OfferStatus.Accepted).GroupBy(o => o.BookingId).Where(g => g.Count() > 1))
            bad.Add($"booking {g.Key} có {g.Count()} offer còn hiệu lực");
        foreach (var g in Offers.Where(o => o.Status is OfferStatus.Open or OfferStatus.Accepted).GroupBy(o => o.DriverId).Where(g => g.Count() > 1))
            bad.Add($"tài xế {g.Key} bị giữ bởi {g.Count()} offer");
        foreach (var b in Bookings.Values.Where(b => b.State == BookingState.Assigned))
            if (Offers.Count(o => o.BookingId == b.Id && o.Status == OfferStatus.Accepted) != 1)
                bad.Add($"booking {b.Id} Assigned nhưng không có đúng một offer Accepted");
        foreach (var d in Drivers.Values.Where(d => d.State != DriverState.Idle))
            if (!Offers.Any(o => o.DriverId == d.Id && o.Status is OfferStatus.Open or OfferStatus.Accepted))
                bad.Add($"tài xế {d.Id} bận ({d.State}) nhưng không có offer");
        return bad;
    }
}

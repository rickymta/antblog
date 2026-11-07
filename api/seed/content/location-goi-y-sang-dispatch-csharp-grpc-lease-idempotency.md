---
slug: location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency
title: "Từ gợi ý tới assignment: Rust đề xuất, C# quyết định qua gRPC SuggestMatch"
summary: "Bài 46: hợp đồng gRPC SuggestMatch giữa geo-engine Rust và Dispatch C#: gợi ý có hạn, lease, offer có TTL, accept idempotent, gợi ý cũ, phản hồi muộn. Contract test 23 PASS với stub .NET và những gì nó không chứng minh."
category: system-design
level: advanced
tags: [toi-noi, location, rust, csharp, grpc, dispatch, idempotency, contract-test, matching]
language: vi
---

Lab của bài này dựng một lô ba booking trên đồ thị fixture, gọi `SuggestMatch`, và nhận lại ba gợi ý: `BK-001` ← `DR-001` (ETA 54 giây), `BK-002` ← `DR-002` (17 giây), `BK-003` ← `DR-003` (20 giây). Đó là cặp tối ưu theo ETA đường bộ tại thời điểm Rust tính. Nhưng giữa lúc Rust tính xong và lúc Dispatch đọc xong phản hồi, `DR-001` có thể vừa nhận một offer khác, vừa bị hủy ca, hoặc vừa di chuyển tới chỗ khác. Gợi ý lúc đó vẫn đúng với bức ảnh Rust được xem, và sai với thế giới hiện tại.

Câu hỏi của bài: **ai có quyền nói «tài xế này thuộc chuyến này»?** Các bài 41 đến 45 đã tìm cặp tốt; bài này vẽ ranh giới giữa «tìm cặp tốt» và «gán». Câu trả lời của cả loạt là một dòng: Rust đề xuất, C# quyết định. Phần còn lại là cơ chế để dòng đó đúng khi có gợi ý cũ, tài xế từ chối, câu trả lời đến muộn và lời gọi bị lặp.

Đây là bài 46 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng ma trận ETA của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch), bộ giải ghép theo lô của [bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust) và dịch vụ gRPC có ngân sách thời gian của [bài 29](/articles/location-routing-service-tonic-cache-fallback-p99). Mọi số đo trong bài là **trên dữ liệu mô phỏng** hoặc trên đồ thị tổng hợp của lab; không có tài xế hay khách thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Giả sử <code>geo-engine</code> không chỉ đề xuất mà còn giữ luôn danh sách «tài xế đã được gán» để tự loại họ khỏi lô sau. Hai lô gần nhau cùng chọn <code>DR-001</code>, và tài xế vừa bấm từ chối một offer. Rust và Dispatch bây giờ có mấy nguồn sự thật về <code>DR-001</code>, và chúng có thể lệch nhau ở đâu?</p>
<details><summary>Xem đáp án</summary><p>Có hai nguồn: danh sách của Rust và bản ghi của Dispatch. Chúng lệch ngay khi tài xế từ chối (Dispatch biết, Rust chưa), khi offer hết hạn (Dispatch nhả lease, Rust vẫn coi là bận), hoặc khi Rust khởi động lại. Mỗi lần lệch cần một cơ chế đối soát. Thiết kế của bài này tránh cả lớp lỗi đó: Rust không lưu gì về assignment, mọi gợi ý mang theo <code>driver_state_version</code> mà Dispatch đã gửi, và Dispatch tự kiểm lại bằng trạng thái của chính nó trước khi lease.</p></details>
</div>

## Ranh giới: đề xuất không phải quyết định

Kiến trúc tổng thể của Tới Nơi (mục 4.5, Matching và Dispatch) đặt cho Dispatch những bất biến cứng: một booking chỉ có một assignment hợp lệ tại một thời điểm; tránh hai tài xế cùng nhận một đơn bằng lease hoặc optimistic concurrency; offer có TTL và `offer_id`; accept và reject phải idempotent; chống gửi offer lặp và xử lý được phản hồi muộn; Redis có thể hỗ trợ tìm ứng viên hay lease, nhưng trạng thái assignment chính thức phải được persist vào kho của Dispatch theo transaction. Đó là việc của backend C# ([bài ADR](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service) đã giải thích vì sao loạt này không viết lại phần đó).

`geo-engine` có thứ Dispatch không có: đồ thị đường, ma trận ETA và bộ giải phân công. Nó **không** có thứ quan trọng nhất của việc gán: trạng thái đúng của tài xế ngay lúc này. Vì vậy `SuggestMatch` là một hàm của ảnh chụp: Dispatch gửi booking đang chờ cùng tài xế rảnh kèm `state_version`, Rust trả cặp đề xuất kèm hạn dùng. Không có RPC nào trong hợp đồng giữ chỗ tài xế hoặc ghi assignment.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 480" role="img" aria-label="Hình 1. Một vòng từ gợi ý tới assignment. Rust chỉ xuất hiện ở bước 1 và 2 và không ghi trạng thái nào. Bước 3 là chỗ Dispatch kiểm lại gợi ý bằng trạng thái của chính nó; lease, offer và assignment đều nằm phía C#."><defs><marker id="loc46-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="10" y="10" width="860" height="460" rx="10" class="f-bg s-line" stroke-width="1.5"/><rect x="40" y="28" width="220" height="52" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="150" y="50" class="t-b" text-anchor="middle">Dispatch (C#)</text><text x="150" y="68" class="t-small" text-anchor="middle">nguồn sự thật của assignment</text><path d="M150 80 L150 455" class="f-none s-line dash" stroke-width="1.5"/><rect x="330" y="28" width="220" height="52" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="440" y="50" class="t-b" text-anchor="middle">geo-engine (Rust)</text><text x="440" y="68" class="t-small" text-anchor="middle">đề xuất, không ghi trạng thái</text><path d="M440 80 L440 455" class="f-none s-line dash" stroke-width="1.5"/><rect x="620" y="28" width="220" height="52" rx="6" class="f-surface s-muted" stroke-width="1.5"/><text x="730" y="50" class="t-b" text-anchor="middle">App tài xế</text><text x="730" y="68" class="t-small" text-anchor="middle">trả lời offer</text><path d="M730 80 L730 455" class="f-none s-line dash" stroke-width="1.5"/><text x="165" y="112">1. SuggestMatch: booking chờ + tài xế rảnh</text><path d="M150 120 L438 120" class="f-none s-muted" stroke-width="2" marker-end="url(#loc46-f1-a)"/><text x="165" y="138" class="t-small">kèm request_id, snapshot_version, state_version</text><text x="455" y="172">2. Suggestion: cặp + valid_until = now + 10 s</text><path d="M440 180 L152 180" class="f-none s-muted" stroke-width="2" marker-end="url(#loc46-f1-a)"/><text x="455" y="198" class="t-small">không giữ chỗ tài xế, không ghi gì</text><rect x="30" y="220" width="250" height="112" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="42" y="242" class="t-b">3. C# kiểm lại gợi ý</text><text x="42" y="262" class="t-small">còn hạn? booking còn Pending?</text><text x="42" y="278" class="t-small">tài xế còn Idle, state_version khớp?</text><text x="42" y="298" class="t-small">đủ: lease tài xế + tạo offer</text><text x="42" y="314" class="t-small">OF-0001, hạn trả lời 15 s</text><text x="165" y="360">4. offer OF-0001 tới tài xế</text><path d="M150 368 L728 368" class="f-none s-muted" stroke-width="2" marker-end="url(#loc46-f1-a)"/><text x="470" y="402">5. accept + khoá idempotency</text><path d="M730 410 L152 410" class="f-none s-muted" stroke-width="2" marker-end="url(#loc46-f1-a)"/><rect x="30" y="420" width="250" height="36" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="42" y="443" class="t-b">6. booking Assigned, C# persist</text></svg>
</div>
<figcaption>Hình 1. Một vòng từ gợi ý tới assignment. Rust chỉ xuất hiện ở bước 1 và 2 và không ghi trạng thái nào. Bước 3 là chỗ Dispatch kiểm lại gợi ý bằng trạng thái của chính nó; lease, offer và assignment đều nằm phía C#.</figcaption>
</figure>

Bên được lợi từ ranh giới này là cả hai phía của chuyến: khách không bị gán trùng, tài xế không nhận hai offer chồng nhau. Cái giá là một vòng gọi thêm, và mỗi gợi ý cũ bị từ chối là một lần tính bỏ đi. Lab chưa đo tần suất gợi ý cũ trong dòng tải thật, nên bài không có con số cho cái giá này.

## Hợp đồng SuggestMatch

Hợp đồng nằm ở `proto/location/v1/matching.proto`, dùng chung quy ước của `routing.proto` (tọa độ E7, thời gian mili giây) và chỉ thêm field mới. Hai thông điệp mang ý nghĩa nặng nhất:

`examples/toi-noi-location/proto/location/v1/matching.proto`

```protobuf
// Tài xế ứng viên. `state_version` tăng đơn điệu mỗi khi Dispatch đổi trạng thái tài xế (nhận offer, rảnh lại, di chuyển đáng kể…);
// gợi ý ghi lại phiên bản đã dùng để C# phát hiện gợi ý cũ.
message DriverCandidate {
  string driver_id = 1;
  LatLngE7 position = 2;
  uint64 state_version = 3;
  int64 idle_since_ms = 4;
  // Có giá trị = tài xế đang chở khách, còn chừng này ms nữa rảnh (ước lượng); `position` khi đó là nơi dự kiến trả khách (chain, LOC-47).
  optional uint32 busy_remaining_ms = 5;
}
```

`examples/toi-noi-location/proto/location/v1/matching.proto`

```protobuf
message Suggestion {
  // Tất định: băm của (request_id, booking_id, driver_id, driver_state_version).
  string suggestion_id = 1;
  string booking_id = 2;
  string driver_id = 3;
  // ETA tài xế tới điểm đón (gồm `busy_remaining_ms` nếu là chain), ms.
  uint32 eta_ms = 4;
  // Khoảng chim bay tài xế → điểm đón, mét (gợi ý rẻ, không phải quãng đường lái).
  uint32 distance_hint_m = 5;
  // Chi phí theo chính sách (ms tương đương), thấp hơn là tốt hơn.
  int64 cost = 6;
  // Phiên bản trạng thái tài xế mà gợi ý này dựa trên.
  uint64 driver_state_version = 7;
  // Hạn áp dụng, theo đồng hồ của người gọi.
  int64 valid_until_ms = 8;
  string policy_version = 9;
}
```

Mỗi field ở đó có một lý do:

- **`state_version`** tăng đơn điệu mỗi khi Dispatch đổi trạng thái tài xế (nhận offer, rảnh lại, di chuyển đáng kể). Gợi ý ghi lại phiên bản đã dùng. Chỉ cần so một số nguyên là Dispatch biết bức ảnh Rust xem đã cũ.
- **`valid_until_ms`** là hạn để *áp dụng* gợi ý, tính từ `now_ms` trong yêu cầu, tức đồng hồ của người gọi. Rust không đọc đồng hồ của chính nó để tính hạn, nên danh sách gợi ý là hàm thuần của yêu cầu và tất định; chỉ `SolverStats` đo thời gian nên khác nhau giữa các lần gọi.
- **`busy_remaining_ms`** cho phép đưa tài xế đang chở khách vào lô; bài 47 dùng field này, và contract test kiểm rằng ETA cộng đúng thời gian còn lại.
- **`policy_version`** gắn chính sách chi phí (bài 45) vào từng gợi ý, để một cặp được chọn có thể giải thích lại sau này. `MatchPolicy` có giới hạn đón xa, phạt mềm, thưởng chờ và thưởng rảnh, nhưng **không có** số hạng rủi ro hủy: `to_policy` đặt `risk_ms = 0`.

`request_id` tham gia vào mã gợi ý, là băm tất định của bốn thứ. Rust không lưu lại lời gọi nào; idempotent ở đây nghĩa là cùng đầu vào cho cùng đầu ra:

`examples/toi-noi-location/rust/crates/matcher/src/suggest.rs`

```rust
/// Mã gợi ý tất định: cùng (request_id, booking, tài xế, phiên bản trạng thái tài xế) cho cùng mã.
pub fn suggestion_id(request_id: &str, booking: &str, driver: &str, state_version: u64) -> String {
    format!(
        "sg-{:016x}",
        fnv1a(&format!("{request_id}|{booking}|{driver}|{state_version}"))
    )
}
```

Cùng `request_id`, cùng nội dung thì cùng `suggestion_id`; đổi `request_id` thì cùng cặp nhưng khác mã. Dispatch gọi lại sau một lỗi mạng, gửi đúng nội dung cũ kể cả `now_ms`, sẽ nhận đúng gợi ý cũ. Gửi cùng `request_id` với nội dung khác thì Rust không phát hiện: máy chủ không nhớ gì, nên việc giữ nội dung ổn định thuộc về Dispatch.

## Phía Rust: ngân sách thời gian và trả ESTIMATED thay vì lỗi

Dịch vụ tonic đi theo khuôn của `RoutingSvc` ở bài 29: kiểm đầu vào (sai thì `InvalidArgument`), tính ngân sách từ deadline gRPC, rồi chạy phần nặng trong `spawn_blocking`. Điểm riêng của `SuggestMatch` là **hết ngân sách không phải lỗi**:

`examples/toi-noi-location/rust/crates/geo-engine/src/matching.rs`

```rust
        let client = request
            .metadata()
            .get("grpc-timeout")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_grpc_timeout);
        let total = client.unwrap_or(Duration::from_millis(self.cfg.default_deadline_ms));
        let mut budget = total.saturating_sub(Duration::from_millis(self.cfg.reserve_ms));
        let req = request.into_inner();
        let explicit = req
            .compute_budget_ms
            .map(|ms| Duration::from_millis(u64::from(ms)));
        if let Some(b) = explicit {
            budget = budget.min(b);
        }
        let parsed = convert(&req).inspect_err(|_| {
            self.metrics.invalid.fetch_add(1, Ordering::Relaxed);
        })?;

        let estimated = |why: &str, c: &AtomicU64| {
            c.fetch_add(1, Ordering::Relaxed);
            Ok(Response::new(to_pb(suggest::suggest_estimated(
                &parsed, why,
            ))))
        };
        if budget.is_zero() {
            return if explicit.is_some_and(|b| b.is_zero()) {
                estimated("budget", &self.metrics.estimated_budget)
            } else {
                estimated("deadline", &self.metrics.estimated_deadline)
            };
        }
```

Khi hết ngân sách (hoặc quá tải), dịch vụ dựng lại cùng bài toán Hungarian trên ETA chim bay và trả `quality = ESTIMATED`. Đó vẫn là một phép gán hợp lệ, chỉ kém tin cậy hơn; Dispatch tự quyết có dùng hay không. Với một hệ thống đặt xe, trả lời gần đúng trong hạn thường tốt hơn trả lỗi đúng hạn, nhưng quyết định đó thuộc về bên chịu hậu quả, tức C#.

## Phía C#: kiểm lại, lease, rồi mới tạo offer

Stub của Dispatch nằm ở `dotnet/DispatchStub`. Hàm quan trọng nhất là chỗ gợi ý của Rust gặp trạng thái của C#:

`examples/toi-noi-location/dotnet/DispatchStub/InMemoryDispatch.cs`

```csharp
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
```

Năm lý do từ chối, theo thứ tự kiểm: gợi ý quá `valid_until` (`StaleSuggestionExpired`), booking hoặc tài xế không tồn tại, booking không còn `Pending`, tài xế không còn `Idle`, và `StateVersion` đã đổi (`StaleDriverVersion`). Chỉ khi qua cả năm cửa, C# mới **lease** tài xế (chuyển sang `Leased`, tăng `StateVersion`) và tạo offer với `offer_id` riêng.

Có **hai TTL độc lập** và đây là chỗ dễ nhầm. TTL của gợi ý (10 giây) là hạn để tạo offer. TTL của offer (15 giây) là hạn tài xế trả lời. Offer tạo muộn nhất ở giây thứ 10 thì còn sống tới giây thứ 25 kể từ lúc Rust nhận `now_ms`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 2. Hai TTL độc lập. Gợi ý hết hạn ở 10 s thì không tạo được offer; offer đã tạo chạy theo TTL riêng 15 s. Hai ca B4a và B4b của contract test kiểm đúng hai mốc này."><rect x="10" y="10" width="860" height="330" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="40" class="t-b">Hai TTL độc lập, tính từ đồng hồ của Dispatch (giây kể từ lúc Rust nhận now_ms)</text><path d="M360.0 60 L360.0 250" class="f-none s-line dash" stroke-width="1.5"/><path d="M500.0 60 L500.0 250" class="f-none s-line dash" stroke-width="1.5"/><path d="M780.0 60 L780.0 250" class="f-none s-line dash" stroke-width="1.5"/><rect x="80.0" y="70" width="280.0" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="90.0" y="90" class="t-small">gợi ý: áp dụng được tới valid_until (10 s)</text><rect x="80.0" y="120" width="420.0" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="90.0" y="140" class="t-small">offer 1: tạo lúc 0 s, hạn trả lời 15 s</text><rect x="360.0" y="170" width="420.0" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="370.0" y="190" class="t-small">offer 2: tạo lúc 10 s (muộn nhất), hết hạn 25 s</text><path d="M80 250 L780.0 250" class="f-none s-muted" stroke-width="1.5"/><path d="M80.0 250 L80.0 256" class="f-none s-muted" stroke-width="1.5"/><text x="80.0" y="272" class="t-small" text-anchor="middle">0 s</text><path d="M220.0 250 L220.0 256" class="f-none s-muted" stroke-width="1.5"/><text x="220.0" y="272" class="t-small" text-anchor="middle">5 s</text><path d="M360.0 250 L360.0 256" class="f-none s-muted" stroke-width="1.5"/><text x="360.0" y="272" class="t-small" text-anchor="middle">10 s</text><path d="M500.0 250 L500.0 256" class="f-none s-muted" stroke-width="1.5"/><text x="500.0" y="272" class="t-small" text-anchor="middle">15 s</text><path d="M640.0 250 L640.0 256" class="f-none s-muted" stroke-width="1.5"/><text x="640.0" y="272" class="t-small" text-anchor="middle">20 s</text><path d="M780.0 250 L780.0 256" class="f-none s-muted" stroke-width="1.5"/><text x="780.0" y="272" class="t-small" text-anchor="middle">25 s</text><text x="30" y="302" class="t-small">B4a: áp dụng gợi ý ở 10.001 ms → StaleSuggestionExpired, không tạo offer</text><text x="30" y="322" class="t-small">B4b: accept ở 15.001 ms sau khi tạo offer → LateResponse, tài xế nhả về Idle</text></svg>
</div>
<figcaption>Hình 2. Hai TTL độc lập. Gợi ý hết hạn ở 10 s thì không tạo được offer; offer đã tạo chạy theo TTL riêng 15 s. Hai ca B4a và B4b của contract test kiểm đúng hai mốc này.</figcaption>
</figure>

Tài xế từ chối thì sao? Stub nhả lease, tăng `StateVersion`, nhớ `RejectedBy` trên booking, và lần gợi ý lại **loại tài xế đó khỏi danh sách ứng viên** trước khi gửi cho Rust. Rust không biết lý do và không cần biết: ở ca B2 của lab, `DR-001` từ chối rồi gợi ý lại chọn `DR-003`, với `offer_id` mới.

## Accept idempotent và phản hồi muộn

Mạng di động làm tài xế bấm nhận hai lần, hoặc ứng dụng gửi lại khi chưa thấy phản hồi. Stub xử lý bằng khóa idempotency gắn với thao tác và offer:

`examples/toi-noi-location/dotnet/DispatchStub/InMemoryDispatch.cs`

```csharp
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
```

Cùng khóa thì trả đúng kết quả lần đầu, không tác dụng phụ; khác khóa mà offer đã quyết thì `AlreadyDecided`. Nếu accept đến sau hạn offer, kết quả là `LateResponse`: offer chuyển `Expired`, tài xế nhả về `Idle`, booking về `Pending` và không có assignment nào. Backend thật thay `Dictionary` bằng unique constraint trong cơ sở dữ liệu, như loạt C# đã làm ở bài về [transaction, concurrency và idempotency](/articles/dotnet10-transaction-concurrency-idempotency) và [outbox, inbox](/articles/dotnet10-outbox-inbox-idempotent-consumer).

## Contract test: 23 kiểm tra, và những điều chúng không chứng minh

Client .NET thật (`Grpc.Net.Client`) gọi máy chủ Rust thật qua gRPC trên `127.0.0.1`; lab Rust dựng máy chủ rồi chạy `dotnet run` và đọc mã thoát. Kết quả: **23 PASS, 0 FAIL**. Đây là phần chúng bao phủ:

| Nhóm | Số kiểm tra | Chạm vào gì |
|---|---:|---|
| A1–A9 | 9 | Hợp đồng thông điệp với máy chủ Rust thật: chất lượng `ROUTED`, mỗi booking và tài xế tối đa một lần, TTL, tính tất định theo `request_id`, `InvalidArgument`, rơi về `ESTIMATED` khi `compute_budget_ms = 0`, giới hạn đón xa, ETA của tài xế đang chở khách |
| B1–B6 | 14 | `InMemoryDispatch` điều khiển bằng gợi ý thật của Rust: đường vui, từ chối rồi gợi ý lại, gợi ý cũ, hai TTL, accept idempotent, hai gợi ý cùng chọn một tài xế |

Và đây là điều **23/23 không nói**. Một stub không phải Dispatch thật, và con số 23 dễ đọc quá mức:

1. **Stub và test do cùng một tác giả viết.** Chúng chứng minh hợp đồng nhất quán và cài đặt được, không chứng minh Dispatch thật của Tới Nơi đúng.
2. **Không có tranh chấp đa luồng.** Ca B6 («hai gợi ý cùng chọn `DR-001`, chỉ một lease») chạy tuần tự: gợi ý thứ hai bị từ chối vì tài xế đã `Leased`, không phải vì hai luồng đua nhau. Cuộc đua thật cần atomic claim hoặc optimistic concurrency của cơ sở dữ liệu, và stub không có thứ đó.
3. **Không persist, không Redis, không transaction.** Sập tiến trình giữa lúc lease và lúc ghi assignment chưa được thử.
4. **Đồng hồ là đồng hồ giả.** Các ca hết hạn tăng `FakeClock` thêm 10.001 ms hoặc 15.001 ms; không có thời gian chờ thật, cũng không có lệch đồng hồ giữa hai máy.
5. **Nhánh `ESTIMATED` được kích bằng `compute_budget_ms = 0`**, chưa phải bằng deadline gRPC thật hết hạn.
6. **Không có TLS, xác thực gRPC, hay kiểm tương thích phiên bản proto** giữa hai bên.
7. **Không có ca áp dụng lại đúng một gợi ý hai lần liền** khi offer còn mở. Đọc code thì kết quả sẽ là `BookingNotPending`, nhưng đó là suy luận, chưa phải kiểm tra đã chạy (xem bài tập 2).
8. **Chỉ một nút chính sách được thử:** `hard_pickup_s` (A8). `soft_*`, `age_bonus`, `idle_bonus` không có ca nào, và trong các lý do chưa ghép chỉ `BEYOND_PICKUP_LIMIT` được kiểm; `NO_ROUTE`, `NOT_SNAPPABLE` thì chưa.

Nói gọn: bộ test bắt được khi một field đã có ca kiểm đổi nghĩa, chẳng hạn bỏ echo `snapshot_version` hay ETA không còn cộng `busy_remaining_ms`. Việc Rust không giữ trạng thái chỉ được kiểm gián tiếp: ở B6 hai lời gọi liên tiếp vẫn chọn cùng `DR-001`. Nếu một field chưa có ca kiểm đổi nghĩa, hoặc Dispatch thật có lỗi đua luồng, bộ test này sẽ im lặng.

## Độ trễ của SuggestMatch trên đồ thị HCM

Phần đo độ trễ chạy trên đồ thị HCM thật (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), bể 120 tài xế rảnh, 227 ghim khách từ kịch bản `hcm-baseline` (seed 20261106), 100 lời gọi mỗi cỡ, client tonic cùng tiến trình. **Đo khi máy đang chịu tải nặng** (load average khoảng 398 trên 10 lõi), nên chỉ đọc tương đối.

| Booking mỗi lô | p50 (ms) | p95 (ms) | p99 (ms) | Ma trận (µs) | Giải (µs) |
|---:|---:|---:|---:|---:|---:|
| 10 | 30,35 | 125,68 | 220,26 | 3.614 | 9 |
| 20 | 12,40 | 93,47 | 234,81 | 3.625 | 16 |
| 40 | 19,51 | 99,26 | 118,40 | 4.503 | 32 |

Cột ma trận và giải là số máy chủ tự báo trong `SolverStats` (trung vị). Hai điều đáng đọc. Phần máy chủ tự báo (ma trận CH vài mili giây, giải Hungarian dưới 40 µs) chỉ là một phần của p50 đo ngoài; lab không tách phần còn lại, nhiều khả năng phần lớn là tải máy. Và p50 của cỡ 10 cao hơn cỡ 20 là dấu hiệu của nhiễu, không phải quy luật. Với tám client đồng thời, mỗi client 50 lời gọi 20 booking: p50 17,86 ms, p95 66,02 ms, p99 157,09 ms, không lời gọi nào rơi về `ESTIMATED` (deadline mặc định 250 ms).

## Chạy ví dụ và kiểm chứng

Cần .NET SDK (lab dùng 10.0.401) và, cho phần đo độ trễ, `data/hcm-graph.bin`. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
dotnet build examples/toi-noi-location/dotnet/DispatchStub -c Release     # lần đầu restore NuGet
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc46_goi_y_sang_dispatch
```

Tham số: `--no-hcm` bỏ phần đo độ trễ, `--no-dotnet` bỏ contract test, `--requests N` đổi số lời gọi mỗi cỡ. Output (rút gọn, nguyên văn từ `results/p05.md`):

```text
== A. hợp đồng SuggestMatch với máy chủ Rust ==
  gợi ý: BK-001←DR-001 eta 54s, BK-002←DR-002 eta 17s, BK-003←DR-003 eta 20s; stats hungarian 3×4, ma trận 4590 µs, giải 3 µs
  [PASS] A1 chất lượng ROUTED, đủ cặp, echo request_id/snapshot/policy — 3 gợi ý, source ch-bucket
  [PASS] A2 mỗi booking và mỗi tài xế tối đa một lần; suggestion_id duy nhất
  [PASS] A3 TTL: valid_until = now + 10 s; driver_state_version khớp bản đã gửi; eta > 0
  [PASS] A4 cùng request_id, cùng nội dung → đúng cùng gợi ý (idempotent, tất định)
  [PASS] A5 đổi request_id → cùng cặp nhưng suggestion_id khác
  [PASS] A6 yêu cầu sai (thiếu request_id / không tài xế / thiếu điểm đón / trùng driver_id) → InvalidArgument
  [PASS] A7 hết ngân sách tính toán (compute_budget_ms = 0) → ESTIMATED, không lỗi, vẫn là phép gán hợp lệ — source straight-line:budget, algo hungarian-estimated
  [PASS] A8 giới hạn cứng đón xa 1 s → không gợi ý nào, mọi booking unmatched BEYOND_PICKUP_LIMIT, policy_version được echo
  [PASS] A9 tài xế đang chở khách (busy_remaining_ms = 60 s) → ETA cộng đúng 60 s — 54182 ms → 114182 ms

== B. Dispatch stub (C# quyết định) dùng gợi ý thật của Rust ==
  TTL gợi ý 10 s (hạn để ÁP DỤNG gợi ý); TTL offer 15 s (hạn tài xế trả lời). Hai TTL độc lập: gợi ý hết hạn thì không tạo được offer, offer đã tạo chỉ theo TTL của offer.
  [PASS] B1a áp dụng 3 gợi ý → 3 offer, mỗi offer có offer_id riêng, hết hạn sau 15 s, tài xế bị lease
  [PASS] B1b accept cả ba → 3 booking Assigned, mỗi booking một tài xế, bất biến §4.5 đúng
  [PASS] B2 từ chối → offer cũ Rejected, tài xế cũ rảnh lại (state_version tăng), gợi ý lại chọn tài xế khác, offer mới có offer_id mới — DR-001 từ chối → DR-003
  [PASS] B3 gợi ý cũ (tài xế đã đổi state_version) bị từ chối, không lease, booking vẫn chờ; các gợi ý còn lại vẫn áp dụng được
  [PASS] B3b gợi ý lại với phiên bản mới của tài xế thì áp dụng được
  [PASS] B4a áp dụng gợi ý sau valid_until (10 s) → từ chối StaleSuggestionExpired, không offer
  [PASS] B4b accept muộn (sau TTL offer 15 s) → LateResponse; offer Expired; tài xế nhả về Idle; booking về Pending; không assignment
  [PASS] B4c gửi lại cùng khoá idempotency → đúng kết quả cũ (LateResponse), không tác dụng phụ
  [PASS] B4d sau khi nhả, booking được gợi ý và áp dụng lại bình thường
  [PASS] B4e quét hết hạn nền (ExpireDue) cũng nhả lease
  [PASS] B5 accept hai lần cùng khoá → cùng kết quả, một assignment; khoá khác → AlreadyDecided; reject sau accept → AlreadyDecided
  [PASS] B5b offer không tồn tại → UnknownOffer, lặp lại cùng khoá cho cùng kết quả
  [PASS] B6 hai gợi ý (hai lời gọi Rust khác nhau) cùng chọn tài xế DR-001 → chỉ một lease, gợi ý thứ hai bị từ chối, booking thứ hai vẫn chờ
  [PASS] B6b gợi ý cho booking đã Assigned → BookingNotPending

KẾT QUẢ: 23 PASS, 0 FAIL
```

```text
-- 2. độ trễ SuggestMatch trên đồ thị HCM: bể 120 tài xế rảnh, 10 / 20 / 40 booking, client tonic cùng tiến trình --
  đồ thị: examples/toi-noi-location/data/hcm-graph.bin: 126.699 nút, 319.074 cạnh; kịch bản hcm-baseline (seed 20261106)
  bể: 120 tài xế, 227 ghim khách có sẵn từ simulator
    booking      n   p50 (ms)   p95 (ms)   p99 (ms)  ma trận µs*     giải µs* gợi ý/lời gọi
         10    100      30.35     125.68     220.26         3614            9       10.0
         20    100      12.40      93.47     234.81         3625           16       20.0
         40    100      19.51      99.26     118.40         4503           32       40.0
  8 client đồng thời × 50 lời gọi, 20 booking: p50 17.86 ms, p95 66.02 ms, p99 157.09 ms, tổng 1.50 s, 0 lời gọi rơi về ESTIMATED (deadline mặc định 250 ms)
  (* ma trận µs, giải µs: do máy chủ báo trong SolverStats, trung vị. Đo khi máy đang chịu tải nặng, chỉ so sánh tương đối.)
```

`bash examples/toi-noi-location/verify-p05.sh` chạy lại lab này với `--fixture` (gồm contract test C# nếu máy có `dotnet`) cùng các test Rust của `matcher` và `geo-engine`.

## Giới hạn của ví dụ

- **Dispatch ở đây là stub trong bộ nhớ**: một tiến trình, không lưu bền, không Redis, không kiểm tranh chấp đồng thời, không TLS hay xác thực. Tám điểm ở mục trên nói chi tiết những gì 23 kiểm tra không bao phủ.
- **Chưa đo tần suất gợi ý cũ.** Không biết bao nhiêu phần trăm gợi ý bị từ chối vì `StaleDriverVersion` trong dòng tải thật; số đó phụ thuộc tốc độ đổi trạng thái tài xế của Dispatch thật.
- **Số thời gian** đo khi máy dùng chung quá tải. Chúng cho thấy phần tính ở Rust cỡ vài mili giây trong lần đo này, không nói gì về p99 của production.
- **Chưa có ca tài xế từ chối hàng loạt** hay gợi ý lại nhiều vòng; ca B2 chỉ thử một lần từ chối.

## Bài tập

**Bài 1.** Gợi ý có `valid_until` = `now_ms` + 10 s, offer có TTL 15 s. Từ lúc Rust nhận `now_ms`, accept muộn nhất còn hợp lệ là mấy giây? Vì sao không nên đặt hai TTL bằng nhau?

<details><summary>Lời giải bài 1</summary>
<p>25 giây: offer được tạo muộn nhất khi đồng hồ chưa quá <code>valid_until</code> (10 s), rồi sống thêm 15 s. Hai TTL đo hai việc khác nhau. TTL gợi ý giới hạn độ cũ của bức ảnh Rust xem; TTL offer giới hạn thời gian tài xế phải quyết định. Gộp thành một thì hoặc offer quá ngắn để tài xế kịp đọc, hoặc gợi ý quá dài làm thế giới thật lệch khỏi bức ảnh.</p>
</details>

**Bài 2.** Đọc `ApplySuggestion` và đoán: áp dụng cùng một gợi ý hai lần liên tiếp khi offer đầu còn mở sẽ trả `DriverNotIdle` hay `BookingNotPending`? Viết ca kiểm tra B7 để xác nhận, rồi giải thích vì sao ca B6 trả kết quả khác.

<details><summary>Lời giải bài 2</summary>
<p>Trả <code>BookingNotPending</code>, vì sau lần áp dụng đầu booking đã sang <code>Offered</code> và điều kiện booking được kiểm trước điều kiện tài xế. Ở B6 hai gợi ý thuộc hai booking khác nhau; booking thứ hai vẫn <code>Pending</code>, nên kiểm tra tài xế mới là chỗ chặn và kết quả là <code>DriverNotIdle</code>. Kết luận này đọc từ mã, lab chưa chạy ca B7; người viết ca kiểm tra cần chạy thật để xác nhận.</p>
</details>

**Bài 3.** Vì sao `valid_until_ms` tính theo `now_ms` do người gọi gửi lên thay vì theo đồng hồ của máy chủ Rust? Điều gì sẽ hỏng nếu Rust tự đọc đồng hồ?

<details><summary>Lời giải bài 3</summary>
<p>Hạn dùng là thứ Dispatch kiểm, nên phải đo bằng đồng hồ Dispatch; nếu Rust dùng đồng hồ riêng, mọi lệch đồng hồ giữa hai máy trở thành lỗi ngầm làm gợi ý hết hạn sớm hoặc muộn. Ngoài ra, lấy thời gian từ yêu cầu giữ hàm gợi ý là hàm thuần: cùng đầu vào cho cùng đầu ra (ca A4), kiểm thử không cần đồng hồ giả bên Rust.</p>
</details>

Bài trước: [bài 45](/articles/location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy). Bài tiếp: [bài 47](/articles/location-chain-dispatch-tai-xe-sap-tra-khach).

## Tài liệu gốc

- [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/): cách deadline lan truyền giữa client và server, nền của ngân sách thời gian trong bài.
- [Protocol Buffers: hướng dẫn proto3](https://protobuf.dev/programming-guides/proto3/): field `optional`, enum và quy tắc thêm field mới mà không phá hợp đồng.
- [tonic trên docs.rs](https://docs.rs/tonic/latest/tonic/): thư viện gRPC phía Rust của máy chủ `MatchingService`.
- [Gọi dịch vụ gRPC bằng client .NET](https://learn.microsoft.com/en-us/aspnet/core/grpc/client): `Grpc.Net.Client`, phía client của contract test.

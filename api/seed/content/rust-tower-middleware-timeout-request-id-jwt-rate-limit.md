---
slug: rust-tower-middleware-timeout-request-id-jwt-rate-limit
title: "Middleware với tower: timeout, request id, JWT và rate limit"
summary: "Bài 33: dựng chồng middleware tower cho API sổ cái, chạy thật: thứ tự layer, request id vào span, timeout 504, JWT HS256, phân quyền finance/cs, giới hạn tải bằng tower::limit và các bẫy axum gặp phải."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, axum, tower, jwt]
language: vi
---

Payment BFF viết bằng C# gọi `POST /v1/entries` suốt ngày. Đội vận hành hỏi ba câu trước mọi câu hỏi nghiệp vụ: yêu cầu quá lâu thì bị cắt ra sao, một lần gọi chậm tìm lại trong log bằng gì, và ai được ghi bút toán còn ai chỉ xem số dư. Cả ba nằm ở middleware.

Đây là bài 33 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab nối tiếp [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du) (endpoint) và [bài 32](/articles/rust-validation-problem-details-openapi-utoipa) (Problem Details) nhưng tự chứa: sổ cái là `ledger-core` trong bộ nhớ, không cần PostgreSQL, handler viết lại gọn, thân lỗi cùng tiền tố `type` với bài 32 nhưng rút gọn, chưa có `instance` và `traceId`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Viết <code>.layer(A).layer(B)</code> trên <code>ServiceBuilder</code> và viết đúng như vậy trên <code>Router</code>. Hai cách có cho request chạm A trước không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Trên <code>ServiceBuilder</code>, A chạm trước. Trên <code>Router</code>, lớp gọi sau bọc ra ngoài nên B chạm trước. Lab in thứ tự thật ra màn hình thay vì nhờ người đọc nhớ.</p></details>
</div>

## Request đi vào lớp nào trước?

Trong tower, một `Layer` nhận một service và trả về service mới bọc nó. Chồng middleware là các lớp bọc lồng nhau, nên hỏi thứ tự là hỏi lớp nào ngoài cùng. Lab đặt ba lớp dò A, B, C, mỗi lớp ghi `in` trước khi gọi tiếp và `out` khi response quay về:

```rust
// src/order.rs (rút gọn)
async fn probe(State(p): State<Probe>, req: Request, next: Next) -> Response {
    p.note("in");
    let response = next.run(req).await;
    p.note("out");
    response
}

// ServiceBuilder: lớp viết trước là lớp ngoài cùng.
handler_router(log).layer(
    ServiceBuilder::new()
        .layer(from_fn_with_state(Probe::new("A", log), probe))
        .layer(from_fn_with_state(Probe::new("B", log), probe))
        .layer(from_fn_with_state(Probe::new("C", log), probe)),
)

// Router::layer nối tiếp: lớp gọi sau bọc ra ngoài.
handler_router(log)
    .layer(from_fn_with_state(Probe::new("A", log), probe))
    .layer(from_fn_with_state(Probe::new("B", log), probe))
    .layer(from_fn_with_state(Probe::new("C", log), probe))
```

```text
ServiceBuilder A, B, C : A:in B:in C:in handler C:out B:out A:out
Router.layer A, B, C   : C:in B:in A:in handler A:out B:out C:out
```

`Router::layer` bọc từng route hiện có, lớp gọi sau nằm ngoài lớp gọi trước, nên gọi ba lần cho thứ tự đảo. `ServiceBuilder` gom layer thành một khối đọc từ trên xuống; tài liệu axum cũng nói cách này dễ đọc hơn, và lab dùng nó cho chồng thật. Tài liệu `Router::layer` còn dặn: chỉ route đã có lúc gọi mới được bọc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 256" role="img" aria-label="Hình 1. Cùng ba lớp A, B, C: ServiceBuilder cho request đi qua A rồi B rồi C; ba lệnh Router.layer liên tiếp cho request đi qua C rồi B rồi A."><defs><marker id="rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="32" class="t-b">ServiceBuilder::new().layer(A).layer(B).layer(C)</text><rect x="20" y="46" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="36" y="76" class="t-b">A</text><rect x="220" y="46" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="236" y="76" class="t-b">B</text><rect x="420" y="46" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="436" y="76" class="t-b">C</text><rect x="640" y="46" width="220" height="50" rx="8" class="f-pk-soft s-pk"/><text x="656" y="76" class="t-b">handler</text><path d="M170 71 L218 71" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a)"/><path d="M370 71 L418 71" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a)"/><path d="M570 71 L638 71" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a)"/><text x="20" y="140" class="t-b">Router::new()...layer(A).layer(B).layer(C)</text><rect x="20" y="154" width="150" height="50" rx="8" class="f-warn-soft s-warn"/><text x="36" y="184" class="t-b">C</text><rect x="220" y="154" width="150" height="50" rx="8" class="f-warn-soft s-warn"/><text x="236" y="184" class="t-b">B</text><rect x="420" y="154" width="150" height="50" rx="8" class="f-warn-soft s-warn"/><text x="436" y="184" class="t-b">A</text><rect x="640" y="154" width="220" height="50" rx="8" class="f-pk-soft s-pk"/><text x="656" y="184" class="t-b">handler</text><path d="M170 179 L218 179" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a)"/><path d="M370 179 L418 179" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a)"/><path d="M570 179 L638 179" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-tower-middleware-timeout-request-id-jwt-rate-limit-0-a)"/><text x="20" y="240" class="t-small">Mũi tên là chiều đi của request; response quay lại theo đường ngược.</text></svg>
</div>
<figcaption>Hình 1. Cùng ba lớp, hai cách ghép, hai thứ tự.</figcaption>
</figure>

ASP.NET Core cùng quy tắc: thứ tự đăng ký là thứ tự request đi vào, response quay ngược ([middleware, DI, cấu hình và logging](/articles/dotnet10-middleware-di-options-logging)). Ở lab này, xác thực và quyền không nằm trong chồng mà ở tham số handler (xem mục extractor).

## Chồng middleware của service gồm những lớp nào?

Chồng thật nằm trong hàm `apply` ở `src/stack.rs`, đọc từ trên xuống là thứ tự request đi vào:

```rust
// src/stack.rs (rút gọn)
let stack = ServiceBuilder::new()
    .layer(SetSensitiveRequestHeadersLayer::new(sensitive))      // 1. che Authorization
    .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid))  // 2. gán id nếu thiếu
    .layer(TraceLayer::new_for_http()                            // 3. span có request_id
        .make_span_with(make_span)
        .on_response(DefaultOnResponse::new().level(Level::INFO)))
    .layer(PropagateRequestIdLayer::new(REQUEST_ID))             // 4. chép id sang response
    .option_layer(cors)                                          // 5. tuỳ chọn
    .layer(TimeoutLayer::with_status_code(TIMEOUT_STATUS, settings.timeout)) // 6.
    .layer(HandleErrorLayer::new(overloaded))                    // 7. Overloaded -> 503
    .layer(LoadShedLayer::new())                                 // 8. hết chỗ thì từ chối
    .layer(GlobalConcurrencyLimitLayer::new(settings.max_in_flight)); // 9. một semaphore
router.with_state(()).layer(stack)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 332" role="img" aria-label="Hình 2. Chín lớp middleware từ trên xuống rồi tới handler. Bốn lớp đầu quan sát, năm lớp sau bảo vệ service. Request đi từ trên xuống, response đi ngược lên."><rect x="20" y="14" width="560" height="26" rx="6" class="f-phys-soft s-phys"/><text x="32" y="32" class="t-small">1 SetSensitiveRequestHeaders: che Authorization khỏi log</text><rect x="20" y="44" width="560" height="26" rx="6" class="f-phys-soft s-phys"/><text x="32" y="62" class="t-small">2 SetRequestId: gán x-request-id nếu client chưa gửi</text><rect x="20" y="74" width="560" height="26" rx="6" class="f-phys-soft s-phys"/><text x="32" y="92" class="t-small">3 TraceLayer: mở span có request_id, log status và latency</text><rect x="20" y="104" width="560" height="26" rx="6" class="f-phys-soft s-phys"/><text x="32" y="122" class="t-small">4 PropagateRequestId: chép id sang mọi response, kể cả 504</text><rect x="20" y="134" width="560" height="26" rx="6" class="f-warn-soft s-warn"/><text x="32" y="152" class="t-small">5 CorsLayer (tuỳ chọn): chỉ khi trình duyệt gọi thẳng</text><rect x="20" y="164" width="560" height="26" rx="6" class="f-warn-soft s-warn"/><text x="32" y="182" class="t-small">6 TimeoutLayer: quá hạn thì 504, huỷ future của handler</text><rect x="20" y="194" width="560" height="26" rx="6" class="f-warn-soft s-warn"/><text x="32" y="212" class="t-small">7 HandleError: đổi lỗi Overloaded thành 503</text><rect x="20" y="224" width="560" height="26" rx="6" class="f-warn-soft s-warn"/><text x="32" y="242" class="t-small">8 LoadShed: hết chỗ thì từ chối ngay, không xếp hàng</text><rect x="20" y="254" width="560" height="26" rx="6" class="f-warn-soft s-warn"/><text x="32" y="272" class="t-small">9 GlobalConcurrencyLimit: một semaphore cho mọi route</text><rect x="20" y="288" width="560" height="30" rx="6" class="f-pk-soft s-pk"/><text x="32" y="308" class="t-small">Handler: extractor Authorized rồi Json, sau đó mới tới thân hàm</text><rect x="620" y="14" width="240" height="44" rx="8" class="f-phys-soft s-phys"/><text x="634" y="42" class="t-b">Lớp quan sát (1-4)</text><rect x="620" y="68" width="240" height="44" rx="8" class="f-warn-soft s-warn"/><text x="634" y="96" class="t-b">Lớp bảo vệ (5-9)</text><rect x="620" y="122" width="240" height="44" rx="8" class="f-pk-soft s-pk"/><text x="634" y="150" class="t-b">Xác thực và quyền</text><text x="620" y="204" class="t-small">Request đi từ trên xuống,</text><text x="620" y="224" class="t-small">response quay ngược từ dưới lên.</text></svg>
</div>
<figcaption>Hình 2. Chồng middleware thật của lab; xác thực và quyền là extractor của handler.</figcaption>
</figure>

Che header phải đứng trước mọi lớp ghi log. Request id phải được gán trước `TraceLayer` vì span đọc id từ request (tài liệu `tower-http` dặn đúng vậy). `PropagateRequestId` nằm trong `TraceLayer` và ngoài `TimeoutLayer` nên response quá hạn vẫn mang id và `TraceLayer` thấy nó; mục sau chứng minh.

CORS là cơ chế của trình duyệt; Payment BFF là server gọi server nên không có preflight. Lab không gắn `CorsLayer` mặc định. Khi một trang quản trị cần gọi thẳng, `Settings.cors_origin` bật nó cho đúng một origin:

```text
không gắn CORS, preflight   : 405 Method Not Allowed, allow-origin = None
gắn CORS, origin được phép  : 200 OK, allow-origin = Some("https://admin.toi-noi.example")
gắn CORS, origin lạ         : allow-origin = None
```

`allow_origin(HeaderValue)` cố định trả header cho cả origin lạ (trình duyệt tự chặn, nhưng header là thừa); `AllowOrigin::list` chỉ trả khi origin khớp, nên lab dùng nó. CORS không thay xác thực: nó không ngăn `curl` hay BFF.

## Request id, trace và timeout cho thấy gì khi chạy?

`MakeRequestUuid` sinh UUID khi request chưa có `x-request-id`; id client gửi lên được giữ nguyên, nên BFF C# gửi id của nó thì log hai phía khớp nhau. Span của `TraceLayer` đọc header đó:

```rust
// src/stack.rs (rút gọn)
let make_span = move |req: &Request<_>| {
    let id = req.headers().get(&REQUEST_ID).and_then(|v| v.to_str().ok()).unwrap_or("-");
    // Chỉ ghi path, không ghi query: query là nơi người ta hay lỡ tay đặt token.
    tracing::info_span!("http", method = %req.method(), path = %req.uri().path(), request_id = %id)
};
```

Lab bắt log vào bộ nhớ: dòng log của handler mang đúng id của response, nên tra một lần gọi chỉ cần lọc theo `request_id`:

```text
response x-request-id: 1818178f-13ab-48fc-bc8b-4c1cb838465b
log của handler      :  INFO http{method=POST path=/v1/entries request_id=1818178f-13ab-48fc-bc8b-4c1cb838465b}: ghi bút toán user=svc:test entry=JE-000001 replayed=false
```

Id `bff-req-42` do client gửi cũng được giữ trong response và vào span; response 401 cũng có `x-request-id`.

Với timeout, lab gắn `/slow` ngủ 600 ms vào chồng thật với hạn 150 ms:

```text
TimeoutLayer::with_status_code(GATEWAY_TIMEOUT, 150ms): 504 Gateway Timeout, body "", mất 152 ms
log của TraceLayer:  INFO http{method=GET path=/slow request_id=ba8061d9-...}: finished processing request latency=152 ms status=504
700 ms sau, handler có chạy tới cuối không: false
thân request gửi dở, hạn 150 ms: HTTP/1.1 504 Gateway Timeout
TimeoutLayer::new (deprecated), mặc định        : 408 Request Timeout
```

Response 504 có body rỗng, vẫn mang request id, và `TraceLayer` ghi đúng `status=504`. Handler không chạy tới cuối: future bị drop tại điểm `.await`, cùng cơ chế huỷ ở [bài 21](/articles/rust-async-await-tokio-task-timeout-cancellation). Thân request gửi dở cũng bị cắt, vì extractor đọc body nằm trong future của handler. `TimeoutLayer::new` (deprecated từ `tower-http` 0.6.7) mặc định trả 408; `with_status_code` bắt người viết chọn mã.

RFC 9110 định nghĩa 408 là server không nhận đủ request trong thời gian chờ; 504 dành cho gateway hoặc proxy không nhận kịp phản hồi từ upstream. Không mã nào khớp hoàn toàn với «handler của chính ta chậm». Lab chọn 504; 503 cũng hợp lý. Quan trọng là ghi vào OpenAPI của bài 32.

Timeout không nói handler đã commit hay chưa. Đoạn này là suy luận từ cơ chế huỷ, không phải kết quả đo: sổ cái của lab nằm trong bộ nhớ nên lab không dựng ca này. Future bị drop tại một `.await`; commit xong trước điểm đó thì bút toán vẫn nằm trong sổ cái dù BFF nhận 504, còn bị huỷ đúng lúc đang chờ `COMMIT` thì service cũng không biết kết quả. Vì thế BFF chỉ retry `POST` an toàn nhờ `idempotencyKey`: cùng key trả 200 nếu bút toán đã có, 201 nếu chưa, như [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) và [bài .NET về transaction và idempotency](/articles/dotnet10-transaction-concurrency-idempotency).

## JWT HS256 được kiểm những gì, và bẫy nào có thật?

Khoá lab sinh từ hai UUID v4 mỗi lần chạy, chỉ là giá trị lab, không phải secret; service thật lấy khoá từ kho secret. `jsonwebtoken` 11 buộc chọn đúng một backend mật mã bằng feature `aws_lc_rs` hoặc `rust_crypto`, nếu không phải tự cài `CryptoProvider`, không thì panic lúc chạy; lab dùng `rust_crypto` (thuần Rust). Phần kiểm tra nằm trong `AuthConfig`:

```rust
// src/auth.rs (rút gọn)
let mut validation = Validation::new(Algorithm::HS256);
validation.leeway = LEEWAY_SECS;                  // 5 giây
validation.set_audience(&[AUDIENCE]);             // "ledger-api"
validation.set_required_spec_claims(&["exp", "aud", "sub"]);
```

Bảng dưới là kết quả `GET /v1/accounts/driver:DR-001/balance` với từng loại token:

| Token | Kết quả |
|---|---|
| hợp lệ, vai trò `cs` | 200 |
| `exp` đã qua 120 giây | 401 |
| ký bằng khoá khác | 401 |
| `aud` là `payment-api` | 401 |
| thiếu `aud` | 401 |
| ký HS512 bằng cùng khoá | 401 |
| thiếu `roles` | 401 |
| chuỗi rác, không header, scheme `Basic` | 401 |

Hai bẫy `--verify` đã chặn. Một, `leeway` mặc định là 60 giây: token hết hạn 30 giây trước vẫn được chấp nhận, còn lab (5 giây) trả 401. Hai, `aud` chỉ bị kiểm khi token có `aud`, và `required_spec_claims` mặc định chỉ có `exp`:

```text
hết hạn 30 giây trước: leeway mặc định (60 s) chấp nhận ; lab (5 s) -> 401 Unauthorized
token không có aud: rule mặc định chấp nhận = true; có required_spec_claims -> Some("MissingRequiredClaim(\"aud\")")
```

`Claims` của lab còn khai `aud` không phải `Option`. `decode` của `jsonwebtoken` 11.1.0 deserialize claims trước khi kiểm, nên trong lab token thiếu `aud` bị serde chặn trước (`missing field`); `required_spec_claims` là lớp cho kiểu claims để `aud` tuỳ chọn.

Response 401 giống nhau cho mọi lý do, kèm `WWW-Authenticate: Bearer realm="ledger"` và không nêu nguyên nhân. Lý do chỉ vào log dưới dạng tên lỗi (`ExpiredSignature`, `InvalidSignature`, `InvalidAudience`, `InvalidAlgorithm`); `--verify` kiểm không token nào của bảng xuất hiện trong log.

Phần «không log token» có đối chứng: lab bật ghi cả header vào span, rồi bỏ lớp che để thấy token lộ thật.

```text
che Authorization : {"authorization": Sensitive, "x-request-id": "cd305151-..."}}: tra số dư user=svc:test account=driver:DR-001
bỏ lớp che        : token có trong log = true
```

Lớp che đánh dấu `Authorization` là nhạy cảm nên `Debug` của header chỉ in `Sensitive`. Nó chỉ che header: token trong query hay body vẫn bị ghi nếu bạn ghi chúng, nên span chỉ lấy `path`.

## Extractor AuthUser và phân quyền: 401, 403 hay 400 đến trước?

Extractor cài `FromRequestParts`: axum chạy nó trước thân handler, nó trả lỗi thì thân handler không chạy. `AuthUser` đọc `Authorization`, kiểm token, trả người gọi hoặc 401. Quyền là lớp thứ hai, khai báo ở kiểu:

```rust
// src/auth.rs (rút gọn)
pub trait Policy { const ALLOWED: &'static [Role]; }

pub struct Authorized<P: Policy> { pub user: AuthUser, policy: PhantomData<P> }

impl<S, P> FromRequestParts<S> for Authorized<P>
where Arc<AuthConfig>: FromRef<S>, S: Send + Sync, P: Policy {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let user = AuthUser::from_request_parts(parts, state).await?;   // 401
        user.require(P::ALLOWED)?;                                       // 403
        Ok(Authorized { user, policy: PhantomData })
    }
}

// src/api.rs: chữ ký handler chính là bảng quyền.
async fn post_entry(
    Authorized { user, .. }: Authorized<Writers>,   // chỉ finance
    State(state): State<AppState>,
    Json(body): Json<PostEntryBody>,
) -> Result<(StatusCode, Json<EntryCreated>), ApiError> { /* ... */ }
```

Bản đầu của tôi gọi `user.require(...)` trong thân handler. `--verify` bắt lỗi ngay: `cs` gửi body rác nhận 400 chứ không phải 403, vì mọi extractor, kể cả `Json`, chạy xong thân hàm mới được vào. Chuyển kiểm quyền vào extractor đặt trước `Json` sửa được thứ tự:

```text
body rác: không token 401 Unauthorized, cs 403 Forbidden, finance 400 Bad Request
finance ghi TN-0001        -> 201 Created {"entryId":"JE-000001","replayed":false}
finance ghi lại cùng key   -> 200 OK {"entryId":"JE-000001","replayed":true}
cs ghi bút toán            -> 403 Forbidden {"type":"https://toi-noi.example/problems/forbidden","title":"Không đủ quyền","status":403,...}
cs tra số dư DR-001        -> 200 OK {"account":"driver:DR-001","balanceDong":20400,"asOf":"..."}
```

Thứ tự là 401, rồi 403, rồi mới tới lỗi body. Vai trò lạ (`admin`) nhận 403. `GET /healthz` không khai báo extractor nên không cần token. Tham số handler đóng vai `RequireAuthorization` của ASP.NET Core: quyền hiện ngay trong chữ ký. Quên khai báo thì route vẫn mở, nên vẫn cần bảng test 401/403.

## tower::limit giới hạn được gì, và vì sao RateLimit cần Buffer?

`ConcurrencyLimit` giới hạn số request chạy đồng thời và mặc định **xếp hàng**: request thứ ba chờ chỗ trống, không bị từ chối. Muốn từ chối ngay phải thêm `LoadShed`. Lab bắn 6 request cùng lúc vào hai route có handler ngủ 100 ms, giới hạn 2:

```text
layer thẳng, chưa with_state : đỉnh đồng thời = 6 (giới hạn 2 không có tác dụng)
layer thẳng, axum::serve, 6 kết nối, 1 route : đỉnh đồng thời = 6
ConcurrencyLimitLayer(2), 2 route : đỉnh đồng thời = 4 (mỗi route một semaphore)
GlobalConcurrencyLimit(2), xếp hàng : 6/6 trả 200, đỉnh đồng thời = 2
GlobalConcurrencyLimit(2) + LoadShed : 2 trả 200, 4 trả 503, đỉnh đồng thời = 2
```

Ba dòng đầu là hai bẫy thật, không kèm lỗi biên dịch hay cảnh báo. Một: `Router` chưa gọi `with_state` giữ handler ở dạng chưa dựng, kèm cả layer. Dùng nó làm service thì `MethodRouter::call_with_state` của axum 0.8.9 gọi `handler.clone().into_route(state)`, nên `ConcurrencyLimitLayer` mang semaphore mới cho **mỗi request**. Đi qua `axum::serve(listener, router)` thì axum gọi `with_state(())` cho **mỗi kết nối**: 6 kết nối vào một route vẫn ra đỉnh 6. Router đã `with_state` (như `routes(state)` của lab) hoặc qua `into_make_service()` chỉ dựng một lần; `apply` gọi `with_state(())` để router nào vào cũng an toàn.

Hai: dù đã dựng một lần, `ConcurrencyLimitLayer` tạo semaphore riêng cho từng method của từng route, hai route cho tới 2 + 2 request. `GlobalConcurrencyLimitLayer` giữ một semaphore ngay trong layer nên mới là giới hạn của cả service.

`RateLimit` gặp trở ngại khác: nó giữ bộ hẹn giờ nên không `Clone`, trong khi `Router::layer` đòi `Clone`. Viết thẳng `router.layer(RateLimitLayer::new(5, Duration::from_secs(1)))` cho lỗi thật:

```text
error[E0277]: the trait bound `RateLimit<Route>: Clone` is not satisfied
note: required by a bound in `Router::<S>::layer`
```

Cách sửa chuẩn là `Buffer`: một worker tokio giữ service thật, phía ngoài giữ tay cầm `Clone` gửi request qua kênh. Buffer đổi kiểu lỗi thành `BoxError` nên cần `HandleErrorLayer`; `RateLimit` cũng chờ chứ không từ chối, nên `LoadShed` phải nằm **trong** Buffer:

```rust
// src/limits.rs (rút gọn)
let service = ServiceBuilder::new()
    .layer(HandleErrorLayer::new(to_503))     // Buffer và LoadShed trả BoxError
    .layer(BufferLayer::new(64))              // tay cầm Clone tới worker
    .layer(LoadShedLayer::new())              // hết hạn mức: từ chối, không chờ
    .layer(RateLimitLayer::new(num, per))
    .service(router.with_state(()));          // bọc cả Router thành MỘT service
Router::new().fallback_service(service)
```

Đoạn cuối tránh bẫy thứ ba, cùng gốc với `ConcurrencyLimitLayer`: gắn khối này qua `Router::layer` thì mỗi route có worker và bucket riêng. Hạn mức 3 request mỗi 600 ms, lab đo cả hai cách:

```text
Router::layer, 2 route : /a lần 4: 503 Service Unavailable; /b lần đầu: 200 OK
một service, 2 route   : bff-a /a 3 lần: [200, 200, 200]; bff-b /b: 503 Service Unavailable; sau 650 ms: 200 OK
```

Khi bọc thành một service, `bff-b` chưa gọi lần nào vẫn nhận 503 vì `tower::limit` không biết client là ai: hạn mức chung cho cả process. Nó là cầu chì bảo vệ process, không công bằng giữa các client. Rate limit theo client, dùng chung giữa các instance, là token bucket Redis ở [bài 28](/articles/rust-redis-cache-so-du-rate-limit-lock-ttl).

## Chạy ví dụ và kiểm chứng

Từ gốc repository, không cần PostgreSQL hay Redis:

```bash
cargo run --manifest-path examples/toi-noi-rust/33-tower-middleware/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/33-tower-middleware/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/33-tower-middleware/Cargo.toml
```

`--verify` chạy mười kiểm chứng, thoát mã 1 nếu có cái sai và in `VERIFY OK` khi xong. `cargo test` có mười test tương ứng, chạy song song được: socket bind `127.0.0.1:0` rồi đọc cổng thật. Log đi qua một subscriber toàn cục ghi vào bộ đệm theo thread, nên test dùng runtime một thread. Bản đầu dùng `set_default` cho từng test và đỏ oan 1 trong 72 lần chạy dưới tải: span rớt khỏi log. Tôi suy ra nguyên nhân là `tracing` cache `Interest` của callsite cho cả process; cài subscriber trước mọi callsite thì 480 lần không đỏ.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Khoá JWT sinh tại chỗ, không xoay vòng, không kiểm <code>iss</code>, và token chỉ an toàn qua TLS mà lab không có. Sổ cái trong bộ nhớ nên mất khi tắt process. Body 503 là chữ trần, body 504 rỗng: chưa phải Problem Details. <code>TimeoutLayer</code> chỉ đếm tới khi có response head: thân response stream chậm không bị cắt (tower-http có lớp riêng cho chặng body). Kiểm chứng timeout và rate limit dựa vào đồng hồ thật (150 ms, 600 ms); 480 lần chạy song song ở load trên 300 không đỏ, nhưng đó không phải bảo đảm, máy quá chậm có thể phải nới.</p>
</div>

**Bài tập:** BFF gọi `POST /v1/entries` và nhận 504 sau hai giây. Nó nên làm gì, và service cần thêm gì để BFF phân biệt «xử lý chậm» với «đang quá tải»?

<details><summary>Xem lời giải</summary><p>BFF gọi lại với cùng <code>idempotencyKey</code>: 200 và <code>replayed: true</code> nghĩa là lần trước đã commit, 201 nghĩa là chưa; không đổi key. Service đã tách hai trường hợp bằng mã: 504 là quá hạn xử lý, 503 là bị <code>LoadShed</code> từ chối trước khi chạy. Nên thêm <code>Retry-After</code> cho 503 và ghi cả hai mã vào hợp đồng OpenAPI để BFF đặt chính sách thử lại riêng cho từng mã.</p></details>

Bài trước: [bài 32](/articles/rust-validation-problem-details-openapi-utoipa). Bài tiếp: [bài 34](/articles/rust-axum-api-test-oneshot-hop-dong-bff).

## Tài liệu gốc

- [docs.rs: crate axum 0.8](https://docs.rs/axum/0.8.9/axum/).
- [docs.rs: crate tower 0.5](https://docs.rs/tower/0.5.3/tower/).
- [docs.rs: crate tower-http 0.7](https://docs.rs/tower-http/0.7.1/tower_http/).
- [docs.rs: crate jsonwebtoken 11](https://docs.rs/jsonwebtoken/11.1.0/jsonwebtoken/).
- [RFC 9110: HTTP Semantics, mã 408 và 504](https://www.rfc-editor.org/rfc/rfc9110.html#name-408-request-timeout).
- [RFC 7519: JSON Web Token](https://www.rfc-editor.org/rfc/rfc7519.html).

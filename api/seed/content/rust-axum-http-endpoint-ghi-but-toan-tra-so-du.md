---
slug: rust-axum-http-endpoint-ghi-but-toan-tra-so-du
title: "HTTP với axum: endpoint ghi bút toán và tra số dư"
summary: "Bài 31: đưa sổ cái ra HTTP bằng axum. Router, handler async, extractor State/Json/Path và thứ tự của chúng, AppError thành 400/404/409/413/415/422, Router là tower::Service, chạy trên PostgreSQL thật."
category: rust
level: intermediate
tags: [rust, toi-noi, ledger, axum, http, tower]
language: vi
---

Dịch vụ Payment viết bằng C# cần ghi một bút toán vào sổ cái sau khi thu cước chuyến `TN-0001`, và cần hỏi số dư của `driver:DR-001`. Đến bài 30, sổ cái mới chỉ là một thư viện cùng một repository chạy trên PostgreSQL. Muốn service C# gọi được, ta phải đặt một ranh giới mạng quanh nó: hai endpoint HTTP, trả đúng mã trạng thái cho từng kết quả.

Đây là bài 31 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace), kho PostgreSQL rút gọn từ [bài 26](/articles/rust-repository-unit-of-work-trait-transaction) và cách cô lập schema từ [bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci). Cần Docker Compose của lab đang chạy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một handler nhận <code>Json&lt;PostEntryRequest&gt;</code> rồi mới đến <code>State&lt;AppState&gt;</code>. Code có biên dịch được không, và nếu không thì lỗi nằm ở đâu?</p>
<details><summary>Xem đáp án</summary><p>Không biên dịch được. <code>Json</code> đọc và tiêu thụ body nên phải là tham số cuối. Lỗi báo ở chỗ đăng ký handler vào route, không phải ở chỗ khai báo tham số. Mục «Extractor» có thông điệp rustc thật.</p></details>
</div>

## Một request đi qua những bước nào?

Hợp đồng của lab theo «API chung» của loạt: `POST /v1/entries` nhận `{ idempotencyKey, postings: [{ account, amountDong }] }`, và `GET /v1/accounts/{id}/balance` trả `{ account, balanceDong, asOf }`. Hình 1 cho thấy năm bước từ lúc request tới đến lúc có response.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 195" role="img" aria-label="Hình 1. Năm bước của một request trong axum: Router khớp route, extractor lấy dữ liệu, handler async chạy, kết quả Ok hoặc Err(AppError), IntoResponse đổi thành HTTP."><defs><marker id="rs31-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="144" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="62" class="t-b">1. Router</text><text x="32" y="90" class="t-small">khớp method</text><text x="32" y="112" class="t-small">và đường dẫn</text><path d="M168 85 L190 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs31-0-a)"/><rect x="194" y="30" width="144" height="110" rx="10" class="f-phys-soft s-phys"/><text x="206" y="62" class="t-b">2. Extractor</text><text x="206" y="90" class="t-small">State, Path, Json</text><text x="206" y="112" class="t-small">Json đứng cuối</text><path d="M342 85 L364 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs31-0-a)"/><rect x="368" y="30" width="144" height="110" rx="10" class="f-pk-soft s-pk"/><text x="380" y="62" class="t-b">3. Handler</text><text x="380" y="90" class="t-small">async fn, gọi kho</text><text x="380" y="112" class="t-small">PgLedger</text><path d="M516 85 L538 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs31-0-a)"/><rect x="542" y="30" width="144" height="110" rx="10" class="f-phys-soft s-phys"/><text x="554" y="62" class="t-b">4. Kết quả</text><text x="554" y="90" class="t-small">Ok(status, Json)</text><text x="554" y="112" class="t-small">Err(AppError)</text><path d="M690 85 L712 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs31-0-a)"/><rect x="716" y="30" width="144" height="110" rx="10" class="f-warn-soft s-warn"/><text x="728" y="62" class="t-b">5. Response</text><text x="728" y="90" class="t-small">IntoResponse</text><text x="728" y="112" class="t-small">5xx ẩn chi tiết</text><text x="20" y="178" class="t-small">Với Result&lt;Json&lt;_&gt;, JsonRejection&gt;, lỗi body vào handler như một giá trị và đi chung đường lỗi ở bước 4.</text></svg>
</div>
<figcaption>Hình 1. Năm bước của một request: Router, extractor, handler, kết quả, IntoResponse.</figcaption>
</figure>

Router là bảng ánh xạ từ (phương thức, đường dẫn) sang handler. Phần dựng nó:

```rust
// src/app.rs
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/entries", post(post_entry))
        .route("/v1/accounts/{id}/balance", get(get_balance))
        // Một Service viết tay đứng cạnh handler: cả hai cùng là Service trong mắt Router.
        .route_service("/healthz", Healthz)
        .fallback(not_found)
        .with_state(state)
}
```

Từ axum 0.8, tham số đường dẫn viết `{id}`, không còn `:id`. `.fallback` bắt mọi request không khớp route nào, và `.with_state` gắn state dùng chung. `AppState` chỉ chứa `PgLedger`, bên trong là một `PgPool` (một `Arc`), nên `Clone` rẻ: axum clone state cho mỗi request.

Server là ba dòng: bind, đọc cổng thật, `axum::serve`. Lab bind `127.0.0.1:0` để hệ điều hành chọn cổng trống, nhờ vậy `--verify` và `cargo test` chạy song song không đụng cổng của nhau.

```rust
// src/scenario.rs (rút gọn)
let listener = TcpListener::bind("127.0.0.1:0").await?;
let addr = listener.local_addr()?; // cổng thật, ví dụ 127.0.0.1:50827
tokio::spawn(async move { axum::serve(listener, app).await });
```

## Extractor là gì, và thứ tự tham số có quan trọng không?

Handler là một `async fn` bình thường. Mỗi tham số là một extractor: kiểu của tham số quyết định axum lấy gì từ request. `State<T>` lấy state, `Path<T>` lấy phần `{id}` của đường dẫn, `Json<T>` đọc body rồi giải mã bằng serde. Trả về là bất kỳ kiểu nào có `IntoResponse`.

```rust
// src/app.rs
// Thứ tự tham số là thứ tự extractor chạy. `State` không đọc body; `Json` tiêu thụ body nên phải đứng cuối.
// Dùng `Result<Json<_>, JsonRejection>` để lỗi body đi qua `AppError` thay vì response mặc định của axum.
async fn post_entry(
    State(state): State<AppState>,
    body: Result<Json<PostEntryRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<PostEntryResponse>), AppError> {
    let Json(request) = body?;
    let idempotency_key = request.idempotency_key.clone();
    let entry = request.into_entry()?;
    let outcome = state.ledger.post(&entry).await?;
    // 201 khi tạo mới, 200 khi gửi lại cùng key cùng nội dung.
    let status = if outcome.replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((
        status,
        Json(PostEntryResponse {
            entry_id: outcome.id.to_string(),
            idempotency_key,
            replayed: outcome.replayed,
        }),
    ))
}
```

Body HTTP là luồng chỉ đọc được một lần. axum chia extractor thành hai loại: loại cài `FromRequestParts` chỉ nhìn phần đầu request (`State`, `Path`, `Method`, `Uri`), loại cài `FromRequest` tiêu thụ cả request (`Json`, `String`, `Bytes`). Loại sau chỉ được đứng cuối. Đảo thứ tự thì handler không còn thỏa trait `Handler`. Thử đặt `Json` lên trước `State` trong một file thử rồi `cargo build` (rút gọn):

```text
error[E0277]: the trait bound `fn(Json<PostEntryRequest>, ...) -> ... {post_entry}: Handler<_, _>` is not satisfied
 --> examples/wrong_order.rs:4:82
  |
4 | fn main() { let _app: Router<AppState> = Router::new().route("/v1/entries", post(post_entry)); }
  |                                                                             ---- ^^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Json<PostEntryRequest>, State<AppState>) -> ... {post_entry}`
  |
  = note: Consider using `#[axum::debug_handler]` to improve the error message
```

Thông điệp này không nói gì về thứ tự, và handler trả về kiểu chưa có `IntoResponse` cũng ra đúng thông điệp ấy. Gợi ý ở dòng cuối đáng nghe: bật feature `macros` của axum và thêm `#[axum::debug_handler]` thì rustc nói thẳng:

```text
error: `Json<_>` consumes the request body and thus must be the last argument to the handler function
 --> examples/dbg_order.rs:4:31
```

Lab không bật `macros` để giữ ít dependency; tôi chạy ca này trong một bản sao tạm. Handler đã có `State<AppState>` trong tham số thì macro tự suy ra kiểu state, không cần viết `state = AppState`.

Lưu ý thứ hai nằm ở kiểu của `body`. Nếu viết `Json<PostEntryRequest>` trần, mọi lỗi body do axum tự trả về với thân `text/plain`, không đi qua code của ta: JSON cụt trả 400, sai hình dạng trả 422, thiếu `Content-Type: application/json` trả 415. Bọc trong `Result<_, JsonRejection>` thì lỗi trở thành một giá trị, và ta đổi nó sang `AppError` bằng toán tử `?` như mọi lỗi khác. `Path` cũng có rejection riêng (`%FF` giải mã không ra UTF-8), nên `get_balance` bọc nó theo cùng cách.

Lưu ý thứ ba là kích thước. `Json`, `String` và `Bytes` không nhận body lớn hơn 2 MB (đúng 2.097.152 byte): đó là mặc định của `DefaultBodyLimit`, không cần cấu hình gì. Vượt ngưỡng thì rejection mang mã 413. Muốn siết cho một route, thêm `.layer(DefaultBodyLimit::max(64 * 1024))`. Extractor đọc thẳng luồng `Body` không chịu giới hạn này.

## AppError và IntoResponse: một đường ra cho mọi lỗi

Handler trả `Result<T, AppError>`. Muốn dùng được, `AppError` phải có `IntoResponse`: axum gọi nó cho cả nhánh `Ok` lẫn nhánh `Err`. Đây là chỗ duy nhất quyết định lỗi nào thành mã nào, nên đổi chính sách chỉ phải sửa một nơi.

```rust
// src/error.rs (rút gọn)
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> AppError {
        // Giữ mã axum đã chọn: cú pháp hỏng 400, sai hình dạng 422, thiếu Content-Type 415, quá 2 MB 413.
        AppError::Body {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        if status.is_server_error() {
            eprintln!("lỗi 5xx: {self:?}");
        }
        (status, Json(ErrorBody { code, message })).into_response()
    }
}
```

Một tuple `(StatusCode, Json<T>)` tự là `IntoResponse`, nên không cần dựng `Response` bằng tay. Thân lỗi lúc này đơn giản, `{"code": "...", "message": "..."}`; bài 32 đổi nó sang `application/problem+json` mà bảng ánh xạ không đổi.

Lỗi hạ tầng (mất kết nối PostgreSQL, SQL lỗi) trả 500 với thông điệp cố định «lỗi nội bộ». Chi tiết chỉ vào log của server. Test `storage_error_is_500_without_details` ghim điều này: dựng `sqlx::Error::PoolTimedOut`, kiểm response là 500, `message` là «lỗi nội bộ» và không chứa chữ `pool`. 

## Mỗi kết quả của sổ cái ứng với mã nào?

| Tình huống | Mã | `code` trong thân lỗi |
| --- | --- | --- |
| Bút toán mới ghi xong | 201 | (thân thành công, `replayed: false`) |
| Gửi lại cùng key, cùng nội dung | 200 | (thân thành công, `replayed: true`) |
| Tổng posting khác 0, thiếu posting, posting bằng 0, tài khoản chưa mở | 422 | `unbalanced`, `too_few_postings`, `zero_amount`, `unknown_account` |
| JSON đúng cú pháp nhưng sai hình dạng: thiếu trường, sai kiểu, trường lạ | 422 | `bad_body` |
| Cùng key nhưng nội dung khác | 409 | `idempotency_conflict` |
| JSON sai cú pháp; mã tài khoản trên đường dẫn rỗng hoặc không phải UTF-8 | 400 | `bad_body`, `bad_account_id` |
| Thiếu `Content-Type: application/json` | 415 | `bad_body` |
| Body lớn hơn 2 MB | 413 | `bad_body` |
| Số dư của tài khoản chưa mở; route không tồn tại | 404 | `account_not_found`, `route_not_found` |
| Route có nhưng sai phương thức | 405 | (axum tự trả: thân rỗng, có header `Allow`) |
| Lỗi hạ tầng | 500 | `internal` |

Ranh giới 400 và 422 theo RFC 9110: 422 là server hiểu kiểu nội dung, cú pháp đúng, nhưng không xử lý được điều nội dung yêu cầu. JSON cụt không qua nổi bước cú pháp nên là 400. JSON thiếu trường và bút toán lệch 100 đồng đều đúng cú pháp nên là 422; client phân biệt hai ca đó bằng `code`, không phải bằng mã HTTP. Đây cũng là mã axum chọn sẵn, nên lab giữ nguyên. Riêng 201 và 200 mang một ý nghĩa quan trọng với client C# có retry: cả hai đều là thành công, và `replayed` cho biết lần gọi này có tạo thêm gì không. Idempotency ở tầng HTTP không phải phát minh mới, nó dựa trên `ON CONFLICT` của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key).

Số dư ở `GET` là số đã commit. `asOf` lấy từ đồng hồ PostgreSQL (`now()` đổi sang UTC) ngay trong câu truy vấn đọc số dư, chứ không phải đồng hồ của process axum, để hai giá trị đến từ cùng một nguồn. Tài khoản đã mở mà chưa có bút toán trả 200 với `balanceDong: 0` (như `psp:vixanh:clearing` trong ví dụ); tài khoản chưa từng mở trả 404.

## Vì sao nói Router là một tower::Service?

axum không tự định nghĩa mô hình «nhận request, trả response». Nó dùng trait `Service` của crate `tower`: một Service có `poll_ready` (sẵn sàng nhận việc chưa) và `call` (nhận một request, trả về một future cho response). Source axum 0.8.9 viết `impl<B> Service<Request<B>> for Router<()>`, với `B` là mọi kiểu body thỏa `HttpBody<Data = Bytes> + Send + 'static`, và `Error = Infallible`: lỗi không bao giờ là `Err` của Service mà luôn được đổi thành một response (như `AppError` ở trên).

Chữ `()` đáng chú ý. Tham số `S` của `Router<S>` là kiểu state **còn thiếu**: `Router<AppState>` là router đang chờ một `AppState`, và `.with_state(state)` đổi nó thành `Router<()>`. Chỉ `Router<()>` là Service, nên quên `.with_state` là lỗi biên dịch chứ không phải lỗi lúc chạy. Router không dùng state thì ngay từ đầu đã là `Router<()>`. Thử gọi `oneshot` trên router còn thiếu state (rút gọn):

```text
error[E0599]: the method `oneshot` exists for struct `Router<St>`, but its trait bounds were not satisfied
 --> examples/missing_state.rs:7:17
  |
7 |     let _ = app.oneshot(Request::new(Body::empty())).await;
  |                 ^^^^^^^
  |
  = note: the following trait bounds were not satisfied:
          `Router<St>: Service<_>`
          which is required by `Router<St>: tower::ServiceExt<_>`
```

Mỗi handler async cũng được axum bọc thành một Service. Để thấy rằng không có phép màu nào ở đây, lab viết tay một Service và gắn nó cạnh handler:

```rust
// src/healthz.rs
impl Service<Request<Body>> for Healthz {
    type Response = Response;
    type Error = Infallible;
    type Future = Ready<Result<Response, Infallible>>;

    // Không có gì để chờ (không giới hạn đồng thời, không kết nối): luôn sẵn sàng.
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _request: Request<Body>) -> Self::Future {
        ready(Ok((StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()))
    }
}
```

`route_service("/healthz", Healthz)` nhận nó như một route. Hình 2 vẽ hệ quả: Router chỉ là một Service chứa các Service con.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 262" role="img" aria-label="Hình 2. Router là Service chứa ba Service con: hai handler async được bọc thành HandlerService và một Service viết tay."><defs><marker id="rs31-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="70" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="32" y="102" class="t-b">Request</text><text x="32" y="126" class="t-small">method, path,</text><text x="32" y="146" class="t-small">header, body</text><path d="M164 115 L208 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rs31-1-a)"/><rect x="214" y="30" width="452" height="170" rx="10" class="f-pk-soft s-pk"/><text x="230" y="56" class="t-b">Router = Service</text><rect x="230" y="76" width="132" height="100" rx="8" class="f-phys-soft s-phys"/><text x="242" y="104" class="t-b">post_entry</text><text x="242" y="128" class="t-small">async fn</text><text x="242" y="148" class="t-small">HandlerService</text><rect x="374" y="76" width="132" height="100" rx="8" class="f-phys-soft s-phys"/><text x="386" y="104" class="t-b">get_balance</text><text x="386" y="128" class="t-small">async fn</text><text x="386" y="148" class="t-small">HandlerService</text><rect x="518" y="76" width="132" height="100" rx="8" class="f-warn-soft s-warn"/><text x="530" y="104" class="t-b">Healthz</text><text x="530" y="128" class="t-small">impl Service</text><text x="530" y="148" class="t-small">viết tay</text><path d="M670 115 L714 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rs31-1-a)"/><rect x="720" y="70" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="732" y="102" class="t-b">Response</text><text x="732" y="126" class="t-small">status, header,</text><text x="732" y="146" class="t-small">body</text><text x="20" y="226" class="t-small">Mỗi ô trong Router là một Service: nhận Request, trả future ra Result&lt;Response, Infallible&gt;.</text><text x="20" y="248" class="t-small">Layer (bài 33) bọc ngoài một Service mà không cần biết bên trong là gì.</text></svg>
</div>
<figcaption>Hình 2. Router là một Service chứa các Service con; handler async và Service viết tay đứng ngang hàng.</figcaption>
</figure>

Hệ quả thực dụng đầu tiên là test không cần cổng mạng: có `Service` thì có `oneshot`, đưa thẳng một `Request` vào và nhận `Response`. Hàm trợ giúp dưới đây chỉ đòi hỏi một Service nhận `Request<Body>`, trả `Response<Body>`, không bao giờ lỗi:

```rust
// tests/http.rs
async fn call<S>(service: S, request: Request<Body>) -> (StatusCode, Value)
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible>,
{
    let response = service.oneshot(request).await.unwrap_or_else(|e| match e {});
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 64 * 1024).await.expect("đọc body");
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}
```

Hệ quả thứ hai nằm ở bài 33: timeout, request id, giới hạn tốc độ đều là `Layer`, tức là một Service bọc quanh một Service khác. Vì Router cũng là Service, một layer đặt được ở một route, một nhóm route hoặc cả router bằng cùng một cách.

## So với Minimal API của .NET

[Bài 16 của loạt C#](/articles/dotnet10-http-minimal-api) dựng `POST /quotes` bằng Minimal API. Hai mô hình giống nhau ở ý tưởng (hàm mỏng nối HTTP với logic nghiệp vụ) và khác ở chỗ lỗi được bắt.

| Ý | Minimal API (ASP.NET Core 10) | axum |
| --- | --- | --- |
| Đăng ký route | `app.MapPost("/quotes", handler)` | `Router::new().route("/v1/entries", post(handler))` |
| Lấy dữ liệu vào | tham số handler được bind theo kiểu; service lấy từ DI | tham số là extractor; dùng chung qua `State` |
| Body chỉ đọc một lần | framework quyết định cách bind | luật nằm trong hệ kiểu: `Json` phải đứng cuối, sai thì không biên dịch |
| Kết quả | `Results.Ok`, `Results.Created` hoặc trả object | `IntoResponse`, tuple `(StatusCode, Json<T>)` |
| Lỗi | exception và ProblemDetails (bài 32 đối chiếu tiếp) | `Result<T, AppError>`, `?`, `AppError: IntoResponse` |
| Test không mở cổng | `WebApplicationFactory` ([bài C# về integration test](/articles/dotnet10-webapplicationfactory-integration-test)) | `oneshot` trên `Router` |

Điểm khác đáng nhớ: ở axum, đường lỗi nằm trong chữ ký hàm: nhìn `Result<_, AppError>` là biết handler có thể thất bại theo những cách nào, và trình biên dịch buộc mọi nhánh lỗi có một mã HTTP. Mặt trái là thông điệp lỗi biên dịch của `Handler` khó đọc như đã thấy; `debug_handler` có mặt vì lý do đó.

## Chạy ví dụ và kiểm chứng

Từ gốc repository, sau khi bật hạ tầng của lab:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml
```

Lệnh đầu tự khởi động server trên một cổng ngẫu nhiên, gọi qua TCP thật rồi dọn schema. `reqwest` chỉ là client của `cargo run` và `--verify`; server không dùng nó. Output (rút gọn phần `asOf`; đây là in lại qua `serde_json::Value` nên khóa JSON xếp theo thứ tự chữ cái):

```text
POST /v1/entries                           -> 201 {"entryId":"JE-000001","idempotencyKey":"trip:TN-0001:capture","replayed":false}
POST /v1/entries                           -> 200 {"entryId":"JE-000001","idempotencyKey":"trip:TN-0001:capture","replayed":true}
POST /v1/entries                           -> 409 {"code":"idempotency_conflict","message":"key trip:TN-0001:capture đã dùng cho bút toán khác nội dung"}
POST /v1/entries                           -> 422 {"code":"unbalanced","message":"bút toán lệch, tổng bằng -100 VND"}
POST /v1/entries                           -> 422 {"code":"unknown_account","message":"tài khoản customer:CU-999 chưa mở"}
POST /v1/entries                           -> 400 {"code":"bad_body","message":"Failed to parse the request body as JSON: postings: EOF while parsing a list at line 1 column 56"}
GET /v1/accounts/customer:CU-001/balance   -> 200 {"account":"customer:CU-001","asOf":"…","balanceDong":-25500}
GET /v1/accounts/driver:DR-001/balance     -> 200 {"account":"driver:DR-001","asOf":"…","balanceDong":20400}
GET /v1/accounts/platform:revenue/balance  -> 200 {"account":"platform:revenue","asOf":"…","balanceDong":5100}
GET /v1/accounts/psp:vixanh:clearing/balance -> 200 {"account":"psp:vixanh:clearing","asOf":"…","balanceDong":0}
GET /v1/accounts/customer:CU-999/balance   -> 404 {"code":"account_not_found","message":"không có tài khoản customer:CU-999"}
GET /v1/accounts/%20/balance               -> 400 {"code":"bad_account_id","message":"mã tài khoản không hợp lệ: mã tài khoản rỗng"}
GET /v1/nope                               -> 404 {"code":"route_not_found","message":"không có route GET /v1/nope"}
GET /healthz                               -> 200 {"status":"ok"}
```

Số dư khớp bài toán `TN-0001`: khách -25.500, tài xế +20.400, nền tảng +5.100, tổng bằng 0. Để tự gọi bằng `curl`, chạy `cargo run --manifest-path … -- --serve` (Ctrl-C để dừng và dọn schema). Sau khi `POST` bút toán `TN-0001`, `GET` số dư cho thân JSON đúng thứ tự khai báo của struct (rút gọn header):

```text
HTTP/1.1 200 OK
content-type: application/json

{"account":"driver:DR-001","balanceDong":20400,"asOf":"2026-11-06T13:51:53.561040Z"}
```

`--verify` kiểm toàn bộ bảng mã ở trên, thêm các ca: thiếu trường, sai kiểu, trường lạ trả 422; thiếu `Content-Type` trả 415; `GET /v1/entries` trả 405 với `Allow: POST` (RFC 9110 bắt buộc header này); `%FF` trên đường dẫn trả 400; và tám request song song cùng một key mới cho đúng một 201 cùng bảy 200, số dư tài xế sau hai chuyến là 40.800. Kết thúc nó in `VERIFY OK`. `cargo test` chạy 8 ca: 2 ca đơn vị cho bảng ánh xạ lỗi (không cần PostgreSQL) và 6 ca `oneshot` với PostgreSQL thật. Biên 2 MB nằm ở `oneshot`: body đúng 2.097.152 byte qua được giới hạn rồi hỏng cú pháp (400), thêm 1 byte là 413. Qua TCP, server đóng kết nối khi chưa đọc hết body nên client có thể nhận reset thay vì 413; tôi đã gặp đúng ca đó. Thiếu `TOI_NOI_PG_URL`, lab thoát mã 1 và không bỏ qua.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là hai endpoint để học cơ chế, chưa phải API production: chưa xác thực, chưa phân quyền, giới hạn body vẫn là 2 MB mặc định dù một bút toán chỉ cần vài KB, chưa timeout, chưa request id, chưa tracing, nên lỗi 5xx chỉ ghi bằng <code>eprintln!</code>. JSON lỗi tự đặt, chưa theo Problem Details. Server chưa tắt êm: <code>--serve</code> hủy task thẳng tay, graceful shutdown là bài 39. Kho PostgreSQL là bản rút gọn, không có trait nên không thay được bằng bản giả. Máy lab chạy client và server cùng một máy, không TLS.</p>
</div>

**Bài tập:** client gửi `POST /v1/entries` với `Content-Type: text/plain` và body `{}`. Response là gì? Nếu đổi tham số `body` của handler thành `Json<PostEntryRequest>` trần thì thân response đổi thế nào, và vì sao một client C# phân tích lỗi theo `code` sẽ gặp rắc rối?

<details><summary>Xem lời giải</summary><p>Bản hiện tại trả 415 với thân JSON <code>{"code":"bad_body","message":"…"}</code> (ca này nằm trong <code>--verify</code>). Với <code>Json&lt;PostEntryRequest&gt;</code> trần, axum tự trả cùng mã 415 nhưng thân là <code>text/plain</code>: <code>Expected request with `Content-Type: application/json`</code>. Tôi đã chạy ca này trong một bản sao tạm. Mã không đổi, nhưng client phân tích thân theo <code>code</code> sẽ gặp một chuỗi không phải JSON, cả ở ca 400 và 422 của extractor. Bọc <code>Result&lt;Json&lt;_&gt;, JsonRejection&gt;</code> giữ cho mọi lỗi body cùng một dạng.</p></details>

Bài trước: [bài 30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan). Bài tiếp: [bài 32](/articles/rust-validation-problem-details-openapi-utoipa).

## Tài liệu gốc

- [docs.rs: axum](https://docs.rs/axum), gồm mục `extract` về thứ tự extractor và `response::IntoResponse`.
- [docs.rs: tower, trait `Service`](https://docs.rs/tower/latest/tower/trait.Service.html).
- [docs.rs: axum, `Router`](https://docs.rs/axum/latest/axum/struct.Router.html).
- [docs.rs: axum, `DefaultBodyLimit`](https://docs.rs/axum/latest/axum/extract/struct.DefaultBodyLimit.html).
- [RFC 9110: mã trạng thái 4xx](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.5).
- [The Rust Book: Asynchronous Programming](https://doc.rust-lang.org/book/ch17-00-async-await.html).

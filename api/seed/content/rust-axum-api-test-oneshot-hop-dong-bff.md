---
slug: rust-axum-api-test-oneshot-hop-dong-bff
title: "API test với axum: oneshot, DB thật và hợp đồng với BFF C#"
summary: "Bài 34: gọi Router bằng oneshot không cần socket, khi nào cần server thật; ca 401, 403, 409, 422 trên PostgreSQL thật; snapshot OpenAPI bằng insta; file JSON chung để test Rust và client C# cùng kiểm."
category: programming
level: intermediate
tags: [rust, toi-noi, ledger, axum, testing, openapi]
language: vi
---

Payment BFF viết bằng C# gọi `POST /v1/entries` của ledger-api. Một buổi chiều, ai đó đổi `amountDong` thành `amount` trong DTO của Rust vì "tên ngắn hơn". `cargo test` xanh, vì test Rust cũng dùng tên mới. Test C# cũng xanh, vì nó còn dùng tên cũ. Lỗi chỉ lộ ra khi hai service nói chuyện với nhau, tức là ở môi trường chung, tức là muộn.

Đây là bài 34 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab là bản thu nhỏ của ledger-api ở [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du), [bài 32](/articles/rust-validation-problem-details-openapi-utoipa) và [bài 33](/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit): hai route, extractor JWT rút gọn từ bài 33 (`scope` thay cho vai trò, không có chồng tower), adapter PostgreSQL rút gọn từ [bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Test Rust xanh, test C# xanh. Hợp đồng giữa hai bên đã được chứng minh chưa?</p>
<details><summary>Xem đáp án</summary><p>Chưa. Mỗi bên có thể đang kiểm một bản hợp đồng riêng của mình và hai bản đã lệch nhau. Chỉ khi hai test cùng đọc một nguồn dữ liệu, ví dụ một file JSON chung, thì "cả hai xanh" mới có nghĩa. Ngay cả khi đó nó chỉ chứng minh những gì file nói.</p></details>
</div>

## Router là một Service: gọi nó không cần cổng

`axum::Router` implement `tower::Service<Request>`, tức là một hàm bất đồng bộ từ request sang response. Test có thể tự dựng `Request`, đưa vào router và nhận `Response` mà không bind cổng, không có HTTP parser chen giữa. `oneshot` của `tower::ServiceExt` làm đúng việc đó một lần. Lab gói thành hàm `send`:

```rust
// src/testkit.rs (rút gọn)
pub async fn send(app: &Router, method: Method, uri: &str,
                  token: Option<&str>, json_body: Option<&str>) -> Reply {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    // ... thêm content-type và Body nếu có json_body
    let request = builder.body(body).expect("request hợp lệ");
    // oneshot tiêu thụ service; Router::clone rẻ nên mỗi lần gọi dùng một bản sao.
    let response = app.clone().oneshot(request).await.expect("Router không lỗi");
    // ... tách status, header, đọc thân bằng axum::body::to_bytes
}
```

`expect` hợp lý vì lỗi của `Router` là `Infallible`: handler lỗi vẫn trả một `Response` (404, 500), không trả `Err`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Hai đường vào cùng một Router. Đường oneshot đi thẳng Request, Router, Response. Đường server thật đi qua TcpStream, hyper phân tích HTTP/1.1, Router, hyper ghi phản hồi và thêm header date, content-length, rồi TcpStream. Hai ô hyper là phần oneshot bỏ qua."><defs><marker id="rs34-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="24" y="30" class="t-b">oneshot: gọi Service trực tiếp</text><rect x="24" y="42" width="144" height="70" rx="8" class="f-phys-soft s-phys"/><text x="36" y="68" class="t-b">Request</text><text x="36" y="88" class="t-small">dựng tay bằng</text><text x="36" y="103" class="t-small">Request::builder</text><rect x="368" y="42" width="144" height="70" rx="8" class="f-pk-soft s-pk"/><text x="380" y="68" class="t-b">Router</text><text x="380" y="88" class="t-small">extractor, handler,</text><text x="380" y="103" class="t-small">lỗi, JSON</text><rect x="712" y="42" width="144" height="70" rx="8" class="f-phys-soft s-phys"/><text x="724" y="68" class="t-b">Response</text><text x="724" y="88" class="t-small">status, header</text><text x="724" y="103" class="t-small">của handler</text><path d="M168 77 L366 77" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-0-a)"/><path d="M512 77 L710 77" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-0-a)"/><text x="24" y="156" class="t-b">server thật trên 127.0.0.1:0</text><rect x="24" y="168" width="144" height="70" rx="8" class="f-phys-soft s-phys"/><text x="36" y="194" class="t-b">TcpStream</text><text x="36" y="214" class="t-small">byte HTTP viết tay</text><rect x="196" y="168" width="144" height="70" rx="8" class="f-warn-soft s-warn"/><text x="208" y="194" class="t-b">hyper đọc</text><text x="208" y="214" class="t-small">parse HTTP/1.1,</text><text x="208" y="229" class="t-small">ConnectInfo</text><rect x="368" y="168" width="144" height="70" rx="8" class="f-pk-soft s-pk"/><text x="380" y="194" class="t-b">Router</text><text x="380" y="214" class="t-small">cùng router</text><text x="380" y="229" class="t-small">với đường trên</text><rect x="540" y="168" width="144" height="70" rx="8" class="f-warn-soft s-warn"/><text x="552" y="194" class="t-b">hyper ghi</text><text x="552" y="214" class="t-small">thêm date,</text><text x="552" y="229" class="t-small">content-length</text><rect x="712" y="168" width="144" height="70" rx="8" class="f-phys-soft s-phys"/><text x="724" y="194" class="t-b">TcpStream</text><text x="724" y="214" class="t-small">đọc tới khi đóng</text><path d="M168 203 L194 203" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-0-a)"/><path d="M340 203 L366 203" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-0-a)"/><path d="M512 203 L538 203" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-0-a)"/><path d="M684 203 L710 203" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-0-a)"/><text x="24" y="270" class="t-small">Hai ô màu cảnh báo là phần của giao thức mà oneshot không đi qua.</text></svg>
</div>
<figcaption>Hình 1. Cùng một <code>Router</code>, hai đường vào. <code>oneshot</code> bỏ qua hai ô hyper.</figcaption>
</figure>

## Khi nào oneshot không đủ và phải mở cổng thật?

"Không có socket" nghĩa là một số thứ không tồn tại trong test. Lab chạy cả hai đường để thấy chỗ khác. Server thật bind `127.0.0.1:0` rồi đọc cổng do hệ điều hành cấp, không ghi cứng cổng nào:

```rust
// src/rawhttp.rs (rút gọn)
let listener = TcpListener::bind("127.0.0.1:0").await?;
let addr = listener.local_addr()?;
let task = tokio::spawn(async move {
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service).await
});
```

Client là `TcpStream` ghi tay byte HTTP/1.1, để lab bớt một dependency; thực tế hãy dùng client HTTP thật.

Handler dùng địa chỉ client (ví dụ giới hạn theo IP, thứ `tower::limit` ở bài 33 không làm) là ví dụ rõ nhất:

```rust
// tests/real_server.rs (rút gọn)
fn ip_router() -> Router {
    Router::new().route("/ip",
        get(|ConnectInfo(addr): ConnectInfo<SocketAddr>| async move { addr.ip().to_string() }))
}
// oneshot: không ai chèn ConnectInfo -> 500
assert_eq!(send(&ip_router(), Method::GET, "/ip", None, None).await.status, 500);
// chữa trong test bằng MockConnectInfo -> 203.0.113.7
// server thật: địa chỉ của kết nối TCP thật
assert_eq!(rawhttp::request(addr, "GET", "/ip", &[], None).await?.body, "127.0.0.1");
```

| Điều cần kiểm | `oneshot` | Server thật |
| --- | --- | --- |
| Routing, extractor, handler, status, JSON | Có | Có |
| Header `date`, `content-length` | Không có (`date` là `None`) | Có, do hyper thêm |
| `ConnectInfo<SocketAddr>` | 500; chữa bằng `MockConnectInfo` | `127.0.0.1` thật |
| Request sai giao thức (tên header chứa dấu cách) | Không dựng nổi: `Request` đã dựng sẵn | hyper trả 400, router không được gọi |
| Chi phí mỗi request, máy tải cao | 120–191 µs | 1,29–1,49 ms |

Dòng cuối do `cargo run --release -- --measure` đo: 2.000 lần `GET /api-docs/openapi.json` (không chạm database), ba lần chạy, server thật mở kết nối mới mỗi lần. Số trong bảng đo lúc load khoảng 180; bản lab đầu, tài liệu OpenAPI nhỏ hơn và không rõ tải máy, đo được 49–69 µs và 201–412 µs. Số tuyệt đối đổi theo tải, chỉ nên đọc chênh lệch. Số đo trên máy viết bài (Apple M1 Max), không phải SLA. Một điều tôi từng đoán sai: hyper không từ chối request HTTP/1.1 thiếu `Host`, nó trả 200; tôi thay bằng ca tên header sai, ca hyper thực sự trả 400.

Viết hầu hết test bằng `oneshot`; chỉ mở cổng thật cho thứ phụ thuộc vào kết nối (địa chỉ client, hành vi của hyper, graceful shutdown ở bài 39). Cả hai đường đều không kiểm TLS hay reverse proxy.

## Test với PostgreSQL thật: thêm gì so với bài 27?

Test lỗi nghiệp vụ như 409 phải chạy trên PostgreSQL thật, vì `ON CONFLICT` và trigger là thứ cần kiểm. Fixture `TestDb` giữ nguyên ý bài 27: mỗi ca một schema `lab_<uuid>` có dấu tuổi trong `COMMENT ON SCHEMA`, `search_path` đặt bằng `after_connect`, migration chạy trong schema đó.

Lab đổi một chỗ so với bài 27: `Drop` giờ cũng dọn. Bài 27 để ca panic sót schema rồi dọn từ ngoài, vì `Drop` đồng bộ không `await` được, còn `block_on` trong runtime của test thì panic. Điều đó vẫn đúng; ở đây ta đi vòng bằng một thread khác có runtime riêng. Lý do đổi: thí nghiệm đổi tên trường ở mục sau làm tám ca đỏ, sáu ca dùng database để sót schema trong database dùng chung.

```rust
// src/testkit.rs (rút gọn)
impl Drop for TestDb {
    fn drop(&mut self) {
        if self.closed { return; }
        let (url, schema) = (self.url.clone(), self.schema.clone());
        let cleanup = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            runtime.block_on(drop_schema(&url, &schema))
        });
        // ... cảnh báo nếu không dọn được
    }
}
```

Cái giá là `join` chặn thread của test. Tôi thử bằng test tạm cố ý panic (đã xoá):

| Tình huống | Kết quả |
| --- | --- |
| Panic thường | schema biến mất khỏi `pg_namespace` |
| Runtime của test đã tắt trước `Drop` | vẫn dọn được, thread có runtime riêng |
| Panic khi còn transaction mở, runtime `multi_thread` | dọn được |
| Như trên, runtime `current_thread` (mặc định của `#[tokio::test]`) | bản đầu **treo vô hạn**; có `lock_timeout` thì đỏ sau khoảng 4 giây, cảnh báo, schema sót |
| Tiến trình bị kill | `Drop` không chạy, schema sót |

Ca treo đáng nhớ: trong lúc `join`, runtime của test đứng yên nên transaction không rollback được và vẫn giữ khoá, còn `DROP SCHEMA` chờ khoá đó. Vì thế `drop_schema` đặt `lock_timeout = '3s'`: chấp nhận sót còn hơn treo cả bộ test. Schema sót vẫn mang dấu cùng dạng bài 27, nên `--cleanup-stale` của lab 27 dọn được theo tuổi.

## Mỗi mã lỗi đến từ tầng nào?

`post_entry` nhận `Writer` (extractor xác thực và kiểm quyền, cùng lý do với `Authorized<P>` ở bài 33) trước `ApiJson` (đọc body), nên 401 và 403 đều đến trước lỗi body. Hình 2 là thứ tự kiểm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 2. Thứ tự kiểm trong POST /v1/entries và mã trả về ở mỗi tầng: extractor Writer trả 401 hoặc 403, ApiJson trả 400, 415 hoặc 422, kiểm tra bút toán trả 422, PostgreSQL trả 409 hoặc 422, thành công trả 201 hoặc 200."><defs><marker id="rs34-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="15" y="30" width="150" height="80" rx="8" class="f-phys-soft s-phys"/><text x="27" y="56" class="t-b">1. Writer</text><text x="27" y="76" class="t-small">đọc JWT, kiểm</text><text x="27" y="91" class="t-small">chữ ký, hạn, quyền</text><rect x="190" y="30" width="150" height="80" rx="8" class="f-phys-soft s-phys"/><text x="202" y="56" class="t-b">2. ApiJson</text><text x="202" y="76" class="t-small">cú pháp, content-</text><text x="202" y="91" class="t-small">type, hình dạng</text><rect x="365" y="30" width="150" height="80" rx="8" class="f-phys-soft s-phys"/><text x="377" y="56" class="t-b">3. Bút toán</text><text x="377" y="76" class="t-small">tổng bằng 0, đủ</text><text x="377" y="91" class="t-small">dòng, số tiền khác 0</text><rect x="540" y="30" width="150" height="80" rx="8" class="f-pk-soft s-pk"/><text x="552" y="56" class="t-b">4. PostgreSQL</text><text x="552" y="76" class="t-small">ON CONFLICT,</text><text x="552" y="91" class="t-small">khoá ngoại</text><rect x="715" y="30" width="150" height="80" rx="8" class="f-phys-soft s-phys"/><text x="727" y="56" class="t-b">5. Thành công</text><text x="727" y="76" class="t-small">ghi và trả</text><text x="727" y="91" class="t-small">entryId</text><path d="M165 70 L188 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M340 70 L363 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M515 70 L538 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M690 70 L713 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-1-a)"/><rect x="15" y="140" width="150" height="60" rx="8" class="f-warn-soft s-warn"/><text x="27" y="166" class="t-b">401 · 403</text><text x="27" y="186" class="t-small">chưa biết / thiếu quyền</text><rect x="190" y="140" width="150" height="60" rx="8" class="f-warn-soft s-warn"/><text x="202" y="166" class="t-b">400 · 415 · 422</text><text x="202" y="186" class="t-small">hỏng / sai loại / sai dạng</text><rect x="365" y="140" width="150" height="60" rx="8" class="f-warn-soft s-warn"/><text x="377" y="166" class="t-b">422</text><text x="377" y="186" class="t-small">lệch, thiếu dòng, 0 đồng</text><rect x="540" y="140" width="150" height="60" rx="8" class="f-warn-soft s-warn"/><text x="552" y="166" class="t-b">409 · 422</text><text x="552" y="186" class="t-small">xung đột / khoá ngoại</text><rect x="715" y="140" width="150" height="60" rx="8" class="f-phys-soft s-phys"/><text x="727" y="166" class="t-b">201 · 200</text><text x="727" y="186" class="t-small">mới / replay</text><path d="M90 110 L90 138" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M265 110 L265 138" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M440 110 L440 138" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M615 110 L615 138" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs34-1-a)"/><path d="M790 110 L790 138" class="f-none s-phys" stroke-width="2" marker-end="url(#rs34-1-a)"/></svg>
</div>
<figcaption>Hình 2. Mỗi tầng trả mã riêng. Test nên kiểm cả thứ tự: body hỏng mà không có token vẫn là 401, thiếu quyền vẫn là 403, không phải 400.</figcaption>
</figure>

`axum::Json` mặc định trả văn bản khi body lỗi. Lab bọc nó trong `ApiJson` để mọi lỗi cùng dạng problem+json, và tách hai trường hợp hay bị gộp: cú pháp hỏng là 400, đúng cú pháp nhưng sai hình dạng là 422. Hàm chuyển đổi lấy nguyên từ bài 32:

```rust
// src/api.rs (rút gọn)
let instance = req.uri().path().to_string();
match Json::<T>::from_request(req, state).await {
    Ok(Json(value)) => Ok(ApiJson(value)),
    // Giữ mã axum đã chọn (400, 415, 422, 413), chỉ đổi cách trình bày.
    Err(rejection) => Err(problem_from_rejection(&rejection, &instance).into()),
}
```

Về token: `leeway` mặc định 60 giây của `jsonwebtoken`, bài 33 hạ xuống 5, lab này đặt 0.

Thân lỗi theo đúng hợp đồng của [bài 32](/articles/rust-validation-problem-details-openapi-utoipa): `type` dạng `https://toi-noi.example/problems/<slug>`, cùng slug và mã trạng thái, có `instance`, `traceId`, `errors`. Bốn chỗ khác có chủ ý, vì đây là bản thu nhỏ trên PostgreSQL:

- `POST` vào tài khoản chưa mở là 422 `unknown-account`: bảng `accounts` có khoá ngoại, sổ trong bộ nhớ của bài 32 không có ca này.
- `GET` tài khoản đã mở mà chưa có bút toán trả 200 số dư 0; 404 `unknown-account` chỉ dành cho tài khoản chưa mở. Bài 32 không có bảng tài khoản nên trả 404 cho cả hai.
- Có thêm 401 `unauthenticated` và 403 `forbidden`, vì bài 32 chưa có xác thực.
- Tài liệu vẫn ở `/api-docs/openapi.json` nhưng không có Swagger UI, để bớt một dependency.

Các ca chạy qua `oneshot` trên PostgreSQL thật, mỗi ca kiểm cả hệ quả trong database:

| Ca | Kết quả | Kiểm thêm |
| --- | --- | --- |
| Gửi `post-entry.request.json` | 201, `JE-000001` | số dư tài xế `DR-001` = 20.400 |
| Gửi lại cùng body | 200, `replayed: true` | số dư không nhân đôi |
| Cùng key, khác số tiền | 409 `idempotency-conflict` | `journal_entries` vẫn có 1 dòng |
| Lệch, một dòng, dòng 0 đồng, tài khoản chưa có, tràn `i64` | 422, mỗi loại một `type` | `journal_entries` có 0 dòng |
| JSON cụt; thiếu trường; số viết dạng chuỗi; trường lạ | 400; ba ca 422 | |
| Không `Content-Type` | 415 | |
| Mã tài khoản rỗng trên đường dẫn; route lạ; sai method | 400; 404; 405 | đều là problem+json |
| Không token, khoá lạ, hết hạn, `Basic` | 401 và `WWW-Authenticate: Bearer` | thân giống nhau cho mọi lý do |
| Token chỉ đọc gọi POST, kể cả body hỏng; chỉ ghi gọi GET | 403 | 0 dòng được ghi |

Ca "tài khoản chưa có" chỉ lộ trên database thật: khoá ngoại (SQLSTATE `23503`) vỡ khi chèn posting, giữa transaction, nên cả header phải rollback; `journal_entries` còn 0 dòng.

## Snapshot OpenAPI: đổi hợp đồng phải hiện trong diff

`utoipa` sinh tài liệu OpenAPI 3.1 từ code (`#[derive(OpenApi)]`, `#[utoipa::path]`, `ToSchema`). Tài liệu là đầu ra của chương trình, nên so nó với bản đã commit bằng `insta`:

```rust
// tests/openapi.rs
let reply = send(&offline_router(), Method::GET, OPENAPI_PATH, None, None).await;
assert_eq!(reply.status, StatusCode::OK);
insta::assert_json_snapshot!("openapi_document", reply.json());
```

`offline_router` dùng pool `connect_lazy` nên test này không cần database. Snapshot nằm ở `tests/snapshots/openapi__openapi_document.snap` (409 dòng) và được commit. Lần đầu test đỏ vì chưa có snapshot; tạo bằng `INSTA_UPDATE=always cargo test --test openapi` rồi đọc file. Trên CI đặt `INSTA_UPDATE=no`: không ghi file nào, chỉ chạy test (mặc định `auto` cũng thành `no` nếu `insta` nhận ra CI). Không commit file `.snap.new` mà `insta` để lại trên máy khi test đỏ.

Để thấy test đỏ thật, tôi thêm `#[serde(rename = "amount")]` vào trường `amount_dong` (đúng cái "đổi tên cho ngắn" ở mở bài). Output của `cargo test --test openapi`:

```text
-old snapshot
+new results
────────────┬───────────────────────────────────────────────────────
  119       │-          "amountDong": {
        119 │+          "amount": {
  ...
  128       │-          "amountDong"
        128 │+          "amount"
test openapi_document_matches_the_committed_snapshot ... FAILED
```

Cùng thay đổi làm đỏ thêm 4 ca gọi API và 3 ca hợp đồng; ca đọc file mẫu báo `unknown field amountDong, expected account or amount`. Khôi phục `src/api.rs`, xoá `.snap.new`: toàn bộ xanh. Snapshot không nói thay đổi sai; nó buộc người đổi phải thấy diff và duyệt. Muốn giữ tên mới thì sửa snapshot, file mẫu và client C# trong cùng pull request.

## Hai bên cùng đọc một file: hợp đồng với client C#

Snapshot OpenAPI không nói gì về client. Cách rẻ nhất để hai bên cùng kiểm là chung một nguồn: ba file trong `contracts/`, gồm `post-entry.request.json`, `post-entry.response.json` của chuyến `TN-0001` và `post-entry.unbalanced.problem.json` cho thân lỗi. Hình 3 cho thấy ai đọc gì.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Ba file JSON trong thư mục contracts nằm giữa. Bên trái là test Rust: parse vào DTO của server, gửi request thật qua oneshot và so thân trả về với file response. Bên phải là client C#: Deserialize bằng System.Text.Json Web defaults, serialize lại và so với file."><defs><marker id="rs34-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="270" height="170" rx="10" class="f-phys-soft s-phys"/><text x="36" y="68" class="t-b">Test Rust (tests/contract.rs)</text><text x="36" y="96" class="t-small">parse request vào DTO của server</text><text x="36" y="116" class="t-small">POST bằng oneshot, DB thật</text><text x="36" y="136" class="t-small">thân trả về = response.json</text><text x="36" y="156" class="t-small">trường lạ bị từ chối (422)</text><text x="36" y="176" class="t-small">khoá chỉ camelCase</text><rect x="320" y="70" width="240" height="130" rx="10" class="f-pk-soft s-pk"/><text x="336" y="98" class="t-b">contracts/</text><text x="336" y="124" class="t-small">post-entry.request.json</text><text x="336" y="144" class="t-small">post-entry.response.json</text><text x="336" y="164" class="t-small">….unbalanced.problem.json</text><text x="336" y="184" class="t-small">một bản, không sao chép</text><rect x="590" y="40" width="270" height="170" rx="10" class="f-phys-soft s-phys"/><text x="606" y="68" class="t-b">Client C# (dotnet-client/)</text><text x="606" y="96" class="t-small">Deserialize, JsonSerializerDefaults.Web</text><text x="606" y="116" class="t-small">tổng posting = 0</text><text x="606" y="136" class="t-small">Serialize lại = file gốc</text><text x="606" y="156" class="t-small">thiếu trường hay số dạng chuỗi</text><text x="606" y="176" class="t-small">cũng được kiểm</text><path d="M320 125 L292 125" class="f-none s-pk" stroke-width="2" marker-end="url(#rs34-2-a)"/><path d="M560 125 L588 125" class="f-none s-pk" stroke-width="2" marker-end="url(#rs34-2-a)"/><text x="296" y="240" class="t-small">Đổi một trường: phía nào không cập nhật file sẽ đỏ.</text></svg>
</div>
<figcaption>Hình 3. Ba file JSON là nguồn duy nhất của hợp đồng thủ công; mỗi bên có test riêng đọc chúng.</figcaption>
</figure>

Phía Rust, ca quan trọng nhất: gửi đúng file request, nhận 201, và thân trả về phải bằng file response.

```rust
// tests/contract.rs (rút gọn)
const REQUEST: &str = include_str!("../contracts/post-entry.request.json");
const RESPONSE: &str = include_str!("../contracts/post-entry.response.json");
// ... POST REQUEST bằng oneshot trên schema riêng
assert_eq!(reply.status, StatusCode::CREATED);
let expected: Value = serde_json::from_str(RESPONSE).unwrap();
assert_eq!(reply.json(), expected, "thân trả về phải bằng file hợp đồng");
```

Ca lỗi cũng so như vậy, chỉ trừ `traceId` sinh mới mỗi lần (kiểm dạng 32 hex). Phía C# (thư mục `dotnet-client/`, .NET SDK 10.0.401) dùng record, đọc đúng ba file đó và serialize ngược lại hai file đầu:

```csharp
// dotnet-client/src/LedgerClient/Contracts.cs (rút gọn)
public sealed record Posting(
    [property: JsonRequired] string Account,
    [property: JsonRequired] long AmountDong);
public static readonly JsonSerializerOptions Web = new(JsonSerializerDefaults.Web);
```

Output thật của `dotnet run --project src/LedgerClient`:

```text
request:  key=trip:TN-0001:capture, 3 posting, tổng=0
  customer:CU-001     -25500
  platform:revenue      5100
  driver:DR-001        20400
response: entryId=JE-000001, replayed=False, 3 posting
problem:  type=https://toi-noi.example/problems/unbalanced-entry, status=422, instance=/v1/entries
OK   tổng posting của request bằng 0
OK   response lặp lại đúng key của request
OK   response lặp lại đúng posting của request
OK   serialize request ngược lại khớp file
OK   serialize response ngược lại khớp file
OK   problem là unbalanced-entry 422 và có traceId 32 ký tự
CONTRACT OK (C#)
```

`dotnet test tests/LedgerClient.Tests` chạy 9 ca xanh. Hai ca là phát hiện thật khi viết bài, và là lý do hợp đồng phải được kiểm ở **cả hai** phía:

- Với `JsonSerializerDefaults.Web`, body `{"account": "driver:DR-001"}` thiếu `amountDong` **không lỗi**: record không đánh dấu bắt buộc nhận `AmountDong = 0`. Lần đầu tôi viết test mong `JsonException` và nó đỏ. Rust trả 422 cho đúng body đó. Sửa bằng `[property: JsonRequired]` trên mọi tham số của record hợp đồng.
- Web defaults cho phép đọc số viết dạng chuỗi: `"amountDong": "100"` vào được `long` 100, còn `serde` với `i64` từ chối nó (ca 422 ở trên). Nếu muốn client chặt như server, dùng `NumberHandling = JsonNumberHandling.Strict`; test `Strict_number_handling_...` kiểm điều đó.

Đây là **hợp đồng thủ công**: các file do con người giữ cho khớp, chỉ chứng minh đúng các ví dụ trong file, không biết gì về ca lỗi khác, header hay trường tuỳ chọn mới. Khi số consumer tăng, bước tiếp theo là consumer-driven contract (Pact: consumer ghi kỳ vọng, provider chạy lại trong CI) hoặc schema registry cho sự kiện Kafka. Lab không có cả hai.

## So với WebApplicationFactory: giống và khác

[Bài .NET về WebApplicationFactory](/articles/dotnet10-webapplicationfactory-integration-test) kiểm pipeline ASP.NET Core trong `TestServer`, cũng không mở cổng. `oneshot` đóng đúng vai đó: cả hai chạy pipeline ứng dụng trong tiến trình test và cùng không chứng minh Kestrel, TLS hay proxy.

Bên .NET, test thay dependency bằng cách cấu hình lại container DI. Bên Rust, `AppState` là struct test tự tạo, nên test chỉ cần truyền thứ khác vào `router(state)`. Cái giá là `router` phải nhận trạng thái từ ngoài, không tự đọc biến môi trường.

## Chạy ví dụ và kiểm chứng

Từ gốc repo, với PostgreSQL lab đã bật:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/34-api-test-contract/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/34-api-test-contract/Cargo.toml
cd examples/toi-noi-rust/34-api-test-contract/dotnet-client
dotnet run  --project src/LedgerClient
dotnet test tests/LedgerClient.Tests
```

`--verify` in (rút gọn):

```text
-- oneshot (không socket) --
POST /v1/entries -> 201 {"entryId":"JE-000001","idempotencyKey":"trip:TN-0001:capture","postings":[...],"replayed":false}
OK   201 và thân bằng file hợp đồng
POST lại cùng key -> 200 replayed=true
POST cùng key khác số tiền -> 409 "https://toi-noi.example/problems/idempotency-conflict"
POST bút toán lệch -> 422 "https://toi-noi.example/problems/unbalanced-entry"
POST JSON hỏng -> 400 "https://toi-noi.example/problems/malformed-json"
không token -> 401; token chỉ đọc -> 403
-- server thật --
GET /v1/accounts/driver:DR-001/balance -> 200 account="driver:DR-001" balanceDong=20400
OK   header date/content-length chỉ có trên socket
-- hợp đồng --
OpenAPI paths: ["/v1/accounts/{id}/balance", "/v1/entries"]
VERIFY OK
```

`cargo test` chạy 20 ca xanh: 9 ca API, 5 ca hợp đồng, 2 ca OpenAPI, 4 ca server thật; chạy lặp lại và song song được, không để sót schema. Bỏ `TOI_NOI_PG_URL` thì cả `--verify` lẫn test **fail** và nói cách bật PostgreSQL, không skip.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là ledger-api thu nhỏ: auth là một extractor HS256 với khoá chia sẻ, không có JWKS hay xoay khoá, rate limit, timeout, request id (bài 33). Hợp đồng thủ công chỉ phủ ba ví dụ (hai thành công, một lỗi), không phủ phiên bản. Snapshot OpenAPI chỉ phát hiện thay đổi, không phán đúng sai. Client HTTP trong test là <code>TcpStream</code> tối thiểu, không có chunked, keep-alive hay TLS. Số đo là của một máy.</p></div>

**Bài tập:** thêm trường tuỳ chọn `memo` (chuỗi, có thể vắng) vào `PostEntryRequest`. Test nào đỏ trước, và bạn phải sửa những file nào để tất cả xanh mà không phá client C# đang chạy?

<details><summary>Xem lời giải</summary><p>Đỏ trước là snapshot OpenAPI: schema có thêm thuộc tính nên diff hiện ngay; duyệt diff rồi cập nhật snapshot. Trường là <code>Option&lt;String&gt;</code> với <code>#[serde(default)]</code>; file mẫu không cần đổi vì trường vắng vẫn hợp lệ, và client C# hiện tại không gửi <code>memo</code> nên vẫn được nhận. Vì có <code>deny_unknown_fields</code>, client gửi <code>memo</code> trước khi server nâng cấp sẽ bị 422: triển khai server trước, client sau. Muốn có bằng chứng thì thêm một file mẫu có <code>memo</code> cho cả hai bên đọc.</p></details>

Bài trước: [bài 33](/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit). Bài tiếp: [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet).

## Tài liệu gốc

- [docs.rs: `tower::ServiceExt`](https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html), trait có `oneshot`.
- [docs.rs: `axum::Router`](https://docs.rs/axum/0.8.9/axum/struct.Router.html) và [`MockConnectInfo`](https://docs.rs/axum/0.8.9/axum/extract/connect_info/struct.MockConnectInfo.html).
- [docs.rs: `insta::assert_json_snapshot`](https://docs.rs/insta/1.49.0/insta/macro.assert_json_snapshot.html).
- [docs.rs: `utoipa`](https://docs.rs/utoipa/6.0.0/utoipa/), sinh OpenAPI từ code.
- [Microsoft Learn: tuỳ chọn `System.Text.Json`, mục Web defaults](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/configure-options).
- [Pact: consumer-driven contract testing](https://docs.pact.io/).

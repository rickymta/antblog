---
slug: rust-validation-problem-details-openapi-utoipa
title: "Validation, Problem Details và OpenAPI với utoipa: hợp đồng API"
summary: "Bài 32: validate thủ công gom mọi lỗi theo trường, trả application/problem+json (RFC 9457), ánh xạ LedgerError sang 422/409, bắt rejection của axum::Json và sinh OpenAPI 3.1 bằng utoipa kèm Swagger UI."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, axum, openapi, utoipa]
language: vi
---

Payment BFF viết bằng C# gọi `POST /v1/entries` để ghi bút toán `trip:TN-0001:capture`. Khi nhận 422, đội BFF cần biết ngay ba điều: lỗi nằm ở trường nào, lỗi thuộc loại nào để rẽ nhánh trong code, và làm sao tra log của đúng lần gọi đó. Một chuỗi tự do như `"bút toán lệch"` đủ cho người đọc, không đủ cho chương trình.

Đây là bài 32 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du) dựng endpoint; bài này biến phản hồi lỗi thành một hợp đồng có cấu trúc và có tài liệu máy đọc được.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Client gửi JSON bị cụt giữa chừng tới handler dùng <code>axum::Json</code>. Handler còn chạy không, và client nhận được gì?</p>
<details><summary>Xem đáp án</summary><p>Handler không chạy: extractor thất bại trước đó và axum tự trả lỗi 400 với <code>text/plain</code>. Client đã quen đọc <code>application/problem+json</code> sẽ gặp một định dạng thứ hai trừ khi ta tự bắt rejection. Lab chạy đúng ca này ở mục về rejection bên dưới.</p></details>
</div>

## Validate ở biên cần gom được gì, và không được làm gì?

Validation ở biên HTTP trả lời một câu hỏi hẹp: request có đúng hình dạng và miền giá trị cơ bản không. Nó phải **gom mọi lỗi** thay vì dừng ở lỗi đầu tiên, để client sửa ba trường trong một lần gửi.

Crate `validator` (bản 0.21.0 khi viết) là lựa chọn quen thuộc, với `#[derive(Validate)]`. Bài chỉ nhắc, lab không dùng: `ValidationErrors` là cây lồng nhau nên để ra đường dẫn `postings[0].account` vẫn phải tự duyệt cây, và hàm `schema` mặc định không chạy khi một trường đã lỗi. Lab chọn **validate thủ công**: ít phụ thuộc hơn, đường dẫn của từng trường nằm ngay trong code.

Lý do lớn hơn nằm ở chỗ đặt bất biến. Quy tắc «bút toán phải cân, không tràn `i64`» **không** thuộc về DTO. Bút toán còn đến từ consumer Kafka (bài 36) và gRPC (bài 35); nếu quy tắc sống trong DTO HTTP, các cửa đó phải chép lại. Vì vậy nó nằm ở `ledger-core`: trường của `JournalEntry` là private, nên không dựng được một bút toán lệch ([bài 13](/articles/rust-double-entry-journal-entry-posting), [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace)). Lớp HTTP chỉ kiểm thứ nó sở hữu: chuỗi không rỗng, độ dài, số dòng tối đa, `amountDong` khác 0.

```rust
// src/validate.rs
/// Kiểm hình dạng của request, trả mọi lỗi tìm được (rỗng nghĩa là đạt).
pub fn field_errors(req: &PostEntryRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    check_text(
        "idempotencyKey".to_string(),
        &req.idempotency_key,
        &mut errors,
    );
    if req.postings.len() > MAX_POSTINGS {
        errors.push(field_error("postings", "too_many", "tối đa 100 dòng"));
    }
    for (i, posting) in req.postings.iter().enumerate() {
        check_text(
            format!("postings[{i}].account"),
            &posting.account,
            &mut errors,
        );
        if posting.amount_dong == 0 {
            errors.push(field_error(
                format!("postings[{i}].amountDong"),
                "zero_amount",
                "số tiền phải khác 0",
            ));
        }
    }
    errors
}
```

Chỉ khi hình dạng đạt, `into_entry` mới gọi `JournalEntry::try_new` và để miền quyết định.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Request đi qua bốn tầng: body JSON, extractor ApiJson, validate theo trường, ledger-core. Mỗi tầng có một nhóm lỗi riêng, đều ra Problem Details."><defs><marker id="rust-validation-problem-details-openapi-utoipa-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">Body JSON</text><text x="36" y="84" class="t-small">từ client C#</text><rect x="240" y="30" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="256" y="60" class="t-b">ApiJson</text><text x="256" y="84" class="t-small">đọc và giải mã</text><rect x="460" y="30" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="476" y="60" class="t-b">validate</text><text x="476" y="84" class="t-small">gom lỗi theo trường</text><rect x="680" y="30" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="696" y="60" class="t-b">ledger-core</text><text x="696" y="84" class="t-small">bút toán cân, key</text><path d="M210 65 L238 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-0-a)"/><path d="M430 65 L458 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-0-a)"/><path d="M650 65 L678 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-0-a)"/><path d="M335 100 L335 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-0-a)"/><path d="M555 100 L555 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-0-a)"/><path d="M775 100 L775 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-0-a)"/><rect x="240" y="150" width="190" height="120" rx="10" class="f-warn-soft s-warn"/><text x="256" y="178" class="t-b">400, 413, 415, 422</text><text x="256" y="200" class="t-small">JSON cụt: 400</text><text x="256" y="218" class="t-small">quá 2 MB: 413</text><text x="256" y="236" class="t-small">thiếu header: 415</text><text x="256" y="254" class="t-small">thiếu trường: 422</text><rect x="460" y="150" width="190" height="120" rx="10" class="f-warn-soft s-warn"/><text x="476" y="182" class="t-b">422</text><text x="476" y="208" class="t-small">validation-failed</text><text x="476" y="228" class="t-small">errors: mọi trường sai</text><text x="476" y="248" class="t-small">traceId để tra log</text><rect x="680" y="150" width="190" height="120" rx="10" class="f-warn-soft s-warn"/><text x="696" y="182" class="t-b">422 hoặc 409</text><text x="696" y="208" class="t-small">lệch, tràn số: 422</text><text x="696" y="228" class="t-small">key lệch nội dung: 409</text><text x="696" y="248" class="t-small">bất biến nằm ở đây</text></svg>
</div>
<figcaption>Hình 1. Mỗi tầng chặn đúng nhóm lỗi của nó; đường thành công đi hết bốn hộp rồi trả 201 hoặc 200.</figcaption>
</figure>

## Problem Details trả lời client những gì?

RFC 9457 (thay thế RFC 7807) định nghĩa media type `application/problem+json` với năm thành phần chuẩn, không thành phần nào bắt buộc:

- `type`: URI tham chiếu định danh loại lỗi. Vắng mặt thì coi là `about:blank`, nghĩa là lỗi không có ý nghĩa gì thêm ngoài mã HTTP, và khi đó `title` nên trùng cụm từ chuẩn của mã.
- `title`: tóm tắt theo loại lỗi, không đổi giữa các lần xảy ra (trừ khi dịch).
- `status`: mã HTTP lặp lại trong thân, chỉ để tiện, và phải trùng mã thật của response.
- `detail`: giải thích riêng cho lần này. RFC khuyên client không phân tích chuỗi này.
- `instance`: URI tham chiếu của lần xảy ra.

Lab không dùng `about:blank`, vì client cần một `type` riêng để rẽ nhánh. RFC khuyến nghị `type` là URI tuyệt đối, cho phép URI không mở được nhưng khuyến khích URI mở ra tài liệu; miền `toi-noi.example` chỉ để minh họa. Schema của lab chặt hơn RFC: năm thành phần cộng `traceId` đều `required`, đó là cam kết riêng của API này.

RFC cho phép thành phần mở rộng, và client phải bỏ qua mở rộng nó không nhận ra. Lab thêm hai, đều không thuộc chuẩn: `errors` (chỉ có khi lỗi validation) và `traceId`. RFC có ví dụ `errors` gồm `detail` và `pointer`, nhưng đó chỉ là ví dụ; lab chọn `field`, `code`, `message` để client rẽ nhánh theo `code`.

```rust
// src/problem.rs (rút gọn: bỏ chú thích schema)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    #[serde(rename = "type")]
    pub type_uri: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FieldError>,
    pub trace_id: String,
}
```

`IntoResponse` cho `Problem` gọi `Json` rồi **ghi đè** `Content-Type` thành `application/problem+json`, vì `Json` đặt `application/json`. Một ca validation thật trong lab:

```text
POST /v1/entries -> 422 application/problem+json
detail: 3 trường không hợp lệ
  idempotencyKey [blank] không được để trống
  postings[0].account [blank] không được để trống
  postings[1].amountDong [zero_amount] số tiền phải khác 0
```

Client rẽ nhánh theo `code` (`blank`, `zero_amount`), không phân tích câu tiếng Việt. Còn `traceId` (32 ký tự hex, dạng W3C trace-id) sinh ngẫu nhiên cho từng lỗi; bài 33 mới có `x-request-id` do middleware cấp và đi vào log, còn nối hai thứ đó là việc của service thật. `instance` ở đây là đường dẫn request, một cách đơn giản hóa: hai lần gọi cùng đường dẫn có cùng `instance`, nên nó chưa định danh được lần xảy ra, và `traceId` gánh phần đó.

Lỗi nội bộ không kèm chi tiết: `LedgerError::InvalidFee` ra 500 với `detail` cố định «xem log với traceId».

## LedgerError đổi thành mã HTTP nào?

Phần dịch từ lỗi miền sang HTTP gói trong một `match` **vét cạn, không có nhánh `_`**:

```rust
// src/error_map.rs (rút gọn: hai nhánh đầu và nhánh 409)
match err {
    LedgerError::EmptyAccountId => unprocessable("empty-account-id", "Mã tài khoản rỗng"),
    LedgerError::Unbalanced { .. } => unprocessable("unbalanced-entry", "Bút toán lệch"),
    LedgerError::IdempotencyConflict { .. } => Problem::new(
        StatusCode::CONFLICT,
        "idempotency-conflict",
        "Idempotency key đã dùng cho nội dung khác",
        err.to_string(),
        instance,
    ),
    // ... các biến thể còn lại, đủ chín biến thể
}
```

| Lỗi miền | HTTP | Vì sao |
|---|---|---|
| `Unbalanced`, `TooFewPostings`, `Overflow`, `EmptyAccountId`, `EmptyIdempotencyKey` | 422 | Cú pháp đúng, nội dung bị từ chối |
| `IdempotencyConflict` | 409 | Cùng key, khác nội dung: xung đột trạng thái ([bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key)) |
| `UnknownEntry` | 404 | Không có tài nguyên |
| `InvalidEntryId` | 400 | Tham số sai dạng |
| `InvalidFee` | 500 | Lỗi của server, ẩn chi tiết |

Bảng này giữ nguyên các mã của [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du), chỉ đổi thân lỗi. `EmptyAccountId` là 422 khi nó đến từ body; mã rỗng trên đường dẫn của `GET` là tham số sai dạng, handler trả 400 như bài 31. Lab chưa có route nào chạm tới `UnknownEntry` và `InvalidEntryId`, nhưng `match` vét cạn vẫn buộc chọn mã cho chúng.

Lợi ích của việc không có nhánh `_` lộ ra khi `ledger-core` thêm biến thể. Tôi thử trên một bản sao: thêm `AccountFrozen` vào `LedgerError` rồi biên dịch lab, kết quả là lỗi chứ không phải một mã 500 âm thầm:

```text
error[E0004]: non-exhaustive patterns: `&LedgerError::AccountFrozen` not covered
  --> src/error_map.rs:28:11
   |
28 |     match err {
   |           ^^^ pattern `&LedgerError::AccountFrozen` not covered
```

Người thêm lỗi buộc phải chọn mã HTTP cho nó trong cùng một thay đổi. Enum lỗi nghiệp vụ riêng của [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te) làm được điều này.

Lần đầu tôi để handler trả `Result<_, Problem>` và `cargo clippy -- -D warnings` từ chối: ``the `Err`-variant is at least 152 bytes`` (`result_large_err`, ngưỡng mặc định 128 byte), vì `Problem` mang năm `String`, một `Vec` và một `u16`. Lab bọc nó trong `ApiError(Box<Problem>)` với `From<Problem>`: `?` vẫn chuyển lỗi tự động, còn đường thành công không gánh 152 byte.

## Rejection của axum::Json có ra Problem Details không?

Mặc định là không. Lab dựng hai server cùng nhận body cụt: server của lab dùng `ApiJson`, server đối chứng dùng `axum::Json` nguyên bản.

```text
body cụt -> 400 application/problem+json
{"type":"https://toi-noi.example/problems/malformed-json","title":"JSON không đúng cú pháp","status":400,"detail":"Failed to parse the request body as JSON: postings: EOF while parsing a list at line 1 column 37","instance":"/v1/entries","traceId":"b3c1f9c875e14e30b1562936fa3f36f8"}
...
axum::Json -> 400 text/plain; charset=utf-8
Failed to parse the request body as JSON: idempotencyKey: EOF while parsing a value at line 1 column 19
```

`ApiJson<T>` là extractor tự viết, bọc `axum::Json<T>` và đổi `Rejection` thành `ApiError`:

```rust
// src/extract.rs
impl<S, T> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let instance = instance_of(&req);
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(ApiJson(value)),
            Err(rejection) => Err(problem_from_rejection(&rejection, &instance).into()),
        }
    }
}
```

`problem_from_rejection` giữ nguyên mã trạng thái axum đã chọn (`rejection.status()`) và chỉ đổi cách trình bày: cú pháp sai là 400 `malformed-json`, thiếu trường, sai kiểu hay trường lạ là 422 `invalid-json-body`, thiếu `Content-Type` là 415, vượt 2 MB của `DefaultBodyLimit` là 413 `unreadable-body`. Đây đúng là bảng mã của bài 31. `get_balance` bọc `Path` theo cùng ý: `%FF` trên đường dẫn ra 400 Problem Details thay vì `text/plain`. Trong axum 0.8, `FromRequest` dùng `async fn` trực tiếp trong trait, không còn `#[async_trait]`.

`detail` ở các ca này là thông điệp tiếng Anh của serde, không phải văn bản ta cam kết ổn định. Hợp đồng ổn định là `type` và `status`. `instance` lấy từ `OriginalUri`, vì `nest("/v1", …)` cắt tiền tố khỏi `req.uri()` trước khi extractor chạy.

## utoipa sinh OpenAPI từ code ra sao?

Cách tiếp cận **code-first**: ba thứ trong code là nguồn của tài liệu. DTO và `Problem` derive `ToSchema`; mỗi handler gắn `#[utoipa::path]` khai báo route và các status hợp đồng cam kết; `#[derive(OpenApi)]` gom lại.

```rust
// src/openapi.rs (rút gọn: bỏ mô tả info)
#[derive(OpenApi)]
#[openapi(
    paths(handlers::post_entry, handlers::get_balance),
    components(schemas(
        PostEntryRequest, PostingDto, EntryResponse,
        BalanceResponse, Problem, FieldError
    )),
    tags((name = "ledger", description = "Ghi bút toán và tra số dư"))
)]
pub struct ApiDoc;
```

Mỗi response lỗi khai báo `content_type = "application/problem+json"` để tài liệu nói đúng media type thật. Version API là một tiền tố: route của phiên bản 1 nằm dưới `Router::nest("/v1", …)`, còn `path` trong `#[utoipa::path]` ghi đầy đủ `/v1/...`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. DTO, handler và Problem là nguồn; ApiDoc gom thành tài liệu; tài liệu được phục vụ dạng JSON và Swagger UI, đồng thời một test đối chiếu tài liệu với route thật."><defs><marker id="rust-validation-problem-details-openapi-utoipa-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="220" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="46" class="t-b">DTO + ToSchema</text><text x="36" y="66" class="t-small">PostEntryRequest, …</text><rect x="20" y="100" width="220" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="126" class="t-b">#[utoipa::path]</text><text x="36" y="146" class="t-small">route, status, body</text><rect x="20" y="180" width="220" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="206" class="t-b">Problem</text><text x="36" y="226" class="t-small">problem+json mọi lỗi</text><rect x="320" y="85" width="220" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="118" class="t-b">ApiDoc</text><text x="336" y="142" class="t-small">#[derive(OpenApi)]</text><text x="336" y="162" class="t-small">ApiDoc::openapi()</text><rect x="620" y="20" width="240" height="60" rx="10" class="f-phys-soft s-phys"/><text x="636" y="46" class="t-b">openapi.json</text><text x="636" y="66" class="t-small">OpenAPI 3.1.0, 2 paths</text><rect x="620" y="100" width="240" height="60" rx="10" class="f-phys-soft s-phys"/><text x="636" y="126" class="t-b">Swagger UI</text><text x="636" y="146" class="t-small">/swagger-ui, vendored</text><rect x="620" y="180" width="240" height="60" rx="10" class="f-warn-soft s-warn"/><text x="636" y="206" class="t-b">Test đối chiếu</text><text x="636" y="226" class="t-small">spec khớp route thật</text><path d="M240 50 L318 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-1-a)"/><path d="M240 130 L318 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-1-a)"/><path d="M240 210 L318 148" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-1-a)"/><path d="M540 110 L618 52" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-1-a)"/><path d="M540 130 L618 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-1-a)"/><path d="M540 150 L618 208" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-validation-problem-details-openapi-utoipa-1-a)"/></svg>
</div>
<figcaption>Hình 2. Tài liệu sinh từ code; test là thứ giữ cho tài liệu không nói dối.</figcaption>
</figure>

`utoipa` 6.0.0 sinh OpenAPI 3.1.0 theo mặc định (changelog của crate có thêm 3.2 ở bản này, nhưng mặc định vẫn là 3.1). `utoipa-swagger-ui` 10.0.2 biến `SwaggerUi` thành một `Router` để `merge` vào app:

```rust
// src/lib.rs
Router::new()
    .nest("/v1", v1)
    .merge(SwaggerUi::new(SWAGGER_PATH).url(OPENAPI_PATH, ApiDoc::openapi()))
    .fallback(handlers::route_not_found)
    .method_not_allowed_fallback(handlers::method_not_allowed)
    .with_state(state)
```

Lab bật feature `vendored` của `utoipa-swagger-ui` nên build không cần tải gì; không có feature đó, theo README của crate, gói Swagger UI được tải về lúc build. Hai `fallback` biến 404 và 405 của chính router thành Problem Details, để lỗi của API cùng một dạng; `method_not_allowed_fallback` vẫn giữ header `Allow` mà RFC 9110 bắt buộc với 405, và một test kiểm điều đó.

Điều tài liệu **không** nói được cũng phải biết. Kiểu `i64` thành `integer` `int64`, `deny_unknown_fields` thành `additionalProperties: false` (lab in ra giá trị `false`), nhưng «khác 0» của `amountDong` không biểu diễn được bằng minimum hay maximum, nên lab chỉ ghi nó trong mô tả (OpenAPI 3.1 có `not` của JSON Schema, lab không dùng). Quy tắc đó vẫn thực thi ở `validate.rs`.

## So với validation và OpenAPI trong .NET 10 thì sao?

[Bài .NET tương ứng](/articles/dotnet10-validation-problem-details-openapi) bật `AddValidation` để Minimal API tự kiểm `Required` và `Range`, trả 400 với `errors` theo trường, rồi `AddOpenApi` cho tài liệu. axum không kiểm DTO, nên ta tự viết `validate` và tự quyết ai gọi nó.

Hai điểm khác là lựa chọn, không phải đúng sai: lab này (cùng bài 31) dùng 422 cho sai hình dạng còn bài .NET dùng 400, và lab phục vụ cả Swagger UI còn bài .NET chỉ xuất JSON. Điều quan trọng là hợp đồng nói rõ và BFF C# đọc đúng. Điểm chung: tài liệu đúng không chứng minh runtime đúng, nên cần test.

## Chạy ví dụ và kiểm chứng

Lab dùng **sổ cái trong bộ nhớ** (`Ledger` của `ledger-core`, bọc `Mutex`) thay vì PostgreSQL, vì bài chỉ bàn tầng HTTP, nên không cần hạ tầng; adapter PostgreSQL đã có ở [bài 26](/articles/rust-repository-unit-of-work-trait-transaction). Mã trạng thái của phần chung không đổi, nhưng thiếu bảng `accounts` nên có hai chỗ khác bài 31: `POST` nhận mọi tài khoản (không có 422 `unknown_account`), và `GET` trả 404 cho tài khoản chưa có bút toán thay vì 200 với số dư 0. `EntryResponse` cũng trả thêm `postings` để client đối chiếu dòng đã ghi. Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/32-validation-openapi/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/32-validation-openapi/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/32-validation-openapi/Cargo.toml
```

Server bind `127.0.0.1:0`, và `src/probe.rs` là client HTTP/1.1 tối thiểu qua TCP thật. Phần OpenAPI của lần chạy (rút gọn):

```text
GET /api-docs/openapi.json -> 200 application/json
openapi: "3.1.0"; info.version: "1.0.0"
paths: ["/v1/accounts/{id}/balance", "/v1/entries"]
schemas: ["BalanceResponse", "EntryResponse", "FieldError", "PostEntryRequest", "PostingDto", "Problem"]
POST /v1/entries 422 content: Some(["application/problem+json"])
Problem.required: ["type","title","status","detail","instance","traceId"]
PostingDto.additionalProperties: false
GET /swagger-ui/ -> 200 text/html
```

Ca lệch và ca xung đột cho:

```text
POST /v1/entries -> 422 application/problem+json
{"type":"https://toi-noi.example/problems/unbalanced-entry","title":"Bút toán lệch","status":422,"detail":"bút toán lệch, tổng bằng -5.100 VND","instance":"/v1/entries","traceId":"bea65a215a9243f4829a58ae8eb0ed8e"}
POST /v1/entries -> 409 application/problem+json
```

`--verify` kiểm 43 điều kiện và in `VERIFY OK (43 điều kiện)`, thoát mã 1 nếu một điều kiện sai. `cargo test` có bốn test đơn vị cho `validate` và mười test tích hợp. Chín test dùng `tower::ServiceExt::oneshot` (không mở cổng): một test kiểm biên 2 MB, một test duyệt từng đường dẫn và phương thức trong tài liệu rồi gọi thật. Test còn lại chạy toàn bộ kịch bản qua TCP.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái trong bộ nhớ mất khi tiến trình dừng. <code>traceId</code> là UUID sinh tại chỗ, chưa phải trace thật; <code>asOf</code> lấy từ đồng hồ process, không phải PostgreSQL như bài 31. Giới hạn body vẫn là 2 MB mặc định. Chưa có xác thực, phân quyền, timeout hay rate limit (bài 33). Chưa có thương lượng nội dung (<code>Accept</code>) hay đa ngôn ngữ cho <code>title</code>. Snapshot tài liệu OpenAPI bằng <code>insta</code> thuộc bài 34.</p>
</div>

## Bài tập

**Bài tập:** `ledger-core` thêm biến thể `LedgerError::AccountFrozen`. (a) Việc gì xảy ra ở lab, và bạn chọn mã HTTP nào? (b) Sau khi sửa, hợp đồng OpenAPI cần đổi gì và test nào bắt nếu bạn quên?

<details><summary>Xem lời giải</summary><p>(a) <code>problem_from_ledger</code> không biên dịch (E0004) tới khi có nhánh mới. 422 hợp lý nếu coi là «nội dung bị từ chối»; 409 hợp lý nếu coi là xung đột trạng thái tài khoản. Chọn một, ghi vào bảng ánh xạ, và dùng một <code>type</code> mới như <code>account-frozen</code>. (b) Thêm dòng <code>(status = ..., body = Problem, content_type = "application/problem+json")</code> vào <code>#[utoipa::path]</code> của <code>post_entry</code> nếu mã đó chưa được liệt kê. Test duyệt tài liệu không bắt việc quên: nó chỉ chắc route tồn tại. Cần một test gọi ca đóng băng và kiểm mã trả về có trong <code>responses</code> của tài liệu.</p></details>

Bài trước: [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du). Bài tiếp: [bài 33](/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit).

## Tài liệu gốc

- [RFC 9457: Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html).
- [docs.rs: crate axum](https://docs.rs/axum/0.8.9/axum/) (`FromRequest`, `JsonRejection`, `OriginalUri`).
- [docs.rs: crate utoipa](https://docs.rs/utoipa/6.0.0/utoipa/) (`ToSchema`, `OpenApi`, `#[utoipa::path]`).
- [docs.rs: crate utoipa-swagger-ui](https://docs.rs/utoipa-swagger-ui/10.0.2/utoipa_swagger_ui/).
- [docs.rs: crate validator](https://docs.rs/validator/0.21.0/validator/).
- [OpenAPI Specification 3.1.0](https://spec.openapis.org/oas/v3.1.0.html).

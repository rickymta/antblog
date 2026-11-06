---
slug: rust-tonic-grpc-get-balance-post-entry-interop-dotnet
title: "gRPC với tonic: GetBalance và PostEntry cho service C# gọi"
summary: "Bài 35: ledger.proto với số tiền int64, sinh code bằng tonic-prost-build, server tonic map LedgerError sang Status, deadline và interceptor, client .NET Grpc.Net.Client chạy thật, và bảng chọn gRPC hay REST."
category: programming
level: intermediate
tags: [rust, toi-noi, ledger, tonic, grpc, dotnet]
language: vi
---

Sau chuyến `TN-0001`, service Payment viết bằng C# cần ghi bút toán thu cước vào sổ cái và hỏi lại số dư của tài xế `DR-001`. Hai bên khác ngôn ngữ, khác nhóm, và hiểu sai đơn vị tiền là loại lỗi đắt nhất. Cái cần là một hợp đồng viết đúng một lần, rồi cả hai phía sinh code từ đó.

Đây là bài 35 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) và cách ghi idempotent của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key). Sổ cái có hai cửa: HTTP ở [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du) và [bài 34](/articles/rust-axum-api-test-oneshot-hop-dong-bff), gRPC ở bài này.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong <code>.proto</code>, trường <code>int64 balance_dong = 2</code> mang giá trị 20400. Client C# in response theo ánh xạ JSON chuẩn của proto3. Kết quả là <code>"balanceDong": 20400</code> hay <code>"balanceDong": "20400"</code>?</p>
<details><summary>Xem đáp án</summary><p>Là chuỗi: <code>"balanceDong": "20400"</code>. Ánh xạ JSON của proto3 đổi <code>int64</code> thành chuỗi vì số JSON trong nhiều ngôn ngữ là <code>double</code>. Dòng chạy thật nằm ở mục client .NET bên dưới.</p></details>
</div>

## Hợp đồng là một file proto, và tiền là `int64`

Hợp đồng nằm trong một file, `proto/ledger/v1/ledger.proto`; Rust và C# cùng sinh code từ nó (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Một file ledger.proto sinh code cho Rust bằng tonic-prost-build và cho C# bằng Grpc.Tools; hai phía nói chuyện qua HTTP/2 với message protobuf."><defs><marker id="rust-tonic-grpc-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="190" height="110" rx="10" class="f-pk-soft s-pk"/><text x="36" y="125" class="t-b">ledger.proto</text><text x="36" y="148" class="t-small">service Ledger</text><text x="36" y="166" class="t-small">GetBalance, PostEntry</text><text x="36" y="184" class="t-small">int64 amount_dong</text><rect x="290" y="20" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="306" y="50" class="t-b">Rust: build.rs</text><text x="306" y="73" class="t-small">tonic-prost-build + protoc</text><text x="306" y="91" class="t-small">đóng gói sẵn (vendored)</text><text x="306" y="109" class="t-small">ra ledger_server, ledger_client</text><rect x="290" y="170" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="306" y="200" class="t-b">C#: Grpc.Tools</text><text x="306" y="223" class="t-small">mục Protobuf trong .csproj</text><text x="306" y="241" class="t-small">sinh trong lúc dotnet build</text><text x="306" y="259" class="t-small">ra Ledger.LedgerClient</text><rect x="620" y="20" width="240" height="110" rx="10" class="f-warn-soft s-warn"/><text x="636" y="50" class="t-b">Server tonic (Rust)</text><text x="636" y="73" class="t-small">interceptor, handler mỏng,</text><text x="636" y="91" class="t-small">PostgreSQL qua LedgerStore</text><rect x="620" y="170" width="240" height="110" rx="10" class="f-warn-soft s-warn"/><text x="636" y="200" class="t-b">Client Grpc.Net.Client</text><text x="636" y="223" class="t-small">gọi qua http:// (h2c)</text><text x="636" y="241" class="t-small">header authorization,</text><text x="636" y="259" class="t-small">x-request-id, deadline</text><path d="M210 135 L286 75" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tonic-grpc-0-a)"/><path d="M210 165 L286 225" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tonic-grpc-0-a)"/><path d="M540 75 L616 75" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tonic-grpc-0-a)"/><path d="M540 225 L616 225" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tonic-grpc-0-a)"/><path d="M740 170 L740 134" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-tonic-grpc-0-a)"/><text x="752" y="156" class="t-small">HTTP/2 + protobuf</text></svg>
</div>
<figcaption>Hình 1. Một file proto, hai phía sinh code. Nét đứt là dây thật: HTTP/2 mang message protobuf.</figcaption>
</figure>

```proto
// proto/ledger/v1/ledger.proto (rút gọn)
service Ledger {
  rpc GetBalance(GetBalanceRequest) returns (GetBalanceResponse);
  rpc PostEntry(PostEntryRequest) returns (PostEntryResponse);
}
message GetBalanceResponse {
  string account = 1;
  int64 balance_dong = 2;                  // đồng, không dùng double
  google.protobuf.Timestamp as_of = 3;
}
message Posting { string account = 1; int64 amount_dong = 2; }  // dương = vào, âm = ra
message PostEntryRequest { string idempotency_key = 1; repeated Posting postings = 2; }
message PostEntryResponse { string entry_id = 1; bool replayed = 2; }
```

Vì sao `int64` mà không `double`? Đồng Việt Nam không có phần lẻ, nên `int64` khớp `Money(i64)` của [bài 3](/articles/rust-kieu-so-tien-vnd-i64-decimal). `double` có 53 bit định trị, nên không phải số nguyên nào lớn hơn 2^53 cũng đi qua nó nguyên vẹn: lab in `9007199254740993` qua `f64` thành `9007199254740992`, lệch một đồng. Số dư thật còn xa mức đó, nhưng hợp đồng nói "số nguyên" thì không ai phải suy luận về độ lớn. Số trường (`= 1`, `= 2`) mới là danh tính của trường trên dây: đổi tên an toàn, đổi hay tái dùng số thì không.

Cái giá của `int64` nằm ở JSON: ánh xạ chuẩn của proto3 viết nó thành chuỗi, nên log, gateway hay công cụ debug đọc qua JSON sẽ thấy `"20400"`.

## Sinh code bằng tonic khi máy không có `protoc`

`protoc` không đi kèm cargo và máy viết bài không có nó. Crate `protoc-bin-vendored` đóng gói sẵn binary theo hệ điều hành; `build.rs` trỏ tới đó. Với tonic 0.14.9, phần prost nằm trong crate `tonic-prost-build` (lúc build) và `tonic-prost` (codec lúc chạy); hàm `tonic_build::compile_protos` của các bản trước (vẫn có ở 0.13.1) không còn trong `tonic-build` 0.14.9; tên tương ứng nay là `tonic_prost_build::compile_protos`.

```toml
# Cargo.toml (rút gọn)
[dependencies]
tonic = "0.14.9"
tonic-prost = "0.14.9"
prost = "0.14.4"
prost-types = "0.14.4"       # google.protobuf.Timestamp

[build-dependencies]
protoc-bin-vendored = "3.3.0"
tonic-prost-build = "0.14.9"
```

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/ledger/v1/ledger.proto");

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // `Config::protoc_executable` thay cho biến môi trường PROTOC (set_var là unsafe trong edition 2024).
    let mut config = tonic_prost_build::Config::new();
    config.protoc_executable(protoc);
    tonic_prost_build::configure().compile_with_config(
        config,
        &["proto/ledger/v1/ledger.proto"],
        &["proto"],
    )?;
    Ok(())
}
```

Code sinh ra nằm trong `OUT_DIR`, không commit; `src/lib.rs` kéo vào bằng `tonic::include_proto!("ledger.v1")` (tên gói trong proto). Từ đó có trait `ledger_server::Ledger` để hiện thực và `ledger_client::LedgerClient` để gọi.

## Server tonic: handler mỏng, nghiệp vụ ở `ledger-core`

Handler chỉ làm ba việc: đổi message sang kiểu miền, gọi kho, đổi kết quả hoặc lỗi sang message và `Status`. Bất biến (ít nhất hai posting, tổng bằng 0) do `JournalEntry::try_new` của `ledger-core` kiểm, không viết lại ở tầng gRPC. Kho là một trait nhỏ với hai adapter: `PgStore` (sqlx, schema `lab_<uuid>`) và `MemoryStore`. `--verify` chạy trên PostgreSQL thật; adapter bộ nhớ dùng trong test và kịch bản deadline.

```rust
// src/service.rs (rút gọn)
async fn post_entry(
    &self,
    request: Request<PostEntryRequest>,
) -> Result<Response<PostEntryResponse>, Status> {
    let ctx = self.context(&request);
    let entry = entry_from_request(request.get_ref()).map_err(|e| ledger_error_status(&e))?;
    let outcome = within_deadline(self.own_deadline, &ctx, async {
        Ok(self.store.post(&entry).await?)
    })
    .await?;
    let mut response = Response::new(PostEntryResponse {
        entry_id: outcome.entry_id,
        replayed: outcome.replayed,
    });
    response.metadata_mut().insert("x-request-id", ascii(&ctx.request_id)?);
    Ok(response)
}
```

Server bind `127.0.0.1:0` rồi đọc cổng thật, nên test chạy song song không đụng nhau:

```rust
// src/server.rs (rút gọn)
let listener = TcpListener::bind("127.0.0.1:0").await.context("bind 127.0.0.1:0")?;
let addr = listener.local_addr()?;
let service = LedgerServer::with_interceptor(ledger, auth_interceptor(token));
Server::builder()
    .add_service(service)
    .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async { let _ = rx.await; })
```

**Interceptor** chạy trước handler trên metadata của request, như middleware ở mức gRPC ([bài 33](/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit) làm việc này cho HTTP). Một closure `FnMut(Request<()>) -> Result<Request<()>, Status>` tự là `Interceptor`, và phải `Clone` vì tonic nhân bản service cho mỗi kết nối. Interceptor của lab kiểm `authorization`, đọc `x-request-id` (thiếu thì sinh UUID) và `grpc-timeout`, rồi đặt vào `extensions` cho handler:

```rust
// src/interceptor.rs (rút gọn: bỏ khai báo hàm, gộp dòng)
move |mut req: Request<()>| {
    match req.metadata().get("authorization").and_then(|v| v.to_str().ok()) {
        Some(v) if v == expected => {}
        Some(_) => return Err(Status::unauthenticated("token không đúng")),
        None => return Err(Status::unauthenticated("thiếu header authorization")),
    }
    let header = |name: &str| {
        req.metadata().get(name).and_then(|v| v.to_str().ok()).map(str::to_owned)
    };
    let ctx = RequestContext {
        request_id: header("x-request-id").unwrap_or_else(|| Uuid::new_v4().to_string()),
        grpc_timeout: header("grpc-timeout"),
    };
    req.extensions_mut().insert(ctx);
    Ok(req)
}
```

Handler trả `x-request-id` lại trong metadata phản hồi để đối chiếu log hai phía. Thiếu hoặc sai token trả `UNAUTHENTICATED` trước khi chạm vào kho.

## `LedgerError` map sang `Status` thế nào?

Client quyết định có gửi lại hay không dựa vào mã của `Status`, nên chọn mã là quyết định về hợp đồng. Lab chọn như bảng dưới và viết `match` không có nhánh `_` (xem [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te)): thêm biến thể vào `LedgerError` thì `status.rs` không biên dịch cho tới khi có người chọn mã.

| Lỗi trong sổ cái | Mã gRPC | Lý do chọn |
|---|---|---|
| `Unbalanced`, `TooFewPostings`, `EmptyIdempotencyKey`, `ZeroAmount` | `INVALID_ARGUMENT` | Sai dù sổ ở trạng thái nào; gửi lại vẫn sai |
| Tài khoản chưa mở (khi ghi) | `FAILED_PRECONDITION` | Request đúng, sổ chưa sẵn sàng; mở tài khoản rồi gửi lại |
| Tài khoản chưa mở (khi đọc số dư) | `NOT_FOUND` | Đọc một thực thể không tồn tại |
| `IdempotencyConflict` | `ALREADY_EXISTS` | Key đã gắn với nội dung khác; như 409 của REST |
| PostgreSQL `40001` hoặc `40P01` | `ABORTED` | Đã rollback vì tranh chấp; gửi lại cả request |
| Hết pool, mất kết nối | `UNAVAILABLE` | Tạm thời, gửi lại với backoff |
| Lỗi SQL, dữ liệu lạ | `INTERNAL` | Lỗi server; không lộ tên bảng ra dây |

Replay không phải lỗi: ghi lại cùng key và cùng nội dung trả `OK` với `replayed = true` (REST dùng 200 thay vì 201), để client thử lại sau timeout nhận kết quả như lần đầu. Và một lỗi có thể mang mã khác nhau theo RPC: tài khoản chưa mở là `FAILED_PRECONDITION` khi ghi, `NOT_FOUND` khi đọc, nên `get_balance` đổi mã trước khi dùng `From<StoreError>`.

```rust
// src/status.rs (rút gọn)
impl From<StoreError> for Status {
    fn from(e: StoreError) -> Status {
        match &e {
            StoreError::Domain(inner) => ledger_error_status(inner),
            StoreError::ZeroAmount(_) => Status::invalid_argument(e.to_string()),
            StoreError::UnknownAccount(_) => Status::failed_precondition(e.to_string()),
            StoreError::Aborted(_) => Status::aborted(e.to_string()),
            StoreError::Unavailable(_) => Status::unavailable(e.to_string()),
            StoreError::Storage(_) => Status::internal("lỗi lưu trữ"),
        }
    }
}
```

Adapter PostgreSQL nhận ra `ABORTED` qua SQLSTATE: `classify_sqlx` coi `40001` (serialization failure) và `40P01` (deadlock) là tranh chấp. **`ABORTED` chỉ được kiểm bằng test đơn vị**: một test nhận SQLSTATE, một test map `StoreError::Aborted` sang mã. Lab ghi ở READ COMMITTED và cập nhật số dư theo thứ tự tài khoản cố định nên không sinh ra xung đột thật; chưa có lời gọi nào nhận `ABORTED` qua dây.

## Deadline: client đặt, server thấy và dừng

Client tonic đặt deadline bằng `request.set_timeout(Duration::from_millis(300))`; tonic ghi nó vào header `grpc-timeout` dạng `300000u` (micro giây). Interceptor đọc header này, và lab giả lập một kho chậm 1200 ms để xem server có dừng không và client nhận mã nào.

Trên tonic 0.14.9, nếu chỉ dựa vào tầng timeout có sẵn, mã hết giờ là `CANCELLED`. Tầng `GrpcTimeout` nằm ở cả server lẫn client của tonic, và `status.rs` của tonic đổi `TimeoutExpired` thành `Status::cancelled("Timeout expired")`. [Tài liệu gRPC về deadline](https://grpc.io/docs/guides/deadlines/) nói client hết hạn thì thấy `DEADLINE_EXCEEDED`, còn server tự hủy call với `CANCELLED`. Ở tonic, mã của server đi thẳng ra dây, và chính client tonic cũng báo `CANCELLED`. Việc dừng vẫn đúng (future của handler bị thả), nhưng client phân biệt hai mã sẽ xử lý sai.

Lab để handler tự đếm ngược: lấy `grpc-timeout`, trừ dự trữ 25 ms, bọc việc gọi kho trong `tokio::time::timeout`. Hết giờ thì future của kho bị thả và client nhận `DEADLINE_EXCEEDED`.

```rust
// src/service.rs
async fn within_deadline<T>(
    own_deadline: bool,
    ctx: &RequestContext,
    work: impl Future<Output = Result<T, Status>>,
) -> Result<T, Status> {
    match ctx.budget().filter(|_| own_deadline) {
        None => work.await,
        Some(budget) => tokio::time::timeout(budget, work)
            .await
            .unwrap_or_else(|_| Err(Status::deadline_exceeded("hết deadline ở server"))),
    }
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Trục thời gian 0 đến 1200 ms: client đặt deadline 300 ms, tầng timeout của tonic ở cả hai phía hết giờ ở 300 ms và trả CANCELLED, handler tự đếm hết giờ ở 275 ms và trả DEADLINE_EXCEEDED, kho chậm cần 1200 ms nên phần còn lại bị bỏ."><rect x="40" y="26" width="204" height="22" rx="4" class="f-pk-soft s-pk"/><text x="260" y="42" class="t-small">client đặt grpc-timeout = 300 ms (dây mang 300000u)</text><rect x="40" y="66" width="204" height="22" rx="4" class="f-warn-soft s-warn"/><text x="260" y="82" class="t-small">tầng timeout của tonic, hai phía: hết giờ ở 300 ms, trả CANCELLED</text><rect x="40" y="106" width="187" height="22" rx="4" class="f-phys-soft s-phys"/><text x="243" y="122" class="t-small">handler tự đếm (300 trừ 25 = 275 ms): trả DEADLINE_EXCEEDED</text><rect x="40" y="146" width="187" height="22" rx="4" class="f-phys-soft s-phys"/><rect x="227" y="146" width="629" height="22" rx="4" class="f-none s-phys dash"/><text x="240" y="190" class="t-small">kho chậm cần 1200 ms: future bị thả ở 275 ms, phần nét đứt không bao giờ chạy</text><path d="M40 210 L856 210" class="f-none s-line" stroke-width="2"/><path d="M40 204 L40 216 M227 204 L227 216 M244 204 L244 216 M856 204 L856 216" class="f-none s-line" stroke-width="2"/><text x="36" y="236" class="t-small">0</text><text x="203" y="236" class="t-small">275</text><text x="250" y="236" class="t-small">300</text><text x="826" y="236" class="t-small">1200 ms</text></svg>
</div>
<figcaption>Hình 2. Hai bộ đếm cho một deadline. Handler phải hết giờ trước tầng timeout của tonic thì mới chọn được mã trả về.</figcaption>
</figure>

Chạy thật cho các số sau (rút gọn; thời gian đo trên máy viết bài, không phải SLA):

```text
== Deadline, chỉ tầng timeout của tonic (kho cần 1200 ms, client cho 300 ms)
deadline 300 ms        -> Cancelled, client chờ 302 ms
grpc-timeout handler thấy -> Some("300000u")
handler bị hủy giữa chừng: 1, chạy hết: 0
đối chứng, deadline 5 s -> Ok
client tonic, peer không trả lời -> Cancelled
== Deadline, handler tự đếm ngược
deadline 300 ms        -> DeadlineExceeded, client chờ 278 ms
handler bị hủy giữa chừng: 1, chạy hết: 0
```

Dòng "đối chứng" gọi lại request đó với deadline 5 s trên cùng server chậm và nhận `OK`: lỗi ở lần đầu là do deadline, không do server hỏng. Dòng "peer không trả lời" gọi vào một socket không bao giờ đáp, nên `CANCELLED` ở đó do chính client tonic sinh.

Handler tự đếm là một cuộc đua, không phải bảo đảm: nó bắt đầu sau cả hai bộ đếm của tonic, nên nếu scheduler trễ hơn khoảng dự trữ thì tonic thắng và client lại thấy `CANCELLED`. Điều đó đã xảy ra một lần khi dự trữ còn 10 ms và máy đang bận biên dịch. Với 25 ms, 36 lần `--verify` (12 tiến trình song song, ba vòng) đều ra `DEADLINE_EXCEEDED`, nhưng lần chậm nhất là 297 ms, chỉ còn dư 3 ms. Vì vậy dòng này có thể đỏ trên máy quá tải, và trên mạng thật dự trữ còn phải trừ độ trễ đường truyền. Việc dừng không phụ thuộc cuộc đua này, chỉ mã trả về là có.

Kịch bản deadline chạy trên kho bộ nhớ. Với PostgreSQL, thả future là thả `Transaction` chưa commit, tức rollback như [bài 26](/articles/rust-repository-unit-of-work-trait-transaction); lab này không dựng lại ca đó.

## Client .NET thật gọi vào server Rust

Client C# nằm trong `dotnet-client/`, dùng `Grpc.Net.Client` 2.85.0, `Grpc.Tools` 2.85.0 và `Google.Protobuf` 3.36.2. Chỗ then chốt là một dòng trong `.csproj`: nó trỏ tới đúng file proto mà server Rust dùng, nên không có bản sao hợp đồng nào để lệch nhau.

```xml
<!-- dotnet-client/LedgerClient.csproj (rút gọn) -->
<ItemGroup>
  <Protobuf Include="../proto/ledger/v1/ledger.proto" ProtoRoot="../proto" GrpcServices="Client" />
</ItemGroup>
```

```csharp
// dotnet-client/Program.cs (rút gọn)
// Server lab không có TLS: địa chỉ `http://` chạy HTTP/2 không mã hóa (h2c).
using var channel = GrpcChannel.ForAddress(args[0]);
var client = new Ledger.LedgerClient(channel);

var headers = new Metadata
{
    { "authorization", $"Bearer {args[1]}" },
    { "x-request-id", "req-csharp-0001" },
};
var balance = await client.GetBalanceAsync(
    new GetBalanceRequest { Account = "driver:DR-001" }, headers, deadline: Deadline());
Console.WriteLine($"GetBalance driver:DR-001: balanceDong={balance.BalanceDong}");
Console.WriteLine($"JSON proto3: {JsonFormatter.Default.Format(balance)}");
```

Trên .NET SDK 10.0.401, địa chỉ `http://` chạy HTTP/2 không TLS mà không cần `AppContext` switch nào. Công tắc `System.Net.Http.SocketsHttpHandler.Http2UnencryptedSupport`, theo [tài liệu Microsoft](https://learn.microsoft.com/aspnet/core/grpc/troubleshoot), chỉ cần cho .NET Core 3.x; từ .NET 5 chỉ cần `Grpc.Net.Client` 2.32.0 trở lên. Ở production, kênh nội bộ nên là TLS hoặc mTLS.

`cargo run -- --with-dotnet` khởi động server Rust trên PostgreSQL, lấy cổng thật rồi chạy `dotnet run` với cổng đó. Output thật; dòng `[C#]` do tiến trình .NET in:

```text
server Rust nghe ở http://127.0.0.1:52376; chạy: dotnet run --project dotnet-client -- http://127.0.0.1:52376
[C#] PostEntry lần 1: entryId=JE-000001 replayed=False x-request-id=req-csharp-0001
[C#] PostEntry lần 2: entryId=JE-000001 replayed=True x-request-id=req-csharp-0001
[C#] GetBalance driver:DR-001: balanceDong=20400
[C#] JSON proto3: { "account": "driver:DR-001", "balanceDong": "20400", "asOf": "2026-11-06T13:53:14.729818Z" }
[C#] không có authorization: Unauthenticated
[C#] bút toán lệch tổng: InvalidArgument
[C#] cùng key, khác nội dung: AlreadyExists
```

Khớp với phía Rust: `JE-000001`, replay ở lần hai, số dư tài xế 20.400 đồng, `x-request-id` đi và về, ba mã lỗi đúng bảng. Dòng JSON trả lời câu hỏi mở bài: `"balanceDong": "20400"`, một chuỗi. Cổng `52376` đổi mỗi lần chạy.

Lệnh đó còn gọi hai server chậm với deadline 300 ms, ba lần trên cùng kết nối:

```text
== Deadline từ client C#, chỉ tầng timeout của tonic
[C#] lần 1: DeadlineExceeded, chi tiết "", 331 ms
[C#] lần 2: Cancelled, chi tiết "Timeout expired", 310 ms
[C#] lần 3: Cancelled, chi tiết "Timeout expired", 303 ms
== Deadline từ client C#, handler tự đếm
[C#] lần 1: DeadlineExceeded, chi tiết "", 337 ms
[C#] lần 2: DeadlineExceeded, chi tiết "hết deadline ở server", 284 ms
[C#] lần 3: DeadlineExceeded, chi tiết "hết deadline ở server", 278 ms
```

`Grpc.Net.Client` cũng tự đếm deadline: chi tiết rỗng là mã do nó sinh, chi tiết có chữ là mã server gửi. Lần 1 chậm hơn (nhiều khả năng do mở kết nối) nên bộ đếm của client thắng; từ lần 2, `CANCELLED` của server về trước. Nếu không có handler tự đếm, code C# chỉ bắt `DeadlineExceeded` sẽ bỏ sót phần lớn ca hết giờ. Thời gian đo trên máy viết bài, không phải SLA.

## gRPC hay REST trong Tới Nơi?

Sổ cái mở cả hai cửa. Quy tắc ở đây: nội bộ giữa service thì gRPC, từ ngoài hoặc từ trình duyệt thì REST.

| Tình huống | Chọn | Vì sao |
|---|---|---|
| Payment hoặc Admin BFF (C#) gọi `GetBalance`, `PostEntry` | gRPC | Code sinh hai phía từ một proto, `int64` tường minh, có deadline |
| Trình duyệt hoặc ứng dụng web gọi trực tiếp | REST | Trình duyệt không nói gRPC thuần (cần gRPC-Web) |
| Webhook từ VíXanh | REST | Đối tác quyết định giao thức; xem [bài 38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung) |
| Cần OpenAPI cho đối tác đọc tài liệu | REST | Hợp đồng ở [bài 32](/articles/rust-validation-problem-details-openapi-utoipa); tiền trong JSON phải quy ước rõ |
| Cache HTTP, kiểm bằng `curl` khi sự cố | REST | Hạ tầng web hiểu sẵn |

Cái giá của gRPC là công cụ: khó đọc payload nhị phân bằng mắt, và phải quản lý phiên bản proto giữa các nhóm. Nếu chỉ một nhóm gọi, vài lần mỗi giờ, REST với OpenAPI vẫn đơn giản hơn.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `infra/` và biến `TOI_NOI_PG_URL` theo `infra/README.md`; thiếu biến thì lab thoát mã 1 và nói cách bật.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml
# cần .NET SDK 10; lần đầu dotnet tải gói NuGet về
cargo run  --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml -- --with-dotnet
```

`--verify` khởi động server trên `127.0.0.1:0`, gọi bằng client tonic. Phần đầu output:

```text
== Số tiền trên dây
9007199254740993 qua f64 thành 9007199254740992 (lệch 1 đồng); PostEntryRequest 3 posting = 96 byte protobuf
== Server tonic + PostgreSQL (schema lab riêng), client tonic
không token            -> Unauthenticated
token sai              -> Unauthenticated
PostEntry lần 1        -> JE-000001 replayed=false
PostEntry lần 2        -> JE-000001 replayed=true
GetBalance             -> customer=-25500 platform=5100 driver=20400
cùng key, khác tiền    -> AlreadyExists
lệch tổng              -> InvalidArgument
ghi tài khoản chưa mở  -> FailedPrecondition
đọc tài khoản chưa mở  -> NotFound
x-request-id           -> server thấy req-lab-0001, trả lại req-lab-0001
dọn schema lab_bafcc5fad1f84f2bb8ac8d14df6dbc99: đã xoá
```

Tiếp theo là bảng lỗi sang `Status`, hai khối deadline, rồi dòng `VERIFY OK`; tên schema đổi mỗi lần chạy.  `cargo test` chạy 15 test: 8 test đơn vị (parse `grpc-timeout`, ngân sách deadline, interceptor, bảng mã, SQLSTATE) và 7 test tích hợp (cùng một kịch bản trên kho bộ nhớ và PostgreSQL, ba ca deadline, tám lời gọi đồng thời cùng key, cộng phép thử `int64` qua `f64` không cần mạng). Client C# không nằm trong `--verify` hay `cargo test` vì cần .NET SDK; `--with-dotnet` chạy nó và thoát mã khác 0 nếu `dotnet` lỗi.

Test đồng thời dạy một điều: mã bút toán không nhất thiết là `JE-000001`. Cột identity của PostgreSQL cấp số cho cả lần `INSERT` thua `ON CONFLICT`, nên bút toán thắng có thể mang số 2 trở lên. Test chỉ khẳng định tám lời gọi thấy cùng một mã và bảng có một dòng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Token là chuỗi tĩnh so sánh bằng <code>==</code> (không hằng thời gian), không phải JWT hay mTLS; kênh là HTTP/2 không TLS trên <code>127.0.0.1</code>. Chưa có reflection, health check, chi tiết lỗi có cấu trúc (<code>google.rpc.ErrorInfo</code>), retry policy, streaming hay kiểm tương thích proto giữa các phiên bản. <code>as_of</code> là giờ server khi trả lời, chưa phải mốc của truy vấn số dư. Deadline được kiểm trên kho bộ nhớ chậm giả lập, không phải truy vấn PostgreSQL thật; <code>ABORTED</code> chưa từng đi qua dây. Nhiều nhóm cần kho proto riêng và quy trình đổi hợp đồng.</p>
</div>

**Bài tập:** `PostEntry` vào tài khoản chưa mở hiện trả `FAILED_PRECONDITION`. Một đồng nghiệp muốn đổi thành `NOT_FOUND`. Nêu một lập luận cho mỗi bên, và liệt kê những chỗ phải sửa nếu đổi.

<details><summary>Xem lời giải</summary>
<p>Cho <code>NOT_FOUND</code>: tài khoản được nhắc tới không tồn tại, và đó là mã người ta đoán đầu tiên. Cho <code>FAILED_PRECONDITION</code>: request đúng, lỗi nằm ở trạng thái sổ, và cách sửa là mở tài khoản rồi gửi lại y nguyên; <code>NOT_FOUND</code> dễ bị hiểu là "đừng gọi lại". Nếu đổi, phải sửa nhánh <code>UnknownAccount</code> trong <code>src/status.rs</code>, test <code>each_error_gets_the_documented_code</code>, <code>check_all</code> trong <code>src/main.rs</code>, <code>assert_flow</code> trong <code>tests/grpc.rs</code> và báo mọi client (kể cả code C# bắt <code>RpcException</code>): mã trạng thái là một phần của hợp đồng, đổi nó là thay đổi phá vỡ.</p>
</details>

Bài trước: [bài 34](/articles/rust-axum-api-test-oneshot-hop-dong-bff). Bài tiếp: [bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema).

## Tài liệu gốc

- [Crate tonic](https://docs.rs/tonic): server, client, `Interceptor`, `Status`, `Request::set_timeout`.
- [Crate tonic-prost-build](https://docs.rs/tonic-prost-build) và [protoc-bin-vendored](https://docs.rs/protoc-bin-vendored).
- [Crate prost](https://docs.rs/prost) và [Cargo: build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
- [gRPC over HTTP/2: header `grpc-timeout`](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md), [deadline](https://grpc.io/docs/guides/deadlines/) và [mã trạng thái gRPC](https://grpc.io/docs/guides/status-codes/).
- [Protocol Buffers: ánh xạ JSON](https://protobuf.dev/programming-guides/json/).
- [gRPC trên .NET: Grpc.Net.Client](https://learn.microsoft.com/aspnet/core/grpc/client) và [gọi service không TLS](https://learn.microsoft.com/aspnet/core/grpc/troubleshoot).

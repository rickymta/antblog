---
slug: dotnet10-tong-hop-service-dat-chuyen
title: "Bài tổng hợp: service đặt chuyến từ domain tới vận hành có kiểm chứng"
summary: "Bài 38: tổng kết một lát cắt của Tới Nơi dùng Clean Architecture, CQRS/Mediator, Repository/UoW và PostgreSQL; phân biệt điều đã test với phần cần ghép để thành sản phẩm."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 38** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hoàn thành 38 bài có đồng nghĩa với việc code lab đã là sản phẩm đặt xe production hoàn chỉnh không?</p>
<details><summary>Xem đáp án</summary><p>Không. Loạt bài hoàn tất một lộ trình và service mẫu trong phạm vi đã nêu; nhiều yêu cầu về sản phẩm và hạ tầng vẫn cần được thiết kế và kiểm tra riêng.</p></details>
</div>

## Quay lại câu hỏi đầu tiên: khách phải trả bao nhiêu?

Bài đầu chỉ in giá 5 km trong terminal. Đến đây cùng quy tắc đi qua JSON, validation, identity, command handler, domain, repository, transaction và response có thể đọc lại. Ta đã thêm cách kiểm chứng từng ranh giới thay vì để một chuỗi class chỉ đẹp trên sơ đồ.

Service hỗ trợ tạo chuyến theo key, đọc theo owner và nhận chuyến theo version. Cước, tiền tệ và thời gian có kiểu dữ liệu và hợp đồng. Hai tài xế không thể cùng thắng một lượt accept; request lặp không tạo nhiều Trip/Outbox; domain không biết HTTP/EF.

Code cuối ở 31-38-trip-service là một lát cắt học tập, không phải tuyên bố đã dựng mọi luồng dịch vụ ride, food và parcel của Tới Nơi. Đọc README về phạm vi trước khi dùng nó làm nền cho một service thật.

## Các trách nhiệm đã ghép vào đâu?

Domain giữ invariant và event. Application có command/query, validator và port. Mediator nhỏ điều phối các behavior và handler trong cùng scope. Infrastructure ánh xạ giữa aggregate và row, giữ original version, query DTO và commit Outbox cùng dữ liệu. API xác thực principal và chuyển lỗi sang HTTP có cấu trúc.

Migration có tệp thực thi riêng, readiness không tự áp schema, probe live không phụ thuộc DB. CI có PostgreSQL thật và artifact image; cloud spec nhận secret/identity từ môi trường. Telemetry nối request tới handler/commit và giữ dimension của metric ở mức hữu hạn.

Không pattern nào thay thế được tất cả phần còn lại. DDD không thay auth, UoW không thay broker, optimistic concurrency không thay idempotency và retry không tự tạo tính đúng của một khoản thanh toán.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Đi từ hành vi và bằng chứng tới cấu trúc, không coi danh sách pattern là mục tiêu cuối."><defs><marker id="cs38-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Ý định khách</text><text x="34" y="123" class="t-small">Tạo / đọc / nhận chuyến</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs38-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Service có ranh giới</text><text x="319" y="123" class="t-small">Domain + pipeline + commit</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs38-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Bằng chứng</text><text x="604" y="123" class="t-small">Test + artifact + quan sát</text></svg>
</div>
<figcaption>Hình 1. Đi từ hành vi và bằng chứng tới cấu trúc, không coi danh sách pattern là mục tiêu cuối.</figcaption>
</figure>

## Sổ bằng chứng thay vì một nhãn “done” chung

Suite cuối có 20 test: HTTP/JWT chạy với PostgreSQL, validation, duplicate, ownership, accept conflict, rehydrate, cancellation, health, ranh giới dependency và telemetry. OCI archive đã tạo bằng SDK; Bicep đã compile. Các bài trước còn có bài thực hành unit/integration chuyên sâu và báo cáo benchmark thật.

Chưa chạy GitHub Actions trên remote vì chưa push; chưa dựng Docker engine trên máy này; chưa deploy AWS/Azure. Không lấy compile template hoặc TestServer làm bằng chứng TLS/proxy/cloud role đã đúng. Không lấy hai lần publish giả lập ở bài 26 làm bằng chứng broker production đã cấu hình.

Chạy lại test là một phần của quá trình thay đổi. Khi thêm trường vào command hoặc đổi rule, cập nhật fingerprint, validation, domain, test, schema và hợp đồng response theo phạm vi; không sửa mỗi DTO rồi coi mọi ranh giới đã tự hiểu.

## Bước tiếp theo để thành sản phẩm

Cần bổ sung vòng đời đầy đủ, tài xế đủ điều kiện, định tuyến và giá theo vùng, huỷ và hoàn tiền, ledger và đối soát. Các thành phần Python (fraud, OCR), Rust và Go sẽ chia sẻ hợp đồng integration theo context, không truy cập chéo bảng nghiệp vụ.

Outbox hiện lưu intent; phải ghép thêm delivery worker có claim/lease, retry, broker confirm và consumer Inbox theo yêu cầu thật. Cần test crash/recovery, dữ liệu lớn, backup/restore, capacity, secret rotation, identity mapping và các trường hợp thất bại liên service.

PostgreSQL là nền đầu tiên, còn MySQL/MongoDB được đưa vào khi workload hoặc mô hình dữ liệu có bằng chứng phù hợp. Giữ nguyên nguyên tắc đã nêu ở hồ sơ Tới Nơi: không có ngưỡng concurrency chung nào buộc phải đổi database chỉ vì vacuum.

**Bài tập tổng hợp:** thêm loại dịch vụ giao bưu kiện (parcel). Hãy liệt kê các phần hợp đồng bị ảnh hưởng trước khi viết class. Khác biệt nào thuộc pricing domain, khác biệt nào thuộc query/UI, khác biệt nào làm thay đổi fingerprint và integration schema?

<details><summary>Gợi ý tự đánh giá</summary><p>Bắt đầu từ yêu cầu và invariant, định nghĩa input, response và phiên bản event, cập nhật idempotency scope và rule giá, rồi chọn pattern như Strategy khi có nhu cầu thay thuật toán. Không sinh toàn bộ tầng/class trước khi biết hành vi cần bảo vệ.</p></details>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Phân biệt rõ bằng chứng đã có với các gate triển khai còn cần thực hiện."><defs><marker id="cs38-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Đã kiểm local</text><text x="34" y="123" class="t-small">PostgreSQL + HTTP tests</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Đã đóng gói/compile</text><text x="319" y="123" class="t-small">OCI archive + Bicep</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Cần kiểm môi trường</text><text x="604" y="123" class="t-small">Cloud / broker / tải thật</text></svg>
</div>
<figcaption>Hình 2. Phân biệt rõ bằng chứng đã có với các gate triển khai còn cần thực hiện.</figcaption>
</figure>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xoá database; JWT dùng khoá ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự bỏ qua khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, điều kiện tiên quyết và phần chưa triển khai thật.

### README.md — 31-38-trip-service/README.md

````markdown
# Tới Nơi Trips — service mẫu cho bài 31–38

Bốn tầng `Domain → Application ← Infrastructure`, API là composition root. Domain/Application không tham chiếu EF hoặc ASP.NET. Dispatcher nhỏ minh họa Mediator trong process với typed handler và pipeline; không phải broker hay thư viện thay thế đầy đủ.

Các endpoint:

| Endpoint | Quyền | Hành vi |
| --- | --- | --- |
| POST /trips | customer | Tạo chuyến với `Idempotency-Key`; body `{"distanceKm":5}` |
| GET /trips/{id} | customer | Chỉ chủ sở hữu đọc; khác chủ/không có trả 404 |
| POST /trips/{id}/accept | driver | Body `{"expectedVersion":0}`; driver ID lấy từ sub đã xác thực |
| GET /health/live | Probe | Không truy vấn database |
| GET /health/ready | Probe | Có kết nối và không còn migration pending |

`sub` được ánh xạ thành Guid trong lab. Provider thực phải có mapping đáng tin cậy theo hợp đồng danh tính; không lấy owner/driver ID từ body. Policy driver chưa kiểm hồ sơ/khu vực/eligibility của sản phẩm thật.

## Kiểm chứng local

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` như [bài 21](../21-efcore-postgres/README.md). Từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Mỗi ca tạo và dọn schema `svc_<GUID>` riêng, không xóa database. JWT handler thật nhận khóa ngẫu nhiên qua test fixture, không dùng credential production hay gọi identity provider ngoài. Test không skip nếu thiếu PostgreSQL.

## Chạy host và migration có chủ đích

Đặt `ConnectionStrings__Trips`, `Auth__Authority` (HTTPS), `Auth__Audience` trong môi trường. Production yêu cầu database `SslMode=VerifyFull`. Không đưa credential vào source. Dùng role migration riêng và duyệt SQL trước; API không tự migrate khi startup.

```bash
dotnet run --project examples/toi-noi-dotnet/31-38-trip-service/Migrator
dotnet run --project examples/toi-noi-dotnet/31-38-trip-service/Api -- --urls http://127.0.0.1:5200
```

Create và replay hợp lệ trả 201 với cùng TripId/Fare; dùng lại key nhưng đổi input trả 409. Một SaveChanges commit Trip, request receipt và Outbox cùng transaction local. Query DTO không qua aggregate. UoW clear domain events chỉ sau commit thành công.

Telemetry dùng ActivitySource/Meter `ToiNoi.Trips`; bật `Telemetry__Export=true` và cấu hình OTLP endpoint được tin cậy để xuất. Mặc định không gửi telemetry ra ngoài. Test kiểm traceparent đi tới handler/commit và metric chỉ gắn tag operation có tập giá trị hữu hạn.

## Phạm vi

Service minh họa create/read/accept, chưa là sản phẩm đặt xe đầy đủ. Outbox lưu intent; broker, dispatcher đa worker, Inbox phía service nhận và chính sách vận hành giao hàng cần được ghép theo bài 26. Không có token issuer dev, khóa cố định, thu tiền thật hay dữ liệu OCR. Các bài 31–38 giải thích lần lượt cùng code cuối của lab; chúng không phải các bản release độc lập.
````

Bài trước: [bài 37](/articles/dotnet10-opentelemetry-traces-metrics-logs). Đây là bài cuối của lộ trình 38 bài; tiếp tục dùng các tình huống Tới Nơi để học những nhánh ngôn ngữ/database liên quan.

## Tài liệu gốc

- [.NET architecture guides](https://dotnet.microsoft.com/en-us/learn/dotnet/architecture-guides).
- [Distributed data](https://learn.microsoft.com/en-us/dotnet/architecture/cloud-native/distributed-data).

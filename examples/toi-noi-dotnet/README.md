# C# và .NET 10 qua dự án Tới Nơi

Khung biên tập từ cơ bản tới nâng cao. Mỗi bài có câu hỏi mở đầu, sơ đồ/hình minh họa giải thích cơ chế, ví dụ chạy được, lỗi thường gặp, bài tập có đáp án và nguồn chính thức. Tách rõ code minh họa với giải pháp production. Không tạo link cho bài chưa viết.

Đối tượng: người mới biết sử dụng máy tính và terminal; không yêu cầu biết C#. Nhánh web dùng ASP.NET Core 10, nhánh dữ liệu dùng EF Core 10. Không dùng tên “.NET Framework 10”: .NET Framework là dòng sản phẩm khác.

## 01 · Nền tảng C#

01. C# và .NET 10: từ source code đến chương trình tính cước — **đã viết**
02. Biến, kiểu dữ liệu và decimal: biểu diễn tiền và khoảng cách — **đã viết**
03. Điều kiện, vòng lặp và phương thức: tách quy tắc tính cước — **đã viết**
04. Chuỗi, ngày giờ và văn hóa: hiển thị đúng, lưu trữ đúng — **đã viết**
05. Debug và exception: lần theo một báo giá sai — **đã viết**

## 02 · Mô hình hóa nghiệp vụ

06. Class, property và encapsulation: bảo vệ trạng thái chuyến — **đã viết**
07. Record, struct, equality và immutability: thiết kế Money, Quote — **đã viết**
08. Interface, composition và dependency injection: thay chính sách cước — **đã viết**
09. Generics và collections: quản lý chuyến và tài xế — **đã viết**
10. LINQ và deferred execution: truy vấn dữ liệu trong bộ nhớ — **đã viết**
11. Nullable reference types và pattern matching: xử lý trạng thái thiếu — **đã viết**

## 03 · Bất đồng bộ và kiểm thử

12. Task, async/await và CancellationToken: gọi dịch vụ báo giá — **đã viết**
13. Concurrency và thread safety: tránh nhận trùng chuyến — **đã viết**
14. Unit test và test double: kiểm thử chính sách cước — **đã viết**
15. Integration test: kiểm chứng ranh giới hệ thống — **đã viết**

## 04 · ASP.NET Core 10

16. HTTP và Minimal API: xuất bản endpoint báo giá — **đã viết**
17. Validation, Problem Details và OpenAPI: hợp đồng API — **đã viết**
18. Middleware, DI lifetime, cấu hình và logging — **đã viết**
19. Authentication và authorization: khách, tài xế, điều phối — **đã viết**
20. API integration test với WebApplicationFactory — **đã viết**

## 05 · Dữ liệu và độ tin cậy

21. EF Core 10 và PostgreSQL: mapping, migration, truy vấn — **đã viết**
22. Tracking, projection và N+1: đọc SQL trước khi tối ưu — **đã viết**
23. Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn — **đã viết**
24. BackgroundService và hàng đợi: tác vụ sau khi đặt chuyến — **đã viết**
25. Cache, timeout, retry và rate limiting: giữ API ổn định — **đã viết**
26. Outbox và xử lý thông điệp trùng: nối dịch vụ có kiểm soát — **đã viết**

## 06 · Nâng cao và vận hành

27. Allocation, GC, IDisposable và async disposal — **đã viết**
28. BenchmarkDotNet, profiling và Span: tối ưu có số đo — **đã viết**
29. C# 14 trong thực tế: extension members, field và tính năng mới — **đã viết**
30. DDD: bounded context, aggregate, value object và invariant của chuyến — **đã viết**
31. Clean Architecture: chia Domain, Application, Infrastructure, API — dự kiến
32. CQRS và Mediator: command/query handler và pipeline behavior — dự kiến
33. Repository và Unit of Work với EF Core: aggregate, tracking và commit — dự kiến
34. Ghép use case đặt chuyến: validation, transaction, domain event và kiểm thử — dự kiến
35. Container và CI/CD: build, test, publish ứng dụng .NET 10 — dự kiến
36. Triển khai AWS và Azure: cấu hình, secrets và health check — dự kiến
37. OpenTelemetry: lần theo một yêu cầu chậm — dự kiến
38. Bài tổng hợp: dịch vụ đặt chuyến từ code tới vận hành — dự kiến

## Quy ước xuyên suốt

SDK .NET 10, target net10.0, C# mặc định của target (C# 14). Dùng bản vá SDK còn được hỗ trợ; không đặt LangVersion=latest. Tiền VND dùng decimal, thời điểm lưu UTC, dữ liệu test giả. Bắt đầu từ console tính cước, phát triển thành API đặt chuyến; các bài nhập môn tập trung vào từng cơ chế trước khi ghép thành service theo kiến trúc đích.

## Kiến trúc đích đã thống nhất

Tất cả backend service chính dùng DDD + Clean Architecture + CQRS + Mediator + Repository + Unit of Work. Hồ sơ chuẩn nằm trong bài `api/seed/content/du-an-mau-toi-noi.md`, mục “Kiến trúc đích cho các backend service chính”. Các bài API/dữ liệu ban đầu là bước học từng phần; đến bài 30–34 sẽ tổ chức lại thành service hoàn chỉnh, không xem demo một file là cấu trúc production.

Command thao tác qua aggregate và repository; Unit of Work commit transaction cục bộ. Query dùng projection/abstraction đọc phù hợp. Mediator điều phối trong tiến trình; CQRS không mặc định yêu cầu event sourcing hoặc hai database. Các pattern khác chọn theo bài toán cụ thể và giải thích trade-off. Mỗi bài có hình minh họa theo concept Design Pattern/Database, caption, mô tả truy cập và code kiểm chứng.

## Liên kết các loạt bài đa ngôn ngữ và database

C#/.NET phục vụ các backend nghiệp vụ chính; Python cho fraud scoring, OCR và phân tích/dự đoán; Rust cho sổ cái/đối soát và thành phần cần kiểm soát tài nguyên; Go cho luồng tiếp nhận vị trí, fan-out và worker nhiều tác vụ đồng thời. Đây là lựa chọn thiết kế của Tới Nơi, cần kiểm chứng theo workload. Các loạt dùng chung hợp đồng API/event và thuật ngữ, không chia sẻ quyền ghi dữ liệu nội bộ.

PostgreSQL là database chủ đạo giai đoạn đầu. Các chặng sau khảo sát MySQL/InnoDB và MongoDB theo workload/mô hình dữ liệu, có benchmark và quy trình migration; không khẳng định concurrency cao bắt buộc đổi database vì vacuum. Tham chiếu phần “Đa ngôn ngữ theo trách nhiệm service” và “Hành trình PostgreSQL, MySQL và MongoDB” trong hồ sơ Tới Nơi để giữ nội dung thống nhất. Loạt .NET hiện tập trung PostgreSQL/EF Core; adapter lưu trữ khác chỉ bổ sung ở bài phù hợp.

## Chạy ba bài đầu

```bash
dotnet run --project examples/toi-noi-dotnet/01-csharp-va-dotnet
dotnet run --project examples/toi-noi-dotnet/02-bien-kieu-du-lieu-decimal
dotnet run --project examples/toi-noi-dotnet/03-dieu-kien-vong-lap-phuong-thuc
dotnet run --project examples/toi-noi-dotnet/03-dieu-kien-vong-lap-phuong-thuc -- --verify
```

Các bài 01–03 đều có ảnh bìa riêng và sơ đồ SVG nội tuyến theo hệ thống hình minh họa của blog. Chế độ --verify của bài 03 gọi đúng hàm CalculateFare đang minh họa với 10 ca biên độc lập về kết quả kỳ vọng, bao gồm từ chối ngoài miền.

## Chạy bài 04–10

Mỗi project có --verify kiểm chứng các kết quả kỳ vọng, không cần package NuGet bổ sung.

```bash
dotnet run --project examples/toi-noi-dotnet/04-chuoi-ngay-gio-culture -- --verify
dotnet run --project examples/toi-noi-dotnet/05-debug-exception -- --verify
dotnet run --project examples/toi-noi-dotnet/06-class-encapsulation -- --verify
dotnet run --project examples/toi-noi-dotnet/07-record-struct-money-quote -- --verify
dotnet run --project examples/toi-noi-dotnet/08-interface-composition-di -- --verify
dotnet run --project examples/toi-noi-dotnet/09-generics-collections -- --verify
dotnet run --project examples/toi-noi-dotnet/10-linq-deferred-execution -- --verify
```

## Chạy bài 11–20

```bash
node examples/toi-noi-dotnet/verify-11-20.mjs
```

Script chạy 3 project console (16 ca kiểm chứng), 9 unit test, 4 integration test JSON/file và 24 API test. Cần SDK .NET 10 và mạng ở lần restore NuGet đầu. API 16–19 được test bằng WebApplicationFactory bài 20, không mở port; README từng bài có lệnh mở host localhost để thử tay.

Các API dùng lại FareDomain bài 14, chưa ghép service DDD/Clean/CQRS hoàn chỉnh. Package ASP.NET pin 10.0.10; Microsoft.OpenApi pin 2.7.5 đã vá advisory trên dòng 2.x. Không có credential production trong fixture. Các file tạm của bài 15 nằm trong thư mục riêng dưới thư mục temp của hệ điều hành và được dọn sau từng ca.

## Chạy bài 21–30

Thiết lập TOI_NOI_PG cho database PostgreSQL local riêng `toi_noi_dotnet_lab` theo [hướng dẫn bài 21](21-efcore-postgres/README.md). Sau đó:

```bash
node examples/toi-noi-dotnet/verify-21-30.mjs
```

Script chạy 20 test xUnit (mapping, query, transaction, worker, resilience, Outbox và domain) cùng 3 harness console về lifetime, parser và C# 14. Test database fail rõ nếu thiếu cấu hình; không dùng InMemory để thay PostgreSQL.

Benchmark thật là lệnh riêng trong [bài 28](28-benchmark-span/README.md); script verify không tự chạy phép đo hiệu năng dài. [Report lượt đo ngắn](28-benchmark-span/results/short-run.md) ghi runtime/hardware, số lần lặp và error. Schema của các ca database có tên GUID, được tạo/dọn riêng. Không có connection string chứa credential thật trong source.

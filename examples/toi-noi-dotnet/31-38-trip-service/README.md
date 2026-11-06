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

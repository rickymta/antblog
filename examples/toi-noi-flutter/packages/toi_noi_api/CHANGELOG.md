# Changelog

## 0.2.0

Thêm ở bài 20 của loạt Flutter qua Tới Nơi. Tương thích ngược: chỉ thêm kiểu, hàm và tham số tuỳ chọn; API của 0.1.0 không đổi.

- `sealed class ApiFailure` với năm biến thể: `NetworkFailure` (`reason`: `NetworkReason.unreachable` hoặc `badCertificate`), `TimeoutFailure` (`phase`: `TimeoutPhase.connect`, `send`, `receive`; `limit`), `ServerFailure` (5xx: `statusCode`, `title`, `traceId`), `ClientFailure` (4xx: `statusCode`, `title`, `fieldErrors`, `traceId`), `ParseFailure` (`dto`, `field`, `message`). Mỗi lỗi có `cause` (lỗi gốc) và `retryable`: "lỗi tạm thời, máy có thể tự thử lại an toàn nếu request lặp lại được" (`NetworkFailure` unreachable, `TimeoutFailure`, `ServerFailure`, `ClientFailure` 408 và 429 là `true`; `ParseFailure`, chứng chỉ sai và các 4xx khác là `false`). Đây không phải quy tắc cho nút Thử lại trên giao diện.
- `toApiFailure(error, [stackTrace])` đổi `DioException`, `ApiException`, `DtoFormatException` sang `ApiFailure`. Lỗi mạng mà dio để ở `unknown` được nhận ra: `HttpException` / `SocketException` thành `unreachable`, `TlsException` (kể cả `HandshakeException`) thành `badCertificate`. `DioException` kiểu `cancel` và lỗi không nhận ra được ném lại nguyên vẹn. `guardApi(() => ...)` bọc một lời gọi và giữ stack trace gốc. Package import `dart:io` trực tiếp (chỉ nhắm iOS và Android, không dùng được cho Flutter web).
- `RetryInterceptor({dio, maxAttempts = 3, baseDelay = 500 ms, maxDelay = 8 s, maxElapsed, retryStatusCodes = {408, 429, 500, 502, 503, 504}, readOnlyPostPaths = {'/quotes'}, sleep, random, now, onRetry})`: backoff luỹ thừa với full jitter; chỉ thử lại lỗi tạm thời (quá hạn, lỗi mạng, mã đã liệt kê), không thử lại các 4xx còn lại; chỉ thử request lặp lại được (`GET`, `HEAD`, có `Idempotency-Key`, `POST` tới `readOnlyPostPaths`, hoặc mang `RetryInterceptor.safeToRetryExtra`); tôn trọng `Retry-After` dạng số giây (vượt `maxDelay` thì bỏ cuộc); `maxElapsed` là hạn tổng, chặn timeout của lần thử lại bằng phần hạn còn lại. `sleep`, `random`, `now` tiêm được để test không chờ thật. Thứ tự: đặt sau interceptor log; interceptor đặt sau thấy `onRequest` mỗi lần thử nhưng `onError` và `onResponse` đúng một lần, và header chúng nối thêm không tích luỹ.
- `ToiNoiApiClient`: thêm tham số tuỳ chọn `sendTimeout` ở constructor và `cancelToken` ở `quote`, `createTrip`, `getTrip`, `acceptTrip`.
- `package:toi_noi_api/fault_injection.dart`: `FaultyAdapter` bọc một `HttpClientAdapter` và chèn `NetworkFault.refuse`, `connectTimeout`, `hang`, `loseResponse`, `reset`, `tlsFailure` theo kịch bản, để test lỗi tầng mạng và timeout mà fake API (chỉ tiêm 500 và độ trễ) không tạo được.
- Dependency mới: `clock` (đồng hồ của `RetryInterceptor`, thay được bằng `fake_async`); dev dependency `fake_async`.
- Gắn retry bằng `client.dio.interceptors.add(RetryInterceptor(dio: client.dio))`, sau interceptor log.

## 0.1.0

- Bản đầu tiên, tạo ở bài 19 của loạt Flutter qua Tới Nơi.
- `ToiNoiApiClient({baseUrl, adapter, interceptors, connectTimeout, receiveTimeout})` dựng trên `dio` với `quote`, `createTrip` (`Idempotency-Key`), `getTrip`, `acceptTrip(id, expectedVersion:)`, `close`. `getTrip` và `acceptTrip` trả `TripDto`; `acceptTrip` không còn tham số `driverId` (fake API đọc header `x-fake-driver-id`, gắn bằng interceptor ở client tài xế). Mọi request dùng `ResponseType.plain` và tự `jsonDecode`: body hỏng ở 200 ra `DtoFormatException`, 500 giữ mã trong `ApiException`. Base URL do người dùng package truyền vào, có thể kèm tiền tố gateway.
- DTO viết tay `QuoteRequestDto`, `QuoteDto`, `TripDto` với `fromJson`/`toJson` kiểm kiểu từng trường; `TripDto.toDomain()` đổi sang `Trip` và `Money` của `toi_noi_core`, `QuoteDto.toMoney()`. Tiền là `int` đồng, khoảng cách `int` mét, thời điểm UTC ISO-8601 có múi giờ, trạng thái là tên enum (đọc không phân biệt hoa thường).
- `ApiException` (HTTP không 2xx, đọc `problem+json`) và `DtoFormatException` (payload sai hợp đồng hoặc vi phạm bất biến của `Trip`). Lỗi tầng mạng vẫn là `DioException` nguyên bản, chờ bài 20.
- `SafeLogInterceptor`: log theo danh sách cho phép, che giá trị header lạ, chỉ ghi tên khoá query, không ghi body.
- `TripSnapshot {Trip trip; int version}` và `TripDto.toSnapshot()` cho luồng tài xế (`expectedVersion`) và bài 45; `TripDto.version` vẫn giữ.
- Từ chối: mã chuyến rỗng, `.`, `..` hoặc chứa `/` (`ArgumentError`); `baseUrl` có query hoặc fragment; thời điểm có độ lệch ngoài ±23:59; số vượt 2^53.
- `InProcessAdapter` và `RecordedRequest` nằm ở `package:toi_noi_fake_api/testing.dart` (không còn `testing.dart` trong package này); `shelf` chỉ còn là dev dependency.

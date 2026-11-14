---
slug: flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong
title: "Xử lý lỗi, retry, timeout và mất mạng: mạng di động không ổn định"
summary: "Bài 20: phân loại lỗi mạng thành sealed ApiFailure, đặt timeout có ngân sách, viết RetryInterceptor với backoff có jitter chỉ cho request lặp lại được, bọc connectivity_plus sau interface và dựng màn hình lỗi có nút Thử lại, Mở cài đặt; test lỗi mạng trên đồng hồ giả."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, dio, riverpod, retry, networking]
language: vi
---

Khách Tới Nơi bấm **Đặt chuyến** ngay khi xe buýt chui vào đường hầm. Sóng rơi từ ba vạch về không. Request có thể đã tới server nhưng response không về. Sau mười giây app báo lỗi, khách bấm lại, và tài khoản có hai chuyến.

Màn hình bài 19 chưa xử lý được cảnh này: lỗi mạng vẫn là `DioException` trần và spinner có thể quay rất lâu. Hôm nay đội gọi tên từng loại lỗi, giới hạn thời gian chờ, thử lại đúng chỗ và hiện nút làm được việc.

Đây là bài 20 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 19 về dio và JSON](/articles/flutter-http-dio-json-dto-goi-api-bao-gia) trước, vì bài này dựng tiếp trên `toi_noi_api` và `tools/fake_api`, và [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen). Phía server có [bài về timeout và retry](/articles/dotnet10-cache-timeout-retry-rate-limit) và [bài về idempotency](/articles/dotnet10-transaction-concurrency-idempotency) của lộ trình .NET.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>POST /trips</code> hết thời gian chờ nhận sau 10 giây, không có response. App gửi lại ngay lần nữa thì server có thể có bao nhiêu chuyến: chắc chắn một, hay có thể hai?</p>
<details><summary>Xem đáp án</summary><p>Có thể hai. Hết thời gian chờ chỉ nói app thôi chờ, không nói server chưa làm. Test <code>POST không khoá, response mất</code> dựng đúng trường hợp này. Gửi lại chỉ an toàn khi server nhận ra cùng một ý định nhờ <code>Idempotency-Key</code>; mục 3 biến điều đó thành luật cho interceptor.</p></details>
</div>

## Năm loại lỗi, một kiểu sealed

Bài 19 để lỗi rơi vào ba kiểu: `ApiException`, `DtoFormatException`, `DioException`. Giao diện không nên biết dio, nên ta gom chúng vào một `sealed class` mà mỗi biến thể trả lời: người dùng làm được gì?

### failure.dart · packages/toi_noi_api/lib/src/failure.dart (trích)

```dart
sealed class ApiFailure implements Exception {
  const ApiFailure({this.cause});

  // ...

  final Object? cause;

  // ...

  bool get retryable;
}
```

Năm biến thể là `NetworkFailure`, `TimeoutFailure`, `ServerFailure` (5xx), `ClientFailure` (4xx, mang `fieldErrors`) và `ParseFailure`. Vì `sealed`, `switch` thiếu nhánh không biên dịch. `retryable` có nghĩa hẹp: máy có thể tự thử lại mà không lo sai (nếu request lặp lại được). Nút Thử lại là quyết định riêng của giao diện. Hình 1 là bảng chuyển đổi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 490" role="img" aria-label="Hình 1. Từ lỗi gốc sang ApiFailure rồi sang hành động trên màn hình. connectionError, cùng HttpException hoặc SocketException nằm trong unknown, thành NetworkFailure unreachable, hành động Thử lại và Mở cài đặt. HandshakeException thành NetworkFailure badCertificate, không có nút. Bốn loại timeout thành TimeoutFailure, Thử lại. ApiException từ 500 trở lên thành ServerFailure, Thử lại kèm mã hỗ trợ. ApiException dưới 500 thành ClientFailure, riêng 408 và 429 có Thử lại, còn lại không nút. DtoFormatException thành ParseFailure, Thử lại thủ công kèm gợi ý Wi-Fi. DioException kiểu cancel và lỗi lạ không đổi mà được ném lại nguyên vẹn.">
<defs><marker id="fl20-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="22" class="t-b">Lỗi gốc</text><text x="350" y="22" class="t-b">ApiFailure</text><text x="630" y="22" class="t-b">Hành động trên màn hình</text>
<rect x="20" y="36" width="290" height="54" rx="8" class="f-phys-soft s-phys"/><text x="34" y="58" class="t-mono">connectionError</text><text x="34" y="76" class="t-small">unknown mang HttpException hoặc SocketException</text>
<rect x="350" y="36" width="240" height="54" rx="8" class="f-pk-soft s-pk"/><text x="364" y="58" class="t-b">NetworkFailure</text><text x="364" y="76" class="t-small">reason: unreachable</text>
<rect x="630" y="36" width="230" height="54" rx="8" class="f-surface s-muted"/><text x="644" y="67" class="t-small">Thử lại + Mở cài đặt</text>
<path d="M312 63 L348 63" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/><path d="M592 63 L628 63" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/>
<rect x="20" y="100" width="290" height="54" rx="8" class="f-phys-soft s-phys"/><text x="34" y="122" class="t-mono">HandshakeException (unknown)</text><text x="34" y="140" class="t-small">badCertificate chỉ có khi đặt validateCertificate</text>
<rect x="350" y="100" width="240" height="54" rx="8" class="f-pk-soft s-pk"/><text x="364" y="122" class="t-b">NetworkFailure</text><text x="364" y="140" class="t-small">reason: badCertificate</text>
<rect x="630" y="100" width="230" height="54" rx="8" class="f-surface s-muted"/><text x="644" y="131" class="t-small">Không nút, báo không an toàn</text>
<path d="M312 127 L348 127" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/><path d="M592 127 L628 127" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/>
<rect x="20" y="164" width="290" height="54" rx="8" class="f-phys-soft s-phys"/><text x="34" y="186" class="t-mono">connectionTimeout, sendTimeout,</text><text x="34" y="204" class="t-mono">receiveTimeout, transformTimeout</text>
<rect x="350" y="164" width="240" height="54" rx="8" class="f-pk-soft s-pk"/><text x="364" y="186" class="t-b">TimeoutFailure</text><text x="364" y="204" class="t-small">phase, limit</text>
<rect x="630" y="164" width="230" height="54" rx="8" class="f-surface s-muted"/><text x="644" y="195" class="t-small">Thử lại</text>
<path d="M312 191 L348 191" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/><path d="M592 191 L628 191" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/>
<rect x="20" y="228" width="290" height="54" rx="8" class="f-phys-soft s-phys"/><text x="34" y="250" class="t-mono">ApiException &gt;= 500</text><text x="34" y="268" class="t-small">badResponse 500, 502, 503...</text>
<rect x="350" y="228" width="240" height="54" rx="8" class="f-pk-soft s-pk"/><text x="364" y="250" class="t-b">ServerFailure</text><text x="364" y="268" class="t-small">statusCode, traceId</text>
<rect x="630" y="228" width="230" height="54" rx="8" class="f-surface s-muted"/><text x="644" y="250" class="t-small">Thử lại</text><text x="644" y="268" class="t-small">+ mã hỗ trợ (traceId)</text>
<path d="M312 255 L348 255" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/><path d="M592 255 L628 255" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/>
<rect x="20" y="292" width="290" height="54" rx="8" class="f-phys-soft s-phys"/><text x="34" y="314" class="t-mono">ApiException &lt; 500</text><text x="34" y="332" class="t-small">422, 409, 404, 429, 408...</text>
<rect x="350" y="292" width="240" height="54" rx="8" class="f-pk-soft s-pk"/><text x="364" y="314" class="t-b">ClientFailure</text><text x="364" y="332" class="t-small">statusCode, fieldErrors</text>
<rect x="630" y="292" width="230" height="54" rx="8" class="f-surface s-muted"/><text x="644" y="314" class="t-small">408, 429: Thử lại</text><text x="644" y="332" class="t-small">còn lại: không nút</text>
<path d="M312 319 L348 319" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/><path d="M592 319 L628 319" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/>
<rect x="20" y="356" width="290" height="54" rx="8" class="f-phys-soft s-phys"/><text x="34" y="378" class="t-mono">DtoFormatException</text><text x="34" y="396" class="t-small">200 nhưng sai hợp đồng</text>
<rect x="350" y="356" width="240" height="54" rx="8" class="f-pk-soft s-pk"/><text x="364" y="378" class="t-b">ParseFailure</text><text x="364" y="396" class="t-small">dto, field, message</text>
<rect x="630" y="356" width="230" height="54" rx="8" class="f-surface s-muted"/><text x="644" y="378" class="t-small">Thử lại tay + gợi ý</text><text x="644" y="396" class="t-small">đăng nhập Wi-Fi</text>
<path d="M312 383 L348 383" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/><path d="M592 383 L628 383" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-1-a)"/>
<rect x="20" y="430" width="840" height="44" rx="8" class="f-warn-soft s-warn"/><text x="34" y="457" class="t-small t-warn">DioException kiểu cancel và mọi lỗi lạ (StateError, TypeError...) không đổi: toApiFailure ném lại nguyên vẹn</text>
</svg>
</div>
<figcaption>Hình 1. Ba kiểu lỗi cũ đi vào, năm kiểu ApiFailure đi ra, mỗi kiểu kèm hành động nên hiện.</figcaption>
</figure>

Chỗ dễ sai nhất là nhận ra "mất mạng". Thử với socket thật ở localhost: đóng kết nối trước header cho `connectionError`, nhưng đứt giữa body cho `unknown` mang `HttpException`, và bắt tay TLS hỏng cho `unknown` mang `HandshakeException`; `badCertificate` chỉ có khi client đặt `validateCertificate`. Bản đầu của ta chỉ nhìn `connectionError`, nên hai kiểu kia không được thử lại và hiện "lỗi không mong muốn". Giờ một hàm trả lời cho cả hai nơi:

### failure.dart · packages/toi_noi_api/lib/src/failure.dart (trích)

```dart
NetworkReason? networkReasonOf(DioException e) {
  switch (e.type) {
    case DioExceptionType.connectionError:
      return NetworkReason.unreachable;
    case DioExceptionType.badCertificate:
      return NetworkReason.badCertificate;
    case DioExceptionType.unknown:
      final inner = e.error;
      if (inner is TlsException) {
        return NetworkReason.badCertificate;
      }
      if (inner is HttpException || inner is SocketException) {
        return NetworkReason.unreachable;
      }
      return null;
```

Package này import `dart:io` trực tiếp (chỉ nhắm iOS và Android, không dùng cho Flutter web). Hai nhóm không đổi: `cancel` (huỷ không phải thất bại) và `unknown` mang lỗi lạ như `StateError`, vì che chúng thành "mạng yếu" sẽ giấu lỗi lập trình.

### failure.dart · packages/toi_noi_api/lib/src/failure.dart (trích)

```dart
case DioExceptionType.cancel:
  Error.throwWithStackTrace(e, stackTrace ?? e.stackTrace);
case DioExceptionType.unknown:
  final reason = networkReasonOf(e);
  if (reason != null) {
    return NetworkFailure(reason: reason, cause: e);
  }
  final inner = e.error;
  if (inner is ApiFailure ||
      inner is ApiException ||
      inner is DtoFormatException) {
    return toApiFailure(inner!, stackTrace);
  }
  Error.throwWithStackTrace(e, stackTrace ?? e.stackTrace);
```

Nơi gọi dùng `guardApi(() => client.quote(request))` để đổi những lỗi nhận diện được thành `ApiFailure`.

## Timeout: ba hạn và một ngân sách

Dio có ba hạn: `connectTimeout`, `sendTimeout` và `receiveTimeout`. Theo tài liệu của dio, `receiveTimeout` đo tới byte đầu tiên rồi đo khoảng cách giữa các lần dữ liệu về, không phải tổng thời gian nhận, nên body nhỏ giọt không bao giờ chạm nó. `null` hoặc `Duration.zero` là không giới hạn, và `sendTimeout` mặc định là `null`. Lab đặt cả ba:

### providers.dart · 20-error-retry-connectivity/lib/providers.dart (trích)

```dart
class NetworkConfig {
  const NetworkConfig({
    this.connectTimeout = const Duration(seconds: 5),
    this.sendTimeout = const Duration(seconds: 10),
    this.receiveTimeout = const Duration(seconds: 10),
    this.maxAttempts = 3,
    this.baseDelay = const Duration(milliseconds: 500),
    this.maxDelay = const Duration(seconds: 4),
    this.maxElapsed = const Duration(seconds: 30),
  });
```

Một lần thử tốn tối đa `connectTimeout + receiveTimeout` = 15 giây, nên ba lần cộng backoff (0,5 và 1 giây) là 3 × 15 + 1,5 = 46,5 giây. Nếu server nhận kết nối rồi im lặng thì mỗi lần chỉ 10 giây, tổng 31,5 giây. Quá dài, nên `RetryInterceptor` có hạn tổng `maxElapsed`: lần thử nào không kịp thì không bắt đầu, và timeout của lần thử lại bị chặn bằng phần hạn còn lại. Với 30 giây, trường hợp server im lặng dừng đúng ở 30 giây (lần 3 chỉ còn 8,5 giây). Test chạy trên đồng hồ giả:

### controllers_test.dart · 20-error-retry-connectivity/test/controllers_test.dart (trích)

```dart
test('server treo cả ba lần: hạn tổng 30 giây chặn lần thử cuối', () {
  final harness = Harness(
    script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
    realSleep: true,
  );
  final container = harness.container();
  final watch = Stopwatch()..start();
  final run = runFake(
    () => container.read(quoteControllerProvider.notifier).fetch(),
  );
  expect(run.done, isTrue);
  final error = container.read(quoteControllerProvider).error;
  expect(
    error,
    isA<TimeoutFailure>()
        .having((f) => f.phase, 'phase', TimeoutPhase.receive)
        .having(
          (f) => f.limit,
          'limit',
          const Duration(seconds: 8, milliseconds: 500),
        ),
  );
  // Lần 1 tới 10 s, chờ 0,5 s, lần 2 tới 20,5 s, chờ 1 s: còn 8,5 s trong
  // hạn tổng 30 s nên lần 3 bị chặn receiveTimeout xuống 8,5 s.
  expect(run.elapsed, const Duration(seconds: 30));
  expect(harness.faulty.calls, 3);
  expect(watch.elapsed, lessThan(const Duration(seconds: 5)));
});
```

Hạn này không với tới lần gửi đầu, vốn chỉ theo timeout của client. `NetworkFault.hang` chỉ giả lập "không có byte nào"; cách adapter thật đếm giờ ta lấy từ tài liệu.

## Retry: lỗi tạm thời và request lặp lại được

`RetryInterceptor` thử lại khi cả hai câu trả lời là "có". Một: lỗi có tạm thời không? Hết thời gian chờ, lỗi mạng (`networkReasonOf`) và mã 408, 429, 500, 502, 503, 504 thì có. Các 4xx còn lại (400, 401, 404, 409, 422) không, vì gửi lại y nguyên thì server trả y nguyên; 408 và 429 là chính server bảo thử lại sau. Lỗi TLS và huỷ request cũng không. Hai: request có lặp lại được an toàn không? Hình 2 vẽ cả hai cổng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Quyết định thử lại. Một DioException đi qua ba cổng: lỗi có tạm thời không, request có lặp lại được không, còn lượt thử không. Trả lời không ở bất kỳ cổng nào thì lỗi được trả nguyên vẹn lên tầng trên. Trả lời có ở cả ba thì chờ backoff có jitter, gửi lại và quay về cổng đầu.">
<defs><marker id="fl20-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl20-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="60" class="t-b">Lỗi xảy ra</text><text x="34" y="82" class="t-mono">DioException</text>
<rect x="190" y="30" width="170" height="70" rx="10" class="f-pk-soft s-pk"/><text x="204" y="56" class="t-b">1. Tạm thời?</text><text x="204" y="74" class="t-small">quá hạn, lỗi mạng, 408,</text><text x="204" y="90" class="t-small">429, 500/502/503/504</text>
<rect x="400" y="30" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="414" y="56" class="t-b">2. Lặp lại được?</text><text x="414" y="74" class="t-small">GET, HEAD, có khoá,</text><text x="414" y="90" class="t-small">POST chỉ đọc đã khai báo</text>
<rect x="630" y="30" width="130" height="70" rx="10" class="f-pk-soft s-pk"/><text x="644" y="56" class="t-b">3. Còn lượt?</text><text x="644" y="74" class="t-small">dưới maxAttempts</text>
<path d="M152 65 L188 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-2-a)"/>
<path d="M362 65 L398 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-2-a)"/>
<path d="M592 65 L628 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-2-a)"/>
<text x="366" y="56" class="t-small">có</text><text x="596" y="56" class="t-small">có</text>
<rect x="20" y="170" width="520" height="70" rx="10" class="f-warn-soft s-warn"/><text x="34" y="200" class="t-b">Trả lỗi nguyên vẹn lên tầng trên</text><text x="34" y="222" class="t-small">guardApi đổi thành ApiFailure; không thử lại, không chờ</text>
<rect x="600" y="170" width="260" height="70" rx="10" class="f-pk-soft s-pk"/><text x="614" y="196" class="t-b">Chờ backoff có jitter,</text><text x="614" y="214" class="t-b">gửi lại, quay về cổng 1</text>
<path d="M275 102 L275 168" class="f-none s-warn" stroke-width="2" marker-end="url(#fl20-2-w)"/><text x="283" y="140" class="t-small t-warn">không</text>
<path d="M495 102 L495 168" class="f-none s-warn" stroke-width="2" marker-end="url(#fl20-2-w)"/><text x="503" y="140" class="t-small t-warn">không</text>
<path d="M645 102 L542 188" class="f-none s-warn" stroke-width="2" marker-end="url(#fl20-2-w)"/><text x="578" y="150" class="t-small t-warn">không</text>
<path d="M715 102 L715 168" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-2-a)"/><text x="723" y="140" class="t-small">có</text>
</svg>
</div>
<figcaption>Hình 2. Ba cổng nối tiếp: chỉ khi cả ba đều "có" mới chờ rồi gửi lại.</figcaption>
</figure>

Với Tới Nơi, cổng hai cho kết quả sau:

| Request | Lặp lại được? | Lý do |
|---|---|---|
| `GET /trips/{id}` | có | chỉ đọc |
| `POST /trips` kèm `Idempotency-Key` | có | server nhận ra cùng khoá |
| `POST /quotes` | có, phải khai báo | chỉ đọc dù dùng POST |
| `POST` không khoá, kể cả `accept` | không | lần đầu có thể đã chạy xong |

`POST /quotes` chỉ đọc nhưng interceptor không tự đoán: ta khai báo `readOnlyPostPaths` (mặc định `{'/quotes'}`) hoặc đánh dấu bằng `RetryInterceptor.safeToRetryExtra`. Với `accept`, test dựng trường hợp mất response: interceptor không gửi lại, và lần gửi lại thủ công nhận 409. `expectedVersion` chặn hai tài xế nhận trùng nhưng không làm lần gửi lại an toàn.

### retry_interceptor.dart · packages/toi_noi_api/lib/src/retry_interceptor.dart (trích)

```dart
bool isRepeatable(RequestOptions options) {
  final method = options.method.toUpperCase();
  if (method == 'GET' || method == 'HEAD') {
    return true;
  }
  if (options.extra[_safeKey] == true) {
    return true;
  }
  if (method == 'POST' && readOnlyPostPaths.contains(options.path)) {
    return true;
  }
  for (final entry in options.headers.entries) {
    if (entry.key.toLowerCase() == 'idempotency-key' &&
        '${entry.value}'.isNotEmpty) {
      return true;
    }
  }
  return false;
}
```

Backoff luỹ thừa chờ 0,5, 1, 2 giây. Hàng nghìn máy cùng nối lại sẽ dồn request vào server trong cùng một giây, nên ta thêm **jitter**: chờ một khoảng thời gian ngẫu nhiên, phân bố đều trong `[0, trần]`. Nguồn ngẫu nhiên và hàm chờ được inject vào để test biết chính xác con số. Response có `Retry-After` dạng số giây thì ta chờ ít nhất chừng đó, và bỏ cuộc nếu nó vượt `maxDelay`:

### retry_interceptor.dart · packages/toi_noi_api/lib/src/retry_interceptor.dart (trích)

```dart
Duration backoff(int attempt) {
  assert(attempt >= 2, 'lần thử đầu không có khoảng chờ');
  final doublings = min(attempt - 2, 30);
  final ceiling = min(
    maxDelay.inMicroseconds,
    baseDelay.inMicroseconds * (1 << doublings),
  );
  return Duration(microseconds: (ceiling * _random()).round());
}

// ...

for (var attempt = 2; attempt <= maxAttempts; attempt++) {
  var wait = backoff(attempt);
  final retryAfter = _retryAfter(last);
  if (retryAfter != null) {
    if (retryAfter > maxDelay) {
      break; // server đòi chờ lâu hơn mức ta chịu
    }
    if (retryAfter > wait) {
      wait = retryAfter;
    }
  }
  Duration? remaining;
  if (maxElapsed != null && started is DateTime) {
    remaining = maxElapsed! - _now().difference(started) - wait;
    if (remaining <= Duration.zero) {
      break; // không kịp hạn tổng
    }
  }
```

**Thứ tự interceptor** cần cẩn thận, vì mỗi lần gửi lại chạy `dio.fetch`, tức qua mọi interceptor một lần nữa. Log đặt **trước** retry để mỗi lần thử có một dòng. Interceptor đặt **sau** (bài 24 gắn token và refresh 401 ở đó) thấy `onRequest` mỗi lần thử; bản đầu của ta còn cho nó thấy `onError` của từng lần thử lại cộng lỗi cuối, và header nó nối thêm tích luỹ. Bản đúng:

### retry_interceptor.dart · packages/toi_noi_api/lib/src/retry_interceptor.dart (trích)

```dart
if (!options.extra.containsKey(_startKey)) {
  final snapshot = options.copyWith();
  options.extra[_startKey] = _now();
  options.extra[_snapshotKey] = snapshot;
}

// ...

if (options.extra[_insideKey] == true) {
  handler.reject(err);
  return;
}
```

Mỗi lần thử lại bắt đầu từ bản sao request lúc vào retry, nên header không tích luỹ. Lỗi trung gian bị `reject` để interceptor sau không thấy; lỗi cuối đi tiếp bằng `next` nên chúng thấy đúng một lần (thử `reject` cả lỗi cuối thì chúng không thấy lỗi nào, test đỏ). Log thật khi hai request đầu bị inject lỗi 500:

### Log thật (fake API chạy với failFirst: 2, random cố định ở 1.0)

```text
--> POST /quotes {accept: application/json, content-type: application/json}
<-- ERROR badResponse 500 POST /quotes (29 ms)
retry: lần 2/3 sau 500 ms, vì badResponse
--> POST /quotes {accept: application/json, content-type: application/json, content-length: 39}
<-- ERROR badResponse 500 POST /quotes (2 ms)
retry: lần 3/3 sau 1000 ms, vì badResponse
--> POST /quotes {accept: application/json, content-type: application/json, content-length: 39}
<-- 200 POST /quotes (10 ms) {content-type: application/json; charset=utf-8, content-length: 74}
```

Interceptor gắn sau khi tạo client, vì cần `Dio` của client để gửi lại:

### providers.dart · 20-error-retry-connectivity/lib/providers.dart (trích)

```dart
client.dio.interceptors.add(
  RetryInterceptor(
    dio: client.dio,
    maxAttempts: config.maxAttempts,
    baseDelay: config.baseDelay,
    maxDelay: config.maxDelay,
    maxElapsed: config.maxElapsed,
    sleep: ref.watch(retrySleepProvider),
    random: ref.watch(retryRandomProvider),
    onRetry: (event) => log(
      'retry: lần ${event.attempt}/${config.maxAttempts} sau '
      '${event.wait.inMilliseconds} ms, vì ${event.error.type.name}',
    ),
  ),
);
```

## Test lỗi mạng mà không chờ thật

Fake API chỉ inject lỗi 500 và độ trễ. Mất mạng xảy ra trước response, nên `FaultyAdapter` giả lập ở tầng adapter: phần tử thứ i của kịch bản áp cho lần gọi thứ i.

| `NetworkFault` | Điều giả lập | Tới server? |
|---|---|---|
| `refuse` | server từ chối kết nối | không |
| `connectTimeout` | chờ đúng `connectTimeout` rồi quá hạn | không |
| `hang` | server im lặng, chờ đúng `receiveTimeout` | có thể; adapter giả không chuyển vào |
| `loseResponse` | server xong việc, `connectionError` | có |
| `reset` | server xong việc, `unknown` + `HttpException` | có |
| `tlsFailure` | bắt tay TLS hỏng, `unknown` + `HandshakeException` | không |

Trường hợp `loseResponse` là trường hợp xấu nhất của câu hỏi mở đầu. Có khoá thì interceptor gửi lại an toàn:

### retry_test.dart · packages/toi_noi_api/test/retry_test.dart (trích)

```dart
test(
  'mất response sau khi server tạo chuyến: thử lại vẫn chỉ một chuyến',
  () async {
    final lab = RetryLab(script: [NetworkFault.loseResponse]);
    final trip = await lab.client.createTrip(
      ride5km,
      idempotencyKey: 'book-7',
    );
    expect(trip.id, 'T-1');
    expect(lab.faulty.calls, 2);
    expect(lab.inProcess.requests, hasLength(2)); // server nhận hai lần
    expect(lab.store.trips, hasLength(1)); // nhưng chỉ có một chuyến
  },
);
```

Server nhận hai request cùng khoá và chỉ tạo một chuyến; trường hợp không khoá có test riêng. Adapter giả chưa đủ: file test với socket thật ở localhost (đóng sớm, đứt giữa body, TLS hỏng, cổng đóng, server im lặng) mới lộ ra lỗ hổng `unknown` ở mục 1.

Thử phá để chắc test bắt được lỗi: `isRepeatable` luôn `true` thì chín test đỏ, thử lại mọi mã từ 400 thì chín test đỏ, chỉ nhìn `connectionError` thì năm test đỏ.

## Connectivity chỉ là gợi ý

Plugin `connectivity_plus` cho biết loại đường truyền, nhưng README của nó dặn không dựa vào đó để quyết định request có gọi được hay không: Wi-Fi quán cà phê chưa đăng nhập vẫn báo Wi-Fi. README còn ghi: trên iOS có thể thấy "none" rồi "wifi" liền sau khi nối lại.

Vì thế lab chỉ dùng connectivity để hiện banner và chọn câu chữ, không chặn request. Plugin nằm sau một interface để test thay được:

### connectivity.dart · 20-error-retry-connectivity/lib/connectivity.dart (trích)

```dart
abstract interface class ConnectivityMonitor {
  /// Tình trạng hiện tại.
  Future<LinkType> current();

  /// Mỗi lần tình trạng đổi. Với plugin, iOS có thể phát "none" rồi "wifi" liền
  /// sau khi nối lại, nên đừng coi mỗi sự kiện là một biến cố thật.
  Stream<LinkType> get changes;
}
```

Test dựng platform giả của plugin để kiểm ánh xạ. Một lỗi thật xuất hiện khi đặt luồng lên Riverpod: bản đầu hỏi giá trị hiện tại rồi mới nghe, và một thay đổi đến giữa hai bước bị mất. Bản đúng nghe trước rồi mới hỏi:

### connectivity.dart · 20-error-retry-connectivity/lib/connectivity.dart (trích)

```dart
onListen: () {
  var changed = false;
  subscription = monitor.changes.listen((link) {
    changed = true;
    controller.add(link);
  }, onError: controller.addError);
  monitor.current().then((link) {
    if (!changed && !controller.isClosed) {
      controller.add(link);
    }
  }, onError: controller.addError);
},
```

Hai test màn hình khoá triết lý này: báo `wifi` mà server không tới được thì hai nút vẫn hiện; báo `none` mà server gọi được thì request vẫn đi và ra 25.500 VND.

## Màn hình lỗi có hành động, và Riverpod đừng thử lại thêm

Mỗi lỗi đã có tên, nên chọn hành động là một `switch` không nhánh mặc định. `describeFailure` không đưa `title` tiếng Anh của server lên giao diện, chỉ đưa `traceId`.

### failure_text.dart · 20-error-retry-connectivity/lib/failure_text.dart (trích)

```dart
case NetworkFailure():
  return (
    message: link == LinkType.none
        ? 'Thiết bị đang không có kết nối mạng. Bật Wi-Fi hoặc dữ liệu di '
              'động rồi thử lại.'
        : 'Không tới được máy chủ. Mạng có thể đang yếu, hoặc Wi-Fi cần '
              'đăng nhập trước.',
    supportCode: null,
    actions: _both,
  );

// ...

case ClientFailure(statusCode: 422, :final fieldErrors)
    when fieldErrors.containsKey('distanceKm'):
  return (
    message: 'Quãng đường phải nằm trong khoảng 0 đến 100 km.',
    supportCode: null,
    actions: _none,
  );
```

`FailureView` chỉ vẽ nút có trong tập `actions`. Lỗi 422 không có nút vì bấm lại gửi y nguyên; 408 và 429 có. `ParseFailure` không retryable, nhưng nếu đó là trang đăng nhập Wi-Fi thì khách có thể đăng nhập rồi thử, nên nút bấm thủ công vẫn hiện kèm gợi ý: nút không suy ra từ `retryable`. Request bị huỷ (đổi form, thoát màn hình) không phải lỗi: controller huỷ request cũ bằng `CancelToken`, và `FailureView` không vẽ gì cho nó.

Nút **Mở cài đặt** đi qua interface `SystemSettings` bọc `app_settings`, nhãn nói đúng nơi nó dẫn tới. Android mở cài đặt không dây. Trên iOS, `AppSettingsPlugin.swift` của plugin cho thấy loại không biết rơi vào `openSettingsURLString`, tức trang cài đặt của chính app, không có Wi-Fi, nên nhãn là "Mở cài đặt ứng dụng". Ta chỉ đọc source, chưa chạy trên thiết bị.

Cuối cùng là bẫy chồng retry. Theo bài 18, Riverpod 3 tự thử lại provider ném `Exception` (11 lần gọi), và `ApiFailure` là `Exception`: provider gọi API trong `build` kéo theo ba request của interceptor mỗi lần. Hình 3 đếm số request.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Hai tầng retry nhân với nhau. Làn trên: FutureProvider gọi API trong build với retry mặc định của Riverpod, 11 lần gọi build, mỗi lần RetryInterceptor gửi tối đa 3 request, tổng 33 request tới server. Làn dưới: retry tắt bằng noProviderRetry hoặc gọi trong method của notifier, 1 lần gọi, RetryInterceptor gửi tối đa 3 request, tổng 3 request.">
<defs><marker id="fl20-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl20-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="28" class="t-b">Retry mặc định của Riverpod còn bật</text>
<rect x="20" y="40" width="240" height="90" rx="10" class="f-warn-soft s-warn"/><text x="36" y="72" class="t-b">Riverpod</text><text x="36" y="96" class="t-small">FutureProvider gọi API</text><text x="36" y="114" class="t-small">trong build: 11 lần gọi</text>
<rect x="320" y="40" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="72" class="t-b">RetryInterceptor</text><text x="336" y="96" class="t-small">mỗi lần gọi: tối đa</text><text x="336" y="114" class="t-small">3 request</text>
<rect x="620" y="40" width="240" height="90" rx="10" class="f-warn-soft s-warn"/><text x="636" y="72" class="t-big t-warn">33</text><text x="636" y="100" class="t-small">request tới server</text><text x="636" y="116" class="t-small">11 x 3</text>
<path d="M262 85 L318 85" class="f-none s-warn" stroke-width="2" marker-end="url(#fl20-3-w)"/><path d="M562 85 L618 85" class="f-none s-warn" stroke-width="2" marker-end="url(#fl20-3-w)"/>
<text x="20" y="198" class="t-b">noProviderRetry, hoặc gọi trong method của notifier</text>
<rect x="20" y="210" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="242" class="t-b">Riverpod</text><text x="36" y="266" class="t-small">không thử lại:</text><text x="36" y="284" class="t-small">1 lần gọi</text>
<rect x="320" y="210" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="242" class="t-b">RetryInterceptor</text><text x="336" y="266" class="t-small">mỗi lần gọi: tối đa</text><text x="336" y="284" class="t-small">3 request</text>
<rect x="620" y="210" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="636" y="242" class="t-big t-pk">3</text><text x="636" y="270" class="t-small">request tới server</text><text x="636" y="286" class="t-small">1 x 3</text>
<path d="M262 255 L318 255" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-3-a)"/><path d="M562 255 L618 255" class="f-none s-pk" stroke-width="2" marker-end="url(#fl20-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Số request là tích của các tầng retry. Ta giữ tầng biết request nào lặp lại được và tắt tầng của Riverpod.</figcaption>
</figure>

Test dùng đồng hồ giả: server luôn 500 thì chính sách mặc định ra 33 request, tắt đi còn 3. Với 4xx, Riverpod vẫn gọi lại 11 lần. Method của notifier (`QuoteController.fetch`) không bị thử lại.

### providers.dart · 20-error-retry-connectivity/lib/providers.dart (trích)

```dart
Duration? noProviderRetry(int retryCount, Object error) => null;
```

### main.dart · 20-error-retry-connectivity/lib/main.dart (trích)

```dart
void main() =>
    runApp(ProviderScope(retry: noProviderRetry, child: const ToiNoiApp()));
```

Quy tắc: một tầng thử lại, đặt ở nơi biết request nào an toàn; tắt cả app ở `main` thay vì từng provider.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi xử lý mạng</span>
<ul>
<li><strong>Thử lại mọi POST.</strong> Dấu hiệu: một lần bấm Đặt chuyến ra hai chuyến sau lúc mất sóng. Nguyên nhân: hết thời gian chờ không có nghĩa server chưa làm. Cách sửa: chỉ thử lại GET, request có <code>Idempotency-Key</code> và POST chỉ đọc đã khai báo, với backoff có jitter và trần.</li>
<li><strong>Nhận mất mạng chỉ bằng <code>connectionError</code>.</strong> Dấu hiệu: đứt giữa body hay TLS hỏng hiện "lỗi không mong muốn", không được thử lại. Nguyên nhân: dio để chúng ở <code>unknown</code>. Cách sửa: nhìn cả <code>HttpException</code>, <code>SocketException</code>, <code>TlsException</code> bên trong, và kiểm bằng socket thật.</li>
<li><strong>Lấy connectivity làm cổng.</strong> Dấu hiệu: request không gửi dù mạng dùng được, hoặc vẫn treo trên Wi-Fi chưa đăng nhập. Nguyên nhân: connectivity chỉ báo loại đường truyền. Cách sửa: coi nó là gợi ý, luôn bắt lỗi mạng thật.</li>
<li><strong>Riverpod thử lại chồng lên interceptor.</strong> Dấu hiệu: một provider hỏng sinh hàng chục request, kể cả với 422. Nguyên nhân: <code>ApiFailure</code> là <code>Exception</code> nên bị retry mặc định. Cách sửa: <code>ProviderScope(retry: noProviderRetry)</code>.</li>
<li><strong>Bắt mọi lỗi thành "lỗi mạng".</strong> Dấu hiệu: <code>StateError</code> hiện thành "mạng yếu", không ai báo bug. Nguyên nhân: <code>catch (e)</code> gom hết. Cách sửa: chỉ đổi loại lỗi nhận diện được, ném lại phần còn lại như <code>toApiFailure</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository, chạy package rồi lab:

```bash
cd examples/toi-noi-flutter/packages/toi_noi_api
dart pub get
dart analyze
dart test

cd ../../20-error-retry-connectivity
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: hai nơi đều `No issues found!`, `toi_noi_api` 0.2.0 có 192 test xanh (106 mới của bài này), lab có 61 test xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị hay mạng ngoài. Phiên bản dùng: `connectivity_plus` 7.3.2, `app_settings` 9.0.1, `fake_async` 1.3.3, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3` giải ra bản này).

| Test | Chứng minh điều gì |
|---|---|
| `toi_noi_api`: `failure_test` (28), `real_socket_test` (7) | phân loại từng kiểu lỗi, cả bằng socket thật ở localhost; huỷ và lỗi lạ ném lại |
| `toi_noi_api`: `retry_test`, `retry_budget_test` (59) | số lần thử; 4xx; POST không khoá; Retry-After; hạn tổng; thứ tự interceptor |
| `toi_noi_api`: `faulty_adapter_test`, `client_options_test` (12) | kịch bản lỗi; `cancelToken`, `sendTimeout` |
| lab: `controllers_test`, `quote_screen_test` (36) | hạn tổng 30 giây; huỷ request; nút Thử lại, Mở cài đặt; connectivity không phải cổng |
| lab: `riverpod_retry_test`, `connectivity_test`, `failure_text_test` (25) | 33 request so với 3; ánh xạ connectivity; câu chữ và nhãn nút |

**Chưa làm**: chạy trên thiết bị (hai plugin native, màn hình cài đặt); cách adapter thật đếm `receiveTimeout`; timeout riêng từng loại request; `Retry-After` dạng ngày giờ; hạn tổng cho lần gửi đầu; hiện "đang thử lại lần 2/3"; hàng đợi gửi lại khi có mạng (bài 46).

## Bài tập

1. Tắt `maxElapsed`, đặt `maxAttempts` 5, `receiveTimeout` 10 giây, `maxDelay` 4 giây, `random` 1,0; server luôn im lặng sau khi nhận kết nối. Tổng thời gian?
2. App tài xế bấm nhận chuyến, response bị mất. Interceptor có thử lại không, và bấm lại thì nhận gì?
3. Ở dev (HTTP), Wi-Fi quán cà phê trả trang HTML đăng nhập với mã 200 cho `GET /trips/{id}`. Lỗi kiểu gì, có thử lại không, màn hình hiện gì? Còn khi API chạy HTTPS?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> 5 × 10 giây = 50 giây, cộng 0,5 + 1 + 2 + 4 = 7,5 giây chờ (lần cuối 0,5 × 2³ = 4 giây vừa bằng trần, chưa bị chặn). Tổng 57,5 giây, đó là lý do ta có <code>maxElapsed</code>.</p>
<p><strong>2.</strong> Không: <code>accept</code> là POST không khoá. Bấm lại nhận 409 vì lần đầu đã chạy xong (test <code>nhận chuyến: POST có expectedVersion nhưng không có khoá</code>). App nên GET chuyến để biết mình đã nhận chưa.</p>
<p><strong>3.</strong> <code>DtoFormatException</code>, nên <code>ParseFailure</code>. Interceptor không thử lại vì mã 200 không phải lỗi. Màn hình nhắc đăng nhập Wi-Fi kèm nút Thử lại, không có banner vì connectivity vẫn báo Wi-Fi (test <code>Wi-Fi công cộng trả HTML với mã 200</code>). Portal chỉ thay được response khi dùng HTTP; với HTTPS, theo cơ chế TLS, lỗi sẽ nằm ở bước bắt tay (<code>NetworkFailure</code> kiểu <code>badCertificate</code>), nhưng ta chưa thử với portal thật.</p>
</details>

Bài trước: [HTTP, dio và JSON: gọi API báo giá của trip-service](/articles/flutter-http-dio-json-dto-goi-api-bao-gia). Bài tiếp: [Clean Architecture cho mobile: domain, data, presentation và repository](/articles/flutter-clean-architecture-domain-data-presentation-mobile).

## Tài liệu gốc

- [connectivity_plus](https://pub.dev/packages/connectivity_plus) và [app_settings](https://pub.dev/packages/app_settings) (pub.dev).
- [dio](https://pub.dev/packages/dio) và [tài liệu API của dio](https://pub.dev/documentation/dio/latest/dio/dio-library.html) (pub.dev).
- [Automatic Retry](https://riverpod.dev/docs/concepts2/retry) và [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) (riverpod.dev, pub.dev).
- [fake_async](https://pub.dev/packages/fake_async) (pub.dev) và [Class modifiers: sealed](https://dart.dev/language/class-modifiers#sealed) (dart.dev).
- [Fetch data from the internet](https://docs.flutter.dev/cookbook/networking/fetch-data) (docs.flutter.dev).

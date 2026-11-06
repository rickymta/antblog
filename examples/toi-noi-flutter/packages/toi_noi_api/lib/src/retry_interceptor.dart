import 'dart:math';

import 'package:clock/clock.dart';
import 'package:dio/dio.dart';

import 'failure.dart';

/// Hàm chờ một khoảng thời gian. Mặc định là `Future.delayed`; test truyền
/// hàm ghi lại khoảng chờ rồi trả ngay, hoặc dùng `fake_async`.
typedef Sleep = Future<void> Function(Duration duration);

/// Một lần thử lại sắp diễn ra, báo cho [RetryInterceptor.onRetry].
final class RetryEvent {
  const RetryEvent({
    required this.attempt,
    required this.wait,
    required this.error,
  });

  /// Số thứ tự của lần thử sắp chạy (lần đầu là 1, nên giá trị đầu tiên ở đây là 2).
  final int attempt;

  /// Thời gian sẽ chờ trước khi thử.
  final Duration wait;

  /// Lỗi của lần thử vừa rồi.
  final DioException error;
}

/// Thử lại request khi lỗi tạm thời, với backoff luỹ thừa có jitter.
///
/// Quyết định thử lại gồm hai câu hỏi, và cả hai phải "có":
///
/// 1. Lỗi có tạm thời không? Hết thời gian chờ (kết nối, gửi, nhận), lỗi mạng
///    (xem `networkReasonOf`: `connectionError`, kết nối đứt giữa body, bị
///    reset), và các mã trong [retryStatusCodes] (mặc định 408, 429, 500, 502,
///    503, 504). Các 4xx còn lại (400, 401, 404, 409, 422...) **không** được
///    thử lại: gửi lại y nguyên thì server trả y nguyên. Lỗi TLS và huỷ request
///    cũng không.
/// 2. Request có lặp lại an toàn không ([isRepeatable])? `GET` và `HEAD`;
///    request mang header `Idempotency-Key`; `POST` tới một đường dẫn chỉ đọc
///    trong [readOnlyPostPaths] (mặc định `/quotes`: báo giá dùng POST vì có
///    body nhưng không đổi gì ở server); và request được đánh dấu bằng
///    [safeToRetryExtra]. Một `POST` khác không được thử lại, vì lần đầu có
///    thể đã chạy xong ở server (hết thời gian chờ không có nghĩa là server
///    chưa làm).
///
/// Thời gian chờ trước lần thử thứ n (n = 2, 3, ...) là một số ngẫu nhiên
/// đều trong `[0, min(maxDelay, baseDelay * 2^(n-2))]` ("full jitter"), để
/// hàng nghìn máy cùng lỗi một lúc không cùng gõ lại server ở cùng một giây.
/// Nếu response mang `Retry-After` dạng số giây thì chờ ít nhất chừng đó; nếu
/// server đòi chờ lâu hơn [maxDelay] thì ta bỏ cuộc thay vì treo người dùng.
/// (Dạng ngày giờ HTTP của `Retry-After` chưa được đọc.)
///
/// [maxElapsed] (nếu đặt) là hạn tổng của cả chuỗi thử, tính từ lúc request đầu
/// được gửi: lần thử nào không kịp trong phần hạn còn lại thì không bắt đầu, và
/// lần thử nào bắt đầu thì `connectTimeout`, `sendTimeout`, `receiveTimeout` của
/// nó bị chặn bằng phần hạn còn lại. Hạn này không với tới lần gửi đầu: lần đó
/// vẫn theo timeout của client, và `receiveTimeout` đo khoảng cách giữa các
/// lần dữ liệu về nên một body nhỏ giọt không bao giờ chạm nó.
///
/// Gắn sau khi tạo client, vì cần chính `Dio` của nó để gửi lại:
///
/// ```dart
/// final client = ToiNoiApiClient(baseUrl: baseUrl, interceptors: [log]);
/// client.dio.interceptors.add(RetryInterceptor(dio: client.dio));
/// ```
///
/// **Thứ tự interceptor.** Mỗi lần thử lại chạy lại toàn bộ interceptor bằng
/// `dio.fetch`, nên:
/// - Đặt log (và mọi thứ chỉ quan sát) **trước** retry: mỗi lần thử có một
///   dòng log riêng.
/// - Interceptor đặt **sau** retry (gắn token ở bài 24, chuẩn hoá lỗi) thấy
///   `onRequest` một lần cho mỗi lần thử, nhưng `onError` đúng **một** lần với
///   lỗi cuối cùng và `onResponse` đúng một lần với response thành công. Các
///   lỗi trung gian của lần thử lại bị chặn lại ở đây (`handler.reject`) để
///   chúng không bị xử lý hai lần.
/// - Mỗi lần thử lại bắt đầu từ bản sao của request **lúc nó đi vào** retry,
///   nên header mà interceptor sau retry nối thêm không tích luỹ qua các lần.
///
/// Body phải gửi lại được (JSON, `Map`); `FormData` và `Stream` thì không.
final class RetryInterceptor extends Interceptor {
  RetryInterceptor({
    required this.dio,
    this.maxAttempts = 3,
    this.baseDelay = const Duration(milliseconds: 500),
    this.maxDelay = const Duration(seconds: 8),
    this.maxElapsed,
    this.retryStatusCodes = const {408, 429, 500, 502, 503, 504},
    this.readOnlyPostPaths = const {'/quotes'},
    Sleep? sleep,
    double Function()? random,
    DateTime Function()? now,
    this.onRetry,
  }) : assert(maxAttempts >= 1, 'maxAttempts tính cả lần đầu, tối thiểu 1'),
       assert(!baseDelay.isNegative && !maxDelay.isNegative),
       _sleep = sleep ?? Future<void>.delayed,
       _random = random ?? Random().nextDouble,
       _now = now ?? (() => clock.now());

  /// Đặt vào `Options(extra: ...)` của một request chỉ đọc mà đường dẫn không
  /// nằm trong [readOnlyPostPaths].
  ///
  /// ```dart
  /// dio.post('/search', data: body, options: Options(extra: RetryInterceptor.safeToRetryExtra));
  /// ```
  static const Map<String, Object?> safeToRetryExtra = {_safeKey: true};

  static const _safeKey = 'toi_noi_api/safe_to_retry';
  static const _insideKey = 'toi_noi_api/retrying';
  static const _startKey = 'toi_noi_api/started_at';
  static const _snapshotKey = 'toi_noi_api/snapshot';

  /// Tổng số lần gửi tối đa, tính cả lần đầu: 3 nghĩa là 1 lần đầu và 2 lần thử lại.
  final int maxAttempts;

  /// Trần chờ trước lần thử lại đầu tiên (nhân đôi ở mỗi lần sau).
  final Duration baseDelay;

  /// Trần của mọi khoảng chờ backoff, và ngưỡng bỏ cuộc với `Retry-After`.
  final Duration maxDelay;

  /// Hạn tổng của cả chuỗi thử (xem mô tả lớp); `null` là không đặt hạn.
  final Duration? maxElapsed;

  /// Mã HTTP được coi là tạm thời.
  final Set<int> retryStatusCodes;

  /// Đường dẫn (như truyền cho dio, không gồm `baseUrl`) của các `POST` chỉ
  /// đọc: gửi lại không làm thay đổi gì ở server.
  final Set<String> readOnlyPostPaths;

  /// Gọi trước mỗi lần thử lại, để ghi log hoặc cập nhật giao diện.
  final void Function(RetryEvent event)? onRetry;

  /// `Dio` dùng để gửi lại, thường là `client.dio`.
  final Dio dio;

  final Sleep _sleep;
  final double Function() _random;
  final DateTime Function() _now;

  /// Request này lặp lại được an toàn không (xem mô tả lớp).
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

  /// Khoảng chờ trước lần thử thứ [attempt] (từ 2 trở lên): số ngẫu nhiên
  /// đều trong `[0, trần]`, trần là `min(maxDelay, baseDelay * 2^(attempt-2))`.
  Duration backoff(int attempt) {
    assert(attempt >= 2, 'lần thử đầu không có khoảng chờ');
    final doublings = min(attempt - 2, 30);
    final ceiling = min(
      maxDelay.inMicroseconds,
      baseDelay.inMicroseconds * (1 << doublings),
    );
    return Duration(microseconds: (ceiling * _random()).round());
  }

  bool _isTransient(DioException error) {
    switch (error.type) {
      case DioExceptionType.connectionTimeout:
      case DioExceptionType.sendTimeout:
      case DioExceptionType.receiveTimeout:
        return true;
      case DioExceptionType.badResponse:
        return retryStatusCodes.contains(error.response?.statusCode);
      case DioExceptionType.transformTimeout:
      case DioExceptionType.cancel:
        return false;
      case DioExceptionType.connectionError:
      case DioExceptionType.badCertificate:
      case DioExceptionType.unknown:
        return networkReasonOf(error) == NetworkReason.unreachable;
    }
  }

  bool _shouldRetry(DioException error) =>
      _isTransient(error) && isRepeatable(error.requestOptions);

  /// `Retry-After` dạng số giây của response lỗi, nếu có.
  Duration? _retryAfter(DioException error) {
    final raw = error.response?.headers.value('retry-after');
    final seconds = raw == null ? null : int.tryParse(raw.trim());
    return seconds == null || seconds < 0 ? null : Duration(seconds: seconds);
  }

  /// Chặn [timeout] bằng [remaining]; không có hạn riêng thì dùng [remaining].
  Duration _clamp(Duration? timeout, Duration remaining) =>
      timeout == null || timeout == Duration.zero || timeout > remaining
      ? remaining
      : timeout;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    // Chỉ lần gửi đầu: nhớ lúc bắt đầu và bản sao của request lúc đi vào đây.
    if (!options.extra.containsKey(_startKey)) {
      final snapshot = options.copyWith();
      options.extra[_startKey] = _now();
      options.extra[_snapshotKey] = snapshot;
    }
    handler.next(options);
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    final options = err.requestOptions;
    // Lần gửi lại do chính vòng lặp dưới đây gây ra: để vòng lặp quyết định, và
    // đừng cho interceptor đứng sau thấy lỗi trung gian này.
    if (options.extra[_insideKey] == true) {
      handler.reject(err);
      return;
    }
    if (!_shouldRetry(err)) {
      handler.next(err);
      return;
    }
    final started = options.extra[_startKey];
    final snapshot = options.extra[_snapshotKey];
    final original = snapshot is RequestOptions ? snapshot : options;
    var last = err;
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
      onRetry?.call(RetryEvent(attempt: attempt, wait: wait, error: last));
      await _sleep(wait);
      final cancelled = options.cancelToken?.cancelError;
      if (cancelled != null) {
        handler.next(cancelled);
        return;
      }
      var next = original.copyWith(
        extra: {...original.extra, _insideKey: true},
      );
      if (remaining != null) {
        next = next.copyWith(
          connectTimeout: _clamp(next.connectTimeout, remaining),
          sendTimeout: _clamp(next.sendTimeout, remaining),
          receiveTimeout: _clamp(next.receiveTimeout, remaining),
        );
      }
      try {
        final response = await dio.fetch<dynamic>(next);
        handler.resolve(response);
        return;
      } on DioException catch (e) {
        last = e;
        if (!_shouldRetry(e)) {
          break;
        }
      }
    }
    handler.next(last);
  }
}

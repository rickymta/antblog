import 'dart:async';
import 'dart:convert';
import 'dart:io' show SocketException;
import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:shelf/shelf.dart';

/// Một request mà [InProcessAdapter] đã nhận, để test kiểm đúng thứ client gửi.
final class RecordedRequest {
  const RecordedRequest({
    required this.method,
    required this.uri,
    required this.headers,
    required this.body,
  });

  final String method;
  final Uri uri;

  /// Tên header chữ thường -> giá trị.
  final Map<String, String> headers;

  /// Body đã giải UTF-8 (chuỗi rỗng nếu không có body).
  final String body;
}

/// `HttpClientAdapter` giả: thay vì gửi byte qua socket, nó chuyển request cho
/// một shelf [Handler] chạy ngay trong tiến trình (thường là
/// `createFakeApiHandler()` của `toi_noi_fake_api`).
///
/// Dùng được cả trong `dart test` lẫn `flutter test`. Khi binding của
/// `flutter_test` đã khởi tạo (trong `testWidgets` hoặc sau
/// `TestWidgetsFlutterBinding.ensureInitialized()`), nó thay `HttpClient`
/// toàn cục bằng bản luôn trả 400 và không mở kết nối nào, nên `dio` với
/// adapter mặc định không gọi được API thật. Còn `test()` thuần mà chưa khởi
/// tạo binding thì vẫn mở socket thật. Adapter này không đi qua `HttpClient`,
/// nên chạy giống nhau trong cả hai trường hợp.
///
/// Giống adapter thật: nếu request đặt `receiveTimeout` và handler chưa trả lời
/// kịp thì ném `DioException.receiveTimeout`; nếu handler ném `SocketException`
/// thì ném `DioException.connectionError`.
final class InProcessAdapter implements HttpClientAdapter {
  InProcessAdapter(this.handler);

  final Handler handler;
  final List<RecordedRequest> _recorded = [];

  /// Các request đã nhận theo thứ tự.
  List<RecordedRequest> get requests => List.unmodifiable(_recorded);

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    final bodyBytes = <int>[];
    if (requestStream != null) {
      await for (final chunk in requestStream) {
        bodyBytes.addAll(chunk);
      }
    }
    final headers = <String, String>{
      for (final entry in options.headers.entries)
        if (entry.value != null) entry.key.toLowerCase(): '${entry.value}',
    };
    _recorded.add(
      RecordedRequest(
        method: options.method,
        uri: options.uri,
        headers: headers,
        body: utf8.decode(bodyBytes),
      ),
    );

    var pending =
        Future.sync(
          () => handler(
            Request(
              options.method,
              options.uri,
              headers: headers,
              body: bodyBytes,
            ),
          ),
        ).catchError((Object error, StackTrace stack) {
          if (error is SocketException) {
            throw DioException.connectionError(
              requestOptions: options,
              reason: error.toString(),
              error: error,
              stackTrace: stack,
            );
          }
          Error.throwWithStackTrace(error, stack);
        });
    final timeout = options.receiveTimeout;
    if (timeout != null && timeout > Duration.zero) {
      pending = pending.timeout(
        timeout,
        onTimeout: () => throw DioException.receiveTimeout(
          timeout: timeout,
          requestOptions: options,
        ),
      );
    }
    final response = await pending;
    final bytes = await response.read().expand((chunk) => chunk).toList();
    return ResponseBody.fromBytes(
      bytes,
      response.statusCode,
      headers: {
        for (final entry in response.headers.entries) entry.key: [entry.value],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

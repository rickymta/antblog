import 'dart:async';
// Chỉ nhắm iOS và Android (xem failure.dart), nên import trực tiếp dart:io.
import 'dart:io' show HandshakeException, HttpException;
import 'dart:typed_data';

import 'package:dio/dio.dart';

/// Một lỗi tầng mạng mà [FaultyAdapter] có thể giả lập. Fake API chỉ tiêm
/// được lỗi 500 và độ trễ ở tầng HTTP; các lỗi dưới đây xảy ra trước khi có
/// response nên phải giả lập ở tầng adapter.
enum NetworkFault {
  /// Server từ chối kết nối. Request không tới được server.
  refuse,

  /// Không mở được kết nối: chờ đúng `connectTimeout` của request rồi ném
  /// `connectionTimeout`. Request không tới được server.
  connectTimeout,

  /// Server không bao giờ trả lời: chờ đúng `receiveTimeout` rồi ném
  /// `receiveTimeout`. Ngoài đời request có thể đã tới server; adapter giả
  /// không chuyển nó cho adapter bên trong, nên kho dữ liệu của test không đổi.
  hang,

  /// Ca xấu nhất: server xử lý xong (adapter bên trong chạy thật) nhưng kết
  /// nối đứt trước khi response về tới máy, nên client nhận `connectionError`.
  loseResponse,

  /// Kết nối bị reset giữa body: server xử lý xong (adapter bên trong chạy
  /// thật), client nhận `DioException` kiểu `unknown` mang `HttpException`,
  /// đúng thứ adapter thật của dio ném khi kết nối đứt giữa chừng (đã thử với
  /// socket thật ở localhost). Khác [loseResponse], cái ném `connectionError`.
  reset,

  /// Bắt tay TLS hỏng: `DioException` kiểu `unknown` mang `HandshakeException`,
  /// như khi chứng chỉ tự ký không được tin cậy. Request không tới server.
  tlsFailure,
}

/// Bọc một [HttpClientAdapter] và chèn lỗi tầng mạng theo kịch bản.
///
/// [script] là danh sách lỗi theo thứ tự các lần gọi: phần tử thứ i áp cho lần
/// gọi thứ i; phần tử `null` hoặc hết kịch bản thì gọi thẳng [inner].
///
/// Thời gian chờ của `connectTimeout` và `hang` đi qua `Future.delayed`, nên
/// dùng được với `fake_async` để test không chờ thật.
final class FaultyAdapter implements HttpClientAdapter {
  FaultyAdapter(this.inner, {Iterable<NetworkFault?> script = const []})
    : script = List.unmodifiable(script);

  final HttpClientAdapter inner;
  final List<NetworkFault?> script;

  int _calls = 0;

  /// Số lần `fetch` đã được gọi (kể cả lần bị tiêm lỗi).
  int get calls => _calls;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    final index = _calls++;
    final fault = index < script.length ? script[index] : null;
    switch (fault) {
      case null:
        return inner.fetch(options, requestStream, cancelFuture);
      case NetworkFault.refuse:
        throw DioException.connectionError(
          requestOptions: options,
          reason: 'kết nối bị từ chối (giả lập)',
        );
      case NetworkFault.connectTimeout:
        final limit = options.connectTimeout;
        await _waitOut(limit);
        throw DioException.connectionTimeout(
          timeout: limit ?? Duration.zero,
          requestOptions: options,
        );
      case NetworkFault.hang:
        final limit = options.receiveTimeout;
        await _waitOut(limit);
        throw DioException.receiveTimeout(
          timeout: limit ?? Duration.zero,
          requestOptions: options,
        );
      case NetworkFault.loseResponse:
        await inner.fetch(options, requestStream, cancelFuture);
        throw DioException.connectionError(
          requestOptions: options,
          reason: 'kết nối đứt trước khi nhận response (giả lập)',
        );
      case NetworkFault.reset:
        await inner.fetch(options, requestStream, cancelFuture);
        throw DioException(
          requestOptions: options,
          error: HttpException(
            'Connection closed while receiving data (giả lập)',
            uri: options.uri,
          ),
        );
      case NetworkFault.tlsFailure:
        throw DioException(
          requestOptions: options,
          error: const HandshakeException(
            'Connection terminated during handshake',
          ),
        );
    }
  }

  /// Chờ hết [limit]; không có hạn thì chờ mãi, giống mạng thật.
  Future<void> _waitOut(Duration? limit) =>
      limit == null || limit <= Duration.zero
      ? Completer<void>().future
      : Future<void>.delayed(limit);

  @override
  void close({bool force = false}) => inner.close(force: force);
}

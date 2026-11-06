import 'dart:async';
import 'dart:io';

import 'package:web_socket_channel/io.dart';

/// Một kết nối đã mở. Phần của `web_socket_channel` mà client cần, gói thành
/// interface để test thay bằng bản giả, không cần socket.
abstract interface class RealtimeConnection {
  /// Các text frame server gửi tới. Kết thúc (done) khi kết nối đóng; có thể
  /// phát lỗi trước đó.
  Stream<String> get incoming;

  /// Gửi một text frame. Ném [StateError] nếu kết nối đã đóng.
  void send(String text);

  /// Mã đóng của phía bên kia; chỉ có nghĩa sau khi [incoming] đã kết thúc.
  int? get closeCode;

  Future<void> close([int? code, String? reason]);
}

/// Cách mở một [RealtimeConnection] tới [url] với access token [token].
abstract interface class RealtimeTransport {
  /// Hoàn tất khi bắt tay WebSocket xong, ném lỗi khi không nối được.
  Future<RealtimeConnection> connect(Uri url, {required String token});
}

/// Gateway từ chối token ngay ở bước nâng cấp bằng HTTP 401. Cùng một token thì
/// lần nào cũng bị từ chối nên client không nối lại.
final class RealtimeAuthRejected implements Exception {
  const RealtimeAuthRejected();

  @override
  String toString() => 'RealtimeAuthRejected (HTTP 401 ở bước nâng cấp)';
}

/// Cài đặt thật trên `dart:io` (iOS, Android, desktop).
final class WebSocketTransport implements RealtimeTransport {
  const WebSocketTransport({this.connectTimeout = const Duration(seconds: 10)});

  /// Chờ bắt tay tối đa chừng này; quá hạn thì `connect` ném `TimeoutException`
  /// và socket nào tới muộn sau đó bị đóng ngay.
  final Duration connectTimeout;

  @override
  Future<RealtimeConnection> connect(Uri url, {required String token}) async {
    final connecting = WebSocket.connect(
      url.toString(),
      headers: {'Authorization': 'Bearer $token'},
    );
    final WebSocket socket;
    try {
      socket = await connecting.timeout(connectTimeout);
    } on TimeoutException {
      // `Future.timeout` không huỷ việc đang chạy: nếu không đóng, socket tới
      // muộn sẽ mở mà không ai giữ.
      unawaited(
        connecting.then(
          (lateSocket) => lateSocket.close(),
          onError: (Object _) {},
        ),
      );
      rethrow;
    } on WebSocketException catch (e) {
      if (e.httpStatusCode == 401) throw const RealtimeAuthRejected();
      rethrow;
    }
    return _ChannelConnection(IOWebSocketChannel(socket));
  }
}

final class _ChannelConnection implements RealtimeConnection {
  _ChannelConnection(this._channel);

  final IOWebSocketChannel _channel;

  @override
  Stream<String> get incoming =>
      _channel.stream.where((data) => data is String).cast<String>();

  @override
  void send(String text) => _channel.sink.add(text);

  @override
  int? get closeCode => _channel.closeCode;

  @override
  Future<void> close([int? code, String? reason]) =>
      _channel.sink.close(code, reason);
}

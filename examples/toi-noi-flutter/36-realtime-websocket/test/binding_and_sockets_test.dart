// Ghi lại điều quan sát được với Flutter 3.47.4 / Dart 3.13.3 khi có
// TestWidgetsFlutterBinding. Đây là hành vi hiện tại, không phải hợp đồng của
// SDK: lý do để test socket thật nằm ở một file Dart thuần (real_socket_test).
//
// Cơ chế (suy từ source, không có tài liệu chính thức): `WebSocket.connect` mặc
// định dùng một `HttpClient` lưu trong biến static của dart:io. Phần khởi động
// của `flutter test` (flutter_tools, flutter_platform.dart) gọi
// `WebSocket.connect` để nối với tiến trình điều khiển, trước khi binding đặt
// `HttpOverrides`, nên biến static đó giữ một client thật.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_transport.dart';

void main() {
  testWidgets('HttpClient() trong testWidgets là bản giả luôn trả 400', (
    tester,
  ) async {
    final status = await tester.runAsync(() async {
      final client = HttpClient();
      final request = await client.getUrl(Uri.parse('http://127.0.0.1:1/'));
      final response = await request.close();
      return response.statusCode;
    });

    expect(status, 400);
  });

  testWidgets(
    'WebSocket.connect mặc định dùng HttpClient tạo trước khi binding đặt override: vẫn là socket thật',
    (tester) async {
      final failure = await tester.runAsync(
        () => const WebSocketTransport()
            .connect(Uri.parse('ws://127.0.0.1:1/realtime'), token: 'x')
            .then<Object?>((_) => null, onError: (Object e) => e),
      );

      // Cổng 1 không ai nghe: lỗi là "Connection refused" của hệ điều hành, không
      // phải mã 400 của HttpClient giả.
      expect(failure, isA<SocketException>());
    },
  );
}

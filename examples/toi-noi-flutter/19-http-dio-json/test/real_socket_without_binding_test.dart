import 'package:dio/dio.dart' hide Response;
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;

/// Đối chứng cho `controllers_test.dart`: `flutter test` chỉ trả 400 khi
/// binding của flutter_test đã khởi tạo (`testWidgets` hoặc
/// `TestWidgetsFlutterBinding.ensureInitialized()`). File này không làm cả hai,
/// nên `test()` thuần vẫn mở socket thật tới server loopback.
void main() {
  test('chưa khởi tạo binding: dio mở socket thật, không bị chặn', () async {
    final server = await shelf_io.serve(
      (Request request) => Response.ok('xin chào từ socket thật'),
      'localhost',
      0,
    );
    addTearDown(() => server.close(force: true));

    final response = await Dio().get<String>(
      'http://localhost:${server.port}/health',
    );

    expect(response.statusCode, 200);
    expect(response.data, 'xin chào từ socket thật');
  });
}

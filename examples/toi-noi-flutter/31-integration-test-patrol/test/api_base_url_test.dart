import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_31/api_base_url.dart';

void main() {
  group('resolveApiBaseUrl', () {
    test('Android emulator gọi máy dev qua 10.0.2.2', () {
      final url = resolveApiBaseUrl(platform: TargetPlatform.android);
      expect(url.toString(), 'http://10.0.2.2:8787');
    });

    test('iOS Simulator gọi localhost', () {
      final url = resolveApiBaseUrl(platform: TargetPlatform.iOS);
      expect(url.toString(), 'http://localhost:8787');
    });

    test('API_BASE_URL thắng nền tảng (thiết bị thật dùng địa chỉ LAN)', () {
      final url = resolveApiBaseUrl(
        defined: 'http://192.168.1.20:8787',
        platform: TargetPlatform.android,
      );
      expect(url.host, '192.168.1.20');
    });

    test(
      'không có define trong flutter test thì dùng mặc định theo nền tảng',
      () {
        // Bản `flutter test` thường không truyền --dart-define=API_BASE_URL.
        expect(
          resolveApiBaseUrl(platform: TargetPlatform.iOS).port,
          fakeApiPort,
        );
      },
    );
  });
}

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show ProviderException;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_25/config/app_config.dart';
import 'package:toi_noi_25/providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/config_files.dart';
import 'support/harness.dart';

AppConfig _configOf(String name) => AppConfig.fromMap(readFlavorDefines(name));

void main() {
  group('appConfigProvider', () {
    test('quên ghi đè thì đọc là lỗi ngay, nêu cách sửa', () {
      final container = ProviderContainer.test();
      expect(
        () => container.read(appConfigProvider),
        throwsA(
          isA<ProviderException>().having(
            (e) => e.exception,
            'lỗi gốc',
            isA<StateError>().having(
              (e) => e.message,
              'message',
              contains('overrideWithValue'),
            ),
          ),
        ),
      );
    });

    test('ghi đè bằng cấu hình nào thì đọc ra đúng cấu hình đó', () {
      for (final name in flavorFileNames) {
        final config = _configOf(name);
        final container = Harness().containerFor(config);
        expect(container.read(appConfigProvider), same(config));
      }
    });
  });

  group('apiClientProvider theo từng flavor', () {
    for (final name in flavorFileNames) {
      final config = _configOf(name);

      test('$name: địa chỉ và timeout lấy từ cấu hình', () {
        final client = Harness().containerFor(config).read(apiClientProvider);
        expect(client.baseUrl, config.apiBaseUrl.toString());
        expect(client.dio.options.connectTimeout, config.connectTimeout);
        expect(client.dio.options.receiveTimeout, config.receiveTimeout);
      });

      test('$name: log HTTP có mặt khi và chỉ khi LOG_HTTP bật', () {
        final client = Harness().containerFor(config).read(apiClientProvider);
        expect(
          client.dio.interceptors.any((i) => i is SafeLogInterceptor),
          config.logHttp,
        );
      });

      test(
        '$name: cùng một đoạn code báo giá đi tới đúng host của flavor',
        () async {
          final harness = Harness();
          final container = harness.containerFor(config);
          addTearDown(() => container.dispose());

          await container.read(sampleQuoteProvider.notifier).fetch();

          final request = harness.adapter.requests.single;
          expect(request.uri.host, config.apiBaseUrl.host);
          expect(request.uri.scheme, config.apiBaseUrl.scheme);
          expect(request.uri.path, '/quotes');
          // 5 km là 25.500 VND theo bảng giá minh hoạ, tính bằng calculateFare.
          expect(container.read(sampleQuoteProvider).value, calculateFare(5));
          expect(container.read(sampleQuoteProvider).value, const Money(25500));
        },
      );
    }

    test('dev ghi log request, prod thì không ghi dòng nào', () async {
      final devHarness = Harness();
      final dev = devHarness.containerFor(_configOf('dev'));
      await dev.read(sampleQuoteProvider.notifier).fetch();
      expect(devHarness.logs, isNotEmpty);
      expect(devHarness.logs.first, startsWith('--> POST'));

      final prodHarness = Harness();
      final prod = prodHarness.containerFor(_configOf('prod'));
      await prod.read(sampleQuoteProvider.notifier).fetch();
      expect(prodHarness.logs, isEmpty);
    });

    test('đổi cấu hình trong container mới cho client mới, không dùng lại client cũ', () {
      final a = Harness()
          .containerFor(_configOf('dev'))
          .read(apiClientProvider);
      final b = Harness()
          .containerFor(_configOf('prod'))
          .read(apiClientProvider);
      expect(a.baseUrl, isNot(b.baseUrl));
    });
  });
}

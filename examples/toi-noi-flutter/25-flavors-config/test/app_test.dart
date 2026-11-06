import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show appFlavor;
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Response;
import 'package:toi_noi_25/bootstrap.dart';
import 'package:toi_noi_25/config/app_config.dart';
import 'package:toi_noi_25/config_error_app.dart';
import 'package:toi_noi_25/environment_screen.dart';

import 'support/config_files.dart';
import 'support/harness.dart';

AppConfig _configOf(String name) => AppConfig.fromMap(readFlavorDefines(name));

/// Dòng `label` của màn hình chẩn đoán với giá trị `value`.
Finder _row(String label, String value) => find.byWidgetPredicate(
  (widget) =>
      widget is ListTile &&
      (widget.title as Text).data == label &&
      (widget.subtitle as Text).data == value,
);

Future<Harness> _pumpRoot(
  WidgetTester tester,
  AppConfig config, {
  Harness? harness,
}) async {
  final h = harness ?? Harness();
  await tester.pumpWidget(
    buildRoot(
      loadConfig: () => config,
      // buildRoot đã ghi đè appConfigProvider; ở đây chỉ thêm adapter và log.
      overrides: h.overridesFor(config).skip(1).toList(),
    ),
  );
  return h;
}

void main() {
  group('buildRoot với cấu hình hợp lệ', () {
    for (final name in flavorFileNames) {
      testWidgets('$name: màn hình chẩn đoán hiện đúng cấu hình', (
        tester,
      ) async {
        final config = _configOf(name);
        await _pumpRoot(tester, config);

        expect(find.byType(EnvironmentScreen), findsOneWidget);
        expect(_row('Môi trường', name), findsOneWidget);
        expect(_row('API', config.apiBaseUrl.toString()), findsOneWidget);
        expect(
          _row('Log HTTP', config.logHttp ? 'bật' : 'tắt'),
          findsOneWidget,
        );
        expect(
          _row(
            'Timeout kết nối / phản hồi',
            '${config.connectTimeout.inMilliseconds} / '
                '${config.receiveTimeout.inMilliseconds} ms',
          ),
          findsOneWidget,
        );
        // `flutter test` thường: không có --flavor nên appFlavor là null.
        // `flutter test --flavor <tên>`: appFlavor là chính tên đó.
        expect(_row('Flavor native', appFlavor ?? 'không đặt'), findsOneWidget);
      });
    }

    for (final name in flavorFileNames) {
      testWidgets(
        '$name: ruy-băng tên môi trường chỉ có ở bản không phải prod',
        (tester) async {
          await _pumpRoot(tester, _configOf(name));

          // MaterialApp trong debug còn vẽ một Banner "DEBUG", nên lọc theo chữ.
          final envBanner = find.byWidgetPredicate(
            (widget) =>
                widget is Banner && widget.message == name.toUpperCase(),
          );
          expect(envBanner, name == 'prod' ? findsNothing : findsOneWidget);
          expect(
            tester.widget<MaterialApp>(find.byType(MaterialApp)).title,
            name == 'prod' ? 'Tới Nơi' : 'Tới Nơi $name',
          );
        },
      );
    }

    testWidgets('bấm báo giá thử: 25.500 VND từ máy chủ của flavor', (
      tester,
    ) async {
      final config = _configOf('staging');
      final harness = await _pumpRoot(tester, config);

      await tester.tap(find.text('Báo giá thử 5 km'));
      await tester.pumpAndSettle();

      expect(find.text('25.500 VND'), findsOneWidget);
      expect(
        harness.adapter.requests.single.uri.host,
        'api.staging.toinoi.example',
      );
    });

    testWidgets('máy chủ lỗi thì hiện loại lỗi, không sập', (tester) async {
      final harness = Harness((request) => Response.internalServerError());
      await _pumpRoot(tester, _configOf('dev'), harness: harness);

      await tester.tap(find.text('Báo giá thử 5 km'));
      await tester.pumpAndSettle();

      expect(find.textContaining('Không báo giá được'), findsOneWidget);
    });
  });

  group('buildRoot với cấu hình sai', () {
    testWidgets('hiện ConfigErrorApp liệt kê từng lỗi, không dựng app', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildRoot(loadConfig: () => AppConfig.fromMap(const {})),
      );

      expect(find.byType(ConfigErrorApp), findsOneWidget);
      expect(find.byType(EnvironmentScreen), findsNothing);
      expect(find.text('Cấu hình không hợp lệ'), findsOneWidget);
      for (final key in AppConfig.keys) {
        expect(find.textContaining(key), findsWidgets, reason: key);
      }
      expect(
        find.byIcon(Icons.error_outline),
        findsNWidgets(AppConfig.keys.length),
      );
    });

    testWidgets('prod trỏ localhost: màn hình lỗi nói rõ lý do', (
      tester,
    ) async {
      final env = readFlavorDefines('prod')
        ..[AppConfig.keyApiBaseUrl] = 'http://localhost:8787';
      await tester.pumpWidget(
        buildRoot(loadConfig: () => AppConfig.fromMap(env)),
      );

      expect(find.textContaining('prod phải dùng https'), findsOneWidget);
      expect(
        find.textContaining('không được trỏ API_BASE_URL tới máy cục bộ'),
        findsOneWidget,
      );
    });

    testWidgets('lỗi không phải ConfigException không bị nuốt', (tester) async {
      expect(
        () => buildRoot(loadConfig: () => throw StateError('lỗi lạ')),
        throwsStateError,
      );
    });
  });
}

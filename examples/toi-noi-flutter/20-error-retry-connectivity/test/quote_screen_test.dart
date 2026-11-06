import 'dart:convert';

import 'package:dio/dio.dart' hide Response;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Response;
import 'package:toi_noi_20/app.dart';
import 'package:toi_noi_20/connectivity.dart';
import 'package:toi_noi_20/failure_view.dart';
import 'package:toi_noi_20/system_settings.dart';
import 'package:toi_noi_20/providers.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

Future<void> _pumpApp(WidgetTester tester, Harness harness) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: harness.overrides,
      retry: noProviderRetry,
      child: const ToiNoiApp(),
    ),
  );
  await tester.pump(); // cho linkProvider đọc giá trị đầu
}

Future<void> _tapQuote(WidgetTester tester) async {
  await tester.tap(find.text('Báo giá'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('lỗi tạm thời được interceptor nuốt: người dùng chỉ thấy giá', (
    tester,
  ) async {
    final harness = Harness(faults: const FaultConfig(failFirst: 2));
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.text('Thử lại'), findsNothing);
    expect(harness.inProcess.requests, hasLength(3));
  });

  testWidgets('5xx hết lượt: câu tiếng Việt, mã hỗ trợ, nút Thử lại', (
    tester,
  ) async {
    final harness = Harness(faults: const FaultConfig(failFirst: 3));
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(find.textContaining('Máy chủ đang gặp sự cố'), findsOneWidget);
    expect(find.text('Mã hỗ trợ: fake-000003'), findsOneWidget);
    expect(find.text('Thử lại'), findsOneWidget);
    expect(find.text('Mở cài đặt'), findsNothing);
    // Không lộ title tiếng Anh của server.
    expect(find.textContaining('Unexpected'), findsNothing);
    expect(harness.inProcess.requests, hasLength(3));

    // Server đã hồi phục (request 4 thành công): bấm Thử lại.
    await tester.tap(find.text('Thử lại'));
    await tester.pumpAndSettle();
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.text('Thử lại'), findsNothing);
    expect(harness.inProcess.requests, hasLength(4));
  });

  testWidgets('bấm Thử lại khi vẫn hỏng: thêm một vòng đủ ba request', (
    tester,
  ) async {
    final harness = Harness(faults: const FaultConfig(failFirst: 99));
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(harness.inProcess.requests, hasLength(3));
    await tester.tap(find.text('Thử lại'));
    await tester.pumpAndSettle();
    expect(harness.inProcess.requests, hasLength(6));
    expect(find.text('Thử lại'), findsOneWidget);
  });

  testWidgets('mất mạng: có Mở cài đặt, bấm thì gọi SystemSettings', (
    tester,
  ) async {
    final harness = Harness(
      script: [NetworkFault.refuse, NetworkFault.refuse, NetworkFault.refuse],
    );
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(find.text('Thử lại'), findsOneWidget);
    expect(find.text('Mở cài đặt'), findsOneWidget);
    await tester.tap(find.text('Mở cài đặt'));
    await tester.pump();
    expect(harness.settings.opened, 1);
  });

  testWidgets(
    'connectivity báo none: banner và câu nói thẳng là không có mạng',
    (tester) async {
      final harness = Harness(
        link: LinkType.none,
        script: [NetworkFault.refuse, NetworkFault.refuse, NetworkFault.refuse],
      );
      await _pumpApp(tester, harness);
      expect(find.text('Thiết bị đang không có kết nối mạng.'), findsOneWidget);
      await _tapQuote(tester);
      expect(
        find.textContaining('không có kết nối mạng. Bật Wi-Fi'),
        findsOneWidget,
      );
    },
  );

  testWidgets(
    'connectivity báo wifi nhưng không gọi được: không có banner, câu khác',
    (tester) async {
      final harness = Harness(
        link: LinkType.wifi,
        script: [NetworkFault.refuse, NetworkFault.refuse, NetworkFault.refuse],
      );
      await _pumpApp(tester, harness);
      await _tapQuote(tester);
      expect(find.text('Thiết bị đang không có kết nối mạng.'), findsNothing);
      expect(find.textContaining('Không tới được máy chủ'), findsOneWidget);
      expect(find.text('Mở cài đặt'), findsOneWidget);
    },
  );

  testWidgets(
    'connectivity báo none nhưng server gọi được: vẫn gửi và vẫn thành công',
    (tester) async {
      final harness = Harness(link: LinkType.none);
      await _pumpApp(tester, harness);
      expect(find.text('Thiết bị đang không có kết nối mạng.'), findsOneWidget);
      await _tapQuote(tester);
      // Ta không chặn request theo connectivity vì nó chỉ là gợi ý.
      expect(find.text('25.500 VND'), findsOneWidget);
      expect(harness.inProcess.requests, hasLength(1));
    },
  );

  testWidgets('banner đi theo luồng thay đổi của connectivity', (tester) async {
    final harness = Harness();
    await _pumpApp(tester, harness);
    const banner = 'Thiết bị đang không có kết nối mạng.';
    expect(find.text(banner), findsNothing);
    harness.connectivity.emit(LinkType.none);
    await tester.pumpAndSettle();
    expect(find.text(banner), findsOneWidget);
    harness.connectivity.emit(LinkType.cellular);
    await tester.pumpAndSettle();
    expect(find.text(banner), findsNothing);
  });

  testWidgets('422 ngoài miền: câu rõ ràng và không có nút nào', (
    tester,
  ) async {
    // Thanh trượt không cho kéo quá 100 km, nên server được dựng để trả 422.
    final harness = Harness.custom(
      (request) => Response(
        422,
        body: jsonEncode({
          'title': 'Unprocessable request',
          'status': 422,
          'errors': {
            'distanceKm': ['Expected 0..100 km'],
          },
          'traceId': 'fake-000009',
        }),
        headers: const {'content-type': 'application/problem+json'},
      ),
    );
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(
      find.text('Quãng đường phải nằm trong khoảng 0 đến 100 km.'),
      findsOneWidget,
    );
    expect(find.text('Thử lại'), findsNothing);
    expect(find.text('Mở cài đặt'), findsNothing);
    expect(find.textContaining('Mã hỗ trợ'), findsNothing);
    expect(harness.inProcess.requests, hasLength(1));
  });

  testWidgets(
    'Wi-Fi công cộng trả HTML với mã 200: ParseFailure, nhắc đăng nhập Wi-Fi',
    (tester) async {
      final harness = Harness.custom(
        (request) => Response.ok(
          '<html>Đăng nhập Wi-Fi</html>',
          headers: {'content-type': 'text/html'},
        ),
        link: LinkType.wifi,
      );
      await _pumpApp(tester, harness);
      await _tapQuote(tester);
      expect(find.textContaining('Wi-Fi công cộng'), findsOneWidget);
      expect(find.text('Thử lại'), findsOneWidget);
      // Connectivity vẫn nói "có Wi-Fi": đó là lý do nó không đủ để biết có Internet.
      expect(find.text('Thiết bị đang không có kết nối mạng.'), findsNothing);
    },
  );

  testWidgets('đồng hồ giả: server treo, vòng chờ dừng ở hạn tổng 30 giây', (
    tester,
  ) async {
    final harness = Harness(
      script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
      realSleep: true,
    );
    await _pumpApp(tester, harness);
    await tester.tap(find.text('Báo giá'));
    await tester.pump();
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    await tester.pump(const Duration(seconds: 29));
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    await tester.pump(const Duration(seconds: 1, milliseconds: 100));
    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(find.textContaining('phản hồi quá chậm'), findsOneWidget);
    expect(find.text('Thử lại'), findsOneWidget);
    expect(find.text('Mở cài đặt'), findsNothing);
    expect(harness.faulty.calls, 3);
  });

  testWidgets('đặt chuyến: mất response, interceptor gửi lại, một chuyến', (
    tester,
  ) async {
    final harness = Harness(script: [NetworkFault.loseResponse]);
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();
    expect(find.text('Chuyến T-1'), findsOneWidget);
    expect(harness.store.trips, hasLength(1));
  });

  testWidgets(
    'đặt chuyến thất bại rồi Thử lại: không làm mới nhầm, một chuyến',
    (tester) async {
      // Request 1 là báo giá. Đặt chuyến gồm request 2-4 (500) rồi lần bấm lại.
      final harness = Harness(
        wrap: (fake) {
          var calls = 0;
          return (request) {
            calls++;
            return calls >= 2 && calls <= 4
                ? Response(500, body: '{"title":"x","status":500}')
                : fake(request);
          };
        },
      );
      await _pumpApp(tester, harness);
      await _tapQuote(tester);
      await tester.tap(find.text('Đặt chuyến'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Máy chủ đang gặp sự cố'), findsOneWidget);
      await tester.tap(find.text('Thử lại'));
      await tester.pumpAndSettle();
      expect(find.text('Chuyến T-1'), findsOneWidget);
      expect(harness.store.trips, hasLength(1));
      expect(harness.keysIssued, 1);
    },
  );

  testWidgets(
    'kết nối bị reset giữa body (dio để ở unknown): vẫn là lỗi mạng có hai nút',
    (tester) async {
      final harness = Harness(
        script: [NetworkFault.reset, NetworkFault.reset, NetworkFault.reset],
      );
      await _pumpApp(tester, harness);
      await _tapQuote(tester);
      expect(find.textContaining('Không tới được máy chủ'), findsOneWidget);
      expect(find.text('Thử lại'), findsOneWidget);
      expect(find.text('Mở cài đặt'), findsOneWidget);
      expect(find.textContaining('không mong muốn'), findsNothing);
      expect(harness.faulty.calls, 3);
    },
  );

  testWidgets('lỗi TLS: nói kết nối không an toàn, không nút, không thử lại', (
    tester,
  ) async {
    final harness = Harness(script: [NetworkFault.tlsFailure]);
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(find.textContaining('không an toàn'), findsOneWidget);
    expect(find.text('Thử lại'), findsNothing);
    expect(find.text('Mở cài đặt'), findsNothing);
    expect(harness.faulty.calls, 1);
  });

  testWidgets('429 hết lượt: câu riêng, mã hỗ trợ và nút Thử lại', (
    tester,
  ) async {
    final harness = Harness.custom(
      (request) => Response(
        429,
        body: '{"title":"x","status":429,"traceId":"fake-000042"}',
        headers: const {'content-type': 'application/problem+json'},
      ),
    );
    await _pumpApp(tester, harness);
    await _tapQuote(tester);
    expect(find.textContaining('quá nhiều yêu cầu'), findsOneWidget);
    expect(find.text('Mã hỗ trợ: fake-000042'), findsOneWidget);
    expect(find.text('Thử lại'), findsOneWidget);
  });

  testWidgets('nhãn nút lấy từ SystemSettings (iOS nói là cài đặt ứng dụng)', (
    tester,
  ) async {
    final harness = Harness(
      script: [NetworkFault.refuse, NetworkFault.refuse, NetworkFault.refuse],
    );
    await tester.pumpWidget(
      ProviderScope(
        overrides: harness.overrides,
        retry: noProviderRetry,
        child: ProviderScope(
          overrides: [
            systemSettingsProvider.overrideWithValue(
              const AppSettingsOpener(platform: TargetPlatform.iOS),
            ),
          ],
          child: const ToiNoiApp(),
        ),
      ),
    );
    await tester.pump();
    await _tapQuote(tester);
    expect(find.text('Mở cài đặt ứng dụng'), findsOneWidget);
    expect(find.text('Mở cài đặt'), findsNothing);
  });

  testWidgets('FailureView không vẽ gì cho lỗi huỷ request', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: Harness().overrides,
        child: MaterialApp(
          home: Scaffold(
            body: FailureView(
              error: DioException.requestCancelled(
                requestOptions: RequestOptions(path: '/x'),
                reason: 'đổi ý',
              ),
              onRetry: () {},
            ),
          ),
        ),
      ),
    );
    expect(find.byType(Text), findsNothing);
    expect(find.byType(FilledButton), findsNothing);
  });
}

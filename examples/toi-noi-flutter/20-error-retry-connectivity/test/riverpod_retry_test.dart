import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_20/providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

/// Một provider gọi API ngay trong `build`: kiểu code mà Riverpod 3 tự thử lại.
final _autoQuote = FutureProvider<Money>(
  (ref) => guardApi(() async {
    final dto = await ref
        .watch(apiClientProvider)
        .quote(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        );
    return dto.toMoney();
  }),
);

/// Tiến đồng hồ giả của `testWidgets` thêm [total], từng giây một.
Future<void> _elapse(WidgetTester tester, Duration total) async {
  for (var i = 0; i < total.inSeconds; i++) {
    await tester.pump(const Duration(seconds: 1));
  }
}

void main() {
  const alwaysFail = FaultConfig(failFirst: 1000);

  testWidgets(
    'Riverpod mặc định chồng lên interceptor: một provider hỏng thành 33 request',
    (tester) async {
      final harness = Harness(faults: alwaysFail);
      // ProviderContainer thường, không truyền retry: dùng chính sách mặc định.
      final container = ProviderContainer(overrides: harness.overrides);
      addTearDown(container.dispose);
      container.listen(_autoQuote, (_, _) {});

      // 10 lần thử lại của Riverpod (200 ms nhân đôi, trần 6,4 s) trải trong
      // khoảng 38 giây; chờ dư để chắc đã hết.
      await _elapse(tester, const Duration(seconds: 60));
      expect(container.read(_autoQuote).error, isA<ServerFailure>());
      // (1 lần đầu + 10 lần Riverpod thử lại) * 3 lần của RetryInterceptor.
      expect(harness.inProcess.requests, hasLength(33));
    },
  );

  testWidgets('tắt retry của Riverpod thì chỉ còn ba request của interceptor', (
    tester,
  ) async {
    final harness = Harness(faults: alwaysFail);
    final container = ProviderContainer(
      overrides: harness.overrides,
      retry: noProviderRetry,
    );
    addTearDown(container.dispose);
    container.listen(_autoQuote, (_, _) {});

    await _elapse(tester, const Duration(seconds: 60));
    expect(container.read(_autoQuote).error, isA<ServerFailure>());
    expect(harness.inProcess.requests, hasLength(3));
  });

  testWidgets('4xx cũng bị Riverpod mặc định gọi lại, dù interceptor không', (
    tester,
  ) async {
    // Server nào cũng trả 422: interceptor không thử lại (1 request mỗi vòng),
    // nhưng Riverpod coi ClientFailure là Exception nên vẫn thử lại 10 lần.
    final harness = Harness.custom(
      (request) => okJson({'title': 'x', 'status': 422}, status: 422),
    );
    final container = ProviderContainer(overrides: harness.overrides);
    addTearDown(container.dispose);
    container.listen(_autoQuote, (_, _) {});

    await _elapse(tester, const Duration(seconds: 60));
    expect(container.read(_autoQuote).error, isA<ClientFailure>());
    expect(harness.inProcess.requests, hasLength(11));
  });

  testWidgets(
    'phương thức của notifier không bị Riverpod thử lại, kể cả retry mặc định',
    (tester) async {
      final harness = Harness(faults: alwaysFail);
      final container = ProviderContainer(overrides: harness.overrides);
      addTearDown(container.dispose);
      container.listen(quoteControllerProvider, (_, _) {});

      unawaited(container.read(quoteControllerProvider.notifier).fetch());
      await _elapse(tester, const Duration(seconds: 60));
      expect(
        container.read(quoteControllerProvider).error,
        isA<ServerFailure>(),
      );
      // Chỉ ba request của interceptor; không có vòng thử lại nào của Riverpod.
      expect(harness.inProcess.requests, hasLength(3));
    },
  );
}

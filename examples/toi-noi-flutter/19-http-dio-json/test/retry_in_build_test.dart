import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_19/providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Cái bẫy của Riverpod 3 mà bài 19 nhắc: provider ném `Exception` trong
/// `build` bị thử lại tự động, kể cả khi đó là một 422 do khách nhập sai.
void main() {
  const outOfRange = QuoteRequestDto(
    serviceKind: ServiceKind.ride,
    distanceKm: 100.5,
  );

  testWidgets('FutureProvider gọi API trong build: 422 vẫn bị gọi lại', (
    tester,
  ) async {
    final harness = Harness();
    final container = ProviderContainer(overrides: harness.overrides);
    final badQuote = FutureProvider<Money>((ref) async {
      final dto = await ref.watch(apiClientProvider).quote(outOfRange);
      return dto.toMoney();
    });

    container.listen(badQuote, (_, _) {});
    await tester.pumpAndSettle();
    expect(harness.adapter.requests, hasLength(1));
    expect(
      container.read(badQuote).error,
      isA<ApiException>().having((e) => e.statusCode, 'status', 422),
    );

    // Mặc định thử lại sau 200 ms (ProviderContainer.defaultRetry).
    await tester.pump(const Duration(milliseconds: 250));
    await tester.pumpAndSettle();
    expect(harness.adapter.requests, hasLength(2));

    // Huỷ container để các lần thử lại còn lại không để timer treo.
    container.dispose();
  });
}

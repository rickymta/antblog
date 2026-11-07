import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/payment_flow.dart';
import 'package:toi_noi_39/payment_providers.dart';
import 'package:toi_noi_39/payment_store.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/harness.dart';
import 'support/memory_gateway.dart';

void main() {
  late MemoryGateway gateway;
  late ProviderContainer container;

  setUp(() {
    gateway = MemoryGateway();
    container = ProviderContainer.test(
      overrides: [
        paymentGatewayProvider.overrideWithValue(gateway),
        paymentStoreProvider.overrideWithValue(InMemoryPaymentStore()),
        checkoutLauncherProvider.overrideWithValue(FakeLauncher()),
        paymentDelayProvider.overrideWithValue((_) async {}),
      ],
    );
  });

  test('state đi theo PaymentFlow', () async {
    final controller = container.read(paymentControllerProvider.notifier);
    container.listen(paymentControllerProvider, (_, _) {});

    await controller.pay(tripId: 'trip-1', amount: const Money(25500));

    expect(
      container.read(paymentControllerProvider),
      isA<PaymentAwaitingCheckout>(),
    );
  });

  test('invalidate controller: build() chạy lại, không ném LateInitializationError', () async {
    container.listen(paymentControllerProvider, (_, _) {});
    await container
        .read(paymentControllerProvider.notifier)
        .pay(tripId: 'trip-1', amount: const Money(25500));

    container.invalidate(paymentControllerProvider);

    expect(
      container.read(paymentControllerProvider),
      isA<PaymentAwaitingCheckout>(),
      reason: 'flow vẫn là flow cũ nên state được đọc lại nguyên vẹn',
    );
  });

  test(
    'dependency được watch đổi (flow mới): controller dựng lại bằng flow mới',
    () async {
      container.listen(paymentControllerProvider, (_, _) {});
      await container
          .read(paymentControllerProvider.notifier)
          .pay(tripId: 'trip-1', amount: const Money(25500));
      final oldFlow = container.read(paymentFlowProvider);

      container.invalidate(paymentFlowProvider);

      expect(container.read(paymentFlowProvider), isNot(same(oldFlow)));
      // Flow mới nạp lần thanh toán dang dở từ kho dùng chung và hỏi server.
      await pumpEventQueue();
      expect(
        container.read(paymentControllerProvider),
        isA<PaymentPendingConfirmation>(),
      );
    },
  );
}

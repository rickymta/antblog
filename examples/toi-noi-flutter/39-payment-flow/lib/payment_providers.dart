import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'checkout_launcher.dart';
import 'idempotency_key.dart';
import 'payment_flow.dart';
import 'payment_gateway.dart';
import 'payment_store.dart';

/// Các cổng ra ngoài. App thật ghi đè bằng `HttpPaymentGateway`, kho bền và
/// `UrlLauncherCheckout`; test ghi đè bằng đối tượng giả.
final paymentGatewayProvider = Provider<PaymentGateway>(
  (ref) => throw UnimplementedError('Ghi đè paymentGatewayProvider'),
);
final paymentStoreProvider = Provider<PaymentStore>(
  (ref) => throw UnimplementedError('Ghi đè paymentStoreProvider'),
);
final checkoutLauncherProvider = Provider<CheckoutLauncher>(
  (ref) => UrlLauncherCheckout(),
);
final paymentNowProvider = Provider<DateTime Function()>(
  (ref) =>
      () => DateTime.now().toUtc(),
);
final paymentPollProvider = Provider<PollPolicy>((ref) => PollPolicy.standard);
final paymentDelayProvider = Provider<Future<void> Function(Duration)>(
  (ref) =>
      (d) => Future<void>.delayed(d),
);

final paymentFlowProvider = Provider<PaymentFlow>((ref) {
  final flow = PaymentFlow(
    gateway: ref.watch(paymentGatewayProvider),
    launcher: ref.watch(checkoutLauncherProvider),
    store: ref.watch(paymentStoreProvider),
    newKey: idempotencyKeyGenerator(),
    now: ref.watch(paymentNowProvider),
    poll: ref.watch(paymentPollProvider),
    delay: ref.watch(paymentDelayProvider),
  );
  ref.onDispose(flow.dispose);
  return flow;
});

/// Cầu nối giữa [PaymentFlow] và widget: giữ state để `ref.watch` được.
final paymentControllerProvider =
    NotifierProvider<PaymentController, PaymentState>(PaymentController.new);

class PaymentController extends Notifier<PaymentState> {
  // Không phải `late final`: `build()` chạy lại khi provider bị invalidate
  // hoặc khi một dependency được watch đổi.
  late PaymentFlow _flow;

  @override
  PaymentState build() {
    _flow = ref.watch(paymentFlowProvider);
    final sub = _flow.changes.listen((next) => state = next);
    ref.onDispose(sub.cancel);
    // Có lần thanh toán dang dở từ lần mở app trước thì nạp và hỏi server.
    unawaited(_flow.restore());
    return _flow.state;
  }

  Future<void> pay({required String tripId, required Money amount}) =>
      _flow.start(tripId: tripId, amount: amount);

  Future<ReturnHandling> onDeepLink(Uri link) => _flow.handleReturn(link);

  Future<void> reopen() => _flow.reopenCheckout();

  Future<void> refresh() => _flow.refresh();

  /// Gọi khi app trở lại foreground: khách có thể đóng tab bằng nút Done hoặc
  /// Back, khi đó không có deep link nào được gửi.
  Future<void> onAppResumed() => _flow.refresh();
}

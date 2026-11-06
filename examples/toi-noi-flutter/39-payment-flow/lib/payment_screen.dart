import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'payment_flow.dart';
import 'payment_providers.dart';

/// Màn hình thanh toán một chuyến qua VíXanh.
class PaymentScreen extends ConsumerStatefulWidget {
  const PaymentScreen({super.key, required this.tripId, required this.amount});

  final String tripId;
  final Money amount;

  @override
  ConsumerState<PaymentScreen> createState() => _PaymentScreenState();
}

class _PaymentScreenState extends ConsumerState<PaymentScreen> {
  late final AppLifecycleListener _lifecycle;

  @override
  void initState() {
    super.initState();
    // Khách có thể đóng tab bằng Done hoặc Back: không có deep link nào được
    // gửi, nên khi app trở lại foreground ta hỏi lại server.
    _lifecycle = AppLifecycleListener(
      onStateChange: (state) {
        if (state == AppLifecycleState.resumed) {
          unawaited(
            ref.read(paymentControllerProvider.notifier).onAppResumed(),
          );
        }
      },
    );
  }

  @override
  void dispose() {
    _lifecycle.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final tripId = widget.tripId;
    final amount = widget.amount;
    final state = ref.watch(paymentControllerProvider);
    final controller = ref.read(paymentControllerProvider.notifier);

    final (
      String message,
      List<(String, VoidCallback)> actions,
    ) = switch (state) {
      PaymentIdle() => (
        'Thanh toán $amount bằng ví VíXanh.',
        [('Thanh toán', () => controller.pay(tripId: tripId, amount: amount))],
      ),
      PaymentCreating() => ('Đang tạo phiên thanh toán...', const []),
      PaymentCreateFailed(:final retryable) => (
        retryable
            ? 'Chưa tạo được phiên thanh toán. Bạn chưa bị trừ tiền.'
            : 'Server từ chối yêu cầu thanh toán này.',
        [
          if (retryable)
            ('Thử lại', () => controller.pay(tripId: tripId, amount: amount)),
        ],
      ),
      PaymentAwaitingCheckout(:final launchFailed) => (
        launchFailed
            ? 'Không mở được trang thanh toán.'
            : 'Hoàn tất thanh toán ở trang VíXanh rồi quay lại.',
        [
          ('Mở lại trang thanh toán', controller.reopen),
          ('Kiểm tra lại', controller.refresh),
        ],
      ),
      PaymentConfirming() => ('Đang xác nhận với hệ thống...', const []),
      PaymentPendingConfirmation(:final reason) => (
        switch (reason) {
          PendingReason.unreachable => 'Đang chờ xác nhận. Chưa kết nối được tới hệ thống; bấm Kiểm tra lại khi có mạng.',
          PendingReason.stillPending =>
            'Đang chờ xác nhận. Hệ thống chưa nhận được kết quả từ ví.',
          PendingReason.rejected =>
            'Chưa xác nhận được giao dịch. Vui lòng liên hệ hỗ trợ.',
        },
        [('Kiểm tra lại', controller.refresh)],
      ),
      PaymentSucceeded() => ('Thanh toán thành công.', const []),
      PaymentFailed() => (
        'Thanh toán thất bại.',
        [('Thử lại', () => controller.pay(tripId: tripId, amount: amount))],
      ),
      PaymentCancelled() => (
        'Bạn đã huỷ thanh toán.',
        [
          (
            'Thanh toán lại',
            () => controller.pay(tripId: tripId, amount: amount),
          ),
        ],
      ),
      PaymentExpired() => (
        'Phiên thanh toán đã hết hạn.',
        [
          (
            'Thanh toán lại',
            () => controller.pay(tripId: tripId, amount: amount),
          ),
        ],
      ),
    };

    return Scaffold(
      appBar: AppBar(title: const Text('Thanh toán')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(message, key: const Key('payment-message')),
            const SizedBox(height: 16),
            for (final (label, onPressed) in actions)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: FilledButton(onPressed: onPressed, child: Text(label)),
              ),
          ],
        ),
      ),
    );
  }
}

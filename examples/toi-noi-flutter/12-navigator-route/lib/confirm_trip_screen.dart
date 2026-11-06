import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'clock.dart';
import 'payment_method.dart';
import 'payment_method_sheet.dart';
import 'quote.dart';

/// Màn hình xác nhận chuyến: nhận [Quote] qua constructor và khi đóng lại
/// trả về `Trip?` cho route đã mở nó.
///
/// `Trip` nghĩa là khách đã xác nhận; `null` nghĩa là khách rời đi.
class ConfirmTripScreen extends StatefulWidget {
  const ConfirmTripScreen({
    super.key,
    required this.quote,
    this.clock = systemClock,
  });

  final Quote quote;
  final Clock clock;

  @override
  State<ConfirmTripScreen> createState() => _ConfirmTripScreenState();
}

class _ConfirmTripScreenState extends State<ConfirmTripScreen> {
  static const _defaultMethod = PaymentMethod.cash;

  PaymentMethod _method = _defaultMethod;

  /// Có thay đổi chưa lưu khi khách đã đổi khỏi lựa chọn mặc định.
  bool get _hasChanges => _method != _defaultMethod;

  Future<void> _pickMethod() async {
    final picked = await showPaymentMethodSheet(context, selected: _method);
    // Trong lúc sheet mở, màn hình này có thể đã bị gỡ khỏi cây.
    if (!mounted || picked == null) return;
    setState(() => _method = picked);
  }

  void _confirm() {
    final quote = widget.quote;
    final trip = Trip.request(
      id: 'T-${quote.id}',
      serviceKind: quote.serviceKind,
      distanceKm: quote.distanceKm,
      requestedAt: widget.clock(),
    );
    // `Navigator.pop` không hỏi PopScope: đây là lối ra có chủ đích,
    // nên không bị chặn dù `_hasChanges` đang đúng.
    Navigator.of(context).pop(trip);
  }

  @override
  Widget build(BuildContext context) {
    final quote = widget.quote;
    return PopScope<Trip>(
      canPop: !_hasChanges,
      onPopInvokedWithResult: (didPop, result) {
        if (didPop) return;
        askToLeave(context);
      },
      child: Scaffold(
        appBar: AppBar(title: const Text('Xác nhận chuyến')),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              quote.serviceKind.label,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text('Đón: ${quote.pickup}'),
            Text('Đến: ${quote.dropoff}'),
            Text('Quãng đường: ${quote.distanceKm} km'),
            const SizedBox(height: 16),
            Text(
              '${quote.fare}',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const Divider(height: 32),
            ListTile(
              contentPadding: EdgeInsets.zero,
              leading: Icon(_method.icon),
              title: const Text('Thanh toán'),
              subtitle: Text(_method.label),
              trailing: const Icon(Icons.chevron_right),
              onTap: _pickMethod,
            ),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: _confirm,
              child: const Text('Xác nhận đặt chuyến'),
            ),
          ],
        ),
      ),
    );
  }
}

/// Hỏi khách có chắc muốn rời màn hình khi còn thay đổi chưa lưu.
///
/// Gọi từ `onPopInvokedWithResult` khi lần pop đã bị chặn (`didPop == false`).
/// Chỉ khi khách chọn "Rời đi" mới pop thật, và `Navigator.pop` bỏ qua
/// `PopScope` nên không bị chặn lần hai.
Future<void> askToLeave(BuildContext context) async {
  final leave = await showDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      title: const Text('Rời màn hình?'),
      content: const Text('Lựa chọn thanh toán chưa được lưu.'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(dialogContext).pop(false),
          child: const Text('Ở lại'),
        ),
        TextButton(
          onPressed: () => Navigator.of(dialogContext).pop(true),
          child: const Text('Rời đi'),
        ),
      ],
    ),
  );
  // Sau await, context có thể thuộc về màn hình đã bị gỡ.
  if (leave != true || !context.mounted) return;
  Navigator.of(context).pop();
}

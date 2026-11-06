import 'package:flutter/material.dart';

import 'payment_method.dart';

/// Mở bottom sheet chọn phương thức trả tiền.
///
/// Trả `null` khi khách đóng sheet mà không chọn (chạm ra ngoài, vuốt xuống,
/// nút back), nên nơi gọi phải coi `null` là "giữ nguyên lựa chọn cũ".
Future<PaymentMethod?> showPaymentMethodSheet(
  BuildContext context, {
  required PaymentMethod selected,
}) {
  return showModalBottomSheet<PaymentMethod>(
    context: context,
    showDragHandle: true,
    builder: (sheetContext) => SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (final method in PaymentMethod.values)
            ListTile(
              leading: Icon(method.icon),
              title: Text(method.label),
              trailing: method == selected ? const Icon(Icons.check) : null,
              // Đóng đúng route của sheet và mang lựa chọn về cho nơi gọi.
              onTap: () => Navigator.of(sheetContext).pop(method),
            ),
        ],
      ),
    ),
  );
}

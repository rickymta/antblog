import 'package:flutter/material.dart';

/// Phương thức trả tiền khách chọn ở màn hình xác nhận.
enum PaymentMethod {
  cash('Tiền mặt', Icons.payments_outlined),
  wallet('Ví VíXanh', Icons.account_balance_wallet_outlined);

  const PaymentMethod(this.label, this.icon);

  final String label;
  final IconData icon;
}

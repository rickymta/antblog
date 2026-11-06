import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Hiển thị một [Money] bằng chữ số có độ rộng đều (tabular figures) để các
/// số tiền xếp cột thẳng hàng.
///
/// Mặc định chuỗi hiển thị lấy từ `Money.toString()` (ví dụ `25.500 VND`);
/// truyền [formatter] để định dạng theo locale.
class MoneyText extends StatelessWidget {
  /// Tạo widget hiển thị [money]. Màu chữ kế thừa từ `DefaultTextStyle`;
  /// [style] được gộp lên `titleMedium` của theme.
  const MoneyText(
    this.money, {
    super.key,
    this.style,
    this.textAlign,
    this.formatter,
  });

  /// Số tiền cần hiển thị.
  final Money money;

  /// Kiểu chữ ghi đè lên `titleMedium`. Nếu có `color` (như các kiểu lấy từ
  /// `textTheme`, vốn ghim `onSurface`) thì màu đó thắng màu kế thừa.
  final TextStyle? style;

  /// Căn lề chữ, mặc định theo `Text`.
  final TextAlign? textAlign;

  /// Hàm tạo chuỗi hiển thị (ví dụ định dạng theo locale); `null` thì dùng
  /// `Money.toString()`.
  final String Function(Money money)? formatter;

  @override
  Widget build(BuildContext context) {
    // `titleMedium` ghim màu `onSurface`; lấy màu của `DefaultTextStyle` để
    // chữ đổi màu theo nơi đặt (ví dụ onPrimary trong FilledButton).
    final inherited = DefaultTextStyle.of(context).style.color;
    final base = Theme.of(context).textTheme.titleMedium
        ?.copyWith(color: inherited);
    return Text(
      formatter?.call(money) ?? money.toString(),
      textAlign: textAlign,
      style: base
          ?.merge(style)
          .copyWith(fontFeatures: const [FontFeature.tabularFigures()]),
    );
  }
}

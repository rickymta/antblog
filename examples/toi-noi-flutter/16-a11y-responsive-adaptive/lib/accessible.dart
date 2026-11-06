import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Câu để trình đọc màn hình đọc cho một số tiền: `25.500 đồng`.
///
/// Chữ hiển thị là `25.500 VND`; ta thay viết tắt bằng từ đầy đủ để không
/// phụ thuộc cách từng trình đọc xử lý chữ hoa liền nhau.
String spokenMoney(Money money) =>
    '${money.toString().replaceAll(' VND', '')} đồng';

/// Quãng đường dạng chữ, số lẻ dùng dấu phẩy: `5 km`, `2,5 km`.
String formatKm(double km) {
  final text = km == km.roundToDouble()
      ? km.round().toString()
      : km.toStringAsFixed(1).replaceAll('.', ',');
  return '$text km';
}

/// Một câu đầy đủ mô tả chuyến cho trình đọc màn hình.
String spokenTrip(Trip trip) =>
    '${trip.serviceKind.label}, ${formatKm(trip.distanceKm)}, '
    'trạng thái ${trip.status.label}, ${spokenMoney(trip.fare)}';

/// [MoneyText] kèm nhãn đọc được. Phần chữ trực quan bị loại khỏi cây
/// semantics để chỉ còn đúng một nút đọc `25.500 đồng`.
class AccessibleMoney extends StatelessWidget {
  const AccessibleMoney(this.money, {super.key, this.prefix, this.style});

  final Money money;

  /// Tiền tố đọc trước số tiền, ví dụ `Cước`.
  final String? prefix;

  final TextStyle? style;

  @override
  Widget build(BuildContext context) {
    final spoken = spokenMoney(money);
    return Semantics(
      label: prefix == null ? spoken : '$prefix: $spoken',
      excludeSemantics: true,
      child: MoneyText(money, style: style),
    );
  }
}

/// [TripStatusChip] kèm nhãn `Trạng thái: ...`. Icon của chip chỉ để trang
/// trí, nên bị loại khỏi cây semantics cùng chữ.
///
/// [liveRegion] bật khi trạng thái đổi trong lúc người dùng đang xem, để
/// trình đọc màn hình tự đọc giá trị mới.
class AccessibleStatus extends StatelessWidget {
  const AccessibleStatus(this.status, {super.key, this.liveRegion = false});

  final TripStatus status;
  final bool liveRegion;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: 'Trạng thái: ${status.label}',
      liveRegion: liveRegion,
      excludeSemantics: true,
      child: TripStatusChip(status: status),
    );
  }
}

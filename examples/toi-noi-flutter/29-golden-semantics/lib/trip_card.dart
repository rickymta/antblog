import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Quãng đường dạng chữ, số lẻ dùng dấu phẩy: `5 km`, `2,5 km`.
String formatKm(double km) {
  final text = km == km.roundToDouble()
      ? km.round().toString()
      : km.toStringAsFixed(1).replaceAll('.', ',');
  return '$text km';
}

/// Số tiền cho trình đọc màn hình: `25.500 đồng` thay cho `25.500 VND`.
String spokenMoney(Money money) =>
    '${money.toString().replaceAll(' VND', '')} đồng';

/// Một câu đủ ý mô tả chuyến cho trình đọc màn hình.
String spokenTrip(
  Trip trip, {
  required String pickup,
  required String dropoff,
}) =>
    '${trip.serviceKind.label}, mã ${trip.id}, từ $pickup đến $dropoff, '
    '${formatKm(trip.distanceKm)}, trạng thái ${trip.status.label}, '
    '${spokenMoney(trip.fare)}';

/// Thẻ chuyến trong danh sách lịch sử: dịch vụ, hai địa chỉ, chip trạng thái
/// và cước. Đây là widget mà golden test và semantics test của bài 29 khoá.
///
/// Phần thân là một nút duy nhất với nhãn đọc trọn câu; nút ẩn chuyến là một
/// nút riêng có `tooltip`, nằm ngoài vùng bị loại khỏi cây semantics.
class TripCard extends StatelessWidget {
  const TripCard({
    super.key,
    required this.trip,
    required this.pickup,
    required this.dropoff,
    this.onTap,
    this.onDismiss,
  });

  /// Nhãn của nút ẩn chuyến, cũng là `tooltip` để trình đọc đọc được.
  static const dismissLabel = 'Ẩn chuyến';

  final Trip trip;
  final String pickup;
  final String dropoff;
  final VoidCallback? onTap;
  final VoidCallback? onDismiss;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    final scheme = Theme.of(context).colorScheme;

    return Card(
      clipBehavior: Clip.antiAlias,
      child: Stack(
        children: [
          Semantics(
            button: onTap != null,
            label: spokenTrip(trip, pickup: pickup, dropoff: dropoff),
            onTap: onTap,
            excludeSemantics: true,
            child: InkWell(
              onTap: onTap,
              child: Padding(
                padding: const EdgeInsets.all(ToiNoiSpacing.md),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Chừa chỗ bên phải cho nút ẩn chuyến (48 + lề).
                    Padding(
                      padding: EdgeInsets.only(
                        right: onDismiss == null ? 0 : 40,
                      ),
                      child: Row(
                        children: [
                          CircleAvatar(
                            backgroundColor: scheme.primaryContainer,
                            child: Icon(
                              _iconOf(trip.serviceKind),
                              color: scheme.onPrimaryContainer,
                            ),
                          ),
                          const SizedBox(width: ToiNoiSpacing.sm),
                          Expanded(
                            child: Text(
                              '${trip.serviceKind.label} · ${trip.id}',
                              style: textTheme.titleMedium,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: ToiNoiSpacing.sm),
                    _AddressLine(icon: Icons.trip_origin, text: pickup),
                    const SizedBox(height: ToiNoiSpacing.xs),
                    _AddressLine(icon: Icons.place, text: dropoff),
                    const SizedBox(height: ToiNoiSpacing.sm),
                    Wrap(
                      alignment: WrapAlignment.spaceBetween,
                      crossAxisAlignment: WrapCrossAlignment.center,
                      spacing: ToiNoiSpacing.sm,
                      runSpacing: ToiNoiSpacing.xs,
                      children: [
                        TripStatusChip(status: trip.status),
                        Column(
                          mainAxisSize: MainAxisSize.min,
                          crossAxisAlignment: CrossAxisAlignment.end,
                          children: [
                            MoneyText(trip.fare),
                            Text(
                              formatKm(trip.distanceKm),
                              style: textTheme.bodySmall,
                            ),
                          ],
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),
          if (onDismiss != null)
            Positioned(
              top: ToiNoiSpacing.xs,
              right: ToiNoiSpacing.xs,
              child: IconButton(
                tooltip: dismissLabel,
                onPressed: onDismiss,
                icon: const Icon(Icons.close),
              ),
            ),
        ],
      ),
    );
  }

  static IconData _iconOf(ServiceKind kind) => switch (kind) {
    ServiceKind.ride => Icons.directions_car,
    ServiceKind.food => Icons.restaurant,
    ServiceKind.parcel => Icons.inventory_2,
  };
}

class _AddressLine extends StatelessWidget {
  const _AddressLine({required this.icon, required this.text});

  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: Icon(icon, size: 16),
        ),
        const SizedBox(width: ToiNoiSpacing.sm),
        Expanded(
          child: Text(text, maxLines: 2, overflow: TextOverflow.ellipsis),
        ),
      ],
    );
  }
}

import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'accessible.dart';

/// Thẻ một chuyến trong lịch sử.
///
/// Không có chiều cao cố định và dùng [Wrap] cho hàng đầu: khi chữ phóng to,
/// chip trạng thái xuống dòng thay vì tràn. Cả thẻ là một nút duy nhất với
/// nhãn đọc trọn câu ([spokenTrip]); chữ và chip bên trong bị loại khỏi cây
/// semantics để không bị đọc hai lần.
class TripTile extends StatelessWidget {
  const TripTile({
    super.key,
    required this.trip,
    required this.onTap,
    this.selected = false,
  });

  final Trip trip;
  final VoidCallback onTap;

  /// Đánh dấu thẻ đang mở ở khung chi tiết (bố cục hai cột).
  final bool selected;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Semantics(
      button: true,
      selected: selected,
      label: spokenTrip(trip),
      onTap: onTap,
      excludeSemantics: true,
      child: Card(
        color: selected ? theme.colorScheme.secondaryContainer : null,
        child: InkWell(
          borderRadius: BorderRadius.circular(16),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.all(ToiNoiSpacing.md),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Wrap(
                  spacing: ToiNoiSpacing.sm,
                  runSpacing: ToiNoiSpacing.xs,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Text(
                      trip.serviceKind.label,
                      style: theme.textTheme.titleMedium,
                    ),
                    TripStatusChip(status: trip.status),
                  ],
                ),
                const SizedBox(height: ToiNoiSpacing.xs),
                Text('${trip.id} · ${formatKm(trip.distanceKm)}'),
                const SizedBox(height: ToiNoiSpacing.xs),
                MoneyText(trip.fare),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

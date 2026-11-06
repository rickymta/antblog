import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'toi_noi_colors.dart';
import 'toi_noi_spacing.dart';

/// Chip hiển thị một [TripStatus]: nền, màu chữ, icon và nhãn tiếng Việt.
///
/// Màu lấy từ theme gần nhất (`ColorScheme` và [ToiNoiColors]) nên tự đổi
/// theo sáng/tối. Icon đi kèm để trạng thái không chỉ dựa vào màu.
class TripStatusChip extends StatelessWidget {
  /// Tạo chip cho [status].
  const TripStatusChip({required this.status, this.label, super.key});

  /// Trạng thái cần hiển thị.
  final TripStatus status;

  /// Nhãn thay cho `status.label` (ví dụ bản dịch); `null` thì dùng nhãn
  /// tiếng Việt của [status].
  final String? label;

  @override
  Widget build(BuildContext context) {
    final look = _lookOf(
      status,
      Theme.of(context).colorScheme,
      ToiNoiColors.of(context),
    );

    return DecoratedBox(
      decoration: ShapeDecoration(
        color: look.background,
        shape: const StadiumBorder(),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: ToiNoiSpacing.sm + ToiNoiSpacing.xs,
          vertical: ToiNoiSpacing.xs + 2,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(look.icon, size: 16, color: look.foreground),
            const SizedBox(width: ToiNoiSpacing.xs),
            Flexible(
              child: Text(
                label ?? status.label,
                style: Theme.of(context).textTheme.labelMedium
                    ?.copyWith(color: look.foreground),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

typedef _StatusLook = ({Color background, Color foreground, IconData icon});

/// Chọn giao diện cho từng trạng thái.
///
/// `switch` liệt kê đủ tám giá trị và không có nhánh `_`: thêm một
/// `TripStatus` mới ở `toi_noi_core` thì `flutter analyze` báo lỗi ở đây
/// cho tới khi ta quyết định màu cho nó.
_StatusLook _lookOf(
  TripStatus status,
  ColorScheme scheme,
  ToiNoiColors colors,
) => switch (status) {
  TripStatus.requested => (
    background: scheme.surfaceContainerHighest,
    foreground: scheme.onSurfaceVariant,
    icon: Icons.receipt_long,
  ),
  TripStatus.searching => (
    background: colors.warningContainer,
    foreground: colors.onWarningContainer,
    icon: Icons.search,
  ),
  TripStatus.accepted => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.thumb_up_alt_outlined,
  ),
  TripStatus.arriving => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.directions_car,
  ),
  TripStatus.pickedUp => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.person_pin_circle,
  ),
  TripStatus.inTransit => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.route,
  ),
  TripStatus.completed => (
    background: colors.successContainer,
    foreground: colors.onSuccessContainer,
    icon: Icons.check_circle,
  ),
  TripStatus.cancelled => (
    background: colors.dangerContainer,
    foreground: colors.onDangerContainer,
    icon: Icons.cancel,
  ),
};

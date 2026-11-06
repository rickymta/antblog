import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Các ca sai có chủ đích. Mỗi widget dưới đây vi phạm đúng một guideline để
/// test chứng minh guideline bắt được lỗi, không chỉ "chạy mà không báo gì".

/// `GestureDetector` bọc icon 20: vùng chạm 20 x 20 và không có nhãn.
class TinyDismissIcon extends StatelessWidget {
  const TinyDismissIcon({super.key, required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: const Icon(Icons.close, size: 20),
    );
  }
}

/// `IconButton` đủ lớn nhưng quên `tooltip`: trình đọc chỉ đọc "nút".
class SilentDismissButton extends StatelessWidget {
  const SilentDismissButton({super.key, required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return IconButton(onPressed: onTap, icon: const Icon(Icons.close));
  }
}

/// Nhãn trạng thái kiểu cũ của thẻ chuyến ở bài 08: chữ màu `tertiary` trên
/// nền cùng màu pha 12%. Đẹp nhưng không ai đo tương phản.
///
/// Ở đây ta ép chữ nhạt đi để ca "nhãn nhạt trên nền nhạt" xảy ra rõ ràng:
/// [washedOut] pha thêm 55% trắng vào màu chữ.
class FaintStatusBadge extends StatelessWidget {
  const FaintStatusBadge({
    super.key,
    required this.status,
    this.washedOut = true,
  });

  final TripStatus status;
  final bool washedOut;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final base = status == TripStatus.cancelled
        ? scheme.error
        : scheme.tertiary;
    final text = washedOut ? Color.lerp(base, Colors.white, 0.55)! : base;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: base.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Text(
          status.label,
          style: Theme.of(context).textTheme.labelMedium?.copyWith(color: text),
        ),
      ),
    );
  }
}

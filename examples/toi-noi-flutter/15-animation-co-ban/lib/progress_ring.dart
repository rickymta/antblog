import 'package:flutter/material.dart';

import 'motion.dart';

/// Vòng tiến độ của chuyến. Khi [progress] đổi, vòng chạy từ giá trị đang
/// hiển thị tới giá trị mới bằng `TweenAnimationBuilder`.
class ProgressRing extends StatelessWidget {
  const ProgressRing({
    super.key,
    required this.progress,
    this.size = 120,
    this.onEnd,
  }) : assert(progress >= 0 && progress <= 1, 'progress phải trong 0..1');

  /// Đích của vòng, từ 0 đến 1.
  final double progress;

  /// Đường kính (logical pixel).
  final double size;

  /// Gọi mỗi khi một lượt chạy kết thúc.
  final VoidCallback? onEnd;

  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<double>(
      // Tween mới ở mỗi build, chỉ có `end`: lần dựng đầu không chạy, lần
      // đổi đích sau đó chạy từ giá trị đang hiển thị.
      tween: Tween<double>(end: progress),
      duration: ToiNoiMotion.duration(context, ToiNoiMotion.slow),
      curve: ToiNoiMotion.enterCurve,
      onEnd: onEnd,
      builder: (context, value, _) {
        return SizedBox.square(
          dimension: size,
          child: Stack(
            alignment: Alignment.center,
            children: [
              SizedBox.expand(
                child: CircularProgressIndicator(
                  value: value,
                  strokeWidth: 10,
                  semanticsLabel: 'Tiến độ chuyến',
                ),
              ),
              Text(
                '${(value * 100).round()}%',
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ],
          ),
        );
      },
    );
  }
}

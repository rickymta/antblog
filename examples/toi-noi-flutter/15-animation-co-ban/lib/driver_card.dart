import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'driver.dart';
import 'motion.dart';

/// Thẻ tài xế. Khi chuyến sang `arriving` thẻ được nhấn mạnh: đổi màu nền và
/// viền dày hơn (`AnimatedContainer`) và dòng "tới điểm đón sau
/// ... phút" hiện dần (`AnimatedOpacity`). Thẻ bọc trong `Hero` để bay sang màn
/// chi tiết.
class DriverCard extends StatelessWidget {
  const DriverCard({
    super.key,
    required this.driver,
    required this.status,
    this.onTap,
  });

  final Driver driver;
  final TripStatus status;

  /// Bấm vào thẻ; `null` thì thẻ không bấm được (màn chi tiết).
  final VoidCallback? onTap;

  static const BorderRadius _radius = BorderRadius.all(Radius.circular(16));

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;
    final highlighted = status == TripStatus.arriving;
    final duration = ToiNoiMotion.duration(context, ToiNoiMotion.standard);

    return Hero(
      tag: driverHeroTag(driver),
      child: AnimatedContainer(
        key: const Key('driver-card-box'),
        duration: duration,
        curve: ToiNoiMotion.standardCurve,
        decoration: BoxDecoration(
          color: highlighted
              ? scheme.primaryContainer
              : scheme.surfaceContainerLow,
          borderRadius: _radius,
          border: Border.all(
            color: highlighted ? scheme.primary : scheme.outlineVariant,
            width: highlighted ? 2 : 1,
          ),
        ),
        // Material trong suốt đặt BÊN TRONG hộp đã tô nền: InkWell vẽ hiệu ứng
        // chạm lên Material này nên không bị lớp nền che. Material cũng cấp
        // DefaultTextStyle cho chữ khi thẻ bay trong overlay của Hero.
        child: Material(
          type: MaterialType.transparency,
          child: InkWell(
            onTap: onTap,
            borderRadius: _radius,
            child: Padding(
              padding: const EdgeInsets.all(ToiNoiSpacing.md),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      CircleAvatar(child: Text(driver.initial)),
                      const SizedBox(width: ToiNoiSpacing.md),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              driver.name,
                              style: textTheme.titleMedium,
                              overflow: TextOverflow.ellipsis,
                            ),
                            Text(
                              '${driver.vehicle} · ${driver.plate}',
                              style: textTheme.bodyMedium,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ],
                        ),
                      ),
                      const Icon(Icons.star, size: 16),
                      const SizedBox(width: ToiNoiSpacing.xs),
                      Text(
                        driver.rating.toStringAsFixed(1),
                        style: textTheme.labelLarge,
                      ),
                    ],
                  ),
                  // Luôn chiếm chỗ; chỉ độ mờ đổi nên thẻ không nhảy chiều cao.
                  AnimatedOpacity(
                    duration: ToiNoiMotion.duration(
                      context,
                      ToiNoiMotion.quick,
                    ),
                    curve: ToiNoiMotion.standardCurve,
                    opacity: highlighted ? 1 : 0,
                    child: Padding(
                      padding: const EdgeInsets.only(top: ToiNoiSpacing.sm),
                      child: Text(
                        'Tới điểm đón sau khoảng ${driver.etaMinutes} phút',
                        style: textTheme.bodyMedium,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

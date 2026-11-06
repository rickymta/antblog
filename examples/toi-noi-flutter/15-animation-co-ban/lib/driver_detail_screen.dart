import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'driver.dart';
import 'driver_card.dart';
import 'motion.dart';

/// Màn chi tiết tài xế. Thẻ ở đầu màn hình dùng cùng `DriverCard`, nên cùng
/// `Hero` tag với thẻ ở màn tìm tài xế và thẻ "bay" sang đây.
class DriverDetailScreen extends StatelessWidget {
  const DriverDetailScreen({
    super.key,
    required this.driver,
    required this.status,
  });

  final Driver driver;
  final TripStatus status;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    return Scaffold(
      appBar: AppBar(title: const Text('Tài xế của bạn')),
      body: ListView(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          DriverCard(driver: driver, status: status),
          const SizedBox(height: ToiNoiSpacing.lg),
          Text('Thông tin', style: textTheme.titleMedium),
          const SizedBox(height: ToiNoiSpacing.sm),
          Text('Phương tiện: ${driver.vehicle}'),
          Text('Biển số: ${driver.plate}'),
          Text('Số chuyến đã chạy: ${driver.trips} (số minh hoạ)'),
        ],
      ),
    );
  }
}

/// Route sang màn chi tiết. Thời lượng chuyển trang tính một lần lúc tạo
/// route: bật giảm chuyển động thì `Duration.zero`, trang hiện ngay và `Hero`
/// không bay.
Route<void> driverDetailRoute(
  BuildContext context, {
  required Driver driver,
  required TripStatus status,
}) {
  final duration = ToiNoiMotion.duration(context, ToiNoiMotion.standard);
  return PageRouteBuilder<void>(
    transitionDuration: duration,
    reverseTransitionDuration: duration,
    pageBuilder: (context, animation, secondaryAnimation) =>
        DriverDetailScreen(driver: driver, status: status),
    transitionsBuilder: (context, animation, secondaryAnimation, child) =>
        FadeTransition(opacity: animation, child: child),
  );
}

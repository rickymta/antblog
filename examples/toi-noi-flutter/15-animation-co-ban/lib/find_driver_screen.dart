import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'driver.dart';
import 'driver_card.dart';
import 'driver_detail_screen.dart';
import 'progress_ring.dart';
import 'search_status_panel.dart';

/// Sự kiện kế tiếp trên đường đi bình thường của [trip], hoặc `null` khi
/// chuyến đã kết thúc. Lab dùng nó để "mô phỏng" tài xế thay cho realtime.
TripEvent? nextEventFor(Trip trip) => switch (trip.status) {
  TripStatus.requested => const SearchStarted(),
  TripStatus.searching => DriverAccepted(kDrivers.first.id),
  TripStatus.accepted => const DriverArrived(),
  TripStatus.arriving => const PickedUp(),
  TripStatus.pickedUp => const TripStarted(),
  TripStatus.inTransit => const Completed(),
  TripStatus.completed || TripStatus.cancelled => null,
};

/// Màn hình tìm tài xế: vòng tiến độ, khung trạng thái, thẻ tài xế.
class FindDriverScreen extends StatefulWidget {
  const FindDriverScreen({super.key, required this.initialTrip});

  final Trip initialTrip;

  @override
  State<FindDriverScreen> createState() => _FindDriverScreenState();
}

class _FindDriverScreenState extends State<FindDriverScreen> {
  late Trip _trip = widget.initialTrip;

  void _apply(TripEvent event) {
    setState(() => _trip = _trip.apply(event));
  }

  @override
  Widget build(BuildContext context) {
    final status = _trip.status;
    final driver = findDriver(_trip.driverId);
    final next = nextEventFor(_trip);

    return Scaffold(
      appBar: AppBar(title: const Text('Tìm tài xế')),
      body: ListView(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          Center(child: ProgressRing(progress: status.progress)),
          const SizedBox(height: ToiNoiSpacing.lg),
          Center(child: SearchStatusPanel(status: status)),
          const SizedBox(height: ToiNoiSpacing.lg),
          if (driver != null && status != TripStatus.cancelled)
            DriverCard(
              driver: driver,
              status: status,
              onTap: () => Navigator.of(context).push(
                driverDetailRoute(context, driver: driver, status: status),
              ),
            ),
          const SizedBox(height: ToiNoiSpacing.lg),
          FilledButton(
            key: const Key('next-step'),
            onPressed: next == null ? null : () => _apply(next),
            child: const Text('Bước tiếp theo (mô phỏng)'),
          ),
          const SizedBox(height: ToiNoiSpacing.sm),
          TextButton(
            key: const Key('cancel-trip'),
            onPressed: status.canCancel
                ? () => _apply(const Cancelled(by: CancelledBy.customer))
                : null,
            child: const Text('Huỷ chuyến'),
          ),
        ],
      ),
    );
  }
}

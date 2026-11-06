import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Danh sách gọn các chuyến gần đây: `ListView.separated` đơn giản nhất.
///
/// `itemBuilder` và `separatorBuilder` chỉ được gọi cho các dòng sắp hiện ra,
/// dù [trips] có bao nhiêu phần tử.
class RecentTripsList extends StatelessWidget {
  const RecentTripsList({super.key, required this.trips});

  final List<Trip> trips;

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      itemCount: trips.length,
      itemBuilder: (context, index) {
        final trip = trips[index];
        final completed = trip.status == TripStatus.completed;
        return ListTile(
          title: Text('${trip.serviceKind.label} · ${trip.id}'),
          // Chuyến huỷ không tính cước nên hiện trạng thái thay cho số tiền.
          trailing: Text(completed ? trip.fare.toString() : trip.status.label),
        );
      },
      separatorBuilder: (context, index) => const Divider(height: 1),
    );
  }
}

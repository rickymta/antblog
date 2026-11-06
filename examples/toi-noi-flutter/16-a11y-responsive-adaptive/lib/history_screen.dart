import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'trip_tile.dart';

/// Danh sách lịch sử chuyến. Không biết chuyện chạm thẻ thì mở trang mới hay
/// đổi khung chi tiết: việc đó do [onSelect] của cha quyết định.
class HistoryScreen extends StatelessWidget {
  const HistoryScreen({
    super.key,
    required this.trips,
    required this.onSelect,
    this.selectedId,
  });

  final List<Trip> trips;
  final ValueChanged<Trip> onSelect;
  final String? selectedId;

  @override
  Widget build(BuildContext context) {
    if (trips.isEmpty) {
      return const Center(child: Text('Chưa có chuyến nào'));
    }
    return ListView.separated(
      padding: const EdgeInsets.all(ToiNoiSpacing.md),
      itemCount: trips.length,
      separatorBuilder: (_, _) => const SizedBox(height: ToiNoiSpacing.sm),
      itemBuilder: (context, index) {
        final trip = trips[index];
        return TripTile(
          key: ValueKey(trip.id),
          trip: trip,
          selected: trip.id == selectedId,
          onTap: () => onSelect(trip),
        );
      },
    );
  }
}

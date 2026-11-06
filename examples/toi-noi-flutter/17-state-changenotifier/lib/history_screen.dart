import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'trip_book.dart';
import 'trip_detail_page.dart';

/// Lịch sử chuyến. Đọc [TripBook] từ scope, không nhận `trips` qua constructor
/// và không có callback `onSelect` hay `onChanged` nào chạy ngược lên shell.
class HistoryScreen extends StatelessWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final trips = TripBookScope.of(context).trips;
    if (trips.isEmpty) {
      return const Center(child: Text('Chưa có chuyến nào'));
    }
    return ListView.separated(
      padding: const EdgeInsets.all(ToiNoiSpacing.md),
      itemCount: trips.length,
      separatorBuilder: (_, _) => const SizedBox(height: ToiNoiSpacing.sm),
      itemBuilder: (context, index) {
        final trip = trips[index];
        return Card(
          key: ValueKey(trip.id),
          child: ListTile(
            title: Text('${trip.serviceKind.label} · ${trip.id}'),
            subtitle: Padding(
              padding: const EdgeInsets.only(top: ToiNoiSpacing.xs),
              child: Align(
                alignment: Alignment.centerLeft,
                child: TripStatusChip(status: trip.status),
              ),
            ),
            trailing: MoneyText(trip.fare),
            onTap: () => Navigator.of(context).push(
              MaterialPageRoute<void>(
                builder: (_) => TripDetailPage(tripId: trip.id),
              ),
            ),
          ),
        );
      },
    );
  }
}

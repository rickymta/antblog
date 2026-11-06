import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'paths.dart';
import 'trips.dart';

/// Tab "Lịch sử": mỗi dòng mở `/trip/:id` bằng `push`.
class HistoryScreen extends ConsumerWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final trips = ref.watch(tripListProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Lịch sử')),
      body: trips.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) => Center(child: Text('Không tải được: $error')),
        data: (trips) => ListView(
          children: [
            for (final trip in trips)
              ListTile(
                key: ValueKey(trip.id),
                title: Text('${trip.id} · ${trip.serviceKind.label}'),
                subtitle: Align(
                  alignment: Alignment.centerLeft,
                  child: TripStatusChip(status: trip.status),
                ),
                trailing: MoneyText(trip.fare),
                onTap: () => context.push(AppPaths.trip(trip.id)),
              ),
          ],
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'error_text.dart';
import 'providers.dart';

/// Chuyến vừa đặt, đọc lại qua `tripByIdProvider(tripId)`.
class TripScreen extends ConsumerWidget {
  const TripScreen({super.key, required this.tripId});

  final String tripId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final trip = ref.watch(tripByIdProvider(tripId));
    return Scaffold(
      appBar: AppBar(title: Text('Chuyến $tripId')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: trip.when(
          data: (trip) => Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TripStatusChip(status: trip.status),
              const SizedBox(height: ToiNoiSpacing.md),
              Text('${trip.serviceKind.label}, ${trip.distanceKm} km'),
              MoneyText(trip.fare),
            ],
          ),
          error: (error, _) => Text(describeError(error)),
          loading: () => const Center(child: CircularProgressIndicator()),
        ),
      ),
    );
  }
}

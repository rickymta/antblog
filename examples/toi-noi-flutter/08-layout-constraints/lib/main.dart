import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'sample_trips.dart';
import 'trip_card.dart';

void main() {
  runApp(const ToiNoiApp());
}

class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
        useMaterial3: true,
      ),
      home: const TripCardGallery(),
    );
  }
}

/// Bốn thẻ chuyến mẫu: địa chỉ ngắn, địa chỉ dài, đồ ăn, đã huỷ.
class TripCardGallery extends StatelessWidget {
  const TripCardGallery({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Chuyến của tôi')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TripCard(
              trip: tripAt(TripStatus.inTransit),
              pickup: shortAddress,
              dropoff: 'Số 40 đường Cây Me, Quận 3',
            ),
            const SizedBox(height: 12),
            TripCard(
              trip: tripAt(TripStatus.arriving, id: 'T-1043', distanceKm: 12.4),
              pickup: longAddress,
              dropoff: shortAddress,
            ),
            const SizedBox(height: 12),
            TripCard(
              trip: tripAt(
                TripStatus.searching,
                id: 'T-1044',
                serviceKind: ServiceKind.food,
                distanceKm: 2.001,
              ),
              pickup: 'Quán Bún Chả Cô Lan, ngõ 8',
              dropoff: shortAddress,
            ),
            const SizedBox(height: 12),
            TripCard(
              trip: tripAt(
                TripStatus.cancelled,
                id: 'T-1045',
                serviceKind: ServiceKind.parcel,
                distanceKm: 100,
              ),
              pickup: shortAddress,
              dropoff: longAddress,
            ),
          ],
        ),
      ),
    );
  }
}

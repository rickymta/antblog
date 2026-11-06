import 'package:flutter/material.dart';
import 'package:toi_noi_29/sample_trips.dart';
import 'package:toi_noi_29/trip_card.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Bốn thẻ chuyến mẫu xếp dọc: đang tìm, đang chở, hoàn thành, đã huỷ.
/// Địa chỉ hư cấu; mọi số cước do `calculateFare` tính trong `Trip.request`.
Widget sampleCardColumn({
  bool dismissible = true,
  TripStatus first = TripStatus.searching,
}) {
  final trips = [
    tripAt(first, id: 'T-1045', km: 5),
    tripAt(TripStatus.inTransit, id: 'T-1044', kind: ServiceKind.food, km: 2.5),
    tripAt(
      TripStatus.completed,
      id: 'T-1043',
      kind: ServiceKind.parcel,
      km: 12,
    ),
    tripAt(TripStatus.cancelled, id: 'T-1042', km: 3),
  ];
  return Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      for (final trip in trips) ...[
        TripCard(
          trip: trip,
          pickup: 'Chung cư Hoa Sen, 12 Đường Mẫu',
          dropoff: 'Ga Trung Tâm, cổng B',
          onTap: () {},
          onDismiss: dismissible ? () {} : null,
        ),
        const SizedBox(height: 12),
      ],
    ],
  );
}

/// Một thẻ chuyến với trạng thái và địa chỉ đón tuỳ chọn.
Widget oneCard({
  TripStatus status = TripStatus.searching,
  String pickup = 'Chung cư Hoa Sen, 12 Đường Mẫu',
  bool dismissible = true,
}) => TripCard(
  trip: tripAt(status, id: 'T-1045', km: 5),
  pickup: pickup,
  dropoff: 'Ga Trung Tâm, cổng B',
  onTap: () {},
  onDismiss: dismissible ? () {} : null,
);

/// Chip của cả tám trạng thái, để golden và test contrast cùng dùng.
Widget allStatusChips() => Wrap(
  spacing: 8,
  runSpacing: 8,
  children: [
    for (final status in TripStatus.values) TripStatusChip(status: status),
  ],
);

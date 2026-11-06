import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'trip_book.dart';

/// Chi tiết một chuyến, mở bằng `Navigator.push`.
///
/// Chỉ nhận `tripId`. Chuyến hiện tại lấy từ [TripBook] nên sau khi huỷ, cả
/// trang này lẫn danh sách phía sau cùng thấy trạng thái mới mà không cần
/// `onChanged`. Route được đẩy dưới `MaterialApp`, nên thấy được scope đặt
/// phía trên `MaterialApp`.
class TripDetailPage extends StatelessWidget {
  const TripDetailPage({super.key, required this.tripId});

  final String tripId;

  @override
  Widget build(BuildContext context) {
    final trip = TripBookScope.of(context).byId(tripId);
    if (trip == null) {
      return Scaffold(
        appBar: AppBar(title: Text('Chuyến $tripId')),
        body: const Center(child: Text('Không tìm thấy chuyến')),
      );
    }
    final textTheme = Theme.of(context).textTheme;
    return Scaffold(
      appBar: AppBar(title: Text('Chuyến ${trip.id}')),
      body: ListView(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          Text(trip.serviceKind.label, style: textTheme.headlineSmall),
          const SizedBox(height: ToiNoiSpacing.sm),
          Align(
            alignment: Alignment.centerLeft,
            child: TripStatusChip(status: trip.status),
          ),
          const SizedBox(height: ToiNoiSpacing.md),
          Text('Tài xế: ${trip.driverId ?? 'chưa có'}'),
          const SizedBox(height: ToiNoiSpacing.md),
          MoneyText(trip.fare, style: textTheme.headlineSmall),
          const SizedBox(height: ToiNoiSpacing.lg),
          if (trip.status.canCancel)
            OutlinedButton(
              onPressed: () => TripBookScope.read(context).cancel(trip.id),
              child: const Text('Huỷ chuyến'),
            ),
        ],
      ),
    );
  }
}

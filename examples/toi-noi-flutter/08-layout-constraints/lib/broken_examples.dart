import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// SAI CÓ CHỦ ĐÍCH: thẻ chuyến viết như người mới. `Text` không nằm trong
/// `Expanded` nên `Row` đo nó với bề rộng không giới hạn; địa chỉ dài làm hàng
/// rộng hơn màn hình và Flutter báo `RenderFlex overflowed`.
/// Bản đúng là `TripCard` trong `trip_card.dart`.
class BrokenTripCard extends StatelessWidget {
  const BrokenTripCard({super.key, required this.trip, required this.pickup});

  final Trip trip;
  final String pickup;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const Icon(Icons.directions_car),
            const SizedBox(width: 12),
            Text(pickup),
            const SizedBox(width: 12),
            Text(trip.fare.toString()),
          ],
        ),
      ),
    );
  }
}

/// SAI CÓ CHỦ ĐÍCH: `ListView` trong `Column` không bọc `Expanded`.
/// `Column` cho con chiều cao không giới hạn, `ListView` đòi chiều cao hữu hạn.
class ListInColumnBroken extends StatelessWidget {
  const ListInColumnBroken({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('Lịch sử chuyến'),
        ListView(children: const [Text('T-1042'), Text('T-1041')]),
      ],
    );
  }
}

/// Bản đúng: `Expanded` cho `ListView` đúng phần chiều cao còn lại.
class ListInColumnFixed extends StatelessWidget {
  const ListInColumnFixed({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('Lịch sử chuyến'),
        Expanded(
          child: ListView(children: const [Text('T-1042'), Text('T-1041')]),
        ),
      ],
    );
  }
}

/// SAI CÓ CHỦ ĐÍCH: `Expanded` trong `Row` đặt trong khung cuộn ngang.
/// Khung cuộn ngang cho `Row` bề rộng không giới hạn nên không còn "phần còn
/// lại" để chia.
class ExpandedInHorizontalScrollBroken extends StatelessWidget {
  const ExpandedInHorizontalScrollBroken({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(children: const [Expanded(child: Text('Điểm đón'))]),
    );
  }
}

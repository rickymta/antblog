import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show ScrollCacheExtent;
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'build_log.dart';

/// Chiều cao cố định của một dòng lịch sử, để tính được số dòng nhìn thấy.
const double tripTileHeight = 72;

/// Một dòng lịch sử chuyến. Mỗi lần build ghi nhãn `tile`.
class TripTile extends StatelessWidget {
  /// Tạo dòng cho [trip].
  const TripTile(this.trip, {super.key});

  /// Chuyến cần hiển thị.
  final Trip trip;

  @override
  Widget build(BuildContext context) {
    BuildLog.hit('tile');
    return SizedBox(
      height: tripTileHeight,
      child: ListTile(
        title: Text(trip.id),
        subtitle: TripStatusChip(status: trip.status),
        trailing: MoneyText(trip.fare),
      ),
    );
  }
}

/// Dựng một dòng và ghi nhãn `tile-created`: đếm số widget được tạo ra, khác
/// với số widget được build.
TripTile createTile(Trip trip) {
  BuildLog.hit('tile-created');
  return TripTile(trip);
}

/// Ba cách đưa danh sách dài lên màn hình.
enum ListStrategy {
  /// `SingleChildScrollView` + `Column`: không có cơ chế lười.
  column,

  /// `ListView(children: [...])`: tạo hết widget, nhưng chỉ build dòng gần
  /// khung nhìn.
  children,

  /// `ListView.builder`: chỉ tạo và build dòng gần khung nhìn.
  builder,
}

/// Danh sách lịch sử [trips] theo [strategy].
///
/// [cacheExtent] chỉ có tác dụng với hai cách dùng `ListView`.
class TripHistoryList extends StatelessWidget {
  /// Tạo danh sách.
  const TripHistoryList({
    required this.trips,
    required this.strategy,
    this.cacheExtent,
    super.key,
  });

  /// Dữ liệu.
  final List<Trip> trips;

  /// Cách dựng danh sách.
  final ListStrategy strategy;

  /// Vùng đệm ngoài khung nhìn; `null` thì dùng mặc định của framework.
  final ScrollCacheExtent? cacheExtent;

  @override
  Widget build(BuildContext context) => switch (strategy) {
    ListStrategy.column => SingleChildScrollView(
      child: Column(children: [for (final trip in trips) createTile(trip)]),
    ),
    ListStrategy.children => ListView(
      scrollCacheExtent: cacheExtent,
      children: [for (final trip in trips) createTile(trip)],
    ),
    ListStrategy.builder => ListView.builder(
      scrollCacheExtent: cacheExtent,
      itemCount: trips.length,
      itemExtent: tripTileHeight,
      itemBuilder: (context, index) => createTile(trips[index]),
    ),
  };
}

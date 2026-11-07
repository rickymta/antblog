import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'build_log.dart';

/// Trạng thái theo dõi một chuyến: chuyến và vị trí tài xế.
///
/// Vị trí đổi nhiều lần mỗi giây, còn chuyến và cước hiếm khi đổi. Provider so
/// state bằng `==`, nên class phải bất biến và có `==`.
@immutable
class TripLive {
  /// Tạo trạng thái theo dõi.
  const TripLive({
    required this.trip,
    required this.driverLat,
    required this.driverLng,
  });

  /// Chuyến đang theo dõi.
  final Trip trip;

  /// Vĩ độ tài xế (toạ độ hư cấu).
  final double driverLat;

  /// Kinh độ tài xế (toạ độ hư cấu).
  final double driverLng;

  /// Bản sao với vị trí hoặc chuyến mới.
  TripLive copyWith({Trip? trip, double? driverLat, double? driverLng}) =>
      TripLive(
        trip: trip ?? this.trip,
        driverLat: driverLat ?? this.driverLat,
        driverLng: driverLng ?? this.driverLng,
      );

  @override
  bool operator ==(Object other) =>
      other is TripLive &&
      other.trip == trip &&
      other.driverLat == driverLat &&
      other.driverLng == driverLng;

  @override
  int get hashCode => Object.hash(trip, driverLat, driverLng);
}

/// Notifier giữ [TripLive].
class TripLiveNotifier extends Notifier<TripLive> {
  @override
  TripLive build() => TripLive(
    trip: Trip.request(
      id: 'T-41',
      serviceKind: ServiceKind.ride,
      distanceKm: 5,
      requestedAt: DateTime.utc(2026, 11, 6, 8),
    ),
    driverLat: 10.0,
    driverLng: 106.0,
  );

  /// Tài xế di chuyển: chỉ vị trí đổi.
  void moveDriver(double lat, double lng) =>
      state = state.copyWith(driverLat: lat, driverLng: lng);

  /// Áp một sự kiện lên chuyến (đổi trạng thái).
  void apply(TripEvent event) =>
      state = state.copyWith(trip: state.trip.apply(event));

  /// Đổi quãng đường nên đổi cước; chuyến mới vẫn ở `requested`.
  void reroute(double distanceKm) => state = state.copyWith(
    trip: Trip.request(
      id: state.trip.id,
      serviceKind: state.trip.serviceKind,
      distanceKm: distanceKm,
      requestedAt: state.trip.requestedAt,
    ),
  );
}

/// Provider của [TripLive].
final tripLiveProvider = NotifierProvider<TripLiveNotifier, TripLive>(
  TripLiveNotifier.new,
);

/// Xem cả state: rebuild mỗi khi tài xế nhúc nhích dù chỉ hiện cước.
class FareLabelWatchAll extends ConsumerWidget {
  /// Tạo nhãn cước xem cả state.
  const FareLabelWatchAll({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    BuildLog.hit('fare-all');
    final live = ref.watch(tripLiveProvider);
    return Text(live.trip.fare.toString());
  }
}

/// Chỉ xem cước bằng `select`: rebuild khi và chỉ khi `Money` đổi.
class FareLabelSelect extends ConsumerWidget {
  /// Tạo nhãn cước dùng `select`.
  const FareLabelSelect({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    BuildLog.hit('fare-select');
    final fare = ref.watch(tripLiveProvider.select((s) => s.trip.fare));
    return Text(fare.toString());
  }
}

/// Lỗi hay gặp: `select` trả về `List` mới mỗi lần, mà `List` không có `==`
/// theo giá trị nên lần nào cũng bị coi là đã đổi.
class FareLabelSelectList extends ConsumerWidget {
  /// Tạo nhãn cước với `select` trả danh sách.
  const FareLabelSelectList({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    BuildLog.hit('fare-select-list');
    final fares = ref.watch(tripLiveProvider.select((s) => [s.trip.fare]));
    return Text(fares.single.toString());
  }
}

/// Chọn nhiều giá trị bằng một record: record so sánh theo giá trị nên chỉ
/// build lại khi cước hoặc trạng thái chuyến đổi, không phải khi tài xế di
/// chuyển.
class FareStatusLabel extends ConsumerWidget {
  /// Tạo nhãn cước và trạng thái.
  const FareStatusLabel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    BuildLog.hit('fare-status');
    final (fare, status) = ref.watch(
      tripLiveProvider.select((s) => (s.trip.fare, s.trip.status)),
    );
    return Text('$fare · ${status.name}');
  }
}

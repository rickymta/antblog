import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'auth.dart';

/// Không có chuyến với mã đã cho. Thử lại cũng không giúp được gì.
final class TripNotFound implements Exception {
  const TripNotFound(this.id);

  final String id;

  @override
  String toString() => 'TripNotFound: $id';
}

abstract interface class TripRepository {
  Future<List<Trip>> list();

  /// Ném [TripNotFound] nếu không có.
  Future<Trip> byId(String id);

  /// Tạo chuyến `requested` và trả về nó.
  Future<Trip> place(double distanceKm);
}

/// Dữ liệu mẫu: cước luôn do `calculateFare` tính, không gõ tay.
List<Trip> sampleTrips() {
  final at = DateTime.utc(2026, 11, 6, 8);
  Trip request(String id, ServiceKind kind, double km) =>
      Trip.request(id: id, serviceKind: kind, distanceKm: km, requestedAt: at);
  Trip accepted(Trip t) =>
      t.apply(const SearchStarted()).apply(const DriverAccepted('D-07'));

  final done = accepted(request('T-1001', ServiceKind.ride, 5))
      .apply(const DriverArrived())
      .apply(const PickedUp())
      .apply(const TripStarted())
      .apply(const Completed());
  final moving = accepted(request('T-1002', ServiceKind.food, 2.001))
      .apply(const DriverArrived())
      .apply(const PickedUp())
      .apply(const TripStarted());
  final cancelled = request('T-1003', ServiceKind.parcel, 100)
      .apply(const SearchStarted())
      .apply(const Cancelled(by: CancelledBy.customer));
  final arriving = accepted(request('T-1004', ServiceKind.ride, 12))
      .apply(const DriverArrived());
  return [arriving, cancelled, moving, done];
}

/// Bản trong bộ nhớ, mã chuyến mới tăng dần từ `T-1005`.
class FakeTripRepository implements TripRepository {
  FakeTripRepository({List<Trip>? trips})
    : _trips = [...(trips ?? sampleTrips())];

  final List<Trip> _trips;
  int _next = 1005;

  @override
  Future<List<Trip>> list() async => List.unmodifiable(_trips);

  @override
  Future<Trip> byId(String id) async {
    for (final trip in _trips) {
      if (trip.id == id) return trip;
    }
    throw TripNotFound(id);
  }

  @override
  Future<Trip> place(double distanceKm) async {
    final trip = Trip.request(
      id: 'T-${_next++}',
      serviceKind: ServiceKind.ride,
      distanceKm: distanceKm,
      requestedAt: DateTime.now().toUtc(),
    );
    _trips.insert(0, trip);
    return trip;
  }
}

final tripRepositoryProvider = Provider<TripRepository>(
  (ref) => FakeTripRepository(),
);

/// Lịch sử chuyến của người đang đăng nhập; chưa đăng nhập thì rỗng.
/// Đọc `authProvider` bằng `watch` để đăng xuất xong danh sách bị bỏ.
final tripListProvider = FutureProvider.autoDispose<List<Trip>>((ref) async {
  final session = ref.watch(authProvider).value;
  if (session == null) return const [];
  return ref.watch(tripRepositoryProvider).list();
});

/// Một chuyến theo mã. `TripNotFound` không đáng thử lại nên tắt retry.
final tripByIdProvider = FutureProvider.autoDispose.family<Trip, String>(
  (ref, id) => ref.watch(tripRepositoryProvider).byId(id),
  retry: (_, _) => null,
);

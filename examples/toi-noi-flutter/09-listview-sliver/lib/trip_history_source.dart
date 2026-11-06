import 'package:toi_noi_core/toi_noi_core.dart';

/// Một trang lịch sử chuyến: [trips] của trang này và [hasMore] cho biết còn
/// trang sau hay không.
final class TripPage {
  const TripPage({required this.trips, required this.hasMore});

  final List<Trip> trips;
  final bool hasMore;
}

/// Nguồn dữ liệu lịch sử chuyến, phân trang theo vị trí.
///
/// Màn hình chỉ biết giao diện này. Ở bài 19 ta thay bằng bản gọi HTTP; ở bài
/// này chỉ có bản giả bên dưới, không có mạng.
abstract interface class TripHistorySource {
  /// Lấy tối đa [limit] chuyến, bắt đầu từ vị trí [offset] (0 là chuyến mới
  /// nhất). Ném [ArgumentError] khi [offset] âm hoặc [limit] không dương.
  Future<TripPage> fetchPage({required int offset, required int limit});
}

/// Bản giả của [TripHistorySource]: dữ liệu sinh sẵn, trả sau [latency].
///
/// Mọi chuyến đều đã kết thúc (`completed` hoặc `cancelled`), cước tính bằng
/// `calculateFare` của `toi_noi_core`. Tên tài xế, mã chuyến là hư cấu.
class FakeTripHistorySource implements TripHistorySource {
  FakeTripHistorySource({
    int total = 47,
    this.latency = const Duration(milliseconds: 400),
    DateTime? newestAt,
  }) : assert(total >= 0),
       assert(newestAt == null || newestAt.isUtc),
       _trips = [
         for (var i = 0; i < total; i++)
           _buildTrip(
             index: i,
             total: total,
             newestAt: newestAt ?? _defaultNewestAt,
           ),
       ];

  static final DateTime _defaultNewestAt = DateTime.utc(2026, 11, 5, 10);

  /// Độ trễ giả lập của mạng.
  final Duration latency;

  final List<Trip> _trips;

  /// Số lần [fetchPage] đã được gọi, để test và demo đếm.
  int fetchCount = 0;

  /// Tổng số chuyến hiện có.
  int get total => _trips.length;

  /// Thêm một chuyến mới nhất vào đầu danh sách, mô phỏng chuyến vừa kết thúc
  /// ở máy khác trong lúc người dùng đang xem.
  void prepend(Trip trip) => _trips.insert(0, trip);

  @override
  Future<TripPage> fetchPage({required int offset, required int limit}) async {
    if (offset < 0) {
      throw ArgumentError.value(offset, 'offset', 'không được âm');
    }
    if (limit <= 0) {
      throw ArgumentError.value(limit, 'limit', 'phải dương');
    }
    fetchCount++;
    await Future<void>.delayed(latency);
    final end = (offset + limit).clamp(0, _trips.length);
    final start = offset.clamp(0, end);
    return TripPage(
      trips: List.unmodifiable(_trips.sublist(start, end)),
      hasMore: end < _trips.length,
    );
  }
}

/// Quãng đường (km) quay vòng cho dữ liệu giả. Có 2,001 km và 100 km để thấy
/// cả hai đầu của bảng giá.
const _distancesKm = <double>[5, 2.001, 8.4, 12, 3.2, 100, 0.8, 6.5, 15.3, 2];

/// Chuyến thứ [index] tính từ mới nhất. Cứ 5 chuyến có 1 chuyến bị khách huỷ.
Trip _buildTrip({
  required int index,
  required int total,
  required DateTime newestAt,
}) {
  final number = total - index;
  final trip = Trip.request(
    id: 'T-${number.toString().padLeft(4, '0')}',
    serviceKind: ServiceKind.values[index % ServiceKind.values.length],
    distanceKm: _distancesKm[index % _distancesKm.length],
    requestedAt: newestAt.subtract(Duration(hours: 7 * index)),
  ).apply(const SearchStarted());

  if (index % 5 == 4) {
    return trip.apply(const Cancelled(by: CancelledBy.customer));
  }
  return trip
      .apply(DriverAccepted('D-${(number % 9 + 1).toString().padLeft(3, '0')}'))
      .apply(const DriverArrived())
      .apply(const PickedUp())
      .apply(const TripStarted())
      .apply(const Completed());
}

import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote.dart';

/// Lỗi tạm thời của dịch vụ báo giá (mạng chập chờn, server bận). Thử lại
/// có thể thành công.
final class QuoteUnavailable implements Exception {
  const QuoteUnavailable([this.message = 'dịch vụ báo giá đang bận']);

  final String message;

  @override
  String toString() => 'QuoteUnavailable: $message';
}

/// Không có chuyến với mã đã cho. Thử lại cũng không giúp được gì.
final class TripNotFound implements Exception {
  const TripNotFound(this.id);

  final String id;

  @override
  String toString() => 'TripNotFound: $id';
}

/// Cổng tới backend của báo giá và chuyến. Bài này chỉ có bản giả trong bộ
/// nhớ; bài 19 thay bằng client HTTP mà màn hình và provider không phải sửa.
abstract interface class QuoteRepository {
  /// Lấy báo giá cho [serviceKind] và [distanceKm].
  ///
  /// Ném [QuoteUnavailable] khi dịch vụ bận.
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  });

  /// Đặt chuyến từ [quote]; trả về chuyến ở trạng thái `requested`.
  Future<Trip> placeTrip(Quote quote);

  /// Đọc một chuyến; ném [TripNotFound] nếu không có.
  Future<Trip> fetchTrip(String id);
}

/// Bản giả trong bộ nhớ: tính cước bằng `calculateFare` của `toi_noi_core`.
///
/// [latency] mô phỏng độ trễ mạng. Nếu đặt [unavailableAboveKm] thì quãng
/// đường dài hơn ngưỡng đó luôn ném [QuoteUnavailable], để thử đường lỗi.
final class FakeQuoteRepository implements QuoteRepository {
  FakeQuoteRepository({
    this.latency = Duration.zero,
    this.unavailableAboveKm,
    DateTime Function()? now,
  }) : _now = now ?? (() => DateTime.now().toUtc());

  final Duration latency;
  final double? unavailableAboveKm;
  final DateTime Function() _now;

  final Map<String, Trip> _trips = {};
  var _nextNumber = 1;

  Future<void> _delay() =>
      latency == Duration.zero ? Future.value() : Future.delayed(latency);

  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) async {
    await _delay();
    final limit = unavailableAboveKm;
    if (limit != null && distanceKm > limit) {
      throw const QuoteUnavailable();
    }
    return Quote(
      serviceKind: serviceKind,
      distanceKm: distanceKm,
      fare: calculateFare(distanceKm),
    );
  }

  @override
  Future<Trip> placeTrip(Quote quote) async {
    await _delay();
    final id = 'T-${(_nextNumber++).toString().padLeft(4, '0')}';
    final trip = Trip.request(
      id: id,
      serviceKind: quote.serviceKind,
      distanceKm: quote.distanceKm,
      requestedAt: _now(),
    );
    _trips[id] = trip;
    return trip;
  }

  @override
  Future<Trip> fetchTrip(String id) async {
    await _delay();
    final trip = _trips[id];
    if (trip == null) throw TripNotFound(id);
    return trip;
  }
}

import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote.dart';

/// Lỗi tạm thời của dịch vụ báo giá. Thử lại có thể thành công.
final class QuoteUnavailable implements Exception {
  const QuoteUnavailable([this.message = 'dịch vụ báo giá đang bận']);

  final String message;

  @override
  String toString() => 'QuoteUnavailable: $message';
}

/// Cổng tới backend của báo giá và đặt chuyến. Widget và provider chỉ biết
/// interface này, nên test thay bằng bản giả mà không đụng mạng.
abstract interface class QuoteRepository {
  /// Lấy báo giá cho [serviceKind] và [distanceKm]; ném [QuoteUnavailable]
  /// khi dịch vụ bận.
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  });

  /// Đặt chuyến từ [quote]; trả về chuyến ở trạng thái `requested`.
  Future<Trip> placeTrip(Quote quote);
}

/// Bản giả trong bộ nhớ để chạy thử: tính cước bằng `calculateFare`.
///
/// [latency] mô phỏng độ trễ mạng.
final class FakeQuoteRepository implements QuoteRepository {
  FakeQuoteRepository({this.latency = Duration.zero});

  final Duration latency;
  var _nextNumber = 1;

  Future<void> _delay() =>
      latency == Duration.zero ? Future.value() : Future.delayed(latency);

  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) async {
    await _delay();
    return Quote(
      serviceKind: serviceKind,
      distanceKm: distanceKm,
      fare: calculateFare(distanceKm),
    );
  }

  @override
  Future<Trip> placeTrip(Quote quote) async {
    await _delay();
    return Trip.request(
      id: 'T-${(_nextNumber++).toString().padLeft(4, '0')}',
      serviceKind: quote.serviceKind,
      distanceKm: quote.distanceKm,
      requestedAt: DateTime.now().toUtc(),
    );
  }
}

import 'package:toi_noi_core/toi_noi_core.dart';

import 'booking_log.dart';
import 'quote_request.dart';
import 'trip_failure.dart';
import 'trip_repository.dart';

/// Use case: khách bấm **Đặt chuyến** (bản rút gọn của lab 21, thêm nhật ký).
///
/// 1. Quãng đường ngoài miền bị từ chối ngay: không gọi repository, không ghi
///    nhật ký, không sinh `intentId`.
/// 2. Một ý định có một `intentId`. Gửi lại sau lỗi dùng lại đúng `intentId`
///    đó nên dịch vụ không tạo chuyến thứ hai. Đổi yêu cầu hoặc đặt thành công
///    thì ý định kết thúc.
/// 3. Nhật ký ghi một dòng cho mỗi lần đã gửi: `booked` khi thành công,
///    `failed` khi gặp [TripFailure]. Lỗi lập trình (không phải [TripFailure])
///    đi thẳng lên, không bị ghi như lỗi nghiệp vụ.
final class RequestTrip {
  RequestTrip(this._repository, this._log, {required this._newIntentId});

  final TripRepository _repository;
  final BookingLog _log;
  final String Function() _newIntentId;

  ({String id, QuoteRequest request})? _pending;

  Future<Trip> call(QuoteRequest request) async {
    request.requireWithinRange();
    final pending = _pending;
    final intent = pending != null && pending.request == request
        ? pending
        : (id: _newIntentId(), request: request);
    _pending = intent;
    try {
      final trip = await _repository.requestTrip(request, intentId: intent.id);
      if (identical(_pending, intent)) {
        _pending = null;
      }
      _log.booked(trip);
      return trip;
    } on TripFailure catch (failure) {
      _log.failed(request, failure);
      rethrow;
    }
  }
}

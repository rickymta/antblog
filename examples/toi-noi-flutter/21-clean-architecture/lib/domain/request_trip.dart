import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote_request.dart';
import 'trip_repository.dart';

/// Use case: khách bấm **Đặt chuyến**.
///
/// Lý do chính để nó tồn tại là một quy tắc không thuộc về giao diện lẫn tầng
/// mạng, và cần giữ trạng thái giữa các lần gọi:
///
/// 1. Một ý định đặt chuyến chỉ có một `intentId`. Gửi lại sau lỗi (kể cả khi
///    không biết lần đầu đã tới server hay chưa) dùng lại đúng `intentId` đó,
///    nên dịch vụ nhận ra và không tạo chuyến thứ hai. Đổi yêu cầu hoặc đặt
///    thành công thì ý định kết thúc, lần sau là ý định mới.
///
/// Quãng đường ngoài miền cũng bị từ chối ngay, không tốn một request, nhưng
/// quy tắc đó nằm ở `QuoteRequest.requireWithinRange` vì báo giá dùng chung.
///
/// Giới hạn đã biết: ý định chỉ sống trong bộ nhớ và chỉ nhớ một yêu cầu.
/// Mất response rồi đổi yêu cầu (hoặc tắt app) thì khoá cũ bị quên và lần đặt
/// sau có thể tạo chuyến thứ hai. Lưu khoá bền: bài 46.
final class RequestTrip {
  RequestTrip(this._repository, {required this._newIntentId});

  final TripRepository _repository;
  final String Function() _newIntentId;

  ({String id, QuoteRequest request})? _pending;

  Future<Trip> call(QuoteRequest request) async {
    request.requireWithinRange();
    final pending = _pending;
    final intent = pending != null && pending.request == request
        ? pending
        : (id: _newIntentId(), request: request);
    _pending = intent;
    final trip = await _repository.requestTrip(request, intentId: intent.id);
    if (identical(_pending, intent)) {
      _pending = null;
    }
    return trip;
  }
}

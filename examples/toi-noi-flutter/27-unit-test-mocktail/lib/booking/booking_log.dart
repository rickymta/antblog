import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote_request.dart';
import 'trip_failure.dart';

/// Nhật ký nghiệp vụ của luồng đặt chuyến. Chỉ ghi, không trả gì về.
///
/// Đây là loại phụ thuộc "ra ngoài mà không có kết quả": cách duy nhất để
/// biết use case có dùng nó là hỏi xem nó đã được gọi thế nào.
abstract interface class BookingLog {
  /// Một lần đặt chuyến thành công.
  void booked(Trip trip);

  /// Một lần đặt chuyến đã gửi đi nhưng thất bại.
  void failed(QuoteRequest request, TripFailure failure);
}

import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote_request.dart';

/// Cổng vào dữ liệu chuyến mà use case cần, do domain định nghĩa.
///
/// Mọi phương thức ném `TripFailure` cho lỗi nhận ra được.
abstract interface class TripRepository {
  /// Đặt chuyến cho [request]. Gửi lại cùng [intentId] và cùng nội dung thì
  /// không tạo thêm chuyến.
  Future<Trip> requestTrip(QuoteRequest request, {required String intentId});

  /// Đọc lại chuyến [id] từ dịch vụ.
  Future<Trip> getTrip(String id);
}

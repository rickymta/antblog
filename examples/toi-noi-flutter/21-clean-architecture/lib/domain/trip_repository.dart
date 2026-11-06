import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote_request.dart';

/// Cổng vào dữ liệu chuyến mà domain cần, do domain định nghĩa.
///
/// Domain **sở hữu** interface này; lớp data **thực thi** nó. Mũi tên phụ
/// thuộc vì vậy chỉ đi một chiều: data biết domain, domain không biết data.
///
/// Mọi phương thức ném [TripFailure] cho lỗi nhận ra được. Lỗi không nhận ra
/// không bị đổi và không bị nuốt: nơi gọi coi mọi thứ không phải [TripFailure]
/// là lỗi lập trình. Chúng có thể mang kiểu của tầng dưới (một `StateError`
/// trong tầng mạng lên tới đây dưới dạng `DioException` kiểu `unknown` bọc
/// nó), nên đừng bắt chúng; để lên trình báo lỗi của app.
abstract interface class TripRepository {
  /// Cước dịch vụ báo cho [request].
  Future<Money> quote(QuoteRequest request);

  /// Đặt chuyến cho [request]. [intentId] nhận diện một ý định đặt chuyến:
  /// gửi lại cùng [intentId] và cùng nội dung thì không tạo thêm chuyến.
  Future<Trip> requestTrip(QuoteRequest request, {required String intentId});

  /// Đọc lại chuyến [id].
  Future<Trip> getTrip(String id);
}

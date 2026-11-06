import 'package:dio/dio.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

/// Câu hiển thị cho người dùng. Không đưa nguyên `title` của server (tiếng
/// Anh, viết cho lập trình viên) lên giao diện; chỉ mã hỗ trợ (`traceId`) đi
/// kèm để đối chiếu log.
///
/// Đây mới là bản tối thiểu: bài 20 phân loại lỗi đầy đủ và thêm nút hành động.
({String message, String? supportCode}) describeError(Object error) {
  switch (error) {
    case ApiException(statusCode: 422, :final fieldErrors)
        when fieldErrors.containsKey('distanceKm'):
      return (
        message: 'Quãng đường phải nằm trong khoảng 0 đến 100 km.',
        supportCode: null,
      );
    case ApiException(statusCode: 409, :final traceId):
      return (
        message: 'Chuyến đã thay đổi. Hãy làm mới rồi thử lại.',
        supportCode: traceId,
      );
    case ApiException(statusCode: 404, :final traceId):
      return (message: 'Không tìm thấy chuyến này.', supportCode: traceId);
    case ApiException(statusCode: >= 500, :final traceId):
      return (
        message: 'Máy chủ đang gặp sự cố. Hãy thử lại sau ít phút.',
        supportCode: traceId,
      );
    case ApiException(:final traceId):
      return (message: 'Yêu cầu không được chấp nhận.', supportCode: traceId);
    case DtoFormatException():
      return (
        message: 'Dữ liệu máy chủ trả về không đọc được.',
        supportCode: null,
      );
    case DioException():
      return (message: 'Không kết nối được tới máy chủ.', supportCode: null);
    default:
      return (message: 'Đã có lỗi không mong muốn.', supportCode: null);
  }
}

import 'package:dio/dio.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'connectivity.dart';

/// Việc người dùng có thể làm ngay trên màn hình lỗi.
enum FailureAction {
  /// Gửi lại đúng thao tác vừa thất bại.
  retry,

  /// Mở cài đặt mạng của hệ điều hành.
  openSettings,
}

/// Điều hiển thị cho một lỗi: câu chữ, mã hỗ trợ và các nút.
typedef FailureInfo = ({
  String message,
  String? supportCode,
  Set<FailureAction> actions,
});

const _both = {FailureAction.retry, FailureAction.openSettings};
const _retryOnly = {FailureAction.retry};
const _none = <FailureAction>{};

/// Request bị huỷ (người dùng đổi ý, thoát màn hình): không phải thất bại, và
/// `toApiFailure` cố ý không đổi nó thành [ApiFailure].
bool isCancellation(Object error) =>
    error is DioException && error.type == DioExceptionType.cancel;

/// Chọn câu chữ và hành động cho [error].
///
/// Nút ở đây là quyết định của lớp trình bày, không suy ra từ
/// `ApiFailure.retryable` (nghĩa là "máy tự thử lại được"): `ParseFailure` không
/// retryable nhưng vẫn có nút Thử lại bấm tay, vì người dùng có thể vừa đăng
/// nhập Wi-Fi công cộng. Huỷ request không qua hàm này (xem [isCancellation]).
/// `switch` trên `ApiFailure` không có nhánh mặc định: thêm một loại lỗi mới ở
/// `toi_noi_api` thì file này không biên dịch cho tới khi ta quyết định cách
/// hiển thị nó. [link] chỉ làm câu chữ chính xác hơn, không quyết định có hiện
/// nút hay không: connectivity báo "có mạng" không bảo đảm gọi được server.
///
/// Không đưa `title` của server (tiếng Anh, viết cho lập trình viên) lên giao
/// diện; chỉ `traceId` đi kèm dưới dạng mã hỗ trợ.
FailureInfo describeFailure(Object error, {LinkType? link}) {
  if (error is! ApiFailure) {
    return (
      message: 'Đã có lỗi không mong muốn.',
      supportCode: null,
      actions: _none,
    );
  }
  switch (error) {
    case NetworkFailure(reason: NetworkReason.badCertificate):
      return (
        message: 'Kết nối không an toàn: chứng chỉ của máy chủ không hợp lệ.',
        supportCode: null,
        actions: _none,
      );
    case NetworkFailure():
      return (
        message: link == LinkType.none
            ? 'Thiết bị đang không có kết nối mạng. Bật Wi-Fi hoặc dữ liệu di '
                  'động rồi thử lại.'
            : 'Không tới được máy chủ. Mạng có thể đang yếu, hoặc Wi-Fi cần '
                  'đăng nhập trước.',
        supportCode: null,
        actions: _both,
      );
    case TimeoutFailure():
      return (
        message: 'Máy chủ phản hồi quá chậm. Hãy thử lại khi mạng ổn hơn.',
        supportCode: null,
        actions: _retryOnly,
      );
    case ServerFailure(:final traceId):
      return (
        message: 'Máy chủ đang gặp sự cố. Hãy thử lại sau ít phút.',
        supportCode: traceId,
        actions: _retryOnly,
      );
    case ClientFailure(statusCode: 422, :final fieldErrors)
        when fieldErrors.containsKey('distanceKm'):
      return (
        message: 'Quãng đường phải nằm trong khoảng 0 đến 100 km.',
        supportCode: null,
        actions: _none,
      );
    case ClientFailure(statusCode: 408):
      return (
        message: 'Máy chủ chờ yêu cầu quá lâu. Hãy thử lại khi mạng ổn hơn.',
        supportCode: null,
        actions: _retryOnly,
      );
    case ClientFailure(statusCode: 429, :final traceId):
      return (
        message:
            'Hệ thống đang nhận quá nhiều yêu cầu. Hãy thử lại sau ít phút.',
        supportCode: traceId,
        actions: _retryOnly,
      );
    case ClientFailure(statusCode: 409, :final traceId):
      return (
        message: 'Chuyến đã thay đổi. Hãy làm mới rồi thử lại.',
        supportCode: traceId,
        actions: _none,
      );
    case ClientFailure(statusCode: 404, :final traceId):
      return (
        message: 'Không tìm thấy chuyến này.',
        supportCode: traceId,
        actions: _none,
      );
    case ClientFailure(:final traceId):
      return (
        message: 'Yêu cầu không được chấp nhận.',
        supportCode: traceId,
        actions: _none,
      );
    case ParseFailure():
      // Thử lại cùng request thường cho cùng kết quả, trừ khi đây là trang
      // đăng nhập Wi-Fi trả về thay cho JSON; nút vẫn rẻ nên ta để người dùng thử.
      return (
        message:
            'Dữ liệu nhận về không đọc được. Nếu đang dùng Wi-Fi công cộng, '
            'hãy đăng nhập Wi-Fi đó rồi thử lại.',
        supportCode: null,
        actions: _retryOnly,
      );
  }
}

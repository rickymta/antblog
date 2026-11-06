import 'package:toi_noi_21/domain/domain.dart';

/// Điều hiển thị cho một lỗi: câu chữ, mã hỗ trợ và có nút Thử lại hay không.
typedef FailureInfo = ({String message, String? supportCode, bool canRetry});

/// Chọn câu chữ cho [error].
///
/// `switch` trên [TripFailure] không có nhánh mặc định: thêm một loại lỗi mới
/// ở domain thì file này không biên dịch cho tới khi ta quyết định câu chữ.
/// Việc có nút Thử lại hay không lấy từ `TripFailure.canRetryByUser`, quyết định
/// của domain, nên giao diện không tự đoán lại.
FailureInfo describeFailure(Object error) {
  if (error is! TripFailure) {
    return (
      message: 'Đã có lỗi không mong muốn.',
      supportCode: null,
      canRetry: false,
    );
  }
  final message = switch (error) {
    Unreachable(reason: UnreachableReason.insecure) =>
      'Kết nối không an toàn nên app không gửi yêu cầu.',
    Unreachable(reason: UnreachableReason.noConnection) =>
      'Không tới được máy chủ. Hãy kiểm tra mạng rồi thử lại.',
    Unreachable(reason: UnreachableReason.tooSlow) =>
      'Máy chủ phản hồi quá chậm. Hãy thử lại khi mạng ổn hơn.',
    ServiceFault() => 'Máy chủ đang gặp sự cố. Hãy thử lại sau ít phút.',
    Rejected(kind: RejectionKind.invalidDistance) =>
      'Quãng đường phải nằm trong khoảng 0 đến 100 km.',
    Rejected(kind: RejectionKind.notFound) => 'Không tìm thấy chuyến này.',
    Rejected(kind: RejectionKind.conflict) =>
      'Chuyến đã thay đổi. Hãy làm mới rồi thử lại.',
    Rejected(kind: RejectionKind.other) => 'Yêu cầu không được chấp nhận.',
    Unreadable() =>
      'Dữ liệu nhận về không đọc được. Nếu đang dùng Wi-Fi công cộng, '
          'hãy đăng nhập Wi-Fi đó rồi thử lại.',
  };
  return (
    message: message,
    supportCode: error.supportCode,
    canRetry: error.canRetryByUser,
  );
}

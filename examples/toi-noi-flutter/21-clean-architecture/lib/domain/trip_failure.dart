/// Vì sao không tới được dịch vụ.
enum UnreachableReason {
  /// Không có đường tới máy chủ (mất mạng, kết nối bị từ chối hoặc đứt).
  noConnection,

  /// Máy chủ không trả lời kịp.
  tooSlow,

  /// Kết nối không an toàn. Thử lại không giúp gì.
  insecure,
}

/// Lý do dịch vụ từ chối một yêu cầu.
enum RejectionKind {
  /// Quãng đường ngoài miền phục vụ.
  invalidDistance,

  /// Không có chuyến này.
  notFound,

  /// Yêu cầu mâu thuẫn với điều dịch vụ đã biết (ví dụ cùng ý định, nội dung khác).
  conflict,

  /// Từ chối vì lý do khác.
  other,
}

/// Mọi thất bại mà lớp trên được phép thấy khi dùng `TripRepository`.
///
/// Đây là lỗi **của domain**: nó nói điều người dùng làm được, không nói
/// "dio", "HTTP 503" hay "DioException". Lớp data đổi lỗi kỹ thuật thành các
/// biến thể này, nên giao diện không cần biết dữ liệu đến bằng đường nào.
sealed class TripFailure implements Exception {
  const TripFailure({this.supportCode, this.cause});

  /// Mã để người dùng đọc cho bộ phận hỗ trợ, nếu có.
  final String? supportCode;

  /// Lỗi gốc để ghi log. Là `Object?` nên domain không phải biết kiểu của nó.
  final Object? cause;

  /// Giao diện nên cho người dùng bấm Thử lại.
  ///
  /// Đây là quyết định của người dùng, không phải của máy: nó không nói máy
  /// được tự gửi lại an toàn (việc đó thuộc `RetryInterceptor` của lớp data).
  /// Với [Unreadable], thử lại tay là rẻ và đôi khi có ích (Wi-Fi công cộng
  /// trả trang đăng nhập), dù máy sẽ không tự thử.
  bool get canRetryByUser;
}

/// Không tới được dịch vụ.
final class Unreachable extends TripFailure {
  const Unreachable(this.reason, {super.cause});

  final UnreachableReason reason;

  @override
  bool get canRetryByUser => reason != UnreachableReason.insecure;

  @override
  String toString() => 'Unreachable(${reason.name})';
}

/// Dịch vụ gặp sự cố của chính nó.
final class ServiceFault extends TripFailure {
  const ServiceFault({super.supportCode, super.cause});

  @override
  bool get canRetryByUser => true;

  @override
  String toString() => 'ServiceFault(${supportCode ?? '-'})';
}

/// Dịch vụ nhận được yêu cầu và từ chối. Gửi lại y nguyên thì bị từ chối y nguyên.
final class Rejected extends TripFailure {
  const Rejected(this.kind, {super.supportCode, super.cause});

  final RejectionKind kind;

  @override
  bool get canRetryByUser => false;

  @override
  String toString() => 'Rejected(${kind.name})';
}

/// Dịch vụ trả về dữ liệu không đọc được.
final class Unreadable extends TripFailure {
  const Unreadable({super.cause});

  @override
  bool get canRetryByUser => true;

  @override
  String toString() => 'Unreadable()';
}

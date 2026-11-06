/// Lý do dịch vụ từ chối một yêu cầu.
enum RejectionKind {
  /// Quãng đường ngoài miền phục vụ (0 đến 100 km).
  invalidDistance,

  /// Cùng ý định nhưng nội dung khác với lần đã gửi.
  conflict,

  /// Không có chuyến này.
  notFound,
}

/// Thất bại mà lớp trên được phép thấy khi đặt chuyến.
///
/// Bản rút gọn của `TripFailure` ở lab 21: đủ ba biến thể để use case phân
/// biệt "không tới được", "dịch vụ lỗi" và "bị từ chối".
sealed class TripFailure implements Exception {
  const TripFailure();
}

/// Không tới được dịch vụ (mất mạng, quá chậm).
final class Unreachable extends TripFailure {
  const Unreachable();

  @override
  String toString() => 'Unreachable()';
}

/// Dịch vụ gặp sự cố của chính nó.
final class ServiceFault extends TripFailure {
  const ServiceFault();

  @override
  String toString() => 'ServiceFault()';
}

/// Dịch vụ nhận yêu cầu và từ chối.
final class Rejected extends TripFailure {
  const Rejected(this.kind);

  final RejectionKind kind;

  @override
  String toString() => 'Rejected(${kind.name})';
}

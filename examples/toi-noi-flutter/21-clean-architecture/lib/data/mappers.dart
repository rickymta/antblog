import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

/// Domain sang dây dẫn: đổi [QuoteRequest] thành DTO để gửi đi.
///
/// Chiều ngược lại (DTO sang `Trip`, `Money`) đã nằm trong `toi_noi_api`
/// (`TripDto.toDomain`, `QuoteDto.toMoney`) vì DTO và hàm đổi sống cùng nơi.
extension QuoteRequestMapper on QuoteRequest {
  QuoteRequestDto toDto() =>
      QuoteRequestDto(serviceKind: serviceKind, distanceKm: distanceKm);
}

/// Lỗi kỹ thuật sang lỗi của domain.
///
/// `switch` trên `ApiFailure` không có nhánh mặc định: thêm một loại lỗi mới ở
/// `toi_noi_api` thì hàm này không biên dịch cho tới khi ta quyết định domain
/// gọi nó là gì. `title` tiếng Anh của server không đi qua đây; chỉ `traceId`
/// thành `supportCode`.
extension ApiFailureMapper on ApiFailure {
  TripFailure toTripFailure() => switch (this) {
    NetworkFailure(reason: NetworkReason.badCertificate) => Unreachable(
      UnreachableReason.insecure,
      cause: this,
    ),
    NetworkFailure() => Unreachable(
      UnreachableReason.noConnection,
      cause: this,
    ),
    TimeoutFailure() => Unreachable(UnreachableReason.tooSlow, cause: this),
    ServerFailure(:final traceId) => ServiceFault(
      supportCode: traceId,
      cause: this,
    ),
    ClientFailure(statusCode: 422, :final fieldErrors, :final traceId)
        when fieldErrors.containsKey('distanceKm') =>
      Rejected(
        RejectionKind.invalidDistance,
        supportCode: traceId,
        cause: this,
      ),
    ClientFailure(statusCode: 404, :final traceId) => Rejected(
      RejectionKind.notFound,
      supportCode: traceId,
      cause: this,
    ),
    ClientFailure(statusCode: 409, :final traceId) => Rejected(
      RejectionKind.conflict,
      supportCode: traceId,
      cause: this,
    ),
    ClientFailure(:final traceId) => Rejected(
      RejectionKind.other,
      supportCode: traceId,
      cause: this,
    ),
    ParseFailure() => Unreadable(cause: this),
  };
}

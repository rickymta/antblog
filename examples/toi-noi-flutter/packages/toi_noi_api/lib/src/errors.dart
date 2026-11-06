import 'dart:convert';

/// Server trả mã không phải 2xx.
///
/// Nếu body là `problem+json` (cách service .NET và fake API trả lỗi) thì
/// [title], [fieldErrors] và [traceId] được đọc ra; nếu không (ví dụ trang HTML
/// của một proxy) thì [title] là mô tả chung theo mã. Phân loại lỗi thành
/// các nhóm hành động được (thử lại, đăng nhập lại, báo người dùng) thuộc
/// bài 20; ở đây chỉ mang dữ kiện lên.
final class ApiException implements Exception {
  const ApiException({
    required this.statusCode,
    required this.title,
    this.fieldErrors = const {},
    this.traceId,
  });

  /// Đọc [data] (body của response) thành [ApiException].
  ///
  /// [data] có thể là chuỗi JSON thô (client dùng `ResponseType.plain`) hoặc
  /// đã được giải mã thành `Map`. Chuỗi không phải JSON object, như trang
  /// HTML của proxy, cho tiêu đề chung `HTTP <mã>`.
  factory ApiException.fromResponse(int statusCode, Object? data) {
    if (data is String) {
      try {
        data = jsonDecode(data);
      } on FormatException {
        data = null;
      }
    }
    if (data is! Map<String, Object?>) {
      return ApiException(statusCode: statusCode, title: 'HTTP $statusCode');
    }
    final rawTitle = data['title'];
    final rawTrace = data['traceId'];
    final fieldErrors = <String, List<String>>{};
    final rawErrors = data['errors'];
    if (rawErrors is Map<String, Object?>) {
      for (final entry in rawErrors.entries) {
        final messages = entry.value;
        if (messages is List<Object?>) {
          fieldErrors[entry.key] = [
            for (final message in messages)
              if (message is String) message,
          ];
        }
      }
    }
    return ApiException(
      statusCode: statusCode,
      title: rawTitle is String ? rawTitle : 'HTTP $statusCode',
      fieldErrors: fieldErrors,
      traceId: rawTrace is String ? rawTrace : null,
    );
  }

  final int statusCode;

  /// Tiêu đề do server đặt (tiếng Anh); đừng đưa nguyên văn lên giao diện.
  final String title;

  /// Tên trường -> thông báo, có khi server trả 4xx kiểm tra đầu vào.
  final Map<String, List<String>> fieldErrors;

  /// Mã để đối chiếu với log phía server.
  final String? traceId;

  @override
  String toString() =>
      'ApiException($statusCode, $title'
      '${traceId == null ? '' : ', trace $traceId'})';
}

/// Dữ liệu server trả không khớp hợp đồng của một DTO: body không phải JSON,
/// thiếu trường, sai kiểu, giá trị ngoài tập cho phép hoặc vi phạm bất biến
/// của domain.
///
/// Khác [ApiException]: ở đây server trả 2xx nhưng nội dung không dùng được.
///
/// Thông báo có thể chứa giá trị lấy từ payload (ví dụ `nhận "..."`). Nếu
/// payload có dữ liệu cá nhân, đừng gửi nguyên văn thông báo lên crash report
/// hay log dùng chung; ghi [dto] và [field] là đủ để tìm lỗi.
final class DtoFormatException extends FormatException {
  DtoFormatException(this.dto, this.field, String problem)
    : super('$dto.$field: $problem');

  /// Tên DTO, ví dụ `TripDto`.
  final String dto;

  /// Tên trường JSON lỗi, hoặc `$` khi cả payload sai.
  final String field;
}

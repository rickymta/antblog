import 'package:json_annotation/json_annotation.dart';
import 'package:toi_noi_api/toi_noi_api.dart' show DtoFormatException;

/// Giải một DTO sinh mã và giữ nguyên hợp đồng lỗi của `toi_noi_api`.
///
/// Bản viết tay ném [DtoFormatException] kèm tên DTO và trường; `toApiFailure`
/// (bài 20) đổi nó thành `ParseFailure`. Code sinh với `checked: true` ném
/// [CheckedFromJsonException]. Hàm này dịch lỗi đó về cùng loại, để phần còn
/// lại của app không biết DTO nào do tay viết, DTO nào do máy sinh.
T decodeDto<T>(
  String dto,
  Object? json,
  T Function(Map<String, dynamic> json) build,
) {
  if (json is! Map<String, Object?>) {
    throw DtoFormatException(dto, r'$', 'cần một JSON object');
  }
  try {
    return build(json);
  } on CheckedFromJsonException catch (e) {
    throw DtoFormatException(
      dto,
      e.key ?? r'$',
      e.message ?? e.innerError.toString(),
    );
  }
}

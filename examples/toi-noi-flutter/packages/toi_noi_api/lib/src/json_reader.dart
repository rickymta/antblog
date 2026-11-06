import 'errors.dart';

/// Số nguyên lớn nhất mà mọi nền tảng (kể cả JavaScript) biểu diễn chính xác:
/// 2^53. Tiền, mét và phiên bản vượt mức này là dữ liệu hỏng, không phải
/// chuyến thật.
const maxSafeInteger = 9007199254740992;

/// Đọc từng trường của một JSON object với kiểu được kiểm rõ ràng.
///
/// Mỗi hàm ném [DtoFormatException] kèm tên DTO và tên trường, để lỗi cho biết
/// ngay trường nào sai thay vì một `TypeError` ở chỗ khác.
final class JsonReader {
  JsonReader._(this.dto, this._json);

  /// Bọc [raw] nếu nó là JSON object, ngược lại ném [DtoFormatException].
  factory JsonReader.of(String dto, Object? raw) {
    if (raw is Map<String, Object?>) {
      return JsonReader._(dto, raw);
    }
    throw DtoFormatException(dto, r'$', 'cần một JSON object');
  }

  final String dto;
  final Map<String, Object?> _json;

  DtoFormatException _error(String field, String problem) =>
      DtoFormatException(dto, field, problem);

  String string(String field) {
    final value = _json[field];
    if (value is String) {
      return value;
    }
    throw _error(
      field,
      _json.containsKey(field) ? 'cần chuỗi' : 'thiếu trường',
    );
  }

  /// Chuỗi hoặc `null`; trường vắng mặt coi như `null`.
  String? stringOrNull(String field) {
    final value = _json[field];
    if (value == null || value is String) {
      return value as String?;
    }
    throw _error(field, 'cần chuỗi hoặc null');
  }

  /// Số nguyên (`int`) không âm. `5000.0` bị từ chối vì là `double`.
  int nonNegativeInt(String field) {
    final value = _json[field];
    if (value is! int) {
      throw _error(
        field,
        _json.containsKey(field) ? 'cần số nguyên' : 'thiếu trường',
      );
    }
    if (value < 0) {
      throw _error(field, 'không được âm');
    }
    if (value > maxSafeInteger) {
      throw _error(field, 'quá lớn (tối đa 2^53)');
    }
    return value;
  }

  /// Số hữu hạn (`int` hoặc `double` của JSON) đọc thành `double`.
  double finiteDouble(String field) {
    final value = _json[field];
    if (value is num && value.isFinite) {
      return value.toDouble();
    }
    throw _error(
      field,
      _json.containsKey(field) ? 'cần số hữu hạn' : 'thiếu trường',
    );
  }

  /// Số đồng không âm. Nhận `25500` và cả `25500.0` (một số backend
  /// serialize `decimal` có dấu thập phân) nhưng từ chối `25500.5`.
  int wholeVnd(String field) {
    final value = _json[field];
    if (value is int) {
      return nonNegativeInt(field);
    }
    if (value is double) {
      if (value.isFinite && value == value.truncateToDouble()) {
        if (value < 0) {
          throw _error(field, 'không được âm');
        }
        if (value > maxSafeInteger) {
          throw _error(field, 'quá lớn (tối đa 2^53)');
        }
        return value.toInt();
      }
      throw _error(field, 'số đồng phải là số nguyên');
    }
    throw _error(field, _json.containsKey(field) ? 'cần số' : 'thiếu trường');
  }

  /// Giá trị enum theo tên, không phân biệt hoa thường: `pickedUp` của fake
  /// API và `PickedUp` của service .NET đều nhận ra.
  T enumByName<T extends Enum>(String field, List<T> values) {
    final text = string(field).toLowerCase();
    for (final value in values) {
      if (value.name.toLowerCase() == text) {
        return value;
      }
    }
    throw _error(
      field,
      'giá trị "${_json[field]}" không thuộc {${values.map((v) => v.name).join(', ')}}',
    );
  }

  static final _instant = RegExp(
    r'^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(\.\d+)?(Z|[+-]\d{2}:\d{2})$',
  );

  /// Thời điểm ISO-8601 dạng `2026-11-06T03:30:00.000Z` hoặc `...+07:00`, trả
  /// về UTC.
  ///
  /// Hai điều `DateTime.parse` tự làm mà ta không muốn: chuỗi thiếu múi giờ
  /// bị hiểu là giờ địa phương của máy (hai máy khác múi giờ đọc ra hai thời
  /// điểm khác nhau), và ngày giờ tràn như `2026-13-45`, `T25:61:61` hay độ
  /// lệch `+99:99` bị cuộn sang ngày khác thay vì báo lỗi. Cả hai đều bị từ
  /// chối ở đây.
  DateTime utcInstant(String field) {
    final text = string(field);
    final match = _instant.firstMatch(text);
    if (match != null) {
      final year = int.parse(match[1]!);
      final month = int.parse(match[2]!);
      final day = int.parse(match[3]!);
      final hour = int.parse(match[4]!);
      final minute = int.parse(match[5]!);
      final second = int.parse(match[6]!);
      final probe = DateTime.utc(year, month, day, hour, minute, second);
      final offset = match[8]!;
      final offsetOk =
          offset == 'Z' ||
          (int.parse(offset.substring(1, 3)) <= 23 &&
              int.parse(offset.substring(4, 6)) <= 59);
      final inRange =
          offsetOk &&
          probe.month == month &&
          probe.day == day &&
          probe.hour == hour &&
          probe.minute == minute &&
          probe.second == second;
      final parsed = inRange ? DateTime.tryParse(text) : null;
      if (parsed != null) {
        return parsed.toUtc();
      }
    }
    throw _error(field, 'cần ISO-8601 có múi giờ, hợp lệ, nhận "$text"');
  }

  /// Chỉ chấp nhận đúng [expected] (ví dụ tiền tệ `VND`).
  void expectString(String field, String expected) {
    final value = string(field);
    if (value != expected) {
      throw _error(field, 'chỉ hỗ trợ "$expected", nhận "$value"');
    }
  }
}

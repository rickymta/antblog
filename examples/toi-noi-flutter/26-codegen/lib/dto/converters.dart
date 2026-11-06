import 'package:json_annotation/json_annotation.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Số nguyên lớn nhất mà mọi nền tảng (kể cả JavaScript) biểu diễn chính xác:
/// 2^53. Giá trị lớn hơn là dữ liệu hỏng, không phải chuyến thật.
const maxSafeInteger = 9007199254740992;

/// Mét hoặc số phiên bản: `int` không âm, tối đa 2^53.
///
/// `json_serializable` mặc định đọc `int` bằng `(json as num).toInt()`, nên
/// `5000.5` thành `5000` và `-1` đi qua. Converter này từ chối cả hai, và từ
/// chối `5000.0` như `JsonReader.nonNegativeInt` của bản viết tay.
final class NonNegativeIntConverter implements JsonConverter<int, Object?> {
  const NonNegativeIntConverter();

  @override
  int fromJson(Object? json) {
    if (json is! int) {
      throw const FormatException('cần số nguyên');
    }
    if (json < 0) {
      throw const FormatException('không được âm');
    }
    if (json > maxSafeInteger) {
      throw const FormatException('quá lớn (tối đa 2^53)');
    }
    return json;
  }

  @override
  Object? toJson(int object) => object;
}

/// Số đồng không âm. Nhận `25500` và cả `25500.0` (một số backend serialize
/// `decimal` có dấu thập phân) nhưng từ chối `25500.5`.
final class WholeVndConverter implements JsonConverter<int, Object?> {
  const WholeVndConverter();

  @override
  int fromJson(Object? json) {
    final int value;
    if (json is int) {
      value = json;
    } else if (json is double) {
      if (!json.isFinite || json != json.truncateToDouble()) {
        throw const FormatException('số đồng phải là số nguyên');
      }
      if (json.abs() > maxSafeInteger) {
        throw const FormatException('quá lớn (tối đa 2^53)');
      }
      value = json.toInt();
    } else {
      throw const FormatException('cần số');
    }
    if (value < 0) {
      throw const FormatException('không được âm');
    }
    if (value > maxSafeInteger) {
      throw const FormatException('quá lớn (tối đa 2^53)');
    }
    return value;
  }

  @override
  Object? toJson(int object) => object;
}

/// Tiền tệ duy nhất Tới Nơi hỗ trợ. Từ chối mọi giá trị khác, kể cả thiếu.
final class VndCurrencyConverter implements JsonConverter<String, Object?> {
  const VndCurrencyConverter();

  @override
  String fromJson(Object? json) {
    if (json is! String) {
      throw const FormatException('cần chuỗi');
    }
    if (json != 'VND') {
      throw const FormatException('chỉ hỗ trợ "VND"');
    }
    return json;
  }

  @override
  Object? toJson(String object) => object;
}

/// Enum theo tên, không phân biệt hoa thường: `pickedUp` của fake API và
/// `PickedUp` của service .NET đều nhận ra. Mặc định của `json_serializable`
/// là so khớp đúng từng chữ.
abstract base class NameEnumConverter<T extends Enum>
    implements JsonConverter<T, Object?> {
  const NameEnumConverter(this.values);

  final List<T> values;

  @override
  T fromJson(Object? json) {
    if (json is! String) {
      throw const FormatException('cần chuỗi');
    }
    final text = json.toLowerCase();
    for (final value in values) {
      if (value.name.toLowerCase() == text) {
        return value;
      }
    }
    throw FormatException(
      'giá trị không thuộc {${values.map((v) => v.name).join(', ')}}',
    );
  }

  @override
  Object? toJson(T object) => object.name;
}

final class ServiceKindConverter extends NameEnumConverter<ServiceKind> {
  const ServiceKindConverter() : super(ServiceKind.values);
}

final class TripStatusConverter extends NameEnumConverter<TripStatus> {
  const TripStatusConverter() : super(TripStatus.values);
}

final _instant = RegExp(
  r'^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(\.\d+)?(Z|[+-]\d{2}:\d{2})$',
);

/// Thời điểm ISO-8601 có múi giờ, hợp lệ, trả về UTC.
///
/// `DateTime.parse` (thứ `json_serializable` gọi) hiểu chuỗi thiếu múi giờ là
/// giờ địa phương của máy, nên hai máy khác múi giờ đọc ra hai thời điểm khác
/// nhau. Converter này từ chối chuỗi thiếu múi giờ, ngày giờ không có thật
/// (tháng 13, giờ 25) và độ lệch ngoài `±23:59`.
final class UtcInstantConverter implements JsonConverter<DateTime, Object?> {
  const UtcInstantConverter();

  @override
  DateTime fromJson(Object? json) {
    if (json is! String) {
      throw const FormatException('cần chuỗi ISO-8601');
    }
    final match = _instant.firstMatch(json);
    if (match != null) {
      final month = int.parse(match[2]!);
      final day = int.parse(match[3]!);
      final hour = int.parse(match[4]!);
      final minute = int.parse(match[5]!);
      final second = int.parse(match[6]!);
      final probe = DateTime.utc(
        int.parse(match[1]!),
        month,
        day,
        hour,
        minute,
        second,
      );
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
      final parsed = inRange ? DateTime.tryParse(json) : null;
      if (parsed != null) {
        return parsed.toUtc();
      }
    }
    throw const FormatException('cần ISO-8601 có múi giờ và hợp lệ');
  }

  @override
  Object? toJson(DateTime object) => object.toUtc().toIso8601String();
}

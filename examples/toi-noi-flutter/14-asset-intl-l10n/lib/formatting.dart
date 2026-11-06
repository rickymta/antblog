import 'package:intl/intl.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Định dạng [money] theo [locale] với ký hiệu đồng.
///
/// `vi` cho `25.500 ₫` (dấu chấm nghìn, ký hiệu đứng sau, cách bằng U+00A0);
/// `en` cho `₫25,500`. VND không có phần lẻ nên `decimalDigits: 0`.
String formatVnd(Money money, String locale) {
  final format = NumberFormat.currency(
    locale: locale,
    name: 'VND',
    symbol: '₫',
    decimalDigits: 0,
  );
  return format.format(money.vnd);
}

/// Hiển thị thời điểm [utc] theo giờ của thiết bị và quy ước của [locale]:
/// ngày dạng ngắn, rồi giờ theo kiểu 12 hoặc 24 giờ của locale.
///
/// [utc] bắt buộc là UTC (đúng quy ước lưu và truyền của Tới Nơi). Mặc định
/// đổi sang múi giờ của máy bằng `toLocal()`. [offset] chỉ để thay múi giờ
/// của máy bằng một độ lệch cố định, giúp test không phụ thuộc máy chạy test.
String formatTripTime(DateTime utc, String locale, {Duration? offset}) {
  if (!utc.isUtc) {
    throw ArgumentError.value(utc, 'utc', 'phải là giờ UTC');
  }
  final shown = offset == null ? utc.toLocal() : utc.add(offset);
  return DateFormat.yMd(locale).add_jm().format(shown);
}

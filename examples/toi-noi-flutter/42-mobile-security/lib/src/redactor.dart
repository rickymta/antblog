/// Che dữ liệu nhạy cảm trong chuỗi trước khi ghi log.
///
/// Thuần Dart, không đọc đồng hồ, không I/O: cùng chuỗi vào thì cùng chuỗi ra,
/// nên test được bằng bảng ca. Đây là lớp phòng thủ thứ hai cho chuỗi tự do
/// do code app tạo ra; log HTTP đã có danh sách cho phép ở `SafeLogInterceptor`
/// (bài 19).
///
/// Không có bộ che nào bắt được mọi thứ: quy tắc nào cũng có chuỗi lọt qua.
/// Cách an toàn nhất vẫn là không đưa dữ liệu nhạy cảm vào câu log.
final class Redactor {
  const Redactor._(this._rules);

  /// Bộ quy tắc của Tới Nơi: số điện thoại, token, trường nhạy cảm theo tên
  /// và toạ độ.
  factory Redactor.standard() => Redactor._(_standardRules);

  /// Bộ quy tắc tự chọn (cho test hoặc app khác).
  const Redactor.withRules(List<RedactionRule> rules) : this._(rules);

  final List<RedactionRule> _rules;

  /// Trả về [input] đã che. Quy tắc áp dụng theo thứ tự khai báo.
  String redact(String input) {
    var result = input;
    for (final rule in _rules) {
      result = result.replaceAllMapped(rule.pattern, rule.replace);
    }
    return result;
  }

  /// Che đệ quy một cấu trúc kiểu JSON: giá trị của khoá nhạy cảm bị thay
  /// hoàn toàn, số của khoá `lat`/`lng`... làm tròn 2 chữ số thập phân, mọi
  /// chuỗi còn lại đi qua [redact].
  Object? redactJson(Object? value) => switch (value) {
    final Map<Object?, Object?> map => {
      for (final entry in map.entries)
        entry.key: switch (entry.value) {
          _ when isSensitiveKey('${entry.key}') => redactedMarker,
          final num degrees when _isCoordinateKey('${entry.key}') =>
            double.parse(_coarse('$degrees')),
          final other => redactJson(other),
        },
    },
    final List<Object?> list => [for (final item in list) redactJson(item)],
    final String text => redact(text),
    _ => value,
  };

  static const redactedMarker = '[redacted]';

  /// Tên khoá (không phân biệt hoa thường, bỏ `_` và `-`) được coi là nhạy cảm.
  static bool isSensitiveKey(String key) {
    final normalized = key.toLowerCase().replaceAll(RegExp('[_-]'), '');
    return _sensitiveKeys.contains(normalized);
  }
}

/// Một quy tắc: mẫu cần tìm và cách thay.
final class RedactionRule {
  const RedactionRule(this.name, this.pattern, this.replace);

  final String name;
  final RegExp pattern;
  final String Function(Match match) replace;
}

bool _isCoordinateKey(String key) => const {
  'lat',
  'lng',
  'lon',
  'latitude',
  'longitude',
}.contains(key.toLowerCase());

const _sensitiveKeys = {
  'password',
  'otp',
  'accesstoken',
  'refreshtoken',
  'token',
  'authorization',
  'secret',
  'clientsecret',
  'phone',
  'phonenumber',
};

// Tên khoá trong chuỗi dạng `key=value` hoặc `"key":"value"`.
const _keyAlternatives =
    'password|otp|access_?token|refresh_?token|token|secret|'
    'client_?secret|phone(?:_?number)?';

final _standardRules = <RedactionRule>[
  // JWT: ba đoạn base64url, đoạn đầu và đoạn hai bắt đầu bằng `eyJ` (`{"`).
  RedactionRule(
    'jwt',
    RegExp(r'eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]*'),
    (_) => '[jwt]',
  ),
  RedactionRule(
    'bearer',
    RegExp(r'Bearer\s+[^\s,;"]+', caseSensitive: false),
    (_) => 'Bearer [redacted]',
  ),
  // Chuỗi chữ-số dài >= 32 ký tự liền nhau (hex, base64 không có - và _):
  // trông như token. UUID có dấu gạch nên không bị bắt.
  RedactionRule(
    'opaque-token',
    RegExp(r'(?<![A-Za-z0-9])[A-Za-z0-9]{32,}(?![A-Za-z0-9])'),
    (_) => '[token]',
  ),
  // Số điện thoại Việt Nam: 0 hoặc +84/84 rồi 9 chữ số, cho phép khoảng
  // trắng, dấu chấm, gạch ngang giữa các chữ số.
  RedactionRule(
    'phone-vn',
    RegExp(r'(?<![A-Za-z0-9.])(?:\+?84|0)(?:[\s.-]?\d){9}(?!\d)'),
    (_) => '[phone]',
  ),
  // `key=value`, `key: value`, `"key":"value"`. Giữ tên khoá, bỏ giá trị.
  RedactionRule(
    'sensitive-field',
    RegExp(
      '(["\']?(?:$_keyAlternatives)["\']?\\s*[:=]\\s*)'
      '(?:"[^"]*"|\'[^\']*\'|[^\\s,;&}]+)',
      caseSensitive: false,
    ),
    (m) => '${m[1]}${Redactor.redactedMarker}',
  ),
  // Cặp toạ độ `10.762622, 106.660172`: giữ 2 chữ số thập phân (~1,1 km). Cần
  // từ 4 chữ số thập phân để không nuốt nhầm cặp số tiền như `25.500, 12.000`.
  RedactionRule(
    'coordinate-pair',
    RegExp(r'(-?\d{1,3}\.\d{4,})\s*,\s*(-?\d{1,3}\.\d{4,})'),
    (m) => '${_coarse(m[1]!)},${_coarse(m[2]!)}',
  ),
  // `lat=10.762622`, `"longitude": 106.660172`.
  RedactionRule(
    'coordinate-field',
    RegExp(
      r'''((?:lat|latitude|lng|lon|longitude)["']?\s*[:=]\s*)(-?\d{1,3}\.\d{3,})''',
      caseSensitive: false,
    ),
    (m) => '${m[1]}${_coarse(m[2]!)}',
  ),
];

/// Làm tròn về 2 chữ số thập phân (khoảng 1,1 km theo vĩ độ).
String _coarse(String degrees) {
  final value = double.parse(degrees);
  final text = value.toStringAsFixed(2);
  return text == '-0.00' ? '0.00' : text;
}

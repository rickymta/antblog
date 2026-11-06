/// Bốn khoá bắt buộc của `key.properties`, theo tài liệu Flutter.
const List<String> requiredKeyPropertyNames = [
  'storeFile',
  'storePassword',
  'keyAlias',
  'keyPassword',
];

/// Nội dung `android/key.properties`, đọc như `java.util.Properties` mà Gradle
/// dùng, đủ cho các dòng `khoá=giá trị` của file này.
class KeyProperties {
  KeyProperties._(this.values);

  /// Đọc dạng `.properties`: bỏ dòng trống và dòng bắt đầu bằng `#` hoặc `!`,
  /// tách ở dấu `=` hoặc `:` đầu tiên, giải mã `\\`, `\t`, `\n`, `\uXXXX`.
  /// Ném [FormatException] khi gặp `\u` không kèm đúng bốn chữ số hex, giống
  /// `IllegalArgumentException` của Java.
  factory KeyProperties.parse(String text) {
    final values = <String, String>{};
    for (final rawLine in text.split(RegExp(r'\r?\n'))) {
      final line = rawLine.trimLeft();
      if (line.isEmpty || line.startsWith('#') || line.startsWith('!')) {
        continue;
      }
      final separator = line.indexOf(RegExp('[=:]'));
      final key = separator < 0 ? line : line.substring(0, separator);
      final value = separator < 0 ? '' : line.substring(separator + 1);
      values[key.trim()] = _unescape(value.trimLeft());
    }
    return KeyProperties._(values);
  }

  final Map<String, String> values;

  /// Các khoá bắt buộc còn thiếu hoặc rỗng, theo thứ tự cố định.
  List<String> get missingKeys => [
    for (final name in requiredKeyPropertyNames)
      if ((values[name] ?? '').trim().isEmpty) name,
  ];

  bool get isComplete => missingKeys.isEmpty;

  String? operator [](String key) => values[key];

  /// Một dòng báo lỗi rõ ràng, hoặc `null` nếu đủ khoá.
  String? get problem {
    final missing = missingKeys;
    if (missing.isEmpty) return null;
    return 'key.properties thiếu khoá: ${missing.join(', ')}';
  }

  static String _unescape(String value) {
    final out = StringBuffer();
    for (var i = 0; i < value.length; i++) {
      final ch = value[i];
      if (ch != r'\') {
        out.write(ch);
        continue;
      }
      i++;
      if (i >= value.length) break; // dấu \ cuối dòng: Java coi là nối dòng
      switch (value[i]) {
        case 't':
          out.write('\t');
        case 'n':
          out.write('\n');
        case 'r':
          out.write('\r');
        case 'f':
          out.write('\f');
        case 'u':
          final hex = value.length >= i + 5
              ? value.substring(i + 1, i + 5)
              : '';
          final code = RegExp(r'^[0-9a-fA-F]{4}$').hasMatch(hex)
              ? int.parse(hex, radix: 16)
              : null;
          if (code == null) {
            throw const FormatException(r'Malformed \uxxxx encoding');
          }
          out.writeCharCode(code);
          i += 4;
        default:
          out.write(value[i]);
      }
    }
    return out.toString();
  }
}

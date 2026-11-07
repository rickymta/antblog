/// Đọc file `.xcconfig` thành map `KEY -> value`.
///
/// Chỉ hỗ trợ phần lab cần: dòng `KEY = value`, chú thích `//`, dòng trống và
/// dòng `#include` (bị bỏ qua). Khoá có điều kiện như `KEY[sdk=iphoneos*]`
/// được giữ nguyên cả phần trong ngoặc. Nếu một khoá xuất hiện nhiều lần thì
/// dòng sau thắng, giống cách Xcode xử lý trong một file.
///
/// Ném [FormatException] khi gặp dòng không phải `KEY = value`.
Map<String, String> parseXcconfig(String source) {
  final result = <String, String>{};
  final lines = source.split(RegExp(r'\r?\n'));
  for (var i = 0; i < lines.length; i++) {
    final line = lines[i].trim();
    if (line.isEmpty || line.startsWith('//') || line.startsWith('#include')) {
      continue;
    }
    final equals = _assignmentIndex(line);
    if (equals <= 0) {
      throw FormatException('Dòng ${i + 1} không phải KEY = value: "$line"');
    }
    final key = line.substring(0, equals).trim();
    var value = line.substring(equals + 1).trim();
    final comment = value.indexOf('//');
    if (comment >= 0) {
      value = value.substring(0, comment).trim();
    }
    if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) {
      value = value.substring(1, value.length - 1);
    }
    result[key] = value;
  }
  return result;
}

/// Vị trí dấu `=` của phép gán, bỏ qua dấu `=` nằm trong `[...]` của khoá
/// có điều kiện (`KEY[sdk=iphoneos*] = value`). Trả -1 nếu không có.
int _assignmentIndex(String line) {
  var depth = 0;
  for (var i = 0; i < line.length; i++) {
    final c = line[i];
    if (c == '[') {
      depth++;
    } else if (c == ']' && depth > 0) {
      depth--;
    } else if (c == '=' && depth == 0) {
      return i;
    }
  }
  return -1;
}

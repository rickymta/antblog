import 'dart:convert';

/// Bản đồ tên do `--save-obfuscation-map` ghi ra: một mảng JSON phẳng
/// `[tênGốc, tênĐãRối, tênGốc, tênĐãRối, ...]` (tài liệu Flutter, mục
/// "Obfuscation map").
///
/// Dùng để đọc lại stack trace của bản release đã làm rối. Bản đồ chỉ đúng với
/// đúng bản build tạo ra nó, nên phải lưu cùng mỗi bản phát hành.
final class ObfuscationMap {
  ObfuscationMap._(this._originalByObfuscated);

  /// Đọc nội dung file bản đồ. Ném [FormatException] nếu không phải mảng chuỗi
  /// có độ dài chẵn.
  factory ObfuscationMap.parse(String json) {
    final decoded = jsonDecode(json);
    if (decoded is! List || decoded.length.isOdd) {
      throw const FormatException(
        'Bản đồ phải là mảng JSON có số phần tử chẵn',
      );
    }
    final map = <String, String>{};
    for (var i = 0; i < decoded.length; i += 2) {
      final original = decoded[i];
      final obfuscated = decoded[i + 1];
      if (original is! String || obfuscated is! String) {
        throw FormatException('Phần tử $i không phải chuỗi');
      }
      map[obfuscated] = original;
    }
    return ObfuscationMap._(map);
  }

  final Map<String, String> _originalByObfuscated;

  /// Tên gốc của [obfuscated], hoặc chính nó nếu bản đồ không biết.
  String originalName(String obfuscated) =>
      _originalByObfuscated[obfuscated] ?? obfuscated;

  // Dòng stack trace của Dart: `#0      Lop.phuongThuc (tep:dong)` hoặc
  // `#0      Lop.phuongThuc (tep:dong:cot)`.
  static final _frame = RegExp(r'^(#\d+\s+)(.+?)( \()(.+?)(:\d+(?::\d+)?\))$');

  /// Dịch từng dòng frame của [trace] về tên gốc; dòng khác giữ nguyên.
  ///
  /// Chỉ dịch ở vị trí tên hàm và tên file của frame, để một từ thường như
  /// "state" trong thông báo lỗi không bị thay nhầm.
  String deobfuscateStackTrace(String trace) {
    return trace
        .split('\n')
        .map((line) {
          final match = _frame.firstMatch(line);
          if (match == null) return line;
          final symbol = match[2]!.split('.').map(originalName).join('.');
          return '${match[1]}$symbol${match[3]}${originalName(match[4]!)}'
              '${match[5]}';
        })
        .join('\n');
  }
}

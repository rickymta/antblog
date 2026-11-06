import 'dart:convert';
import 'dart:io';

import 'package:toi_noi_25/config/flavor.dart';

/// Tên ba file cấu hình, trùng tên enum [Flavor].
final flavorFileNames = [for (final flavor in Flavor.values) flavor.name];

/// JSON nguyên bản của `config/<name>.json`. `flutter test` chạy với thư mục
/// hiện hành là gốc project nên đường dẫn tương đối dùng được.
Map<String, Object?> readRawConfig(String path) {
  final decoded = jsonDecode(File(path).readAsStringSync());
  return Map<String, Object?>.from(decoded as Map<dynamic, dynamic>);
}

/// Cách công cụ Flutter biến JSON thành define: mỗi cặp thành `KEY=$value`
/// (`extractDartDefines` trong flutter_tools), tức giá trị nào cũng thành chuỗi
/// bằng nội suy: `15000` thành `15000`, `true` thành `true`, `null` thành `null`.
Map<String, String> asDefines(Map<String, Object?> json) => {
  for (final entry in json.entries) entry.key: '${entry.value}',
};

/// Hai bước trên gộp lại cho một file theo tên flavor.
Map<String, String> readFlavorDefines(String name) =>
    asDefines(readRawConfig('config/$name.json'));

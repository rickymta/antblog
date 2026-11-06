import 'dart:io';

import 'package:toi_noi_53/native_config_audit.dart';

/// Mọi file dưới `native-config/` (kể cả `.env.example`), khoá là đường dẫn
/// tương đối trong thư mục đó. Test chạy từ thư mục gốc của lab.
Map<String, String> readNativeConfig() {
  final root = Directory('native-config');
  return {
    for (final f in root.listSync(recursive: true).whereType<File>())
      f.path.substring('native-config/'.length): f.readAsStringSync(),
  };
}

/// Bản sao của [files] sau khi thay mọi [from] bằng [to] trong [path]. Ném lỗi nếu
/// [from] không có, để test "ca sai" không vô tình thành no-op.
Map<String, String> mutated(
  Map<String, String> files,
  String path,
  String from,
  String to,
) {
  final text = files[path];
  if (text == null || !text.contains(from)) {
    throw StateError('$path không chứa "$from"; sửa test.');
  }
  return {...files, path: text.replaceAll(from, to)};
}

/// Mã của các vấn đề, để so khớp ngắn gọn.
Set<String> codesOf(Map<String, String> files) => {
  for (final p in auditNativeConfig(files)) p.code,
};

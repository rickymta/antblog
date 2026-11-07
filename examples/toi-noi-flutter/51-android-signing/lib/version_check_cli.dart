import 'dart:io';

import 'app_version.dart';

/// Lõi của `bin/check_version_code.dart`, tách ra để test được.
///
/// Dùng: `check_version_code <pubspec.yaml> --previous <versionCode>`.
/// Trả mã thoát: 0 hợp lệ, 1 versionCode không tăng, 2 dùng sai hoặc file lỗi.
int runVersionCheck(List<String> args, StringSink out, StringSink err) {
  final previousFlag = args.indexOf('--previous');
  if (args.length != 3 || previousFlag != 1) {
    err.writeln(
      'Cách dùng: check_version_code <pubspec.yaml> --previous <versionCode>',
    );
    return 2;
  }
  final previous = int.tryParse(args[2]);
  if (previous == null || previous < 0) {
    err.writeln('--previous phải là số nguyên không âm: ${args[2]}');
    return 2;
  }

  try {
    final version = AppVersion.fromPubspec(File(args[0]).readAsStringSync());
    version.ensureIncreasesFrom(previous);
    out.writeln(
      'OK: $version (versionName ${version.versionName}, '
      'versionCode ${version.versionCode}, bản trước $previous)',
    );
    return 0;
  } on VersionCodeRegression catch (e) {
    err.writeln('LỖI: $e');
    return 1;
  } on FormatException catch (e) {
    err.writeln('LỖI: ${e.message}');
    return 2;
  } on FileSystemException catch (e) {
    err.writeln('LỖI: không đọc được ${e.path}');
    return 2;
  }
}

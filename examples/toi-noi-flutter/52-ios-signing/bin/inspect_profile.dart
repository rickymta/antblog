import 'dart:convert';
import 'dart:io';

import 'package:toi_noi_52/toi_noi_52.dart';

const _usage = '''
Dùng: dart run bin/inspect_profile.dart <profile.plist> [tuỳ chọn]

  --now=<ISO-8601>        thời điểm so hạn (mặc định: giờ hiện tại, UTC)
  --warn-days=<N>         ngưỡng cảnh báo sắp hết hạn (mặc định 30)
  --device=<UDID>         kiểm thiết bị có trong profile
  --entitlements=<plist>  kiểm quyền app xin (vd. Runner.entitlements)

Mã thoát: 0 ổn, 1 có cảnh báo, 2 lỗi dùng hoặc file không đọc được.''';

void main(List<String> args) => exitCode = run(args);

/// Chạy công cụ và trả mã thoát; tách khỏi `main` để dễ đọc.
int run(List<String> args) {
  String? path;
  DateTime? now;
  var warnDays = 30;
  String? udid;
  String? entitlementsPath;

  for (final arg in args) {
    if (arg.startsWith('--now=')) {
      now = DateTime.tryParse(arg.substring(6));
      if (now == null) {
        stderr.writeln('--now không phải ISO-8601');
        return 2;
      }
    } else if (arg.startsWith('--warn-days=')) {
      final parsed = int.tryParse(arg.substring(12));
      if (parsed == null || parsed < 0) {
        stderr.writeln('--warn-days phải là số nguyên không âm');
        return 2;
      }
      warnDays = parsed;
    } else if (arg.startsWith('--device=')) {
      udid = arg.substring(9);
    } else if (arg.startsWith('--entitlements=')) {
      entitlementsPath = arg.substring(15);
    } else if (arg.startsWith('--')) {
      stderr.writeln('Tuỳ chọn lạ: $arg\n$_usage');
      return 2;
    } else {
      path = arg;
    }
  }
  if (path == null) {
    stderr.writeln(_usage);
    return 2;
  }

  final file = File(path);
  if (!file.existsSync()) {
    stderr.writeln('Không thấy file: $path');
    return 2;
  }
  final bytes = file.readAsBytesSync();
  switch (sniffProfileFile(bytes)) {
    case ProfileFileFormat.cms:
      stderr.writeln(cmsHint);
      return 2;
    case ProfileFileFormat.unknown:
      stderr.writeln('Không nhận ra định dạng file (cần plist XML).');
      return 2;
    case ProfileFileFormat.xmlPlist:
      break;
  }

  try {
    final profile = ProvisioningProfile.parse(utf8.decode(bytes));
    final claimed = entitlementsPath == null
        ? const <String>[]
        : parsePlist(File(entitlementsPath).readAsStringSync()).keys;
    final report = inspectProfile(
      profile,
      now: now ?? DateTime.now().toUtc(),
      warnWithin: Duration(days: warnDays),
      claimed: claimed,
      udid: udid,
    );
    stdout.writeln(report);
    return report.hasWarnings ? 1 : 0;
  } on FormatException catch (e) {
    stderr.writeln('File không hợp lệ: ${e.message}');
    return 2;
  } on FileSystemException catch (e) {
    stderr.writeln('Không đọc được file: ${e.message}');
    return 2;
  }
}

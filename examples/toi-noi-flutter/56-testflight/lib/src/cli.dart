import 'dart:io';

import 'beta_checklist.dart';
import 'plist.dart';

const usage = '''
Dùng: dart run bin/beta_check.dart <customer|driver> <Info.plist> [tuỳ chọn]

  --build-name=1.4.0        giá trị cho \$(FLUTTER_BUILD_NAME)
  --build-number=43         giá trị cho \$(FLUTTER_BUILD_NUMBER)
  --last-build=42           build number của bản tải lên gần nhất
  --entitlements=<file>     file entitlements (plist XML) để kiểm aps-environment

Mã thoát: 0 không có lỗi (cảnh báo vẫn in), 1 có lỗi, 2 dùng sai hoặc không đọc được file.''';

/// Chạy công cụ dòng lệnh; ghi kết quả vào [out] và [err], trả mã thoát.
int runBetaCheck(List<String> args, StringSink out, StringSink err) {
  final positional = <String>[];
  final options = <String, String>{};
  for (final a in args) {
    if (a.startsWith('--')) {
      final eq = a.indexOf('=');
      if (eq < 0) {
        err.writeln('Tuỳ chọn thiếu giá trị: $a\n$usage');
        return 2;
      }
      options[a.substring(2, eq)] = a.substring(eq + 1);
    } else {
      positional.add(a);
    }
  }
  const known = {'build-name', 'build-number', 'last-build', 'entitlements'};
  final unknown = options.keys.where((k) => !known.contains(k));
  if (unknown.isNotEmpty) {
    err.writeln(
      'Tuỳ chọn lạ: ${unknown.map((k) => '--$k').join(', ')}\n$usage',
    );
    return 2;
  }
  if (positional.length != 2) {
    err.writeln(usage);
    return 2;
  }
  final role = AppRole.values.where((r) => r.name == positional[0]);
  if (role.isEmpty) {
    err.writeln(
      'Vai trò phải là customer hoặc driver, nhận "${positional[0]}"',
    );
    return 2;
  }

  Map<String, Object?> read(String path) {
    final file = File(path);
    if (!file.existsSync()) {
      throw FileSystemException('Không thấy file', path);
    }
    return parsePlist(file.readAsStringSync());
  }

  try {
    final info = read(positional[1]);
    final entitlementsPath = options['entitlements'];
    final entitlements = entitlementsPath == null
        ? null
        : read(entitlementsPath);
    final checklist = BetaChecklist(
      role: role.first,
      lastUploadedBuild: options['last-build'],
      buildSettings: {
        if (options['build-name'] != null)
          'FLUTTER_BUILD_NAME': options['build-name']!,
        if (options['build-number'] != null)
          'FLUTTER_BUILD_NUMBER': options['build-number']!,
      },
    );
    final issues = checklist.check(info, entitlements: entitlements);
    for (final issue in issues) {
      out.writeln(issue);
    }
    out.writeln(
      issues.ok
          ? 'Không có lỗi (${issues.length} cảnh báo).'
          : '${issues.errors.length} lỗi, '
                '${issues.length - issues.errors.length} cảnh báo.',
    );
    return issues.ok ? 0 : 1;
  } on FileSystemException catch (e) {
    err.writeln('${e.message}: ${e.path}');
    return 2;
  } on FormatException catch (e) {
    err.writeln(e.message);
    return 2;
  } on ArgumentError catch (e) {
    err.writeln('Tham số không hợp lệ: ${e.message}');
    return 2;
  }
}

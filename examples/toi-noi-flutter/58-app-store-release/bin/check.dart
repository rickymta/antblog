import 'dart:io';

import 'package:toi_noi_58/toi_noi_58.dart';

/// In kết quả checklist của một app: `dart run bin/check.dart customer`.
///
/// Mã thoát 1 nếu có lỗi. Không dùng mạng: URL chỉ được kiểm cú pháp, không
/// được mở thử.
void main(List<String> args) {
  final app = StoreApp.values.asNameMap()[args.isEmpty ? 'customer' : args[0]];
  if (app == null) {
    stderr.writeln(
      'Dùng: dart run bin/check.dart [customer|driver] [--update]',
    );
    exit(64);
  }
  final root = Directory.current;
  final input = loadReleaseInput(
    root,
    app,
    isFirstVersion: !args.contains('--update'),
  );
  final findings = [
    ...ReleaseChecklist(input).run(),
    ...checkScreenshots(
      iphone: loadScreenshots(
        Directory('store/${app.name}/screenshots/iphone'),
      ),
      ipad: loadScreenshots(Directory('store/${app.name}/screenshots/ipad')),
      supportsIpad: false,
    ),
  ];
  stdout.writeln('${app.bundleId}: ${findings.length} phát hiện');
  findings.forEach(stdout.writeln);
  exit(findings.any((f) => f.severity == Severity.error) ? 1 : 0);
}

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

final scriptPath = path(
  'native-config/customer/ios/scripts/copy-google-service-info.sh',
);
final hasPlistBuddy = File('/usr/libexec/PlistBuddy').existsSync();
final skipAll = Platform.isWindows ? 'cần sh' : null;

/// Dựng một "PROJECT_DIR" giả và chạy script như Xcode sẽ chạy.
class Build {
  Build() : root = Directory.systemTemp.createTempSync('tn50_copy_') {
    projectDir = Directory('${root.path}/ios')..createSync();
    targetDir = Directory('${root.path}/out/Runner.app')
      ..createSync(recursive: true);
  }

  final Directory root;
  late final Directory projectDir;
  late final Directory targetDir;

  /// Chép plist của [flavor] (bản `.example`) thành file thật trong PROJECT_DIR.
  void placeRealPlist(String flavor, {String? bundleId}) {
    final dir = Directory('${projectDir.path}/Runner/Firebase/$flavor')
      ..createSync(recursive: true);
    var text = read(
      native(
        'customer',
        'ios/Runner/Firebase/$flavor/GoogleService-Info.plist.example',
      ),
    );
    if (bundleId != null) {
      text = text.replaceFirst(
        RegExp(r'(?<=<key>BUNDLE_ID</key>\s*<string>)[^<]*'),
        bundleId,
      );
    }
    File('${dir.path}/GoogleService-Info.plist').writeAsStringSync(text);
  }

  ProcessResult run({String? flavor, required String bundleId}) {
    return Process.runSync(
      'sh',
      [scriptPath],
      includeParentEnvironment: false,
      environment: {
        'PATH': '/usr/bin:/bin',
        'TN_FLAVOR': ?flavor,
        'PROJECT_DIR': projectDir.path,
        'TARGET_BUILD_DIR': targetDir.parent.path,
        'UNLOCALIZED_RESOURCES_FOLDER_PATH': 'Runner.app',
        'PRODUCT_BUNDLE_IDENTIFIER': bundleId,
      },
    );
  }

  File get copied => File('${targetDir.path}/GoogleService-Info.plist');
  void dispose() => root.deleteSync(recursive: true);
}

void main() {
  late Build build;
  setUp(() => build = Build());
  tearDown(() => build.dispose());

  test('chép plist của đúng flavor vào gốc bundle', () {
    build.placeRealPlist('staging');
    build.placeRealPlist('dev');
    final result = build.run(
      flavor: 'staging',
      bundleId: 'vn.toinoi.customer.staging',
    );
    expect(result.exitCode, 0, reason: '${result.stderr}');
    expect(
      plistStrings(build.copied.readAsStringSync())['BUNDLE_ID'],
      'vn.toinoi.customer.staging',
    );
  }, skip: skipAll);

  test('ba flavor chép ra ba file khác nhau', () {
    final ids = <String>{};
    for (final flavor in flavors) {
      final b = Build()..placeRealPlist(flavor);
      final id = flavor == 'prod'
          ? 'vn.toinoi.customer'
          : 'vn.toinoi.customer.$flavor';
      expect(b.run(flavor: flavor, bundleId: id).exitCode, 0);
      ids.add(plistStrings(b.copied.readAsStringSync())['GOOGLE_APP_ID']!);
      b.dispose();
    }
    expect(ids, hasLength(3));
  }, skip: skipAll);

  test('chỉ có bản .example thì dừng và nói thiếu file', () {
    // Không gọi placeRealPlist: trong PROJECT_DIR không có GoogleService-Info.plist.
    final dir = Directory('${build.projectDir.path}/Runner/Firebase/dev')
      ..createSync(recursive: true);
    File('${dir.path}/GoogleService-Info.plist.example')
        .writeAsStringSync('mẫu');
    final result = build.run(flavor: 'dev', bundleId: 'vn.toinoi.customer.dev');
    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('error: thiếu'));
    expect(build.copied.existsSync(), isFalse);
  }, skip: skipAll);

  test('thiếu TN_FLAVOR (xcconfig chưa áp vào configuration) thì dừng', () {
    build.placeRealPlist('dev');
    final result = build.run(bundleId: 'vn.toinoi.customer.dev');
    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('TN_FLAVOR'));
    expect(build.copied.existsSync(), isFalse);
  }, skip: skipAll);

  test('plist của flavor khác (BUNDLE_ID lệch) bị chặn', () {
    build.placeRealPlist('staging', bundleId: 'vn.toinoi.customer.dev');
    final result = build.run(
      flavor: 'staging',
      bundleId: 'vn.toinoi.customer.staging',
    );
    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('BUNDLE_ID'));
    expect(build.copied.existsSync(), isFalse);
  }, skip: skipAll ?? (hasPlistBuddy ? null : 'cần PlistBuddy (macOS)'));

  test(
    'app tài xế dùng cùng script với app khách (không có gì riêng theo app)',
    () {
      expect(
        read('native-config/driver/ios/scripts/copy-google-service-info.sh'),
        read('native-config/customer/ios/scripts/copy-google-service-info.sh'),
      );
    },
  );

  test('cú pháp sh hợp lệ', () {
    expect(Process.runSync('sh', ['-n', scriptPath]).exitCode, 0);
  }, skip: skipAll);
}

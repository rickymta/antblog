import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

final nodeAvailable = () {
  try {
    return Process.runSync('node', ['--version']).exitCode == 0;
  } on ProcessException {
    return false;
  }
}();
final skip = nodeAvailable ? null : 'cần node trên PATH';

/// Bản sao của lab (flavors.json, config/, native-config/) để cố ý làm hỏng.
class Copy {
  Copy() : root = Directory.systemTemp.createTempSync('tn50_verify_') {
    File(path('flavors.json')).copySync('${root.path}/flavors.json');
    copyTree(Directory(path('config')), Directory('${root.path}/config'));
    copyTree(
      Directory(path('native-config')),
      Directory('${root.path}/native-config'),
    );
  }

  final Directory root;

  File file(String relative) => File('${root.path}/$relative');

  /// Sửa nội dung một file; thất bại nếu chuỗi cần sửa không có (tránh test vô nghĩa).
  void mutate(String relative, String from, String to) {
    final f = file(relative);
    final text = f.readAsStringSync();
    expect(text, contains(from), reason: 'không thấy "$from" trong $relative');
    f.writeAsStringSync(text.replaceFirst(from, to));
  }

  ProcessResult verify() => Process.runSync('node', [
    path('tool/verify-flavors.mjs'),
    '--root',
    root.path,
  ]);

  void dispose() => root.deleteSync(recursive: true);
}

const gradle = 'native-config/customer/android/app/build.gradle.kts';
const devPlist =
    'native-config/customer/ios/Runner/Firebase/dev/GoogleService-Info.plist.example';

void main() {
  test('trên lab thật: thoát 0 và báo đủ ba flavor, hai app', () {
    final result = Process.runSync('node', [path('tool/verify-flavors.mjs')]);
    expect(result.exitCode, 0, reason: '${result.stderr}');
    expect(result.stdout, contains('3 flavor x 2 app'));
  }, skip: skip);

  // Mỗi ca làm hỏng đúng một chỗ và mong script nêu đúng chỗ đó.
  final cases = <String, (void Function(Copy), String)>{
    'hậu tố staging bị sửa thành .dev: trùng ID và lệch hợp đồng': (
      (c) => c.mutate(gradle, '".staging"', '".dev"'),
      'trùng',
    ),
    'mất flavor prod': (
      (c) => c.mutate(gradle, 'create("prod") {', 'create("production") {'),
      'thiếu productFlavors "prod"',
    ),
    'applicationId gốc bị đổi': (
      (c) => c.mutate(
        gradle,
        'applicationId = "vn.toinoi.customer"',
        'applicationId = "vn.toinoi.cust"',
      ),
      'application ID vn.toinoi.cust.dev',
    ),
    'quay lại resValue cho tên app': (
      (c) => c.mutate(
        gradle,
        'applicationIdSuffix = ".dev"',
        'applicationIdSuffix = ".dev"\n            resValue("string", "app_name", "Dev")',
      ),
      'resValue',
    ),
    'tên app staging trùng dev': (
      (c) => c.mutate(
        'native-config/customer/android/app/src/staging/res/values/strings.xml',
        'Tới Nơi Stg',
        'Tới Nơi Dev',
      ),
      'app_name là "Tới Nơi Dev", hợp đồng là "Tới Nơi Stg"',
    ),
    'Debug-dev include nhầm flavor staging': (
      (c) => c.mutate(
        'native-config/customer/ios/Flutter/Debug-dev.xcconfig',
        'Flavors/dev.xcconfig',
        'Flavors/staging.xcconfig',
      ),
      '#include phải là',
    ),
    'Profile-dev quên đổi base sang Release.xcconfig': (
      (c) => c.mutate(
        'native-config/customer/ios/Flutter/Profile-dev.xcconfig',
        '#include "Release.xcconfig"',
        '#include "Debug.xcconfig"',
      ),
      'Profile-dev.xcconfig: #include phải là',
    ),
    'bundle ID iOS của staging bị sửa': (
      (c) => c.mutate(
        'native-config/customer/ios/Flutter/Flavors/staging.xcconfig',
        'vn.toinoi.customer.staging',
        'vn.toinoi.customer',
      ),
      'PRODUCT_BUNDLE_IDENTIFIER là vn.toinoi.customer, hợp đồng là vn.toinoi.customer.staging',
    ),
    'Info.plist viết cứng tên hiển thị': (
      (c) => c.mutate(
        'native-config/customer/ios/Runner/Info.plist',
        r'$(APP_DISPLAY_NAME)',
        'Tới Nơi',
      ),
      'CFBundleDisplayName',
    ),
    'plist mẫu chứa khoá giống thật': (
      (c) => c.mutate(devPlist, 'FAKE_API_KEY_REPLACE_ME', 'AIza${'x' * 35}'),
      'giống khoá thật',
    ),
    'BUNDLE_ID trong plist mẫu của staging là của dev': (
      (c) => c.mutate(
        'native-config/customer/ios/Runner/Firebase/staging/GoogleService-Info.plist.example',
        'vn.toinoi.customer.staging',
        'vn.toinoi.customer.dev',
      ),
      'BUNDLE_ID là vn.toinoi.customer.dev, cần vn.toinoi.customer.staging',
    ),
    'google-services.json mẫu không có client của flavor': (
      (c) => c.mutate(
        'native-config/customer/android/app/src/prod/google-services.json.example',
        '"vn.toinoi.customer"',
        '"vn.toinoi.other"',
      ),
      'không có client package_name vn.toinoi.customer',
    ),
    'file Firebase thật nằm trong repo': (
      (c) => c
          .file(
            'native-config/customer/android/app/src/dev/google-services.json',
          )
          .writeAsStringSync('{}'),
      'file Firebase thật không được nằm trong repo',
    ),
    'icon staging là bản sao của icon dev': (
      (c) => c
          .file(
            'native-config/customer/android/app/src/dev/res/mipmap-hdpi/ic_launcher.png',
          )
          .copySync(
            '${c.root.path}/native-config/customer/android/app/src/staging/res/mipmap-hdpi/ic_launcher.png',
          ),
      'icon của dev và staging giống hệt nhau',
    ),
    'icon sai cỡ': (
      (c) => c
          .file(
            'native-config/customer/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png',
          )
          .copySync(
            '${c.root.path}/native-config/customer/android/app/src/main/res/mipmap-mdpi/ic_launcher.png',
          ),
      'mipmap-mdpi cần 48x48',
    ),
    'file cấu hình Dart của staging khai FLAVOR dev': (
      (c) => c.mutate(
        'config/staging.json',
        '"FLAVOR": "staging"',
        '"FLAVOR": "dev"',
      ),
      'FLAVOR là dev, phải trùng tên file (staging)',
    ),
    'app tài xế: icon prod giống icon dev': (
      (c) => c
          .file(
            'native-config/driver/ios/Runner/Assets.xcassets/AppIcon-dev.appiconset/icon-1024.png',
          )
          .copySync(
            '${c.root.path}/native-config/driver/ios/Runner/Assets.xcassets/AppIcon.appiconset/icon-1024.png',
          ),
      'driver: iOS AppIcon: icon của dev và prod giống hệt nhau',
    ),
    'app tài xế: thiếu script chép plist': (
      (c) => c
          .file('native-config/driver/ios/scripts/copy-google-service-info.sh')
          .deleteSync(),
      'thiếu file native-config/driver/ios/scripts/copy-google-service-info.sh',
    ),
    'app tài xế: settings.gradle.kts thiếu plugin google-services': (
      (c) => c.mutate(
        'native-config/driver/android/settings.gradle.kts',
        'id("com.google.gms.google-services") version "4.5.0" apply false',
        '',
      ),
      'driver/android/settings.gradle.kts: thiếu plugin com.google.gms.google-services',
    ),
    'thiếu flavor driver staging': (
      (c) => c
          .file('native-config/driver/ios/Flutter/Flavors/staging.xcconfig')
          .deleteSync(),
      'thiếu file native-config/driver/ios/Flutter/Flavors/staging.xcconfig',
    ),
  };

  for (final entry in cases.entries) {
    test('bắt lỗi: ${entry.key}', () {
      final copy = Copy();
      addTearDown(copy.dispose);
      entry.value.$1(copy);
      final result = copy.verify();
      expect(
        result.exitCode,
        1,
        reason: 'script phải thất bại: ${result.stdout}',
      );
      expect(result.stderr, contains(entry.value.$2));
    }, skip: skip);
  }

  flavorCommandTests();

  test('bản sao chưa sửa gì vẫn sạch (kiểm chính bộ khung test)', () {
    final copy = Copy();
    addTearDown(copy.dispose);
    final result = copy.verify();
    expect(result.exitCode, 0, reason: '${result.stderr}');
  }, skip: skip);
}

/// `flavor-command.mjs` ghép hai cờ từ một tên flavor.
void flavorCommandTests() {
  ProcessResult command(List<String> args) =>
      Process.runSync('node', [path('tool/flavor-command.mjs'), ...args]);

  group('flavor-command.mjs', () {
    test('mỗi cặp (lệnh, flavor) ra đúng hai cờ đi cùng nhau', () {
      const expected = {
        'run': 'flutter run',
        'apk': 'flutter build apk',
        'appbundle': 'flutter build appbundle',
        'ipa': 'flutter build ipa',
      };
      for (final entry in expected.entries) {
        for (final flavor in flavors) {
          final result = command([entry.key, flavor, '--dry-run']);
          expect(result.exitCode, 0);
          expect(
            (result.stdout as String).trim(),
            '${entry.value} --flavor $flavor --dart-define-from-file=config/$flavor.json',
          );
        }
      }
    });

    test('cờ thêm sau -- được giữ nguyên ở cuối', () {
      final result = command(['run', 'dev', '--dry-run', '--', '-d', 'chrome']);
      expect(
        (result.stdout as String).trim(),
        endsWith('config/dev.json -d chrome'),
      );
    });

    test('flavor lạ hoặc lệnh lạ bị từ chối với mã 2', () {
      expect(command(['apk', 'qa', '--dry-run']).exitCode, 2);
      expect(command(['aab', 'dev', '--dry-run']).exitCode, 2);
      expect(command(['apk']).exitCode, 2);
    });
  }, skip: skip);
}

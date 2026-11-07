import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/patch_preflight.dart';

void main() {
  PatchVerdict v(List<String> paths) => PatchPreflight.classify(paths);

  test('chỉ code Dart và test là ứng viên patch', () {
    expect(
      v(['lib/features/quote/quote_screen.dart', 'test/quote_test.dart']),
      PatchVerdict.patchCandidate,
    );
  });

  test('danh sách rỗng không có gì ngăn patch', () {
    expect(v([]), PatchVerdict.patchCandidate);
  });

  test('đường dẫn tính từ gốc workspace vẫn nhận ra lib/', () {
    expect(v(['apps/customer/lib/main.dart']), PatchVerdict.patchCandidate);
  });

  test('thư mục native bắt buộc phát hành bản mới', () {
    for (final p in [
      'android/app/src/main/AndroidManifest.xml',
      'ios/Runner/Info.plist',
      'apps/customer/android/app/build.gradle.kts',
      'apps/customer/ios/Runner.xcodeproj/project.pbxproj',
      'macos/Runner/AppDelegate.swift',
    ]) {
      expect(v([p]), PatchVerdict.needsStoreRelease, reason: p);
    }
  });

  test('pubspec và asset cần người xem', () {
    for (final p in [
      'pubspec.yaml',
      'pubspec.lock',
      'assets/images/logo.png',
      'apps/customer/assets/l10n/app_vi.arb',
      'lib/l10n/app_vi.arb',
    ]) {
      expect(v([p]), PatchVerdict.needsReview, reason: p);
    }
  });

  test('một file native trong số nhiều file Dart là đủ để chặn', () {
    expect(
      v(['lib/a.dart', 'lib/b.dart', 'ios/Runner/AppDelegate.swift']),
      PatchVerdict.needsStoreRelease,
    );
  });

  test('kết quả chọn mức nặng nhất, không phụ thuộc thứ tự', () {
    final paths = [
      'pubspec.yaml',
      'lib/a.dart',
      'android/app/build.gradle.kts',
    ];
    expect(v(paths), PatchVerdict.needsStoreRelease);
    expect(v(paths.reversed.toList()), PatchVerdict.needsStoreRelease);
  });

  test('dấu gạch ngược kiểu Windows được chuẩn hoá', () {
    expect(
      v([r'android\app\build.gradle.kts']),
      PatchVerdict.needsStoreRelease,
    );
    expect(v([r'lib\main.dart']), PatchVerdict.patchCandidate);
  });

  test('file .dart ngoài lib và test không được coi là chắc chắn', () {
    expect(v(['tool/gen.dart']), PatchVerdict.needsReview);
  });

  test('file không phải Dart trong lib cần người xem', () {
    expect(v(['lib/shaders/route.frag']), PatchVerdict.needsReview);
  });
}

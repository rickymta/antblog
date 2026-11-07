import 'key_properties.dart';

/// Kiểm văn bản `android/app/build.gradle.kts` có ký bản release đúng cách.
///
/// Đây là kiểm tra văn bản, không phải chạy Gradle: nó bắt các lỗi cấu hình
/// phổ biến trước khi mất vài phút build. Trả về danh sách vấn đề, rỗng nếu ổn.
List<String> auditGradleSigning(String buildGradleKts) {
  final code = _stripComments(buildGradleKts);
  final problems = <String>[];

  if (!code.contains('rootProject.file("key.properties")')) {
    problems.add('không đọc key.properties từ rootProject (thư mục android/)');
  }
  for (final name in requiredKeyPropertyNames) {
    if (!code.contains('getProperty("$name")')) {
      problems.add('không lấy khoá $name từ key.properties');
    }
  }
  if (!code.contains('create("release")')) {
    problems.add('thiếu signingConfigs { create("release") }');
  }
  if (!RegExp(
    r'buildTypes\s*\{\s*release\s*\{[^}]*signingConfig\s*=\s*'
    r'signingConfigs\.getByName\("release"\)',
  ).hasMatch(code)) {
    problems.add('buildTypes.release không dùng signingConfigs "release"');
  }
  if (code.contains('signingConfigs.getByName("debug")') ||
      code.contains('signingConfigs.debug')) {
    problems.add('có đường lui ký bản release bằng khoá debug');
  }
  if (RegExp(r'(storePassword|keyPassword)\s*=\s*"').hasMatch(code)) {
    problems.add('mật khẩu keystore viết thẳng trong file Gradle');
  }
  if (!code.contains('GradleException') ||
      !RegExp(r'dependsOn\(\s*verifyReleaseSigning\s*\)').hasMatch(code) ||
      !code.contains('ReleaseBuild')) {
    problems.add(
      'không có bước dừng sớm (GradleException) gắn vào các task '
      'pre*ReleaseBuild',
    );
  }
  if (!code.contains('versionCode = flutter.versionCode') ||
      RegExp(r'versionCode\s*=\s*\d').hasMatch(code)) {
    problems.add('versionCode phải lấy từ flutter.versionCode, không gõ số');
  }
  return problems;
}

String _stripComments(String source) => source
    .replaceAll(RegExp(r'/\*.*?\*/', dotAll: true), '')
    .split('\n')
    .map((line) => line.replaceFirst(RegExp(r'\s//.*$|^//.*$'), ''))
    .join('\n');

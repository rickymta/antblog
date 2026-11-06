import 'package:yaml/yaml.dart';

/// Giới hạn `versionCode` của Google Play cho mỗi lần tải lên.
const int maxVersionCode = 2100000000;

/// ABI mà `flutter build apk --split-per-abi` tách ra, kèm hệ số Flutter cộng
/// vào `versionCode` (nhân 1000). Giá trị lấy từ `FlutterPluginConstants.kt`
/// của Flutter 3.47.4: x86 đã bị gỡ, số 3 để trống.
enum Abi {
  armeabiV7a(1),
  arm64V8a(2),
  x86_64(4);

  const Abi(this.versionCodeFactor);

  final int versionCodeFactor;
}

/// Lỗi khi `build` của bản mới không lớn hơn bản đã phát hành.
class VersionCodeRegression implements Exception {
  VersionCodeRegression({required this.previous, required this.candidate});

  final int previous;
  final int candidate;

  /// `true` nếu bằng nhau (trùng), `false` nếu thấp hơn (giảm).
  bool get isDuplicate => candidate == previous;

  @override
  String toString() => isDuplicate
      ? 'versionCode $candidate đã được dùng; Play Console từ chối bản trùng. '
            'Tăng số sau dấu + trong pubspec.yaml.'
      : 'versionCode giảm từ $previous xuống $candidate; mỗi bản tải lên phải '
            'lớn hơn bản trước. Tăng số sau dấu + trong pubspec.yaml.';
}

/// Phiên bản app theo `version: x.y.z+build` của `pubspec.yaml`.
///
/// `name` là phần trước dấu `+` (thành `versionName`), `build` là số nguyên
/// sau dấu `+` (thành `versionCode` khi Flutter build Android).
class AppVersion {
  const AppVersion._(this.name, this.build);

  /// Đọc chuỗi như `1.4.2+37` hoặc `1.4.2-beta.1+37`.
  ///
  /// Ném [FormatException] khi thiếu `+build`, sai dạng, `build` không dương
  /// hoặc vượt [maxVersionCode].
  factory AppVersion.parse(String raw) {
    final match = _pattern.firstMatch(raw.trim());
    if (match == null) {
      throw FormatException(
        'Phải có dạng x.y.z+build với build là số nguyên, ví dụ 1.4.2+37',
        raw,
      );
    }
    final build = int.tryParse(match.group(2)!);
    if (build == null || build < 1 || build > maxVersionCode) {
      throw FormatException(
        'build phải nằm trong khoảng 1..$maxVersionCode',
        raw,
      );
    }
    return AppVersion._(match.group(1)!, build);
  }

  /// Đọc trường `version` từ nội dung `pubspec.yaml`.
  factory AppVersion.fromPubspec(String pubspecYaml) {
    final doc = loadYaml(pubspecYaml);
    final version = doc is YamlMap ? doc['version'] : null;
    if (version is! String) {
      throw const FormatException('pubspec.yaml không có trường version');
    }
    return AppVersion.parse(version);
  }

  static final RegExp _pattern = RegExp(
    r'^(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)\+(\d+)$',
  );

  final String name;
  final int build;

  /// `versionName` trong APK/AAB.
  String get versionName => name;

  /// `versionCode` của AAB và của APK không tách ABI.
  int get versionCode => build;

  /// `versionCode` của APK tách theo [abi]: Flutter cộng `hệ số * 1000`.
  ///
  /// Ném [StateError] nếu kết quả vượt [maxVersionCode]. Lưu ý va chạm: với
  /// `build >= 1000`, `build 1037` trên armeabi-v7a trùng `build 37` trên
  /// arm64-v8a, nên đừng dùng build number lớn nếu còn phát hành APK tách ABI.
  int splitVersionCode(Abi abi) {
    final code = abi.versionCodeFactor * 1000 + build;
    if (code > maxVersionCode) {
      throw StateError('versionCode $code vượt giới hạn $maxVersionCode');
    }
    return code;
  }

  /// Kiểm bản này lớn hơn hẳn [previous] (versionCode đã phát hành).
  ///
  /// Ném [VersionCodeRegression] khi bằng hoặc thấp hơn.
  void ensureIncreasesFrom(int previous) {
    if (versionCode <= previous) {
      throw VersionCodeRegression(previous: previous, candidate: versionCode);
    }
  }

  @override
  String toString() => '$name+$build';
}

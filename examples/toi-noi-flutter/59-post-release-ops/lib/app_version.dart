/// Phiên bản app dạng `major.minor.patch+build`, khớp `version:` trong
/// `pubspec.yaml` (bài 54: `build` là số tăng từ CI, cũng là `versionCode` ở
/// Android).
///
/// So sánh theo thứ tự major, minor, patch, build. Chuỗi `1.4.0` không có
/// phần `+build` được hiểu là build 0 để làm mốc "từ 1.4.0 trở lên".
final class AppVersion implements Comparable<AppVersion> {
  const AppVersion(this.major, this.minor, this.patch, [this.build = 0])
    : assert(major >= 0 && minor >= 0 && patch >= 0 && build >= 0);

  /// Đọc `x.y.z` hoặc `x.y.z+build`. Ném [FormatException] với mọi dạng khác,
  /// kể cả pre-release (`1.4.0-beta`) và số âm.
  factory AppVersion.parse(String source) {
    final match = _pattern.firstMatch(source.trim());
    if (match == null) {
      throw FormatException('Phiên bản không hợp lệ: "$source"', source);
    }
    int part(int group) => int.parse(match.group(group) ?? '0');
    return AppVersion(part(1), part(2), part(3), part(4));
  }

  // Mỗi phần là chuỗi chữ số thuần. `\d` của Dart chỉ khớp 0-9 ASCII.
  static final RegExp _pattern = RegExp(
    r'^(\d{1,9})\.(\d{1,9})\.(\d{1,9})(?:\+(\d{1,10}))?$',
  );

  final int major;
  final int minor;
  final int patch;
  final int build;

  @override
  int compareTo(AppVersion other) {
    final byParts = <int>[
      major.compareTo(other.major),
      minor.compareTo(other.minor),
      patch.compareTo(other.patch),
      build.compareTo(other.build),
    ];
    return byParts.firstWhere((c) => c != 0, orElse: () => 0);
  }

  bool operator <(AppVersion other) => compareTo(other) < 0;
  bool operator <=(AppVersion other) => compareTo(other) <= 0;
  bool operator >(AppVersion other) => compareTo(other) > 0;
  bool operator >=(AppVersion other) => compareTo(other) >= 0;

  @override
  bool operator ==(Object other) =>
      other is AppVersion && compareTo(other) == 0;

  @override
  int get hashCode => Object.hash(major, minor, patch, build);

  @override
  String toString() => '$major.$minor.$patch+$build';
}

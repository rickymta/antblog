/// Giá trị `method` hiện hành của `xcodebuild -exportArchive` (trích từ
/// `xcodebuild -help` do Apple DTS dẫn lại; danh sách thực tế phụ thuộc loại
/// archive).
const currentExportMethods = <String>{
  'app-store-connect',
  'release-testing',
  'enterprise',
  'debugging',
  'developer-id',
  'mac-application',
  'validation',
};

/// Tên cũ đã deprecated và tên thay thế.
const deprecatedExportMethods = <String, String>{
  'app-store': 'app-store-connect',
  'ad-hoc': 'release-testing',
  'development': 'debugging',
};

/// Một vấn đề tìm thấy trong cấu hình ký.
class ConfigIssue {
  const ConfigIssue(this.code, this.message);

  /// Mã ngắn, ổn định, để test so sánh.
  final String code;
  final String message;

  @override
  String toString() => '$code: $message';
}

/// Kiểm một `ExportOptions.plist` đã đọc thành map.
///
/// [expectedTeamId] là Team ID của `DEVELOPMENT_TEAM` trong xcconfig; truyền
/// `null` để bỏ qua phép so.
List<ConfigIssue> lintExportOptions(
  Map<String, Object?> plist, {
  String? expectedTeamId,
}) {
  final issues = <ConfigIssue>[];

  final method = plist['method'];
  if (method is! String) {
    issues.add(const ConfigIssue('missing-method', 'Thiếu khoá method'));
  } else if (deprecatedExportMethods.containsKey(method)) {
    issues.add(
      ConfigIssue(
        'deprecated-method',
        'method "$method" đã deprecated, dùng '
            '"${deprecatedExportMethods[method]}"',
      ),
    );
  } else if (!currentExportMethods.contains(method)) {
    issues.add(ConfigIssue('unknown-method', 'method "$method" không hợp lệ'));
  }

  final teamId = plist['teamID'];
  if (teamId is! String || teamId.isEmpty) {
    issues.add(const ConfigIssue('missing-team', 'Thiếu teamID'));
  } else if (expectedTeamId != null && teamId != expectedTeamId) {
    issues.add(
      ConfigIssue(
        'team-mismatch',
        'teamID $teamId khác DEVELOPMENT_TEAM $expectedTeamId',
      ),
    );
  }

  if (plist['signingStyle'] == 'manual') {
    final profiles = plist['provisioningProfiles'];
    if (profiles is! Map<String, Object?> || profiles.isEmpty) {
      issues.add(
        const ConfigIssue(
          'manual-without-profiles',
          'signingStyle manual cần provisioningProfiles (bundle ID -> tên '
              'profile)',
        ),
      );
    }
  }
  return issues;
}

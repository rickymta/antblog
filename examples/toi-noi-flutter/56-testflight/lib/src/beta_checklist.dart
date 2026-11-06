/// Vai trò của app Tới Nơi: mỗi vai trò là một app riêng trên App Store Connect.
enum AppRole { customer, driver }

enum Severity { error, warning }

/// Một phát hiện của [BetaChecklist].
class CheckIssue {
  const CheckIssue(this.code, this.severity, this.subject, this.message);

  /// Mã ngắn, ổn định, để test so sánh.
  final String code;
  final Severity severity;

  /// Khoá Info.plist hay mục bị nói tới.
  final String subject;
  final String message;

  @override
  String toString() => '${severity.name} $code [$subject]: $message';
}

extension CheckIssues on List<CheckIssue> {
  List<CheckIssue> get errors =>
      where((i) => i.severity == Severity.error).toList();

  /// Không có lỗi nào (cảnh báo vẫn cho qua).
  bool get ok => errors.isEmpty;
}

const _whenInUse = 'NSLocationWhenInUseUsageDescription';
const _always = 'NSLocationAlwaysAndWhenInUseUsageDescription';
const _backgroundModes = 'UIBackgroundModes';
const _encryption = 'ITSAppUsesNonExemptEncryption';
const _encryptionCode = 'ITSEncryptionExportComplianceCode';

/// Kiểm một `Info.plist` (đã đọc thành map) trước khi tải lên TestFlight.
///
/// Đây là kiểm tra tĩnh trên file, không phải Apple: nó bắt những lỗi mà ta
/// biết chắc sẽ làm bản tải lên bị hỏi lại hay bị từ chối, trước khi tốn một
/// vòng build và xử lý. Nó không nói app có thuộc diện miễn trừ mã hoá hay không.
class BetaChecklist {
  const BetaChecklist({
    required this.role,
    this.lastUploadedBuild,
    this.buildSettings = const {},
  });

  final AppRole role;

  /// Build number của bản đã tải lên gần nhất, ví dụ `"41"`. `null` nghĩa là
  /// chưa có bản nào (không kiểm tăng dần).
  final String? lastUploadedBuild;

  /// Giá trị biến build như `FLUTTER_BUILD_NAME`, `FLUTTER_BUILD_NUMBER`.
  /// Template Flutter ghi `CFBundleVersion` là `$(FLUTTER_BUILD_NUMBER)`, nên
  /// phải thay biến trước khi kiểm.
  final Map<String, String> buildSettings;

  List<CheckIssue> check(
    Map<String, Object?> infoPlist, {
    Map<String, Object?>? entitlements,
  }) {
    final issues = <CheckIssue>[];
    _checkPurposeString(infoPlist, _whenInUse, issues);
    _checkLocation(infoPlist, issues);
    _checkPush(infoPlist, entitlements, issues);
    _checkEncryption(infoPlist, issues);
    _checkVersions(infoPlist, issues);
    return issues;
  }

  void _checkPurposeString(
    Map<String, Object?> plist,
    String key,
    List<CheckIssue> issues,
  ) {
    if (!plist.containsKey(key)) {
      issues.add(
        CheckIssue(
          'purpose-missing',
          Severity.error,
          key,
          'Thiếu $key. Apple ghi khoá này là bắt buộc khi app truy cập vị trí.',
        ),
      );
      return;
    }
    final value = plist[key];
    if (value is! String || value.trim().isEmpty) {
      issues.add(
        CheckIssue(
          'purpose-blank',
          Severity.error,
          key,
          '$key phải là chuỗi không rỗng, người dùng đọc nó trong hộp thoại.',
        ),
      );
    } else if (_placeholder.hasMatch(value)) {
      issues.add(
        CheckIssue(
          'purpose-placeholder',
          Severity.warning,
          key,
          'Mô tả còn dấu vết giữ chỗ (TODO, Lorem, xxx, biến \$(...)).',
        ),
      );
    }
  }

  static final _placeholder = RegExp(
    r'TODO|lorem|xxx|\$\(|\$\{',
    caseSensitive: false,
  );

  void _checkLocation(Map<String, Object?> plist, List<CheckIssue> issues) {
    final modes = _modes(plist, issues);
    final hasBackgroundLocation = modes?.contains('location') ?? false;
    switch (role) {
      case AppRole.customer:
        if (plist.containsKey(_always) || hasBackgroundLocation) {
          issues.add(
            const CheckIssue(
              'customer-background-location',
              Severity.warning,
              _always,
              'App khách chỉ cần vị trí khi đang dùng. Khai quyền nền là thêm '
                  'một điều phải giải thích với App Review.',
            ),
          );
        }
      case AppRole.driver:
        if (plist.containsKey(_always)) {
          _checkPurposeString(plist, _always, issues);
        }
        if (!hasBackgroundLocation) {
          issues.add(
            const CheckIssue(
              'background-mode-missing',
              Severity.error,
              'location',
              'App tài xế gửi vị trí khi chuyển sang app dẫn đường: '
                  'UIBackgroundModes phải có "location".',
            ),
          );
        }
    }
  }

  /// Đọc `UIBackgroundModes`. Trả `null` khi khoá không có hoặc sai kiểu; sai
  /// kiểu đã được ghi vào [issues].
  Set<String>? _modes(Map<String, Object?> plist, List<CheckIssue> issues) {
    final raw = plist[_backgroundModes];
    if (raw == null) return null;
    if (raw is! List || raw.any((e) => e is! String)) {
      issues.add(
        const CheckIssue(
          'background-modes-type',
          Severity.error,
          _backgroundModes,
          'UIBackgroundModes phải là mảng chuỗi.',
        ),
      );
      return null;
    }
    return raw.cast<String>().toSet();
  }

  void _checkPush(
    Map<String, Object?> plist,
    Map<String, Object?>? entitlements,
    List<CheckIssue> issues,
  ) {
    final modes = _modes(plist, <CheckIssue>[]) ?? const <String>{};
    if (!modes.contains('remote-notification')) {
      issues.add(
        const CheckIssue(
          'background-mode-missing',
          Severity.error,
          'remote-notification',
          'Tới Nơi nhận push báo trạng thái chuyến khi app ở nền: '
              'UIBackgroundModes phải có "remote-notification".',
        ),
      );
    }
    if (entitlements != null && !entitlements.containsKey('aps-environment')) {
      issues.add(
        const CheckIssue(
          'push-entitlement-missing',
          Severity.error,
          'aps-environment',
          'Entitlements thiếu aps-environment: capability Push Notifications '
              'chưa bật cho target này.',
        ),
      );
    }
  }

  void _checkEncryption(Map<String, Object?> plist, List<CheckIssue> issues) {
    if (!plist.containsKey(_encryption)) {
      issues.add(
        const CheckIssue(
          'encryption-key-missing',
          Severity.warning,
          _encryption,
          'Không có khoá này, App Store Connect hỏi câu hỏi export compliance '
              'ở mỗi bản tải lên và build dừng ở Missing Compliance cho tới '
              'khi có người trả lời.',
        ),
      );
      return;
    }
    final value = plist[_encryption];
    if (value is! bool) {
      issues.add(
        const CheckIssue(
          'encryption-key-type',
          Severity.error,
          _encryption,
          'Giá trị phải là <true/> hoặc <false/>, không phải chuỗi "NO"/"YES".',
        ),
      );
      return;
    }
    final code = plist[_encryptionCode];
    final hasCode = code is String && code.trim().isNotEmpty;
    if (value && !hasCode) {
      issues.add(
        const CheckIssue(
          'encryption-code-missing',
          Severity.error,
          _encryptionCode,
          'Khai true thì thường phải kèm mã Apple cấp sau khi duyệt tài liệu '
              'export compliance.',
        ),
      );
    } else if (!value && plist.containsKey(_encryptionCode)) {
      issues.add(
        const CheckIssue(
          'encryption-code-unused',
          Severity.warning,
          _encryptionCode,
          'Có mã compliance nhưng ITSAppUsesNonExemptEncryption là false: '
              'hai khai báo mâu thuẫn nhau.',
        ),
      );
    }
  }

  void _checkVersions(Map<String, Object?> plist, List<CheckIssue> issues) {
    final name = _resolved(plist, 'CFBundleShortVersionString', issues);
    if (name != null && !_marketing.hasMatch(name)) {
      issues.add(
        CheckIssue(
          'version-format',
          Severity.error,
          'CFBundleShortVersionString',
          'Apple yêu cầu ba số nguyên cách nhau bằng dấu chấm, ví dụ 1.4.0; '
              'nhận "$name".',
        ),
      );
    }

    final build = _resolved(plist, 'CFBundleVersion', issues);
    if (build == null) return;
    final parsed = parseBuild(build);
    if (parsed == null) {
      issues.add(
        CheckIssue(
          'build-format',
          Severity.error,
          'CFBundleVersion',
          'Cần một đến ba số nguyên không âm cách nhau bằng dấu chấm. Hệ thống '
              'bỏ qua phần từ số thứ tư, nên 1.2.3.4 và 1.2.3.5 sẽ trùng nhau; '
              'nhận "$build".',
        ),
      );
      return;
    }
    final last = lastUploadedBuild;
    if (last != null) {
      final lastParsed = parseBuild(last);
      if (lastParsed == null) {
        throw ArgumentError.value(last, 'lastUploadedBuild', 'không hợp lệ');
      }
      if (compareBuilds(parsed, lastParsed) <= 0) {
        issues.add(
          CheckIssue(
            'build-not-increasing',
            Severity.error,
            'CFBundleVersion',
            'Build $build không lớn hơn bản đã tải lên ($last). Mỗi lần tải '
                'lên cần build number khác; lab đòi thêm là phải tăng, để '
                'khớp với versionCode của Android và run_number của CI.',
          ),
        );
      }
    }
  }

  static final _marketing = RegExp(r'^\d+\.\d+\.\d+$');

  String? _resolved(
    Map<String, Object?> plist,
    String key,
    List<CheckIssue> issues,
  ) {
    final raw = plist[key];
    if (raw is! String || raw.trim().isEmpty) {
      issues.add(
        CheckIssue(
          'version-missing',
          Severity.error,
          key,
          '$key phải có giá trị chuỗi.',
        ),
      );
      return null;
    }
    final value = resolveBuildSettings(raw.trim(), buildSettings);
    if (value == null) {
      issues.add(
        CheckIssue(
          'version-unresolved',
          Severity.error,
          key,
          '$key là "$raw" nhưng biến build tương ứng chưa được cấp giá trị '
              '(flutter build ipa --build-name/--build-number).',
        ),
      );
    }
    return value;
  }
}

/// Thay `$(TÊN)` hoặc `${TÊN}` bằng giá trị trong [settings]. Trả `null` nếu
/// còn biến không có giá trị.
String? resolveBuildSettings(String value, Map<String, String> settings) {
  var unresolved = false;
  final result = value.replaceAllMapped(RegExp(r'\$[({]([A-Za-z0-9_]+)[)}]'), (
    m,
  ) {
    final v = settings[m.group(1)!];
    if (v == null || v.isEmpty) {
      unresolved = true;
      return m.group(0)!;
    }
    return v;
  });
  return unresolved ? null : result;
}

/// Đọc build number thành ba thành phần (thiếu thì là 0), hoặc `null` nếu sai
/// dạng: rỗng, ký tự lạ, nhiều hơn ba thành phần, số quá lớn.
List<int>? parseBuild(String build) {
  if (!RegExp(r'^\d+(\.\d+){0,2}$').hasMatch(build)) return null;
  final parts = <int>[];
  for (final p in build.split('.')) {
    final n = int.tryParse(p);
    if (n == null) return null;
    parts.add(n);
  }
  while (parts.length < 3) {
    parts.add(0);
  }
  return parts;
}

/// So hai build đã qua [parseBuild], theo từng thành phần số.
int compareBuilds(List<int> a, List<int> b) {
  for (var i = 0; i < 3; i++) {
    if (a[i] != b[i]) return a[i].compareTo(b[i]);
  }
  return 0;
}

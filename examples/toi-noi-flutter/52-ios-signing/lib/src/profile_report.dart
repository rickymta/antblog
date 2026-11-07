import 'provisioning_profile.dart';

/// Định dạng của file đầu vào.
enum ProfileFileFormat {
  /// Plist XML đã giải CMS: đọc được bằng [ProvisioningProfile.parse].
  xmlPlist,

  /// File `.mobileprovision` thật: CMS (DER) bọc quanh plist.
  cms,

  /// Không nhận ra, kể cả plist nhị phân (`bplist`).
  unknown,
}

/// Đoán định dạng từ các byte đầu. File `.mobileprovision` thật bắt đầu bằng
/// byte 0x30 (SEQUENCE của DER), plist XML bắt đầu bằng `<` sau khoảng trắng
/// hoặc BOM UTF-8.
ProfileFileFormat sniffProfileFile(List<int> bytes) {
  var i = 0;
  if (bytes.length >= 3 &&
      bytes[0] == 0xEF &&
      bytes[1] == 0xBB &&
      bytes[2] == 0xBF) {
    i = 3;
  }
  while (i < bytes.length &&
      const [0x20, 0x09, 0x0A, 0x0D].contains(bytes[i])) {
    i++;
  }
  if (i >= bytes.length) return ProfileFileFormat.unknown;
  if (bytes[i] == 0x3C) return ProfileFileFormat.xmlPlist;
  if (bytes[i] == 0x30) return ProfileFileFormat.cms;
  return ProfileFileFormat.unknown;
}

/// Hướng dẫn in ra khi gặp file CMS.
const cmsHint =
    'Đây là file CMS (.mobileprovision thật), script này chỉ đọc plist đã '
    'giải. Chạy trên macOS:\n'
    '  security cms -D -i <file>.mobileprovision > profile.plist\n'
    'rồi truyền profile.plist vào script.';

/// Kết quả kiểm một profile: các dòng mô tả và danh sách cảnh báo.
class ProfileReport {
  const ProfileReport({required this.lines, required this.warnings});

  final List<String> lines;
  final List<String> warnings;

  bool get hasWarnings => warnings.isNotEmpty;

  @override
  String toString() => [
    ...lines,
    for (final warning in warnings) 'CẢNH BÁO: $warning',
  ].join('\n');
}

String _kindLabel(ProfileKind kind) => switch (kind) {
  ProfileKind.development => 'development (suy ra)',
  ProfileKind.adHoc => 'ad hoc (suy ra)',
  ProfileKind.appStore => 'App Store (suy ra)',
  ProfileKind.enterprise => 'enterprise (suy ra)',
};

/// Kiểm [profile] tại thời điểm [now] (truyền vào để test không phụ thuộc
/// đồng hồ). [claimed] là tên các entitlement app xin; [udid] là thiết bị
/// cần chạy bản ký bằng profile này.
ProfileReport inspectProfile(
  ProvisioningProfile profile, {
  required DateTime now,
  Duration warnWithin = const Duration(days: 30),
  Iterable<String> claimed = const [],
  String? udid,
}) {
  final warnings = <String>[];
  final left = profile.remaining(now);

  switch (profile.statusAt(now, warnWithin: warnWithin)) {
    case ExpiryStatus.expired:
      warnings.add(
        'profile đã hết hạn lúc ${profile.expirationDate.toIso8601String()}, '
        'tạo lại trên Apple Developer rồi ký lại',
      );
    case ExpiryStatus.expiringSoon:
      warnings.add(
        'profile còn ${left.inDays} ngày là hết hạn '
        '(ngưỡng cảnh báo ${warnWithin.inDays} ngày)',
      );
    case ExpiryStatus.valid:
      break;
  }

  final missing = profile.missingEntitlements(claimed).toList()..sort();
  if (missing.isNotEmpty) {
    warnings.add(
      'app xin entitlement mà profile không cho phép: ${missing.join(', ')}; '
      'bật capability trên App ID rồi tạo lại profile',
    );
  }

  if (udid != null && !profile.allowsDevice(udid)) {
    warnings.add('thiết bị $udid không nằm trong profile');
  }

  final devices = profile.provisionedDevices;
  final lines = <String>[
    'Tên:        ${profile.name}',
    'UUID:       ${profile.uuid}',
    'Team:       ${profile.teamIds.join(', ')}',
    'Bundle ID:  ${profile.bundleId ?? '(không có)'}',
    'Loại:       ${_kindLabel(profile.kind)}',
    'Hết hạn:    ${profile.expirationDate.toIso8601String()} '
        '(còn ${left.inDays} ngày)',
    'Thiết bị:   ${profile.provisionsAllDevices
        ? 'tất cả (enterprise)'
        : devices == null
        ? 'không có danh sách'
        : devices.length}',
    'Entitlement cho phép: ${(profile.entitlements.keys.toList()..sort()).join(', ')}',
  ];
  return ProfileReport(lines: lines, warnings: warnings);
}

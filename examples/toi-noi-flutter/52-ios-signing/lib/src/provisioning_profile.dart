import 'plist.dart';

/// Loại profile, suy ra từ nội dung (Apple không ghi loại thành một khoá).
enum ProfileKind { development, adHoc, appStore, enterprise }

/// Tình trạng hạn dùng của profile tại một thời điểm.
enum ExpiryStatus { valid, expiringSoon, expired }

/// Phần của provisioning profile mà lab quan tâm, đọc từ plist đã giải CMS.
///
/// Định dạng profile không được Apple ghi thành tài liệu ổn định (TN3125 nói
/// rõ), nên lớp này chỉ dùng để học và gỡ lỗi, không phải để dựng sản phẩm.
class ProvisioningProfile {
  ProvisioningProfile({
    required this.name,
    required this.uuid,
    required this.teamIds,
    required this.expirationDate,
    required this.entitlements,
    this.provisionedDevices,
    this.provisionsAllDevices = false,
  });

  /// Đọc từ plist đã giải. Ném [FormatException] khi thiếu khoá bắt buộc
  /// (`Name`, `UUID`, `TeamIdentifier`, `ExpirationDate`, `Entitlements`)
  /// hoặc khoá có kiểu sai.
  factory ProvisioningProfile.fromPlist(Map<String, Object?> plist) {
    T require<T>(String key) {
      final value = plist[key];
      if (value is! T) {
        throw FormatException('Profile thiếu "$key" hoặc sai kiểu');
      }
      return value;
    }

    final teamIds = require<List<Object?>>('TeamIdentifier').cast<String>();
    if (teamIds.isEmpty) {
      throw const FormatException('TeamIdentifier rỗng');
    }
    final devices = plist['ProvisionedDevices'];
    return ProvisioningProfile(
      name: require<String>('Name'),
      uuid: require<String>('UUID'),
      teamIds: teamIds,
      expirationDate: require<DateTime>('ExpirationDate'),
      entitlements: require<Map<String, Object?>>('Entitlements'),
      provisionedDevices: devices is List<Object?>
          ? devices.cast<String>()
          : null,
      provisionsAllDevices: plist['ProvisionsAllDevices'] == true,
    );
  }

  /// Đọc từ chuỗi XML của plist đã giải.
  factory ProvisioningProfile.parse(String xml) =>
      ProvisioningProfile.fromPlist(parsePlist(xml));

  final String name;
  final String uuid;
  final List<String> teamIds;

  /// Thời điểm hết hạn, UTC.
  final DateTime expirationDate;

  /// Danh sách quyền profile cho phép (allowlist), không phải quyền app xin.
  final Map<String, Object?> entitlements;

  /// UDID của các thiết bị; `null` khi profile không có khoá này (App Store).
  final List<String>? provisionedDevices;

  /// Profile Enterprise dùng khoá `ProvisionsAllDevices` thay cho danh sách.
  final bool provisionsAllDevices;

  /// Giá trị `application-identifier`: App ID prefix, dấu chấm, bundle ID.
  String? get applicationIdentifier {
    final value = entitlements['application-identifier'];
    return value is String ? value : null;
  }

  /// Bundle ID (hoặc mẫu wildcard) mà profile áp dụng.
  String? get bundleId {
    final id = applicationIdentifier;
    if (id == null) return null;
    final dot = id.indexOf('.');
    return dot < 0 ? null : id.substring(dot + 1);
  }

  /// Có áp dụng cho [candidate] không, tính cả wildcard `com.example.*`.
  bool coversBundleId(String candidate) {
    final pattern = bundleId;
    if (pattern == null) return false;
    if (pattern.endsWith('*')) {
      return candidate.startsWith(pattern.substring(0, pattern.length - 1));
    }
    return pattern == candidate;
  }

  /// Suy ra loại profile: không có danh sách thiết bị là App Store;
  /// `ProvisionsAllDevices` là Enterprise; `get-task-allow` bật là
  /// development; còn lại là ad hoc. Đây là heuristic dựa trên profile phổ
  /// biến, không phải quy tắc Apple công bố.
  ProfileKind get kind {
    if (provisionsAllDevices) return ProfileKind.enterprise;
    if (provisionedDevices == null) return ProfileKind.appStore;
    if (entitlements['get-task-allow'] == true) return ProfileKind.development;
    return ProfileKind.adHoc;
  }

  /// Thời gian còn lại tới lúc hết hạn; âm nếu đã hết.
  Duration remaining(DateTime now) => expirationDate.difference(now.toUtc());

  /// Tình trạng tại [now]: hết hạn khi `now` chạm hoặc vượt hạn, sắp hết hạn
  /// khi còn không quá [warnWithin].
  ExpiryStatus statusAt(
    DateTime now, {
    Duration warnWithin = const Duration(days: 30),
  }) {
    final left = remaining(now);
    if (left <= Duration.zero) return ExpiryStatus.expired;
    if (left <= warnWithin) return ExpiryStatus.expiringSoon;
    return ExpiryStatus.valid;
  }

  /// Thiết bị [udid] có được phép chạy bản ký bằng profile này không.
  /// Enterprise cho mọi thiết bị; App Store không chạy cục bộ nên là `false`.
  bool allowsDevice(String udid) {
    if (provisionsAllDevices) return true;
    return provisionedDevices?.contains(udid) ?? false;
  }

  /// Các quyền app xin ([claimed]) mà profile không có trong allowlist.
  ///
  /// Theo TN3125, mọi entitlement app xin phải nằm trong allowlist của
  /// profile; chiều ngược lại thì không bắt buộc. Chỉ so tên khoá: luật so
  /// giá trị (wildcard, môi trường push) do Apple kiểm lúc ký.
  Set<String> missingEntitlements(Iterable<String> claimed) =>
      claimed.where((key) => !entitlements.containsKey(key)).toSet();
}

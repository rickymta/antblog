/// Nơi một mẩu dữ liệu được phép nằm trên máy, xếp từ ít bị lộ nhất tới
/// nhiều bị lộ nhất.
enum Store {
  /// Chỉ trong RAM của tiến trình; mất khi app bị đóng.
  memoryOnly,

  /// Keychain (iOS) hoặc Keystore (Android) qua `flutter_secure_storage`.
  secureStorage,

  /// Cơ sở dữ liệu của app (drift, bài 22).
  appDatabase,

  /// `shared_preferences`: tệp thường, vào cả bản sao lưu.
  preferences,
}

/// Các loại dữ liệu của app Tới Nơi cần quyết định chỗ lưu.
enum DataKind {
  accessToken,
  refreshToken,
  otpCode,
  tripHistory,
  driverLocationBuffer,
  themeMode,
  apiBaseUrl,
  thirdPartyApiKey,
  cardNumber,
}

/// Quy tắc của một loại dữ liệu: nơi **ít an toàn nhất** còn chấp nhận được,
/// hoặc `null` nếu app không được giữ nó ở bất cứ đâu.
final class StorageRule {
  const StorageRule(this.leastSecureStore, this.reason);

  final Store? leastSecureStore;
  final String reason;
}

/// Chính sách lưu trữ. Dùng làm tài liệu sống: test khoá các quyết định.
abstract final class StoragePolicy {
  static const rules = <DataKind, StorageRule>{
    DataKind.accessToken: StorageRule(
      Store.secureStorage,
      'Cho phép hành động như người dùng; ngắn hạn nhưng vẫn là bí mật.',
    ),
    DataKind.refreshToken: StorageRule(
      Store.secureStorage,
      'Sống lâu hơn access token, nên là thứ đáng lấy cắp nhất trên máy.',
    ),
    DataKind.otpCode: StorageRule(
      Store.memoryOnly,
      'Dùng một lần, vài phút; không có lý do để ghi xuống đĩa.',
    ),
    DataKind.tripHistory: StorageRule(
      Store.appDatabase,
      'Địa chỉ đón trả là dữ liệu cá nhân; nằm trong sandbox của app.',
    ),
    DataKind.driverLocationBuffer: StorageRule(
      Store.appDatabase,
      'Vị trí chính xác; chỉ giữ tới khi gửi lên server rồi xoá.',
    ),
    DataKind.themeMode: StorageRule(Store.preferences, 'Không nhạy cảm.'),
    DataKind.apiBaseUrl: StorageRule(
      Store.preferences,
      'Địa chỉ công khai của môi trường (bài 25).',
    ),
    DataKind.thirdPartyApiKey: StorageRule(
      null,
      'Khoá của bên thứ ba nằm ở server; app đi qua BFF.',
    ),
    DataKind.cardNumber: StorageRule(
      null,
      'Số thẻ không đi qua app: thanh toán qua cổng bên ngoài (bài 39).',
    ),
  };

  /// [store] có được dùng cho [kind] không: được nếu nó an toàn bằng hoặc hơn
  /// mức tối thiểu của quy tắc.
  static bool isAllowed(DataKind kind, Store store) {
    final least = rules[kind]!.leastSecureStore;
    return least != null && store.index <= least.index;
  }
}

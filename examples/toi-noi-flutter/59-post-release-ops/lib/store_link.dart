import 'store_platform.dart';

/// Link mở trang app trên cửa hàng.
///
/// Đích của nút "Cập nhật" đến từ server, nên app chỉ chấp nhận `https` tới
/// hai máy chủ của cửa hàng; mọi thứ khác bị thay bằng link dựng sẵn trong app.
abstract final class StoreLink {
  /// Link dựng sẵn cho app khách bản `prod` (bundle ID theo bài 25/50).
  /// ID App Store là số giả; thay bằng ID thật của app sau khi tạo bản ghi.
  static final Uri androidDefault = Uri.parse(
    'https://play.google.com/store/apps/details?id=vn.toinoi.customer',
  );
  static final Uri iosDefault = Uri.parse(
    'https://apps.apple.com/app/id0000000000',
  );

  static Uri defaultFor(StorePlatform platform) => switch (platform) {
    StorePlatform.android => androidDefault,
    StorePlatform.ios => iosDefault,
  };

  /// Trả [raw] nếu là link `https` hợp lệ của cửa hàng đúng nền tảng, nếu
  /// không trả link dựng sẵn.
  static Uri resolve(String? raw, StorePlatform platform) {
    final uri = raw == null ? null : Uri.tryParse(raw.trim());
    final expectedHost = switch (platform) {
      StorePlatform.android => 'play.google.com',
      StorePlatform.ios => 'apps.apple.com',
    };
    final ok =
        uri != null &&
        uri.scheme == 'https' &&
        uri.host == expectedHost &&
        uri.userInfo.isEmpty;
    return ok ? uri : defaultFor(platform);
  }
}

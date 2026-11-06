import 'package:url_launcher/url_launcher.dart';

import 'url_opener.dart';

/// Lớp bọc mỏng quanh `url_launcher`. Mở trong app cửa hàng/trình duyệt ngoài.
/// Chưa chạy trên thiết bị ở lab này, chỉ kiểm bằng `flutter analyze`.
final class UrlLauncherOpener implements UrlOpener {
  const UrlLauncherOpener();

  @override
  Future<bool> open(Uri uri) =>
      launchUrl(uri, mode: LaunchMode.externalApplication);
}

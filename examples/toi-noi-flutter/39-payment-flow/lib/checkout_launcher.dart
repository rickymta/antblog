import 'package:flutter/services.dart' show PlatformException;
import 'package:url_launcher/url_launcher.dart';

/// Chỉ cho mở trang cổng thanh toán nếu là `https` và đúng host đã biết.
///
/// Địa chỉ trang cổng do server gửi về; kiểm lại ở client để một phản hồi bị
/// sửa hoặc một cấu hình sai không đưa khách tới trang lạ.
final class CheckoutUrlPolicy {
  const CheckoutUrlPolicy({required this.trustedHosts});

  /// Host hư cấu của cổng VíXanh.
  static const vixanh = CheckoutUrlPolicy(trustedHosts: {'pay.vixanh.example'});

  final Set<String> trustedHosts;

  bool allows(Uri url) =>
      url.scheme == 'https' && trustedHosts.contains(url.host);
}

/// Mở trang cổng thanh toán. Lớp bọc mỏng quanh plugin để phần còn lại của app
/// không biết `url_launcher` hay `webview_flutter`.
abstract interface class CheckoutLauncher {
  /// `true` nếu trang đã được mở.
  Future<bool> open(Uri url);

  /// Đóng trang đang mở nếu nền tảng cho phép; không làm gì nếu không.
  Future<void> close();
}

typedef LaunchFn = Future<bool> Function(Uri url, {required LaunchMode mode});
typedef SupportsCloseFn = Future<bool> Function(LaunchMode mode);

Future<bool> _defaultLaunch(Uri url, {required LaunchMode mode}) =>
    launchUrl(url, mode: mode);

/// Mở bằng trình duyệt trong app: Custom Tabs trên Android, và
/// SFSafariViewController trên iOS.
final class UrlLauncherCheckout implements CheckoutLauncher {
  UrlLauncherCheckout({
    this.policy = CheckoutUrlPolicy.vixanh,
    this._launch = _defaultLaunch,
    this._supportsClose = supportsCloseForLaunchMode,
    this._closeView = closeInAppWebView,
  });

  final CheckoutUrlPolicy policy;
  final LaunchFn _launch;
  final SupportsCloseFn _supportsClose;
  final Future<void> Function() _closeView;

  @override
  Future<bool> open(Uri url) async {
    if (!policy.allows(url)) return false;
    try {
      return await _launch(url, mode: LaunchMode.inAppBrowserView);
    } on PlatformException {
      // Không mở được thì plugin ném PlatformException thay vì trả false:
      // Android (ACTIVITY_NOT_FOUND, sau khi đã thử cả Custom Tabs lẫn
      // WebViewActivity) và iOS (SFSafariViewController load lỗi).
      return false;
    }
  }

  /// iOS đóng được SFSafariViewController; Android thì plugin chỉ hỗ trợ đóng
  /// cho `inAppWebView`, không đóng được Custom Tabs.
  @override
  Future<void> close() async {
    try {
      if (await _supportsClose(LaunchMode.inAppBrowserView)) await _closeView();
    } on PlatformException {
      // Đóng là việc làm cho gọn: lỗi ở đây không được làm hỏng luồng.
    }
  }
}

import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';

import 'checkout_launcher.dart';
import 'payment_return.dart';

/// Quyết định cho một lần điều hướng trong WebView.
enum WebViewVerdict {
  /// Trang của cổng: cho đi tiếp.
  allow,

  /// Link quay về app: chặn trong WebView và đưa cho `PaymentFlow`.
  returnToApp,

  /// Mọi thứ khác: chặn.
  block,
}

/// Với WebView, chính app phải tự chặn điều hướng ra ngoài danh sách tin cậy.
/// Hàm thuần để test được mà không cần nền tảng.
WebViewVerdict judgeNavigation(Uri target, CheckoutUrlPolicy policy) {
  if (PaymentReturn.matches(target)) return WebViewVerdict.returnToApp;
  return policy.allows(target) ? WebViewVerdict.allow : WebViewVerdict.block;
}

/// Phương án WebView, chỉ để so sánh với `UrlLauncherCheckout`. Lab này chỉ
/// analyze và biên dịch lớp này; chưa chạy trên thiết bị.
class WebViewCheckoutPage extends StatefulWidget {
  const WebViewCheckoutPage({
    super.key,
    required this.checkoutUrl,
    required this.onReturn,
    this.policy = CheckoutUrlPolicy.vixanh,
  });

  final Uri checkoutUrl;
  final CheckoutUrlPolicy policy;
  final void Function(Uri link) onReturn;

  @override
  State<WebViewCheckoutPage> createState() => _WebViewCheckoutPageState();
}

class _WebViewCheckoutPageState extends State<WebViewCheckoutPage> {
  late final WebViewController _controller;

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setNavigationDelegate(
        NavigationDelegate(
          onNavigationRequest: (request) {
            final verdict = judgeNavigation(
              Uri.parse(request.url),
              widget.policy,
            );
            if (verdict == WebViewVerdict.returnToApp) {
              widget.onReturn(Uri.parse(request.url));
            }
            return verdict == WebViewVerdict.allow
                ? NavigationDecision.navigate
                : NavigationDecision.prevent;
          },
        ),
      );
    if (widget.policy.allows(widget.checkoutUrl)) {
      _controller.loadRequest(widget.checkoutUrl);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('VíXanh')),
    body: WebViewWidget(controller: _controller),
  );
}

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/checkout_launcher.dart';
import 'package:toi_noi_39/webview_checkout.dart';

void main() {
  const policy = CheckoutUrlPolicy.vixanh;
  WebViewVerdict judge(String raw) => judgeNavigation(Uri.parse(raw), policy);

  test('trang của cổng được đi tiếp', () {
    expect(
      judge('https://pay.vixanh.example/checkout/s-1'),
      WebViewVerdict.allow,
    );
  });

  test('link quay về bị chặn trong WebView và chuyển cho app', () {
    expect(
      judge('toinoi://payment/return?session=s-1&status=success'),
      WebViewVerdict.returnToApp,
    );
  });

  test('mọi nơi khác bị chặn: http, host lạ, scheme lạ', () {
    for (final raw in [
      'http://pay.vixanh.example/checkout/s-1',
      'https://evil.example/login',
      'intent://scan/#Intent;scheme=zxing;end',
      'toinoi://trip/T-1002',
    ]) {
      expect(judge(raw), WebViewVerdict.block, reason: raw);
    }
  });
}

import 'package:flutter/services.dart' show PlatformException;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/checkout_launcher.dart';
import 'package:url_launcher/url_launcher.dart' show LaunchMode;

void main() {
  final good = Uri.parse('https://pay.vixanh.example/checkout/s-1');

  group('CheckoutUrlPolicy', () {
    test('chỉ https và đúng host', () {
      const policy = CheckoutUrlPolicy.vixanh;
      expect(policy.allows(good), isTrue);
      for (final raw in [
        'http://pay.vixanh.example/checkout/s-1',
        'https://pay.vixanh.example.evil.example/checkout',
        'https://evil.example/pay.vixanh.example',
        'https://pay.vixanh.example@evil.example/checkout',
        'toinoi://payment/return?session=s-1',
        'javascript:alert(1)',
      ]) {
        expect(policy.allows(Uri.parse(raw)), isFalse, reason: raw);
      }
    });
  });

  group('UrlLauncherCheckout', () {
    test(
      'mở bằng inAppBrowserView (Custom Tabs, SFSafariViewController)',
      () async {
        LaunchMode? usedMode;
        Uri? launched;
        final launcher = UrlLauncherCheckout(
          launch: (url, {required mode}) async {
            launched = url;
            usedMode = mode;
            return true;
          },
        );
        expect(await launcher.open(good), isTrue);
        expect(launched, good);
        expect(usedMode, LaunchMode.inAppBrowserView);
      },
    );

    test('địa chỉ ngoài danh sách tin cậy thì không bao giờ được mở', () async {
      var calls = 0;
      final launcher = UrlLauncherCheckout(
        launch: (url, {required mode}) async {
          calls++;
          return true;
        },
      );
      expect(await launcher.open(Uri.parse('https://evil.example/x')), isFalse);
      expect(calls, 0);
    });

    test('close: gọi hàm đóng khi nền tảng hỗ trợ, bỏ qua khi không', () async {
      var closed = 0;
      LaunchMode? asked;
      UrlLauncherCheckout launcher(bool supported) => UrlLauncherCheckout(
        supportsClose: (mode) async {
          asked = mode;
          return supported;
        },
        closeView: () async => closed++,
      );

      await launcher(true).close(); // kiểu iOS
      expect(closed, 1);
      expect(asked, LaunchMode.inAppBrowserView);

      await launcher(false)
          .close(); // kiểu Android: không đóng được Custom Tabs
      expect(closed, 1);
    });

    test('close nuốt PlatformException', () async {
      final launcher = UrlLauncherCheckout(
        supportsClose: (_) async => true,
        closeView: () async => throw PlatformException(code: 'x'),
      );
      await launcher.close();
    });

    test('PlatformException của Android được đổi thành false', () async {
      final launcher = UrlLauncherCheckout(
        launch: (url, {required mode}) async =>
            throw PlatformException(code: 'ACTIVITY_NOT_FOUND'),
      );
      expect(await launcher.open(good), isFalse);
    });
  });
}

# Lab 39: thanh toán qua VíXanh

Lab của bài **FL-39** trong loạt **Flutter qua Tới Nơi**: [Thanh toán qua VíXanh: WebView, deep link quay về và idempotency key](../../../api/seed/content/flutter-thanh-toan-vi-xanh-webview-deep-link-idempotency.md).

VíXanh là ví điện tử **hư cấu** dùng trong các ví dụ của blog (hồ sơ Tới Nơi chỉ nói "Adapter cho cổng thanh toán", miền Payment & Wallet là chung cho mọi cổng); `pay.vixanh.example` và `toinoi.example` không tồn tại. App không xử lý thẻ: nó xin server của Tới Nơi một phiên thanh toán, mở trang cổng bằng trình duyệt trong app, nhận link `toinoi://payment/return?...` khi khách quay lại, rồi hỏi server kết quả. Thời hạn phiên (15 phút), lịch hỏi (1, 2, 4, 8 giây) và số tiền là số minh hoạ; cước chuyến 5 km là 25.500 VND do `calculateFare` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/payment_models.dart` | `PaymentStatus` (trạng thái của server), `PaymentRequest` (có idempotency key), `PaymentSession`; có `toJson`/`fromJson` |
| `lib/idempotency_key.dart` | `idempotencyKeyGenerator`: `pay-` cộng 128 bit ngẫu nhiên |
| `lib/payment_return.dart` | `PaymentReturn`: đọc `toinoi://payment/return`; `claimed` là lời khẳng định chưa xác thực |
| `lib/payment_gateway.dart` | `PaymentGateway` (interface), `HttpPaymentGateway` (dio), `PaymentGatewayException` phân `unreachable` / `rejected` |
| `lib/payment_store.dart` | `PaymentStore` (interface bền), `InMemoryPaymentStore` lưu chuỗi JSON |
| `lib/checkout_launcher.dart` | `CheckoutUrlPolicy` (https + host tin cậy), `CheckoutLauncher` (`open`, `close`), `UrlLauncherCheckout` (`LaunchMode.inAppBrowserView`; `close` chỉ khi `supportsCloseForLaunchMode`) |
| `lib/payment_flow.dart` | `PaymentFlow` và các `PaymentState`; Dart thuần, không biết Flutter hay plugin |
| `lib/payment_providers.dart`, `lib/payment_screen.dart` | Riverpod 3: `paymentControllerProvider` (`_flow` là `late`, không phải `late final`, vì `build()` chạy lại khi invalidate), `PaymentScreen` (nối `AppLifecycleListener` với `onAppResumed`, nút Kiểm tra lại) |
| `lib/webview_checkout.dart` | Phương án WebView để so sánh: `judgeNavigation` (thuần, có test) và `WebViewCheckoutPage` (chỉ analyze) |
| `native-config/` | Intent filter Android và `CFBundleURLTypes` iOS cho `toinoi://payment/return` |
| `test/support/` | `FakePaymentModule` (một `FakeApiModule`, không sửa `tools/fake_api`), `PaymentHarness`, mạng giả có thể tắt hoặc làm rơi phản hồi |

## Kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/39-payment-flow
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và 84 test xanh. Mức kiểm chứng: **Kiểm chứng trên thiết bị** theo `docs/flutter-master-curriculum.md` §3.3, nhưng máy này không có thiết bị, nên chỉ phần logic được test tự động:

- Tự động (không mạng, không thiết bị, không chờ thời gian thật): `PaymentFlow`, server giả, idempotency, đọc link, đóng tab, chính sách URL, `UrlLauncherCheckout` với hàm giả, `judgeNavigation`, controller (kể cả `invalidate`), màn hình và `AppLifecycleListener` (đẩy chuỗi vòng đời tới `resumed`), giá trị trong `native-config/`.
- Chỉ analyze và biên dịch: `WebViewCheckoutPage` (cần nền tảng thật mới dựng được `WebViewWidget`).
- Chưa chạy trên thiết bị: mở Custom Tabs / SFSafariViewController thật; hệ điều hành đưa `toinoi://payment/return` về app; việc đóng SFSafariViewController có phát `resumed` hay không (chưa rõ nên có nút Kiểm tra lại); `closeInAppWebView` có đóng đúng tab trên iOS (Android không đóng được Custom Tabs bằng plugin); `launchMode="singleTop"` (mặc định của template Flutter 3.47) có tạo thêm `MainActivity` khi Custom Tab nằm trên stack hay không, cần thử `singleTask`; trang cổng thật của một ví.

Đã kiểm với Flutter 3.47.4 / Dart 3.13.3: `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `dio` 5.11.1, `url_launcher` 6.3.3, `webview_flutter` 4.14.1, `toi_noi_fake_api` 0.3.0.

## Chạy trên máy (tự thử)

Lab không có `main.dart` và không kèm `android/`, `ios/`. Muốn thử: `flutter create --platforms=ios,android --project-name toi_noi_39 --org vn.toinoi .`, xoá `test/widget_test.dart` nếu được sinh, dán `native-config/` vào manifest và `Info.plist`, rồi dựng `ProviderScope` ghi đè `paymentGatewayProvider` (`HttpPaymentGateway` trỏ tới server của bạn), `paymentStoreProvider` (kho bền thật) và đưa mọi link vào `paymentControllerProvider.notifier.onDeepLink`. Thử link bằng `adb shell am start -W -a android.intent.action.VIEW -d "toinoi://payment/return?session=<id>" vn.toinoi.customer` hoặc `xcrun simctl openurl booted "toinoi://payment/return?session=<id>"`.

## Chưa làm

- Server thanh toán thật, webhook của VíXanh và chữ ký (hồ sơ kiến trúc §4.9): chỉ có module giả.
- Kho bền thật (drift, secure storage: bài 22); `InMemoryPaymentStore` chỉ chứng minh dữ liệu đi qua JSON.
- Universal Link / App Link cho đường quay về: đây là đường khuyên dùng cho production (custom scheme ai cũng đăng ký được; Apple nói nhiều app cùng scheme thì app nhận link là không xác định). Lab dùng custom scheme để minh hoạ cơ chế; cấu hình xác minh theo bài 23.
- Xác thực của `GET /payments/sessions/{id}` (token đăng nhập, bài 24) và nghe thay đổi kết nối để tự kiểm tra lại.
- Hoàn tiền, thanh toán một phần, nhiều phương thức, Apple Pay / Google Pay.
- Đồng bộ trạng thái thanh toán với trạng thái chuyến (bài 45) và hàng đợi gửi lại (bài 46).

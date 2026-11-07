---
slug: flutter-push-notification-fcm-apns-deep-link
title: "Push notification: FCM, APNs, local notification và deep link từ thông báo"
summary: "Bài 37: đưa tin push của Tới Nơi tới đúng màn hình chuyến: cấu hình Firebase, APNs key, quyền Android 13 và kênh thông báo, ba trạng thái foreground, background, terminated, PushRouter từ chối payload lạ, đăng ký FID và huỷ khi đăng xuất. Phần trên thiết bị chưa chạy."
category: programming
level: advanced
tags: [flutter, dart, mobile, toi-noi, push-notification, fcm, apns, go-router, ios, android]
language: vi
---

Khách đặt xe rồi khoá máy bỏ vào túi. Ba phút sau tài xế nhận chuyến. Kết nối realtime của [bài 36](/articles/flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe) chỉ có ích khi app còn chạy; lúc app ở nền hay bị tắt, chỉ còn thông báo của hệ điều hành để báo cho khách. Khách chạm vào, app phải mở thẳng màn hình chuyến đó, dù app đang mở, ở nền hay đã bị hệ điều hành thu hồi.

Ba trạng thái này đi ba đường khác nhau trong plugin, và mỗi nền tảng có thêm cấu hình riêng: APNs trên iOS, quyền và kênh thông báo trên Android. Bài này dựng cả chuỗi, từ cấu hình tới đăng ký và huỷ đăng ký thiết bị.

Đây là bài 37 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 36](/articles/flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe) và [bài 23 về go_router](/articles/flutter-go-router-deep-link-shellroute-guard) cho `go` và deep link. Lab dùng Flutter 3.47.4, Dart 3.13.3, `firebase_messaging` 17.0.1, `flutter_local_notifications` 23.0.0 và `go_router` 18.1.3.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p><strong>Chưa có tin push nào được gửi tới một máy thật.</strong> Chỉ <code>flutter analyze</code> và <code>flutter test</code> đã chạy; chi tiết ở mục kiểm chứng.</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>App Tới Nơi đang mở, backend gửi một tin FCM có khối <code>notification</code> kèm <code>data</code>. Hệ điều hành có hiện banner không? <code>onMessageOpenedApp</code> có chạy không?</p>
<details><summary>Xem đáp án</summary><p>Mặc định <strong>không</strong> cả hai. Tài liệu Firebase nói tin dạng notification nhận lúc foreground không hiện thông báo trên Android lẫn iOS; app nhận nó ở <code>FirebaseMessaging.onMessage</code> và tự quyết định. <code>onMessageOpenedApp</code> chỉ phát khi người dùng chạm vào một thông báo đã hiện và app đang ở nền. Mục 3 xử lý ca này.</p></details>
</div>

## 1. Đường đi của một tin push và thứ không nên đặt trong nó

Backend Tới Nơi không nói chuyện với điện thoại. Nó gửi tin cho FCM; FCM chuyển cho tầng vận chuyển của nền tảng (APNs với thiết bị Apple, Google Play services với Android); hệ điều hành nhận tin rồi quyết định hiện hay đưa cho app.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Đường đi của một tin push. Backend Tới Nơi gửi tin cho FCM, kèm định danh thiết bị FID, khối notification và khối data. FCM chuyển tiếp qua APNs cho iPhone hoặc qua Google Play services cho Android. Cả hai tới thiết bị. Trên thiết bị, hệ điều hành hiện thông báo khi app ở nền; khi app ở foreground, tin đi thẳng vào app qua onMessage.">
<defs><marker id="fl37-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="90" width="170" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="118" class="t-b">Backend Tới Nơi</text><text x="32" y="142" class="t-small">máy chủ tin cậy</text><text x="32" y="164" class="t-small">biết FID của máy</text><text x="32" y="186" class="t-small">payload tối giản</text>
<rect x="236" y="90" width="130" height="110" rx="10" class="f-pk-soft s-pk"/><text x="252" y="118" class="t-b">FCM</text><text x="252" y="142" class="t-small">nhận tin</text><text x="252" y="164" class="t-small">chuyển tiếp</text>
<rect x="416" y="30" width="210" height="80" rx="10" class="f-surface s-line"/><text x="432" y="58" class="t-b">APNs</text><text x="432" y="82" class="t-small">thiết bị Apple</text>
<rect x="416" y="180" width="210" height="80" rx="10" class="f-surface s-line"/><text x="432" y="208" class="t-b">FCM trên Android</text><text x="432" y="232" class="t-small">Google Play services</text>
<rect x="656" y="30" width="208" height="230" rx="12" class="f-surface s-line"/><text x="672" y="56" class="t-b">Điện thoại</text>
<rect x="672" y="74" width="176" height="74" rx="10" class="f-warn-soft s-warn"/><text x="686" y="100" class="t-b">Hệ điều hành</text><text x="686" y="122" class="t-small">ở nền: hiện tin</text><text x="686" y="138" class="t-small">trong khay</text>
<rect x="672" y="164" width="176" height="82" rx="10" class="f-pk-soft s-pk"/><text x="686" y="190" class="t-b">App Tới Nơi</text><text x="686" y="212" class="t-small">foreground:</text><text x="686" y="228" class="t-small">vào onMessage</text>
<path d="M188 145 L234 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-1-a)"/>
<path d="M368 120 L414 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-1-a)"/>
<path d="M368 170 L414 210" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-1-a)"/>
<path d="M628 70 L654 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-1-a)"/>
<path d="M628 220 L654 200" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-1-a)"/>
<text x="16" y="286" class="t-small">Tối đa 4096 byte mỗi tin. Tin đi qua máy chủ của Google và Apple trước khi tới máy khách.</text>
</svg>
</div>
<figcaption>Hình 1. Mũi tên chỉ chiều đi của tin, từ backend tới điện thoại.</figcaption>
</figure>

Đường đi này quyết định nội dung tin. Tin băng qua máy chủ của hai hãng khác, nằm trong khay thông báo (người đứng cạnh có thể đọc, tuỳ cài đặt), bị giới hạn 4096 byte và có thể bị bỏ khi máy tích quá 100 tin chờ không thu gọn được, theo tài liệu FCM. Vì vậy push chỉ là lời nhắc "có chuyện mới": payload mang loại tin và mã chuyến; tên tài xế, biển số, số điện thoại, giá, vị trí do app hỏi server theo mã chuyến. Tin mẫu của backend:

### fcm-send.json.example · 37-push-notification/native-config/backend/fcm-send.json.example

```json
{
  "message": {
    "fid": "FAKE-FID-REPLACE-ME",
    "notification": {
      "title": "Tài xế đã nhận chuyến",
      "body": "Mở Tới Nơi để xem tài xế và vị trí."
    },
    "data": {
      "type": "trip_update",
      "tripId": "T-1002"
    },
    "android": {
      "notification": { "channel_id": "trip_updates" }
    }
  }
}
```

Trường `fid` là đích gửi (`token` bị tài liệu Firebase đánh dấu deprecated, xem mục 5). `data` là map chuỗi sang chuỗi. Test kiểm: `data` chỉ có hai khoá, `PushRouter` đọc ra đúng chuyến, văn bản hiển thị không có dãy số dài hay "VND", tổng dưới 4096 byte.

## 2. Cấu hình một lần: Firebase, APNs, quyền và kênh

Cấu hình sai không báo lỗi ồn ào: tin chỉ không tới.

| Việc | Android | iOS |
|---|---|---|
| File Firebase | `google-services.json` đặt ở `android/app/` | `GoogleService-Info.plist` thêm vào target Runner bằng Xcode |
| Khoá gửi tin | Không cần thêm | APNs authentication key (`.p8`) tải lên Firebase, kèm Key ID |
| Capability | Quyền `POST_NOTIFICATIONS` | Push Notifications và Background Modes (Background fetch, Remote notifications) |
| Bật FID | `<meta-data>` trong manifest | Khoá trong `Info.plist` |
| Gradle (`flutter_local_notifications` 23) | `compileSdk` 37, desugaring, AGP từ 9.1.1; template 3.47.4 sinh 36 và 9.1.0 | Không |
| `AppDelegate` | Không | Template dùng UIScene nên phải gọi `configureNotificationCenterDelegate()` và đặt delegate của `UNUserNotificationCenter` |

Hai file Firebase là **file thật của project**, không commit. Lab chỉ có bản `.example` với giá trị giả rõ ràng và `.gitignore` chặn tên thật. Mẫu `google-services.json.example` có ba client ứng với ba flavor của app khách, mỗi flavor một app Firebase; cách chọn file theo flavor khi build thuộc bài 50.

**APNs key hay certificate.** Hướng dẫn iOS của Firebase chỉ nói về authentication key: tải file `.p8` lên mục Cloud Messaging, nhập Key ID, cần ít nhất một key (development, production hoặc cả hai). Theo Apple, [provider certificate](https://developer.apple.com/documentation/usernotifications/establishing-a-certificate-based-connection-to-apns) có hạn một năm và phải cập nhật, mỗi certificate gắn với một app; còn [token-based](https://developer.apple.com/documentation/usernotifications/establishing-a-token-based-connection-to-apns) dùng một token cho nhiều app trong cùng team, và mỗi môi trường có tối đa hai key theo team. Với hai app và ba flavor, authentication key gọn hơn.

Phía Android, hai `meta-data` quan trọng nằm ở manifest:

### AndroidManifest.xml.example · 37-push-notification/native-config/android/AndroidManifest.xml.example (trích)

```xml
<!-- Bật định danh đăng ký kiểu FID (register/onRegistered). -->
<meta-data
    android:name="firebase_messaging_installation_id_enabled"
    android:value="true"/>

<!-- Kênh FCM dùng khi tin không nói rõ kênh. Phải trùng
     tripUpdatesChannelId trong lib/notification_channel.dart. -->
<meta-data
    android:name="com.google.firebase.messaging.default_notification_channel_id"
    android:value="trip_updates"/>
```

Phía iOS, `Info.plist` thêm khoá FID và background mode:

### Info.plist.fragment.example · 37-push-notification/native-config/ios/Info.plist.fragment.example

```xml
<dict>
    <key>FirebaseMessagingInstallationIdEnabled</key>
    <true/>
    <key>UIBackgroundModes</key>
    <array>
        <string>fetch</string>
        <string>remote-notification</string>
    </array>
</dict>
```

Test `native_config_test.dart` đọc các file này và kiểm quyền `POST_NOTIFICATIONS`, id kênh **trùng** hằng số Dart, khoá FID, `aps-environment`, `AppDelegate.swift`, đoạn Gradle, và không có dạng khoá thật. Test chỉ biết giá trị trong file, không biết Gradle hay Xcode có chấp nhận.

**Quyền trên Android 13.** Từ Android 13 (API 33), thông báo của app mới cài bị tắt cho tới khi người dùng cho phép. App nhắm API 33 trở lên tự chọn lúc hiện hộp thoại; tài liệu Android khuyên chọn lúc người dùng hiểu lợi ích, nên Tới Nơi hỏi ngay sau khi đặt chuyến. Quyền đã được SDK và plugin tự khai, nhưng vẫn phải xin lúc chạy. Ở `firebase_messaging` 17.0.1, `requestPermission()` trên Android 13 trở lên hiện đúng hộp thoại này (ta đọc mã Android của plugin để chắc). `deniedPermanently` chỉ có trên Android; iOS báo `denied` cả khi hệ điều hành sẽ không hỏi nữa, nên với cả hai trạng thái UI hiện hướng dẫn mở Cài đặt.

**Kênh thông báo.** Từ Android 8.0 mọi thông báo phải thuộc một kênh; đăng không kèm kênh thì không hiện. Tạo xong, app không đổi được độ ưu tiên (quyền đó thuộc người dùng); tạo lại với đúng giá trị gốc thì không làm gì, nên tạo lúc khởi động là an toàn. Thông báo foreground cần kênh mức cao để có banner:

### local_notifier.dart · 37-push-notification/lib/local_notifier.dart (trích)

```dart
// Tạo kênh khi khởi động: tạo lại kênh đã có là không làm gì.
await notifier._plugin
    .resolvePlatformSpecificImplementation<
      AndroidFlutterLocalNotificationsPlugin
    >()
    ?.createNotificationChannel(
      const AndroidNotificationChannel(
        tripUpdatesChannelId,
        tripUpdatesChannelName,
        description: tripUpdatesChannelDescription,
        importance: Importance.high,
      ),
    );
```

Tin FCM đến lúc app ở nền dùng kênh ở `default_notification_channel_id` nếu tin không nói kênh; ta nói rõ bằng `channel_id` và đặt cùng id ở cả hai nơi. Trên iOS, mặc định `flutter_local_notifications` xin quyền ngay lúc `initialize`, nên lab đặt ba cờ `request*Permission` thành `false` để hộp thoại chỉ hiện khi bấm **Bật thông báo**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Một cấu hình hai nền tảng. Project Firebase ở giữa. Phía Android: google-services.json, manifest với POST_NOTIFICATIONS, hai meta-data và kênh trip_updates. Phía iOS: GoogleService-Info.plist, Info.plist với FID và background mode, entitlements aps-environment, và APNs key tải lên Firebase. Mũi tên chỉ chiều dữ liệu cấu hình đi vào app, hoặc khoá đi lên Firebase.">
<defs><marker id="fl37-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl37-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="330" y="110" width="220" height="110" rx="12" class="f-pk-soft s-pk"/><text x="346" y="138" class="t-b">Project Firebase</text><text x="346" y="162" class="t-small">một app cho mỗi flavor</text><text x="346" y="184" class="t-small">và mỗi nền tảng</text>
<rect x="16" y="20" width="270" height="290" rx="12" class="f-surface s-line"/><text x="32" y="46" class="t-b">App Android</text>
<rect x="32" y="62" width="238" height="52" rx="8" class="f-phys-soft s-phys"/><text x="46" y="84" class="t-mono">google-services.json</text><text x="46" y="102" class="t-small">file thật, không commit</text>
<rect x="32" y="128" width="238" height="52" rx="8" class="f-phys-soft s-phys"/><text x="46" y="150" class="t-mono">POST_NOTIFICATIONS</text><text x="46" y="168" class="t-small">khai và xin lúc chạy</text>
<rect x="32" y="194" width="238" height="52" rx="8" class="f-phys-soft s-phys"/><text x="46" y="216" class="t-mono">meta-data FID, kênh</text><text x="46" y="234" class="t-small">kênh trip_updates</text>
<rect x="32" y="256" width="238" height="42" rx="8" class="f-warn-soft s-warn"/><text x="46" y="282" class="t-small">người dùng quyết quyền</text>
<rect x="594" y="20" width="270" height="290" rx="12" class="f-surface s-line"/><text x="610" y="46" class="t-b">App iOS</text>
<rect x="610" y="62" width="238" height="52" rx="8" class="f-phys-soft s-phys"/><text x="624" y="84" class="t-mono">GoogleService-Info.plist</text><text x="624" y="102" class="t-small">thêm vào target Runner</text>
<rect x="610" y="128" width="238" height="52" rx="8" class="f-phys-soft s-phys"/><text x="624" y="150" class="t-mono">Info.plist</text><text x="624" y="168" class="t-small">FID, background mode</text>
<rect x="610" y="194" width="238" height="52" rx="8" class="f-phys-soft s-phys"/><text x="624" y="216" class="t-mono">aps-environment</text><text x="624" y="234" class="t-small">Push Notifications</text>
<rect x="610" y="256" width="238" height="42" rx="8" class="f-warn-soft s-warn"/><text x="624" y="282" class="t-small">key .p8 tải lên Firebase</text>
<path d="M552 165 L592 165" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-2-a)"/>
<path d="M328 165 L288 165" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-2-a)"/>
<path d="M608 277 L440 277 L440 224" class="f-none s-warn" stroke-width="2" marker-end="url(#fl37-2-w)"/>
</svg>
</div>
<figcaption>Hình 2. Mũi tên ngang chỉ chiều cấu hình đi từ Firebase vào từng app. Đường cam đi lên Firebase: khoá APNs nằm trên Firebase, không nằm trong app.</figcaption>
</figure>

## 3. Ba trạng thái, một hàm `open`

Ba trạng thái của app đi ba đường khác nhau trong plugin:

| App đang ở | Điều xảy ra | Nơi bắt tin |
|---|---|---|
| Foreground | Tin vào app, hệ điều hành không hiện gì | `FirebaseMessaging.onMessage` |
| Background | Hệ điều hành hiện thông báo; người dùng chạm | `FirebaseMessaging.onMessageOpenedApp` |
| Terminated | Như trên, nhưng chạm sẽ khởi chạy app | `getInitialMessage()`, trả **một lần** |

Tài liệu Firebase có một lưu ý cho hàng cuối: nếu người dùng vuốt tắt app (iOS) hoặc Force stop (Android), tin nền ngừng tới cho tới khi họ mở lại app. Phần còn lại của app không biết `FirebaseMessaging`; mọi thứ đi qua một interface nhỏ nên cả ba trạng thái test được bằng bản giả:

### push_source.dart · 37-push-notification/lib/push_source.dart

```dart
abstract interface class PushSource {
  /// Tin tới khi app đang mở (foreground). Hệ điều hành không tự hiện gì.
  Stream<PushMessage> get onForeground;

  /// Người dùng chạm vào thông báo khi app đang ở nền (chưa bị tắt hẳn).
  Stream<PushMessage> get onOpenedFromBackground;

  /// Tin đã khởi chạy app từ trạng thái bị tắt hẳn (terminated). Plugin chỉ
  /// trả một lần; các lần sau là `null`.
  Future<PushMessage?> takeInitialMessage();

  /// Định danh đăng ký push của thiết bị (FID) mỗi khi có hoặc đổi.
  Stream<String> get onRegistered;

  Future<PushPermission> permission();

  /// Hiện hộp thoại xin quyền của hệ điều hành nếu còn hỏi được.
  Future<PushPermission> requestPermission();

  /// Đăng ký thiết bị với FCM; kết quả đi về qua [onRegistered].
  Future<void> register();

  /// Huỷ đăng ký: tin gửi tới định danh cũ sẽ không tới máy này nữa.
  Future<void> unregister();
}
```

Phần nối nằm ở `PushNavigator`. Nó nghe ba luồng rồi mới hỏi tin khởi chạy, và cuối cùng mọi đường đều qua cùng `open(location)`:

### push_navigator.dart · 37-push-notification/lib/push_navigator.dart (trích)

```dart
Future<void> start() async {
  if (_started) return;
  _started = true;
  _subscriptions
    ..add(_source.onForeground.listen(_showForeground))
    ..add(
      _source.onOpenedFromBackground.listen(
        (message) => _openFrom(_router.decide(message.data)),
      ),
    )
    ..add(
      _notifier.taps.listen(
        (payload) => _openFrom(_router.decideFromPayload(payload)),
      ),
    );

  final initial = await _source.takeInitialMessage();
  if (initial != null) {
    _openFrom(_router.decide(initial.data));
    return;
  }
  final launch = await _notifier.launchPayload();
  if (launch != null) {
    _openFrom(_router.decideFromPayload(launch));
  }
}
```

Ba chi tiết đáng nhớ. **Thứ nhất**, tin foreground không tự chuyển màn hình: kéo khách khỏi việc đang làm chỉ vì có tin là thô bạo. Ta hiện thông báo cục bộ, khách chạm mới đi:

### push_navigator.dart · 37-push-notification/lib/push_navigator.dart (trích)

```dart
Future<void> _showForeground(PushMessage message) async {
  final decision = _router.decide(message.data);
  if (decision is! OpenTrip) {
    _reject(decision);
    return;
  }
  await _notifier.show(
    id: _notificationId++,
    title: message.title ?? 'Tới Nơi',
    body: message.body ?? 'Chuyến của bạn có cập nhật.',
    payload: decision.toPayload(),
  );
}
```

**Thứ hai**, `getInitialMessage` chỉ trả tin một lần (chú thích trong mã plugin), nên `start()` gọi nó đúng một lần. Chú thích đó còn nói: trên Android, tin nhận lúc app ở foreground rồi bị chạm lúc app ở nền hoặc bị tắt cũng cho `null`. Thông báo cục bộ ta tự hiện rơi đúng ca này, nên `LocalNotifier` có `launchPayload()` để vớt. **Thứ ba**, `/trip/:id` lồng trong `/` ở router của lab, nên Back từ thông báo về Trang chủ thay vì thoát app.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 390" role="img" aria-label="Hình 3. Bốn nguồn tin hội tụ về một hàm open. Foreground (onMessage) được kiểm bằng decide nhưng chỉ hiện một thông báo cục bộ rồi chờ người dùng chạm. Chạm thông báo cục bộ (taps, launchPayload) đi qua decideFromPayload; background (onMessageOpenedApp) và terminated (getInitialMessage) đi qua decide. Cả ba cùng cho ra quyết định. Payload hợp lệ thành OpenTrip rồi tới open(location) của go_router; payload lạ thành IgnorePush và dừng.">
<defs><marker id="fl37-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl37-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="16" y="16" width="260" height="70" rx="10" class="f-warn-soft s-warn"/><text x="32" y="44" class="t-b">Foreground</text><text x="32" y="66" class="t-mono">onMessage</text>
<rect x="16" y="116" width="260" height="70" rx="10" class="f-pk-soft s-pk"/><text x="32" y="144" class="t-b">Chạm thông báo cục bộ</text><text x="32" y="166" class="t-mono">taps, launchPayload</text>
<rect x="16" y="216" width="260" height="70" rx="10" class="f-pk-soft s-pk"/><text x="32" y="244" class="t-b">Background</text><text x="32" y="266" class="t-mono">onMessageOpenedApp</text>
<rect x="16" y="306" width="260" height="70" rx="10" class="f-pk-soft s-pk"/><text x="32" y="334" class="t-b">Terminated</text><text x="32" y="356" class="t-mono">getInitialMessage</text>
<path d="M146 88 L146 114" class="f-none s-warn" stroke-width="2" marker-end="url(#fl37-3-w)"/>
<text x="160" y="106" class="t-small">decide, hiện thông báo, chờ chạm</text>
<rect x="360" y="120" width="230" height="170" rx="10" class="f-surface s-line"/><text x="376" y="148" class="t-b">PushRouter</text><text x="376" y="176" class="t-small">type và tripId hợp lệ:</text><text x="376" y="194" class="t-mono">OpenTrip</text><text x="376" y="222" class="t-small">payload lạ:</text><text x="376" y="240" class="t-mono">IgnorePush, không mở gì</text>
<rect x="670" y="120" width="194" height="100" rx="10" class="f-phys-soft s-phys"/><text x="686" y="148" class="t-b">open(location)</text><text x="686" y="172" class="t-mono">router.go</text><text x="686" y="194" class="t-small">/trip/T-1002</text>
<path d="M278 151 L358 175" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-3-a)"/>
<path d="M278 251 L358 215" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-3-a)"/>
<path d="M278 341 L358 245" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-3-a)"/>
<path d="M592 170 L668 170" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Mũi tên chỉ chiều đi của tin. Foreground là ngoại lệ có chủ đích: tin được kiểm bằng <code>decide</code> nhưng chỉ hiện thông báo cục bộ; màn hình chỉ mở khi người dùng chạm, qua <code>decideFromPayload</code>.</figcaption>
</figure>

Test foreground bằng bản giả của `PushSource`:

### push_navigator_test.dart · 37-push-notification/test/push_navigator_test.dart (trích)

```dart
test('hiện thông báo cục bộ và KHÔNG tự chuyển màn hình', () async {
  await navigator.start();
  source.emitForeground(tripMessage('T-1002'));
  await pumpEventQueue();

  expect(opened, isEmpty);
  expect(notifier.shown, hasLength(1));
  expect(notifier.shown.single.title, 'Tài xế đã nhận chuyến');
  expect(
    const PushRouter().decideFromPayload(notifier.shown.single.payload),
    const OpenTrip('T-1002'),
  );
});
```

Các test background, terminated (kể cả "chỉ trả một lần"), chạm thông báo cục bộ và `launchPayload` cùng dạng; `widget_flow_test.dart` dựng go_router thật và kiểm màn hình hiện ra cùng Back về Trang chủ.

Handler cho tin lúc app ở nền hoặc bị tắt chỉ làm việc ngắn, không mở màn hình. Tài liệu Firebase đặt ba điều kiện: hàm top-level, không ẩn danh, và với Flutter 3.3 trở lên có `@pragma('vm:entry-point')` để không bị loại khi build release. Trên Android hàm chạy trong isolate riêng, không cập nhật được state hay UI. Handler của lab khởi tạo Firebase và ghi log tên khoá; `background_handler_test.dart` đọc file để kiểm ba điều kiện và việc `main()` đăng ký đúng hàm này:

### background_handler.dart · 37-push-notification/lib/background_handler.dart

```dart
@pragma('vm:entry-point')
Future<void> firebaseMessagingBackgroundHandler(RemoteMessage message) async {
  await Firebase.initializeApp();
  debugPrint(PushRouter.safeSummary(message.data));
}
```

## 4. `PushRouter`: payload lạ không đi đâu cả

Payload là dữ liệu từ ngoài vào: một thông báo cũ có thể còn trong khay từ phiên bản trước, và backend có thể có lỗi. `PushRouter` là hàm thuần, chỉ nhận map và trả một quyết định:

### push_router.dart · 37-push-notification/lib/push_router.dart (trích)

```dart
static final tripIdPattern = RegExp(r'^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$');

PushDecision decide(Map<String, Object?>? data) {
  if (data == null || data.isEmpty) {
    return const IgnorePush('payload rỗng');
  }
  final type = data['type'];
  if (type is! String) {
    return const IgnorePush('thiếu type');
  }
  if (type != tripUpdateType) {
    return const IgnorePush('type không được hỗ trợ');
  }
  final tripId = data['tripId'];
  if (tripId is! String || !tripIdPattern.hasMatch(tripId)) {
    return const IgnorePush('tripId thiếu hoặc sai định dạng');
  }
  return OpenTrip(tripId);
}
```

Mã chuyến được kiểm bằng biểu thức hẹp chứ không chỉ "khác rỗng", vì ta ghép nó vào đường dẫn. Ghép thẳng trông vô hại nhưng không an toàn: `Uri` của Dart chuẩn hoá `..` ngay khi parse, `Uri.parse('/trip/../admin').path` là `/admin`. Test cho thấy cách sai thật sự sai:

### path_traversal_test.dart · 37-push-notification/test/path_traversal_test.dart (trích)

```dart
testWidgets('cách sai: nối thẳng tripId "../admin" thì tới route /admin', (
  tester,
) async {
  final router = await pumpRouter(tester);
  const tripId = '../admin'; // lấy nguyên từ payload, không kiểm
  router.go('/trip/$tripId');
  await tester.pumpAndSettle();
  expect(find.text('ADMIN'), findsOneWidget);
});
```

Payload `tripId: "../admin"` nhảy sang `/admin` nếu ta nối chuỗi. Với `PushRouter` thì không: `IgnorePush` mang lý do nhưng **không** mang giá trị, để log không rò dữ liệu. Test đi qua 18 payload lạ: thiếu hoặc sai `type`, `tripId` là số hoặc `null`, có `/`, `?`, `%`, khoảng trắng, dài 65 ký tự, xuống dòng cuối. Khoá thừa bị bỏ qua và không lọt vào payload thông báo cục bộ, vì `toPayload()` chỉ ghi hai trường ta hiểu.

## 5. Đăng ký thiết bị với backend, và huỷ khi đăng xuất

Backend cần biết gửi tin tới đâu. Cách cũ là `getToken()` rồi nghe `onTokenRefresh`. Ở `firebase_messaging` 17.0.1, `getToken`, `onTokenRefresh` và `deleteToken` đã `@Deprecated`, thay bằng `register()`, `onRegistered` và `unregister()`; tài liệu Firebase (kiểm 11/2026) khuyên dùng Firebase Installation ID (FID). Ta theo cách mới: bật FID bằng hai khoá ở mục 2 và không dùng lẫn API FID với API token, như tài liệu dặn. Khi auto-init bật (mặc định), FCM tự đăng ký lúc khởi động và `onRegistered` phát FID; `register()` thủ công chủ yếu cần sau `unregister()`.

`PushRegistrar` giữ backend luôn biết FID hiện tại. Backend lưu một dòng cho mỗi lần cài app, khoá theo `deviceId` (mã ngẫu nhiên do app sinh) chứ không theo FID: FID đổi thì cập nhật cùng dòng. Quy tắc nằm trong `_begin`:

### push_registrar.dart · 37-push-notification/lib/push_registrar.dart (trích)

```dart
Future<void> _begin() async {
  // Nghe trước khi gọi register: tài liệu dặn vậy, vì sự kiện có thể tới
  // ngay trong lúc register còn chạy.
  _subscription ??= _source.onRegistered.listen(_onRegistered);
  if (status.value != PushStatus.synced) status.value = PushStatus.waiting;
  try {
    await _source.register();
    _registerFailed = false;
  } on Exception {
    _registerFailed = true;
    status.value = PushStatus.failed;
  }
}
```

Ta **nghe `onRegistered` trước khi gọi `register()`**, đúng như tài liệu dặn. Tài liệu nói `register()` xin quyền nếu chưa có, nên `resume()` (chạy mỗi lần mở app) chỉ gọi `_begin` khi quyền đã cấp; hộp thoại chỉ hiện từ nút **Bật thông báo**. FID mới vào hàng đợi và gửi theo thứ tự, FID trùng không gửi lại, lỗi thì `failed` và `retry()` gửi lại đúng FID đang chờ.

FID đi trong **body** (`PUT /devices/{deviceId}`), không bao giờ trong URL vì URL hay lọt vào log proxy; `http_device_api_test.dart` kiểm điều đó và kiểm `404` khi xoá được coi là đã xoá.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 4. Vòng đời đăng ký. Hàng trên: bật thông báo, xin quyền, register, onRegistered phát FID, rồi PUT lên backend. Hàng dưới, khi đăng xuất: DELETE ở backend trước, rồi unregister ở FCM, rồi trạng thái tắt. Khi FID đổi, onRegistered phát lại và app gửi PUT lại với cùng deviceId.">
<defs><marker id="fl37-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl37-4-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="16" y="24" class="t-b">Bật thông báo</text>
<rect x="16" y="40" width="150" height="76" rx="10" class="f-pk-soft s-pk"/><text x="30" y="66" class="t-b">enable()</text><text x="30" y="88" class="t-small">từ nút bấm</text>
<rect x="198" y="40" width="150" height="76" rx="10" class="f-surface s-line"/><text x="212" y="66" class="t-b">Xin quyền</text><text x="212" y="88" class="t-small">hộp thoại của OS</text>
<rect x="380" y="40" width="150" height="76" rx="10" class="f-surface s-line"/><text x="394" y="66" class="t-mono">register()</text><text x="394" y="88" class="t-small">đã nghe trước</text>
<rect x="562" y="40" width="150" height="76" rx="10" class="f-surface s-line"/><text x="576" y="66" class="t-mono">onRegistered</text><text x="576" y="88" class="t-small">phát FID</text>
<rect x="744" y="40" width="120" height="76" rx="10" class="f-phys-soft s-phys"/><text x="758" y="66" class="t-mono">PUT</text><text x="758" y="88" class="t-mono">/devices/id</text>
<path d="M168 78 L196 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-4-a)"/>
<path d="M350 78 L378 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-4-a)"/>
<path d="M532 78 L560 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-4-a)"/>
<path d="M714 78 L742 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-4-a)"/>
<text x="16" y="146" class="t-small">FID đổi: onRegistered phát lại, PUT lại với cùng deviceId</text>
<text x="16" y="164" class="t-b">Đăng xuất</text>
<rect x="16" y="180" width="150" height="76" rx="10" class="f-pk-soft s-pk"/><text x="30" y="206" class="t-b">signOut()</text><text x="30" y="228" class="t-small">ngừng nghe FID</text>
<rect x="198" y="180" width="200" height="76" rx="10" class="f-phys-soft s-phys"/><text x="212" y="206" class="t-mono">DELETE</text><text x="212" y="228" class="t-mono">/devices/id (trước)</text>
<rect x="430" y="180" width="200" height="76" rx="10" class="f-warn-soft s-warn"/><text x="444" y="206" class="t-mono">unregister()</text><text x="444" y="228" class="t-small">FCM (sau)</text>
<rect x="662" y="180" width="150" height="76" rx="10" class="f-surface s-line"/><text x="676" y="206" class="t-b">Trạng thái off</text><text x="676" y="228" class="t-small">không nhận push</text>
<path d="M168 218 L196 218" class="f-none s-pk" stroke-width="2" marker-end="url(#fl37-4-a)"/>
<path d="M400 218 L428 218" class="f-none s-warn" stroke-width="2" marker-end="url(#fl37-4-w)"/>
<path d="M632 218 L660 218" class="f-none s-warn" stroke-width="2" marker-end="url(#fl37-4-w)"/>
</svg>
</div>
<figcaption>Hình 4. Hàng trên là đường đăng ký, hàng dưới là đường đăng xuất. Mũi tên chỉ chiều thực hiện: ở hàng dưới, backend được xoá trước khi FCM huỷ đăng ký.</figcaption>
</figure>

Đăng xuất là chỗ dễ sót nhất. Nếu chỉ xoá phiên đăng nhập mà không huỷ đăng ký, backend vẫn gửi push chuyến của người cũ tới chiếc máy giờ thuộc người khác. Thứ tự có chủ đích: xoá ở backend **trước**, lúc phiên còn hợp lệ, rồi mới `unregister()`:

### push_registrar.dart · 37-push-notification/lib/push_registrar.dart (trích)

```dart
/// Đăng xuất. Thứ tự có chủ đích: xoá ở backend TRƯỚC, khi phiên còn hợp
/// lệ; rồi mới huỷ đăng ký ở FCM. Backend lỗi thì vẫn huỷ ở máy: tin gửi
/// tới định danh cũ sẽ lỗi và backend dọn dòng đó khi gặp.
Future<({bool serverRemoved, bool unregistered})> signOut() async {
  await _subscription?.cancel();
  _subscription = null;
  await _queue;

  var serverRemoved = true;
  try {
    await _api.remove(_deviceId);
  } on DeviceApiException {
    serverRemoved = false;
  }
  var unregistered = true;
  try {
    await _source.unregister();
  } on Exception {
    unregistered = false;
  }
  _uploaded = null;
  _pending = null;
  _registerFailed = false;
  status.value = PushStatus.off;
  return (serverRemoved: serverRemoved, unregistered: unregistered);
}
```

Backend lỗi thì ta vẫn huỷ ở máy và trả `serverRemoved: false`. Dòng còn lại ở backend sẽ chết dần: theo tài liệu Firebase, gửi tới đăng ký không còn hợp lệ trả `UNREGISTERED` (HTTP 404) và backend nên xoá dòng đó. `onUnregistered` cũng phát FID cần xoá khỏi backend, nhưng lab không dựa vào nó vì lúc đó phiên đăng nhập có thể đã hết. Hàng đợi gửi lại việc xoá thuộc bài 46. Test kiểm thứ tự bằng nhật ký chung:

### push_registrar_test.dart · 37-push-notification/test/push_registrar_test.dart (trích)

```dart
test('xoá ở backend TRƯỚC, rồi mới unregister ở FCM', () async {
  await registrar.enable();
  await pumpEventQueue();
  timeline.clear();

  final result = await registrar.signOut();

  expect(timeline, ['api:remove:dev-A', 'fcm:unregister']);
  expect(result, (serverRemoved: true, unregistered: true));
  expect(registrar.status.value, PushStatus.off);
});
```

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với push</span>
<ul>
<li><strong>Handler nền là closure, method, hoặc thiếu pragma.</strong> Dấu hiệu: <code>ArgumentError</code> khi đăng ký (tài liệu plugin nói thế với hàm ẩn danh), hoặc handler im lặng chỉ ở bản release. Cách sửa: hàm top-level có <code>@pragma('vm:entry-point')</code>.</li>
<li><strong>Chờ banner ở foreground.</strong> Dấu hiệu: tin tới mà không hiện gì, hoặc tự chuyển màn hình làm khách mất việc đang làm. Cách sửa: nghe <code>onMessage</code>, hiện thông báo cục bộ, mở màn hình khi người dùng chạm.</li>
<li><strong>Gọi <code>getInitialMessage</code> hai lần.</strong> Dấu hiệu: mở từ trạng thái bị tắt nhưng không vào đúng màn hình. Nguyên nhân: tin chỉ trả một lần. Cách sửa: gọi một lần lúc khởi động, vớt thông báo cục bộ bằng <code>launchPayload</code>.</li>
<li><strong>Nối <code>tripId</code> thẳng vào đường dẫn.</strong> Dấu hiệu: <code>../admin</code> đưa tới route khác (test ở mục 4). Cách sửa: kiểm bằng biểu thức hẹp trong <code>PushRouter</code>.</li>
<li><strong>Đăng xuất mà không huỷ đăng ký.</strong> Dấu hiệu: người dùng mới nhận thông báo chuyến của người cũ. Cách sửa: xoá dòng ở backend rồi <code>unregister()</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

**Mức kiểm chứng: trên thiết bị (TB), và máy viết bài không có thiết bị.** Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/37-push-notification
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` 121 test xanh. Các nhóm test chứng minh:

- **`PushRouter`**: payload hợp lệ, 18 payload lạ, khoá thừa không lọt qua, lý do và log không chứa giá trị.
- **Ba trạng thái và go_router**: foreground chỉ hiện thông báo, background, terminated (trả một lần), chạm thông báo cục bộ, và màn hình hiện ra cùng Back về Trang chủ.
- **Đăng ký**: nghe trước `register`, FID trùng và FID mới, backend lỗi rồi `retry`, đăng xuất đúng thứ tự, từ chối quyền; request HTTP có FID trong body.
- **Cấu hình**: giá trị trong `native-config/` (kể cả `AppDelegate.swift` và đoạn Gradle), tin mẫu của backend, handler nền (đọc mã nguồn), và quét không có khoá thật, không có `google-services.json` hay `.p8`.

**Chưa chạy**: nhận push thật từ FCM và APNs; hộp thoại quyền; `FirebasePushSource` và `FlutterLocalNotifier` với plugin thật (chỉ analyze); build `native-config/` bằng Gradle và Xcode (chỉ được đọc giá trị); việc chờ token APNs trên iOS. **Chưa làm**: guard đăng nhập cho `/trip/:id` (bài 23), tải trạng thái chuyến khi mở màn hình, lưu `deviceId` bền vững, hàng đợi gửi lại khi đăng xuất lỗi.

## Bài tập

1. Backend thêm loại tin `driver_arrived` kèm `tripId`. Phải sửa gì, và app cũ chưa cập nhật xử lý tin đó ra sao?
2. Người dùng từ chối thông báo: Android 13 báo `deniedPermanently` (lab gọi là `deniedForever`), iOS báo `denied`. Lab làm gì, và vì sao không chỉ dựa vào nút bấm lại?
3. Khách đăng xuất lúc máy mất mạng. Dòng thiết bị ở backend ra sao, và nên bổ sung gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Thêm nhánh cho <code>type</code> mới trong <code>PushRouter.decide</code>, kèm test. App cũ trả <code>IgnorePush</code>; tin có khối <code>notification</code> vẫn hiện trong khay vì do hệ điều hành hiện, chạm vào thì app mở nhưng không điều hướng. Chấp nhận được, và đó là lý do <code>PushRouter</code> từ chối thay vì đoán.</p>
<p><strong>2.</strong> <code>enable()</code> trả <code>denied</code> hoặc <code>deniedForever</code>, không gọi <code>register</code>, trạng thái giữ <code>off</code>, và Trang chủ hiện dòng hướng dẫn mở Cài đặt cho cả hai. Hệ điều hành có thể không hỏi lại (Android sau từ chối vĩnh viễn, iOS báo <code>denied</code> trong cùng tình huống), nên nút bấm lại có thể vô ích.</p>
<p><strong>3.</strong> <code>signOut()</code> trả <code>serverRemoved: false</code> nhưng vẫn <code>unregister()</code> ở máy. Dòng ở backend còn đó cho tới khi FCM trả <code>UNREGISTERED</code> và backend dọn. Nên đưa việc xoá vào hàng đợi gửi lại (bài 46).</p>
</details>

Bài trước: [Realtime WebSocket: reconnect, backoff và theo dõi tài xế](/articles/flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe). Bài tiếp: [Tác vụ nền: workmanager, vị trí nền cho app tài xế và chính sách store](/articles/flutter-background-work-workmanager-vi-tri-nen-app-tai-xe).

## Tài liệu gốc

- [Firebase: Set up a Firebase Cloud Messaging client app on Flutter](https://firebase.google.com/docs/cloud-messaging/flutter/client)
- [Firebase: Receive messages in Flutter](https://firebase.google.com/docs/cloud-messaging/flutter/receive)
- [Firebase: Manage registrations and tokens](https://firebase.google.com/docs/cloud-messaging/manage-tokens)
- [Firebase: Notification messages and data messages](https://firebase.google.com/docs/cloud-messaging/customize-messages/set-message-type)
- [Firebase: Set up an iOS client app](https://firebase.google.com/docs/cloud-messaging/ios/client)
- [Firebase: Set up an Android client app](https://firebase.google.com/docs/cloud-messaging/android/client)
- [Firebase: FCM HTTP v1 REST, projects.messages](https://firebase.google.com/docs/reference/fcm/rest/v1/projects.messages)
- [Android: Notification runtime permission](https://developer.android.com/develop/ui/views/notifications/notification-permission)
- [Android: Create and manage notification channels](https://developer.android.com/develop/ui/views/notifications/channels)
- [Apple: Establishing a token-based connection to APNs](https://developer.apple.com/documentation/usernotifications/establishing-a-token-based-connection-to-apns)
- [Apple: Establishing a certificate-based connection to APNs](https://developer.apple.com/documentation/usernotifications/establishing-a-certificate-based-connection-to-apns)
- [pub.dev: firebase_messaging](https://pub.dev/packages/firebase_messaging)
- [pub.dev: flutter_local_notifications](https://pub.dev/packages/flutter_local_notifications)
- [pub.dev: go_router](https://pub.dev/packages/go_router)

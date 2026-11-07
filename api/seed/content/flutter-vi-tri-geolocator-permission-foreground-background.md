---
slug: flutter-vi-tri-geolocator-permission-foreground-background
title: "Vị trí: geolocator, quyền foreground/background và cấu hình iOS, Android"
summary: "Bài 35: dựng lớp vị trí cho app khách và app tài xế Tới Nơi bằng geolocator: khai báo Info.plist và AndroidManifest khác nhau theo app, LocationSource bọc plugin, máy trạng thái quyền bảy trạng thái, xin quyền sau khi giải thích, getPositionStream theo vai trò. Phần trên thiết bị chưa chạy."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, geolocator, location, permission, ios, android]
language: vi
---

Một khách mở Tới Nơi lần đầu, chạm vào ô điểm đón, và app lập tức bật hộp thoại xin quyền vị trí. Khách chưa biết vì sao app cần, bấm **Don't Allow**. Từ đó ghim điểm đón không bao giờ nhảy tới chỗ khách đứng, và trên iOS app không hiện lại được hộp thoại. Tình huống là minh hoạ, cơ chế thì thật.

App tài xế còn khó hơn: vị trí phải tiếp tục được gửi khi tài xế chuyển sang app dẫn đường. Bài này dựng lớp vị trí cho cả hai app: khai báo native, lớp bọc `LocationSource`, máy trạng thái quyền, xin quyền sau khi giải thích và `getPositionStream` theo vai trò.

Đây là bài 35 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 34 về bản đồ flutter_map](/articles/flutter-ban-do-flutter-map-marker-polyline-chon-diem-don) trước: ghim điểm đón ở đó là nơi vị trí hiện tại đổ vào. Lab dùng Flutter 3.47.4, `geolocator` 14.1.2 và `permission_handler` 13.0.2.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Máy viết bài không có Android SDK, Xcode đầy đủ, <code>adb</code> hay <code>simctl</code>. Hộp thoại quyền thật, Cài đặt, foreground service và vị trí giả lập <strong>chưa chạy ở đâu cả</strong>; chỉ có <code>flutter test</code> chạy được. Nhận định về plugin lấy từ mã nguồn, về hệ điều hành từ developer.android.com và developer.apple.com (kiểm tại 11/2026).</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tới Nơi chạy trên iPhone, chưa hỏi quyền vị trí lần nào. <code>Geolocator.checkPermission()</code> trả về gì? Rồi người dùng bấm Don't Allow: lần gọi <code>checkPermission()</code> kế tiếp trả về gì?</p>
<details><summary>Xem đáp án</summary><p><code>denied</code>, rồi <code>deniedForever</code>. Mã nguồn <code>geolocator_apple</code> ánh xạ "chưa quyết định" (và "restricted") thành <code>denied</code>, còn "đã từ chối" thành <code>deniedForever</code>. Vậy <code>denied</code> không phân biệt "chưa hỏi" với "từ chối nhưng còn hỏi lại được", nên app phải tự nhớ đã hỏi chưa; và trên iOS <code>requestPermission()</code> trả về ngay khi trạng thái đã quyết định.</p></details>
</div>

## Hai tầng: khai báo native và hộp thoại lúc chạy

Quyền vị trí có hai tầng: khai báo trong file native (thiếu nó, plugin ném `PermissionDefinitionsNotFoundException` khi xin quyền) và hộp thoại lúc chạy. Hai app Tới Nơi khác nhau ở tầng một (Hình 1), có chủ đích.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 390" role="img" aria-label="Hình 1. Hai hộp cạnh nhau. App khách chỉ khai quyền khi dùng app: trên iOS khoá WhenInUse và từ điển mục đích chính xác tạm thời, trên Android ACCESS_COARSE_LOCATION và ACCESS_FINE_LOCATION. App tài xế có tất cả những thứ đó và thêm: trên iOS khoá AlwaysAndWhenInUse và UIBackgroundModes location, trên Android FOREGROUND_SERVICE và FOREGROUND_SERVICE_LOCATION. Mũi tên từ app khách sang app tài xế nghĩa là tài xế bằng khách cộng phần thêm."><defs><marker id="fl35-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="16" width="420" height="300" rx="10" class="f-phys-soft s-phys"/>
<text x="32" y="46" class="t-b">App khách</text>
<text x="32" y="78" class="t-b">iOS · Info.plist</text>
<text x="32" y="102" class="t-mono">NSLocationWhenInUseUsageDescription</text>
<text x="32" y="124" class="t-small">từ điển mục đích cho vị trí chính xác tạm thời</text>
<text x="32" y="146" class="t-small">không khoá Always, không UIBackgroundModes</text>
<text x="32" y="184" class="t-b">Android · AndroidManifest.xml</text>
<text x="32" y="206" class="t-mono">ACCESS_COARSE_LOCATION</text>
<text x="32" y="224" class="t-mono">ACCESS_FINE_LOCATION</text>
<text x="32" y="250" class="t-small">không quyền nền, không foreground service</text>
<rect x="444" y="16" width="420" height="300" rx="10" class="f-pk-soft s-pk"/>
<text x="460" y="46" class="t-b">App tài xế</text>
<text x="460" y="78" class="t-b">iOS · Info.plist</text>
<text x="460" y="102" class="t-mono">NSLocationWhenInUseUsageDescription</text>
<text x="460" y="124" class="t-small t-warn">NSLocationAlwaysAndWhenInUseUsageDescription</text>
<text x="460" y="146" class="t-small t-warn">UIBackgroundModes: location</text>
<text x="460" y="184" class="t-b">Android · AndroidManifest.xml</text>
<text x="460" y="206" class="t-mono">ACCESS_COARSE_LOCATION</text>
<text x="460" y="224" class="t-mono">ACCESS_FINE_LOCATION</text>
<text x="460" y="242" class="t-mono t-warn">FOREGROUND_SERVICE</text>
<text x="460" y="260" class="t-mono t-warn">FOREGROUND_SERVICE_LOCATION</text>
<text x="460" y="284" class="t-small">không ACCESS_BACKGROUND_LOCATION</text>
<path d="M226 318 L226 346 L650 346 L650 320" class="f-none s-pk" stroke-width="2" marker-end="url(#fl35-1-a)"/>
<text x="240" y="372" class="t-small">tài xế = khách + phần thêm (chữ nhấn màu), mỗi mục thêm có lý do riêng</text>
</svg>
</div>
<figcaption>Hình 1. Khai báo native của hai app. Phần thêm của app tài xế là phần phải giải thích với App Review và Google Play.</figcaption>
</figure>

Trên iOS, app tài xế khai thêm `NSLocationAlwaysAndWhenInUseUsageDescription` và `UIBackgroundModes` có `location`. Theo Apple, `requestAlwaysAuthorization()` cần cả khoá WhenInUse lẫn khoá Always (từ iOS 11 là khoá vừa nêu; trang của phương thức còn ghi tên cũ), và đặt `allowsBackgroundLocationUpdates = true` mà thiếu `UIBackgroundModes` là lỗi làm app dừng. Mô tả tiếng Việt nói rõ mục đích và lúc ngừng gửi, vì người dùng đọc nó trong hộp thoại.

### Info.plist · 35-location-permission/native-config/driver/ios/Info.plist (trích)

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>Tới Nơi dùng vị trí của bạn khi đang mở app để biết bạn đang online ở đâu và gợi ý cuốc gần nhất.</string>
<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>Tới Nơi gửi vị trí của bạn cho khách trong lúc có chuyến, kể cả khi bạn chuyển sang app dẫn đường. Hết chuyến, Tới Nơi ngừng gửi.</string>
<key>NSLocationTemporaryUsageDescriptionDictionary</key>
<dict>
	<key>PickupPrecise</key>
	<string>Tới Nơi cần vị trí chính xác để khách thấy đúng chỗ xe đang đứng.</string>
</dict>
<key>UIBackgroundModes</key>
<array>
	<string>location</string>
</array>
```

Trên Android, `ACCESS_COARSE_LOCATION` và `ACCESS_FINE_LOCATION` luôn đi cặp, vì Android 12 cho người dùng chỉ cấp vị trí gần đúng dù app xin FINE. Foreground service là phần riêng của tài xế:

### AndroidManifest.xml · 35-location-permission/native-config/driver/android/AndroidManifest.xml (trích)

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
    <uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_LOCATION" />
</manifest>
```

Tài liệu và mã của plugin mâu thuẫn. README của `geolocator` 14.1.2 nói plugin tự khai `FOREGROUND_SERVICE_LOCATION`, nhưng `CHANGELOG` của `geolocator_android` ghi ở bản 5.1.1+1 rằng khai báo này đã bị rút lại vì khiến Google tự động từ chối app không cần vị trí nền, và manifest bản 5.2.1 ta dùng cũng không có dòng đó. Vậy app tài xế tự khai, app khách không khai.

Ta cố ý không khai `ACCESS_BACKGROUND_LOCATION`. Tài liệu Android coi foreground service đang chạy là truy cập vị trí foreground; chỉ lấy vị trí ngoài "có màn hình mở" và "có foreground service" mới cần quyền nền. Tài xế bấm Online lúc app đang mở rồi chạy foreground service thì không cần nó. Khai thừa còn gây hại: `geolocator_android` thêm quyền này vào mọi yêu cầu xin quyền, kể cả nâng lên vị trí chính xác, mà Android 11 trở lên bỏ qua yêu cầu gộp foreground với nền, nên nút "chính xác" vô tác dụng. Chính sách Google Play cũng giới hạn vị trí nền cho tính năng cốt lõi.

Trên iOS, Flutter 3.47 stable dùng Swift Package Manager mặc định và `flutter create` không sinh Podfile; cả hai plugin đều có `Package.swift`. Với app khách, README của `geolocator` khuyên bỏ phần xin quyền Always, để khỏi cần khoá Always và tránh bị Apple hỏi khi nộp. Bản SPM của cờ đó là biến môi trường lúc build:

### build-ios.example.sh · 35-location-permission/native-config/customer/ios/build-ios.example.sh (trích)

```sh
BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios
```

Build từ Xcode.app thì cần `PERMISSION_HANDLER_INFO_PLIST` trỏ tới `Info.plist`, và đổi biến môi trường nào cũng phải xoá DerivedData. App tài xế không cần macro nào: `Package.swift` của `permission_handler_apple` bật macro theo khoá có trong `Info.plist`. Lab giữ thêm đoạn Podfile cho ai đã tắt SPM.

## LocationSource: một lớp bọc mỏng

Plugin chỉ nên xuất hiện ở một file. Phần còn lại (máy trạng thái, màn hình, tracker) chỉ biết interface này, nên test bằng fake mà không cần platform channel:

### location_source.dart · 35-location-permission/lib/location_source.dart (trích)

```dart
abstract interface class LocationSource {
  /// Dịch vụ vị trí của thiết bị có đang bật không.
  Future<bool> isServiceEnabled();

  /// Quyền hiện tại, không hiện hộp thoại.
  Future<SystemPermission> checkPermission();

  /// Độ chính xác đang được cấp. Chỉ có nghĩa khi đã có quyền.
  Future<LocationPrecision> precision();

  /// Hiện hộp thoại xin quyền "khi dùng app". Trả về quyền sau khi người
  /// dùng trả lời.
  Future<SystemPermission> requestForeground();

  /// Nền tảng này có bước xin nâng lên "luôn cho phép" không. Android thì
  /// không: foreground service đã đủ, và app không khai quyền nền.
  bool get canRequestBackground;

  /// Xin nâng lên "luôn cho phép". Chỉ gọi sau khi đã có quyền "khi dùng app".
  Future<SystemPermission> requestBackground();

  /// Xin nâng từ vị trí gần đúng lên chính xác.
  Future<LocationPrecision> requestPrecise();

  Future<bool> openAppSettings();

  Future<bool> openLocationSettings();

  /// Luồng vị trí theo [profile]. Huỷ subscription là dừng cảm biến.
  Stream<LocationFix> positionStream(TrackingProfile profile);
}
```

`GeolocatorLocationSource` cài interface đó. Nó đổi exception của plugin thành exception của ta, gộp enum quyền về bốn giá trị, và đưa bước xin nền, chỉ có trên iOS, đi đường khác:

### geolocator_location_source.dart · 35-location-permission/lib/geolocator_location_source.dart (trích, requestForeground)

```dart
Future<SystemPermission> requestForeground() async {
  try {
    return _map(await Geolocator.requestPermission());
  } on PermissionDefinitionsNotFoundException catch (e) {
    throw LocationConfigurationException(e.toString());
  } on PermissionRequestInProgressException catch (e) {
    throw LocationRequestInProgressException(e.toString());
  }
}
```

### geolocator_location_source.dart · 35-location-permission/lib/geolocator_location_source.dart (trích, requestBackground)

```dart
Future<SystemPermission> requestBackground() async {
  // geolocator_apple chỉ gọi requestWhenInUseAuthorization khi Info.plist có
  // NSLocationWhenInUseUsageDescription, và trả về ngay khi quyền đã được
  // quyết định, nên không thể nâng lên "Always". Bước này đi qua
  // permission_handler (requestAlwaysAuthorization). Chỉ dùng trên iOS.
  if (!canRequestBackground) {
    return checkPermission();
  }
  await ph.Permission.locationAlways.request();
  return checkPermission();
}
```

Lý do nằm trong `geolocator_apple`: hàm xin quyền của nó chỉ gọi `requestWhenInUseAuthorization` khi `Info.plist` có khoá WhenInUse, và trả về ngay nếu trạng thái đã quyết định. Sau khi tài xế cho phép "khi dùng app", `Geolocator.requestPermission()` không nâng lên Always được. `permission_handler` gọi `requestAlwaysAuthorization` cho `Permission.locationAlways`. Android thì không có bước này: `canRequestBackground` trả `false` và giao diện không mời.

Lớp bọc này test được không cần thiết bị, vì `GeolocatorPlatform.instance` và `PermissionHandlerPlatform.instance` cho gán bản giả.

## Bảy trạng thái bằng một hàm thuần

Với giao diện, quyền vị trí có bảy trạng thái: **chưa hỏi** (`NotAsked`), **đang hỏi** (`Asking`), **cho phép khi dùng app** (`WhileInUse`), **luôn cho phép** (`Always`), **từ chối** (`Denied`), **từ chối vĩnh viễn** (`DeniedForever`) và **dịch vụ vị trí tắt** (`ServiceOff`). Hình 2 vẽ các chuyển tiếp chính.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Sơ đồ trạng thái. NotAsked sang Asking foreground khi người dùng bấm Tiếp tục. Asking foreground sang WhileInUse khi đồng ý, sang Denied khi từ chối, và Denied quay lại Asking foreground khi hỏi lại. Trên iOS lần từ chối đầu đi thẳng tới DeniedForever. Trên Android, lần từ chối thứ hai làm Denied sang DeniedForever. WhileInUse sang Asking background khi tài xế iOS có chuyến, rồi sang Always khi đồng ý. Mọi trạng thái đều có thể chuyển sang ServiceOff khi dịch vụ vị trí của máy tắt."><defs><marker id="fl35-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl35-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="30" y="60" width="150" height="50" rx="10" class="f-pk-soft s-pk"/><text x="48" y="90" class="t-b">NotAsked</text>
<rect x="260" y="60" width="170" height="50" rx="10" class="f-pk-soft s-pk"/><text x="272" y="90" class="t-b">Asking · foreground</text>
<rect x="530" y="60" width="150" height="50" rx="10" class="f-phys-soft s-phys"/><text x="546" y="90" class="t-b">WhileInUse</text>
<rect x="530" y="180" width="150" height="50" rx="10" class="f-pk-soft s-pk"/><text x="540" y="210" class="t-b">Asking · nền</text>
<rect x="740" y="180" width="110" height="50" rx="10" class="f-phys-soft s-phys"/><text x="760" y="210" class="t-b">Always</text>
<rect x="260" y="180" width="150" height="50" rx="10" class="f-warn-soft s-warn"/><text x="298" y="210" class="t-b">Denied</text>
<rect x="30" y="180" width="170" height="50" rx="10" class="f-warn-soft s-warn"/><text x="50" y="210" class="t-b">DeniedForever</text>
<rect x="30" y="266" width="820" height="44" rx="10" class="f-warn-soft s-warn"/><text x="48" y="293" class="t-small">Mọi trạng thái có thể chuyển sang ServiceOff khi dịch vụ vị trí của máy tắt</text>
<path d="M182 85 L258 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl35-2-a)"/><text x="188" y="76" class="t-small">Tiếp tục</text>
<path d="M432 85 L528 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl35-2-a)"/><text x="456" y="76" class="t-small">đồng ý</text>
<path d="M400 112 L400 178" class="f-none s-warn" stroke-width="2" marker-end="url(#fl35-2-w)"/><text x="408" y="150" class="t-small">từ chối</text>
<path d="M330 178 L330 112" class="f-none s-pk" stroke-width="2" marker-end="url(#fl35-2-a)"/><text x="338" y="150" class="t-small">hỏi lại</text>
 <path d="M258 205 L202 205" class="f-none s-warn" stroke-width="2" marker-end="url(#fl35-2-w)"/><text x="205" y="248" class="t-small">Android: từ chối lần hai</text>
<path d="M275 112 L275 140 L115 140 L115 178" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl35-2-w)"/><text x="130" y="132" class="t-small">iOS: từ chối lần đầu</text>
<path d="M605 112 L605 178" class="f-none s-pk" stroke-width="2" marker-end="url(#fl35-2-a)"/><text x="613" y="150" class="t-small">tài xế iOS, có chuyến</text>
<path d="M682 205 L738 205" class="f-none s-pk" stroke-width="2" marker-end="url(#fl35-2-a)"/><text x="690" y="196" class="t-small">đồng ý</text>
</svg>
</div>
<figcaption>Hình 2. Bảy trạng thái truy cập vị trí. Mũi tên đi theo hướng chuyển trạng thái; nét đứt là đường chỉ iOS có.</figcaption>
</figure>

Trạng thái là sealed class nên `switch` phải vét cạn cả bảy. Máy trạng thái là hàm thuần `reduce(state, event)`. Sự kiện `Observed` mang ảnh chụp những gì hệ điều hành báo và bỏ qua trạng thái cũ, vì hệ điều hành là nguồn sự thật. Sự kiện `RequestStarted` cần trạng thái cũ: nó chỉ mở hộp thoại ở trạng thái cho phép.

### location_access.dart · 35-location-permission/lib/location_access.dart (reduce)

```dart
LocationAccessState reduce(
  LocationAccessState state,
  LocationAccessEvent event,
) {
  return switch (event) {
    Observed(:final snapshot) => _fromSnapshot(snapshot),
    RequestStarted(target: AskTarget.foreground) => switch (state) {
      NotAsked() || Denied() => const Asking(AskTarget.foreground),
      Asking() ||
      WhileInUse() ||
      Always() ||
      DeniedForever() ||
      ServiceOff() => state,
    },
    RequestStarted(target: AskTarget.background) => switch (state) {
      WhileInUse() => const Asking(AskTarget.background),
      NotAsked() ||
      Asking() ||
      Always() ||
      Denied() ||
      DeniedForever() ||
      ServiceOff() => state,
    },
  };
}
```

### location_access.dart · 35-location-permission/lib/location_access.dart (_fromSnapshot)

```dart
LocationAccessState _fromSnapshot(LocationSnapshot s) {
  if (!s.serviceEnabled) {
    return const ServiceOff();
  }
  return switch (s.permission) {
    SystemPermission.always => Always(s.precision),
    SystemPermission.whileInUse => WhileInUse(
      s.precision,
      backgroundAsked: s.askedBackground,
    ),
    SystemPermission.deniedForever => const DeniedForever(),
    SystemPermission.denied =>
      !s.askedForeground
          ? const NotAsked()
          : s.blockedByRequest
          ? const DeniedForever()
          : const Denied(),
  };
}
```

Dịch vụ vị trí tắt thắng mọi quyền khác, vì xin quyền lúc đó vô ích. Còn `denied` cần hai cờ do app tự nhớ qua `AskedStore`: `askedForeground` phân biệt `NotAsked` với `Denied` (đáp án câu hỏi đầu bài), và `blockedByRequest` biến `Denied` thành `DeniedForever`, vì `checkPermission` của Android (`PermissionManager.java`) không bao giờ trả `deniedForever`; chỉ kết quả của chính lần xin mới trả. Lab lưu cờ trong bộ nhớ; bản thật dùng `shared_preferences` ([bài 22](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline)).

`LocationAccessController` nối hàm thuần với `LocationSource`: chỉ gọi hệ điều hành khi `reduce` cho phép, giữ kết quả lần xin, rồi đọc lại trạng thái:

### location_access_controller.dart · 35-location-permission/lib/location_access_controller.dart (trích)

```dart
Future<void> requestForeground() async {
  final next = reduce(value, const RequestStarted(AskTarget.foreground));
  if (value is Asking || next is! Asking) {
    return;
  }
  value = next;
  try {
    final result = await _source.requestForeground();
    await _store.markAsked(AskTarget.foreground);
    // Android: chỉ kết quả của lần xin này mới biết là bị chặn vĩnh viễn.
    await _store.setBlocked(result == SystemPermission.deniedForever);
  } on LocationRequestInProgressException {
    // Hộp thoại khác đang hiện: không đánh dấu đã hỏi, đọc lại bên dưới.
  } finally {
    await _observe(force: true);
  }
}
```

Bản đầu của controller có hai lỗi. Nó bỏ kết quả lần xin rồi đọc lại bằng `checkPermission`, nên trên Android `DeniedForever` không bao giờ tới được và nút "Tiếp tục" bấm không có gì; fake đầu che lỗi vì cho `checkPermission` trả `deniedForever` như iOS, nên ta thêm fake mô phỏng Android. Nó cũng chỉ kiểm `next is! Asking`, và test "hai lần bấm chỉ mở một hộp thoại" bị timeout: `reduce` trả chính `Asking` cho `Asking`.

## Xin đúng lúc, sau khi giải thích

Tài liệu Android dặn chờ người dùng chạm vào tính năng cần vị trí rồi mới xin, và xin tăng dần, foreground trước, nền sau. Với app nhắm Android 11 trở lên, xin cả hai cùng lúc thì hệ thống bỏ qua yêu cầu; hộp thoại cũng không có "Allow all the time", người dùng phải bật trong Cài đặt. Tới Nơi vì vậy bỏ hẳn bước nền trên Android. Trên iOS, Apple mô tả: khi người dùng đã cho "When In Use", `requestAlwaysAuthorization()` hiện ngay hộp thoại nâng quyền; nếu họ đã chọn Allow Once thì các lần gọi sau bị bỏ qua.

`PickupLocationPanel` theo đúng thứ tự đó. Dựng màn hình không xin gì; nút "Dùng vị trí hiện tại" chỉ mở lời giải thích; chỉ nút "Tiếp tục" mới gọi hệ điều hành:

### pickup_location_panel.dart · 35-location-permission/lib/pickup_location_panel.dart (trích)

```dart
case LocationAdvice.explainThenRequest:
  final ok = await _explain(
    context,
    title: 'Tới Nơi dùng vị trí để làm gì?',
    body:
        'Để đặt ghim điểm đón ngay chỗ bạn đứng và tìm tài xế gần nhất. '
        'Tới Nơi chỉ lấy vị trí khi bạn đang mở app.',
  );
  if (ok) {
    await controller.requestForeground();
  }
case LocationAdvice.offerBackground:
  final ok = await _explain(
    context,
    title: 'Gửi vị trí khi app ở nền',
    body:
        'Trong chuyến, khách theo dõi xe của bạn trên bản đồ. Cho phép '
        'để vị trí vẫn được gửi khi bạn chuyển sang app dẫn đường. '
        'Hết chuyến, Tới Nơi ngừng gửi.',
  );
  if (ok) {
    await controller.requestBackground();
  }
```

Phần còn lại nằm trong `adviceFor`. `DeniedForever` chỉ có nút mở Cài đặt, vì hệ điều hành không hiện hộp thoại nữa. Lời mời gửi vị trí nền chỉ có với app tài xế trên iOS, khi có chuyến, và một lần nhờ `backgroundAsked`; app khách không bao giờ thấy nó.

**Chính xác hay gần đúng.** Android 12 và iOS 14 trở lên cho người dùng chỉ cấp vị trí gần đúng, và tài liệu Android dặn app vẫn phải chạy được khi đó. Panel báo "Vị trí gần đúng" và gợi ý kéo ghim, vì bản đồ bài 34 cho sửa tay. Nút "Cho phép vị trí chính xác" gọi `requestPrecise`: trên iOS là `requestTemporaryFullAccuracy` với `purposeKey` khớp một khoá trong `NSLocationTemporaryUsageDescriptionDictionary`, trên Android là hỏi lại cặp COARSE và FINE (đúng vì manifest không khai quyền nền). Độ chính xác lấy từ `getLocationAccuracy()`, hàm ném `PlatformException` trên Android khi chưa có quyền nào (changelog 14.1.2); lớp bọc coi đó là gần đúng.

## getPositionStream: độ chính xác, distanceFilter và pin

`LocationSettings` có ba tham số chung: `accuracy`, `distanceFilter` (mét tối thiểu giữa hai bản cập nhật) và `timeLimit`. `AndroidSettings` thêm `intervalDuration` và `foregroundNotificationConfig`; `AppleSettings` thêm `activityType`, `allowBackgroundLocationUpdates` và `showBackgroundLocationIndicator`. Tới Nơi dùng ba profile; các số là minh hoạ, không phải số đã đo:

| Profile | accuracy | distanceFilter | interval | Nền |
|---|---|---|---|---|
| `customerPickup` (khách chọn điểm đón) | high | 10 m | 5 giây | không |
| `driverOnline` (online, chưa có chuyến) | medium | 50 m | 15 giây | không |
| `driverOnTrip` (có chuyến) | high | 25 m | 5 giây | có |

`profileFor(role, tripStatus)` chọn profile; test duyệt cả tám `TripStatus` để khẳng định app khách không bao giờ có profile nền và tài xế chỉ chạy nền từ `accepted` tới `inTransit`. Hàm dựng `LocationSettings`:

### geolocator_location_source.dart · 35-location-permission/lib/geolocator_location_source.dart

```dart
LocationSettings buildLocationSettings(
  TrackingProfile profile, {
  TargetPlatform? platform,
}) {
  final accuracy = switch (profile.accuracy) {
    TrackingAccuracy.balanced => LocationAccuracy.medium,
    TrackingAccuracy.high => LocationAccuracy.high,
  };
  return switch (platform ?? defaultTargetPlatform) {
    TargetPlatform.android => AndroidSettings(
      accuracy: accuracy,
      distanceFilter: profile.distanceFilterMeters,
      intervalDuration: profile.interval,
      // Có thông báo là chạy như foreground service; chỉ profile nền mới có.
      foregroundNotificationConfig: profile.background
          ? const ForegroundNotificationConfig(
              notificationTitle: 'Tới Nơi đang chia sẻ vị trí',
              notificationText:
                  'Khách đang theo dõi chuyến này. Hết chuyến, app sẽ dừng.',
              notificationChannelName: 'Vị trí trong chuyến đi',
              setOngoing: true,
            )
          : null,
    ),
    TargetPlatform.iOS || TargetPlatform.macOS => AppleSettings(
      accuracy: accuracy,
      distanceFilter: profile.distanceFilterMeters,
      activityType: ActivityType.automotiveNavigation,
      pauseLocationUpdatesAutomatically: false,
      // Mặc định của plugin là true; đặt rõ để app khách không phụ thuộc vào
      // việc Info.plist có UIBackgroundModes hay không.
      allowBackgroundLocationUpdates: profile.background,
      showBackgroundLocationIndicator: profile.background,
    ),
    TargetPlatform.fuchsia ||
    TargetPlatform.linux ||
    TargetPlatform.windows => LocationSettings(
      accuracy: accuracy,
      distanceFilter: profile.distanceFilterMeters,
    ),
  };
}
```

Ba chi tiết. Một, `foregroundNotificationConfig` biến luồng Android thành foreground service; comment của plugin cảnh báo nó chỉ tăng độ ưu tiên, không giữ app sống nếu hệ thống giết Activity. Hai, `allowBackgroundLocationUpdates` của `AppleSettings` mặc định là `true`; `geolocator_apple` có kiểm `UIBackgroundModes` trước khi bật, nhưng ta vẫn đặt rõ `false` cho app khách. Ba, theo comment của `LocationAccuracy`, `high` là khoảng 10 m trên iOS và 0 đến 100 m trên Android.

Ta không có số đo pin, chỉ có phép tính minh hoạ. Xe chạy 36 km/h là 10 m/s; trên iOS, `distanceFilter` 25 m cho tối đa một bản mỗi 2,5 giây, khoảng 24 bản mỗi phút. Khi plugin dùng `FusedLocationClient` trên Android 13 trở lên, nó đặt cả khoảng thời gian tối thiểu (mặc định 5 giây) lẫn khoảng cách tối thiểu, nên chu kỳ 5 giây mới là giới hạn: tối đa 12 bản mỗi phút. Nhánh Android cũ hơn đặt khoảng nhanh nhất bằng nửa chu kỳ, và máy không có Google Play Services dùng `LocationManagerClient`, nên con số không áp dụng ở đó. Pin thật phải đo trên máy thật; chiều giảm là độ chính xác thấp hơn, `distanceFilter` lớn hơn và dừng luồng khi không cần.

Dừng đúng lúc là việc của `PositionTracker`. Tài liệu `getPositionStream` nói cảm biến chạy tới khi ta huỷ subscription, nên tracker đóng luồng ngay khi quyền mất, dịch vụ tắt hoặc profile đổi:

### position_tracker.dart · 35-location-permission/lib/position_tracker.dart

```dart
void _sync() {
  final wanted = _access.value.canTrack ? _wantedProfile : null;
  if (wanted == _runningProfile) {
    return;
  }
  unawaited(_subscription?.cancel());
  _subscription = null;
  _runningProfile = wanted;
  if (wanted == null) {
    lastFix.value = null;
    return;
  }
  _subscription = _source
      .positionStream(wanted)
      .listen(
        (fix) {
          lastError.value = null;
          lastFix.value = fix;
        },
        onError: (Object error) {
          if (error is LocationSourceException) {
            lastError.value = error;
          }
        },
      );
}
```

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp với vị trí</span>
<ul>
<li><strong>Xin quyền ngay khi mở app.</strong> Dấu hiệu: nhiều người từ chối vì chưa hiểu vì sao. Cách sửa: chỉ xin sau khi họ chạm vào tính năng và đọc lời giải thích.</li>
<li><strong>Coi <code>denied</code> là "chưa hỏi".</strong> Dấu hiệu: nút bấm mà không có gì hiện ra, hoặc app hỏi đi hỏi lại. Nguyên nhân: plugin gộp hai trường hợp, và <code>deniedForever</code> không còn hộp thoại (Android chỉ báo nó qua kết quả lần xin). Cách sửa: tự nhớ đã hỏi và giữ kết quả; với <code>deniedForever</code>, dẫn người dùng tới Cài đặt.</li>
<li><strong>Xin FINE đứng riêng, hoặc xin foreground cùng nền.</strong> Dấu hiệu: không hộp thoại nào hiện, hoặc không quyền nào được cấp. Nguyên nhân: theo tài liệu Android, FINE đứng riêng bị bỏ qua trên một số bản Android 12, và từ Android 11 yêu cầu gộp foreground với nền bị bỏ qua. Cách sửa: xin cặp COARSE và FINE, nền ở bước sau.</li>
<li><strong>Khai quyền nền thừa.</strong> Dấu hiệu: bị hỏi khi nộp App Store hoặc Google Play, hoặc nút xin chính xác trên Android không làm gì. Nguyên nhân: app khách không cần vị trí nền, và trên Android foreground service đã đủ cho tài xế. Cách sửa: chỉ app tài xế khai <code>UIBackgroundModes</code>, app khách build với <code>BYPASS_PERMISSION_LOCATION_ALWAYS=1</code>, và để test giữ cấu hình khác nhau.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

**Mức kiểm chứng: trên thiết bị (TB), và máy viết bài không có thiết bị.** Chạy được là phần logic. Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/35-location-permission
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` 91 test xanh. Các test chứng minh:

- **Máy trạng thái và controller**: ảnh chụp nào ra trạng thái nào, hộp thoại chỉ mở ở trạng thái cho phép, hai lần bấm chỉ một hộp thoại, Android bị chặn sau lần xin thứ hai, app khách không xin được nền, `profileFor` với đủ tám trạng thái chuyến.
- **Widget và tracker**: không xin quyền khi dựng màn hình, "Để sau" không gọi gì, luồng đóng khi mất quyền hoặc dịch vụ tắt.
- **Lớp bọc plugin** (plugin giả): ánh xạ quyền và lỗi, settings theo nền tảng, bước nền chỉ ở iOS.
- **Cấu hình native**: app khách không có khoá hay quyền nền, cấu hình khớp với `profileFor`.

**Chưa chạy**: mọi thứ cần thiết bị, gồm hộp thoại thật, Cài đặt, nâng lên Always, Precise/Approximate, foreground service, pin và việc build SPM với `Info.plist` thật. Giả lập vị trí cũng chưa chạy: `adb emu geo fix <kinh độ> <vĩ độ>` đặt vị trí cho emulator (kinh độ trước, theo tài liệu emulator console); với iOS Simulator, xem cú pháp bằng `xcrun simctl help location`.

## Bài tập

1. Màn hình khách đã `WhileInUse` nhưng chỉ có vị trí gần đúng. Trên iOS, gọi hàm nào để xin vị trí chính xác cho riêng lần chọn điểm đón này, và `Info.plist` phải có gì?
2. Đổi `driverOnTrip.distanceFilterMeters` từ 25 sang 50. Ở 36 km/h, thời gian tối thiểu giữa hai bản cập nhật thay đổi thế nào, và trên Android chu kỳ 5 giây còn là giới hạn không (giả sử Android 13 trở lên)?
3. Khách tắt dịch vụ vị trí của điện thoại khi đang xem bản đồ. `PositionTracker` làm gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> <code>Geolocator.requestTemporaryFullAccuracy(purposeKey: ...)</code>, và <code>Info.plist</code> phải có <code>NSLocationTemporaryUsageDescriptionDictionary</code> chứa đúng khoá đó (lab dùng <code>PickupPrecise</code>, có test kiểm).</p>
<p><strong>2.</strong> Từ 2,5 giây lên 5 giây ở iOS (25 m và 50 m chia cho 10 m/s). Với <code>FusedLocationClient</code> trên Android 13 trở lên, 50 m cũng là 5 giây nên hai giới hạn trùng nhau; với 25 m, chu kỳ 5 giây đã là giới hạn. Vậy đổi sang 50 m không làm thưa thêm.</p>
<p><strong>3.</strong> Khi <code>refresh</code> chạy, <code>reduce</code> ra <code>ServiceOff</code>, <code>canTrack</code> thành <code>false</code>, tracker huỷ subscription và xoá <code>lastFix</code>. Nếu plugin ném <code>LocationServiceDisabledException</code> trước đó, lớp bọc đổi thành <code>LocationServiceOffException</code> và tracker ghi vào <code>lastError</code>. Panel hiện nút "Bật dịch vụ vị trí".</p>
</details>

Bài trước: [Bản đồ với flutter_map](/articles/flutter-ban-do-flutter-map-marker-polyline-chon-diem-don). Bài tiếp: [Realtime WebSocket: reconnect, backoff và theo dõi tài xế](/articles/flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe).

## Tài liệu gốc

- [package:geolocator](https://pub.dev/packages/geolocator), [geolocator_android (changelog 5.1.1+1)](https://pub.dev/packages/geolocator_android/changelog) và [package:permission_handler](https://pub.dev/packages/permission_handler) (pub.dev).
- [Location permissions](https://developer.android.com/develop/sensors-and-location/location/permissions), [Request location permissions](https://developer.android.com/develop/sensors-and-location/location/permissions/runtime), [Foreground service types](https://developer.android.com/develop/background-work/services/fgs/service-types) và [Emulator console](https://developer.android.com/studio/run/emulator-console) (developer.android.com).
- [requestAlwaysAuthorization()](https://developer.apple.com/documentation/corelocation/cllocationmanager/requestalwaysauthorization()), [allowsBackgroundLocationUpdates](https://developer.apple.com/documentation/corelocation/cllocationmanager/allowsbackgroundlocationupdates), [accuracyAuthorization](https://developer.apple.com/documentation/corelocation/cllocationmanager/accuracyauthorization), [requestTemporaryFullAccuracyAuthorization(withPurposeKey:)](https://developer.apple.com/documentation/corelocation/cllocationmanager/requesttemporaryfullaccuracyauthorization(withpurposekey:)) và [NSLocationAlwaysAndWhenInUseUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationalwaysandwheninuseusagedescription) (developer.apple.com).
- [Chính sách vị trí của Google Play](https://support.google.com/googleplay/android-developer/answer/9799150) (support.google.com/googleplay).

---
slug: flutter-integration-test-patrol-luong-dat-chuyen-tren-thiet-bi
title: "Integration test và patrol: luồng đặt chuyến trên simulator/emulator"
summary: "Bài 31: viết luồng đặt chuyến một lần thành runBookingFlow, chạy bằng flutter test, integration_test và Patrol. Fake API qua 10.0.2.2, chính sách HTTP của nền tảng không áp cho dart:io, bấm hộp thoại quyền thật, chụp màn hình, chi phí của test chậm. Phần trên thiết bị chưa chạy."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, testing, integration-test, patrol, android, ios]
language: vi
---

Đội QA báo một lỗi lạ trên emulator Android 13: ở màn hình theo dõi chuyến, bấm **Bật thông báo** mà dòng trạng thái vẫn ghi "chưa bật". Widget test của màn hình này xanh từ lâu, vì nó thay quyền thông báo bằng bản giả trả lời ngay nên hộp thoại của hệ điều hành chưa bao giờ xuất hiện. Tình huống là minh hoạ, nhưng khoảng trống thì thật: lỗi ở chỗ app tương tác với hệ điều hành chỉ lộ ra trên thiết bị.

Bài này viết luồng đặt chuyến của app khách **một lần**, rồi chạy nó bằng ba cách: `flutter test`, `integration_test` và Patrol. Ta cũng cho emulator gọi fake API trên máy dev, chụp màn hình, và tính xem loại test này đắt tới đâu.

Đây là bài 31 của lộ trình **Flutter qua Tới Nơi**. Nên đọc [bài 30 về kiểm thử Riverpod, async và Stream](/articles/flutter-test-riverpod-async-stream-fake-async-providercontainer) trước, và [bài 28 về widget test](/articles/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia) cho `pump` và finder. Lab dùng client của [bài 19](/articles/flutter-http-dio-json-dto-goi-api-bao-gia), Flutter 3.47.4, Dart 3.13.3 và `patrol` 4.11.0.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Máy viết bài không có Android SDK và Xcode đầy đủ. Vì vậy <code>integration_test/</code> và <code>patrol_test/</code> mới chỉ được <code>flutter analyze</code> kiểm kiểu và tên API, <strong>chưa chạy trên thiết bị nào</strong>. Phần chạy được là <code>flutter test</code>, nên logic của luồng được kiểm tự động.</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một test <code>integration_test</code> chạy trên emulator Android 13, bấm nút xin quyền thông báo và hộp thoại "Allow" của hệ điều hành hiện ra. Test có tự bấm được "Allow" không? Nếu không thì chuyện gì xảy ra với test?</p>
<details><summary>Xem đáp án</summary><p>Không. <code>integration_test</code> điều khiển cây widget của Flutter; hộp thoại do hệ điều hành vẽ, nằm ngoài cây đó. Không ai trả lời hộp thoại thì lời gọi xin quyền không hoàn tất và test hết thời gian chờ ở bước chờ trạng thái "đã bật" (ta suy ra từ cách công cụ hoạt động, chưa chạy trên thiết bị). Tài liệu Patrol nêu đúng khoảng trống này. Mục 5 dùng Patrol để bấm hộp thoại.</p></details>
</div>

## 1. Ba tầng test và chỗ integration test lấp vào

Tài liệu Flutter so sánh ba loại test bằng một bảng. Các xếp hạng dưới đây lấy từ trang đó:

| | Unit | Widget | Integration |
|---|---|---|---|
| Độ tin cậy | Thấp | Cao hơn | Cao nhất |
| Chi phí bảo trì | Thấp | Cao hơn | Cao nhất |
| Phụ thuộc | Ít | Nhiều hơn | Nhiều nhất |
| Tốc độ chạy | Nhanh | Nhanh | Chậm |

Với Tới Nơi: unit test bảo vệ `calculateFare` và máy trạng thái chuyến; widget test bảo vệ từng màn hình với server giả; integration test trả lời câu còn lại là "app thật, trên hệ điều hành thật, có đi hết luồng chính không". Integration test đắt nhất nên chỉ giữ ở tầng này những gì tầng dưới không thấy được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Ba thanh ngang ngắn dần. Unit test là thanh dài nhất, nhiều nhất, nhanh và bảo trì thấp. Widget test ngắn hơn, nhanh và bảo trì cao hơn. Integration và Patrol là thanh ngắn nhất, đủ cho các luồng quan trọng, chậm và bảo trì cao nhất. Độ dài thanh chỉ thể hiện thứ tự, không phải số lượng.">
<rect x="16" y="30" width="560" height="64" rx="10" class="f-pk-soft s-pk"/><text x="32" y="58" class="t-b">Unit test: nhiều nhất</text><text x="32" y="80" class="t-small">cước, máy trạng thái chuyến</text>
<rect x="16" y="112" width="360" height="64" rx="10" class="f-phys-soft s-phys"/><text x="32" y="140" class="t-b">Widget test: nhiều</text><text x="32" y="162" class="t-small">từng màn hình, server giả</text>
<rect x="16" y="194" width="110" height="64" rx="10" class="f-warn-soft s-warn"/><text x="142" y="222" class="t-b">Integration và Patrol</text><text x="142" y="244" class="t-small">đủ cho các luồng quan trọng</text>
<text x="596" y="58" class="t-small">nhanh, bảo trì thấp</text>
<text x="596" y="140" class="t-small">nhanh, bảo trì cao hơn</text>
<text x="596" y="222" class="t-small">chậm, bảo trì cao nhất</text>
</svg>
</div>
<figcaption>Hình 1. Tài liệu Flutter khuyên có nhiều unit và widget test, cộng đủ integration test cho các use case quan trọng. Độ dài thanh chỉ thể hiện thứ tự, không phải số lượng.</figcaption>
</figure>

## 2. Một kịch bản, ba runner

Viết luồng đặt chuyến ba lần thì ba bản sẽ lệch nhau. Ta viết nó **một lần** thành `runBookingFlow(WidgetTester)` và để mỗi runner chỉ lo phần của mình:

| Runner | Chạy ở đâu | Server | Quyền thông báo | Thấy hộp thoại của OS |
|---|---|---|---|---|
| `flutter test` | Máy dev, đồng hồ giả | `InProcessAdapter` trong tiến trình | Giả | Không có |
| `integration_test` | Simulator, emulator, máy thật | Fake API thật qua HTTP | Giả | Không bấm được |
| `patrol test` | Như trên | Fake API thật qua HTTP | Thật | Bấm được |

Hàm nhận `WidgetTester` thường, không biết runner nào đang gọi. Điều khác nhau đi vào bằng tham số: cách để tài xế nhận chuyến, cách trả lời hộp thoại, cách chụp màn hình. Phần lõi là vòng chờ:

### booking_flow.dart · 31-integration-test-patrol/test_support/booking_flow.dart

```dart
Future<void> pumpUntilFound(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 10),
  Duration step = const Duration(milliseconds: 100),
}) async {
  var waited = Duration.zero;
  while (finder.evaluate().isEmpty) {
    if (waited >= timeout) {
      throw TestFailure(
        'Không thấy widget cần tìm (${finder.describeMatch(Plurality.zero)}) '
        'sau ${timeout.inSeconds} giây.',
      );
    }
    await tester.pump(step);
    waited += step;
  }
}
```

Ta không dùng `pumpAndSettle` vì màn hình chờ tài xế có một `LinearProgressIndicator` chạy mãi, nên "yên" không bao giờ tới. Test dưới đây chứng minh điều đó:

### booking_flow_test.dart · 31-integration-test-patrol/test/booking_flow_test.dart (trích)

```dart
      expect(find.byType(LinearProgressIndicator), findsOneWidget);
      Object? error;
      try {
        await tester.pumpAndSettle(
          const Duration(milliseconds: 100),
          EnginePhase.sendSemanticsUpdate,
          const Duration(seconds: 2),
        );
      } catch (e) {
        error = e;
      }
      expect(
        error,
        isA<FlutterError>().having(
          (e) => e.message,
          'message',
          'pumpAndSettle timed out',
        ),
      );
```

`timeout` của `pumpUntilFound` là tổng các `step` đã pump, không phải đồng hồ treo tường. Trong `flutter test`, mỗi `pump(step)` đẩy đồng hồ giả đi đúng `step`, nên vòng chờ 10 giây không tốn giây nào. Trên thiết bị, binding là `LiveTestWidgetsFlutterBinding`: `pump(duration)` tạo một `Timer` thật, nên mỗi bước tốn ít nhất `step` thật và tổng thời gian thật có thể dài hơn.

Hàm chính ghép các bước lại. Bước "tài xế nhận" đi qua `actAsDriver`, vì trong `testWidgets` một `await` cần timer thật sẽ không bao giờ xong nếu thiếu `tester.runAsync`. Trên binding live, `runAsync` chỉ chạy và chờ việc đó (đọc `binding.dart`).

```dart
Future<BookingOutcome> runBookingFlow(
  WidgetTester tester, {
  required DriverAccepts driverAccepts,
  AnswerPermissionDialog? answerPermissionDialog,
  Screenshot? screenshot,
  ServiceKind service = ServiceKind.ride,
  double distanceKm = 5,
  Duration timeout = const Duration(seconds: 10),
}) async {
  final fare = await requestQuote(
    tester,
    service: service,
    distanceKm: distanceKm,
    timeout: timeout,
  );
  await screenshot?.call('1-bao-gia');

  final tripId = await placeBooking(tester, timeout: timeout);
  await askForNotifications(
    tester,
    answerDialog: answerPermissionDialog,
    timeout: timeout,
  );
  await screenshot?.call('2-dang-tim-tai-xe');

  await actAsDriver(tester, () => driverAccepts(tripId));
  final driverId = await waitForDriver(tester, timeout: timeout);
  await screenshot?.call('3-tai-xe-da-nhan');
  return (tripId: tripId, fare: fare, driverId: driverId);
}
```

Giá hiển thị được so với `calculateFare(distanceKm)` của `toi_noi_core` chứ không gõ tay. Mọi widget cần tìm có tên trong `AppKeys`: app dùng tên đó để gắn `key:`, test dùng để tìm. Với Patrol, `$(#distanceField)` tương đương `find.byKey(Key('distanceField'))` (đọc `createFinder` trong `patrol_finders`), nên tên key phải là một từ, không có gạch nối hay dấu cách.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 2. Ba runner cùng gọi một hàm runBookingFlow. flutter test chạy trên máy dev với adapter giả và đã chạy ở máy viết bài. integration_test chạy trên thiết bị với server thật và quyền giả, chưa chạy ở máy viết bài. patrol test chạy trên thiết bị với server thật và quyền thật, chưa chạy ở máy viết bài. Mũi tên đi từ mỗi runner tới hàm dùng chung.">
<defs><marker id="fl31-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="20" width="320" height="92" rx="10" class="f-pk-soft s-pk"/><text x="30" y="48" class="t-b">flutter test</text><text x="30" y="70" class="t-mono">test/booking_flow_test.dart</text><text x="30" y="92" class="t-small">adapter giả, đồng hồ giả (đã chạy)</text>
<rect x="16" y="134" width="320" height="92" rx="10" class="f-warn-soft s-warn"/><text x="30" y="162" class="t-b">integration_test</text><text x="30" y="184" class="t-mono">integration_test/..._test.dart</text><text x="30" y="206" class="t-small">server thật, quyền giả (chưa chạy)</text>
<rect x="16" y="248" width="320" height="92" rx="10" class="f-warn-soft s-warn"/><text x="30" y="276" class="t-b">patrol test</text><text x="30" y="298" class="t-mono">patrol_test/..._test.dart</text><text x="30" y="320" class="t-small">server thật, quyền thật (chưa chạy)</text>
<rect x="520" y="110" width="344" height="140" rx="10" class="f-phys-soft s-phys"/><text x="536" y="140" class="t-b">runBookingFlow(tester, ...)</text><text x="536" y="168" class="t-small">1. chọn dịch vụ, nhập km, báo giá</text><text x="536" y="190" class="t-small">2. đặt chuyến, xin quyền thông báo</text><text x="536" y="212" class="t-small">3. tài xế nhận, chờ màn hình đổi</text><text x="536" y="234" class="t-mono">test_support/booking_flow.dart</text>
<path d="M338 66 L518 150" class="f-none s-pk" stroke-width="2" marker-end="url(#fl31-2-a)"/>
<path d="M338 180 L518 180" class="f-none s-pk" stroke-width="2" marker-end="url(#fl31-2-a)"/>
<path d="M338 294 L518 210" class="f-none s-pk" stroke-width="2" marker-end="url(#fl31-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Mũi tên chỉ chiều gọi: mỗi runner gọi hàm dùng chung, hàm không biết runner nào đang gọi. Điều khác nhau giữa các runner đi vào bằng tham số.</figcaption>
</figure>

## 3. Chạy với fake API trên máy dev

Trên thiết bị, app gọi fake API đang chạy trên máy dev (`dart run bin/server.dart` trong `tools/fake_api`, mặc định chỉ nghe ở loopback). Địa chỉ phụ thuộc nơi app chạy:

### api_base_url.dart · 31-integration-test-patrol/lib/api_base_url.dart

```dart
Uri resolveApiBaseUrl({
  String defined = _definedBaseUrl,
  TargetPlatform? platform,
}) {
  if (defined.isNotEmpty) {
    return Uri.parse(defined);
  }
  final host = (platform ?? defaultTargetPlatform) == TargetPlatform.android
      ? androidEmulatorHost
      : 'localhost';
  return Uri(scheme: 'http', host: host, port: fakeApiPort);
}
```

`localhost` bên trong Android emulator là chính emulator. Tài liệu Android mô tả `10.0.2.2` là bí danh của loopback trên máy dev. iOS Simulator chạy ngay trên máy Mac nên `localhost` là máy dev. Máy thật không có lối tắt nào: ta truyền `--dart-define=API_BASE_URL=http://<ip-máy-dev>:8787` và chạy server với `--host 0.0.0.0`, chỉ trên mạng tin cậy.

Có cần mở HTTP không mã hoá trong cấu hình native không? Với lab này thì không. `dio` mặc định gửi qua `dart:io`, và socket do Dart sở hữu không chịu chính sách mạng của nền tảng. Engine ghi: "By default, dart:io allows all socket connections" (`switch_defs.h`). `FlutterLoader.java` không bật cờ `--disallow-insecure-connections`, `FlutterDartProject.mm` đặt `may_insecurely_connect_to_all_domains = true`, và trang network policy của Flutter nói chính sách chỉ áp cho socket do Android và iOS sở hữu. Chính sách đó (Android từ API 28, ATS trên iOS) chỉ chặn tầng nền tảng: `HttpURLConnection`, `URLSession`, WebView và adapter dựa trên chúng như `cronet_http`. Kết luận này đọc từ source và tài liệu, chưa chạy trên thiết bị.

Lab vẫn giữ cấu hình dưới đây như lớp phòng thủ khi đổi adapter hoặc mở WebView, đặt **chỉ trong thư mục `debug`** để bản release không có nó:

### network_security_config.xml · 31-integration-test-patrol/native-config/android/app/src/debug/res/xml/network_security_config.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<network-security-config>
    <domain-config cleartextTrafficPermitted="true">
        <domain includeSubdomains="false">10.0.2.2</domain>
    </domain-config>
</network-security-config>
```

Manifest của bản debug trỏ tới file bằng `android:networkSecurityConfig="@xml/network_security_config"`; manifest chính không nhắc tới nó. `native_config_test.dart` đọc các file này để lớp phòng thủ không vô tình rộng ra: manifest chính có `usesCleartextTraffic`, hoặc domain rộng hơn `10.0.2.2`, đều làm test đỏ (ta đã thử).

Trên iOS, ATS cũng chỉ áp cho tầng nền tảng. Với adapter native, tài liệu Apple cho biết từ iOS 17 ATS không còn cho kết nối tới địa chỉ IP theo mặc định; `NSAllowsLocalNetworking` mở lại mạng cục bộ mà không tắt ATS toàn app. Lab giữ đoạn plist đó trong `native-config/ios/` như lớp phòng thủ.

Cuối cùng là "tài xế". Test không có app tài xế, nên `DriverSimulator` là một `ToiNoiApiClient` thứ hai gửi header `x-fake-driver-id`, gọi `getTrip` rồi `acceptTrip` với đúng phiên bản vừa đọc. Một test chạy nó qua HTTP thật tới fake API trên loopback (`test()` thuần, vì binding của `flutter_test` chặn HTTP thật), để chứng minh đường đi trên thiết bị không phụ thuộc `InProcessAdapter`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 3. Bố cục khi chạy Patrol. Trên máy dev có patrol_cli và fake API ở 127.0.0.1 cổng 8787. Trên emulator hoặc simulator có app Tới Nơi chạy test Dart, cấu hình mạng debug chỉ áp cho tầng nền tảng, hộp thoại quyền của hệ điều hành và native automator. patrol_cli build, cài và chạy app trên thiết bị. App gọi HTTP tới fake API. Test Dart gọi native automator, native automator bấm hộp thoại quyền.">
<defs><marker id="fl31-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl31-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="16" y="20" width="270" height="330" rx="12" class="f-surface s-line"/><text x="32" y="48" class="t-b">Máy dev</text>
<rect x="36" y="70" width="230" height="84" rx="10" class="f-pk-soft s-pk"/><text x="50" y="98" class="t-b">patrol test</text><text x="50" y="122" class="t-small">build, cài, chạy test</text>
<rect x="36" y="240" width="230" height="84" rx="10" class="f-phys-soft s-phys"/><text x="50" y="268" class="t-b">fake API</text><text x="50" y="292" class="t-mono">127.0.0.1:8787</text>
<rect x="340" y="20" width="524" height="330" rx="12" class="f-surface s-line"/><text x="356" y="48" class="t-b">Emulator hoặc simulator</text>
<rect x="360" y="70" width="230" height="110" rx="10" class="f-pk-soft s-pk"/><text x="374" y="98" class="t-b">App Tới Nơi</text><text x="374" y="122" class="t-small">test Dart chạy bên trong</text><text x="374" y="144" class="t-small">gọi HTTP tới fake API</text>
<rect x="360" y="240" width="230" height="84" rx="10" class="f-surface s-line"/><text x="374" y="268" class="t-b">Cấu hình mạng debug</text><text x="374" y="292" class="t-small">lớp phòng thủ, tầng nền tảng</text>
<rect x="640" y="70" width="204" height="84" rx="10" class="f-warn-soft s-warn"/><text x="654" y="98" class="t-b">Hộp thoại quyền</text><text x="654" y="122" class="t-small">do hệ điều hành vẽ</text>
<rect x="640" y="240" width="204" height="84" rx="10" class="f-phys-soft s-phys"/><text x="654" y="268" class="t-b">Native automator</text><text x="654" y="292" class="t-small">UiAutomator, XCUITest</text>
<path d="M268 112 L358 112" class="f-none s-pk" stroke-width="2" marker-end="url(#fl31-3-a)"/>
<path d="M362 168 L268 262" class="f-none s-pk" stroke-width="2" marker-end="url(#fl31-3-a)"/>
<path d="M592 150 L638 250" class="f-none s-warn" stroke-width="2" marker-end="url(#fl31-3-w)"/>
<path d="M742 238 L742 156" class="f-none s-warn" stroke-width="2" marker-end="url(#fl31-3-w)"/>
<text x="16" y="382" class="t-small">Android emulator: app gọi http://10.0.2.2:8787. iOS Simulator: http://localhost:8787.</text>
</svg>
</div>
<figcaption>Hình 3. Mũi tên chỉ chiều tác động: patrol build và chạy app; app gọi fake API bằng socket của Dart; test Dart gọi native automator và native automator bấm hộp thoại.</figcaption>
</figure>

## 4. integration_test thuần: chạy và chụp màn hình

Ở cấp thấp nhất, `integration_test` là binding riêng cho `flutter test` trên thiết bị. Tài liệu Flutter chỉ lệnh chạy là `flutter test integration_test/app_test.dart` với thiết bị đang cắm. Test của lab ngắn vì mọi việc nằm ở `runBookingFlow`:

### booking_flow_test.dart · 31-integration-test-patrol/integration_test/booking_flow_test.dart (trích)

```dart
void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('đặt chuyến 5 km với fake API chạy trên máy dev', (tester) async {
    final driver = DriverSimulator(baseUrl: resolveApiBaseUrl());
    addTearDown(driver.close);

    await tester.pumpWidget(
      buildApp(
        overrides: [
          notificationPermissionProvider.overrideWithValue(
            FakeNotificationPermission(),
          ),
        ],
      ),
    );
    // Android phải chuyển bề mặt Flutter thành ảnh trước khi chụp; iOS bỏ qua.
    await binding.convertFlutterSurfaceToImage();

    final outcome = await runBookingFlow(
      tester,
      driverAccepts: driver.accept,
      timeout: const Duration(seconds: 30),
      screenshot: (name) async {
        await tester.pump();
        await binding.takeScreenshot(name);
      },
    );

    expect(outcome.fare.vnd, 25500);
    expect(outcome.driverId, 'D-01');
  });
}
```

`buildApp` là điểm gắn kết: `main()` gọi nó rồi `runApp`, còn test `pumpWidget` cùng widget gốc kèm `overrides`. Quyền thông báo là bản giả vì `integration_test` không bấm được hộp thoại của hệ điều hành.

Chụp màn hình có hai lưu ý đọc từ source của `integration_test`. Trên Android phải gọi `convertFlutterSurfaceToImage()` trước (không gọi thì `takeScreenshot` ném `StateError`), iOS bỏ qua. Ảnh chỉ về máy dev nếu chạy bằng `flutter drive`: `test_driver/integration_test.dart` nhận byte ảnh qua `onScreenshot` và ghi vào `build/screenshots/`.

## 5. Patrol: bấm được thứ nằm ngoài Flutter

Patrol thêm hai thứ. Một là `patrol_cli`, dựng app và bộ chạy instrumentation rồi chạy test native (Gradle trên Android, xcodebuild trên iOS). Hai là `$`, bộ finder của `patrol_finders` kèm `$.platform.mobile` để gọi hệ điều hành. Cấu hình bắt đầu ở `pubspec.yaml`:

```yaml
patrol:
  app_name: Toi Noi
  android:
    package_name: vn.toinoi.toi_noi_31
  ios:
    bundle_id: vn.toinoi.toiNoi31
```

Patrol tìm test trong `patrol_test/` (mặc định, đổi bằng `test_directory`), chỉ nhận file `_test.dart`; lab giữ `integration_test/` và `patrol_test/` riêng. Phía Android cần các dòng sau trong `native-config/android/app/build.gradle.kts`:

```kotlin
        testInstrumentationRunner = "pl.leancode.patrol.PatrolJUnitRunner"
        testInstrumentationRunnerArguments["clearPackageData"] = "true"
    }

    testOptions {
        execution = "ANDROIDX_TEST_ORCHESTRATOR"
    }
```

cộng `androidTestUtil("androidx.test:orchestrator:1.5.1")` và `MainActivityTest.java` trong `androidTest/java/vn/toinoi/toi_noi_31/`. `clearPackageData` xoá dữ liệu app, kể cả quyền đã cấp, giữa các test. Manifest chính khai `POST_NOTIFICATIONS`: quyền này chỉ có từ Android 13 (API 33), và `permission_handler` chỉ xử lý nó khi manifest có khai báo (đọc `PermissionUtils.java`). Phía iOS phải thêm target `RunnerUITests` bằng Xcode; README lab dịch đủ các bước của tài liệu Patrol, dễ sót nhất là tắt "Execute in parallel" và đặt User Script Sandboxing = No. Với Swift Package Manager, `permission_handler` bật sẵn quyền thông báo trên iOS.

Test đầu tiên của lab chạy lại đúng `runBookingFlow` nhưng truyền thêm cách trả lời hộp thoại thật:

### booking_native_test.dart · 31-integration-test-patrol/patrol_test/booking_native_test.dart (trích)

```dart
    // Không ghi đè quyền thông báo: app dùng permission_handler thật.
    await $.pumpWidgetAndSettle(buildApp());

    await runBookingFlow(
      $.tester,
      driverAccepts: driver.accept,
      timeout: const Duration(seconds: 30),
      answerPermissionDialog: () async {
        // Không phải mọi lần chạy đều có hộp thoại: Android dưới API 33 không
        // có, và quyền đã cấp từ trước thì hệ điều hành không hỏi lại.
        if (await $.platform.mobile.isPermissionDialogVisible(
          timeout: const Duration(seconds: 5),
        )) {
          await $.platform.mobile.grantPermissionWhenInUse();
        }
      },
      screenshot: $.takeNativeScreenshot,
    );
```

Nhánh `if` là chỗ hiếm hoi rẽ nhánh trong test là đúng: `grantPermissionWhenInUse` ném lỗi khi không có hộp thoại (theo tài liệu API), còn `isPermissionDialogVisible` chờ tối đa `timeout` rồi trả `true` hoặc `false`. `$.native` trong `patrol` 4.11.0 đã deprecated; dùng `$.platform.mobile`. `takeNativeScreenshot` chỉ chụp trên Android (iOS không làm gì), cần `patrol_cli` 4.8.0 và `patrol` 4.10.0 trở lên.

Test thứ hai dùng thứ `integration_test` không có: đưa app xuống nền. Ta đặt chuyến, `pressHome()`, để tài xế nhận lúc app ở nền, rồi `openApp()` và chờ màn hình đổi:

```dart
    await $.platform.mobile.pressHome();
    await actAsDriver($.tester, () => driver.accept(tripId));
    await $.platform.mobile.openApp();
```

Cùng nhóm API còn có `openNotifications`, `getNotifications` và `tapOnNotificationBySelector` để đọc và chạm vào thông báo thật. Lab chưa dùng vì app chưa phát thông báo; bài 37 làm việc đó. Lệnh chạy: `patrol test --target patrol_test/booking_native_test.dart --device <id>`, thêm `--dart-define=API_BASE_URL=...` khi cần (cờ này có trong `patrol_cli` 4.9.0).

## 6. Chi phí và khi nào đáng chạy

Chi phí của integration test không chỉ là giờ chạy. Mô tả trong bảng là minh hoạ để so sánh, không phải đo đạc:

| Khoản | Widget test | Integration/Patrol |
|---|---|---|
| Một lần chạy | Vài giây, cả nhóm trong một tiến trình | Vài phút: dựng app, cài, khởi động thiết bị |
| Hạ tầng | Máy dev, CI Linux | Emulator hoặc simulator (macOS cho iOS), server giả |
| Nguồn gây flaky | Ít, đồng hồ giả | Đồng hồ thật, mạng, hộp thoại, thiết bị khởi động chậm |

Khuyến nghị (không phải luật): giữ luồng chính và những chỗ chỉ thiết bị mới lộ (quyền, app ở nền, cấu hình native); mọi trường hợp biên (150 km, 503, mất kết nối) đã có widget test. Một lỗi tìm ra bằng integration test thì sửa xong nên thêm test ở tầng thấp nhất tái hiện được nó. Chạy ở mọi commit thường quá đắt; nên chọn lịch chạy ban đêm, trước khi phát hành, hoặc khi đổi file trong `android/` và `ios/`. Bài 33 dựng CI cho analyze và `flutter test`; test trên thiết bị cần runner có emulator nên là quyết định riêng.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp khi test trên thiết bị</span>
<ul>
<li><strong>Dùng <code>pumpAndSettle</code> khi màn hình có animation chạy mãi.</strong> Dấu hiệu: <code>pumpAndSettle timed out</code>. Với finder của Patrol, mặc định là <code>trySettle</code> giới hạn 10 giây (đọc <code>PatrolTesterConfig</code>): không ném lỗi nhưng mỗi thao tác chờ tới 10 giây. Cách sửa: chờ theo điều kiện bằng <code>pumpUntilFound</code>, hoặc <code>settlePolicy: SettlePolicy.noSettle</code>.</li>
<li><strong>Emulator gọi <code>localhost</code>.</strong> Dấu hiệu: app báo lỗi kết nối dù server đang chạy trên máy dev. Nguyên nhân: <code>localhost</code> là chính emulator. Cách sửa: dùng <code>10.0.2.2</code>, hoặc truyền <code>API_BASE_URL</code> (thiết bị thật).</li>
<li><strong>Gọi <code>grantPermissionWhenInUse</code> khi không có hộp thoại.</strong> Dấu hiệu: test lần đầu xanh, lần sau đỏ, hoặc đỏ trên Android dưới API 33. Nguyên nhân: quyền đã cấp từ trước hoặc máy không có hộp thoại này. Cách sửa: bọc bằng <code>isPermissionDialogVisible</code>.</li>
<li><strong>Gọi <code>runApp</code>, <code>WidgetsFlutterBinding.ensureInitialized</code> hoặc sửa <code>FlutterError.onError</code> trong test Patrol.</strong> Dấu hiệu: test không thấy ngoại lệ nên không dừng khi lỗi. Tài liệu Patrol dặn không làm cả ba: dùng <code>$.pumpWidget</code> và đặt phần khởi tạo chung vào một hàm như <code>buildApp</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

**Mức kiểm chứng: trên thiết bị (TB), và máy viết bài không có thiết bị.** Ở đây, `integration_test/` và `patrol_test/` chỉ được `flutter analyze` kiểm kiểu và tên API (đủ để bắt sai tên hàm của `patrol` 4.11.0); **chưa chạy trên thiết bị**. File trong `native-config/` chưa được build bằng Gradle hay Xcode. Phần chạy được là `flutter test`, nên logic của kịch bản được kiểm tự động. Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/31-integration-test-patrol
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` 31 test xanh. Các nhóm test chứng minh:

- **Luồng đầy đủ**: thứ tự request, khoá idempotency, header tài xế, giá của ba dịch vụ, từ chối thông báo mà luồng vẫn đi tiếp.
- **Lỗi**: 422 cho 150 km; 503 khi đặt chuyến (giữ giá, dùng lại cùng khoá); mất kết nối khi hỏi trạng thái rồi "Thử lại"; `pumpAndSettle` hết thời gian chờ; rời màn hình thì ngừng hỏi.
- **Cấu hình native**: package khớp khối `patrol:`, runner và orchestrator, cleartext chỉ ở bản debug và chỉ cho `10.0.2.2`, không có bí mật.
- **Còn lại**: `resolveApiBaseUrl` theo nền tảng, `DriverSimulator` qua HTTP thật trên loopback.

Để chạy trên thiết bị, làm theo README của lab: sinh runner bằng `flutter create`, chạy `node tool/apply-native-config.mjs` (script cũng xoá `test/widget_test.dart` mà `flutter create` sinh, vì file đó làm `flutter analyze` báo lỗi), bật fake API, rồi `flutter test integration_test/booking_flow_test.dart -d <id>` hoặc `patrol test`. **Chưa làm**: chạy trên thiết bị (cả hai runner), thông báo thật, device farm và CI, flavor.

## Bài tập

1. Thêm một test Patrol cho trường hợp người dùng bấm "Don't allow". Cần dùng hàm nào của Patrol, và `runBookingFlow` có chạy được nguyên vẹn không?
2. Ta chạy bản debug trên một điện thoại Android thật cắm cáp, fake API ở máy dev có IP LAN `192.168.1.20`. Cần đổi những gì ở app, server và cấu hình native?
3. Một đồng nghiệp muốn thêm test trên thiết bị cho trường hợp "nhập 150 km thì báo lỗi". Bạn có đồng ý không?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Dùng <code>$.platform.mobile.denyPermission()</code> thay cho <code>grantPermissionWhenInUse()</code>. <code>runBookingFlow</code> không chạy nguyên vẹn vì nó chờ chữ "Thông báo: đã bật". Ghép từng bước: <code>requestQuote</code>, <code>placeBooking</code>, rồi <code>askForNotifications(expected: NotificationPermissionStatus.denied)</code>, như test từ chối ở bản <code>flutter test</code>.</p>
<p><strong>2.</strong> App: truyền <code>--dart-define=API_BASE_URL=http://192.168.1.20:8787</code>, vì <code>10.0.2.2</code> chỉ có trong emulator. Server: chạy với <code>--host 0.0.0.0</code> và chỉ trên mạng tin cậy. Native: với <code>dio</code> mặc định (<code>dart:io</code>) không phải đổi; chỉ khi dùng adapter native mới thêm <code>192.168.1.20</code> vào <code>network_security_config.xml</code> của bản debug (và cấu hình ATS trên iOS).</p>
<p><strong>3.</strong> Không. Trường hợp đó kiểm tra đầu vào, không phụ thuộc hệ điều hành, và đã có widget test chạy trong vài giây. Test trên thiết bị đắt hơn mà không thấy thêm lỗi nào; chỉ thêm khi có bằng chứng lỗi chỉ xảy ra trên thiết bị.</p>
</details>

Bài trước: [Kiểm thử Riverpod, async và Stream](/articles/flutter-test-riverpod-async-stream-fake-async-providercontainer). Bài tiếp: [Lint, analyzer, dart format và analyzer plugin](/articles/flutter-lint-analyzer-format-analyzer-plugin-chat-luong-code).

## Tài liệu gốc

- [Testing Flutter apps](https://docs.flutter.dev/testing/overview) và [Integration testing](https://docs.flutter.dev/testing/integration-tests) (docs.flutter.dev).
- [package:integration_test](https://pub.dev/packages/integration_test) và [package:patrol](https://pub.dev/packages/patrol) (pub.dev).
- [Install Patrol](https://patrol.leancode.co/documentation), [Write your first test](https://patrol.leancode.co/documentation/write-your-first-test) và [patrol test](https://patrol.leancode.co/cli-commands/test) (patrol.leancode.co).
- [Network security configuration](https://developer.android.com/privacy-and-security/security-config), [Notification runtime permission](https://developer.android.com/develop/ui/views/notifications/notification-permission) và [Network address space](https://developer.android.com/studio/run/emulator-networking-address) (developer.android.com).
- [NSAllowsLocalNetworking](https://developer.apple.com/documentation/bundleresources/information-property-list/nsapptransportsecurity/nsallowslocalnetworking) (developer.apple.com).
- [package:permission_handler](https://pub.dev/packages/permission_handler) (pub.dev).

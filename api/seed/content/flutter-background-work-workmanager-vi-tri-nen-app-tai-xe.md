---
slug: flutter-background-work-workmanager-vi-tri-nen-app-tai-xe
title: "Tác vụ nền: workmanager, vị trí nền cho app tài xế và chính sách store"
summary: "Bài 38: giới hạn nền của iOS và Android, workmanager cho việc chờ được, DriverLocationPolicy chỉ cho gửi vị trí nền khi chuyến từ accepted tới inTransit, bộ đệm giới hạn khi mất mạng, foreground service location, chỉ báo xanh dương iOS và điều khoản Google Play, App Store. Chưa chạy trên thiết bị."
category: programming
level: advanced
tags: [flutter, dart, mobile, toi-noi, android, ios, background, location, workmanager, store]
language: vi
---

Đội QA của Tới Nơi chạy thử app tài xế trên một máy Android và báo lỗi: tài xế nhận chuyến rồi chuyển sang app bản đồ để dẫn đường, vài chục giây sau trên app khách chiếc xe đứng yên rồi nhảy một đoạn dài. App tài xế không sai; khi rời màn hình, nó thôi là việc ưu tiên của hệ điều hành. Tình huống là minh hoạ.

Bài này xếp việc nền thành ba loại: việc chờ được dùng `workmanager`; việc người dùng thấy, như chia sẻ vị trí trong chuyến, dùng foreground service (Android) và background mode `location` (iOS); loại thứ ba, chạy ngầm mãi mà không ai thấy, không có cách nào. Ta viết `DriverLocationPolicy` để chỉ thu vị trí khi có chuyến hợp lệ, thêm bộ đệm giới hạn khi mất mạng, rồi đối chiếu điều khoản Google Play và Apple.

Đây là bài 38 của lộ trình **Flutter qua Tới Nơi**. Nên đọc trước [bài 35 về vị trí và quyền](/articles/flutter-vi-tri-geolocator-permission-foreground-background), [bài 36 về realtime](/articles/flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe) và [bài 37 về push notification](/articles/flutter-push-notification-fcm-apns-deep-link). Lab dùng Flutter 3.47.4, Dart 3.13.3, `workmanager` 0.10.10, `geolocator` 14.1.2, trên hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Máy viết bài không có Android SDK hay Xcode đầy đủ. Logic được test tự động (106 test); foreground service, chỉ báo xanh dương, <code>BGTaskScheduler</code> và việc duyệt của store <strong>chưa chạy trên thiết bị nào</strong>.</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Task định kỳ của <code>workmanager</code> đăng ký với <code>frequency</code> 5 phút. Android có chạy mỗi 5 phút không? iOS thì sao?</p>
<details><summary>Xem đáp án</summary><p>Không trên cả hai. Android tự nâng chu kỳ dưới 15 phút lên 15 phút (chú thích API của workmanager, khớp tài liệu WorkManager). Trên iOS plugin không dùng <code>frequency</code> và hệ điều hành không đảm bảo thời điểm chạy (mục 2).</p></details>
</div>

## 1. Nền của iOS và Android cho ta những gì

Android coi app là "chạy ở nền" khi không activity nào hiển thị và không có foreground service nào. Tài liệu Android chia công cụ nền thành asynchronous work, task scheduling APIs (WorkManager) và foreground services; Hình 1 xếp việc của Tới Nơi theo đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Việc cần làm khi app không ở trên màn hình rẽ ba nhánh. Chờ được vài phút thì dùng workmanager: Android tối thiểu 15 phút, iOS không đảm bảo. Đang diễn ra và người dùng thấy thì dùng dịch vụ nền hiển thị: Android foreground service loại location kèm thông báo, iOS background mode location với chỉ báo. Cần chạy ngầm mãi mà không ai thấy thì không có cách."><defs><marker id="fl38-1-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="110" width="190" height="120" rx="10" class="f-surface s-line"/><text x="36" y="148" class="t-b">Việc cần làm</text><text x="36" y="174" class="t-small">khi app không ở</text><text x="36" y="194" class="t-small">trên màn hình</text>
<rect x="290" y="20" width="570" height="92" rx="10" class="f-phys-soft s-phys"/><text x="306" y="48" class="t-b">Chờ được vài phút: workmanager</text><text x="306" y="72" class="t-small">Android: tối thiểu 15 phút, hệ điều hành chọn giờ chạy</text><text x="306" y="94" class="t-small">iOS: không đảm bảo, tác vụ ngắn (plugin ghi khoảng 30 giây)</text>
<rect x="290" y="124" width="570" height="92" rx="10" class="f-pk-soft s-pk"/><text x="306" y="152" class="t-b">Đang diễn ra và người dùng thấy: dịch vụ nền hiển thị</text><text x="306" y="176" class="t-small">Android: foreground service loại location kèm thông báo</text><text x="306" y="198" class="t-small">iOS: background mode location, hệ điều hành hiện chỉ báo</text>
<rect x="290" y="228" width="570" height="92" rx="10" class="f-warn-soft s-warn"/><text x="306" y="256" class="t-b t-warn">Cần chạy ngầm mãi mà không ai thấy: không có cách</text><text x="306" y="280" class="t-small">Làm khi app đang mở, hoặc để server và push notification làm</text><text x="306" y="302" class="t-small">Thiết kế lại luồng thay vì tìm mẹo</text>
<path d="M212 150 L288 66" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-1-m)"/>
<path d="M212 170 L288 170" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-1-m)"/>
<path d="M212 190 L288 274" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-1-m)"/>
</svg>
</div>
<figcaption>Hình 1. Ba loại việc nền. Mốc 15 phút từ tài liệu WorkManager, mốc 30 giây từ chú thích API của workmanager (kiểm tại 11/2026).</figcaption>
</figure>

Dọn mẫu vị trí cũ và gửi bù khi có mạng là việc chờ được; tài liệu Android khuyên WorkManager và cảnh báo việc nền quá khoảng 10 phút dễ bị ngắt. Chia sẻ vị trí lúc đón hoặc chở khách là việc người dùng thấy, thuộc về foreground service. "Đánh thức app mỗi 10 giây khi tài xế không có chuyến" nằm ở nhánh thứ ba: không làm được và không nên.

## 2. workmanager: việc chờ được

`workmanager` bọc WorkManager của Android và `BGTaskScheduler` của iOS. Hệ điều hành gọi task từ phía native, nên điểm vào phải là hàm cấp cao nhất gắn `@pragma('vm:entry-point')` (README plugin yêu cầu), để trình biên dịch AOT không cắt hàm này.

### background_dispatcher.dart · 38-background-work/lib/background_dispatcher.dart (trích)

```dart
@pragma('vm:entry-point')
void callbackDispatcher() {
  Workmanager().executeTask((taskName, inputData) async {
    try {
      final dir = await getApplicationSupportDirectory();
      return await runBackgroundTask(
        taskName,
        store: FileBufferStore(File('${dir.path}/pending_samples.json')),
        sink: const LoggingLocationSink(),
        now: () => DateTime.now().toUtc(),
      );
    } on Exception {
      return false;
    }
  });
}
```

Hàm này giao cho `runBackgroundTask`, hàm thuần Dart test được bằng fake; chu kỳ được kiểm trước khi chạm plugin:

### background_tasks.dart · 38-background-work/lib/background_tasks.dart (trích)

```dart
/// Mô tả một task định kỳ, kiểm giá trị trước khi chạm vào plugin.
final class PeriodicTaskPlan {
  PeriodicTaskPlan({required this.name, required this.frequency}) {
    if (frequency < minimumPeriodicFrequency) {
      throw ArgumentError.value(
        frequency,
        'frequency',
        'tối thiểu ${minimumPeriodicFrequency.inMinutes} phút; Android sẽ tự '
            'nâng lên mức đó nên đặt thấp hơn chỉ là hứa điều không giữ',
      );
    }
  }

  final String name;
  final Duration frequency;
}

// ...

Future<bool> runBackgroundTask(
  String taskName, {
  required BufferStore store,
  required LocationSink sink,
  required DateTime Function() now,
}) async {
  final buffer = LocationBuffer()..replaceAll(await store.load());
  switch (taskName) {
    case TaskNames.purgeExpired:
      buffer.purgeExpired(now());
      await store.save(buffer.items);
      return true;
    case TaskNames.flushPending:
      final result = await LocationUploader(
        sink: sink,
        buffer: buffer,
        store: store,
        now: now,
      ).flush();
      return !result.failed;
    default:
      return false; // tên lạ: lỗi lập trình, không có gì để làm
  }
}
```

Theo chú thích API, trả `false` trên Android làm WorkManager thử lại theo backoff, còn trên iOS giá trị trả về bị bỏ qua; việc hẹn lại trên iOS là việc của app.

Cấu hình iOS có ba điểm dễ sai. Một, identifier của `BGTaskScheduler` phải khai trong `Info.plist`, trùng từng ký tự với `uniqueName`. Hai, task định kỳ cần background mode `fetch`. Ba, template Flutter 3.47 dùng UIScene nên plugin đăng ký sau `didFinishLaunching`, trong khi iOS chỉ giao task cho handler đăng ký ở đó; `AppDelegate` phải gọi `WorkmanagerPlugin.registerLaunchHandlers()` (theo chú thích của plugin; Swift chưa biên dịch).

### Info.plist.fragment.xml · 38-background-work/native-config/driver/ios/Runner/Info.plist.fragment.xml (trích)

```xml
	<!-- Chỉ quyền "khi đang dùng": tài xế bắt đầu chia sẻ lúc app đang mở (nhận
	     chuyến) rồi chuyển sang app dẫn đường, đúng kịch bản mà Apple mô tả cho
	     background mode location. Không khai NSLocationAlways... -->
	<key>NSLocationWhenInUseUsageDescription</key>
	<string>Tới Nơi dùng vị trí của bạn để ghép chuyến gần nhất. Trong chuyến đang chạy, app tiếp tục gửi vị trí khi bạn chuyển sang app dẫn đường để khách thấy xe đang tới. Ngoài chuyến, app không thu vị trí.</string>

	<!-- location: nhận vị trí khi app ở nền. fetch: cần cho BGAppRefreshTask của workmanager. -->
	<key>UIBackgroundModes</key>
	<array>
		<string>location</string>
		<string>fetch</string>
	</array>

	<!-- Mọi identifier của BGTaskScheduler phải khai ở đây, trùng từng ký tự với
	     uniqueName khi gọi registerPeriodicTask, nếu không iOS từ chối (lỗi
	     BGTaskSchedulerErrorDomain code 3 theo tài liệu workmanager). -->
	<key>BGTaskSchedulerPermittedIdentifiers</key>
	<array>
		<string>vn.toinoi.driver.purge-expired</string>
	</array>
```

Thiếu một trong hai thì iOS báo `notPermitted` (`BGTaskSchedulerErrorDomain` code 3); trang Apple của lỗi này xác nhận đúng hai nguyên nhân đó (kiểm tại 11/2026). Trên iOS `registerPeriodicTask` bỏ qua `frequency`; `initialDelay` chỉ thành `earliestBeginDate`, tức cận dưới.

Điều đáng nhớ nhất là bộ nhớ. Trên Android mỗi worker dựng một `FlutterEngine` nền riêng (`BackgroundWorker.kt`), nên `LocationBuffer` của giao diện không tồn tại ở đó (Hình 2). Trên iOS, task một lần chạy trên engine chính khi app còn sống, còn task định kỳ do `BGTaskScheduler` giao thì chạy trên engine nền (`WorkmanagerPlugin.swift`). Không đoán trước được đường nào, nên code phải giả định hai bên **không** chung bộ nhớ và chỉ tin dữ liệu trên đĩa.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Hai isolate có bộ nhớ riêng. Isolate giao diện có LocationBuffer với 1 mẫu. Isolate của workmanager có LocationBuffer với 0 mẫu. Hai bên không thấy nhau trong RAM. Cả hai đọc và ghi cùng file pending_samples.json trên đĩa."><defs><marker id="fl38-2-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="20" width="300" height="160" rx="10" class="f-phys-soft s-phys"/><text x="36" y="48" class="t-b">Isolate giao diện</text><text x="36" y="68" class="t-small">controller, uploader</text>
<rect x="40" y="86" width="260" height="70" rx="8" class="f-surface s-line"/><text x="56" y="112" class="t-mono">LocationBuffer</text><text x="56" y="136" class="t-small">1 mẫu, trong RAM của isolate này</text>
<rect x="560" y="20" width="300" height="160" rx="10" class="f-phys-soft s-phys"/><text x="576" y="48" class="t-b">Isolate của workmanager</text><text x="576" y="68" class="t-small">callbackDispatcher, runBackgroundTask</text>
<rect x="580" y="86" width="260" height="70" rx="8" class="f-surface s-line"/><text x="596" y="112" class="t-mono">LocationBuffer</text><text x="596" y="136" class="t-small">0 mẫu, bản khởi tạo riêng</text>
<path d="M324 100 L556 100" class="f-none s-warn dash" stroke-width="2"/><text x="352" y="90" class="t-small t-warn">RAM riêng, không thấy nhau</text>
<rect x="290" y="240" width="300" height="80" rx="10" class="f-pk-soft s-pk"/><text x="306" y="268" class="t-mono">pending_samples.json</text><text x="306" y="292" class="t-small">ghi qua file tạm riêng rồi đổi tên</text><text x="306" y="306" class="t-small">ghi đồng thời: bản sau cùng thắng</text>
<path d="M170 182 L330 238" class="f-none s-pk" stroke-width="2" marker-end="url(#fl38-2-p)"/><text x="190" y="228" class="t-small">save, load</text>
<path d="M710 182 L550 238" class="f-none s-pk" stroke-width="2" marker-end="url(#fl38-2-p)"/><text x="640" y="228" class="t-small">save, load</text>
</svg>
</div>
<figcaption>Hình 2. Khi task nền chạy ở isolate khác (như trên Android), chỉ file trên đĩa là cả hai bên cùng thấy. Số 1 và 0 là kết quả test bên dưới.</figcaption>
</figure>

### buffer_store_test.dart · 38-background-work/test/buffer_store_test.dart (trích)

```dart
  group('bộ nhớ không chia sẻ giữa isolate, đĩa thì có', () {
    test('biến trong bộ nhớ: isolate khác thấy bộ đệm trống', () async {
      processBuffer.add(sample(0));
      expect(processBuffer.length, 1);

      // Closure không bắt processBuffer; isolate mới tự khởi tạo biến cấp
      // cao nhất của nó từ đầu.
      final seenElsewhere = await Isolate.run(() => processBuffer.length);
      expect(seenElsewhere, 0);
    });

    test('file trên đĩa: isolate khác đọc được mẫu đã lưu', () async {
      await FileBufferStore(file).save([sample(0), sample(1)]);
      final path = file.path;

      final seenElsewhere = await Isolate.run(
        () async => (await FileBufferStore(File(path)).load()).length,
      );
      expect(seenElsewhere, 2);
    });
  });
```

Mỗi lần ghi dùng file tạm riêng (test cho 3 isolate ghi 100 lần, không lỗi), vì dùng chung một tên tạm thì các isolate đổi tên mất file của nhau. Hai lần ghi đồng thời không hợp nhất: bản sau cùng thắng, bản kia mất. Hai bên còn có thể gửi trùng một mẫu; `LocationSample.dedupeKey` để server khử trùng (lab chưa kiểm).

## 3. DriverLocationPolicy: chỉ gửi khi có chuyến hợp lệ

Quy tắc nằm trong một lớp thuần Dart, không biết gì về plugin. Hình 3 cho thấy tám trạng thái của `toi_noi_core` rơi vào bên nào.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 3. Bảy trạng thái theo đường đi bình thường từ requested tới completed. Bốn trạng thái accepted, arriving, pickedUp, inTransit nằm trong vùng luồng vị trí bật. Trạng thái cancelled nằm riêng, có mũi tên đứt từ searching, accepted và arriving. Hết chuyến hoặc tài xế offline thì dừng nguồn vị trí và xoá bộ đệm."><defs><marker id="fl38-3-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker><marker id="fl38-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="225" y="52" class="t-b t-pk">Luồng vị trí BẬT: foreground service, chỉ báo xanh dương</text>
<path d="M269 84 L269 66 L733 66 L733 84" class="f-none s-pk" stroke-width="2"/>
<rect x="25" y="100" width="98" height="56" rx="8" class="f-surface s-line"/><text x="41" y="133" class="t-mono">requested</text>
<rect x="147" y="100" width="98" height="56" rx="8" class="f-surface s-line"/><text x="163" y="133" class="t-mono">searching</text>
<rect x="269" y="100" width="98" height="56" rx="8" class="f-pk-soft s-pk"/><text x="288" y="133" class="t-mono">accepted</text>
<rect x="391" y="100" width="98" height="56" rx="8" class="f-pk-soft s-pk"/><text x="410" y="133" class="t-mono">arriving</text>
<rect x="513" y="100" width="98" height="56" rx="8" class="f-pk-soft s-pk"/><text x="534" y="133" class="t-mono">pickedUp</text>
<rect x="635" y="100" width="98" height="56" rx="8" class="f-pk-soft s-pk"/><text x="654" y="133" class="t-mono">inTransit</text>
<rect x="757" y="100" width="98" height="56" rx="8" class="f-surface s-line"/><text x="772" y="133" class="t-mono">completed</text>
<path d="M124 128 L145 128" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-3-m)"/>
<path d="M246 128 L267 128" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-3-m)"/>
<path d="M368 128 L389 128" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-3-m)"/>
<path d="M490 128 L511 128" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-3-m)"/>
<path d="M612 128 L633 128" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-3-m)"/>
<path d="M734 128 L755 128" class="f-none s-muted" stroke-width="2" marker-end="url(#fl38-3-m)"/>
<rect x="269" y="226" width="98" height="56" rx="8" class="f-surface s-line"/><text x="288" y="259" class="t-mono">cancelled</text>
<path d="M196 158 L296 224" class="f-none s-warn dash" stroke-width="2" marker-end="url(#fl38-3-w)"/>
<path d="M318 158 L318 224" class="f-none s-warn dash" stroke-width="2" marker-end="url(#fl38-3-w)"/>
<path d="M440 158 L340 224" class="f-none s-warn dash" stroke-width="2" marker-end="url(#fl38-3-w)"/>
<text x="392" y="256" class="t-small">huỷ được ở searching, accepted, arriving</text>
<rect x="20" y="304" width="840" height="44" rx="10" class="f-warn-soft s-warn"/><text x="36" y="331" class="t-b t-warn">Hết chuyến (completed, cancelled) hoặc tài xế offline: dừng nguồn vị trí, xoá bộ đệm</text>
</svg>
</div>
<figcaption>Hình 3. Bốn trạng thái có tài xế đang làm việc với khách được theo dõi; chưa có tài xế hoặc đã kết thúc thì không.</figcaption>
</figure>

### driver_location_policy.dart · 38-background-work/lib/driver_location_policy.dart

```dart
final class DriverLocationPolicy {
  const DriverLocationPolicy();

  /// Chỉ khi tài xế đang online và có một chuyến ở trạng thái cần theo dõi.
  ///
  /// [tripStatus] là `null` khi tài xế chưa có chuyến nào.
  bool allowsBackgroundSharing({
    required TripStatus? tripStatus,
    required bool driverOnline,
  }) {
    if (!driverOnline || tripStatus == null) return false;
    return tracksStatus(tripStatus);
  }

  /// Trạng thái của chuyến mà khách cần thấy xe di chuyển.
  ///
  /// Liệt kê đủ tám trạng thái, không dùng `_`: thêm trạng thái mới thì
  /// `dart analyze` buộc ta quyết định nó có được theo dõi hay không.
  bool tracksStatus(TripStatus status) => switch (status) {
    TripStatus.accepted ||
    TripStatus.arriving ||
    TripStatus.pickedUp ||
    TripStatus.inTransit => true,
    // Chưa có tài xế hoặc chuyến đã kết thúc: không có lý do gì để thu vị trí.
    TripStatus.requested ||
    TripStatus.searching ||
    TripStatus.completed ||
    TripStatus.cancelled => false,
  };
}
```

`switch` liệt kê đủ tám trạng thái, không có nhánh `_`: khi `toi_noi_core` thêm trạng thái thứ chín, `dart analyze` báo ngay và ta buộc phải quyết định có thu vị trí ở đó không.

Test lặp trên `TripStatus.values`, đối chiếu với một bảng đặc tả viết tay cũng được kiểm là phủ đủ tám trạng thái:

### driver_location_policy_test.dart · 38-background-work/test/driver_location_policy_test.dart (trích)

```dart
  // Bảng đặc tả viết tay, độc lập với cài đặt: bốn trạng thái có tài xế đang
  // làm việc với khách thì được theo dõi, bốn trạng thái còn lại thì không.
  const expectedTracked = <TripStatus, bool>{
    TripStatus.requested: false,
    TripStatus.searching: false,
    TripStatus.accepted: true,
    TripStatus.arriving: true,
    TripStatus.pickedUp: true,
    TripStatus.inTransit: true,
    TripStatus.completed: false,
    TripStatus.cancelled: false,
  };

  test('bảng đặc tả phủ đủ tám trạng thái của toi_noi_core', () {
    expect(TripStatus.values, hasLength(8));
    expect(expectedTracked.keys.toSet(), TripStatus.values.toSet());
  });

  group('tài xế online', () {
    for (final status in TripStatus.values) {
      final expected = expectedTracked[status]!;
      test('${status.name} -> ${expected ? 'được gửi' : 'không gửi'}', () {
        expect(
          policy.allowsBackgroundSharing(
            tripStatus: status,
            driverOnline: true,
          ),
          expected,
        );
        expect(policy.tracksStatus(status), expected);
      });
    }
  });
```

`DriverLocationController` bật hoặc tắt luồng vị trí mỗi khi trạng thái chuyến hoặc online đổi. Test của nó lặp đủ tám trạng thái và chứng minh: chỉ bật một lần cho cả chuyến, dừng ngay ở `completed`, `cancelled` hoặc khi offline, không trộn vị trí hai chuyến.

## 4. Bộ đệm có giới hạn khi mất mạng

Xe qua hầm thì mất mạng nhưng GPS vẫn có vị trí. Ta giữ mẫu để gửi bù, **có giới hạn** theo hai trục: tối đa 120 mẫu (đầy thì bỏ mẫu cũ nhất, vì mẫu mới mô tả xe đúng hơn) và tối đa 10 phút tuổi (vị trí cũ hơn không còn giúp khách). Hai số này là minh hoạ: mỗi 5 giây một mẫu thì 120 mẫu đúng 10 phút. Giữ vô hạn còn để lại trên máy dữ liệu vị trí không còn lý do tồn tại.

### location_buffer.dart · 38-background-work/lib/location_buffer.dart (trích)

```dart
  void add(LocationSample sample) {
    if (_items.length >= capacity) {
      _items.removeFirst();
      _droppedCount++;
    }
    _items.addLast(sample);
  }

  /// Bỏ mẫu già hơn [maxAge] so với [now] và trả về số mẫu đã bỏ.
  int purgeExpired(DateTime now) {
    final before = _items.length;
    _items.removeWhere((s) => now.difference(s.capturedAt) > maxAge);
    return before - _items.length;
  }
```

Bộ gửi lấy mẫu theo lô nhưng **chỉ xoá sau khi server đã nhận**:

### location_uploader.dart · 38-background-work/lib/location_uploader.dart (trích)

```dart
  /// Gửi lần lượt từng lô cũ nhất cho tới khi hết hoặc có lỗi.
  ///
  /// Hai lời gọi chồng nhau dùng chung một lần gửi, nếu không cùng một mẫu
  /// sẽ lên server hai lần.
  Future<FlushResult> flush() =>
      _inFlight ??= _flush().whenComplete(() => _inFlight = null);

  Future<FlushResult> _flush() async {
    buffer.purgeExpired(_now());
    var sent = 0;
    var failed = false;
    while (!buffer.isEmpty) {
      final batch = buffer.peek(batchSize);
      try {
        await sink.send(batch);
      } on Exception {
        failed = true;
        break;
      }
      buffer.removeSent(batch);
      sent += batch.length;
    }
    await _persist();
    final result = FlushResult(
      sent: sent,
      pending: buffer.length,
      failed: failed,
    );
    if (failed) onFailed?.call(result);
    return result;
  }
```

Việc xoá theo danh tính mẫu (`removeSent`), không theo số lượng: lúc đang gửi, mẫu mới có thể đẩy một mẫu cũ ra khỏi bộ đệm đầy, và xoá "n mẫu đầu" sẽ xoá nhầm mẫu chưa gửi (có test). Khi chuyến kết thúc, `discardPending` xoá cả bộ đệm lẫn file.

Test bộ điều khiển dựng kịch bản hầm: bộ đệm 3 mẫu, năm mẫu đến lúc mất mạng, mẫu thứ sáu khi có mạng. Server nhận ba mẫu mới nhất (3, 4, 5), `droppedCount` ghi 3.

## 5. Vị trí nền: foreground service và chỉ báo xanh dương

`workmanager` không làm được việc này: enum `ForegroundServiceType` của nó chỉ có `dataSync` và `shortService` (test kiểm trên bản đã pin). Luồng vị trí đi qua `geolocator`, cấu hình theo nền tảng nằm trong một hàm:

### location_tracker.dart · 38-background-work/lib/location_tracker.dart (trích)

```dart
LocationSettings buildBackgroundLocationSettings(TargetPlatform platform) {
  const accuracy = LocationAccuracy.high;
  const distanceFilter = 25; // mét: xe đứng yên thì không phát thêm mẫu
  return switch (platform) {
    TargetPlatform.android => AndroidSettings(
      accuracy: accuracy,
      distanceFilter: distanceFilter,
      intervalDuration: const Duration(seconds: 5),
      foregroundNotificationConfig: const ForegroundNotificationConfig(
        notificationTitle: 'Tới Nơi đang chia sẻ vị trí',
        notificationText: 'Khách đang theo dõi xe của bạn trong chuyến này.',
        notificationChannelName: 'Vị trí chuyến đi',
        setOngoing: true,
      ),
    ),
    TargetPlatform.iOS => AppleSettings(
      accuracy: accuracy,
      distanceFilter: distanceFilter,
      activityType: ActivityType.automotiveNavigation,
      pauseLocationUpdatesAutomatically: false,
      allowBackgroundLocationUpdates: true,
      showBackgroundLocationIndicator: true,
    ),
    TargetPlatform.fuchsia ||
    TargetPlatform.linux ||
    TargetPlatform.macOS ||
    TargetPlatform.windows => const LocationSettings(
      accuracy: accuracy,
      distanceFilter: distanceFilter,
    ),
  };
}
```

`distanceFilter` 25 mét và chu kỳ 5 giây là số minh hoạ chưa đo pin.

`distanceFilter` 25 mét và chu kỳ 5 giây là số minh hoạ chưa đo pin.

**Android.** Khi `foregroundNotificationConfig` có giá trị, plugin chạy luồng vị trí trong foreground service của nó; huỷ subscription thì dừng service (đọc `StreamHandlerImpl.java`, `geolocator_android` 5.2.1). Tài liệu Android đặt các điều kiện: từ Android 14 phải khai loại `location` cùng `FOREGROUND_SERVICE_LOCATION` và một quyền vị trí lúc chạy; service phải có thông báo (nếu người dùng từ chối `POST_NOTIFICATIONS` thì thông báo chỉ hiện trong Task Manager); và service loại `location` chỉ được tạo khi app đang hiển thị, trừ khi app có `ACCESS_BACKGROUND_LOCATION`.

Ta chọn **không** khai `ACCESS_BACKGROUND_LOCATION`: service khởi động lúc tài xế bấm nhận chuyến, khi activity đang hiện, rồi chạy tiếp lúc app sang nền. Đây là suy ra từ tài liệu, chưa chạy trên máy; Logcat không được có dòng "Foreground service started from background can not have location/camera/microphone access". Nếu cần bắt đầu khi app đã ở nền (ví dụ tự gán chuyến), quyền này thành bắt buộc. `geolocator` ghi thêm rằng foreground service không ngăn Android huỷ activity.

**iOS.** Theo Apple, `allowsBackgroundLocationUpdates` cần `UIBackgroundModes` có `location`; bật luồng vị trí lúc app đang ở foreground thì cập nhật tiếp tục khi app sang nền. Chỉ báo xanh dương không phải lý do để xin quyền "luôn luôn": với quyền "khi đang dùng" hệ thống tự đổi thanh trạng thái, còn `showBackgroundLocationIndicator` chỉ tác động app đã có quyền "luôn luôn". Tài xế bắt đầu chia sẻ lúc app đang mở, nên app chỉ xin "khi đang dùng", và mô tả quyền nói rõ chỉ thu trong chuyến.

**App khách khác app tài xế ở đâu.** Mỗi quyền thừa là một câu hỏi của người dùng và của đội duyệt store, nên hai app có cấu hình riêng và test kiểm sự khác biệt:

| | App khách | App tài xế |
|---|---|---|
| Quyền Android | `INTERNET`, vị trí gần đúng và chính xác | thêm `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_LOCATION`, `POST_NOTIFICATIONS` |
| `UIBackgroundModes` | không có | `location`, `fetch` |
| `BGTaskSchedulerPermittedIdentifiers` | không có | `vn.toinoi.driver.purge-expired` |
| Cờ `BYPASS_PERMISSION_LOCATION_ALWAYS` | có | có |

Cờ cuối của `geolocator_apple` bỏ lời gọi `requestAlwaysAuthorization`, để app chỉ-khi-đang-dùng khỏi khai mô tả "luôn luôn" (comment trong `Package.swift` nhắc lỗi ITMS-90683 khi tải lên), nên cả hai app đều cần. Với CocoaPods, đặt trong `Podfile`. Swift Package Manager (mặc định trên Flutter stable, `geolocator_apple` có `Package.swift`) thì Podfile vô tác dụng: phải build bằng `BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios` (README geolocator). Test kiểm cả hai dạng. Vì `workmanager` tự thêm quyền foreground service vào app dùng nó, lab giữ nó ngoài app khách.

### AndroidManifest.xml · 38-background-work/native-config/driver/android/app/src/main/AndroidManifest.xml (trích)

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET"/>

    <!-- Vị trí khi app đang mở. Người dùng có thể chỉ cấp "gần đúng". -->
    <uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION"/>
    <uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"/>

    <!-- Foreground service chia sẻ vị trí trong chuyến (Android 14 trở lên bắt
         buộc khai quyền theo loại: loại `location` cần FOREGROUND_SERVICE_LOCATION).
         Service của geolocator (GeolocatorLocationService) khai sẵn
         foregroundServiceType="location" trong manifest của plugin. -->
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE"/>
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_LOCATION"/>

    <!-- Từ Android 13, thông báo cần quyền lúc chạy. Không cần quyền này để chạy
         foreground service, nhưng nếu người dùng từ chối thì thông báo của
         service không hiện trong khay thông báo (vẫn hiện trong Task Manager). -->
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS"/>

    <!-- CỐ Ý KHÔNG khai ACCESS_BACKGROUND_LOCATION: service khởi động lúc tài
         xế vừa bấm nhận chuyến (activity đang hiện). Tài liệu Android chỉ đòi
         quyền này khi tạo service loại location lúc app đã ở nền. Lưu ý: Google
         Play vẫn có thể coi việc chia sẻ vị trí bằng foreground service là
         tương đương vị trí nền, nên đừng coi việc bỏ quyền là thoát thủ tục. -->
```

README `geolocator` nói plugin đã khai `FOREGROUND_SERVICE_LOCATION`, nhưng manifest của `geolocator_android` 5.2.1 không có; ta khai trong app.

## 6. Chính sách store và lý do bị từ chối

Các điều dưới đây tra tại 11/2026; kiểm lại lúc phát hành.

**Google Play** có hai thủ tục, và bỏ `ACCESS_BACKGROUND_LOCATION` không đảm bảo thoát cả hai:

- Khai `ACCESS_BACKGROUND_LOCATION` thì phải điền biểu mẫu vị trí trong App content: chỉ **một** tính năng cần vị trí nền, giải thích vì sao thiếu nó thì không chạy, kèm video (khuyến nghị tối đa 30 giây) cho thấy tính năng chạy ở nền cùng hộp thoại quyền. Trước hộp thoại phải có lời giải thích trong app, không chỉ ghi trong chính sách riêng tư.
- App nhắm Android 14 trở lên phải khai loại foreground service kèm video. Với `location`, use case "Background Location Updates" có mục navigation, ví dụ "ride tracking for ride share"; video phải cho thấy người dùng bắt đầu theo dõi chuyến.
- Chính sách vị trí nền còn đòi foreground service tiếp nối một hành động của người dùng trong app và dừng ngay khi xong; nếu dùng vị trí qua foreground service mà tương đương vị trí nền thì vẫn chịu yêu cầu vị trí nền. Trang này nêu theo dõi giao hàng hoặc chuyến "cho người dùng (không phải tài xế)" là ví dụ truy cập foreground, nên với app tài xế hãy chuẩn bị biểu mẫu và video như thể đã khai quyền (suy luận của bài, hỏi lại Play Console).

**Apple** (App Review Guidelines): điều 5.1.5 chỉ cho dùng vị trí khi liên quan trực tiếp đến tính năng của app, và phải xin đồng ý trước khi thu thập. Điều 2.5.4 cho dịch vụ nền chạy đúng mục đích, trong đó có vị trí. Điều 5.1.1 đòi mô tả mục đích rõ ràng đầy đủ (ii), chỉ xin dữ liệu liên quan chức năng cốt lõi (iii), không ép người dùng đồng ý (iv). Điều 4.5.4 là về push notification (bài 37), không phải vị trí nền.

Lý do bị từ chối hay gặp, suy ra từ các điều trên chứ không phải thống kê của store:

- Mô tả mục đích chung chung, không nói rõ ngoài chuyến app không thu.
- Khai background mode `location` mà người duyệt không thấy tính năng nào cần tới, hoặc khai hơn một tính năng cho vị trí nền trên Play.
- Thu vị trí cả ngoài chuyến, hoặc xin quyền nền ngay màn hình đầu (5.1.5, 5.1.1 iii và iv).
- `ACCESS_BACKGROUND_LOCATION` bị plugin kéo vào mà không ai biết (tài liệu Android tính vị trí do SDK truy cập cho app): đọc merged manifest sau mỗi lần thêm plugin; lab chưa build nên chưa kiểm.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp với việc nền</span>
<ul>
<li><strong>iOS từ chối task.</strong> Dấu hiệu: <code>BGTaskSchedulerErrorDomain</code> code 3 (<code>notPermitted</code>). Nguyên nhân: thiếu mode <code>fetch</code> trong <code>UIBackgroundModes</code>, hoặc <code>uniqueName</code> không có trong <code>BGTaskSchedulerPermittedIdentifiers</code> (kể cả sai một ký tự). Cách sửa: một hằng cho Dart và plist, test đối chiếu.</li>
<li><strong>Tưởng task nền chung bộ nhớ với giao diện.</strong> Dấu hiệu: task thấy bộ đệm trống. Nguyên nhân: task thường chạy ở isolate khác. Cách sửa: ghi ra file hoặc database, như <code>FileBufferStore</code>.</li>
<li><strong>Tạo foreground service location khi app đã ở nền.</strong> Dấu hiệu: <code>SecurityException</code> (Android 14 trở lên), dòng "Foreground service started from background" trong Logcat, hoặc <code>ForegroundServiceStartNotAllowedException</code> (từ Android 12, app ở nền không được khởi động foreground service trừ các trường hợp miễn). Cách sửa: bắt đầu khi activity đang hiện, hoặc từ tương tác với thông báo hay FCM ưu tiên cao. Thêm <code>ACCESS_BACKGROUND_LOCATION</code> chưa đủ vì còn phải thuộc diện miễn, và nó kéo theo thủ tục Play.</li>
<li><strong>Quên dừng luồng khi hết chuyến.</strong> Dấu hiệu: thông báo "đang chia sẻ vị trí" và chỉ báo xanh dương còn sau <code>completed</code>. Nguyên nhân: subscription chưa huỷ. Cách sửa: để controller huỷ subscription và dọn bộ đệm khi chính sách trả <code>false</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/38-background-work
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` sạch, 106 test xanh. Mức: **Kiểm chứng trên thiết bị (TB)**; ở đây chỉ chạy được logic và cấu hình.

- **Chạy tự động, xanh:** chính sách trên tám trạng thái, bộ đệm, bộ gửi, bộ điều khiển với tracker giả, việc của task, `native-config/` của hai app và các điều đọc từ source plugin đã pin.
- **Chỉ biên dịch:** `WorkmanagerScheduler`, `callbackDispatcher`, `GeolocatorLocationTracker` với stream thật, `main.dart`.
- **Chưa chạy:** foreground service và thông báo, chỉ báo xanh dương, task `workmanager` thật, `AppDelegate.swift`, merged manifest, thủ tục Play và App Review.
- **Chưa làm:** gửi thật qua `toi_noi_api` (sink chỉ ghi log), xin quyền lúc chạy, màn hình giải thích quyền, server khử trùng mẫu. README của lab hướng dẫn thử trên máy thật.

## Bài tập

1. `toi_noi_core` thêm trạng thái `waitingAtPickup` giữa `arriving` và `pickedUp`. `dart analyze` báo gì và ta làm gì?
2. Mất mạng 20 phút giữa chuyến, `capacity` 120, mỗi 5 giây một mẫu. Khi có mạng, server nhận bao nhiêu mẫu?
3. Sản phẩm muốn thu vị trí cả lúc tài xế online chờ cuốc, kể cả khi app ở nền. Chính sách hiện tại trả gì và store đòi thêm gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> <code>tracksStatus</code> là <code>switch</code> vét cạn nên <code>dart analyze</code> báo <code>non_exhaustive_switch_expression</code>. Ta quyết định trạng thái mới có được theo dõi không (nhiều khả năng <code>true</code>) và thêm nó vào bảng đặc tả của test, nếu không test "phủ đủ tám trạng thái" đỏ.</p>
<p><strong>2.</strong> 240 mẫu sinh ra nhưng bộ đệm chỉ giữ 120 mẫu cuối, đúng 10 phút gần nhất; 120 mẫu cũ bị bỏ và <code>droppedCount</code> ghi 120.</p>
<p><strong>3.</strong> Trả <code>false</code>, vì <code>searching</code> không nằm trong bốn trạng thái. Cho phép thì mô tả quyền không còn là "chỉ trong chuyến", phải qua 5.1.5 và 5.1.1 (iii) của Apple và khai trên Play. Rẻ hơn: chỉ thu khi app đang mở.</p>
</details>

Bài trước: [Push notification: FCM, APNs, local notification và deep link từ thông báo](/articles/flutter-push-notification-fcm-apns-deep-link). Bài tiếp: [Thanh toán qua VíXanh: WebView, deep link quay về và idempotency key](/articles/flutter-thanh-toan-vi-xanh-webview-deep-link-idempotency).

## Tài liệu gốc

- [Choose the right background task](https://developer.android.com/guide/background), [Define work requests](https://developer.android.com/develop/background-work/background-tasks/persistent/getting-started/define-work) (developer.android.com).
- [Foreground service types](https://developer.android.com/develop/background-work/services/fgs/service-types), [Launch a foreground service](https://developer.android.com/develop/background-work/services/fgs/launch) và [Restrictions on starting foreground services from the background](https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start) (developer.android.com).
- [Notification runtime permission](https://developer.android.com/develop/ui/views/notifications/notification-permission) và [Access location in the background](https://developer.android.com/develop/sensors-and-location/location/background) (developer.android.com).
- [Understanding location in the background permissions](https://support.google.com/googleplay/android-developer/answer/9799150) và [Understanding foreground service and full-screen intent requirements](https://support.google.com/googleplay/android-developer/answer/13392821) (support.google.com/googleplay).
- [App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/), [BGTaskScheduler](https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler) và [Handling location updates in the background](https://developer.apple.com/documentation/corelocation/handling-location-updates-in-the-background) (developer.apple.com).
- [workmanager](https://pub.dev/packages/workmanager) (pub.dev) và [tài liệu của workmanager](https://docs.page/fluttercommunity/flutter_workmanager/quickstart).
- [geolocator](https://pub.dev/packages/geolocator) (pub.dev).

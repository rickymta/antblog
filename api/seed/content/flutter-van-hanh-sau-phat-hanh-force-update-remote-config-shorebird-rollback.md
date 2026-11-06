---
slug: flutter-van-hanh-sau-phat-hanh-force-update-remote-config-shorebird-rollback
title: "Vận hành sau phát hành: force update, remote config, Shorebird và rollback cho Tới Nơi"
summary: "Bài 59: store không có nút lùi, nên ta dựng sẵn force update fail open, remote config với kill switch, đối chiếu Shorebird theo tài liệu, và runbook halt hay pause rollout theo crash-free. Test tự động cho logic; chưa chạy với store, Firebase hay Shorebird thật. Kết Phần 7."
category: programming
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, android, release, remote-config, shorebird]
language: vi
---

Chín giờ sáng, bản 1.6.0 của app khách Tới Nơi lên 10% người dùng Android. Mười một giờ, kênh hỗ trợ nhận nhiều tin giống nhau: bấm chọn VíXanh là app đóng. Crash-free sessions thấp hơn hẳn bản 1.5.3. Phản xạ đầu tiên là "lùi về bản trước", và ta khựng lại: cả hai cửa hàng đều không có nút hạ máy đã cập nhật về bản cũ.

Bài này dựng những thứ phải có trước sự cố: kiểm phiên bản tối thiểu để buộc bản lỗi phải cập nhật, remote config với kill switch, Shorebird như lựa chọn sửa code Dart không qua store, và runbook halt hay pause theo crash-free.

Đây là bài 59 của lộ trình **Flutter qua Tới Nơi**, bài kết Phần 7. Ta nên đọc [bài 58 về App Store production](/articles/flutter-app-store-production-metadata-privacy-review-phased-release) trước: phased release ở đó là công cụ ta dùng lại. Nói thẳng về kiểm chứng: lab không có tài khoản store, Firebase hay Shorebird; logic được test tự động, hai lớp bọc plugin chỉ qua `flutter analyze`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Server chính sách phiên bản của Tới Nơi sập đúng lúc ta phát bản 1.6.0. Một người mở app bản 1.3.0, thấp hơn mức tối thiểu 1.4.0 từng đặt. App nên chặn hay cho vào?</p>
<details><summary>Xem đáp án</summary><p>Cho vào. App không đọc được chính sách, tức không có bằng chứng nào để chặn. Chặn khi không biết sẽ biến sự cố của server thành sự cố của mọi người dùng; cái giá là bản quá cũ chạy thêm một lúc.</p></details>
</div>

## Store không có nút lùi

Theo tài liệu Google Play, staged rollout có nút **Halt rollout**: không thêm ai nhận bản đó, người đã nhận ở nguyên bản đó. Bản đã lên 100% cũng halt được: bản trước từng phát đủ 100% tự thay thế cho người mới và người chưa có bản bị halt. Tài liệu cảnh báo halt kém tác dụng khi phần lớn người dùng đã cập nhật, và không nói máy đã chạy bản bị halt được hạ cấp.

Muốn quay về code cũ, ta phát release mới có `versionCode` cao hơn mà chứa code 1.5.3 (hotfix dạng revert). Theo developer.android.com, không tải lên được APK có `versionCode` đã dùng, và hệ thống chặn cài bản thấp hơn đè lên bản đang có, nên dựng lại file 1.5.3 với số cũ không dùng được.

App Store tương tự. Phased release (bài 58) cho tạm dừng tổng cộng tối đa 30 ngày. Nó chỉ áp cho cập nhật tự động: bản đang phát dần vẫn tải tay được, nên pause không chặn người tự bấm cập nhật.

Vì vậy "rollback" thực tế là bốn biện pháp (Hình 1); ta dùng biện pháp nhanh nhất còn đủ để sửa.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. Bốn biện pháp khi bản lỗi đã lên store, xếp từ trái sang phải theo phạm vi sửa rộng dần và tốc độ chậm dần: kill switch tắt tính năng có cờ, halt hoặc pause rollout chặn bản lỗi lan thêm, Shorebird patch chỉ đổi code Dart và là tuỳ chọn, hotfix bản mới sửa được mọi thứ nhưng chậm nhất."><defs><marker id="fl59-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="10" y="22" class="t-small">Phạm vi sửa rộng dần, tốc độ tới người dùng chậm dần</text>
<path d="M10 40 L866 40" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-1-a)"/>
<rect x="10" y="60" width="188" height="180" rx="10" class="f-pk-soft s-pk"/><text x="26" y="90" class="t-b">1. Kill switch</text><text x="26" y="118" class="t-small">Sửa được:</text><text x="26" y="136">tính năng có cờ</text><text x="26" y="166" class="t-small">Giới hạn:</text><text x="26" y="184">cờ phải có từ</text><text x="26" y="202">trước sự cố</text>
<rect x="234" y="60" width="188" height="180" rx="10" class="f-phys-soft s-phys"/><text x="250" y="90" class="t-b">2. Halt / pause</text><text x="250" y="118" class="t-small">Sửa được:</text><text x="250" y="136">chặn bản lỗi lan</text><text x="250" y="166" class="t-small">Giới hạn:</text><text x="250" y="184">máy đã cập nhật</text><text x="250" y="202">vẫn giữ bản lỗi;</text><text x="250" y="220">iOS: vẫn tải tay</text>
<rect x="458" y="60" width="188" height="180" rx="10" class="f-surface s-muted" stroke-dasharray="8 6"/><text x="474" y="90" class="t-b">3. Shorebird</text><text x="474" y="118" class="t-small">Sửa được:</text><text x="474" y="136">chỉ code Dart</text><text x="474" y="166" class="t-small">Giới hạn:</text><text x="474" y="184">không đổi native,</text><text x="474" y="202">là lựa chọn tuỳ ý</text>
<rect x="682" y="60" width="188" height="180" rx="10" class="f-warn-soft s-warn"/><text x="698" y="90" class="t-b">4. Hotfix 1.6.1</text><text x="698" y="118" class="t-small">Sửa được:</text><text x="698" y="136">mọi thứ</text><text x="698" y="166" class="t-small">Giới hạn:</text><text x="698" y="184">qua review và</text><text x="698" y="202">rollout, chậm nhất</text>
</svg>
</div>
<figcaption>Hình 1. Bốn biện pháp sau khi bản lỗi đã ra store. Không biện pháp nào hạ máy đã cập nhật về bản cũ; ô nét đứt là lựa chọn không bắt buộc.</figcaption>
</figure>

Hotfix luôn dùng được nên là chốt cuối; khi nó lên, ta có thể nâng `minSupported` để bản lỗi buộc cập nhật. Force update phải có sẵn trong app từ trước, nên ta bắt đầu từ đó.

## Force update: chính sách, so phiên bản và fail open

Tài liệu Android nói hệ thống chỉ dùng thông tin phiên bản để chống hạ cấp; mọi giới hạn phiên bản do app tự thực thi và tự báo người dùng. Ta áp cùng cách cho iOS. Server (BFF) trả JSON như `native-config/min-version.example.json`: mỗi nền tảng có `minSupported`, `recommended` và `latestPublished`, tách riêng vì `build` của hai bên khác nhau (bài [54](/articles/flutter-ci-cd-github-actions-macos-build-ios-android-secrets)).

`AppVersion` đọc `x.y.z+build` và so lần lượt major, minor, patch, build, bằng số chứ không bằng chuỗi (1.9.0 nhỏ hơn 1.10.0). Bản `1.4.0` không ghi build là build 0, tức mốc "từ 1.4.0 trở lên". Phép quyết định là hàm thuần:

### min_version_policy.dart · 59-post-release-ops/lib/min_version_policy.dart (trích)

```dart
static UpdateDecision evaluate({
  required AppVersion current,
  required StorePlatform platform,
  required VersionPolicyConfig config,
}) {
  final rule = config.rules[platform];
  if (rule == null) return const UpdateOk(OkReason.noRuleForPlatform);

  if (current < rule.minSupported) {
    return UpdateForce(
      minSupported: rule.minSupported,
      message: rule.message ?? defaultForceMessage,
      storeUri: rule.storeUri,
    );
  }
  final recommended = rule.recommended;
  if (recommended != null && current < recommended) {
    return UpdateRecommend(
      recommended: recommended,
      message: rule.message ?? defaultRecommendMessage,
      storeUri: rule.storeUri,
    );
  }
  return const UpdateOk(OkReason.upToDate);
}
```

Gõ nhầm `minSupported` thành `9.0.0` là khoá cả người dùng. Vì thế `PlatformRule` từ chối cấu hình tự mâu thuẫn, và `latestPublished` (bản đã có trên store) là trần của mọi mức khác:

### min_version_policy.dart · 59-post-release-ops/lib/min_version_policy.dart (trích)

```dart
// Hai ràng buộc chống cấu hình sai: không đòi phiên bản chưa có trên
// cửa hàng, và thứ tự min <= recommended <= latest.
final rec = recommended;
if (minSupported > latestPublished) {
  throw FormatException(
    'minSupported $minSupported lớn hơn latestPublished $latestPublished',
  );
}
if (rec != null && (rec < minSupported || rec > latestPublished)) {
  throw FormatException(
    'recommended $rec phải nằm trong [$minSupported, $latestPublished]',
  );
}
```

Cùng nguyên tắc đó dẫn tới **fail open** (Hình 2). `UpdateGate` hỏi server với thời gian chờ 3 giây. Lỗi mạng, hết giờ, JSON hỏng, cấu hình mâu thuẫn, kể cả `Error` do bug: tất cả cho `UpdateOk`. Chỉ chính sách hợp lệ mới có quyền chặn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Luồng kiểm phiên bản lúc mở app. Mở app rồi hỏi server, chờ tối đa 3 giây, người dùng đã thấy app. Nếu chính sách đọc được và hợp lệ thì so phiên bản ra ba kết quả: force hiện màn hình chặn, recommend hiện banner, ok vào app. Nếu lỗi mạng, hết giờ, JSON hỏng hoặc min lớn hơn latest thì vào app theo nguyên tắc fail open."><defs><marker id="fl59-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl59-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="10" y="110" width="140" height="60" rx="10" class="f-phys-soft s-phys"/><text x="26" y="136" class="t-b">Mở app</text><text x="26" y="156" class="t-small">đang chạy 1.3.0</text>
<path d="M152 140 L168 140" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-2-a)"/>
<rect x="170" y="100" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="186" y="126" class="t-b">Hỏi server</text><text x="186" y="148" class="t-small">chờ tối đa 3 giây</text><text x="186" y="166" class="t-small">app đã hiện sẵn</text>
<path d="M362 125 L385 125 L385 70 L408 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-2-a)"/><text x="318" y="92" class="t-small">hợp lệ</text>
<rect x="410" y="30" width="260" height="80" rx="10" class="f-pk-soft s-pk"/><text x="426" y="58" class="t-b">Chính sách đọc được</text><text x="426" y="80" class="t-small">min ≤ recommended ≤ latest</text><text x="426" y="98" class="t-small">so với phiên bản đang chạy</text>
<path d="M672 70 L680 70 M680 32 L680 136" class="f-none s-pk" stroke-width="2"/>
<path d="M680 32 L698 32" class="f-none s-warn" stroke-width="2" marker-end="url(#fl59-2-w)"/><path d="M680 84 L698 84" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-2-a)"/><path d="M680 136 L698 136" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-2-a)"/>
<rect x="700" y="10" width="170" height="44" rx="8" class="f-warn-soft s-warn"/><text x="714" y="37" class="t-b">force: chặn</text>
<rect x="700" y="62" width="170" height="44" rx="8" class="f-pk-soft s-pk"/><text x="714" y="89">recommend: banner</text>
<rect x="700" y="114" width="170" height="44" rx="8" class="f-phys-soft s-phys"/><text x="714" y="141">ok: vào app</text>
<path d="M362 155 L385 155 L385 230 L408 230" class="f-none s-warn" stroke-width="2" marker-end="url(#fl59-2-w)"/><text x="318" y="210" class="t-small t-warn">lỗi</text>
<rect x="410" y="190" width="260" height="80" rx="10" class="f-warn-soft s-warn"/><text x="426" y="218" class="t-b">Không có chính sách hợp lệ</text><text x="426" y="240" class="t-small">lỗi mạng, hết 3 giây,</text><text x="426" y="258" class="t-small">JSON hỏng, min lớn hơn latest</text>
<path d="M672 230 L698 230" class="f-none s-warn" stroke-width="2" marker-end="url(#fl59-2-w)"/>
<rect x="700" y="208" width="170" height="44" rx="8" class="f-warn-soft s-warn"/><text x="714" y="227" class="t-b">vào app</text><text x="714" y="244" class="t-small">fail open</text>
</svg>
</div>
<figcaption>Hình 2. Hai nhánh sau khi hỏi server. Nhánh trên là nơi duy nhất có thể chặn người dùng; nhánh dưới luôn cho vào.</figcaption>
</figure>

### update_gate.dart · 59-post-release-ops/lib/update_gate.dart (trích)

```dart
Future<UpdateDecision> check({
  required AppVersion current,
  required StorePlatform platform,
}) async {
  final VersionPolicyConfig config;
  try {
    final raw = await _source.fetchRaw().timeout(timeout);
    config = VersionPolicyConfig.fromJson(raw);
  } on Object {
    return const UpdateOk(OkReason.serverUnavailable);
  }

  final decision = MinVersionPolicy.evaluate(
    current: current,
    platform: platform,
    config: config,
  );
  if (decision is! UpdateRecommend) return decision;

  if (!await _throttle.shouldRemind(decision.recommended)) {
    return const UpdateOk(OkReason.throttled);
  }
  // Ghi nhận ngay khi trả về: từ lúc này ta coi như người dùng đã được nhắc.
  await _throttle.markShown(decision.recommended);
  return decision;
}
```

`UpdateGuard` cho người dùng thấy `child` lúc chờ và khi gate trả `UpdateOk`; chỉ `UpdateForce` mới thay bằng màn hình chặn. Ta đặt nó ở `MaterialApp.builder` để bao cả Navigator. Đặt ở `home:` thì route push trong lúc chờ chính sách sẽ nằm trên guard, và màn hình chặn nằm sau route đó (test có ca đối chứng).

### update_guard_test.dart · 59-post-release-ops/test/update_guard_test.dart (trích)

```dart
// Cách dùng đúng: guard bao cả Navigator qua MaterialApp.builder.
Widget app(PolicySource source, String current) => MaterialApp(
  builder: (context, child) => guard(source, current, child!),
  home: const Scaffold(body: Text('Màn hình chính')),
);
```

Navigator bị gỡ nên không còn route nào để Back đóng: framework chuyển Back cho hệ thống (test bắt được `SystemNavigator.pop`) và người dùng không vào được app. `PopScope(canPop: false)` của màn hình chỉ còn tác dụng khi nó được push như route. Đích của nút lấy từ server, nên `StoreLink.resolve` chỉ nhận link `https` tới đúng máy chủ cửa hàng, còn lại bị thay bằng link dựng sẵn.

### force_update_screen.dart · 59-post-release-ops/lib/force_update_screen.dart (trích)

```dart
return PopScope(
  canPop: false,
  child: Scaffold(
    body: SafeArea(
```

```dart
Future<void> _openStore(BuildContext context) async {
  final messenger = ScaffoldMessenger.of(context);
  var opened = false;
  try {
    opened = await opener.open(storeUri);
  } on Object {
    opened = false;
  }
  if (!opened) {
    messenger.showSnackBar(const SnackBar(content: Text(openFailedText)));
  }
}
```

Mức recommend mà nhắc quá tay thì người dùng gỡ app. `ReminderThrottle` chỉ nhắc khi chưa từng nhắc, đã qua 3 ngày, hoặc `recommended` sang bản khác. Đồng hồ lùi thì nhắc; bộ nhớ lỗi thì im lặng. Mức force không qua throttle.

### reminder_throttle.dart · 59-post-release-ops/lib/reminder_throttle.dart (trích)

```dart
Future<bool> shouldRemind(AppVersion recommended) async {
  final ({DateTime shownAt, AppVersion version})? last;
  try {
    last = await _store.read();
  } on Object {
    // Không đọc được bộ nhớ: im lặng còn hơn nhắc ở mỗi lần mở app.
    return false;
  }
  if (last == null || last.version != recommended) return true;
  final now = _clock();
  // Đồng hồ máy lùi về trước lần nhắc cuối: coi như đã hết hạn thay vì
  // im lặng vô thời hạn.
  if (now.isBefore(last.shownAt)) return true;
  return now.difference(last.shownAt) >= interval;
}
```

## Remote config và kill switch

Remote config đổi hành vi của binary đang chạy, là biện pháp 1 ở Hình 1. Ba quy tắc, đều nằm ở `FeatureFlags` chứ không ở plugin:

- **Mặc định thuộc về app.** Đọc source `firebase_remote_config` 7.0.1, `getBool` trả `false` cho khoá không tồn tại, nên cờ mặc định `true` đọc thẳng bằng SDK sẽ tắt khi mạng lỗi và cache trống. `setDefaults` là cách chuẩn của SDK; ta vẫn đặt mặc định trong `FlagDef` để nó nằm cạnh kiểu, khoảng hợp lệ và test được không cần plugin. Lớp bọc chỉ lấy giá trị có nguồn `ValueSource.valueRemote`.
- **Kiểm giá trị.** `asBool` của SDK nhận cả `1`; `FlagDef.boolean` chỉ nhận `true` và `false`. Số ngoài khoảng bị bỏ.
- **Kill switch áp ngay, cờ thường chờ lần mở sau.** `load()` đọc cache lúc khởi động nên cả phiên dùng một bộ giá trị ổn định. Tài liệu Firebase có chiến lược nạp giá trị mới cho lần khởi động tiếp theo, và cảnh báo đừng kích hoạt ngay khi UI có thể đổi lúc người dùng đang dùng.

Mặc định an toàn: tính năng mới tắt, chức năng đang chạy bật.

### feature_flags.dart · 59-post-release-ops/lib/feature_flags.dart (trích)

```dart
static final vixanhPayment = FlagDef.boolean(
  'payment_vixanh_enabled',
  defaultValue: true,
  killSwitch: true,
);

/// Tính năng mới, chưa phát cho mọi người.
static final newRouting = FlagDef.boolean(
  'driver_new_routing_enabled',
  defaultValue: false,
);
```

`refresh()` không ném lỗi. Nó kiểm giá trị, ghi cache, rồi chỉ cập nhật ngay các cờ `killSwitch` (Hình 3); khoá bị server gỡ thì kill switch về mặc định ngay.

### feature_flags.dart · 59-post-release-ops/lib/feature_flags.dart (trích)

```dart
Future<bool> refresh() async {
  final Map<String, Object?> fetched;
  try {
    fetched = await _source.fetch();
  } on Object {
    return false;
  }
  final valid = _validated(fetched);
  try {
    await _cache.write(valid);
  } on Object {
    // Không ghi được cache: lần mở sau dùng mặc định hoặc cache cũ.
  }
  for (final flag in _catalog.where((f) => f.killSwitch)) {
    final next = valid[flag.key];
    if (next == null) {
      _active.remove(flag.key);
    } else {
      _active[flag.key] = next;
    }
  }
  return true;
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Đường đi của một giá trị cờ. Server Remote Config trả giá trị, fetch và kiểm giá trị, ghi vào cache. Cờ kill switch có hiệu lực ngay trong phiên. Cờ thường có hiệu lực ở lần mở app sau, khi load đọc cache. Khoá thiếu, giá trị hỏng hoặc chưa tải được thì dùng mặc định trong FlagDef."><defs><marker id="fl59-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="10" y="120" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="26" y="148" class="t-b">Server</text><text x="26" y="168" class="t-small">Remote Config</text>
<path d="M162 155 L218 155" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-3-a)"/>
<rect x="220" y="105" width="210" height="100" rx="10" class="f-pk-soft s-pk"/><text x="236" y="130" class="t-b">fetch và kiểm giá trị</text><text x="236" y="152" class="t-small">bool: true hoặc false</text><text x="236" y="170" class="t-small">int: nằm trong khoảng</text><text x="236" y="188" class="t-small">khoá lạ bị bỏ</text>
<path d="M432 155 L478 155" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-3-a)"/>
<rect x="480" y="120" width="140" height="70" rx="10" class="f-pk-soft s-pk"/><text x="496" y="148" class="t-b">Cache</text><text x="496" y="168" class="t-small">giá trị hợp lệ</text>
<path d="M325 103 L325 60 L678 60" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-3-a)"/><text x="340" y="52" class="t-small">cờ killSwitch</text>
<rect x="680" y="20" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="696" y="50" class="t-b">Kill switch</text><text x="696" y="72" class="t-small">áp ngay trong phiên</text>
<path d="M545 192 L545 240 L678 240" class="f-none s-pk" stroke-width="2" marker-end="url(#fl59-3-a)"/><text x="560" y="232" class="t-small">lần mở sau</text>
<rect x="680" y="200" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="696" y="230" class="t-b">Cờ thường</text><text x="696" y="252" class="t-small">áp ở lần mở app sau</text><text x="696" y="268" class="t-small">(load đọc cache)</text>
<rect x="10" y="230" width="400" height="50" rx="10" class="f-surface s-muted"/><text x="26" y="251" class="t-small">Khoá thiếu, giá trị hỏng hoặc chưa tải được:</text><text x="26" y="268" class="t-small">dùng mặc định trong FlagDef (tính năng mới tắt)</text>
</svg>
</div>
<figcaption>Hình 3. Giá trị cờ đi qua kiểm tra rồi vào cache. Chỉ kill switch được áp ngay; thiếu dữ liệu thì rơi về mặc định của app.</figcaption>
</figure>

`firebase_remote_config` chỉ có mặt ở lớp bọc sau interface `RemoteFlagSource`, mới kiểm bằng `flutter analyze`, chưa fetch thật:

### firebase_flag_source.dart · 59-post-release-ops/lib/firebase_flag_source.dart (trích)

```dart
Future<Map<String, Object?>> fetch() async {
  await _config.fetchAndActivate();
  return {
    for (final entry in _config.getAll().entries)
      if (entry.value.source == ValueSource.valueRemote)
        entry.key: entry.value.asString(),
  };
}
```

Firebase giới hạn tần suất fetch bằng `minimumFetchInterval`, nên `refresh()` chưa phải kênh khẩn cấp tức thì; cập nhật thời gian thực thì không bị giới hạn này (plugin có `onConfigUpdated`).

## Shorebird: sửa code Dart không qua store

Shorebird là dịch vụ thương mại của bên thứ ba cho code push của Flutter. Với loạt bài này nó là lựa chọn không bắt buộc; phần này chỉ nêu điều tài liệu của họ nói, kiểm tại 11/2026.

- Patch chỉ đổi code Dart. Native, asset và phiên bản Flutter cần release mới qua store.
- Mỗi patch gắn với đúng một release version (`--release-version`); `shorebird patch` cảnh báo hoặc chặn mặc định khi thấy khác biệt native hay asset so với release đã lưu.
- Patch tải ở nền, áp ở lần khởi động sau, chỉ có tác dụng ở bản release.
- `shorebird patches rollback` gỡ patch: máy chỉ biết khi kiểm tra cập nhật, rồi quay về patch tốt trước đó hoặc bản gốc ở lần khởi động sau.
- Có gói Hobby miễn phí, phí tính theo số lần patch được cài. Hết hạn mức thì Shorebird từ chối kiểm tra patch: máy không nhận cả patch mới lẫn rollback. Bài này không kiểm giá.

Về chính sách store, ta đọc tài liệu của Apple và Google thay vì tin lời ai. Guideline 2.5.2 của App Review cấm app tải, cài hay thực thi code làm thêm hoặc đổi tính năng. Điều 3.3.1(B) của Apple Developer Program License Agreement cho tải interpreted code nếu không đổi mục đích chính của app, không vượt cơ chế ký hay sandbox, không lập cửa hàng. Chính sách Device and Network Abuse của Google Play cấm app tự cập nhật ngoài cơ chế của Play và cấm tải executable code (dex, JAR, `.so`), ngoại lệ cho code chạy trong máy ảo hoặc interpreter chỉ cho truy cập gián tiếp tới Android API. Shorebird cho rằng cách họ làm nằm trong các điều kiện đó; đó là lời nhà cung cấp, không phải của store. Hãy kiểm lại tại thời điểm phát hành.

Với Tới Nơi, cách dùng thận trọng là patch lỗi logic Dart nhỏ, không đổi mục đích app; tính năng mới vẫn qua review. Theo tài liệu, `shorebird.yaml` do `shorebird init` sinh ra, được commit, `app_id` không phải bí mật và `app_id` cấp trên mặc định là của flavor đầu tiên. Mỗi flavor có một `app_id`:

### shorebird.yaml.example · 59-post-release-ops/native-config/shorebird.yaml.example

```yaml
app_id: 00000000-0000-0000-0000-000000000001
flavors:
  dev: 00000000-0000-0000-0000-000000000001
  staging: 00000000-0000-0000-0000-000000000002
  prod: 00000000-0000-0000-0000-000000000003
```

Nếu có dùng, CI cần một bước chọn đường. `PatchPreflight` phân loại file đổi theo đường dẫn, bảo thủ, và không thay công cụ vì chỉ `shorebird patch` so được artifact thật:

### patch_preflight.dart · 59-post-release-ops/lib/patch_preflight.dart (trích)

```dart
static PatchVerdict _classify(String path) {
  final segments = path.split('/');
  const native = {'android', 'ios', 'macos', 'linux', 'windows'};
  if (segments.any(native.contains)) return PatchVerdict.needsStoreRelease;
  final isDart =
      path.endsWith('.dart') &&
      (segments.contains('lib') || segments.contains('test'));
  return isDart ? PatchVerdict.patchCandidate : PatchVerdict.needsReview;
}
```

## Runbook sự cố: phát hiện, quyết định, thông báo

Phát hiện bắt đầu từ crash-free. Tài liệu Firebase mô tả crash-free users và crash-free sessions, chỉ tính sự cố fatal; chỉ số theo người dùng thường nhỏ hơn khi cửa sổ dài hơn. Vì vậy ta so bản mới với bản trước **trong cùng cửa sổ**.

Quyết định là một hàm thuần, đề xuất nhanh và nhất quán lúc nửa đêm; người phát hành vẫn là người quyết. `RolloutGuard.decide` nhận crash-free bằng điểm cơ bản (9.810 là 98,10%) để so bằng số nguyên. Ngưỡng là số minh hoạ, mỗi đội tự hiệu chỉnh. Mức giảm chỉ xét khi đủ 2.000 phiên: giảm từ 0,30 điểm thì giữ nguyên, từ 1,00 điểm thì dừng. Sàn tuyệt đối 97,00% xét sớm hơn, từ 300 phiên.

### rollout_guard.dart · 59-post-release-ops/lib/rollout_guard.dart (trích)

```dart
static RolloutDecision decide(
  RolloutStats stats, [
  RolloutThresholds t = const RolloutThresholds(),
]) {
  final isAndroid = stats.platform == StorePlatform.android;
  // iOS không có "giữ nguyên %": phased release chạy theo ngày, chỉ có tạm dừng.
  final hold = isAndroid
      ? StoreAction.holdAndroidPercent
      : StoreAction.continueRollout;
  final pause = isAndroid
      ? StoreAction.holdAndroidPercent
      : StoreAction.pauseIosPhasedRelease;
  final halt = isAndroid
      ? StoreAction.haltAndroidRollout
      : StoreAction.pauseIosPhasedRelease;

  if (stats.sessions < t.floorMinSessions) {
    return RolloutDecision(
      RolloutAction.wait,
      hold,
      'Mới ${stats.sessions} phiên, cần ít nhất ${t.floorMinSessions} để xét',
    );
  }
  if (stats.crashFreeBp < t.floorBp) {
    return RolloutDecision(
      RolloutAction.halt,
      halt,
      'Crash-free ${_pct(stats.crashFreeBp)} thấp hơn sàn ${_pct(t.floorBp)}',
    );
  }
  if (stats.sessions < t.minSessions) {
    return RolloutDecision(
      RolloutAction.wait,
      hold,
      'Mới ${stats.sessions} phiên, cần ít nhất ${t.minSessions} để tin mức giảm',
    );
  }
  if (stats.dropBp >= t.haltDropBp) {
    return RolloutDecision(
      RolloutAction.halt,
      halt,
      'Crash-free giảm ${_pts(stats.dropBp)} so với bản trước',
    );
  }
  if (stats.dropBp >= t.pauseDropBp) {
    return RolloutDecision(
      RolloutAction.pause,
      pause,
      'Crash-free giảm ${_pts(stats.dropBp)}, giữ nguyên để điều tra',
    );
  }
  return RolloutDecision(
    RolloutAction.proceed,
    StoreAction.continueRollout,
    'Crash-free ${_pct(stats.crashFreeBp)}, giảm ${_pts(stats.dropBp)}',
  );
}
```

Hành động trên store khác nhau theo nền tảng. Ở Android, staged rollout không tự tăng, nên "giữ nguyên" chỉ là không bấm tăng. iOS không có "giữ nguyên": phased release chạy theo ngày, nên chỉ có tạm dừng. Số minh hoạ: 1.6.0 ở 10% người dùng Android, 4.200 phiên, crash-free 98,10% so với 99,30% của 1.5.3, tức giảm 1,20 điểm.

### rollout_guard_test.dart · 59-post-release-ops/test/rollout_guard_test.dart (trích)

```dart
test('số minh hoạ của bài: 98,10% so với 99,30% là halt', () {
  final d = RolloutGuard.decide(stats(crashFree: 9810));
  expect(d.action, RolloutAction.halt);
  expect(d.reason, contains('1,20 điểm %'));
});
```

Kết quả là `halt` với việc `haltAndroidRollout`, và `RolloutGuard.notice` soạn sẵn thông báo: `[HALT] Tới Nơi 1.6.0 (Android, 10% người dùng, 4200 phiên): Crash-free giảm 1,20 điểm % so với bản trước.` Runbook của ta, sau khi cảnh báo crash-free đã nổ:

| Bước | Việc |
|---|---|
| 1. Quyết định | Chạy `RolloutGuard.decide` cho từng nền tảng. Người phát hành quyết, hàm chỉ đề xuất |
| 2. Chặn lan | Android: Halt rollout. iOS: Pause phased release (người tự cập nhật vẫn nhận bản lỗi). Máy đã cập nhật vẫn giữ bản lỗi |
| 3. Giảm thiệt hại | Tắt kill switch `payment_vixanh_enabled`, nếu cờ đã có từ trước |
| 4. Thông báo | Gửi `RolloutGuard.notice` vào kênh vận hành và cho bộ phận hỗ trợ |
| 5. Sửa | Hotfix từ nhánh `release/1.6`, đặt `1.6.1` với build mới từ CI, phát lại từ tỷ lệ nhỏ. iOS vẫn qua review |
| 6. Dọn | Nâng `minSupported` lên 1.6.1 nếu lỗi đủ nặng, nhưng chỉ khi 1.6.1 đã tới mọi người (Android: 100%; iOS: Release to All Users), kẻo người chưa nhận được bản mới đã bị chặn. Đừng force vì lỗi nhỏ |

## Kết Phần 7

Mục tiêu của Phần 7 là đưa Tới Nơi từ source tới người dùng bằng quy trình lặp lại được, từ ba flavor, ký, CI, phân phối nội bộ, TestFlight và Google Play testing tới production và vận hành sau phát hành. Đối chiếu ngắn với bảng môi trường và kênh phân phối của lộ trình:

| Flavor | Kênh | Bài |
|---|---|---|
| `dev` | `flutter run`, simulator, thiết bị cắm dây | [01](/articles/flutter-va-dart-tu-source-code-den-ung-dung-toi-noi), 25, [50](/articles/flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build) |
| `staging` | App Distribution, TestFlight, Google Play testing | [55](/articles/flutter-firebase-app-distribution-phan-phoi-noi-bo-tester), [56](/articles/flutter-testflight-internal-external-testing-beta-review), [57](/articles/flutter-google-play-internal-closed-open-production-staged-rollout) |
| `prod` | App Store (phased release), Google Play production (staged rollout) | [58](/articles/flutter-app-store-production-metadata-privacy-review-phased-release), 59 |

Dòng `prod` giờ có thêm phần sau phát hành; chuỗi ký và build ở bài [51](/articles/flutter-ky-ung-dung-android-keystore-play-app-signing-aab) tới [54](/articles/flutter-ci-cd-github-actions-macos-build-ios-android-secrets) nằm dưới mọi dòng. Nói thẳng: cả Phần 7 chỉ kiểm ở mức cấu hình, vì repository không có tài khoản store hay chứng chỉ.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp sau phát hành</span>
<ul>
<li><strong>Force update chặn khi không có chính sách.</strong> Dấu hiệu: server sập và mọi người kẹt ở màn hình chặn. Nguyên nhân: coi "không hỏi được" như "phải chặn". Cách sửa: fail open.</li>
<li><strong><code>minSupported</code> vượt bản đã lên store.</strong> Dấu hiệu: cả bản mới nhất cũng bị chặn. Nguyên nhân: gõ nhầm, hoặc nâng khi bản mới chưa tới mọi người. Cách sửa: kiểm <code>min ≤ recommended ≤ latestPublished</code> ở cả server và app.</li>
<li><strong>Đọc cờ bằng <code>getBool</code> mà chưa đặt mặc định <code>true</code>.</strong> Dấu hiệu: tính năng đang chạy tự tắt khi chưa tải được cấu hình. Nguyên nhân: SDK trả <code>false</code> cho khoá vắng mặt. Cách sửa: <code>setDefaults</code>, hoặc mặc định trong app như bài này.</li>
<li><strong>Coi halt như rollback.</strong> Dấu hiệu: đã halt mà người dùng cũ vẫn báo lỗi. Nguyên nhân: máy đã cập nhật vẫn giữ bản lỗi. Cách sửa: kill switch, hotfix, rồi nâng <code>minSupported</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/59-post-release-ops
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3 trên macOS Apple silicon: `flutter analyze` báo `No issues found!`, `flutter test` chạy 161 test xanh. Mức: **Kiểm chứng cấu hình**, phần logic kiểm tự động. Test không dùng mạng, Firebase, thiết bị hay đồng hồ thật.

Test chứng minh: ba mức quyết định kể cả ca biên của build; cấu hình mâu thuẫn bị từ chối; fail open và hết giờ đúng ở 3 giây (đồng hồ giả); throttle ở ranh giới 3 ngày; `builder:` so với `home:` và hành vi Back; kill switch và lỗi cache; ngưỡng `RolloutGuard` kể cả sàn với mẫu nhỏ; file mẫu trong `native-config/` khớp với code.

**Chưa làm**, nói thẳng:

- `FirebaseRemoteFlagSource` chưa fetch với Firebase thật; `UrlLauncherOpener` chưa mở cửa hàng thật; chưa có endpoint chính sách thật.
- Chưa chạy lệnh Shorebird nào, chưa bấm Halt rollout hay Pause phased release; mọi số crash-free là minh hoạ.
- Chưa nhớ chính sách lần trước khi server lỗi, chưa dùng `onConfigUpdated`; `FlagCache` và `ReminderStore` mới có bản trong bộ nhớ.

## Bài tập

1. Chính sách Android: `minSupported 1.4.0+60`, `recommended 1.6.0+87`, `latestPublished 1.6.0+87`. Ba người dùng chạy `1.4.0+59`, `1.4.0+60` và `1.6.1+88` nhận quyết định nào?
2. Server đặt `driver_new_routing_enabled` thành `true` lúc 10:00; chị Lan mở app tài xế từ 9:30 và không tắt. Bao giờ chị thấy tính năng? Nếu là kill switch thì sao?
3. Bản 1.6.0 trên iOS có baseline 99,30%. Số liệu mới: 99,00% với 1.500 phiên, rồi 98,95% với 3.000 phiên. Hàm cho quyết định gì, và việc trên App Store Connect là gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> <code>1.4.0+59</code> là force vì build thấp hơn mức tối thiểu. <code>1.4.0+60</code> không bị chặn nhưng thấp hơn <code>recommended</code> nên được nhắc (nếu throttle cho phép). <code>1.6.1+88</code> mới hơn mọi mốc nên ok.</p>
<p><strong>2.</strong> Cờ thường chỉ đổi ở lần mở app sau khi đã tải giá trị mới, nên chị chưa thấy trong phiên này. Kill switch áp ngay khi <code>refresh()</code> thành công; thời điểm phụ thuộc lúc app gọi <code>refresh()</code> và giới hạn fetch của Firebase.</p>
<p><strong>3.</strong> Lần một mới 1.500 phiên, ít hơn 2.000, nên <code>wait</code>; phased release vẫn chạy, chưa có việc gì làm. Lần hai giảm 35 điểm cơ bản, từ 30 tới dưới 100, nên <code>pause</code>: tạm dừng phased release và điều tra.</p>
</details>

Bài trước: [App Store production: metadata, privacy labels, App Review và phased release](/articles/flutter-app-store-production-metadata-privacy-review-phased-release). Bài tiếp: [CustomPainter và RenderObject: vẽ tuyến đường và đồng hồ cước](/articles/flutter-custom-painter-renderobject-ve-tuyen-duong).

## Tài liệu gốc

- [Version your app](https://developer.android.com/studio/publish/versioning): `versionCode`, chống hạ cấp, app tự thực thi giới hạn phiên bản (developer.android.com).
- [Release app updates with staged rollouts](https://support.google.com/googleplay/android-developer/answer/6346149): halt, resume, giới hạn (support.google.com/googleplay).
- [Device and Network Abuse](https://support.google.com/googleplay/android-developer/answer/9888379): chính sách cập nhật và tải code (support.google.com/googleplay).
- [Release a version update in phases](https://developer.apple.com/help/app-store-connect/update-your-app/release-a-version-update-in-phases/): lịch 7 ngày, tạm dừng 30 ngày (developer.apple.com).
- [App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/): mục 2.5.2 (developer.apple.com).
- [Apple Developer Program License Agreement](https://developer.apple.com/support/terms/apple-developer-program-license-agreement/): mục 3.3.1(B) (developer.apple.com).
- [Remote Config loading strategies](https://firebase.google.com/docs/remote-config/loading) và [Remote Config REST: RemoteConfig](https://firebase.google.com/docs/reference/remote-config/rest/v1/RemoteConfig) (firebase.google.com).
- [Understand crash-free metrics](https://firebase.google.com/docs/crashlytics/crash-free-metrics) (firebase.google.com).
- [firebase_remote_config](https://pub.dev/packages/firebase_remote_config) và [url_launcher](https://pub.dev/packages/url_launcher) (pub.dev).
- [PopScope](https://api.flutter.dev/flutter/widgets/PopScope-class.html) (api.flutter.dev).
- [Shorebird docs](https://docs.shorebird.dev/), [FAQ](https://docs.shorebird.dev/code-push/faq/), [Patch](https://docs.shorebird.dev/code-push/patch/), [Rollback](https://docs.shorebird.dev/code-push/rollback/), [Initialize](https://docs.shorebird.dev/code-push/initialize/) và [Plans](https://docs.shorebird.dev/account/plans/) (docs.shorebird.dev).

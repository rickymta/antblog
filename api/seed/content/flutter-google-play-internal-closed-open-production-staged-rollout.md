---
slug: flutter-google-play-internal-closed-open-production-staged-rollout
title: "Google Play: internal, closed, open testing, production và staged rollout cho Tới Nơi"
summary: "Bài 57: đưa app tài xế Tới Nơi qua bốn track của Google Play. Cổng 12 tester và 14 ngày cho tài khoản cá nhân mới, Data safety khớp manifest, vị trí nền và foreground service, target API, staged rollout, halt và vì sao không có rollback. Chưa chạy trên tài khoản thật."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, android, release, google-play]
language: vi
---

Lane `internal` của bài 53 đã đưa bản 43 của app tài xế lên Google Play, và QA cài được sau vài phút. Nhưng tuần sau bản này phải tới vài chục tài xế thử nghiệm, rồi tới người dùng thật. Tài khoản Play của đội là tài khoản cá nhân mới tạo, và Google có luật riêng cho loại tài khoản đó.

Bài này đi qua bốn track của Play với ba mô hình Dart nhỏ: `RolloutPlan`, `TrackPromotion`, `DataSafetyDraft`. Đây là bài 57 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết TestFlight ở [bài 56](/articles/flutter-testflight-internal-external-testing-beta-review), lane `supply` ở [bài 53](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build), `versionCode` ở [bài 51](/articles/flutter-ky-ung-dung-android-keystore-play-app-signing-aab) và quyền vị trí của app tài xế ở [bài 35](/articles/flutter-vi-tri-geolocator-permission-foreground-background).

Repository này không có tài khoản Play Console, nên **chưa có thao tác nào chạy trên Play thật**. Chính sách đổi theo thời gian: số ghi "kiểm tại 11/2026" lấy từ trang trợ giúp lúc viết, hãy kiểm lại trước khi phát hành. Ngưỡng crash-free và bước phần trăm của rollout là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bản 43 đang phát cho 20% người dùng thì có crash, và ta bấm <strong>Halt rollout</strong>. Máy của những người đã nhận bản 43 sẽ ra sao?</p>
<details><summary>Xem đáp án</summary><p>Vẫn giữ bản 43. Trang trợ giúp về staged rollout ghi rằng halt chỉ ngừng phát cho thêm người, còn người đã nhận vẫn ở bản đó. Halt là cái phanh, không phải nút quay về. Cách sửa là phát bản mới đã sửa lỗi, với <code>versionCode</code> cao hơn.</p></details>
</div>

## Bốn track và ai thấy bản nào

Mỗi track là một nhóm người nhận khác nhau và giữ bản riêng: internal test chạy song song với closed và open cho các phiên bản khác nhau. Bảng sau tóm tắt (kiểm tại 11/2026).

| Track | Ai nhận | Giới hạn | Tới Nơi dùng cho |
|---|---|---|---|
| internal | danh sách email, nhập thủ công hoặc CSV | tới 100 tester mỗi app | QA, PM, flavor staging |
| closed | danh sách email, Google Groups, hoặc tổ chức theo ID | tới 200 danh sách mỗi tài khoản, 50 mỗi track, 2.000 tester mỗi danh sách; Google Groups không giới hạn cỡ | tài xế thử nghiệm |
| open | ai cũng vào được qua trang app hoặc link | không giới hạn (mặc định), hoặc đặt giới hạn tối thiểu 1.000 | tuỳ chọn |
| production | mọi người dùng | không có giới hạn tester | người dùng thật |

Internal nhanh: bản cập nhật tới tester trong vài phút, còn link tham gia của lần xuất bản đầu có thể mất vài giờ mới dùng được. Internal test có thể không chịu rà soát chính sách và bảo mật tiêu chuẩn, trong khi bản trên mọi track đang hoạt động, kể cả closed và open, đều chịu rà soát (trang chính sách vị trí nền). Qua internal chưa có nghĩa là qua cửa Google.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 1. Bốn hộp xếp ngang: internal, closed, open, production. Mũi tên đi từ internal sang closed, từ closed sang open, từ open sang production, và một mũi tên cong đi thẳng từ closed sang production. Một khung cảnh báo bên dưới nói tài khoản cá nhân tạo sau 13 tháng 11 năm 2023 cần closed test 12 tester, 14 ngày liên tục, trước khi vào production.">
<defs><marker id="fl57-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl57-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="240" y="22" class="t-small">closed đi thẳng production được, open là tuỳ chọn</text>
<rect x="25" y="70" width="185" height="100" rx="10" class="f-phys-soft s-phys"/><text x="41" y="102" class="t-b">internal</text><text x="41" y="128" class="t-small">tới 100 tester</text><text x="41" y="148" class="t-small">QA, PM</text>
<rect x="240" y="70" width="185" height="100" rx="10" class="f-pk-soft s-pk"/><text x="256" y="102" class="t-b">closed</text><text x="256" y="128" class="t-small">nhóm được mời</text><text x="256" y="148" class="t-small">tài xế thử</text>
<rect x="455" y="70" width="185" height="100" rx="10" class="f-pk-soft s-pk"/><text x="471" y="102" class="t-b">open</text><text x="471" y="128" class="t-small">ai cũng vào được</text><text x="471" y="148" class="t-small">tuỳ chọn</text>
<rect x="670" y="70" width="185" height="100" rx="10" class="f-phys-soft s-phys"/><text x="686" y="102" class="t-b">production</text><text x="686" y="128" class="t-small">mọi người dùng</text><text x="686" y="148" class="t-small">staged rollout</text>
<path d="M212 120 L238 120" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-1-a)"/>
<path d="M427 120 L453 120" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-1-a)"/>
<path d="M642 120 L668 120" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-1-a)"/>
<path d="M332 68 C332 28 762 28 762 68" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-1-a)"/>
<rect x="25" y="205" width="830" height="62" rx="10" class="f-warn-soft s-warn"/><text x="41" y="231" class="t-b">Cổng trước production: tài khoản cá nhân tạo sau 13/11/2023</text><text x="41" y="253" class="t-small">closed test cần 12 tester opt-in liên tục 14 ngày (kiểm tại 11/2026)</text>
<path d="M762 203 L762 172" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl57-1-w)"/>
</svg>
</div>
<figcaption>Hình 1. Đường đi giữa bốn track theo quy trình của Tới Nơi. Mũi tên đứt là cổng của Google, không phải của ta.</figcaption>
</figure>

Đường đi trong Hình 1 là quy trình của Tới Nơi, không phải luật của Google: open là tuỳ chọn nên closed đi thẳng production được, internal thì không. Lab ghi nó thành một bảng:

### track_promotion.dart · 57-google-play/lib/track_promotion.dart (trích)

```dart
enum Track { internal, closed, open, production }

// ... trong class TrackPromotion
  static const Map<Track, Set<Track>> allowed = {
    Track.internal: {Track.closed},
    Track.closed: {Track.open, Track.production},
    Track.open: {Track.production},
    Track.production: {},
  };

  // ...
  TrackPromotion promote(Track to, PreflightInput input) {
    if (!allowed[track]!.contains(to)) throw IllegalMove(track, to);
    final blockers = preflight(to, input);
    if (blockers.isNotEmpty) throw PromotionBlocked(to, blockers);
    return TrackPromotion._(versionCode, to, [...history, to]);
  }
```

`promote` kiểm đường đi trước, rồi hỏi `preflight` còn điều chặn nào. Kết quả là một `TrackPromotion` mới; bản cũ giữ nguyên.

## Cổng production cho tài khoản cá nhân mới

Theo trang trợ giúp về yêu cầu kiểm thử cho tài khoản cá nhân mới (kiểm tại 11/2026), tài khoản cá nhân tạo sau 13/11/2023 phải chạy closed test với **ít nhất 12 tester**, opt-in **liên tục ít nhất 14 ngày**, rồi mới bấm "Apply for production" trên Dashboard. Tester rời giữa chừng không được tính; rời rồi vào lại thì 14 ngày phải liên tục. Google nói xét duyệt thường không quá bảy ngày, đôi khi lâu hơn. Trang chỉ nói về tài khoản cá nhân, nên lab không chặn tài khoản tổ chức; hãy tự kiểm loại tài khoản của bạn.

Hai con số này do Google quyết và có thể đổi, nên lab giữ chúng ở một chỗ:

### track_promotion.dart · 57-google-play/lib/track_promotion.dart (trích)

```dart
const int requiredClosedTesters = 12;
const int requiredClosedTestDays = 14;

// ... trong class DeveloperAccount
  bool get mustPassClosedTest {
    if (kind != AccountKind.personal) return false;
    final c = createdAt.toUtc();
    return DateTime.utc(c.year, c.month, c.day).isAfter(_personalRuleCutoff);
  }

// ... trong class ClosedTestProgress
  bool get satisfiesProductionAccess =>
      testersOptedIn >= requiredClosedTesters &&
      consecutiveDays >= requiredClosedTestDays;
```

"Sau 13/11/2023" mơ hồ ở đúng ngày 13. Lab đọc theo ngày UTC: tạo ngày 13 thì chưa bị áp, ngày 14 thì bị áp. `testersOptedIn` là số tester ở lại suốt kỳ, không phải số tester được mời.

Hệ quả cho lịch: 14 ngày cộng thời gian xét duyệt là ít nhất khoảng ba tuần trước production. Hãy mời dư vài tài xế để đề phòng có người rời.

## Data safety, quyền nhạy cảm và target API khớp với manifest

Khi bản rời internal, ba thứ phải khớp với những gì app thật sự làm.

**Data safety.** Form bắt buộc với app có trên closed, open hoặc production; app chỉ có trên internal thì không cần. Form hỏi dữ liệu nào rời khỏi máy (kể cả qua SDK), chia sẻ cho ai, để làm gì, có mã hoá khi truyền và có đường xoá không, và đòi link chính sách quyền riêng tư. Người khai tự chịu trách nhiệm; khai lệch có thể dẫn đến việc bị chặn cập nhật hoặc gỡ app.

Lab bắt thiếu sót hiển nhiên bằng cách đọc quyền trong manifest. Bảng "quyền kéo theo loại dữ liệu" là quy tắc của lab, chỉ bao gồm hai loại vị trí:

### data_safety.dart · 57-google-play/lib/data_safety.dart (trích)

```dart
Set<DataType> requiredTypesFor(Set<String> permissions) => {
  if (permissions.contains(coarseLocation)) DataType.approximateLocation,
  if (permissions.contains(fineLocation)) ...{
    DataType.preciseLocation,
    // Người dùng có thể chỉ cấp vị trí gần đúng dù app khai FINE (tài liệu
    // Android về quyền vị trí), nên khi có FINE ta khai cả hai loại.
    DataType.approximateLocation,
  },
};
```

`DataSafetyDraft.check(xml)` trả về danh sách `Finding`. Thiếu loại dữ liệu manifest đòi, thiếu mục đích, link chính sách thiếu hoặc không phải https là **lỗi**. Khai vị trí khi manifest không có quyền, không mã hoá khi truyền, không có đường xoá là **cảnh báo**, vì dữ liệu có thể đến từ nguồn khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 2. Hộp manifest nguồn ở bên trái liệt kê ba quyền: ACCESS_COARSE_LOCATION, ACCESS_FINE_LOCATION, FOREGROUND_SERVICE_LOCATION. Ba mũi tên đi sang bên phải. Mũi tên thứ nhất tới khai báo Data safety cho vị trí gần đúng và chính xác. Mũi tên thứ hai tới khai báo foreground service loại location ở App content. Mũi tên đứt thứ ba tới Permissions Declaration Form: lab không kích hoạt vì manifest không có ACCESS_BACKGROUND_LOCATION, nhưng Google vẫn có thể áp nếu foreground service tương đương vị trí nền. Một khung cảnh báo bên dưới nói Play đọc manifest đã gộp với manifest của plugin.">
<defs><marker id="fl57-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl57-2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="20" width="300" height="250" rx="10" class="f-phys-soft s-phys"/><text x="36" y="52" class="t-b">Manifest nguồn (app tài xế)</text>
<text x="36" y="92" class="t-mono">ACCESS_COARSE_LOCATION</text><text x="36" y="118" class="t-mono">ACCESS_FINE_LOCATION</text><text x="36" y="144" class="t-mono">FOREGROUND_SERVICE_LOCATION</text>
<text x="36" y="196" class="t-small">cùng INTERNET, FOREGROUND_SERVICE,</text><text x="36" y="216" class="t-small">POST_NOTIFICATIONS</text><text x="36" y="248" class="t-small">không có vị trí nền</text>
<rect x="500" y="20" width="360" height="70" rx="10" class="f-pk-soft s-pk"/><text x="516" y="48" class="t-b">Data safety</text><text x="516" y="72" class="t-small">Approximate và Precise location</text>
<rect x="500" y="110" width="360" height="70" rx="10" class="f-pk-soft s-pk"/><text x="516" y="138" class="t-b">Khai báo foreground service</text><text x="516" y="162" class="t-small">App content, loại location</text>
<rect x="500" y="200" width="360" height="70" rx="10" class="f-surface s-muted"/><text x="516" y="228" class="t-b">Permissions Declaration Form</text><text x="516" y="252" class="t-small">lab không kích hoạt; Google vẫn có thể áp</text>
<path d="M322 80 L498 55" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-2-a)"/>
<path d="M322 145 L498 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-2-a)"/>
<path d="M322 210 L498 235" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl57-2-m)"/>
<rect x="20" y="295" width="840" height="62" rx="10" class="f-warn-soft s-warn"/><text x="36" y="321" class="t-b">Play đọc manifest đã gộp với manifest của plugin, không phải file nguồn này</text><text x="36" y="343" class="t-small">build/app/outputs/logs/manifest-merger-buildVariant-report.txt cho biết ai thêm quyền nào</text>
</svg>
</div>
<figcaption>Hình 2. Quyền trong manifest kéo theo khai báo trên Console. Mũi tên đứt: lab không kích hoạt, Google vẫn có thể áp (xem dưới).</figcaption>
</figure>

`bin/check.dart` trên manifest mẫu cho thấy bản nháp khớp và một khai báo Console được kéo theo:

```text
Khai báo Console kéo theo: foregroundServiceLocation
Data safety: khớp.
```

**Vị trí nền và foreground service.** `ACCESS_BACKGROUND_LOCATION` đòi Permissions Declaration Form ở App content: một tính năng duy nhất, là chức năng cốt lõi, có thông báo trong app trước hộp thoại xin quyền, kèm video. App tài xế của bài 35 cố ý không khai quyền này, nhưng trang chính sách của Google nói foreground service lấy vị trí mà tương đương vị trí nền thì vẫn rơi vào yêu cầu đó. Với app nhắm Android 14 trở lên, mỗi loại foreground service còn phải khai ở App content, kèm mô tả và video; loại `location` có use case navigation, nêu cả ride tracking, đúng với tài xế Tới Nơi. Bỏ quyền nền không có nghĩa là thoát thủ tục.

**Target API.** Bản mới và bản cập nhật phải nhắm API 36 từ 31/8/2026; trang nhắc việc gia hạn tới 1/11/2026, mốc đã qua khi bài này viết (kiểm tại 11/2026). `flutter_tools` 3.47.4 mặc định `targetSdkVersion` là 36.

**Pre-launch report.** Khi ta tải bản lên, Play chạy app vài phút trên thiết bị (Android 9 trở lên) và báo về ổn định, tương thích, hiệu năng, trợ năng. Với màn đăng nhập, khai một tài khoản thử riêng trong Pre-launch report, Settings.

Lab gom các điều kiện này vào `preflight`, trả về mọi điều chặn theo thứ tự cố định:

### track_promotion.dart · 57-google-play/lib/track_promotion.dart (trích)

```dart
List<Blocker> preflight(Track to, PreflightInput input) {
  final declarations = consoleDeclarationsFor(input.manifestPermissions);
  return [
    if (!input.dataSafetyComplete) Blocker.dataSafetyIncomplete,
    if (input.targetSdk < minTargetSdk) Blocker.targetSdkTooLow,
    if (declarations.contains(ConsoleDeclaration.backgroundLocation) &&
        !input.backgroundLocationDeclared)
      Blocker.backgroundLocationNotDeclared,
    if (declarations.contains(ConsoleDeclaration.foregroundServiceLocation) &&
        input.targetSdk >= foregroundServiceDeclarationSdk &&
        !input.foregroundServiceDeclared)
      Blocker.foregroundServiceNotDeclared,
    if (to == Track.production &&
        input.account.mustPassClosedTest &&
        !(input.closedTest?.satisfiesProductionAccess ?? false))
      Blocker.productionAccessNotEarned,
  ];
}
```

Lab chỉ đọc manifest **nguồn**. Plugin thêm quyền qua manifest riêng và Gradle gộp tất cả; muốn bỏ quyền do plugin thêm, ta ghi `tools:node="remove"` (tài liệu Android về gộp manifest). `readPermissions` bỏ qua chú thích và quyền có `tools:node="remove"`, nhưng không thấy quyền plugin thêm, nên trước khi phát hành hãy đối chiếu manifest đã gộp. Report của bộ gộp nằm ở `build/outputs/logs` của module; ở dự án Flutter ta kỳ vọng thấy report này tại `build/app/outputs/logs`, chưa kiểm trên máy này.

## Staged rollout, halt và không có rollback

Trang trợ giúp về staged rollout (kiểm tại 11/2026) nêu các dữ kiện làm nền. Staged rollout chỉ dùng cho **bản cập nhật**, không dùng cho lần xuất bản đầu. Ta tự chọn phần trăm, và mức này không tự tăng. Halt ngừng phát cho thêm người, người đã nhận vẫn giữ bản; resume tiếp tục với cùng nhóm người. Trang không có thao tác quay về bản cũ, chỉ có phát bản mới đã sửa, và Android đòi `versionCode` tăng ở mỗi bản phát hành để tránh hạ phiên bản nhầm (bài 51).

Bản đã 100% vẫn halt được (trang "Halting a fully rolled-out release"): bản hoàn chỉnh trước đó được phát lại cho người dùng mới và người chưa cài, và resume được ở mức bất kỳ. Cách này không áp dụng cho internal và lần phát hành đầu của một track. Người đã cập nhật không bị hạ bản, nên "không có rollback" vẫn đúng với họ.

Vì Play không tự quyết, quy trình nằm ở ta. `RolloutPlan` mô hình hoá nó: bản cập nhật bắt đầu ở 1%, mỗi lần đo cho ra một trong ba quyết định (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Năm hộp xếp ngang là các bước 1%, 5%, 20%, 50% và 100%. Mũi tên giữa các hộp ghi điều kiện tiến: crash-free từ 99,50% trở lên với đủ 500 người. Từ cả năm bước, các đường đứt gom lại thành một mũi tên đi xuống hộp Halt rollout, kích hoạt khi crash-free dưới 99,00% với đủ 500 người. Hộp 100% cũng nối xuống Halt. Hộp Halt ghi rằng người đã nhận vẫn giữ bản này, người chưa cập nhật nhận bản trước, và cách sửa là bản mới có versionCode cao hơn.">
<defs><marker id="fl57-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl57-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="40" width="100" height="70" rx="10" class="f-pk-soft s-pk"/><text x="36" y="72" class="t-b">1%</text><text x="36" y="94" class="t-small">bước đầu</text>
<rect x="200" y="40" width="100" height="70" rx="10" class="f-pk-soft s-pk"/><text x="216" y="72" class="t-b">5%</text>
<rect x="380" y="40" width="100" height="70" rx="10" class="f-pk-soft s-pk"/><text x="396" y="72" class="t-b">20%</text>
<rect x="560" y="40" width="100" height="70" rx="10" class="f-pk-soft s-pk"/><text x="576" y="72" class="t-b">50%</text>
<rect x="740" y="40" width="100" height="70" rx="10" class="f-phys-soft s-phys"/><text x="756" y="72" class="t-b">100%</text><text x="756" y="94" class="t-small">completed</text>
<path d="M122 80 L198 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-3-a)"/><text x="127" y="68" class="t-small">≥ 99,50%</text>
<path d="M302 80 L378 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-3-a)"/><text x="307" y="68" class="t-small">≥ 99,50%</text>
<path d="M482 80 L558 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-3-a)"/><text x="487" y="68" class="t-small">≥ 99,50%</text>
<path d="M662 80 L738 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl57-3-a)"/><text x="667" y="68" class="t-small">≥ 99,50%</text>
<path d="M70 112 L70 145 L790 145 M250 112 L250 145 M430 112 L430 145 M610 112 L610 145 M790 112 L790 145" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5"/>
<path d="M340 145 L340 188" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl57-3-w)"/><text x="356" y="172" class="t-small">dưới 99,00%, đủ 500 người</text>
<rect x="190" y="190" width="300" height="90" rx="10" class="f-warn-soft s-warn"/><text x="206" y="218" class="t-b">Halt rollout</text><text x="206" y="244" class="t-small">người đã nhận vẫn giữ bản này</text><text x="206" y="264" class="t-small">sửa bằng bản mới, versionCode cao hơn</text><text x="506" y="218" class="t-small">bản 100% cũng halt được:</text><text x="506" y="238" class="t-small">người chưa cập nhật nhận lại</text><text x="506" y="258" class="t-small">bản hoàn chỉnh trước đó</text>
</svg>
</div>
<figcaption>Hình 3. Rollout minh hoạ. Giữa hai ngưỡng, hoặc khi chưa đủ 500 người, kế hoạch đứng yên. Halt dùng được ở mọi bước, kể cả 100%.</figcaption>
</figure>

Ngưỡng và quyết định nằm trong mã:

### rollout_plan.dart · 57-google-play/lib/rollout_plan.dart (trích)

```dart
// ... trong class RolloutPolicy
  const RolloutPolicy({
    this.steps = const [1, 5, 20, 50, 100],
    this.minUsers = 500,
    this.advanceAtBp = 9950,
    this.haltBelowBp = 9900,
  });

// ... trong class RolloutPlan
  RolloutAction decide(RolloutObservation observation) {
    _requireInProgress();
    if (observation.users < policy.minUsers) return RolloutAction.hold;
    if (observation.crashFreeUsersBp < policy.haltBelowBp) {
      return RolloutAction.halt;
    }
    if (observation.crashFreeUsersBp >= policy.advanceAtBp) {
      return RolloutAction.advance;
    }
    return RolloutAction.hold;
  }
```

Crash-free tính bằng **phần vạn** (9950 là 99,50%) để so sánh với ngưỡng mà không bị sai số `double`. Số này đến từ Crashlytics của bài 48 hoặc nguồn bạn chọn. Cỡ mẫu có ngưỡng riêng vì 1% của một nhóm tài xế nhỏ chỉ là vài người. Dưới `minUsers`, kế hoạch giữ nguyên dù số đo đẹp hay xấu; thấy nghiêm trọng thì người phát hành vẫn gọi `halt()` thủ công.

Phần còn lại là halt, resume và sửa lỗi:

### rollout_plan.dart · 57-google-play/lib/rollout_plan.dart (trích)

```dart
  RolloutPlan halt() {
    if (status == RolloutStatus.halted) {
      throw StateError('rollout đã dừng rồi');
    }
    if (previousVersionCode == null) {
      throw StateError('không halt được lần phát hành đầu của một track');
    }
    return _copy(status: RolloutStatus.halted);
  }

  // ... resume({int? percent}) chọn mức trong policy.steps
  RolloutPlan fixForward({required int newVersionCode}) => RolloutPlan.start(
    versionCode: newVersionCode,
    previousVersionCode: versionCode,
    policy: policy,
  );
```

`halt` giữ nguyên bước nên `percent` không đổi, và dùng được cả khi bản đã 100%, trừ lần phát hành đầu vì không có bản trước để quay lại; `resume(percent: ...)` nhận một mức trong bậc thang. Không có phương thức nào đưa `versionCode` đi xuống: `RolloutPlan.start` từ chối số nhỏ hơn hoặc bằng bản đang chạy, và số ngoài 1 đến 2.100.000.000, mức lớn nhất Play cho phép. Bản đầu tiên (`previousVersionCode` là `null`) đi thẳng tới 100%. Chạy `bin/check.dart` cho một lịch minh hoạ:

```text
Rollout 43 (thay 42), bước: 1 > 5 > 20 > 50 > 100
  120 người, crash-free 99.9% -> hold, còn 1% (inProgress)
  800 người, crash-free 99.6% -> advance, còn 5% (inProgress)
  4000 người, crash-free 99.3% -> hold, còn 5% (inProgress)
  9000 người, crash-free 98.5% -> halt, còn 5% (halted)
```

## Promote bằng supply

Lab có lane `promote` riêng, đặt trong file `import` vào Fastfile của bài 53, vì audit của bài đó cấm `track_promote_to` và các chuỗi `"production"`, `"beta"`, `"alpha"` trong Fastfile chính. Đặt file trong app của bạn, đừng đặt vào `native-config` của lab 53, kẻo test danh sách file của lab đó đỏ. Bảng đường đi bằng Ruby được test so với bảng Dart:

### Fastfile.promote.example · 57-google-play/native-config/fastlane/Fastfile.promote.example (trích)

```ruby
# ... pick_version_code đòi version_code là số nguyên từ 1 đến 2100000000
def pick_rollout_fraction(options, to)
  raw = options[:rollout_percent].to_s
  if to != "production"
    UI.user_error!("rollout_percent chỉ dùng khi promote lên production") unless raw.empty?
    return nil
  end
  unless raw =~ /\A[1-9][0-9]?\z/
    UI.user_error!("promote lên production cần rollout_percent từ 1 đến 99")
  end
  raw.to_i / 100.0
end
```

```ruby
    # ... trong lane :promote
    params = {
      package_name: package_name,
      track: TRACK_NAMES.fetch(from),
      track_promote_to: TRACK_NAMES.fetch(to),
      version_code: version_code,
      json_key_data: json_key_data,
      skip_upload_apk: true,
      skip_upload_aab: true,
      skip_upload_metadata: true,
      skip_upload_images: true,
      skip_upload_screenshots: true,
      skip_upload_changelogs: true,
    }
    params[:rollout] = fraction.to_s unless fraction.nil?
    upload_to_play_store(**params)
```

Theo tài liệu fastlane, `track` kết hợp với `track_promote_to` để promote bản đã có, nên sáu cờ `skip_upload_*` giữ lane khỏi động tới file nhị phân, metadata và ảnh. Phần đáng đọc nằm ở `rollout`. Theo mã nguồn `supply` trên GitHub (đọc tại 11/2026), khi promote, nếu `rollout` nằm giữa 0 và 1 thì bản được đặt ở `inProgress` với tỉ lệ đó; ngoài khoảng ấy, trạng thái lấy từ `track_promote_release_status`, mặc định `completed`. Vậy promote lên production mà quên `rollout` thì bản tới 100% người dùng ngay. Lane đòi `rollout_percent` từ 1 đến 99 và đổi nó thành tỉ lệ. Cũng theo mã nguồn đó, `supply` chọn bản ở track nguồn theo `version_code` nếu có, nếu không thì theo trạng thái; lane đòi `version_code` để bản được promote đúng là bản đã qua `preflight`.

Lane chưa làm ba điều. Tên track trong Play API chưa được kiểm. docs.fastlane.tools liệt kê `internal`, `alpha`, `beta`, `production`; trang Tracks của Google ghép open testing với `beta`, có ví dụ `alpha`, nhưng ghi closed testing tạo tay có tên tuỳ ý và ghi internal là `qa`. `supply` lọc track theo tên chính xác, nên nếu API trả `qa` thì lane dừng. Hãy gọi `tracks.list` cho package của bạn trước lần chạy đầu và sửa `TRACK_NAMES`. Halt không có lane: docs.fastlane.tools liệt kê `halted` là giá trị hợp lệ nhưng không có ví dụ, nên halt làm trên Console. Lần phát hành production đầu tiên, vốn không có staged rollout, cũng làm thủ công.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với Google Play</span>
<ul>
<li><strong>Closed test không đủ người ở lại.</strong> Dấu hiệu: tới ngày thứ 14 vẫn không xin được production. Nguyên nhân: dưới 12 tester opt-in liên tục, hoặc có người rời giữa kỳ. Cách sửa: mời dư người và nhắc họ ở lại.</li>
<li><strong>Khai Data safety theo trí nhớ.</strong> Dấu hiệu: bản cập nhật bị chặn hoặc cảnh báo vi phạm chính sách. Nguyên nhân: form lệch với hành vi hoặc manifest đã gộp, thường do một SDK mới. Cách sửa: đối chiếu form với manifest gộp mỗi lần thêm plugin.</li>
<li><strong>Khai <code>ACCESS_BACKGROUND_LOCATION</code> cho chắc.</strong> Dấu hiệu: Console đòi Permissions Declaration Form và video. Nguyên nhân: quyền do ta hoặc plugin thêm, dù foreground service đã đủ. Cách sửa: bỏ quyền, hoặc <code>tools:node="remove"</code>, rồi đọc report của manifest merger để biết ai thêm.</li>
<li><strong>Promote lên production mà quên <code>rollout</code>.</strong> Dấu hiệu: bản mới tới mọi người dùng ngay. Nguyên nhân: không có <code>rollout</code> thì trạng thái mặc định là <code>completed</code>. Cách sửa: bắt buộc <code>rollout_percent</code> trong lane, như lane <code>promote</code>.</li>
<li><strong>Tưởng halt là rollback.</strong> Dấu hiệu: bản lỗi vẫn chạy trên máy người đã cập nhật. Nguyên nhân: halt chỉ ngừng phát thêm. Cách sửa: phát bản mới có <code>versionCode</code> cao hơn, như <code>fixForward</code>; đừng tìm cách tải lên số thấp hơn.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/57-google-play
dart pub get
dart analyze
dart test
dart run bin/check.dart
```

Đã kiểm với Dart 3.13.3 (Flutter 3.47.4): `dart analyze` 0 issue, **71 test xanh**. Mức: **Kiểm chứng cấu hình**. Test không dùng mạng, không cần thiết bị.

`rollout_plan_test.dart` (25 test) kiểm ngưỡng ở đúng biên (9950 tiến, 9949 giữ, 9900 giữ, 9899 dừng), thiếu mẫu, halt (kể cả bản 100% và lần phát hành đầu), resume ở mức chọn, và `versionCode` bằng hoặc thấp hơn bị từ chối. `track_promotion_test.dart` (16 test) duyệt 16 cặp track và các biên 11 tester, 13 ngày, mốc 13 và 14/11/2023, target API 35 và 36. `data_safety_test.dart` (17 test) đọc manifest mẫu, thử chú thích, `tools:node="remove"`, rồi từng loại lỗi và cảnh báo. `fastlane_promote_test.dart` (13 test) so bảng Ruby với bảng Dart, chạy `ruby -c`, và thi hành lane `promote` bằng Ruby 2.6.10 của hệ thống với bản giả của `UI` và `upload_to_play_store` (tự bỏ qua nếu không có `ruby`).

**Chưa làm**, nói thẳng: không có tài khoản Play Console, nên chưa tải, promote, chạy staged rollout hay halt, chưa thấy Pre-launch report và các form khai báo. Không cài fastlane, nên tên tham số supply chỉ được đối chiếu với tài liệu và mã nguồn trên GitHub; harness Ruby kiểm logic của file, không kiểm fastlane. Không chạy Gradle, nên chưa đọc manifest đã gộp. Mọi con số chính sách đúng theo trang trợ giúp 11/2026, không kiểm với Play thật.

## Bài tập

1. Tài khoản cá nhân tạo 20/3/2025 mời 14 tester vào closed test. Ba người rời ở ngày thứ 10 và không quay lại; hôm nay là ngày thứ 15. Promote từ closed lên production cho kết quả gì?
2. `RolloutPlan` đang ở 20%. Lần đo thứ nhất có 300 người, crash-free 98,0%; lần thứ hai có 700 người, 98,9%. `decide` trả về gì, và sau đó ta làm gì?
3. Ta thêm `ACCESS_BACKGROUND_LOCATION` vào manifest mẫu. Điều gì đổi trong `preflight`, và test nào đỏ?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Bị chặn với <code>productionAccessNotEarned</code>. Tài khoản bị áp, và chỉ 11 tester ở lại suốt kỳ, dưới 12, dù đủ 14 ngày.</p>
<p><strong>2.</strong> Lần thứ nhất trả <code>hold</code> vì 300 người dưới <code>minUsers</code> là 500, dù 98,0% xấu. Lần thứ hai trả <code>halt</code> vì 9890 dưới 9900 và mẫu đã đủ. Người đã nhận vẫn giữ bản đó; ta phát bản mới qua <code>fixForward</code> với <code>versionCode</code> cao hơn và chạy rollout mới từ 1%.</p>
<p><strong>3.</strong> <code>preflight</code> trả <code>backgroundLocationNotDeclared</code> cho tới khi Permissions Declaration Form được khai. Trong <code>data_safety_test.dart</code>, hai test đọc manifest mẫu đỏ: test "khai đúng sáu quyền và không có vị trí nền", và test "chỉ kéo theo khai báo foreground service". Trong <code>track_promotion_test.dart</code>, hầu hết test đỏ, vì đầu vào mẫu đọc manifest mẫu nên mọi lần promote đều dính <code>backgroundLocationNotDeclared</code>. Bản nháp Data safety vẫn khớp, vì quyền nền không thêm loại dữ liệu mới; nó chỉ thêm thủ tục.</p>
</details>

Bài trước: [TestFlight: internal/external testing, Beta App Review và export compliance](/articles/flutter-testflight-internal-external-testing-beta-review). Bài tiếp: [App Store production: metadata, privacy labels, App Review và phased release](/articles/flutter-app-store-production-metadata-privacy-review-phased-release).

## Tài liệu gốc

- [Set up an open, closed, or internal test](https://support.google.com/googleplay/android-developer/answer/9845334) và [Testing requirements for new personal developer accounts](https://support.google.com/googleplay/android-developer/answer/14151465) (support.google.com/googleplay).
- [Prepare and roll out a release](https://support.google.com/googleplay/android-developer/answer/9859348), [Release app updates with staged rollouts](https://support.google.com/googleplay/android-developer/answer/6346149) và [Halting a fully rolled-out release](https://support.google.com/googleplay/android-developer/answer/16285429) (support.google.com/googleplay).
- [Pre-launch report](https://support.google.com/googleplay/android-developer/answer/7002270) và [Provide information for Google Play's Data safety section](https://support.google.com/googleplay/android-developer/answer/10787469) (support.google.com/googleplay).
- [Understanding location in the background permissions](https://support.google.com/googleplay/android-developer/answer/9799150) và [Understanding foreground service and full-screen intent requirements](https://support.google.com/googleplay/android-developer/answer/13392821) (support.google.com/googleplay).
- [Target API level requirements for Google Play apps](https://support.google.com/googleplay/android-developer/answer/11926878) và [Version codes](https://support.google.com/googleplay/android-developer/answer/17367361) (support.google.com/googleplay).
- [Tracks](https://developers.google.com/android-publisher/tracks) (developers.google.com/android-publisher).
- [Merge multiple manifest files](https://developer.android.com/build/manage-manifests) và [Location permissions](https://developer.android.com/develop/sensors-and-location/location/permissions) (developer.android.com).
- [upload_to_play_store (supply)](https://docs.fastlane.tools/actions/upload_to_play_store/) và [Fastfile, import](https://docs.fastlane.tools/advanced/Fastfile/) (docs.fastlane.tools).

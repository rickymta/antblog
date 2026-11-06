---
slug: flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities
title: "Ký ứng dụng iOS: Apple Developer Program, certificate, provisioning profile và capabilities"
summary: "Bài 52: vì sao iPhone chỉ chạy app được Apple cho phép. Vai trò tài khoản, certificate, App ID, profile development, ad hoc và App Store, capabilities và allowlist entitlement, ký tự động hay thủ công theo flavor, ExportOptions.plist, công cụ Dart đọc profile. Kiểm chứng cấu hình, chưa ký thật."
category: programming
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, release, signing]
language: vi
---

Bản staging của app khách hôm qua cài tốt trên mọi iPhone của đội QA. Sáng nay pipeline báo đỏ ở bước ký vì profile "không có entitlement `aps-environment`", và chị Lan vừa nhận iPhone mới thì không cài được bản cũ. Hai lỗi nghe khác nhau nhưng chung một gốc: iPhone chỉ chạy code của bên thứ ba khi Apple đã cho phép bằng một **provisioning profile**, và ta chưa đọc profile ấy nói gì.

Ký iOS khó hơn Android ở số mảnh phải khớp nhau: tài khoản, certificate, App ID, thiết bị và profile. Bài này đi từng mảnh rồi nối chúng vào ba flavor của Tới Nơi. Đây là bài 52 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 51 về ký Android](/articles/flutter-ky-ung-dung-android-keystore-play-app-signing-aab) và [bài 50 về flavor khi build](/articles/flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build). Bài tiếp theo tự động hoá phần này bằng [fastlane match](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Ký iOS cần macOS, Xcode và tài khoản Apple Developer Program trả phí. Máy viết bài không có tài khoản hay chứng chỉ, nên <strong>chưa có lệnh <code>xcodebuild archive</code> hay <code>flutter build ipa</code> nào chạy thật</strong>. Lab chỉ kiểm file cấu hình và công cụ Dart đọc profile giả.</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>IPA staging ký bằng profile ad hoc đúng, nhưng iPhone của chị Lan chưa nằm trong profile. Ta bật thêm Push Notifications cho App ID, thêm <code>aps-environment</code> vào entitlements của app rồi build lại với <em>profile cũ</em>. Máy chị Lan cài được không, và bản build mới có ký được không?</p>
<details><summary>Xem đáp án</summary><p>Cả hai đều không. Theo Apple, người thử chỉ chạy được app trên các thiết bị đã nằm trong profile. Và khi bật hay tắt app service, profile phải được tạo lại. Mục 3 cho thấy lý do và cách công cụ của lab bắt lỗi này.</p></details>
</div>

## 1. Tài khoản, vai trò và Team ID

Mọi thứ bắt đầu từ **Apple Developer Program**. Trang đăng ký ghi phí 99 USD mỗi năm thành viên (kiểm 11/2026; giá hiển thị theo tiền địa phương lúc đăng ký). Tài khoản tổ chức cần thêm D-U-N-S Number, website của tổ chức và người có thẩm quyền ràng buộc pháp lý. Tới Nơi là tổ chức nên dùng loại này.

Mỗi thành viên có một **Team ID** 10 ký tự do Apple cấp, đứng đầu App ID và có trong mọi profile. Bài này dùng `ABCDE12345`, rõ ràng là giả.

Vai trò quyết định ai làm được bước nào, và các trang của Apple không hoàn toàn khớp nhau. Trang hướng dẫn ghi "Required role: Account Holder or Admin" cho việc đăng ký App ID, tạo profile ad hoc và App Store; trang certificate nói chỉ Account Holder hoặc Admin tạo được certificate Distribution. Bảng Roles thì nói Developer và App Manager cũng làm được các việc đó nếu được cấp quyền truy cập Certificates, Identifiers & Profiles trong Users and Access; với Developer, automatic signing của Xcode chỉ áp cho profile development và việc thêm UDID. Ta xem trang Roles là bảng quyền đầy đủ, và kiểm lại cả hai trang khi làm.

Hệ quả: lập trình viên bình thường không tự tạo được profile staging. Hoặc một Admin tạo sẵn, hoặc ta dùng công cụ như `match` (bài 53). Cấu hình của flavor staging nêu rõ team, kiểu ký và tên profile:

### Signing-staging.xcconfig.example · 52-ios-signing/native-config/ios/Signing-staging.xcconfig.example

```text
// Flavor staging: build trên CI, ký bằng Apple Distribution và profile ad-hoc.
// Dán vào Release-staging.xcconfig.
PRODUCT_BUNDLE_IDENTIFIER = vn.toinoi.customer.staging
CODE_SIGN_ENTITLEMENTS = Runner/Runner.entitlements
CODE_SIGN_STYLE = Manual
// ABCDE12345 là Team ID giả. Thay bằng Team ID của bạn.
DEVELOPMENT_TEAM = ABCDE12345
CODE_SIGN_IDENTITY = Apple Distribution
// Tên profile đúng như trong Certificates, Identifiers & Profiles.
PROVISIONING_PROFILE_SPECIFIER = ToiNoi Customer Staging AdHoc
```

## 2. Certificate, App ID, thiết bị và profile

Technote TN3125 của Apple mô tả profile như câu trả lời cho năm câu hỏi: ai được ký, ký app nào, chạy ở đâu, chạy tới khi nào, app được quyền gì. Mỗi câu ứng với một khoá hoặc một mục trong profile đã giải mã (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Ba nguyên liệu gồm certificate, App ID và danh sách thiết bị đi vào provisioning profile do Apple ký. Profile được nhúng vào tệp IPA khi export. iPhone kiểm chữ ký Apple trên profile rồi kiểm app khớp các điều kiện."><defs><marker id="fl52-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="20" width="250" height="84" rx="10" class="f-phys-soft s-phys"/><text x="36" y="48" class="t-b">Certificate Distribution</text><text x="36" y="70" class="t-small">khoá riêng nằm ở Mac</text><text x="36" y="88" class="t-small">đã tạo certificate</text>
<rect x="20" y="128" width="250" height="84" rx="10" class="f-phys-soft s-phys"/><text x="36" y="156" class="t-b">App ID</text><text x="36" y="178" class="t-small">bundle ID + capabilities</text><text x="36" y="196" class="t-small">đã bật</text>
<rect x="20" y="236" width="250" height="84" rx="10" class="f-phys-soft s-phys"/><text x="36" y="264" class="t-b">Danh sách thiết bị</text><text x="36" y="286" class="t-small">UDID đã đăng ký</text><text x="36" y="304" class="t-small">(ad hoc, development)</text>
<rect x="330" y="20" width="250" height="300" rx="10" class="f-pk-soft s-pk"/><text x="346" y="48" class="t-b">Provisioning profile</text><text x="346" y="68" class="t-small">Apple ký, có hạn dùng</text>
<text x="346" y="110" class="t-mono">DeveloperCertificates</text><text x="346" y="128" class="t-small">ai được ký</text>
<text x="346" y="156" class="t-mono">ProvisionedDevices</text><text x="346" y="174" class="t-small">chạy ở đâu</text>
<text x="346" y="202" class="t-mono">ExpirationDate</text><text x="346" y="220" class="t-small">chạy tới khi nào</text>
<text x="346" y="248" class="t-mono">Entitlements</text><text x="346" y="266" class="t-small">ký app nào, qua khoá</text><text x="346" y="284" class="t-mono">application-identifier</text><text x="346" y="302" class="t-small">và được quyền gì</text>
<path d="M272 62 L328 62" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-1-a)"/>
<path d="M272 170 L328 170" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-1-a)"/>
<path d="M272 278 L328 278" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-1-a)"/>
<rect x="630" y="20" width="230" height="120" rx="10" class="f-phys-soft s-phys"/><text x="646" y="48" class="t-b">Tệp IPA</text><text x="646" y="72" class="t-mono">embedded.mobileprovision</text><text x="646" y="94" class="t-small">ký bằng khoá riêng</text><text x="646" y="112" class="t-small">của certificate</text>
<rect x="630" y="200" width="230" height="120" rx="10" class="f-warn-soft s-warn"/><text x="646" y="228" class="t-b">iPhone</text><text x="646" y="252" class="t-small">kiểm chữ ký Apple</text><text x="646" y="270" class="t-small">trên profile, rồi kiểm</text><text x="646" y="288" class="t-small">app khớp từng điều kiện</text>
<path d="M582 80 L628 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-1-a)"/><text x="586" y="68" class="t-small">export</text>
<path d="M745 142 L745 198" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-1-a)"/><text x="756" y="176" class="t-small">cài</text>
</svg>
</div>
<figcaption>Hình 1. Ba nguyên liệu vào profile; profile đi theo IPA tới thiết bị. Thiếu hoặc lệch một nguyên liệu thì thiết bị từ chối app.</figcaption>
</figure>

**Certificate** là danh tính người ký: Apple Development để chạy lúc phát triển, Apple Distribution để phân phối thử nghiệm hoặc gửi App Store Connect. Certificate Development thuộc từng cá nhân. Certificate Distribution thuộc team và theo trang Apple mỗi loại chỉ có một cái cho cả team (nguồn ngoài có nơi ghi con số khác), nên hai đồng nghiệp tạo trùng sẽ vướng nhau. Phần công khai nằm trên trang Apple, còn **khoá riêng** chỉ nằm ở keychain của máy đã tạo yêu cầu ký. Máy CI không có khoá này thì không ký được (bài 53, 54).

**App ID** ghép Team ID với bundle ID; các capability bật cho nó là allowlist cho mọi app dùng nó. Tới Nơi dùng App ID tường minh cho từng bundle ID, không dùng wildcard, vì Apple ghi một số capability chỉ bật được với App ID tường minh.

**Thiết bị** chỉ cần cho profile development và ad hoc. Mỗi team đăng ký tối đa 100 thiết bị cho mỗi dòng máy (iPhone, iPad, ...) mỗi năm thành viên (kiểm 11/2026). Vô hiệu hoá thiết bị giữa năm không trả lại chỗ; số đếm chỉ được dọn khi sang năm mới và Account Holder, Admin hoặc App Manager chọn xoá. Vì vậy ta chỉ đăng ký máy QA thật sự dùng.

Có ba loại profile, khác nhau ở thiết bị và certificate:

| Loại | Thiết bị | Certificate | Dùng cho |
|---|---|---|---|
| Development | thiết bị đã đăng ký | Development của các thành viên | `flutter run` trên máy thật (flavor dev) |
| Ad hoc | thiết bị đã chọn | một Distribution | bản staging cho QA ngoài TestFlight |
| App Store | không có | một Distribution | TestFlight và App Store |

Apple không ghi loại profile thành một khoá. Lab suy ra từ nội dung; đó là heuristic, không phải quy tắc Apple công bố:

### provisioning_profile.dart · 52-ios-signing/lib/src/provisioning_profile.dart (trích)

```dart
  /// Suy ra loại profile: không có danh sách thiết bị là App Store;
  /// `ProvisionsAllDevices` là Enterprise; `get-task-allow` bật là
  /// development; còn lại là ad hoc. Đây là heuristic dựa trên profile phổ
  /// biến, không phải quy tắc Apple công bố.
  ProfileKind get kind {
    if (provisionsAllDevices) return ProfileKind.enterprise;
    if (provisionedDevices == null) return ProfileKind.appStore;
    if (entitlements['get-task-allow'] == true) return ProfileKind.development;
    return ProfileKind.adHoc;
  }
```

## 3. Capabilities và entitlements: hai lớp allowlist

Capability là cánh cửa, entitlement là chìa khoá app cầm. Muốn nhận push, app phải **xin** entitlement `aps-environment`; muốn mở link của `toinoi.example` thẳng vào app, app xin `com.apple.developer.associated-domains`. Apple chỉ cho khi cả App ID lẫn profile đều cho phép (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Capability bật trên App ID được đưa vào profile khi tạo hoặc tạo lại profile. Khi ký, mọi entitlement trong Runner.entitlements phải nằm trong allowlist của profile. Ví dụ với profile cũ staging-adhoc-old, hai entitlement app xin bị thiếu."><defs><marker id="fl52-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl52-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="20" width="230" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="48" class="t-b">App ID</text><text x="36" y="72" class="t-small">capability đã bật:</text><text x="36" y="90" class="t-mono">Push, Associated Domains</text>
<rect x="340" y="20" width="230" height="100" rx="10" class="f-pk-soft s-pk"/><text x="356" y="48" class="t-b">Profile (allowlist)</text><text x="356" y="72" class="t-small">khoá Entitlements,</text><text x="356" y="90" class="t-small">chụp lúc tạo profile</text>
<rect x="660" y="20" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="676" y="48" class="t-b">Runner.entitlements</text><text x="676" y="72" class="t-small">thứ app xin khi ký</text>
<path d="M252 70 L338 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-2-a)"/><text x="260" y="58" class="t-small">tạo lại</text>
<path d="M658 70 L572 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl52-2-a)"/><text x="592" y="58" class="t-small">nằm trong</text>
<rect x="20" y="170" width="840" height="150" rx="10" class="f-warn-soft s-warn"/><text x="36" y="198" class="t-b t-warn">Ví dụ trong lab: samples/staging-adhoc-old.plist</text>
<text x="36" y="224" class="t-small">app xin</text><text x="150" y="224" class="t-mono">aps-environment, com.apple.developer.associated-domains</text>
<text x="36" y="248" class="t-small">profile cho phép</text><text x="150" y="248" class="t-mono">application-identifier, com.apple.developer.team-identifier, get-task-allow</text>
<text x="36" y="280" class="t-b">Thiếu hai khoá: profile tạo trước khi bật capability.</text>
<text x="36" y="302" class="t-small">Sửa: bật capability trên App ID, tạo lại profile, tải về, ký lại.</text>
<path d="M455 122 L455 168" class="f-none s-warn dash" stroke-width="2" marker-end="url(#fl52-2-w)"/>
</svg>
</div>
<figcaption>Hình 2. Chiều đi: App ID tạo profile, app bị profile kiểm. Đổi capability sau khi tạo profile thì profile cũ lỗi thời.</figcaption>
</figure>

TN3125 nói rõ: mọi entitlement app xin phải nằm trong allowlist của profile; chiều ngược lại không bắt buộc. Tài liệu Apple cũng nói khi bật hay tắt app service ta phải tạo lại profile. File entitlements của app khách có hai khoá:

### Runner.entitlements.example · 52-ios-signing/native-config/ios/Runner.entitlements.example (trích)

```xml
<dict>
    <!-- Capability "Push Notifications". Xcode đặt giá trị theo profile đang dùng. -->
    <key>aps-environment</key>
    <string>development</string>
    <!-- Capability "Associated Domains": định dạng <service>:<tên miền đầy đủ>. -->
    <key>com.apple.developer.associated-domains</key>
    <array>
        <string>applinks:toinoi.example</string>
    </array>
</dict>
```

Với `aps-environment`, tài liệu Apple nói Xcode đặt giá trị theo profile đang dùng: profile development cho `development`, profile production và bản TestFlight cho `production`. File `apple-app-site-association` trên `toinoi.example` là chuyện của phía tên miền, không thuộc ký ứng dụng (bài 23, 37).

**Background Modes** là ngoại lệ dễ nhầm: capability này ghi `UIBackgroundModes` vào `Info.plist`, không vào file entitlements, và test của lab chặn nhầm lẫn đó. App khách chỉ cần `remote-notification`; app tài xế sẽ thêm `location` (bài 38):

### Info.plist.fragment.example · 52-ios-signing/native-config/ios/Info.plist.fragment.example (trích)

```xml
    <!-- Đoạn cần chép vào ios/Runner/Info.plist.
         Capability "Background Modes" ghi vào Info.plist, không phải file entitlements. -->
    <key>UIBackgroundModes</key>
    <array>
        <string>remote-notification</string>
    </array>
```

Công cụ của lab bắt lỗi allowlist bằng một phép trừ tập hợp, chỉ so tên khoá:

### provisioning_profile.dart · 52-ios-signing/lib/src/provisioning_profile.dart (trích)

```dart
  /// Các quyền app xin ([claimed]) mà profile không có trong allowlist.
  ///
  /// Theo TN3125, mọi entitlement app xin phải nằm trong allowlist của
  /// profile; chiều ngược lại thì không bắt buộc. Chỉ so tên khoá: luật so
  /// giá trị (wildcard, môi trường push) do Apple kiểm lúc ký.
  Set<String> missingEntitlements(Iterable<String> claimed) =>
      claimed.where((key) => !entitlements.containsKey(key)).toSet();
```

Test chứng minh: profile mới đủ quyền, profile cũ thiếu đúng hai khoá, allowlist rộng hơn thì không sao.

### profile_test.dart · 52-ios-signing/test/profile_test.dart (trích)

```dart
  group('allowlist entitlement', () {
    final claimed = configPlist('Runner.entitlements.example').keys;

    test('profile tạo sau khi bật capability cho phép mọi quyền app xin', () {
      expect(sample('staging-adhoc').missingEntitlements(claimed), isEmpty);
    });

    test('profile cũ thiếu push và associated domains', () {
      expect(sample('staging-adhoc-old').missingEntitlements(claimed), {
        'aps-environment',
        'com.apple.developer.associated-domains',
      });
    });

    test('allowlist được phép rộng hơn thứ app xin', () {
      expect(sample('staging-adhoc').missingEntitlements(const []), isEmpty);
    });
  });
```

## 4. Ký tự động hay thủ công, theo flavor

Build setting `CODE_SIGN_STYLE` chọn cách tìm tài sản ký. Theo tài liệu build settings của Apple, `Automatic` để Xcode tự tạo và cập nhật profile, App ID và certificate; `Manual` để ta tự làm trên trang developer, và tự cấu hình các bước của capability. `DEVELOPMENT_TEAM` là Team ID, `CODE_SIGN_IDENTITY` là tên certificate trong keychain, `PROVISIONING_PROFILE_SPECIFIER` là tên hoặc UUID của profile; profile thiếu hay sai thì build lỗi.

Tài liệu Flutter nói automatic signing đủ cho hầu hết app. Tới Nơi chọn theo flavor; đó là quyết định của đội, không phải luật của Apple:

- **dev**: Automatic. Xcode trên máy lập trình viên đăng ký thiết bị và tạo profile development khi ta bấm chạy.
- **staging, prod**: Manual với profile đặt tên cố định. Build trên CI lặp lại được, không sinh profile bất ngờ, không cần phiên đăng nhập Xcode.

### Signing-dev.xcconfig.example · 52-ios-signing/native-config/ios/Signing-dev.xcconfig.example

```text
// Flavor dev: chạy trên máy lập trình viên, để Xcode tự quản lý.
// Dán vào Debug-dev.xcconfig (bài 50 dạy cách tạo configuration theo flavor).
PRODUCT_BUNDLE_IDENTIFIER = vn.toinoi.customer.dev
CODE_SIGN_ENTITLEMENTS = Runner/Runner.entitlements
CODE_SIGN_STYLE = Automatic
// ABCDE12345 là Team ID giả. Thay bằng Team ID của bạn.
DEVELOPMENT_TEAM = ABCDE12345
```

Test khoá quy ước này: flavor Manual phải có profile, flavor Automatic thì không được ghim.

### native_config_test.dart · 52-ios-signing/test/native_config_test.dart (trích)

```dart
    test('flavor manual luôn có profile; flavor automatic thì không', () {
      for (final flavor in ['dev', 'staging', 'prod']) {
        final c = xcconfig(flavor);
        final manual = c['CODE_SIGN_STYLE'] == 'Manual';
        expect(
          c['PROVISIONING_PROFILE_SPECIFIER'],
          manual ? isNotEmpty : isNull,
          reason: flavor,
        );
      }
    });
```

Có một điểm nối với Flutter, đã đọc trong source Flutter 3.47.4. Với build release hoặc profile, `CODE_SIGN_STYLE = Manual` và đủ `DEVELOPMENT_TEAM` lẫn `PROVISIONING_PROFILE_SPECIFIER`, lệnh `flutter build ipa` không kèm `--export-options-plist` tự dựng file export, tìm profile theo tên hoặc UUID trong `~/Library/Developer/Xcode/UserData/Provisioning Profiles/`. Không thấy thì nó quay về file chỉ có `method`. Vì vậy ta chủ động viết ExportOptions riêng.

## 5. ExportOptions.plist và flutter build ipa

Lệnh `xcodebuild -exportArchive` nhận `-exportOptionsPlist` để biết xuất archive ra sao, và khoá `method` chọn cách phân phối. Apple đã đổi tên các giá trị này; source Flutter đặt mốc ở Xcode 15.4. Theo trích dẫn `xcodebuild -help` của một DTS engineer trên diễn đàn Apple, giá trị hiện hành gồm `app-store-connect`, `release-testing`, `enterprise`, `debugging`, `developer-id`, `mac-application` và `validation`; còn `app-store`, `ad-hoc`, `development` đã deprecated.

| Tên cũ | Tên hiện hành | Tới Nơi dùng cho |
|---|---|---|
| `development` | `debugging` | máy thật của lập trình viên |
| `ad-hoc` | `release-testing` | staging cho QA (Firebase App Distribution, bài 55) |
| `app-store` | `app-store-connect` | TestFlight và App Store (bài 56, 58) |

Flutter 3.47.4 chỉ nhận tên cũ ở `--export-method` (`app-store`, `ad-hoc`, `development`, `enterprise`; tên mới bị từ chối) và tự đổi sang tên mới khi Xcode từ 15.4 trở lên. Hai tuỳ chọn `--export-method` và `--export-options-plist` không dùng chung được, nên khi dùng file riêng ta tự ghi tên mới:

### ExportOptions-adhoc.plist.example · 52-ios-signing/native-config/ios/ExportOptions-adhoc.plist.example (trích)

```xml
    <!-- Bản staging cho thiết bị đã đăng ký (Firebase App Distribution, bài 55).
         "release-testing" là tên hiện hành của "ad-hoc". -->
    <key>method</key>
    <string>release-testing</string>
    <!-- ABCDE12345 là Team ID giả. Thay bằng Team ID của bạn. -->
    <key>teamID</key>
    <string>ABCDE12345</string>
    <key>signingStyle</key>
    <string>manual</string>
    <key>signingCertificate</key>
    <string>Apple Distribution</string>
    <key>provisioningProfiles</key>
    <dict>
        <key>vn.toinoi.customer.staging</key>
        <string>ToiNoi Customer Staging AdHoc</string>
    </dict>
    <key>destination</key>
    <string>export</string>
    <key>stripSwiftSymbols</key>
    <true/>
```

Bản App Store cùng khuôn, khác `method`, bundle ID, tên profile và có `uploadSymbols`. Lệnh build (cần macOS, Xcode và chứng chỉ thật; chưa chạy ở đây):

```bash
flutter build ipa --flavor staging \
  --export-options-plist=ios/ExportOptions-adhoc.plist

flutter build ipa --flavor prod \
  --export-options-plist=ios/ExportOptions-appstore.plist
```

Theo tài liệu Flutter, archive nằm ở `build/ios/archive/` và IPA ở `build/ios/ipa`; upload thuộc bài 53 và 56. Hàm `lintExportOptions` bắt ba lỗi hay gặp: tên method cũ, `teamID` lệch `DEVELOPMENT_TEAM`, và `signingStyle` manual mà thiếu `provisioningProfiles`:

### export_options.dart · 52-ios-signing/lib/src/export_options.dart (trích)

```dart
List<ConfigIssue> lintExportOptions(
  Map<String, Object?> plist, {
  String? expectedTeamId,
}) {
  final issues = <ConfigIssue>[];
  // ...
  final method = plist['method'];
  if (method is! String) {
    issues.add(const ConfigIssue('missing-method', 'Thiếu khoá method'));
  } else if (deprecatedExportMethods.containsKey(method)) {
    issues.add(
      ConfigIssue(
        'deprecated-method',
        'method "$method" đã deprecated, dùng '
            '"${deprecatedExportMethods[method]}"',
      ),
    );
  } else if (!currentExportMethods.contains(method)) {
    issues.add(ConfigIssue('unknown-method', 'method "$method" không hợp lệ'));
  }
  // ...
  if (plist['signingStyle'] == 'manual') {
    final profiles = plist['provisioningProfiles'];
    if (profiles is! Map<String, Object?> || profiles.isEmpty) {
      issues.add(
        const ConfigIssue(
          'manual-without-profiles',
          'signingStyle manual cần provisioningProfiles (bundle ID -> tên '
              'profile)',
        ),
      );
    }
  }
  return issues;
}
```

Test duyệt từng tên cũ và đòi gợi ý đúng tên mới:

### native_config_test.dart · 52-ios-signing/test/native_config_test.dart (trích)

```dart
    test('tên cũ được gợi ý tên mới', () {
      for (final entry in deprecatedExportMethods.entries) {
        final issues = lintExportOptions(base()..['method'] = entry.key);
        expect(issues.single.code, 'deprecated-method');
        expect(issues.single.message, contains(entry.value));
      }
    });
```

## 6. Đọc profile bằng Dart: hạn dùng, thiết bị, entitlement

File `.mobileprovision` thật là một plist bọc trong lớp CMS. TN3125 hướng dẫn gỡ lớp đó bằng `security cms -D -i`, và cảnh báo định dạng profile không phải tài liệu ổn định; profile mới còn mang khoá `DER-Encoded-Profile` làm nguồn chân lý. Vì thế công cụ của lab dành cho học và gỡ lỗi, không để dựng sản phẩm. Nó đọc plist đã giải; Flutter tool cũng đọc `Name`, `UUID`, `TeamIdentifier` và `ExpirationDate` theo đúng tên đó.

Hạn dùng nằm trong `statusAt`; thời điểm hiện tại được truyền vào để test không phụ thuộc đồng hồ:

### provisioning_profile.dart · 52-ios-signing/lib/src/provisioning_profile.dart (trích)

```dart
  /// Tình trạng tại [now]: hết hạn khi `now` chạm hoặc vượt hạn, sắp hết hạn
  /// khi còn không quá [warnWithin].
  ExpiryStatus statusAt(
    DateTime now, {
    Duration warnWithin = const Duration(days: 30),
  }) {
    final left = remaining(now);
    if (left <= Duration.zero) return ExpiryStatus.expired;
    if (left <= warnWithin) return ExpiryStatus.expiringSoon;
    return ExpiryStatus.valid;
  }
```

Test khoá các ca biên: đúng 30 ngày trước hạn thì cảnh báo, 31 ngày thì chưa, đúng giây hết hạn thì coi là hết hạn.

### profile_test.dart · 52-ios-signing/test/profile_test.dart (trích)

```dart
  group('hạn dùng', () {
    final p = sample('staging-adhoc'); // hết hạn 2027-01-20T00:00:00Z
    final expiry = p.expirationDate;

    test('còn xa thì hợp lệ', () {
      expect(p.statusAt(DateTime.utc(2026, 11, 7)), ExpiryStatus.valid);
    });

    test('đúng 30 ngày trước hạn bắt đầu cảnh báo', () {
      final now = expiry.subtract(const Duration(days: 30));
      expect(p.statusAt(now), ExpiryStatus.expiringSoon);
    });

    test('31 ngày trước hạn chưa cảnh báo', () {
      final now = expiry.subtract(const Duration(days: 31));
      expect(p.statusAt(now), ExpiryStatus.valid);
    });

    test('đúng giây hết hạn đã coi là hết hạn', () {
      expect(p.statusAt(expiry), ExpiryStatus.expired);
    });
```

Chạy công cụ với profile cũ của lab, so với `Runner.entitlements.example` và máy của chị Lan (UDID giả); kết quả dưới đây là trích:

```bash
dart bin/inspect_profile.dart samples/staging-adhoc-old.plist \
  --now=2026-11-07T00:00:00Z \
  --entitlements=native-config/ios/Runner.entitlements.example \
  --device=00008030-000000000000000C
```

```text
Tên:        ToiNoi Customer Staging AdHoc
Bundle ID:  vn.toinoi.customer.staging
Loại:       ad hoc (suy ra)
Hết hạn:    2026-11-20T00:00:00.000Z (còn 13 ngày)
Thiết bị:   2
CẢNH BÁO: profile còn 13 ngày là hết hạn (ngưỡng cảnh báo 30 ngày)
CẢNH BÁO: app xin entitlement mà profile không cho phép: aps-environment, com.apple.developer.associated-domains; ...
CẢNH BÁO: thiết bị 00008030-000000000000000C không nằm trong profile
```

Mã thoát 1 khi có cảnh báo, nên CI có thể chặn build trước khi tốn phút macOS. Gặp file CMS thật, công cụ nhận ra byte đầu của DER, nhắc lệnh `security cms -D -i` rồi thoát với mã 2.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi ký iOS hay gặp</span>
<ul>
<li><strong>Không tìm thấy profile.</strong> Dấu hiệu: thông báo kiểu (diễn đạt lại, không nguyên văn) "No profiles for 'vn.toinoi.customer.staging' were found ... Automatic signing is disabled" (chữ đổi theo phiên bản Xcode). Nguyên nhân thường gặp: Manual mà máy build chưa cài profile, hoặc tên trong <code>PROVISIONING_PROFILE_SPECIFIER</code> lệch tên thật. Cách sửa: cài profile, giải bằng <code>security cms</code>, so <code>Name</code> từng ký tự.</li>
<li><strong>Certificate có mà không ký được.</strong> Dấu hiệu: thông báo kiểu (diễn đạt lại) "No ... signing certificate matching team ID ... with a private key was found". Nguyên nhân: certificate nằm trên trang Apple nhưng khoá riêng ở máy khác, thường là Mac của người đã tạo. Cách sửa: xuất certificate kèm khoá riêng từ máy gốc sang CI, hoặc để <code>match</code> giữ (bài 53). Tạo mới có thể đụng giới hạn số certificate Distribution của team.</li>
<li><strong>Profile thiếu entitlement.</strong> Dấu hiệu: lỗi kiểu (diễn đạt lại) "Provisioning profile ... doesn't include the aps-environment entitlement". Nguyên nhân: bật capability sau khi tạo profile (Hình 2), hoặc khai sai tên khoá như <code>com.apple.developer.push-notifications</code> (test của lab chặn tên này). Cách sửa: bật trên App ID, tạo lại profile, tải về, ký lại; kiểm bằng <code>--entitlements</code>.</li>
<li><strong>IPA không cài hoặc không mở được trên máy tester.</strong> Dấu hiệu: cài lỗi, hoặc cài xong không chạy. Kiểm theo thứ tự: UDID có trong profile không (<code>--device</code>); profile còn hạn không; máy đã bật Developer Mode chưa (Apple ghi riêng cho IPA cài từ tệp); với thành viên tạo sau 6/6/2021, app development và ad hoc phải có mạng lần mở đầu để kiểm tra chữ ký. Sửa: thêm thiết bị, tạo lại profile, export lại.</li>
<li><strong>Cảnh báo method deprecated.</strong> Dấu hiệu: log hoặc công cụ CI báo tên <code>ad-hoc</code> đã deprecated (<code>xcodebuild -help</code> ghi "ad-hoc (deprecated: use release-testing)"), hoặc công cụ cũ từ chối giá trị mới. Cách sửa: dùng <code>release-testing</code>, <code>app-store-connect</code>, <code>debugging</code> và nâng công cụ cũ; <code>lintExportOptions</code> bắt ca này sớm.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

**Mức kiểm chứng: cấu hình (CH).** Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/52-ios-signing
dart pub get
dart analyze
dart test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `dart analyze` 0 issue, `dart test` 69 test xanh. Các nhóm test chứng minh:

- **Đọc plist và xcconfig**: các kiểu dữ liệu, XML hỏng, khoá thiếu giá trị, khoá có điều kiện `[sdk=...]`.
- **Profile mẫu**: loại, hạn dùng ở các ca biên, thiết bị, wildcard bundle ID, allowlist entitlement.
- **`native-config/`**: giá trị entitlements, `UIBackgroundModes`, xcconfig ba flavor, hai `ExportOptions`, và đối chiếu chéo tên profile, bundle ID, Team ID với nhau và với profile mẫu.
- **Dòng lệnh**: mã thoát 0, 1, 2 và nhận ra file CMS.
- **Bí mật**: không khoá riêng, không chứng chỉ; mọi Team ID và UDID là số giả.

**Chưa chạy**: `xcodebuild archive`, `-exportArchive`, `flutter build ipa`; cài IPA lên thiết bị; tạo certificate, App ID, profile trên tài khoản thật; đọc `.mobileprovision` thật. Tên `method` đối chiếu bằng trích dẫn của Apple DTS và source Flutter, chưa tự chạy `xcodebuild -help` trên máy này. **Chưa làm**: app tài xế (cùng cấu trúc, bộ riêng), profile staging cho TestFlight (bài 56), fastlane (bài 53) và secrets trên CI (bài 54).

## Bài tập

1. Ta đã đăng ký UDID của chị Lan trên trang Apple nhưng máy vẫn không cài được bản staging. Còn thiếu bước nào, và công cụ lab giúp gì?
2. Đội thêm capability Associated Domains cho app tài xế. Liệt kê các bước theo thứ tự để pipeline ký không đỏ.
3. Viết thêm `ExportOptions` cho staging đi TestFlight (cùng bundle ID staging). Lab cần thêm gì để test vẫn khớp?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Đăng ký UDID chưa đủ: profile ad hoc phải được chọn thêm thiết bị hoặc tạo lại, tải về, rồi export lại IPA bằng profile mới. Chạy <code>inspect_profile.dart --device=UDID</code> trên profile đã tải; nếu máy chưa có thì IPA vừa build vẫn mang profile cũ.</p>
<p><strong>2.</strong> Bật Associated Domains trên App ID của app tài xế; tạo lại mọi profile dùng App ID đó (ad hoc và App Store); thêm khoá <code>com.apple.developer.associated-domains</code> vào entitlements; thay profile trên máy build hoặc kho chứng chỉ; chạy công cụ với <code>--entitlements</code> để chắc profile mới chứa khoá này trước khi build.</p>
<p><strong>3.</strong> Thêm <code>ExportOptions-staging-testflight.plist.example</code> với <code>method</code> là <code>app-store-connect</code>, bundle ID staging và tên profile App Store của staging (tạo trên trang Apple). Thêm test riêng cho file mới, và <code>lintExportOptions</code> phải trả danh sách rỗng. Xcconfig staging vẫn ghim profile ad hoc; profile TestFlight chỉ xuất hiện ở bước export.</p>
</details>

Bài trước: [Ký ứng dụng Android: keystore, Play App Signing, AAB và versionCode](/articles/flutter-ky-ung-dung-android-keystore-play-app-signing-aab). Bài tiếp: [fastlane cho Flutter: match, gym, pilot, supply](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build).

## Tài liệu gốc

- [Apple: TN3125, Inside Code Signing: Provisioning Profiles](https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles)
- [Apple Developer Program: đăng ký và điều kiện](https://developer.apple.com/programs/enroll/)
- [Apple Developer Account Help: Roles](https://developer.apple.com/help/account/access/roles/)
- [Apple Developer Account Help: Certificates overview](https://developer.apple.com/help/account/certificates/certificates-overview/)
- [Apple Developer Account Help: Devices overview](https://developer.apple.com/help/account/devices/devices-overview/)
- [Apple Developer Account Help: Register an App ID](https://developer.apple.com/help/account/identifiers/register-an-app-id/)
- [Apple Developer Account Help: Create an ad hoc provisioning profile](https://developer.apple.com/help/account/provisioning-profiles/create-an-ad-hoc-provisioning-profile/)
- [Apple Developer Account Help: Create an App Store provisioning profile](https://developer.apple.com/help/account/provisioning-profiles/create-an-app-store-provisioning-profile/)
- [Apple Developer Account Help: Edit, download, or delete provisioning profiles](https://developer.apple.com/help/account/provisioning-profiles/edit-download-or-delete-profiles/)
- [Apple Developer Account Help: Provisioning profile updates](https://developer.apple.com/help/account/provisioning-profiles/provisioning-profile-updates/)
- [Apple: Adding capabilities to your app](https://developer.apple.com/documentation/xcode/adding-capabilities-to-your-app)
- [Apple: Distributing your app to registered devices](https://developer.apple.com/documentation/xcode/distributing-your-app-to-registered-devices)
- [Apple: Build settings reference](https://developer.apple.com/documentation/xcode/build-settings-reference)
- [Apple: APS Environment Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/aps-environment)
- [Apple: Associated Domains Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.associated-domains)
- [Apple Developer Forums: danh sách khoá của -exportOptionsPlist và tên method mới](https://developer.apple.com/forums/thread/773749)
- [Flutter: Build and release an iOS app](https://docs.flutter.dev/deployment/ios)

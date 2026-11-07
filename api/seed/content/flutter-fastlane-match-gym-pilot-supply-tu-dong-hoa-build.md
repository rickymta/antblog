---
slug: flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build
title: "fastlane cho Flutter: match, gym, pilot và supply tự động hoá build Tới Nơi"
summary: "Bài 53: viết Fastfile cho Tới Nơi: match giữ chứng chỉ mã hoá và chỉ đọc trên CI, gym xuất IPA, pilot lên TestFlight, supply lên track internal của Google Play. Đăng nhập bằng App Store Connect API key, bí mật từ ENV, lane chọn theo flavor. Chưa chạy thật."
category: programming
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, android, release, fastlane]
language: vi
---

Người phát hành của Tới Nơi mất khoảng 25 phút (con số minh hoạ) để đưa một bản staging lên TestFlight bằng tay: build IPA, mở Xcode, chọn đúng profile, tải lên, rồi lặp lại cho Android với đúng `versionCode`. Hôm nay, lần thứ ba trong tháng, bản tải lên bị từ chối vì build number trùng bản trước.

**fastlane** gom các bước đó vào một file `Fastfile` chạy lại được: một lệnh cho iOS, một lệnh cho Android, cùng một cách chọn app, flavor và build number. Bài này viết bộ file đó cho hai app Tới Nơi: `match` giữ chứng chỉ mã hoá, `gym` xuất IPA, `pilot` lên TestFlight, `supply` lên Google Play. Đây là bài 53 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết cách ký iOS ở [bài 52](/articles/flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities), ký Android ở [bài 51](/articles/flutter-ky-ung-dung-android-keystore-play-app-signing-aab), flavor khi build ở [bài 50](/articles/flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build) và `--dart-define-from-file` ở [bài 25](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod).

Một lời nói trước: repository này không có tài khoản Apple Developer hay Google Play Console, và máy kiểm không cài fastlane. Vì vậy **chưa lane nào được chạy thật**. Ta kiểm được cú pháp Ruby và nội dung file cấu hình bằng test Dart, không kiểm được việc store chấp nhận bản tải lên.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một job CI chạy <code>flutter build ipa --release</code> và lệnh thoát với mã 0. Ta có chắc thư mục <code>build/ios/ipa</code> đã có file IPA không?</p>
<details><summary>Xem đáp án</summary><p>Không chắc. Trong mã nguồn <code>flutter_tools</code> 3.47.4 (<code>commands/build_ios.dart</code>), nếu bước xuất IPA thất bại sau khi archive thành công, lệnh in hướng dẫn mở Xcode rồi vẫn trả thành công vì archive đã có. Lane phải tự kiểm file mình cần.</p></details>
</div>

## fastlane đứng ở đâu trong chuỗi build

fastlane là công cụ Ruby. Nó không thay `flutter build` mà gọi Flutter, rồi nối kết quả với store. Một `Fastfile` gồm các **lane**; mỗi lane là một chuỗi **action**. Tên ta dùng trong bài là bí danh: `gym` là `build_app`, `pilot` là `upload_to_testflight`, `supply` là `upload_to_play_store` (docs.fastlane.tools). Hình 1 vẽ hai lane của Tới Nơi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 1. Lane ios beta chạy match, flutter build, gym, pilot lên TestFlight. Lane android internal chạy flutter build appbundle, supply lên Google Play internal. Biến môi trường chứa bí mật được match, pilot và supply đọc.">
<defs><marker id="fl53-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl53-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="22" class="t-b">ios beta</text>
<rect x="20" y="34" width="170" height="86" rx="10" class="f-pk-soft s-pk"/><text x="36" y="66" class="t-b">match</text><text x="36" y="92" class="t-small">profile, chỉ đọc</text>
<rect x="240" y="34" width="170" height="86" rx="10" class="f-phys-soft s-phys"/><text x="256" y="66" class="t-b">flutter build</text><text x="256" y="92" class="t-small">tạo xcarchive</text>
<rect x="460" y="34" width="170" height="86" rx="10" class="f-pk-soft s-pk"/><text x="476" y="66" class="t-b">gym</text><text x="476" y="92" class="t-small">xuất IPA</text>
<rect x="680" y="34" width="170" height="86" rx="10" class="f-pk-soft s-pk"/><text x="696" y="66" class="t-b">pilot</text><text x="696" y="92" class="t-small">lên TestFlight</text>
<path d="M192 77 L238 77" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-1-a)"/>
<path d="M412 77 L458 77" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-1-a)"/>
<path d="M632 77 L678 77" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-1-a)"/>
<rect x="20" y="150" width="830" height="62" rx="10" class="f-warn-soft s-warn"/><text x="36" y="176" class="t-b">Bí mật chỉ đến từ biến môi trường, không nằm trong repo</text><text x="36" y="200" class="t-mono">ASC_KEY_ID  ASC_ISSUER_ID  ASC_KEY_CONTENT  MATCH_PASSWORD  PLAY_JSON_KEY_DATA</text>
<path d="M105 148 L105 122" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl53-1-w)"/>
<path d="M765 148 L765 122" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl53-1-w)"/>
<path d="M545 214 L545 250" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl53-1-w)"/>
<text x="20" y="242" class="t-b">android internal</text>
<rect x="240" y="256" width="170" height="86" rx="10" class="f-phys-soft s-phys"/><text x="256" y="288" class="t-b">flutter build</text><text x="256" y="314" class="t-small">appbundle (AAB)</text>
<rect x="460" y="256" width="170" height="86" rx="10" class="f-pk-soft s-pk"/><text x="476" y="288" class="t-b">supply</text><text x="476" y="314" class="t-small">track internal</text>
<rect x="680" y="256" width="170" height="86" rx="10" class="f-phys-soft s-phys"/><text x="696" y="288" class="t-b">Google Play</text><text x="696" y="314" class="t-small">internal testing</text>
<path d="M412 299 L458 299" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-1-a)"/>
<path d="M632 299 L678 299" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-1-a)"/>
</svg>
</div>
<figcaption>Hình 1. Hai lane của Tới Nơi. Các hộp match, gym, pilot, supply là action của fastlane; flutter build và hai cửa hàng nằm ngoài fastlane. Mũi tên đứt là biến môi trường được đọc lúc chạy.</figcaption>
</figure>

Công cụ cần ghim phiên bản. `Gemfile` giới hạn khoảng, `Gemfile.lock` ghi đúng từng gem; tài liệu Flutter về continuous delivery dặn commit cả hai và luôn chạy `bundle exec fastlane`. Ta viết `~> 2.241.2`, chỉ nhận bản vá: `~> 2.241` sẽ nhận cả 2.999.0. Lab không có `Gemfile.lock` vì không chạy được `bundle`; bạn sinh nó ở lần `bundle install` đầu tiên và commit:

### Gemfile · 53-fastlane/native-config/ios/Gemfile

```ruby
source "https://rubygems.org"

gem "fastlane", "~> 2.241.2"
```

Hai điều cần biết về môi trường. Thứ nhất, bản fastlane mới nhất trên rubygems.org (kiểm tại 11/2026) là 2.241.2 và cần Ruby từ 3.2.0, cao hơn mốc 3.1 trong khung bài. Thứ hai, thư mục chạy: theo tài liệu fastlane, mã người dùng trong `Fastfile` chạy trong `./fastlane`, còn mọi action chạy ở gốc dự án, tức thư mục `ios/` hoặc `android/`. Vì thế `sh` phải `cd ../..` để tới gốc Flutter, còn tham số đường dẫn của action viết là `../build/...`.

`Appfile` chỉ giữ Team ID; `app_identifier` để mỗi lane chọn theo app và flavor:

### Appfile · 53-fastlane/native-config/ios/fastlane/Appfile

```ruby
# Appfile iOS của Tới Nơi.
# app_identifier không đặt ở đây: mỗi lane chọn theo app và flavor (xem Fastfile).
# Không có apple_id: đăng nhập bằng App Store Connect API key, không bằng mật khẩu.
team_id(ENV["APPLE_TEAM_ID"])

```

Đầu `Fastfile` có ba hàm nhỏ, chạy trước mọi bước tốn thời gian để lỗi tham số hay thiếu biến hiện ra sau vài giây, không phải sau mười phút build:

### Fastfile · 53-fastlane/native-config/ios/fastlane/Fastfile (trích)

```ruby
# Flavor được phân phối qua cửa hàng. dev chỉ chạy trên máy lập trình viên.
STORE_FLAVORS = %w[staging prod].freeze

def env!(name)
  value = ENV[name].to_s.strip
  UI.user_error!("Thiếu biến môi trường #{name}, xem native-config/.env.example") if value.empty?
  value
end

def pick_target(options)
  app = options[:app].to_s
  flavor = options[:flavor].to_s
  unless STORE_FLAVORS.include?(flavor)
    UI.user_error!("flavor phải là một trong: #{STORE_FLAVORS.join(', ')}")
  end
  bundle_id = APP_IDS["#{app}/#{flavor}"]
  UI.user_error!("app phải là customer hoặc driver") if bundle_id.nil?
  [flavor, bundle_id]
end

def build_number!(options)
  value = options[:build_number].to_s
  UI.user_error!("build_number phải là số nguyên dương") unless value =~ /\A[1-9][0-9]*\z/
  value
end
```

`env!` coi biến rỗng hoặc toàn khoảng trắng là thiếu. `pick_target` từ chối flavor không nằm trong `STORE_FLAVORS`, và bảng `APP_IDS` (sáu dòng, không trích ở đây) chứa đúng Bundle ID của curriculum §3.2. `build_number!` chỉ nhận số nguyên dương, vì App Store Connect đòi mỗi lần tải lên một build number khác (tài liệu Flutter về triển khai iOS) và Play đòi `versionCode` tăng (bài 51).

## Chọn lane theo flavor

Kênh phân phối đi theo flavor, không theo nhánh git. Quyết định đó nên là mã thuần, test được, không nằm rải trong script CI (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Flavor dev không có lane và chạy bằng flutter run. Flavor staging và prod cùng đi qua lane ios beta tới TestFlight và lane android internal tới Google Play internal.">
<defs><marker id="fl53-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl53-2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="20" width="260" height="80" rx="10" class="f-surface s-muted"/><text x="36" y="52" class="t-b">dev</text><text x="36" y="80" class="t-mono">vn.toinoi.customer.dev</text>
<rect x="20" y="150" width="260" height="150" rx="10" class="f-phys-soft s-phys"/><text x="36" y="182" class="t-b">staging và prod</text><text x="36" y="212" class="t-mono">vn.toinoi.customer.staging</text><text x="36" y="236" class="t-mono">vn.toinoi.customer</text><text x="36" y="276" class="t-small">cùng hai lane</text>
<rect x="380" y="30" width="200" height="60" rx="10" class="f-surface s-muted"/><text x="396" y="56" class="t-b">không có lane</text><text x="396" y="78" class="t-small">flutter run</text>
<rect x="380" y="150" width="200" height="60" rx="10" class="f-pk-soft s-pk"/><text x="396" y="176" class="t-b">ios beta</text><text x="396" y="198" class="t-small">gym, pilot</text>
<rect x="380" y="240" width="200" height="60" rx="10" class="f-pk-soft s-pk"/><text x="396" y="266" class="t-b">android internal</text><text x="396" y="288" class="t-small">supply</text>
<rect x="680" y="150" width="180" height="60" rx="10" class="f-phys-soft s-phys"/><text x="696" y="176" class="t-b">TestFlight</text><text x="696" y="198" class="t-small">App Store Connect</text>
<rect x="680" y="240" width="180" height="60" rx="10" class="f-phys-soft s-phys"/><text x="696" y="266" class="t-b">Google Play</text><text x="696" y="288" class="t-small">track internal</text>
<path d="M282 60 L378 60" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl53-2-m)"/>
<path d="M282 190 L378 180" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-2-a)"/>
<path d="M282 260 L378 270" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-2-a)"/>
<path d="M582 180 L678 180" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-2-a)"/>
<path d="M582 270 L678 270" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Chọn lane theo flavor. Nhánh dev có đường đứt và không dẫn tới lane nào; staging và prod dùng chung lane, chỉ khác Bundle ID, cấu hình và build number.</figcaption>
</figure>

Mã Dart của lab biểu diễn bảng đó và các lane:

### release_plan.dart · 53-fastlane/lib/release_plan.dart (trích)

```dart
/// Lane iOS: `match` chỉ đọc, build archive, `gym` xuất IPA, đẩy lên TestFlight.
const LaneSpec iosBeta = LaneSpec(
  platform: StorePlatform.ios,
  lane: 'beta',
  requiredEnv: {
    'ASC_KEY_ID',
    'ASC_ISSUER_ID',
    'ASC_KEY_CONTENT',
    'MATCH_GIT_URL',
    'MATCH_PASSWORD',
  },
);

/// Lane Android: build AAB, đẩy lên track `internal` của Google Play.
const LaneSpec androidInternal = LaneSpec(
  platform: StorePlatform.android,
  lane: 'internal',
  track: 'internal',
  requiredEnv: {'PLAY_JSON_KEY_DATA'},
);
```

```dart
);

/// Lane phân phối của [flavor].
///
```

`dev` trả danh sách rỗng chứ không ném lỗi: một flavor không phân phối là trạng thái hợp lệ, và bài 55 sẽ cho nó một kênh riêng. Với `staging` và `prod`, bước đẩy đầu tiên giống nhau, vì bản gửi App Review hay promote lên track Play đều xuất phát từ một bản đã nằm trên store (bài 56 đến 58). Hai flavor khác nhau ở Bundle ID, file `config/<flavor>.json` và số build. Đây là quyết định thiết kế của lab, không phải luật của store: muốn `prod` đi đường khác, hãy sửa đúng hàm này và test sẽ chỉ ra Fastfile nào lệch.

`planRelease` biến kế hoạch thành lệnh, và từ chối build number không dương ngay cả với `dev`:

### release_plan.dart · 53-fastlane/lib/release_plan.dart (trích)

```dart
  final LaneSpec spec;
  final AppKind app;
  final Flavor flavor;
```

```dart
}

/// Danh sách lệnh phát hành [app] ở [flavor] với [buildNumber].
///
/// [only] giới hạn một cửa hàng (ví dụ CI chạy iOS trên macOS, Android trên
/// ubuntu ở hai job riêng, bài 54). Ném [ArgumentError] khi [buildNumber]
/// không dương: `versionCode` của Play và build number của App Store Connect
/// đều phải là số nguyên lớn hơn 0.
List<LaneInvocation> planRelease({
  required Flavor flavor,
  required AppKind app,
  required int buildNumber,
  StorePlatform? only,
}) {
  if (buildNumber < 1) {
    throw ArgumentError.value(
      buildNumber,
      'buildNumber',
      'phải là số nguyên dương',
    );
  }
  return [
    for (final spec in lanesFor(flavor))
      if (only == null || spec.platform == only)
```

Tham số `only` cho CI chạy iOS trên máy macOS và Android trên ubuntu ở hai job riêng (bài 54). Chạy thử từ thư mục lab:

```bash
dart run bin/plan.dart staging customer 42
```

```text
vn.toinoi.customer.staging:
(cd ios && bundle exec fastlane ios beta app:customer flavor:staging build_number:42)
(cd android && bundle exec fastlane android internal app:customer flavor:staging build_number:42)
```

Dạng `khoá:giá_trị` sau tên lane là cách fastlane truyền tham số; lane đọc chúng qua `options[:flavor]` (docs.fastlane.tools, mục lanes). Test đối chiếu `Fastfile` với `release_plan.dart`: `STORE_FLAVORS` phải bằng tập flavor có lane, Bundle ID phải khớp bảng viết tay trong test.

## Đăng nhập bằng API key, không bằng mật khẩu

Cách cũ là `FASTLANE_USER` và `FASTLANE_PASSWORD`: mật khẩu Apple ID, kéo theo xác thực hai bước. Cách của bài là **App Store Connect API key**: một cặp `key_id`, `issuer_id` và khoá riêng `.p8`, dùng để tạo token truy cập có thời hạn (tham số `duration`). Theo tài liệu fastlane, `pilot` và `match` dùng được API key.

Có hai loại key. Theo Apple, **team key** truy cập mọi app theo vai trò đã chọn, còn **individual key** mang quyền của một người dùng và không dùng được các endpoint Provisioning. Tài liệu fastlane khuyên dùng team key và nói rõ nó cần cho phần liên quan đến provisioning. Apple cũng nhắc hai điều: khoá riêng chỉ tải được một lần, và không được đưa khoá vào kho mã. Lane nhận khoá qua ENV:

### Fastfile · 53-fastlane/native-config/ios/fastlane/Fastfile (trích)

```ruby
def asc_api_key
  app_store_connect_api_key(
    key_id: env!("ASC_KEY_ID"),
    issuer_id: env!("ASC_ISSUER_ID"),
    key_content: env!("ASC_KEY_CONTENT"),
    is_key_content_base64: true,
    in_house: false,
  )
end
```

`key_content` nhận nội dung file `.p8`; vì ENV khó chứa xuống dòng nên ta mã hoá base64 thành một dòng và báo `is_key_content_base64: true`. `.env.example` liệt kê tên biến, mọi giá trị để trống:

### .env.example · 53-fastlane/native-config/.env.example (trích)

```bash
# --- iOS: App Store Connect API key (thay cho Apple ID và mật khẩu) ---
# Key ID của API key tạo ở App Store Connect.
ASC_KEY_ID=
# Issuer ID hiện ở trang Integrations của App Store Connect.
ASC_ISSUER_ID=
# Nội dung file .p8 mã hoá base64 một dòng. macOS: base64 -i AuthKey_XXXX.p8
# Linux (GNU coreutils): base64 -w0 AuthKey_XXXX.p8
ASC_KEY_CONTENT=
```

Trên máy cá nhân, chép thành `ios/.env` và `android/.env`: fastlane chỉ tìm `.env` trong `fastlane/` và thư mục cha của nó, nên `.env` ở gốc app không được nạp. Hai file `gitignore.fragment.example` có các dòng cần thêm vào `ios/.gitignore` và `android/.gitignore` để `.env` không vào repo. Trên CI, đặt vào kho secret của nhà cung cấp, không tạo file. Lệnh `base64 -i` chỉ dành cho macOS; Linux dùng `base64 -w0`. Test giữ ba lớp: không chuỗi nào giống khoá PEM hay token; tham số bí mật chỉ nhận giá trị bắt đầu bằng `env!(` hoặc `ENV[`; tập biến mã đọc bằng đúng tập khai trong `.env.example`.

## match: chứng chỉ mã hoá trong repo riêng

Bài 52 cho thấy mỗi bản phát hành cần một chứng chỉ phân phối và một provisioning profile. Nếu mỗi máy tự tạo, chứng chỉ rải rác và build chỉ chạy trên máy giữ khoá. **match** lưu chứng chỉ và profile vào một nơi chung, đã mã hoá, và mọi máy lấy từ đó (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Máy quản trị chạy sync_certs với readonly false và ghi vào repo riêng. Repo giữ chứng chỉ và profile đã mã hoá bằng MATCH_PASSWORD. CI chạy lane beta với readonly true và chỉ đọc từ repo.">
<defs><marker id="fl53-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl53-3-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="30" width="230" height="110" rx="10" class="f-pk-soft s-pk"/><text x="36" y="62" class="t-b">Máy quản trị</text><text x="36" y="90" class="t-mono">sync_certs</text><text x="36" y="114" class="t-mono">readonly: false</text>
<rect x="330" y="20" width="220" height="130" rx="10" class="f-phys-soft s-phys"/><text x="346" y="52" class="t-b">Repo riêng</text><text x="346" y="80" class="t-small">chứng chỉ và profile</text><text x="346" y="104" class="t-small">mã hoá bằng</text><text x="346" y="128" class="t-mono">MATCH_PASSWORD</text>
<rect x="630" y="30" width="230" height="110" rx="10" class="f-pk-soft s-pk"/><text x="646" y="62" class="t-b">CI hoặc máy khác</text><text x="646" y="90" class="t-mono">ios beta</text><text x="646" y="114" class="t-mono">readonly: true</text>
<path d="M252 85 L328 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl53-3-a)"/><text x="268" y="72" class="t-small">ghi</text>
<path d="M552 85 L628 85" class="f-none s-phys" stroke-width="2" marker-end="url(#fl53-3-p)"/><text x="568" y="72" class="t-small">đọc</text>
<rect x="20" y="190" width="840" height="64" rx="10" class="f-warn-soft s-warn"/><text x="36" y="218" class="t-b">fastlane match nuke thu hồi chứng chỉ và profile của một môi trường</text><text x="36" y="242" class="t-small">Chạy nhầm là phải tạo lại; bản đã lên store vẫn chạy, bản Ad Hoc và Enterprise thì ngừng.</text>
</svg>
</div>
<figcaption>Hình 3. Chỉ máy quản trị được ghi vào kho match. CI chỉ đọc, nên một lỗi cấu hình trên CI không tạo thêm chứng chỉ.</figcaption>
</figure>

Theo tài liệu, nội dung kho git được mã hoá bằng OpenSSL với một passphrase, và `MATCH_PASSWORD` là biến để match giải mã mà không hỏi. Mất mật khẩu này là mất kho. `Matchfile` đặt mặc định an toàn:

### Matchfile · 53-fastlane/native-config/ios/fastlane/Matchfile

```ruby
# Matchfile của Tới Nơi: chứng chỉ và profile nằm trong một repo riêng, mã hoá
# bằng MATCH_PASSWORD. Mặc định chỉ đọc; chỉ lane sync_certs trên máy quản trị
# mới truyền readonly: false.
git_url(ENV["MATCH_GIT_URL"])
storage_mode("git")
type("appstore")
readonly(true)

# Xác thực vào repo match trên CI: HTTPS dùng git_basic_authorization (base64 của
# "tên:token"), deploy key dùng git_private_key. Một trong hai; máy cá nhân có thể
# dùng SSH hoặc credential helper sẵn có nên để trống.
git_basic_authorization(ENV["MATCH_GIT_BASIC_AUTHORIZATION"])
git_private_key(ENV["MATCH_GIT_PRIVATE_KEY"])

```

`readonly` mặc định của match là `false`, tức match được phép tạo chứng chỉ mới; tài liệu khuyên luôn bật `readonly` trên CI. Ta bật ở `Matchfile`, và `setup_ci` trong `before_all` cũng chuyển match sang chỉ đọc, tạo keychain tạm cho CI và chỉ tác dụng khi chạy trên CI (docs.fastlane.tools, `setup_ci`). Trên CI, match còn cần quyền vào chính repo chứng chỉ: `MATCH_GIT_BASIC_AUTHORIZATION` (base64 của `tên:token`, cho HTTPS) hoặc `MATCH_GIT_PRIVATE_KEY` (deploy key). Cả hai có trong `.env.example` và `Matchfile`, không bắt buộc ở máy đã có SSH. Việc ghi chỉ có ở một lane, và lane đó từ chối chạy trên CI:

### Fastfile · 53-fastlane/native-config/ios/fastlane/Fastfile (trích)

```ruby
  desc "Quản trị viên, máy cá nhân: tạo hoặc làm mới chứng chỉ và profile trong kho match."
  lane :sync_certs do |options|
    UI.user_error!("sync_certs chỉ chạy trên máy quản trị, không chạy trên CI") if is_ci
    _flavor, bundle_id = pick_target(options)
    env!("MATCH_GIT_URL")
    env!("MATCH_PASSWORD")
    match(type: "appstore", app_identifier: bundle_id, readonly: false, api_key: asc_api_key)
  end
```

Audit đòi đúng một `readonly: false` trong `Fastfile` iOS; thêm lệnh ghi thứ hai vào `beta` là test đỏ. Nơi lưu có thể là git hoặc S3 (`storage_mode`); `Matchfile` có ví dụ S3 ở dạng chú thích.

## Lane iOS beta: build, gym, pilot

Lane `beta` kiểm tham số, đăng nhập, lấy profile bằng `match`, ép ký bằng profile đó, build archive, rồi xuất IPA và tải lên.

### Fastfile · 53-fastlane/native-config/ios/fastlane/Fastfile (trích)

```ruby
  desc "Đẩy bản build iOS lên TestFlight. Tuỳ chọn ipa:<đường dẫn> để dùng IPA có sẵn."
  lane :beta do |options|
    flavor, bundle_id = pick_target(options)
    api_key = asc_api_key

    ipa = options[:ipa].to_s
    if ipa.empty?
      build_number = build_number!(options)
      env!("MATCH_GIT_URL")
      env!("MATCH_PASSWORD")
      match(type: "appstore", app_identifier: bundle_id, readonly: true, api_key: api_key)

      # Xcconfig của bài 52 cho staging trỏ profile ad-hoc, không tải lên App Store
      # Connect được. Với lane này, ép Release-<flavor> ký bằng profile của match.
      # Lệnh sửa Runner.xcodeproj trong thư mục làm việc: trên máy cá nhân hãy
      # git checkout file đó sau khi chạy.
      update_code_signing_settings(
        use_automatic_signing: false,
        path: "Runner.xcodeproj",
        targets: ["Runner"],
        build_configurations: ["Release-#{flavor}"],
        code_sign_identity: "Apple Distribution",
        bundle_identifier: bundle_id,
        profile_name: lane_context[SharedValues::MATCH_PROVISIONING_PROFILE_MAPPING][bundle_id],
      )

      sh("cd ../.. && flutter build ipa --release --flavor #{flavor} --build-number #{build_number} --dart-define-from-file=config/#{flavor}.json")
      # flutter build ipa trả mã thành công kể cả khi bước xuất IPA lỗi, nên kiểm archive.
      unless File.exist?("../../build/ios/archive/Runner.xcarchive")
        UI.user_error!("Không thấy build/ios/archive/Runner.xcarchive sau flutter build ipa")
      end

      ipa = gym(
        scheme: flavor,
        skip_build_archive: true,
        archive_path: "../build/ios/archive/Runner.xcarchive",
        export_method: "app-store",
        export_options: {
          provisioningProfiles: lane_context[SharedValues::MATCH_PROVISIONING_PROFILE_MAPPING],
        },
        output_directory: "../build/ios/fastlane",
      )
    end

    upload_to_testflight(
      api_key: api_key,
      app_identifier: bundle_id,
      ipa: ipa,
      skip_waiting_for_build_processing: true,
    )
  end
```

Có bốn chi tiết đáng đọc. Một, chữ ký. Xcconfig của bài 52 cho staging trỏ profile ad-hoc, loại dành cho thiết bị đã đăng ký và không dùng để tải lên App Store Connect; lên TestFlight cần profile loại App Store, đúng loại `match(type: "appstore")` lấy về. Vì vậy `update_code_signing_settings` ép cấu hình `Release-<flavor>` ký thủ công bằng profile trong `MATCH_PROVISIONING_PROFILE_MAPPING`, biến lane mà match điền sau khi chạy, không gõ tên profile bằng tay. Lệnh này sửa `Runner.xcodeproj` trong thư mục làm việc; trên máy cá nhân hãy `git checkout` file đó sau khi chạy.

Hai, `flutter build ipa` tạo `build/ios/archive/Runner.xcarchive` (tài liệu Flutter) và còn tự thử xuất IPA theo cách của nó. Ta không dựa vào kết quả đó: lane kiểm archive rồi để `gym` xuất lại với `skip_build_archive: true`. `gym` phải nhận `scheme: flavor`: bài 50 tạo ba scheme, và khi project có nhiều scheme mà không chỉ định, `gym` không biết chọn cái nào. Ba, tuỳ chọn `ipa:` bỏ qua match và build để tải lên một IPA có sẵn; các biến `MATCH_*` và `build_number` chỉ bị đòi trong nhánh build. Bốn, `skip_waiting_for_build_processing: true` không chờ App Store Connect xử lý; tài liệu ghi rõ khi đó việc phân phối cho tester sẽ không diễn ra. Lane chỉ tải lên; nhóm tester, export compliance và Beta App Review thuộc [bài 56](/articles/flutter-testflight-internal-external-testing-beta-review). Audit cấm `distribute_external` trong lane này.

## Lane Android internal: supply

Lane Android không có bước chứng chỉ: khoá ký là upload keystore của bài 51.

### Fastfile · 53-fastlane/native-config/android/fastlane/Fastfile (trích)

```ruby
platform :android do
  desc "Đẩy AAB lên track internal của Google Play."
  lane :internal do |options|
    flavor, package_name = pick_target(options)
    build_number = build_number!(options)
    env!("PLAY_JSON_KEY_DATA") # dừng sớm, trước khi mất vài phút để build

    sh("cd ../.. && flutter build appbundle --release --flavor #{flavor} --build-number #{build_number} --dart-define-from-file=config/#{flavor}.json")
    aab = "build/app/outputs/bundle/#{flavor}Release/app-#{flavor}-release.aab"
    UI.user_error!("Không thấy #{aab} sau flutter build appbundle") unless File.exist?("../../#{aab}")

    upload_to_play_store(
      package_name: package_name,
      track: "internal",
      aab: "../#{aab}",
      json_key_data: env!("PLAY_JSON_KEY_DATA"),
      skip_upload_metadata: true,
      skip_upload_images: true,
      skip_upload_screenshots: true,
      skip_upload_changelogs: true,
    )
  end
```

`track: "internal"` được viết tường minh vì `upload_to_play_store` mặc định là `production` (bảng tham số trong docs.fastlane.tools). Quên dòng này là đẩy thẳng lên người dùng thật. Audit đòi `track: "internal"`, cấm `track_promote_to` và cấm các chuỗi `"production"`, `"beta"`, `"alpha"`; việc promote giữa track thuộc [bài 57](/articles/flutter-google-play-internal-closed-open-production-staged-rollout).

Đường dẫn AAB có flavor theo đúng cách `flutter_tools` 3.47.4 tìm file: `build/app/outputs/bundle/<flavor>Release/app-<flavor>-release.aab`. Lane kiểm file tồn tại trước khi gọi `supply`. `json_key_data` nhận nội dung JSON của service account từ ENV, không phải đường dẫn file; tài liệu Flutter cũng làm vậy cho CI. Bốn cờ `skip_upload_*` giữ lane chỉ đẩy bản nhị phân; mô tả và ảnh cửa hàng để bài 58. Tài liệu fastlane nêu một điều kiện: ứng dụng phải có ít nhất một bản tải lên thành công qua Play Console trước khi `supply` dùng được, nên lần đầu của mỗi app vẫn làm tay. Giới hạn tester của internal testing do Google quy định và đổi theo thời gian; kiểm tại trang trợ giúp Play Console khi phát hành.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với fastlane</span>
<ul>
<li><strong>Tin mã thoát 0 của <code>flutter build ipa</code>.</strong> Dấu hiệu: bước sau báo không thấy IPA dù bước build xanh. Nguyên nhân: Flutter trả thành công khi archive xong nhưng xuất IPA lỗi. Cách sửa: kiểm file trong lane, như lane <code>beta</code>.</li>
<li><strong>Chạy <code>match</code> không chỉ đọc trên CI.</strong> Dấu hiệu: CI tạo thêm chứng chỉ hoặc profile, nhiều máy ghi vào cùng kho. Nguyên nhân: <code>readonly</code> mặc định là <code>false</code>. Cách sửa: <code>readonly(true)</code> trong <code>Matchfile</code>, <code>setup_ci</code>, một lane quản trị duy nhất được ghi.</li>
<li><strong>Quên <code>track</code> khi gọi <code>supply</code>.</strong> Dấu hiệu: bản build xuất hiện ở production thay vì internal. Nguyên nhân: mặc định là <code>production</code>. Cách sửa: luôn truyền <code>track</code>, có test cấm track khác.</li>
<li><strong>Build number hoặc <code>versionCode</code> trùng.</strong> Dấu hiệu: store từ chối bản tải lên. Nguyên nhân: số build lấy tay hoặc từ nguồn không tăng. Cách sửa: <code>build_number</code> là tham số bắt buộc, CI truyền số tăng đơn điệu (bài 54).</li>
<li><strong>Đường dẫn tương đối sai thư mục.</strong> Dấu hiệu: <code>File.exist?</code> hay <code>aab:</code> báo không thấy file dù build xong. Nguyên nhân: mã Fastfile chạy trong <code>fastlane/</code>, action chạy ở thư mục chứa nó. Cách sửa: <code>../..</code> cho mã Fastfile, <code>..</code> cho tham số action.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/53-fastlane
dart pub get
dart analyze
dart test
dart run bin/plan.dart staging customer 42
```

Đã kiểm với Dart 3.13.3 (Flutter 3.47.4): `dart analyze` 0 issue, **75 test xanh**. Mức: **Kiểm chứng cấu hình**. Test không dùng mạng, không cần thiết bị, và đọc file trong `native-config/` theo đường dẫn tương đối.

`release_plan_test.dart` (15 test) chứng minh logic chọn lane: Bundle ID khớp §3.2, `dev` không có lane, `staging` và `prod` cùng đi `ios beta` rồi `android internal`, biến rỗng tính là thiếu, build number 0 và âm bị từ chối. `native_config_test.dart` (28 test) đọc file thật: đủ lane, bí mật chỉ từ ENV, `match` chỉ đọc, ký bằng profile của match trước khi build, `gym` có `scheme`, `supply` đẩy AAB lên `internal`, và `ruby -c` báo `Syntax OK` cho sáu file bằng Ruby 2.6.10 của hệ thống (tự bỏ qua nếu máy không có `ruby`). `audit_rules_test.dart` (32 test) sửa đúng một chỗ của file thật và đòi audit đỏ đúng mã lỗi, ví dụ khoá PEM giả, `gym` thiếu `scheme`, Gemfile `~> 2.241`.

**Chưa làm**, nói thẳng: không cài fastlane và không chạy `bundle install` (Ruby hệ thống 2.6.10 thấp hơn yêu cầu của fastlane 2.241.2), nên tên action và tham số chỉ được đối chiếu với docs.fastlane.tools (11/2026), chưa với bản cài thật. Không chạy `match`, `update_code_signing_settings`, `flutter build ipa`, `gym` hay `flutter build appbundle` với flavor; không tải gì lên TestFlight hay Google Play; chưa kiểm `Release-<flavor>` trong Xcode project thật có nhận profile của match. Test chứng minh file có các tính chất trên, không chứng minh lane chạy được. Hãy chạy lần đầu từng lane một trên máy bạn, trước khi đưa lên CI.

## Bài tập

1. Đồng nghiệp chạy `bundle exec fastlane ios beta app:customer flavor:dev build_number:5`. Lane làm gì, và vì sao?
2. Vì sao `match` trong `beta` có `readonly: true` còn `sync_certs` thì không, và vì sao `sync_certs` kiểm `is_ci`?
3. Ai đó thêm `track: "beta"` vào `upload_to_play_store` để "thử nhanh". Test nào đỏ, và đỏ vì lý do gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Lane dừng ngay với lỗi "flavor phải là một trong: staging, prod". <code>pick_target</code> là dòng đầu tiên và <code>dev</code> không nằm trong <code>STORE_FLAVORS</code>, nên chưa lệnh nào chạm tới Apple hay Flutter.</p>
<p><strong>2.</strong> Build thường xuyên chỉ cần lấy chứng chỉ có sẵn, nên chỉ đọc là đủ và an toàn. Tạo hoặc làm mới cần quyền cao hơn, do một người làm, nên nằm ở lane riêng. Lane đó kiểm <code>is_ci</code> để job CI lỡ gọi nó cũng không ghi được vào kho.</p>
<p><strong>3.</strong> Trong <code>native_config_test.dart</code>, nhóm <code>supply</code> đòi <code>track: "internal"</code> và cấm chuỗi <code>beta</code>; trong <code>audit_rules_test.dart</code>, mã <code>android-track</code> báo cùng lỗi. Cú pháp Ruby vẫn hợp lệ nên <code>ruby -c</code> không bắt được; chỉ test đọc nội dung mới bắt được.</p>
</details>

Bài trước: [Ký ứng dụng iOS: Apple Developer Program, certificate, provisioning profile và capabilities](/articles/flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities). Bài tiếp: [CI/CD build: GitHub Actions macOS/ubuntu, secrets, build number và artifact](/articles/flutter-ci-cd-github-actions-macos-build-ios-android-secrets).

## Tài liệu gốc

- [Continuous delivery with Flutter](https://docs.flutter.dev/deployment/cd) và [Build and release an iOS app](https://docs.flutter.dev/deployment/ios) (docs.flutter.dev).
- [Fastfile và lanes](https://docs.fastlane.tools/advanced/lanes/), [fastlane và thư mục chạy](https://docs.fastlane.tools/advanced/fastlane/) (docs.fastlane.tools).
- [match](https://docs.fastlane.tools/actions/match/), [setup_ci](https://docs.fastlane.tools/actions/setup_ci/) và [build_app (gym)](https://docs.fastlane.tools/actions/build_app/) (docs.fastlane.tools).
- [upload_to_testflight (pilot)](https://docs.fastlane.tools/actions/upload_to_testflight/) và [app_store_connect_api_key](https://docs.fastlane.tools/actions/app_store_connect_api_key/) (docs.fastlane.tools).
- [Using App Store Connect API](https://docs.fastlane.tools/app-store-connect-api/) (docs.fastlane.tools).
- [upload_to_play_store (supply)](https://docs.fastlane.tools/actions/upload_to_play_store/) và [Android setup](https://docs.fastlane.tools/getting-started/android/setup/) (docs.fastlane.tools).
- [Creating API Keys for App Store Connect API](https://developer.apple.com/documentation/appstoreconnectapi/creating-api-keys-for-app-store-connect-api) (developer.apple.com).
- [Set up an open, closed, or internal test](https://support.google.com/googleplay/android-developer/answer/9845334) (support.google.com/googleplay).

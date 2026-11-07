---
slug: flutter-testflight-internal-external-testing-beta-review
title: "TestFlight: internal và external testing, Beta App Review và export compliance cho Tới Nơi"
summary: "Bài 56: đưa bản staging của Tới Nơi lên TestFlight. Tải lên bằng Xcode, Transporter hay pilot, đọc trạng thái build, nhóm internal tới 100 người và external tới 10.000, Beta App Review, export compliance, hạn 90 ngày, feedback và crash. Chưa chạy với tài khoản thật."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, release, testflight]
language: vi
---

Thứ Sáu, bản staging của app tài xế Tới Nơi lên được App Store Connect. Thứ Bảy, chị PM mở TestFlight và thấy build ở trạng thái **Missing Compliance**: chưa ai trả lời câu hỏi về mã hoá. Thứ Hai, mười hai tài xế thử nghiệm bên ngoài công ty là tester external, nên build còn phải qua **Beta App Review**. Tình huống là minh hoạ, nhưng cả hai trở ngại đều có thật.

Bài này đi hết đường của một build lên TestFlight: tải lên, trạng thái, nhóm internal và external, Beta App Review, export compliance, hạn 90 ngày, feedback và crash. Lab `56-testflight` là Dart thuần: `BetaChecklist` kiểm `Info.plist` trước khi tải lên, `TesterGroups` kiểm kế hoạch nhóm tester, cùng vài công cụ nhỏ cho thông tin test và trạng thái build.

Đây là bài 56 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết cách phân phối nội bộ ở [bài 55](/articles/flutter-firebase-app-distribution-phan-phoi-noi-bo-tester), lệnh `pilot` ở [bài 53](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build) và profile ký iOS ở [bài 52](/articles/flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities). Mọi con số của Apple trong bài kiểm tại 11/2026; chính sách đổi theo thời gian, hãy kiểm lại trước khi dựa vào.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Repository không có tài khoản Apple Developer và máy viết bài không build được IPA. Vì vậy <strong>chưa có build nào được tải lên TestFlight</strong>, chưa chạy <code>pilot</code>, chưa qua Beta App Review. Chỉ có test Dart chạy được. Nhận định về Apple lấy từ trang chính thức của developer.apple.com (kiểm tại 11/2026).</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Năm nhân viên Tới Nơi có vai trò trong tài khoản Apple Developer; mười hai tài xế thì không. Build mới tải lên có cần chờ Beta App Review trước khi mỗi nhóm test không?</p>
<details><summary>Xem đáp án</summary><p>Nhóm nội bộ thì không: Ready to Submit nghĩa là build phân phối được cho tester internal ngay, hoặc gửi review để test external. Tài xế bên ngoài thì có: build đầu tiên của mỗi version gửi cho nhóm external phải qua review đầy đủ, còn các bản sau của cùng version thì "có thể" không cần (Apple viết "might not", không hứa).</p></details>
</div>

## Đường đi của một build

Hình 1 vẽ đường đi từ IPA tới tester. Hai điểm dễ vấp: hộp compliance và nhánh review.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 470" role="img" aria-label="Hình 1. IPA ký bằng profile App Store được tải lên bằng Xcode, Transporter hoặc pilot, Apple xử lý, rồi tới bước export compliance. Khi đã khai, build ở trạng thái Ready to Submit. Từ đó build đi tới nhóm internal không qua review, hoặc qua Beta App Review rồi tới nhóm external. Mọi build hết hạn sau 90 ngày.">
<defs><marker id="fl56-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl56-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="15" y="30" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="31" y="62" class="t-b">IPA staging</text><text x="31" y="90" class="t-small">profile App Store</text>
<rect x="245" y="30" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="261" y="62" class="t-b">Tải lên</text><text x="261" y="86" class="t-small">Xcode · Transporter</text><text x="261" y="106" class="t-small">altool · pilot</text>
<rect x="475" y="30" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="491" y="62" class="t-b">Xử lý bởi Apple</text><text x="491" y="90" class="t-small">có email khi xong</text>
<rect x="705" y="30" width="160" height="90" rx="10" class="f-warn-soft s-warn"/><text x="721" y="62" class="t-b">Export</text><text x="721" y="82" class="t-b">compliance</text><text x="721" y="106" class="t-small">đã khai chưa?</text>
<path d="M207 75 L243 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-1-a)"/>
<path d="M437 75 L473 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-1-a)"/>
<path d="M667 75 L703 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-1-a)"/>
<text x="480" y="162" class="t-small t-warn">đã khai: đi thẳng; chưa: Missing Compliance</text>
<path d="M785 122 L785 142 L455 142 L455 168" class="f-none s-warn" stroke-width="2" marker-end="url(#fl56-1-w)"/>
<rect x="335" y="170" width="240" height="70" rx="10" class="f-pk-soft s-pk"/><text x="351" y="198" class="t-b">Ready to Submit</text><text x="351" y="222" class="t-small">gán nhóm hoặc gửi review</text>
<rect x="15" y="300" width="250" height="80" rx="10" class="f-phys-soft s-phys"/><text x="31" y="330" class="t-b">Nhóm internal, tới 100</text><text x="31" y="356" class="t-small">không qua Beta App Review</text>
<rect x="330" y="300" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="346" y="330" class="t-b">Beta App Review</text><text x="346" y="356" class="t-small">bản đầu của mỗi version</text>
<rect x="645" y="300" width="220" height="80" rx="10" class="f-phys-soft s-phys"/><text x="661" y="330" class="t-b">Nhóm external</text><text x="661" y="356" class="t-small">tới 10.000 mỗi app</text>
<path d="M390 242 L390 270 L140 270 L140 298" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-1-a)"/>
<path d="M480 242 L480 298" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-1-a)"/>
<path d="M582 340 L643 340" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-1-a)"/>
<rect x="15" y="410" width="850" height="46" rx="10" class="f-warn-soft s-warn"/><text x="31" y="438" class="t-b">Mỗi build hết hạn sau 90 ngày: trạng thái Expired, tester không dùng được nữa</text>
</svg>
</div>
<figcaption>Hình 1. Đường đi của một build. Mũi tên vàng là nhánh chờ compliance; nhóm external phải qua Beta App Review.</figcaption>
</figure>

Có bốn cách tải IPA lên. **Xcode**: Validate App rồi Distribute App từ archive. **Transporter**: app macOS, kéo file IPA vào. **`xcrun altool --upload-app`**: chạy được trên CI với API key. **`fastlane pilot`** (bài 53): gói bước tải lên vào một lane. Theo trang Upload builds của Apple, app iOS phải build bằng Xcode 26 trở lên; tài liệu Flutter nói mỗi lần tải cần một build number khác. Lệnh dưới đây chưa chạy ở đây, vì cần chứng chỉ và tài khoản của bạn:

```bash
flutter build ipa --release --flavor staging --build-name 1.4.0 --build-number 43 \
  --dart-define-from-file=config/staging.json \
  --export-options-plist=ExportOptions-staging-testflight.plist
xcrun altool --upload-app --type ios -f build/ios/ipa/*.ipa \
  --apiKey "$ASC_KEY_ID" --apiIssuer "$ASC_ISSUER_ID"
```

Build cho TestFlight ký bằng profile loại **App Store**, không phải ad hoc của bài 55. Bài 52 để việc này làm bài tập; lab có sẵn file `ExportOptions` cho staging:

### ExportOptions-staging-testflight.plist.example · 56-testflight/native-config/ios/ExportOptions-staging-testflight.plist.example (trích)

```xml
    <key>method</key>
    <string>app-store-connect</string>
    <!-- ... -->
    <key>provisioningProfiles</key>
    <dict>
        <key>vn.toinoi.customer.staging</key>
        <string>ToiNoi Customer Staging AppStore</string>
    </dict>
```

Theo Apple, profile thiếu application identifier thì build ở trạng thái Not Available for Testing. Sau khi tải lên, Apple xử lý build và gửi email khi xong; tài liệu Flutter nói thường trong vòng 30 phút. Trang App build statuses liệt kê mười hai trạng thái, lab giữ chúng trong `BuildStatus`:

| Trạng thái | Việc cần làm |
|---|---|
| Missing Compliance | Trả lời câu hỏi, hoặc khai khoá trong Info.plist |
| Ready to Submit | Gán nhóm internal, hoặc gửi review |
| Waiting for Review, In Beta Review | Chờ |
| Ready to Test, Testing | Thêm nhóm vào build |
| Rejected, Invalid Binary, Expired | Sửa rồi tải build mới |

`BuildStatus.parse` đọc đúng tên Apple và ném `FormatException` với tên lạ, kể cả "Processing", vì trạng thái đó không có trong trang của Apple.

## Kiểm Info.plist trước khi tải lên

Một bản tải lên hỏng tốn một vòng build và xử lý. `BetaChecklist` đọc `Info.plist` đã chuyển thành map và bắt trước những lỗi ta biết chắc. Hình 2 cho thấy ai đọc mỗi khoá.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 410" role="img" aria-label="Hình 2. Các khoá trong Info.plist và entitlements và nơi đọc chúng. Hai khoá mô tả vị trí cho hộp thoại xin vị trí. UIBackgroundModes cho chạy nền. aps-environment cho APNs. ITSAppUsesNonExemptEncryption cho câu hỏi export compliance. CFBundleVersion cho định danh bản tải lên. Thông báo push không có khoá mô tả.">
<defs><marker id="fl56-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="15" y="15" width="420" height="70" rx="10" class="f-pk-soft s-pk"/><text x="31" y="42" class="t-mono">NSLocationWhenInUseUsageDescription</text><text x="31" y="66" class="t-mono">NSLocationAlwaysAndWhenInUseUsageDescription</text>
<rect x="540" y="15" width="325" height="70" rx="10" class="f-phys-soft s-phys"/><text x="556" y="42" class="t-b">Hộp thoại xin vị trí</text><text x="556" y="66" class="t-small">người dùng đọc chuỗi này</text>
<rect x="15" y="105" width="420" height="56" rx="10" class="f-pk-soft s-pk"/><text x="31" y="130" class="t-mono">UIBackgroundModes</text><text x="31" y="150" class="t-small">location · remote-notification</text>
<rect x="540" y="105" width="325" height="56" rx="10" class="f-phys-soft s-phys"/><text x="556" y="130" class="t-b">Chạy nền</text><text x="556" y="150" class="t-small">gửi vị trí, nhận push</text>
<rect x="15" y="181" width="420" height="56" rx="10" class="f-pk-soft s-pk"/><text x="31" y="206" class="t-mono">aps-environment</text><text x="31" y="226" class="t-small">nằm ở entitlements, không ở Info.plist</text>
<rect x="540" y="181" width="325" height="56" rx="10" class="f-phys-soft s-phys"/><text x="556" y="206" class="t-b">APNs</text><text x="556" y="226" class="t-small">capability Push Notifications</text>
<rect x="15" y="257" width="420" height="56" rx="10" class="f-pk-soft s-pk"/><text x="31" y="282" class="t-mono">ITSAppUsesNonExemptEncryption</text><text x="31" y="302" class="t-small">true hoặc false, kiểu Boolean</text>
<rect x="540" y="257" width="325" height="56" rx="10" class="f-phys-soft s-phys"/><text x="556" y="282" class="t-b">App Store Connect</text><text x="556" y="302" class="t-small">bỏ câu hỏi compliance mỗi lần</text>
<rect x="15" y="333" width="420" height="56" rx="10" class="f-pk-soft s-pk"/><text x="31" y="358" class="t-mono">CFBundleVersion</text><text x="31" y="378" class="t-small">= $(FLUTTER_BUILD_NUMBER)</text>
<rect x="540" y="333" width="325" height="56" rx="10" class="f-phys-soft s-phys"/><text x="556" y="358" class="t-b">Định danh bản tải lên</text><text x="556" y="378" class="t-small">lab đòi phải tăng</text>
<path d="M437 50 L538 50" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-2-a)"/>
<path d="M437 133 L538 133" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-2-a)"/>
<path d="M437 209 L538 209" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-2-a)"/>
<path d="M437 285 L538 285" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-2-a)"/>
<path d="M437 361 L538 361" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Mỗi khoá và nơi đọc nó. Hộp thoại xin quyền thông báo không có khoá mô tả nào.</figcaption>
</figure>

**Vị trí.** Apple ghi `NSLocationWhenInUseUsageDescription` là bắt buộc khi app đọc vị trí lúc đang mở. Cả hai app chỉ khai quyền "khi đang dùng", không khai `NSLocationAlwaysAndWhenInUseUsageDescription` (bài 38 đã bỏ quyền Always). App tài xế thêm `UIBackgroundModes` có `location` vì vị trí phải tiếp tục gửi khi tài xế chuyển sang app dẫn đường. App khách khai quyền nền thì bị cảnh báo: đó là thêm một điều phải giải thích với App Review.

**Push.** Trang Apple về xin quyền thông báo chỉ nêu lời gọi `requestAuthorization`, không có khoá mô tả nào trong Info.plist. Cái cần là entitlement `aps-environment` (capability Push Notifications) và, để app được đánh thức xử lý background notification như bài 37, `UIBackgroundModes` có `remote-notification`. Đó là cách lab hiểu "đủ khai báo cho push".

### Info.plist · 56-testflight/native-config/driver/ios/Info.plist (trích)

```xml
	<key>ITSAppUsesNonExemptEncryption</key>
	<false/>

	<key>NSLocationWhenInUseUsageDescription</key>
	<string>Tới Nơi dùng vị trí của bạn để ghép chuyến gần nhất. Trong chuyến đang chạy, app tiếp tục gửi vị trí khi bạn chuyển sang app dẫn đường để khách thấy xe đang tới. Ngoài chuyến, app không thu vị trí.</string>
	<!-- ... -->
	<key>UIBackgroundModes</key>
	<array>
		<string>location</string>
		<string>remote-notification</string>
	</array>
```

**Số build.** Template Flutter ghi `CFBundleVersion` là `$(FLUTTER_BUILD_NUMBER)`, nên checklist thay biến trước khi so. Apple định nghĩa khoá này là một đến ba số nguyên cách nhau bằng dấu chấm, phần từ số thứ tư bị hệ thống bỏ qua, nên lab coi bốn thành phần là sai.

### beta_checklist.dart · 56-testflight/lib/src/beta_checklist.dart (trích)

```dart
String? resolveBuildSettings(String value, Map<String, String> settings) {
  var unresolved = false;
  final result = value.replaceAllMapped(RegExp(r'\$[({]([A-Za-z0-9_]+)[)}]'), (
    m,
  ) {
    final v = settings[m.group(1)!];
    if (v == null || v.isEmpty) {
      unresolved = true;
      return m.group(0)!;
    }
    return v;
  });
  return unresolved ? null : result;
}
```

```dart
      if (compareBuilds(parsed, lastParsed) <= 0) {
        issues.add(
          CheckIssue(
            'build-not-increasing',
            Severity.error,
            'CFBundleVersion',
            'Build $build không lớn hơn bản đã tải lên ($last). Mỗi lần tải '
                'lên cần build number khác; lab đòi thêm là phải tăng, để '
                'khớp với versionCode của Android và run_number của CI.',
          ),
        );
      }
```

Apple đòi tối thiểu là build khác nhau; "phải tăng" là luật của lab, cùng tinh thần với `versionCode` ở bài 51. `compareBuilds` so theo số, nên 10 lớn hơn 9 và 1.10 lớn hơn 1.9. Công cụ dòng lệnh gói checklist lại:

```bash
dart run bin/beta_check.dart driver native-config/driver/ios/Info.plist \
  --build-name=1.4.0 --build-number=42 --last-build=42
```

```text
error build-not-increasing [CFBundleVersion]: Build 42 không lớn hơn bản đã tải lên (42). ...
1 lỗi, 0 cảnh báo.
```

Mã thoát là 0 khi không có lỗi (cảnh báo vẫn in), 1 khi có lỗi, 2 khi dùng sai. Checklist chỉ đọc file, không thấy mã đã biên dịch: `Package.swift` của `geolocator_apple` ghi rằng app chỉ-khi-đang-dùng có thể gặp lỗi ITMS-90683 khi tải lên, trừ khi build với `BYPASS_PERMISSION_LOCATION_ALWAYS=1` (bài 35). Checklist sạch chưa đảm bảo bản tải lên sạch.

## Export compliance và ITSAppUsesNonExemptEncryption

Apple coi app có dùng mã hoá là phần mềm xuất khẩu, nên chịu quy định xuất khẩu của Mỹ và quy định nhập khẩu của nước phân phối. Khoá `ITSAppUsesNonExemptEncryption` trả lời ngay trong file: `NO` nghĩa là app, kể cả thư viện bên thứ ba, không dùng mã hoá hoặc chỉ dùng loại được miễn tài liệu; `YES` là ngược lại, thường kèm `ITSEncryptionExportComplianceCode`, mã Apple cấp sau khi duyệt tài liệu. Thiếu khoá, App Store Connect hỏi bộ câu hỏi ở mỗi bản tải lên, và build dừng ở Missing Compliance cho tới khi có người trả lời.

Lab dùng `false` cho cả hai app, và ba điều cần nói thẳng. Một, ví dụ miễn trừ trong tài liệu Apple là HTTPS qua `URLSession`, tức mã hoá có sẵn trong hệ điều hành. Hai, `dio` mặc định dùng `HttpClient` của `dart:io` (đọc trong mã nguồn dio 5.11.1), và theo thông báo của nhóm Dart, `dart:io` dùng BoringSSL cho TLS từ năm 2015, nên đường đó không đi qua `URLSession`. Ba, Apple nói trách nhiệm phân loại là của nhà phát triển, và app dùng mã hoá được miễn vẫn có thể phải nộp báo cáo tự phân loại hằng năm cho chính phủ Mỹ.

Vì vậy "app chỉ dùng HTTPS thì khai `NO`" là một lập trường cần chủ sản phẩm hoặc người phụ trách pháp lý xác nhận, **không phải kết luận của bài, cũng không phải tư vấn pháp lý**. Lab chỉ kiểm tính nhất quán của khai báo:

### beta_checklist.dart · 56-testflight/lib/src/beta_checklist.dart (trích)

```dart
    final value = plist[_encryption];
    if (value is! bool) {
      issues.add(
        const CheckIssue(
          'encryption-key-type',
          Severity.error,
          _encryption,
          'Giá trị phải là <true/> hoặc <false/>, không phải chuỗi "NO"/"YES".',
        ),
      );
      return;
    }
    final code = plist[_encryptionCode];
    final hasCode = code is String && code.trim().isNotEmpty;
    if (value && !hasCode) {
      // ... encryption-code-missing (error)
    } else if (!value && plist.containsKey(_encryptionCode)) {
      // ... encryption-code-unused (warning)
    }
```

Khoá thiếu là cảnh báo, kiểu sai là lỗi, `true` không có mã là lỗi, `false` mà vẫn có mã là cảnh báo mâu thuẫn. Build đã ở Missing Compliance thì trả lời ngay trong TestFlight (nút Manage của build), cần vai trò Account Holder, Admin hoặc App Manager.

## Nhóm tester: internal, external và public link

Mỗi app trên App Store Connect có nhóm riêng, nên app khách và app tài xế có hai kế hoạch (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 3. Nhóm internal gồm người dùng App Store Connect, tới 100 người, năm vai trò, không qua Beta App Review. Nhóm external tới 10.000 tester mỗi app, qua Beta App Review ở bản đầu của mỗi version. Mũi tên từ nhóm external sang nhóm internal: nhóm external đòi nhóm internal có trước. Nhóm external mời bằng email, biết tên tester, hoặc bằng public link, ẩn danh.">
<defs><marker id="fl56-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl56-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="20" width="360" height="170" rx="10" class="f-phys-soft s-phys"/><text x="36" y="50" class="t-b">Nhóm internal</text><text x="36" y="78" class="t-small">người dùng App Store Connect, tới 100</text><text x="36" y="100" class="t-small">Account Holder, Admin, App Manager,</text><text x="36" y="120" class="t-small">Developer hoặc Marketing</text><text x="36" y="148" class="t-small">không qua Beta App Review</text><text x="36" y="170" class="t-small">dùng được bản Internal Only của Xcode</text>
<rect x="500" y="20" width="360" height="170" rx="10" class="f-pk-soft s-pk"/><text x="516" y="50" class="t-b">Nhóm external</text><text x="516" y="78" class="t-small">tới 10.000 tester mỗi app</text><text x="516" y="100" class="t-small">Beta App Review ở bản đầu</text><text x="516" y="120" class="t-small">của mỗi version</text><text x="516" y="148" class="t-small">cần Test Information đầy đủ</text>
<path d="M498 105 L382 105" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl56-3-w)"/>
<text x="396" y="90" class="t-small t-warn">cần có trước</text>
<rect x="400" y="250" width="220" height="100" rx="10" class="f-pk-soft s-pk"/><text x="416" y="280" class="t-b">Mời bằng email</text><text x="416" y="306" class="t-small">biết ai là ai</text><text x="416" y="328" class="t-small">tài xế thử nghiệm</text>
<rect x="650" y="250" width="215" height="100" rx="10" class="f-warn-soft s-warn"/><text x="666" y="280" class="t-b">Public link</text><text x="666" y="306" class="t-small">ẩn danh, lọc thiết bị</text><text x="666" y="328" class="t-small">khách thử nghiệm</text>
<path d="M640 192 L640 220 L510 220 L510 248" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-3-a)"/>
<path d="M720 192 L720 220 L757 220 L757 248" class="f-none s-pk" stroke-width="2" marker-end="url(#fl56-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Hai loại nhóm của một app. Mũi tên vàng đứt: phải có nhóm internal trước khi tạo nhóm external. Mũi tên xanh: hai cách mời vào nhóm external.</figcaption>
</figure>

Theo Apple, internal là người dùng App Store Connect trong đội, tối đa 100 người, giữ một trong năm vai trò ở hình; external tối đa 10.000 mỗi app. Public link mở cho mọi người hoặc lọc theo thiết bị và phiên bản hệ điều hành, có thể đặt giới hạn từ 1 đến 10.000 tester. Link ai cũng chia sẻ được và người vào bằng link hiện ẩn danh. Vì vậy ta mời tài xế từng người bằng email, còn khách thử dùng link:

### tester_groups.dart · 56-testflight/lib/src/tester_groups.dart (trích)

```dart
  factory TesterGroups.driverBeta() => TesterGroups([
    _internalGroup(),
    TesterGroup.external(
      name: 'Tài xế thử nghiệm',
      externalEmails: {
        for (var i = 1; i <= 12; i++)
          'taixe${i.toString().padLeft(2, '0')}@toinoi.example',
      },
    ),
  ]);

  /// App khách: nhóm nội bộ và một public link lọc theo thiết bị. Người vào
  /// bằng link hiện ẩn danh, chấp nhận được với khách thử.
  factory TesterGroups.customerBeta() => TesterGroups([
    _internalGroup(),
    const TesterGroup.external(
      name: 'Khách thử nghiệm',
      publicLink: PublicLink.filtered(
        criteria: ['iPhone, iOS 17.0 trở lên'],
        testerLimit: 100,
      ),
    ),
  ]);
```

Mười hai tài xế và giới hạn 100 khách là số minh hoạ. `validate()` kiểm các luật vừa nêu:

```dart
    if (link == null) return;
    final limit = link.testerLimit;
    if (limit != null && (limit < 1 || limit > externalLimit)) {
      issues.add(
        GroupIssue(
          'public-link-limit-range',
          g.name,
          'Giới hạn của public link phải từ 1 đến $externalLimit, nhận $limit.',
        ),
      );
    }
    if (link.mode == LinkMode.filterByCriteria && link.criteria.isEmpty) {
      issues.add(
        GroupIssue(
          'public-link-no-criteria',
          g.name,
          'Link lọc theo tiêu chí cần ít nhất một tiêu chí.',
        ),
      );
    }
```

`validate()` còn bắt người thứ 101 của internal, vai trò ngoài năm vai trò trên, external khi chưa có internal, tên nhóm trùng và email sai dạng. Con số 100 là của từng nhóm hay cả app thì trang Apple viết không rõ, nên lab đếm số người khác nhau trong mọi nhóm internal của một app, theo hướng chặt hơn.

## Beta App Review và thông tin test

Thêm build vào một nhóm external là gửi nó cho Beta App Review. Bản đầu của mỗi version cần review đầy đủ; các bản sau của cùng version có thể không. Apple thêm hai giới hạn: tối đa sáu build gửi review trong 24 giờ, và mỗi version chỉ một build đang review tại một thời điểm. Lab biểu diễn cả hai:

### beta_review.dart · 56-testflight/lib/src/beta_review.dart (trích)

```dart
  final windowStart = now.subtract(const Duration(hours: 24));
  final recent =
      history.where((s) => s.submittedAt.isAfter(windowStart)).toList()
        ..sort((a, b) => a.submittedAt.compareTo(b.submittedAt));
  if (recent.length >= maxBetaSubmissionsPer24h) {
    // Mốc giải phóng một chỗ: bản cũ nhất cần thiết để còn dưới giới hạn.
    final oldestBlocking =
        recent[recent.length - maxBetaSubmissionsPer24h].submittedAt;
    retryAt = oldestBlocking.add(const Duration(hours: 24));
```

Hàm trả thêm `retryAt`, thời điểm sớm nhất gửi lại được; test kiểm rằng đúng mốc đó thì gửi được và sớm một giây thì chưa. Build bị Rejected không dùng lại được trong TestFlight: bạn sửa rồi tải build mới, hoặc kháng nghị với TestFlight App Review.

Trước khi gửi, App Store Connect đòi **Test Information**: mô tả beta (bắt buộc), email nhận phản hồi, và thông tin review. Trang App Review information của Apple nêu liên hệ (số điện thoại dạng quốc tế, có dấu +) và tài khoản demo nếu app đòi đăng nhập; tài khoản đó không được hết hạn. Tới Nơi đòi đăng nhập nên cần tài khoản demo. Trang đó nói về bản App Store; danh sách trường riêng của TestFlight hãy đối chiếu ngay trên form. File mẫu của lab dùng giá trị giữ chỗ, vì mật khẩu thật không bao giờ nằm trong repo:

### test-information.json.example · 56-testflight/native-config/test-information.json.example (trích)

```json
  "contactPhone": "<số điện thoại quốc tế, bắt đầu bằng +>",
  "requiresSignIn": true,
  "demoAccountName": "<tài khoản demo, tạo riêng cho người review>",
  "demoAccountPassword": "<điền trực tiếp trong App Store Connect, không commit>",
```

`TestInformation.validateForExternal()` coi trường rỗng hoặc còn dạng `<...>` là chưa điền; test chứng minh file mẫu đỏ đúng bảy trường, điền đủ thì hết lỗi. Tester còn đọc **What to Test** trong app TestFlight, nên văn bản đó phải nói rõ bản nào và việc cần thử. `buildWhatToTest` sinh nó, và cước trong kịch bản lấy từ `calculateFare(5)` của `toi_noi_core`, không gõ tay:

```text
Bản 1.4.0 (build 43).
Việc cần thử:
1. Bật online, nhận một cuốc thử 5 km (cước 25.500 VND).
2. Chuyển sang app dẫn đường giữa chuyến, quay lại sau một phút: vị trí vẫn được gửi cho khách.
3. Hoàn thành chuyến và kiểm tra cước hiển thị cho tài xế.
Lỗi đã biết, không cần báo lại:
- Ảnh đại diện tài xế chưa tải được
Báo lỗi: chụp màn hình ngay trong TestFlight để kèm ảnh và mô tả.
```

Với `pilot`, lane bổ sung của lab truyền mô tả beta, email phản hồi và thông tin review từ biến môi trường, liệt kê ở đầu file. Lane đặt thành file `import` riêng, không dán vào Fastfile của bài 53, vì audit của lab đó cấm `distribute_external`. Hai điều cần để ý: không đặt `skip_waiting_for_build_processing`, vì với giá trị true tài liệu fastlane nói `distribute_external` không chạy; và không đặt `uses_non_exempt_encryption`, vì tham số này chỉ dùng khi Info.plist thiếu khoá. Theo tài liệu, vai trò Developer chỉ tải lên được; cập nhật thông tin build và tester cần App Manager hoặc Admin.

### Fastfile.beta-external.example · 56-testflight/native-config/ios/fastlane/Fastfile.beta-external.example (trích)

```ruby
    upload_to_testflight(
      api_key: asc_api_key,
      app_identifier: bundle_id,
      ipa: ipa,
      changelog: File.read(changelog_file),
      beta_app_description: beta_description,
      beta_app_feedback_email: feedback_email,
      distribute_external: true,
      groups: [group],
      notify_external_testers: true,
      demo_account_required: true,
      beta_app_review_info: review_info,
    )
```

## Hết hạn, feedback và crash

Apple cho test một build tối đa 90 ngày; sau đó build chuyển sang Expired và không dùng được. Trang ta đọc không nói mốc 90 ngày bắt đầu từ đâu. `BuildExpiry` tính từ lúc tải lên, mốc sớm nhất có thể, để cảnh báo sớm; số ngày thật xem trong App Store Connect.

### beta_review.dart · 56-testflight/lib/src/beta_review.dart (trích)

```dart
  /// Số ngày còn lại, làm tròn lên; 0 khi đã hết hạn.
  static int daysLeft(DateTime uploadedAt, DateTime now) {
    final left = expiresAt(uploadedAt).difference(now);
    if (left <= Duration.zero) return 0;
    return (left.inSeconds / Duration.secondsPerDay).ceil();
  }
```

Còn một giây vẫn là còn một ngày; đúng thời điểm hết hạn là 0. `isExpiringSoon` báo khi còn từ 1 đến 14 ngày, con số 14 do lab chọn.

Phản hồi đi theo hai đường. Tester dùng TestFlight 2.3 trở lên gửi ảnh chụp màn hình có ghi chú, nhận xét về crash và góp ý chung; chúng hiện ở mục Feedback của tab TestFlight, và báo cáo crash tải về được trong 120 ngày. Bản cũ hơn gửi qua email phản hồi trong Test Information. Mỗi build có số lượt cài, số phiên trong bảy ngày và số crash. Bạn tắt được phản hồi trong app cho một nhóm; tester vẫn gửi được qua email. Crashlytics và Sentry ở bài 48 là kênh khác, không thay thế kênh này.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với TestFlight</span>
<ul>
<li><strong>Build kẹt ở Missing Compliance mỗi lần tải lên.</strong> Dấu hiệu: bản nào cũng chờ có người bấm trả lời. Nguyên nhân: Info.plist thiếu <code>ITSAppUsesNonExemptEncryption</code>. Cách sửa: khai khoá sau khi chủ sản phẩm chốt câu trả lời; <code>encryption-key-missing</code> bắt ca này.</li>
<li><strong>Không gửi được cho tester external.</strong> Dấu hiệu: App Store Connect đòi thêm thông tin trước khi gửi review. Nguyên nhân: mô tả beta hoặc thông tin review còn trống, hoặc thiếu tài khoản demo. Cách sửa: điền Test Information; <code>validateForExternal</code> liệt kê trường thiếu.</li>
<li><strong>Build number trùng hoặc không tăng.</strong> Dấu hiệu: bản tải lên bị từ chối. Nguyên nhân: số build gõ tay hoặc lấy từ nguồn không tăng. Cách sửa: CI truyền số tăng đơn điệu (bài 54); <code>build-not-increasing</code> bắt sớm.</li>
<li><strong>Thiếu hoặc để trống mô tả quyền.</strong> Dấu hiệu: email báo lỗi sau khi tải lên, ví dụ ITMS-90683 với app khách. Nguyên nhân: khoá mô tả thiếu, hoặc mã đã biên dịch của plugin tham chiếu API cần khoá đó. Cách sửa: khai đủ khoá, và với app khách build có cờ bỏ quyền Always của <code>geolocator_apple</code>.</li>
<li><strong>Build hết hạn giữa đợt thử.</strong> Dấu hiệu: trạng thái Expired, tester không mở được bản cài. Nguyên nhân: đợt thử dài hơn 90 ngày mà không tải bản mới. Cách sửa: theo dõi <code>BuildExpiry</code> và tải build mới trước hạn.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/56-testflight
dart pub get
dart analyze
dart test
dart run bin/beta_check.dart customer native-config/customer/ios/Info.plist \
  --build-name=1.4.0 --build-number=43 --last-build=42 \
  --entitlements=native-config/ios/Runner.entitlements.example
```

Đã kiểm với Dart 3.13.3 (Flutter 3.47.4): `dart analyze` 0 issue, **128 test xanh**, lệnh cuối thoát mã 0. Mức: **Kiểm chứng cấu hình**. Test không dùng mạng, không cần thiết bị, và thời điểm luôn được truyền vào, không đọc đồng hồ thật.

Test của checklist chứng minh từng luật: mô tả vị trí thiếu, rỗng hoặc còn giữ chỗ, khác biệt khách và tài xế, push, khoá mã hoá, thay biến build, và so số build ở các ca biên (10 so với 9, 1.10 so với 1.9, "043" so với 43). Test nhóm tester chứng minh các biên 100, 101, 10.000, 10.001; test review chứng minh mốc đúng 24 giờ, `retryAt` và hạn 90 ngày. `native_config_test.dart` đọc file thật trong `native-config/` và chạy `ruby -c` trên lane mẫu (bỏ qua khi máy không có Ruby). Ta cũng thử đổi bảy chỗ trong mã nguồn (như `<=` thành `<`, các mốc 100, 10.000, giới hạn Notes 4.000 byte, biên cửa sổ 24 giờ) và mỗi lần đều có test đỏ.

**Chưa làm**, nói thẳng: chưa tải lên TestFlight, chưa chạy `flutter build ipa`, `altool`, `pilot` hay Transporter; chưa tạo nhóm, mời tester, gửi Beta App Review, và chưa thấy trạng thái nào của Apple thật. Lane `beta_external` chỉ được kiểm cú pháp Ruby (bản 2.6.10 của hệ thống) và nội dung file; tên tham số `pilot` đối chiếu với docs.fastlane.tools, chưa với bản cài thật. Danh sách trường Test Information riêng của TestFlight chưa đối chiếu được trên form. Build tăng dần, ngưỡng 14 ngày và cách đếm 100 internal là quyết định của lab, không phải luật Apple.

## Bài tập

1. Nhóm internal có một nhân viên với vai trò ngoài năm vai trò Apple cho phép. `validate()` báo gì, và bạn xử lý thế nào?
2. Build 43 của version 1.4.0 đang In Beta Review. Bạn vá lỗi nhỏ thành build 44 cùng version. `checkBetaSubmission` nói gì?
3. Hôm nay 7/11/2026 lúc 00:00 UTC. Build tải lên 9/8/2026 lúc 08:00 UTC còn bao nhiêu ngày theo `BuildExpiry`, và `isExpiringSoon` trả gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Mã <code>internal-role-ineligible</code> cho đúng email đó. Đổi vai trò của họ trong App Store Connect nếu hợp lý, hoặc mời họ làm tester external qua email (khi đó cần Beta App Review).</p>
<p><strong>2.</strong> Bị chặn: version 1.4.0 đã có build 43 đang review và mỗi version chỉ một build được review cùng lúc. <code>retryAt</code> là <code>null</code> vì phải chờ review xong, không phải chờ giờ.</p>
<p><strong>3.</strong> Hạn là 7/11/2026 lúc 08:00 UTC, còn 8 giờ, làm tròn lên thành <strong>1 ngày</strong>, và <code>isExpiringSoon</code> trả <code>true</code> vì 1 nằm trong khoảng 1 đến 14. Số thật xem trong App Store Connect.</p>
</details>

Bài trước: [Firebase App Distribution: phân phối nội bộ cho tester iOS và Android](/articles/flutter-firebase-app-distribution-phan-phoi-noi-bo-tester). Bài tiếp: [Google Play: internal, closed, open testing, production và staged rollout](/articles/flutter-google-play-internal-closed-open-production-staged-rollout).

## Tài liệu gốc

- [TestFlight overview](https://developer.apple.com/help/app-store-connect/test-a-beta-version/testflight-overview/) và [TestFlight](https://developer.apple.com/testflight/) (developer.apple.com).
- [Add internal testers](https://developer.apple.com/help/app-store-connect/test-a-beta-version/add-internal-testers/), [Invite external testers](https://developer.apple.com/help/app-store-connect/test-a-beta-version/invite-external-testers/) và [Provide test information](https://developer.apple.com/help/app-store-connect/test-a-beta-version/provide-test-information/) (developer.apple.com).
- [View tester feedback](https://developer.apple.com/help/app-store-connect/test-a-beta-version/view-tester-feedback/) và [View build status and metrics](https://developer.apple.com/help/app-store-connect/test-a-beta-version/view-build-status-and-metrics/) (developer.apple.com).
- [App build statuses](https://developer.apple.com/help/app-store-connect/reference/app-uploads/app-build-statuses/) và [Upload builds](https://developer.apple.com/help/app-store-connect/manage-builds/upload-builds) (developer.apple.com).
- [App Review information](https://developer.apple.com/help/app-store-connect/reference/app-review-information) (developer.apple.com).
- [Overview of export compliance](https://developer.apple.com/help/app-store-connect/manage-app-information/overview-of-export-compliance), [Provide export compliance information for beta builds](https://developer.apple.com/help/app-store-connect/test-a-beta-version/provide-export-compliance-information-for-beta-builds) và [Complying with Encryption Export Regulations](https://developer.apple.com/documentation/security/complying-with-encryption-export-regulations) (developer.apple.com).
- [ITSAppUsesNonExemptEncryption](https://developer.apple.com/documentation/bundleresources/information-property-list/itsappusesnonexemptencryption), [CFBundleVersion](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion) và [NSLocationWhenInUseUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationwheninuseusagedescription) (developer.apple.com).
- [Asking permission to use notifications](https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications) và [Registering your app with APNs](https://developer.apple.com/documentation/usernotifications/registering-your-app-with-apns) (developer.apple.com).
- [Build and release an iOS app](https://docs.flutter.dev/deployment/ios) (docs.flutter.dev).
- [upload_to_testflight (pilot)](https://docs.fastlane.tools/actions/upload_to_testflight/) (docs.fastlane.tools).
- [Dart adopts BoringSSL](https://news.dartlang.org/2015/09/dart-adopts-boringssl.html) (news.dartlang.org).

---
slug: flutter-app-store-production-metadata-privacy-review-phased-release
title: "App Store production: metadata, privacy labels, App Review và phased release"
summary: "Bài 58: đưa Tới Nơi lên App Store. Giới hạn metadata (keywords tính bằng byte), ảnh chụp, PrivacyInfo.xcprivacy và required reason API, privacy label khớp mô tả quyền, demo account, điều App Review hỏi app gọi xe, phased release 7 ngày. Chưa gửi duyệt thật."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, release, app-store, privacy]
language: vi
---

Bản 1.0.0 của app khách Tới Nơi đã qua TestFlight. Sáng thứ Hai, PM bấm **Submit for Review**. Sáng thứ Ba, App Review trả lời: không đăng nhập được, vì OTP gửi bằng SMS tới số Việt Nam mà người duyệt không có. Mất một ngày (số minh hoạ), và đó còn là lỗi dễ nhất: label thiếu vị trí, ảnh có alpha hay từ khoá quá 100 byte cũng chặn ta như vậy.

Bài này đi nốt đoạn từ bản build tới người dùng: metadata, ảnh chụp, privacy manifest và label, tài khoản demo, điều App Review hỏi app gọi xe, rồi phát hành theo đợt. Đây là bài 58 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết TestFlight ở [bài 56](/articles/flutter-testflight-internal-external-testing-beta-review), fastlane ở [bài 53](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build) và Google Play ở [bài 57](/articles/flutter-google-play-internal-closed-open-production-staged-rollout).

Repository này không có tài khoản Apple Developer và máy kiểm không cài fastlane, nên **chưa bản nào được gửi cho App Review**. Ta kiểm bằng Dart những gì kiểm được từ file, còn lại dẫn trang chính thức. Mọi con số của Apple ghi "kiểm tại 11/2026".

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Từ khoá tiếng Việt dài 99 ký tự, nhiều chữ có dấu; trường keywords cho tối đa 100. Ta có chắc nộp được không?</p>
<details><summary>Xem đáp án</summary><p>Không chắc. App Store Connect Help ghi tối đa 100 <strong>byte</strong>, không phải ký tự. Trong UTF-8, chữ "ớ" chiếm 3 byte: 34 chữ "ớ" là 102 byte. Tên app và description thì tính theo ký tự. Lab có test cho đúng ca này.</p></details>
</div>

## Bản ghi app và metadata: giới hạn kiểm được bằng máy

Mỗi app một bản ghi trong App Store Connect, nên Tới Nơi có hai: `vn.toinoi.customer` và `vn.toinoi.driver`. Bundle ID không đổi được sau khi tải build lên; tên sửa được tới lúc gửi duyệt. Metadata phần lớn là giới hạn:

| Trường | Giới hạn | Đơn vị | Ghi chú |
|---|---|---|---|
| Tên | 2 đến 30 | ký tự | 2.3.7: không nhồi giá hay thương hiệu |
| Subtitle | 30 | ký tự | không nêu app khác |
| Keywords | 100 | **byte** | mỗi từ trên 2 ký tự; không lặp tên app |
| Description | 4.000 | ký tự | văn bản thuần |
| Promotional text | 170 | ký tự | sửa không cần duyệt lại |
| What's New | 4.000 | ký tự | không có ở bản đầu |
| Ghi chú App Review | 4.000 | **byte** | ngôn ngữ nào cũng được |

Support URL và privacy policy URL bắt buộc. Apple không nói cách đếm "ký tự": lab đếm code point (`runes`) và cảnh báo dạng NFD, nơi chữ "ớ" chiếm ba code point.

Với fastlane, metadata nằm trong git theo bố cục của `deliver`: mỗi ngôn ngữ một thư mục (`vi` có trong danh sách của deliver), mỗi trường một file `.txt` (Hình 1). Theo docs.fastlane.tools, tên, subtitle và keywords không sửa được khi phiên bản đang live.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Thư mục store/customer/metadata gồm các file txt theo ngôn ngữ và review_information. Fastlane deliver đẩy chúng lên các trường của App Store Connect, mỗi trường có giới hạn riêng. ReleaseChecklist bằng Dart đọc cùng các file đó và chặn bước đẩy nếu có lỗi."><defs><marker id="fl58-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl58-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="20" width="290" height="250" rx="10" class="f-phys-soft s-phys"/><text x="36" y="48" class="t-b">store/customer/metadata/</text>
<text x="36" y="80" class="t-mono">vi/name.txt</text><text x="36" y="104" class="t-mono">vi/subtitle.txt</text><text x="36" y="128" class="t-mono">vi/keywords.txt</text><text x="36" y="152" class="t-mono">vi/description.txt</text><text x="36" y="176" class="t-mono">vi/promotional_text.txt</text><text x="36" y="200" class="t-mono">review_information/notes.txt</text><text x="36" y="224" class="t-mono">copyright.txt</text><text x="36" y="252" class="t-small">tên file theo bố cục deliver</text>
<rect x="355" y="75" width="210" height="100" rx="10" class="f-pk-soft s-pk"/><text x="371" y="108" class="t-b">fastlane deliver</text><text x="371" y="134" class="t-mono">upload_to_app_store</text><text x="371" y="158" class="t-small">metadata_path</text>
<rect x="610" y="20" width="250" height="250" rx="10" class="f-phys-soft s-phys"/><text x="626" y="48" class="t-b">App Store Connect</text>
<text x="626" y="80" class="t-small">Tên: 30 ký tự</text><text x="626" y="104" class="t-small">Subtitle: 30 ký tự</text><text x="626" y="128" class="t-small">Keywords: 100 byte</text><text x="626" y="152" class="t-small">Description: 4.000 ký tự</text><text x="626" y="176" class="t-small">Promo: 170 ký tự</text><text x="626" y="200" class="t-small">Notes: 4.000 byte</text><text x="626" y="252" class="t-small t-warn">Từ chối chỉ lộ khi đã đẩy</text>
<path d="M312 125 L353 125" class="f-none s-pk" stroke-width="2" marker-end="url(#fl58-1-a)"/>
<path d="M567 125 L608 125" class="f-none s-pk" stroke-width="2" marker-end="url(#fl58-1-a)"/>
<rect x="335" y="228" width="250" height="92" rx="10" class="f-warn-soft s-warn"/><text x="351" y="258" class="t-b">ReleaseChecklist</text><text x="351" y="282" class="t-small">đọc các file này, báo lỗi</text><text x="351" y="302" class="t-small">trước khi đẩy lên</text>
<path d="M312 250 L333 262" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl58-1-w)"/>
<path d="M460 226 L460 177" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl58-1-w)"/><text x="474" y="206" class="t-small">chặn nếu có lỗi</text>
</svg>
</div>
<figcaption>Hình 1. Metadata nằm trong git, deliver đẩy lên App Store Connect, ReleaseChecklist đọc đúng các file đó trước. Mũi tên đứt là bước kiểm chạy trước khi đẩy.</figcaption>
</figure>

Giới hạn nằm trong một lớp hằng, mỗi hằng ghi đơn vị:

### store_limits.dart · 58-app-store-release/lib/src/store_limits.dart (trích)

```dart
abstract final class StoreLimits {
  static const int nameMin = 2;
  static const int nameMax = 30;
  static const int subtitleMax = 30;

  /// Tính bằng **byte** (UTF-8), không phải ký tự: chữ có dấu tiếng Việt
  /// chiếm 2 hoặc 3 byte.
  static const int keywordsMaxBytes = 100;

  /// Mỗi từ khoá phải dài hơn hai ký tự.
  static const int keywordMinChars = 3;
  static const int promotionalTextMax = 170;
  static const int descriptionMax = 4000;
  static const int releaseNotesMax = 4000;

  /// Ghi chú cho App Review: 4.000 **byte**, viết bằng ngôn ngữ nào cũng được.
  static const int reviewNotesMaxBytes = 4000;

  static const int screenshotsMaxPerSize = 10;
}
```

Keywords dùng `_bytes`, còn tên và subtitle dùng `_chars`:

### release_checklist.dart · 58-app-store-release/lib/src/release_checklist.dart (trích)

```dart
int _chars(String s) => s.runes.length;
int _bytes(String s) => utf8.encode(s).length;
```

### release_checklist.dart · 58-app-store-release/lib/src/release_checklist.dart (trích, trong _keywords)

```dart
    if (raw == null || raw.trim().isEmpty) return;
    // Đếm cả dấu phẩy: Apple chỉ nói "100 byte nội dung", đếm thừa là an toàn.
    final bytes = _bytes(raw.trim());
    if (bytes > StoreLimits.keywordsMaxBytes) {
      out.add(
        Finding.error(
          'keywords-bytes',
          '$at keywords chiếm $bytes byte, tối đa '
              '${StoreLimits.keywordsMaxBytes}.',
        ),
      );
```

Test dựng đúng ca của câu hỏi đầu bài. Dấu phẩy tính vào 100 byte (đếm thừa là an toàn); bản mẫu của app khách dùng 69 byte cho 54 ký tự.

### metadata_test.dart · 58-app-store-release/test/metadata_test.dart (trích)

```dart
    test('34 chữ ớ chỉ 34 ký tự nhưng 102 byte: vượt giới hạn byte', () {
      final kw = 'ớ' * 34;
      expect(kw.runes.length, 34);
      expect(utf8.encode(kw).length, 102);
      expect(metaCodes((m) => withLocale(m, keywords: kw)), {'keywords-bytes'});
    });
```

**Ảnh chụp.** Tối đa 10 ảnh mỗi cỡ; jpeg, jpg hoặc png; không alpha hay trong suốt. Cỡ iPhone bắt buộc là Dynamic Island medium: 1179 x 2556 hoặc 1206 x 2622, cỡ nhỏ hơn được store tự co. App chạy được trên iPad cần thêm ảnh 13 inch: 2064 x 2752 hoặc 2048 x 2732 (Screenshot specifications). Template Flutter 3.47.4 đặt `TARGETED_DEVICE_FAMILY = "1,2"`, tức hỗ trợ iPad, nên mặc định ta bị đòi ảnh iPad; hai app Tới Nơi chọn chỉ iPhone. Từ 4/2027, app dùng SDK iOS 27.1 trở lên còn cần ảnh iPhone Duo (trang Apple, 11/2026). Guideline 2.3.3: ảnh phải cho thấy app đang dùng.

Kiểm alpha chỉ cần đọc phần đầu file PNG:

### screenshot.dart · 58-app-store-release/lib/src/screenshot.dart (trích, trong PngInfo.parse)

```dart
    final width = data.getUint32(16);
    final height = data.getUint32(20);
    final colorType = bytes[25];
    var hasAlpha = colorType == 4 || colorType == 6;

    // Duyệt các chunk sau IHDR để tìm tRNS; dừng ở IDAT vì tRNS phải đứng trước.
    var offset = 33;
    while (!hasAlpha && offset + 8 <= bytes.length) {
      final length = data.getUint32(offset);
      final type = String.fromCharCodes(bytes.sublist(offset + 4, offset + 8));
      if (type == 'tRNS') hasAlpha = true;
      if (type == 'IDAT' || type == 'IEND') break;
      offset += 12 + length;
    }
    return PngInfo(width: width, height: height, hasAlpha: hasAlpha);
```

Chunk `tRNS` cũng là trong suốt nên vòng lặp tìm nó. Test tạo PNG trong bộ nhớ.

## Privacy manifest và required reason API

`PrivacyInfo.xcprivacy` là plist trong bundle, ghi: có tracking không, domain tracking, kiểu dữ liệu app thu thập, và các **required reason API** app dùng kèm lý do. Đó là nhóm API có thể bị lạm dụng để nhận dạng thiết bị: thời gian file, thời gian khởi động, dung lượng đĩa, bàn phím đang bật, `UserDefaults`. Từ 1/5/2024, bản tải lên dùng chúng mà không khai lý do bị App Store Connect từ chối.

Với Flutter, engine và nhiều plugin nằm trong danh sách SDK Apple bắt buộc có manifest (`Flutter`, `geolocator_apple`, `FirebaseMessaging`...). Trong `~/.pub-cache` của máy kiểm, `shared_preferences_foundation` 2.6.1 khai UserDefaults với `1C8F.1`, `firebase_messaging` 17.0.1 với `CA92.1`, `geolocator_apple` 2.4.3 khai mảng rỗng. Mỗi plugin chỉ khai phần code của nó. Ngược lại, template app Flutter 3.47.4 **không** sinh `PrivacyInfo.xcprivacy` cho Runner (chỉ template plugin có), nên file này ta tự thêm vào target Runner.

Manifest của app chỉ khai dữ liệu chính app thu thập và API mà code native của app gọi. Giả sử code native không gọi API nào trong năm nhóm (ta chưa dựng runner iOS nên chưa tìm), mảng API để rỗng và app khách khai bốn loại dữ liệu:

### PrivacyInfo.xcprivacy · 58-app-store-release/native-config/customer/ios/PrivacyInfo.xcprivacy (trích)

```xml
<dict>
	<key>NSPrivacyTracking</key>
	<false/>
	<key>NSPrivacyTrackingDomains</key>
	<array/>
	<key>NSPrivacyCollectedDataTypes</key>
	<array>
		<dict>
			<key>NSPrivacyCollectedDataType</key>
			<string>NSPrivacyCollectedDataTypePhoneNumber</string>
			<key>NSPrivacyCollectedDataTypeLinked</key>
			<true/>
			<key>NSPrivacyCollectedDataTypeTracking</key>
			<false/>
			<key>NSPrivacyCollectedDataTypePurposes</key>
			<array>
				<string>NSPrivacyCollectedDataTypePurposeAppFunctionality</string>
			</array>
		</dict>
```

Mỗi nhóm chỉ nhận lý do của nó (`CA92.1` đúng cho UserDefaults, sai cho FileTimestamp), và `0A2A.1`, `C56D.1` dành riêng cho SDK nên app khai chúng là sai. Lab mã hoá bảng của Apple và kiểm từng lý do:

### store_limits.dart · 58-app-store-release/lib/src/store_limits.dart (trích, trong RequiredReasonApi)

```dart
  static const Map<String, Set<String>> reasonsByCategory = {
    fileTimestamp: {'DDA9.1', 'C617.1', '3B52.1', '0A2A.1'},
    systemBootTime: {'35F9.1', '8FFB.1', '3D61.1'},
    diskSpace: {'85F4.1', 'E174.1', '7D9E.1', 'B728.1'},
    activeKeyboards: {'3EC4.1', '54BD.1'},
    userDefaults: {'CA92.1', '1C8F.1', 'C56D.1', 'AC6B.1'},
  };

  /// Chỉ SDK bên thứ ba được khai: bọc API cho app dùng.
  static const Set<String> sdkOnlyReasons = {'0A2A.1', 'C56D.1'};
```

### release_checklist.dart · 58-app-store-release/lib/src/release_checklist.dart (trích, trong _manifest)

```dart
      for (final reason in api.reasons) {
        if (!allowed.contains(reason)) {
          out.add(
            Finding.error(
              'manifest-api-reason-invalid',
              '$reason không phải lý do hợp lệ của ${api.category}.',
            ),
          );
        } else if (RequiredReasonApi.sdkOnlyReasons.contains(reason)) {
          out.add(
            Finding.error(
              'manifest-api-sdk-only-reason',
              '$reason chỉ SDK bên thứ ba được khai, app không được.',
            ),
          );
        }
      }
```

Xcode có bước đối chiếu cuối: **Product > Archive**, chuột phải archive, **Generate Privacy Report** (gộp manifest của app và SDK). Lab chưa chạy vì cần Xcode đầy đủ.

## Privacy label: ba nguồn phải khớp nhau

Privacy label do ta trả lời trong App Store Connect cho từng loại dữ liệu: mục đích, có gắn danh tính (linked) không, có dùng để theo dõi không. "Thu thập" là dữ liệu rời máy và ta hoặc đối tác giữ lâu hơn mức cần để phục vụ yêu cầu lúc đó, tính cả SDK bên thứ ba. Docs Firebase ghi: Cloud Messaging lưu APNs token gắn mã cài đặt, Crashlytics thu stack trace và thông tin máy. Vì thế label app khách có `DeviceID` và `CrashData` dù manifest của app không có.

Apple viết: nếu thông tin thanh toán nhập ở dịch vụ ngoài app và nhà phát triển không bao giờ truy cập được, Payment Info không cần khai. Bài 39 chọn trình duyệt trong app (SFSafariViewController) vì app không đọc được điều khách gõ, nên ta khai theo hướng đó; đó là cách hiểu của ta, không phải xác nhận của Apple. Dùng WebView thì app đọc được nội dung nhập, đừng dựa vào ngoại lệ này.

Ba file phải nhất quán (Hình 2). Lệch nhau thì App Review hoặc người dùng thấy trước ta.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Ba nguồn đổ vào privacy label. Info.plist: có quyền vị trí thì label phải khai vị trí. PrivacyInfo.xcprivacy của app: mọi loại dữ liệu đã khai phải có trong label. Manifest của plugin và SDK: dữ liệu SDK thu thập vẫn phải khai trong label. Xcode Generate Privacy Report gộp các manifest."><defs><marker id="fl58-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">Info.plist</text><text x="36" y="88" class="t-mono">NS...UsageDescription</text><text x="36" y="112" class="t-mono">UIBackgroundModes</text>
<rect x="315" y="30" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="331" y="62" class="t-b">PrivacyInfo.xcprivacy</text><text x="331" y="88" class="t-small">dữ liệu app tự thu thập</text><text x="331" y="112" class="t-small">API bắt buộc có lý do</text>
<rect x="610" y="30" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="626" y="62" class="t-b">Manifest plugin và SDK</text><text x="626" y="88" class="t-small">Firebase, geolocator...</text><text x="626" y="112" class="t-small">mỗi SDK tự khai</text>
<rect x="250" y="270" width="380" height="100" rx="10" class="f-pk-soft s-pk"/><text x="266" y="302" class="t-b">Privacy label (App Store Connect)</text><text x="266" y="328" class="t-small">mỗi loại dữ liệu: mục đích,</text><text x="266" y="348" class="t-small">linked, tracking</text>
<path d="M145 142 L300 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl58-2-a)"/><text x="20" y="200" class="t-small">quyền vị trí ⇒</text><text x="20" y="216" class="t-small">khai vị trí</text>
<path d="M440 142 L440 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl58-2-a)"/><text x="452" y="206" class="t-small">mọi loại đã khai</text>
<path d="M735 142 L580 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl58-2-a)"/><text x="705" y="200" class="t-small">dữ liệu SDK</text><text x="705" y="216" class="t-small">vẫn phải khai</text>
<text x="20" y="392" class="t-small">Xcode: Archive, Generate Privacy Report gộp manifest của app và SDK để đối chiếu</text>
</svg>
</div>
<figcaption>Hình 2. Mọi thứ phải có trong label. Label nhập tay, nên ta giữ bản sao privacy-label.json trong git để so với hai nguồn kia.</figcaption>
</figure>

Bản sao ghi mỗi dòng với nguồn `app` (phải có trong manifest của app) hoặc `sdk:<tên>` (không bắt buộc):

### privacy-label.json · 58-app-store-release/store/customer/privacy-label.json (trích)

```json
{
  "tracking": false,
  "dataTypes": [
    {
      "type": "NSPrivacyCollectedDataTypePhoneNumber",
      "linked": true,
      "tracking": false,
      "purposes": [
        "NSPrivacyCollectedDataTypePurposeAppFunctionality"
      ],
      "source": "app"
    },
```

Các quy tắc của `ReleaseChecklist`:

- Có `NSLocation...UsageDescription` mà label không khai Precise hay Coarse Location thì `label-missing-location`; chiều ngược lại là `location-without-usage-description`.
- Mọi loại trong manifest phải có trong label, cùng `linked`, `tracking`, đủ mục đích (`label-missing-type`, `label-manifest-mismatch`); dòng nguồn `app` mà manifest không có là `label-app-type-not-in-manifest`.
- Quyền liên hệ, micro, ảnh, camera chỉ cảnh báo, vì xin quyền chưa chắc đã thu thập. Mô tả quyền dưới 20 ký tự cũng chỉ cảnh báo (ngưỡng của lab); nội dung tạm là lỗi (5.1.1(ii)).

### release_checklist.dart · 58-app-store-release/lib/src/release_checklist.dart (trích, trong _labelAgainstUsage)

```dart
  List<Finding> _labelAgainstUsage() {
    final out = <Finding>[];
    final info = input.info;
    final labelTypes = input.label.types;
    final hasLocationLabel =
        labelTypes.contains(ManifestVocabulary.preciseLocation) ||
        labelTypes.contains(ManifestVocabulary.coarseLocation);

    if (info.usesLocation && !hasLocationLabel) {
      out.add(
        const Finding.error(
          'label-missing-location',
          'Info.plist xin quyền vị trí nhưng label không khai Precise/Coarse Location.',
        ),
      );
    }
    if (!info.usesLocation && hasLocationLabel) {
      out.add(
        const Finding.error(
          'location-without-usage-description',
          'Label khai vị trí nhưng Info.plist không có NSLocation...UsageDescription.',
        ),
      );
    }
```

`deliver` không cập nhật label; fastlane có action riêng `upload_app_privacy_details_to_app_store`, docs liệt kê tham số `username`, không thấy API key. Lab nhập tay.

## App Review: những câu hỏi dành cho app gọi xe

Apple nói thường duyệt 90% bản trong dưới 48 giờ, và hơn 40% vấn đề chưa giải quyết thuộc Guideline 2.1 (crash, nội dung tạm, thiếu thông tin). Thất bại hay gặp nhất là thiếu chuẩn bị.

**Tài khoản demo.** Mục App Review Information gồm liên hệ (số điện thoại quốc tế có dấu +), ghi chú tối đa 4.000 byte và tài khoản demo. Theo Platform version information, tài khoản demo không được hết hạn; theo 2.1(a), máy chủ phải đang chạy, và demo mode chỉ thay được khi Apple đồng ý trước. Bản prod gọi API production, không có server duyệt riêng: tài khoản demo nằm trên production, cô lập theo tài khoản, và chuyến mô phỏng không được tới tài xế thật. Reviewer có thể bấm xoá tài khoản (5.1.1(v)), nên server tự tạo lại tài khoản demo và notes nói điều đó.

Người duyệt không nhận được SMS, nên tài khoản demo có OTP riêng. Đó là thiết kế của ta, không phải yêu cầu của Apple, và là một cửa sau: mã phải ngẫu nhiên, lấy từ secret hoặc ENV, nằm ở ô Password của App Store Connect chứ không trong git, kèm giới hạn số lần thử, cảnh báo mỗi lần đăng nhập, đổi mã sau mỗi lần duyệt. Giá trị trong `demo_password.txt` của lab là hư cấu; mã `123456` của bài 24 chỉ dành cho fake API. Mỗi app tự khép kín: bên mô phỏng đóng vai tài xế hoặc khách. Apple không nói người duyệt ở đâu, nên ghi chú chỉ cách đi tiếp khi ở ngoài khu vực dịch vụ.

### notes.txt · 58-app-store-release/store/customer/metadata/review_information/notes.txt (trích)

```text
HƯỚNG DẪN DUYỆT APP KHÁCH TỚI NƠI

1. Đăng nhập
Nhập số điện thoại của tài khoản demo (ô Username ở trên) rồi bấm Tiếp tục. Mã OTP của tài khoản này nằm ở ô Password, không gửi SMS. Tài khoản demo nằm trên hệ thống production, cô lập theo tài khoản, và chuyến mô phỏng không tới tài xế thật. Tài khoản không hết hạn; nếu bạn xoá nó (mục 5), hệ thống tự tạo lại.

2. Đặt thử một chuyến xe
- Dịch vụ chỉ hoạt động trong khu vực thử nghiệm Hà Nội. Nếu máy duyệt ở ngoài Việt Nam, bấm "Chọn trên bản đồ" thay vì "Dùng vị trí hiện tại": app vẫn đặt được chuyến mà không cần cấp quyền vị trí.
- Chọn điểm đón và điểm đến cách nhau khoảng 5 km. Cước dự kiến hiển thị 25.500 VND.
- Với tài khoản demo, hệ thống mô phỏng tài xế: tài xế tự nhận chuyến sau khoảng 10 giây và đi qua các bước cho tới "Hoàn thành".
- Khách huỷ được chuyến khi tài xế chưa đón khách.
```

Số 25.500 VND không gõ tay: test so với `calculateFare(5)` của `toi_noi_core`.

| Điều | Apple đòi (bản 16/10/2026) | Tới Nơi |
|---|---|---|
| 5.1.1(v) | Có tạo tài khoản thì cho xoá ngay trong app, không chỉ vô hiệu hoá | Hồ sơ > Xoá tài khoản, xác nhận OTP; demo tự tạo lại, ghi trong notes |
| 5.1.5, 5.1.1 | Vị trí chỉ khi liên quan trực tiếp, xin đồng ý, có phương án khác | Khách xin "khi dùng app", có "Chọn trên bản đồ" |
| 5.1.2(i) | Không bắt bật vị trí, push, tracking | Từ chối push vẫn dùng đủ |
| 2.5.4 | Dịch vụ nền đúng mục đích | Tài xế: `location` chỉ trong chuyến, có video (bài 38) |
| 3.1.3(e) | Dịch vụ dùng ngoài app thu tiền bằng cách khác IAP | VíXanh, không IAP (bài 39) |
| 4.5.4 | Push không bắt buộc; quảng cáo qua push cần đồng ý | Tách thông báo chuyến và khuyến mãi (bài 37) |

**Xếp hạng độ tuổi:** từ 31/1/2026 Apple dùng mức 4+, 9+, 13+, 16+, 18+, tính từ bảng câu hỏi trong App Information; ta khai đúng tính năng thật (có chat thì đọc Guideline 1.2). **Công cụ build:** từ 28/4/2026 phải dùng Xcode 26 trở lên, từ 9/9/2026 phải nhắm iOS 13 trở lên (template Flutter 3.47.4 đặt 15.0). **Duyệt nhanh:** Apple nêu hai ca, sửa lỗi nghiêm trọng (kèm bước tái hiện) và app gắn với sự kiện; đó là đặc cách từng lần qua biểu mẫu, không phải lane.

Hai lane fastlane: `listing` chỉ đẩy metadata và ảnh; `submit` chọn build có sẵn và gửi duyệt, đòi `confirm:yes` vì đó là quyết định của người phát hành. Cả hai tắt `precheck_include_in_app_purchases` (mặc định làm deliver báo lỗi khi dùng `api_key`), và `submit` không được bỏ qua metadata, vì khi đó deliver bỏ luôn `automatic_release` và `phased_release`:

### Fastfile.store.example · 58-app-store-release/native-config/ios/fastlane/Fastfile.store.example (trích)

```ruby
  lane :submit do |options|
    app = options[:app].to_s
    UI.user_error!("Gửi duyệt là quyết định của người phát hành: thêm confirm:yes") unless options[:confirm].to_s == "yes"
    build_number = options[:build_number].to_s
    UI.user_error!("build_number phải là số nguyên dương") unless build_number =~ /\A[1-9][0-9]*\z/
    # Phased release chỉ áp cho bản cập nhật; lần đầu ta tự bấm phát hành.
    first_release = options[:first_release].to_s == "true"
    upload_to_app_store(
      api_key: asc_api_key,
      app_identifier: bundle_id!(options),
      build_number: build_number,
      # Không bỏ qua metadata (tham số skip_metadata): khi bỏ qua, deliver return
      # sớm và automatic_release, phased_release bị bỏ qua mà không báo.
      metadata_path: "../store/#{app}/metadata",
      skip_binary_upload: true,
      skip_screenshots: true,
      submit_for_review: true,
      automatic_release: !first_release,
      phased_release: !first_release,
      precheck_include_in_app_purchases: false,
      force: true,
    )
  end
```

## Phased release 7 ngày: dừng không phải quay lại

Phased release thả bản cập nhật cho người bật tự động cập nhật trong 7 ngày (App Store Connect Help); ai tự bấm Cập nhật trong App Store thì nhận ngay.

| Ngày | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|
| Người dùng | 1% | 2% | 5% | 10% | 20% | 50% | 100% |

Tuỳ chọn được viết cho bản cập nhật (bản đầu chọn thủ công, tự động hoặc theo mốc giờ). Tạm dừng tối đa 30 ngày cộng dồn. **Release to All Users** phát ngay cho mọi người dùng bật tự động cập nhật. Gỡ app khỏi bán thì đợt dừng hẳn, và khi app trở lại nó tới mọi người ngay.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Bảy cột theo ngày: 1, 2, 5, 10, 20, 50, 100 phần trăm người dùng bật tự động cập nhật. Tạm dừng giữ nguyên phần trăm của ngày hiện tại và người đã cập nhật vẫn giữ bản đó."><defs><marker id="fl58-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<path d="M40 250 L850 250" class="f-none s-line" stroke-width="2"/>
<rect x="60" y="248.4" width="70" height="1.6" class="f-pk-soft s-pk"/><text x="95" y="236" text-anchor="middle" class="t-b">1%</text><text x="95" y="272" text-anchor="middle" class="t-small">Ngày 1</text>
<rect x="175" y="246.8" width="70" height="3.2" class="f-pk-soft s-pk"/><text x="210" y="234" text-anchor="middle" class="t-b">2%</text><text x="210" y="272" text-anchor="middle" class="t-small">Ngày 2</text>
<rect x="290" y="242" width="70" height="8" class="f-pk-soft s-pk"/><text x="325" y="229" text-anchor="middle" class="t-b">5%</text><text x="325" y="272" text-anchor="middle" class="t-small">Ngày 3</text>
<rect x="405" y="234" width="70" height="16" class="f-pk-soft s-pk"/><text x="440" y="221" text-anchor="middle" class="t-b">10%</text><text x="440" y="272" text-anchor="middle" class="t-small">Ngày 4</text>
<rect x="520" y="218" width="70" height="32" class="f-pk-soft s-pk"/><text x="555" y="205" text-anchor="middle" class="t-b">20%</text><text x="555" y="272" text-anchor="middle" class="t-small">Ngày 5</text>
<rect x="635" y="170" width="70" height="80" class="f-pk-soft s-pk"/><text x="670" y="157" text-anchor="middle" class="t-b">50%</text><text x="670" y="272" text-anchor="middle" class="t-small">Ngày 6</text>
<rect x="750" y="90" width="70" height="160" class="f-pk s-pk"/><text x="785" y="77" text-anchor="middle" class="t-b">100%</text><text x="785" y="272" text-anchor="middle" class="t-small">Ngày 7</text>
<rect x="20" y="20" width="380" height="64" rx="10" class="f-warn-soft s-warn"/><text x="36" y="46" class="t-b">Pause giữ nguyên % của ngày đó</text><text x="36" y="68" class="t-small">Người đã cập nhật vẫn giữ bản này.</text>
<path d="M200 86 L315 212" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl58-3-w)"/>
<text x="20" y="312" class="t-small">Minh hoạ: 20.000 người bật tự động cập nhật nhận 200, 400, 1.000, 2.000 người ở bốn ngày đầu.</text>
</svg>
</div>
<figcaption>Hình 3. Các mốc phased release, cột vẽ đúng tỉ lệ: ngày đầu rất thấp để bản lỗi chỉ chạm ít người.</figcaption>
</figure>

`PhasedReleasePlan` mô hình hoá bảng và quy tắc dừng. Mỗi ngày là 24 giờ kể từ lúc bắt đầu (mô hình của lab; Apple không nói mốc đổi ngày). Đang dừng thì ngày đứng yên:

### phased_release.dart · 58-app-store-release/lib/src/phased_release.dart (trích)

```dart
  /// Phần trăm của ngày thứ [dayNumber] (1..7); sau ngày 7 là 100.
  static int percentOnDay(int dayNumber) {
    if (dayNumber < 1) {
      throw ArgumentError.value(dayNumber, 'dayNumber', 'phải từ 1 trở lên');
    }
    return dayNumber > percentByDay.length ? 100 : percentByDay[dayNumber - 1];
  }
```

### phased_release.dart · 58-app-store-release/lib/src/phased_release.dart (trích)

```dart
  int dayAt(DateTime now) {
    final clock = isPaused ? _pauses.last.from : now;
    final elapsed = clock.difference(startedAt) - _closedPauses();
    if (elapsed.isNegative) return 0;
    return elapsed.inMicroseconds ~/ day.inMicroseconds + 1;
  }
```

Quyết định dựa trên số liệu phiên (Crashlytics, bài 48). Ngưỡng 99,50% và 500 phiên chỉ minh hoạ; so sánh bằng số nguyên (điểm cơ bản) để biên không dính sai số `double`:

### phased_release.dart · 58-app-store-release/lib/src/phased_release.dart (trích, trong decide)

```dart
  RolloutAction decide(
    RolloutMetrics metrics, {
    RolloutThresholds thresholds = const RolloutThresholds(),
  }) {
    if (metrics.sessions < 0 ||
        metrics.crashedSessions < 0 ||
        metrics.crashedSessions > metrics.sessions) {
      throw ArgumentError('Số phiên không hợp lệ: $metrics');
    }
    if (metrics.sessions < thresholds.minSessions) return RolloutAction.wait;
    final crashFree = (metrics.sessions - metrics.crashedSessions) * 10000;
    return crashFree < thresholds.minCrashFreeBasisPoints * metrics.sessions
        ? RolloutAction.pause
        : RolloutAction.proceed;
  }
```

Ngày 1 chỉ có 200 trên 20.000 người, nên `decide` trả `wait` thay vì kết luận từ mẫu nhỏ. Tạm dừng không phải rollback: người đã cập nhật giữ bản lỗi. Thao tác Make a version unavailable for download gỡ phiên bản khỏi App Store để xử lý vấn đề pháp lý hay khả dụng (trang đó yêu cầu gửi bản cập nhật trừ khi không thể); nó không đưa ai về bản cũ. Lối ra là bản sửa lỗi (có thể xin duyệt nhanh); force update và hotfix ở bài 59.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi lên App Store</span>
<ul>
<li><strong>Đếm keywords bằng ký tự.</strong> Dấu hiệu: store từ chối dù dưới 100 ký tự. Nguyên nhân: giới hạn là 100 byte UTF-8. Cách sửa: đếm byte như <code>_bytes</code>.</li>
<li><strong>Ảnh có alpha, hoặc thiếu ảnh iPad.</strong> Dấu hiệu: không nộp được ảnh, hoặc bị đòi ảnh iPad 13 inch. Nguyên nhân: PNG còn alpha; template hỗ trợ iPad. Cách sửa: xuất không alpha; chọn chỉ iPhone hoặc chụp ảnh iPad.</li>
<li><strong>Khai lý do dành cho SDK trong manifest của app.</strong> Dấu hiệu: <code>0A2A.1</code> hoặc <code>C56D.1</code> trong app. Nguyên nhân: nhầm vai app và SDK. Cách sửa: app chỉ khai API mà code native của nó gọi.</li>
<li><strong>Tài khoản demo không dùng được.</strong> Dấu hiệu: "Information Needed" (2.1). Nguyên nhân: OTP qua SMS, tài khoản demo bị xoá hoặc khoá, thiếu bên đối diện. Cách sửa: OTP riêng cho tài khoản demo, tự tạo lại sau khi xoá, bên đối diện mô phỏng.</li>
<li><strong>Label, quyền và ghi chú mâu thuẫn nhau.</strong> Dấu hiệu: reviewer hỏi vì sao dùng vị trí mà label không có. Nguyên nhân: sửa một nơi, quên hai nơi. Cách sửa: chạy <code>bin/check.dart</code> trước mỗi lần gửi.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/58-app-store-release
dart pub get
dart analyze
dart test
dart run bin/check.dart customer
dart run bin/check.dart driver --update
```

Đã kiểm với Dart 3.13.3 (Flutter 3.47.4): `dart analyze` 0 issue, **135 test xanh**. Mức: **Kiểm chứng cấu hình**, không dùng mạng hay thiết bị. Hai lệnh `bin/check.dart` chỉ báo thiếu ảnh chụp (và `release-notes-missing` với `--update`), vì repo không chứa ảnh.

Các test sửa đúng một chỗ của file thật rồi đòi checklist đỏ đúng mã lỗi: giới hạn metadata ở biên, lý do sai nhóm hay dành cho SDK, label lệch manifest hoặc quyền vị trí, ảnh PNG dựng trong bộ nhớ, bảng phased release và biên 99,50%, và `ruby -c` cùng luật cho `Fastfile.store.example` (chỉ so chuỗi, chưa chạy lane).

**Chưa làm**, nói thẳng: không tạo bản ghi app, không nhập label, không tải ảnh, không gửi duyệt, không chạy phased release thật. Không dựng runner iOS nên chưa thêm manifest vào target Runner và chưa chạy **Generate Privacy Report**. Không cài fastlane nên tên action và tham số chỉ đối chiếu với docs.fastlane.tools. URL chỉ kiểm cú pháp. Xếp hạng độ tuổi và Digital Services Act nằm ngoài lab.

## Bài tập

1. Bản 1.2.0 đang ở ngày 3 của phased release (5%) thì Crashlytics báo crash khi mở app. Ta bấm tạm dừng. Người đã cập nhật có về 1.1.0 không, và người tự tải trong App Store nhận bản nào?
2. Tính năng gửi ảnh bưu kiện thêm `NSPhotoLibraryUsageDescription`. Checklist báo gì, và nếu ảnh được gửi lên server thì ta sửa những file nào?
3. Thêm `,xe công nghệ` vào cuối keywords của app khách. Tổng bao nhiêu byte, còn hợp lệ không?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Không về. Tạm dừng chỉ chặn người nhận tự động từ giờ; ai đã nhận 1.2.0 vẫn giữ nó, và ai tự bấm Cập nhật vẫn nhận 1.2.0 (tài liệu phased release nói rõ). Muốn sửa phải phát bản mới, xem bài 59.</p>
<p><strong>2.</strong> Cảnh báo <code>label-missing-for-usage</code>: label chưa có <code>PhotosorVideos</code>. Nếu ảnh rời máy và ta giữ lâu hơn mức cần, khai nó trong <code>PrivacyInfo.xcprivacy</code> và <code>privacy-label.json</code>, rồi cập nhật label trên App Store Connect.</p>
<p><strong>3.</strong> "xe công nghệ" dài 12 ký tự nhưng "ô" chiếm 2 byte và "ệ" chiếm 3 byte nên là 15 byte, thêm dấu phẩy là 16. Tổng 69 + 16 = 85 byte, hợp lệ.</p>
</details>

Bài trước: [Google Play: internal, closed, open testing, production và staged rollout](/articles/flutter-google-play-internal-closed-open-production-staged-rollout). Bài tiếp: [Vận hành sau phát hành: force update, remote config, Shorebird và rollback](/articles/flutter-van-hanh-sau-phat-hanh-force-update-remote-config-shorebird-rollback).

## Tài liệu gốc

- [Screenshot specifications](https://developer.apple.com/help/app-store-connect/reference/app-information/screenshot-specifications), [App information](https://developer.apple.com/help/app-store-connect/reference/app-information/app-information) và [Platform version information](https://developer.apple.com/help/app-store-connect/reference/app-information/platform-version-information) (developer.apple.com, App Store Connect Help).
- [Manage app privacy](https://developer.apple.com/help/app-store-connect/manage-app-information/manage-app-privacy), [App privacy details](https://developer.apple.com/app-store/app-privacy-details/) và [Third-party SDK requirements](https://developer.apple.com/support/third-party-SDK-requirements/) (developer.apple.com).
- [Privacy manifest files](https://developer.apple.com/documentation/bundleresources/privacy-manifest-files), [Describing use of required reason API](https://developer.apple.com/documentation/bundleresources/describing-use-of-required-reason-api) và [Describing data use in privacy manifests](https://developer.apple.com/documentation/bundleresources/describing-data-use-in-privacy-manifests) (developer.apple.com).
- [App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/), [5.1.1(v) Offering account deletion in your app](https://developer.apple.com/support/offering-account-deletion-in-your-app/) và [App Review](https://developer.apple.com/distribute/app-review/) (developer.apple.com).
- [Upcoming requirements](https://developer.apple.com/news/upcoming-requirements/), [Set an app age rating](https://developer.apple.com/help/app-store-connect/manage-app-information/set-an-app-age-rating) và [Age ratings values and definitions](https://developer.apple.com/help/app-store-connect/reference/app-information/age-ratings-values-and-definitions) (developer.apple.com).
- [Release a version update in phases](https://developer.apple.com/help/app-store-connect/update-your-app/release-a-version-update-in-phases) , [Select an App Store version release option](https://developer.apple.com/help/app-store-connect/manage-your-apps-availability/select-an-app-store-version-release-option) và [Make a version unavailable for download](https://developer.apple.com/help/app-store-connect/manage-your-apps-availability/make-a-version-unavailable-for-download) (developer.apple.com).
- [Build and release an iOS app](https://docs.flutter.dev/deployment/ios) (docs.flutter.dev).
- [deliver (upload_to_app_store)](https://docs.fastlane.tools/actions/deliver/) và [upload_app_privacy_details_to_app_store](https://docs.fastlane.tools/actions/upload_app_privacy_details_to_app_store/) (docs.fastlane.tools).
- [Apple App Store Data Collection Disclosure Guidelines](https://firebase.google.com/docs/ios/app-store-data-collection) (firebase.google.com).

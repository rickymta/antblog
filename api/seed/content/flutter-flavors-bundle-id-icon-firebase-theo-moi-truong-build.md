---
slug: flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build
title: "Flavors khi build: bundle ID, application ID, icon, tên app và Firebase theo môi trường"
summary: "Bài 50: một tên flavor chọn productFlavor của Gradle và scheme, configuration, xcconfig của Xcode, để ba bản dev, staging, prod có ID, tên, icon và file Firebase riêng, cài cạnh nhau được. Kiểm bằng script và 131 test; chưa build trên Gradle, Xcode hay thiết bị."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, flavors, android, ios, firebase, release]
language: vi
---

Một tester QA cài bản staging của app khách lên chiếc điện thoại mà lập trình viên vừa cắm dây chạy bản dev. Màn hình đăng nhập biến mất, danh sách chuyến trống trơn. Bản mới đè lên bản cũ vì cả hai mang cùng một application ID, và hệ điều hành coi chúng là một app. Tình huống này là minh hoạ.

[Bài 25](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod) lo lớp Dart: `--dart-define-from-file` đưa địa chỉ API, log và timeout vào code. Bài này lo lớp native, bốn thứ Gradle và Xcode đọc chứ Dart không đọc: ID ứng dụng, tên hiển thị, icon và file Firebase. Mục tiêu là ba bản `dev`, `staging`, `prod` cài cạnh nhau, mỗi bản trỏ đúng backend và đúng Firebase app.

Đây là bài 50 của lộ trình **Flutter qua Tới Nơi**, mở đầu Phần 7. Ta nên đọc bài 25 trước, và [bài tổng hợp Phần 6](/articles/flutter-tong-hop-phan-6-dat-chuyen-end-to-end-app-khach-tai-xe) cho bối cảnh hai app. Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); tên, màu icon và mọi giá trị Firebase là hư cấu. Bài dựa trên Flutter 3.47.4 (template dùng AGP 9.1.0) và tài liệu hiện hành tháng 11/2026.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>Flavors/dev.xcconfig</code> đặt <code>PRODUCT_BUNDLE_IDENTIFIER = vn.toinoi.customer.dev</code> và đã gắn vào configuration <code>Debug-dev</code>. Nhưng Build Settings của target Runner vẫn còn dòng <code>PRODUCT_BUNDLE_IDENTIFIER = vn.toinoi.customer</code> do template sinh sẵn. Bản <code>Debug-dev</code> ra bundle ID nào?</p>
<details><summary>Xem đáp án</summary>
<p><code>vn.toinoi.customer</code>, tức xcconfig bị bỏ qua. Theo tài liệu Apple, Xcode áp settings của project, rồi settings của file xcconfig, rồi settings của target, và cái áp sau thắng. Mục 3 cho thấy cách xoá dòng ghi đè và một script bắt lỗi này.</p>
</details>
</div>

## 1. Một tên flavor, hai bộ công cụ build

Flutter không biết "staging" nghĩa là gì. Cờ `--flavor staging` chỉ là một chuỗi truyền cho hai bên. Gradle tìm một `productFlavor` tên `staging`. Xcode tìm một scheme tên `staging` và các build configuration `Debug-staging`, `Profile-staging`, `Release-staging`. Mỗi bên tự quyết ID, tên, icon, file Firebase. Chuỗi đó còn trở thành `appFlavor` trong Dart, mà bài 25 so với khoá `FLAVOR` của file cấu hình.

Bảng dưới là ba flavor của app khách theo bảng §3.2 của khung loạt bài:

| Flavor | Application ID / Bundle ID | Tên trên màn hình chính | Icon giữ chỗ | Firebase app |
|---|---|---|---|---|
| `dev` | `vn.toinoi.customer.dev` | Tới Nơi Dev | nền cam, dải đen | app `dev` |
| `staging` | `vn.toinoi.customer.staging` | Tới Nơi Stg | nền xanh dương, dải đen | app `staging` |
| `prod` | `vn.toinoi.customer` | Tới Nơi | nền xanh lá | app `prod` |

App tài xế theo đúng khuôn, với `vn.toinoi.driver` làm gốc. ID của `prod` không có hậu tố; theo tài liệu Android, đổi application ID sau khi phát hành khiến Google Play coi đó là app khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 1. Lệnh flutter build với --flavor staging và --dart-define-from-file đi tới ba nơi. Cờ --flavor đi tới Gradle, nơi productFlavor staging cho ra APK vn.toinoi.customer.staging. Cờ --flavor cũng đi tới Xcode, nơi scheme và configuration staging cho ra IPA cùng ID. File cấu hình đi vào code Dart, nơi lúc khởi động appFlavor được so với FLAVOR bằng AppConfig của bài 25."><defs><marker id="fl50-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="125" width="230" height="150" rx="10" class="f-surface s-line"/><text x="36" y="158" class="t-b">flutter build apk</text><text x="36" y="188" class="t-mono">--flavor staging</text><text x="36" y="220" class="t-mono">--dart-define-from-file=</text><text x="36" y="240" class="t-mono">config/staging.json</text>
<rect x="350" y="20" width="290" height="110" rx="10" class="f-phys-soft s-phys"/><text x="366" y="50" class="t-b">Gradle (Android)</text><text x="366" y="76" class="t-small">productFlavors + hậu tố ID</text><text x="366" y="98" class="t-small">tên, icon: src/staging/res</text><text x="366" y="118" class="t-small">Firebase: src/staging/</text>
<rect x="350" y="145" width="290" height="110" rx="10" class="f-phys-soft s-phys"/><text x="366" y="175" class="t-b">Xcode (iOS)</text><text x="366" y="201" class="t-small">scheme staging</text><text x="366" y="223" class="t-small">cấu hình *-staging, xcconfig</text><text x="366" y="243" class="t-small">Run Script chép plist</text>
<rect x="350" y="270" width="290" height="110" rx="10" class="f-pk-soft s-pk"/><text x="366" y="300" class="t-b">Code Dart</text><text x="366" y="326" class="t-small">hằng lúc biên dịch:</text><text x="366" y="348" class="t-mono">FLAVOR, API_BASE_URL</text><text x="366" y="368" class="t-small">và appFlavor từ --flavor</text>
<path d="M252 160 L348 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-1-a)"/><text x="262" y="102" class="t-small">--flavor</text>
<path d="M252 200 L348 200" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-1-a)"/><text x="262" y="190" class="t-small">--flavor</text>
<path d="M252 240 L348 325" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-1-a)"/><text x="262" y="296" class="t-small">define</text>
<rect x="670" y="20" width="190" height="110" rx="10" class="f-surface s-line"/><text x="686" y="50" class="t-b">APK / AAB</text><text x="686" y="78" class="t-mono">vn.toinoi.customer</text><text x="686" y="98" class="t-mono">.staging</text>
<rect x="670" y="145" width="190" height="110" rx="10" class="f-surface s-line"/><text x="686" y="175" class="t-b">IPA</text><text x="686" y="203" class="t-mono">vn.toinoi.customer</text><text x="686" y="223" class="t-mono">.staging</text>
<rect x="670" y="270" width="190" height="110" rx="10" class="f-warn-soft s-warn"/><text x="686" y="300" class="t-b">Lúc khởi động</text><text x="686" y="326" class="t-small">so appFlavor với</text><text x="686" y="346" class="t-small">FLAVOR (AppConfig)</text>
<path d="M642 75 L668 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-1-a)"/>
<path d="M642 200 L668 200" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-1-a)"/>
<path d="M642 325 L668 325" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-1-a)"/>
</svg>
</div>
<figcaption>Hình 1. Một lệnh build, ba nơi đọc. Gradle và Xcode độc lập với nhau nên mỗi bên phải tự khớp với cùng một tên flavor; chỉ phía Dart thấy được cả hai nguồn, nên phép so appFlavor với FLAVOR nằm ở đó. AppConfig của bài 25 dừng app khi lệch; <code>FlavorScreen</code> của lab chỉ hiện cảnh báo.</figcaption>
</figure>

Ba nơi đọc độc lập nên có thể lệch nhau. Lab lập một "hợp đồng" `flavors.json` chứa sáu ID và tên hiển thị, rồi đối chiếu mọi file native với nó. Test gõ lại sáu ID theo §3.2 để hợp đồng không tự xác nhận chính nó.

## 2. Android: productFlavors và source set

Gradle gom một phần cấu hình vào `productFlavors`. Đây là phần ta thêm vào `build.gradle.kts` của template:

### build.gradle.kts · 50-flavors-build/native-config/customer/android/app/build.gradle.kts (trích)

```kotlin
android {
    namespace = "vn.toinoi.customer"
    // ...
    defaultConfig {
        applicationId = "vn.toinoi.customer"
        // ...
    }

    flavorDimensions += "env"
    productFlavors {
        create("dev") {
            dimension = "env"
            applicationIdSuffix = ".dev"
        }
        create("staging") {
            dimension = "env"
            applicationIdSuffix = ".staging"
        }
        create("prod") {
            dimension = "env"
        }
    }
}
```

Mỗi flavor phải thuộc một dimension (khi chỉ có một, plugin tự gán, nhưng ta khai rõ). Application ID cuối cùng là `applicationId` cộng `applicationIdSuffix`; prod giữ nguyên gốc.

Hai khái niệm dễ lẫn: `namespace` là package của code Kotlin và của lớp `R`, `BuildConfig` sinh ra; `applicationId` là danh tính của app trên máy và Google Play. Ta chỉ đổi cái sau theo flavor, nên `MainActivity` vẫn ở `vn.toinoi.customer` cả ba bản. Tài liệu Android khuyên khai `applicationId` tường minh, vì nếu không nó lấy giá trị của `namespace`.

Tên app và icon đi theo **source set**: `src/<flavor>/res` chỉ có mặt trong build của flavor đó, và file cùng tên trong đó thắng file trong `main`. Tên app nằm trong `strings.xml` mỗi flavor:

### strings.xml · 50-flavors-build/native-config/customer/android/app/src/dev/res/values/strings.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">Tới Nơi Dev</string>
</resources>
```

Trong `src/main/AndroidManifest.xml` chỉ còn `android:label="@string/app_name"` và `android:icon="@mipmap/ic_launcher"`. Có người quen đặt `app_name` bằng `resValue(...)` trong Gradle. Với AGP 9, `resValues` bị tắt mặc định; trang flavors của Flutter cảnh báo build sẽ lỗi và khuyên chuyển sang `strings.xml`, nên lab không dùng `resValue` và test chặn nó.

Icon theo cách tương tự: `src/dev/res/mipmap-*/ic_launcher.png` và `src/staging/...` ghi đè icon mặc định ở `main`, còn prod dùng thẳng icon mặc định đó. Ta làm đủ năm mật độ, vì ở mật độ thiếu, file của `main` không bị ghi đè nên vẫn được chọn (`icons_test.dart` kiểm). Icon trong lab là ảnh giữ chỗ do `tool/make-placeholder-icons.mjs` sinh; icon thích ứng (`mipmap-anydpi-v26`) chưa làm.

Gõ sai tên flavor thì công cụ Flutter đọc danh sách task của Gradle và in các flavor có thật (hàm `flavorUndefinedHandler` trong `flutter_tools`), kết thúc bằng "You must specify a --flavor option to select one of them".

## 3. iOS: scheme, configuration và xcconfig

Xcode không có khái niệm flavor. Theo trang flavors iOS của Flutter, một flavor là một **scheme** cộng các **build configuration** tên `<Chế độ>-<flavor>`. Ta làm một lần trong Xcode:

1. **Product > Scheme > New Scheme**, target Runner, tên `dev`; lặp cho `staging` và `prod`. Mở **Manage Schemes** và tích **Shared**: tài liệu Flutter nói scheme phải được chia sẻ thì flavor mới hoạt động đúng.
2. Thêm xcconfig vào project: **File > Add Files to "Runner"**, chọn chín file `Flutter/<Chế độ>-<flavor>.xcconfig`, bỏ chọn mọi target để Xcode không nhúng chúng vào bundle (theo Apple). Template không có synchronized group (`project.pbxproj` sinh ra không có `PBXFileSystemSynchronizedRootGroup`), nên chép file ra đĩa chưa tạo file reference và bước 4 sẽ không liệt kê chúng. Đây là hiểu biết về Xcode, chưa chạy thử.
3. Project Runner, tab **Info**, mục Configurations: nhân bản `Debug`, `Profile`, `Release` thành chín cấu hình như `Debug-dev`, `Release-staging`. Phần flavor viết thường.
4. Với target Runner, gắn mỗi cấu hình vào đúng file xcconfig cùng tên.
5. **Manage Schemes**: scheme `dev` dùng `Debug-dev` cho Run, Test, Analyze, `Profile-dev` cho Profile, `Release-dev` cho Archive.

Template Flutter 3.47.4 dùng Swift Package Manager, không có Podfile. Dự án có Podfile thì tài liệu yêu cầu thêm chín cấu hình vào khối `project 'Runner'` của nó.

Theo `XcodeProjectInfo` trong `flutter_tools`, scheme khớp flavor không phân biệt hoa thường và configuration mong đợi là `Debug-<scheme>`. Không thấy thì công cụ thử khớp gần đúng, rồi quay về `Debug` kèm cảnh báo `Unable to find "Debug-dev" build configuration`. Build vẫn chạy, nhưng mang ID mặc định.

Giá trị theo flavor nằm trong ba file xcconfig nhỏ; mỗi cấu hình chỉ `#include` base rồi file flavor:

### dev.xcconfig · 50-flavors-build/native-config/customer/ios/Flutter/Flavors/dev.xcconfig

```text
TN_FLAVOR = dev
PRODUCT_BUNDLE_IDENTIFIER = vn.toinoi.customer.dev
APP_DISPLAY_NAME = Tới Nơi Dev
ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon-dev
```

### Debug-dev.xcconfig · 50-flavors-build/native-config/customer/ios/Flutter/Debug-dev.xcconfig

```text
#include "Debug.xcconfig"
#include "Flavors/dev.xcconfig"
```

`Profile-dev` và `Release-dev` đều `#include "Release.xcconfig"`, vì theo tài liệu Flutter, Profile và Release dùng chung base. `Debug.xcconfig` gốc kéo theo `Generated.xcconfig` của công cụ Flutter, nên ta include base thay vì chép. `TN_FLAVOR` là biến riêng để script Firebase ở mục 4 chọn thư mục.

`Info.plist` lấy tên từ biến: `CFBundleDisplayName` là `$(APP_DISPLAY_NAME)`, còn `CFBundleIdentifier` đã là `$(PRODUCT_BUNDLE_IDENTIFIER)` sẵn trong template. Ta không đụng `PRODUCT_NAME`. Icon là ba bộ `AppIcon`, `AppIcon-dev`, `AppIcon-staging` trong `Assets.xcassets`, mỗi bộ một ảnh 1024 điểm, chọn bằng `ASSETCATALOG_COMPILER_APPICON_NAME`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Thứ tự Xcode áp settings: project, rồi file xcconfig, rồi target Runner; cái áp sau thắng. Nếu target không đặt PRODUCT_BUNDLE_IDENTIFIER, giá trị từ xcconfig vn.toinoi.customer.dev được giữ. Nếu template chưa sửa, target đặt vn.toinoi.customer và thắng xcconfig."><defs><marker id="fl50-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl50-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="28" class="t-small">Xcode áp theo thứ tự này, giá trị đặt sau thắng giá trị đặt trước</text>
<rect x="20" y="44" width="220" height="110" rx="10" class="f-surface s-line"/><text x="36" y="76" class="t-b">1. Project</text><text x="36" y="102" class="t-small">Build Settings của</text><text x="36" y="122" class="t-small">project Runner</text>
<rect x="300" y="44" width="290" height="110" rx="10" class="f-pk-soft s-pk"/><text x="316" y="76" class="t-b">2. File xcconfig</text><text x="316" y="102" class="t-mono">Debug-dev.xcconfig</text><text x="316" y="122" class="t-small">Debug.xcconfig, rồi</text><text x="316" y="142" class="t-small">Flavors/dev.xcconfig</text>
<rect x="650" y="44" width="210" height="110" rx="10" class="f-phys-soft s-phys"/><text x="666" y="76" class="t-b">3. Target Runner</text><text x="666" y="102" class="t-small">Build Settings của</text><text x="666" y="122" class="t-small">chính target</text>
<path d="M242 99 L298 99" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-2-a)"/>
<path d="M592 99 L648 99" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-2-a)"/>
<rect x="150" y="240" width="330" height="110" rx="10" class="f-pk-soft s-pk"/><text x="166" y="272" class="t-b">Target không đặt gì</text><text x="166" y="298" class="t-small">xcconfig còn hiệu lực:</text><text x="166" y="320" class="t-mono">vn.toinoi.customer.dev</text>
<rect x="540" y="240" width="320" height="110" rx="10" class="f-warn-soft s-warn"/><text x="556" y="272" class="t-b">Template chưa sửa</text><text x="556" y="298" class="t-small">target đặt giá trị cứng, thắng:</text><text x="556" y="320" class="t-mono">vn.toinoi.customer</text>
<path d="M445 156 L445 238" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-2-a)"/>
<path d="M755 156 L755 238" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl50-2-w)"/>
</svg>
</div>
<figcaption>Hình 2. Với PRODUCT_BUNDLE_IDENTIFIER, kết quả phụ thuộc vào việc target Runner còn đặt giá trị hay không. Template Flutter 3.47.4 đặt sẵn giá trị này, cùng ASSETCATALOG_COMPILER_APPICON_NAME, ngay trong target.</figcaption>
</figure>

Đã đọc `project.pbxproj` do `flutter create` sinh: cấu hình Debug của target Runner vừa gắn `Debug.xcconfig` vừa đặt `PRODUCT_BUNDLE_IDENTIFIER` và `ASSETCATALOG_COMPILER_APPICON_NAME`. Sau khi nhân bản cấu hình, ta xoá hai dòng đó trong Build Settings của target. Lab có `tool/check-runner.mjs` đọc `project.pbxproj` và kiểm hai điều: đủ chín cấu hình gắn đúng xcconfig, và target không còn đặt hai khoá kia. Script này chỉ xét khối của target Runner, không xét khối cùng tên ở mức project; trên pbxproj dựng từ template, nó báo hai khoá ghi đè ở mỗi configuration.

## 4. Firebase theo flavor: mỗi bản một app

Ba bản Tới Nơi thuộc ba app Firebase, nên mỗi bản cần cấu hình riêng. Mỗi nền tảng chọn file theo một cách.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 3. Chọn cấu hình Firebase theo flavor. Android: variant stagingDebug, rồi plugin google-services tìm src/staging/google-services.json, rồi so package_name với applicationId. iOS: configuration Debug-staging đặt TN_FLAVOR, rồi Run Script chép Runner/Firebase/staging/GoogleService-Info.plist vào bundle, rồi kiểm BUNDLE_ID bằng bundle ID đang build."><defs><marker id="fl50-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="24" class="t-b">Android</text>
<rect x="20" y="36" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="66" class="t-b">Variant stagingDebug</text><text x="36" y="94" class="t-small">flavor staging, build</text><text x="36" y="114" class="t-small">type debug</text>
<rect x="315" y="36" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="331" y="66" class="t-b">Plugin google-services</text><text x="331" y="94" class="t-small">tìm trong src/staging/</text><text x="331" y="116" class="t-mono">google-services.json</text>
<rect x="610" y="36" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="626" y="66" class="t-b">So package_name</text><text x="626" y="94" class="t-small">cần một client khớp</text><text x="626" y="114" class="t-small">applicationId đang build</text>
<path d="M272 91 L313 91" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-3-a)"/>
<path d="M567 91 L608 91" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-3-a)"/>
<text x="20" y="196" class="t-b">iOS</text>
<rect x="20" y="208" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="238" class="t-b">Debug-staging</text><text x="36" y="266" class="t-small">xcconfig đặt</text><text x="36" y="288" class="t-mono">TN_FLAVOR = staging</text>
<rect x="315" y="208" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="331" y="238" class="t-b">Run Script phase</text><text x="331" y="266" class="t-small">chép Runner/Firebase/</text><text x="331" y="286" class="t-small">staging/ vào gốc bundle</text>
<rect x="610" y="208" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="626" y="238" class="t-b">Kiểm BUNDLE_ID</text><text x="626" y="266" class="t-small">so với bundle ID đang</text><text x="626" y="286" class="t-small">build, lệch thì dừng</text>
<path d="M272 263 L313 263" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-3-a)"/>
<path d="M567 263 L608 263" class="f-none s-pk" stroke-width="2" marker-end="url(#fl50-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Android để plugin tự chọn file theo vị trí thư mục; iOS không có cơ chế tương đương nên một Run Script phase chép file theo <code>TN_FLAVOR</code>. Cả hai đều có bước đối chiếu ID.</figcaption>
</figure>

**Android.** File thật `google-services.json` đặt ở `android/app/src/<flavor>/`. Trang tài liệu Firebase chỉ liệt kê `src/<build_type>` và `src/<build_type>/<flavor>`; vị trí `src/<flavor>` lấy từ mã nguồn plugin (hàm `getJsonLocations`, kiểm 11/2026), nên cần kiểm lại khi nâng plugin. Plugin so `package_name` của từng `client` trong file với application ID đang build, và ném lỗi `No matching client found for package name` nếu không client nào khớp. Thiếu hẳn file thì task cũng ném lỗi, vì `missingGoogleServicesStrategy` mặc định là ERROR (đã đọc mã nguồn plugin). Plugin (4.5.0, kiểm 11/2026) khai ở `settings.gradle.kts` và bật ở `app/build.gradle.kts`.

Một file có ba client cũng chạy được, vì plugin chọn client theo `package_name`. Ta vẫn tách file: tài liệu Firebase nói file riêng cho production cho phép giữ một project Firebase riêng.

**iOS.** Xcode không chọn `GoogleService-Info.plist` theo configuration, nên một Run Script phase chép đúng file vào bundle. Tạo phase: target Runner, tab **Build Phases**, dấu **+** rồi **New Run Script Phase**, kéo xuống dưới "Copy Bundle Resources", và điền nội dung `sh "${SRCROOT}/scripts/copy-google-service-info.sh"` (chưa chạy thử trong Xcode). Script:

### copy-google-service-info.sh · 50-flavors-build/native-config/customer/ios/scripts/copy-google-service-info.sh (trích)

```sh
set -eu

: "${TN_FLAVOR:?TN_FLAVOR chưa đặt: Flavors/<flavor>.xcconfig chưa được áp vào configuration này}"

source_plist="${PROJECT_DIR}/Runner/Firebase/${TN_FLAVOR}/GoogleService-Info.plist"
target_dir="${TARGET_BUILD_DIR}/${UNLOCALIZED_RESOURCES_FOLDER_PATH}"

if [ ! -f "$source_plist" ]; then
  echo "error: thiếu $source_plist. Tải file của flavor ${TN_FLAVOR} từ Firebase console (file thật không commit)." >&2
  exit 1
fi

# BUNDLE_ID trong plist phải bằng bundle ID đang build (PlistBuddy có sẵn trên macOS).
if [ -x /usr/libexec/PlistBuddy ]; then
  plist_bundle_id=$(/usr/libexec/PlistBuddy -c "Print :BUNDLE_ID" "$source_plist")
  if [ "$plist_bundle_id" != "${PRODUCT_BUNDLE_IDENTIFIER}" ]; then
    echo "error: $source_plist có BUNDLE_ID $plist_bundle_id nhưng đang build ${PRODUCT_BUNDLE_IDENTIFIER}." >&2
    exit 1
  fi
fi

mkdir -p "$target_dir"
cp "$source_plist" "$target_dir/GoogleService-Info.plist"
```

`TARGET_BUILD_DIR` và `UNLOCALIZED_RESOURCES_FOLDER_PATH` có trong tài liệu build settings của Apple. Template Flutter đặt `ENABLE_USER_SCRIPT_SANDBOXING = NO`; nếu bật sandbox, phase phải khai input và output file. Test chạy script thật với một `PROJECT_DIR` giả: chép đúng file theo flavor, dừng khi chỉ có bản `.example`, dừng khi thiếu `TN_FLAVOR`, dừng khi plist của flavor khác bị đặt nhầm.

Repo chỉ chứa bản `.example` với giá trị giả (`FAKE_API_KEY_REPLACE_ME`). `.gitignore` chặn đúng tên `google-services.json` và `GoogleService-Info.plist`, không chặn đuôi `.example`; dòng `/android/`, `/ios/` có neo `/` để không nuốt `native-config/`. Lab chưa khởi tạo Firebase trong Dart.

## 5. Kiểm bằng script, build bằng một lệnh, áp dụng vào toi-noi-app

Đọc bằng mắt không đủ: sáu ID, sáu tên, hai mươi bốn xcconfig, mười hai file Firebase mẫu, sai một chỗ là cài đè. `tool/verify-flavors.mjs` chỉ cần Node, đối chiếu mọi file với hợp đồng và in hết vấn đề. Phép kiểm Gradle của nó:

### verify-flavors.mjs · 50-flavors-build/tool/verify-flavors.mjs (trích)

```js
const id = (gradle.applicationId ?? '') + spec.suffix;
if (gradleIds.has(id)) fail(`${where}: flavor ${gradleIds.get(id)} và ${flavor} cùng ra application ID ${id} (trùng, không cài cạnh nhau được)`);
gradleIds.set(id, flavor);
const expected = contract.apps[app].ids[flavor];
if (id !== expected) fail(`${where}: flavor ${flavor} ra application ID ${id}, hợp đồng là ${expected}`);
```

Script còn kiểm xcconfig, `Info.plist`, icon khác nhau, file Firebase mẫu không chứa khoá thật và `FLAVOR` trong `config/<flavor>.json`. Test Dart đọc lại các file bằng bộ phân tích riêng, rồi chạy script trên bản sao bị làm hỏng từng chỗ:

### verify_script_test.dart · 50-flavors-build/test/verify_script_test.dart (trích)

```dart
'Debug-dev include nhầm flavor staging': (
  (c) => c.mutate(
    'native-config/customer/ios/Flutter/Debug-dev.xcconfig',
    'Flavors/dev.xcconfig',
    'Flavors/staging.xcconfig',
  ),
  '#include phải là',
),
```

Hai mươi test case như vậy đều phải khiến script thoát mã 1 với thông báo nêu đúng chỗ hỏng. Lệnh build thì ghép từ một tên duy nhất, để `--flavor prod` không đi với `config/dev.json`:

### flavor-command.mjs · 50-flavors-build/tool/flavor-command.mjs (trích)

```js
const config = `config/${flavor}.json`;
// ...
const args = [...targets[target], '--flavor', flavor, `--dart-define-from-file=${config}`, ...extra];
console.log(`flutter ${args.join(' ')}`);
```

Các lệnh ra từ đó, chạy trong thư mục lab hoặc thư mục app:

```bash
flutter run --flavor dev --dart-define-from-file=config/dev.json
flutter build apk --flavor staging --dart-define-from-file=config/staging.json
flutter build appbundle --flavor prod --dart-define-from-file=config/prod.json
flutter build ipa --flavor prod --dart-define-from-file=config/prod.json
```

`run` và cả ba lệnh `build` đều nhận `--flavor` (đã đọc `--help`); `default-flavor` trong pubspec bị `--flavor` ghi đè. Ký `appbundle` và `ipa` là việc của bài 51 và 52.

**Áp dụng vào `toi-noi-app`.** Workspace thật không commit runner. Ta chạy `node examples/toi-noi-flutter/toi-noi-app/tool/create-runners.mjs`; script dùng `--project-name customer` và `--org vn.toinoi`, nên `namespace` và `applicationId` gốc là `vn.toinoi.customer`, khớp lab. Rồi chép `native-config/customer/android/.` vào `apps/customer/android/` và `native-config/customer/ios/.` vào `apps/customer/ios/`. Trước khi chép `ios/`, xoá các PNG cũ trong `Runner/Assets.xcassets/AppIcon.appiconset`: `Contents.json` mới chỉ nhắc `icon-1024.png`, PNG cũ sẽ nằm thừa. App tài xế làm tương tự với thư mục `driver`; lab sinh cây này từ cây của app khách bằng `tool/derive-driver-config.mjs` (đổi ID gốc, tên, id Firebase giả) và icon hình vuông bằng `make-placeholder-icons.mjs`, còn script kiểm tra cả hai app. Scheme, configuration, file reference và Run Script phase vẫn làm thủ công trong Xcode. `.gitignore` của workspace đang chặn ruột `android/` và `ios/`, nên việc chọn commit file nào thuộc về bài tích hợp.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi tách flavor</span>
<ul>
<li><strong>Target Runner còn đặt <code>PRODUCT_BUNDLE_IDENTIFIER</code> hay <code>ASSETCATALOG_COMPILER_APPICON_NAME</code>.</strong> Dấu hiệu: ba bản iOS cùng ID hoặc cùng icon dù xcconfig đúng. Nguyên nhân: settings của target áp sau xcconfig. Cách sửa: xoá hai dòng trong target, chạy <code>check-runner.mjs</code>.</li>
<li><strong>Tên configuration không theo <code>&lt;Chế độ&gt;-&lt;flavor&gt;</code>.</strong> Dấu hiệu: cảnh báo <code>Unable to find "Debug-dev" build configuration</code>, bản build mang ID mặc định. Nguyên nhân: công cụ quay về configuration gốc. Cách sửa: đổi tên cho đúng, phần flavor viết thường.</li>
<li><strong>Đặt <code>app_name</code> bằng <code>resValue</code> trên AGP 9.</strong> Dấu hiệu: lỗi <code>contains custom resource values, but the feature is disabled</code> (câu trích từ trang flavors của Flutter). Nguyên nhân: <code>resValues</code> tắt mặc định. Cách sửa: <code>strings.xml</code> trong <code>src/&lt;flavor&gt;/res/values</code>.</li>
<li><strong>Thiếu file hoặc lệch <code>package_name</code> trong <code>google-services.json</code>.</strong> Dấu hiệu: thiếu file thì build Android dừng với <code>File google-services.json is missing.</code>; lệch <code>package_name</code> thì dừng với <code>No matching client found for package name</code>. Nguyên nhân: không có file ở đúng <code>src/&lt;flavor&gt;/</code>, hoặc không client nào trùng application ID của flavor. Cách sửa: đăng ký đúng app trong Firebase, tải file về <code>src/&lt;flavor&gt;/</code>.</li>
<li><strong>Chép nhầm plist của flavor khác, hoặc commit file thật.</strong> Dấu hiệu: push và analytics của bản dev chảy vào app prod. Nguyên nhân: không bước nào đối chiếu ID. Cách sửa: giữ phép so <code>BUNDLE_ID</code> trong script, và cho <code>.gitignore</code> chặn tên file thật.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/50-flavors-build
flutter pub get
flutter analyze
flutter test
node tool/verify-flavors.mjs
node tool/flavor-command.mjs apk staging --dry-run
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` **131 test xanh**. Mức: **Kiểm chứng cấu hình**. Test đọc file và chạy script, không đụng Gradle, Xcode hay mạng.

- Hợp đồng, Android, iOS: sáu ID khớp §3.2, `app_name`, xcconfig và icon theo flavor, cho cả hai app.
- Firebase: `package_name` và `BUNDLE_ID` khớp ID, chỉ giá trị giả, `.gitignore` chặn file thật mà không chặn `.example`.
- Script: `verify-flavors.mjs` bắt hai mươi trường hợp hỏng; `check-runner.mjs` bắt thiếu cấu hình và target ghi đè; script chép plist chạy thật với `PROJECT_DIR` giả.
- Dart: `checkFlavors` và `FlavorScreen` cho bốn kết quả của hai nguồn flavor.

Đã chạy thêm ngoài test: sinh runner bằng `flutter create`, chép `native-config/customer` vào đó và `flutter analyze` sạch; `plutil -lint` các plist và `sh -n` script đều đạt. `flutter build apk --flavor staging --dart-define-from-file=config/staging.json` dừng ngay với `No Android SDK found`.

**Chưa làm**: biên dịch Kotlin DSL bằng Gradle (chỉ đối chiếu template và tài liệu); mọi thao tác trong Xcode (scheme, file reference, configuration, Run Script phase) và `xcodebuild`; cài ba bản cạnh nhau trên thiết bị; Firebase thật; icon thích ứng và icon thiết kế thật; cây `native-config/driver` chỉ sinh từ cây app khách; chọn file cấu hình trong CI (bài 54); ký ứng dụng (bài 51, 52).

## Bài tập

1. Thêm flavor `qa` (`vn.toinoi.customer.qa`). Liệt kê file phải đổi và nói vì sao script đỏ ngay.
2. Bản staging vẫn đè lên bản dev dù đã có hậu tố. Nêu ba nguyên nhân và cách xác định.
3. `Profile-dev.xcconfig` include `Release.xcconfig`, không phải `Debug.xcconfig`. Vì sao, và test nào đỏ nếu đổi nhầm?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Phải thêm: mục trong <code>flavors.json</code>, <code>create("qa")</code> trong <code>build.gradle.kts</code>, <code>src/qa/res</code> (tên và năm icon), bốn xcconfig, bộ <code>AppIcon-qa</code>, hai file Firebase mẫu, <code>config/qa.json</code>, scheme và ba configuration trong Xcode. Script đỏ ngay vì <code>flavors.json</code> phải liệt kê đúng <code>dev, staging, prod</code>: ba flavor là hợp đồng của loạt bài, nên thêm <code>qa</code> nghĩa là sửa cả script và test.</p>
<p><strong>2.</strong> (a) Hai flavor cùng ra một ID, ví dụ do hậu tố trùng: <code>verify-flavors.mjs</code> báo "cùng ra application ID". (b) Hai lần cài thực ra cùng một flavor vì lệnh build gõ nhầm tên: xem lệnh thật, dùng <code>flavor-command.mjs</code>. (c) Phía iOS, target Runner còn ghi đè bundle ID: <code>check-runner.mjs</code>. Nhớ rằng đổi hậu tố không đổi <code>namespace</code>, nên đó không phải nguyên nhân.</p>
<p><strong>3.</strong> Tài liệu Flutter nêu Profile và Release dùng chung base <code>Release.xcconfig</code>. Nếu <code>Profile-dev</code> include <code>Debug.xcconfig</code>, test case <code>Profile-dev quên đổi base sang Release.xcconfig</code> của <code>verify_script_test.dart</code> và test <code>mỗi &lt;Mode&gt;-&lt;flavor&gt;.xcconfig include base rồi file của flavor</code> của <code>ios_config_test.dart</code> đỏ.</p>
</details>

Bài trước: [Bài tổng hợp: đặt chuyến end-to-end giữa app khách, app tài xế và trip-service](/articles/flutter-tong-hop-phan-6-dat-chuyen-end-to-end-app-khach-tai-xe). Bài tiếp: [Ký ứng dụng Android: keystore, Play App Signing, AAB và versionCode](/articles/flutter-ky-ung-dung-android-keystore-play-app-signing-aab).

## Tài liệu gốc

- [Set up Flutter flavors for Android](https://docs.flutter.dev/deployment/flavors) và [Set up Flutter flavors for iOS and macOS](https://docs.flutter.dev/deployment/flavors-ios) (docs.flutter.dev).
- [Configure build variants](https://developer.android.com/build/build-variants) và [Configure your app module](https://developer.android.com/build/configure-app-module) (developer.android.com).
- [Android Gradle plugin 9.0 release notes](https://developer.android.com/build/releases/agp-9-0-0-release-notes) (developer.android.com).
- [The Google services Gradle plugin and config file](https://firebase.google.com/docs/android/google-services-plugin-and-file) (firebase.google.com).
- [Adding a build configuration file to your project](https://developer.apple.com/documentation/xcode/adding-a-build-configuration-file-to-your-project) và [Build settings reference](https://developer.apple.com/documentation/xcode/build-settings-reference) (developer.apple.com).
- [appFlavor](https://api.flutter.dev/flutter/services/appFlavor-constant.html) (api.flutter.dev).

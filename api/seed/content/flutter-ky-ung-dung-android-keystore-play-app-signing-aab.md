---
slug: flutter-ky-ung-dung-android-keystore-play-app-signing-aab
title: "Ký ứng dụng Android: keystore, Play App Signing, AAB và versionCode"
summary: "Bài 51: dựng chuỗi ký Android cho Tới Nơi: upload keystore bằng keytool, key.properties ngoài git, signingConfigs không lui về khoá debug, Play App Signing, AAB và APK, kiểm chữ ký bằng apksigner và jarsigner, versionCode luôn tăng. Gradle và Play chưa chạy."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, android, release, signing, google-play]
language: vi
---

Đội Tới Nơi chuẩn bị đưa bản staging của app khách lên Google Play. Người làm bản release đầu tiên tạo keystore trên laptop cá nhân, ký, tải lên. Ba tháng sau cô chuyển dự án, laptop bị xoá sạch, và không ai ký nổi bản cập nhật kế tiếp. Tình huống thứ hai hay gặp hơn: `flutter build appbundle` chạy xanh mà bản ra ký bằng khoá debug, vì template Flutter để sẵn dòng `signingConfig = signingConfigs.getByName("debug")`. Cả hai là minh hoạ, cơ chế thì thật.

Bài này dựng chuỗi ký của Android: upload keystore, `key.properties` ngoài git, `signingConfigs`, Play App Signing, AAB so với APK, cách kiểm chữ ký và `versionCode` luôn tăng. Đây là bài 51 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 50 về flavors khi build](/articles/flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build) trước: mỗi flavor có Application ID riêng, và Play dùng Application ID để phân biệt app. Lab dùng Flutter 3.47.4 và JDK 21.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Máy viết bài không có Android SDK và không có tài khoản Play Console. Gradle, <code>flutter build appbundle</code> và <code>apksigner</code> <strong>chưa chạy ở đâu cả</strong>. Một reviewer chạy Gradle 9.3.1 với AGP 9.1.0: <code>build.gradle.kts</code> của lab qua bước biên dịch script rồi dừng ở "SDK location not found", nên chưa có task graph thật. Chạy được là logic <code>versionCode</code>, đọc <code>key.properties</code>, kiểm văn bản Gradle, và <code>keytool</code>, <code>jarsigner</code> của JDK trên keystore thử. Chính sách của Google kiểm tại 11/2026 và đổi theo thời gian.</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>App Tới Nơi đã lên Google Play. Laptop giữ upload keystore bị mất, không có bản sao. Từ nay ta không bao giờ cập nhật được app này nữa. Đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p>Sai, với app dùng Play App Signing. Khoá ký bản gửi tới người dùng (app signing key) do Google giữ; upload key chỉ chứng minh ai đang tải lên. Mất nó thì tạo upload key mới và xin reset trong Play Console. Thảm hoạ chỉ có khi mất app signing key tự giữ (app tạo trước 8/2021 chưa bật Play App Signing).</p></details>
</div>

## Hai khoá và vai trò của Play App Signing

Android chỉ cài bản cập nhật nếu chứng chỉ ký khớp bản đang cài, nên khoá ký phải dùng được suốt vòng đời của app. Play App Signing tách khoá này thành hai (Hình 1). **App signing key** do Google giữ và ký APK gửi tới người dùng; ta không lấy lại được bản sao. **Upload key** do ta giữ và ký AAB trước khi tải lên; Google dùng chứng chỉ của nó để biết ai đang tải lên. Hai khoá nên khác nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 1. Máy dev hoặc CI giữ upload key và ký AAB. Play Console đối chiếu chứng chỉ upload đã đăng ký. Google giữ app signing key và ký APK tối ưu theo thiết bị gửi tới người dùng. Mất upload key thì tạo khoá mới và xin reset, app signing key không đổi."><defs><marker id="fl51-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl51-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="170" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="66" class="t-b">Máy dev, CI</text><text x="36" y="94" class="t-small">giữ upload key</text><text x="36" y="116" class="t-mono">.jks</text>
<rect x="300" y="30" width="190" height="110" rx="10" class="f-pk-soft s-pk"/><text x="316" y="66" class="t-b">Play Console</text><text x="316" y="94" class="t-small">chứng chỉ upload</text><text x="316" y="116" class="t-small">đã đăng ký</text>
<rect x="590" y="30" width="270" height="110" rx="10" class="f-phys-soft s-phys"/><text x="606" y="66" class="t-b">Google</text><text x="606" y="94" class="t-small">giữ app signing key</text><text x="606" y="116" class="t-small">ta không lấy lại được</text>
<path d="M192 85 L298 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl51-1-a)"/><text x="206" y="72" class="t-small">AAB đã ký</text>
<path d="M492 85 L588 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl51-1-a)"/><text x="506" y="72" class="t-small">đi ký</text>
<rect x="590" y="220" width="270" height="90" rx="10" class="f-pk-soft s-pk"/><text x="606" y="254" class="t-b">APK theo thiết bị</text><text x="606" y="282" class="t-small">ký bằng app signing key</text>
<path d="M725 142 L725 218" class="f-none s-pk" stroke-width="2" marker-end="url(#fl51-1-a)"/><text x="737" y="186" class="t-small">tách và ký</text>
<rect x="330" y="220" width="190" height="90" rx="10" class="f-surface s-muted"/><text x="346" y="254" class="t-b">Người dùng</text><text x="346" y="282" class="t-small">cài từ Play</text>
<path d="M588 265 L522 265" class="f-none s-pk" stroke-width="2" marker-end="url(#fl51-1-a)"/>
<rect x="20" y="220" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="36" y="252" class="t-b t-warn">Mất upload key</text><text x="36" y="280" class="t-small">tạo khoá mới, xin reset</text><text x="36" y="304" class="t-small">app signing key y nguyên</text>
<path d="M105 218 L105 142" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl51-1-w)"/><text x="117" y="186" class="t-small t-warn">khoá mới</text>
</svg>
</div>
<figcaption>Hình 1. Hai khoá của Play App Signing. Ta ký bằng upload key; Google ký bản tới tay người dùng bằng app signing key. Đường đứt là lối thoát khi mất upload key.</figcaption>
</figure>

Chính sách hiện hành (kiểm tại 11/2026): developer.android.com ghi app tạo từ tháng 8/2021 phải phát hành bằng AAB, và Play App Signing là bắt buộc để phân phối qua Google Play, trừ app tạo trước tháng đó. Nếu để Google tạo khoá, khoá ký bản đầu tiên trở thành upload key.

Mất upload key thì làm theo trang hỗ trợ của Play: tạo khoá mới, xuất chứng chỉ công khai bằng `keytool -exportcert -rfc`, chọn **Request upload key reset** trong Play Console và đưa file PEM. Tài liệu không nêu thời gian xử lý, nên đừng hẹn phát hành sát giờ. App signing key không đổi nên người dùng cập nhật bình thường. Lab chứng minh PEM xuất ra không chứa khoá riêng, và keystore mới có vân tay SHA-256 khác nên phải đăng ký lại chứng chỉ.

Staging và prod của Tới Nơi có Application ID khác nhau nên là hai app trên Play, mỗi app có app signing key riêng theo mặc định. Dùng chung upload keystore giảm số bí mật phải giữ; mỗi app một keystore thì sự cố không lan sang app khác. Đó là quyết định của đội.

## Tạo upload keystore bằng keytool

Lệnh trong tài liệu Flutter (macOS và Linux):

```bash
keytool -genkey -v -keystore ~/upload-keystore.jks -keyalg RSA \
        -storetype JKS -keysize 2048 -validity 10000 -alias upload
```

Lab sinh keystore **thử** bằng cùng các cờ, nhưng truyền mật khẩu qua biến môi trường (`-storepass:env`, `-keypass:env` theo trang man của `keytool`) để nó không hiện trong danh sách tiến trình.

### jdk_tools.dart · 51-android-signing/lib/jdk_tools.dart (trích, trong TestKeystore.generate)

```dart
    final result = await _run('keytool', [
      '-genkeypair',
      '-keystore',
      file.path,
      '-storetype',
      'JKS',
      '-keyalg',
      'RSA',
      '-keysize',
      '2048',
      '-validity',
      '$validityDays',
      '-alias',
      alias,
      '-dname',
      distinguishedName,
      '-storepass:env',
      _passwordEnv,
      '-keypass:env',
      _passwordEnv,
    ], password: password);
```

Các ràng buộc (kiểm tại 11/2026): upload key là RSA từ 2048 bit (trang hỗ trợ Play); chứng chỉ phải hết hạn sau 22/10/2033 và nên có hiệu lực từ 25 năm trở lên (developer.android.com), nên `-validity 10000` ngày, khoảng 27 năm, là hợp lý. Tài liệu Flutter nói `-storetype JKS` chỉ cần từ Java 9 vì mặc định đổi sang PKCS12; JDK 21 chấp nhận JKS nhưng có cảnh báo. Ba test của lab kiểm cỡ khoá, năm hết hạn và dạng vân tay SHA-256 mà `keytool -list -v -keystore upload-keystore.jks -alias upload` in ra.

Keystore sống **ngoài repository**, kèm ít nhất một bản sao ở kho bí mật của đội; [bài CI/CD](/articles/flutter-ci-cd-github-actions-macos-build-ios-android-secrets) đưa nó lên CI qua secrets. Lưu ý: vân tay SHA-256 đăng ký cho Firebase hay Google Sign-In phải lấy ở trang App signing của Play Console, vì bản người dùng cài ký bằng app signing key.

## key.properties và signingConfigs

Gradle cần bốn giá trị: đường dẫn keystore, mật khẩu keystore, alias và mật khẩu khoá. Ta đặt chúng trong `android/key.properties`, nằm ngoài git, và commit **mẫu** của nó:

### key.properties.example · 51-android-signing/native-config/key.properties.example

```properties
storeFile=/duong/dan/ngoai/repo/upload-keystore.jks
storePassword=CHANGE_ME
keyAlias=upload
keyPassword=CHANGE_ME
```

Template Flutter 3.47.4, và lab bài 50 kế thừa nó, để bản release ký bằng khoá debug (kèm comment TODO). docs.flutter.dev hướng dẫn đổi sang `signingConfigs.getByName("release")` nhưng không dặn gì về trường hợp thiếu `key.properties`. Lab thêm hai thứ: nhánh `release` không còn đường lui sang khoá debug, và một task dừng build sớm với thông báo rõ.

### build.gradle.kts · 51-android-signing/native-config/build.gradle.kts (trích)

```kotlin
    signingConfigs {
        create("release") {
            keyAlias = keystoreProperties.getProperty("keyAlias")
            keyPassword = keystoreProperties.getProperty("keyPassword")
            storeFile = keystoreProperties.getProperty("storeFile")?.let { file(it) }
            storePassword = keystoreProperties.getProperty("storePassword")
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("release")
        }
    }
```

Thiếu `key.properties` thì các giá trị trên là `null` và bước cấu hình không báo lỗi. Việc báo lỗi thuộc về task dưới đây, gắn làm phụ thuộc của các task `pre<Variant>ReleaseBuild`, nên chặn bản release trước khi ký mà không ảnh hưởng bản debug. Thứ tự giữa cổng này và bước biên dịch chưa quan sát được trên AGP thật (mã Flutter không nối `compileFlutterBuild*` với `pre*Build`):

### build.gradle.kts · 51-android-signing/native-config/build.gradle.kts (trích)

```kotlin
val verifyReleaseSigning by tasks.registering {
    val propsFile = keystorePropertiesFile
    val props = keystoreProperties
    val requiredKeys = requiredSigningKeys
    val appDir = projectDir
    doLast {
        if (!propsFile.exists()) {
            throw GradleException(
                "Thiếu ${propsFile.path}. Build release cần upload keystore: " +
                    "sao chép key.properties.example thành key.properties và điền giá trị. " +
                    "Bản release không tự ký bằng khoá debug."
            )
        }
        val missing = requiredKeys.filter { props.getProperty(it).isNullOrBlank() }
        if (missing.isNotEmpty()) {
            throw GradleException("key.properties thiếu khoá: ${missing.joinToString(", ")}")
        }
        val storeFile = appDir.resolve(props.getProperty("storeFile"))
        if (!storeFile.exists()) {
            throw GradleException("Không thấy upload keystore tại ${storeFile.path}")
        }
    }
}

tasks.configureEach {
    if (Regex("^pre.*ReleaseBuild$").matches(name)) {
        dependsOn(verifyReleaseSigning)
    }
}
```

Hình 2 vẽ luồng này. Bảo đảm của lab là test đọc văn bản: `auditGradleSigning` kiểm file đọc `key.properties` và đủ bốn khoá, nhánh release dùng `create("release")`, không có `signingConfigs.getByName("debug")`, không gõ cứng mật khẩu hay `versionCode`, và có `GradleException` gắn vào `pre*ReleaseBuild`. Đoạn mặc định của template, đưa vào làm mẫu hỏng, bị báo đường lui ký debug.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 2. flutter build appbundle kích hoạt bản release. verifyReleaseSigning chạy trước preReleaseBuild vì preReleaseBuild phụ thuộc vào nó. Nếu key.properties đủ khoá và keystore có thật thì chạy tiếp preReleaseBuild, bundleRelease và ký bằng upload key ra app-release.aab. Nếu thiếu thì GradleException dừng ngay, không ký bằng khoá debug."><defs><marker id="fl51-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl51-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="150" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="66" class="t-b">flutter build</text><text x="36" y="94" class="t-mono">appbundle</text>
<rect x="210" y="30" width="210" height="100" rx="10" class="f-pk-soft s-pk"/><text x="226" y="66" class="t-mono">verifyReleaseSigning</text><text x="226" y="94" class="t-small">kiểm key.properties</text>
<rect x="460" y="30" width="180" height="100" rx="10" class="f-phys-soft s-phys"/><text x="476" y="66" class="t-mono">preReleaseBuild</text><text x="476" y="94" class="t-small">chạy sau cổng</text>
<rect x="680" y="30" width="180" height="100" rx="10" class="f-phys-soft s-phys"/><text x="696" y="66" class="t-mono">bundleRelease</text><text x="696" y="94" class="t-small">đóng gói, rồi ký</text>
<path d="M172 80 L208 80" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-2-a)"/>
<path d="M422 80 L458 80" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-2-a)"/><text x="428" y="68" class="t-small">đủ</text>
<path d="M642 80 L678 80" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-2-a)"/>
<rect x="680" y="210" width="180" height="100" rx="10" class="f-pk-soft s-pk"/><text x="696" y="246" class="t-mono">app-release.aab</text><text x="696" y="274" class="t-small">ký: upload key</text>
<path d="M770 132 L770 208" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-2-a)"/>
<rect x="210" y="210" width="210" height="100" rx="10" class="f-warn-soft s-warn"/><text x="226" y="246" class="t-b t-warn">GradleException</text><text x="226" y="274" class="t-small">dừng sớm, báo rõ lý do</text>
<path d="M315 132 L315 208" class="f-none s-warn" stroke-width="2" marker-end="url(#fl51-2-w)"/><text x="327" y="176" class="t-small t-warn">thiếu</text>
<text x="20" y="346" class="t-small t-warn">Không có nhánh lui sang khoá debug: không ký được thì build hỏng.</text>
</svg>
</div>
<figcaption>Hình 2. Build release đi qua cổng verifyReleaseSigning, chạy trước preReleaseBuild. Thiếu key.properties thì dừng với thông báo rõ thay vì tự ký bằng khoá debug mà không báo gì.</figcaption>
</figure>

Hai chi tiết về `storeFile`. Đường dẫn tương đối được `file(it)` hiểu từ thư mục của module, tức `android/app`. Và `key.properties` được đọc bằng `java.util.Properties`, nơi `\` là ký tự thoát: tài liệu Flutter dặn viết `\\` trên Windows. Đường dẫn `C:\Users\quan\up.jks` làm lớp này ném `Malformed \uxxxx encoding`, vì `\u` (trong `\up`) mở đầu một ký tự Unicode; còn `\` trước chữ khác thì bị bỏ, nên `C:\Users\quan\keys.jks` trở thành `C:Usersquankeys.jks` mà không có lỗi nào. Ta chạy `java.util.Properties` của JDK 21 trên các mẫu này và đối chiếu vào `KeyProperties`; test dưới đây mô phỏng lỗi mà không cần Gradle:

### key_properties_test.dart · 51-android-signing/test/key_properties_test.dart (trích)

```dart
    test(r'một dấu \ rồi chữ u thì lỗi, như Gradle: C:\Users\quan\up.jks', () {
      expect(
        () => KeyProperties.parse(r'storeFile=C:\Users\quan\up.jks'),
        throwsA(
          isA<FormatException>().having(
            (e) => e.message,
            'message',
            contains('uxxxx'),
          ),
        ),
      );
    });
```

## AAB, APK và cách kiểm chữ ký

**AAB** (Android App Bundle) là định dạng để **xuất bản**: chứa mã và tài nguyên, còn việc tạo và ký APK giao cho Google Play. Khác APK, AAB không cài thẳng lên thiết bị được (developer.android.com). **APK** là thứ cài được, dùng khi cần gửi bản thử trực tiếp, như Firebase App Distribution ở bài 55.

| | AAB | APK |
|---|---|---|
| Dùng để | Tải lên Google Play | Cài thẳng, phân phối nội bộ |
| Ai ký bản người dùng nhận | Google, bằng app signing key | Ta, bằng khoá ta chọn |
| Kiểm chữ ký | `jarsigner -verify` | `apksigner verify --print-certs` |
| Lệnh Flutter | `flutter build appbundle` | `flutter build apk` |

`apksigner` nằm trong Android SDK build-tools; máy này không có nên lab chưa xem đầu ra thật của `apksigner verify --print-certs app.apk`. Về ký AAB, developer.android.com ghi rõ không dùng `apksigner` để ký mà dùng `jarsigner`; trang đó không nói riêng về kiểm tra, nên ta kiểm bằng `jarsigner -verify`, cùng công cụ với lúc ký. Lab ký và kiểm một **AAB giả** (file zip tối giản, `bundletool` sẽ không nhận) để thử đúng các lệnh `jarsigner` và `keytool` thật:

### jdk_tools.dart · 51-android-signing/lib/jdk_tools.dart (trích)

```dart
Future<BundleVerification> verifyBundle(File bundle) async {
  final result = await _run('jarsigner', ['-verify', bundle.path]);
  final out = result.output;
  if (result.exitCode != 0 || out.contains('SecurityException')) {
    return BundleVerification(SignatureStatus.invalid, out);
  }
  if (out.contains('jar is unsigned')) {
    return BundleVerification(SignatureStatus.unsigned, out);
  }
  if (out.contains('jar verified.')) {
    return BundleVerification(SignatureStatus.verified, out);
  }
  return BundleVerification(SignatureStatus.invalid, out);
}
```

Ba điều quan sát trên JDK 21, đều có test. Bundle **chưa ký** vẫn cho `jarsigner -verify` thoát mã 0 và in `jar is unsigned.`, nên CI chỉ đọc mã thoát sẽ cho qua AAB chưa ký; hàm trên đọc nội dung. Sửa một file sau khi ký thì `jarsigner` báo `SHA-256 digest error` và thoát mã 1. Và `-strict` thoát mã 4 vì chứng chỉ upload tự ký (trang man xếp `signerSelfSigned` vào mã 4), nên đừng bật nó cho upload key. Để biết ai đã ký, `keytool -printcert -jarfile app.aab` in chứng chỉ người ký cùng vân tay SHA-256, đóng vai trò như `apksigner --print-certs` cho AAB; test so kết quả này với vân tay keystore.

Với APK tách theo ABI (`flutter build apk --split-per-abi`), Flutter cộng `hệ số ABI * 1000` vào `versionCode`: armeabi-v7a là 1, arm64-v8a là 2, x86_64 là 4 (đọc từ `FlutterPluginConstants.kt` của Flutter 3.47.4; x86 đã bị gỡ). AAB không bị ảnh hưởng.

## versionCode luôn tăng

Flutter đọc `version: x.y.z+build` trong `pubspec.yaml`: `x.y.z` trở thành `versionName`, số sau dấu `+` trở thành `versionCode`; `--build-name` và `--build-number` ghi đè (docs.flutter.dev). Play đòi mỗi bản tải lên có `versionCode` lớn hơn bản trước, tối đa 2.100.000.000 (kiểm tại 11/2026). Play chỉ nhìn `versionCode`: `9.0.0+5` sau `1.4.2+37` vẫn bị từ chối dù phiên bản hiển thị tăng.

Lab gói quy tắc đó vào `AppVersion`:

### app_version.dart · 51-android-signing/lib/app_version.dart (trích)

```dart
  /// Kiểm bản này lớn hơn hẳn [previous] (versionCode đã phát hành).
  ///
  /// Ném [VersionCodeRegression] khi bằng hoặc thấp hơn.
  void ensureIncreasesFrom(int previous) {
    if (versionCode <= previous) {
      throw VersionCodeRegression(previous: previous, candidate: versionCode);
    }
  }

  int splitVersionCode(Abi abi) {
    final code = abi.versionCodeFactor * 1000 + build;
    if (code > maxVersionCode) {
      throw StateError('versionCode $code vượt giới hạn $maxVersionCode');
    }
    return code;
  }
```

`bin/check_version_code.dart pubspec.yaml --previous 37` thoát mã 0 nếu `build` lớn hơn 37, mã 1 nếu giảm hoặc trùng, mã 2 nếu dùng sai hoặc file lỗi (Hình 3). `--previous` là `versionCode` mới nhất đã lên Play; CI chịu trách nhiệm lấy nó. Bẫy mà test ghi lại: với `build` từ 1000 trở lên, `1037` trên armeabi-v7a trùng `37` trên arm64-v8a (cùng ra 2037), nên tránh build number lớn nếu còn phát hành APK tách ABI.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Ba bản đã phát hành có build 35, 36, 37. Bản mới phải có build lớn hơn 37: 38 hợp lệ. Build 30 là giảm và build 37 là trùng, cả hai bị chặn trước khi build."><defs><marker id="fl51-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl51-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="64" class="t-mono">1.4.0+35</text><text x="36" y="92" class="t-small">đã phát hành</text>
<rect x="240" y="30" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="256" y="64" class="t-mono">1.4.1+36</text><text x="256" y="92" class="t-small">đã phát hành</text>
<rect x="460" y="30" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="476" y="64" class="t-mono">1.4.2+37</text><text x="476" y="92" class="t-small">mới nhất ở Play</text>
<rect x="680" y="30" width="180" height="80" rx="10" class="f-pk-soft s-pk"/><text x="696" y="64" class="t-mono">1.4.3+38</text><text x="696" y="92" class="t-small">hợp lệ</text>
<path d="M202 70 L238 70" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-3-a)"/>
<path d="M422 70 L458 70" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-3-a)"/>
<path d="M642 70 L678 70" class="f-none s-phys" stroke-width="2" marker-end="url(#fl51-3-a)"/>
<rect x="460" y="170" width="400" height="90" rx="10" class="f-warn-soft s-warn"/><text x="476" y="204" class="t-b t-warn">Bị chặn trước khi build</text><text x="476" y="232" class="t-small">1.5.0+30 giảm; 1.4.2+37 trùng</text>
<path d="M550 112 L550 168" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl51-3-w)"/><text x="562" y="144" class="t-small t-warn">so với 37</text>
</svg>
</div>
<figcaption>Hình 3. Mỗi bản tải lên phải có build lớn hơn bản mới nhất đã lên Play. Giảm hoặc trùng đều là lỗi; kiểm sớm bằng script thay vì đợi Play Console từ chối.</figcaption>
</figure>

## Yêu cầu hiện hành của Play

Các con số dưới đây lấy từ trang chính thức, kiểm tại 11/2026, và phải kiểm lại trước mỗi lần phát hành.

| Yêu cầu | Giá trị | Nguồn |
|---|---|---|
| Target API của app mới và bản cập nhật | API 35 từ 31/8/2025; API 36 từ 31/8/2026; gia hạn tới 1/11/2026 (đã hết tại thời điểm viết) | support.google.com, Target API level requirements |
| Ngưỡng cho app hiện có | target tối thiểu API 35 từ 31/8/2026; dưới ngưỡng thì không hiện cho người dùng mới trên Android mới hơn target API của app | cùng trang |
| `versionCode` tối đa | 2.100.000.000 | support.google.com, Create and set up your app |

Flutter 3.47.4 đặt `targetSdkVersion = 36` trong `FlutterExtension.kt` của SDK, nên `targetSdk = flutter.targetSdkVersion` trong file Gradle của lab đáp ứng ngưỡng API 36. Ta không gõ cứng số, để khi Google nâng ngưỡng thì nâng Flutter (bài 63) là đủ.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi ký Android</span>
<ul>
<li><strong>Bản release ký bằng khoá debug.</strong> Dấu hiệu: Play từ chối, hoặc vân tay SHA-256 của bản build không khớp upload key đã đăng ký. Nguyên nhân: giữ dòng <code>getByName("debug")</code> của template. Cách sửa: dùng <code>signingConfigs.getByName("release")</code> và cổng <code>verifyReleaseSigning</code>; <code>auditGradleSigning</code> bắt lỗi này trong test.</li>
<li><strong>Commit nhầm keystore hoặc key.properties.</strong> Dấu hiệu: file nằm trong lịch sử git. Cách sửa: coi upload key là đã lộ, tạo khoá mới và xin reset, đổi mật khẩu trên CI, rồi mới dọn lịch sử. Phòng ngừa: <code>.gitignore</code> có <code>key.properties</code>, <code>*.jks</code>, <code>*.keystore</code>; <code>android/.gitignore</code> của template Flutter 3.47.4 đã có cả ba.</li>
<li><strong>Đường dẫn Windows một dấu \.</strong> Dấu hiệu: chỉ <code>\u</code> mới gây lỗi <code>Malformed \uxxxx encoding</code>; các <code>\</code> khác bị bỏ mất mà không báo lỗi (<code>C:\Users\quan\keys.jks</code> trở thành <code>C:Usersquankeys.jks</code>) và task báo <code>Không thấy upload keystore</code>. Cách sửa: viết <code>C:\\Users\\quan\\upload-keystore.jks</code>.</li>
<li><strong>versionCode giảm hoặc trùng.</strong> Dấu hiệu: Play từ chối khi tải lên. Cách sửa: tăng số sau dấu <code>+</code>; chạy <code>check_version_code</code> trước khi build.</li>
<li><strong>Tin mã thoát của <code>jarsigner -verify</code>.</strong> Dấu hiệu: CI xanh với AAB chưa ký. Cách sửa: đọc nội dung đầu ra như <code>verifyBundle</code>, và kiểm người ký bằng <code>keytool -printcert -jarfile</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

**Mức kiểm chứng: cấu hình (CH).** Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/51-android-signing
dart pub get
dart analyze
dart test
dart run bin/demo_signing.dart
dart run bin/check_version_code.dart pubspec.yaml --previous 0
```

Đã kiểm với Dart 3.13.3 (Flutter 3.47.4) và JDK 21: `dart analyze` 0 issue, `dart test` 70 test xanh. Không có JDK thì 12 test chạy `keytool` bị bỏ qua kèm lý do, 57 test còn lại vẫn chạy. Các test chứng minh:

- **Logic `versionCode`**: tách `x.y.z+build`, từ chối thiếu `+build` hoặc build ngoài miền, giảm và trùng bị chặn, hệ số ABI, mã thoát của lệnh kiểm.
- **`key.properties`**: đọc đúng luật `java.util.Properties` (đối chiếu tay với JDK 21), đủ bốn khoá, file mẫu chỉ chứa giá trị giữ chỗ.
- **Gradle và `.gitignore`**: file của lab qua `auditGradleSigning`, mẫu hỏng bị báo; `git check-ignore` xác nhận ba mẫu chặn bí mật còn `key.properties.example` không bị chặn.
- **JDK thật**: keystore thử RSA 2048 hết hạn sau 2033, PEM không chứa khoá riêng, AAB giả chưa ký, đã ký, bị sửa, và người ký khớp vân tay keystore. Keystore thử nằm trong thư mục tạm, mật khẩu ngẫu nhiên, bị xoá khi xong; `bin/demo_signing.dart` in vân tay SHA-256 của nó.

**Chưa làm**: chạy task graph thật của `build.gradle.kts` (reviewer chỉ qua được bước biên dịch script với Gradle 9.3.1 và AGP 9.1.0, rồi dừng ở "SDK location not found"), `flutter build appbundle`, `apksigner verify --print-certs` trên APK thật, bản `profile`, đăng ký app và bật Play App Signing, thử reset upload key, và lấy `versionCode` mới nhất từ Play. Ba việc cuối cần tài khoản Play Console của bạn.

## Bài tập

1. `upload-keystore.jks` bị commit vào repository riêng của đội hai tháng trước, rồi bị xoá ở commit sau. Đội nên làm gì, và xoá khỏi lịch sử có đủ không?
2. `pubspec.yaml` ghi `version: 2.0.0+12`, bản mới nhất trên Play có `versionCode` 40. `check_version_code` thoát mã mấy? Sửa thế nào, và khi đó APK arm64-v8a tách ABI có `versionCode` bao nhiêu?
3. CI chạy `jarsigner -verify app.aab`, thoát mã 0 và đội kết luận AAB đã ký. Sai ở đâu, kiểm đúng thế nào?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Coi upload key là đã lộ, dù repository riêng: tạo khoá mới, xuất PEM và xin reset trong Play Console, đổi mật khẩu và secrets trên CI. App signing key do Google giữ nên không phải đổi. Xoá khỏi lịch sử chỉ là dọn dẹp thêm, không làm khoá cũ an toàn lại.</p>
<p><strong>2.</strong> Mã 1, vì <code>12 &lt;= 40</code> là giảm. Sửa thành <code>2.0.0+41</code> hoặc lớn hơn. APK arm64-v8a tách ABI có <code>2 * 1000 + 41 = 2041</code>; AAB dùng thẳng 41.</p>
<p><strong>3.</strong> <code>jarsigner -verify</code> thoát mã 0 cả với bundle chưa ký (in <code>jar is unsigned.</code>). Phải đọc đầu ra tìm <code>jar verified.</code>, rồi đối chiếu vân tay từ <code>keytool -printcert -jarfile app.aab</code> với vân tay upload key đã đăng ký.</p>
</details>

Bài trước: [Flavors khi build](/articles/flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build). Bài tiếp: [Ký ứng dụng iOS](/articles/flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities).

## Tài liệu gốc

- [Build and release an Android app](https://docs.flutter.dev/deployment/android) (docs.flutter.dev).
- [Sign your app](https://developer.android.com/studio/publish/app-signing) và [Android App Bundle](https://developer.android.com/guide/app-bundle) (developer.android.com).
- [Build your app from the command line](https://developer.android.com/build/building-cmdline) và [apksigner](https://developer.android.com/tools/apksigner) (developer.android.com).
- [Use Play App Signing](https://support.google.com/googleplay/android-developer/answer/9842756) và [Create and set up your app](https://support.google.com/googleplay/android-developer/answer/9859152) (support.google.com/googleplay).
- [Target API level requirements for Google Play apps](https://support.google.com/googleplay/android-developer/answer/11926878) (support.google.com/googleplay).
- [keytool](https://docs.oracle.com/en/java/javase/21/docs/specs/man/keytool.html) và [jarsigner](https://docs.oracle.com/en/java/javase/21/docs/specs/man/jarsigner.html) (docs.oracle.com).

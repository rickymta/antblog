---
slug: flutter-bao-mat-mobile-secure-storage-pinning-obfuscation-masvs
title: "Bảo mật mobile: secure storage, pinning, obfuscation và OWASP MASVS"
summary: "Bài 42: mô hình đe doạ app khách và app tài xế, bí mật không nằm trong app, Redactor che log, ghim SPKI bằng dio kiểm trên HTTPS localhost (validateCertificate chạy sau khi gửi), --obfuscate không bảo vệ bí mật, root là tín hiệu, checklist MASVS v2. 86 test."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, security, certificate-pinning, owasp, masvs, obfuscation]
language: vi
---

Đội hỗ trợ của Tới Nơi gửi file log của một chuyến lỗi cho đối tác điều tra. Trong file có số điện thoại khách, toạ độ điểm đón tới sáu chữ số thập phân và một dòng `Authorization: Bearer ...` còn nguyên giá trị. Không ai tấn công: một dòng `print` thiếu thận trọng là đủ. Tình huống là minh hoạ.

Bảo mật mobile là một chuỗi quyết định nhỏ: dữ liệu nào nằm trên máy, log ghi gì, ai được nói chuyện với server. Bài này chỉ phòng thủ cho **chính app của ta** và đo mỗi khẳng định bằng test, rồi đối chiếu OWASP MASVS.

Đây là bài 42 của lộ trình **Flutter qua Tới Nơi**, bài kết Phần 5. Ta nên đọc [bài 41 về hiệu năng](/articles/flutter-hieu-nang-rebuild-isolate-devtools) trước; hai bài nền là [bài 22 về lưu trữ cục bộ](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline) và [bài 24 về authentication](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage). Bài dựa trên Flutter 3.47.4, Dart 3.13.3 và `dio` 5.11.1. Mọi địa chỉ, số điện thoại, toạ độ và chứng chỉ đều là đồ giả.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>App dùng <code>dio</code> với <code>validateCertificate</code> so SPKI hash của chứng chỉ server. Kẻ chen giữa trình một chứng chỉ do CA mà máy tin cấp, nhưng khoá khác khoá server. Request kèm <code>Authorization</code> có tới được máy của kẻ đó không?</p>
<details><summary>Xem đáp án</summary><p><strong>Có.</strong> Chuỗi chứng chỉ hợp lệ nên bắt tay thành công; source <code>IOHttpClientAdapter</code> của dio 5.11.1 chỉ gọi <code>validateCertificate</code> sau <code>request.close()</code>, tức là sau khi header và body đã gửi. Cuộc gọi vẫn ném lỗi, nhưng token đã tới server của kẻ đó (test <code>ghim sai làm request ném lỗi nhưng Authorization đã tới server</code>). Nếu CA không được máy tin thì bắt tay hỏng và token không đi. Mục 4 ghim ngay ở bước bắt tay.</p></details>
</div>

## 1. Mô hình đe doạ: ai muốn lấy gì

Ta viết ra ba thứ: **tài sản** cần giữ, **kẻ tấn công** chạm được vào nó, **biện pháp** của ta (Hình 1). Mũi tên từ cột trái chỉ ý đồ tấn công, mũi tên từ cột phải chỉ biện pháp che chắn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 1. Mô hình đe doạ. Bốn kẻ tấn công ở cột trái nhắm tới bốn tài sản ở cột giữa. Người nhặt được máy nhắm token, biện pháp là Keychain hoặc Keystore. Người nghe mạng nhắm phiên đăng nhập, biện pháp là HTTPS và ghim SPKI. Người đọc log nhắm số điện thoại và toạ độ, biện pháp là Redactor và tắt backup. Người sửa app nhắm vị trí gửi lên và logic cước, biện pháp là server kiểm lại.">
<defs><marker id="fl42-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl42-1-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<text x="16" y="24" class="t-b">Kẻ tấn công</text><text x="315" y="24" class="t-b">Tài sản</text><text x="614" y="24" class="t-b">Biện pháp</text>
<rect x="16" y="40" width="250" height="68" rx="10" class="f-warn-soft s-warn"/><text x="30" y="68" class="t-b">Người nhặt được máy</text><text x="30" y="92" class="t-small">máy đã mở khoá</text>
<rect x="315" y="40" width="250" height="68" rx="10" class="f-pk-soft s-pk"/><text x="329" y="68" class="t-b">Access, refresh token</text><text x="329" y="92" class="t-small">cả hai app</text>
<rect x="614" y="40" width="250" height="68" rx="10" class="f-phys-soft s-phys"/><text x="628" y="68" class="t-b">Keychain, Keystore</text><text x="628" y="92" class="t-small">token ngắn hạn, xoay vòng</text>
<rect x="16" y="130" width="250" height="68" rx="10" class="f-warn-soft s-warn"/><text x="30" y="158" class="t-b">Người nghe mạng</text><text x="30" y="182" class="t-small">Wi-Fi công cộng, proxy</text>
<rect x="315" y="130" width="250" height="68" rx="10" class="f-pk-soft s-pk"/><text x="329" y="158" class="t-b">Phiên đăng nhập</text><text x="329" y="182" class="t-small">cả hai app</text>
<rect x="614" y="130" width="250" height="68" rx="10" class="f-phys-soft s-phys"/><text x="628" y="158" class="t-b">HTTPS và ghim SPKI</text><text x="628" y="182" class="t-small">hai ghim, có rotation</text>
<rect x="16" y="220" width="250" height="68" rx="10" class="f-warn-soft s-warn"/><text x="30" y="248" class="t-b">Người đọc log, backup</text><text x="30" y="272" class="t-small">gồm cả nhân viên hỗ trợ</text>
<rect x="315" y="220" width="250" height="68" rx="10" class="f-pk-soft s-pk"/><text x="329" y="248" class="t-b">SĐT, địa chỉ, toạ độ</text><text x="329" y="272" class="t-small">app khách, app tài xế</text>
<rect x="614" y="220" width="250" height="68" rx="10" class="f-phys-soft s-phys"/><text x="628" y="248" class="t-b">Redactor, tắt backup</text><text x="628" y="272" class="t-small">không ghi body request</text>
<rect x="16" y="310" width="250" height="56" rx="10" class="f-warn-soft s-warn"/><text x="30" y="335" class="t-b">Người sửa app</text><text x="30" y="355" class="t-small">root, hook, giả GPS</text>
<rect x="315" y="310" width="250" height="56" rx="10" class="f-pk-soft s-pk"/><text x="329" y="335" class="t-b">Vị trí gửi lên, cước</text><text x="329" y="355" class="t-small">chủ yếu app tài xế</text>
<rect x="614" y="310" width="250" height="56" rx="10" class="f-phys-soft s-phys"/><text x="628" y="335" class="t-b">Server kiểm lại</text><text x="628" y="355" class="t-small">root chỉ là tín hiệu</text>
<path d="M268 74 L313 74" class="f-none s-warn" stroke-width="2" marker-end="url(#fl42-1-w)"/><path d="M268 164 L313 164" class="f-none s-warn" stroke-width="2" marker-end="url(#fl42-1-w)"/><path d="M268 254 L313 254" class="f-none s-warn" stroke-width="2" marker-end="url(#fl42-1-w)"/><path d="M268 338 L313 338" class="f-none s-warn" stroke-width="2" marker-end="url(#fl42-1-w)"/>
<path d="M612 74 L567 74" class="f-none s-phys" stroke-width="2" marker-end="url(#fl42-1-p)"/><path d="M612 164 L567 164" class="f-none s-phys" stroke-width="2" marker-end="url(#fl42-1-p)"/><path d="M612 254 L567 254" class="f-none s-phys" stroke-width="2" marker-end="url(#fl42-1-p)"/><path d="M612 338 L567 338" class="f-none s-phys" stroke-width="2" marker-end="url(#fl42-1-p)"/>
</svg>
</div>
<figcaption>Hình 1. Bốn hàng của mô hình đe doạ. Hàng cuối đặt biện pháp ở server vì người sửa app kiểm soát máy của họ.</figcaption>
</figure>

Mô hình nằm trong code để test khoá lại. Mỗi dòng ghi biện pháp chạy ở client hay server và thuộc control MASVS nào:

### threat_model.dart · 42-mobile-security/lib/src/threat_model.dart (trích)

```dart
  Threat(
    apps: {AppKind.driver},
    asset: 'Tài khoản nhận tiền của tài xế',
    attacker: Attacker.lostDevice,
    control: 'Đổi tài khoản nhận tiền đòi xác thực lại; server kiểm lần nữa',
    masvs: 'MASVS-AUTH-3',
    enforcedOn: EnforcedOn.server,
  ),
  Threat(
    apps: {AppKind.driver},
    asset: 'Dữ liệu vị trí gửi lên (chống giả GPS)',
    attacker: Attacker.tamperer,
    control: 'Tín hiệu root/hook gửi lên server để chấm rủi ro, không tự chặn',
    masvs: 'MASVS-RESILIENCE-1',
    enforcedOn: EnforcedOn.server,
  ),
```

Một test giữ quy tắc: với người sửa app, biện pháp phải thực thi ở server. Test khác bắt mọi ID MASVS trong mô hình phải có trong checklist ở mục 6.

## 2. Bí mật và nơi lưu

Quy tắc đầu tiên: **bí mật dài hạn không nằm trong app**. Bài 25 đã cho thấy `--dart-define` thành hằng trong binary. Khoá API của bên thứ ba ở server; app gọi BFF bằng token ngắn hạn như bài 24. Số thẻ không đi qua app (thanh toán qua cổng ngoài, bài 39).

Với thứ app buộc phải giữ, bài 22 đặt token sau `TokenStore` dùng `flutter_secure_storage` (Keychain trên iOS, khoá bọc trong Keystore trên Android). Lab viết quyết định đó thành chính sách có test, để ai thêm loại dữ liệu mới phải trả lời "nó nằm ở đâu":

### storage_policy.dart · 42-mobile-security/lib/src/storage_policy.dart (trích)

```dart
    DataKind.refreshToken: StorageRule(
      Store.secureStorage,
      'Sống lâu hơn access token, nên là thứ đáng lấy cắp nhất trên máy.',
    ),
    DataKind.thirdPartyApiKey: StorageRule(
      null,
      'Khoá của bên thứ ba nằm ở server; app đi qua BFF.',
    ),
  };

  static bool isAllowed(DataKind kind, Store store) {
    final least = rules[kind]!.leastSecureStore;
    return least != null && store.index <= least.index;
  }
```

`Store` xếp từ ít bị lộ nhất (RAM) tới nhiều nhất (`shared_preferences`): token vào RAM hoặc Keychain/Keystore thì được, vào database hay preferences thì không. Quy tắc `null` nghĩa là app không được giữ ở bất cứ đâu.

"Secure" không phải tuyệt đối: kho của hệ điều hành chống được app khác và người đọc file, không cứu được máy đã bị chiếm quyền. Trên Android còn đường rò là backup: `android:allowBackup` mặc định `true`, và với app nhắm Android 12 trở lên, `false` có thể không tắt chuyển máy trực tiếp trên một số máy (tài liệu Android). Ta cấu hình cả hai đường, loại cả hai tệp `SharedPreferences` mặc định của plugin 11.2.0, tệp dữ liệu `FlutterSecureStorage` và tệp khoá bọc `FlutterSecureKeyStorage` (đọc ở source plugin; README plugin cảnh báo lỗi `Failed to unwrap key` khi backup, ta chưa thử trên máy):

### data_extraction_rules.xml · 42-mobile-security/native-config/android/app/src/main/res/xml/data_extraction_rules.xml

```xml
<data-extraction-rules>
    <cloud-backup>
        <exclude domain="sharedpref" path="FlutterSecureStorage.xml" />
        <exclude domain="sharedpref" path="FlutterSecureKeyStorage.xml" />
    </cloud-backup>
    <device-transfer>
        <exclude domain="sharedpref" path="FlutterSecureStorage.xml" />
        <exclude domain="sharedpref" path="FlutterSecureKeyStorage.xml" />
    </device-transfer>
</data-extraction-rules>
```

Test đọc file này và mảnh `AndroidManifest` đi kèm để kiểm `allowBackup="false"` và các `exclude`. Đó là kiểm cấu hình, chưa phải sao lưu và khôi phục trên máy thật.

## 3. Log không lộ dữ liệu

`SafeLogInterceptor` của bài 19 không ghi body và chỉ cho vài header. Nhưng log do code app tự viết (`'đón khách tại $lat, $lng'`) không đi qua nó. `Redactor` thuần Dart là lớp thứ hai cho chuỗi tự do; mỗi quy tắc là một `RegExp` kèm cách thay:

### redactor.dart · 42-mobile-security/lib/src/redactor.dart (trích)

```dart
  // Số điện thoại Việt Nam: 0 hoặc +84/84 rồi 9 chữ số, cho phép khoảng
  // trắng, dấu chấm, gạch ngang giữa các chữ số.
  RedactionRule(
    'phone-vn',
    RegExp(r'(?<![A-Za-z0-9.])(?:\+?84|0)(?:[\s.-]?\d){9}(?!\d)'),
    (_) => '[phone]',
  ),
  // Cặp toạ độ `10.762622, 106.660172`: giữ 2 chữ số thập phân (~1,1 km). Cần
  // từ 4 chữ số thập phân để không nuốt nhầm cặp số tiền như `25.500, 12.000`.
  RedactionRule(
    'coordinate-pair',
    RegExp(r'(-?\d{1,3}\.\d{4,})\s*,\s*(-?\d{1,3}\.\d{4,})'),
    (m) => '${_coarse(m[1]!)},${_coarse(m[2]!)}',
  ),
```

Thứ tự quy tắc quan trọng: token dài đứng **trước** số điện thoại, nếu không dãy `0900000001` nằm giữa token bị thay riêng và để hai nửa token lộ ra (test `số điện thoại nằm trong token dài không để lộ nửa token`). Toạ độ làm tròn hai chữ số thập phân: đủ biết "quận nào", không biết "toà nhà nào".

### redactor_test.dart · 42-mobile-security/test/redactor_test.dart (trích)

```dart
    test('cặp toạ độ chính xác giữ 2 chữ số thập phân', () {
      expect(
        redactor.redact('đón tại 10.762622, 106.660172 lúc 8h'),
        'đón tại 10.76,106.66 lúc 8h',
      );
    });
```

Test cũng ghi lại điều bộ che **không** làm: cặp toạ độ chỉ ba chữ số thập phân (cỡ trăm mét) không bị che, vì ta đổi lấy việc không nuốt nhầm cặp số tiền. Không bộ che nào bắt hết: cách an toàn nhất vẫn là đừng đưa dữ liệu nhạy cảm vào câu log.

## 4. Pinning với dio: ghim ở bắt tay, hai ghim, và khi nào không ghim

Mặc định app tin mọi CA mà hệ điều hành tin. **Ghim** (pinning) chỉ tin đúng khoá của server Tới Nơi, để kẻ chen giữa có chứng chỉ "hợp lệ" từ CA khác vẫn bị từ chối. Với `dio`, adapter có hai chỗ để móc vào và chúng khác nhau về thời điểm.

`validateCertificate` của dio 5.11.1 chạy khi đã nhận response headers; `badCertificateCallback` của `HttpClient` chạy trong lúc bắt tay TLS (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Hai thời điểm kiểm ghim. Làn A, validateCertificate: bắt tay TLS tin kho CA, rồi gửi request kèm Authorization, rồi nhận response và kiểm ghim, ghim sai thì ném lỗi, nhưng server đã nhận token. Làn B, badCertificateCallback: bắt tay TLS không tin CA nào, kiểm ghim chứng chỉ lá, khớp thì gửi request, sai thì ngắt kết nối khi chưa gửi gì.">
<defs><marker id="fl42-2-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl42-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="16" y="26" class="t-b">A. validateCertificate: kiểm sau khi gửi</text>
<rect x="16" y="42" width="190" height="72" rx="10" class="f-surface s-line"/><text x="30" y="72" class="t-b">Bắt tay TLS</text><text x="30" y="96" class="t-small">tin kho CA</text>
<rect x="246" y="42" width="210" height="72" rx="10" class="f-warn-soft s-warn"/><text x="260" y="72" class="t-b">Gửi request</text><text x="260" y="96" class="t-small">kèm Authorization</text>
<rect x="496" y="42" width="170" height="72" rx="10" class="f-surface s-line"/><text x="510" y="72" class="t-b">Nhận response</text><text x="510" y="96" class="t-small">rồi mới kiểm ghim</text>
<rect x="706" y="42" width="158" height="72" rx="10" class="f-surface s-line"/><text x="720" y="72" class="t-b">Ghim sai</text><text x="720" y="96" class="t-small">ném lỗi</text>
<path d="M208 78 L244 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-2-p)"/><path d="M458 78 L494 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-2-p)"/><path d="M668 78 L704 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-2-p)"/>
<text x="246" y="138" class="t-small t-warn">server đã nhận token trước khi ta kiểm</text>
<text x="16" y="186" class="t-b">B. badCertificateCallback: kiểm ở bắt tay</text>
<rect x="16" y="204" width="190" height="72" rx="10" class="f-surface s-line"/><text x="30" y="234" class="t-b">Bắt tay TLS</text><text x="30" y="258" class="t-small">không tin CA nào</text>
<rect x="246" y="204" width="210" height="72" rx="10" class="f-pk-soft s-pk"/><text x="260" y="234" class="t-b">Kiểm ghim</text><text x="260" y="258" class="t-small">chứng chỉ lá và host</text>
<rect x="506" y="196" width="230" height="50" rx="10" class="f-surface s-line"/><text x="520" y="226" class="t-b">Khớp: gửi request</text>
<rect x="506" y="256" width="230" height="50" rx="10" class="f-warn-soft s-warn"/><text x="520" y="286" class="t-b">Sai: ngắt, chưa gửi gì</text>
<path d="M208 240 L244 240" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-2-p)"/><path d="M458 232 L504 222" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-2-p)"/><path d="M458 248 L504 280" class="f-none s-warn" stroke-width="2" marker-end="url(#fl42-2-w)"/>
</svg>
</div>
<figcaption>Hình 2. Hai điểm móc của dio, mũi tên theo thứ tự thời gian. Hộp cảnh báo ở làn A là lúc dữ liệu nhạy cảm đã rời máy.</figcaption>
</figure>

Adapter của lab chọn đường B. Client không tin CA nào (`withTrustedRoots: false`), nên mọi chứng chỉ đều rơi vào `badCertificateCallback`, và callback chỉ nhận chứng chỉ lá khớp ghim của đúng host. Ghim ở đây **thay thế** kiểm CA chứ không cộng thêm: không còn kiểm CA, SAN hay thu hồi, chỉ còn danh sách ghim (test cho chứng chỉ chỉ có SAN `other.example` vẫn qua khi gọi `localhost`):

### pinned_adapter.dart · 42-mobile-security/lib/src/pinned_adapter.dart (trích)

```dart
IOHttpClientAdapter createPinnedAdapter(PinPolicy policy) {
  return IOHttpClientAdapter(
    createHttpClient: () {
      final client = HttpClient(
        context: SecurityContext(withTrustedRoots: false),
      );
      client.badCertificateCallback = (certificate, host, port) =>
          _accepts(policy, certificate, host);
      return client;
    },
  );
}
```

Ta ghim **SPKI** (khối `SubjectPublicKeyInfo`), không ghim hash cả chứng chỉ: cấp lại chứng chỉ trên cùng khoá làm hash chứng chỉ đổi, SPKI giữ nguyên. `der.dart` đi qua cấu trúc X.509 để lấy khối này, và hash khớp lệnh `openssl x509 -pubkey -noout | openssl pkey -pubin -outform der | openssl dgst -sha256 -binary | base64` (test `khớp kết quả của openssl`). Chính sách còn kiểm hạn, vì khoá khớp không có nghĩa chứng chỉ còn dùng được:

### cert_pin.dart · 42-mobile-security/lib/src/cert_pin.dart (trích)

```dart
  bool accepts(
    String host,
    Uint8List der, {
    required DateTime startValidity,
    required DateTime endValidity,
  }) {
    final pins = _pinsByHost[host.toLowerCase()];
    if (pins == null) return false;
    final now = _now();
    if (now.isBefore(startValidity) || now.isAfter(endValidity)) return false;
    return pins.any((pin) => pin.matches(der));
  }
```

Test dựng server HTTPS thật trên localhost với **chứng chỉ test** sinh lúc chạy bằng `openssl` (khoá riêng không phải bí mật, chỉ sống một lần chạy). Server đếm request nhận được, nên ta chứng minh được cả hai thời điểm của Hình 2:

### pinned_adapter_test.dart · 42-mobile-security/test/pinned_adapter_test.dart (trích)

```dart
        expect(error.type, DioExceptionType.badCertificate);
        expect(server.requestCount, 1);
        expect(server.receivedAuthorization.single, 'Bearer test-only');
```

Đó là đường A: lỗi ném ra nhưng server nhận đủ một request kèm `Authorization`. Với đường B, test `khoá lạ` cho `requestCount` bằng 0, lỗi có loại `unknown` kèm `HandshakeException` (dio chỉ gắn `badCertificate` ở nhánh `validateCertificate`). Ghim sai và lỗi TLS khác trông giống nhau, nên đừng báo "bị tấn công".

Callback chỉ nhận chứng chỉ **lá**, kể cả khi server gửi kèm chứng chỉ trung gian (đã thử trên macOS; test chuỗi lá và trung gian), nên không ghim được trung gian. Ghim chỉ phủ một `Dio`: tile bản đồ, WebSocket của bài 36 và ảnh dùng client riêng. Flutter ghi rõ: với socket do Dart sở hữu, không chính sách nền tảng nào được áp. Ta suy ra cấu hình ghim của nền tảng cũng không áp cho `dio`, nhưng chưa thử.

**Rotation và khoá chết.** Ghim hỏng khi server đổi khoá mà app không biết. Quy trình ở Hình 3 dùng hai ghim: khoá đang dùng và một khoá dự phòng tạo sẵn, giữ ngoại tuyến. Server chuyển sang khoá dự phòng, app cũ vẫn chạy vì đã có ghim đó; bản app mới thêm khoá dự phòng kế tiếp. `PinPolicy` từ chối cấu hình dưới hai ghim mỗi host.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Ba pha rotation chạy được và một sự cố. Pha 1: server dùng khoá A, app ghim A và B, chạy. Pha 2: server đổi sang khoá B, app cũ vẫn ghim A và B, chạy. Pha 3: phát hành app mới ghim B và C, chạy. Sự cố: app chỉ ghim A trong khi server đã dùng khoá B, mọi request lỗi, app bị khoá chết.">
<defs><marker id="fl42-3-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="16" width="190" height="44" rx="10" class="f-pk-soft s-pk"/><text x="30" y="44" class="t-b">1. Hiện tại</text>
<rect x="236" y="16" width="190" height="44" rx="10" class="f-pk-soft s-pk"/><text x="250" y="44" class="t-b">2. Server đổi khoá</text>
<rect x="456" y="16" width="190" height="44" rx="10" class="f-pk-soft s-pk"/><text x="470" y="44" class="t-b">3. Phát hành app mới</text>
<rect x="676" y="16" width="188" height="44" rx="10" class="f-warn-soft s-warn"/><text x="690" y="44" class="t-b">Sự cố</text>
<path d="M208 38 L234 38" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-3-p)"/><path d="M428 38 L454 38" class="f-none s-pk" stroke-width="2" marker-end="url(#fl42-3-p)"/>
<rect x="16" y="76" width="190" height="56" rx="10" class="f-surface s-line"/><text x="30" y="100" class="t-b">Server</text><text x="30" y="120" class="t-small">dùng khoá A</text>
<rect x="236" y="76" width="190" height="56" rx="10" class="f-surface s-line"/><text x="250" y="100" class="t-b">Server</text><text x="250" y="120" class="t-small">dùng khoá B</text>
<rect x="456" y="76" width="190" height="56" rx="10" class="f-surface s-line"/><text x="470" y="100" class="t-b">Server</text><text x="470" y="120" class="t-small">dùng khoá B</text>
<rect x="676" y="76" width="188" height="56" rx="10" class="f-surface s-line"/><text x="690" y="100" class="t-b">Server</text><text x="690" y="120" class="t-small">dùng khoá B</text>
<rect x="16" y="144" width="190" height="56" rx="10" class="f-surface s-line"/><text x="30" y="168" class="t-b">App ghim</text><text x="30" y="188" class="t-mono">A và B</text>
<rect x="236" y="144" width="190" height="56" rx="10" class="f-surface s-line"/><text x="250" y="168" class="t-b">App cũ ghim</text><text x="250" y="188" class="t-mono">A và B</text>
<rect x="456" y="144" width="190" height="56" rx="10" class="f-surface s-line"/><text x="470" y="168" class="t-b">App mới ghim</text><text x="470" y="188" class="t-mono">B và C</text>
<rect x="676" y="144" width="188" height="56" rx="10" class="f-surface s-line"/><text x="690" y="168" class="t-b">App chỉ ghim</text><text x="690" y="188" class="t-mono">A</text>
<rect x="16" y="212" width="190" height="68" rx="10" class="f-phys-soft s-phys"/><text x="30" y="240" class="t-b">Chạy</text><text x="30" y="262" class="t-small">khoá A khớp</text>
<rect x="236" y="212" width="190" height="68" rx="10" class="f-phys-soft s-phys"/><text x="250" y="240" class="t-b">Chạy</text><text x="250" y="262" class="t-small">B là ghim dự phòng</text>
<rect x="456" y="212" width="190" height="68" rx="10" class="f-phys-soft s-phys"/><text x="470" y="240" class="t-b">Chạy</text><text x="470" y="262" class="t-small">B khớp, C dự phòng</text>
<rect x="676" y="212" width="188" height="68" rx="10" class="f-warn-soft s-warn"/><text x="690" y="240" class="t-b">Khoá chết</text><text x="690" y="262" class="t-small">mọi request lỗi</text>
</svg>
</div>
<figcaption>Hình 3. Ba pha đầu theo thời gian, mũi tên chỉ pha kế tiếp. Cột "Sự cố" không nằm trên đường đó.</figcaption>
</figure>

Test `khoá chết` dựng đúng cột phải: kết quả là `HandshakeException` và server nhận 0 request. Lối thoát duy nhất là cập nhật app, nên cơ chế ép cập nhật (bài 59) đi kèm ghim.

**Khi nào không nên ghim.** Khi chưa có quy trình giữ khoá dự phòng và phát hành app kịp; khi ta không kiểm soát khoá server (CDN bên thứ ba tự đổi chứng chỉ); MASVS-NETWORK-2 chỉ yêu cầu ghim cho endpoint **do ta quản lý**; còn lại HTTPS đúng cách (NETWORK-1) đã là mức nền tốt.

## 5. Obfuscation và root/jailbreak: làm khó, không phải khoá

`--obfuscate` với `--split-debug-info` đổi tên lớp và hàm trong code Dart. Tài liệu Flutter ghi nó **không** mã hoá tài nguyên, không chống được reverse engineering, và lưu bí mật trong app là thực hành kém. Máy kiểm không có Android SDK nên ta không chạy `flutter build`; lab đo bằng `dart compile aot-snapshot` với `--obfuscate --strip`, cùng bộ biên dịch AOT (`flutter_tools` mặc định cũng strip snapshot: iOS strip sau khi xuất dSYM, Android để AGP lo, nền tảng khác thêm `--strip`):

Lệnh thật cho app (chưa chạy ở đây):

```bash
flutter build apk --obfuscate --split-debug-info=build/symbols \
  --extra-gen-snapshot-options=--save-obfuscation-map=build/obfuscation-map.json
```

Lab đo bằng:

```bash
dart compile aot-snapshot \
  --extra-gen-snapshot-options=--obfuscate,--strip,--save-obfuscation-map=map.json \
  tool/obfuscation_target.dart -o target.aot
```

Chương trình đo có class `QuoteCalculator`, một enum và một chuỗi trông như khoá API. Kết quả:

| Điều đo | Bản thường | `--obfuscate --strip` |
|---|---|---|
| `runtimeType` và tên lớp, hàm trong stack trace | tên thật | tên ngắn vô nghĩa |
| tên `QuoteCalculator`, `fareFor` trong file | có | - |
| chuỗi hằng "khoá" | nằm nguyên trong file | **vẫn nằm nguyên** |
| tên enum | giữ | **vẫn giữ** (tài liệu Flutter cũng ghi) |

Hàng ba quan trọng nhất: ai có file app chạy `strings` là thấy chuỗi. Obfuscation làm người đọc chậm hơn, không biến chuỗi thành bí mật.

Cái giá: stack trace bản phát hành thành tên vô nghĩa, nên phải **giữ** thư mục `--split-debug-info` và file bản đồ của từng bản phát hành. Tài liệu Flutter dùng `flutter symbolize`; ta chưa chạy được lệnh đó (README giải thích), nên lab tự kiểm phần bản đồ:

### obfuscation_test.dart · 42-mobile-security/test/obfuscation_test.dart (trích)

```dart
    final restored = map.deobfuscateStackTrace(trace);

    expect(restored, contains('QuoteCalculator.fareFor'));
    expect(restored, contains('tool/obfuscation_target.dart:'));
```

`deobfuscateStackTrace` chỉ dịch tên hàm và tên file của từng frame, để thông báo lỗi không bị thay nhầm. Bản đồ chỉ đúng với đúng bản build sinh ra nó.

**Root và jailbreak** là tín hiệu, không phải khoá: có công cụ che tín hiệu, và máy người dùng thường đôi khi bị báo nhầm. Lab không có bộ phát hiện; nó mô hình hoá phản ứng của app:

### integrity_policy.dart · 42-mobile-security/lib/src/integrity_policy.dart (trích)

```dart
Decision decide(AppAction action, Set<IntegritySignal> signals) {
  if (action.sensitive) return Decision.requireReauthentication;
  return signals.isEmpty ? Decision.allow : Decision.allowAndReport;
}
```

Việc nhạy cảm luôn đòi xác thực lại, vì không có tín hiệu không chứng minh máy sạch; việc thường chỉ báo lên server để chấm rủi ro, không chặn. Test chạy qua mọi tập con của ba tín hiệu: thêm tín hiệu không bao giờ làm quyết định dễ dãi hơn. Kiểm tra toàn vẹn thật cần Play Integrity hoặc App Attest; lab chưa làm.

## 6. Đối chiếu OWASP MASVS và khép Phần 5

OWASP MASVS v2 gom kiểm soát bảo mật mobile thành tám nhóm, 24 control (đối chiếu mas.owasp.org tại 11/2026). Checklist của Tới Nơi nằm trong code: mỗi dòng có ID control, mức đã làm, bài liên quan và ghi chú còn thiếu gì:

### masvs_checklist.dart · 42-mobile-security/lib/src/masvs_checklist.dart (trích)

```dart
  MasvsItem(
    MasvsGroup.network,
    2,
    'Ghim danh tính cho endpoint do ta quản lý',
    Coverage.partial,
    [42],
    'Ghim SPKI với hai ghim, thử trên localhost. Chưa có hash thật, chưa rotation thật.',
  ),
```

Bảng tóm theo nhóm (số trong ngoặc là bài, theo khung loạt bài; có bài chưa viết). Kết quả: **3 control đã làm, 12 một phần, 9 chưa làm**, test khoá các con số đó.

| Nhóm MASVS | Đã làm | Một phần | Chưa làm |
|---|---|---|---|
| STORAGE (Storage) | - | 1 (22, 24, 42), 2 | - |
| CRYPTO (Cryptography) | - | 1, 2 | - |
| AUTH (Authentication and Authorization) | 1 (24) | 3 | 2 |
| NETWORK (Network Communication) | 1 (19, 20, 25) | 2 | - |
| PLATFORM (Platform Interaction) | - | 1 (23) | 2, 3 |
| CODE (Code Quality) | 4 (11, 19) | 3 (63) | 1, 2 |
| RESILIENCE (Resilience Against Reverse Engineering and Tampering) | - | 1, 3 | 2, 4 |
| PRIVACY (Privacy) | - | 1 (35, 38), 2 | 3, 4 |

**Khép Phần 5.** Mục tiêu của phần là các tính năng đặc thù mobile của Tới Nơi (bản đồ, vị trí, realtime, push, nền, thanh toán, native) với cấu hình iOS và Android đúng chính sách nền tảng. Đối chiếu: logic từng bài có test tự động; phần cần thiết bị (quyền vị trí, push, nền, Keychain, Keystore) chưa chạy trên máy thật và được ghi ở mục "Chưa làm" của từng lab. Bài này thêm lớp nằm ngang: token và vị trí đi qua mọi tính năng nên cùng chịu mô hình đe doạ.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp</span>
<ul>
<li><strong><code>HttpClient()</code> mặc định cộng <code>badCertificateCallback</code> để ghim.</strong> Dấu hiệu: chứng chỉ sai do CA công khai cấp vẫn chạy. Nguyên nhân: callback chỉ chạy khi chứng chỉ <em>không</em> qua được kiểm CA; với chứng chỉ CA tin, nó không được gọi dù trả <code>false</code> (test <code>HttpClient thường cộng callback</code>, dùng CA test, không ra mạng ngoài). Sửa: <code>SecurityContext(withTrustedRoots: false)</code>.</li>
<li><strong>Ghim bằng <code>validateCertificate</code> rồi tin là an toàn.</strong> Dấu hiệu: ghim sai ném lỗi nhưng server của kẻ tấn công vẫn thấy request. Nguyên nhân: dio gọi nó sau khi đã gửi. Sửa: ghim ở <code>badCertificateCallback</code> như <code>createPinnedAdapter</code>.</li>
<li><strong>Chỉ có một ghim.</strong> Dấu hiệu: ngày server đổi chứng chỉ, người dùng chưa cập nhật đều lỗi mạng. Nguyên nhân: không có khoá dự phòng. Sửa: hai ghim SPKI và cơ chế ép cập nhật.</li>
<li><strong>Để khoá trong <code>--dart-define</code> rồi bật <code>--obfuscate</code>.</strong> Dấu hiệu: <code>strings</code> trên file app thấy chuỗi khoá. Sửa: khoá dài hạn ở server, app giữ token ngắn hạn.</li>
<li><strong>Không giữ symbol và bản đồ của từng bản phát hành.</strong> Dấu hiệu: báo cáo crash toàn tên vô nghĩa. Sửa: lưu thư mục <code>--split-debug-info</code> và file bản đồ cùng artifact CI theo số build.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Cần `openssl` trên PATH để test sinh chứng chỉ. Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/42-mobile-security
dart pub get
dart analyze
dart test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `dio` 5.11.1, `crypto` 3.0.8: `dart analyze` báo `No issues found!`, `dart test` chạy **86 test** xanh. Mức: **Kiểm chứng tự động** cho toàn bộ logic; bài này thuộc nhóm cần thiết bị (cột KC của khung loạt bài), và phần đó **chưa làm**.

`redactor_test.dart` (31 test) chứng minh từng dạng dữ liệu bị che; hai file ghim (24 test) chứng minh SPKI, hai thời điểm kiểm ghim, rotation, khoá chết và chuỗi trung gian; `obfuscation_test.dart` (8 test) chứng minh bảng ở mục 5; `policy_test.dart` (23 test) khoá các chính sách và cấu hình Android.

**Chưa làm**: simulator và máy thật (Keychain, Keystore, backup Android); bộ phát hiện root/jailbreak, Play Integrity, App Attest; hash ghim thật và chuỗi trung gian trên Linux; `flutter build --obfuscate --split-debug-info` và `flutter symbolize`; sinh trắc học; che màn hình ở app switcher.

## Bài tập

1. Ta đổi `createPinnedAdapter` thành `HttpClient()` mặc định cộng `validateCertificate`. Kẻ chen giữa có chứng chỉ do CA hợp lệ cấp, khoá khác khoá của ta. `POST /trips` kèm token có lỗi không, và token có lộ không?
2. Server gia hạn chứng chỉ trên cùng một khoá. App A ghim hash chứng chỉ, app B ghim SPKI. App nào còn gọi được server?
3. Có người nói "đã bật `--obfuscate` nên để khoá API trong `--dart-define` được". Phản bác, và nói khoá nên nằm ở đâu.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Có lỗi nhưng token vẫn lộ. Chuỗi chứng chỉ hợp lệ nên bắt tay thành công, request kèm <code>Authorization</code> đã được gửi, rồi <code>validateCertificate</code> mới thấy ghim sai và ném <code>badCertificate</code> (test <code>ghim sai làm request ném lỗi nhưng Authorization đã tới server</code>).</p>
<p><strong>2.</strong> App B. Gia hạn trên cùng khoá giữ SPKI nhưng đổi hash chứng chỉ, nên app A bị từ chối (test <code>ghim hash chứng chỉ hỏng khi gia hạn, ghim SPKI thì không</code>).</p>
<p><strong>3.</strong> Obfuscation chỉ đổi tên; chuỗi hằng vẫn nằm trong file (bảng ở mục 5). Khoá dài hạn ở server (BFF), app chỉ giữ token ngắn hạn.</p>
</details>

Bài trước: [Hiệu năng: rebuild, const, RepaintBoundary, isolate và DevTools](/articles/flutter-hieu-nang-rebuild-isolate-devtools). Bài tiếp theo mở đầu Phần 6 với monorepo.

## Tài liệu gốc

- [OWASP MASVS](https://mas.owasp.org/MASVS/) và các trang control như [MASVS-NETWORK-2](https://mas.owasp.org/MASVS/controls/MASVS-NETWORK-2/) và [MASVS-RESILIENCE-1](https://mas.owasp.org/MASVS/controls/MASVS-RESILIENCE-1/) (mas.owasp.org).
- [Obfuscating Dart code](https://docs.flutter.dev/deployment/obfuscate) và [Network policy on iOS and Android](https://docs.flutter.dev/release/breaking-changes/network-policy-ios-android) (docs.flutter.dev).
- [HttpClient.badCertificateCallback](https://api.dart.dev/stable/dart-io/HttpClient/badCertificateCallback.html) và [SecurityContext](https://api.dart.dev/stable/dart-io/SecurityContext/SecurityContext.html) (api.dart.dev).
- [dio](https://pub.dev/packages/dio), [crypto](https://pub.dev/packages/crypto) và [flutter_secure_storage](https://pub.dev/packages/flutter_secure_storage) (pub.dev).
- [Auto Backup for Apps](https://developer.android.com/identity/data/autobackup), [Android Keystore system](https://developer.android.com/privacy-and-security/keystore) và [Play Integrity API](https://developer.android.com/google/play/integrity) (developer.android.com).
- [Keychain services](https://developer.apple.com/documentation/security/keychain-services) và [Establishing your app's integrity](https://developer.apple.com/documentation/devicecheck/establishing-your-app-s-integrity) (developer.apple.com).

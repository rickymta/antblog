---
slug: flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod
title: "Cấu hình môi trường: flavors, --dart-define-from-file và DI theo môi trường"
summary: "Bài 25: tách cấu hình dev, staging, prod bằng --dart-define-from-file. String.fromEnvironment là hằng lúc biên dịch nên giá trị nằm trong binary. AppConfig kiểm tra khi khởi động, đối chiếu appFlavor, và appConfigProvider được ghi đè trong main. 86 test."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, flavors, dart-define, riverpod, configuration]
language: vi
---

Đội QA nhận bản build nội bộ đầu tiên của app khách Tới Nơi và bấm **Báo giá**. Màn hình hiện 25.500 VND, cho tới khi một người hỏi app đang gọi server nào. Câu trả lời là `localhost:8787`: địa chỉ API nằm cứng trong code từ bài 19, và máy QA có sẵn một server giả ở cổng đó nên không ai thấy lỗi. Tình huống này là minh hoạ.

Tới Nơi có ba môi trường cố định: `dev` cho lập trình viên, `staging` cho QA, `prod` cho người dùng. Chúng khác nhau địa chỉ API, mức log, thời gian chờ và cả ID ứng dụng. Bài này lo lớp Dart: file cấu hình, cách đưa vào app, cách kiểm tra khi khởi động và cách dùng qua Riverpod. Phần native (Gradle, Xcode) chỉ giới thiệu sơ lược, vì bài 50 dạy chi tiết.

Đây là bài 25 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 24 về authentication](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage) trước: ở đó app chỉ giữ token ngắn hạn do server cấp, còn bài này cho thấy vì sao khoá bí mật dài hạn không thể nằm trong app. Lab dùng client của [bài 19](/articles/flutter-http-dio-json-dto-goi-api-bao-gia) và Riverpod của [bài 18](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen). Bài dựa trên Flutter 3.47.4, Dart 3.13.3 và `flutter_riverpod` 3.5.0; các địa chỉ `*.toinoi.example` là hư cấu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta chạy <code>flutter test --dart-define=FLAVOR=staging</code>. Trong test có dòng <code>final flavor = String.fromEnvironment('FLAVOR');</code> (không có <code>const</code>). Biến <code>flavor</code> bằng gì?</p>
<details><summary>Xem đáp án</summary><p>Trong <code>flutter test</code> là chuỗi <strong>rỗng</strong>, dù define có mặt (test <code>lời gọi không const: trong flutter test cho chuỗi rỗng dù define có mặt</code> chứng minh, Flutter 3.47.4). Tài liệu API chỉ đảm bảo <code>String.fromEnvironment</code> hoạt động khi gọi bằng <code>const</code>; không có <code>const</code> thì kết quả tuỳ nền tảng. Không có lỗi hay cảnh báo, nên app có thể chạy với địa chỉ rỗng. Mục 2 giải thích và mục 4 cho cách chặn.</p></details>
</div>

## 1. Ba môi trường, hai đường cấu hình

Bảng dưới là các môi trường theo hồ sơ loạt bài (địa chỉ là minh hoạ), với ID của app khách.

| Flavor | ID ứng dụng (app khách) | Backend | Log HTTP | Kênh phân phối (Phần 7) |
|---|---|---|---|---|
| `dev` | `vn.toinoi.customer.dev` | fake API hoặc trip-service chạy máy dev | bật | `flutter run`, cắm dây |
| `staging` | `vn.toinoi.customer.staging` | API staging | bật | Firebase App Distribution, TestFlight internal, Google Play internal |
| `prod` | `vn.toinoi.customer` | API production | tắt | App Store, Google Play production |

Hai nhóm thông tin đi theo hai đường. Địa chỉ API, log và timeout là thứ **code Dart đọc**: ta đưa vào bằng `--dart-define-from-file`. ID ứng dụng, tên, icon và file Firebase là thứ **bộ công cụ native đọc**: chúng thuộc flavor của Gradle (`productFlavors`, mỗi flavor có `applicationIdSuffix`) và của Xcode (scheme, build configuration, bundle identifier), chọn bằng cờ `--flavor`. ID khác nhau nên ba bản cài cạnh nhau được. Bài 50 làm phần này; lab của bài không có thư mục `android/` hay `ios/`.

Hình 1 cho thấy hai đường gặp nhau ở một lệnh build. Vì hai đường độc lập nên chúng có thể lệch nhau: chạy `--flavor prod` nhưng quên đổi file, bản mang ID thật sẽ trỏ vào backend dev. Mục 4 bắt lỗi này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Hai đường cấu hình vào cùng một bản build. File config/staging.json đi vào bằng --dart-define-from-file và là thứ code Dart đọc. Cờ --flavor staging đi vào Gradle và Xcode và quyết định ID, tên, icon. Cả hai vào lệnh flutter build hoặc run, ra bản staging có ID vn.toinoi.customer.staging. Khi khởi động AppConfig so FLAVOR trong file với appFlavor và dừng nếu lệch.">
<defs><marker id="fl25-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl25-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="16" y="30" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="30" y="60" class="t-mono">config/staging.json</text><text x="30" y="84" class="t-small">Code Dart đọc: địa chỉ API,</text><text x="30" y="104" class="t-small">log HTTP, timeout</text>
<rect x="16" y="190" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="30" y="220" class="t-mono">--flavor staging</text><text x="30" y="244" class="t-small">Native đọc: ID ứng dụng,</text><text x="30" y="264" class="t-small">tên, icon, file Firebase</text>
<rect x="340" y="100" width="220" height="110" rx="10" class="f-surface s-line"/><text x="354" y="132" class="t-b">flutter run / build</text><text x="354" y="156" class="t-small">một lệnh, hai đầu vào:</text><text x="354" y="176" class="t-mono">--dart-define-from-file</text><text x="354" y="196" class="t-mono">--flavor</text>
<rect x="624" y="30" width="240" height="100" rx="10" class="f-pk-soft s-pk"/><text x="638" y="60" class="t-b">Bản staging</text><text x="638" y="84" class="t-mono">vn.toinoi.customer.staging</text><text x="638" y="106" class="t-small">AppConfig + appFlavor</text>
<rect x="624" y="190" width="240" height="120" rx="10" class="f-warn-soft s-warn"/><text x="638" y="220" class="t-b">Lúc khởi động</text><text x="638" y="244" class="t-small">AppConfig so FLAVOR (file)</text><text x="638" y="264" class="t-small">với appFlavor (--flavor),</text><text x="638" y="284" class="t-small">lệch thì dừng, báo lỗi</text>
<path d="M278 80 L338 140" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-1-a)"/>
<path d="M278 240 L338 180" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-1-a)"/>
<path d="M562 140 L622 90" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-1-a)"/>
<path d="M744 132 L744 188" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl25-1-w)"/>
</svg>
</div>
<figcaption>Hình 1. Hai đầu vào của một bản build. Mũi tên chỉ chiều dữ liệu đi: từ file và cờ vào lệnh build, từ lệnh build ra bản cài. Phép so khớp ở khung cuối chạy mỗi lần app khởi động.</figcaption>
</figure>

## 2. --dart-define-from-file: JSON thành hằng lúc biên dịch

Mỗi môi trường có một file JSON phẳng với đúng năm khoá. Đây là hai file đầu và cuối của dải:

### dev.json · 25-flavors-config/config/dev.json

```json
{
  "FLAVOR": "dev",
  "API_BASE_URL": "http://localhost:8787",
  "LOG_HTTP": true,
  "CONNECT_TIMEOUT_MS": 15000,
  "RECEIVE_TIMEOUT_MS": 30000
}
```

### prod.json · 25-flavors-config/config/prod.json

```json
{
  "FLAVOR": "prod",
  "API_BASE_URL": "https://api.toinoi.example",
  "LOG_HTTP": false,
  "CONNECT_TIMEOUT_MS": 10000,
  "RECEIVE_TIMEOUT_MS": 15000
}
```

`staging.json` cùng khuôn với `prod.json`, khác địa chỉ và bật `LOG_HTTP`. Một cờ đưa file vào, dùng được với `flutter run`, `flutter build` lẫn `flutter test`:

```bash
flutter run --dart-define-from-file=config/staging.json
flutter test --dart-define-from-file=config/staging.json
```

Công cụ Flutter biến mỗi cặp trong JSON thành define `KEY=value` bằng nội suy chuỗi (hàm `extractDartDefines` trong `flutter_tools`). Hệ quả, đã chạy thử:

- Mọi giá trị đều trở thành chuỗi: `15000` thành `"15000"`, `true` thành `"true"`.
- `null` trong JSON trở thành chuỗi `"null"`, không phải "không có giá trị". Một đối tượng lồng nhau trở thành `"{x: 1}"`.
- `bool.fromEnvironment` chỉ nhận đúng chữ `true`: `True` cho ra `false`.
- `int.fromEnvironment('X', defaultValue: -1)` với `X=abc` trả `-1` mà không báo gì.
- `--dart-define` luôn thắng file, đứng trước hay sau trong dòng lệnh cũng vậy, vì `extractDartDefines` thêm chúng sau các khoá của file. Giữa nhiều file, file sau thắng. Mục 6 dùng điều này.

Phía Dart, mỗi khoá đọc bằng `String.fromEnvironment`. Hàm này **phải** được gọi bằng `const` và nhận tên khoá là hằng. Lab gom cả năm lời gọi vào một bảng hằng:

### app_config.dart · 25-flavors-config/lib/config/app_config.dart (trích)

```dart
/// Mỗi lời gọi `String.fromEnvironment` phải là `const` và nhận tên khoá là
/// hằng: chỉ `const` mới được tài liệu đảm bảo. Không `const` thì kết quả
/// tuỳ nền tảng (trong `flutter test` và `dart compile exe` ta đo được chuỗi
/// rỗng, âm thầm). Tên khoá lấy từ biến thì không biên dịch được.
static const Map<String, String> _compileTimeEnvironment = {
  keyFlavor: String.fromEnvironment(keyFlavor),
  keyApiBaseUrl: String.fromEnvironment(keyApiBaseUrl),
  keyLogHttp: String.fromEnvironment(keyLogHttp),
  keyConnectTimeoutMs: String.fromEnvironment(keyConnectTimeoutMs),
  keyReceiveTimeoutMs: String.fromEnvironment(keyReceiveTimeoutMs),
};
```

Hai kiểu sai đều đã chạy thử. Gọi không `const` thì kết quả tuỳ nền tảng: `flutter test` và `dart compile exe` cho chuỗi rỗng, `dart -DX=hello` (chạy JIT) cho `hello`, còn `dart compile js` ném `UnsupportedError` khi chạy. Ta chưa thử trên thiết bị; chỉ `const` được đảm bảo. Truyền tên khoá lấy từ biến vào `const String.fromEnvironment(name)` thì không biên dịch được (`Not a constant expression`). Test sau, thuộc nhóm "có define", chứng minh kiểu thứ nhất trong `flutter test`:

### environment_test.dart · 25-flavors-config/test/environment_test.dart (trích)

```dart
    // Cùng tên khoá, cùng lần chạy, chỉ khác có `const` hay không. Tài liệu
    // chỉ đảm bảo bản `const`; nền tảng khác có thể cho kết quả khác.
    final nonConst = String.fromEnvironment('FLAVOR');
    expect(nonConst, isEmpty);
    expect(const String.fromEnvironment('FLAVOR'), _flavorDefine);
  },
);
```

Khoá không có define cũng cho chuỗi rỗng, nên `String.fromEnvironment` không phân biệt "thiếu" với "rỗng" (`bool.hasEnvironment` thì có). Lab coi cả hai là thiếu.

## 3. Giá trị nằm trong binary: không phải chỗ cho bí mật

Vì lời gọi là `const`, trình biên dịch tính giá trị đó ngay lúc biên dịch và thay bằng chuỗi cố định. Chuỗi đó nằm trong file app, không còn gắn với file JSON hay dòng lệnh nào (Hình 2). Lab có chương trình ba dòng để tự thấy:

### embed_demo.dart · 25-flavors-config/tool/embed_demo.dart

```dart
// Chứng minh giá trị define nằm trong binary (bài viết, mục 3).
//
//   dart compile exe -DDEMO_VALUE=gia-tri-thu tool/embed_demo.dart -o /tmp/embed_demo
//   env -i /tmp/embed_demo                       # chạy KHÔNG có biến môi trường nào
//   grep -a -c gia-tri-thu /tmp/embed_demo       # chuỗi nằm ngay trong file binary
const _value = String.fromEnvironment('DEMO_VALUE');

// ignore: avoid_print
void main() => print('value=$_value');
```

```bash
dart compile exe -DDEMO_VALUE=gia-tri-thu tool/embed_demo.dart -o /tmp/embed_demo
env -i /tmp/embed_demo                  # in value=gia-tri-thu dù không có biến môi trường nào
grep -a -c gia-tri-thu /tmp/embed_demo  # 1: chuỗi nằm trong chính file binary
```

Đã chạy ba lệnh trên: binary in được giá trị khi môi trường rỗng, và `grep` thấy chuỗi trong file. Với Flutter, `flutter build bundle --dart-define=...` cũng để chuỗi trong `kernel_blob.bin` (đã kiểm, bản debug). Ta chưa dựng `libapp.so` của APK release hay IPA vì máy kiểm thiếu Android SDK và Xcode đầy đủ; ta suy ra cùng kết quả nhưng chưa tự kiểm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Đường đi của một giá trị define. File JSON qua công cụ Flutter thành KEY=value, compiler Dart thay lời gọi const bằng chuỗi cố định, chuỗi nằm trong binary của app. Ai có file app đều đọc được chuỗi. Bí mật thật nằm ở server: server giữ khoá của bên thứ ba, app chỉ nhận token ngắn hạn sau khi đăng nhập.">
<defs><marker id="fl25-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl25-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="16" y="30" width="180" height="100" rx="10" class="f-pk-soft s-pk"/><text x="30" y="60" class="t-mono">config/prod.json</text><text x="30" y="84" class="t-small">API_BASE_URL,</text><text x="30" y="104" class="t-small">LOG_HTTP, ...</text>
<rect x="236" y="30" width="180" height="100" rx="10" class="f-surface s-line"/><text x="250" y="60" class="t-b">Công cụ Flutter</text><text x="250" y="84" class="t-small">gửi KEY=value</text><text x="250" y="104" class="t-small">cho compiler</text>
<rect x="456" y="30" width="190" height="100" rx="10" class="f-surface s-line"/><text x="470" y="60" class="t-b">Compiler Dart</text><text x="470" y="84" class="t-small">const: thay lời gọi</text><text x="470" y="104" class="t-small">bằng chuỗi cố định</text>
<rect x="686" y="30" width="178" height="100" rx="10" class="f-pk-soft s-pk"/><text x="700" y="60" class="t-b">Binary của app</text><text x="700" y="84" class="t-small">chuỗi nằm sẵn</text><text x="700" y="104" class="t-small">trong file</text>
<path d="M198 80 L234 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-2-a)"/>
<path d="M418 80 L454 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-2-a)"/>
<path d="M648 80 L684 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-2-a)"/>
<rect x="686" y="200" width="178" height="90" rx="10" class="f-warn-soft s-warn"/><text x="700" y="232" class="t-b">Ai cầm file app</text><text x="700" y="256" class="t-small">đều đọc được</text><text x="700" y="276" class="t-small">chuỗi này</text>
<path d="M775 132 L775 198" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl25-2-w)"/>
<rect x="16" y="200" width="420" height="90" rx="10" class="f-phys-soft s-phys"/><text x="30" y="230" class="t-b">Chỗ đúng cho bí mật: server</text><text x="30" y="254" class="t-small">Server (BFF) giữ khoá của bên thứ ba;</text><text x="30" y="274" class="t-small">app chỉ nhận token ngắn hạn sau đăng nhập</text>
</svg>
</div>
<figcaption>Hình 2. Một giá trị define đi từ file JSON vào binary. Mũi tên chỉ chiều của quá trình build; mũi tên đứt chỉ điều xảy ra sau khi app được phát hành. Khung dưới trái không nằm trên đường đó: bí mật thật không bao giờ vào file cấu hình.</figcaption>
</figure>

Quy tắc suy ra: file JSON và `--dart-define` chỉ chứa thứ ai cũng được phép biết (địa chỉ công khai, cờ, thời gian chờ). Khoá API của bên thứ ba, `client_secret` và mật khẩu phải nằm ở server; app chỉ gọi BFF bằng token ngắn hạn như bài 24. Obfuscation không biến chuỗi thành bí mật (bài 42). Keystore và chứng chỉ ký nằm trong kho bí mật của CI (bài 51 đến 54).

File JSON nằm trong Git, nên bí mật lỡ commit còn nằm lại trong lịch sử. Lab thêm một lưới an toàn: bộ quét tìm khoá có tên gợi ý bí mật và giá trị có dạng khoá phổ biến, chạy trên cả ba file.

### secret_scan.dart · 25-flavors-config/test/support/secret_scan.dart (trích)

```dart
final _valueRules = [
  _Rule('khoá PEM', RegExp('-----BEGIN [A-Z ]+-----')),
  _Rule('access key AWS', RegExp(r'\bAKIA[0-9A-Z]{16}\b')),
  _Rule('API key Google', RegExp(r'\bAIza[0-9A-Za-z_-]{35}\b')),
  _Rule('JWT', RegExp(r'^eyJ[\w-]+\.[\w-]+\.[\w-]*$')),
  _Rule('token GitHub', RegExp(r'\bgh[pousr]_[0-9A-Za-z]{20,}')),
  _Rule('khoá Stripe', RegExp(r'\b[sr]k_(live|test)_[0-9A-Za-z]{10,}')),
  _Rule('user:password trong URL', RegExp(r'://[^/\s:@]+:[^/\s@]+@')),
  _Rule('chuỗi ngẫu nhiên dài', RegExp(r'^[A-Za-z0-9+/_=-]{32,}$')),
];
```

Bộ quét chỉ báo **tên khoá**, không bao giờ in giá trị, vì kết quả sẽ hiện trên CI (test `kết quả không bao giờ chứa giá trị`). Mẫu giả trong test được ghép lúc chạy để mã nguồn không chứa chuỗi trông như thật. Đây là lưới thô, không thay được kỷ luật không bỏ bí mật vào file.

## 4. AppConfig: kiểm tra khi khởi động và fail sớm

Thiếu một define thì app vẫn chạy với chuỗi rỗng và lỗi nổ ở request đầu tiên, xa chỗ gây ra. Ta muốn sai cấu hình là dừng ngay lúc khởi động và nói rõ khoá nào sai. `AppConfig.fromMap` nhận một bảng chuỗi (cùng dạng công cụ gửi cho compiler), nên là hàm thuần, test được mọi trường hợp mà không cần build lại:

### app_config.dart · 25-flavors-config/lib/config/app_config.dart (trích)

```dart
factory AppConfig.fromMap(Map<String, String> env, {String? nativeFlavor}) {
  final problems = <String>[];

  String? read(String key) {
    final value = env[key];
    if (value == null || value.trim().isEmpty) {
      problems.add('thiếu khoá $key (hoặc giá trị rỗng)');
      return null;
    }
    return value;
  }

  final flavor = _flavor(read(keyFlavor), nativeFlavor, problems);
  final apiBaseUrl = _baseUrl(read(keyApiBaseUrl), flavor, problems);
  final logHttp = _bool(keyLogHttp, read(keyLogHttp), problems);
  final connectTimeout = _timeout(
    keyConnectTimeoutMs,
    read(keyConnectTimeoutMs),
    problems,
  );
  final receiveTimeout = _timeout(
    keyReceiveTimeoutMs,
    read(keyReceiveTimeoutMs),
    problems,
  );

  if (flavor == Flavor.prod && logHttp == true) {
    problems.add('prod không được bật $keyLogHttp');
  }
  if (problems.isNotEmpty) {
    throw ConfigException(problems);
  }
  return AppConfig(
    flavor: flavor!,
    apiBaseUrl: apiBaseUrl!,
    logHttp: logHttp!,
    connectTimeout: connectTimeout!,
    receiveTimeout: receiveTimeout!,
  );
}
```

Hàm gom **mọi** lỗi vào một `ConfigException`, để người sửa không phải chạy đi chạy lại. Các quy tắc, mỗi dòng có test riêng:

| Khoá | Quy tắc | `dev` | `staging` | `prod` |
|---|---|---|---|---|
| `FLAVOR` | đúng `dev`, `staging` hoặc `prod`, không đoán hoa thường | có | có | có |
| `API_BASE_URL` | `http(s)` tuyệt đối có host, không `user:password`, không query, không fragment | có | có | có |
| `API_BASE_URL` | bắt buộc `https` | không | có | có |
| `API_BASE_URL` | không trỏ máy cục bộ (`localhost`, `10.0.2.2`, dải riêng, `.local`...) | không | có | có |
| `LOG_HTTP` | đúng `true` hoặc `false` | có | có | có, và phải là `false` |
| `*_TIMEOUT_MS` | số nguyên trong 1000 đến 60000 | có | có | có |

Phần địa chỉ là phần chặt nhất:

### app_config.dart · 25-flavors-config/lib/config/app_config.dart (trích)

```dart
static Uri? _baseUrl(String? raw, Flavor? flavor, List<String> problems) {
  if (raw == null) {
    return null;
  }
  final uri = Uri.tryParse(raw);
  final isHttp =
      uri != null && (uri.scheme == 'http' || uri.scheme == 'https');
  if (uri == null || !isHttp || uri.host.isEmpty) {
    problems.add('$keyApiBaseUrl phải là địa chỉ http(s) tuyệt đối có host');
    return null;
  }
  if (uri.userInfo.isNotEmpty) {
    problems.add('$keyApiBaseUrl không được chứa user:password');
  }
  if (uri.hasQuery || uri.hasFragment) {
    problems.add('$keyApiBaseUrl không được kèm query hoặc fragment');
  }
  if (flavor != null && !flavor.allowsLocalBackend) {
    if (uri.scheme != 'https') {
      problems.add('${flavor.name} phải dùng https cho $keyApiBaseUrl');
    }
    if (isLocalHost(uri.host)) {
      problems.add(
        '${flavor.name} không được trỏ $keyApiBaseUrl tới máy cục bộ '
        '(host ${uri.host})',
      );
    }
  }
  return uri;
}
```

Thông báo lỗi không in lại giá trị URL, vì URL sai có thể mang `user:password@` hay `?key=...` và thông báo sẽ lên màn hình (test `không chứa user:password, query hay fragment; thông báo không lộ giá trị`). Bắt buộc `https` cho `staging` và `prod`, hay cấm log HTTP ở `prod`, là chính sách của lab, không phải luật của Flutter.

### appFlavor khác dart-define ở chỗ nào

`appFlavor` nằm trong `package:flutter/services.dart`. Theo mã nguồn, nó chỉ là `String.fromEnvironment('FLUTTER_APP_FLAVOR')`, trả `null` khi chuỗi rỗng. Công cụ Flutter đặt define đó từ cờ `--flavor` (hoặc `default-flavor` trong `pubspec.yaml`). Hai nguồn so với nhau như sau:

| | `FLAVOR` trong file JSON | `appFlavor` |
|---|---|---|
| Ai đặt | ta, trong `config/<tên>.json` | công cụ Flutter, từ `--flavor` |
| Đi cùng | `--dart-define-from-file` | Gradle `productFlavors`, Xcode scheme |
| Khi không đặt | thiếu khoá, `AppConfig` báo lỗi | `null`, nghĩa là không build theo flavor native |
| Ta tự đặt được không | được | không: công cụ từ chối `FLUTTER_APP_FLAVOR` trong `--dart-define` và file |

Dòng cuối đã chạy thử: `flutter test --dart-define=FLUTTER_APP_FLAVOR=prod` dừng với thông báo `FLUTTER_APP_FLAVOR is used by the framework...`. `AppConfig` đối chiếu hai nguồn khi `appFlavor` có mặt:

### app_config.dart · 25-flavors-config/lib/config/app_config.dart (trích)

```dart
static Flavor? _flavor(
  String? raw,
  String? nativeFlavor,
  List<String> problems,
) {
  if (raw == null) {
    return null;
  }
  final flavor = Flavor.tryParse(raw);
  if (flavor == null) {
    problems.add('$keyFlavor chỉ nhận dev, staging hoặc prod');
    return null;
  }
  if (nativeFlavor != null && nativeFlavor != flavor.name) {
    problems.add(
      'build bằng --flavor $nativeFlavor nhưng $keyFlavor trong file cấu '
      'hình là ${flavor.name}: hai nguồn này không khớp',
    );
  }
  return flavor;
}
```

Chạy `flutter test --flavor prod --dart-define-from-file=config/dev.json` cho ra `build bằng --flavor prod nhưng FLAVOR trong file cấu hình là dev: hai nguồn này không khớp`. Không có `--flavor` thì `appFlavor` là `null` và phép so bị bỏ qua.

Ném ngoại lệ trước `runApp` không giúp được tester cầm bản TestFlight vì tester không có console. Lab dựng widget gốc trong một hàm: cấu hình đúng thì chạy app, sai thì chạy màn hình liệt kê từng lỗi.

### bootstrap.dart · 25-flavors-config/lib/bootstrap.dart (trích)

```dart
Widget buildRoot({
  AppConfig Function() loadConfig = AppConfig.fromEnvironment,
  List<Override> overrides = const [],
}) {
  try {
    final config = loadConfig();
    return ProviderScope(
      overrides: [appConfigProvider.overrideWithValue(config), ...overrides],
      child: const ToiNoiApp(),
    );
  } on ConfigException catch (error) {
    return ConfigErrorApp(problems: error.problems);
  }
}
```

Chỉ `ConfigException` được bắt; lỗi khác vẫn nổ để không che bug thật (test `lỗi không phải ConfigException không bị nuốt`).

## 5. appConfigProvider: DI theo môi trường

Cấu hình đọc xong phải đến được mọi nơi cần nó mà không dùng biến toàn cục. Ta đặt nó vào một provider **cố ý không có giá trị mặc định**:

### providers.dart · 25-flavors-config/lib/providers.dart (trích)

```dart
final appConfigProvider = Provider<AppConfig>(
  (ref) => throw StateError(
    'appConfigProvider chưa được ghi đè: thêm '
    'appConfigProvider.overrideWithValue(config) vào ProviderScope.',
  ),
);
```

`bootstrap.dart` ở trên ghi đè provider này bằng `AppConfig.fromEnvironment()` đã kiểm tra, còn test ghi đè bằng cấu hình của từng flavor. Quên ghi đè thì `container.read(appConfigProvider)` ném `ProviderException` bọc `StateError` có câu hướng dẫn (test `quên ghi đè thì đọc là lỗi ngay, nêu cách sửa`), không chạy với địa chỉ đoán mò.

`apiClientProvider` là nơi duy nhất quyết định client trông thế nào theo môi trường:

### providers.dart · 25-flavors-config/lib/providers.dart (trích)

```dart
/// Client dùng chung. Địa chỉ, timeout và việc có log hay không đều lấy từ
/// [appConfigProvider], không có `if (kDebugMode)` rải rác.
final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final config = ref.watch(appConfigProvider);
  final client = ToiNoiApiClient(
    baseUrl: config.apiBaseUrl,
    adapter: ref.watch(httpAdapterProvider),
    interceptors: [
      if (config.logHttp) SafeLogInterceptor(ref.watch(logSinkProvider)),
    ],
    connectTimeout: config.connectTimeout,
    receiveTimeout: config.receiveTimeout,
  );
  ref.onDispose(client.close);
  return client;
});
```

Không có `if (kDebugMode)` rải rác: `kDebugMode` phản ánh chế độ build (debug, profile, release), một trục độc lập với flavor, và bản release của `dev` vẫn có thể tồn tại. Muốn thay cả implementation theo môi trường, ta viết `switch (config.flavor)` ở đúng một provider như thế. Ngoại lệ có chủ ý là ruy-băng `DEV` hay `STAGING` ở góc màn hình: `ToiNoiApp` đọc `config.isProd` để tester không nhầm bản staging với bản thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 420" role="img" aria-label="Hình 3. Đồ thị phụ thuộc. EnvironmentScreen dùng sampleQuoteProvider và appConfigProvider. sampleQuoteProvider dùng apiClientProvider. apiClientProvider dùng httpAdapterProvider, logSinkProvider và appConfigProvider. main ghi đè appConfigProvider bằng AppConfig.fromEnvironment, test ghi đè bằng cấu hình của từng flavor.">
<defs><marker id="fl25-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl25-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="216" y="16" width="648" height="292" rx="12" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 5"/><text x="232" y="38" class="t-small">ProviderContainer: nơi giữ state</text>
<rect x="16" y="130" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="30" y="156" class="t-b">EnvironmentScreen</text><text x="30" y="176" class="t-small">ConsumerWidget</text>
<rect x="236" y="70" width="210" height="56" rx="10" class="f-pk-soft s-pk"/><text x="250" y="96" class="t-mono">sampleQuoteProvider</text><text x="250" y="116" class="t-small">AsyncNotifier</text>
<rect x="236" y="190" width="210" height="56" rx="10" class="f-pk-soft s-pk"/><text x="250" y="216" class="t-mono">apiClientProvider</text><text x="250" y="236" class="t-small">Provider</text>
<rect x="640" y="56" width="210" height="50" rx="10" class="f-pk-soft s-pk"/><text x="654" y="86" class="t-mono">httpAdapterProvider</text>
<rect x="640" y="126" width="210" height="50" rx="10" class="f-pk-soft s-pk"/><text x="654" y="156" class="t-mono">logSinkProvider</text>
<rect x="640" y="206" width="210" height="76" rx="10" class="f-pk-soft s-pk"/><text x="654" y="234" class="t-mono">appConfigProvider</text><text x="654" y="254" class="t-small">không có mặc định</text>
<path d="M188 154 L234 108" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-3-a)"/>
<path d="M100 192 L100 316 L820 316 L820 284" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-3-a)"/>
<path d="M341 128 L341 188" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-3-a)"/>
<path d="M448 205 L638 90" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-3-a)"/>
<path d="M448 216 L638 152" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-3-a)"/>
<path d="M448 228 L638 236" class="f-none s-pk" stroke-width="2" marker-end="url(#fl25-3-a)"/>
<rect x="564" y="336" width="300" height="70" rx="10" class="f-warn-soft s-warn"/><text x="578" y="362" class="t-b">main()</text><text x="578" y="382" class="t-small">ghi đè bằng cấu hình đã kiểm tra</text><text x="578" y="400" class="t-mono">AppConfig.fromEnvironment()</text>
<rect x="216" y="336" width="300" height="70" rx="10" class="f-warn-soft s-warn"/><text x="230" y="362" class="t-b">Test</text><text x="230" y="382" class="t-small">ghi đè theo từng flavor</text><text x="230" y="400" class="t-small">và httpAdapterProvider</text>
<path d="M714 334 L740 284" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl25-3-w)"/>
<path d="M480 334 L640 280" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl25-3-w)"/>
</svg>
</div>
<figcaption>Hình 3. Mũi tên liền đi từ bên dùng tới bên được dùng; mũi tên đứt là ghi đè. Chỉ <code>appConfigProvider</code> cần ghi đè theo môi trường. <code>apiClientProvider</code> đọc nó trực tiếp nên tự đổi hành vi theo flavor; <code>httpAdapterProvider</code> và <code>logSinkProvider</code> không đọc nó, chúng chỉ là điểm để test inject bản thay thế vào.</figcaption>
</figure>

Lợi ích thấy ngay trong test: cùng một đoạn code báo giá, chạy với cấu hình của cả ba file, đi tới đúng host của từng flavor. Request đi qua `InProcessAdapter` vào fake API trong tiến trình nên không cần mạng:

### providers_test.dart · 25-flavors-config/test/providers_test.dart (trích)

```dart
test(
  '$name: cùng một đoạn code báo giá đi tới đúng host của flavor',
  () async {
    final harness = Harness();
    final container = harness.containerFor(config);
    addTearDown(() => container.dispose());

    await container.read(sampleQuoteProvider.notifier).fetch();

    final request = harness.adapter.requests.single;
    expect(request.uri.host, config.apiBaseUrl.host);
    expect(request.uri.scheme, config.apiBaseUrl.scheme);
    expect(request.uri.path, '/quotes');
    // 5 km là 25.500 VND theo bảng giá minh hoạ, tính bằng calculateFare.
    expect(container.read(sampleQuoteProvider).value, calculateFare(5));
    expect(container.read(sampleQuoteProvider).value, const Money(25500));
  },
);
```

Cước 25.500 VND là kết quả của `calculateFare(5)` theo bảng giá minh hoạ của loạt bài.

## 6. Chạy từng flavor

Khi đã có runner (lab hướng dẫn sinh bằng `flutter create`), mỗi flavor là một lệnh:

```bash
flutter run --dart-define-from-file=config/dev.json
flutter run --dart-define-from-file=config/staging.json
flutter run --dart-define-from-file=config/prod.json
```

Vướng mắc thực tế đều xoay quanh địa chỉ `dev`:

- Android emulator: `localhost` là chính emulator, và tài liệu về mạng của emulator dành `10.0.2.2` cho máy dev. Ta không sửa `dev.json` mà thêm một define ghi đè nhờ quy tắc "`--dart-define` thắng file": `flutter run --dart-define-from-file=config/dev.json --dart-define=API_BASE_URL=http://10.0.2.2:8787`.
- Điện thoại thật: `localhost` là chính điện thoại. Lab có `config/local.json.example`; sao chép thành `config/local.json` (đã nằm trong `.gitignore`) và điền địa chỉ LAN của máy dev.

Lệnh build (`flutter build apk`, `appbundle`, `ipa`) nhận đúng hai cờ đó, cộng `--flavor` khi bài 50 đã có flavor native. Ta chưa chạy chúng vì máy kiểm không có Android SDK.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi tách cấu hình theo môi trường</span>
<ul>
<li><strong>Gọi <code>String.fromEnvironment</code> không có <code>const</code>.</strong> Dấu hiệu: địa chỉ API rỗng dù có define, không báo lỗi (tuỳ nền tảng). Nguyên nhân: tài liệu chỉ đảm bảo hoạt động khi gọi bằng <code>const</code>. Cách sửa: <code>const</code> ở mọi lời gọi và để <code>AppConfig</code> từ chối giá trị rỗng.</li>
<li><strong>File cấu hình và <code>--flavor</code> lệch nhau.</strong> Dấu hiệu: bản mang ID <code>vn.toinoi.customer</code> nhưng gọi API dev. Nguyên nhân: hai đường độc lập. Cách sửa: so <code>FLAVOR</code> với <code>appFlavor</code> lúc khởi động, và đưa cả hai cờ vào một script.</li>
<li><strong>Đặt bí mật vào JSON hoặc <code>--dart-define</code>.</strong> Dấu hiệu: khoá tên <code>*_SECRET</code> hay <code>*_KEY</code> trong <code>config/</code>, hoặc <code>grep -a</code> thấy chuỗi đó trong binary. Nguyên nhân: giá trị define trở thành hằng nằm trong app. Cách sửa: chuyển về server, thu hồi khoá đã lộ.</li>
<li><strong>Dùng <code>int.fromEnvironment</code> hay <code>bool.fromEnvironment</code> trực tiếp.</strong> Dấu hiệu: timeout luôn bằng mặc định, hoặc <code>LOG_HTTP</code> không bật dù file ghi <code>True</code>. Nguyên nhân: giá trị sai bị nuốt và thay bằng mặc định. Cách sửa: đọc chuỗi, tự <code>int.tryParse</code> và báo lỗi nêu tên khoá.</li>
<li><strong><code>null</code> trong JSON.</strong> Dấu hiệu: <code>API_BASE_URL</code> là chuỗi <code>"null"</code>. Nguyên nhân: công cụ nội suy mọi giá trị thành chuỗi. Cách sửa: không dùng <code>null</code>; test <code>kiểu JSON đúng ý</code> chặn nó.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/25-flavors-config
flutter pub get
flutter analyze
flutter test
node tool/verify-flavors.mjs
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `flutter_riverpod` 3.5.0, `dio` 5.11.1: `flutter analyze` 0 issue, `flutter test` **86 test xanh, 5 test bỏ qua** có chủ ý. Mức: **Kiểm chứng tự động**, không cần thiết bị hay mạng. `tool/verify-flavors.mjs` chạy thêm bộ test với từng flavor (88 xanh, 3 bỏ qua mỗi lần), một lần chỉ có file, rồi hai lần cố ý thất bại: file `dev` lệch `--flavor prod`, và tự đặt `FLUTTER_APP_FLAVOR`.

| Test | Chứng minh điều gì |
|---|---|
| `app_config_test.dart` (25) | Mọi quy tắc của mục 4: thiếu khoá, giá trị biên của timeout, https, host cục bộ và host công cộng không bị nhầm, không lộ giá trị trong thông báo, lệch `appFlavor` |
| `config_files_test.dart` (22) | Mỗi JSON đúng bộ khoá và kiểu, qua `AppConfig`, không giống bí mật; prod không có localhost, staging khác host prod, `local.json` bị `.gitignore` chặn |
| `secret_scan_test.dart` (12) | Bộ quét bắt tám dạng mẫu và tên khoá gợi ý, không báo nhầm cấu hình thật, không in giá trị |
| `environment_test.dart` (3 + 5 bỏ qua) | Không define thì `fromEnvironment` báo đủ năm khoá; có define thì khớp file, lời gọi không `const` cho ra chuỗi rỗng trong `flutter test` |
| `providers_test.dart` (13) | `appConfigProvider` chưa ghi đè; client, log và host theo từng flavor; dev có log, prod không |
| `app_test.dart` (11) | `buildRoot`: màn hình đúng cấu hình, ruy-băng, báo giá thử 25.500 VND, màn hình lỗi cấu hình |

**Chưa làm**: Gradle `productFlavors`, Xcode scheme, icon, tên app và Firebase theo môi trường (bài 50); chọn file cấu hình trong CI (bài 54); chạy trên simulator, emulator hay thiết bị; dựng `libapp.so` hoặc IPA.

## Bài tập

1. Thêm khoá bool `ENABLE_PROMO` vào `dev.json` mà chưa sửa gì khác. Test nào đỏ đầu tiên, và cần sửa những nơi nào để xanh lại?
2. Giả sử `staging.json` bị sửa nhầm thành địa chỉ của `prod`. `AppConfig.fromMap` có bắt được không? Test nào bắt?
3. Đoán kết quả của `flutter test --flavor prod --dart-define-from-file=config/dev.json` và nói vì sao nhóm "không define" không chạy.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> <code>có đúng bộ khoá của AppConfig, không thừa không thiếu</code> đỏ cho <code>dev.json</code>: file có sáu khoá, <code>AppConfig.keys</code> có năm. Để xanh: thêm khoá vào hai file còn lại, thêm hằng vào <code>AppConfig.keys</code> và bảng <code>_compileTimeEnvironment</code> (với <code>const</code>), đọc nó trong <code>fromMap</code> chỉ nhận <code>true</code> hoặc <code>false</code>, rồi thêm test giá trị sai.</p>
<p><strong>2.</strong> Không. <code>fromMap</code> chỉ thấy một cấu hình mỗi lần, và địa chỉ đó hợp lệ với <code>staging</code>. Quy tắc "hai file khác nhau" cần nhìn cả hai nên nằm ở test: <code>staging: https, không trỏ máy cục bộ, khác host của prod</code> và <code>ba file khác nhau ở địa chỉ API</code> đỏ.</p>
<p><strong>3.</strong> Thất bại. Có define nên nhóm "không define" bị bỏ qua; nhóm "có define" gọi <code>AppConfig.fromEnvironment()</code>, thấy <code>appFlavor</code> là <code>prod</code> còn <code>FLAVOR</code> là <code>dev</code>, và ném <code>ConfigException</code> với câu <em>hai nguồn này không khớp</em>. <code>tool/verify-flavors.mjs</code> chạy đúng lệnh này và coi thất bại là mong đợi.</p>
</details>

Bài trước: [Authentication: OTP, JWT, refresh token rotation và interceptor](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage). Bài tiếp: [Code generation: freezed, json_serializable, riverpod_generator và build_runner](/articles/flutter-code-generation-freezed-json-serializable-riverpod-generator).

## Tài liệu gốc

- [Set up Flutter flavors for Android](https://docs.flutter.dev/deployment/flavors) và [Set up Flutter flavors for iOS and macOS](https://docs.flutter.dev/deployment/flavors-ios) (docs.flutter.dev).
- [Configuring apps with compilation environment declarations](https://dart.dev/libraries/core/environment-declarations) (dart.dev).
- [String.fromEnvironment](https://api.dart.dev/stable/dart-core/String/String.fromEnvironment.html) (api.dart.dev).
- [appFlavor](https://api.flutter.dev/flutter/services/appFlavor-constant.html) (api.flutter.dev).
- [Configure build variants](https://developer.android.com/build/build-variants) và [Emulator networking](https://developer.android.com/studio/run/emulator-networking) (developer.android.com).
- [Testing your providers](https://riverpod.dev/docs/how_to/testing) và [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) (riverpod.dev, pub.dev).

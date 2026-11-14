---
slug: flutter-va-dart-tu-source-code-den-ung-dung-toi-noi
title: "Flutter và Dart: từ source code đến ứng dụng Tới Nơi chạy trên iOS và Android"
summary: "Bài 01: phân biệt Dart, Flutter framework, engine và tooling; tạo app Tới Nơi đầu tiên, đọc cấu trúc dự án, hiểu JIT khi dev và AOT khi release, hot reload và cách chạy trên iOS Simulator, Android emulator. Có widget test chạy được."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, ios, android]
language: vi
---

Một khách mở app [Tới Nơi](/articles/du-an-mau-toi-noi) trên iPhone, gõ quãng đường 5 km và muốn thấy giá trước khi bấm đặt xe. Cùng lúc đó, một tài xế mở app trên điện thoại Android và cần thấy đúng con số ấy. Hai máy khác hệ điều hành, nhưng ta chỉ muốn viết quy tắc tính cước và màn hình báo giá một lần. Flutter sinh ra để làm việc đó.

Đây là bài 01 của lộ trình **Flutter qua Tới Nơi**. Bài này không có bài trước. Bạn chỉ cần mở được terminal và một editor. Ta sẽ tạo một app có một màn hình, tính cước 5 km, kiểm chứng bằng test, rồi lần theo đường đi từ file `.dart` tới pixel trên màn hình. Loạt C# đã làm đúng việc này trong [bài 01 của C#](/articles/csharp-va-dotnet-10-tu-source-code-den-chuong-trinh) với cùng bảng giá, nên bạn có thể đối chiếu hai ngôn ngữ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã cài Flutter SDK thì còn phải cài Dart SDK riêng để chạy <code>flutter test</code>, đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p><b>Sai.</b> Flutter SDK đã đóng gói sẵn một Dart SDK đúng phiên bản. Lệnh <code>flutter</code> dùng nó, và ta cũng có thể gọi <code>dart</code> từ cùng thư mục <code>bin</code> của Flutter. Cài thêm một Dart SDK khác chỉ dễ gây lệch phiên bản.</p></details>
</div>

## Dart, framework, engine và tooling khác nhau ở đâu?

Lệnh đầu tiên cho thấy ba thứ được đóng gói cùng nhau trong một SDK:

```text
$ flutter --version
Flutter 3.47.4 • channel stable
Framework • revision 9584c6713b (2026-09-10)
Engine • hash 0e228ec8c8d2abc9fcf1d053e8a40665bb859ec7
Tools • Dart 3.13.3 • DevTools 2.60.0
```

Dòng Framework, Engine và Tools tương ứng ba hàng của bảng dưới; Dart 3.13.3 đi kèm trong Tools.

Người mới thường gọi tất cả là "Flutter". Khi gặp lỗi, cách gọi đó khiến ta không biết nên tìm ở đâu. Bảng dưới tách từng tầng. Dart đóng vai ngôn ngữ, giống C#. **Flutter SDK** (cả gói) là một nền tảng ứng dụng; riêng tầng framework bên trong nó là một thư viện Dart.

| Thành phần | Trả lời câu hỏi nào? | Trong bài này |
| --- | --- | --- |
| Dart | Viết chương trình bằng ngôn ngữ gì? | `int`, hàm, class, `if` |
| Flutter framework | Dựng giao diện thế nào? | Widget `MaterialApp`, `Scaffold`, `Text`; viết bằng Dart |
| Flutter engine | Ai vẽ pixel, chạy Dart, xếp chữ? | Viết bằng C++; có bộ vẽ **Impeller** và Dart runtime |
| Embedder | Làm sao engine chạy được trên một hệ điều hành? | Lớp mỏng riêng cho iOS và Android: cửa sổ, sự kiện chạm, vòng lặp |
| Tooling | Tạo, chạy, kiểm thử, đóng gói thế nào? | Lệnh `flutter create`, `flutter run`, `flutter test`, `flutter doctor` |

**Flutter framework** là thư viện Dart: mọi thứ trên màn hình là một **widget**, từ nút bấm tới cả cái app. **Engine** nằm dưới, nhận cây giao diện đã tính và biến nó thành pixel. Flutter không dùng nút bấm gốc của iOS hay Android; nó tự vẽ, nên một widget trông gần như giống nhau trên hai hệ điều hành; một số hành vi như cuộn và chuyển trang vẫn tự theo nền tảng.

**Impeller** là bộ vẽ của engine. Trên iOS engine luôn dùng Impeller; trên Android từ Android 10 (API 29) trở lên cũng vậy, còn máy cũ hơn hoặc GPU không hỗ trợ thì engine tự lùi về Skia (OpenGL ES). Ta không gọi nó trực tiếp: nó thuộc engine, không nằm trong `lib/`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Bốn tầng xếp chồng: code Tới Nơi viết bằng Dart ở trên cùng, rồi Flutter framework viết bằng Dart, rồi engine viết bằng C++ gồm Impeller và Dart runtime, dưới cùng là hai embedder riêng cho iOS và Android."><rect x="20" y="15" width="840" height="70" rx="10" class="f-pk-soft s-pk"/><text x="36" y="46" class="t-b">Code của ta (Dart)</text><text x="36" y="70" class="t-small">lib/main.dart, lib/fare.dart: ToiNoiApp, QuoteScreen, calculateFareVnd</text><text x="430" y="104" class="t-b">↓</text><rect x="20" y="115" width="840" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="146" class="t-b">Flutter framework (Dart)</text><text x="36" y="170" class="t-small">Widget, Material, layout, gesture, animation: gói flutter</text><text x="430" y="204" class="t-b">↓</text><rect x="20" y="215" width="840" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="246" class="t-b">Engine (C++)</text><text x="36" y="270" class="t-small">Impeller vẽ pixel, Dart runtime, xếp chữ, kênh nói chuyện với nền tảng</text><text x="430" y="304" class="t-b">↓</text><rect x="20" y="315" width="410" height="70" rx="10" class="f-warn-soft s-warn"/><text x="36" y="346" class="t-b">Embedder iOS</text><text x="36" y="370" class="t-small">Swift/Objective-C trong thư mục ios/</text><rect x="450" y="315" width="410" height="70" rx="10" class="f-warn-soft s-warn"/><text x="466" y="346" class="t-b">Embedder Android</text><text x="466" y="370" class="t-small">Kotlin/Java trong thư mục android/</text></svg>
</div>
<figcaption>Hình 1. Các tầng của một app Flutter. Ta chỉ viết tầng trên cùng; embedder là phần nền tảng riêng nên có hai thư mục <code>ios/</code> và <code>android/</code>.</figcaption>
</figure>

Hình 1 giải thích vì sao một mã nguồn chạy được trên hai hệ điều hành: code Dart và framework giống nhau; engine cùng mã nguồn nhưng được biên dịch riêng cho từng nền tảng (Metal trên iOS, Vulkan hoặc GLES trên Android); embedder khác hẳn, và `flutter create` đã sinh sẵn.

## Chuẩn bị môi trường

Cài Flutter SDK theo hướng dẫn chính thức ở [docs.flutter.dev](https://docs.flutter.dev/get-started/install), chọn hệ điều hành của bạn. Loạt này kiểm chứng với **Flutter 3.47.4 stable** và **Dart 3.13.3**. Dùng channel stable, không dùng beta hay master. Sau khi cài, mở terminal mới rồi chạy:

```bash
flutter --version
flutter doctor
```

`flutter doctor` kiểm tra từng phần của máy và báo bằng ký hiệu. Ví dụ rút gọn trên một máy chưa cài đủ công cụ:

```text
[✓] Flutter (Channel stable, 3.47.4, on macOS, locale en-VN)
[✗] Android toolchain - develop for Android devices
    ✗ Unable to locate Android SDK.
[!] Xcode - develop for iOS and macOS
    ✗ Xcode installation is incomplete; a full installation is necessary.
    ! CocoaPods not installed.
[✓] Chrome - develop for the web
[✓] Connected device (2 available)
[✓] Network resources
```

| Dòng / dòng con | Ý nghĩa | Cần cho |
| --- | --- | --- |
| Flutter | SDK, Dart và tooling hoạt động | Mọi bài. Dòng này phải xanh |
| Android toolchain | Android SDK, JDK, giấy phép đã chấp nhận | Chạy trên Android emulator hoặc máy Android |
| Xcode, gồm dòng con CocoaPods | Xcode đầy đủ, không chỉ command line tools; CocoaPods quản lý thư viện native của iOS | Chạy trên iOS Simulator (chỉ macOS); CocoaPods chỉ cần khi có plugin iOS chưa hỗ trợ Swift Package Manager |
| Chrome | Trình duyệt để chạy Flutter web | Không cần cho loạt này |
| Connected device | Thiết bị, simulator, emulator đang thấy được | Lệnh `flutter run` |

Trong mẫu trên, `Connected device (2 available)` là macOS và Chrome. Chúng không chạy được project tạo với `--platforms ios,android`, nên chưa có đích chạy nào cho app của ta.

Ký hiệu `[✓]` là ổn, `[✗]` là thiếu thứ bắt buộc cho nền tảng đó, còn `[!]` là nhóm đạt một phần: luôn đọc các dòng con. Dòng con `✗` vẫn chặn nền tảng đó (như ví dụ, Xcode chưa đầy đủ thì chưa chạy được iOS); dòng con `!` là cảnh báo (thiếu CocoaPods thì plugin iOS không chạy). Bạn không cần đủ cả iOS lẫn Android để học: trên Windows hoặc Linux dùng Android, trên macOS chọn một trong hai. Bài này chỉ cần dòng Flutter xanh, vì `flutter test` chạy trên máy phát triển. Khi muốn chạy app thật, hãy sửa đúng dòng của nền tảng mình chọn theo thông báo ngay dưới nó.

## Tạo dự án và đọc cấu trúc

Tạo dự án bằng lệnh sau. Cờ `--platforms` chỉ sinh hai nền tảng ta cần, `--org` tạo tiền tố cho application ID trên Android và bundle ID trên iOS, `--project-name` đặt tên package Dart:

```bash
mkdir toi_noi_01 && cd toi_noi_01
flutter create --project-name toi_noi_01 --platforms ios,android --org vn.toinoi .
```

Dấu `.` cuối nghĩa là tạo ngay trong thư mục hiện tại. Lệnh sinh ra khoảng 75 file. Ta chỉ cần đọc vài chỗ quan trọng:

```text
toi_noi_01/
├── pubspec.yaml            mô tả dự án và dependency
├── analysis_options.yaml   luật lint
├── lib/
│   ├── main.dart           điểm vào của app
│   └── fare.dart           quy tắc tính cước (ta tự thêm)
├── test/                   test chạy bằng flutter test
├── android/                project Gradle cho Android
├── ios/                    project Xcode cho iOS
├── .dart_tool/             dữ liệu trung gian do tooling sinh
└── build/                  đầu ra build
```

**`pubspec.yaml`** là trung tâm cấu hình, giống file `.csproj` bên C#. Đây là bản dùng trong lab:

```yaml
name: toi_noi_01
description: "Lab bài 01: app Tới Nơi đầu tiên in cước minh hoạ cho quãng đường 5 km."
publish_to: 'none'
version: 1.0.0+1

environment:
  sdk: ^3.13.3

dependencies:
  flutter:
    sdk: flutter
  cupertino_icons: ^1.0.8

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^6.0.0

flutter:
  uses-material-design: true
```

`name` là tên package; các `import 'package:toi_noi_01/...'` dựa vào nó. `version: 1.0.0+1` gồm số phiên bản người dùng thấy và số build. `dependencies` là thứ app cần lúc chạy; `dev_dependencies` chỉ dùng khi phát triển như test và lint. `flutter_lints` cung cấp bộ luật cho `flutter analyze`. `uses-material-design: true` cho phép dùng biểu tượng Material.

**`lib/`** chứa code Dart của app. **`test/`** chứa test. **`android/`** và **`ios/`** là hai project nền tảng thật: Android Studio mở được `android/`, Xcode mở được `ios/`. Ta hiếm khi sửa chúng ở những bài đầu, nhưng sẽ phải động vào khi cấu hình quyền, push và ký ứng dụng.

**`.dart_tool/`** và **`build/`** do tooling sinh ra, có thể xoá và dựng lại; không đưa chúng vào Git. Lab cũng bỏ qua `pubspec.lock` vì là ví dụ, không phải app phát hành. Khi có plugin cần CocoaPods, thư mục `ios/Pods/` xuất hiện; cũng không commit.

## Từ main() tới pixel

Mở `lib/main.dart` của lab. Phần đầu là điểm vào và hai widget:

```dart
import 'package:flutter/material.dart';

import 'fare.dart';

void main() {
  runApp(const ToiNoiApp());
}

class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
        useMaterial3: true,
      ),
      home: const QuoteScreen(),
    );
  }
}
```

`main()` là hàm Dart đầu tiên được gọi, giống `Main` bên C#. Hàm này gọi `runApp`, truyền vào widget gốc. Flutter gắn widget đó vào cửa sổ do embedder cung cấp, rồi bắt đầu vòng lặp: nhận sự kiện, gọi `build`, vẽ khung hình.

`ToiNoiApp` là một `StatelessWidget`: widget không giữ trạng thái riêng, chỉ có phương thức `build` trả về một widget khác. Từ khoá `const` báo Dart rằng đối tượng này bất biến và được tạo sẵn lúc biên dịch. `MaterialApp` bọc toàn bộ app, cấp theme và điều hướng. `useMaterial3: true` bật Material 3; đây vốn là mặc định, ta ghi ra cho rõ. `ColorScheme.fromSeed` sinh cả bảng màu từ một màu gốc.

Màn hình nằm ở widget thứ hai:

```dart
class QuoteScreen extends StatelessWidget {
  const QuoteScreen({super.key});

  @override
  Widget build(BuildContext context) {
    const distanceKm = 5.0;
    final fareVnd = calculateFareVnd(distanceKm);

    return Scaffold(
      appBar: AppBar(title: const Text('Tới Nơi')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Quãng đường: ${distanceKm.toInt()} km'),
            Text('Cước minh hoạ: ${formatVnd(fareVnd)}'),
          ],
        ),
      ),
    );
  }
}
```

`Scaffold` dựng khung một màn hình Material: thanh tiêu đề `appBar` và phần thân `body`. `Padding` chừa lề 16, `Column` xếp hai `Text` theo chiều dọc, `${...}` chèn giá trị vào chuỗi. Cước đến từ `calculateFareVnd`, hàm thuần Dart ở mục cuối.

Ta vừa mô tả một **cây widget**: `ToiNoiApp` chứa `MaterialApp`, chứa `QuoteScreen`, chứa `Scaffold`, rồi tới `Column` và các `Text`. Mỗi `build` trả về một nhánh của cây. Framework so sánh cây mới với cây cũ rồi chuyển phần thay đổi xuống engine. Cách cây được dựng và cập nhật sẽ có bài riêng; hôm nay chỉ cần bạn nhìn ra hình dạng của nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 190" role="img" aria-label="main gọi runApp với ToiNoiApp; framework dựng cây MaterialApp, Scaffold, Column, Text; engine vẽ pixel lên màn hình do embedder cung cấp."><rect x="20" y="40" width="190" height="110" rx="10" class="f-pk-soft s-pk"/><text x="32" y="82" class="t-b">main()</text><text x="32" y="113" class="t-small">runApp(ToiNoiApp)</text><text x="213" y="98" class="t-b">→</text><rect x="235" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="247" y="82" class="t-b">Cây widget</text><text x="247" y="113" class="t-small">Widget lồng nhau</text><text x="428" y="98" class="t-b">→</text><rect x="450" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="462" y="82" class="t-b">Engine</text><text x="462" y="113" class="t-small">Vẽ khung hình</text><text x="643" y="98" class="t-b">→</text><rect x="665" y="40" width="195" height="110" rx="10" class="f-warn-soft s-warn"/><text x="677" y="82" class="t-b">Màn hình</text><text x="677" y="113" class="t-small">iOS hoặc Android</text></svg>
</div>
<figcaption>Hình 2. Đường đi từ hàm main tới pixel. Bước thứ hai là phần ta viết bằng widget; hai bước sau do engine và embedder lo.</figcaption>
</figure>

## Chạy ở đâu và chạy thế nào

Khi phát triển, ta muốn sửa một dòng và thấy ngay. Khi phát hành, ta muốn app khởi động nhanh và chạy mượt. Hai nhu cầu cần hai cách biên dịch, và Dart hỗ trợ cả hai.

- **JIT (just-in-time)** dùng ở chế độ debug. Dart VM chạy trên thiết bị và biên dịch code khi cần. Vì VM còn nguyên trong app, Flutter có thể nạp code mới vào VM đang chạy mà không dựng lại app. Đó là **hot reload**.
- **AOT (ahead-of-time)** dùng ở chế độ release. Toàn bộ Dart được biên dịch trước thành mã máy khi build. App không còn VM biên dịch động, khởi động nhanh và chạy gần với tốc độ native. Trên iOS, bản release buộc phải là AOT.

Chế độ **profile** nằm giữa: AOT nhưng còn công cụ đo, dùng để đo hiệu năng trên thiết bị thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hai hàng. Hàng debug: code Dart nạp vào Dart VM dùng JIT trên thiết bị, sửa code thì hot reload nạp lại. Hàng release: code Dart biên dịch AOT thành mã máy đóng trong gói app, không còn VM biên dịch động."><text x="20" y="28" class="t-b">Debug: flutter run (JIT)</text><rect x="20" y="42" width="250" height="90" rx="10" class="f-phys-soft s-phys"/><text x="32" y="82" class="t-b">Source .dart</text><text x="32" y="110" class="t-small">Sửa rồi lưu</text><text x="274" y="92" class="t-b">→</text><rect x="295" y="42" width="270" height="90" rx="10" class="f-phys-soft s-phys"/><text x="307" y="82" class="t-b">Dart VM trên thiết bị</text><text x="307" y="110" class="t-small">Biên dịch JIT khi cần</text><text x="570" y="92" class="t-b">→</text><rect x="590" y="42" width="270" height="90" rx="10" class="f-pk-soft s-pk"/><text x="602" y="82" class="t-b">Hot reload</text><text x="602" y="110" class="t-small">Nạp code mới, giữ state</text><text x="20" y="188" class="t-b">Release: flutter build (AOT)</text><rect x="20" y="202" width="250" height="90" rx="10" class="f-phys-soft s-phys"/><text x="32" y="242" class="t-b">Source .dart</text><text x="32" y="270" class="t-small">Đã qua analyze và test</text><text x="274" y="252" class="t-b">→</text><rect x="295" y="202" width="270" height="90" rx="10" class="f-warn-soft s-warn"/><text x="307" y="242" class="t-b">Biên dịch AOT</text><text x="307" y="270" class="t-small">Dart thành mã máy</text><text x="570" y="252" class="t-b">→</text><rect x="590" y="202" width="270" height="90" rx="10" class="f-warn-soft s-warn"/><text x="602" y="242" class="t-b">APK, AAB, IPA</text><text x="602" y="270" class="t-small">Không có hot reload</text></svg>
</div>
<figcaption>Hình 3. Hai đường biên dịch của Dart. Hàng trên tối ưu tốc độ sửa code; hàng dưới tối ưu tốc độ chạy và kích thước gói.</figcaption>
</figure>

Để chạy app, lấy danh sách thiết bị rồi chọn một cái:

```bash
flutter devices
flutter emulators
flutter emulators --launch <emulator-id>
flutter run -d <id>
```

`flutter devices` in ra các đích chạy mà tooling nhìn thấy: iOS Simulator, Android emulator, máy thật cắm dây, thậm chí Chrome. `flutter emulators` liệt kê emulator và simulator đã tạo sẵn để bạn khởi động. `flutter run -d <id>` build bản debug, cài vào thiết bị đó và mở app.

Trong lúc `flutter run` đang chạy, terminal nhận phím tắt. Bấm `r` để **hot reload**: Flutter nạp lại code đã sửa vào VM và rebuild cây widget, giữ nguyên state hiện có. Bấm `R` để **hot restart**: khởi động lại ứng dụng Dart từ `main()`, state bị xoá nhưng không phải build lại phần native. Sửa chữ trong `Text` chỉ cần hot reload. Đổi nội dung `main()` (hoặc `initState` đã chạy), giá trị khởi tạo của biến toàn cục hay static, đổi một class thành enum hoặc ngược lại, đổi tham số generic của một kiểu, hay sửa code native thì cần hot restart (code native còn cần build lại).

Để đóng gói app phát hành, chỉ cần nhớ hai lệnh. `flutter build apk` hoặc `flutter build appbundle` tạo gói Android, `flutter build ipa` tạo gói iOS (cần macOS và tài khoản ký). Cả hai dùng AOT ở chế độ release. Bài này chưa chạy chúng vì cần chứng chỉ ký; chi tiết nằm ở phần phát hành.

## Tính cước đầu tiên và kiểm chứng bằng test

Trước khi tin giao diện, ta tin quy tắc. `lib/fare.dart` không import gì của Flutter: đây là Dart thuần nên test chạy nhanh, không cần thiết bị.

### lib/fare.dart (01-flutter-va-dart)

```dart
/// Bảng giá minh hoạ của Tới Nơi, không phải bảng giá thực tế.
const int _baseFareVnd = 12000;
const double _includedKm = 2;
const int _pricePerExtraKmVnd = 4500;
const double _maxDistanceKm = 100;

/// Tính cước (đồng VND) cho quãng đường [distanceKm].
///
/// 12.000 VND cho 2 km đầu, 4.500 VND cho mỗi km vượt, làm tròn bằng
/// `round()`. Ném [ArgumentError] khi khoảng cách ngoài miền 0 đến 100 km.
int calculateFareVnd(double distanceKm) {
  if (distanceKm < 0 || distanceKm > _maxDistanceKm) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0 đến 100 km',
    );
  }
  final extraKm = distanceKm > _includedKm ? distanceKm - _includedKm : 0.0;
  return (_baseFareVnd + extraKm * _pricePerExtraKmVnd).round();
}
```

Tiền VND là `int` đồng, không có phần lẻ; khoảng cách là `double` km. Tên bắt đầu bằng `_` nghĩa là private trong file. Hàm chặn hai đầu: âm hoặc quá 100 km thì ném `ArgumentError` thay vì trả 0, vì 0 VND trông như một chuyến miễn phí hợp lệ. Với 5 km, phần vượt là 3 km, tổng bằng 12.000 + 3 × 4.500 = 25.500. Với 2,001 km, phần vượt 0,001 km tính ra 4,5 VND, và `round()` đưa tổng lên 12.005. Kết quả này phụ thuộc cách `double` biểu diễn số lẻ nhị phân; bài 02 sẽ nói kỹ vì sao tiền không nên tính bằng `double` lâu dài. Bảng giá là số liệu minh hoạ, không phải giá thực tế của sản phẩm.

Màn hình cần hiển thị `25500` thành `25.500 VND`. Ta chưa dùng `intl`, nên viết tay một hàm nhỏ ở cuối `main.dart`:

### lib/main.dart, hàm định dạng (01-flutter-va-dart)

```dart
/// Định dạng số tiền bằng tay: 25500 thành "25.500 VND".
/// Bài về `intl` sẽ thay hàm này bằng định dạng theo locale.
String formatVnd(int amount) {
  final digits = amount.toString();
  final buffer = StringBuffer();
  for (var i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 == 0) {
      buffer.write('.');
    }
    buffer.write(digits[i]);
  }
  return '$buffer VND';
}
```

Giờ tới kiểm chứng. Test của `calculateFareVnd` đi qua các test case biên của bảng giá:

### test/fare_test.dart (01-flutter-va-dart)

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_01/fare.dart';

void main() {
  group('calculateFareVnd', () {
    test('0 km vẫn thu giá cơ bản', () {
      expect(calculateFareVnd(0), 12000);
    });

    test('2 km đầu nằm trong giá cơ bản', () {
      expect(calculateFareVnd(2), 12000);
    });

    test('5 km: 12.000 + 3 x 4.500', () {
      expect(calculateFareVnd(5), 25500);
    });

    test('2,001 km: phần lẻ làm tròn bằng round() thành 12.005', () {
      expect(calculateFareVnd(2.001), 12005);
    });

    test('100 km là biên trên hợp lệ', () {
      expect(calculateFareVnd(100), 12000 + 98 * 4500);
    });

    test('khoảng cách âm bị từ chối', () {
      expect(() => calculateFareVnd(-1), throwsArgumentError);
    });

    test('vượt 100 km bị từ chối', () {
      expect(() => calculateFareVnd(100.5), throwsArgumentError);
    });
  });
}
```

Còn **widget test** dựng cả app trong một môi trường giả, không cần máy ảo, rồi tìm chữ trên màn hình:

### test/widget_test.dart (01-flutter-va-dart)

```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_01/main.dart';

void main() {
  testWidgets('màn hình báo giá hiển thị tiêu đề và cước 5 km',
      (WidgetTester tester) async {
    await tester.pumpWidget(const ToiNoiApp());

    expect(find.text('Tới Nơi'), findsOneWidget);
    expect(find.text('Quãng đường: 5 km'), findsOneWidget);
    expect(find.text('Cước minh hoạ: 25.500 VND'), findsOneWidget);
    expect(find.byType(Scaffold), findsOneWidget);
  });

  group('formatVnd', () {
    test('chèn dấu chấm mỗi ba chữ số', () {
      expect(formatVnd(0), '0 VND');
      expect(formatVnd(999), '999 VND');
      expect(formatVnd(12000), '12.000 VND');
      expect(formatVnd(25500), '25.500 VND');
      expect(formatVnd(1234567), '1.234.567 VND');
    });
  });
}
```

`tester.pumpWidget` đặt `ToiNoiApp` vào một cây widget thử và dựng khung hình đầu tiên. `find.text` tìm widget `Text` có đúng nội dung, `findsOneWidget` khẳng định chỉ có một. Widget test chạy trên máy phát triển, không cần simulator. Widget test chứng minh cây widget thực sự hiển thị đúng chữ, điều test thuần Dart không kiểm được.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi người mới hay gặp</span>
<ul>
<li><b>Thấy <code>[✗] Android toolchain</code> nên nghĩ không học được.</b> Dòng này chỉ ảnh hưởng việc chạy Android. Nếu bạn chỉ làm iOS hoặc chỉ chạy <code>flutter test</code>, hãy bỏ qua nó. Khi cần Android, cài Android Studio rồi chạy <code>flutter doctor --android-licenses</code>.</li>
<li><b>Sửa code, bấm <code>r</code> nhưng màn hình không đổi.</b> Bạn có thể đã sửa <code>main()</code>, biến toàn cục hay static, những thứ hot reload không chạy lại. Bấm <code>R</code> để hot restart; nếu vẫn không đổi thì dừng và <code>flutter run</code> lại.</li>
<li><b>Chạy <code>flutter test</code> báo không có <code>pubspec.yaml</code> hoặc thư mục <code>test</code>.</b> Bạn đang đứng sai thư mục. Chạy lệnh trong thư mục chứa <code>pubspec.yaml</code> của lab, hoặc truyền đường dẫn file test.</li>
<li><b>App debug chạy giật rồi kết luận Flutter chậm.</b> Chế độ debug dùng JIT và bật nhiều kiểm tra phụ. Hiệu năng chỉ đo đúng ở chế độ profile hoặc release trên thiết bị thật, không đo trên debug và càng không trên simulator.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Code của bài nằm ở `examples/toi-noi-flutter/01-flutter-va-dart`. Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/01-flutter-va-dart
flutter pub get
flutter analyze
flutter test
```

Kết quả mong đợi là `No issues found!` và `All tests passed!` với 9 test. Bảng sau cho thấy test nào chứng minh điều gì.

| Test | Chứng minh |
| --- | --- |
| 0 km, 2 km | Hai đầu của giá cơ bản đều ra 12.000 VND |
| 5 km | 12.000 + 3 × 4.500 = 25.500 VND |
| 2,001 km | Phần lẻ đồng được làm tròn bằng `round()` thành 12.005 |
| 100 km | Biên trên của miền vẫn hợp lệ |
| -1 km, 100,5 km | Ngoài miền thì ném `ArgumentError`, không trả tiền |
| Widget test | Màn hình có tiêu đề "Tới Nơi", dòng quãng đường và "25.500 VND" |
| `formatVnd` | Dấu chấm phân cách hàng nghìn đặt đúng chỗ |

Mức kiểm chứng của bài là **Kiểm chứng tự động**: `flutter analyze` và `flutter test` chạy xanh trên máy không cần thiết bị. Ta **chưa** chạy app trên iOS Simulator, Android emulator hay máy thật, vì máy kiểm không có Android SDK và Xcode đầy đủ; các lệnh `flutter run` ở trên là hướng dẫn để bạn tự thử, chưa có ảnh chụp thật. `flutter build apk` và `flutter build ipa` cũng chưa chạy.

## Bài tập

1. Đổi giá mỗi km vượt thành 5.000 VND. Khách đi 4 km phải trả bao nhiêu, và test nào của lab sẽ đỏ?
2. Với mỗi thay đổi sau, ta dùng hot reload hay hot restart: (a) đổi chữ "Quãng đường" thành "Khoảng cách"; (b) thêm một dòng `print` vào `main()` và muốn thấy nó chạy?
3. Vì sao `calculateFareVnd(-1)` nên ném `ArgumentError` thay vì trả 0?

<details><summary>Xem lời giải</summary>
<p><b>1.</b> Phần vượt là 2 km nên tổng là 12.000 + 2 × 5.000 = <b>22.000 VND</b>. Các test của 5 km và 100 km sẽ đỏ vì đang kỳ vọng 4.500 VND/km. Test 2,001 km vẫn xanh vì 0,001 × 5.000 = 5 đồng tròn nên ra 12.005; nhưng lúc đó test không còn kiểm làm tròn ở điểm giữa nữa. Widget test cũng đỏ vì kỳ vọng "25.500 VND".</p>
<p><b>2.</b> (a) là <b>hot reload</b>: đổi chữ trong <code>build</code> được nạp ngay. (b) là <b>hot restart</b>: <code>main()</code> không chạy lại khi hot reload.</p>
<p><b>3.</b> Trả 0 làm một yêu cầu sai trông như một chuyến miễn phí hợp lệ, và lỗi bị giấu đi. <code>ArgumentError</code> báo rõ hợp đồng bị vi phạm để nơi gọi xử lý.</p>
</details>

Bài trước: chưa có, đây là bài mở đầu. Bài tiếp: [bài 02](/articles/dart-bien-kieu-du-lieu-null-safety-tinh-cuoc) đi vào biến, kiểu số, `final`/`const` và null safety của Dart.

## Tài liệu gốc

- [Cài đặt Flutter](https://docs.flutter.dev/get-started/install).
- [Kiến trúc tổng quan của Flutter](https://docs.flutter.dev/resources/architectural-overview).
- [Hot reload](https://docs.flutter.dev/tools/hot-reload).
- [Các chế độ build của Flutter](https://docs.flutter.dev/testing/build-modes).
- [Impeller](https://docs.flutter.dev/perf/impeller).
- [Giới thiệu về widget test](https://docs.flutter.dev/cookbook/testing/widget/introduction).
- [Ngôn ngữ Dart](https://dart.dev/language).
- [Tổng quan về biên dịch của Dart](https://dart.dev/overview#platform).
- [Gói flutter_lints](https://pub.dev/packages/flutter_lints).

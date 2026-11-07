---
slug: flutter-platform-channel-pigeon-ffi-goi-native
title: "Platform channel, Pigeon và FFI: gọi native khi plugin không đủ"
summary: "Bài 40: app tài xế đọc pin và chế độ tiết kiệm pin để giãn tần suất gửi vị trí. So sánh plugin có sẵn, MethodChannel, Pigeon và dart:ffi (DynamicLibrary, build hook), luồng platform thread, PlatformException, MissingPluginException, test bằng messenger giả. Phần native chưa chạy trên thiết bị."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, platform-channel, pigeon, ffi, android, ios]
language: vi
---

Tài xế Tới Nơi chạy ca tối. Máy còn 18% pin và đã bật chế độ tiết kiệm pin, nhưng app vẫn gửi vị trí mỗi 5 giây như lúc pin đầy. Đội muốn app đọc pin rồi giãn khoảng cách gửi ra 10 hoặc 30 giây (các mốc là số minh hoạ, không phải chính sách thật). Dart không có API đọc pin, nên app phải hỏi hệ điều hành.

Với riêng pin, ngoài đời ta nên dùng plugin `battery_plus` trên pub.dev: trang của nó có mức pin, trạng thái sạc và chế độ tiết kiệm pin. Bài này vẫn tự làm để thấy cơ chế bên dưới mọi plugin, và để trả lời khi nào plugin không còn đủ. Đây là bài 40 của lộ trình **Flutter qua Tới Nơi**. Nên đọc [bài 39 về thanh toán qua VíXanh](/articles/flutter-thanh-toan-vi-xanh-webview-deep-link-idempotency), nơi app đã chạm vào ranh giới với hệ điều hành bằng WebView và deep link, cùng [bài 38 về tác vụ nền](/articles/flutter-background-work-workmanager-vi-tri-nen-app-tai-xe) cho lý do gửi vị trí tốn pin và [bài 21 về Clean Architecture](/articles/flutter-clean-architecture-domain-data-presentation-mobile) cho interface ở ranh giới.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Điều bài này chưa kiểm chứng</span>
<p>Máy viết bài không có Android SDK và Xcode đầy đủ. Mã Kotlin và Swift trong <code>native-config/</code> <strong>chưa được biên dịch và chưa chạy trên thiết bị nào</strong>. Phần chạy được là Dart (với messenger giả) và hàm C (biên dịch bằng <code>cc</code> của máy).</p>
</div>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong <code>flutter test</code>, ta gọi <code>invokeMethod('getBatteryStatus')</code> trên một <code>MethodChannel</code> mà không đăng ký handler giả nào. Kết quả là <code>null</code>, một giá trị mặc định, hay một lỗi? Nếu là lỗi thì loại gì?</p>
<details><summary>Xem đáp án</summary><p>Lỗi, loại <code>MissingPluginException</code>. Phía native không trả lời nên framework nhận <code>null</code>, và <code>MethodChannel.invokeMethod</code> coi <code>null</code> là "không ai cài đặt method này" (đọc <code>_invokeMethod</code> trong <code>platform_channel.dart</code>; test của lab xác nhận). Chỉ <code>OptionalMethodChannel</code> trả <code>null</code> thay vì ném.</p></details>
</div>

## 1. Bốn cách gọi native và cách chọn

Dart không thấy API của iOS hay Android. Có bốn đường sang native (Hình 1), và mỗi bước sang phải thêm một thứ để bảo trì, nên ta chọn cách nhẹ nhất đủ dùng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Bốn cách gọi native xếp từ trái sang phải: plugin có sẵn, MethodChannel, Pigeon, dart:ffi. Mũi tên chỉ hướng thử tiếp khi cách trước chưa đủ. Một mũi tên bên dưới cho thấy chi phí bảo trì tăng dần theo hướng đó.">
<defs><marker id="fl40-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl40-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="15" y="28" class="t-small">Thử từ trái sang phải, dừng ở cách đầu tiên đủ dùng</text>
<rect x="15" y="48" width="190" height="130" rx="10" class="f-pk-soft s-pk"/><text x="31" y="82" class="t-b">Plugin có sẵn</text><text x="31" y="112" class="t-mono">battery_plus</text><text x="31" y="136" class="t-small">native do người khác</text><text x="31" y="158" class="t-small">duy trì</text>
<rect x="235" y="48" width="190" height="130" rx="10" class="f-pk-soft s-pk"/><text x="251" y="82" class="t-b">MethodChannel</text><text x="251" y="112" class="t-small">vài method,</text><text x="251" y="136" class="t-small">dữ liệu đơn giản,</text><text x="251" y="158" class="t-small">ta viết cả hai phía</text>
<rect x="455" y="48" width="190" height="130" rx="10" class="f-pk-soft s-pk"/><text x="471" y="82" class="t-b">Pigeon</text><text x="471" y="112" class="t-small">nhiều method và kiểu,</text><text x="471" y="136" class="t-small">hợp đồng sinh từ</text><text x="471" y="158" class="t-small">một file Dart</text>
<rect x="675" y="48" width="190" height="130" rx="10" class="f-warn-soft s-warn"/><text x="691" y="82" class="t-b">dart:ffi</text><text x="691" y="112" class="t-small">thư viện C có sẵn,</text><text x="691" y="136" class="t-small">gọi đồng bộ, không</text><text x="691" y="158" class="t-small">qua message</text>
<path d="M207 113 L233 113" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-1-a)"/>
<path d="M427 113 L453 113" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-1-a)"/>
<path d="M647 113 L673 113" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-1-a)"/>
<text x="15" y="212" class="t-small">nâng phiên bản plugin</text><text x="235" y="212" class="t-small">khớp tên ở ba nơi</text><text x="455" y="212" class="t-small">chạy lại generator</text><text x="675" y="212" class="t-small">build theo từng ABI</text>
<path d="M15 250 L860 250" class="f-none s-warn" stroke-width="2" marker-end="url(#fl40-1-w)"/><text x="15" y="274" class="t-small t-warn">chi phí bảo trì tăng dần</text>
</svg>
</div>
<figcaption>Hình 1. Chọn cách nhẹ nhất đủ dùng. Mỗi bước sang phải thêm một thứ phải bảo trì: phiên bản plugin, tên khớp tay, file sinh, hoặc mã C build cho từng nền tảng.</figcaption>
</figure>

| Cách | Dùng khi | Native do ai viết | Chi phí bảo trì |
|---|---|---|---|
| Plugin có sẵn | Việc phổ biến: pin, vị trí, camera | Người duy trì plugin | Theo dõi phiên bản, rủi ro plugin bị bỏ rơi |
| `MethodChannel` | Một đến ba method, dữ liệu đơn giản | Ta | Tên channel, tên method, khoá `Map` phải khớp tay ở ba nơi |
| Pigeon | Nhiều method, kiểu dữ liệu riêng, hai chiều | Ta viết xử lý, hợp đồng được sinh | Chạy lại generator, commit file sinh, nâng phiên bản pigeon |
| `dart:ffi` | Thư viện C/C++ có sẵn, hoặc tính toán gọi đồng bộ | Ta hoặc bên thứ ba (C) | Build cho từng hệ điều hành và kiến trúc, lỗi bộ nhớ làm sập cả app |

Khi nào viết plugin riêng? Khi app khách và app tài xế (hoặc nhiều nhóm) cùng cần một phần native. Nếu chỉ một app dùng, để mã native trong app và giấu nó sau một interface như `BatteryGateway`: phần còn lại của app không biết bên dưới là channel, Pigeon hay plugin.

### battery_gateway.dart · 40-platform-channel-pigeon/lib/src/battery_gateway.dart (trích)

```dart
/// Ranh giới tới native: phần còn lại của app chỉ biết interface này, không
/// biết bên dưới là MethodChannel hay Pigeon.
abstract interface class BatteryGateway {
  /// Đọc pin ngay bây giờ. Ném [BatteryUnavailableException] khi không đọc được.
  Future<BatteryStatus> read();

  /// Native chủ động báo khi pin hoặc chế độ tiết kiệm pin đổi.
  Stream<BatteryStatus> get changes;

  void dispose();
}

/// Không đọc được pin: máy không có pin (simulator), native báo lỗi, chưa có
/// code native đăng ký, hoặc native trả dữ liệu sai.
final class BatteryUnavailableException implements Exception {
  const BatteryUnavailableException(this.code, [this.message]);

  /// Mã lỗi của native (ví dụ `UNAVAILABLE`), hoặc mã do Dart đặt:
  /// `missing-plugin`, `bad-payload`, `null-reply`.
  final String code;
  final String? message;

  @override
  String toString() => 'BatteryUnavailableException($code, $message)';
}
```

## 2. MethodChannel hai chiều

`MethodChannel` gửi tên method và tham số sang native, rồi nhận một kết quả. Dữ liệu đi qua `StandardMessageCodec`: tài liệu Flutter liệt kê `bool`, `int`, `double`, `String`, các mảng số, `List` và `Map`. Ta gửi một `Map` ba khoá và tự kiểm khi nhận. Chiều Dart gọi native nằm trong `read()`; chiều native gọi `onBatteryChanged` nằm ở `setMethodCallHandler`:

### method_channel_battery_gateway.dart · 40-platform-channel-pigeon/lib/src/method_channel_battery_gateway.dart (trích)

```dart
  Future<BatteryStatus> read() async {
    try {
      // invokeMapMethod ép kiểu bên trong framework và ném TypeError khi
      // native trả String, List hoặc int; nhận Object? rồi tự kiểm.
      final reply = await _channel.invokeMethod<Object?>(
        getBatteryStatusMethod,
      );
      if (reply == null) {
        throw const BatteryUnavailableException('null-reply');
      }
      if (reply is! Map<Object?, Object?>) {
        throw BatteryUnavailableException(
          'bad-payload',
          'Cần Map, nhận ${reply.runtimeType}',
        );
      }
      return BatteryStatus.fromMap(reply);
    } on PlatformException catch (e) {
      throw BatteryUnavailableException(e.code, e.message);
    } on MissingPluginException catch (e) {
      throw BatteryUnavailableException('missing-plugin', e.message);
    } on FormatException catch (e) {
      throw BatteryUnavailableException('bad-payload', e.message);
    }
  }

  Future<Object?> _onNativeCall(MethodCall call) async {
    switch (call.method) {
      case onBatteryChangedMethod:
        final arguments = call.arguments;
        if (arguments is! Map<Object?, Object?>) {
          throw PlatformException(
            code: 'bad-payload',
            message: 'onBatteryChanged cần một Map',
          );
        }
        try {
          _changes.add(BatteryStatus.fromMap(arguments));
        } on FormatException catch (e) {
          throw PlatformException(code: 'bad-payload', message: e.message);
        }
        return null;
      default:
        // Native gọi method Dart chưa có: trả "chưa cài đặt" thay vì lỗi.
        throw MissingPluginException('Dart không có method ${call.method}');
    }
  }
```

Phía Android và iOS tương ứng (mã chưa biên dịch ở máy này):

### MethodChannelBattery.kt · 40-platform-channel-pigeon/native-config/android/app/src/main/kotlin/vn/toinoi/toi_noi_40/MethodChannelBattery.kt (trích)

```kotlin
    // Chạy trên main thread (UI thread), nên đọc nhanh và không chặn.
    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "getBatteryStatus" ->
                try {
                    result.success(reader.read().toMap())
                } catch (e: BatteryUnavailable) {
                    result.error("UNAVAILABLE", e.message, null)
                }
            else -> result.notImplemented()
        }
    }

    /** Native gọi Dart. Phải gọi trên main thread. */
    fun push(status: BatteryStatusMessage) {
        channel.invokeMethod("onBatteryChanged", status.toMap())
    }
```

### MethodChannelBattery.swift · 40-platform-channel-pigeon/native-config/ios/Runner/MethodChannelBattery.swift (trích)

```swift
  private func handle(_ call: FlutterMethodCall, result: FlutterResult) {
    switch call.method {
    case "getBatteryStatus":
      do {
        result(try reader.read().toMap())
      } catch let error as BatteryUnavailable {
        result(FlutterError(code: "UNAVAILABLE", message: error.message, details: nil))
      } catch {
        result(FlutterError(code: "UNKNOWN", message: "\(error)", details: nil))
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  /// Native gọi Dart. Phải gọi trên main thread.
  func push(_ status: BatteryStatusMessage) {
    channel.invokeMethod("onBatteryChanged", arguments: status.toMap())
  }
```

Hai điểm cần để ý. Một, bảng kiểu của tài liệu Flutter tách `int` 32 bit và 64 bit thành hai dòng; Pigeon tránh rắc rối này bằng cách ghi mọi `int` thành 64 bit (đọc `_PigeonCodec` trong file sinh). Hai, iOS Simulator không có pin (`batteryState` là `.unknown`), nên native trả `UNAVAILABLE`; tài liệu Flutter nêu đúng trường hợp này và dặn bọc lời gọi bằng `try/catch`.

Khi native chỉ đẩy dữ liệu liên tục, có `EventChannel`: native nhận `listen` rồi `cancel`, nên chỉ giữ receiver khi có người nghe. Lab có `batteryEvents()` phía Dart kèm test; phía native của nó chưa được viết.

## 3. Luồng, lỗi và Future

Lời gọi qua channel là bất đồng bộ phía Dart (Hình 2). Theo tài liệu Flutter, handler phía native mặc định chạy trên **platform thread**: main thread trên iOS, UI thread trên Android. Việc nặng trong handler chặn cả giao diện của hệ điều hành. Muốn chạy handler ở thread nền, dùng Task Queue (`makeBackgroundTaskQueue`).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Hai làn. Làn trên là isolate gốc của Dart, gồm await invokeMethod, setMethodCallHandler và lời gọi hàm C. Làn dưới là platform thread của iOS và Android, gồm handler Kotlin hoặc Swift và receiver hoặc observer. Mũi tên 1 đi xuống gửi message, mũi tên 2 đi lên trả lời, mũi tên 3 đi lên khi native gọi Dart. Lời gọi hàm C nằm trong làn Dart, đồng bộ.">
<defs><marker id="fl40-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="10" y="10" width="860" height="140" rx="10" class="f-pk-soft s-pk"/><text x="28" y="38" class="t-b">Isolate gốc của Dart</text>
<rect x="40" y="62" width="200" height="70" rx="8" class="f-surface s-line"/><text x="56" y="92" class="t-mono">await invokeMethod</text><text x="56" y="116" class="t-small">Future, không chặn</text>
<rect x="300" y="62" width="220" height="70" rx="8" class="f-surface s-line"/><text x="316" y="92" class="t-mono">setMethodCallHandler</text><text x="316" y="116" class="t-small">nhận lời gọi từ native</text>
<rect x="600" y="62" width="250" height="70" rx="8" class="f-warn-soft s-warn"/><text x="616" y="92" class="t-b">asFunction: gọi hàm C</text><text x="616" y="116" class="t-small">đồng bộ, chặn isolate này</text>
<rect x="10" y="200" width="860" height="130" rx="10" class="f-phys-soft s-phys"/><text x="28" y="322" class="t-b">Platform thread (main thread iOS, UI thread Android)</text>
<rect x="40" y="245" width="240" height="56" rx="8" class="f-surface s-line"/><text x="56" y="270" class="t-b">handler Kotlin hoặc Swift</text><text x="56" y="290" class="t-small">mặc định chạy ở đây</text>
<rect x="320" y="245" width="240" height="56" rx="8" class="f-surface s-line"/><text x="336" y="270" class="t-b">receiver, observer</text><text x="336" y="290" class="t-small">pin đổi, gọi lên Dart</text>
<text x="600" y="270" class="t-small">Việc nặng ở đây chặn giao diện</text><text x="600" y="290" class="t-small">của hệ điều hành: dùng Task Queue</text>
<path d="M100 134 L100 243" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-2-a)"/><text x="110" y="184" class="t-small">1. gửi message</text>
<path d="M220 243 L220 134" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-2-a)"/><text x="230" y="184" class="t-small">2. trả lời</text>
<path d="M440 243 L440 134" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-2-a)"/><text x="450" y="184" class="t-small">3. native gọi Dart</text>
</svg>
</div>
<figcaption>Hình 2. Lời gọi channel là bất đồng bộ phía Dart; handler native chạy trên platform thread, và native gọi Dart cũng phải từ thread đó. Lời gọi FFI không qua channel: nó chạy đồng bộ trong chính isolate đang gọi.</figcaption>
</figure>

Chiều ngược lại: native gọi Dart phải gọi từ platform thread. Trong lab, receiver Android đăng ký không kèm `Handler` nên `onReceive` chạy trên main thread, còn observer iOS đặt `queue: .main`. `dart:ffi` đi đường khác: `asFunction` trả `int`, không phải `Future`, nên Dart đứng chờ hàm C xong; hàm C chạy lâu sẽ làm đứng isolate đang gọi.

Ba loại lỗi cần phân biệt, và gateway đổi cả ba thành một kiểu `BatteryUnavailableException` có `code`:

| Tình huống | Dart thấy | `code` trong gateway |
|---|---|---|
| Native gọi `result.error(...)` hoặc ném `FlutterError`/`PigeonError` | `PlatformException` | mã của native, ví dụ `UNAVAILABLE` |
| Chưa có handler phía native, hoặc method lạ | `MissingPluginException` (MethodChannel); `PlatformException` mã `channel-error` (Pigeon) | `missing-plugin` hoặc `channel-error` |
| Native trả sai hình dạng (không phải `Map`) hoặc mức pin ngoài 0..100 | Không lỗi nào từ framework nếu ta nhận `Object?`; ta tự kiểm và ném `FormatException` | `bad-payload` |

`MissingPluginException` nói app thiếu mã native, tức lỗi cài đặt; lab chỉ hạ mức (gửi mỗi 10 giây, không đoán bừa 5 hay 30) chứ không nuốt nó im lặng.

Isolate mới không dùng được channel ngay. Tài liệu Flutter yêu cầu isolate là isolate gốc hoặc đã gọi `BackgroundIsolateBinaryMessenger.ensureInitialized(token)`; ta thử gọi trong `Isolate.run` và nhận `StateError` nói đúng điều đó. Bài 41 dùng `Isolate.run`, nên cần nhớ.

## 4. Kiểm thử channel bằng messenger giả

`flutter_test` cho ta đóng vai native. `setMockMethodCallHandler` bắt các lời gọi Dart gửi đi, còn `handlePlatformMessage` giả vờ native gọi lên Dart và trả về phản hồi để ta đọc:

### method_channel_gateway_test.dart · 40-platform-channel-pigeon/test/method_channel_gateway_test.dart (trích)

```dart
/// Giả vờ là native: gọi method `onBatteryChanged` lên Dart, trả về phần
/// native nhận được làm phản hồi.
Future<ByteData?> _nativeCall(String method, Object? arguments) =>
    _messenger.handlePlatformMessage(
      batteryChannelName,
      _codec.encodeMethodCall(MethodCall(method, arguments)),
      null,
    );
```

```dart
  late MethodChannelBatteryGateway gateway;
  setUp(() => gateway = MethodChannelBatteryGateway());
  tearDown(() {
    gateway.dispose();
    // Handler giả không tự mất sau mỗi test (đã thử với Flutter 3.47.4, kể cả
    // trong testWidgets), nên test sau sẽ thấy handler của test trước.
    _messenger.setMockMethodCallHandler(_channel, null);
    _messenger.setMockMessageHandler(batteryEventsChannelName, null);
  });

  group('Dart gọi native', () {
    test('read gửi đúng tên method và không kèm tham số', () async {
      final calls = <MethodCall>[];
      _messenger.setMockMethodCallHandler(_channel, (call) async {
        calls.add(call);
        return _okPayload;
      });

      final status = await gateway.read();

      expect(calls, hasLength(1));
      expect(calls.single.method, 'getBatteryStatus');
      expect(calls.single.arguments, isNull);
      expect(
        status,
        const BatteryStatus(
          levelPercent: 64,
          isCharging: false,
          isPowerSaveMode: true,
        ),
      );
    });

    test('native trả lỗi: PlatformException thành BatteryUnavailable', () {
      _messenger.setMockMethodCallHandler(_channel, (call) async {
        throw PlatformException(
          code: 'UNAVAILABLE',
          message: 'Battery level not available.',
        );
      });

      expect(
        gateway.read(),
        throwsA(
          isA<BatteryUnavailableException>()
              .having((e) => e.code, 'code', 'UNAVAILABLE')
              .having((e) => e.message, 'message', contains('not available')),
        ),
      );
    });
```

Tài liệu API của `setMockMethodCallHandler` ghi handler giả được xoá sau mỗi test. Trên Flutter 3.47.4 ta thấy khác: handler đặt ở test này vẫn còn ở test sau, kể cả trong `testWidgets` (ta thử hai test liên tiếp). Vì vậy `tearDown` ở trên tự gỡ handler; quên bước này, test "chưa ai đăng ký" sẽ thấy handler của test trước.

Test cũng không chứng minh được Kotlin hay Swift dùng đúng tên channel. Vì thế `native_config_test.dart` đọc file native và so các chuỗi: tên channel, tên method, mã `UNAVAILABLE`, khoá của `Map`, cách đăng ký trên `binaryMessenger`.

## 5. Pigeon: một định nghĩa, ba ngôn ngữ

Với `MethodChannel`, hợp đồng nằm rải ở ba file. Pigeon dồn nó về một file Dart và sinh phần còn lại (Hình 3):

### battery.dart · 40-platform-channel-pigeon/pigeons/battery.dart

```dart
import 'package:pigeon/pigeon.dart';

@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/src/generated/battery_api.g.dart',
    dartPackageName: 'toi_noi_40',
    kotlinOut:
        'native-config/android/app/src/main/kotlin/vn/toinoi/toi_noi_40/BatteryApi.g.kt',
    kotlinOptions: KotlinOptions(package: 'vn.toinoi.toi_noi_40'),
    swiftOut: 'native-config/ios/Runner/BatteryApi.g.swift',
  ),
)
/// Ảnh chụp trạng thái pin tại một thời điểm.
class BatteryStatusMessage {
  BatteryStatusMessage({
    required this.levelPercent,
    required this.isCharging,
    required this.isPowerSaveMode,
  });

  /// 0..100.
  int levelPercent;
  bool isCharging;

  /// Android: chế độ tiết kiệm pin; iOS: Low Power Mode.
  bool isPowerSaveMode;
}

/// Dart gọi sang native.
@HostApi()
abstract class BatteryHostApi {
  BatteryStatusMessage getBatteryStatus();
}

/// Native gọi sang Dart.
@FlutterApi()
abstract class BatteryFlutterApi {
  @asyncCallback
  void onBatteryChanged(BatteryStatusMessage status);
}
```

Chạy `dart run pigeon --input pigeons/battery.dart` (pigeon 29.1.0, thêm làm `dev_dependency`; lệnh chạy được trên máy này). Ba file được sinh ra và commit để reviewer thấy diff khi hợp đồng đổi. Đừng sửa tay file `.g.*`: đổi `battery.dart` rồi sinh lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. File pigeons/battery.dart là định nghĩa duy nhất. Lệnh dart run pigeon đọc nó và sinh ba file: battery_api.g.dart được test bằng messenger giả ở máy này, BatteryApi.g.kt và BatteryApi.g.swift chưa được biên dịch ở máy này.">
<defs><marker id="fl40-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="15" y="100" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="31" y="136" class="t-mono">battery.dart</text><text x="31" y="162" class="t-small">định nghĩa duy nhất</text>
<rect x="290" y="110" width="170" height="70" rx="10" class="f-surface s-line"/><text x="306" y="140" class="t-mono">dart run pigeon</text><text x="306" y="162" class="t-small">generator</text>
<rect x="525" y="20" width="340" height="80" rx="10" class="f-pk-soft s-pk"/><text x="541" y="52" class="t-mono">battery_api.g.dart</text><text x="541" y="78" class="t-small">test được ở đây</text>
<rect x="525" y="125" width="340" height="80" rx="10" class="f-warn-soft s-warn"/><text x="541" y="157" class="t-mono">BatteryApi.g.kt</text><text x="541" y="183" class="t-small">chưa biên dịch ở đây</text>
<rect x="525" y="230" width="340" height="80" rx="10" class="f-warn-soft s-warn"/><text x="541" y="262" class="t-mono">BatteryApi.g.swift</text><text x="541" y="288" class="t-small">chưa biên dịch ở đây</text>
<path d="M227 145 L288 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-3-a)"/>
<path d="M462 135 L523 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-3-a)"/>
<path d="M462 145 L523 165" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-3-a)"/>
<path d="M462 155 L523 265" class="f-none s-pk" stroke-width="2" marker-end="url(#fl40-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Tên channel, codec và kiểu dữ liệu nằm trong ba file sinh từ cùng một định nghĩa, nên hai phía không lệch nhau về hợp đồng. Chỉ phần Dart được chạy thử ở máy viết bài.</figcaption>
</figure>

Hai lựa chọn đáng ghi lại. `getBatteryStatus` là method đồng bộ phía native; Pigeon cho phép viết vậy dù phía Dart vẫn là `Future`. `onBatteryChanged` mang `@asyncCallback`: khi thử, file Kotlin sinh ra không có hàm `suspend` và không import `kotlinx.coroutines`, nên app khỏi thêm dependency đó (đổi lại là callback thay cho `await`).

Kiểu dữ liệu giờ là thật: `levelPercent` là `int`, không còn `Map` khoá chuỗi. Nhưng kiểu đúng chưa là giá trị đúng: gateway vẫn kiểm 0..100, và test có ca native trả 250. Lỗi thiếu native cũng khác: Pigeon báo `PlatformException` mã `channel-error`, không phải `MissingPluginException` (test chứng minh). Test dùng cùng messenger giả, với codec do Pigeon sinh:

### pigeon_gateway_test.dart · 40-platform-channel-pigeon/test/pigeon_gateway_test.dart (trích)

```dart
/// Giả native cho HostApi: nhận danh sách tham số, trả danh sách phản hồi theo
/// đúng hình dạng Pigeon dùng (`[kết quả]` hoặc `[code, message, details]`).
void _fakeHost(Object? Function(List<Object?> args) reply) {
  _messenger.setMockDecodedMessageHandler<Object?>(
    const BasicMessageChannel<Object?>(_hostChannel, _codec),
    (message) async => reply((message as List<Object?>?) ?? <Object?>[]),
  );
}
```

Tuỳ chọn `dartTestOut` (sinh lớp giả cho test) bị đánh dấu deprecated ở 29.1.0, nên lab tự giả messenger. README của pigeon còn mô tả Native Interop (qua FFI và JNI) và ghi là thử nghiệm; bài này không dùng.

## 6. dart:ffi: gọi hàm C `toi_noi_round_fare`

Giả sử app tài xế thu tiền mặt và cước cần làm tròn tới 500 hoặc 1.000 đồng bằng một thư viện C có sẵn của đội. Hàm C trả -1 khi đầu vào sai:

### toi_noi_round_fare.c · 40-platform-channel-pigeon/native/src/toi_noi_round_fare.c

```c
#include "toi_noi_round_fare.h"

int64_t toi_noi_round_fare(int64_t vnd, int32_t step) {
  if (vnd < 0 || step <= 0) {
    return -1;
  }
  const int64_t s = (int64_t)step;
  const int64_t rest = vnd % s;
  const int64_t down = vnd - rest;
  /* rest * 2 không tràn: rest < s <= INT32_MAX. */
  if (rest * 2 < s) {
    return down;
  }
  if (down > INT64_MAX - s) {
    return -1;
  }
  return down + s;
}
```

Cách thứ nhất là `DynamicLibrary`: nạp thư viện đã có, `lookup` symbol, `asFunction` thành hàm Dart. Ta tự lo biên dịch và đặt file đúng chỗ:

### fare_rounder.dart · 40-platform-channel-pigeon/lib/src/fare_rounder.dart (trích)

```dart
typedef _RoundFareNative = Int64 Function(Int64 vnd, Int32 step);
typedef _RoundFareDart = int Function(int vnd, int step);

/// Gọi `toi_noi_round_fare` trong thư viện C đã nạp bằng [DynamicLibrary].
final class FfiFareRounder implements FareRounder {
  /// Ném [ArgumentError] nếu [library] không có symbol `toi_noi_round_fare`.
  FfiFareRounder(DynamicLibrary library)
    : _round = library
          .lookup<NativeFunction<_RoundFareNative>>('toi_noi_round_fare')
          .asFunction<_RoundFareDart>(isLeaf: true);

  final _RoundFareDart _round;

  /// Gọi thẳng hàm C, không kiểm gì: chỉ để test chứng minh việc Dart không
  /// tự kiểm kiểu số nguyên khi truyền qua FFI.
  int callRawForTest(int vnd, int step) => _round(vnd, step);

  @override
  Money round(Money fare, {int step = 1000}) {
    // `Int32` ở phía C: số lớn hơn bị cắt bit chứ không báo lỗi, nên Dart
    // phải kiểm trước khi gọi.
    _checkStep(step);
    final result = _round(fare.vnd, step);
    if (result < 0) {
      throw ArgumentError.value(fare, 'fare', 'native từ chối đầu vào');
    }
    return Money(result);
  }
}
```

Phía C là `Int32`, nhưng Dart truyền `int` 64 bit và FFI không báo lỗi khi số quá lớn: test của lab cho thấy `step = 4294967297` (2³² + 1) bị cắt thành 1. Vì vậy lớp bọc phải kiểm miền giá trị trước khi gọi. Ta cũng đặt `isLeaf: true`: theo tài liệu `asFunction`, hàm leaf phải nhỏ, chạy ngắn, không chặn và không gọi ngược vào Dart, đổi lại lời gọi nhanh hơn; hàm làm tròn của ta đúng như vậy.

Cách thứ hai là cách hiện tại để đóng gói mã C: **build hook**. Trang hooks của dart.dev nói build hook có từ Dart 3.10 và chạy tự động với lệnh `run`, `build`, `test`; tài liệu Flutter coi template `package_ffi` là cách khuyến nghị để build và đóng gói mã native từ Flutter 3.38. Hook là script Dart `hook/build.dart`, biên dịch C và khai báo một *code asset*:

### build.dart · 40-platform-channel-pigeon/native/hook/build.dart

```dart
import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';
import 'package:native_toolchain_c/native_toolchain_c.dart';

// Hook chạy tự động khi `dart test`, `dart run`, `dart build` (theo dart.dev) và
// `flutter test` (đã thử trên macOS). Nó biên dịch file C bằng compiler của máy
// và khai báo kết quả là một code asset tên
// package:toi_noi_round_fare/toi_noi_round_fare.dart.
void main(List<String> args) async {
  await build(args, (input, output) async {
    if (!input.config.buildCodeAssets) return;
    await CBuilder.library(
      name: 'toi_noi_round_fare',
      assetName: 'toi_noi_round_fare.dart',
      sources: ['src/toi_noi_round_fare.c'],
    ).run(input: input, output: output);
  });
}
```

Phía Dart không còn đường dẫn thư viện, chỉ có `@Native`:

### toi_noi_round_fare.dart · 40-platform-channel-pigeon/native/lib/toi_noi_round_fare.dart

```dart
import 'dart:ffi';

/// Làm tròn [vnd] tới bội số gần nhất của [step] (nửa bước làm tròn lên).
///
/// Trả -1 khi [vnd] âm, [step] không dương hoặc kết quả vượt `int64`; lớp bọc
/// ở lab chính đổi -1 thành [ArgumentError].
///
/// Không có đường dẫn thư viện ở đây: assetId mặc định là URI của file này,
/// khớp `assetName` trong `hook/build.dart`. `symbol` là tên hàm trong C;
/// bỏ nó đi thì Dart tìm symbol tên `toiNoiRoundFare` và không thấy.
@Native<Int64 Function(Int64, Int32)>(
  symbol: 'toi_noi_round_fare',
  isLeaf: true,
)
external int toiNoiRoundFare(int vnd, int step);
```

Ta thử: `dart test` trong `native/` chạy hook, biên dịch bằng clang của máy rồi gọi hàm thật; `flutter test` trong một project phụ thuộc gói này cũng chạy hook (thử trên macOS arm64). Về độ chín: `hooks` và `code_assets` do dart.dev xuất bản, còn `native_toolchain_c` do `labs.dart.dev` xuất bản và README của nó ghi trạng thái thử nghiệm. Vì thế hãy ghim phiên bản (lab dùng 0.19.6) và đọc changelog trước khi nâng. Hook ở đây dùng `CBuilder.library`, đủ cho trường hợp chỉ có hook build; trang dart.dev dùng `CLibrary`, lớp bọc hỗ trợ cả hook link.

Test phần C cần compiler. Lab biên dịch bằng `cc` ngay trong test và tách ba trường hợp: không có `cc` thì `skip` có lý do; có `cc` mà thoát khác 0 thì test đỏ, vì mã C sai hay toolchain hỏng đều cần được thấy:

### fare_rounder_test.dart · 40-platform-channel-pigeon/test/fare_rounder_test.dart (trích)

```dart
_Build _buildNativeLibrary(Directory out) {
  if (!Platform.isMacOS && !Platform.isLinux) {
    return _Build.skipped('chỉ kiểm bằng cc trên macOS và Linux');
  }
  final extension = Platform.isMacOS ? 'dylib' : 'so';
  final output = '${out.path}/libtoi_noi_round_fare.$extension';
  final ProcessResult result;
  try {
    result = Process.runSync('cc', [
      Platform.isMacOS ? '-dynamiclib' : '-shared',
      '-fPIC',
      '-O2',
      '-o',
      output,
      'native/src/toi_noi_round_fare.c',
    ]);
  } on ProcessException {
    // Không có trình biên dịch: bỏ qua có lý do, không làm đỏ CI.
    return _Build.skipped('máy không có trình biên dịch C (cc)');
  }
  if (result.exitCode != 0) {
    // Có cc mà thoát khác 0: mã C sai, hoặc toolchain hỏng (thiếu SDK, chưa
    // chấp nhận giấy phép Xcode). Cả hai đều phải đỏ; stderr cho biết là gì.
    return _Build.failed('cc thoát ${result.exitCode}: ${result.stderr}');
  }
  return _Build.ok(output);
}
```

Ta thử gỡ `cc` khỏi `PATH`: 6 test FFI bị bỏ qua kèm lý do và cả bộ vẫn xanh. Cước trong test lấy từ `calculateFare` (2,001 km là 12.005 VND, 5 km là 25.500 VND, 100 km là 453.000 VND), và 5.000 cặp ngẫu nhiên (hạt giống cố định) so bản C với `DartFareRounder`. Hai bản luôn giống nhau nên khi thiếu thư viện, app có thể lùi về bản Dart (lab chưa có mã chọn tự động).

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi gọi native</span>
<ul>
<li><strong>Sai tên channel hoặc quên đăng ký phía native.</strong> Dấu hiệu: <code>MissingPluginException</code> ngay lần gọi đầu. Nguyên nhân: tên chỉ là chuỗi, sai một ký tự là hai phía không gặp nhau; hoặc quên đăng ký trong <code>configureFlutterEngine</code> hay <code>didInitializeImplicitFlutterEngine</code>. Cách sửa: một hằng cho tên, test đọc file native để so, hoặc dùng Pigeon để tên được sinh.</li>
<li><strong>Gọi channel từ isolate mới.</strong> Dấu hiệu: <code>StateError</code> nói <code>BackgroundIsolateBinaryMessenger.instance</code> chưa hợp lệ. Cách sửa: truyền <code>RootIsolateToken.instance</code> sang isolate và gọi <code>BackgroundIsolateBinaryMessenger.ensureInitialized</code> ở đó.</li>
<li><strong>Native gọi Dart từ thread nền.</strong> Nguyên nhân: tài liệu Flutter yêu cầu gọi về Dart từ platform thread. Cách sửa: chuyển về main thread bằng <code>Handler(Looper.getMainLooper()).post</code> hoặc <code>DispatchQueue.main.async</code> rồi mới gọi.</li>
<li><strong>Truyền số quá miền qua FFI.</strong> Dấu hiệu: kết quả sai mà không có lỗi. Nguyên nhân: tham số <code>Int32</code> bị cắt bit. Cách sửa: kiểm miền trong lớp bọc trước khi gọi, và để hàm C tự từ chối đầu vào xấu.</li>
<li><strong>Tên Dart khác tên symbol C với <code>@Native</code>.</strong> Dấu hiệu: <code>Couldn't resolve native function 'toiNoiRoundFare'</code>. Nguyên nhân: không có <code>symbol</code> thì Dart tìm đúng tên hàm Dart. Cách sửa: <code>@Native(symbol: 'toi_noi_round_fare')</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

**Mức kiểm chứng: trên thiết bị (TB), và máy viết bài không có thiết bị.** Phần Dart và hàm C trên máy host được kiểm tự động. Kotlin và Swift trong `native-config/` **chưa được biên dịch**: chỉ có test Dart đọc file, và `swiftc -parse` kiểm cú pháp Swift. Chưa chạy app trên simulator, emulator hay máy thật, và chưa chạy build hook cho Android hay iOS. Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/40-platform-channel-pigeon
flutter pub get
flutter analyze
flutter test

cd native
dart pub get
dart analyze
dart test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3 trên macOS arm64 (Apple clang 21.0.0): `flutter analyze` 0 issue; `flutter test` 62 test xanh; `dart analyze` và `dart test` trong `native/` xanh (4 test). Các nhóm test chứng minh:

- **MethodChannel và Pigeon**: tham số gửi đi, lỗi native, thiếu native, phản hồi null, dữ liệu sai, cả chiều native gọi Dart.
- **Logic**: khoảng cách gửi vị trí (biên 20% và 50%, tiết kiệm pin thắng đang sạc), bỏ kết quả `read()` cũ hơn sự kiện.
- **FFI**: các ca của `calculateFare`, 5.000 cặp ngẫu nhiên, `-1`, cắt bit `Int32`; tự skip khi không có `cc`.
- **Cấu hình native**: so chuỗi giữa Dart, Kotlin, Swift và ba file Pigeon sinh ra.

Để chạy trên thiết bị, làm theo README của lab. **Chưa làm**: chạy trên thiết bị, `EventChannel` phía native, hook cho Android và iOS, plugin đóng gói riêng.

## Bài tập

1. Thêm trường `temperatureCelsius` (số thực) vào `BatteryStatusMessage`. Cần chạy lệnh gì và file nào đổi?
2. Handler Kotlin của `getBatteryStatus` mất 300 ms vì gọi một API chậm. Làm sao để giao diện hệ điều hành không bị chặn, và Dart có phải đổi gì?
3. App tài xế cần băm một chuỗi 50 MB bằng thư viện C có sẵn. Chọn cách nào trong bốn cách, và vì sao không chọn `MethodChannel`?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Thêm <code>double temperatureCelsius</code> vào <code>pigeons/battery.dart</code> rồi chạy <code>dart run pigeon --input pigeons/battery.dart</code>; ba file <code>.g.*</code> đổi theo. Sau đó sửa <code>_fromMessage</code> và <code>BatteryStatus</code> nếu muốn dùng trường mới, và cập nhật code Kotlin, Swift tạo <code>BatteryStatusMessage</code>: constructor mới đòi giá trị này nên code native cũ không biên dịch, và đó là điều ta muốn.</p>
<p><strong>2.</strong> Đừng làm việc 300 ms trên platform thread: dùng Task Queue (<code>makeBackgroundTaskQueue</code>) cho channel, hoặc chuyển việc sang thread khác rồi quay về platform thread bằng <code>Handler(Looper.getMainLooper()).post</code> (iOS: <code>DispatchQueue.main.async</code>) trước khi gọi <code>result</code>, theo cách tài liệu Flutter mô tả để nhảy luồng. Dart không phải đổi vì <code>read()</code> vốn là <code>Future</code>; chỉ nên cân nhắc timeout phía Dart.</p>
<p><strong>3.</strong> Dùng <code>dart:ffi</code> (với build hook để đóng gói). <code>MethodChannel</code> phải đưa 50 MB qua message và vẫn cần mã native bọc thư viện. Vì lời gọi FFI là đồng bộ, hãy chạy nó trong isolate khác (bài 41) để không đứng giao diện.</p>
</details>

Bài trước: [Thanh toán qua VíXanh: WebView, deep link quay về và idempotency key](/articles/flutter-thanh-toan-vi-xanh-webview-deep-link-idempotency). Bài tiếp: [Hiệu năng: rebuild, const, RepaintBoundary, isolate và DevTools](/articles/flutter-hieu-nang-rebuild-isolate-devtools).

## Tài liệu gốc

- [Writing custom platform-specific code](https://docs.flutter.dev/platform-integration/platform-channels) và [Developing packages and plugins](https://docs.flutter.dev/packages-and-plugins/developing-packages) (docs.flutter.dev).
- [C interop](https://dart.dev/interop/c-interop) và [Hooks](https://dart.dev/tools/hooks) (dart.dev).
- [pigeon](https://pub.dev/packages/pigeon), [battery_plus](https://pub.dev/packages/battery_plus), [hooks](https://pub.dev/packages/hooks) và [native_toolchain_c](https://pub.dev/packages/native_toolchain_c) (pub.dev).
- [MethodChannel](https://api.flutter.dev/flutter/services/MethodChannel-class.html), [PlatformException](https://api.flutter.dev/flutter/services/PlatformException-class.html), [MissingPluginException](https://api.flutter.dev/flutter/services/MissingPluginException-class.html) và [BackgroundIsolateBinaryMessenger](https://api.flutter.dev/flutter/services/BackgroundIsolateBinaryMessenger-class.html) (api.flutter.dev).
- [BatteryManager](https://developer.android.com/reference/android/os/BatteryManager) và [PowerManager](https://developer.android.com/reference/android/os/PowerManager) (developer.android.com).
- [isLowPowerModeEnabled](https://developer.apple.com/documentation/foundation/processinfo/islowpowermodeenabled) (developer.apple.com).

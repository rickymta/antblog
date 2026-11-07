import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_40/toi_noi_40.dart';

const MethodCodec _codec = StandardMethodCodec();
const MethodChannel _channel = MethodChannel(batteryChannelName);

TestDefaultBinaryMessenger get _messenger =>
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

/// Giả vờ là native: gọi method `onBatteryChanged` lên Dart, trả về phần
/// native nhận được làm phản hồi.
Future<ByteData?> _nativeCall(String method, Object? arguments) =>
    _messenger.handlePlatformMessage(
      batteryChannelName,
      _codec.encodeMethodCall(MethodCall(method, arguments)),
      null,
    );

const Map<String, Object> _okPayload = <String, Object>{
  'levelPercent': 64,
  'isCharging': false,
  'isPowerSaveMode': true,
};

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

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

    test('chưa ai đăng ký phía native: MissingPluginException', () async {
      // Không gọi setMockMethodCallHandler: giống chạy trên nền tảng chưa cài
      // code native (desktop, hoặc quên đăng ký trong configureFlutterEngine).
      await expectLater(
        _channel.invokeMethod<Object?>('getBatteryStatus'),
        throwsA(isA<MissingPluginException>()),
      );
      expect(
        gateway.read(),
        throwsA(
          isA<BatteryUnavailableException>().having(
            (e) => e.code,
            'code',
            'missing-plugin',
          ),
        ),
      );
    });

    test('native trả null: null-reply', () {
      _messenger.setMockMethodCallHandler(_channel, (call) async => null);
      expect(
        gateway.read(),
        throwsA(
          isA<BatteryUnavailableException>().having(
            (e) => e.code,
            'code',
            'null-reply',
          ),
        ),
      );
    });

    test('native trả String, List hoặc int thay vì Map: bad-payload', () async {
      for (final Object wrong in <Object>[
        '64%',
        <Object>[64],
        64,
      ]) {
        _messenger.setMockMethodCallHandler(_channel, (call) async => wrong);
        await expectLater(
          gateway.read(),
          throwsA(
            isA<BatteryUnavailableException>()
                .having((e) => e.code, 'code', 'bad-payload')
                .having((e) => e.message, 'message', contains('Cần Map')),
          ),
          reason: 'native trả ${wrong.runtimeType}',
        );
      }
    });

    test('native trả mức pin 150: bad-payload', () {
      _messenger.setMockMethodCallHandler(
        _channel,
        (call) async => <String, Object>{..._okPayload, 'levelPercent': 150},
      );
      expect(
        gateway.read(),
        throwsA(
          isA<BatteryUnavailableException>().having(
            (e) => e.code,
            'code',
            'bad-payload',
          ),
        ),
      );
    });
  });

  group('native gọi Dart', () {
    test('onBatteryChanged đẩy một BatteryStatus vào changes', () async {
      final next = gateway.changes.first;

      final reply = await _nativeCall(onBatteryChangedMethod, _okPayload);

      expect((await next).levelPercent, 64);
      // Phản hồi thành công mang giá trị null.
      expect(_codec.decodeEnvelope(reply!), isNull);
    });

    test(
      'payload sai: native nhận PlatformException, không có sự kiện',
      () async {
        var emitted = 0;
        final sub = gateway.changes.listen((_) => emitted++);

        final reply = await _nativeCall(
          onBatteryChangedMethod,
          <String, Object>{
            'levelPercent': 101,
            'isCharging': false,
            'isPowerSaveMode': false,
          },
        );

        expect(
          () => _codec.decodeEnvelope(reply!),
          throwsA(
            isA<PlatformException>().having(
              (e) => e.code,
              'code',
              'bad-payload',
            ),
          ),
        );
        await sub.cancel();
        expect(emitted, 0);
      },
    );

    test('method lạ: Dart trả "chưa cài đặt" (phản hồi null)', () async {
      expect(await _nativeCall('khongTonTai', null), isNull);
    });

    test('dispose gỡ handler: native gọi lên thì không ai trả lời', () async {
      gateway.dispose();
      expect(await _nativeCall(onBatteryChangedMethod, _okPayload), isNull);
    });
  });

  group('EventChannel', () {
    test(
      'listen và cancel đi qua channel, sự kiện thành BatteryStatus',
      () async {
        final events = <String>[];
        _messenger.setMockStreamHandler(
          const EventChannel(batteryEventsChannelName),
          MockStreamHandler.inline(
            onListen: (arguments, sink) {
              events.add('listen');
              sink.success(_okPayload);
              sink.endOfStream();
            },
            onCancel: (arguments) => events.add('cancel'),
          ),
        );

        final statuses = await batteryEvents().toList();

        expect(statuses.single.levelPercent, 64);
        expect(events.first, 'listen');
      },
    );
  });
}

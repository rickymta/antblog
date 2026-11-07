import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_40/src/generated/battery_api.g.dart';
import 'package:toi_noi_40/toi_noi_40.dart';

const String _hostChannel =
    'dev.flutter.pigeon.toi_noi_40.BatteryHostApi.getBatteryStatus';
const String _flutterChannel =
    'dev.flutter.pigeon.toi_noi_40.BatteryFlutterApi.onBatteryChanged';

const MessageCodec<Object?> _codec = BatteryHostApi.pigeonChannelCodec;

TestDefaultBinaryMessenger get _messenger =>
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

/// Giả native cho HostApi: nhận danh sách tham số, trả danh sách phản hồi theo
/// đúng hình dạng Pigeon dùng (`[kết quả]` hoặc `[code, message, details]`).
void _fakeHost(Object? Function(List<Object?> args) reply) {
  _messenger.setMockDecodedMessageHandler<Object?>(
    const BasicMessageChannel<Object?>(_hostChannel, _codec),
    (message) async => reply((message as List<Object?>?) ?? <Object?>[]),
  );
}

/// Giả native gọi FlutterApi, trả về phản hồi đã giải mã.
Future<Object?> _nativePush(BatteryStatusMessage status) async {
  final reply = await _messenger.handlePlatformMessage(
    _flutterChannel,
    _codec.encodeMessage(<Object?>[status]),
    null,
  );
  return _codec.decodeMessage(reply);
}

BatteryStatusMessage _message({int level = 35}) => BatteryStatusMessage(
  levelPercent: level,
  isCharging: true,
  isPowerSaveMode: false,
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late PigeonBatteryGateway gateway;
  setUp(() => gateway = PigeonBatteryGateway());
  tearDown(() {
    gateway.dispose();
    _messenger.setMockMessageHandler(_hostChannel, null);
  });

  group('Dart gọi native qua HostApi', () {
    test('kiểu dữ liệu đi và về nguyên vẹn, không phải tự tách Map', () async {
      var sent = <Object?>[];
      _fakeHost((args) {
        sent = args;
        return <Object?>[_message()];
      });

      final status = await gateway.read();

      expect(sent, isEmpty);
      expect(
        status,
        const BatteryStatus(
          levelPercent: 35,
          isCharging: true,
          isPowerSaveMode: false,
        ),
      );
    });

    test('native trả lỗi [code, message, details]: PlatformException', () {
      _fakeHost(
        (_) => <Object?>['UNAVAILABLE', 'Battery level not available.', null],
      );
      expect(
        gateway.read(),
        throwsA(
          isA<BatteryUnavailableException>()
              .having((e) => e.code, 'code', 'UNAVAILABLE')
              .having((e) => e.message, 'message', contains('not available')),
        ),
      );
    });

    test('chưa ai đăng ký phía native: Pigeon báo channel-error, không phải '
        'MissingPluginException', () async {
      await expectLater(
        BatteryHostApi().getBatteryStatus(),
        throwsA(
          isA<PlatformException>().having(
            (e) => e.code,
            'code',
            'channel-error',
          ),
        ),
      );
      expect(
        gateway.read(),
        throwsA(
          isA<BatteryUnavailableException>().having(
            (e) => e.code,
            'code',
            'channel-error',
          ),
        ),
      );
    });

    test('native trả null cho kiểu không null: null-error', () {
      _fakeHost((_) => <Object?>[null]);
      expect(
        BatteryHostApi().getBatteryStatus(),
        throwsA(
          isA<PlatformException>().having((e) => e.code, 'code', 'null-error'),
        ),
      );
    });

    test('mức pin ngoài 0..100 vẫn bị Dart từ chối: kiểu đúng chưa là giá trị đúng', () {
      _fakeHost((_) => <Object?>[_message(level: 250)]);
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

  group('native gọi Dart qua FlutterApi', () {
    test('onBatteryChanged đi vào changes, phản hồi rỗng', () async {
      final next = gateway.changes.first;

      final reply = await _nativePush(_message(level: 18));

      expect((await next).levelPercent, 18);
      expect(reply, isEmpty);
    });

    test(
      'dữ liệu sai: native nhận lỗi code "error", không có sự kiện',
      () async {
        var emitted = 0;
        final sub = gateway.changes.listen((_) => emitted++);

        final reply = await _nativePush(_message(level: 250));

        expect(reply, isA<List<Object?>>());
        expect((reply! as List<Object?>).first, 'error');
        await sub.cancel();
        expect(emitted, 0);
      },
    );

    test('dispose gỡ handler: native gọi lên thì không ai trả lời', () async {
      gateway.dispose();
      final reply = await _messenger.handlePlatformMessage(
        _flutterChannel,
        _codec.encodeMessage(<Object?>[_message()]),
        null,
      );
      expect(reply, isNull);
    });
  });
}

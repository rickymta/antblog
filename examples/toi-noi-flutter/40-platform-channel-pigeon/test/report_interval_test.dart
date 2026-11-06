import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_40/toi_noi_40.dart';

import 'support/fake_battery_gateway.dart';

BatteryStatus _s(int level, {bool charging = false, bool powerSave = false}) =>
    BatteryStatus(
      levelPercent: level,
      isCharging: charging,
      isPowerSaveMode: powerSave,
    );

void main() {
  group('locationReportInterval', () {
    final cases = <String, (BatteryStatus?, Duration)>{
      'pin đầy, không sạc': (_s(100), ReportInterval.normal),
      'đúng 50%': (_s(50), ReportInterval.normal),
      '49%': (_s(49), ReportInterval.reduced),
      'đúng 20%': (_s(20), ReportInterval.reduced),
      '19%': (_s(19), ReportInterval.minimal),
      '0%': (_s(0), ReportInterval.minimal),
      '5% nhưng đang sạc': (_s(5, charging: true), ReportInterval.normal),
      '90% nhưng tiết kiệm pin': (
        _s(90, powerSave: true),
        ReportInterval.minimal,
      ),
      'đang sạc và tiết kiệm pin: tiết kiệm pin thắng': (
        _s(70, charging: true, powerSave: true),
        ReportInterval.minimal,
      ),
      'không đọc được pin': (null, ReportInterval.reduced),
    };
    cases.forEach((name, c) {
      test(name, () => expect(locationReportInterval(c.$1), c.$2));
    });
  });

  group('watchReportInterval', () {
    late FakeBatteryGateway gateway;
    late List<Duration> seen;
    late StreamSubscription<Duration> sub;

    setUp(() {
      gateway = FakeBatteryGateway();
      seen = <Duration>[];
      sub = watchReportInterval(gateway).listen(seen.add);
    });
    tearDown(() async {
      await sub.cancel();
      gateway.dispose();
    });

    test('giá trị đầu từ read, sau đó theo changes, bỏ giá trị lặp', () async {
      gateway.completeRead(_s(80));
      await settle();
      gateway.emit(_s(30));
      gateway.emit(_s(25)); // vẫn reduced: không phát lại
      gateway.emit(_s(10));
      await settle();

      expect(seen, [
        ReportInterval.normal,
        ReportInterval.reduced,
        ReportInterval.minimal,
      ]);
    });

    test('read lỗi: dùng mốc giữa rồi vẫn theo changes', () async {
      gateway.failRead();
      await settle();
      gateway.emit(_s(90));
      await settle();

      expect(seen, [ReportInterval.reduced, ReportInterval.normal]);
    });

    test(
      'native báo đổi trước khi read trả về: kết quả read cũ bị bỏ',
      () async {
        gateway.emit(_s(10, powerSave: true));
        gateway.completeRead(_s(100)); // ảnh chụp cũ hơn sự kiện vừa tới
        await settle();

        expect(seen, [ReportInterval.minimal]);
      },
    );

    test('huỷ subscription thì ngừng nghe gateway', () async {
      gateway.completeRead(_s(80));
      await settle();
      expect(gateway.hasListener, isTrue);

      await sub.cancel();

      expect(gateway.hasListener, isFalse);
    });
  });
}

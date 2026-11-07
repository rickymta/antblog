import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/app_version.dart';
import 'package:toi_noi_59/reminder_throttle.dart';

import 'support.dart';

void main() {
  final v160 = AppVersion.parse('1.6.0+87');
  final v161 = AppVersion.parse('1.6.1+88');
  late MemoryReminderStore store;
  late FakeClock clock;
  late ReminderThrottle throttle;

  setUp(() {
    store = MemoryReminderStore();
    clock = FakeClock(DateTime.utc(2026, 11, 7, 9));
    throttle = ReminderThrottle(store: store, clock: clock.call);
  });

  test('chưa từng nhắc thì nhắc', () async {
    expect(await throttle.shouldRemind(v160), isTrue);
  });

  test('vừa nhắc xong thì im lặng', () async {
    await throttle.markShown(v160);
    expect(await throttle.shouldRemind(v160), isFalse);
  });

  test('ranh giới: thiếu 1 giây là im lặng, đủ 3 ngày là nhắc', () async {
    await throttle.markShown(v160);
    clock.advance(const Duration(days: 3) - const Duration(seconds: 1));
    expect(await throttle.shouldRemind(v160), isFalse);
    clock.advance(const Duration(seconds: 1));
    expect(await throttle.shouldRemind(v160), isTrue);
  });

  test('server nâng recommended lên bản khác thì nhắc ngay', () async {
    await throttle.markShown(v160);
    expect(await throttle.shouldRemind(v161), isTrue);
  });

  test('khoảng cách nhắc cấu hình được', () async {
    final weekly = ReminderThrottle(
      store: store,
      clock: clock.call,
      interval: const Duration(days: 7),
    );
    await weekly.markShown(v160);
    clock.advance(const Duration(days: 6));
    expect(await weekly.shouldRemind(v160), isFalse);
  });

  test(
    'đồng hồ máy lùi về trước lần nhắc cuối thì nhắc, không im vô hạn',
    () async {
      await throttle.markShown(v160);
      clock.advance(const Duration(days: -10));
      expect(await throttle.shouldRemind(v160), isTrue);
    },
  );

  test(
    'không đọc được bộ nhớ thì im lặng thay vì nhắc mỗi lần mở app',
    () async {
      store.failReads = true;
      expect(await throttle.shouldRemind(v160), isFalse);
    },
  );

  test('lỗi ghi không ném ra ngoài', () async {
    store.failWrites = true;
    await throttle.markShown(v160);
    expect(store.last, isNull);
  });

  test('markShown ghi đúng thời điểm và phiên bản', () async {
    await throttle.markShown(v160);
    expect(store.last, (shownAt: clock.now, version: v160));
  });
}

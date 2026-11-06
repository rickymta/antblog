import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/app_version.dart';
import 'package:toi_noi_59/min_version_policy.dart';
import 'package:toi_noi_59/reminder_throttle.dart';
import 'package:toi_noi_59/store_platform.dart';
import 'package:toi_noi_59/update_gate.dart';

import 'support.dart';

Map<String, Object?> policy({
  String min = '1.4.0+60',
  String? rec = '1.6.0+87',
}) => {
  'android': {
    'minSupported': min,
    'recommended': ?rec,
    'latestPublished': '1.6.0+87',
  },
};

void main() {
  late MemoryReminderStore store;
  late FakeClock clock;

  setUp(() {
    store = MemoryReminderStore();
    clock = FakeClock(DateTime.utc(2026, 11, 7, 9));
  });

  UpdateGate gateFor(PolicySource source) => UpdateGate(
    source: source,
    throttle: ReminderThrottle(store: store, clock: clock.call),
  );

  Future<UpdateDecision> run(UpdateGate gate, String current) => gate.check(
    current: AppVersion.parse(current),
    platform: StorePlatform.android,
  );

  group('fail open', () {
    test('server ném lỗi thì cho vào, dù bản đang chạy rất cũ', () async {
      final gate = gateFor(FakePolicySource.throwing(StateError('503')));
      final d = await run(gate, '0.0.1');
      expect((d as UpdateOk).reason, OkReason.serverUnavailable);
    });

    test('lỗi không phải Exception (Error) cũng không làm app hỏng', () async {
      final gate = gateFor(FakePolicySource.throwing(AssertionError('bug')));
      expect(await run(gate, '0.0.1'), isA<UpdateOk>());
    });

    test('JSON hỏng thì cho vào', () async {
      final gate = gateFor(FakePolicySource.value('<html>502</html>'));
      expect(await run(gate, '0.0.1'), isA<UpdateOk>());
    });

    test('cấu hình tự mâu thuẫn (min > latest) thì cho vào', () async {
      final gate = gateFor(
        FakePolicySource.value({
          'android': {'minSupported': '9.0.0', 'latestPublished': '1.6.0'},
        }),
      );
      final d = await run(gate, '1.6.0+87');
      expect((d as UpdateOk).reason, OkReason.serverUnavailable);
    });

    test('hết thời gian chờ đúng ở 3 giây thì cho vào', () {
      fakeAsync((async) {
        final gate = gateFor(FakePolicySource.hanging());
        UpdateDecision? result;
        run(gate, '0.0.1').then((d) => result = d);

        async.elapse(const Duration(milliseconds: 2999));
        expect(result, isNull);
        async.elapse(const Duration(milliseconds: 1));
        expect((result! as UpdateOk).reason, OkReason.serverUnavailable);
      });
    });

    test('chỉ chính sách đọc được và hợp lệ mới có quyền chặn', () async {
      final gate = gateFor(FakePolicySource.value(policy()));
      expect(await run(gate, '1.3.0+10'), isA<UpdateForce>());
    });
  });

  group('giới hạn tần suất nhắc', () {
    test('recommend lần đầu được trả về và ghi nhận', () async {
      final gate = gateFor(FakePolicySource.value(policy()));
      expect(await run(gate, '1.5.0+70'), isA<UpdateRecommend>());
      expect(store.last?.version, AppVersion.parse('1.6.0+87'));
    });

    test('mở app lại trong 3 ngày thì không nhắc nữa', () async {
      final gate = gateFor(FakePolicySource.value(policy()));
      await run(gate, '1.5.0+70');
      clock.advance(const Duration(days: 2));
      final d = await run(gate, '1.5.0+70');
      expect((d as UpdateOk).reason, OkReason.throttled);
    });

    test('qua 3 ngày thì nhắc lại', () async {
      final gate = gateFor(FakePolicySource.value(policy()));
      await run(gate, '1.5.0+70');
      clock.advance(const Duration(days: 3));
      expect(await run(gate, '1.5.0+70'), isA<UpdateRecommend>());
    });

    test('force không bao giờ bị giới hạn', () async {
      final gate = gateFor(FakePolicySource.value(policy()));
      for (var i = 0; i < 3; i++) {
        expect(await run(gate, '1.3.0+10'), isA<UpdateForce>());
      }
    });

    test('force vẫn hiện dù vừa nhắc recommend xong', () async {
      store.last = (shownAt: clock.now, version: AppVersion.parse('1.6.0+87'));
      final gate = gateFor(FakePolicySource.value(policy()));
      expect(await run(gate, '1.3.0+10'), isA<UpdateForce>());
    });

    test('lỗi ghi bộ nhớ vẫn trả recommend', () async {
      store.failWrites = true;
      final gate = gateFor(FakePolicySource.value(policy()));
      expect(await run(gate, '1.5.0+70'), isA<UpdateRecommend>());
    });

    test('bản đã mới thì không chạm vào bộ nhớ nhắc', () async {
      final gate = gateFor(FakePolicySource.value(policy()));
      await run(gate, '1.6.0+87');
      expect(store.last, isNull);
    });
  });

  test('mỗi lần check hỏi server một lần', () async {
    final source = FakePolicySource.value(policy());
    await run(gateFor(source), '1.6.0+87');
    expect(source.calls, 1);
  });
}

import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

final DateTime _t0 = DateTime.utc(2026, 11, 10, 8);

DateTime _at(int days, [int hours = 0]) =>
    _t0.add(Duration(days: days, hours: hours));

void main() {
  group('bảng phần trăm', () {
    test(
      '1, 2, 5, 10, 20, 50, 100 trong 7 ngày (App Store Connect Help, 11/2026)',
      () {
        expect(PhasedReleasePlan.percentByDay, [1, 2, 5, 10, 20, 50, 100]);
        expect(
          [for (var d = 1; d <= 7; d++) PhasedReleasePlan.percentOnDay(d)],
          [
            1, 2, 5, 10, 20, 50, 100, //
          ],
        );
      },
    );

    test('sau ngày 7 vẫn là 100; ngày 0 hoặc âm ném ArgumentError', () {
      expect(PhasedReleasePlan.percentOnDay(8), 100);
      expect(PhasedReleasePlan.percentOnDay(365), 100);
      expect(() => PhasedReleasePlan.percentOnDay(0), throwsArgumentError);
      expect(() => PhasedReleasePlan.percentOnDay(-1), throwsArgumentError);
    });
  });

  group('ngày theo đồng hồ (mô hình 24 giờ)', () {
    final plan = PhasedReleasePlan(startedAt: _t0);

    test('trước lúc bắt đầu là ngày 0, 0%', () {
      final before = _t0.subtract(const Duration(seconds: 1));
      expect(plan.dayAt(before), 0);
      expect(plan.percentAt(before), 0);
    });

    test(
      'đúng lúc bắt đầu là ngày 1; 23:59:59 vẫn ngày 1; đủ 24 giờ sang ngày 2',
      () {
        expect(plan.dayAt(_t0), 1);
        expect(
          plan.dayAt(_at(0, 23).add(const Duration(minutes: 59, seconds: 59))),
          1,
        );
        expect(plan.dayAt(_at(1)), 2);
      },
    );

    test('biên từng đợt: ngày 3 là 5%, ngày 6 là 50%, ngày 7 là 100%', () {
      expect(plan.percentAt(_at(2)), 5);
      expect(plan.percentAt(_at(5)), 50);
      expect(plan.percentAt(_at(6)), 100);
      expect(plan.percentAt(_at(30)), 100);
    });

    test('startedAt không phải UTC bị chặn bằng assert', () {
      expect(
        () => PhasedReleasePlan(startedAt: DateTime(2026, 11, 10)),
        throwsA(isA<AssertionError>()),
      );
    });
  });

  group('exposedUsers', () {
    final plan = PhasedReleasePlan(startedAt: _t0);

    test(
      '20.000 người bật tự động cập nhật: 200, 400, 1.000, 2.000 (số minh hoạ)',
      () {
        expect(
          [for (var d = 0; d < 4; d++) plan.exposedUsers(20000, _at(d))],
          [
            200, 400, 1000, 2000, //
          ],
        );
      },
    );

    test('làm tròn xuống: 150 người ở 1% là 1 người, 99 người là 0', () {
      expect(plan.exposedUsers(150, _t0), 1);
      expect(plan.exposedUsers(99, _t0), 0);
    });

    test('trước khi bắt đầu là 0; số âm ném ArgumentError', () {
      expect(plan.exposedUsers(1000, _t0.subtract(const Duration(days: 1))), 0);
      expect(() => plan.exposedUsers(-1, _t0), throwsArgumentError);
    });
  });

  group('tạm dừng và tiếp tục', () {
    test('đang tạm dừng thì giữ nguyên ngày tại lúc dừng', () {
      final paused = PhasedReleasePlan(startedAt: _t0).pause(_at(2, 5));
      expect(paused.isPaused, isTrue);
      expect(paused.dayAt(_at(2, 5)), 3);
      expect(paused.dayAt(_at(20)), 3);
      expect(paused.percentAt(_at(20)), 5);
    });

    test('tiếp tục thì chạy tiếp từ ngày đã dừng, không bù thời gian dừng', () {
      final plan = PhasedReleasePlan(startedAt: _t0)
          .pause(_at(1, 12))
          .resume(_at(4, 12));
      // Đã chạy 1,5 ngày trước khi dừng; dừng 3 ngày.
      expect(plan.isPaused, isFalse);
      expect(plan.dayAt(_at(4, 12)), 2);
      expect(plan.dayAt(_at(4, 23)), 2);
      expect(plan.dayAt(_at(5)), 3);
    });

    test('hạn mức dừng là 30 ngày cộng dồn qua nhiều lần', () {
      var plan = PhasedReleasePlan(startedAt: _t0).pause(_at(1));
      expect(plan.pauseBudgetLeft(_at(11)), const Duration(days: 20));
      plan = plan.resume(_at(11)).pause(_at(12));
      expect(plan.pauseBudgetLeft(_at(22)), const Duration(days: 10));
      // Dừng thêm 30 ngày nữa: hạn mức không âm.
      expect(plan.pauseBudgetLeft(_at(42)), Duration.zero);
    });

    test('hết hạn mức thì không dừng thêm được', () {
      var plan = PhasedReleasePlan(startedAt: _t0).pause(_at(1));
      plan = plan.resume(_at(31));
      expect(plan.pauseBudgetLeft(_at(31)), Duration.zero);
      expect(() => plan.pause(_at(32)), throwsStateError);
    });

    test(
      'dừng hai lần liên tiếp hoặc tiếp tục khi không dừng ném StateError',
      () {
        final plan = PhasedReleasePlan(startedAt: _t0);
        expect(() => plan.resume(_at(1)), throwsStateError);
        expect(() => plan.pause(_at(1)).pause(_at(2)), throwsStateError);
      },
    );

    test('các thao tác không sửa kế hoạch gốc', () {
      final plan = PhasedReleasePlan(startedAt: _t0);
      plan.pause(_at(1));
      expect(plan.isPaused, isFalse);
    });
  });

  group('decide', () {
    const plan = RolloutThresholds();

    RolloutAction decide(int sessions, int crashed) => PhasedReleasePlan(
      startedAt: _t0,
    ).decide(RolloutMetrics(sessions: sessions, crashedSessions: crashed));

    test('ít phiên hơn ngưỡng thì chờ, dù crash nhiều', () {
      expect(decide(499, 499), RolloutAction.wait);
      expect(decide(0, 0), RolloutAction.wait);
    });

    test('đúng biên 99,50% phiên không crash thì đi tiếp, thấp hơn một phiên thì dừng', () {
      expect(plan.minCrashFreeBasisPoints, 9950);
      expect(decide(1000, 5), RolloutAction.proceed); // 99,50%
      expect(decide(1000, 6), RolloutAction.pause); // 99,40%
      expect(decide(1000, 0), RolloutAction.proceed);
    });

    test('đúng 500 phiên là đủ mẫu', () {
      expect(decide(500, 0), RolloutAction.proceed);
      expect(decide(500, 3), RolloutAction.pause); // 99,40%
      expect(decide(500, 2), RolloutAction.proceed); // 99,60%
    });

    test('ngưỡng tuỳ chỉnh', () {
      final action = PhasedReleasePlan(startedAt: _t0).decide(
        const RolloutMetrics(sessions: 100, crashedSessions: 1),
        thresholds: const RolloutThresholds(
          minSessions: 100,
          minCrashFreeBasisPoints: 9900,
        ),
      );
      expect(action, RolloutAction.proceed); // đúng 99,00%
      expect(
        PhasedReleasePlan(startedAt: _t0).decide(
          const RolloutMetrics(sessions: 100, crashedSessions: 2),
          thresholds: const RolloutThresholds(
            minSessions: 100,
            minCrashFreeBasisPoints: 9900,
          ),
        ),
        RolloutAction.pause, // 98,00%
      );
    });

    test('số liệu vô lý ném ArgumentError', () {
      expect(() => decide(-1, 0), throwsArgumentError);
      expect(() => decide(10, -1), throwsArgumentError);
      expect(() => decide(10, 11), throwsArgumentError);
    });
  });
}

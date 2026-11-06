import 'package:test/test.dart';
import 'package:toi_noi_57/rollout_plan.dart';

RolloutObservation obs(int users, int bp) =>
    RolloutObservation(users: users, crashFreeUsersBp: bp);

void main() {
  group('RolloutPolicy', () {
    test('mặc định hợp lệ và kết thúc ở 100', () {
      const policy = RolloutPolicy();
      policy.validate();
      expect(policy.steps, [1, 5, 20, 50, 100]);
    });

    test(
      'từ chối bước không tăng ngặt, ngoài miền, hoặc không kết thúc 100',
      () {
        expect(
          () => const RolloutPolicy(steps: [1, 5, 5, 100]).validate(),
          throwsArgumentError,
        );
        expect(
          () => const RolloutPolicy(steps: [0, 50, 100]).validate(),
          throwsArgumentError,
        );
        expect(
          () => const RolloutPolicy(steps: [1, 5, 50]).validate(),
          throwsArgumentError,
        );
        expect(
          () => const RolloutPolicy(steps: <int>[]).validate(),
          throwsArgumentError,
        );
      },
    );

    test('ngưỡng dừng phải thấp hơn ngưỡng tiến, cỡ mẫu phải dương', () {
      expect(
        () => const RolloutPolicy(
          advanceAtBp: 9900,
          haltBelowBp: 9900,
        ).validate(),
        throwsArgumentError,
      );
      expect(
        () => const RolloutPolicy(minUsers: 0).validate(),
        throwsArgumentError,
      );
    });
  });

  group('RolloutObservation', () {
    test('từ chối số âm và crash-free ngoài 0..10000', () {
      expect(() => obs(-1, 9900), throwsArgumentError);
      expect(() => obs(10, -1), throwsArgumentError);
      expect(() => obs(10, 10001), throwsArgumentError);
      expect(obs(0, 0).users, 0);
      expect(obs(10, 10000).crashFreeUsersBp, 10000);
    });
  });

  group('RolloutPlan.start', () {
    test('bản cập nhật bắt đầu ở bước đầu, đang chạy', () {
      final plan = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);
      expect(plan.percent, 1);
      expect(plan.status, RolloutStatus.inProgress);
    });

    test('bản đầu tiên không có staged rollout: 100% và đã xong', () {
      final plan = RolloutPlan.start(versionCode: 1);
      expect(plan.percent, 100);
      expect(plan.status, RolloutStatus.completed);
      expect(() => plan.decide(obs(1000, 9990)), throwsStateError);
    });

    test('luật chỉ có một bước 100 cũng xong ngay', () {
      final plan = RolloutPlan.start(
        versionCode: 5,
        previousVersionCode: 4,
        policy: const RolloutPolicy(steps: [100]),
      );
      expect(plan.status, RolloutStatus.completed);
    });

    test(
      'versionCode phải lớn hơn bản đang chạy (bằng nhau cũng bị từ chối)',
      () {
        expect(
          () => RolloutPlan.start(versionCode: 42, previousVersionCode: 42),
          throwsArgumentError,
        );
        expect(
          () => RolloutPlan.start(versionCode: 41, previousVersionCode: 42),
          throwsArgumentError,
        );
        expect(
          RolloutPlan.start(
            versionCode: 43,
            previousVersionCode: 42,
          ).versionCode,
          43,
        );
      },
    );

    test('versionCode nằm trong 1..2.100.000.000', () {
      expect(() => RolloutPlan.start(versionCode: 0), throwsArgumentError);
      expect(
        () => RolloutPlan.start(versionCode: maxVersionCode + 1),
        throwsArgumentError,
      );
      expect(
        RolloutPlan.start(versionCode: maxVersionCode).versionCode,
        maxVersionCode,
      );
    });
  });

  group('decide', () {
    final plan = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);

    test('đúng ngưỡng tiến thì tiến, thấp hơn một phần vạn thì giữ', () {
      expect(plan.decide(obs(500, 9950)), RolloutAction.advance);
      expect(plan.decide(obs(500, 9949)), RolloutAction.hold);
    });

    test('đúng ngưỡng dừng thì còn giữ, thấp hơn một phần vạn thì dừng', () {
      expect(plan.decide(obs(500, 9900)), RolloutAction.hold);
      expect(plan.decide(obs(500, 9899)), RolloutAction.halt);
    });

    test('thiếu mẫu thì giữ, dù số đo đẹp hay xấu', () {
      expect(plan.decide(obs(499, 10000)), RolloutAction.hold);
      expect(plan.decide(obs(499, 5000)), RolloutAction.hold);
      expect(plan.decide(obs(500, 5000)), RolloutAction.halt);
    });

    test('decide không đổi trạng thái', () {
      plan.decide(obs(900, 9999));
      expect(plan.percent, 1);
    });
  });

  group('observe', () {
    test('đi hết các bước thì xong ở 100', () {
      var plan = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);
      final seen = <int>[plan.percent];
      while (plan.status == RolloutStatus.inProgress) {
        plan = plan.observe(obs(1000, 9990));
        seen.add(plan.percent);
      }
      expect(seen, [1, 5, 20, 50, 100]);
      expect(plan.status, RolloutStatus.completed);
    });

    test('vùng giữ không đổi bước', () {
      var plan = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);
      plan = plan.observe(obs(1000, 9990));
      final held = plan.observe(obs(1000, 9930));
      expect(held.percent, 5);
      expect(identical(held, plan), isTrue);
    });

    test('số đo xấu dừng rollout và giữ nguyên mức đã phát', () {
      var plan = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);
      plan = plan.observe(obs(1000, 9990));
      plan = plan.observe(obs(3000, 9800));
      expect(plan.status, RolloutStatus.halted);
      expect(plan.percent, 5);
    });

    test('đã dừng thì không nhận số đo và không dừng lần hai', () {
      final halted = RolloutPlan.start(
        versionCode: 43,
        previousVersionCode: 42,
      ).halt();
      expect(() => halted.observe(obs(1000, 9990)), throwsStateError);
      expect(() => halted.halt(), throwsStateError);
    });
  });

  group('halt, resume, fixForward', () {
    final base = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);

    test('resume trả về đúng mức cũ và chỉ dùng được khi đang dừng', () {
      final stepped = base.observe(obs(1000, 9990));
      final resumed = stepped.halt().resume();
      expect(resumed.status, RolloutStatus.inProgress);
      expect(resumed.percent, 5);
      expect(() => stepped.resume(), throwsStateError);
    });

    test(
      'bản đã 100% vẫn halt được, giữ percent 100 và resume được ở mức khác',
      () {
        var done = base;
        while (done.status == RolloutStatus.inProgress) {
          done = done.observe(obs(1000, 9990));
        }
        expect(done.status, RolloutStatus.completed);
        final halted = done.halt();
        expect(halted.status, RolloutStatus.halted);
        expect(halted.percent, 100);
        final back = halted.resume(percent: 20);
        expect(back.status, RolloutStatus.inProgress);
        expect(back.percent, 20);
      },
    );

    test('lần phát hành đầu không có bản trước nên không halt được', () {
      expect(() => RolloutPlan.start(versionCode: 1).halt(), throwsStateError);
    });

    test('resume ở mức 100 thì xong; mức ngoài bậc thang bị từ chối', () {
      final halted = base.halt();
      expect(halted.resume(percent: 100).status, RolloutStatus.completed);
      expect(halted.resume(percent: 1).percent, 1);
      expect(() => halted.resume(percent: 7), throwsArgumentError);
      expect(() => halted.resume(percent: 0), throwsArgumentError);
    });

    test(
      'resume không đổi mức khi không truyền percent, và xong nếu đang ở 100',
      () {
        var done = base;
        while (done.status == RolloutStatus.inProgress) {
          done = done.observe(obs(1000, 9990));
        }
        expect(done.halt().resume().status, RolloutStatus.completed);
      },
    );

    test('fixForward mở rollout mới từ bước đầu với versionCode cao hơn', () {
      final stepped = base.observe(obs(1000, 9990)).observe(obs(1000, 9990));
      final fix = stepped.halt().fixForward(newVersionCode: 44);
      expect(fix.versionCode, 44);
      expect(fix.previousVersionCode, 43);
      expect(fix.percent, 1);
      expect(fix.status, RolloutStatus.inProgress);
    });

    test('không có đường về versionCode thấp hơn hoặc bằng', () {
      final halted = base.halt();
      expect(() => halted.fixForward(newVersionCode: 42), throwsArgumentError);
      expect(() => halted.fixForward(newVersionCode: 43), throwsArgumentError);
    });

    test('kế hoạch cũ không đổi sau khi tạo bản mới', () {
      final halted = base.halt();
      halted.fixForward(newVersionCode: 44);
      expect(halted.status, RolloutStatus.halted);
      expect(halted.versionCode, 43);
    });
  });
}

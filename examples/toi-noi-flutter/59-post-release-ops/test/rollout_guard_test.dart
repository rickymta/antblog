import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/rollout_guard.dart';
import 'package:toi_noi_59/store_platform.dart';

RolloutStats stats({
  StorePlatform platform = StorePlatform.android,
  int percent = 10,
  int sessions = 4200,
  int crashFree = 9930,
  int baseline = 9930,
}) => RolloutStats(
  version: '1.6.0',
  platform: platform,
  rolloutPercent: percent,
  sessions: sessions,
  crashFreeBp: crashFree,
  baselineCrashFreeBp: baseline,
);

void main() {
  group('Android', () {
    test('không đổi so với bản trước: tiếp tục', () {
      final d = RolloutGuard.decide(stats());
      expect(d.action, RolloutAction.proceed);
      expect(d.storeAction, StoreAction.continueRollout);
    });

    test('giảm 20 bp (0,20 điểm): vẫn tiếp tục', () {
      expect(
        RolloutGuard.decide(stats(crashFree: 9910)).action,
        RolloutAction.proceed,
      );
    });

    test('giảm đúng 30 bp: giữ nguyên tỷ lệ để điều tra', () {
      final d = RolloutGuard.decide(stats(crashFree: 9900));
      expect(d.action, RolloutAction.pause);
      expect(d.storeAction, StoreAction.holdAndroidPercent);
    });

    test('giảm 99 bp: vẫn mới là pause', () {
      expect(
        RolloutGuard.decide(stats(crashFree: 9831)).action,
        RolloutAction.pause,
      );
    });

    test('giảm đúng 100 bp: halt rollout', () {
      final d = RolloutGuard.decide(stats(crashFree: 9830));
      expect(d.action, RolloutAction.halt);
      expect(d.storeAction, StoreAction.haltAndroidRollout);
    });

    test('số minh hoạ của bài: 98,10% so với 99,30% là halt', () {
      final d = RolloutGuard.decide(stats(crashFree: 9810));
      expect(d.action, RolloutAction.halt);
      expect(d.reason, contains('1,20 điểm %'));
    });

    test('bản tốt hơn bản trước (drop âm) vẫn là tiếp tục', () {
      expect(
        RolloutGuard.decide(stats(crashFree: 9960)).action,
        RolloutAction.proceed,
      );
    });
  });

  group('sàn tuyệt đối', () {
    test('dưới 97,00% là halt dù bản trước cũng tệ', () {
      final d = RolloutGuard.decide(stats(crashFree: 9699, baseline: 9710));
      expect(d.action, RolloutAction.halt);
      expect(d.reason, contains('sàn'));
    });

    test('đúng 97,00% và không giảm so với bản trước thì không halt', () {
      expect(
        RolloutGuard.decide(stats(crashFree: 9700, baseline: 9700)).action,
        RolloutAction.proceed,
      );
    });
  });

  group('chưa đủ dữ liệu', () {
    test('1.999 phiên: số bình thường hoặc rất đẹp thì chờ', () {
      for (final crashFree in [9930, 9810, 10000]) {
        final d = RolloutGuard.decide(
          stats(sessions: 1999, crashFree: crashFree),
        );
        expect(d.action, RolloutAction.wait, reason: '$crashFree');
        expect(d.storeAction, StoreAction.holdAndroidPercent);
      }
    });

    test('mức giảm 1,20 điểm chưa đủ 2.000 phiên thì chưa dám halt', () {
      expect(
        RolloutGuard.decide(stats(sessions: 1999, crashFree: 9810)).action,
        RolloutAction.wait,
      );
    });

    test('dưới sàn 97% thì halt sớm từ 300 phiên, kể cả khi mẫu còn nhỏ', () {
      final d = RolloutGuard.decide(stats(sessions: 300, crashFree: 9000));
      expect(d.action, RolloutAction.halt);
      expect(d.reason, contains('sàn'));
    });

    test('299 phiên: ngay cả số rất xấu vẫn chờ', () {
      expect(
        RolloutGuard.decide(stats(sessions: 299, crashFree: 9000)).action,
        RolloutAction.wait,
      );
    });

    test('đúng 2.000 phiên thì đã quyết định được', () {
      expect(
        RolloutGuard.decide(stats(sessions: 2000, crashFree: 9810)).action,
        RolloutAction.halt,
      );
    });

    test('ngưỡng tuỳ chỉnh được', () {
      final d = RolloutGuard.decide(
        stats(sessions: 500, crashFree: 9810),
        const RolloutThresholds(minSessions: 500),
      );
      expect(d.action, RolloutAction.halt);
    });
  });

  group('iOS không có "giữ nguyên %"', () {
    test('pause và halt đều là tạm dừng phased release', () {
      final pause = RolloutGuard.decide(
        stats(platform: StorePlatform.ios, crashFree: 9900),
      );
      final halt = RolloutGuard.decide(
        stats(platform: StorePlatform.ios, crashFree: 9810),
      );
      expect(pause.storeAction, StoreAction.pauseIosPhasedRelease);
      expect(halt.storeAction, StoreAction.pauseIosPhasedRelease);
      expect(pause.action, RolloutAction.pause);
      expect(halt.action, RolloutAction.halt);
    });

    test(
      'chờ thêm dữ liệu: phased release cứ chạy, không có việc gì để làm',
      () {
        final d = RolloutGuard.decide(
          stats(platform: StorePlatform.ios, sessions: 10),
        );
        expect(d.action, RolloutAction.wait);
        expect(d.storeAction, StoreAction.continueRollout);
      },
    );
  });

  group('thông báo', () {
    test('có mức, bản, nền tảng, tỷ lệ, số phiên và lý do', () {
      final s = stats(crashFree: 9810);
      final text = RolloutGuard.notice(s, RolloutGuard.decide(s));
      expect(text, startsWith('[HALT] Tới Nơi 1.6.0 (Android, 10% người dùng'));
      expect(text, contains('4200 phiên'));
      expect(text, contains('1,20 điểm %'));
    });

    test('iOS ghi đúng tên nền tảng', () {
      final s = stats(platform: StorePlatform.ios);
      expect(RolloutGuard.notice(s, RolloutGuard.decide(s)), contains('(iOS,'));
    });

    test('định dạng tỷ lệ giữ số 0 đứng sau dấu phẩy', () {
      final s = stats(crashFree: 9905, baseline: 9905);
      final d = RolloutGuard.decide(s);
      expect(d.reason, contains('99,05%'));
    });
  });
}

import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_54/build_number.dart';
import 'package:toi_noi_54/cost_model.dart';

import 'support.dart';

void main() {
  group('computeBuildNumber', () {
    test('run_number cộng offset', () {
      expect(computeBuildNumber(runNumber: 1), 1);
      expect(computeBuildNumber(runNumber: 42, offset: 500), 542);
    });

    test('tăng ngặt theo run_number cho mọi offset', () {
      for (final offset in [0, 1, 500, 1000000]) {
        final numbers = [
          for (var run = 1; run <= 200; run++)
            computeBuildNumber(runNumber: run, offset: offset),
        ];
        expect(isStrictlyIncreasing(numbers), isTrue, reason: 'offset $offset');
      }
    });

    test(
      'chạy lại (cùng run_number) cho cùng số: không giảm nhưng không tăng',
      () {
        // Lịch sử: run 5 chạy hai lần (re-run), rồi run 6.
        final history = [
          for (final run in [4, 5, 5, 6])
            computeBuildNumber(runNumber: run, offset: 10),
        ];
        expect(isNonDecreasing(history), isTrue);
        expect(isStrictlyIncreasing(history), isFalse);
      },
    );

    test('đổi offset giữa chừng làm số GIẢM: lý do chỉ tăng offset', () {
      final before = computeBuildNumber(runNumber: 30, offset: 500);
      final after = computeBuildNumber(runNumber: 31, offset: 0);
      expect(isNonDecreasing([before, after]), isFalse);
    });

    test('trần Google Play: 2.100.000.000 qua, quá thì RangeError', () {
      expect(computeBuildNumber(runNumber: maxVersionCode), maxVersionCode);
      expect(
        () => computeBuildNumber(runNumber: maxVersionCode, offset: 1),
        throwsRangeError,
      );
    });

    test('0 và số âm bị từ chối', () {
      expect(() => computeBuildNumber(runNumber: 0), throwsRangeError);
      expect(() => computeBuildNumber(runNumber: -1), throwsArgumentError);
      expect(
        () => computeBuildNumber(runNumber: 1, offset: -1),
        throwsArgumentError,
      );
    });
  });

  group('mô hình chi phí (giá kiểm tại 11/2026, thời gian minh hoạ)', () {
    test(
      'GitHub: một lần phát hành 15 phút Linux + 30 phút macOS = 1,95 USD',
      () {
        final cost = releaseCost(linux: githubLinux, mac: githubMacos);
        expect(cost, 15 * 6 + 30 * 62);
        expect(formatUsd(cost), '1,95 USD');
      },
    );

    test('Codemagic pay as you go, cùng thời gian: 3,53 USD', () {
      final cost = releaseCost(linux: codemagicLinux, mac: codemagicMacM2);
      expect(cost, 15 * 45 + 30 * 95);
      expect(formatUsd(cost), '3,53 USD');
    });

    test('macOS đắt khoảng 10,3 lần Linux trên GitHub', () {
      expect(
        githubMacos.milliUsdPerMinute / githubLinux.milliUsdPerMinute,
        closeTo(10.33, 0.01),
      );
    });

    test('phút macOS chiếm phần lớn hoá đơn: 95% của lần phát hành GitHub', () {
      final total = releaseCost(linux: githubLinux, mac: githubMacos);
      final mac = githubMacos.cost(illustrativeIosMinutes);
      expect(mac * 100 ~/ total, 95);
    });

    test(
      'GitHub Free (repository riêng): 12 USD included đủ 6 lần phát hành',
      () {
        final cost = releaseCost(linux: githubLinux, mac: githubMacos);
        expect(releasesWithin(githubFreeIncludedMilliUsd, cost), 6);
      },
    );

    test(
      'job iOS đắt hơn job Android hơn 20 lần (1.860 so với 90 milli-USD)',
      () {
        final android = githubLinux.cost(illustrativeAndroidMinutes);
        final ios = githubMacos.cost(illustrativeIosMinutes);
        expect(ios, 1860);
        expect(android, 90);
        expect(ios ~/ android, 20); // 20,67 chia nguyên
      },
    );

    test('cùng đơn vị: chỉ dựng iOS 30 phút, 12 USD cũng đủ 6 lần', () {
      final iosOnly = githubMacos.cost(illustrativeIosMinutes);
      expect(releasesWithin(githubFreeIncludedMilliUsd, iosOnly), 6);
      // Codemagic: 500 phút miễn phí chia 30 phút mỗi lần dựng iOS.
      expect(500 ~/ illustrativeIosMinutes, 16);
    });

    test('hàm biên: ngân sách 0 và chi phí không dương', () {
      expect(releasesWithin(0, 1950), 0);
      expect(() => releasesWithin(100, 0), throwsArgumentError);
    });

    test('formatUsd làm tròn tới cent', () {
      expect(formatUsd(0), '0,00 USD');
      expect(formatUsd(5), '0,01 USD');
      expect(formatUsd(4), '0,00 USD');
      expect(formatUsd(12000), '12,00 USD');
    });

    test('giá GitHub khớp bảng trong bài 33 (cùng nguồn docs.github.com)', () {
      final article33 = File(
        '${repoRoot.path}/api/seed/content/flutter-ci-github-actions-analyze-test-golden-cache.md',
      ).readAsStringSync();
      expect(article33, contains('| Linux 2 nhân (x64) | 0,006 |'));
      expect(article33, contains('| macOS 3 hoặc 4 nhân | 0,062 |'));
    });
  });
}

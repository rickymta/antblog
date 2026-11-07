import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/backoff.dart';

import 'support.dart';

void main() {
  group('trần của exponential backoff', () {
    test('gấp đôi mỗi lần rồi dừng ở cap', () {
      final backoff = Backoff();
      expect(
        [for (var i = 0; i < 8; i++) backoff.ceiling(i).inSeconds],
        [1, 2, 4, 8, 16, 30, 30, 30],
      );
    });

    test('base lớn hơn cap/2: bước đầu chưa chạm cap, bước sau thì chạm', () {
      final backoff = Backoff(
        base: const Duration(seconds: 20),
        cap: const Duration(seconds: 30),
      );
      expect(backoff.ceiling(0), const Duration(seconds: 20));
      expect(backoff.ceiling(1), const Duration(seconds: 30));
    });

    test('attempt rất lớn không tràn số: vẫn là cap', () {
      final backoff = Backoff();
      for (final attempt in [29, 30, 31, 1000, 1 << 40]) {
        expect(backoff.ceiling(attempt), const Duration(seconds: 30));
      }
    });

    test('attempt âm là lỗi lập trình', () {
      expect(() => Backoff().ceiling(-1), throwsRangeError);
    });

    test('tham số vô lý bị assert chặn', () {
      expect(() => Backoff(base: Duration.zero), throwsAssertionError);
      expect(
        () => Backoff(
          base: const Duration(seconds: 10),
          cap: const Duration(seconds: 5),
        ),
        throwsAssertionError,
      );
    });
  });

  group('full jitter', () {
    test(
      'biên: random 0 cho 0, random sát 1 cho sát trần nhưng không vượt',
      () {
        final low = Backoff(random: FixedRandom(0));
        final high = Backoff(random: FixedRandom(0.999999));

        expect(low.delay(3), Duration.zero);
        expect(high.delay(3), lessThan(low.ceiling(3)));
        expect(high.delay(3), greaterThan(const Duration(milliseconds: 7990)));
      },
    );

    test('10.000 mẫu nằm trong [0, trần), trung bình gần trần/2', () {
      final backoff = Backoff(random: Random(42));
      final ceiling = backoff.ceiling(4); // 16 s
      final samples = [for (var i = 0; i < 10000; i++) backoff.delay(4)];

      expect(samples.every((d) => d >= Duration.zero && d < ceiling), isTrue);
      final mean =
          samples.map((d) => d.inMicroseconds).reduce((a, b) => a + b) /
          samples.length;
      expect(
        mean,
        closeTo(ceiling.inMicroseconds / 2, 0.03 * ceiling.inMicroseconds),
      );
      // Có cả giá trị rất nhỏ lẫn sát trần: không dồn về một chỗ.
      expect(samples.any((d) => d < const Duration(milliseconds: 100)), isTrue);
      expect(
        samples.any((d) => d > const Duration(milliseconds: 15900)),
        isTrue,
      );
    });

    test('1000 client mất kết nối cùng lúc: jitter rải đều, không jitter dồn vào một mốc', () {
      int busiestBucket(Iterable<Duration> delays) {
        final buckets = <int, int>{};
        for (final d in delays) {
          buckets.update(
            d.inMilliseconds ~/ 500,
            (n) => n + 1,
            ifAbsent: () => 1,
          );
        }
        return buckets.values.reduce(max);
      }

      final backoff = Backoff(random: Random(7));
      final withJitter = [for (var i = 0; i < 1000; i++) backoff.delay(3)];
      final withoutJitter = [for (var i = 0; i < 1000; i++) backoff.ceiling(3)];

      // Trần 8 s chia 16 ô 500 ms: kỳ vọng ~63 client mỗi ô.
      expect(busiestBucket(withJitter), lessThan(100));
      expect(busiestBucket(withoutJitter), 1000);
    });
  });
}

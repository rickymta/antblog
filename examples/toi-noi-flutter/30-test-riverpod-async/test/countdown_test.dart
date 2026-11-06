import 'dart:async';

import 'package:clock/clock.dart';
import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/countdown.dart';

import 'support.dart';

const _second = Duration(seconds: 1);

/// Nghe [stream] trong lúc test; trả các giá trị đã nhận và cờ đã đóng.
({List<Duration> values, bool Function() isDone}) _collect(
  Stream<Duration> stream,
) {
  final values = <Duration>[];
  var done = false;
  stream.listen(values.add, onDone: () => done = true);
  return (values: values, isDone: () => done);
}

void main() {
  group('remainingUntil', () {
    test('hạn 3 giây: phát 3, 2, 1, 0 rồi đóng', () {
      fakeAsync((async) {
        final result = _collect(
          remainingUntil(fixedNow.add(const Duration(seconds: 3))),
        );

        async.flushMicrotasks();
        expect(result.values, [const Duration(seconds: 3)]);

        async.elapse(const Duration(seconds: 3));

        expect(result.values, [
          for (final s in [3, 2, 1, 0]) Duration(seconds: s),
        ]);
        expect(result.isDone(), isTrue);
        expect(async.pendingTimers, isEmpty);
      }, initialTime: fixedNow);
    });

    test('hạn lẻ nửa giây: phần còn lại giữ phần lẻ, rồi về 0', () {
      fakeAsync((async) {
        final result = _collect(
          remainingUntil(fixedNow.add(const Duration(milliseconds: 2500))),
        );

        async.elapse(const Duration(seconds: 3));

        expect(result.values, [
          const Duration(milliseconds: 2500),
          const Duration(milliseconds: 1500),
          const Duration(milliseconds: 500),
          Duration.zero,
        ]);
        expect(result.isDone(), isTrue);
      }, initialTime: fixedNow);
    });

    test('hạn đã qua: phát đúng một số 0 và đóng, không tạo Timer', () {
      fakeAsync((async) {
        final result = _collect(
          remainingUntil(fixedNow.subtract(const Duration(minutes: 1))),
        );

        async.flushMicrotasks();

        expect(result.values, [Duration.zero]);
        expect(result.isDone(), isTrue);
        expect(async.pendingTimers, isEmpty);
      }, initialTime: fixedNow);
    });

    test('đúng hạn (còn 0): coi là hết hạn', () {
      fakeAsync((async) {
        final result = _collect(remainingUntil(fixedNow));

        async.flushMicrotasks();

        expect(result.values, [Duration.zero]);
        expect(result.isDone(), isTrue);
      }, initialTime: fixedNow);
    });

    test('máy ngủ 7 giây: lần phát kế tiếp vẫn đúng, vì tính từ hạn', () {
      fakeAsync((async) {
        final result = _collect(
          remainingUntil(fixedNow.add(const Duration(seconds: 10))),
        );
        async.flushMicrotasks();

        async.elapseBlocking(const Duration(seconds: 7)); // Timer chưa chạy
        async.elapse(_second);

        // Đồng hồ đã ở giây thứ 8 nên còn 2 giây; một bộ đếm số lần tick
        // thì không hề biết 7 giây đã trôi.
        expect(result.values.last, const Duration(seconds: 2));
      }, initialTime: fixedNow);
    });

    test('huỷ subscription là huỷ luôn Timer', () {
      fakeAsync((async) {
        final subscription = remainingUntil(
          fixedNow.add(const Duration(minutes: 1)),
        ).listen((_) {});
        async.flushMicrotasks();
        expect(async.periodicTimerCount, 1);

        subscription.cancel();

        expect(async.periodicTimerCount, 0);
      }, initialTime: fixedNow);
    });

    test('Clock.fixed bên trong fakeAsync làm đếm ngược đứng yên', () {
      fakeAsync((async) {
        withClock(Clock.fixed(fixedNow), () {
          final result = _collect(
            remainingUntil(fixedNow.add(const Duration(seconds: 3))),
          );

          async.elapse(const Duration(seconds: 10));

          // Timer đã tick 10 lần nhưng clock không đi: mãi còn 3 giây.
          expect(result.values.toSet(), {const Duration(seconds: 3)});
          expect(result.isDone(), isFalse);
        });
      }, initialTime: fixedNow);
    });
  });

  group('countdownProvider', () {
    test('ghi lại chuỗi state: loading, rồi mỗi giây một AsyncData', () {
      fakeAsync((async) {
        final container = ProviderContainer.test();
        final states = record(
          container,
          countdownProvider(fixedNow.add(const Duration(seconds: 3))),
        );

        async.elapse(const Duration(seconds: 3));

        expect(states.first.isLoading, isTrue);
        expect(states.skip(1).map((s) => s.requireValue.inSeconds), [
          3,
          2,
          1,
          0,
        ]);
        expect(states.last.isLoading, isFalse);
      }, initialTime: fixedNow);
    });

    test('đếm ngược nhận cuốc: 15 giây từ lúc được đề nghị', () {
      fakeAsync((async) {
        final deadline = offerDeadline(fixedNow);
        expect(deadline, fixedNow.add(const Duration(seconds: 15)));
        final container = ProviderContainer.test();
        final states = record(container, countdownProvider(deadline));

        async.elapse(const Duration(seconds: 14));
        expect(states.last.requireValue, _second);

        async.elapse(_second);
        expect(states.last.requireValue, Duration.zero);
      }, initialTime: fixedNow);
    });

    test('bỏ provider giữa chừng thì không còn Timer nào', () {
      fakeAsync((async) {
        final container = ProviderContainer.test();
        container.listen(
          countdownProvider(fixedNow.add(const Duration(minutes: 1))),
          (_, _) {},
        );
        async.elapse(const Duration(seconds: 5));
        expect(async.periodicTimerCount, 1);

        container.dispose();

        expect(async.periodicTimerCount, 0);
      }, initialTime: fixedNow);
    });
  });

  group('formatRemaining', () {
    test('làm tròn lên giây, "0:00" chỉ khi đã hết hạn', () {
      const cases = <(Duration, String)>[
        (Duration(seconds: 15), '0:15'),
        (Duration(milliseconds: 14200), '0:15'),
        (Duration(milliseconds: 1), '0:01'),
        (Duration.zero, '0:00'),
        (Duration(seconds: 125), '2:05'),
        (Duration(minutes: 2), '2:00'),
      ];

      for (final (remaining, text) in cases) {
        expect(formatRemaining(remaining), text, reason: '$remaining');
      }
    });
  });
}

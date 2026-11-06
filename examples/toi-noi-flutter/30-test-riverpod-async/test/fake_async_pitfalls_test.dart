import 'dart:async';
import 'dart:io';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/countdown.dart';
import 'package:toi_noi_30/quote_providers.dart';

import 'support.dart';

void main() {
  group('microtask và Timer là hai hàng đợi khác nhau', () {
    test('Future.value().then chạy khi flushMicrotasks', () {
      fakeAsync((async) {
        var ran = false;
        Future<void>.value().then((_) => ran = true);
        expect(ran, isFalse);

        async.flushMicrotasks();

        expect(ran, isTrue);
      });
    });

    test(
      'Future(...) và Future.delayed(zero) là Timer: flushMicrotasks không đủ',
      () {
        fakeAsync((async) {
          final ran = <String>[];
          Future<void>(() => ran.add('Future'));
          Future<void>.delayed(Duration.zero, () => ran.add('delayed'));

          async.flushMicrotasks();
          expect(ran, isEmpty);

          async.elapse(Duration.zero);
          expect(ran, ['Future', 'delayed']);
        });
      },
    );

    test('request của dio cần elapse(zero), flushMicrotasks không đưa nó tới adapter', () {
      fakeAsync((async) {
        final api = ApiHarness();
        final container = ProviderContainer.test(
          overrides: [api.override],
          retry: (_, _) => null,
        );
        container.listen(quoteProvider(ride5km), (_, _) {});

        async.flushMicrotasks();
        expect(api.attempts, 0);

        async.elapse(Duration.zero);
        expect(api.attempts, 1);
        expect(container.read(quoteProvider(ride5km)).hasValue, isTrue);
      });
    });

    test('hàm async dừng ở await cho tới khi elapse đủ lâu', () {
      fakeAsync((async) {
        var after = false;
        // Hàm async chạy đồng bộ tới await đầu tiên rồi dừng.
        unawaited(() async {
          await Future<void>.delayed(const Duration(seconds: 1));
          after = true;
        }());

        expect(after, isFalse);
        async.elapse(const Duration(seconds: 1));
        expect(after, isTrue);
      });
    });
  });

  group('ngoài vùng của fakeAsync', () {
    test('Timer tạo trước khi vào fakeAsync không bị điều khiển', () {
      var fired = false;
      final real = Timer(const Duration(milliseconds: 1), () => fired = true);

      fakeAsync((async) {
        async.elapse(const Duration(hours: 1));
      });

      expect(fired, isFalse); // một giờ giả không làm Timer thật chạy
      real.cancel();
    });

    test('I/O thật (đọc file) không hoàn thành trong fakeAsync', () {
      fakeAsync((async) {
        var done = false;
        File('pubspec.yaml').exists().then((_) => done = true);

        async.elapse(const Duration(hours: 1));
        async.flushMicrotasks();

        // Kết quả của I/O đến từ vòng lặp sự kiện thật, mà thân test đồng
        // bộ này chưa nhường cho nó lần nào.
        expect(done, isFalse);
      });
    });
  });

  group('fakeAsync không tự dọn Timer', () {
    test('Timer còn lại vẫn nằm trong pendingTimers khi test kết thúc', () {
      late FakeAsync captured;
      fakeAsync((async) {
        captured = async;
        remainingUntil(fixedNow.add(const Duration(minutes: 1))).listen((_) {});
        async.elapse(const Duration(seconds: 3));
      }, initialTime: fixedNow);

      expect(captured.periodicTimerCount, 1);
      expect(captured.pendingTimersDebugString, hasLength(1));
    });

    test(
      'flushTimers với Timer.periodic chưa huỷ ném StateError sau 1 giờ giả',
      () {
        fakeAsync((async) {
          remainingUntil(fixedNow.add(const Duration(days: 1))).listen((_) {});

          expect(async.flushTimers, throwsStateError);
        }, initialTime: fixedNow);
      },
    );

    test('đếm ngược đã hết hạn thì flushTimers chạy xong, không còn Timer', () {
      fakeAsync((async) {
        remainingUntil(fixedNow.add(const Duration(seconds: 5))).listen((_) {});

        async.flushTimers();

        expect(async.pendingTimers, isEmpty);
        expect(async.elapsed, const Duration(seconds: 5));
      }, initialTime: fixedNow);
    });
  });
}

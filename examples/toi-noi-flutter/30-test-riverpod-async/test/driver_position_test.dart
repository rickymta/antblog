import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/driver_position.dart';

import 'support.dart';

Matcher _data(DriverPosition expected) =>
    isA<AsyncData<DriverPosition>>().having((s) => s.value, 'value', expected);

void main() {
  late FakeLocationSource source;

  ProviderContainer newContainer() => ProviderContainer.test(
    overrides: [driverLocationSourceProvider.overrideWithValue(source)],
  );

  setUp(() => source = FakeLocationSource());

  group('thứ tự sự kiện', () {
    test('emitsInOrder: loading, vị trí 1, vị trí 2', () async {
      final states = stateStream(newContainer(), driverPositionProvider('D1'));

      final expectation = expectLater(
        states,
        emitsInOrder([
          isA<AsyncLoading<DriverPosition>>(),
          _data(position(10.77)),
          _data(position(10.78)),
        ]),
      );
      source.emit(position(10.77));
      source.emit(position(10.78));

      await expectation;
    });

    test(
      'lỗi từ nguồn thành AsyncError, vị trí sau đó đưa về AsyncData',
      () async {
        final states = stateStream(
          newContainer(),
          driverPositionProvider('D1'),
        );

        final expectation = expectLater(
          states,
          emitsInOrder([
            isA<AsyncLoading<DriverPosition>>(),
            _data(position(10.77)),
            isA<AsyncError<DriverPosition>>()
                .having((s) => s.error, 'error', isA<StateError>())
                .having((s) => s.value, 'vị trí cũ', position(10.77)),
            _data(position(10.79)),
          ]),
        );
        source.emit(position(10.77));
        source.fail(StateError('mất kết nối'));
        source.emit(position(10.79));

        await expectation;
      },
    );

    test('mỗi tài xế một provider, mỗi provider nghe nguồn một lần', () {
      source = FakeLocationSource(broadcast: true);
      final container = newContainer();

      container.listen(driverPositionProvider('D1'), (_, _) {});
      container.listen(driverPositionProvider('D2'), (_, _) {});
      container.listen(driverPositionProvider('D1'), (_, _) {});

      expect(source.watched, ['D1', 'D2']);
    });
  });

  group('timeout khi im lặng (đồng hồ giả)', () {
    test('9,999 giây im lặng chưa sao, 10 giây thì AsyncError', () {
      fakeAsync((async) {
        final states = record(newContainer(), driverPositionProvider('D1'));
        async.flushMicrotasks();
        source.emit(position(10.77));
        async.flushMicrotasks();

        async.elapse(positionStaleAfter - const Duration(milliseconds: 1));
        expect(states.last, _data(position(10.77)));

        async.elapse(const Duration(milliseconds: 1));
        expect(
          states.last,
          isA<AsyncError<DriverPosition>>()
              .having((s) => s.error, 'error', isA<TimeoutException>())
              .having((s) => s.value, 'vị trí cũ', position(10.77)),
        );
      });
    });

    test('chưa có vị trí nào mà im 10 giây: lỗi và chưa có giá trị', () {
      fakeAsync((async) {
        final states = record(newContainer(), driverPositionProvider('D1'));

        async.elapse(positionStaleAfter);

        expect(states.last, isA<AsyncError<DriverPosition>>());
        expect(states.last.hasValue, isFalse);
      });
    });

    test('mỗi vị trí mới đặt lại đồng hồ chờ', () {
      fakeAsync((async) {
        final states = record(newContainer(), driverPositionProvider('D1'));
        async.flushMicrotasks();

        for (var i = 0; i < 5; i++) {
          source.emit(position(10.0 + i));
          async.elapse(const Duration(seconds: 9));
        }

        expect(states.every((s) => !s.hasError), isTrue);
        expect(states.last, _data(position(14.0)));
      });
    });

    test('im lặng suốt 25 giây chỉ báo một lỗi, không báo mỗi 10 giây', () {
      fakeAsync((async) {
        final states = record(newContainer(), driverPositionProvider('D1'));
        async.flushMicrotasks();
        source.emit(position(10.77));

        async.elapse(const Duration(seconds: 25));

        expect(states.where((s) => s.hasError), hasLength(1));
      });
    });

    test(
      'quá hạn, rồi có vị trí mới thì quay về AsyncData, không thử nghe lại',
      () {
        fakeAsync((async) {
          final states = record(newContainer(), driverPositionProvider('D1'));
          async.flushMicrotasks();
          source.emit(position(10.77));
          async.elapse(positionStaleAfter);
          expect(states.last.hasError, isTrue);

          source.emit(position(10.80));
          async.flushMicrotasks();

          expect(states.last, _data(position(10.80)));
          expect(source.watched, ['D1']); // retry tắt: không nghe lại nguồn
        });
      },
    );
  });

  group('retry mặc định với Stream', () {
    test('không tắt retry: TimeoutException làm Riverpod nghe lại nguồn', () {
      fakeAsync((async) {
        source = FakeLocationSource(broadcast: true);
        // Cùng thân với driverPositionProvider nhưng giữ retry mặc định.
        final provider = StreamProvider.autoDispose<DriverPosition>(
          (ref) => source.watch('D1').timeout(positionStaleAfter),
        );
        final states = record(ProviderContainer.test(), provider);
        async.flushMicrotasks();

        async.elapse(positionStaleAfter);
        expect(source.watched, hasLength(1));

        async.elapse(const Duration(milliseconds: 200));
        expect(source.watched, hasLength(2)); // đã nghe lại sau 200 ms
        expect(states.last.retrying, isTrue);
      });
    });
  });

  group('huỷ subscription', () {
    test('dispose container thì nguồn bị huỷ và không còn Timer', () {
      fakeAsync((async) {
        final container = newContainer();
        container.listen(driverPositionProvider('D1'), (_, _) {});
        async.flushMicrotasks();
        expect(source.hasListener, isTrue);
        expect(async.pendingTimers, isNotEmpty);

        container.dispose();

        expect(source.hasListener, isFalse);
        expect(source.cancels, 1);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('đóng listener cuối thì autoDispose huỷ nguồn', () async {
      final container = newContainer();
      final subscription = container.listen(
        driverPositionProvider('D1'),
        (_, _) {},
      );
      expect(source.hasListener, isTrue);

      subscription.close();
      await container.pump();

      expect(source.hasListener, isFalse);
      expect(source.cancels, 1);
    });

    test('còn một listener thì chưa huỷ', () async {
      final container = newContainer();
      final first = container.listen(driverPositionProvider('D1'), (_, _) {});
      final second = container.listen(driverPositionProvider('D1'), (_, _) {});

      first.close();
      await container.pump();
      expect(source.hasListener, isTrue);

      second.close();
      await container.pump();
      expect(source.hasListener, isFalse);
    });
  });
}

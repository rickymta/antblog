import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show ProviderException;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_26/dto/trip_dto.dart';
import 'package:toi_noi_26/providers.dart';
import 'package:toi_noi_26/quote_state.dart';
import 'package:toi_noi_api/toi_noi_api.dart' hide TripDto;
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/fake_source.dart';
import 'support/fixtures.dart';

const _input = QuoteInput(serviceKind: ServiceKind.ride, distanceKm: 5);

ProviderContainer _container(FakeQuoteSource source) => ProviderContainer.test(
  // Riverpod 3 tự thử lại provider ném Exception; test không muốn chờ.
  retry: (_, _) => null,
  overrides: [quoteSourceProvider.overrideWithValue(source)],
);

void main() {
  group('quoteSourceProvider (hàm + @Riverpod(keepAlive: true))', () {
    test('chưa ghi đè thì đọc là lỗi, nêu cách sửa', () {
      final container = ProviderContainer.test();
      expect(
        () => container.read(quoteSourceProvider),
        throwsA(
          isA<ProviderException>().having(
            (e) => e.exception.toString(),
            'exception',
            contains('quoteSourceProvider chưa được ghi đè'),
          ),
        ),
      );
    });

    test('cờ do generator sinh: keepAlive thì không auto-dispose', () {
      expect(quoteSourceProvider.isAutoDispose, isFalse);
      expect(quoteControllerProvider.isAutoDispose, isTrue);
      expect(tripSnapshotProvider('T-1').isAutoDispose, isTrue);
    });
  });

  group('QuoteController (class + @riverpod)', () {
    test('idle, loading rồi ready: đúng thứ tự, đúng dữ liệu', () async {
      final source = FakeQuoteSource();
      final container = _container(source);
      final seen = <QuoteScreenState>[];
      container.listen(
        quoteControllerProvider,
        (_, next) => seen.add(next),
        fireImmediately: true,
      );

      final done = container
          .read(quoteControllerProvider.notifier)
          .fetch(_input);
      source.answerQuote(0, calculateFare(5));
      await done;

      expect(seen, [
        const QuoteScreenState.idle(),
        const QuoteScreenState.loading(input: _input),
        QuoteScreenState.ready(input: _input, fare: calculateFare(5)),
      ]);
      expect(calculateFare(5), const Money(25500));
    });

    test('lỗi đã phân loại nằm trong state, không ném ra', () async {
      final source = FakeQuoteSource();
      final container = _container(source);
      container.listen(quoteControllerProvider, (_, _) {});

      final done = container
          .read(quoteControllerProvider.notifier)
          .fetch(_input);
      source.quoteCalls.single.completeError(
        const TimeoutFailure(phase: TimeoutPhase.receive),
      );
      await done;

      expect(
        container.read(quoteControllerProvider),
        isA<QuoteFailed>()
            .having((s) => s.failure, 'failure', isA<TimeoutFailure>())
            .having((s) => s.input, 'input', _input),
      );
    });

    test(
      'lỗi lập trình ném ra và state quay về idle, không kẹt loading',
      () async {
        final source = FakeQuoteSource();
        final container = _container(source);
        container.listen(quoteControllerProvider, (_, _) {});

        final done = container
            .read(quoteControllerProvider.notifier)
            .fetch(_input);
        expect(container.read(quoteControllerProvider), isA<QuoteLoading>());
        source.quoteCalls.single.completeError(StateError('bug'));

        await expectLater(done, throwsStateError);
        expect(container.read(quoteControllerProvider), isA<QuoteIdle>());
      },
    );

    test('bỏ provider trong lúc chờ: kết quả đến muộn không ném lỗi', () async {
      final source = FakeQuoteSource();
      final container = ProviderContainer(
        retry: (_, _) => null,
        overrides: [quoteSourceProvider.overrideWithValue(source)],
      );
      container.listen(quoteControllerProvider, (_, _) {});
      final done = container
          .read(quoteControllerProvider.notifier)
          .fetch(_input);

      container.dispose();
      source.answerQuote(0, const Money(25500));

      // Nếu code đụng `state` sau khi bị dispose thì `done` hoàn thành bằng lỗi.
      await done;
    });

    test('auto-dispose: hết người nghe thì state về lại idle', () async {
      final source = FakeQuoteSource();
      final container = _container(source);
      final sub = container.listen(quoteControllerProvider, (_, _) {});
      final done = container
          .read(quoteControllerProvider.notifier)
          .fetch(_input);
      source.answerQuote(0, const Money(25500));
      await done;
      expect(container.read(quoteControllerProvider), isA<QuoteReady>());

      sub.close();
      await container.pump();

      container.listen(quoteControllerProvider, (_, _) {});
      expect(container.read(quoteControllerProvider), isA<QuoteIdle>());
    });
  });

  group('tripSnapshotProvider (hàm có tham số thành family)', () {
    TripDto trip(String id) =>
        TripDto.fromJson(changed(validTripJson(), {'id': id}));

    test('mỗi id có state riêng và được gọi đúng một lần', () async {
      final source = FakeQuoteSource();
      final container = _container(source);
      container.listen(tripSnapshotProvider('T-1'), (_, _) {});
      container.listen(tripSnapshotProvider('T-2'), (_, _) {});
      source.answerTrip('T-1', trip('T-1'));
      source.answerTrip('T-2', trip('T-2'));

      final one = await container.read(tripSnapshotProvider('T-1').future);
      final two = await container.read(tripSnapshotProvider('T-2').future);

      expect(one.trip.id, 'T-1');
      expect(two.trip.id, 'T-2');
      expect(source.tripCalls, ['T-1', 'T-2']);
    });

    test(
      'cùng tham số là cùng provider (== theo tham số), không gọi lại',
      () async {
        final source = FakeQuoteSource();
        final container = _container(source);
        expect(tripSnapshotProvider('T-1'), tripSnapshotProvider('T-1'));
        expect(tripSnapshotProvider('T-1'), isNot(tripSnapshotProvider('T-2')));

        container.listen(tripSnapshotProvider('T-1'), (_, _) {});
        source.answerTrip('T-1', trip('T-1'));
        await container.read(tripSnapshotProvider('T-1').future);
        await container.read(tripSnapshotProvider('T-1').future);

        expect(source.tripCalls, ['T-1']);
      },
    );

    test('ApiFailure của một id không lan sang id khác', () async {
      final source = FakeQuoteSource();
      final container = _container(source);
      container.listen(tripSnapshotProvider('T-1'), (_, _) {});
      container.listen(tripSnapshotProvider('T-2'), (_, _) {});
      source.failTrip(
        'T-1',
        const ClientFailure(statusCode: 404, title: 'Trip not found'),
      );
      source.answerTrip('T-2', trip('T-2'));

      await expectLater(
        container.read(tripSnapshotProvider('T-1').future),
        throwsA(isA<ClientFailure>()),
      );
      expect(
        (await container.read(tripSnapshotProvider('T-2').future)).trip.id,
        'T-2',
      );
    });
  });
}

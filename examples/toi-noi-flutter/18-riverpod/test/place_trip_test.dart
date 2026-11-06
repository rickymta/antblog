import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show ProviderException;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/providers.dart';
import 'package:toi_noi_18/quote.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_18/trip_draft.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Bản sai có chủ đích: giống `PlaceTripNotifier` nhưng quên kiểm `ref.mounted`.
class _UnsafePlaceTripNotifier extends AsyncNotifier<Trip?> {
  @override
  Trip? build() => null;

  Future<void> place(Quote quote) async {
    state = const AsyncLoading();
    final repository = ref.read(quoteRepositoryProvider);
    state = await AsyncValue.guard(() => repository.placeTrip(quote));
  }
}

final _unsafePlaceTripProvider =
    AsyncNotifierProvider.autoDispose<_UnsafePlaceTripNotifier, Trip?>(
      _UnsafePlaceTripNotifier.new,
    );

class _RefNotifier extends Notifier<double> {
  @override
  double build() => ref.watch(tripDraftProvider).distanceKm;

  Ref get currentRef => ref;
}

void main() {
  group('PlaceTripNotifier', () {
    test('lúc đầu là AsyncData(null): chưa đặt chuyến', () {
      final container = ProviderContainer.test();

      expect(container.read(placeTripProvider), const AsyncData<Trip?>(null));
    });

    test(
      'place thành công: AsyncLoading rồi AsyncData(chuyến requested)',
      () async {
        final repository = FakeQuoteRepository(
          now: () => DateTime.utc(2026, 11, 6, 8),
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        final states = <AsyncValue<Trip?>>[];
        container.listen(placeTripProvider, (_, next) => states.add(next));

        await container.read(placeTripProvider.notifier).place(quoteFor(5));

        expect(states.first, isA<AsyncLoading<Trip?>>());
        final trip = states.last.value!;
        expect(trip.id, 'T-0001');
        expect(trip.status, TripStatus.requested);
        expect(trip.fare, const Money(25500));
        expect(trip.driverId, isNull);
      },
    );

    test('place gửi đúng báo giá khách đã thấy', () async {
      final repository = StubQuoteRepository();
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );
      container.listen(placeTripProvider, (_, _) {});

      await container.read(placeTripProvider.notifier).place(quoteFor(2.001));

      expect(repository.placed.single.fare, const Money(12005));
    });

    test(
      'place lỗi: AsyncError mang đúng lỗi, và đặt lại được sau đó',
      () async {
        var attempt = 0;
        final repository = StubQuoteRepository(
          onPlace: (quote) {
            attempt++;
            return attempt == 1
                ? Future<Trip>.error(const QuoteUnavailable())
                : Future.value(tripFor('T-0002'));
          },
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        container.listen(placeTripProvider, (_, _) {});
        final notifier = container.read(placeTripProvider.notifier);

        await notifier.place(quoteFor(5));
        expect(
          container.read(placeTripProvider).error,
          isA<QuoteUnavailable>(),
        );

        await notifier.place(quoteFor(5));
        expect(container.read(placeTripProvider).value?.id, 'T-0002');
        expect(repository.placed, hasLength(2));
      },
    );

    test(
      'rời màn hình giữa chừng: kiểm ref.mounted giúp place() kết thúc êm',
      () async {
        final reply = Completer<Trip>();
        final repository = StubQuoteRepository(onPlace: (_) => reply.future);
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        final subscription = container.listen(placeTripProvider, (_, _) {});
        final notifier = container.read(placeTripProvider.notifier);

        final pending = notifier.place(quoteFor(5));
        subscription.close(); // khách rời màn hình
        await container.pump();
        expect(container.exists(placeTripProvider), isFalse);

        reply.complete(tripFor('T-0001'));
        await pending; // không ném lỗi
      },
    );

    test(
      'quên kiểm ref.mounted: ghi state sau khi bị dispose ném lỗi',
      () async {
        final reply = Completer<Trip>();
        final repository = StubQuoteRepository(onPlace: (_) => reply.future);
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        final subscription = container.listen(
          _unsafePlaceTripProvider,
          (_, _) {},
        );
        final notifier = container.read(_unsafePlaceTripProvider.notifier);

        final pending = notifier.place(quoteFor(5));
        subscription.close();
        await container.pump();
        reply.complete(tripFor('T-0001'));

        await expectLater(
          pending,
          throwsA(
            isA<Exception>().having(
              (e) => e.toString(),
              'message',
              contains('after it has been disposed'),
            ),
          ),
        );
      },
    );
  });

  group('ref.mounted sau khi provider tính lại', () {
    test('ref của provider dạng hàm là ref của lần dựng cũ; trong Notifier thì không', () async {
      final functionRefs = <Ref>[];
      final functional = Provider<double>((ref) {
        functionRefs.add(ref);
        return ref.watch(tripDraftProvider).distanceKm;
      });
      final notifierProvider = NotifierProvider<_RefNotifier, double>(
        _RefNotifier.new,
      );
      final container = ProviderContainer.test();
      container.listen(functional, (_, _) {});
      container.listen(notifierProvider, (_, _) {});
      final notifier = container.read(notifierProvider.notifier);

      container.read(tripDraftProvider.notifier).setDistanceKm(9);
      await container.pump();
      container.read(functional);
      container.read(notifierProvider);

      expect(functionRefs.first.mounted, isFalse); // ref cũ, provider vẫn sống
      expect(functionRefs.last.mounted, isTrue);
      expect(notifier.currentRef.mounted, isTrue); // ref hiện tại
    });
  });

  group('ProviderException', () {
    test('provider đồng bộ ném lỗi thì container.read ném ProviderException bọc lỗi gốc', () {
      final broken = Provider<int>((ref) => throw const TripNotFound('Z'));
      final container = ProviderContainer.test();

      expect(
        () => container.read(broken),
        throwsA(
          isA<ProviderException>().having(
            (e) => e.exception,
            'exception',
            isA<TripNotFound>(),
          ),
        ),
      );
    });
  });
}

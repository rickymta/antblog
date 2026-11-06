import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/providers.dart';
import 'package:toi_noi_18/quote.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Chạy [body] trong đồng hồ giả: `async.elapse` đẩy thời gian đi mà không
/// chờ thật, nên test thử lại 38 giây chạy trong mili giây.
void _fake(void Function(FakeAsync async) body) => fakeAsync(body);

void main() {
  group(
    'ProviderContainer.defaultRetry (chính sách mặc định của Riverpod 3)',
    () {
      test('Exception: 200 ms nhân đôi mỗi lần, trần 6400 ms', () {
        const exception = QuoteUnavailable();
        final delays = [
          for (var n = 0; n < 10; n++)
            ProviderContainer.defaultRetry(n, exception)?.inMilliseconds,
        ];

        expect(delays, [
          200,
          400,
          800,
          1600,
          3200,
          6400,
          6400,
          6400,
          6400,
          6400,
        ]);
      });

      test('lỗi không phải Exception (ném một chuỗi) vẫn được thử lại', () {
        expect(
          ProviderContainer.defaultRetry(0, 'boom'),
          const Duration(milliseconds: 200),
        );
      });

      test('từ lần thử thứ 11 thì dừng', () {
        expect(
          ProviderContainer.defaultRetry(10, const QuoteUnavailable()),
          isNull,
        );
      });

      test('Error (lỗi lập trình) không bao giờ được thử lại', () {
        expect(ProviderContainer.defaultRetry(0, StateError('bug')), isNull);
        expect(ProviderContainer.defaultRetry(0, ArgumentError('bug')), isNull);
      });
    },
  );

  group('provider không đặt retry: thử lại mặc định', () {
    // Không truyền `retry:`, nên dùng chính sách mặc định.
    FutureProvider<Quote> failingWith(
      Object error,
      List<Duration> calls,
      FakeAsync async,
    ) => FutureProvider.autoDispose<Quote>((ref) {
      calls.add(async.elapsed);
      return Future<Quote>.error(error);
    });

    test('Exception: 11 lần gọi trong 38,2 giây rồi mới thành AsyncError', () {
      _fake((async) {
        final calls = <Duration>[];
        final provider = failingWith(const QuoteUnavailable(), calls, async);
        final container = ProviderContainer.test();
        container.listen(provider, (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(seconds: 38));
        expect(container.read(provider).isLoading, isTrue);

        async.elapse(const Duration(milliseconds: 200));
        expect(calls.map((d) => d.inMilliseconds), [
          0,
          200,
          600,
          1400,
          3000,
          6200,
          12600,
          19000,
          25400,
          31800,
          38200,
        ]);
        final state = container.read(provider);
        expect(state, isA<AsyncError<Quote>>());
        expect(state.isLoading, isFalse);
        expect(state.retrying, isFalse);
      });
    });

    test(
      'trong lúc chờ thử lại, state là AsyncLoading mang lỗi cũ (retrying)',
      () {
        _fake((async) {
          final provider = failingWith(const QuoteUnavailable(), [], async);
          final container = ProviderContainer.test();
          container.listen(provider, (_, _) {});

          async.flushMicrotasks();
          final state = container.read(provider);

          expect(state, isA<AsyncLoading<Quote>>());
          expect(state.retrying, isTrue);
          expect(state.hasError, isTrue);
          expect(state.error, isA<QuoteUnavailable>());
        });
      },
    );

    test('Error không được thử lại: AsyncError ngay, chỉ một lần gọi', () {
      _fake((async) {
        final calls = <Duration>[];
        final provider = failingWith(StateError('bug'), calls, async);
        final container = ProviderContainer.test();
        container.listen(provider, (_, _) {});

        async.flushMicrotasks();
        expect(container.read(provider), isA<AsyncError<Quote>>());
        async.elapse(const Duration(minutes: 1));

        expect(calls, hasLength(1));
      });
    });

    test('thử lại thành công thì ra AsyncData, các lần sau không gọi nữa', () {
      _fake((async) {
        var calls = 0;
        final provider = FutureProvider.autoDispose<int>((ref) async {
          calls++;
          if (calls == 1) throw const QuoteUnavailable();
          return 7;
        });
        final container = ProviderContainer.test();
        container.listen(provider, (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(milliseconds: 200));
        async.flushMicrotasks();

        expect(container.read(provider).value, 7);
        async.elapse(const Duration(minutes: 1));
        expect(calls, 2);
      });
    });

    test(
      'ProviderContainer.test(retry: ...) đổi chính sách cho cả container',
      () {
        _fake((async) {
          final calls = <Duration>[];
          final provider = failingWith(const QuoteUnavailable(), calls, async);
          final container = ProviderContainer.test(retry: (_, _) => null);
          container.listen(provider, (_, _) {});

          async.flushMicrotasks();
          async.elapse(const Duration(minutes: 1));

          expect(calls, hasLength(1));
          expect(container.read(provider), isA<AsyncError<Quote>>());
        });
      },
    );

    test('provider tự đặt retry thì không theo chính sách của container', () {
      _fake((async) {
        final calls = <Duration>[];
        final provider = FutureProvider.autoDispose<Quote>((ref) {
          calls.add(async.elapsed);
          return Future<Quote>.error(const QuoteUnavailable());
        }, retry: (count, _) => count < 1 ? const Duration(seconds: 5) : null);
        final container = ProviderContainer.test(retry: (_, _) => null);
        container.listen(provider, (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(seconds: 6));

        expect(calls.map((d) => d.inSeconds), [0, 5]);
      });
    });
  });

  group('retry tự viết', () {
    test('bỏ điều kiện QuoteUnavailable: TripNotFound bị thử lại 3 lần', () {
      _fake((async) {
        final times = <int>[];
        final provider = FutureProvider.autoDispose.family<Trip, String>(
          (ref, id) {
            times.add(async.elapsed.inMilliseconds);
            return Future<Trip>.error(TripNotFound(id));
          },
          retry: (count, error) =>
              count < 3 ? const Duration(seconds: 1) : null,
        );
        final container = ProviderContainer.test();
        container.listen(provider('T-9'), (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(seconds: 10));

        expect(times, [0, 1000, 2000, 3000]);
        expect(container.read(provider('T-9')), isA<AsyncError<Trip>>());
      });
    });
  });

  group('retry tự viết thay luật mặc định', () {
    test('hàm trả Duration cho mọi lỗi thì StateError cũng bị thử lại', () {
      _fake((async) {
        final times = <int>[];
        final provider = FutureProvider.autoDispose<Quote>(
          (ref) {
            times.add(async.elapsed.inMilliseconds);
            return Future<Quote>.error(StateError('bug'));
          },
          retry: (count, error) =>
              count < 2 ? const Duration(seconds: 1) : null,
        );
        final container = ProviderContainer.test();
        container.listen(provider, (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(seconds: 10));

        expect(times, [0, 1000, 2000]); // luật mặc định chỉ gọi một lần
      });
    });
  });

  group('retryTransientOnce của lab', () {
    test(
      'invalidate sau khi hết lượt không cấp lượt mới: chỉ gọi thêm một lần',
      () {
        _fake((async) {
          final times = <int>[];
          final repository = StubQuoteRepository(
            onQuote: (_, _) {
              times.add(async.elapsed.inMilliseconds);
              return Future<Quote>.error(const QuoteUnavailable());
            },
          );
          final container = ProviderContainer.test(
            overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
          );
          container.listen(quoteProvider, (_, _) {});
          async.flushMicrotasks();
          async.elapse(const Duration(seconds: 2));
          expect(times, [0, 1000]); // hết lượt: lần đầu và một lần thử lại

          container.invalidate(quoteProvider);
          async.flushMicrotasks();
          async.elapse(const Duration(minutes: 1));

          expect(times, hasLength(3)); // thêm đúng một lần, không thử lại nữa
          final state = container.read(quoteProvider);
          expect(state, isA<AsyncError<Quote>>());
          expect(state.retrying, isFalse);
        });
      },
    );

    test('QuoteUnavailable: thử lại đúng một lần sau 1 giây', () {
      expect(
        retryTransientOnce(0, const QuoteUnavailable()),
        const Duration(seconds: 1),
      );
      expect(retryTransientOnce(1, const QuoteUnavailable()), isNull);
    });

    test('lỗi khác không thử lại, kể cả Exception', () {
      expect(retryTransientOnce(0, const TripNotFound('T-1')), isNull);
      expect(retryTransientOnce(0, StateError('bug')), isNull);
    });

    test('quoteProvider: hai lần gọi cách 1 giây rồi AsyncError', () {
      _fake((async) {
        final times = <Duration>[];
        final repository = StubQuoteRepository(
          onQuote: (_, _) {
            times.add(async.elapsed);
            return Future<Quote>.error(const QuoteUnavailable());
          },
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        container.listen(quoteProvider, (_, _) {});

        async.flushMicrotasks();
        expect(container.read(quoteProvider).retrying, isTrue);
        async.elapse(const Duration(seconds: 1));
        async.flushMicrotasks();
        async.elapse(const Duration(minutes: 1));

        expect(times.map((d) => d.inMilliseconds), [0, 1000]);
        final state = container.read(quoteProvider);
        expect(state, isA<AsyncError<Quote>>());
        expect(state.retrying, isFalse);
      });
    });

    test('quoteProvider: lần thử lại thành công thì ra AsyncData', () {
      _fake((async) {
        var calls = 0;
        final repository = StubQuoteRepository(
          onQuote: (kind, km) {
            calls++;
            return calls == 1
                ? Future<Quote>.error(const QuoteUnavailable())
                : Future.value(quoteFor(km, kind: kind));
          },
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        container.listen(quoteProvider, (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(seconds: 1));
        async.flushMicrotasks();

        expect(container.read(quoteProvider).value?.fare, const Money(25500));
        expect(calls, 2);
      });
    });

    test('tripByIdProvider: TripNotFound báo ngay, không thử lại', () {
      _fake((async) {
        final repository = StubQuoteRepository(
          onTrip: (id) => Future<Trip>.error(TripNotFound(id)),
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        container.listen(tripByIdProvider('T-9'), (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(minutes: 1));

        expect(repository.tripRequests, ['T-9']);
        expect(
          container.read(tripByIdProvider('T-9')).error,
          isA<TripNotFound>(),
        );
      });
    });

    test('place() là method, không phải build: lỗi không bị thử lại', () {
      _fake((async) {
        final repository = StubQuoteRepository(
          onPlace: (_) => Future<Trip>.error(const QuoteUnavailable()),
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        container.listen(placeTripProvider, (_, _) {});

        container.read(placeTripProvider.notifier).place(quoteFor(5));
        async.flushMicrotasks();
        async.elapse(const Duration(minutes: 1));

        expect(repository.placed, hasLength(1));
        expect(container.read(placeTripProvider), isA<AsyncError<Trip?>>());
      });
    });
  });
}

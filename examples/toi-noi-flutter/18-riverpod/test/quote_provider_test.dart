import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/providers.dart';
import 'package:toi_noi_18/quote.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_18/trip_draft.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  group('quoteRepositoryProvider', () {
    test('mặc định là bản giả trong bộ nhớ', () {
      final container = ProviderContainer.test();

      expect(
        container.read(quoteRepositoryProvider),
        isA<FakeQuoteRepository>(),
      );
    });

    test('overrideWithValue thay bằng đối tượng của test', () {
      final stub = StubQuoteRepository();
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(stub)],
      );

      expect(identical(container.read(quoteRepositoryProvider), stub), isTrue);
    });
  });

  group('quoteProvider phụ thuộc tripDraftProvider', () {
    late StubQuoteRepository repository;
    late ProviderContainer container;

    setUp(() {
      repository = StubQuoteRepository();
      container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );
    });

    test(
      'bắt đầu ở AsyncLoading rồi thành AsyncData với cước đúng công thức',
      () async {
        expect(container.read(quoteProvider), isA<AsyncLoading<Quote>>());

        final quote = await container.read(quoteProvider.future);

        expect(quote.fare, const Money(25500)); // 5 km
        expect(repository.quoteRequests, [(kind: ServiceKind.ride, km: 5.0)]);
        expect(container.read(quoteProvider), isA<AsyncData<Quote>>());
      },
    );

    test('đổi quãng đường thì hỏi lại: 2,001 km là 12.005 VND, 100 km là 453.000 VND', () async {
      final draft = container.read(tripDraftProvider.notifier);

      draft.setDistanceKm(2.001);
      expect(
        (await container.read(quoteProvider.future)).fare,
        const Money(12005),
      );

      draft.setDistanceKm(100);
      expect(
        (await container.read(quoteProvider.future)).fare,
        const Money(453000),
      );
      expect(repository.quoteRequests.map((r) => r.km), [2.001, 100.0]);
    });

    test('đổi dịch vụ cũng hỏi lại với dịch vụ mới', () async {
      await container.read(quoteProvider.future);

      container
          .read(tripDraftProvider.notifier)
          .selectService(ServiceKind.parcel);
      final quote = await container.read(quoteProvider.future);

      expect(quote.serviceKind, ServiceKind.parcel);
      expect(repository.quoteRequests.last.kind, ServiceKind.parcel);
    });

    test('đặt lại bản nháp bằng đúng giá trị cũ không hỏi lại', () async {
      final subscription = container.listen(quoteProvider, (_, _) {});
      await container.read(quoteProvider.future);

      container.read(tripDraftProvider.notifier).setDistanceKm(5);
      await container.pump();

      expect(repository.quoteRequests, hasLength(1));
      subscription.close();
    });

    test(
      'trong lúc tính lại, AsyncLoading vẫn mang giá trị cũ (isReloading)',
      () async {
        final subscription = container.listen(quoteProvider, (_, _) {});
        await container.read(quoteProvider.future);

        container.read(tripDraftProvider.notifier).setDistanceKm(6);
        final during = container.read(quoteProvider);

        expect(during, isA<AsyncLoading<Quote>>());
        expect(during.isReloading, isTrue);
        expect(during.value?.fare, const Money(25500)); // của 5 km, đã cũ
        subscription.close();
      },
    );
  });

  group('invalidate sau lỗi', () {
    test('vẫn là AsyncError nhưng đang làm mới (isRefreshing), không phải AsyncLoading', () async {
      final reply = Completer<Quote>();
      var calls = 0;
      final repository = StubQuoteRepository(
        onQuote: (_, _) => ++calls == 1
            ? Future<Quote>.error(StateError('hỏng'))
            : reply.future,
      );
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );
      container.listen(quoteProvider, (_, _) {});
      await container.pump();
      await Future<void>.delayed(Duration.zero);
      expect(container.read(quoteProvider), isA<AsyncError<Quote>>());

      container.invalidate(quoteProvider);
      final during = container.read(quoteProvider);

      expect(during, isA<AsyncError<Quote>>());
      expect(during.isLoading, isTrue);
      expect(during.isRefreshing, isTrue);
      expect(during.isReloading, isFalse);

      reply.complete(quoteFor(5));
      await container.pump();
      expect(container.read(quoteProvider), isA<AsyncData<Quote>>());
    });
  });

  group('phản hồi về sai thứ tự', () {
    test('phản hồi của lần hỏi cũ về muộn không ghi đè kết quả mới', () async {
      final pending = <double, Completer<Quote>>{};
      final repository = StubQuoteRepository(
        onQuote: (kind, km) => (pending[km] = Completer<Quote>()).future,
      );
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );
      final seen = <AsyncValue<Quote>>[];
      container.listen(quoteProvider, (_, next) => seen.add(next));

      container.read(quoteProvider); // hỏi 5 km
      container.read(tripDraftProvider.notifier).setDistanceKm(7);
      await container.pump(); // provider tính lại: hỏi 7 km
      expect(pending.keys, [5.0, 7.0]);

      pending[7]!.complete(quoteFor(7)); // lần mới về trước
      await container.pump();
      pending[5]!.complete(quoteFor(5)); // lần cũ về muộn
      await container.pump();

      final latest = container.read(quoteProvider);
      expect(latest.value?.distanceKm, 7);
      expect(
        seen.whereType<AsyncData<Quote>>().map((d) => d.value.distanceKm),
        [7],
      );
    });
  });

  group('auto-dispose', () {
    test('hết người nghe thì state bị bỏ, nghe lại thì hỏi lại', () async {
      final repository = StubQuoteRepository();
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      final first = container.listen(quoteProvider, (_, _) {});
      await container.read(quoteProvider.future);
      expect(container.exists(quoteProvider), isTrue);

      first.close();
      await container.pump();
      expect(container.exists(quoteProvider), isFalse);

      container.listen(quoteProvider, (_, _) {});
      await container.read(quoteProvider.future);
      expect(repository.quoteRequests, hasLength(2));
    });

    test('còn người nghe thì giữ state, không hỏi lại', () async {
      final repository = StubQuoteRepository();
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      final a = container.listen(quoteProvider, (_, _) {});
      final b = container.listen(quoteProvider, (_, _) {});
      await container.read(quoteProvider.future);

      a.close();
      await container.pump();
      expect(container.exists(quoteProvider), isTrue);

      b.close();
      await container.pump();
      expect(container.exists(quoteProvider), isFalse);
      expect(repository.quoteRequests, hasLength(1));
    });

    test(
      'provider không auto-dispose thì giữ state sau khi hết người nghe',
      () async {
        final repository = StubQuoteRepository();
        final keepAlive = FutureProvider<Quote>(
          (ref) => ref
              .watch(quoteRepositoryProvider)
              .fetchQuote(serviceKind: ServiceKind.ride, distanceKm: 5),
        );
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );

        final subscription = container.listen(keepAlive, (_, _) {});
        await container.read(keepAlive.future);
        subscription.close();
        await container.pump();

        expect(container.exists(keepAlive), isTrue);
      },
    );

    test('isAutoDispose: true và .autoDispose cho cùng hành vi', () async {
      final withFlag = FutureProvider<int>((ref) => 1, isAutoDispose: true);
      final withBuilder = FutureProvider.autoDispose<int>((ref) => 1);
      final container = ProviderContainer.test();

      final subscriptions = [
        container.listen(withFlag, (_, _) {}),
        container.listen(withBuilder, (_, _) {}),
      ];
      await container.read(withFlag.future);
      await container.read(withBuilder.future);
      for (final s in subscriptions) {
        s.close();
      }
      await container.pump();

      expect(container.exists(withFlag), isFalse);
      expect(container.exists(withBuilder), isFalse);
    });

    test('ref.keepAlive() giữ state của provider auto-dispose', () async {
      final provider = FutureProvider.autoDispose<int>((ref) {
        ref.keepAlive();
        return 42;
      });
      final container = ProviderContainer.test();

      final subscription = container.listen(provider, (_, _) {});
      await container.read(provider.future);
      subscription.close();
      await container.pump();

      expect(container.exists(provider), isTrue);
    });

    test('ref.onDispose chạy cả mỗi lần provider tính lại', () async {
      var disposed = 0;
      final provider = Provider<double>((ref) {
        ref.onDispose(() => disposed++);
        return ref.watch(tripDraftProvider).distanceKm;
      });
      final container = ProviderContainer.test();
      container.listen(provider, (_, _) {});
      expect(disposed, 0);

      container.read(tripDraftProvider.notifier).setDistanceKm(6);
      await container.pump();

      expect(container.read(provider), 6);
      expect(disposed, 1); // state cũ bị dọn trước khi dựng lại
    });

    test('ref.onDispose chạy khi state bị bỏ', () async {
      var disposed = 0;
      final provider = FutureProvider.autoDispose<int>((ref) {
        ref.onDispose(() => disposed++);
        return 1;
      });
      final container = ProviderContainer.test();

      final subscription = container.listen(provider, (_, _) {});
      await container.read(provider.future);
      expect(disposed, 0);

      subscription.close();
      await container.pump();
      expect(disposed, 1);
    });
  });

  group('watch, read và listen', () {
    test('ref.watch tính lại khi nguồn đổi, ref.read thì không', () {
      final watched = Provider<double>(
        (ref) => ref.watch(tripDraftProvider).distanceKm,
      );
      final readOnce = Provider<double>(
        (ref) => ref.read(tripDraftProvider).distanceKm,
      );
      final container = ProviderContainer.test();
      expect(container.read(watched), 5);
      expect(container.read(readOnce), 5);

      container.read(tripDraftProvider.notifier).setDistanceKm(9);

      expect(container.read(watched), 9);
      expect(container.read(readOnce), 5); // vẫn là giá trị cũ
    });

    test(
      'container.read chỉ cho giá trị hiện tại, listen nhận mọi thay đổi',
      () {
        final container = ProviderContainer.test();
        final seen = <double>[];
        container.listen(
          tripDraftProvider.select((d) => d.distanceKm),
          (_, km) => seen.add(km),
        );
        final draft = container.read(tripDraftProvider.notifier);

        draft.setDistanceKm(6);
        draft.selectService(ServiceKind.food); // không đụng quãng đường
        draft.setDistanceKm(7);

        expect(seen, [6, 7]);
      },
    );
  });
}

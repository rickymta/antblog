import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show ProviderException;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_21/presentation/providers.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  test('quên ghi đè tripRepositoryProvider thì lỗi ngay, không chạy nhầm', () {
    final container = ProviderContainer.test(retry: (_, _) => null);
    expect(
      () => container.read(tripRepositoryProvider),
      throwsA(
        isA<ProviderException>().having(
          (e) => e.exception,
          'exception',
          isA<UnimplementedError>(),
        ),
      ),
    );
  });

  group('QuoteController', () {
    test('báo giá thành công', () async {
      final repository = FakeTripRepository();
      final container = containerFor(repository);
      await container.read(quoteControllerProvider.notifier).fetch();
      expect(container.read(quoteControllerProvider).value, const Money(25500));
      expect(repository.quoted, [ride5km]);
    });

    test(
      'lỗi của domain nằm trong AsyncError, switch được không cần dio',
      () async {
        final repository = FakeTripRepository()
          ..onQuote = (_) async => throw const ServiceFault(supportCode: 'x1');
        final container = containerFor(repository);
        await container.read(quoteControllerProvider.notifier).fetch();
        final error = container.read(quoteControllerProvider).error;
        expect(error, isA<ServiceFault>());
      },
    );

    test(
      'quãng đường ngoài miền: Rejected ngay, repository không được gọi',
      () async {
        final repository = FakeTripRepository();
        final container = containerFor(repository);
        container.read(quoteRequestProvider.notifier).setDistanceKm(250);
        await container.read(quoteControllerProvider.notifier).fetch();
        expect(
          container.read(quoteControllerProvider).error,
          isA<Rejected>().having(
            (r) => r.kind,
            'kind',
            RejectionKind.invalidDistance,
          ),
        );
        expect(repository.quoted, isEmpty);
      },
    );

    test('đổi yêu cầu khi đang chờ: kết quả cũ bị bỏ', () async {
      final slow = Completer<Money>();
      final repository = FakeTripRepository()..onQuote = (_) => slow.future;
      final container = containerFor(repository);
      container.listen(quoteControllerProvider, (_, _) {});
      final pending = container.read(quoteControllerProvider.notifier).fetch();
      container.read(quoteRequestProvider.notifier).setDistanceKm(8);
      slow.complete(const Money(1));
      await pending;
      expect(container.read(quoteControllerProvider).value, isNull);
    });
  });

  group('BookingController', () {
    test('đặt chuyến đi qua use case: có intentId do provider sinh', () async {
      final repository = FakeTripRepository();
      final container = containerFor(repository);
      await container.read(bookingControllerProvider.notifier).book();
      final trip = container.read(bookingControllerProvider).value!;
      expect(trip.status, TripStatus.requested);
      expect(repository.requested.single.intentId, 'book-1');
    });

    test('lỗi rồi retry: dùng lại cùng intentId, rồi thành công', () async {
      final repository = FakeTripRepository();
      repository.onRequest = (request, intentId) async {
        if (repository.requested.length == 1) {
          throw const Unreachable(UnreachableReason.noConnection);
        }
        return tripFor(request);
      };
      final container = containerFor(repository);
      final controller = container.read(bookingControllerProvider.notifier);
      await controller.book();
      expect(
        container.read(bookingControllerProvider).error,
        isA<Unreachable>(),
      );
      await controller.retry();
      expect(container.read(bookingControllerProvider).value, isA<Trip>());
      expect(repository.requested.map((r) => r.intentId), ['book-1', 'book-1']);
    });

    test(
      'quãng đường ngoài miền bị use case từ chối, repository không được gọi',
      () async {
        final repository = FakeTripRepository();
        final container = containerFor(repository);
        container.read(quoteRequestProvider.notifier).setDistanceKm(250);
        await container.read(bookingControllerProvider.notifier).book();
        expect(
          container.read(bookingControllerProvider).error,
          isA<Rejected>().having(
            (r) => r.kind,
            'kind',
            RejectionKind.invalidDistance,
          ),
        );
        expect(repository.requested, isEmpty);
      },
    );

    test(
      'làm mới đọc lại chuyến qua repository; retry sau lỗi làm mới là làm mới',
      () async {
        final repository = FakeTripRepository();
        final container = containerFor(repository);
        final controller = container.read(bookingControllerProvider.notifier);
        await controller.book();
        repository.onGet = (id) async => throw const ServiceFault();
        await controller.refresh();
        expect(
          container.read(bookingControllerProvider).error,
          isA<ServiceFault>(),
        );
        repository.onGet = null;
        await controller.retry();
        expect(container.read(bookingControllerProvider).value?.id, 'T-1');
        expect(repository.fetched, ['T-1', 'T-1']);
        expect(repository.requested, hasLength(1));
      },
    );
  });
}

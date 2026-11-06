import 'dart:async';

import 'package:dio/dio.dart' hide Response;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Request, Response;
import 'package:toi_noi_19/providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

Map<String, Object?> _quoteJson(double km) {
  final fare = calculateFare(km);
  return {
    'serviceKind': 'ride',
    'distanceMeters': (km * 1000).round(),
    'fare': fare.vnd,
    'currency': 'VND',
  };
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('QuoteController', () {
    test('5 km: AsyncData(25.500 VND), đúng calculateFare', () async {
      final container = Harness().container();
      await container.read(quoteControllerProvider.notifier).fetch();
      final quote = container.read(quoteControllerProvider);
      expect(quote.value, const Money(25500));
      expect(quote.value, calculateFare(5));
    });

    test('2,001 km: 12.005 VND', () async {
      final container = Harness().container();
      container.read(quoteFormProvider.notifier).setDistanceKm(2.001);
      await container.read(quoteControllerProvider.notifier).fetch();
      expect(container.read(quoteControllerProvider).value, const Money(12005));
    });

    test('chuyển trạng thái: null, loading, data', () async {
      final container = Harness().container();
      final seen = <AsyncValue<Money?>>[];
      container.listen(quoteControllerProvider, (_, next) => seen.add(next));
      expect(container.read(quoteControllerProvider).value, isNull);
      await container.read(quoteControllerProvider.notifier).fetch();
      expect(seen.map((v) => v.runtimeType.toString()), [
        'AsyncLoading<Money?>',
        'AsyncData<Money?>',
      ]);
      expect(seen.last.value, const Money(25500));
    });

    test('ngoài miền: AsyncError(ApiException 422)', () async {
      final harness = Harness();
      final container = harness.container();
      container.read(quoteFormProvider.notifier).setDistanceKm(100.5);
      await container.read(quoteControllerProvider.notifier).fetch();
      final quote = container.read(quoteControllerProvider);
      expect(quote.hasError, isTrue);
      expect(
        quote.error,
        isA<ApiException>().having((e) => e.statusCode, 'status', 422),
      );
      expect(
        harness.logs.last,
        matches(RegExp(r'^<-- ERROR badResponse 422 POST /quotes ')),
      );
      // Gọi trong phương thức, không trong build: không bị Riverpod thử lại.
      expect(harness.adapter.requests, hasLength(1));
    });

    test('lỗi 500 rồi bấm lại thì thành công', () async {
      final container = Harness(faults: const FaultConfig(failFirst: 1))
          .container();
      final notifier = container.read(quoteControllerProvider.notifier);
      await notifier.fetch();
      expect(container.read(quoteControllerProvider).hasError, isTrue);
      await notifier.fetch();
      expect(container.read(quoteControllerProvider).value, const Money(25500));
    });

    test('đổi form thì báo giá cũ bị bỏ', () async {
      final container = Harness().container();
      container.listen(quoteControllerProvider, (_, _) {});
      await container.read(quoteControllerProvider.notifier).fetch();
      expect(container.read(quoteControllerProvider).value, isNotNull);
      container
          .read(quoteFormProvider.notifier)
          .selectService(ServiceKind.food);
      await container.pump();
      expect(container.read(quoteControllerProvider).value, isNull);
    });

    test('đổi form khi request đang bay: response cũ bị bỏ qua', () async {
      final pending = Completer<Response>();
      final harness = Harness.custom((request) => pending.future);
      final container = harness.container();
      container.listen(quoteControllerProvider, (_, _) {});
      final inFlight = container.read(quoteControllerProvider.notifier).fetch();
      await pumpEventQueue();
      container.read(quoteFormProvider.notifier).setDistanceKm(7);
      await container.pump();
      pending.complete(okJson(_quoteJson(5)));
      await inFlight;
      // Báo giá của 5 km không được hiện cho form 7 km.
      expect(container.read(quoteControllerProvider).value, isNull);
      expect(container.read(quoteControllerProvider).isLoading, isFalse);
    });

    test(
      'hai lần bấm liên tiếp: response đến muộn không đè kết quả mới',
      () async {
        final first = Completer<Response>();
        final second = Completer<Response>();
        final queue = [first, second];
        final harness = Harness.custom((request) => queue.removeAt(0).future);
        final container = harness.container();
        container.listen(quoteControllerProvider, (_, _) {});
        final notifier = container.read(quoteControllerProvider.notifier);
        final one = notifier.fetch();
        await pumpEventQueue();
        final two = notifier.fetch();
        await pumpEventQueue();
        second.complete(okJson(_quoteJson(5)));
        await two;
        first.complete(okJson(_quoteJson(99)));
        await one;
        expect(container.read(quoteControllerProvider).value, calculateFare(5));
      },
    );

    test('payload sai hợp đồng: AsyncError(DtoFormatException)', () async {
      final harness = Harness.custom(
        (request) => okJson({'serviceKind': 'ride', 'fare': 'rất rẻ'}),
      );
      final container = harness.container();
      await container.read(quoteControllerProvider.notifier).fetch();
      expect(
        container.read(quoteControllerProvider).error,
        isA<DtoFormatException>(),
      );
    });

    test(
      'huỷ container giữa chừng: không ném lỗi khi response về muộn',
      () async {
        final pending = Completer<Response>();
        final harness = Harness.custom((request) => pending.future);
        final container = ProviderContainer(overrides: harness.overrides);
        final inFlight = container
            .read(quoteControllerProvider.notifier)
            .fetch();
        await pumpEventQueue();
        container.dispose();
        pending.complete(okJson(_quoteJson(5)));
        await expectLater(inFlight, completes);
      },
    );
  });

  group('BookingController', () {
    test(
      'đặt chuyến: Trip requested với cước đúng và khoá được gửi đi',
      () async {
        final harness = Harness();
        final container = harness.container();
        await container.read(bookingControllerProvider.notifier).book();
        final trip = container.read(bookingControllerProvider).value!;
        expect(trip.id, 'T-1');
        expect(trip.status, TripStatus.requested);
        expect(trip.fare, const Money(25500));
        expect(trip.driverId, isNull);
        expect(trip.requestedAt, fixedNow);
        expect(
          harness.adapter.requests.single.headers['idempotency-key'],
          'book-1',
        );
      },
    );

    test(
      'lỗi 500 rồi bấm lại: dùng lại đúng khoá, server chỉ có một chuyến',
      () async {
        final harness = Harness(faults: const FaultConfig(failFirst: 1));
        final container = harness.container();
        final notifier = container.read(bookingControllerProvider.notifier);
        await notifier.book();
        expect(container.read(bookingControllerProvider).hasError, isTrue);
        await notifier.book();
        expect(container.read(bookingControllerProvider).value?.id, 'T-1');
        expect(
          harness.adapter.requests.map((r) => r.headers['idempotency-key']),
          ['book-1', 'book-1'],
        );
        expect(harness.keysIssued, 1);
        expect(harness.store.trips, hasLength(1));
      },
    );

    test('response bị mất sau khi server đã tạo chuyến: gửi lại không tạo chuyến thứ hai', () async {
      var calls = 0;
      final harness = Harness(
        wrap: (fake) => (Request request) async {
          final response = await fake(request);
          if (++calls == 1) {
            // Server đã xử lý xong, nhưng kết nối đứt trước khi khách nhận được.
            throw const FormatException('kết nối đứt');
          }
          return response;
        },
      );
      final container = harness.container();
      final notifier = container.read(bookingControllerProvider.notifier);
      await notifier.book();
      expect(container.read(bookingControllerProvider).hasError, isTrue);
      expect(harness.store.trips, hasLength(1)); // server đã có chuyến
      await notifier.book();
      expect(container.read(bookingControllerProvider).value?.id, 'T-1');
      expect(harness.store.trips, hasLength(1)); // vẫn chỉ một
    });

    test('đặt thành công rồi đặt tiếp: khoá mới, chuyến mới', () async {
      final harness = Harness();
      final container = harness.container();
      final notifier = container.read(bookingControllerProvider.notifier);
      await notifier.book();
      notifier.reset();
      await notifier.book();
      expect(
        harness.adapter.requests.map((r) => r.headers['idempotency-key']),
        ['book-1', 'book-2'],
      );
      expect(harness.store.trips, hasLength(2));
    });

    test('đổi nội dung sau một lần lỗi: khoá mới, không đụng 409', () async {
      final harness = Harness(faults: const FaultConfig(failFirst: 1));
      final container = harness.container();
      final notifier = container.read(bookingControllerProvider.notifier);
      await notifier.book();
      container.read(quoteFormProvider.notifier).setDistanceKm(6);
      await notifier.book();
      expect(container.read(bookingControllerProvider).value, isNotNull);
      expect(
        harness.adapter.requests.map((r) => r.headers['idempotency-key']),
        ['book-1', 'book-2'],
      );
    });

    test('bấm đúp khi đang gửi: chỉ một request', () async {
      final pending = Completer<Response>();
      final harness = Harness.custom((request) => pending.future);
      final container = harness.container();
      final notifier = container.read(bookingControllerProvider.notifier);
      final first = notifier.book();
      await notifier.book(); // bị bỏ qua vì đang tải
      await pumpEventQueue();
      expect(harness.adapter.requests, hasLength(1));
      pending.complete(Response(500));
      await first;
    });

    test('refresh thấy tài xế đã nhận', () async {
      final harness = Harness();
      final container = harness.container();
      final notifier = container.read(bookingControllerProvider.notifier);
      await notifier.book();
      await harness.driver('D-1').acceptTrip('T-1', expectedVersion: 0);
      await notifier.refresh();
      final trip = container.read(bookingControllerProvider).value!;
      expect(trip.status, TripStatus.accepted);
      expect(trip.driverId, 'D-1');
    });

    test('refresh lỗi 500: state.value vẫn là chuyến cũ', () async {
      // Request thứ 2 (refresh) trả 500.
      final harness = Harness(faults: const FaultConfig(failEvery: 2));
      final container = harness.container();
      final notifier = container.read(bookingControllerProvider.notifier);
      await notifier.book();
      final before = container.read(bookingControllerProvider).value!;
      await notifier.refresh();
      final after = container.read(bookingControllerProvider);
      expect(after.hasError, isTrue);
      expect(after.value, before);
      await notifier.refresh();
      expect(container.read(bookingControllerProvider).hasError, isFalse);
    });

    test(
      'đã có chuyến thì book() không đặt thêm; reset() rồi mới đặt được',
      () async {
        final harness = Harness();
        final container = harness.container();
        final notifier = container.read(bookingControllerProvider.notifier);
        await notifier.book();
        await notifier.book();
        expect(harness.store.trips, hasLength(1));
        expect(harness.adapter.requests, hasLength(1));

        notifier.reset();
        expect(container.read(bookingControllerProvider).value, isNull);
        await notifier.book();
        expect(container.read(bookingControllerProvider).value?.id, 'T-2');
        expect(harness.store.trips, hasLength(2));
      },
    );

    test(
      'giới hạn: lỗi mập mờ rồi đổi form thì khoá mới, có thể ra hai chuyến',
      () async {
        // Server đã tạo chuyến nhưng response mất; khách đổi quãng đường rồi
        // đặt lại. Khoá mới nên server không nhận ra đây là cùng một lần đặt.
        var calls = 0;
        final harness = Harness(
          wrap: (fake) => (Request request) async {
            final response = await fake(request);
            if (++calls == 1) {
              throw const FormatException('kết nối đứt');
            }
            return response;
          },
        );
        final container = harness.container();
        final notifier = container.read(bookingControllerProvider.notifier);
        await notifier.book();
        container.read(quoteFormProvider.notifier).setDistanceKm(6);
        await notifier.book();
        expect(harness.store.trips, hasLength(2));
      },
    );

    test('refresh khi chưa có chuyến: không gửi request', () async {
      final harness = Harness();
      final container = harness.container();
      await container.read(bookingControllerProvider.notifier).refresh();
      expect(harness.adapter.requests, isEmpty);
    });

    test('server trả chuyến accepted không có tài xế: AsyncError(DtoFormatException)', () async {
      final harness = Harness.custom(
        (request) => okJson({
          'id': 'T-1',
          'serviceKind': 'ride',
          'status': 'accepted',
          'distanceMeters': 5000,
          'fare': 25500,
          'currency': 'VND',
          'driverId': null,
          'requestedAt': '2026-11-06T03:30:00.000Z',
          'version': 1,
        }, status: 201),
      );
      final container = harness.container();
      await container.read(bookingControllerProvider.notifier).book();
      expect(
        container.read(bookingControllerProvider).error,
        isA<DtoFormatException>().having((e) => e.field, 'field', 'driverId'),
      );
    });
  });

  group('wiring của provider', () {
    test('địa chỉ mặc định là localhost:8787 khi không có --dart-define', () {
      final container = ProviderContainer.test();
      expect(
        container.read(apiBaseUrlProvider),
        Uri.parse('http://localhost:8787'),
      );
    });

    test(
      'interceptor log có trong client do provider dựng, và không lộ khoá',
      () async {
        final harness = Harness();
        final container = harness.container();
        await container.read(bookingControllerProvider.notifier).book();
        final all = harness.logs.join('\n');
        expect(all, contains('--> POST /trips'));
        expect(all, contains('<-- 201 POST /trips'));
        expect(all, contains('idempotency-key: [redacted]'));
        expect(all, isNot(contains('book-1')));
      },
    );

    test('đóng container thì client bị đóng', () async {
      final harness = Harness();
      final container = ProviderContainer(overrides: harness.overrides);
      final client = container.read(apiClientProvider);
      container.dispose();
      await expectLater(
        client.quote(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        ),
        throwsA(isA<DioException>()),
      );
    });
  });

  group('flutter test chặn HTTP thật', () {
    test('dio với adapter mặc định nhận 400 và không có kết nối nào', () async {
      final dio = Dio();
      await expectLater(
        dio.get<Object?>('http://localhost:8787/health'),
        throwsA(
          isA<DioException>()
              .having((e) => e.type, 'type', DioExceptionType.badResponse)
              .having((e) => e.response?.statusCode, 'status', 400),
        ),
      );
    });

    test(
      'quên override adapter: provider thật nhận ApiException 400',
      () async {
        final container = ProviderContainer.test();
        await container.read(quoteControllerProvider.notifier).fetch();
        expect(
          container.read(quoteControllerProvider).error,
          isA<ApiException>().having((e) => e.statusCode, 'status', 400),
        );
      },
    );
  });
}

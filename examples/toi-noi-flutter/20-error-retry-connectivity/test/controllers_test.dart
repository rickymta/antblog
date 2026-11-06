import 'dart:async';

import 'package:dio/dio.dart' hide Response;
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Response;
import 'package:toi_noi_20/providers.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group(
    'QuoteController: retry nằm dưới, controller chỉ thấy kết quả cuối',
    () {
      test(
        '500 hai lần rồi thành công: AsyncData 25.500 VND, ba request',
        () async {
          final harness = Harness(faults: const FaultConfig(failFirst: 2));
          final container = harness.container();
          await container.read(quoteControllerProvider.notifier).fetch();
          expect(
            container.read(quoteControllerProvider).value,
            const Money(25500),
          );
          expect(harness.inProcess.requests, hasLength(3));
          expect(harness.waits, [
            const Duration(milliseconds: 500),
            const Duration(seconds: 1),
          ]);
          expect(harness.logs.where((l) => l.startsWith('retry:')), [
            'retry: lần 2/3 sau 500 ms, vì badResponse',
            'retry: lần 3/3 sau 1000 ms, vì badResponse',
          ]);
        },
      );

      test(
        'server hỏng hẳn: AsyncError(ServerFailure) sau đúng 3 request',
        () async {
          final harness = Harness(faults: const FaultConfig(failFirst: 99));
          final container = harness.container();
          await container.read(quoteControllerProvider.notifier).fetch();
          final quote = container.read(quoteControllerProvider);
          expect(
            quote.error,
            isA<ServerFailure>()
                .having((f) => f.statusCode, 'status', 500)
                .having((f) => f.traceId, 'traceId', 'fake-000003'),
          );
          expect(harness.inProcess.requests, hasLength(3));
        },
      );

      test('422: ClientFailure, một request, không chờ backoff', () async {
        final harness = Harness();
        final container = harness.container();
        container.read(quoteFormProvider.notifier).setDistanceKm(100.5);
        await container.read(quoteControllerProvider.notifier).fetch();
        expect(
          container.read(quoteControllerProvider).error,
          isA<ClientFailure>().having(
            (f) => f.fieldErrors.keys,
            'fields',
            contains('distanceKm'),
          ),
        );
        expect(harness.inProcess.requests, hasLength(1));
        expect(harness.waits, isEmpty);
      });

      test('mất kết nối ở cả ba lần: NetworkFailure', () async {
        final harness = Harness(
          script: [
            NetworkFault.refuse,
            NetworkFault.refuse,
            NetworkFault.refuse,
          ],
        );
        final container = harness.container();
        await container.read(quoteControllerProvider.notifier).fetch();
        expect(
          container.read(quoteControllerProvider).error,
          isA<NetworkFailure>(),
        );
        expect(harness.faulty.calls, 3);
      });

      test(
        '200 nhưng nội dung sai hợp đồng: ParseFailure, không thử lại',
        () async {
          final harness = Harness.custom((request) => okJson({'oops': true}));
          final container = harness.container();
          await container.read(quoteControllerProvider.notifier).fetch();
          expect(
            container.read(quoteControllerProvider).error,
            isA<ParseFailure>().having((f) => f.dto, 'dto', 'QuoteDto'),
          );
          expect(harness.inProcess.requests, hasLength(1));
        },
      );

      test('trang HTML của captive portal với mã 200: ParseFailure', () async {
        final harness = Harness.custom(
          (request) => Response.ok(
            '<html>Đăng nhập Wi-Fi</html>',
            headers: {'content-type': 'text/html'},
          ),
        );
        final container = harness.container();
        await container.read(quoteControllerProvider.notifier).fetch();
        expect(
          container.read(quoteControllerProvider).error,
          isA<ParseFailure>(),
        );
      });

      test(
        'lỗi lập trình không bị che thành ApiFailure và không bị thử lại',
        () async {
          final harness = Harness.custom(
            (request) => throw StateError('bug trong handler'),
          );
          final container = harness.container();
          await container.read(quoteControllerProvider.notifier).fetch();
          final error = container.read(quoteControllerProvider).error;
          expect(error, isNot(isA<ApiFailure>()));
          expect(error, isA<DioException>());
          expect(harness.inProcess.requests, hasLength(1));
        },
      );
    },
  );

  group('timeout trên đồng hồ giả', () {
    test('server treo cả ba lần: hạn tổng 30 giây chặn lần thử cuối', () {
      final harness = Harness(
        script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
        realSleep: true,
      );
      final container = harness.container();
      final watch = Stopwatch()..start();
      final run = runFake(
        () => container.read(quoteControllerProvider.notifier).fetch(),
      );
      expect(run.done, isTrue);
      final error = container.read(quoteControllerProvider).error;
      expect(
        error,
        isA<TimeoutFailure>()
            .having((f) => f.phase, 'phase', TimeoutPhase.receive)
            .having(
              (f) => f.limit,
              'limit',
              const Duration(seconds: 8, milliseconds: 500),
            ),
      );
      // Lần 1 tới 10 s, chờ 0,5 s, lần 2 tới 20,5 s, chờ 1 s: còn 8,5 s trong
      // hạn tổng 30 s nên lần 3 bị chặn receiveTimeout xuống 8,5 s.
      expect(run.elapsed, const Duration(seconds: 30));
      expect(harness.faulty.calls, 3);
      expect(watch.elapsed, lessThan(const Duration(seconds: 5)));
    });

    test('timeout ngắn hơn thì ngân sách ngắn hơn', () {
      final harness = Harness(
        script: [NetworkFault.hang],
        config: const NetworkConfig(
          receiveTimeout: Duration(seconds: 2),
          maxAttempts: 1,
        ),
        realSleep: true,
      );
      final container = harness.container();
      final run = runFake(
        () => container.read(quoteControllerProvider.notifier).fetch(),
      );
      expect(run.elapsed, const Duration(seconds: 2));
      expect(
        container.read(quoteControllerProvider).error,
        isA<TimeoutFailure>(),
      );
    });
  });

  group(
    'BookingController: retry của interceptor cộng với Idempotency-Key',
    () {
      test('mất response sau khi server tạo chuyến: vẫn một chuyến', () async {
        final harness = Harness(script: [NetworkFault.loseResponse]);
        final container = harness.container();
        await container.read(bookingControllerProvider.notifier).book();
        expect(container.read(bookingControllerProvider).value?.id, 'T-1');
        expect(harness.store.trips, hasLength(1));
        expect(harness.keysIssued, 1);
        expect(
          harness.inProcess.requests.map((r) => r.headers['idempotency-key']),
          ['book-1', 'book-1'],
        );
      });

      test('hết lượt thử rồi bấm Thử lại: dùng lại khoá, một chuyến', () async {
        // 3 request đầu lỗi 500 (hết lượt), request thứ 4 (lần bấm lại) thành công.
        final harness = Harness(faults: const FaultConfig(failFirst: 3));
        final container = harness.container();
        final notifier = container.read(bookingControllerProvider.notifier);
        await notifier.book();
        expect(
          container.read(bookingControllerProvider).error,
          isA<ServerFailure>(),
        );
        await notifier.retry();
        expect(container.read(bookingControllerProvider).value?.id, 'T-1');
        expect(harness.keysIssued, 1);
        expect(harness.store.trips, hasLength(1));
        expect(harness.inProcess.requests, hasLength(4));
      });

      test(
        'retry() sau khi làm mới thất bại làm mới lại, không đặt chuyến mới',
        () async {
          // Request 1 đặt chuyến; 2-4 làm mới lỗi (hết lượt); 5 làm mới lại thành công.
          final harness = Harness(
            faults: const FaultConfig(failFirst: 0, failEvery: 0),
            wrap: (fake) {
              var calls = 0;
              return (request) {
                calls++;
                return calls >= 2 && calls <= 4
                    ? Response(500, body: '{"title":"x","status":500}')
                    : fake(request);
              };
            },
          );
          final container = harness.container();
          final notifier = container.read(bookingControllerProvider.notifier);
          await notifier.book();
          await notifier.refresh();
          expect(
            container.read(bookingControllerProvider).error,
            isA<ServerFailure>(),
          );
          await notifier.retry();
          expect(container.read(bookingControllerProvider).value?.id, 'T-1');
          expect(harness.store.trips, hasLength(1));
          expect(harness.inProcess.requests.map((r) => r.method).toList(), [
            'POST',
            'GET',
            'GET',
            'GET',
            'GET',
          ]);
        },
      );

      test('đổi nội dung sau lỗi: khoá mới', () async {
        final harness = Harness(faults: const FaultConfig(failFirst: 3));
        final container = harness.container();
        final notifier = container.read(bookingControllerProvider.notifier);
        await notifier.book();
        container.read(quoteFormProvider.notifier).setDistanceKm(6);
        await notifier.book();
        expect(harness.keysIssued, 2);
        expect(harness.store.trips, hasLength(1));
      });
    },
  );

  group('huỷ request không phải thất bại', () {
    test('đổi form khi báo giá đang bay: request cũ bị huỷ, không có lỗi', () {
      final never = Completer<Response>();
      final harness = Harness.custom((request) => never.future);
      final container = harness.container();
      final run = runFake(() async {
        final pending = container
            .read(quoteControllerProvider.notifier)
            .fetch();
        container.read(quoteFormProvider.notifier).setDistanceKm(6);
        await pending;
      });
      expect(run.done, isTrue);
      expect(run.error, isNull);
      final quote = container.read(quoteControllerProvider);
      expect(quote.error, isNull);
      expect(quote.value, isNull);
    });

    test('bấm Báo giá lần hai: lần một bị huỷ, kết quả là của lần hai', () {
      var calls = 0;
      final first = Completer<Response>();
      final harness = Harness.custom((request) {
        calls++;
        return calls == 1
            ? first.future
            : okJson({
                'serviceKind': 'ride',
                'distanceMeters': 5000,
                'fare': 25500,
                'currency': 'VND',
              });
      });
      final container = harness.container();
      final run = runFake(() async {
        final notifier = container.read(quoteControllerProvider.notifier);
        final one = notifier.fetch();
        final two = notifier.fetch();
        await Future.wait([one, two]);
      });
      expect(run.done, isTrue);
      expect(run.error, isNull);
      expect(container.read(quoteControllerProvider).value, const Money(25500));
    });

    test('đóng container khi request đang bay: fetch kết thúc êm', () {
      final never = Completer<Response>();
      final harness = Harness.custom((request) => never.future);
      final container = harness.container();
      final run = runFake(() async {
        final pending = container
            .read(quoteControllerProvider.notifier)
            .fetch();
        container.dispose();
        await pending;
      });
      expect(run.done, isTrue);
      expect(run.error, isNull);
    });
  });

  group('429 và Retry-After qua cả tầng controller', () {
    test(
      '429 một lần rồi thành công: không lỗi, chờ theo Retry-After',
      () async {
        var calls = 0;
        final harness = Harness.custom(
          (request) => ++calls == 1
              ? Response(
                  429,
                  body: '{"title":"x","status":429}',
                  headers: {'retry-after': '2'},
                )
              : okJson({
                  'serviceKind': 'ride',
                  'distanceMeters': 5000,
                  'fare': 25500,
                  'currency': 'VND',
                }),
        );
        final container = harness.container();
        await container.read(quoteControllerProvider.notifier).fetch();
        expect(
          container.read(quoteControllerProvider).value,
          const Money(25500),
        );
        expect(harness.waits, [const Duration(seconds: 2)]);
      },
    );

    test('429 mãi mãi: ClientFailure retryable, đủ ba request', () async {
      final harness = Harness.custom(
        (request) => Response(429, body: '{"title":"x","status":429}'),
      );
      final container = harness.container();
      await container.read(quoteControllerProvider.notifier).fetch();
      expect(
        container.read(quoteControllerProvider).error,
        isA<ClientFailure>()
            .having((f) => f.statusCode, 'status', 429)
            .having((f) => f.retryable, 'retryable', isTrue),
      );
      expect(harness.inProcess.requests, hasLength(3));
    });
  });
}

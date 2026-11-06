import 'package:dio/dio.dart' show DioException, DioExceptionType;
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Response;
import 'package:toi_noi_21/data/mappers.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

TypeMatcher<Unreachable> _unreachable(UnreachableReason reason) =>
    isA<Unreachable>().having((f) => f.reason, 'reason', reason);

TypeMatcher<Rejected> _rejected(RejectionKind kind) =>
    isA<Rejected>().having((f) => f.kind, 'kind', kind);

void main() {
  group('đổi DTO sang domain', () {
    for (final (km, vnd) in [(5.0, 25500), (2.001, 12005), (100.0, 453000)]) {
      test('báo giá $km km là $vnd VND', () async {
        final harness = Harness();
        final fare = await harness.repository.quote(
          QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: km),
        );
        expect(fare, Money(vnd));
        expect(fare, calculateFare(km));
      });
    }

    test('đặt chuyến trả Trip của domain và gửi intentId làm khoá', () async {
      final harness = Harness();
      final trip = await harness.repository.requestTrip(
        food3km,
        intentId: 'intent-7',
      );
      expect(trip.id, 'T-1');
      expect(trip.serviceKind, ServiceKind.food);
      expect(trip.status, TripStatus.requested);
      expect(trip.distanceKm, 3);
      expect(trip.fare, calculateFare(3));
      expect(trip.driverId, isNull);
      expect(trip.requestedAt, fixedNow);
      expect(
        harness.inProcess.requests.single.headers['idempotency-key'],
        'intent-7',
      );
      expect(
        harness.inProcess.requests.single.body,
        contains('"distanceKm":3.0'),
      );
    });

    test('đọc lại chuyến đã đặt', () async {
      final harness = Harness();
      final booked = await harness.repository.requestTrip(
        ride5km,
        intentId: 'k1',
      );
      expect(await harness.repository.getTrip(booked.id), booked);
    });
  });

  group('đổi lỗi kỹ thuật sang lỗi domain', () {
    test(
      'server từ chối kết nối: Unreachable.noConnection, thử lại được',
      () async {
        final harness = Harness(script: [NetworkFault.refuse]);
        await expectLater(
          harness.repository.quote(ride5km),
          throwsA(
            _unreachable(UnreachableReason.noConnection)
                .having((f) => f.canRetryByUser, 'canRetryByUser', isTrue),
          ),
        );
      },
    );

    test('quá hạn kết nối và quá hạn nhận: Unreachable.tooSlow', () async {
      final harness = Harness(
        script: [NetworkFault.connectTimeout, NetworkFault.hang],
      );
      await expectLater(
        harness.repository.quote(ride5km),
        throwsA(_unreachable(UnreachableReason.tooSlow)),
      );
      await expectLater(
        harness.repository.quote(ride5km),
        throwsA(_unreachable(UnreachableReason.tooSlow)),
      );
    });

    test(
      'mất response: Unreachable.noConnection (server có thể đã xử lý xong)',
      () async {
        final harness = Harness(script: [NetworkFault.loseResponse]);
        await expectLater(
          harness.repository.requestTrip(ride5km, intentId: 'k1'),
          throwsA(_unreachable(UnreachableReason.noConnection)),
        );
        expect(harness.store.trips, hasLength(1));
      },
    );

    test(
      '500: ServiceFault mang mã hỗ trợ, không mang title của server',
      () async {
        final harness = Harness(faults: const FaultConfig(failFirst: 1));
        await expectLater(
          harness.repository.quote(ride5km),
          throwsA(
            isA<ServiceFault>()
                .having((f) => f.supportCode, 'supportCode', 'fake-000001')
                .having(
                  (f) => f.toString(),
                  'toString',
                  isNot(contains('Unexpected')),
                ),
          ),
        );
      },
    );

    test('422 trên distanceKm: Rejected.invalidDistance', () async {
      final harness = Harness();
      await expectLater(
        harness.repository.quote(
          const QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: 150),
        ),
        throwsA(_rejected(RejectionKind.invalidDistance)),
      );
    });

    test('404: Rejected.notFound', () async {
      final harness = Harness();
      await expectLater(
        harness.repository.getTrip('khong-co'),
        throwsA(_rejected(RejectionKind.notFound)),
      );
    });

    test('409 khi cùng intentId mà nội dung khác: Rejected.conflict', () async {
      final harness = Harness();
      await harness.repository.requestTrip(ride5km, intentId: 'k1');
      await expectLater(
        harness.repository.requestTrip(food3km, intentId: 'k1'),
        throwsA(_rejected(RejectionKind.conflict)),
      );
    });

    test('4xx khác: Rejected.other, không thử lại được', () async {
      final harness = Harness(
        wrap: (fake) =>
            (request) async => Response(
              418,
              body: '{"title":"teapot","status":418}',
              headers: const {'content-type': 'application/problem+json'},
            ),
      );
      await expectLater(
        harness.repository.quote(ride5km),
        throwsA(
          _rejected(RejectionKind.other)
              .having((f) => f.canRetryByUser, 'canRetryByUser', isFalse),
        ),
      );
    });

    test(
      '200 mà nội dung sai hợp đồng (trang đăng nhập Wi-Fi): Unreadable',
      () async {
        final harness = Harness(
          wrap: (fake) =>
              (request) async => Response.ok(
                '<html>Đăng nhập Wi-Fi</html>',
                headers: const {'content-type': 'text/html'},
              ),
        );
        await expectLater(
          harness.repository.quote(ride5km),
          throwsA(isA<Unreadable>()),
        );
      },
    );

    test('lỗi lập trình đi qua nguyên vẹn: intentId sai định dạng là ArgumentError', () async {
      final harness = Harness();
      await expectLater(
        harness.repository.requestTrip(ride5km, intentId: 'có dấu cách'),
        throwsArgumentError,
      );
      expect(harness.faulty.calls, 0);
    });

    test('lỗi lạ trong adapter không bị che: lên dưới dạng DioException(unknown) bọc StateError', () async {
      final harness = Harness(
        wrap: (fake) =>
            (request) async => throw StateError('hỏng trong handler'),
      );
      await expectLater(
        harness.repository.quote(ride5km),
        throwsA(
          allOf(
            isNot(isA<TripFailure>()),
            isA<DioException>()
                .having((e) => e.type, 'type', DioExceptionType.unknown)
                .having((e) => e.error, 'error', isA<StateError>()),
          ),
        ),
      );
    });
  });

  group('ApiFailureMapper', () {
    test('chứng chỉ sai: Unreachable.insecure, không thử lại được', () {
      final failure = const NetworkFailure(reason: NetworkReason.badCertificate)
          .toTripFailure();
      expect(failure, _unreachable(UnreachableReason.insecure));
      expect(failure.canRetryByUser, isFalse);
    });

    test('giữ lỗi gốc trong cause để ghi log', () {
      const original = ServerFailure(
        statusCode: 503,
        title: 'Busy',
        traceId: 't-9',
      );
      final failure = original.toTripFailure();
      expect(failure.cause, same(original));
      expect(failure.supportCode, 't-9');
    });

    test('mọi lỗi của domain nói đúng về nút Thử lại', () {
      expect(const ServiceFault().canRetryByUser, isTrue);
      expect(const Unreadable().canRetryByUser, isTrue);
      expect(const Rejected(RejectionKind.other).canRetryByUser, isFalse);
    });
  });
}

import 'package:test/test.dart';
import 'package:toi_noi_27/booking/booking.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/doubles.dart';

/// Test use case bằng fake (repository chạy thật trong bộ nhớ), spy (nhật ký
/// chỉ ghi) và dummy (chỗ trống không được chạm). Không dòng nào dựng kỳ vọng
/// về cách use case gọi repository: ta kiểm kết quả mà khách và server thấy.
void main() {
  late InMemoryTripRepository server;
  late SpyBookingLog log;
  late int issued;
  late RequestTrip requestTrip;

  setUp(() {
    server = InMemoryTripRepository();
    log = SpyBookingLog();
    issued = 0;
    requestTrip = RequestTrip(
      server,
      log,
      newIntentId: () => 'intent-${++issued}',
    );
  });

  test(
    'đặt được: chuyến requested, cước 25.500, server có đúng một chuyến',
    () async {
      final trip = await requestTrip(ride5km);
      expect(trip.status, TripStatus.requested);
      expect(trip.fare, calculateFare(5));
      expect(trip.fare, const Money(25500));
      expect(server.trips, [trip]);
      expect(log.bookedTrips, [trip]);
      expect(log.failedAttempts, isEmpty);
    },
  );

  group('ngoài miền: không chạm repository, không chạm nhật ký', () {
    for (final km in [-0.001, 100.001, double.nan, double.infinity]) {
      test('$km km bị từ chối', () async {
        // Dummy: nếu use case lỡ gọi, test hỏng bằng StateError chứ không
        // bằng Rejected, nên không thể xanh nhầm.
        final guarded = RequestTrip(
          DummyTripRepository(),
          DummyBookingLog(),
          newIntentId: () => throw StateError('không được sinh intentId'),
        );
        await expectLater(
          guarded(QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: km)),
          throwsA(
            isA<Rejected>().having(
              (r) => r.kind,
              'kind',
              RejectionKind.invalidDistance,
            ),
          ),
        );
      });
    }

    for (final km in [0.0, 100.0]) {
      test('biên $km km được chấp nhận', () async {
        final trip = await requestTrip(
          QuoteRequest(serviceKind: ServiceKind.parcel, distanceKm: km),
        );
        expect(trip.fare, calculateFare(km));
      });
    }
  });

  group('một ý định, một chuyến', () {
    test(
      'mất response sau khi server đã tạo chuyến: thử lại vẫn chỉ một chuyến',
      () async {
        server.failNext(const Unreachable(), afterStore: true);
        await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
        final trip = await requestTrip(ride5km);
        expect(server.trips, [trip]);
        expect(server.sentIntentIds, ['intent-1', 'intent-1']);
        expect(log.failedAttempts.map((f) => f.failure), [isA<Unreachable>()]);
        expect(log.bookedTrips, [trip]);
      },
    );

    test(
      'request chưa tới server: thử lại dùng lại intentId, đặt được',
      () async {
        server.failNext(const ServiceFault());
        await expectLater(requestTrip(ride5km), throwsA(isA<ServiceFault>()));
        await requestTrip(ride5km);
        expect(server.sentIntentIds, ['intent-1', 'intent-1']);
        expect(server.trips, hasLength(1));
      },
    );

    test('đổi yêu cầu sau lỗi là ý định mới', () async {
      server.failNext(const Unreachable());
      await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
      await requestTrip(food3km);
      expect(server.sentIntentIds, ['intent-1', 'intent-2']);
    });

    test('đặt thành công rồi đặt lại cùng yêu cầu là chuyến thứ hai', () async {
      final first = await requestTrip(ride5km);
      final second = await requestTrip(ride5km);
      expect(second.id, isNot(first.id));
      expect(server.trips, hasLength(2));
    });

    test('hai lần bấm đồng thời cùng yêu cầu chung một intentId', () async {
      final results = await Future.wait([
        requestTrip(ride5km),
        requestTrip(ride5km),
      ]);
      expect(results[0], results[1]);
      expect(server.sentIntentIds, ['intent-1', 'intent-1']);
      expect(server.trips, hasLength(1));
    });
  });

  test(
    'server từ chối conflict: lỗi lên nguyên vẹn và được ghi nhật ký',
    () async {
      // Dựng tình huống: server đã có intent-1 với nội dung khác.
      await server.requestTrip(food3km, intentId: 'intent-1');
      await expectLater(
        requestTrip(ride5km),
        throwsA(
          isA<Rejected>().having((r) => r.kind, 'kind', RejectionKind.conflict),
        ),
      );
      expect(log.failedAttempts.single.request, ride5km);
      expect(log.bookedTrips, isEmpty);
    },
  );
}

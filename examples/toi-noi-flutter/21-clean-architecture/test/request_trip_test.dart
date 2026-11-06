import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Test của use case chỉ cần repository giả: không dio, không adapter.
void main() {
  late FakeTripRepository repository;
  late int issued;
  late RequestTrip requestTrip;

  setUp(() {
    repository = FakeTripRepository();
    issued = 0;
    requestTrip = RequestTrip(
      repository,
      newIntentId: () => 'intent-${++issued}',
    );
  });

  test(
    'đặt được: chuyến requested, cước đúng bảng giá, gửi kèm intentId',
    () async {
      final trip = await requestTrip(ride5km);
      expect(trip.status, TripStatus.requested);
      expect(trip.fare, const Money(25500));
      expect(repository.requested, [(request: ride5km, intentId: 'intent-1')]);
    },
  );

  for (final km in [-0.001, 100.001, double.nan, double.infinity]) {
    test(
      'quãng đường $km km bị từ chối, không tốn request, không sinh intentId',
      () async {
        final request = QuoteRequest(
          serviceKind: ServiceKind.ride,
          distanceKm: km,
        );
        await expectLater(
          requestTrip(request),
          throwsA(
            isA<Rejected>().having(
              (r) => r.kind,
              'kind',
              RejectionKind.invalidDistance,
            ),
          ),
        );
        expect(repository.requested, isEmpty);
        expect(issued, 0);
      },
    );
  }

  for (final km in [0.0, 100.0]) {
    test('biên $km km được chấp nhận', () async {
      final trip = await requestTrip(
        QuoteRequest(serviceKind: ServiceKind.parcel, distanceKm: km),
      );
      expect(trip.distanceKm, km);
      expect(trip.fare, calculateFare(km));
    });
  }

  test('gửi lại sau lỗi dùng lại đúng intentId', () async {
    repository.onRequest = (request, intentId) async {
      if (repository.requested.length == 1) {
        throw const Unreachable(UnreachableReason.noConnection);
      }
      return tripFor(request);
    };
    await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
    await requestTrip(ride5km);
    expect(repository.requested.map((r) => r.intentId), [
      'intent-1',
      'intent-1',
    ]);
    expect(issued, 1);
  });

  test('đổi yêu cầu sau lỗi thì là ý định mới', () async {
    repository.onRequest = (request, intentId) async {
      if (repository.requested.length == 1) {
        throw const ServiceFault();
      }
      return tripFor(request);
    };
    await expectLater(requestTrip(ride5km), throwsA(isA<ServiceFault>()));
    await requestTrip(food3km);
    expect(repository.requested.map((r) => r.intentId), [
      'intent-1',
      'intent-2',
    ]);
  });

  test(
    'đặt thành công rồi đặt lại cùng yêu cầu là ý định mới (chuyến thứ hai)',
    () async {
      await requestTrip(ride5km);
      await requestTrip(ride5km);
      expect(repository.requested.map((r) => r.intentId), [
        'intent-1',
        'intent-2',
      ]);
    },
  );

  test('hai lần bấm đồng thời cùng yêu cầu dùng chung một intentId', () async {
    final gate = Completer<void>();
    repository.onRequest = (request, intentId) async {
      await gate.future;
      return tripFor(request);
    };
    final first = requestTrip(ride5km);
    final second = requestTrip(ride5km);
    gate.complete();
    await Future.wait([first, second]);
    expect(repository.requested.map((r) => r.intentId), [
      'intent-1',
      'intent-1',
    ]);
    expect(issued, 1);
  });

  test(
    'lỗi lập trình đi qua nguyên vẹn, không bị đổi thành TripFailure',
    () async {
      repository.onRequest = (_, _) async =>
          throw StateError('lỗi trong repository');
      await expectLater(requestTrip(ride5km), throwsStateError);
    },
  );
}

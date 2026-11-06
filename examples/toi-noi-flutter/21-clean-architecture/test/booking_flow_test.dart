import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Use case + data thật + fake API: khoá ý định của domain thành
/// `Idempotency-Key` của HTTP, nên mất response không sinh chuyến thứ hai.
void main() {
  test(
    'mất response rồi gửi lại: server nhận hai request, chỉ có một chuyến',
    () async {
      final harness = Harness(script: [NetworkFault.loseResponse]);
      var n = 0;
      final requestTrip = RequestTrip(
        harness.repository,
        newIntentId: () => 'intent-${++n}',
      );

      await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
      final trip = await requestTrip(ride5km);

      expect(trip.id, 'T-1');
      expect(harness.inProcess.requests, hasLength(2));
      expect(
        harness.inProcess.requests.map((r) => r.headers['idempotency-key']),
        ['intent-1', 'intent-1'],
      );
      expect(harness.store.trips, hasLength(1));
      expect(n, 1);
    },
  );

  test(
    'đổi yêu cầu giữa hai lần thử thì khoá đổi theo, không bị 409',
    () async {
      final harness = Harness(script: [NetworkFault.refuse]);
      var n = 0;
      final requestTrip = RequestTrip(
        harness.repository,
        newIntentId: () => 'intent-${++n}',
      );
      await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
      final trip = await requestTrip(food3km);
      expect(trip.serviceKind.name, 'food');
      expect(harness.store.trips, hasLength(1));
    },
  );

  test('giới hạn đã biết: mất response rồi đổi yêu cầu, ý định cũ bị quên (hai chuyến)', () async {
    final harness = Harness(script: [NetworkFault.loseResponse]);
    var n = 0;
    final requestTrip = RequestTrip(
      harness.repository,
      newIntentId: () => 'intent-${++n}',
    );
    await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
    await requestTrip(food3km);
    expect(harness.store.trips, hasLength(2));
    // Quay lại yêu cầu đầu: ý định của nó đã bị quên, thành chuyến thứ ba.
    await requestTrip(ride5km);
    expect(harness.store.trips, hasLength(3));
  });

  test('quãng đường ngoài miền không tốn request nào', () async {
    final harness = Harness();
    final requestTrip = RequestTrip(harness.repository, newIntentId: () => 'k');
    await expectLater(
      requestTrip(
        const QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: 101),
      ),
      throwsA(isA<Rejected>()),
    );
    expect(harness.faulty.calls, 0);
  });
}

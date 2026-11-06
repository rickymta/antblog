import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import '../test_support/driver_simulator.dart';
import '../test_support/harness.dart';

void main() {
  test('tên header chép trong test_support khớp với fake API', () {
    expect(driverIdHeader, fakeDriverIdHeader);
  });

  test('tài xế thứ hai nhận cùng chuyến thì bị 409', () async {
    final h = Harness();
    final customer = ToiNoiApiClient(baseUrl: fakeBaseUrl, adapter: h.adapter);
    final trip = await customer.createTrip(
      const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
      idempotencyKey: 'k-1',
    );

    await h.driver.accept(trip.id);
    final second = DriverSimulator(
      baseUrl: fakeBaseUrl,
      adapter: h.adapter,
      driverId: 'D-02',
    );

    await expectLater(
      second.accept(trip.id),
      throwsA(isA<ApiException>().having((e) => e.statusCode, 'status', 409)),
    );
    expect((await customer.getTrip(trip.id)).driverId, 'D-01');
  });
}

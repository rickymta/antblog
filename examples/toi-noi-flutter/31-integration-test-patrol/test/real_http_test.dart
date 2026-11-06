import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import '../test_support/driver_simulator.dart';

/// Đường đi mà test trên thiết bị dùng: HTTP thật tới một server thật (ở đây
/// là fake API mở trên loopback), không qua `InProcessAdapter`.
///
/// Dùng `test()` thuần, không `testWidgets`: binding của flutter_test chặn
/// HTTP thật (xem README của tools/fake_api), mà `test()` thì chưa khởi tạo nó.
void main() {
  test('DriverSimulator nhận chuyến của app khách qua HTTP thật', () async {
    final server = await shelf_io.serve(
      createFakeApiHandler(newId: () => 'T-real-1'),
      'localhost',
      0,
    );
    addTearDown(() => server.close(force: true));
    final baseUrl = Uri(scheme: 'http', host: 'localhost', port: server.port);
    final customer = ToiNoiApiClient(baseUrl: baseUrl);
    final driver = DriverSimulator(baseUrl: baseUrl);
    addTearDown(customer.close);
    addTearDown(driver.close);

    final created = await customer.createTrip(
      const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
      idempotencyKey: 'real-1',
    );
    expect(created.status, TripStatus.requested);
    expect(created.driverId, isNull);

    await driver.accept(created.id);

    final after = await customer.getTrip(created.id);
    expect(after.status, TripStatus.accepted);
    expect(after.driverId, 'D-01');
    expect(after.fareVnd, 25500);
  });
}

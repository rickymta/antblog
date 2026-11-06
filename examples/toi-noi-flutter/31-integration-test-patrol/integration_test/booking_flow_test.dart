import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:toi_noi_31/api_base_url.dart';
import 'package:toi_noi_31/bootstrap.dart';
import 'package:toi_noi_31/providers.dart';

import '../test_support/booking_flow.dart';
import '../test_support/driver_simulator.dart';
import '../test_support/fake_notification_permission.dart';

/// Chạy trên simulator, emulator hoặc máy thật, gọi fake API THẬT qua HTTP
/// (`dart run bin/server.dart` trong tools/fake_api).
///
///   flutter test integration_test/booking_flow_test.dart -d THIẾT_BỊ
///
/// `integration_test` thuần không bấm được hộp thoại quyền của hệ điều hành,
/// nên quyền thông báo ở đây là bản giả. Bản bấm được là `patrol_test/`.
void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('đặt chuyến 5 km với fake API chạy trên máy dev', (tester) async {
    final driver = DriverSimulator(baseUrl: resolveApiBaseUrl());
    addTearDown(driver.close);

    await tester.pumpWidget(
      buildApp(
        overrides: [
          notificationPermissionProvider.overrideWithValue(
            FakeNotificationPermission(),
          ),
        ],
      ),
    );
    // Android phải chuyển bề mặt Flutter thành ảnh trước khi chụp; iOS bỏ qua.
    await binding.convertFlutterSurfaceToImage();

    final outcome = await runBookingFlow(
      tester,
      driverAccepts: driver.accept,
      timeout: const Duration(seconds: 30),
      screenshot: (name) async {
        await tester.pump();
        await binding.takeScreenshot(name);
      },
    );

    expect(outcome.fare.vnd, 25500);
    expect(outcome.driverId, 'D-01');
  });
}

import 'package:flutter_test/flutter_test.dart';
import 'package:patrol/patrol.dart';
import 'package:toi_noi_31/api_base_url.dart';
import 'package:toi_noi_31/bootstrap.dart';
import 'package:toi_noi_31/providers.dart';

import '../test_support/booking_flow.dart';
import '../test_support/driver_simulator.dart';
import '../test_support/fake_notification_permission.dart';

/// Chạy bằng `patrol test` (không phải `flutter test`): xem README của lab.
/// Cần fake API chạy trên máy dev: `dart run bin/server.dart` trong tools/fake_api.
void main() {
  patrolTest('đặt chuyến và bấm Allow ở hộp thoại thông báo của hệ điều hành', (
    $,
  ) async {
    final driver = DriverSimulator(baseUrl: resolveApiBaseUrl());
    addTearDown(driver.close);

    // Không ghi đè quyền thông báo: app dùng permission_handler thật.
    await $.pumpWidgetAndSettle(buildApp());

    await runBookingFlow(
      $.tester,
      driverAccepts: driver.accept,
      timeout: const Duration(seconds: 30),
      answerPermissionDialog: () async {
        // Không phải mọi lần chạy đều có hộp thoại: Android dưới API 33 không
        // có, và quyền đã cấp từ trước thì hệ điều hành không hỏi lại.
        if (await $.platform.mobile.isPermissionDialogVisible(
          timeout: const Duration(seconds: 5),
        )) {
          await $.platform.mobile.grantPermissionWhenInUse();
        }
      },
      screenshot: $.takeNativeScreenshot,
    );
  });

  patrolTest('tài xế nhận chuyến lúc app đang chạy nền', ($) async {
    final driver = DriverSimulator(baseUrl: resolveApiBaseUrl());
    addTearDown(driver.close);

    await $.pumpWidgetAndSettle(
      buildApp(
        overrides: [
          notificationPermissionProvider.overrideWithValue(
            FakeNotificationPermission(),
          ),
        ],
      ),
    );
    await requestQuote($.tester);
    final tripId = await placeBooking($.tester);

    await $.platform.mobile.pressHome();
    await actAsDriver($.tester, () => driver.accept(tripId));
    await $.platform.mobile.openApp();

    expect(
      await waitForDriver($.tester, timeout: const Duration(seconds: 30)),
      'D-01',
    );
  });
}

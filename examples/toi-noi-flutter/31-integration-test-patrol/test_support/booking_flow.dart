import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_31/keys.dart';
import 'package:toi_noi_31/notification_permission.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Việc làm thay tài xế: nhận chuyến [tripId] trên server.
typedef DriverAccepts = Future<void> Function(String tripId);

/// Việc trả lời hộp thoại quyền của hệ điều hành. `flutter test` và
/// `integration_test` thường không có; Patrol truyền vào bước bấm "Allow".
typedef AnswerPermissionDialog = Future<void> Function();

/// Chụp màn hình, mỗi runner một kiểu.
typedef Screenshot = Future<void> Function(String name);

/// Kết quả của một lần chạy luồng, để test gọi kiểm thêm.
typedef BookingOutcome = ({String tripId, Money fare, String driverId});

/// Chờ tới khi [finder] tìm thấy widget, bằng cách bơm từng khung hình.
///
/// Không dùng `pumpAndSettle`: màn hình chờ tài xế có một
/// `LinearProgressIndicator` chạy mãi, nên `pumpAndSettle` không bao giờ coi
/// là yên và hết giờ. `timeout` tính theo thời gian của đồng hồ test: trong
/// `flutter test` là đồng hồ giả (không chờ thật), trên thiết bị là đồng hồ
/// thật.
Future<void> pumpUntilFound(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 10),
  Duration step = const Duration(milliseconds: 100),
}) async {
  var waited = Duration.zero;
  while (finder.evaluate().isEmpty) {
    if (waited >= timeout) {
      throw TestFailure(
        'Không thấy widget cần tìm (${finder.describeMatch(Plurality.zero)}) '
        'sau ${timeout.inSeconds} giây.',
      );
    }
    await tester.pump(step);
    waited += step;
  }
}

/// Tìm `Text` mang [key] và đúng nội dung [text].
Finder textWithKey(Key key, String text) => find.byWidgetPredicate(
  (widget) => widget is Text && widget.key == key && widget.data == text,
  description: 'Text $key hiện "$text"',
);

Future<void> _tap(WidgetTester tester, Key key) async {
  final finder = find.byKey(key);
  await tester.ensureVisible(finder);
  await tester.tap(finder);
  await tester.pump();
}

/// Chạy một việc ngoài giao diện (gọi server thay tài xế) mà không bị kẹt: trong
/// `testWidgets` thân test chạy trên đồng hồ giả, nên một `await` cần timer
/// thật sẽ không bao giờ xong nếu không có `runAsync`. Trên thiết bị (đồng hồ
/// thật) `runAsync` chỉ chạy việc đó.
Future<void> actAsDriver(
  WidgetTester tester,
  Future<void> Function() action,
) async {
  await tester.runAsync(action);
}

/// Bước 1: chọn dịch vụ, nhập quãng đường, báo giá. Trả giá hiện trên màn hình.
Future<Money> requestQuote(
  WidgetTester tester, {
  ServiceKind service = ServiceKind.ride,
  double distanceKm = 5,
  Duration timeout = const Duration(seconds: 10),
}) async {
  await _tap(tester, AppKeys.service(service));
  await tester.enterText(find.byKey(AppKeys.distanceField), '$distanceKm');
  await _tap(tester, AppKeys.quoteButton);
  await pumpUntilFound(tester, find.byKey(AppKeys.fareText), timeout: timeout);
  final expected = calculateFare(distanceKm);
  expect(
    find.text('Giá dự kiến: $expected'),
    findsOneWidget,
    reason: 'giá hiển thị phải bằng calculateFare($distanceKm)',
  );
  return expected;
}

/// Bước 2: đặt chuyến. Trả mã chuyến server vừa cấp.
Future<String> placeBooking(
  WidgetTester tester, {
  Duration timeout = const Duration(seconds: 10),
}) async {
  await _tap(tester, AppKeys.bookButton);
  await pumpUntilFound(
    tester,
    find.byKey(AppKeys.tripIdText),
    timeout: timeout,
  );
  expect(
    textWithKey(AppKeys.tripStatusText, TripStatus.requested.label),
    findsOneWidget,
  );
  return tester.widget<Text>(find.byKey(AppKeys.tripIdText)).data!;
}

/// Bước 3: bấm "Bật thông báo", để [answerDialog] (nếu có) trả lời hộp thoại
/// của hệ điều hành, rồi chờ màn hình hiện kết quả [expected].
Future<void> askForNotifications(
  WidgetTester tester, {
  AnswerPermissionDialog? answerDialog,
  NotificationPermissionStatus expected = NotificationPermissionStatus.granted,
  Duration timeout = const Duration(seconds: 10),
}) async {
  await _tap(tester, AppKeys.enableNotificationsButton);
  await answerDialog?.call();
  final text = switch (expected) {
    NotificationPermissionStatus.granted => 'Thông báo: đã bật',
    NotificationPermissionStatus.denied => 'Thông báo: chưa bật',
  };
  await pumpUntilFound(
    tester,
    textWithKey(AppKeys.notificationStatusText, text),
    timeout: timeout,
  );
}

/// Bước 4: chờ màn hình chuyển sang "tài xế đã nhận" và trả mã tài xế.
Future<String> waitForDriver(
  WidgetTester tester, {
  Duration timeout = const Duration(seconds: 10),
}) async {
  await pumpUntilFound(
    tester,
    textWithKey(AppKeys.tripStatusText, TripStatus.accepted.label),
    timeout: timeout,
  );
  final text = tester.widget<Text>(find.byKey(AppKeys.driverText)).data!;
  return text.replaceFirst('Tài xế: ', '');
}

/// Toàn bộ luồng đặt chuyến của app khách, từ lúc app đã hiện trên màn hình
/// tới lúc có tài xế nhận. Một hàm cho ba runner: `flutter test` (adapter giả,
/// đồng hồ giả), `integration_test` và Patrol (máy thật hoặc giả lập, server
/// thật). Caller dựng app trước (`pumpWidget`) và quyết định phần native.
Future<BookingOutcome> runBookingFlow(
  WidgetTester tester, {
  required DriverAccepts driverAccepts,
  AnswerPermissionDialog? answerPermissionDialog,
  Screenshot? screenshot,
  ServiceKind service = ServiceKind.ride,
  double distanceKm = 5,
  Duration timeout = const Duration(seconds: 10),
}) async {
  final fare = await requestQuote(
    tester,
    service: service,
    distanceKm: distanceKm,
    timeout: timeout,
  );
  await screenshot?.call('1-bao-gia');

  final tripId = await placeBooking(tester, timeout: timeout);
  await askForNotifications(
    tester,
    answerDialog: answerPermissionDialog,
    timeout: timeout,
  );
  await screenshot?.call('2-dang-tim-tai-xe');

  await actAsDriver(tester, () => driverAccepts(tripId));
  final driverId = await waitForDriver(tester, timeout: timeout);
  await screenshot?.call('3-tai-xe-da-nhan');
  return (tripId: tripId, fare: fare, driverId: driverId);
}

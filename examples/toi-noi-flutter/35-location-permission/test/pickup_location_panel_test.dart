import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_35/location_access_controller.dart';
import 'package:toi_noi_35/location_models.dart';
import 'package:toi_noi_35/pickup_location_panel.dart';

import 'support/fake_location_source.dart';

Future<LocationAccessController> pumpPanel(
  WidgetTester tester,
  FakeLocationSource source, {
  AppRole role = AppRole.customer,
  bool tripActive = false,
}) async {
  final controller = LocationAccessController(
    source: source,
    store: MemoryAskedStore(),
    role: role,
  );
  addTearDown(controller.dispose);
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: PickupLocationPanel(
          controller: controller,
          tripActive: tripActive,
        ),
      ),
    ),
  );
  await controller.refresh();
  await tester.pump();
  return controller;
}

void main() {
  testWidgets('dựng màn hình không xin quyền; chỉ có nút giải thích', (
    tester,
  ) async {
    final source = FakeLocationSource();
    await pumpPanel(tester, source);
    expect(find.text('Chưa dùng vị trí'), findsOneWidget);
    expect(find.text('Dùng vị trí hiện tại'), findsOneWidget);
    expect(source.foregroundRequests, 0);
  });

  testWidgets('bấm nút chỉ mở lời giải thích; "Để sau" không xin quyền', (
    tester,
  ) async {
    final source = FakeLocationSource();
    await pumpPanel(tester, source);

    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    expect(find.text('Tới Nơi dùng vị trí để làm gì?'), findsOneWidget);
    expect(source.foregroundRequests, 0, reason: 'chưa bấm Tiếp tục');

    await tester.tap(find.byKey(PanelKeys.dialogLater));
    await tester.pumpAndSettle();
    expect(source.foregroundRequests, 0);
    expect(find.text('Chưa dùng vị trí'), findsOneWidget);
  });

  testWidgets('Tiếp tục: xin quyền đúng một lần rồi hiện trạng thái mới', (
    tester,
  ) async {
    final source = FakeLocationSource()
      ..foregroundAnswers.add(SystemPermission.whileInUse);
    await pumpPanel(tester, source);

    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(PanelKeys.dialogContinue));
    await tester.pumpAndSettle();

    expect(source.foregroundRequests, 1);
    expect(find.text('Đang dùng vị trí khi mở app'), findsOneWidget);
    expect(find.byKey(PanelKeys.action), findsNothing);
  });

  testWidgets('Denied: giải thích lại rồi mới hỏi lại', (tester) async {
    final source = FakeLocationSource()
      ..foregroundAnswers.add(SystemPermission.denied);
    await pumpPanel(tester, source);
    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(PanelKeys.dialogContinue));
    await tester.pumpAndSettle();
    expect(find.text('Bạn đã từ chối vị trí'), findsOneWidget);

    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    expect(find.byKey(PanelKeys.dialogContinue), findsOneWidget);
    expect(source.foregroundRequests, 1);
  });

  testWidgets('DeniedForever: nút mở Cài đặt, không có hộp thoại giải thích', (
    tester,
  ) async {
    final source = FakeLocationSource(
      permission: SystemPermission.deniedForever,
    );
    await pumpPanel(tester, source);
    expect(find.text('Mở Cài đặt của Tới Nơi'), findsOneWidget);

    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    expect(source.appSettingsOpened, 1);
    expect(find.byKey(PanelKeys.dialogContinue), findsNothing);
    expect(source.foregroundRequests, 0);
  });

  testWidgets('Android: hai lần từ chối thì nút chuyển thành Mở Cài đặt', (
    tester,
  ) async {
    final source = FakeLocationSource(androidLike: true)
      ..foregroundAnswers.addAll([
        SystemPermission.denied,
        SystemPermission.deniedForever,
      ]);
    await pumpPanel(tester, source);
    for (var i = 0; i < 2; i++) {
      await tester.tap(find.byKey(PanelKeys.action));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(PanelKeys.dialogContinue));
      await tester.pumpAndSettle();
    }
    expect(find.text('Mở Cài đặt của Tới Nơi'), findsOneWidget);
    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    expect(source.appSettingsOpened, 1);
    expect(source.foregroundRequests, 2);
  });

  testWidgets('ServiceOff: nút mở cài đặt dịch vụ vị trí', (tester) async {
    final source = FakeLocationSource(serviceEnabled: false);
    await pumpPanel(tester, source);
    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    expect(source.locationSettingsOpened, 1);
  });

  testWidgets('vị trí gần đúng: báo lệch và có nút xin chính xác', (
    tester,
  ) async {
    final source = FakeLocationSource(
      permission: SystemPermission.whileInUse,
      precisionValue: LocationPrecision.reduced,
    )..preciseAnswer = LocationPrecision.precise;
    await pumpPanel(tester, source);
    expect(find.textContaining('Vị trí gần đúng'), findsOneWidget);

    await tester.tap(find.byKey(PanelKeys.precise));
    await tester.pumpAndSettle();
    expect(source.preciseRequests, 1);
    expect(find.byKey(PanelKeys.precise), findsNothing);
    expect(find.text('Đang dùng vị trí khi mở app'), findsOneWidget);
  });

  testWidgets('tài xế: chỉ có chuyến mới được mời cho phép gửi vị trí nền', (
    tester,
  ) async {
    final source = FakeLocationSource(permission: SystemPermission.whileInUse)
      ..backgroundAnswers.add(SystemPermission.always);
    await pumpPanel(tester, source, role: AppRole.driver);
    expect(find.byKey(PanelKeys.action), findsNothing);

    final withTrip = FakeLocationSource(permission: SystemPermission.whileInUse)
      ..backgroundAnswers.add(SystemPermission.always);
    await pumpPanel(tester, withTrip, role: AppRole.driver, tripActive: true);
    await tester.tap(find.byKey(PanelKeys.action));
    await tester.pumpAndSettle();
    expect(withTrip.backgroundRequests, 0, reason: 'chưa bấm Tiếp tục');
    await tester.tap(find.byKey(PanelKeys.dialogContinue));
    await tester.pumpAndSettle();
    expect(withTrip.backgroundRequests, 1);
    expect(find.text('Được dùng vị trí cả khi app ở nền'), findsOneWidget);
  });

  testWidgets('Android: tài xế có chuyến cũng không được mời xin nền', (
    tester,
  ) async {
    final source = FakeLocationSource(
      permission: SystemPermission.whileInUse,
      backgroundSupported: false,
    );
    await pumpPanel(tester, source, role: AppRole.driver, tripActive: true);
    expect(find.byKey(PanelKeys.action), findsNothing);
  });

  testWidgets('app khách có chuyến cũng không được mời cho phép nền', (
    tester,
  ) async {
    final source = FakeLocationSource(permission: SystemPermission.whileInUse);
    await pumpPanel(tester, source, tripActive: true);
    expect(find.byKey(PanelKeys.action), findsNothing);
  });
}

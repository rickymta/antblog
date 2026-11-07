import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/buffer_store.dart';
import 'package:toi_noi_38/driver_app.dart';
import 'package:toi_noi_38/driver_location_controller.dart';
import 'package:toi_noi_38/driver_location_policy.dart';
import 'package:toi_noi_38/location_buffer.dart';
import 'package:toi_noi_38/location_uploader.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'test_helpers.dart';

void main() {
  const policy = DriverLocationPolicy();

  Future<void> choose(WidgetTester tester, TripStatus status) async {
    await tester.tap(find.byKey(const Key('status-dropdown')));
    await tester.pumpAndSettle();
    await tester.tap(find.text(status.label).last);
    await tester.pumpAndSettle();
  }

  String decision(WidgetTester tester) =>
      tester.widget<Text>(find.byKey(const Key('decision'))).data!;

  testWidgets('mặc định (đang tìm tài xế, online): không gửi', (tester) async {
    await tester.pumpWidget(const DriverApp());
    expect(decision(tester), 'Gửi vị trí nền: KHÔNG');
  });

  testWidgets('màn hình khớp chính sách ở cả 8 trạng thái', (tester) async {
    await tester.pumpWidget(const DriverApp());
    for (final status in TripStatus.values) {
      await choose(tester, status);
      expect(
        decision(tester),
        policy.tracksStatus(status)
            ? 'Gửi vị trí nền: CÓ'
            : 'Gửi vị trí nền: KHÔNG',
        reason: status.name,
      );
    }
  });

  testWidgets('tắt online thì không gửi dù chuyến đang chở', (tester) async {
    await tester.pumpWidget(const DriverApp());
    await choose(tester, TripStatus.inTransit);
    expect(decision(tester), 'Gửi vị trí nền: CÓ');
    await tester.tap(find.byKey(const Key('online-switch')));
    await tester.pump();
    expect(decision(tester), 'Gửi vị trí nền: KHÔNG');
  });

  testWidgets(
    'có controller: chọn trạng thái thì bật và tắt luồng vị trí thật',
    (tester) async {
      final tracker = FakeTracker();
      final controller = DriverLocationController(
        policy: policy,
        tracker: tracker,
        uploader: LocationUploader(
          sink: FakeSink(),
          buffer: LocationBuffer(),
          store: MemoryBufferStore(),
        ),
      );
      await tester.pumpWidget(DriverApp(controller: controller));

      await choose(tester, TripStatus.accepted);
      expect(tracker.startedTrips, [demoTripId]);
      expect(controller.isTracking, isTrue);

      await choose(tester, TripStatus.completed);
      expect(controller.isTracking, isFalse);
      expect(tracker.hasListener(demoTripId), isFalse);
    },
  );

  testWidgets('nút xin quyền chỉ hiện khi có hàm xử lý', (tester) async {
    await tester.pumpWidget(const DriverApp());
    expect(find.byKey(const Key('request-permission')), findsNothing);

    var asked = 0;
    await tester.pumpWidget(
      DriverApp(onRequestPermission: () async => asked++),
    );
    await tester.tap(find.byKey(const Key('request-permission')));
    expect(asked, 1);
  });
}

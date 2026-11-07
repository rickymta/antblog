import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_35/location_access_controller.dart';
import 'package:toi_noi_35/location_models.dart';
import 'package:toi_noi_35/position_tracker.dart';

import 'support/fake_location_source.dart';

void main() {
  late FakeLocationSource source;
  late LocationAccessController access;
  late PositionTracker tracker;

  setUp(() {
    source = FakeLocationSource();
    access = LocationAccessController(
      source: source,
      store: MemoryAskedStore(),
      role: AppRole.driver,
    );
    tracker = PositionTracker(source: source, access: access);
  });

  tearDown(() {
    tracker.dispose();
    access.dispose();
  });

  test('chưa có quyền thì chưa mở luồng, dù đã đặt profile', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    await access.refresh();
    expect(tracker.isTracking, isFalse);
    expect(source.streamsOpened, isEmpty);
  });

  test('có quyền thì mở đúng một luồng và nhận vị trí', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    source.permission = SystemPermission.whileInUse;
    await access.refresh();
    expect(source.streamsOpened, [TrackingProfile.driverOnline]);

    source.controllers.single.add(fixAt(10.7769, 106.7009));
    await Future<void>.delayed(Duration.zero);
    expect(tracker.lastFix.value, fixAt(10.7769, 106.7009));

    await access.refresh();
    expect(source.streamsOpened, hasLength(1), reason: 'refresh không mở thêm');
  });

  test('mất quyền: huỷ subscription và xoá vị trí cũ', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    source.permission = SystemPermission.whileInUse;
    await access.refresh();
    source.controllers.single.add(fixAt(10.7, 106.7));
    await Future<void>.delayed(Duration.zero);

    source.permission = SystemPermission.deniedForever;
    await access.refresh();
    expect(tracker.isTracking, isFalse);
    expect(source.controllers.single.hasListener, isFalse);
    expect(tracker.lastFix.value, isNull);
  });

  test('tắt dịch vụ vị trí cũng dừng luồng', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    source.permission = SystemPermission.always;
    await access.refresh();
    expect(tracker.isTracking, isTrue);

    source.serviceEnabled = false;
    await access.refresh();
    expect(tracker.isTracking, isFalse);
    expect(source.controllers.single.hasListener, isFalse);
  });

  test('đổi profile: đóng luồng cũ, mở luồng mới', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    source.permission = SystemPermission.always;
    await access.refresh();

    tracker.setProfile(TrackingProfile.driverOnTrip);
    expect(source.streamsOpened, [
      TrackingProfile.driverOnline,
      TrackingProfile.driverOnTrip,
    ]);
    expect(source.controllers.first.hasListener, isFalse);
    expect(source.controllers.last.hasListener, isTrue);
  });

  test('setProfile(null) dừng theo dõi', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    source.permission = SystemPermission.always;
    await access.refresh();
    tracker.setProfile(null);
    expect(tracker.isTracking, isFalse);
  });

  test(
    'lỗi dịch vụ vị trí tắt giữa chừng được ghi lại, luồng không sập',
    () async {
      tracker.setProfile(TrackingProfile.driverOnline);
      source.permission = SystemPermission.whileInUse;
      await access.refresh();

      source.controllers.single.addError(
        const LocationServiceOffException('tắt'),
      );
      await Future<void>.delayed(Duration.zero);
      expect(tracker.lastError.value, isA<LocationServiceOffException>());

      source.controllers.single.add(fixAt(10.8, 106.7));
      await Future<void>.delayed(Duration.zero);
      expect(
        tracker.lastError.value,
        isNull,
        reason: 'có vị trí mới thì xoá lỗi',
      );
    },
  );

  test('dispose huỷ subscription', () async {
    tracker.setProfile(TrackingProfile.driverOnline);
    source.permission = SystemPermission.whileInUse;
    await access.refresh();
    final controller = source.controllers.single;
    tracker.dispose();
    expect(controller.hasListener, isFalse);
    // tearDown gọi dispose lần hai: phải an toàn.
    tracker = PositionTracker(source: source, access: access);
  });
}

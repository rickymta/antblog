import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/buffer_store.dart';
import 'package:toi_noi_38/driver_location_controller.dart';
import 'package:toi_noi_38/driver_location_policy.dart';
import 'package:toi_noi_38/location_buffer.dart';
import 'package:toi_noi_38/location_uploader.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'test_helpers.dart';

void main() {
  late FakeTracker tracker;
  late FakeSink sink;
  late LocationBuffer buffer;
  late MemoryBufferStore store;
  late DriverLocationController controller;

  setUp(() {
    tracker = FakeTracker();
    sink = FakeSink();
    buffer = LocationBuffer(capacity: 3);
    store = MemoryBufferStore();
    controller = DriverLocationController(
      policy: const DriverLocationPolicy(),
      tracker: tracker,
      uploader: LocationUploader(
        sink: sink,
        buffer: buffer,
        store: store,
        now: () => t0,
      ),
    );
  });

  Future<void> update(
    TripStatus? status, {
    bool online = true,
    String? trip = 'T1',
  }) => controller.onStateChanged(
    tripId: trip,
    status: status,
    driverOnline: online,
  );

  Future<void> pump() => Future<void>.delayed(Duration.zero);

  test('theo dõi đúng ở những trạng thái chính sách cho phép (đủ 8)', () async {
    const policy = DriverLocationPolicy();
    for (final status in TripStatus.values) {
      await update(status);
      expect(
        controller.isTracking,
        policy.tracksStatus(status),
        reason: status.name,
      );
    }
  });

  test('nhận chuyến thì bật nguồn vị trí và gửi mẫu lên sink', () async {
    await update(TripStatus.accepted);
    expect(tracker.startedTrips, ['T1']);
    tracker.emit('T1', sample(0));
    await pump();
    expect(sink.sent, [sample(0)]);
  });

  test('qua accepted, arriving, pickedUp, inTransit chỉ bật một lần', () async {
    for (final s in [
      TripStatus.accepted,
      TripStatus.arriving,
      TripStatus.pickedUp,
      TripStatus.inTransit,
    ]) {
      await update(s);
    }
    expect(tracker.startedTrips, ['T1']);
    expect(controller.isTracking, isTrue);
  });

  test('completed: dừng nguồn vị trí và xoá sạch bộ đệm lẫn kho', () async {
    sink.failing = true; // đang mất mạng nên còn mẫu chờ gửi
    await update(TripStatus.inTransit);
    tracker.emit('T1', sample(0));
    await pump();
    expect(buffer.length, 1);
    expect(await store.load(), hasLength(1));

    await update(TripStatus.completed);

    expect(controller.isTracking, isFalse);
    expect(tracker.hasListener('T1'), isFalse);
    expect(buffer.isEmpty, isTrue);
    expect(await store.load(), isEmpty);
  });

  test('cancelled: dừng ngay', () async {
    await update(TripStatus.arriving);
    await update(TripStatus.cancelled);
    expect(controller.isTracking, isFalse);
    expect(tracker.hasListener('T1'), isFalse);
  });

  test('tài xế offline giữa chuyến: dừng', () async {
    await update(TripStatus.inTransit);
    await update(TripStatus.inTransit, online: false);
    expect(controller.isTracking, isFalse);
  });

  test('không có tripId thì không bật, dù trạng thái hợp lệ', () async {
    await update(TripStatus.accepted, trip: null);
    expect(controller.isTracking, isFalse);
    expect(tracker.startedTrips, isEmpty);
  });

  test(
    'sang chuyến khác: dừng chuyến cũ, bỏ mẫu chờ, bật chuyến mới',
    () async {
      sink.failing = true;
      await update(TripStatus.accepted);
      tracker.emit('T1', sample(0));
      await pump();
      expect(buffer.length, 1);

      await update(TripStatus.accepted, trip: 'T2');

      expect(tracker.hasListener('T1'), isFalse);
      expect(tracker.hasListener('T2'), isTrue);
      expect(controller.activeTripId, 'T2');
      expect(buffer.isEmpty, isTrue, reason: 'không trộn vị trí hai chuyến');
    },
  );

  test('mất mạng: bộ đệm giới hạn 3 mẫu, bỏ mẫu cũ nhất, có mạng thì gửi bù phần còn lại', () async {
    sink.failing = true;
    await update(TripStatus.inTransit);
    for (var i = 0; i < 5; i++) {
      tracker.emit('T1', sample(i));
      await pump();
    }
    expect(buffer.length, 3);
    expect(buffer.droppedCount, 2);

    sink.failing = false;
    tracker.emit('T1', sample(5));
    await pump();
    // Mẫu thứ 6 đẩy mẫu 2 ra khỏi bộ đệm đầy trước khi gửi: mất thêm một mẫu cũ.
    expect(sink.sent, [sample(3), sample(4), sample(5)]);
    expect(buffer.droppedCount, 3);
  });

  test(
    'nguồn vị trí báo lỗi (ví dụ thu hồi quyền): ghi lại, không crash',
    () async {
      await update(TripStatus.accepted);
      tracker.controllers['T1']!.addError(StateError('quyền bị thu hồi'));
      await pump();
      expect(controller.lastError, isA<StateError>());
    },
  );

  test('dispose dừng nguồn vị trí', () async {
    await update(TripStatus.accepted);
    await controller.dispose();
    expect(tracker.hasListener('T1'), isFalse);
  });
}

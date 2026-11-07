import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/background_tasks.dart';
import 'package:toi_noi_38/buffer_store.dart';

import 'test_helpers.dart';

void main() {
  group('PeriodicTaskPlan', () {
    test('chu kỳ dưới 15 phút bị từ chối', () {
      expect(
        () =>
            PeriodicTaskPlan(name: 'x', frequency: const Duration(minutes: 14)),
        throwsArgumentError,
      );
    });

    test('đúng 15 phút là hợp lệ', () {
      final plan = PeriodicTaskPlan(
        name: 'x',
        frequency: minimumPeriodicFrequency,
      );
      expect(plan.frequency, const Duration(minutes: 15));
    });

    test('kế hoạch dọn bộ đệm của app tài xế hợp lệ và đặt đúng tên', () {
      expect(DriverBackgroundPlans.purgeExpired.name, TaskNames.purgeExpired);
      expect(
        DriverBackgroundPlans.purgeExpired.frequency >=
            minimumPeriodicFrequency,
        isTrue,
      );
      expect(DriverBackgroundPlans.flushPending.name, TaskNames.flushPending);
    });
  });

  group('TaskNames', () {
    test('chỉ task định kỳ cần khai identifier cho iOS', () {
      expect(TaskNames.iosPermittedIdentifiers, [TaskNames.purgeExpired]);
    });

    test('tên hai task khác nhau', () {
      expect(TaskNames.purgeExpired, isNot(TaskNames.flushPending));
    });
  });

  group('runBackgroundTask', () {
    late FakeSink sink;
    late MemoryBufferStore store;
    final now = t0.add(const Duration(minutes: 30));

    setUp(() {
      sink = FakeSink();
      // sample(0) cũ 30 phút, sample(350) cũ khoảng 2 phút (350 x 5 giây = 29 phút 10 giây).
      store = MemoryBufferStore([sample(0), sample(350)]);
    });

    test('purgeExpired bỏ mẫu quá hạn và lưu lại phần còn lại', () async {
      final ok = await runBackgroundTask(
        TaskNames.purgeExpired,
        store: store,
        sink: sink,
        now: () => now,
      );
      expect(ok, isTrue);
      expect(await store.load(), [sample(350)]);
      expect(sink.batches, isEmpty, reason: 'dọn dẹp không gửi gì lên server');
    });

    test('flushPending gửi mẫu còn hạn, bỏ mẫu quá hạn và dọn kho', () async {
      final ok = await runBackgroundTask(
        TaskNames.flushPending,
        store: store,
        sink: sink,
        now: () => now,
      );
      expect(ok, isTrue);
      expect(sink.sent, [sample(350)]);
      expect(await store.load(), isEmpty);
    });

    test('flushPending khi sink lỗi trả false để Android thử lại, kho giữ nguyên mẫu còn hạn', () async {
      sink.failing = true;
      final ok = await runBackgroundTask(
        TaskNames.flushPending,
        store: store,
        sink: sink,
        now: () => now,
      );
      expect(ok, isFalse);
      expect(await store.load(), [sample(350)]);
    });

    test('tên task lạ trả false và không đụng tới kho', () async {
      final ok = await runBackgroundTask(
        'khong-ton-tai',
        store: store,
        sink: sink,
        now: () => now,
      );
      expect(ok, isFalse);
      expect(store.saveCount, 0);
    });

    test('kho trống: cả hai task đều xong', () async {
      final empty = MemoryBufferStore();
      for (final name in [TaskNames.purgeExpired, TaskNames.flushPending]) {
        expect(
          await runBackgroundTask(
            name,
            store: empty,
            sink: sink,
            now: () => now,
          ),
          isTrue,
          reason: name,
        );
      }
    });
  });
}

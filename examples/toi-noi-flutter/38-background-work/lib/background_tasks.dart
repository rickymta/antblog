import 'package:workmanager/workmanager.dart';

import 'buffer_store.dart';
import 'location_buffer.dart';
import 'location_uploader.dart';

/// Tên task. Dùng một chuỗi cho cả `uniqueName` và `taskName`: trên iOS task
/// định kỳ được nhận diện bằng `uniqueName` (chính là identifier trong
/// `BGTaskSchedulerPermittedIdentifiers`), trên Android bằng `taskName`.
abstract final class TaskNames {
  /// Dọn mẫu vị trí quá hạn khỏi đĩa. Định kỳ.
  static const purgeExpired = 'vn.toinoi.driver.purge-expired';

  /// Gửi bù mẫu còn lại khi có mạng. Chạy một lần.
  static const flushPending = 'vn.toinoi.driver.flush-pending';

  /// Identifier phải khai trong `Info.plist`. Chỉ task định kỳ qua
  /// `BGTaskScheduler`; task một lần trên iOS chạy bằng
  /// `beginBackgroundTask` nên không cần khai.
  static const iosPermittedIdentifiers = <String>[purgeExpired];
}

/// Chu kỳ tối thiểu của task định kỳ: 15 phút trên Android (xem
/// `PeriodicWorkRequest`). Trên iOS `frequency` không được dùng.
const minimumPeriodicFrequency = Duration(minutes: 15);

/// Mô tả một task định kỳ, kiểm giá trị trước khi chạm vào plugin.
final class PeriodicTaskPlan {
  PeriodicTaskPlan({required this.name, required this.frequency}) {
    if (frequency < minimumPeriodicFrequency) {
      throw ArgumentError.value(
        frequency,
        'frequency',
        'tối thiểu ${minimumPeriodicFrequency.inMinutes} phút; Android sẽ tự '
            'nâng lên mức đó nên đặt thấp hơn chỉ là hứa điều không giữ',
      );
    }
  }

  final String name;
  final Duration frequency;
}

/// Mô tả một task chạy một lần, cần mạng.
final class OneOffTaskPlan {
  const OneOffTaskPlan({
    required this.name,
    this.backoffDelay = const Duration(seconds: 30),
  });

  final String name;

  /// Chờ bao lâu trước lần thử lại đầu tiên khi task trả `false` (Android).
  final Duration backoffDelay;
}

/// Hai kế hoạch của app tài xế.
abstract final class DriverBackgroundPlans {
  static final purgeExpired = PeriodicTaskPlan(
    name: TaskNames.purgeExpired,
    frequency: const Duration(hours: 1),
  );

  static const flushPending = OneOffTaskPlan(name: TaskNames.flushPending);
}

/// Việc mà một task nền làm. Hàm thuần Dart: mọi thứ cần tới được truyền vào,
/// nên test bằng fake mà không cần plugin.
///
/// Giá trị trả về là thứ `workmanager` báo cho hệ điều hành: `true` là xong,
/// `false` là thất bại (Android sẽ thử lại theo backoff; iOS bỏ qua giá trị
/// này).
Future<bool> runBackgroundTask(
  String taskName, {
  required BufferStore store,
  required LocationSink sink,
  required DateTime Function() now,
}) async {
  final buffer = LocationBuffer()..replaceAll(await store.load());
  switch (taskName) {
    case TaskNames.purgeExpired:
      buffer.purgeExpired(now());
      await store.save(buffer.items);
      return true;
    case TaskNames.flushPending:
      final result = await LocationUploader(
        sink: sink,
        buffer: buffer,
        store: store,
        now: now,
      ).flush();
      return !result.failed;
    default:
      return false; // tên lạ: lỗi lập trình, không có gì để làm
  }
}

/// Lớp bọc mỏng quanh `Workmanager`: chỉ đổi kế hoạch thành lời gọi plugin.
final class WorkmanagerScheduler {
  WorkmanagerScheduler({Workmanager? workmanager})
    : _workmanager = workmanager ?? Workmanager();

  final Workmanager _workmanager;

  Future<void> schedulePurge(PeriodicTaskPlan plan) =>
      _workmanager.registerPeriodicTask(
        plan.name,
        plan.name,
        frequency: plan.frequency,
        // `update` áp cấu hình mới mà không huỷ worker đang chạy; `keep` sẽ giữ
        // nguyên chu kỳ cũ dù ta đã đổi.
        existingWorkPolicy: ExistingPeriodicWorkPolicy.update,
      );

  Future<void> scheduleFlushWhenOnline(OneOffTaskPlan plan) =>
      _workmanager.registerOneOffTask(
        plan.name,
        plan.name,
        constraints: Constraints(networkType: NetworkType.connected),
        existingWorkPolicy: ExistingWorkPolicy.keep,
        backoffPolicy: BackoffPolicy.exponential,
        backoffPolicyDelay: plan.backoffDelay,
      );

  Future<void> cancelAll() => _workmanager.cancelAll();
}

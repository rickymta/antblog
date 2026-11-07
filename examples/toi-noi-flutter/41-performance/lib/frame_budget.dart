import 'package:flutter/scheduler.dart';

/// Ngân sách một khung hình ở tần số quét [refreshHz]: 1000 / Hz mili giây.
///
/// 60 Hz cho khoảng 16,7 ms, 120 Hz cho khoảng 8,3 ms.
Duration frameBudget(int refreshHz) {
  if (refreshHz <= 0) {
    throw ArgumentError.value(refreshHz, 'refreshHz', 'phải dương');
  }
  return Duration(
    microseconds: (Duration.microsecondsPerSecond / refreshHz).round(),
  );
}

/// Phân loại khung hình theo ngân sách của một màn hình.
///
/// Một khung "hụt" khi pha build hoặc pha raster vượt ngân sách (đúng
/// quy ước trong tài liệu `FrameTiming`). Một khung "trễ" khi tổng từ vsync
/// tới raster xong vượt ngân sách: không rớt khung nhưng cảm giác chạm chậm.
class JankMeter {
  /// Tạo bộ đo cho màn hình [refreshHz].
  JankMeter({required this.refreshHz}) : budget = frameBudget(refreshHz);

  /// Tần số quét của màn hình.
  final int refreshHz;

  /// Ngân sách mỗi khung.
  final Duration budget;

  int _frames = 0;
  int _missed = 0;
  int _slow = 0;

  /// Tổng số khung đã xem.
  int get frames => _frames;

  /// Số khung hụt (build hoặc raster vượt ngân sách).
  int get missed => _missed;

  /// Số khung có độ trễ vsync tới raster vượt ngân sách.
  int get highLatency => _slow;

  /// Có phải khung hụt không.
  bool isMissed(FrameTiming t) =>
      t.buildDuration > budget || t.rasterDuration > budget;

  /// Ghi nhận một lô khung hình, cùng chữ ký với `TimingsCallback`.
  void onTimings(List<FrameTiming> timings) {
    for (final t in timings) {
      _frames++;
      if (isMissed(t)) {
        _missed++;
      }
      if (t.totalSpan > budget) {
        _slow++;
      }
    }
  }

  /// Đăng ký với `SchedulerBinding`. Chỉ có ý nghĩa ở profile/release trên
  /// thiết bị thật; ở debug, thời gian bị phình bởi assert và không đại diện.
  void attach() => SchedulerBinding.instance.addTimingsCallback(onTimings);

  /// Gỡ đăng ký.
  void detach() => SchedulerBinding.instance.removeTimingsCallback(onTimings);
}

import 'dart:ui' show FrameTiming;

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/frame_budget.dart';

/// Khung hình tổng hợp: thời gian tính bằng mili giây, đổi sang micro giây.
FrameTiming _frame({
  double vsyncToBuildMs = 1,
  required double buildMs,
  required double rasterMs,
}) {
  int us(double ms) => (ms * 1000).round();
  const origin = 1000000;
  final buildStart = origin + us(vsyncToBuildMs);
  final buildFinish = buildStart + us(buildMs);
  final rasterFinish = buildFinish + us(rasterMs);
  return FrameTiming(
    vsyncStart: origin,
    buildStart: buildStart,
    buildFinish: buildFinish,
    rasterStart: buildFinish,
    rasterFinish: rasterFinish,
    rasterFinishWallTime: rasterFinish,
  );
}

void main() {
  group('frameBudget', () {
    test('60 Hz khoảng 16,7 ms', () {
      expect(frameBudget(60), const Duration(microseconds: 16667));
    });

    test('120 Hz khoảng 8,3 ms', () {
      expect(frameBudget(120), const Duration(microseconds: 8333));
    });

    test('tần số không dương ném ArgumentError', () {
      expect(() => frameBudget(0), throwsArgumentError);
      expect(() => frameBudget(-60), throwsArgumentError);
    });
  });

  group('JankMeter', () {
    test('cùng một khung 12 ms build: ổn ở 60 Hz, hụt ở 120 Hz', () {
      final frame = _frame(buildMs: 12, rasterMs: 4);

      expect(JankMeter(refreshHz: 60).isMissed(frame), isFalse);
      expect(JankMeter(refreshHz: 120).isMissed(frame), isTrue);
    });

    test('raster chậm cũng làm hụt khung', () {
      final frame = _frame(buildMs: 3, rasterMs: 20);

      expect(JankMeter(refreshHz: 60).isMissed(frame), isTrue);
    });

    test('đúng bằng ngân sách thì chưa hụt', () {
      final meter = JankMeter(refreshHz: 60);
      final frame = _frame(buildMs: 16.667, rasterMs: 16.667);

      expect(meter.isMissed(frame), isFalse);
    });

    test('khung build 10 + raster 10 không hụt nhưng độ trễ vượt 16,7 ms', () {
      final meter = JankMeter(refreshHz: 60)
        ..onTimings([_frame(buildMs: 10, rasterMs: 10)]);

      expect(meter.frames, 1);
      expect(meter.missed, 0);
      expect(meter.highLatency, 1);
    });

    test('đếm theo lô khung hình', () {
      final meter = JankMeter(refreshHz: 60)
        ..onTimings([
          _frame(buildMs: 4, rasterMs: 3),
          _frame(buildMs: 30, rasterMs: 3),
          _frame(buildMs: 5, rasterMs: 2),
          _frame(buildMs: 6, rasterMs: 25),
        ]);

      expect(meter.frames, 4);
      expect(meter.missed, 2);
    });

    test('lô rỗng không đổi gì', () {
      final meter = JankMeter(refreshHz: 60)..onTimings(const []);

      expect(meter.frames, 0);
      expect(meter.missed, 0);
    });
  });
}

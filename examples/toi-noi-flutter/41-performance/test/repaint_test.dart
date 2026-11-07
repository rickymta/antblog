import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/paint_panels.dart';

void main() {
  setUp(PaintLog.reset);

  Future<void> pumpPanel(WidgetTester tester, {required bool isolate}) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(child: LiveMapPanel(isolatePulse: isolate)),
        ),
      ),
    );
  }

  /// Chạy [frames] khung hình, mỗi khung cách nhau 16 ms trên đồng hồ giả.
  Future<void> runFrames(WidgetTester tester, int frames) async {
    for (var i = 0; i < frames; i++) {
      await tester.pump(const Duration(milliseconds: 16));
    }
  }

  testWidgets('lần dựng đầu: mỗi painter vẽ đúng một lần', (tester) async {
    await pumpPanel(tester, isolate: false);

    expect(PaintLog.of('route'), 1);
    expect(PaintLog.of('pulse'), 1);
  });

  testWidgets('không có RepaintBoundary: 10 khung vẽ lại cả bản đồ', (
    tester,
  ) async {
    await pumpPanel(tester, isolate: false);
    PaintLog.reset();

    await runFrames(tester, 10);

    expect(PaintLog.of('pulse'), 10);
    // Bản đồ không đổi gì nhưng cùng layer với chấm nên bị vẽ lại theo.
    expect(PaintLog.of('route'), 10);
  });

  testWidgets('chấm trong RepaintBoundary: bản đồ không bị vẽ lại', (
    tester,
  ) async {
    await pumpPanel(tester, isolate: true);
    PaintLog.reset();

    await runFrames(tester, 10);

    expect(PaintLog.of('pulse'), 10);
    expect(PaintLog.of('route'), 0);
  });

  testWidgets('RepaintBoundary chỉ có mặt khi isolatePulse bật', (
    tester,
  ) async {
    await pumpPanel(tester, isolate: true);
    final boundaries = find.descendant(
      of: find.byType(LiveMapPanel),
      matching: find.byType(RepaintBoundary),
    );

    expect(boundaries, findsOneWidget);

    await pumpPanel(tester, isolate: false);
    expect(
      find.descendant(
        of: find.byType(LiveMapPanel),
        matching: find.byType(RepaintBoundary),
      ),
      findsNothing,
    );
  });

  testWidgets('bọc phần tĩnh thay vì phần động cũng giữ bản đồ nguyên', (
    tester,
  ) async {
    final controller = AnimationController(
      vsync: tester,
      duration: const Duration(seconds: 1),
    )..repeat();
    await tester.pumpWidget(
      MaterialApp(
        home: Stack(
          children: [
            RepaintBoundary(
              child: CustomPaint(
                size: const Size(300, 200),
                painter: CountingPainter('route'),
              ),
            ),
            CustomPaint(
              size: const Size(40, 40),
              painter: CountingPainter('pulse', repaint: controller),
            ),
          ],
        ),
      ),
    );
    PaintLog.reset();

    await runFrames(tester, 10);

    expect(PaintLog.of('pulse'), 10);
    expect(PaintLog.of('route'), 0);

    // Ticker phải được giải phóng trước khi thân test kết thúc.
    await tester.pumpWidget(const SizedBox());
    controller.dispose();
  });

  testWidgets('dừng animation thì không còn paint nào', (tester) async {
    await pumpPanel(tester, isolate: false);
    // Gỡ panel: controller bị dispose, không còn khung nào cần vẽ.
    await tester.pumpWidget(const SizedBox());
    PaintLog.reset();

    await runFrames(tester, 5);

    expect(PaintLog.of('pulse'), 0);
    expect(PaintLog.of('route'), 0);
  });
}

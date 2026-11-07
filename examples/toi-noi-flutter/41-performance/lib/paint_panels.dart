import 'package:flutter/material.dart';

/// Bộ đếm số lần `paint` theo nhãn, tương tự `BuildLog` nhưng cho giai đoạn vẽ.
abstract final class PaintLog {
  static final Map<String, int> _counts = {};

  /// Ghi một lần paint của [label].
  static void hit(String label) => _counts[label] = of(label) + 1;

  /// Số lần [label] đã paint kể từ lần [reset] gần nhất.
  static int of(String label) => _counts[label] ?? 0;

  /// Xoá mọi bộ đếm.
  static void reset() => _counts.clear();
}

/// Painter ghi lại mỗi lần `paint` được gọi.
///
/// `shouldRepaint` luôn `false`: nó chỉ quyết định có đánh dấu vẽ lại khi
/// widget được cập nhật. Khi layer chứa nó được vẽ lại, `paint` vẫn chạy.
class CountingPainter extends CustomPainter {
  /// Tạo painter ghi vào [PaintLog] với [label]; [repaint] là tín hiệu vẽ lại.
  CountingPainter(this.label, {super.repaint});

  /// Nhãn trong [PaintLog].
  final String label;

  @override
  void paint(Canvas canvas, Size size) {
    PaintLog.hit(label);
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = const Color(0xFF0F766E),
    );
  }

  @override
  bool shouldRepaint(CountingPainter oldDelegate) => false;
}

/// Bản đồ tuyến (tĩnh) với một chấm vị trí đang nhấp nháy.
///
/// Khi [isolatePulse] là `true`, chấm nằm trong `RepaintBoundary` nên mỗi khung
/// hình chỉ vẽ lại chấm. Khi `false`, chấm cùng layer với bản đồ nên mỗi khung
/// hình vẽ lại cả hai.
class LiveMapPanel extends StatefulWidget {
  /// Tạo panel bản đồ.
  const LiveMapPanel({required this.isolatePulse, super.key});

  /// Có bọc chấm nhấp nháy trong `RepaintBoundary` hay không.
  final bool isolatePulse;

  @override
  State<LiveMapPanel> createState() => _LiveMapPanelState();
}

class _LiveMapPanelState extends State<LiveMapPanel>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(seconds: 1),
  )..repeat();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final pulse = CustomPaint(
      size: const Size(40, 40),
      painter: CountingPainter('pulse', repaint: _controller),
    );
    return SizedBox(
      width: 300,
      height: 200,
      child: Stack(
        children: [
          CustomPaint(
            size: const Size(300, 200),
            painter: CountingPainter('route'),
          ),
          Positioned(
            left: 130,
            top: 80,
            child: widget.isolatePulse ? RepaintBoundary(child: pulse) : pulse,
          ),
        ],
      ),
    );
  }
}

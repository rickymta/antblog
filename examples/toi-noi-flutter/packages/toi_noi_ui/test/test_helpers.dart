import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Hai theme cần kiểm trong mọi test giao diện.
final Map<String, ThemeData> themes = {
  'sáng': ToiNoiTheme.light(),
  'tối': ToiNoiTheme.dark(),
};

/// Bọc [child] trong MaterialApp dùng [theme], không phụ thuộc hệ thống.
Future<void> pumpThemed(
  WidgetTester tester,
  Widget child, {
  required ThemeData theme,
}) {
  return tester.pumpWidget(
    MaterialApp(
      theme: theme,
      // Đổi theme có animation; tắt để test thấy giá trị cuối ngay.
      themeAnimationDuration: Duration.zero,
      home: Scaffold(body: Center(child: child)),
    ),
  );
}

/// Tỉ lệ tương phản WCAG giữa hai màu đặc (không trong suốt).
double contrastRatio(Color a, Color b) {
  final la = a.computeLuminance();
  final lb = b.computeLuminance();
  final lighter = la > lb ? la : lb;
  final darker = la > lb ? lb : la;
  return (lighter + 0.05) / (darker + 0.05);
}

/// Chiều cao phần nền nhìn thấy của FilledButton. Khác với `getSize` của chính
/// nút: vùng chạm của nút được đệm tới 48 dù phần nhìn thấy thấp hơn.
double filledButtonVisibleHeight(WidgetTester tester) {
  final material = find.descendant(
    of: find.byType(FilledButton),
    matching: find.byType(Material),
  );
  return tester.getSize(material.first).height;
}

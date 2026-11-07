import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'history_page.dart';

void main() => runApp(const PerformanceDemoApp());

/// App thử tay của bài 41. Chạy `flutter run --profile` trên máy thật rồi mở
/// DevTools, tab Performance.
class PerformanceDemoApp extends StatelessWidget {
  /// Tạo app.
  const PerformanceDemoApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Tới Nơi · Hiệu năng',
    theme: ToiNoiTheme.light(),
    darkTheme: ToiNoiTheme.dark(),
    home: const HistoryPage(),
  );
}

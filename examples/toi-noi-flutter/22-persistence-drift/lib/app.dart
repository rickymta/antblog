import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'history_screen.dart';
import 'providers.dart';

/// Gốc ứng dụng: chế độ sáng/tối lấy từ tuỳ chọn đã lưu.
class ToiNoiApp extends ConsumerWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Trong lúc đọc tuỳ chọn (hoặc nếu đọc lỗi) theo hệ thống.
    final themeMode = ref.watch(themeModeProvider).value ?? ThemeMode.system;
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      themeMode: themeMode,
      home: const HistoryScreen(),
    );
  }
}

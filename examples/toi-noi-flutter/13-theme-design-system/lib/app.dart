import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'gallery_screen.dart';

/// App demo: bảng mẫu design system Tới Nơi.
///
/// [initialThemeMode] mặc định là [ThemeMode.system]: giao diện sáng/tối theo
/// cài đặt của điện thoại, người dùng có thể ghi đè bằng bộ chọn trên màn hình.
class ThemeGalleryApp extends StatefulWidget {
  const ThemeGalleryApp({super.key, this.initialThemeMode = ThemeMode.system});

  final ThemeMode initialThemeMode;

  @override
  State<ThemeGalleryApp> createState() => _ThemeGalleryAppState();
}

class _ThemeGalleryAppState extends State<ThemeGalleryApp> {
  late ThemeMode _themeMode = widget.initialThemeMode;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      themeMode: _themeMode,
      home: GalleryScreen(
        themeMode: _themeMode,
        onThemeModeChanged: (mode) => setState(() => _themeMode = mode),
      ),
    );
  }
}

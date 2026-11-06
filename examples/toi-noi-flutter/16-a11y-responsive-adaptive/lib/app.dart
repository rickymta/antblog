import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'adaptive_shell.dart';

/// App Tới Nơi cuối Phần 2: theme của `toi_noi_ui` và khung thích nghi.
class ToiNoiApp extends StatefulWidget {
  const ToiNoiApp({
    super.key,
    this.initialTrips = const [],
    this.clock = systemUtcNow,
  });

  final List<Trip> initialTrips;
  final UtcClock clock;

  @override
  State<ToiNoiApp> createState() => _ToiNoiAppState();
}

class _ToiNoiAppState extends State<ToiNoiApp> {
  // Dựng ThemeData một lần, không dựng lại trong mỗi build.
  final ThemeData _light = ToiNoiTheme.light();
  final ThemeData _dark = ToiNoiTheme.dark();

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: _light,
      darkTheme: _dark,
      // Chuỗi của chính framework (nút Back, "Tab 1 trong tổng số 3"...) lấy
      // từ MaterialLocalizations; thiếu dòng này chúng còn tiếng Anh.
      locale: const Locale('vi'),
      supportedLocales: const [Locale('vi')],
      localizationsDelegates: GlobalMaterialLocalizations.delegates,
      home: AdaptiveShell(
        initialTrips: widget.initialTrips,
        clock: widget.clock,
      ),
    );
  }
}

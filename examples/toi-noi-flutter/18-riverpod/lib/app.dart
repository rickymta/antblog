import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'booking_screen.dart';

/// Gốc ứng dụng. Không biết gì về Riverpod: `ProviderScope` đặt ở `main()`
/// (hoặc ở test) bên trên widget này.
class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      home: const BookingScreen(),
    );
  }
}

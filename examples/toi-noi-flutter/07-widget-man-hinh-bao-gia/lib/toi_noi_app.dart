import 'package:flutter/material.dart';

import 'quote_screen.dart';

/// Widget gốc của app: dựng `MaterialApp` với theme Material 3 và màn hình
/// báo giá làm màn hình đầu.
class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key, this.distanceKm = 5});

  /// Quãng đường báo giá, đơn vị km. Bài sau sẽ lấy từ form nhập điểm đi/đến.
  final double distanceKm;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
        useMaterial3: true,
      ),
      home: QuoteScreen(distanceKm: distanceKm),
    );
  }
}

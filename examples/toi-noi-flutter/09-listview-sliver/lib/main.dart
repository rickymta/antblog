import 'package:flutter/material.dart';

import 'trip_history_screen.dart';
import 'trip_history_source.dart';

void main() => runApp(ToiNoiApp(source: FakeTripHistorySource()));

/// Ứng dụng của bài 09: một màn hình lịch sử chuyến chạy trên dữ liệu giả.
class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key, required this.source});

  final TripHistorySource source;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
      ),
      home: TripHistoryScreen(source: source),
    );
  }
}

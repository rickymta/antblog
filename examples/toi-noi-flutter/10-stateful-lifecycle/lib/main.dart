import 'package:flutter/material.dart';

import 'find_driver_screen.dart';

void main() => runApp(const ToiNoiApp());

class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
      ),
      home: const FindDriverScreen(cancelDelay: Duration(milliseconds: 400)),
    );
  }
}

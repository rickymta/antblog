import 'package:flutter/material.dart';

import 'request_screen.dart';

void main() => runApp(const ToiNoiApp());

class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(colorSchemeSeed: Colors.teal),
      home: const RequestScreen(),
    );
  }
}

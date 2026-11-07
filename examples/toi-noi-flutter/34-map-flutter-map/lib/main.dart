import 'package:flutter/material.dart';

import 'pickup_screen.dart';

void main() => runApp(const ToiNoiMapApp());

class ToiNoiMapApp extends StatelessWidget {
  const ToiNoiMapApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Tới Nơi · chọn điểm đón',
    theme: ThemeData(colorSchemeSeed: const Color(0xFF0F766E)),
    home: Builder(
      builder: (context) => PickupScreen(
        onConfirm: (pickup) => ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Đã chọn ${pickup.latitude.toStringAsFixed(5)}, '
              '${pickup.longitude.toStringAsFixed(5)}',
            ),
          ),
        ),
      ),
    ),
  );
}

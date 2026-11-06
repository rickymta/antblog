import 'package:flutter/material.dart';

import 'fare.dart';

void main() {
  runApp(const ToiNoiApp());
}

class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
        useMaterial3: true,
      ),
      home: const QuoteScreen(),
    );
  }
}

class QuoteScreen extends StatelessWidget {
  const QuoteScreen({super.key});

  @override
  Widget build(BuildContext context) {
    const distanceKm = 5.0;
    final fareVnd = calculateFareVnd(distanceKm);

    return Scaffold(
      appBar: AppBar(title: const Text('Tới Nơi')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Quãng đường: ${distanceKm.toInt()} km'),
            Text('Cước minh hoạ: ${formatVnd(fareVnd)}'),
          ],
        ),
      ),
    );
  }
}

/// Định dạng số tiền bằng tay: 25500 thành "25.500 VND".
/// Bài về `intl` sẽ thay hàm này bằng định dạng theo locale.
String formatVnd(int amount) {
  final digits = amount.toString();
  final buffer = StringBuffer();
  for (var i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 == 0) {
      buffer.write('.');
    }
    buffer.write(digits[i]);
  }
  return '$buffer VND';
}

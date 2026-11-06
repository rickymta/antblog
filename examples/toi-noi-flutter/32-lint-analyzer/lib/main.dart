import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_32/composition.dart';
import 'package:toi_noi_32/presentation/quote_screen.dart';

void main() {
  runApp(
    ProviderScope(
      overrides: productionOverrides,
      child: const MaterialApp(home: QuoteScreen()),
    ),
  );
}

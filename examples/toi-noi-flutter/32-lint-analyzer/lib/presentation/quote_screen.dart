import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_32/presentation/quote_providers.dart';

class QuoteScreen extends ConsumerWidget {
  const QuoteScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final km = ref.watch(distanceProvider);
    final quote = ref.watch(quoteProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Báo giá Tới Nơi')),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('${km.toStringAsFixed(1)} km'),
            switch (quote) {
              AsyncData(:final value) => Text(
                value.toString(),
                key: const Key('quote'),
              ),
              AsyncError() => const Text('Không báo giá được'),
              _ => const CircularProgressIndicator(),
            },
            FilledButton(
              onPressed: () =>
                  ref.read(distanceProvider.notifier).setKm(km + 1),
              child: const Text('Thêm 1 km'),
            ),
          ],
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show appFlavor;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'providers.dart';

/// Màn hình chẩn đoán: bản build này đang chạy với cấu hình nào.
class EnvironmentScreen extends ConsumerWidget {
  const EnvironmentScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(appConfigProvider);
    final quote = ref.watch(sampleQuoteProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Môi trường')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(ToiNoiSpacing.md),
          children: [
            _Row('Môi trường', config.flavor.name),
            _Row('API', config.apiBaseUrl.toString()),
            _Row('Log HTTP', config.logHttp ? 'bật' : 'tắt'),
            _Row(
              'Timeout kết nối / phản hồi',
              '${config.connectTimeout.inMilliseconds} / '
                  '${config.receiveTimeout.inMilliseconds} ms',
            ),
            _Row('Flavor native', appFlavor ?? 'không đặt'),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(
              onPressed: quote.isLoading
                  ? null
                  : ref.read(sampleQuoteProvider.notifier).fetch,
              child: const Text('Báo giá thử 5 km'),
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            switch (quote) {
              AsyncData(value: final money?) => MoneyText(money),
              AsyncError(:final error) when !quote.isLoading => Text(
                'Không báo giá được: ${error.runtimeType}',
              ),
              AsyncLoading() => const LinearProgressIndicator(),
              _ => const SizedBox.shrink(),
            },
          ],
        ),
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.label, this.value);

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      dense: true,
      contentPadding: EdgeInsets.zero,
      title: Text(label),
      subtitle: Text(value),
    );
  }
}

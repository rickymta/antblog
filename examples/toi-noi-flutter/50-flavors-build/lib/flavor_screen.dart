import 'package:flutter/material.dart';

import 'flavor_check.dart';

/// Màn hình chẩn đoán: bản cài trên máy này đang là flavor nào.
///
/// Cài ba bản dev, staging, prod cạnh nhau, mở từng bản, màn hình này cho thấy
/// hai nguồn flavor có khớp không.
class FlavorScreen extends StatelessWidget {
  const FlavorScreen({super.key, required this.native, required this.config});

  final String? native;
  final String? config;

  static const messages = {
    FlavorCheck.consistent: 'Hai nguồn khớp nhau.',
    FlavorCheck.nativeMissing:
        'Không build bằng --flavor: đây là bản mang ID mặc định của runner.',
    FlavorCheck.configMissing:
        'Thiếu khoá FLAVOR: quên --dart-define-from-file khi build.',
    FlavorCheck.mismatch: 'Lệch flavor: --flavor và file cấu hình đang nói hai môi trường khác nhau.',
  };

  @override
  Widget build(BuildContext context) {
    final check = checkFlavors(native: native, config: config);
    final scheme = Theme.of(context).colorScheme;
    final ok = check == FlavorCheck.consistent;
    return Scaffold(
      appBar: AppBar(title: const Text('Flavor của bản build')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _Row(label: 'Flavor native (--flavor)', value: native),
            _Row(label: 'Flavor cấu hình (FLAVOR)', value: config),
            const SizedBox(height: 16),
            Card(
              color: ok ? scheme.secondaryContainer : scheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Text(messages[check]!, key: const Key('flavor-verdict')),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row({required this.label, required this.value});

  final String label;
  final String? value;

  @override
  Widget build(BuildContext context) {
    final shown = (value == null || value!.isEmpty) ? '(không đặt)' : value!;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(child: Text(label)),
          Text(shown, style: const TextStyle(fontWeight: FontWeight.bold)),
        ],
      ),
    );
  }
}

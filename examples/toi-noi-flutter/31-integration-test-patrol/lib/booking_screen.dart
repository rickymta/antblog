import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'booking_controller.dart';
import 'keys.dart';

class BookingScreen extends ConsumerStatefulWidget {
  const BookingScreen({super.key});

  @override
  ConsumerState<BookingScreen> createState() => _BookingScreenState();
}

class _BookingScreenState extends ConsumerState<BookingScreen> {
  final _distance = TextEditingController();
  ServiceKind _kind = ServiceKind.ride;

  @override
  void dispose() {
    _distance.dispose();
    super.dispose();
  }

  void _quote() {
    // Dấu phẩy thập phân kiểu Việt Nam cũng được nhận.
    final km = double.tryParse(_distance.text.trim().replaceAll(',', '.'));
    if (km == null) {
      return;
    }
    ref.read(bookingProvider.notifier).requestQuote(_kind, km);
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(bookingProvider);
    final busy = state is BookingQuoting || state is BookingSubmitting;
    final fare = switch (state) {
      BookingQuoted(:final fare) || BookingSubmitting(:final fare) => fare,
      _ => null,
    };
    final message = switch (state) {
      BookingIdle(:final message) || BookingQuoted(:final message) => message,
      _ => null,
    };
    return Scaffold(
      appBar: AppBar(title: const Text('Đặt chuyến')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Wrap(
            spacing: 8,
            children: [
              for (final kind in ServiceKind.values)
                ChoiceChip(
                  key: AppKeys.service(kind),
                  label: Text(kind.label),
                  selected: _kind == kind,
                  onSelected: busy ? null : (_) => setState(() => _kind = kind),
                ),
            ],
          ),
          const SizedBox(height: 16),
          TextField(
            key: AppKeys.distanceField,
            controller: _distance,
            enabled: !busy,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            decoration: const InputDecoration(
              labelText: 'Quãng đường (km)',
              border: OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 16),
          FilledButton(
            key: AppKeys.quoteButton,
            onPressed: busy ? null : _quote,
            child: const Text('Báo giá'),
          ),
          if (message != null) ...[
            const SizedBox(height: 16),
            Text(
              message,
              key: AppKeys.errorText,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ],
          if (fare != null) ...[
            const SizedBox(height: 16),
            Text(
              'Giá dự kiến: $fare',
              key: AppKeys.fareText,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 16),
            FilledButton.tonal(
              key: AppKeys.bookButton,
              onPressed: busy
                  ? null
                  : () => ref.read(bookingProvider.notifier).book(),
              child: const Text('Đặt chuyến'),
            ),
          ],
          if (busy) ...[
            const SizedBox(height: 16),
            const LinearProgressIndicator(),
          ],
        ],
      ),
    );
  }
}

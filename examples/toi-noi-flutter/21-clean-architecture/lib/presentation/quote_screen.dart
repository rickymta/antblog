import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'failure_view.dart';
import 'providers.dart';

/// Màn hình báo giá và đặt chuyến. Chỉ biết domain và provider, không biết dio.
class QuoteScreen extends ConsumerWidget {
  const QuoteScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final form = ref.watch(quoteRequestProvider);
    final quote = ref.watch(quoteControllerProvider);
    final booking = ref.watch(bookingControllerProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Báo giá chuyến')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(ToiNoiSpacing.md),
          children: [
            Wrap(
              spacing: ToiNoiSpacing.sm,
              children: [
                for (final kind in ServiceKind.values)
                  ChoiceChip(
                    label: Text(kind.label),
                    selected: form.serviceKind == kind,
                    onSelected: (_) => ref
                        .read(quoteRequestProvider.notifier)
                        .selectService(kind),
                  ),
              ],
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            Text('Quãng đường: ${form.distanceKm.toStringAsFixed(1)} km'),
            Slider(
              value: form.distanceKm,
              max: 100,
              divisions: 200,
              label: '${form.distanceKm.toStringAsFixed(1)} km',
              onChanged: ref.read(quoteRequestProvider.notifier).setDistanceKm,
            ),
            FilledButton(
              onPressed: quote.isLoading
                  ? null
                  : ref.read(quoteControllerProvider.notifier).fetch,
              child: const Text('Báo giá'),
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            _QuoteResult(quote: quote),
            const SizedBox(height: ToiNoiSpacing.md),
            _BookingResult(booking: booking),
          ],
        ),
      ),
    );
  }
}

class _QuoteResult extends ConsumerWidget {
  const _QuoteResult({required this.quote});

  final AsyncValue<Money?> quote;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final booking = ref.watch(bookingControllerProvider);
    return switch (quote) {
      AsyncLoading() => const Center(child: CircularProgressIndicator()),
      AsyncError(:final error) => FailureView(
        error: error,
        onRetry: ref.read(quoteControllerProvider.notifier).fetch,
      ),
      AsyncData(value: null) => const Text(
        'Chọn dịch vụ, kéo quãng đường rồi bấm Báo giá.',
      ),
      AsyncData(:final value?) => Card(
        child: Padding(
          padding: const EdgeInsets.all(ToiNoiSpacing.md),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Cước ước tính'),
              MoneyText(
                value,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: ToiNoiSpacing.sm),
              OutlinedButton(
                onPressed: booking.isLoading
                    ? null
                    : ref.read(bookingControllerProvider.notifier).book,
                child: const Text('Đặt chuyến'),
              ),
            ],
          ),
        ),
      ),
    };
  }
}

class _BookingResult extends ConsumerWidget {
  const _BookingResult({required this.booking});

  final AsyncValue<Trip?> booking;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return switch (booking) {
      AsyncLoading() => const Center(child: CircularProgressIndicator()),
      AsyncError(:final error) => FailureView(
        error: error,
        onRetry: ref.read(bookingControllerProvider.notifier).retry,
      ),
      AsyncData(value: null) => const SizedBox.shrink(),
      AsyncData(:final value?) => Card(
        child: Padding(
          padding: const EdgeInsets.all(ToiNoiSpacing.md),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Chuyến ${value.id}'),
              const SizedBox(height: ToiNoiSpacing.xs),
              TripStatusChip(status: value.status),
              const SizedBox(height: ToiNoiSpacing.xs),
              Text(
                value.driverId == null
                    ? 'Chưa có tài xế'
                    : 'Tài xế ${value.driverId}',
              ),
              MoneyText(value.fare),
              const SizedBox(height: ToiNoiSpacing.sm),
              OutlinedButton(
                onPressed: ref.read(bookingControllerProvider.notifier).refresh,
                child: const Text('Làm mới'),
              ),
            ],
          ),
        ),
      ),
    };
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'error_text.dart';
import 'providers.dart';

/// Màn hình báo giá và đặt chuyến, nói chuyện với fake API qua `dio`.
class QuoteScreen extends ConsumerWidget {
  const QuoteScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final form = ref.watch(quoteFormProvider);
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
                        .read(quoteFormProvider.notifier)
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
              onChanged: ref.read(quoteFormProvider.notifier).setDistanceKm,
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
      AsyncError(:final error) => _ErrorText(error: error),
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
              // Đã có chuyến thì ẩn nút: bấm nữa sẽ ra chuyến thứ hai.
              if (booking.value == null)
                OutlinedButton(
                  onPressed: booking.isLoading
                      ? null
                      : ref.read(bookingControllerProvider.notifier).book,
                  child: Text(booking.hasError ? 'Thử đặt lại' : 'Đặt chuyến'),
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
    // `value` còn chuyến cũ khi đang làm mới hoặc làm mới lỗi (copyWithPrevious).
    final trip = booking.value;
    if (trip == null) {
      return switch (booking) {
        AsyncLoading() => const Center(child: CircularProgressIndicator()),
        AsyncError(:final error) => _ErrorText(error: error),
        AsyncData() => const SizedBox.shrink(),
      };
    }
    final controller = ref.read(bookingControllerProvider.notifier);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Chuyến ${trip.id}'),
            const SizedBox(height: ToiNoiSpacing.xs),
            TripStatusChip(status: trip.status),
            const SizedBox(height: ToiNoiSpacing.xs),
            Text(
              trip.driverId == null
                  ? 'Chưa có tài xế'
                  : 'Tài xế ${trip.driverId}',
            ),
            MoneyText(trip.fare),
            if (booking.isLoading)
              const Center(child: CircularProgressIndicator()),
            if (booking.hasError) _ErrorText(error: booking.error!),
            const SizedBox(height: ToiNoiSpacing.sm),
            Wrap(
              spacing: ToiNoiSpacing.sm,
              children: [
                OutlinedButton(
                  onPressed: booking.isLoading ? null : controller.refresh,
                  child: Text(booking.hasError ? 'Thử lại' : 'Làm mới'),
                ),
                TextButton(
                  onPressed: booking.isLoading ? null : controller.reset,
                  child: const Text('Đặt chuyến khác'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _ErrorText extends StatelessWidget {
  const _ErrorText({required this.error});

  final Object error;

  @override
  Widget build(BuildContext context) {
    final text = describeError(error);
    final color = Theme.of(context).colorScheme.error;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(text.message, style: TextStyle(color: color)),
        if (text.supportCode != null) Text('Mã hỗ trợ: ${text.supportCode}'),
      ],
    );
  }
}

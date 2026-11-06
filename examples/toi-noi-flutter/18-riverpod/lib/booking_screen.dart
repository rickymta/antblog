import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'error_text.dart';
import 'providers.dart';
import 'quote.dart';
import 'trip_draft.dart';
import 'trip_screen.dart';

/// Màn hình đặt chuyến: chọn dịch vụ, chỉnh quãng đường, xem báo giá, đặt.
///
/// `ConsumerWidget` đưa thêm `WidgetRef` vào `build`. Mọi thứ màn hình biết
/// đều đến từ provider; nó không giữ state riêng.
class BookingScreen extends ConsumerWidget {
  const BookingScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // listen: việc phụ (SnackBar, chuyển trang) khi state đổi. Chỉ chạy khi
    // có thay đổi, không chạy ở lần build đầu.
    ref.listen(quoteProvider, (previous, next) {
      // Trong lúc thử lại, state là AsyncLoading nhưng vẫn mang lỗi cũ;
      // chỉ báo khi thật sự hết lượt.
      if (next.hasError && !next.isLoading) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Không lấy được báo giá: ${describeError(next.error!)}',
            ),
          ),
        );
      }
    });
    ref.listen(placeTripProvider, (previous, next) {
      switch (next) {
        case AsyncData(value: final trip?):
          Navigator.of(context).push(
            MaterialPageRoute<void>(
              builder: (_) => TripScreen(tripId: trip.id),
            ),
          );
        case AsyncError(:final error) when !next.isLoading:
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Không đặt được chuyến: ${describeError(error)}'),
            ),
          );
        case _:
          break;
      }
    });

    // watch: phần giao diện phụ thuộc state; state đổi thì build lại.
    final quote = ref.watch(quoteProvider);
    final placing = ref.watch(placeTripProvider).isLoading;

    return Scaffold(
      appBar: AppBar(title: const Text('Đặt chuyến')),
      body: ListView(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          const ServicePicker(),
          const SizedBox(height: ToiNoiSpacing.md),
          const DistanceStepper(),
          const SizedBox(height: ToiNoiSpacing.lg),
          switch (quote) {
            AsyncData(:final value) => QuoteCard(
              quote: value,
              onBook: placing
                  ? null
                  // read: trong handler chỉ cần lấy notifier, không đăng ký
                  // theo dõi.
                  : () => ref.read(placeTripProvider.notifier).place(value),
            ),
            AsyncError(:final error) when !quote.isLoading => QuoteError(
              message: describeError(error),
              onRetry: () => ref.invalidate(quoteProvider),
            ),
            AsyncError() ||
            AsyncLoading() => QuoteLoading(retrying: quote.retrying),
          },
        ],
      ),
    );
  }
}

/// `5.0` hiển thị là "5 km", `2.5` là "2.5 km".
String formatKm(double km) =>
    '${km == km.roundToDouble() ? km.toInt() : km} km';

/// Chọn dịch vụ. Chỉ theo dõi `serviceKind` của bản nháp, nên đổi quãng đường
/// không làm widget này build lại.
class ServicePicker extends ConsumerWidget {
  const ServicePicker({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selected = ref.watch(
      tripDraftProvider.select((draft) => draft.serviceKind),
    );
    return Wrap(
      spacing: ToiNoiSpacing.sm,
      children: [
        for (final kind in ServiceKind.values)
          ChoiceChip(
            label: Text(kind.label),
            selected: kind == selected,
            onSelected: (_) =>
                ref.read(tripDraftProvider.notifier).selectService(kind),
          ),
      ],
    );
  }
}

/// Tăng giảm quãng đường theo từng km, trong miền 1..100 km.
class DistanceStepper extends ConsumerWidget {
  const DistanceStepper({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final km = ref.watch(tripDraftProvider.select((draft) => draft.distanceKm));
    return Row(
      children: [
        IconButton(
          key: const Key('less'),
          tooltip: 'Giảm 1 km',
          onPressed: km > 1
              ? () => ref.read(tripDraftProvider.notifier).setDistanceKm(km - 1)
              : null,
          icon: const Icon(Icons.remove),
        ),
        Text(formatKm(km), key: const Key('distance')),
        IconButton(
          key: const Key('more'),
          tooltip: 'Tăng 1 km',
          onPressed: km < FarePolicy.maxKm
              ? () => ref.read(tripDraftProvider.notifier).setDistanceKm(km + 1)
              : null,
          icon: const Icon(Icons.add),
        ),
      ],
    );
  }
}

class QuoteCard extends StatelessWidget {
  const QuoteCard({super.key, required this.quote, required this.onBook});

  final Quote quote;
  final VoidCallback? onBook;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(quote.serviceKind.label),
            MoneyText(
              quote.fare,
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(onPressed: onBook, child: const Text('Đặt chuyến')),
          ],
        ),
      ),
    );
  }
}

class QuoteLoading extends StatelessWidget {
  const QuoteLoading({super.key, required this.retrying});

  final bool retrying;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const CircularProgressIndicator(),
        const SizedBox(height: ToiNoiSpacing.sm),
        Text(retrying ? 'Mạng chập chờn, đang thử lại…' : 'Đang báo giá…'),
      ],
    );
  }
}

class QuoteError extends StatelessWidget {
  const QuoteError({super.key, required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(message),
        const SizedBox(height: ToiNoiSpacing.sm),
        FilledButton.tonal(onPressed: onRetry, child: const Text('Thử lại')),
      ],
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'providers.dart';
import 'quote.dart';
import 'quote_repository.dart';

/// Màn hình báo giá: chọn dịch vụ, nhập quãng đường, xem cước, đặt chuyến.
class QuoteScreen extends ConsumerStatefulWidget {
  const QuoteScreen({super.key});

  @override
  ConsumerState<QuoteScreen> createState() => _QuoteScreenState();
}

class _QuoteScreenState extends ConsumerState<QuoteScreen> {
  final _distance = TextEditingController(text: '5');
  String? _distanceError;
  bool _placing = false;

  @override
  void dispose() {
    _distance.dispose();
    super.dispose();
  }

  /// Chấp nhận dấu phẩy thập phân kiểu Việt ("2,001"); NaN và vô cực không
  /// qua được phép so sánh miền.
  void _applyDistance() {
    final km = double.tryParse(_distance.text.trim().replaceAll(',', '.'));
    final valid = km != null && km >= 0 && km <= FarePolicy.maxKm;
    setState(() {
      _distanceError = valid ? null : 'Nhập số từ 0 đến ${FarePolicy.maxKm} km';
    });
    if (valid) ref.read(quoteRequestProvider.notifier).setDistanceKm(km);
  }

  Future<void> _placeTrip(Quote quote) async {
    setState(() => _placing = true);
    try {
      final trip = await ref.read(quoteRepositoryProvider).placeTrip(quote);
      if (!mounted) return;
      context.push('/trip/${trip.id}');
    } on QuoteUnavailable {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Chưa đặt được chuyến, thử lại sau')),
      );
    } finally {
      if (mounted) setState(() => _placing = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final request = ref.watch(quoteRequestProvider);
    final quote = ref.watch(quoteProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Báo giá chuyến')),
      body: Stack(
        children: [
          SingleChildScrollView(
            padding: const EdgeInsets.all(ToiNoiSpacing.md),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Wrap(
                  spacing: ToiNoiSpacing.sm,
                  children: [
                    for (final kind in ServiceKind.values)
                      ChoiceChip(
                        key: Key('service-${kind.name}'),
                        label: Text(kind.label),
                        selected: request.serviceKind == kind,
                        onSelected: (_) => ref
                            .read(quoteRequestProvider.notifier)
                            .selectService(kind),
                      ),
                  ],
                ),
                const SizedBox(height: ToiNoiSpacing.md),
                TextField(
                  key: const Key('distance-field'),
                  controller: _distance,
                  keyboardType: const TextInputType.numberWithOptions(
                    decimal: true,
                  ),
                  decoration: InputDecoration(
                    labelText: 'Quãng đường (km)',
                    errorText: _distanceError,
                  ),
                  onSubmitted: (_) => _applyDistance(),
                ),
                const SizedBox(height: ToiNoiSpacing.sm),
                FilledButton(
                  key: const Key('apply-distance'),
                  onPressed: _applyDistance,
                  child: const Text('Xem báo giá'),
                ),
                const SizedBox(height: ToiNoiSpacing.lg),
                // `isLoading` đứng trước: sau `ref.invalidate` state vẫn là
                // AsyncError (hay AsyncData) nhưng mang cờ isLoading = true.
                switch (quote) {
                  AsyncLoading() ||
                  AsyncValue(isLoading: true) => const _QuoteLoading(),
                  AsyncError() => _QuoteError(
                    onRetry: () => ref.invalidate(quoteProvider),
                  ),
                  AsyncData(:final value) => _QuoteDetails(
                    quote: value,
                    onPlace: () => _placeTrip(value),
                  ),
                },
              ],
            ),
          ),
          if (_placing) ...const [
            ModalBarrier(dismissible: false, color: Colors.black26),
            Center(child: CircularProgressIndicator(key: Key('placing'))),
          ],
        ],
      ),
    );
  }
}

class _QuoteLoading extends StatelessWidget {
  const _QuoteLoading();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: CircularProgressIndicator(key: Key('quote-loading')),
    );
  }
}

class _QuoteError extends StatelessWidget {
  const _QuoteError({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Column(
      key: const Key('quote-error'),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Dịch vụ báo giá đang bận'),
        const SizedBox(height: ToiNoiSpacing.sm),
        OutlinedButton(
          key: const Key('retry'),
          onPressed: onRetry,
          child: const Text('Thử lại'),
        ),
      ],
    );
  }
}

class _QuoteDetails extends StatelessWidget {
  const _QuoteDetails({required this.quote, required this.onPlace});

  final Quote quote;
  final VoidCallback onPlace;

  @override
  Widget build(BuildContext context) {
    const base = Money(FarePolicy.baseFareVnd);
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Card(
          key: const Key('quote-card'),
          child: Padding(
            padding: const EdgeInsets.all(ToiNoiSpacing.md),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(quote.serviceKind.label),
                const SizedBox(height: ToiNoiSpacing.xs),
                MoneyText(
                  quote.fare,
                  key: const Key('quote-fare'),
                  style: theme.textTheme.headlineMedium,
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: ToiNoiSpacing.md),
        Column(
          key: const Key('fare-breakdown'),
          children: [
            _BreakdownRow(label: 'Giá mở cửa (2 km đầu)', amount: base),
            _BreakdownRow(label: 'Phần vượt', amount: quote.fare - base),
            _BreakdownRow(
              label: 'Tổng',
              amount: quote.fare,
              amountKey: const Key('breakdown-total'),
            ),
          ],
        ),
        const ListTile(
          contentPadding: EdgeInsets.zero,
          leading: Icon(Icons.payments_outlined),
          title: Text('Thanh toán'),
          subtitle: Text('Tiền mặt'),
        ),
        const SizedBox(height: ToiNoiSpacing.sm),
        Text(
          'Giá minh hoạ, không phải bảng giá thực tế. Huỷ miễn phí khi '
          'chưa có tài xế nhận.',
          style: theme.textTheme.bodySmall,
        ),
        const SizedBox(height: ToiNoiSpacing.md),
        FilledButton(
          key: const Key('place-trip'),
          onPressed: onPlace,
          child: const Text('Đặt chuyến'),
        ),
      ],
    );
  }
}

class _BreakdownRow extends StatelessWidget {
  const _BreakdownRow({
    required this.label,
    required this.amount,
    this.amountKey,
  });

  final String label;
  final Money amount;
  final Key? amountKey;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: ToiNoiSpacing.xs),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          // Nhãn dài thì xuống dòng, số tiền giữ nguyên chỗ.
          Expanded(child: Text(label)),
          const SizedBox(width: ToiNoiSpacing.sm),
          MoneyText(amount, key: amountKey),
        ],
      ),
    );
  }
}

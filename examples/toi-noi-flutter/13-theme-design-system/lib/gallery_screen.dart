import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Màn hình bảng mẫu: mỗi khối cho thấy một phần của design system.
class GalleryScreen extends StatelessWidget {
  const GalleryScreen({
    required this.themeMode,
    required this.onThemeModeChanged,
    super.key,
  });

  final ThemeMode themeMode;
  final ValueChanged<ThemeMode> onThemeModeChanged;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Design system Tới Nơi')),
      body: ListView(
        // Mỗi Text một node semantics riêng, để textContrastGuideline kiểm được
        // từng đoạn chữ (xem test/gallery_test.dart).
        addSemanticIndexes: false,
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          const _SectionTitle('Chế độ giao diện'),
          SegmentedButton<ThemeMode>(
            segments: const [
              ButtonSegment(value: ThemeMode.system, label: Text('Hệ thống')),
              ButtonSegment(value: ThemeMode.light, label: Text('Sáng')),
              ButtonSegment(value: ThemeMode.dark, label: Text('Tối')),
            ],
            selected: {themeMode},
            onSelectionChanged: (selection) =>
                onThemeModeChanged(selection.single),
          ),
          const _SectionTitle('Báo giá'),
          const _QuoteCard(distanceKm: 5),
          const SizedBox(height: ToiNoiSpacing.md),
          const TextField(decoration: InputDecoration(labelText: 'Điểm đến')),
          const _SectionTitle('Trạng thái chuyến'),
          Wrap(
            spacing: ToiNoiSpacing.sm,
            runSpacing: ToiNoiSpacing.sm,
            children: [
              for (final status in TripStatus.values)
                TripStatusChip(status: status),
            ],
          ),
          const _SectionTitle('Màu của theme'),
          const _Swatches(),
          const _SectionTitle('Chữ'),
          const _TypeSamples(),
        ],
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(
        top: ToiNoiSpacing.lg,
        bottom: ToiNoiSpacing.sm,
      ),
      child: Text(text, style: Theme.of(context).textTheme.titleLarge),
    );
  }
}

class _QuoteCard extends StatelessWidget {
  const _QuoteCard({required this.distanceKm});

  final double distanceKm;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Xe máy · ${_formatKm(distanceKm)} km',
              style: textTheme.bodyMedium,
            ),
            const SizedBox(height: ToiNoiSpacing.xs),
            MoneyText(
              calculateFare(distanceKm),
              style: textTheme.headlineSmall,
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(onPressed: () {}, child: const Text('Đặt xe')),
          ],
        ),
      ),
    );
  }
}

/// `5.0` thành `5`, `2.5` giữ nguyên.
String _formatKm(double km) =>
    km == km.roundToDouble() ? '${km.toInt()}' : '$km';

/// Một ô màu: nền và màu chữ đặt trên nền đó.
typedef _Swatch = ({String name, Color background, Color foreground});

class _Swatches extends StatelessWidget {
  const _Swatches();

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final colors = ToiNoiColors.of(context);
    final swatches = <_Swatch>[
      (
        name: 'primary',
        background: scheme.primary,
        foreground: scheme.onPrimary,
      ),
      (
        name: 'primaryContainer',
        background: scheme.primaryContainer,
        foreground: scheme.onPrimaryContainer,
      ),
      (
        name: 'surfaceContainerHighest',
        background: scheme.surfaceContainerHighest,
        foreground: scheme.onSurface,
      ),
      (name: 'error', background: scheme.error, foreground: scheme.onError),
      (
        name: 'success',
        background: colors.success,
        foreground: colors.onSuccess,
      ),
      (
        name: 'warning',
        background: colors.warning,
        foreground: colors.onWarning,
      ),
      (name: 'danger', background: colors.danger, foreground: colors.onDanger),
    ];

    return Wrap(
      spacing: ToiNoiSpacing.sm,
      runSpacing: ToiNoiSpacing.sm,
      children: [
        for (final s in swatches)
          DecoratedBox(
            decoration: BoxDecoration(
              color: s.background,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: ToiNoiSpacing.md,
                vertical: ToiNoiSpacing.sm,
              ),
              child: Text(
                s.name,
                style: Theme.of(context).textTheme.labelLarge
                    ?.copyWith(color: s.foreground),
              ),
            ),
          ),
      ],
    );
  }
}

class _TypeSamples extends StatelessWidget {
  const _TypeSamples();

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('headlineSmall', style: t.headlineSmall),
        Text('titleLarge', style: t.titleLarge),
        Text('titleMedium', style: t.titleMedium),
        Text(
          'bodyMedium: cước minh hoạ, không phải bảng giá thật.',
          style: t.bodyMedium,
        ),
        Text('labelLarge', style: t.labelLarge),
      ],
    );
  }
}

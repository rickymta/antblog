import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'formatting.dart';
import 'l10n/app_localizations.dart';

/// Màn hình báo giá song ngữ: logo từ asset, quãng đường, cước VND, thời điểm
/// đặt theo giờ máy, số tài xế ở gần (số nhiều ICU) và bộ chọn ngôn ngữ.
class QuoteScreen extends StatefulWidget {
  const QuoteScreen({
    required this.distanceKm,
    required this.requestedAt,
    required this.onLocaleChanged,
    this.utcOffset,
    this.initialDrivers = 3,
    super.key,
  });

  final double distanceKm;

  /// Thời điểm đặt chuyến, UTC.
  final DateTime requestedAt;

  /// Độ lệch múi giờ cố định; `null` là dùng múi giờ của máy.
  final Duration? utcOffset;

  final int initialDrivers;
  final ValueChanged<Locale> onLocaleChanged;

  @override
  State<QuoteScreen> createState() => _QuoteScreenState();
}

class _QuoteScreenState extends State<QuoteScreen> {
  late int _drivers = widget.initialDrivers;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final textTheme = Theme.of(context).textTheme;
    final fare = calculateFare(widget.distanceKm);

    return Scaffold(
      appBar: AppBar(title: Text(l10n.appTitle)),
      body: ListView(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          Row(
            children: [
              Image.asset(
                'assets/images/logo.png',
                width: 48,
                height: 48,
                semanticLabel: l10n.logoSemantics,
              ),
              const SizedBox(width: ToiNoiSpacing.md),
              Text(l10n.quoteTitle, style: textTheme.headlineSmall),
            ],
          ),
          const SizedBox(height: ToiNoiSpacing.lg),
          Text(l10n.quoteDistance(widget.distanceKm)),
          Row(
            children: [
              Text(l10n.quoteFare, style: textTheme.titleLarge),
              const SizedBox(width: ToiNoiSpacing.sm),
              MoneyText(
                fare,
                style: textTheme.titleLarge,
                formatter: (money) => formatVnd(money, l10n.localeName),
              ),
            ],
          ),
          Text(
            l10n.quoteRequestedAt(
              formatTripTime(
                widget.requestedAt,
                l10n.localeName,
                offset: widget.utcOffset,
              ),
            ),
          ),
          const SizedBox(height: ToiNoiSpacing.lg),
          Row(
            children: [
              IconButton(
                tooltip: l10n.removeDriver,
                onPressed: _drivers > 0
                    ? () => _setDrivers(_drivers - 1)
                    : null,
                icon: const Icon(Icons.remove),
              ),
              IconButton(
                tooltip: l10n.addDriver,
                onPressed: () => _setDrivers(_drivers + 1),
                icon: const Icon(Icons.add),
              ),
              Expanded(child: Text(l10n.driversNearby(_drivers))),
            ],
          ),
          TripStatusChip(
            status: TripStatus.searching,
            label: l10n.statusSearching,
          ),
          const SizedBox(height: ToiNoiSpacing.lg),
          Text(l10n.languageLabel, style: textTheme.titleMedium),
          const SizedBox(height: ToiNoiSpacing.sm),
          SegmentedButton<String>(
            segments: [
              ButtonSegment(value: 'vi', label: Text(l10n.languageVi)),
              ButtonSegment(value: 'en', label: Text(l10n.languageEn)),
            ],
            selected: {Localizations.localeOf(context).languageCode},
            onSelectionChanged: (selection) =>
                widget.onLocaleChanged(Locale(selection.single)),
          ),
        ],
      ),
    );
  }

  void _setDrivers(int value) => setState(() => _drivers = value);
}

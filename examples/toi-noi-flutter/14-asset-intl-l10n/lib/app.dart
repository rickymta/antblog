import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'l10n/app_localizations.dart';
import 'quote_screen.dart';

/// App demo song ngữ vi/en.
///
/// [initialLocale] là `null` khi để Flutter chọn theo ngôn ngữ của thiết bị;
/// bộ chọn trên màn hình ghi đè bằng một locale cụ thể.
class ToiNoiL10nApp extends StatefulWidget {
  const ToiNoiL10nApp({
    required this.requestedAt,
    this.distanceKm = 5,
    this.utcOffset,
    this.initialLocale,
    super.key,
  });

  final double distanceKm;
  final DateTime requestedAt;
  final Duration? utcOffset;
  final Locale? initialLocale;

  @override
  State<ToiNoiL10nApp> createState() => _ToiNoiL10nAppState();
}

class _ToiNoiL10nAppState extends State<ToiNoiL10nApp> {
  late Locale? _locale = widget.initialLocale;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      // onGenerateTitle chạy dưới Localizations nên dịch được tên app.
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      locale: _locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: QuoteScreen(
        distanceKm: widget.distanceKm,
        requestedAt: widget.requestedAt,
        utcOffset: widget.utcOffset,
        onLocaleChanged: (locale) => setState(() => _locale = locale),
      ),
    );
  }
}

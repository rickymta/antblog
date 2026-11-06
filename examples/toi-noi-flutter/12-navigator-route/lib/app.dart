import 'package:flutter/material.dart';

import 'clock.dart';
import 'quote.dart';
import 'quote_screen.dart';
import 'routes.dart';

/// Ứng dụng Tới Nơi của bài 12: một `Navigator` với route có tên cho màn hình
/// gốc và lịch sử, còn `/confirm` đi qua `onGenerateRoute` vì cần tham số.
class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({
    super.key,
    this.quote = demoQuote,
    this.clock = systemClock,
  });

  final Quote quote;
  final Clock clock;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(colorSchemeSeed: Colors.teal),
      initialRoute: AppRoutes.quote,
      routes: {
        AppRoutes.quote: (_) => QuoteScreen(quote: quote, clock: clock),
        AppRoutes.history: (_) => const HistoryPlaceholderScreen(),
      },
      onGenerateRoute: (settings) => onGenerateAppRoute(settings, clock: clock),
      onUnknownRoute: (settings) => MaterialPageRoute<void>(
        settings: settings,
        builder: (_) =>
            RouteErrorScreen(message: 'Không có route ${settings.name}'),
      ),
    );
  }
}

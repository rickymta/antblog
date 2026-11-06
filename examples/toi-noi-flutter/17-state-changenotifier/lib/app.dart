import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'home_shell.dart';
import 'quote_draft.dart';
import 'quote_scope.dart';
import 'trip_book.dart';

/// App Tới Nơi của bài 17.
///
/// `State` của widget này *sở hữu* hai notifier: tạo trong `initState`, huỷ
/// trong `dispose`. Hai scope nằm trên `MaterialApp` để mọi route đẩy bằng
/// `Navigator` cũng thấy chúng.
class ToiNoiApp extends StatefulWidget {
  const ToiNoiApp({
    super.key,
    this.initialTrips = const [],
    this.clock = systemUtcNow,
  });

  final List<Trip> initialTrips;
  final UtcClock clock;

  @override
  State<ToiNoiApp> createState() => _ToiNoiAppState();
}

class _ToiNoiAppState extends State<ToiNoiApp> {
  final QuoteDraft _draft = QuoteDraft();
  late final TripBook _book = TripBook(
    initial: widget.initialTrips,
    clock: widget.clock,
  );
  final ThemeData _light = ToiNoiTheme.light();
  final ThemeData _dark = ToiNoiTheme.dark();

  @override
  void dispose() {
    _draft.dispose();
    _book.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return QuoteScope(
      draft: _draft,
      child: TripBookScope(
        book: _book,
        child: MaterialApp(
          title: 'Tới Nơi',
          theme: _light,
          darkTheme: _dark,
          home: const HomeShell(),
        ),
      ),
    );
  }
}

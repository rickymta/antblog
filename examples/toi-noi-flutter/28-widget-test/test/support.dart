import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_28/app.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote.dart';
import 'package:toi_noi_28/quote_repository.dart';
import 'package:toi_noi_28/quote_screen.dart';
import 'package:toi_noi_28/router.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Repository giả do test điều khiển bằng [Completer]: yêu cầu nào cũng treo
/// cho tới khi test gọi `complete`. Không có Timer nào, nên không cần
/// `pump(Duration)` để "chờ".
final class ControlledQuoteRepository implements QuoteRepository {
  final quoteRequests = <({ServiceKind serviceKind, double distanceKm})>[];
  final _quotes = <Completer<Quote>>[];
  final _places = <Completer<Trip>>[];

  int get fetchCount => quoteRequests.length;
  int get placeCount => _places.length;

  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) {
    quoteRequests.add((serviceKind: serviceKind, distanceKm: distanceKm));
    final completer = Completer<Quote>();
    _quotes.add(completer);
    return completer.future;
  }

  @override
  Future<Trip> placeTrip(Quote quote) {
    final completer = Completer<Trip>();
    _places.add(completer);
    return completer.future;
  }

  /// Trả lời yêu cầu báo giá thứ [index] (mặc định yêu cầu mới nhất) bằng
  /// cước tính từ `calculateFare`.
  void completeQuote({int? index}) {
    final i = index ?? _quotes.length - 1;
    final request = quoteRequests[i];
    _quotes[i].complete(
      Quote(
        serviceKind: request.serviceKind,
        distanceKm: request.distanceKm,
        fare: calculateFare(request.distanceKm),
      ),
    );
  }

  void failQuote({int? index}) {
    final i = index ?? _quotes.length - 1;
    _quotes[i].completeError(const QuoteUnavailable());
  }

  void completePlace(Quote quote, {String id = 'T-0001'}) {
    _places.last.complete(
      Trip.request(
        id: id,
        serviceKind: quote.serviceKind,
        distanceKm: quote.distanceKm,
        requestedAt: DateTime.utc(2026, 11, 6, 8),
      ),
    );
  }

  void failPlace() => _places.last.completeError(const QuoteUnavailable());
}

/// Bọc [QuoteScreen] trong `ProviderScope` có override và `MaterialApp` dùng
/// theme thật của app (không có router: đủ cho test màn hình đơn).
Future<void> pumpQuoteScreen(
  WidgetTester tester, {
  List<Override> overrides = const [],
}) {
  return tester.pumpWidget(
    ProviderScope(
      overrides: overrides,
      child: MaterialApp(theme: ToiNoiTheme.light(), home: const QuoteScreen()),
    ),
  );
}

/// Dựng cả app với `GoRouter` thật. Trả về router để test đọc vị trí.
/// Router được huỷ khi test kết thúc.
Future<GoRouter> pumpApp(
  WidgetTester tester, {
  required QuoteRepository repository,
  String initialLocation = '/',
}) async {
  final router = createRouter(initialLocation: initialLocation);
  addTearDown(router.dispose);
  await tester.pumpWidget(
    ProviderScope(
      overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      child: ToiNoiApp(router: router),
    ),
  );
  return router;
}

/// Đặt màn hình 360x640 logical (1080x1920 pixel vật lý, tỷ lệ 3) và hẹn
/// trả về mặc định khi test kết thúc.
void usePhoneScreen(WidgetTester tester) {
  tester.view.physicalSize = const Size(1080, 1920);
  tester.view.devicePixelRatio = 3;
  addTearDown(tester.view.reset);
}

/// Văn bản trong ô nhập quãng đường.
String distanceText(WidgetTester tester) => tester
    .widget<TextField>(find.byKey(const Key('distance-field')))
    .controller!
    .text;

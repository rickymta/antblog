import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/app.dart';
import 'package:toi_noi_18/booking_screen.dart';
import 'package:toi_noi_18/quote.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_18/providers.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Báo giá đúng như server sẽ trả: cước luôn do `calculateFare` tính.
Quote quoteFor(double km, {ServiceKind kind = ServiceKind.ride}) =>
    Quote(serviceKind: kind, distanceKm: km, fare: calculateFare(km));

/// Chuyến `requested` với giờ cố định.
Trip tripFor(String id, {double km = 5, ServiceKind kind = ServiceKind.ride}) =>
    Trip.request(
      id: id,
      serviceKind: kind,
      distanceKm: km,
      requestedAt: DateTime.utc(2026, 11, 6, 8),
    );

/// Repository mà từng test điều khiển bằng hàm: ghi lại mọi lần gọi và để
/// test quyết định khi nào, và bằng gì, mỗi lần gọi hoàn thành.
final class StubQuoteRepository implements QuoteRepository {
  StubQuoteRepository({this.onQuote, this.onPlace, this.onTrip});

  final Future<Quote> Function(ServiceKind kind, double km)? onQuote;
  final Future<Trip> Function(Quote quote)? onPlace;
  final Future<Trip> Function(String id)? onTrip;

  final List<({ServiceKind kind, double km})> quoteRequests = [];
  final List<Quote> placed = [];
  final List<String> tripRequests = [];

  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) {
    quoteRequests.add((kind: serviceKind, km: distanceKm));
    final handler = onQuote;
    return handler != null
        ? handler(serviceKind, distanceKm)
        : Future.value(quoteFor(distanceKm, kind: serviceKind));
  }

  @override
  Future<Trip> placeTrip(Quote quote) {
    placed.add(quote);
    final handler = onPlace;
    return handler != null
        ? handler(quote)
        : Future.value(tripFor('T-0001', km: quote.distanceKm));
  }

  @override
  Future<Trip> fetchTrip(String id) {
    tripRequests.add(id);
    final handler = onTrip;
    return handler != null ? handler(id) : Future.value(tripFor(id));
  }
}

/// Dựng app trong `ProviderScope` có ghi đè repository, trả về container để
/// test hỏi state của provider.
Future<ProviderContainer> pumpApp(
  WidgetTester tester, {
  required QuoteRepository repository,
  List<Override> overrides = const [],
}) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        quoteRepositoryProvider.overrideWithValue(repository),
        ...overrides,
      ],
      child: const ToiNoiApp(),
    ),
  );
  return ProviderScope.containerOf(tester.element(find.byType(BookingScreen)));
}

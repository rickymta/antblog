import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_28/quote.dart';
import 'package:toi_noi_28/quote_screen.dart';
import 'package:toi_noi_28/trip_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

const _place = Key('place-trip');

/// Đưa app tới state có báo giá và bấm "Đặt chuyến"; yêu cầu đặt chuyến
/// đang treo ở repository.
Future<({ControlledQuoteRepository repository, GoRouter router})> _startPlacing(
  WidgetTester tester,
) async {
  final repository = ControlledQuoteRepository();
  final router = await pumpApp(tester, repository: repository);
  repository.completeQuote();
  await tester.pump();
  await tester.ensureVisible(find.byKey(_place));
  await tester.tap(find.byKey(_place));
  await tester.pump();
  return (repository: repository, router: router);
}

void main() {
  final quote5km = Quote(
    serviceKind: ServiceKind.ride,
    distanceKm: 5,
    fare: calculateFare(5),
  );

  group('điều hướng với go_router', () {
    testWidgets('đặt chuyến thành công: sang màn hình chuyến T-0001', (
      tester,
    ) async {
      final (:repository, :router) = await _startPlacing(tester);
      expect(router.state.uri.path, '/');

      repository.completePlace(quote5km);
      await tester.pumpAndSettle();

      expect(find.byType(TripScreen), findsOneWidget);
      expect(
        find.descendant(
          of: find.byType(AppBar),
          matching: find.text('Chuyến T-0001'),
        ),
        findsOneWidget,
      );
      expect(find.byType(TripStatusChip), findsOneWidget);
      expect(find.text(TripStatus.requested.label), findsOneWidget);
      // `push`: vị trí của trang trên cùng nằm ở router.state.
      expect(router.state.uri.path, '/trip/T-0001');
    });

    testWidgets('màn hình báo giá còn nằm dưới, bị Offstage', (tester) async {
      final (:repository, router: _) = await _startPlacing(tester);
      repository.completePlace(quote5km);
      await tester.pumpAndSettle();

      // Mặc định finder bỏ qua widget Offstage (route bên dưới).
      expect(find.byType(QuoteScreen), findsNothing);
      expect(find.byType(QuoteScreen, skipOffstage: false), findsOneWidget);
    });

    testWidgets('Back về báo giá và giữ nguyên quãng đường đã nhập', (
      tester,
    ) async {
      final repository = ControlledQuoteRepository();
      final router = await pumpApp(tester, repository: repository);
      repository.completeQuote();
      await tester.pump();

      await tester.enterText(find.byKey(const Key('distance-field')), '12');
      await tester.tap(find.byKey(const Key('apply-distance')));
      await tester.pump();
      repository.completeQuote();
      await tester.pump();
      await tester.ensureVisible(find.byKey(_place));
      await tester.tap(find.byKey(_place));
      await tester.pump();
      repository.completePlace(quote5km);
      await tester.pumpAndSettle();
      expect(router.state.uri.path, '/trip/T-0001');

      await tester.pageBack();
      await tester.pumpAndSettle();

      expect(router.state.uri.path, '/');
      expect(find.byType(TripScreen), findsNothing);
      expect(distanceText(tester), '12');
    });

    testWidgets('đặt chuyến lỗi: ở lại báo giá, hiện SnackBar, bỏ lớp phủ', (
      tester,
    ) async {
      final (:repository, :router) = await _startPlacing(tester);
      expect(find.byKey(const Key('placing')), findsOneWidget);

      repository.failPlace();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 750));

      expect(router.state.uri.path, '/');
      expect(find.byType(TripScreen), findsNothing);
      expect(find.byKey(const Key('placing')), findsNothing);
      expect(
        find.widgetWithText(SnackBar, 'Chưa đặt được chuyến, thử lại sau'),
        findsOneWidget,
      );
      // Có thể bấm lại ngay.
      expect(find.byKey(_place), findsOneWidget);
    });

    testWidgets('mở thẳng /trip/T-0009 (deep link): không có màn báo giá', (
      tester,
    ) async {
      final repository = ControlledQuoteRepository();
      final router = await pumpApp(
        tester,
        repository: repository,
        initialLocation: '/trip/T-0009',
      );

      expect(router.state.uri.path, '/trip/T-0009');
      expect(find.text('Chuyến T-0009'), findsOneWidget);
      expect(find.byType(QuoteScreen, skipOffstage: false), findsNothing);
      // Chưa ai xem báo giá nên chưa có yêu cầu nào tới repository.
      expect(repository.fetchCount, 0);
    });

    testWidgets('router.go thay cả ngăn xếp, router.push thì chồng lên', (
      tester,
    ) async {
      final repository = ControlledQuoteRepository();
      final router = await pumpApp(tester, repository: repository);
      repository.completeQuote();
      await tester.pump();

      router.push('/trip/T-0002');
      await tester.pumpAndSettle();
      expect(find.byType(QuoteScreen, skipOffstage: false), findsOneWidget);

      router.go('/trip/T-0003');
      await tester.pumpAndSettle();
      expect(find.text('Chuyến T-0003'), findsOneWidget);
      expect(find.byType(QuoteScreen, skipOffstage: false), findsNothing);
    });
  });
}

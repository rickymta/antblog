import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_37/push_router.dart';

/// Vì sao `PushRouter` kiểm `tripId` thay vì nối thẳng vào đường dẫn: `Uri`
/// của Dart đã chuẩn hoá `..` ngay khi parse (`Uri.parse('/trip/../admin').path`
/// là `/admin`), nên một payload lạ có thể nhảy sang route khác.
void main() {
  Future<GoRouter> pumpRouter(WidgetTester tester) async {
    final router = GoRouter(
      routes: [
        GoRoute(
          path: '/',
          builder: (context, state) => const Text('home'),
          routes: [
            GoRoute(
              path: 'trip/:id',
              builder: (context, state) =>
                  Text('trip ${state.pathParameters['id']}'),
            ),
            GoRoute(
              path: 'admin',
              builder: (context, state) => const Text('ADMIN'),
            ),
          ],
        ),
      ],
    );
    await tester.pumpWidget(MaterialApp.router(routerConfig: router));
    return router;
  }

  testWidgets('cách sai: nối thẳng tripId "../admin" thì tới route /admin', (
    tester,
  ) async {
    final router = await pumpRouter(tester);
    const tripId = '../admin'; // lấy nguyên từ payload, không kiểm
    router.go('/trip/$tripId');
    await tester.pumpAndSettle();
    expect(find.text('ADMIN'), findsOneWidget);
  });

  testWidgets('cách đúng: PushRouter từ chối nên không đi đâu cả', (
    tester,
  ) async {
    final router = await pumpRouter(tester);
    final decision = const PushRouter().decide({
      'type': 'trip_update',
      'tripId': '../admin',
    });
    expect(decision, isA<IgnorePush>());
    if (decision is OpenTrip) router.go(decision.location);
    await tester.pumpAndSettle();
    expect(find.text('home'), findsOneWidget);
  });
}

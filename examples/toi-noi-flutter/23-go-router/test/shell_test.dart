import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';

import 'support.dart';

double sliderValue(WidgetTester tester) =>
    tester.widget<Slider>(find.byKey(const Key('km-slider'))).value;

Future<void> setKm(WidgetTester tester, double km) async {
  tester.widget<Slider>(find.byKey(const Key('km-slider'))).onChanged!(km);
  await tester.pump();
}

int selectedTab(WidgetTester tester) =>
    tester.widget<NavigationBar>(find.byType(NavigationBar)).selectedIndex;

void main() {
  group('StatefulShellRoute.indexedStack', () {
    testWidgets('ba tab, mỗi tab một đường dẫn riêng', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      expect(selectedTab(tester), 0);
      expect(locationOf(c.router), '/');

      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();
      expect(selectedTab(tester), 1);
      expect(locationOf(c.router), '/history');

      await tester.tap(tab('Tài khoản'));
      await tester.pumpAndSettle();
      expect(selectedTab(tester), 2);
      expect(locationOf(c.router), '/account');
    });

    testWidgets('đổi tab rồi quay lại: state của tab Đặt xe còn nguyên', (
      tester,
    ) async {
      await pumpApp(tester, signedInAs: alice);
      expect(sliderValue(tester), 5);
      await setKm(tester, 12);

      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();
      await tester.tap(tab('Tài khoản'));
      await tester.pumpAndSettle();
      await tester.tap(tab('Đặt xe'));
      await tester.pumpAndSettle();

      expect(sliderValue(tester), 12);
      expect(find.text('Quãng đường: 12 km'), findsOneWidget);
    });

    testWidgets('tab đang ẩn vẫn nằm trong cây widget (IndexedStack)', (
      tester,
    ) async {
      await pumpApp(tester, signedInAs: alice);
      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();

      expect(
        find.byKey(const Key('km-slider'), skipOffstage: true),
        findsNothing,
      );
      expect(
        find.byKey(const Key('km-slider'), skipOffstage: false),
        findsOneWidget,
        reason: 'tab Đặt xe chỉ bị ẩn, không bị huỷ',
      );
    });

    testWidgets('tab chưa mở thì chưa dựng (mặc định không preload)', (
      tester,
    ) async {
      await pumpApp(tester, signedInAs: alice);
      expect(
        find.text('T-1001 · Đặt xe', skipOffstage: false),
        findsNothing,
        reason: 'tab Lịch sử chưa được mở lần nào',
      );
      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();
      expect(find.text('T-1001 · Đặt xe'), findsOneWidget);
    });

    testWidgets('push chuyến từ tab: Back về đúng chỗ, state tab còn nguyên', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await setKm(tester, 12);

      c.router.push('/trip/T-1001');
      await tester.pumpAndSettle();
      expect(find.text('Chuyến T-1001'), findsOneWidget);
      expect(find.byType(NavigationBar), findsNothing, reason: 'toàn màn hình');

      await tester.tap(find.byType(BackButton));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/');
      expect(sliderValue(tester), 12);
    });

    testWidgets('go tới /trip/:id thay cả khung tab: state tab mất', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await setKm(tester, 12);

      c.router.go('/trip/T-1001');
      await tester.pumpAndSettle();
      c.router.go('/');
      await tester.pumpAndSettle();

      expect(sliderValue(tester), 5, reason: 'shell bị dựng lại từ đầu');
    });

    testWidgets('StatefulNavigationShell biết tab hiện tại qua router.go', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/account');
      await tester.pumpAndSettle();
      expect(selectedTab(tester), 2);
      expect(
        StatefulNavigationShell.maybeOf(
          tester.element(find.text('Đang đăng nhập: An')),
        )?.currentIndex,
        2,
      );
    });
  });
}

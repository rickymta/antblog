import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_23/deep_links.dart';

import 'support.dart';

void main() {
  group('khởi động nguội bằng link (defaultRouteName)', () {
    testWidgets('custom scheme toinoi://trip/{id} mở thẳng chuyến', (
      tester,
    ) async {
      launchWithLink(tester, DeepLinks.customTrip('T-1002'));
      final c = await pumpApp(tester, signedInAs: alice);

      expect(locationOf(c.router), '/trip/T-1002');
      expect(find.text('Chuyến T-1002'), findsOneWidget);
      expect(find.text('12.005 VND'), findsOneWidget);
      expect(c.router.canPop(), isFalse, reason: 'ngăn xếp chỉ có màn này');
    });

    testWidgets('universal link https://toinoi.example/trip/{id} cũng vậy', (
      tester,
    ) async {
      launchWithLink(tester, DeepLinks.webTrip('T-1001'));
      final c = await pumpApp(tester, signedInAs: alice);
      expect(locationOf(c.router), '/trip/T-1001');
      expect(find.text('25.500 VND'), findsOneWidget);
    });

    testWidgets('query của link được giữ', (tester) async {
      launchWithLink(tester, Uri.parse('toinoi://trip/T-1002?ref=push'));
      final c = await pumpApp(tester, signedInAs: alice);
      expect(c.router.state.uri.queryParameters, {'ref': 'push'});
    });

    testWidgets(
      'chưa đăng nhập: sang /login với from là đường dẫn, không phải link',
      (tester) async {
        launchWithLink(tester, DeepLinks.webTrip('T-1002'));
        final c = await pumpApp(tester);
        expect(locationOf(c.router), '/login?from=%2Ftrip%2FT-1002');

        await tester.tap(find.byKey(const Key('sign-in')));
        await tester.pumpAndSettle();
        expect(locationOf(c.router), '/trip/T-1002');
        expect(find.text('12.005 VND'), findsOneWidget);
      },
    );

    testWidgets(
      'link tới khi kho phiên còn đang đọc: chờ rồi vào thẳng chuyến',
      (tester) async {
        launchWithLink(tester, DeepLinks.customTrip('T-1002'));
        await tester.pumpWidget(
          pumpAppWidget(
            signedInAs: alice,
            restoreLatency: const Duration(seconds: 3),
          ),
        );
        await tester.pump(const Duration(seconds: 1));
        expect(find.text('Chuyến T-1002'), findsNothing);
        await tester.pump(const Duration(seconds: 3));
        await tester.pumpAndSettle();
        expect(find.text('Chuyến T-1002'), findsOneWidget);
      },
    );
  });

  group('app đang chạy nhận link (pushRouteInformation)', () {
    testWidgets('custom scheme: mở chuyến, thay cả ngăn xếp hiện tại', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();
      c.router.push('/trip/T-1001');
      await tester.pumpAndSettle();
      expect(c.router.canPop(), isTrue);

      await receiveLink(tester, DeepLinks.customTrip('T-1002'));

      expect(find.text('Chuyến T-1002'), findsOneWidget);
      expect(c.router.canPop(), isFalse, reason: 'link là go, không phải push');
    });

    testWidgets(
      'routeInformationProvider.value phản ánh đường dẫn đã redirect',
      (tester) async {
        final c = await pumpApp(tester, signedInAs: alice);
        await receiveLink(tester, DeepLinks.customTrip('T-1002'));

        final value = c.router.routeInformationProvider.value.uri;
        expect(value.hasScheme, isFalse);
        expect(value.path, '/trip/T-1002');
      },
    );

    testWidgets('universal link', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await receiveLink(tester, DeepLinks.webTrip('T-1003'));
      expect(locationOf(c.router), '/trip/T-1003');
      expect(find.text('453.000 VND'), findsOneWidget);
    });

    testWidgets('chưa đăng nhập: login rồi quay lại đúng chuyến', (
      tester,
    ) async {
      final c = await pumpApp(tester);
      await receiveLink(tester, DeepLinks.customTrip('T-1002'));
      expect(locationOf(c.router), '/login?from=%2Ftrip%2FT-1002');

      await tester.tap(find.byKey(const Key('sign-in')));
      await tester.pumpAndSettle();
      expect(find.text('Chuyến T-1002'), findsOneWidget);
    });

    testWidgets('mã chuyến không tồn tại: màn hình lỗi của chuyến', (
      tester,
    ) async {
      await pumpApp(tester, signedInAs: alice);
      await receiveLink(tester, DeepLinks.customTrip('T-9999'));
      expect(find.text('Không tìm thấy chuyến T-9999.'), findsOneWidget);
    });

    testWidgets('host lạ của custom scheme: không tìm thấy', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await receiveLink(tester, Uri.parse('toinoi://payment/confirm'));
      expect(locationOf(c.router), startsWith('/not-found'));
      expect(find.text('toinoi://payment/confirm'), findsOneWidget);
    });

    testWidgets('https tới tên miền khác: không tìm thấy', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await receiveLink(tester, Uri.parse('https://evil.example/trip/T-1002'));
      expect(locationOf(c.router), startsWith('/not-found'));
      expect(find.text('Chuyến T-1002'), findsNothing);
    });

    testWidgets('toinoi://trip thiếu mã: không tìm thấy', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await receiveLink(tester, Uri.parse('toinoi://trip'));
      expect(locationOf(c.router), startsWith('/not-found'));
    });

    testWidgets('universal link tới đường dẫn app không có: không tìm thấy', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await receiveLink(tester, Uri.parse('https://toinoi.example/pricing'));
      expect(locationOf(c.router), startsWith('/not-found'));
    });

    testWidgets('link tới trang chủ web mở tab Đặt xe', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/account');
      await tester.pumpAndSettle();
      await receiveLink(tester, Uri.parse('https://toinoi.example'));
      expect(locationOf(c.router), '/');
      expect(find.byKey(const Key('km-slider')), findsOneWidget);
    });
  });
}

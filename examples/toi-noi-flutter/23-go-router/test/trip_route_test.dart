import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_23/trips.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  group('/trip/:id', () {
    testWidgets('mở bằng router.go: lấy id từ đường dẫn và hiện đúng chuyến', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/trip/T-1002');
      await tester.pumpAndSettle();

      expect(find.text('Chuyến T-1002'), findsOneWidget);
      expect(find.text('Giao đồ ăn, 2.001 km'), findsOneWidget);
      expect(find.text('12.005 VND'), findsOneWidget);
      expect(find.text('Đang chở / giao'), findsOneWidget);
    });

    testWidgets('cước các chuyến mẫu đúng công thức', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      for (final (id, fare) in [
        ('T-1001', '25.500 VND'),
        ('T-1003', '453.000 VND'),
      ]) {
        c.router.go('/trip/$id');
        await tester.pumpAndSettle();
        expect(find.text(fare), findsOneWidget, reason: id);
      }
    });

    testWidgets('id không tồn tại: màn hình lỗi có nút về lịch sử', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/trip/T-9999');
      await tester.pumpAndSettle();

      expect(find.text('Không tìm thấy chuyến T-9999.'), findsOneWidget);
      expect(find.text('12.005 VND'), findsNothing);
      // Lỗi này không thử lại: chỉ một lần gọi repository.
      await tester.tap(find.byKey(const Key('back-to-history')));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/history');
      expect(find.text('T-1001 · Đặt xe'), findsOneWidget);
    });

    testWidgets('TripNotFound không bị Riverpod thử lại 10 lần', (
      tester,
    ) async {
      final repo = _CountingRepository();
      final c = await pumpApp(tester, signedInAs: alice, trips: repo);
      c.router.go('/trip/T-9999');
      await tester.pumpAndSettle();
      expect(repo.byIdCalls, 1);
    });

    testWidgets('mở từ lịch sử bằng push: Back về lịch sử', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('T-1002')));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/trip/T-1002');
      expect(c.router.canPop(), isTrue);

      await tester.tap(find.byType(BackButton));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/history');
      expect(find.byKey(const ValueKey('T-1002')), findsOneWidget);
    });

    testWidgets(
      'mở bằng go (như deep link): không có gì để pop, Back về /history',
      (tester) async {
        final c = await pumpApp(tester, signedInAs: alice);
        c.router.go('/trip/T-1002');
        await tester.pumpAndSettle();
        expect(c.router.canPop(), isFalse);

        await tester.tap(find.byType(BackButton));
        await tester.pumpAndSettle();
        expect(locationOf(c.router), '/history');
      },
    );

    testWidgets('đặt chuyến mới rồi push: chuyến hiện ra và có trong lịch sử', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      await tester.tap(find.byKey(const Key('place-trip')));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/trip/T-1005');
      expect(find.text('Chuyến T-1005'), findsOneWidget);
      expect(find.text('Đã đặt'), findsOneWidget);
      expect(find.text('25.500 VND'), findsOneWidget);

      await tester.tap(find.byType(BackButton));
      await tester.pumpAndSettle();
      await tester.tap(tab('Lịch sử'));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('T-1005')), findsOneWidget);
    });

    testWidgets('mã có ký tự lạ được mã hoá và giải mã đúng', (tester) async {
      final repo = FakeTripRepository(trips: [sampleTrips().first]);
      final c = await pumpApp(tester, signedInAs: alice, trips: repo);
      c.router.go('/trip/a%2Fb');
      await tester.pumpAndSettle();
      expect(find.text('Không tìm thấy chuyến a/b.'), findsOneWidget);
    });
  });

  group('đường dẫn không có route', () {
    testWidgets('onException đưa tới màn hình không tìm thấy kèm link gốc', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/khong-co-trang-nay');
      await tester.pumpAndSettle();

      expect(
        find.text('Link này không dẫn tới màn hình nào trong app.'),
        findsOneWidget,
      );
      expect(find.text('/khong-co-trang-nay'), findsOneWidget);
      expect(locationOf(c.router), startsWith('/not-found'));

      await tester.tap(find.byKey(const Key('go-home')));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/');
    });

    testWidgets('/trip không có id: cũng không tìm thấy', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/trip');
      await tester.pumpAndSettle();
      expect(locationOf(c.router), startsWith('/not-found'));
    });

    testWidgets(
      'chưa đăng nhập vào đường dẫn lạ: đăng nhập trước, rồi mới thấy không tìm thấy',
      (tester) async {
        final c = await pumpApp(tester);
        c.router.go('/khong-co-trang-nay');
        await tester.pumpAndSettle();
        expect(locationOf(c.router), '/login?from=%2Fkhong-co-trang-nay');

        await tester.tap(find.byKey(const Key('sign-in')));
        await tester.pumpAndSettle();
        expect(locationOf(c.router), startsWith('/not-found'));
      },
    );
  });
}

final class _CountingRepository extends FakeTripRepository {
  int byIdCalls = 0;

  @override
  Future<Trip> byId(String id) {
    byIdCalls++;
    return super.byId(id);
  }
}

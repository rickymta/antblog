import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_12/app.dart';
import 'package:toi_noi_12/confirm_trip_screen.dart';
import 'package:toi_noi_12/quote.dart';
import 'package:toi_noi_12/routes.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

DateTime _clock() => DateTime.utc(2026, 11, 6, 8, 30);

NavigatorState _navigator(WidgetTester tester) =>
    tester.state<NavigatorState>(find.byType(Navigator));

void main() {
  group('named route', () {
    testWidgets('/history nằm trong bảng routes và mở từ nút trên AppBar', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));

      await tester.tap(find.byTooltip('Lịch sử chuyến'));
      await tester.pumpAndSettle();

      expect(find.text('Chưa có chuyến nào'), findsOneWidget);
    });

    testWidgets('pushNamed kèm Quote mở màn hình xác nhận và trả Trip', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));

      Trip? result;
      _navigator(tester)
          .pushNamed<Trip>(AppRoutes.confirm, arguments: demoQuote)
          .then((trip) => result = trip);
      await tester.pumpAndSettle();
      expect(find.byType(ConfirmTripScreen), findsOneWidget);

      await tester.tap(find.text('Xác nhận đặt chuyến'));
      await tester.pumpAndSettle();

      expect(result?.id, 'T-0001');
      expect(result?.fare, const Money(25500));
    });

    testWidgets('sai kiểu arguments chỉ lộ ra lúc chạy: hiện màn hình lỗi', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));

      // Compiler không phản đối: arguments là Object?.
      _navigator(tester).pushNamed<Trip>(AppRoutes.confirm, arguments: '5 km');
      await tester.pumpAndSettle();

      expect(find.byType(ConfirmTripScreen), findsNothing);
      expect(find.text('/confirm cần một Quote, nhận String'), findsOneWidget);
    });

    testWidgets('thiếu arguments cũng chỉ lộ ra lúc chạy', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));

      _navigator(tester).pushNamed<Trip>(AppRoutes.confirm);
      await tester.pumpAndSettle();

      expect(find.text('/confirm cần một Quote, nhận Null'), findsOneWidget);
    });

    testWidgets('sai kiểu trả về T của pushNamed cũng chỉ lộ ra lúc chạy', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));

      // Route /confirm là MaterialPageRoute<Trip>, nhưng pushNamed<int> vẫn
      // biên dịch được. Lỗi ép kiểu nổ ngay khi push.
      expect(
        () =>
            _navigator(tester)
                .pushNamed<int>(AppRoutes.confirm, arguments: demoQuote),
        throwsA(isA<TypeError>()),
      );
    });

    testWidgets('tên không có trong bảng đi vào onUnknownRoute', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));

      _navigator(tester).pushNamed<void>('/khong-co');
      await tester.pumpAndSettle();

      expect(find.text('Không có route /khong-co'), findsOneWidget);
    });
  });
}

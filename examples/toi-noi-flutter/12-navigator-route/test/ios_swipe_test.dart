import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_12/app.dart';
import 'package:toi_noi_12/confirm_trip_screen.dart';

DateTime _clock() => DateTime.utc(2026, 11, 6, 8, 30);

Future<void> _openConfirm(WidgetTester tester) async {
  await tester.tap(find.text('Đặt chuyến'));
  await tester.pumpAndSettle();
}

void main() {
  group('vuốt từ mép trái trên iOS', () {
    testWidgets('khi chưa thay đổi: vuốt quay lại được', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);

      await tester.dragFrom(const Offset(2, 300), const Offset(500, 0));
      await tester.pumpAndSettle();

      expect(find.byType(ConfirmTripScreen), findsNothing);
    }, variant: TargetPlatformVariant.only(TargetPlatform.iOS));

    testWidgets(
      'khi có thay đổi: vuốt không làm gì, cũng không hiện hộp thoại',
      (tester) async {
        await tester.pumpWidget(const ToiNoiApp(clock: _clock));
        await _openConfirm(tester);
        await tester.tap(find.text('Thanh toán'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Ví VíXanh').last);
        await tester.pumpAndSettle();

        await tester.dragFrom(const Offset(2, 300), const Offset(500, 0));
        await tester.pumpAndSettle();

        expect(find.byType(ConfirmTripScreen), findsOneWidget);
        expect(find.text('Rời màn hình?'), findsNothing);

        // Nút back trên AppBar vẫn đi qua maybePop và hiện hộp thoại.
        await tester.tap(find.byType(BackButton));
        await tester.pumpAndSettle();
        expect(find.text('Rời màn hình?'), findsOneWidget);
      },
      variant: TargetPlatformVariant.only(TargetPlatform.iOS),
    );
  });
}

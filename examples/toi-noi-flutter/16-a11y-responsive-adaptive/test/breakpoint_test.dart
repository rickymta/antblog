import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/history_screen.dart';
import 'package:toi_noi_16/trip_detail.dart';
import 'package:toi_noi_16/window_class.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

Future<void> openHistory(WidgetTester tester) async {
  await tester.tap(find.text('Lịch sử').last);
  await tester.pumpAndSettle();
}

void main() {
  group('WindowClass.fromWidth', () {
    test('ngưỡng 600 và 840, ngưỡng thuộc về lớp lớn hơn', () {
      expect(WindowClass.fromWidth(0), WindowClass.compact);
      expect(WindowClass.fromWidth(320), WindowClass.compact);
      expect(WindowClass.fromWidth(599.9), WindowClass.compact);
      expect(WindowClass.fromWidth(600), WindowClass.medium);
      expect(WindowClass.fromWidth(839.9), WindowClass.medium);
      expect(WindowClass.fromWidth(840), WindowClass.expanded);
      expect(WindowClass.fromWidth(1280), WindowClass.expanded);
    });
  });

  group('khung app đổi bố cục theo bề rộng cửa sổ', () {
    testWidgets('390: NavigationBar, không có NavigationRail', (tester) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);

      expect(find.byType(NavigationBar), findsOneWidget);
      expect(find.byType(NavigationRail), findsNothing);
    });

    testWidgets('700: NavigationRail, không có NavigationBar', (tester) async {
      setScreen(tester, 700, 800);
      await pumpApp(tester);

      expect(find.byType(NavigationRail), findsOneWidget);
      expect(find.byType(NavigationBar), findsNothing);
    });

    testWidgets('1000: NavigationRail và hai cột ở màn lịch sử', (
      tester,
    ) async {
      setScreen(tester, 1000, 800);
      await pumpApp(tester);
      await openHistory(tester);

      expect(find.byType(NavigationRail), findsOneWidget);
      expect(find.text('Chọn một chuyến để xem chi tiết'), findsOneWidget);
      // Cột danh sách rộng 360 dù cửa sổ rộng 1000: widget con chỉ được cấp
      // phần cha chia cho, không phải cả cửa sổ.
      expect(tester.getSize(find.byType(HistoryScreen)).width, 360);
    });

    testWidgets('ngay ở ngưỡng 599 và 600 bố cục đổi', (tester) async {
      setScreen(tester, 599, 800);
      await pumpApp(tester);
      expect(find.byType(NavigationBar), findsOneWidget);

      setScreen(tester, 600, 800);
      await tester.pump();
      expect(find.byType(NavigationRail), findsOneWidget);
    });
  });

  group('chạm một chuyến', () {
    testWidgets('compact: đẩy trang chi tiết lên Navigator', (tester) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);
      await openHistory(tester);

      await tester.tap(find.textContaining('T-1043'));
      await tester.pumpAndSettle();

      expect(find.byType(TripDetailPage), findsOneWidget);
      expect(find.text('Chuyến T-1043'), findsOneWidget);
    });

    testWidgets('expanded: hiện chi tiết ở cột phải, không đẩy trang', (
      tester,
    ) async {
      setScreen(tester, 1000, 800);
      await pumpApp(tester);
      await openHistory(tester);

      await tester.tap(find.textContaining('T-1043'));
      await tester.pumpAndSettle();

      expect(find.byType(TripDetailPage), findsNothing);
      expect(find.byType(TripDetailView), findsOneWidget);
      expect(find.text('Mã chuyến T-1043'), findsOneWidget);
      // Danh sách vẫn còn bên trái.
      expect(find.textContaining('T-1045'), findsOneWidget);
    });

    testWidgets('expanded: huỷ ở cột phải cập nhật cả danh sách', (
      tester,
    ) async {
      setScreen(tester, 1000, 800);
      await pumpApp(tester);
      await openHistory(tester);
      await tester.tap(find.textContaining('T-1045'));
      await tester.pumpAndSettle();

      await tester.tap(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'));
      await tester.pumpAndSettle();
      await tester.tap(find.widgetWithText(TextButton, 'Huỷ chuyến'));
      await tester.pumpAndSettle();

      // Chi tiết đã sang Đã huỷ và không còn nút huỷ.
      expect(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'), findsNothing);
      expect(find.text('Đã huỷ'), findsWidgets);
    });
  });

  group('đổi cỡ cửa sổ giữa chừng (xoay máy)', () {
    testWidgets('quãng đường đang kéo không mất khi sang bố cục khác', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);

      final slider = find.byType(Slider);
      await tester.tap(slider); // chạm giữa thanh trượt (1 đến 20)
      await tester.pump();
      final before = tester.widget<Slider>(slider).value;
      expect(before, isNot(5));

      setScreen(tester, 1000, 500);
      await tester.pump();

      expect(find.byType(NavigationRail), findsOneWidget);
      expect(tester.widget<Slider>(slider).value, before);
    });

    testWidgets('đích đang mở không đổi khi sang bố cục khác', (tester) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);
      await openHistory(tester);

      setScreen(tester, 700, 800);
      await tester.pump();

      expect(find.byType(HistoryScreen), findsOneWidget);
      expect(find.byType(Slider), findsNothing);
    });
  });

  group('màn báo giá trên màn hình rộng', () {
    testWidgets('nội dung giới hạn bề rộng 560', (tester) async {
      setScreen(tester, 1000, 800);
      await pumpApp(tester);

      final card = tester.getSize(find.byType(Card).first);
      expect(card.width, lessThanOrEqualTo(560));
    });

    testWidgets('đặt chuyến thêm vào lịch sử với trạng thái đang tìm', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);

      await tester.tap(find.widgetWithText(FilledButton, 'Đặt chuyến'));
      await tester.pump();
      expect(find.text('Đã gửi yêu cầu T-1046'), findsOneWidget);

      await tester.pumpAndSettle(const Duration(seconds: 5));
      await openHistory(tester);
      expect(find.textContaining('T-1046'), findsWidgets);
      final trip = Trip.request(
        id: 'T-1046',
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        requestedAt: fixedClock(),
      );
      expect(find.text(trip.fare.toString()), findsWidgets);
    });
  });
}

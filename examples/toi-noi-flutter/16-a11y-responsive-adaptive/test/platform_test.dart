import 'package:flutter/cupertino.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/history_screen.dart';
import 'package:toi_noi_16/platform_adaptive.dart';
import 'package:toi_noi_16/trip_detail.dart';

import 'support.dart';

/// Mở trang chi tiết chuyến T-1045 (đang tìm tài xế nên huỷ được).
Future<void> openDetail(WidgetTester tester) async {
  await tester.tap(find.text('Lịch sử').last);
  await tester.pumpAndSettle();
  await tester.tap(find.textContaining('T-1045'));
  await tester.pumpAndSettle();
}

/// Kéo từ mép trái sang phải, cử chỉ quay lại của iOS.
Future<void> swipeFromLeftEdge(WidgetTester tester) async {
  await tester.dragFrom(const Offset(2, 300), const Offset(500, 0));
  await tester.pumpAndSettle();
}

final _ios = TargetPlatformVariant.only(TargetPlatform.iOS);
final _android = TargetPlatformVariant.only(TargetPlatform.android);

void main() {
  group('Theme.of(context).platform', () {
    testWidgets('mặc định của flutter test là android', (tester) async {
      await pumpApp(tester);
      final context = tester.element(find.byType(Scaffold));
      expect(Theme.of(context).platform, TargetPlatform.android);
      expect(isApplePlatform(context), isFalse);
    });

    testWidgets('debugDefaultTargetPlatformOverride đổi platform của theme', (
      tester,
    ) async {
      await withPlatform(TargetPlatform.iOS, () async {
        await pumpApp(tester);
        final context = tester.element(find.byType(Scaffold));
        expect(Theme.of(context).platform, TargetPlatform.iOS);
        expect(isApplePlatform(context), isTrue);
      });
      // Đã trả về null: không rò sang test khác.
      expect(debugDefaultTargetPlatformOverride, isNull);
    });

    testWidgets('Theme lồng nhau đổi platform chỉ cho nhánh con', (
      tester,
    ) async {
      late BuildContext outside;
      late BuildContext inside;
      await tester.pumpWidget(
        MaterialApp(
          home: Column(
            children: [
              Builder(
                builder: (context) {
                  outside = context;
                  return const SizedBox();
                },
              ),
              Theme(
                data: ThemeData(platform: TargetPlatform.iOS),
                child: Builder(
                  builder: (context) {
                    inside = context;
                    return const SizedBox();
                  },
                ),
              ),
            ],
          ),
        ),
      );
      expect(isApplePlatform(outside), isFalse);
      expect(isApplePlatform(inside), isTrue);
    });
  });

  group('hộp thoại huỷ chuyến: một hàm, hai giao diện', () {
    testWidgets('iOS: CupertinoAlertDialog, nút huỷ là destructive', (
      tester,
    ) async {
      await pumpApp(tester);
      await openDetail(tester);
      await tester.tap(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'));
      await tester.pumpAndSettle();

      expect(find.byType(CupertinoAlertDialog), findsOneWidget);
      expect(find.byType(Dialog), findsNothing);
      final action = tester.widget<CupertinoDialogAction>(
        find.widgetWithText(CupertinoDialogAction, 'Huỷ chuyến'),
      );
      expect(action.isDestructiveAction, isTrue);
    }, variant: _ios);

    testWidgets('Android: AlertDialog của Material, nút huỷ màu error', (
      tester,
    ) async {
      await pumpApp(tester);
      await openDetail(tester);
      await tester.tap(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'));
      await tester.pumpAndSettle();

      // AlertDialog.adaptive dựng AlertDialog của Material, bên trong là Dialog.
      expect(find.byType(Dialog), findsOneWidget);
      expect(find.byType(CupertinoAlertDialog), findsNothing);
      final button = tester.widget<TextButton>(
        find.widgetWithText(TextButton, 'Huỷ chuyến'),
      );
      final scheme = Theme.of(tester.element(find.byType(Dialog))).colorScheme;
      expect(button.style?.foregroundColor?.resolve({}), scheme.error);
    }, variant: _android);

    for (final platform in [TargetPlatform.iOS, TargetPlatform.android]) {
      testWidgets('${platform.name}: xác nhận huỷ đổi trạng thái sang Đã huỷ', (
        tester,
      ) async {
        await pumpApp(tester);
        await openDetail(tester);
        expect(find.text('Tài xế: chưa có'), findsOneWidget);

        await tester.tap(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'));
        await tester.pumpAndSettle();
        // Nút xác nhận có kiểu khác nhau tuỳ nền tảng nhưng cùng chữ.
        await tester.tap(
          find.widgetWithText(
            platform == TargetPlatform.iOS ? CupertinoDialogAction : TextButton,
            'Huỷ chuyến',
          ),
        );
        await tester.pumpAndSettle();

        expect(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'), findsNothing);
        expect(find.text('Đã huỷ'), findsWidgets);
      }, variant: TargetPlatformVariant.only(platform));

      testWidgets('${platform.name}: "Giữ chuyến" không đổi gì', (
        tester,
      ) async {
        await pumpApp(tester);
        await openDetail(tester);
        await tester.tap(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Giữ chuyến'));
        await tester.pumpAndSettle();

        expect(
          find.widgetWithText(OutlinedButton, 'Huỷ chuyến'),
          findsOneWidget,
        );
        expect(find.byType(TripDetailPage), findsOneWidget);
      }, variant: TargetPlatformVariant.only(platform));
    }
  });

  group('công tắc adaptive', () {
    // SwitchListTile.adaptive đọc ThemeData.platform để chọn kiểu vẽ; hành vi
    // phải giống nhau ở cả hai nền tảng.
    for (final platform in [TargetPlatform.iOS, TargetPlatform.android]) {
      testWidgets('${platform.name}: chạm đổi giá trị', (tester) async {
        setScreen(tester, 390, 800);
        await pumpApp(tester);
        await tester.tap(find.text('Cài đặt').last);
        await tester.pumpAndSettle();
        expect(
          tester.widget<SwitchListTile>(find.byType(SwitchListTile)).value,
          isFalse,
        );

        await tester.tap(find.byType(SwitchListTile));
        await tester.pumpAndSettle();

        expect(
          tester.widget<SwitchListTile>(find.byType(SwitchListTile)).value,
          isTrue,
        );
      }, variant: TargetPlatformVariant.only(platform));
    }
  });

  group('cử chỉ quay lại', () {
    testWidgets('iOS: vuốt từ mép trái đóng trang chi tiết', (tester) async {
      await pumpApp(tester);
      await openDetail(tester);
      expect(find.byType(CupertinoPageTransition), findsWidgets);

      await swipeFromLeftEdge(tester);

      expect(find.byType(TripDetailPage), findsNothing);
      expect(find.byType(HistoryScreen), findsOneWidget);
    }, variant: _ios);

    testWidgets('Android: Flutter không tự bắt cử chỉ vuốt từ mép trái', (
      tester,
    ) async {
      await pumpApp(tester);
      await openDetail(tester);
      expect(find.byType(CupertinoPageTransition), findsNothing);

      // Flutter không tự xử lý cử chỉ này trên Android. Với điều hướng cử chỉ
      // (Android 10 trở lên), hệ điều hành bắt nó và gửi Back tới app (xem test
      // `systemBack` bên dưới), nên ở máy thật trang vẫn có thể đóng.
      await swipeFromLeftEdge(tester);

      expect(find.byType(TripDetailPage), findsOneWidget);
    }, variant: _android);

    testWidgets('Android: Back của hệ thống đóng trang chi tiết', (
      tester,
    ) async {
      await pumpApp(tester);
      await openDetail(tester);

      await systemBack(tester);

      expect(find.byType(TripDetailPage), findsNothing);
    }, variant: _android);

    for (final platform in [TargetPlatform.iOS, TargetPlatform.android]) {
      testWidgets('${platform.name}: nút Back trên AppBar đóng trang', (
        tester,
      ) async {
        await pumpApp(tester);
        await openDetail(tester);

        // AppBar dùng BackButton chung; tester.pageBack() thì đòi nút kiểu
        // Cupertino trên iOS nên ta chạm thẳng BackButton.
        await tester.tap(find.byType(BackButton));
        await tester.pumpAndSettle();

        expect(find.byType(TripDetailPage), findsNothing);
      }, variant: TargetPlatformVariant.only(platform));
    }
  });

  group('cuộn: vật lý theo nền tảng', () {
    Future<double> dragPastTop(WidgetTester tester) async {
      // Màn thấp để bốn thẻ vượt khung nhìn: danh sách ngắn hơn khung thì
      // không cuộn được trên cả hai nền tảng.
      setScreen(tester, 390, 400);
      await pumpApp(tester);
      await tester.tap(find.text('Lịch sử').last);
      await tester.pumpAndSettle();
      final list = find.byType(ListView);
      final position = tester
          .state<ScrollableState>(
            find.descendant(of: list, matching: find.byType(Scrollable)),
          )
          .position;
      final gesture = await tester.startGesture(tester.getCenter(list));
      // Bước đầu bị trừ vào ngưỡng nhận diện kéo (slop), nên kéo nhiều bước.
      for (var i = 0; i < 4; i++) {
        await gesture.moveBy(const Offset(0, 30));
        await tester.pump();
      }
      final pixels = position.pixels;
      await gesture.up();
      await tester.pumpAndSettle();
      return pixels;
    }

    testWidgets('iOS: kéo quá đầu danh sách được (bounce)', (tester) async {
      expect(await dragPastTop(tester), lessThan(0));
    }, variant: _ios);

    testWidgets('Android: kéo quá đầu danh sách bị chặn ở 0', (tester) async {
      expect(await dragPastTop(tester), 0);
    }, variant: _android);
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/broken_examples.dart';

import 'support.dart';

/// Ba guideline về vùng chạm và nhãn, chạy trên cây semantics hiện tại.
Future<void> expectMeetsTapGuidelines(WidgetTester tester) async {
  await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
  await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
  await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
}

Future<void> pumpCentered(WidgetTester tester, Widget child) {
  // Đặt giữa màn hình: guideline bỏ qua nút chạm sát mép cửa sổ.
  return pumpThemed(tester, Scaffold(body: Center(child: child)));
}

void main() {
  group('app thật đạt guideline ở mọi đích', () {
    for (final width in [390.0, 700.0, 1000.0]) {
      for (final destination in ['Báo giá', 'Lịch sử', 'Cài đặt']) {
        testWidgets('${width.round()} px, màn $destination', (tester) async {
          setScreen(tester, width, 800);
          await pumpApp(tester);
          await tester.tap(find.text(destination).last);
          await tester.pumpAndSettle();

          await expectMeetsTapGuidelines(tester);
        });
      }
    }

    testWidgets('chữ gấp đôi vẫn đạt guideline vùng chạm', (tester) async {
      setScreen(tester, 390, 800);
      setTextScale(tester, 2);
      await pumpApp(tester);

      await expectMeetsTapGuidelines(tester);
    });

    testWidgets('trang chi tiết (có nút Huỷ chuyến) đạt guideline', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);
      await tester.tap(find.text('Lịch sử').last);
      await tester.pumpAndSettle();
      await tester.tap(find.textContaining('T-1045'));
      await tester.pumpAndSettle();
      expect(find.text('Huỷ chuyến'), findsOneWidget);

      await expectMeetsTapGuidelines(tester);
    });
  });

  group('node sát mép được đo riêng', () {
    // Guideline bỏ qua node chạm mép cửa sổ, và thanh điều hướng nằm sát mép
    // dưới hoặc trái nên không được đo ở nhóm trên. Ta đo kích thước trực tiếp.
    for (final width in [390.0, 700.0]) {
      testWidgets('${width.round()} px: ba đích điều hướng đủ 48 x 48', (
        tester,
      ) async {
        setScreen(tester, width, 800);
        await pumpApp(tester);

        final tabs = find.bySemanticsLabel(RegExp('Tab [123] trong tổng số 3'));
        expect(tabs, findsNWidgets(3));
        for (final tab in tabs.evaluate()) {
          final size = tester
              .getSemantics(find.byElementPredicate((e) => e == tab))
              .rect
              .size;
          expect(size.width, greaterThanOrEqualTo(48));
          expect(size.height, greaterThanOrEqualTo(48));
        }
      });
    }
  });

  group('contrast', () {
    for (final dark in [false, true]) {
      testWidgets('textContrastGuideline, giao diện ${dark ? 'tối' : 'sáng'}', (
        tester,
      ) async {
        setScreen(tester, 390, 800);
        tester.platformDispatcher.platformBrightnessTestValue = dark
            ? Brightness.dark
            : Brightness.light;
        addTearDown(tester.platformDispatcher.clearPlatformBrightnessTestValue);
        await pumpApp(tester);
        await tester.tap(find.text('Lịch sử').last);
        await tester.pumpAndSettle();

        await expectLater(tester, meetsGuideline(textContrastGuideline));
      });
    }
  });

  group('ca sai có chủ đích: guideline phải bắt được', () {
    testWidgets('chữ xám nhạt trên nền trắng: trượt textContrastGuideline', (
      tester,
    ) async {
      await pumpCentered(tester, const LowContrastNote());

      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
    });

    testWidgets('GestureDetector + Icon 20: bé và không nhãn', (tester) async {
      await pumpCentered(tester, UnlabeledTinyIcon(onTap: () {}));

      await expectLater(
        tester,
        doesNotMeetGuideline(androidTapTargetGuideline),
      );
      await expectLater(tester, doesNotMeetGuideline(iOSTapTargetGuideline));
      await expectLater(
        tester,
        doesNotMeetGuideline(labeledTapTargetGuideline),
      );
    });

    testWidgets('vuông 44: qua chuẩn iOS, trượt chuẩn Android', (tester) async {
      await pumpCentered(tester, Square44Button(onTap: () {}));

      await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
      await expectLater(
        tester,
        doesNotMeetGuideline(androidTapTargetGuideline),
      );
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
    });

    testWidgets('IconButton quên tooltip: đủ lớn nhưng không nhãn', (
      tester,
    ) async {
      await pumpCentered(tester, IconButtonWithoutTooltip(onTap: () {}));

      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
      await expectLater(
        tester,
        doesNotMeetGuideline(labeledTapTargetGuideline),
      );
    });

    testWidgets('IconButton có tooltip: đạt cả ba', (tester) async {
      await pumpCentered(tester, LabeledCloseButton(onTap: () {}));

      await expectMeetsTapGuidelines(tester);
      // Tooltip của IconButton đi vào nút semantics, và guideline chấp nhận
      // `tooltip` thay cho `label`.
      expect(find.byTooltip('Đóng'), findsOneWidget);
      expect(tester.getSemantics(find.byType(IconButton)).tooltip, 'Đóng');
    });

    testWidgets('nút sát mép cửa sổ bị guideline bỏ qua', (tester) async {
      // Cạnh nguy hiểm của guideline: nút bé nhưng chạm mép nên không bị đo.
      await pumpThemed(
        tester,
        Scaffold(
          body: Align(
            alignment: Alignment.topLeft,
            child: UnlabeledTinyIcon(onTap: () {}),
          ),
        ),
      );
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
    });
  });
}

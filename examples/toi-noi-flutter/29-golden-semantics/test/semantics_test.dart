import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_29/broken_cards.dart';
import 'package:toi_noi_29/sample_trips.dart';
import 'package:toi_noi_29/trip_card.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support/golden_harness.dart';
import 'support/sample_cards.dart';

/// Dựng [child] giữa màn hình: guideline bỏ qua node sát mép cửa sổ nên ta
/// không đặt widget cần đo ở mép.
Future<void> pumpCentered(
  WidgetTester tester,
  Widget child, {
  Brightness brightness = Brightness.light,
}) {
  setGoldenView(tester);
  return tester.pumpWidget(
    MaterialApp(
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      themeMode: brightness == Brightness.dark
          ? ThemeMode.dark
          : ThemeMode.light,
      home: Scaffold(
        body: Center(
          child: Padding(padding: const EdgeInsets.all(16), child: child),
        ),
      ),
    ),
  );
}

/// Contrast của mọi `Text` đang hiển thị, đo trên ảnh chụp thật.
///
/// `textContrastGuideline` chỉ đo node semantics mà nhãn bằng đúng chữ của
/// một `Text`; thẻ chuyến gộp cả thẻ thành một câu nên guideline đó không
/// còn đo gì trong thẻ. Bản "custom" nhận finder và đo thẳng từng `Text`.
AccessibilityGuideline allTextContrast() => CustomMinimumContrastGuideline(
  finder: find.byType(Text),
  description: 'mọi Text đạt tỉ lệ 4,5:1',
);

void main() {
  group('cây semantics của TripCard', () {
    testWidgets('thân thẻ là một nút đọc trọn câu, với số cước đúng công thức', (
      tester,
    ) async {
      await pumpCentered(tester, oneCard());

      // 5 km: 12.000 + 3 km vượt x 4.500 = 25.500 VND, do `calculateFare` tính.
      const spoken =
          'Đặt xe, mã T-1045, từ Chung cư Hoa Sen, 12 Đường Mẫu đến Ga Trung '
          'Tâm, cổng B, 5 km, trạng thái Đang tìm tài xế, 25.500 đồng';
      expect(calculateFare(5).vnd, 25500);
      expect(
        tester.getSemantics(find.bySemanticsLabel(spoken)),
        isSemantics(label: spoken, isButton: true, hasTapAction: true),
      );
      // Chữ, chip và icon bên trong không được đọc lần hai.
      expect(find.bySemanticsLabel(RegExp('^Đang tìm tài xế')), findsNothing);
    });

    testWidgets('nút ẩn chuyến là nút riêng, có tooltip làm nhãn', (
      tester,
    ) async {
      await pumpCentered(tester, oneCard());

      final dismiss = tester.getSemantics(find.byType(IconButton));
      expect(dismiss.tooltip, TripCard.dismissLabel);
      expect(
        dismiss,
        isSemantics(isButton: true, hasTapAction: true, isEnabled: true),
      );
    });

    testWidgets('bọc cả thẻ trong ExcludeSemantics làm mất nút ẩn chuyến', (
      tester,
    ) async {
      await pumpCentered(tester, ExcludeSemantics(child: oneCard()));

      // Widget vẫn có trên màn hình, nhưng trình đọc không thấy.
      expect(find.byType(IconButton), findsOneWidget);
      expect(find.bySemanticsLabel(TripCard.dismissLabel), findsNothing);
      expect(find.bySemanticsLabel(RegExp('^Đặt xe, mã T-1045')), findsNothing);
    });

    for (final status in TripStatus.values) {
      testWidgets('nhãn đọc có trạng thái "${status.label}"', (tester) async {
        await pumpCentered(tester, oneCard(status: status));

        expect(
          find.bySemanticsLabel(RegExp('trạng thái ${status.label}, ')),
          findsOneWidget,
        );
      });
    }
  });

  group('TripCard đạt guideline', () {
    for (final brightness in Brightness.values) {
      testWidgets('vùng chạm và nhãn, giao diện ${brightness.name}', (
        tester,
      ) async {
        await pumpCentered(tester, oneCard(), brightness: brightness);

        await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
        await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
        await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      });

      testWidgets(
        'contrast của mọi chữ, tám trạng thái, giao diện ${brightness.name}',
        (tester) async {
          for (final status in TripStatus.values) {
            await pumpCentered(
              tester,
              oneCard(status: status),
              brightness: brightness,
            );

            await expectLater(tester, meetsGuideline(allTextContrast()));
          }
        },
      );

      testWidgets('contrast của chip, giao diện ${brightness.name}', (
        tester,
      ) async {
        await pumpCentered(tester, allStatusChips(), brightness: brightness);

        await expectLater(tester, meetsGuideline(allTextContrast()));
      });
    }
  });

  group('ca sai có chủ đích: guideline phải bắt được', () {
    testWidgets('GestureDetector + icon 20: vùng chạm bé và không nhãn', (
      tester,
    ) async {
      await pumpCentered(tester, TinyDismissIcon(onTap: () {}));

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

    testWidgets('IconButton quên tooltip: đủ lớn nhưng không nhãn', (
      tester,
    ) async {
      await pumpCentered(tester, SilentDismissButton(onTap: () {}));

      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(
        tester,
        doesNotMeetGuideline(labeledTapTargetGuideline),
      );
    });

    testWidgets('nhãn nhạt trên nền nhạt trượt textContrastGuideline', (
      tester,
    ) async {
      await pumpCentered(
        tester,
        const FaintStatusBadge(status: TripStatus.searching),
      );

      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
      await expectLater(tester, doesNotMeetGuideline(allTextContrast()));
    });

    testWidgets('nhãn nhạt chỉ trượt ở giao diện sáng, giao diện tối thì qua', (
      tester,
    ) async {
      for (final status in TripStatus.values) {
        await pumpCentered(tester, FaintStatusBadge(status: status));
        await tester.pumpAndSettle();
        await expectLater(tester, doesNotMeetGuideline(allTextContrast()));

        await pumpCentered(
          tester,
          FaintStatusBadge(status: status),
          brightness: Brightness.dark,
        );
        // Đổi themeMode chạy hoạt ảnh theme; đo khi chưa xong sẽ ra màu lai.
        await tester.pumpAndSettle();
        await expectLater(tester, meetsGuideline(allTextContrast()));
      }
    });

    testWidgets(
      'gộp semantics làm textContrastGuideline mù, bản custom vẫn thấy',
      (tester) async {
        await pumpCentered(
          tester,
          Semantics(
            label: 'Trạng thái: Đang tìm tài xế',
            excludeSemantics: true,
            child: const FaintStatusBadge(status: TripStatus.searching),
          ),
        );

        // Nhãn semantics không trùng chữ của Text nào nên guideline mặc định
        // không tìm thấy gì để đo và báo qua.
        await expectLater(tester, meetsGuideline(textContrastGuideline));
        await expectLater(tester, doesNotMeetGuideline(allTextContrast()));
      },
    );
  });

  test(
    'chuyến mẫu dựng bằng Trip.request và apply, không gán trạng thái tay',
    () {
      final trip = tripAt(TripStatus.accepted, id: 'T-1');
      expect(trip.status, TripStatus.accepted);
      expect(trip.driverId, 'D-07');
    },
  );
}

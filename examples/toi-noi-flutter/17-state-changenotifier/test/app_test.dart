import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/history_screen.dart';
import 'package:toi_noi_17/quote_scope.dart';
import 'package:toi_noi_17/quote_screen.dart';
import 'package:toi_noi_17/trip_book.dart';
import 'package:toi_noi_17/trip_detail_page.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

bool _isSelected(WidgetTester tester, String label) =>
    tester.widget<ChoiceChip>(find.widgetWithText(ChoiceChip, label)).selected;

Future<void> _openHistory(WidgetTester tester) async {
  await tester.tap(find.text('Lịch sử').last);
  await tester.pumpAndSettle();
}

/// Chạy [action] rồi trả tên kiểu của mọi widget bị dựng lại trong khung hình
/// kế tiếp, lấy từ `debugOnRebuildDirtyWidget` (chỉ gọi trong chế độ debug).
Future<Set<String>> _rebuiltBy(WidgetTester tester, VoidCallback action) async {
  final rebuilt = <String>{};
  debugOnRebuildDirtyWidget = (element, _) =>
      rebuilt.add(element.widget.runtimeType.toString());
  try {
    action();
    await tester.pump();
  } finally {
    debugOnRebuildDirtyWidget = null;
  }
  return rebuilt;
}

void main() {
  group('app dùng QuoteScope và TripBookScope, không callback qua tầng', () {
    testWidgets('chọn dịch vụ và kéo km: chip và cước cùng đổi', (
      tester,
    ) async {
      final semantics = tester.ensureSemantics();
      try {
        await pumpApp(tester);
        expect(_isSelected(tester, 'Đặt xe'), isTrue);
        expect(find.text('25.500 VND'), findsOneWidget);

        await tester.tap(find.text('Giao đồ ăn'));
        await tester.pump();
        expect(_isSelected(tester, 'Giao đồ ăn'), isTrue);
        expect(_isSelected(tester, 'Đặt xe'), isFalse);

        // Thao tác `increase` mà trình đọc màn hình gửi: 5 km sang 6 km.
        tester.semantics.increase(find.semantics.byValue('5 km'));
        await tester.pump();
        expect(find.text('30.000 VND'), findsOneWidget);
        expect(find.text('Quãng đường: 6 km'), findsOneWidget);
      } finally {
        semantics.dispose();
      }
    });

    testWidgets('đặt chuyến: nút đọc draft lúc bấm, lịch sử có chuyến mới', (
      tester,
    ) async {
      await pumpApp(tester);
      await tester.tap(find.text('Giao bưu kiện'));
      await tester.pump();

      await tester.tap(find.text('Đặt chuyến'));
      await tester.pump();
      expect(find.text('Đã gửi yêu cầu T-1046'), findsOneWidget);

      await _openHistory(tester);
      final first = tester.widget<Card>(find.byType(Card).first);
      expect(first.key, const ValueKey('T-1046'));
      expect(
        find.descendant(
          of: find.byKey(const ValueKey('T-1046')),
          matching: find.text('Giao bưu kiện · T-1046'),
        ),
        findsOneWidget,
      );
      expect(find.text('Đang tìm tài xế'), findsWidgets);
    });

    testWidgets(
      'huỷ ở trang chi tiết: trang và danh sách cùng đổi, không onChanged',
      (tester) async {
        await pumpApp(tester);
        await _openHistory(tester);

        await tester.tap(find.text('Đặt xe · T-1045'));
        await tester.pumpAndSettle();
        expect(find.byType(TripDetailPage), findsOneWidget);
        expect(find.text('Huỷ chuyến'), findsOneWidget);

        await tester.tap(find.text('Huỷ chuyến'));
        await tester.pump();

        // Trang chi tiết dựng lại vì nó phụ thuộc TripBookScope.
        expect(find.text('Huỷ chuyến'), findsNothing);
        expect(find.text('Đã huỷ'), findsOneWidget);

        // Quay lại: danh sách phía sau đã thấy trạng thái mới.
        await tester.pageBack();
        await tester.pumpAndSettle();
        expect(find.byType(HistoryScreen), findsOneWidget);
        final book = TripBookScope.read(
          tester.element(find.byType(HistoryScreen)),
        );
        expect(book.byId('T-1045')!.status, TripStatus.cancelled);
        expect(find.text('Đã huỷ'), findsNWidgets(2)); // T-1045 và T-1042
      },
    );

    testWidgets('chuyến không huỷ được thì không có nút huỷ', (tester) async {
      await pumpApp(tester);
      await _openHistory(tester);

      await tester.tap(find.text('Giao bưu kiện · T-1043')); // completed
      await tester.pumpAndSettle();

      expect(find.text('Huỷ chuyến'), findsNothing);
    });

    testWidgets('đổi tab giữ nguyên bản nháp báo giá', (tester) async {
      await pumpApp(tester);
      await tester.tap(find.text('Giao đồ ăn'));
      await tester.pump();

      await _openHistory(tester);
      await tester.tap(find.text('Báo giá').last);
      await tester.pumpAndSettle();

      expect(_isSelected(tester, 'Giao đồ ăn'), isTrue);
      final draft = QuoteScope.read(
        tester.element(find.byType(ChoiceChip).first),
      );
      expect(draft.service, ServiceKind.food);
    });

    testWidgets(
      'kéo km chỉ dựng lại widget phụ thuộc draft, không dựng lại màn hình',
      (tester) async {
        await pumpApp(tester);
        final draft = QuoteScope.read(tester.element(find.byType(QuoteScreen)));

        // 5 km sang 8 km: cước đổi 25.500 sang 39.000.
        final rebuilt = await _rebuiltBy(tester, () => draft.setKm(8));

        expect(rebuilt, containsAll(['_KmSlider', '_ServiceChips']));
        expect(rebuilt, contains('ValueListenableBuilder<Money>'));
        expect(
          rebuilt,
          isNot(
            anyOf(
              contains('ToiNoiApp'),
              contains('HomeShell'),
              contains('QuoteScreen'),
              contains('HistoryScreen'),
              contains('_BookButton'),
            ),
          ),
        );
      },
    );

    testWidgets('đổi dịch vụ không dựng lại nhãn cước', (tester) async {
      await pumpApp(tester);
      final draft = QuoteScope.read(tester.element(find.byType(QuoteScreen)));

      final rebuilt = await _rebuiltBy(
        tester,
        () => draft.selectService(ServiceKind.food),
      );

      expect(rebuilt, contains('_ServiceChips'));
      expect(rebuilt, isNot(contains('ValueListenableBuilder<Money>')));
      expect(rebuilt, isNot(contains('MoneyText')));
    });

    testWidgets('sổ chuyến đổi không đánh thức màn báo giá', (tester) async {
      await pumpApp(tester);
      final book = TripBookScope.read(
        tester.element(find.byType(HistoryScreen, skipOffstage: false)),
      );

      final rebuilt = await _rebuiltBy(
        tester,
        () => book.book(ServiceKind.ride, 5),
      );

      expect(rebuilt, contains('HistoryScreen'));
      expect(
        rebuilt,
        isNot(anyOf(contains('_ServiceChips'), contains('_KmSlider'))),
      );
    });
  });
}

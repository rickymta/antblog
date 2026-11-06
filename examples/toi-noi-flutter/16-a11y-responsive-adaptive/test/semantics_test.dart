import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/accessible.dart';
import 'package:toi_noi_16/sample_trips.dart';
import 'package:toi_noi_16/trip_tile.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  group('câu đọc thuần', () {
    test('spokenMoney thay VND bằng "đồng", giữ dấu chấm nghìn', () {
      expect(spokenMoney(const Money(25500)), '25.500 đồng');
      expect(spokenMoney(Money.zero), '0 đồng');
      expect(spokenMoney(const Money(1234567)), '1.234.567 đồng');
    });

    test('formatKm: số nguyên bỏ phần lẻ, số lẻ dùng dấu phẩy', () {
      expect(formatKm(5), '5 km');
      expect(formatKm(2.5), '2,5 km');
    });

    test('spokenTrip có đủ dịch vụ, quãng đường, trạng thái và cước', () {
      final trip = tripAt(TripStatus.searching, id: 'T-1');
      expect(
        spokenTrip(trip),
        'Đặt xe, 5 km, trạng thái Đang tìm tài xế, 25.500 đồng',
      );
    });

    test('mọi trạng thái đều xuất hiện trong câu đọc', () {
      for (final status in TripStatus.values) {
        final trip = tripAt(status, id: 'T-1');
        expect(spokenTrip(trip), contains(status.label), reason: '$status');
      }
    });
  });

  group('AccessibleMoney và AccessibleStatus', () {
    testWidgets('số tiền: nhãn đọc là 25.500 đồng, chữ VND không bị đọc', (
      tester,
    ) async {
      await pumpThemed(
        tester,
        const Scaffold(
          body: Center(child: AccessibleMoney(Money(25500), prefix: 'Cước')),
        ),
      );

      // Chữ trực quan vẫn vẽ ra màn hình...
      expect(find.text('25.500 VND'), findsOneWidget);
      // ...nhưng cây semantics chỉ có câu đọc, không có chữ viết tắt.
      expect(find.bySemanticsLabel('Cước: 25.500 đồng'), findsOneWidget);
      expect(find.bySemanticsLabel(RegExp('VND')), findsNothing);
    });

    testWidgets('trạng thái: nhãn "Trạng thái: ..." và live region', (
      tester,
    ) async {
      await pumpThemed(
        tester,
        const Scaffold(
          body: Center(
            child: AccessibleStatus(TripStatus.searching, liveRegion: true),
          ),
        ),
      );

      final node = tester.getSemantics(
        find.bySemanticsLabel('Trạng thái: Đang tìm tài xế'),
      );
      expect(
        node,
        matchesSemantics(
          label: 'Trạng thái: Đang tìm tài xế',
          isLiveRegion: true,
        ),
      );
      // Icon của chip chỉ để trang trí: không có nút nào tên "Đang tìm tài xế".
      expect(find.bySemanticsLabel('Đang tìm tài xế'), findsNothing);
    });

    testWidgets('không bật liveRegion thì nút không phải live region', (
      tester,
    ) async {
      await pumpThemed(
        tester,
        const Scaffold(
          body: Center(child: AccessibleStatus(TripStatus.completed)),
        ),
      );
      expect(
        tester.getSemantics(find.bySemanticsLabel('Trạng thái: Hoàn thành')),
        matchesSemantics(label: 'Trạng thái: Hoàn thành'),
      );
    });
  });

  group('TripTile', () {
    testWidgets('cả thẻ là một nút, nhãn đọc trọn câu, chạm qua semantics', (
      tester,
    ) async {
      var taps = 0;
      await pumpThemed(
        tester,
        Scaffold(
          body: Center(
            child: TripTile(
              trip: tripAt(TripStatus.completed, id: 'T-9', km: 12),
              onTap: () => taps++,
            ),
          ),
        ),
      );

      const spoken = 'Đặt xe, 12 km, trạng thái Hoàn thành, 57.000 đồng';
      expect(find.bySemanticsLabel(spoken), findsOneWidget);
      expect(
        tester.getSemantics(find.bySemanticsLabel(spoken)),
        matchesSemantics(
          label: spoken,
          isButton: true,
          hasTapAction: true,
          hasSelectedState: true,
        ),
      );
      // Chữ bên trong bị loại khỏi cây semantics: không đọc hai lần.
      expect(find.bySemanticsLabel('T-9 · 12 km'), findsNothing);
      expect(find.bySemanticsLabel('Hoàn thành'), findsNothing);

      tester.semantics.tap(find.semantics.byLabel(spoken));
      expect(taps, 1);
    });
  });

  group('app: nhãn trên màn hình thật', () {
    testWidgets('thanh điều hướng đọc tiếng Việt nhờ MaterialLocalizations', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);

      expect(
        find.bySemanticsLabel(
          RegExp('Báo giá.*Tab 1 trong tổng số 3', dotAll: true),
        ),
        findsOneWidget,
      );
      expect(find.bySemanticsLabel(RegExp('Tab 1 of 3')), findsNothing);
    });

    testWidgets('slider quãng đường: nhãn, giá trị và thao tác tăng/giảm', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);

      final slider = find.bySemanticsLabel('Quãng đường');
      expect(
        tester.getSemantics(slider),
        matchesSemantics(
          label: 'Quãng đường',
          value: '5 km',
          increasedValue: '6 km',
          decreasedValue: '4 km',
          isSlider: true,
          hasEnabledState: true,
          isEnabled: true,
          isFocusable: true,
          hasIncreaseAction: true,
          hasDecreaseAction: true,
          hasFocusAction: true,
        ),
      );
      expect(find.bySemanticsLabel(RegExp('25.500 đồng')), findsOneWidget);

      // Thao tác `increase` mà trình đọc màn hình gửi tới: cước đổi theo.
      tester.semantics.increase(find.semantics.byLabel('Quãng đường'));
      await tester.pump();

      expect(
        tester.getSemantics(slider),
        matchesSemantics(
          label: 'Quãng đường',
          value: '6 km',
          increasedValue: '7 km',
          decreasedValue: '5 km',
          isSlider: true,
          hasEnabledState: true,
          isEnabled: true,
          isFocusable: true,
          hasIncreaseAction: true,
          hasDecreaseAction: true,
          hasFocusAction: true,
        ),
      );
      expect(find.text(calculateFare(6).toString()), findsOneWidget);
      expect(find.bySemanticsLabel(RegExp('30.000 đồng')), findsOneWidget);
    });

    testWidgets('ô cước dự kiến chỉ đọc một lần, không lặp tiền tố', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);

      final node = tester.getSemantics(
        find.bySemanticsLabel(RegExp('25.500 đồng')),
      );
      expect(node.label, 'Cước dự kiến\n25.500 đồng');
    });

    testWidgets('huỷ chuyến: trạng thái mới được đọc qua live region', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      await pumpApp(tester);
      await tester.tap(find.text('Lịch sử').last);
      await tester.pumpAndSettle();
      await tester.tap(find.textContaining('T-1045'));
      await tester.pumpAndSettle();
      expect(
        tester.getSemantics(
          find.bySemanticsLabel('Trạng thái: Đang tìm tài xế'),
        ),
        matchesSemantics(
          label: 'Trạng thái: Đang tìm tài xế',
          isLiveRegion: true,
        ),
      );

      await tester.tap(find.widgetWithText(OutlinedButton, 'Huỷ chuyến'));
      await tester.pumpAndSettle();
      await tester.tap(find.widgetWithText(TextButton, 'Huỷ chuyến'));
      await tester.pumpAndSettle();

      expect(
        tester.getSemantics(find.bySemanticsLabel('Trạng thái: Đã huỷ')),
        matchesSemantics(label: 'Trạng thái: Đã huỷ', isLiveRegion: true),
      );
      expect(
        find.bySemanticsLabel('Trạng thái: Đang tìm tài xế'),
        findsNothing,
      );
    });
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/broken_examples.dart';
import 'package:toi_noi_16/quote_screen.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

// Số đo minh hoạ cho điện thoại có tai thỏ: 47 ở trên (status bar + notch),
// 34 ở dưới (home indicator), 44 hai bên khi xoay ngang.
const _top = 47.0;
const _bottom = 34.0;
const _side = 44.0;

void main() {
  group('MediaQuery nhận vùng không an toàn từ view', () {
    testWidgets('view.padding đi tới MediaQuery.paddingOf', (tester) async {
      setScreen(tester, 390, 800);
      setInsets(tester, top: _top, bottom: _bottom);
      late EdgeInsets padding;
      await pumpThemed(
        tester,
        Builder(
          builder: (context) {
            padding = MediaQuery.paddingOf(context);
            return const SizedBox();
          },
        ),
      );
      expect(padding.top, _top);
      expect(padding.bottom, _bottom);
      expect(padding.left, 0);
    });
  });

  group('ca sai: nút đáy không có SafeArea', () {
    testWidgets('không có tai thỏ: hai bản giống nhau', (tester) async {
      setScreen(tester, 390, 800);
      await pumpThemed(tester, const UnsafeFooter());
      final unsafe = tester.getRect(find.byType(FilledButton)).bottom;
      await pumpThemed(tester, const SafeFooter());
      final safe = tester.getRect(find.byType(FilledButton)).bottom;
      expect(unsafe, safe);
    });

    testWidgets('có home indicator 34: bản sai chui vào vùng đó', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      setInsets(tester, bottom: _bottom);
      await pumpThemed(tester, const UnsafeFooter());

      final bottom = tester.getRect(find.byType(FilledButton)).bottom;
      expect(bottom, 784);
      expect(bottom, greaterThan(800 - _bottom));
    });

    testWidgets('có home indicator 34: SafeArea đẩy nút lên trên vùng đó', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      setInsets(tester, bottom: _bottom);
      await pumpThemed(tester, const SafeFooter());

      final bottom = tester.getRect(find.byType(FilledButton)).bottom;
      expect(bottom, 750);
      expect(bottom, lessThanOrEqualTo(800 - _bottom));
    });
  });

  group('app: điện thoại dọc có tai thỏ', () {
    testWidgets('AppBar chừa 47 ở trên, nội dung bắt đầu sau đó', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      setInsets(tester, top: _top, bottom: _bottom);
      await pumpApp(tester);

      expect(tester.getSize(find.byType(AppBar)).height, kToolbarHeight + _top);
      final title = find.descendant(
        of: find.byType(AppBar),
        matching: find.text('Báo giá'),
      );
      expect(tester.getRect(title).top, greaterThanOrEqualTo(_top));
      // Scaffold đã trừ phần trên đi khi có AppBar: SafeArea của nội dung
      // không chừa thêm lần nữa.
      expect(
        tester.getTopLeft(find.byType(IndexedStack)).dy,
        kToolbarHeight + _top,
      );
    });

    testWidgets('NavigationBar đẩy nhãn và icon lên trên home indicator', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      setInsets(tester, top: _top, bottom: _bottom);
      await pumpApp(tester);

      expect(
        tester.getRect(find.byType(NavigationBar)).bottom,
        800,
        reason: 'nền thanh điều hướng vẫn phủ tới sát đáy',
      );
      final label = tester.getRect(find.text('Cài đặt').last);
      expect(label.bottom, lessThanOrEqualTo(800 - _bottom));
    });
  });

  group('app: bố cục thanh bên, nút Đặt chuyến', () {
    testWidgets('home indicator 34 (không có NavigationBar): nút nằm trên nó', (
      tester,
    ) async {
      setScreen(tester, 700, 900);
      setInsets(tester, top: _top, bottom: _bottom);
      await pumpApp(tester);

      final button = tester.getRect(
        find.widgetWithText(FilledButton, 'Đặt chuyến'),
      );
      // Đúng bằng: 34 của home indicator cộng 16 đệm của nút. SafeArea lồng
      // nhau (khung app và màn báo giá) không chừa hai lần.
      expect(button.bottom, 900 - _bottom - ToiNoiSpacing.md);
    });

    testWidgets('QuoteScreen đứng riêng cũng tự chừa home indicator', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      setInsets(tester, bottom: _bottom);
      await pumpThemed(tester, Scaffold(body: QuoteScreen(onBook: (_, _) {})));

      final button = tester.getRect(
        find.widgetWithText(FilledButton, 'Đặt chuyến'),
      );
      expect(button.bottom, 800 - _bottom - ToiNoiSpacing.md);
    });
  });

  group('app: xoay ngang, tai thỏ ở hai bên', () {
    testWidgets('thanh bên và vùng nội dung tránh hai mép', (tester) async {
      setScreen(tester, 1000, 450);
      setInsets(tester, left: _side, right: _side, bottom: 21);
      await pumpApp(tester);

      // Nền NavigationRail vẫn phủ tới mép, còn icon của nó chừa bên trái.
      expect(tester.getRect(find.byType(NavigationRail)).left, 0);
      expect(
        tester.getRect(find.byIcon(Icons.request_quote)).left,
        greaterThanOrEqualTo(_side),
      );
      // Vùng nội dung chừa bên phải.
      expect(tester.getRect(find.byType(IndexedStack)).right, 1000 - _side);
    });
  });
}

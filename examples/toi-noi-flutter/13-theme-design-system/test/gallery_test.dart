import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_13/app.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

void main() {
  testWidgets('hiện đủ 8 chip trạng thái chuyến', (tester) async {
    setPlatformBrightness(tester, Brightness.light);
    await tester.pumpWidget(const ThemeGalleryApp());
    expect(
      find.byType(TripStatusChip),
      findsNWidgets(TripStatus.values.length),
    );
    for (final status in TripStatus.values) {
      expect(find.text(status.label), findsOneWidget, reason: status.name);
    }
  });

  testWidgets('báo giá hiện "5 km", không phải "5.0 km"', (tester) async {
    setPlatformBrightness(tester, Brightness.light);
    await tester.pumpWidget(const ThemeGalleryApp());
    expect(find.text('Xe máy · 5 km'), findsOneWidget);
  });

  testWidgets('báo giá 5 km là 25.500 VND lấy từ toi_noi_core', (tester) async {
    setPlatformBrightness(tester, Brightness.light);
    await tester.pumpWidget(const ThemeGalleryApp());
    expect(calculateFare(5), const Money(25500));
    expect(find.text('25.500 VND'), findsOneWidget);
  });

  testWidgets('nút và ô nhập dùng component theme của toi_noi_ui', (
    tester,
  ) async {
    setPlatformBrightness(tester, Brightness.light);
    await tester.pumpWidget(const ThemeGalleryApp());
    // Phần nhìn thấy của nút cao 48; ThemeData mặc định chỉ cao 40.
    final visible = find.descendant(
      of: find.byType(FilledButton),
      matching: find.byType(Material),
    );
    expect(tester.getSize(visible.first).height, 48);
    final theme = Theme.of(tester.element(find.byType(Scaffold)));
    final decorator = tester.widget<InputDecorator>(
      find.byType(InputDecorator),
    );
    expect(
      decorator.decoration.applyDefaults(theme.inputDecorationTheme).filled,
      isTrue,
    );
  });

  for (final brightness in Brightness.values) {
    testWidgets('cả bảng mẫu đạt textContrastGuideline (${brightness.name})', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      setPlatformBrightness(tester, brightness);
      // Đủ cao để ListView dựng cả bảng mẫu (không chỉ phần đầu).
      tester.view.physicalSize = const Size(800, 3000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await tester.pumpWidget(const ThemeGalleryApp());

      // Guideline chỉ kiểm chữ nhìn thấy và bấm được: xác nhận các khối
      // quan trọng đều nằm trong số đó.
      for (final status in TripStatus.values) {
        expect(find.text(status.label).hitTestable(), findsOneWidget);
      }
      for (final name in ['primary', 'warning', 'danger', 'success']) {
        expect(find.text(name).hitTestable(), findsOneWidget, reason: name);
      }
      expect(find.text('25.500 VND').hitTestable(), findsOneWidget);
      expect(find.text('Xe máy · 5 km').hitTestable(), findsOneWidget);

      await expectLater(tester, meetsGuideline(textContrastGuideline));
      handle.dispose();
    });
  }
}

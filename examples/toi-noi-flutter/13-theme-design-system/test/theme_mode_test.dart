import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_13/app.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

Future<void> _choose(WidgetTester tester, String label) async {
  await tester.tap(find.text(label));
  // AnimatedTheme chuyển theme trong một khoảng thời gian.
  await tester.pumpAndSettle();
}

void main() {
  group('ThemeMode.system theo platformBrightness', () {
    testWidgets('hệ thống sáng thì app sáng', (tester) async {
      setPlatformBrightness(tester, Brightness.light);
      await tester.pumpWidget(const ThemeGalleryApp());
      expect(themeBrightness(tester), Brightness.light);
    });

    testWidgets('hệ thống tối thì app tối', (tester) async {
      setPlatformBrightness(tester, Brightness.dark);
      await tester.pumpWidget(const ThemeGalleryApp());
      expect(themeBrightness(tester), Brightness.dark);
    });

    testWidgets('đổi cài đặt hệ thống khi app đang chạy thì app đổi theo', (
      tester,
    ) async {
      setPlatformBrightness(tester, Brightness.light);
      await tester.pumpWidget(const ThemeGalleryApp());
      expect(themeBrightness(tester), Brightness.light);

      tester.platformDispatcher.platformBrightnessTestValue = Brightness.dark;
      await tester.pumpAndSettle();
      expect(themeBrightness(tester), Brightness.dark);
    });

    testWidgets('màu thật đổi: nền Scaffold và bộ màu trạng thái chuyến', (
      tester,
    ) async {
      setPlatformBrightness(tester, Brightness.light);
      await tester.pumpWidget(const ThemeGalleryApp());
      final lightTheme = Theme.of(tester.element(find.byType(Scaffold)));
      expect(
        lightTheme.colorScheme.surface,
        ToiNoiTheme.light().colorScheme.surface,
      );
      expect(lightTheme.extension<ToiNoiColors>(), ToiNoiColors.light);

      tester.platformDispatcher.platformBrightnessTestValue = Brightness.dark;
      await tester.pumpAndSettle();
      final darkTheme = Theme.of(tester.element(find.byType(Scaffold)));
      expect(
        darkTheme.colorScheme.surface,
        ToiNoiTheme.dark().colorScheme.surface,
      );
      expect(darkTheme.extension<ToiNoiColors>(), ToiNoiColors.dark);
      expect(
        darkTheme.colorScheme.surface,
        isNot(lightTheme.colorScheme.surface),
      );
    });
  });

  testWidgets('quên darkTheme: hệ thống tối vẫn ra giao diện sáng', (
    tester,
  ) async {
    setPlatformBrightness(tester, Brightness.dark);
    await tester.pumpWidget(
      MaterialApp(
        theme: ToiNoiTheme.light(),
        // Không có darkTheme, ThemeMode.system theo hệ thống (tối).
        home: const Scaffold(),
      ),
    );
    expect(themeBrightness(tester), Brightness.light);
  });

  group('ghi đè bằng bộ chọn', () {
    testWidgets('chọn Tối khi hệ thống sáng, rồi quay lại Hệ thống', (
      tester,
    ) async {
      setPlatformBrightness(tester, Brightness.light);
      await tester.pumpWidget(const ThemeGalleryApp());

      await _choose(tester, 'Tối');
      expect(themeBrightness(tester), Brightness.dark);

      await _choose(tester, 'Hệ thống');
      expect(themeBrightness(tester), Brightness.light);
    });

    testWidgets('chọn Sáng khi hệ thống tối: hệ thống đổi cũng không làm app '
        'đổi', (tester) async {
      setPlatformBrightness(tester, Brightness.dark);
      await tester.pumpWidget(const ThemeGalleryApp());
      expect(themeBrightness(tester), Brightness.dark);

      await _choose(tester, 'Sáng');
      expect(themeBrightness(tester), Brightness.light);

      tester.platformDispatcher.platformBrightnessTestValue = Brightness.light;
      await tester.pumpAndSettle();
      tester.platformDispatcher.platformBrightnessTestValue = Brightness.dark;
      await tester.pumpAndSettle();
      expect(themeBrightness(tester), Brightness.light);
    });

    testWidgets('initialThemeMode: dark bỏ qua hệ thống ngay từ đầu', (
      tester,
    ) async {
      setPlatformBrightness(tester, Brightness.light);
      await tester.pumpWidget(
        const ThemeGalleryApp(initialThemeMode: ThemeMode.dark),
      );
      expect(themeBrightness(tester), Brightness.dark);
    });

    testWidgets('segment đang chọn khớp với ThemeMode hiện tại', (
      tester,
    ) async {
      setPlatformBrightness(tester, Brightness.light);
      await tester.pumpWidget(const ThemeGalleryApp());
      SegmentedButton<ThemeMode> button() =>
          tester.widget(find.byType(SegmentedButton<ThemeMode>));
      expect(button().selected, {ThemeMode.system});

      await _choose(tester, 'Tối');
      expect(button().selected, {ThemeMode.dark});
    });
  });
}

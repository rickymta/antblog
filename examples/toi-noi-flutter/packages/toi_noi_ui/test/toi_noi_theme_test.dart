import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'test_helpers.dart';

void main() {
  final light = ToiNoiTheme.light();
  final dark = ToiNoiTheme.dark();

  group('ColorScheme', () {
    test('light và dark có brightness đúng', () {
      expect(light.brightness, Brightness.light);
      expect(dark.brightness, Brightness.dark);
      expect(light.colorScheme.brightness, Brightness.light);
      expect(dark.colorScheme.brightness, Brightness.dark);
    });

    test('hai bản sinh từ cùng seed nhưng chọn màu khác nhau', () {
      expect(light.colorScheme.primary, isNot(dark.colorScheme.primary));
      expect(light.colorScheme.surface, isNot(dark.colorScheme.surface));
      // Dark mode dùng màu chính sáng hơn và nền tối hơn.
      expect(
        dark.colorScheme.primary.computeLuminance(),
        greaterThan(light.colorScheme.primary.computeLuminance()),
      );
      expect(
        dark.colorScheme.surface.computeLuminance(),
        lessThan(light.colorScheme.surface.computeLuminance()),
      );
    });

    test('primary suy ra từ seed chứ không bằng seed', () {
      expect(light.colorScheme.primary, isNot(ToiNoiTheme.seed));
    });

    test('mỗi cặp vai trò "on" đạt tương phản AA ở cả hai theme', () {
      for (final theme in [light, dark]) {
        final s = theme.colorScheme;
        final pairs = [
          (s.onPrimary, s.primary),
          (s.onPrimaryContainer, s.primaryContainer),
          (s.onSurface, s.surface),
          (s.onSurfaceVariant, s.surfaceContainerHighest),
          (s.onError, s.error),
          // Lỗi trong form: chữ và viền `error` đặt trên `surface`.
          (s.error, s.surface),
        ];
        for (final (fg, bg) in pairs) {
          expect(contrastRatio(fg, bg), greaterThanOrEqualTo(4.5));
        }
      }
    });
  });

  group('TextTheme', () {
    test('giữ màu chữ từ ColorScheme, chỉ đổi phần ta khai báo', () {
      for (final theme in [light, dark]) {
        final title = theme.textTheme.titleLarge!;
        expect(title.fontWeight, FontWeight.w600);
        expect(title.color, theme.colorScheme.onSurface);
        expect(theme.textTheme.bodyMedium!.height, 1.4);
        expect(theme.textTheme.bodyMedium!.color, theme.colorScheme.onSurface);
      }
    });

    test('đọc textTheme thẳng từ ThemeData (ngoài cây) chưa có cỡ chữ', () {
      expect(light.textTheme.bodyLarge!.fontSize, isNull);
    });

    testWidgets('vai trò không khai báo vẫn có kiểu mặc định của Material', (
      tester,
    ) async {
      late TextTheme inTree;
      await pumpThemed(
        tester,
        Builder(
          builder: (context) {
            inTree = Theme.of(context).textTheme;
            return const SizedBox();
          },
        ),
        theme: light,
      );
      // Cỡ chữ chỉ có sau khi theme được "localize" trong cây widget.
      expect(inTree.bodyLarge!.fontSize, isNotNull);
      expect(inTree.bodyLarge!.fontWeight, isNot(FontWeight.w600));
      expect(inTree.titleLarge!.fontWeight, FontWeight.w600);
    });
  });

  test('cùng nền tảng thì light() và dark() trả về cùng đối tượng', () {
    expect(identical(ToiNoiTheme.light(), ToiNoiTheme.light()), isTrue);
    expect(identical(ToiNoiTheme.dark(), ToiNoiTheme.dark()), isTrue);
    expect(identical(ToiNoiTheme.light(), ToiNoiTheme.dark()), isFalse);
  });

  test(
    'đổi debugDefaultTargetPlatformOverride thì Theme.platform đổi theo',
    () {
      addTearDown(() => debugDefaultTargetPlatformOverride = null);
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      final ios = ToiNoiTheme.light();
      expect(ios.platform, TargetPlatform.iOS);
      expect(identical(ToiNoiTheme.light(), ios), isTrue);

      debugDefaultTargetPlatformOverride = TargetPlatform.android;
      final android = ToiNoiTheme.light();
      expect(android.platform, TargetPlatform.android);
      expect(identical(android, ios), isFalse);

      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      expect(identical(ToiNoiTheme.light(), ios), isTrue);
      expect(ToiNoiTheme.dark().platform, TargetPlatform.iOS);
    },
  );

  test('bo góc dùng hằng ToiNoiRadius', () {
    expect(ToiNoiRadius.button, 12);
    expect(ToiNoiRadius.input, 12);
    expect(ToiNoiRadius.card, 16);
    final card = light.cardTheme.shape! as RoundedRectangleBorder;
    expect(card.borderRadius, BorderRadius.circular(ToiNoiRadius.card));
    final button =
        light.filledButtonTheme.style!.shape!.resolve({})!
            as RoundedRectangleBorder;
    expect(button.borderRadius, BorderRadius.circular(ToiNoiRadius.button));
    final input = light.inputDecorationTheme.border! as OutlineInputBorder;
    expect(input.borderRadius, BorderRadius.circular(ToiNoiRadius.input));
  });

  group('extensions', () {
    test('mỗi theme mang đúng bộ ToiNoiColors của nó', () {
      expect(light.extension<ToiNoiColors>(), ToiNoiColors.light);
      expect(dark.extension<ToiNoiColors>(), ToiNoiColors.dark);
    });

    test('danger đúng bằng bộ error của ColorScheme, không có hai màu đỏ', () {
      for (final theme in [light, dark]) {
        final c = theme.extension<ToiNoiColors>()!;
        final s = theme.colorScheme;
        expect(c.danger, s.error);
        expect(c.onDanger, s.onError);
        expect(c.dangerContainer, s.errorContainer);
        expect(c.onDangerContainer, s.onErrorContainer);
      }
    });
  });

  group('component theme', () {
    for (final entry in themes.entries) {
      final name = entry.key;
      final theme = entry.value;

      testWidgets('theme $name: FilledButton cao 48', (tester) async {
        await pumpThemed(
          tester,
          FilledButton(onPressed: () {}, child: const Text('Đặt xe')),
          theme: theme,
        );
        expect(filledButtonVisibleHeight(tester), 48);
      });

      testWidgets('theme $name: Card phẳng, có viền outlineVariant', (
        tester,
      ) async {
        await pumpThemed(
          tester,
          const Card(child: SizedBox(width: 100, height: 60)),
          theme: theme,
        );
        final card = tester.widget<Card>(find.byType(Card));
        expect(card.elevation, isNull, reason: 'Card không tự đặt elevation');
        final material = tester.widget<Material>(
          find.descendant(
            of: find.byType(Card),
            matching: find.byType(Material),
          ),
        );
        expect(material.elevation, 0);
        final shape = material.shape! as RoundedRectangleBorder;
        expect(shape.side.color, theme.colorScheme.outlineVariant);
        expect(material.color, theme.colorScheme.surfaceContainerLow);
      });

      testWidgets('theme $name: TextField tô nền và bo góc theo theme', (
        tester,
      ) async {
        await pumpThemed(
          tester,
          const SizedBox(
            width: 300,
            child: TextField(
              decoration: InputDecoration(labelText: 'Điểm đến'),
            ),
          ),
          theme: theme,
        );
        final decorator = tester.widget<InputDecorator>(
          find.byType(InputDecorator),
        );
        final effective = decorator.decoration.applyDefaults(
          theme.inputDecorationTheme,
        );
        expect(effective.filled, isTrue);
        expect(effective.fillColor, theme.colorScheme.surfaceContainerHighest);
        expect(effective.border, isA<OutlineInputBorder>());
      });
    }

    testWidgets('style đặt trên widget thắng component theme', (tester) async {
      await pumpThemed(
        tester,
        FilledButton(
          onPressed: () {},
          style: FilledButton.styleFrom(minimumSize: const Size(64, 56)),
          child: const Text('Đặt xe'),
        ),
        theme: light,
      );
      expect(filledButtonVisibleHeight(tester), 56);
    });

    testWidgets('ThemeData mặc định cho nút thấp hơn (40), nên 48 là do theme '
        'của ta', (tester) async {
      await pumpThemed(
        tester,
        FilledButton(onPressed: () {}, child: const Text('Đặt xe')),
        theme: ThemeData(),
      );
      expect(filledButtonVisibleHeight(tester), 40);
    });

    testWidgets('Card mặc định nổi 1, Card của ta phẳng (0)', (tester) async {
      Future<double> elevationUnder(ThemeData theme) async {
        await pumpThemed(
          tester,
          const Card(child: SizedBox(width: 100, height: 60)),
          theme: theme,
        );
        return tester
            .widget<Material>(
              find.descendant(
                of: find.byType(Card),
                matching: find.byType(Material),
              ),
            )
            .elevation;
      }

      expect(await elevationUnder(ThemeData()), 1);
      expect(await elevationUnder(light), 0);
    });
  });
}

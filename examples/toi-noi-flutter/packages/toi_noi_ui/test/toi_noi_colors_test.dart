import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'test_helpers.dart';

void main() {
  const light = ToiNoiColors.light;
  const dark = ToiNoiColors.dark;

  group('lerp', () {
    test('t = 0 giữ màu của this, t = 1 ra màu của other', () {
      expect(light.lerp(dark, 0), light);
      expect(light.lerp(dark, 1), dark);
    });

    test('t = 0.5 là trung điểm của cả 12 màu', () {
      final mid = light.lerp(dark, 0.5);
      // (tên, giá trị ở giữa, màu sáng, màu tối)
      final fields = <(String, Color, Color, Color)>[
        ('success', mid.success, light.success, dark.success),
        ('onSuccess', mid.onSuccess, light.onSuccess, dark.onSuccess),
        (
          'successContainer',
          mid.successContainer,
          light.successContainer,
          dark.successContainer,
        ),
        (
          'onSuccessContainer',
          mid.onSuccessContainer,
          light.onSuccessContainer,
          dark.onSuccessContainer,
        ),
        ('warning', mid.warning, light.warning, dark.warning),
        ('onWarning', mid.onWarning, light.onWarning, dark.onWarning),
        (
          'warningContainer',
          mid.warningContainer,
          light.warningContainer,
          dark.warningContainer,
        ),
        (
          'onWarningContainer',
          mid.onWarningContainer,
          light.onWarningContainer,
          dark.onWarningContainer,
        ),
        ('danger', mid.danger, light.danger, dark.danger),
        ('onDanger', mid.onDanger, light.onDanger, dark.onDanger),
        (
          'dangerContainer',
          mid.dangerContainer,
          light.dangerContainer,
          dark.dangerContainer,
        ),
        (
          'onDangerContainer',
          mid.onDangerContainer,
          light.onDangerContainer,
          dark.onDangerContainer,
        ),
      ];
      expect(fields, hasLength(12));
      for (final (name, actual, a, b) in fields) {
        expect(actual, Color.lerp(a, b, 0.5), reason: name);
        expect(actual, isNot(a), reason: '$name phải khác màu sáng');
      }
    });

    test('other không phải ToiNoiColors thì trả về chính this', () {
      expect(identical(light.lerp(null, 0.5), light), isTrue);
    });

    test('ThemeData.lerp nội suy cả extension (đổi theme có animation)', () {
      final mid = ThemeData.lerp(ToiNoiTheme.light(), ToiNoiTheme.dark(), 0.5);
      expect(mid.extension<ToiNoiColors>(), light.lerp(dark, 0.5));
    });
  });

  group('copyWith, ==, hashCode', () {
    test('copyWith không tham số cho bản bằng nhau', () {
      expect(light.copyWith(), light);
      expect(light.copyWith().hashCode, light.hashCode);
    });

    test('copyWith chỉ đổi đúng trường được truyền', () {
      const red = Color(0xFFFF0000);
      final changed = light.copyWith(success: red);
      expect(changed.success, red);
      expect(changed.onSuccess, light.onSuccess);
      expect(changed.danger, light.danger);
      expect(changed, isNot(light));
    });

    test('hai bộ màu khác nhau thì không bằng nhau', () {
      expect(light == dark, isFalse);
    });
  });

  group('tương phản WCAG AA (4.5:1)', () {
    final sets = {'sáng': light, 'tối': dark};

    for (final entry in sets.entries) {
      final name = entry.key;
      final c = entry.value;
      final pairs = <String, (Color, Color)>{
        'onSuccess / success': (c.onSuccess, c.success),
        'onSuccessContainer / successContainer': (
          c.onSuccessContainer,
          c.successContainer,
        ),
        'onWarning / warning': (c.onWarning, c.warning),
        'onWarningContainer / warningContainer': (
          c.onWarningContainer,
          c.warningContainer,
        ),
        'onDanger / danger': (c.onDanger, c.danger),
        'onDangerContainer / dangerContainer': (
          c.onDangerContainer,
          c.dangerContainer,
        ),
      };
      for (final pair in pairs.entries) {
        test('$name: ${pair.key}', () {
          expect(
            contrastRatio(pair.value.$1, pair.value.$2),
            greaterThanOrEqualTo(4.5),
          );
        });
      }
    }
  });

  test('dùng nhầm màu của bản sáng trên nền tối thì mất tương phản', () {
    final darkSurface = ToiNoiTheme.dark().colorScheme.surface;
    expect(contrastRatio(light.success, darkSurface), lessThan(4.5));
    expect(contrastRatio(dark.success, darkSurface), greaterThan(4.5));
  });

  group('ToiNoiColors.of', () {
    testWidgets('lấy extension đã đăng ký trong ToiNoiTheme', (tester) async {
      late ToiNoiColors found;
      await pumpThemed(
        tester,
        Builder(
          builder: (context) {
            found = ToiNoiColors.of(context);
            return const SizedBox();
          },
        ),
        theme: ToiNoiTheme.dark(),
      );
      expect(found, dark);
    });

    testWidgets('ThemeData trơn thì chọn bản sáng hoặc tối theo brightness', (
      tester,
    ) async {
      final found = <ToiNoiColors>[];
      Widget probe() => Builder(
        builder: (context) {
          found.add(ToiNoiColors.of(context));
          return const SizedBox();
        },
      );
      await pumpThemed(tester, probe(), theme: ThemeData.light());
      await pumpThemed(tester, probe(), theme: ThemeData.dark());
      expect(found.first, light);
      expect(found.last, dark);
    });
  });
}

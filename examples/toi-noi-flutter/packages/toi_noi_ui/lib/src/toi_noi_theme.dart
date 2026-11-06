import 'package:flutter/foundation.dart' show defaultTargetPlatform;
import 'package:flutter/material.dart';

import 'toi_noi_colors.dart';
import 'toi_noi_radius.dart';
import 'toi_noi_spacing.dart';

/// Theme Material 3 của Tới Nơi, bản sáng và bản tối sinh từ cùng một seed.
///
/// Dùng trong `MaterialApp`:
///
/// ```dart
/// MaterialApp(
///   theme: ToiNoiTheme.light(),
///   darkTheme: ToiNoiTheme.dark(),
///   themeMode: ThemeMode.system,
/// )
/// ```
abstract final class ToiNoiTheme {
  /// Màu gốc của thương hiệu; `ColorScheme.fromSeed` suy ra mọi vai trò màu
  /// khác từ màu này.
  static const Color seed = Color(0xFF0F766E);

  // `ThemeData` bất biến nên dựng một lần cho mỗi nền tảng và dùng lại.
  // Khoá theo `defaultTargetPlatform` vì `ThemeData.platform` được chốt lúc
  // dựng; cache một đối tượng duy nhất sẽ làm
  // `debugDefaultTargetPlatformOverride` vô tác dụng ở các lần gọi sau.
  static final Map<(Brightness, TargetPlatform), ThemeData> _cache = {};

  static ThemeData _cached(Brightness brightness) => _cache.putIfAbsent((
    brightness,
    defaultTargetPlatform,
  ), () => _build(brightness));

  /// Theme sáng. Cùng nền tảng thì trả về cùng một đối tượng.
  static ThemeData light() => _cached(Brightness.light);

  /// Theme tối. Cùng nền tảng thì trả về cùng một đối tượng.
  static ThemeData dark() => _cached(Brightness.dark);

  // Chỉ khai báo phần khác mặc định; `ThemeData` gộp (merge) phần còn lại,
  // gồm cả màu chữ lấy từ ColorScheme.
  static const TextTheme _textTheme = TextTheme(
    headlineSmall: TextStyle(fontWeight: FontWeight.w700),
    titleLarge: TextStyle(fontWeight: FontWeight.w600),
    titleMedium: TextStyle(fontWeight: FontWeight.w600),
    bodyMedium: TextStyle(height: 1.4),
    labelLarge: TextStyle(fontWeight: FontWeight.w600),
  );

  static ThemeData _build(Brightness brightness) {
    final scheme = ColorScheme.fromSeed(
      seedColor: seed,
      brightness: brightness,
    );
    final inputBorder = OutlineInputBorder(
      borderRadius: BorderRadius.circular(ToiNoiRadius.input),
      borderSide: BorderSide(color: scheme.outline),
    );

    return ThemeData(
      colorScheme: scheme,
      textTheme: _textTheme,
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          minimumSize: const Size(64, 48),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(ToiNoiRadius.button),
          ),
        ),
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        margin: EdgeInsets.zero,
        color: scheme.surfaceContainerLow,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(ToiNoiRadius.card),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      inputDecorationTheme: InputDecorationThemeData(
        filled: true,
        fillColor: scheme.surfaceContainerHighest,
        contentPadding: const EdgeInsets.symmetric(
          horizontal: ToiNoiSpacing.md,
          vertical: ToiNoiSpacing.md,
        ),
        border: inputBorder,
        enabledBorder: inputBorder,
        focusedBorder: inputBorder.copyWith(
          borderSide: BorderSide(color: scheme.primary, width: 2),
        ),
        errorBorder: inputBorder.copyWith(
          borderSide: BorderSide(color: scheme.error),
        ),
        focusedErrorBorder: inputBorder.copyWith(
          borderSide: BorderSide(color: scheme.error, width: 2),
        ),
      ),
      extensions: [
        // `danger` lấy thẳng từ `error` của scheme để chỉ có một màu đỏ.
        (brightness == Brightness.dark ? ToiNoiColors.dark : ToiNoiColors.light)
            .copyWith(
              danger: scheme.error,
              onDanger: scheme.onError,
              dangerContainer: scheme.errorContainer,
              onDangerContainer: scheme.onErrorContainer,
            ),
      ],
    );
  }
}

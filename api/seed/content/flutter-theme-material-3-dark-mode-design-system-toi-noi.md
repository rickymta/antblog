---
slug: flutter-theme-material-3-dark-mode-design-system-toi-noi
title: "Theme Material 3, màu, chữ và dark mode: design system Tới Nơi"
summary: "Bài 13: dựng theme Material 3 sáng/tối cho Tới Nơi bằng ColorScheme.fromSeed, TextTheme, ThemeExtension cho màu trạng thái chuyến và component theme; đóng gói thành package toi_noi_ui, kiểm dark mode theo hệ thống bằng widget test."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, theme, material-3, dark-mode, design-system]
language: vi
---

Hai màn hình của bài 12 đã nối với nhau, nhưng đội thiết kế Tới Nơi đặt chúng cạnh nhau và thấy lệch. Nút "Đặt xe" ở màn báo giá xanh một kiểu, nút ở màn kế tiếp xanh kiểu khác. Chip "Hoàn thành" tô thẳng `Colors.green`, nên khi tài xế bật chế độ tối, chữ xanh nhạt nằm trên nền xám và gần như biến mất.

Lối ra là **theme**: ta khai báo màu, kiểu chữ và hình dạng component ở một nơi, widget chỉ hỏi "màu chính lúc này là gì". Bài này dựng theme Material 3 cho Tới Nơi ở hai bản sáng và tối, thêm màu trạng thái chuyến mà Material không có, rồi đóng gói thành package `toi_noi_ui` cho hai app dùng chung. Bảng giá là số minh hoạ.

Đây là bài 13 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 12 về Navigator và route](/articles/flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen). Bạn cần biết widget, `State` và cách dùng package bằng `path:` như `toi_noi_core` ở [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core). Bối cảnh nghiệp vụ nằm ở hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>App chỉ truyền <code>theme: ToiNoiTheme.light()</code> vào <code>MaterialApp</code>, không truyền <code>darkTheme</code>, và để <code>themeMode</code> mặc định. Điện thoại đang ở chế độ tối. App hiện giao diện sáng, tối, hay báo lỗi?</p>
<details><summary>Xem đáp án</summary><p>Giao diện sáng, không có lỗi. <code>themeMode</code> mặc định là <code>ThemeMode.system</code> nên Flutter muốn chọn theme tối, nhưng khi <code>darkTheme</code> là <code>null</code> nó dùng lại <code>theme</code>. Lab có test <code>quên darkTheme</code> chứng minh điều này.</p></details>
</div>

## Theme là dữ liệu đi xuống cây widget

`ThemeData` chứa bộ màu, bộ kiểu chữ và cấu hình của từng component. `MaterialApp` đặt nó vào cây bằng widget `Theme`, một `StatelessWidget` bọc `InheritedWidget` riêng (`_InheritedTheme`). Widget con đọc bằng `Theme.of(context)`, lời gọi này đăng ký phụ thuộc: khi theme đổi, widget đã gọi nó được rebuild.

Ở Hình 1, hệ điều hành báo `platformBrightness`, app giữ `themeMode`, `MaterialApp` chọn đúng một `ThemeData`. `AnimatedTheme` bọc ngoài để chuyển từ sáng sang tối có nội suy, mặc định 200 ms.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Hệ điều hành báo platformBrightness và app giữ themeMode; cả hai đi vào MaterialApp để chọn theme hoặc darkTheme. ThemeData được chọn đi qua AnimatedTheme xuống cây, nơi widget đọc bằng Theme.of(context)."><defs><marker id="fl13-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">Hệ điều hành</text><text x="36" y="86" class="t-mono">platformBrightness</text><text x="36" y="108" class="t-small">Sáng hoặc Tối</text>
<rect x="20" y="170" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="202" class="t-b">State của app</text><text x="36" y="226" class="t-mono">themeMode</text><text x="36" y="248" class="t-small">system, light hoặc dark</text>
<rect x="330" y="30" width="220" height="90" rx="10" class="f-pk-soft s-pk"/><text x="346" y="62" class="t-b">MaterialApp</text><text x="346" y="86" class="t-mono">theme / darkTheme</text><text x="346" y="108" class="t-small">chọn một ThemeData</text>
<rect x="640" y="30" width="220" height="90" rx="10" class="f-pk-soft s-pk"/><text x="656" y="62" class="t-b">AnimatedTheme</text><text x="656" y="86" class="t-small">nội suy khi đổi theme</text><text x="656" y="108" class="t-small">mặc định 200 ms</text>
<rect x="640" y="170" width="220" height="100" rx="10" class="f-phys-soft s-phys"/><text x="656" y="202" class="t-b">Widget trong cây</text><text x="656" y="226" class="t-mono">Theme.of(context)</text><text x="656" y="250" class="t-small">đọc màu, chữ, extension</text>
<path d="M242 75 L328 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl13-1-a)"/>
<path d="M242 215 L286 215 L286 102 L328 102" class="f-none s-pk" stroke-width="2" marker-end="url(#fl13-1-a)"/>
<path d="M552 75 L638 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl13-1-a)"/><text x="562" y="64" class="t-small">ThemeData</text>
<path d="M750 122 L750 168" class="f-none s-pk" stroke-width="2" marker-end="url(#fl13-1-a)"/><text x="740" y="150" text-anchor="end" class="t-small">đăng ký phụ thuộc</text>
</svg>
</div>
<figcaption>Hình 1. Đường đi của một ThemeData. Mũi tên là chiều dữ liệu: hệ thống và themeMode quyết định, MaterialApp chọn, AnimatedTheme đưa xuống cây, widget đọc bằng Theme.of.</figcaption>
</figure>

Quy tắc chọn (các tổ hợp chính có test trong lab):

| `themeMode` | `platformBrightness` | Theme được dùng |
|---|---|---|
| `system` | sáng | `theme` |
| `system` | tối | `darkTheme` (nếu `null` thì `theme`) |
| `light` | bất kỳ | `theme` |
| `dark` | bất kỳ | `darkTheme` (nếu `null` thì `theme`) |

Trong lab `13-theme-design-system`, `ThemeGalleryApp` giữ `ThemeMode` trong `State`, bắt đầu bằng `ThemeMode.system`, và cho ghi đè bằng bộ chọn Sáng, Tối, Hệ thống.

### app.dart · 13-theme-design-system/lib/app.dart

```dart
class _ThemeGalleryAppState extends State<ThemeGalleryApp> {
  late ThemeMode _themeMode = widget.initialThemeMode;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      themeMode: _themeMode,
      home: GalleryScreen(
        themeMode: _themeMode,
        onThemeModeChanged: (mode) => setState(() => _themeMode = mode),
      ),
    );
  }
}
```

`ToiNoiTheme.light()` và `dark()` dựng `ThemeData` một lần cho mỗi nền tảng và trả về cùng đối tượng ở các lần gọi sau, nên gọi chúng trong `build` không tốn thêm chi phí nào.

Dark mode kiểm được không cần điện thoại: test đổi `platformBrightness` của `platformDispatcher` giả, và `MaterialApp` đọc nó qua `MediaQuery.platformBrightnessOf`. Hai hàm phụ:

### support.dart · 13-theme-design-system/test/support.dart

```dart
/// Đặt độ sáng của hệ thống (platformBrightness) cho test, tự trả về mặc định
/// khi test kết thúc.
void setPlatformBrightness(WidgetTester tester, Brightness brightness) {
  tester.platformDispatcher.platformBrightnessTestValue = brightness;
  addTearDown(tester.platformDispatcher.clearPlatformBrightnessTestValue);
}

/// Độ sáng của theme đang áp dụng cho phần thân màn hình.
Brightness themeBrightness(WidgetTester tester) {
  return Theme.of(tester.element(find.byType(Scaffold))).brightness;
}
```

Test dùng chúng để chứng minh hai điều. Một, hệ thống đổi khi app đang chạy thì app đổi theo:

### theme_mode_test.dart · 13-theme-design-system/test/theme_mode_test.dart (trích)

```dart
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
```

Hai, đã chọn Sáng thì hệ thống đổi cũng không kéo app theo:

### theme_mode_test.dart · 13-theme-design-system/test/theme_mode_test.dart (trích)

```dart
Future<void> _choose(WidgetTester tester, String label) async {
  await tester.tap(find.text(label));
  // AnimatedTheme chuyển theme trong một khoảng thời gian.
  await tester.pumpAndSettle();
}

  // ...

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
```

`pumpAndSettle` là bắt buộc vì `AnimatedTheme` cần vài khung hình để tới theme mới.

## ColorScheme.fromSeed: một màu gốc, hai bảng màu

Material 3 mô tả màu bằng **vai trò** (role): `primary`, `onPrimary`, `primaryContainer`, `surface`, `error`. Component xin theo vai trò, theme quyết định giá trị. `ColorScheme.fromSeed` sinh cả bộ từ một màu gốc (seed), biến thể mặc định là `tonalSpot`; `brightness` chọn bản sáng hay tối.

### toi_noi_theme.dart · packages/toi_noi_ui/lib/src/toi_noi_theme.dart (trích)

```dart
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

  // ...

  static ThemeData _build(Brightness brightness) {
    final scheme = ColorScheme.fromSeed(
      seedColor: seed,
      brightness: brightness,
    );
```

Ba điều cần nhớ, đều có test:

- Seed không trở thành `primary` nguyên xi: `primary` bản sáng sinh ra khác seed `0xFF0F766E`.
- Sáng và tối sinh từ cùng seed nhưng chọn tông khác (Hình 2): bản tối có `primary` sáng hơn và `surface` tối hơn; test so độ chói.
- Vai trò `on...` là màu chữ và icon đặt **trên** màu cùng tên. Test kiểm sáu cặp (gồm `error` trên `surface`), ở cả hai theme, đều đạt tương phản tối thiểu 4,5:1.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Một seed đi vào ColorScheme.fromSeed. Tham số brightness tách thành hai bảng màu: bản sáng với primary đậm hơn và surface sáng, bản tối với primary sáng hơn và surface tối. Mỗi vai trò có một cặp on đặt chữ lên trên."><defs><marker id="fl13-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="105" width="170" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="138" class="t-b">Seed</text><text x="36" y="162" class="t-mono">0xFF0F766E</text><text x="36" y="186" class="t-small">màu thương hiệu</text>
<rect x="260" y="105" width="190" height="100" rx="10" class="f-phys-soft s-phys"/><text x="276" y="138" class="t-b">ColorScheme</text><text x="276" y="162" class="t-mono">fromSeed(...)</text><text x="276" y="186" class="t-small">chọn sáng hoặc tối</text>
<rect x="530" y="20" width="330" height="120" rx="10" class="f-surface s-line"/><text x="546" y="50" class="t-b">Bản sáng</text><text x="546" y="76" class="t-small">primary đậm hơn, surface sáng</text><text x="546" y="100" class="t-mono">onPrimary</text><text x="636" y="100" class="t-small">đặt trên primary</text><text x="546" y="124" class="t-mono">onSurface</text><text x="636" y="124" class="t-small">đặt trên surface</text>
<rect x="530" y="170" width="330" height="120" rx="10" class="f-surface s-line"/><text x="546" y="200" class="t-b">Bản tối</text><text x="546" y="226" class="t-small">primary sáng hơn, surface tối</text><text x="546" y="250" class="t-mono">onPrimary</text><text x="636" y="250" class="t-small">đặt trên primary</text><text x="546" y="274" class="t-mono">onSurface</text><text x="636" y="274" class="t-small">đặt trên surface</text>
<path d="M192 155 L258 155" class="f-none s-phys" stroke-width="2" marker-end="url(#fl13-2-a)"/>
<path d="M452 140 L528 90" class="f-none s-phys" stroke-width="2" marker-end="url(#fl13-2-a)"/>
<path d="M452 170 L528 220" class="f-none s-phys" stroke-width="2" marker-end="url(#fl13-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Từ một seed ra hai ColorScheme. Mỗi vai trò đi đôi với một vai trò on...; widget chỉ hỏi vai trò, không biết giá trị.</figcaption>
</figure>

Trong Tới Nơi, các vai trò được dùng như sau:

| Vai trò | Dùng cho |
|---|---|
| `primary` và `onPrimary` | Nút chính "Đặt xe" |
| `primaryContainer` và `onPrimaryContainer` | Chip của chuyến đang diễn ra |
| `surfaceContainerLow`, `outlineVariant` | Nền và viền `Card` |
| `surfaceContainerHighest` | Nền ô nhập, chip "Đã đặt" |
| `error` | Viền và chữ báo lỗi trong form, đặt trên `surface` |

Quy tắc cho cả dự án: widget không viết `Colors.teal` hay `Color(0xFF...)`, vì hằng không biết theme. Widget lấy màu từ `Theme.of(context).colorScheme`.

## TextTheme và ToiNoiSpacing: chữ và khoảng cách cũng là token

`TextTheme` có 15 kiểu: ba cỡ `large`, `medium`, `small` cho mỗi nhóm `display`, `headline`, `title`, `body`, `label`. Ta chỉ ghi phần khác mặc định; `ThemeData` gộp (`merge`) phần còn lại lên bộ mặc định đã gắn màu từ `ColorScheme`.

### toi_noi_theme.dart · packages/toi_noi_ui/lib/src/toi_noi_theme.dart (trích)

```dart
  // Chỉ khai báo phần khác mặc định; `ThemeData` gộp (merge) phần còn lại,
  // gồm cả màu chữ lấy từ ColorScheme.
  static const TextTheme _textTheme = TextTheme(
    headlineSmall: TextStyle(fontWeight: FontWeight.w700),
    titleLarge: TextStyle(fontWeight: FontWeight.w600),
    titleMedium: TextStyle(fontWeight: FontWeight.w600),
    bodyMedium: TextStyle(height: 1.4),
    labelLarge: TextStyle(fontWeight: FontWeight.w600),
  );
```

Nhờ gộp, `titleLarge` mang `w600` của ta và vẫn có màu `onSurface`, nên sang theme tối chữ đổi màu theo. Chỗ dễ vấp: cỡ chữ chỉ có sau khi theme vào cây, vì `Theme.of(context)` trả bản đã "localize" theo `MaterialLocalizations`; đọc `ToiNoiTheme.light().textTheme.bodyLarge!.fontSize` ngoài cây thì ra `null` (có test).

Khoảng cách cũng được đặt tên, trong `abstract final class` chỉ chứa hằng như `FarePolicy` ở bài 06:

### toi_noi_spacing.dart · packages/toi_noi_ui/lib/src/toi_noi_spacing.dart (trích)

```dart
abstract final class ToiNoiSpacing {
  /// 4: khoảng cách giữa icon và chữ trong cùng một chip.
  static const double xs = 4;

  /// 8: khoảng cách giữa hai phần tử liền nhau trong một nhóm.
  static const double sm = 8;
```

Bo góc cũng có hằng: `ToiNoiRadius.button` và `input` là 12, `card` là 16.

Số tiền có `MoneyText`: lấy `titleMedium` của theme, gộp `style` truyền vào lên trên, và bật `FontFeature.tabularFigures()` để chữ số cùng độ rộng nếu font hỗ trợ. Chuỗi mặc định là `Money.toString()` (`25.500 VND`); tham số `formatter` cho bài 14 gắn định dạng theo locale.

Bẫy về màu: `titleMedium` ghim `onSurface`, nên dùng nguyên thì `MoneyText` trong `FilledButton` ra `onSurface` trên nền `primary`, tương phản chỉ 2,64:1 (sáng) và 1,32:1 (tối). Vì vậy `build` lấy màu từ `DefaultTextStyle`, nơi nút đã đặt `onPrimary`; test kiểm cả hai.

### money_text.dart · packages/toi_noi_ui/lib/src/money_text.dart (trích)

```dart
  @override
  Widget build(BuildContext context) {
    // `titleMedium` ghim màu `onSurface`; lấy màu của `DefaultTextStyle` để
    // chữ đổi màu theo nơi đặt (ví dụ onPrimary trong FilledButton).
    final inherited = DefaultTextStyle.of(context).style.color;
    final base = Theme.of(context).textTheme.titleMedium
        ?.copyWith(color: inherited);
    return Text(
      formatter?.call(money) ?? money.toString(),
      textAlign: textAlign,
      style: base
          ?.merge(style)
          .copyWith(fontFeatures: const [FontFeature.tabularFigures()]),
    );
  }
}
```

## ThemeExtension: màu mà ColorScheme không có

`ColorScheme` có `error` nhưng không có "thành công" hay "cảnh báo". Chip "Hoàn thành" cần xanh lá, chip "Đang tìm tài xế" cần vàng. Cách đúng là `ThemeExtension<T>`: lớp ta tự định nghĩa, đăng ký vào `ThemeData.extensions`, đọc bằng `Theme.of(context).extension<T>()`, và phải cài `copyWith` và `lerp`.

`ToiNoiColors` có ba vai trò `success`, `warning`, `danger`, mỗi vai trò bốn màu theo mẫu Material: màu chính, `on...`, `...Container`, `on...Container`. Ta chọn tay một bộ cho sáng và một bộ cho tối. Bộ `danger` đặt đúng bằng `error`, `onError`, `errorContainer`, `onErrorContainer` của scheme (theme gán lại bằng `copyWith`), để app không có hai màu đỏ gần nhau; test kiểm.

### toi_noi_colors.dart · packages/toi_noi_ui/lib/src/toi_noi_colors.dart (trích)

```dart
  /// Bộ màu cho giao diện sáng.
  static const ToiNoiColors light = ToiNoiColors(
    success: Color(0xFF1B6E3C),
    onSuccess: Color(0xFFFFFFFF),
    successContainer: Color(0xFFC8F0D3),
    onSuccessContainer: Color(0xFF0E5A2E),
    warning: Color(0xFF8A5A00),
    onWarning: Color(0xFFFFFFFF),
    warningContainer: Color(0xFFFFE3A6),
    onWarningContainer: Color(0xFF5C3D00),
    danger: Color(0xFFBA1A1A),
    onDanger: Color(0xFFFFFFFF),
    dangerContainer: Color(0xFFFFDAD6),
    onDangerContainer: Color(0xFF93000A),
  );
```

Màu chọn tay thì phải kiểm bằng số: test tính tỉ lệ tương phản WCAG của sáu cặp `on...` và nền ở mỗi bộ, tất cả từ 4,5 trở lên.

### test_helpers.dart · packages/toi_noi_ui/test/test_helpers.dart (trích)

```dart
/// Tỉ lệ tương phản WCAG giữa hai màu đặc (không trong suốt).
double contrastRatio(Color a, Color b) {
  final la = a.computeLuminance();
  final lb = b.computeLuminance();
  final lighter = la > lb ? la : lb;
  final darker = la > lb ? lb : la;
  return (lighter + 0.05) / (darker + 0.05);
}
```

Dùng nhầm bản sáng trên nền tối là lỗi thật: `success` bản sáng trên `surface` bản tối chỉ khoảng 2,9:1, còn `success` bản tối khoảng 10,8:1 (test ghi lại chênh lệch này). Vì vậy widget không viết `ToiNoiColors.light.success` mà gọi `ToiNoiColors.of(context)`:

### toi_noi_colors.dart · packages/toi_noi_ui/lib/src/toi_noi_colors.dart (trích)

```dart
  /// Nếu `ThemeData` không đăng ký [ToiNoiColors] (ví dụ app dùng
  /// `ThemeData()` trơn), trả về [light] hoặc [dark] theo
  /// `ThemeData.brightness` thay vì ném lỗi.
  static ToiNoiColors of(BuildContext context) {
    final theme = Theme.of(context);
    return theme.extension<ToiNoiColors>() ??
        (theme.brightness == Brightness.dark ? dark : light);
  }
```

`extension<ToiNoiColors>()` trả `null` nếu `ThemeData` không đăng ký nó (ví dụ `ThemeData()` trơn); thay vì để `!` crash, `of` chọn bản sáng hoặc tối theo `brightness`.

`lerp` có tác dụng thật. Khi theme đổi, `AnimatedTheme` nội suy `ThemeData` cũ sang mới, và `ThemeData.lerp` gọi `lerp` của từng extension. Bỏ sót một trường thì màu đó giữ giá trị cũ rồi nhảy ở cuối chuyển cảnh thay vì trượt dần.

### toi_noi_colors.dart · packages/toi_noi_ui/lib/src/toi_noi_colors.dart (trích)

```dart
  /// Nội suy từng màu. Khi [other] không phải [ToiNoiColors] trả về chính nó,
  /// đúng quy ước của `ThemeExtension.lerp`.
  @override
  ToiNoiColors lerp(ThemeExtension<ToiNoiColors>? other, double t) {
    if (other is! ToiNoiColors) {
      return this;
    }
    return ToiNoiColors(
      success: Color.lerp(success, other.success, t)!,
      onSuccess: Color.lerp(onSuccess, other.onSuccess, t)!,
      // ...
```

Test kiểm điểm giữa của cả 12 màu, và kiểm `ThemeData.lerp(sáng, tối, 0.5)` ra đúng điểm giữa của hai bộ màu.

## Component theme: chỉnh một lần, mọi nút theo

Hình dạng component cũng nên ở theme. `ThemeData` có một tham số cho mỗi component; ta cấu hình ba cái dùng nhiều nhất:

### toi_noi_theme.dart · packages/toi_noi_ui/lib/src/toi_noi_theme.dart (trích)

```dart
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
```

Mục `extensions` ở cuối khối là chỗ đăng ký `ToiNoiColors` theo độ sáng, kèm bộ `danger` lấy từ scheme. `ThemeData` nhận `inputDecorationTheme` dạng `InputDecorationThemeData` (vẫn chấp nhận lớp cũ `InputDecorationTheme`); `Card` đi qua `CardThemeData`. Màn hình nào dùng `FilledButton`, `Card`, `TextField` trơn cũng ra đúng giao diện Tới Nơi.

Khi một thuộc tính được đặt ở nhiều nơi, nút chọn **từng thuộc tính một** theo thứ tự `style` trên widget, component theme, rồi mặc định của Material (Hình 3). `FilledButtonTheme.of` đọc `FilledButtonTheme` gần nhất trong cây, nếu không có thì đọc `ThemeData.filledButtonTheme`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 3. Thứ tự ưu tiên của một thuộc tính nút. Mặc định của Material cho phần nhìn thấy cao 40. Component theme của Tới Nơi thay bằng 48. Style đặt trên widget thay tiếp bằng 56. Với mỗi thuộc tính, Flutter lấy giá trị đầu tiên khác null theo thứ tự widget, theme, mặc định."><defs><marker id="fl13-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="40" width="220" height="120" rx="10" class="f-surface s-muted"/><text x="36" y="74" class="t-b">Mặc định Material</text><text x="36" y="102" class="t-mono">ThemeData()</text><text x="36" y="130" class="t-small">nút cao 40</text>
<rect x="330" y="40" width="240" height="120" rx="10" class="f-pk-soft s-pk"/><text x="346" y="74" class="t-b">Component theme</text><text x="346" y="102" class="t-mono">filledButtonTheme</text><text x="346" y="130" class="t-small">ToiNoiTheme: nút cao 48</text>
<rect x="660" y="40" width="200" height="120" rx="10" class="f-phys-soft s-phys"/><text x="676" y="74" class="t-b">style trên widget</text><text x="676" y="102" class="t-mono">styleFrom(...)</text><text x="676" y="130" class="t-small">ví dụ: nút cao 56</text>
<path d="M242 100 L328 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl13-3-a)"/><text x="252" y="88" class="t-small">thay bởi</text>
<path d="M572 100 L658 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl13-3-a)"/><text x="582" y="88" class="t-small">thay bởi</text>
<text x="20" y="210" class="t-b">Với mỗi thuộc tính: lấy giá trị đầu tiên khác null theo thứ tự widget, theme, mặc định.</text>
<text x="20" y="238" class="t-small">Con số 40, 48 và 56 là chiều cao phần nhìn thấy của nút, đo bằng test.</text>
</svg>
</div>
<figcaption>Hình 3. Thứ tự ưu tiên khi một thuộc tính của nút được đặt ở nhiều tầng. Mũi tên chỉ tầng nào thay thế tầng nào.</figcaption>
</figure>

Ba con số 40, 48, 56 đều đo được, nhưng có một cái bẫy: dù `ThemeData()` mặc định cho phần nhìn thấy của nút cao 40, cả widget `FilledButton` vẫn chiếm 48. Trên Android và iOS, `ThemeData` đặt `MaterialTapTargetSize.padded`, đệm vùng chạm tới `kMinInteractiveDimension` (48). Nên `tester.getSize(find.byType(FilledButton))` không phân biệt được gì; test của ta đo `Material` bên trong nút:

### toi_noi_theme_test.dart · packages/toi_noi_ui/test/toi_noi_theme_test.dart (trích)

```dart
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
```

Cùng file còn kiểm `Card` mặc định nổi 1 còn `Card` của ta phẳng (0). Lưu ý: `ThemeData.lerp` không nội suy `inputDecorationTheme`, nó nhảy sang bản mới ở nửa chặng.

## Đóng gói thành package toi_noi_ui

App khách và app tài xế cần cùng một giao diện, nên theme nằm trong package `toi_noi_ui`, bố cục giống `toi_noi_core` ở bài 06. File `lib/toi_noi_ui.dart` chỉ chứa `export ... show ...`:

### toi_noi_ui.dart · packages/toi_noi_ui/lib/toi_noi_ui.dart

```dart
export 'src/money_text.dart' show MoneyText;
export 'src/toi_noi_colors.dart' show ToiNoiColors;
export 'src/toi_noi_radius.dart' show ToiNoiRadius;
export 'src/toi_noi_spacing.dart' show ToiNoiSpacing;
export 'src/toi_noi_theme.dart' show ToiNoiTheme;
export 'src/trip_status_chip.dart' show TripStatusChip;
```

Hình 4 vẽ chiều phụ thuộc: `toi_noi_core` là Dart thuần, không import gì từ UI, nên domain không biết màu hay widget.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 4. Chiều phụ thuộc. Lab toi_noi_13 phụ thuộc toi_noi_ui và toi_noi_core. toi_noi_ui phụ thuộc toi_noi_core. toi_noi_core không phụ thuộc package nào trong hai package kia."><defs><marker id="fl13-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="95" width="220" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="128" class="t-b">toi_noi_13</text><text x="36" y="152" class="t-small">lab: app demo</text><text x="36" y="176" class="t-mono">ThemeGalleryApp</text>
<rect x="330" y="20" width="220" height="100" rx="10" class="f-pk-soft s-pk"/><text x="346" y="53" class="t-b">toi_noi_ui</text><text x="346" y="77" class="t-small">theme, màu, widget</text><text x="346" y="101" class="t-mono">TripStatusChip</text>
<rect x="640" y="95" width="220" height="100" rx="10" class="f-phys-soft s-phys"/><text x="656" y="128" class="t-b">toi_noi_core</text><text x="656" y="152" class="t-small">Dart thuần</text><text x="656" y="176" class="t-mono">Money, TripStatus</text>
<path d="M242 120 L328 85" class="f-none s-phys" stroke-width="2" marker-end="url(#fl13-4-a)"/><text x="244" y="86" class="t-mono">path</text>
<path d="M552 85 L638 120" class="f-none s-phys" stroke-width="2" marker-end="url(#fl13-4-a)"/><text x="590" y="86" class="t-mono">path</text>
<path d="M242 175 L638 175" class="f-none s-phys" stroke-width="2" marker-end="url(#fl13-4-a)"/><text x="410" y="166" class="t-mono">path</text>
<text x="20" y="250" class="t-b">toi_noi_core không import toi_noi_ui: domain không biết giao diện.</text>
</svg>
</div>
<figcaption>Hình 4. Mũi tên chỉ chiều phụ thuộc, từ package khai báo dependency tới package được dùng.</figcaption>
</figure>

Widget chính của package là `TripStatusChip`: nhận một `TripStatus` của `toi_noi_core` (và `label` tuỳ chọn thay nhãn tiếng Việt, cho bản dịch ở bài 14), tra giao diện bằng một `switch` liệt kê đủ tám trạng thái, không có nhánh `_`:

### trip_status_chip.dart · packages/toi_noi_ui/lib/src/trip_status_chip.dart (trích)

```dart
typedef _StatusLook = ({Color background, Color foreground, IconData icon});

/// Chọn giao diện cho từng trạng thái.
///
/// `switch` liệt kê đủ tám giá trị và không có nhánh `_`: thêm một
/// `TripStatus` mới ở `toi_noi_core` thì `flutter analyze` báo lỗi ở đây
/// cho tới khi ta quyết định màu cho nó.
_StatusLook _lookOf(
  TripStatus status,
  ColorScheme scheme,
  ToiNoiColors colors,
) => switch (status) {
  TripStatus.requested => (
    background: scheme.surfaceContainerHighest,
    foreground: scheme.onSurfaceVariant,
    icon: Icons.receipt_long,
  ),
  TripStatus.searching => (
    background: colors.warningContainer,
    foreground: colors.onWarningContainer,
    icon: Icons.search,
  ),
  TripStatus.accepted => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.thumb_up_alt_outlined,
  ),
  TripStatus.arriving => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.directions_car,
  ),
  TripStatus.pickedUp => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.person_pin_circle,
  ),
  TripStatus.inTransit => (
    background: scheme.primaryContainer,
    foreground: scheme.onPrimaryContainer,
    icon: Icons.route,
  ),
  TripStatus.completed => (
    background: colors.successContainer,
    foreground: colors.onSuccessContainer,
    icon: Icons.check_circle,
  ),
  TripStatus.cancelled => (
    background: colors.dangerContainer,
    foreground: colors.onDangerContainer,
    icon: Icons.cancel,
  ),
};
```

Nếu ai thêm một `TripStatus` vào `toi_noi_core`, `flutter analyze` báo `non_exhaustive_switch_expression` ngay chỗ này (ta đã thử bằng cách xoá nhánh `cancelled`). Phải quyết định màu cho chip thì code mới build được.

Bốn trạng thái đang diễn ra dùng chung `primaryContainer` nhưng khác icon, có chủ ý: trạng thái không nên chỉ dựa vào màu. Test kiểm tám icon đôi một khác nhau.

Test của chip chạy trên tám trạng thái và hai theme: có nhãn, đúng một icon, tương phản chữ và nền từ 4,5. Thêm `meetsGuideline(textContrastGuideline)` trên cả tám chip; nó có tác dụng, vì đặt chữ chip `completed` trùng màu nền thì cả hai theme đỏ với "found 1.00".

Guideline chụp ảnh màn hình test nhưng cũng đọc `TextStyle` để chọn ngưỡng: chữ lớn (từ 18px, hoặc từ 14px nếu đậm) cần 3:1, chữ thường cần 4,5:1. Xám 3,45:1 trên nền trắng bị báo ở 14px và đạt ở 18px; chữ trắng trên nền trắng đứng riêng bị báo 1,00:1. Nhưng nó chỉ kiểm `Text` mà nó tìm lại được bằng `find.text(nhãn semantics)`. Hai `Text` trong cùng một item `ListView` bị gộp thành một node, nhãn ghép `"Dòng một\nDòng hai"` không khớp `find.text` nào, nên chữ vô hình trong item đó lọt qua. Với `addSemanticIndexes: false`, mỗi `Text` là một node và bị bắt. Cả bốn hành vi đều có test:

### contrast_guideline_test.dart · packages/toi_noi_ui/test/contrast_guideline_test.dart (trích)

```dart
/// Xám 3,45:1 trên nền trắng: dưới ngưỡng 4,5 của chữ thường, trên ngưỡng 3
/// của chữ lớn.
const _grey = Color(0xFF8A8A8A);

  // ...

    testWidgets('chữ 14px 3,45:1 bị báo', (tester) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(
        _app(Center(child: Text('Đang tìm tài xế', style: _style(14)))),
      );
      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
      handle.dispose();
    });

  // ...

    testWidgets('nhãn ghép không khớp find.text nên chữ vô hình lọt qua', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(_app(ListView(children: [item()])));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
      handle.dispose();
    });

  // ...

    testWidgets('addSemanticIndexes: false thì mỗi Text là một node, bị báo', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(
        _app(ListView(addSemanticIndexes: false, children: [item()])),
      );
      expect(tester.getSemantics(find.text('Dòng một')).label, 'Dòng một');
      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
      handle.dispose();
    });
```

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với theme</span>
<ul>
<li><strong>Tô màu thẳng bằng <code>Colors.xxx</code> hay <code>Color(0xFF...)</code> trong widget.</strong> Dấu hiệu: bật dark mode thì vài chữ hoặc icon mất tương phản. Nguyên nhân: hằng màu không biết theme. Cách sửa: lấy từ <code>Theme.of(context).colorScheme</code> hoặc <code>ToiNoiColors.of(context)</code>.</li>
<li><strong>Chỉ truyền <code>theme</code>, quên <code>darkTheme</code>.</strong> Dấu hiệu: điện thoại ở chế độ tối nhưng app luôn sáng, không báo lỗi. Nguyên nhân: <code>darkTheme</code> là <code>null</code> thì Flutter dùng lại <code>theme</code>. Cách sửa: luôn truyền cả hai.</li>
<li><strong>Test đổi theme nhưng <code>Theme.of</code> vẫn trả theme cũ.</strong> Dấu hiệu: <code>pumpWidget</code> với theme mới mà <code>expect</code> brightness ra giá trị cũ. Nguyên nhân: <code>AnimatedTheme</code> mới đi được đoạn đầu của quá trình nội suy. Cách sửa: <code>pumpAndSettle()</code>, hoặc <code>themeAnimationDuration: Duration.zero</code> như <code>pumpThemed</code> ở package.</li>
<li><strong>Đọc <code>textTheme</code> thẳng từ <code>ThemeData</code> rồi gặp <code>null</code>.</strong> Dấu hiệu: <code>fontSize</code> là <code>null</code>, hoặc <code>Null check operator</code> khi gắn <code>!</code>. Nguyên nhân: cỡ chữ chỉ có sau khi <code>Theme.of(context)</code> localize theme. Cách sửa: đọc qua <code>Theme.of(context).textTheme</code> trong cây.</li>
<li><strong>Gọi <code>extension&lt;T&gt;()!</code> khi theme không đăng ký extension.</strong> Dấu hiệu: widget test crash <code>Null check operator used on a null value</code> vì quên bọc theme. Cách sửa: dùng <code>ToiNoiColors.of(context)</code> có bản dự phòng.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/packages/toi_noi_ui
flutter pub get
flutter analyze
flutter test

cd ../../13-theme-design-system
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` 0 issue ở cả hai project, `toi_noi_ui` có 85 test xanh, lab có 15 test xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `toi_noi_colors_test.dart` | `lerp` ở 0, 1 và điểm giữa của 12 màu; `copyWith`, `==`; 12 cặp `on...` đạt 4,5:1; dùng bản sáng trên nền tối mất tương phản; `of` có bản dự phòng |
| `toi_noi_theme_test.dart` | Sáng và tối cùng seed nhưng khác nhau; `TextTheme` giữ màu; nút cao 48 (mặc định 40); style trên widget thắng theme (56); `Card` phẳng; `danger` bằng `error`; theme dựng một lần |
| `trip_status_chip_test.dart` | 8 trạng thái ở hai theme, tương phản từ 4,5; ánh xạ màu; 8 icon khác nhau; `label`; chip qua `textContrastGuideline`; chữ phóng 2 lần không tràn |
| `contrast_guideline_test.dart` | Ngưỡng theo cỡ chữ (14px đỏ, 18px xanh); chữ trùng nền đỏ; chữ trong item `ListView` bị gộp semantics thì lọt, `addSemanticIndexes: false` thì bị bắt |
| `money_text_test.dart`, `toi_noi_spacing_test.dart` | 25.500, 0 và 453.000 VND; `formatter`; `onPrimary` trong `FilledButton`; chữ số đều độ rộng; năm mức khoảng cách |
| `theme_mode_test.dart` (lab) | `ThemeMode.system` theo `platformBrightness`; đổi giữa chừng; ghi đè rồi quay lại Hệ thống; quên `darkTheme` |
| `gallery_test.dart` (lab) | Đủ 8 chip, "5 km", 25.500 VND, component theme; cả bảng mẫu (viewport 800x3000) qua `textContrastGuideline` ở hai theme, đã thử đột biến |

**Chưa làm**: chạy trên iOS Simulator, Android emulator hay máy thật (máy kiểm chưa có Android SDK và Xcode đầy đủ), nên chưa nhìn bằng mắt và chưa đổi Sáng/Tối của hệ điều hành thật. Chưa có font tuỳ chỉnh và `intl` (bài 14), `Semantics` cho tiền và text scale lớn (bài 16), theme tương phản cao, hay lưu lựa chọn Sáng/Tối (bài 22). Muốn chạy lab, sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_13 --org vn.toinoi .`.

## Bài tập

1. Tới Nơi muốn chip "Đang tìm tài xế" dùng cùng màu với bốn trạng thái đang diễn ra (`primaryContainer`) thay vì màu cảnh báo. Sửa ở đâu, và test nào phải đổi?
2. `toi_noi_core` thêm trạng thái `scheduled` (đặt lịch). Không động tới `toi_noi_ui`, `flutter analyze` ở package đó báo gì, và vì sao đây là điều ta muốn?
3. Màn hình hiển thị một khoản giảm giá bằng `MoneyText` màu xanh. Một đồng nghiệp viết `style: TextStyle(color: ToiNoiColors.light.success)`. Có gì sai, và sửa thế nào?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Sửa nhánh <code>TripStatus.searching</code> trong <code>_lookOf</code> sang <code>scheme.primaryContainer</code> và <code>scheme.onPrimaryContainer</code>. Test <code>completed dùng successContainer, cancelled dùng dangerContainer, searching dùng warningContainer</code> sẽ đỏ; chuyển <code>searching</code> sang test nhóm <code>primaryContainer</code>.</p>
<p><strong>2.</strong> <code>non_exhaustive_switch_expression</code> trong <code>_lookOf</code>, vì <code>switch</code> không có nhánh <code>_</code> nên không bao phủ <code>TripStatus.scheduled</code>. Ta muốn vậy: thêm trạng thái mà chưa quyết định giao diện thì code không build được.</p>
<p><strong>3.</strong> <code>ToiNoiColors.light</code> là hằng của bản sáng, nên ở theme tối chữ vẫn xanh đậm trên nền tối, tương phản khoảng 2,9:1. Dùng <code>ToiNoiColors.of(context).success</code>: <code>of</code> đọc extension từ theme hiện tại nên ở theme tối ra màu sáng hơn (khoảng 10,8:1).</p>
</details>

Bài trước: [Navigator, route và truyền dữ liệu](/articles/flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen). Bài tiếp: [Asset, intl và localization](/articles/flutter-asset-intl-localization-tien-vnd-ngay-gio), nơi ta thay `Money.toString()` bằng định dạng tiền theo locale và thêm song ngữ vi/en.

## Tài liệu gốc

- [Use themes to share colors and font styles](https://docs.flutter.dev/cookbook/design/themes) và [Material design in Flutter](https://docs.flutter.dev/ui/design/material) (docs.flutter.dev).
- [ThemeData](https://api.flutter.dev/flutter/material/ThemeData-class.html), [Theme.of](https://api.flutter.dev/flutter/material/Theme/of.html) và [MaterialApp.themeMode](https://api.flutter.dev/flutter/material/MaterialApp/themeMode.html) (api.flutter.dev).
- [ColorScheme.fromSeed](https://api.flutter.dev/flutter/material/ColorScheme/ColorScheme.fromSeed.html) và [TextTheme](https://api.flutter.dev/flutter/material/TextTheme-class.html) (api.flutter.dev).
- [ThemeExtension](https://api.flutter.dev/flutter/material/ThemeExtension-class.html), [FilledButtonThemeData](https://api.flutter.dev/flutter/material/FilledButtonThemeData-class.html), [CardThemeData](https://api.flutter.dev/flutter/material/CardThemeData-class.html) và [InputDecorationThemeData](https://api.flutter.dev/flutter/material/InputDecorationThemeData-class.html) (api.flutter.dev).
- [MediaQueryData.platformBrightness](https://api.flutter.dev/flutter/widgets/MediaQueryData/platformBrightness.html) và [FontFeature.tabularFigures](https://api.flutter.dev/flutter/dart-ui/FontFeature/FontFeature.tabularFigures.html) (api.flutter.dev).
- [textContrastGuideline](https://api.flutter.dev/flutter/flutter_test/textContrastGuideline-constant.html) (api.flutter.dev).
- [Dark Mode](https://developer.apple.com/design/human-interface-guidelines/dark-mode) (developer.apple.com) và [Dark theme](https://developer.android.com/develop/ui/views/theming/darktheme) (developer.android.com).

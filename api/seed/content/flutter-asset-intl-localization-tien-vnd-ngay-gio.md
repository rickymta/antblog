---
slug: flutter-asset-intl-localization-tien-vnd-ngay-gio
title: "Asset, intl và localization: hiển thị tiền VND, ngày giờ và song ngữ vi/en"
summary: "Bài 14: khai báo asset với biến thể 2.0x/3.0x, định dạng tiền VND bằng NumberFormat.currency, đổi giờ UTC sang giờ máy bằng DateFormat, rồi làm song ngữ vi/en với ARB, gen-l10n và số nhiều ICU. Mọi kết quả lấy từ đầu ra thật của intl."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, intl, localization, asset, i18n]
language: vi
---

Một du khách người Anh mở app Tới Nơi ở Đà Nẵng. Màn hình báo giá ghi "Cước tạm tính: 25.500 VND" và "Đón lúc 16:30", trong khi đồng hồ của anh chỉ 23:30. Con số 25.500 cũng làm anh phân vân, vì ở quê anh dấu chấm là dấu thập phân. Cả ba lỗi có cùng một gốc: app in chuỗi cứng, theo quy ước của người viết code.

Bài này sửa gốc đó: khai báo logo như một **asset** nhiều độ phân giải, định dạng tiền và giờ bằng **intl**, rồi đưa chuỗi giao diện vào file **ARB** để app nói tiếng Việt hoặc tiếng Anh theo thiết bị. Bảng giá và số tài xế ở gần là số minh hoạ.

Đây là bài 14 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 13 về theme và design system](/articles/flutter-theme-material-3-dark-mode-design-system-toi-noi). Bạn cần biết widget, `State`, theme của `toi_noi_ui` và `Money` của `toi_noi_core` từ [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core). `Money.toString()` ở bài đó chèn dấu chấm bằng tay; hôm nay ta thay nó bằng `intl`. Bối cảnh nghiệp vụ nằm ở hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta in <code>Money(25500)</code> bằng <code>NumberFormat.currency(locale: 'vi', symbol: '₫', decimalDigits: 0)</code>. Kết quả trông như <code>25.500 ₫</code>. Giữa số và ký hiệu là dấu cách thường. Đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p>Sai. Khoảng trắng đó là U+00A0 (khoảng trắng không ngắt dòng), nhìn giống dấu cách nhưng là ký tự khác, nên <code>text.contains(' ')</code> trả <code>false</code> và so sánh với chuỗi gõ tay <code>'25.500 ₫'</code> thất bại. Test <code>khoảng trắng là NBSP, không phải dấu cách thường</code> trong lab chứng minh điều này.</p></details>
</div>

## Asset: khai báo một lần, Flutter chọn bản theo màn hình

**Asset** là file đóng gói cùng app (ảnh, font, JSON) và được đọc bằng một **key**, chính là đường dẫn khai báo trong `pubspec.yaml`. Nếu không khai báo, file nằm trong thư mục dự án vẫn không vào app. Lab khai báo logo như sau, kèm một mẫu `fonts:` đang comment:

### pubspec.yaml · 14-asset-intl-l10n/pubspec.yaml (trích)

```yaml
flutter:
  uses-material-design: true
  # Bật sinh code localization: `flutter pub get` đọc l10n.yaml và ghi
  # lib/l10n/app_localizations*.dart (các file này không commit).
  generate: true

  assets:
    # Chỉ cần liệt kê bản 1.0x; Flutter tự tìm 2.0x/logo.png và 3.0x/logo.png.
    - assets/images/logo.png

  # Font tuỳ chỉnh khai báo theo mẫu dưới đây. Lab không kèm file font vì
  # không đưa font bản quyền vào repository.
  # fonts:
  #   - family: ToiNoiSans
  #     fonts:
  #       - asset: assets/fonts/ToiNoiSans-Regular.ttf
  #       - asset: assets/fonts/ToiNoiSans-Bold.ttf
  #         weight: 700
```

Ta chỉ liệt kê bản gốc. Màn hình điện thoại có **devicePixelRatio** khác nhau: một điểm logic (dp) tương ứng 1, 2 hoặc 3 điểm ảnh vật lý. Muốn logo 48 dp nét trên mọi máy, ta cần ba file 48, 96 và 144 px. Chúng nằm trong thư mục con đặt tên theo hệ số, cạnh file gốc: `assets/images/logo.png`, `assets/images/2.0x/logo.png`, `assets/images/3.0x/logo.png`. Tài liệu Flutter cho biết chỉ cần khai báo bản gốc hoặc thư mục cha, còn Flutter đóng gói các biến thể. Lab này cũng chạy được khi ta khai báo cả thư mục `assets/images/` (đã thử thủ công, không có test riêng).

Ba file PNG do `tool/generate_assets.dart` tự vẽ, nên không dính bản quyền ảnh của ai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Màn hình báo devicePixelRatio cho AssetImage; AssetImage.obtainKey chọn một trong ba file: logo.png 48 px cho 1.0, 2.0x/logo.png 96 px cho tỉ lệ trên 1.0 đến 2.5, 3.0x/logo.png 144 px cho tỉ lệ trên 2.5."><defs><marker id="fl14-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="100" width="210" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="132" class="t-b">Màn hình</text><text x="36" y="156" class="t-mono">devicePixelRatio</text><text x="36" y="178" class="t-small">1.0, 2.0, 3.0 ...</text>
<rect x="330" y="100" width="230" height="90" rx="10" class="f-pk-soft s-pk"/><text x="346" y="132" class="t-b">AssetImage</text><text x="346" y="156" class="t-mono">obtainKey(config)</text><text x="346" y="178" class="t-small">chọn bản theo quy tắc</text>
<rect x="660" y="20" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="676" y="50" class="t-mono">logo.png</text><text x="676" y="74" class="t-small">48 px, bản 1.0x</text>
<rect x="660" y="110" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="676" y="140" class="t-mono">2.0x/logo.png</text><text x="676" y="164" class="t-small">96 px</text>
<rect x="660" y="200" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="676" y="230" class="t-mono">3.0x/logo.png</text><text x="676" y="254" class="t-small">144 px</text>
<path d="M232 145 L328 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-1-a)"/>
<path d="M562 130 L610 130 L610 55 L658 55" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-1-a)"/>
<path d="M562 145 L658 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-1-a)"/>
<path d="M562 160 L610 160 L610 235 L658 235" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-1-a)"/>
<text x="20" y="302" class="t-small">Tỉ lệ đúng 1.0: bản gốc. Trên 1.0 đến 2.5: bản 2.0x. Trên 2.5: bản 3.0x (kết quả test trong lab).</text>
</svg>
</div>
<figcaption>Hình 1. Từ devicePixelRatio của màn hình tới file ảnh được nạp. Mũi tên là chiều dữ liệu: màn hình báo tỉ lệ, AssetImage chọn đúng một file.</figcaption>
</figure>

Quy tắc chọn nằm trong mã nguồn `AssetImage`: trùng khớp thì lấy đúng bản; dưới 2.0 lấy bản cao hơn gần nhất; từ 2.0 trở lên lấy bản gần nhất, hoà thì lấy bản thấp. Test của lab đo từng mốc:

### assets_test.dart · 14-asset-intl-l10n/test/assets_test.dart (trích)

```dart
    Future<AssetBundleImageKey> keyFor(double dpr) {
      return const AssetImage(logo)
          .obtainKey(ImageConfiguration(devicePixelRatio: dpr));
    }

    for (final (dpr, name, scale) in [
      (1.0, 'assets/images/logo.png', 1.0),
      (1.5, 'assets/images/2.0x/logo.png', 2.0),
      (2.0, 'assets/images/2.0x/logo.png', 2.0),
      (2.5, 'assets/images/2.0x/logo.png', 2.0),
      (2.6, 'assets/images/3.0x/logo.png', 3.0),
      (3.0, 'assets/images/3.0x/logo.png', 3.0),
      (4.0, 'assets/images/3.0x/logo.png', 3.0),
    ]) {
      testWidgets('dpr $dpr chọn $name', (tester) async {
        final key = await keyFor(dpr);
        expect(key.name, name);
        expect(key.scale, scale);
      });
    }
  });
```

Ở màn 3.0, `Image` với `width: 48` dựng ảnh 144 px vào ô 48 dp: test lấy `RenderImage` và thấy `image.width` là 144, `scale` là 3. Test cũng giải mã cả ba PNG để xác nhận chúng là 48, 96 và 144 px.

Theo tài liệu Flutter, `Image` giải mã PNG, JPEG, WebP, GIF và BMP; SVG không nằm trong danh sách nên cần package như `flutter_svg`, lab chưa dùng. Font tuỳ chỉnh khai báo ở khoá `fonts:`; lab chỉ giữ mẫu comment vì không đưa font bản quyền vào repository. Nếu key sai, `rootBundle.load` ném `FlutterError` có câu `Unable to load asset`, còn `Image.asset` có `errorBuilder` để hiện ảnh thay thế; cả hai có test.

## Tiền VND: NumberFormat.currency và formatVnd

`Money.toString()` cho `25.500 VND`, đủ để ghi log nhưng không theo quy ước của người đọc. `NumberFormat.currency` của `intl` biết dấu nghìn, vị trí ký hiệu và số chữ số thập phân theo locale. Lab bọc nó lại:

### formatting.dart · 14-asset-intl-l10n/lib/formatting.dart (trích)

```dart
import 'package:intl/intl.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Định dạng [money] theo [locale] với ký hiệu đồng.
///
/// `vi` cho `25.500 ₫` (dấu chấm nghìn, ký hiệu đứng sau, cách bằng U+00A0);
/// `en` cho `₫25,500`. VND không có phần lẻ nên `decimalDigits: 0`.
String formatVnd(Money money, String locale) {
  final format = NumberFormat.currency(
    locale: locale,
    name: 'VND',
    symbol: '₫',
    decimalDigits: 0,
  );
  return format.format(money.vnd);
}
```

`locale` quyết định quy ước, `symbol: '₫'` chọn ký hiệu thay vì chữ `VND`, `name: 'VND'` là mã tiền tệ. `decimalDigits: 0` ghi rõ VND không có phần lẻ; ở `intl` 0.20.3 bỏ tham số này vẫn ra 0 chữ số, nhưng ta không muốn phụ thuộc bảng nội bộ. Kết quả thật, lấy từ test:

| Locale | `formatVnd(Money(25500), ...)` | Ghi chú |
|---|---|---|
| `vi` | `25.500 ₫` | dấu chấm nghìn; ký hiệu đứng sau; cách bằng U+00A0 |
| `en` | `₫25,500` | dấu phẩy nghìn; ký hiệu đứng trước; không có khoảng trắng |
| `Money.toString()` | `25.500 VND` | chèn tay; dấu cách thường |

Test bám đúng các ký tự này. Hằng `nbsp` được viết bằng escape để người đọc thấy nó khác dấu cách:

### format_vnd_test.dart · 14-asset-intl-l10n/test/format_vnd_test.dart (trích)

```dart
/// Khoảng trắng không ngắt dòng (U+00A0) mà intl chèn giữa số và ký hiệu ₫
/// trong locale vi. Nhìn giống dấu cách nhưng là ký tự khác.
const nbsp = '\u00A0';

void main() {
  group('formatVnd, locale vi', () {
    test('25.500 ₫: dấu chấm nghìn, ký hiệu đứng sau, cách bằng U+00A0', () {
      expect(formatVnd(const Money(25500), 'vi'), '25.500$nbsp₫');
    });

    test('khoảng trắng là NBSP, không phải dấu cách thường', () {
      final text = formatVnd(const Money(25500), 'vi');
      expect(text.contains(' '), isFalse);
      expect(text.contains(nbsp), isTrue);
      expect(text, isNot('25.500 ₫'));
    });
```

Lab dùng `calculateFare` của `toi_noi_core` để lấy số tiền thay vì gõ tay: 5 km ra `25.500 ₫`, 2,001 km ra `12.005 ₫`, 100 km ra `453.000 ₫`. Một locale sai như `xx_YY` ném `ArgumentError`.

Trên màn hình ta không tự ghép chuỗi tiền: `MoneyText` của `toi_noi_ui` nhận tham số `formatter` (mặc định vẫn là `Money.toString()`), và lab truyền `formatVnd` vào đó, ở mục sau. Chuỗi do `intl` trả về chỉ để hiển thị, không gửi lên API: dữ liệu truyền đi vẫn là `int` đồng của `Money`.

## Ngày giờ: lưu UTC, hiển thị theo giờ máy

Quy ước của Tới Nơi là lưu và truyền thời điểm ở UTC theo ISO-8601, hiển thị theo múi giờ thiết bị. Chỗ dễ sai là `DateFormat`: nó chỉ in các trường (giờ, phút, ngày) của `DateTime` được đưa vào. Đưa vào giá trị UTC thì nó in giờ UTC.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Thời điểm lưu ở UTC đi qua DateTime.utc, qua bước đổi múi giờ bằng toLocal hoặc cộng offset, rồi qua DateFormat theo locale để ra chuỗi 23:30 hoặc 11:30 PM. Nếu bỏ bước đổi múi giờ, DateFormat in 16:30 là giờ UTC."><defs><marker id="fl14-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl14-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="180" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">Lưu, truyền</text><text x="36" y="84" class="t-small">ISO-8601, UTC</text><text x="36" y="108" class="t-mono">16:30:00Z</text>
<rect x="240" y="30" width="180" height="100" rx="10" class="f-phys-soft s-phys"/><text x="256" y="60" class="t-b">Trong Dart</text><text x="256" y="84" class="t-mono">DateTime.utc</text><text x="256" y="108" class="t-small">isUtc là true</text>
<rect x="460" y="30" width="180" height="100" rx="10" class="f-pk-soft s-pk"/><text x="476" y="60" class="t-b">Đổi múi giờ</text><text x="476" y="84" class="t-mono">toLocal()</text><text x="476" y="108" class="t-small">hoặc cộng offset</text>
<rect x="680" y="30" width="180" height="100" rx="10" class="f-pk-soft s-pk"/><text x="696" y="60" class="t-b">DateFormat</text><text x="696" y="84" class="t-mono">yMd().add_jm()</text><text x="696" y="108" class="t-small">theo locale</text>
<rect x="190" y="190" width="400" height="80" rx="10" class="f-warn-soft s-warn"/><text x="206" y="222" class="t-b">Bỏ bước đổi múi giờ</text><text x="206" y="248" class="t-small">DateFormat in 16:30, là giờ UTC</text>
<rect x="620" y="190" width="240" height="80" rx="10" class="f-phys-soft s-phys"/><text x="636" y="222" class="t-mono">vi  6/11/2026 23:30</text><text x="636" y="248" class="t-mono">en  11/6/2026 11:30 PM</text>
<path d="M202 80 L238 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-2-a)"/>
<path d="M422 80 L458 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-2-a)"/>
<path d="M642 80 L678 80" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-2-a)"/>
<path d="M770 132 L770 188" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-2-a)"/>
<path d="M330 132 L330 188" class="f-none s-warn" stroke-width="2" marker-end="url(#fl14-2-w)"/>
</svg>
</div>
<figcaption>Hình 2. Từ UTC tới chuỗi hiển thị. Mũi tên giữa các hộp là luồng đúng; hộp cảnh báo là kết quả khi bỏ bước đổi múi giờ. Ví dụ ở UTC+7.</figcaption>
</figure>

Hàm của lab làm đúng thứ tự ở Hình 2:

### formatting.dart · 14-asset-intl-l10n/lib/formatting.dart (trích)

```dart
/// Hiển thị thời điểm [utc] theo giờ của thiết bị và quy ước của [locale]:
/// ngày dạng ngắn, rồi giờ theo kiểu 12 hoặc 24 giờ của locale.
///
/// [utc] bắt buộc là UTC (đúng quy ước lưu và truyền của Tới Nơi). Mặc định
/// đổi sang múi giờ của máy bằng `toLocal()`. [offset] chỉ để thay múi giờ
/// của máy bằng một độ lệch cố định, giúp test không phụ thuộc máy chạy test.
String formatTripTime(DateTime utc, String locale, {Duration? offset}) {
  if (!utc.isUtc) {
    throw ArgumentError.value(utc, 'utc', 'phải là giờ UTC');
  }
  final shown = offset == null ? utc.toLocal() : utc.add(offset);
  return DateFormat.yMd(locale).add_jm().format(shown);
}
```

Giờ không phải UTC bị từ chối bằng `ArgumentError`, để lỗi lộ ngay ở chỗ gọi. Mặc định là `toLocal()`, tức múi giờ của máy chạy app. Tham số `offset` chỉ để test: nó thay múi giờ của máy bằng độ lệch cố định. Test `không truyền offset thì dùng toLocal()` so nhánh mặc định với `utc.toLocal().timeZoneOffset`. Giới hạn: trên máy đặt múi giờ UTC (như nhiều máy CI), test vẫn xanh dù bỏ `toLocal()`; test này chỉ có tác dụng kiểm tra khi múi giờ khác UTC.

Skeleton `yMd().add_jm()` cho `intl` tự chọn thứ tự và kiểu giờ theo locale. Cùng một thời điểm 16:30 UTC:

| Độ lệch | `vi` | `en` |
|---|---|---|
| UTC+0 | `6/11/2026 16:30` | không kiểm riêng |
| UTC+7 | `6/11/2026 23:30` | `11/6/2026 11:30 PM` (U+202F trước PM) |
| UTC-5 | `6/11/2026 11:30` | không kiểm riêng |
| UTC+5:30 | `6/11/2026 22:00` | không kiểm riêng |

Ngày đổi theo giờ: 18:00 UTC cộng 7 tiếng là `7/11/2026 1:00` ở `vi` và `11/7/2026 1:00 AM` ở `en`, tức đã sang ngày hôm sau.

`intl` là package Dart thuần nên chỉ biết quy ước của locale, không biết cài đặt "dùng giờ 24" của người dùng; Flutter có `MediaQuery.alwaysUse24HourFormatOf` cho việc đó, lab chưa làm.

`DateFormat` còn cần **dữ liệu ngày tháng của locale**. Trong unit test thuần Dart ta phải gọi `initializeDateFormatting()`; thiếu bước đó, `DateFormat.yMd('vi')` ném `LocaleDataException` (test `date_symbols_test.dart` chứng minh). Trong app thì `GlobalMaterialLocalizations` nạp giúp khi load, nên test widget của lab không gọi hàm này mà `vi` vẫn chạy.

### format_time_test.dart · 14-asset-intl-l10n/test/format_time_test.dart (trích)

```dart
  });

  group('lỗi dùng sai', () {
    test('giờ không phải UTC bị từ chối', () {
      final local = DateTime(2026, 11, 6, 23, 30);
      expect(() => formatTripTime(local, 'vi'), throwsArgumentError);
    });

    test('quên đổi múi giờ: DateFormat in nguyên trường của giá trị UTC', () {
      // Đây là lỗi mà formatTripTime tránh: DateFormat không tự đổi múi giờ.
      final naive = DateFormat.yMd('vi').add_jm().format(utc);
      expect(naive, '6/11/2026 16:30');
      expect(naive, isNot(formatTripTime(utc, 'vi', offset: vietnam)));
```

## Chuỗi giao diện: ARB và gen-l10n

Giờ đến lượt các chuỗi như "Báo giá chuyến". Cách chuẩn của Flutter là **ARB**, một file JSON cho mỗi ngôn ngữ, và công cụ `gen-l10n` sinh lớp Dart an toàn về kiểu. Bản vi là bản mẫu, có mô tả và kiểu của placeholder cho người dịch:

### app_vi.arb · 14-asset-intl-l10n/lib/l10n/app_vi.arb (trích)

```json
  "quoteDistance": "Quãng đường: {km} km",
  "@quoteDistance": {
    "description": "Quãng đường chuyến, số thực theo locale",
    "placeholders": {
      "km": {
        "type": "double",
        "format": "decimalPattern"
      }
    }
  },
  ...
  "driversNearby": "{count, plural, =0{Chưa có tài xế ở gần} other{{count} tài xế ở gần}}",
  "@driversNearby": {
    "description": "Số tài xế quanh điểm đón",
    "placeholders": {
      "count": {
        "type": "int",
        "format": "decimalPattern"
      }
    }
  },
```

Bản en chỉ cần các cặp khoá và chuỗi:

### app_en.arb · 14-asset-intl-l10n/lib/l10n/app_en.arb (trích)

```json
{
  "@@locale": "en",
  "appTitle": "Toi Noi",
  "quoteTitle": "Trip quote",
  "quoteDistance": "Distance: {km} km",
  "quoteFare": "Estimated fare",
  "quoteRequestedAt": "Requested at {time}",
  "driversNearby": "{count, plural, =0{No drivers nearby} =1{1 driver nearby} other{{count} drivers nearby}}",
```

Câu `driversNearby` dùng **số nhiều ICU**: `{count, plural, =0{...} other{...}}`. `=0` khớp chính xác số 0, `other` là nhánh bắt buộc. Bản en thêm `=1` cho "1 driver"; bản vi chỉ có `=0` và `other` vì tiếng Việt không chia số ít, số nhiều. Placeholder `count` kiểu `int` với `format: decimalPattern` nên số lớn cũng theo locale:

### localization_test.dart · 14-asset-intl-l10n/test/localization_test.dart (trích)

```dart
  group('số nhiều ICU trong ARB', () {
    final vi = lookupAppLocalizations(const Locale('vi'));
    final en = lookupAppLocalizations(const Locale('en'));

    test('en có ba dạng: =0, =1 và other', () {
      expect(en.driversNearby(0), 'No drivers nearby');
      expect(en.driversNearby(1), '1 driver nearby');
      expect(en.driversNearby(2), '2 drivers nearby');
      expect(en.driversNearby(25), '25 drivers nearby');
    });

    test('vi không chia số ít, số nhiều: chỉ =0 và other', () {
      expect(vi.driversNearby(0), 'Chưa có tài xế ở gần');
      expect(vi.driversNearby(1), '1 tài xế ở gần');
      expect(vi.driversNearby(2), '2 tài xế ở gần');
    });

    test('số lớn có dấu phân cách nghìn theo locale', () {
      expect(vi.driversNearby(1000), '1.000 tài xế ở gần');
      expect(en.driversNearby(1000), '1,000 drivers nearby');
    });
```

Để bật bộ sinh, `pubspec.yaml` cần `flutter: generate: true` (đã thấy ở mục đầu) cùng hai dependency `flutter_localizations` (từ SDK) và `intl`; cấu hình nằm ở `l10n.yaml`:

### l10n.yaml · 14-asset-intl-l10n/l10n.yaml

```yaml
arb-dir: lib/l10n
template-arb-file: app_vi.arb
output-localization-file: app_localizations.dart
preferred-supported-locales: [vi]
nullable-getter: false
```

ARB mẫu là `app_vi.arb`; `preferred-supported-locales: [vi]` đặt `vi` đứng đầu danh sách (lý do ở mục sau); `nullable-getter: false` để `AppLocalizations.of(context)` trả kiểu không null.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Hai file ARB đi vào bộ sinh code, chạy bởi flutter pub get hoặc flutter gen-l10n. Bộ sinh ghi các file app_localizations cạnh ARB. MaterialApp nhận delegate và supportedLocales từ code sinh, rồi widget gọi AppLocalizations.of(context)."><defs><marker id="fl14-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="240" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">File ARB</text><text x="36" y="86" class="t-mono">lib/l10n/app_vi.arb</text><text x="36" y="108" class="t-mono">lib/l10n/app_en.arb</text>
<rect x="320" y="30" width="240" height="110" rx="10" class="f-pk-soft s-pk"/><text x="336" y="60" class="t-b">Bộ sinh code</text><text x="336" y="86" class="t-mono">flutter pub get</text><text x="336" y="108" class="t-mono">flutter gen-l10n</text><text x="336" y="128" class="t-small">đọc l10n.yaml</text>
<rect x="620" y="30" width="240" height="110" rx="10" class="f-phys-soft s-phys"/><text x="636" y="60" class="t-b">Code sinh</text><text x="636" y="86" class="t-mono">app_localizations.dart</text><text x="636" y="108" class="t-small">nằm cạnh ARB</text><text x="636" y="128" class="t-small">không commit</text>
<rect x="620" y="190" width="240" height="110" rx="10" class="f-pk-soft s-pk"/><text x="636" y="220" class="t-b">MaterialApp</text><text x="636" y="246" class="t-mono">localizationsDelegates</text><text x="636" y="268" class="t-mono">supportedLocales</text>
<rect x="320" y="190" width="240" height="110" rx="10" class="f-phys-soft s-phys"/><text x="336" y="220" class="t-b">Widget</text><text x="336" y="246" class="t-mono">AppLocalizations</text><text x="336" y="268" class="t-mono">.of(context)</text>
<path d="M262 85 L318 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-3-a)"/>
<path d="M562 85 L618 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-3-a)"/>
<path d="M740 142 L740 188" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-3-a)"/>
<path d="M618 245 L562 245" class="f-none s-pk" stroke-width="2" marker-end="url(#fl14-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Từ ARB tới widget. Mũi tên là chiều dữ liệu: ARB vào bộ sinh, bộ sinh ghi code Dart, MaterialApp đăng ký code đó, widget đọc chuỗi.</figcaption>
</figure>

Có ba điều cần nói rõ về **file sinh**, theo mã nguồn công cụ của Flutter 3.47.4 và các lần chạy thật ở máy kiểm.

- **Nơi sinh**: nếu không đặt `output-dir`, file `app_localizations.dart`, `app_localizations_vi.dart`, `app_localizations_en.dart` được ghi cạnh ARB, tức trong `lib/l10n/`. Cách cũ (synthetic package, import `package:flutter_gen/gen_l10n/app_localizations.dart`) đã bị gỡ, nay ta import thẳng file sinh trong mã nguồn của mình: đặt `synthetic-package: true` trong `l10n.yaml` làm công cụ dừng với thông báo `this feature has been removed`. Tài liệu trên docs.flutter.dev vẫn còn nhắc tuỳ chọn này, nên hãy kiểm lại với phiên bản Flutter của bạn.
- **Có commit không**: lab không commit, và `.gitignore` có dòng `lib/l10n/app_localizations*.dart`. File sinh luôn suy ra được từ ARB, nên commit chúng chỉ tạo cơ hội để hai bên lệch nhau. Người clone phải chạy `flutter pub get` trước.
- **Sinh khi nào**: `flutter pub get` sinh khi có `generate: true` và `l10n.yaml`; `flutter run` và `flutter build` cũng sinh. Nhưng `flutter analyze` và `flutter test` thì **không** sinh lại: ta sửa ARB, chạy `flutter test`, và file sinh vẫn cũ. Sau khi sửa ARB hãy chạy `flutter gen-l10n`.

## Ráp vào MaterialApp: delegate, locale và fallback

Code sinh cung cấp sẵn danh sách delegate và `supportedLocales`. Lab nối chúng vào `MaterialApp`:

### app.dart · 14-asset-intl-l10n/lib/app.dart (trích)

```dart
  Widget build(BuildContext context) {
    return MaterialApp(
      // onGenerateTitle chạy dưới Localizations nên dịch được tên app.
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      locale: _locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: QuoteScreen(
        distanceKm: widget.distanceKm,
        requestedAt: widget.requestedAt,
        utcOffset: widget.utcOffset,
        onLocaleChanged: (locale) => setState(() => _locale = locale),
      ),
    );
  }
```

`AppLocalizations.localizationsDelegates` gồm delegate của ta và ba delegate `GlobalMaterialLocalizations`, `GlobalCupertinoLocalizations`, `GlobalWidgetsLocalizations`. Ba delegate sau dịch chuỗi có sẵn: `MaterialLocalizations.cancelButtonLabel` là `Huỷ` ở vi và `Cancel` ở en (test kiểm cả hai). Thiếu delegate của ta thì `AppLocalizations.of(context)` gặp lỗi null check; test `quên đăng ký delegate` tái hiện lỗi này. `onGenerateTitle` chạy bên dưới `Localizations`, nên tên app cũng dịch được.

Trong màn hình, một nguồn locale duy nhất, `l10n.localeName`, dùng cho cả chuỗi, tiền và giờ. Nhãn "Cước tạm tính" là chuỗi ARB không còn placeholder; số tiền do `MoneyText` vẽ bằng `formatVnd`. Chip trạng thái cũng nhận chuỗi đã dịch qua tham số `label` (`TripStatusChip(status: TripStatus.searching, label: l10n.statusSearching)`), vì nhãn mặc định của `TripStatus` chỉ có một ngôn ngữ:

### quote_screen.dart · 14-asset-intl-l10n/lib/quote_screen.dart (trích)

```dart
          Text(l10n.quoteDistance(widget.distanceKm)),
          Row(
            children: [
              Text(l10n.quoteFare, style: textTheme.titleLarge),
              const SizedBox(width: ToiNoiSpacing.sm),
              MoneyText(
                fare,
                style: textTheme.titleLarge,
                formatter: (money) => formatVnd(money, l10n.localeName),
              ),
            ],
          ),
          Text(
            l10n.quoteRequestedAt(
              formatTripTime(
                widget.requestedAt,
                l10n.localeName,
                offset: widget.utcOffset,
              ),
            ),
          ),
```

Khi ngôn ngữ của thiết bị không được app hỗ trợ, Flutter chọn locale bằng `basicLocaleListResolution`: ưu tiên khớp cả ngôn ngữ và quốc gia, rồi khớp ngôn ngữ; nếu không khớp nào, nó lấy **phần tử đầu của `supportedLocales`**. Thứ tự đó do công cụ sinh quyết định. Ở máy kiểm, khi bỏ `preferred-supported-locales`, danh sách ra `[en, vi]` nên điện thoại tiếng Pháp rơi về tiếng Anh; với `[vi]` danh sách là `[vi, en]` và nó rơi về tiếng Việt. Test dưới đây bắt lỗi cấu hình này:

### localization_test.dart · 14-asset-intl-l10n/test/localization_test.dart (trích)

```dart
    testWidgets('thiết bị fr: không hỗ trợ nên rơi về vi nhờ '
        'preferred-supported-locales', (tester) async {
      setDeviceLocales(tester, [const Locale('fr', 'FR')]);
      await pumpApp(tester);

      expect(find.text('Báo giá chuyến'), findsOneWidget);
      expect(AppLocalizations.supportedLocales.first, const Locale('vi'));
    });

```

Muốn đổi ngôn ngữ ngay trong app, ta truyền `locale:` vào `MaterialApp` và `setState` khi người dùng chọn. Test chạm `English` rồi `Tiếng Việt` và kiểm chữ, tiền (`₫25,500` rồi `25.500 ₫`) và giờ đổi theo. Ngôn ngữ thiết bị trong test được đặt bằng `localesTestValue`:

### support.dart · 14-asset-intl-l10n/test/support.dart (trích)

```dart
/// Đặt danh sách ngôn ngữ ưu tiên của "thiết bị" cho test, tự trả về mặc định
/// khi test kết thúc.
void setDeviceLocales(WidgetTester tester, List<Locale> locales) {
  tester.platformDispatcher.localesTestValue = locales;
  addTearDown(tester.platformDispatcher.clearLocalesTestValue);
}
```

Số thực cũng theo locale: `2,001 km` ở vi nhưng `2.001 km` ở en, nên mọi số hiển thị đều nên đi qua `intl`.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với asset, intl và localization</span>
<ul>
<li><strong>So chuỗi tiền với dấu cách thường.</strong> Dấu hiệu: test báo hai chuỗi khác nhau mà in ra giống hệt. Nguyên nhân: <code>intl</code> chèn U+00A0 (vi) hoặc U+202F trước AM/PM (en). Cách sửa: viết ký tự đó bằng escape, như hằng <code>nbsp</code> của lab.</li>
<li><strong>Sửa ARB mà test vẫn thấy chuỗi cũ.</strong> Dấu hiệu: chuỗi mới không xuất hiện trong test. Nguyên nhân: <code>flutter analyze</code> và <code>flutter test</code> không sinh lại code (còn <code>flutter run</code> thì có). Cách sửa: <code>flutter gen-l10n</code> hoặc <code>flutter pub get</code>. Thiếu <code>generate: true</code> thì <code>flutter gen-l10n</code> dừng với thông báo về cờ này.</li>
<li><strong>Quên <code>localizationsDelegates</code>.</strong> Dấu hiệu: <code>Null check operator used on a null value</code> khi dựng màn hình. Nguyên nhân: <code>Localizations</code> không có <code>AppLocalizations</code> để trả. Cách sửa: truyền <code>AppLocalizations.localizationsDelegates</code>.</li>
<li><strong>Giờ lệch 7 tiếng ở Việt Nam.</strong> Dấu hiệu: màn hình ghi 16:30 khi đồng hồ chỉ 23:30. Nguyên nhân: đưa thẳng <code>DateTime</code> UTC cho <code>DateFormat</code>. Cách sửa: <code>toLocal()</code> trước khi định dạng, như <code>formatTripTime</code>.</li>
<li><strong>Điện thoại đặt ngôn ngữ app không hỗ trợ lại hiện tiếng Anh.</strong> Dấu hiệu: máy đặt tiếng Pháp mở app ra tiếng Anh dù bản mẫu là tiếng Việt. Nguyên nhân: không khớp locale nào thì lấy phần tử đầu của <code>supportedLocales</code>. Cách sửa: <code>preferred-supported-locales: [vi]</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/14-asset-intl-l10n
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` 0 issue, 65 test xanh, kể cả từ trạng thái sạch (xoá `.dart_tool`, `pubspec.lock` và file sinh rồi chạy lại ba lệnh). `flutter pub get` chính là bước sinh code localization. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `format_vnd_test.dart` (12) | Kết quả thật của `NumberFormat.currency` ở `vi` và `en`; U+00A0; số tiền từ `calculateFare`; khác `Money.toString()` |
| `format_time_test.dart` (8) | Đổi UTC sang UTC+7, UTC-5, UTC+5:30; qua nửa đêm; mặc định dùng `toLocal()`; giờ không UTC bị từ chối; quên đổi múi giờ cho kết quả sai |
| `date_symbols_test.dart` (1) | Chưa nạp dữ liệu locale thì ném `LocaleDataException` |
| `assets_test.dart` (17) | Manifest đủ ba bản; chọn bản ở bảy mức `devicePixelRatio`; PNG đúng 48, 96, 144 px; `Image` dựng đúng ở màn 3.0 và 1.0; asset thiếu |
| `localization_test.dart` (19) | Thiết bị `vi`, `en_GB`, `fr`; đổi locale trong app; số nhiều ICU; `MoneyText` và `TripStatusChip` nhận chuỗi đã dịch; chuỗi Material đổi theo locale; nhãn Semantics; thiếu delegate |
| `arb_test.dart` (8) | Hai ARB cùng khoá và placeholder; plural có `other`; `l10n.yaml` đúng; code sinh nằm trong `.gitignore` |

**Chưa làm**: chạy trên iOS Simulator, Android emulator hay máy thật (máy kiểm chưa có Android SDK và Xcode đầy đủ). Ngôn ngữ thiết bị được mô phỏng bằng `localesTestValue` và múi giờ bằng `offset`, nên chưa thử đổi hai thứ này trên hệ điều hành thật, chưa nhìn logo bằng mắt ở màn 1x, 2x, 3x. Với iOS, danh sách locale còn phải khai báo trong `Info.plist` của runner. Chưa có font tuỳ chỉnh, SVG, ngôn ngữ thứ ba, và chưa lưu lựa chọn ngôn ngữ (bài 22). Chip chỉ dịch cho trạng thái `searching`; 7 trạng thái còn lại chưa có chuỗi ARB. Muốn chạy lab, sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_14 --org vn.toinoi .`.

## Bài tập

1. Một điện thoại có `devicePixelRatio` 2.75. Flutter nạp file nào trong lab, ảnh bao nhiêu pixel? Nếu xoá thư mục `3.0x` thì sao?
2. Server trả `"2026-11-06T16:30:00Z"`. Đồng nghiệp viết `DateFormat.yMd('vi').add_jm().format(DateTime.parse(json['requestedAt']))` và cho rằng đúng vì chuỗi có chữ `Z`. Thiết bị ở Việt Nam hiện gì, vì sao, và sửa thế nào?
3. Tới Nơi thêm tiếng Hàn (`ko`). Liệt kê việc tối thiểu, và nói test nào của lab cần mở rộng.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> 2.75 lớn hơn trung điểm 2.5 của hai bản, nên chọn <code>3.0x/logo.png</code>, 144 px (test mốc 2.6 cùng nhánh). Xoá <code>3.0x</code>: giá trị lớn hơn khoá cao nhất còn lại nên lấy <code>2.0x/logo.png</code>, 96 px.</p>
<p><strong>2.</strong> Hiện <code>6/11/2026 16:30</code>. <code>DateTime.parse</code> với hậu tố <code>Z</code> cho <code>DateTime</code> UTC, và <code>DateFormat</code> không đổi múi giờ nên in nguyên giờ UTC, lệch 7 tiếng. Sửa bằng <code>formatTripTime(DateTime.parse(...), localeName)</code>, hàm này gọi <code>toLocal()</code> và từ chối giá trị không phải UTC.</p>
<p><strong>3.</strong> Thêm <code>lib/l10n/app_ko.arb</code> có <code>@@locale</code> là <code>ko</code>, chạy <code>flutter gen-l10n</code>; <code>supportedLocales</code> tự có thêm <code>ko</code>. Thêm một <code>ButtonSegment</code> vào bộ chọn, và với iOS khai báo <code>ko</code> trong danh sách locale của <code>Info.plist</code> (runner). <code>arb_test.dart</code> hiện chỉ so cặp vi và en nên phải mở rộng sang <code>ko</code>, và các test chuỗi, tiền, giờ cần thêm test case cho <code>ko</code>.</p>
</details>

Bài trước: [Theme Material 3, màu, chữ và dark mode](/articles/flutter-theme-material-3-dark-mode-design-system-toi-noi). Bài tiếp: [Animation cơ bản: implicit animation và Hero khi tìm tài xế](/articles/flutter-animation-implicit-hero-tim-tai-xe), nơi ta làm giao diện chuyển động.

## Tài liệu gốc

- [Assets and images](https://docs.flutter.dev/ui/assets/assets-and-images) và [Use a custom font](https://docs.flutter.dev/cookbook/design/fonts) (docs.flutter.dev).
- [Internationalizing Flutter apps](https://docs.flutter.dev/ui/internationalization) và [Breaking change: flutter generate i10n source](https://docs.flutter.dev/release/breaking-changes/flutter-generate-i10n-source) (docs.flutter.dev).
- [AssetImage](https://api.flutter.dev/flutter/painting/AssetImage-class.html), [AssetManifest](https://api.flutter.dev/flutter/services/AssetManifest-class.html) và [Image.asset](https://api.flutter.dev/flutter/widgets/Image/Image.asset.html) (api.flutter.dev).
- [basicLocaleListResolution](https://api.flutter.dev/flutter/widgets/basicLocaleListResolution.html) và [MaterialApp.localizationsDelegates](https://api.flutter.dev/flutter/material/MaterialApp/localizationsDelegates.html) (api.flutter.dev).
- [MediaQuery.alwaysUse24HourFormatOf](https://api.flutter.dev/flutter/widgets/MediaQuery/alwaysUse24HourFormatOf.html) (api.flutter.dev).
- [NumberFormat](https://pub.dev/documentation/intl/latest/intl/NumberFormat-class.html), [DateFormat](https://pub.dev/documentation/intl/latest/intl/DateFormat-class.html) và [package intl](https://pub.dev/packages/intl) (pub.dev).
- [flutter_localizations](https://api.flutter.dev/flutter/flutter_localizations/flutter_localizations-library.html) (api.flutter.dev).
- [Localizing your app](https://developer.apple.com/documentation/xcode/localization) (developer.apple.com) và [Localize your app](https://developer.android.com/guide/topics/resources/localization) (developer.android.com).

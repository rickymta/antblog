# Lab 14: Asset, intl và localization

Lab của bài **FL-14** trong loạt **Flutter qua Tới Nơi**: [Asset, intl và localization: hiển thị tiền VND, ngày giờ và song ngữ vi/en](../../../api/seed/content/flutter-asset-intl-localization-tien-vnd-ngay-gio.md).

Lab là app một màn hình báo giá song ngữ: logo PNG nạp từ asset (ba độ phân giải), quãng đường, cước VND vẽ bằng `MoneyText(formatter: formatVnd)`, thời điểm đặt (lưu UTC, hiển thị theo giờ máy bằng `formatTripTime`), số tài xế ở gần bằng câu số nhiều ICU, chip trạng thái `searching` dịch qua `TripStatusChip.label`, và bộ chọn Tiếng Việt / English. Giao diện dùng theme của [`toi_noi_ui`](../packages/toi_noi_ui); số tiền lấy từ `calculateFare` của [`toi_noi_core`](../packages/toi_noi_core). Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt) và số tài xế là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `pubspec.yaml` | `flutter: generate: true`, khai báo `assets/images/logo.png`, `flutter_localizations`, `intl` |
| `l10n.yaml` | Cấu hình `gen-l10n`: ARB mẫu `app_vi.arb`, `preferred-supported-locales: [vi]`, `nullable-getter: false` |
| `lib/l10n/app_vi.arb`, `app_en.arb` | Chuỗi giao diện (bản vi là bản mẫu, có mô tả và placeholder) |
| `lib/l10n/app_localizations*.dart` | **Code sinh**, nằm cạnh ARB, **không commit** (xem `.gitignore`) |
| `lib/formatting.dart` | `formatVnd(Money, locale)`, `formatTripTime(utc, locale, {offset})` |
| `lib/quote_screen.dart`, `lib/app.dart`, `lib/main.dart` | Màn hình báo giá, `MaterialApp` cấu hình locale, `main()` |
| `assets/images/logo.png`, `2.0x/logo.png`, `3.0x/logo.png` | PNG 48, 96 và 144 px, do `tool/generate_assets.dart` tự vẽ (không dùng ảnh có bản quyền) |
| `tool/generate_assets.dart` | Sinh lại ba file PNG: `dart tool/generate_assets.dart` |
| `test/format_vnd_test.dart` | 12 test: kết quả thật của `NumberFormat.currency` với `vi` và `en`, NBSP, ₫ |
| `test/format_time_test.dart` | 8 test: đổi UTC sang giờ hiển thị, qua nửa đêm, múi giờ lệch nửa giờ, giờ không UTC |
| `test/date_symbols_test.dart` | 1 test: `DateFormat` chưa nạp dữ liệu locale ném `LocaleDataException` |
| `test/assets_test.dart` | 17 test: manifest có đủ ba bản, chọn bản theo `devicePixelRatio`, PNG giải mã được, `Image.asset` dựng đúng, asset thiếu |
| `test/localization_test.dart` | 19 test: đổi locale vi/en, rơi về vi, số nhiều ICU, `MoneyText` và `TripStatusChip` nhận chuỗi đã dịch, Material đổi theo locale, thiếu delegate |
| `test/arb_test.dart` | 8 test: hai ARB cùng khoá, đủ placeholder, plural có `other`, cấu hình `l10n.yaml` |
| `test/support.dart` | Hàm phụ: đặt ngôn ngữ thiết bị, dựng app với giờ UTC+7 cố định |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/14-asset-intl-l10n
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (65 test). Mức: **Kiểm chứng tự động**.

`flutter pub get` chính là bước sinh `lib/l10n/app_localizations*.dart` (vì có `generate: true` và `l10n.yaml`), nên từ một bản clone mới chỉ cần chạy đủ ba lệnh trên theo thứ tự. `flutter analyze` và `flutter test` **không** sinh lại code: sau khi sửa file ARB hãy chạy `flutter gen-l10n` (hoặc `flutter pub get`). `flutter run` và `flutter build` thì tự sinh lại.

Riêng test `không truyền offset thì dùng toLocal()` chỉ có sức kiểm khi máy chạy test ở múi giờ khác UTC (trên máy UTC, `toLocal()` không đổi gì). Test dùng giờ UTC+7 cố định (tham số `utcOffset`) và gán ngôn ngữ thiết bị bằng `localesTestValue`, nên kết quả không phụ thuộc múi giờ hay ngôn ngữ của máy chạy test.

## Chạy trên máy (tự thử)

Lab chỉ commit `pubspec.yaml`, `analysis_options.yaml`, `l10n.yaml`, ARB, `lib/`, `assets/`, `tool/` và `test/`. Muốn chạy trên simulator hoặc emulator, sinh runner một lần:

```bash
flutter create --platforms=ios,android --project-name toi_noi_14 --org vn.toinoi .
flutter run
```

Đổi ngôn ngữ hệ thống (Settings > Language trên máy ảo) để thấy app theo ngôn ngữ thiết bị. Với iOS, danh sách locale hỗ trợ cần được khai báo trong `Info.plist` của runner, theo hướng dẫn trong chú thích của `app_localizations.dart`; lab không commit runner nên chưa làm bước này.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `intl` 0.20.3 (`flutter pub add intl`; `flutter_localizations` của SDK ghim phiên bản này), `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path).

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Ngôn ngữ thiết bị được mô phỏng bằng `localesTestValue`, múi giờ bằng `offset` cố định; chưa thử đổi ngôn ngữ và múi giờ của hệ điều hành thật, chưa kiểm cài đặt "giờ 24 giờ" của thiết bị.
- Chưa nhìn bằng mắt logo ở màn 1x, 2x, 3x: test chỉ kiểm bản nào được chọn và kích thước điểm ảnh.
- Chưa có font tuỳ chỉnh: pubspec chỉ kèm mẫu khai báo (đang comment) vì không đưa font bản quyền vào repository. Chưa có SVG (cần package ngoài như `flutter_svg`).
- Chưa lưu lựa chọn ngôn ngữ giữa các lần mở app (cần lưu trữ cục bộ, bài 22). Chip chỉ dịch cho trạng thái `searching`; 7 trạng thái còn lại chưa có chuỗi ARB.
- Chưa dịch sang ngôn ngữ thứ ba, chưa kiểm RTL, chưa dùng `untranslated-messages-file`, chưa có quy trình đưa ARB cho người dịch.

# Lab 26: sinh mã bằng freezed, json_serializable và riverpod_generator

Lab của bài **FL-26** trong loạt **Flutter qua Tới Nơi**: [Code generation: freezed, json_serializable, riverpod_generator và build_runner](../../../api/seed/content/flutter-code-generation-freezed-json-serializable-riverpod-generator.md). Bài kết thúc Phần 3.

Lab viết lại `QuoteDto` và `TripDto` của [`toi_noi_api`](../packages/toi_noi_api) (viết tay ở bài 19) bằng `freezed` + `json_serializable`, rồi **chứng minh trên cùng một JSON** rằng hai bản cho cùng kết quả: cùng giá trị khi hợp lệ, cùng loại lỗi (`DtoFormatException`) và cùng tên trường khi sai. Kèm theo là một bản "mặc định" (`NaiveTripDto`, không converter) để thấy code sinh lỏng hơn bản viết tay ở đâu, một union `sealed` của `freezed` cho state màn hình báo giá và các provider sinh bằng `@riverpod`.

Cước 25.500 VND (5 km) và 12.005 VND (2,001 km) là kết quả của `calculateFare` theo bảng giá minh hoạ của loạt bài, không phải bảng giá thực tế. Dữ liệu đều hư cấu.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `build.yaml` | Bật `checked: true` cho `json_serializable`: lỗi `fromJson` mang tên khoá thay vì `TypeError` trần |
| `analysis_options.yaml` | Loại trừ `**/*.g.dart`, `**/*.freezed.dart`; tắt `invalid_annotation_target` (xem bài viết) |
| `lib/dto/converters.dart` | `JsonConverter` đặt lại quy tắc chặt của bản viết tay: mét/version, số đồng, tiền tệ VND, enum không phân biệt hoa thường, thời điểm UTC |
| `lib/dto/decode.dart` | `decodeDto`: đổi `CheckedFromJsonException` về `DtoFormatException` để giữ hợp đồng lỗi của `toi_noi_api` |
| `lib/dto/quote_dto.dart`, `trip_dto.dart` | `QuoteDto`, `TripDto` bằng `@freezed` + converter; `toDomain`, `toSnapshot`, `fromDomain` |
| `lib/dto/naive_trip_dto.dart` | Bản mặc định, CHỈ để so sánh (không dùng cho dữ liệu thật) |
| `lib/quote_state.dart` | `QuoteInput` và union `sealed` `QuoteScreenState` (idle, loading, ready, failed) |
| `lib/quote_source.dart` | `QuoteSource` (cổng) và `DioQuoteSource` (giải JSON bằng DTO sinh mã, trả `ApiFailure`) |
| `lib/providers.dart` | `@Riverpod(keepAlive: true)` hàm, `@riverpod` class (`QuoteController`) và `@riverpod` hàm có tham số (family) |
| `lib/quote_screen.dart`, `lib/main.dart` | Màn hình báo giá `switch` vét cạn bốn nhánh; `main()` nối `DioQuoteSource` |
| `lib/**/*.freezed.dart`, `lib/**/*.g.dart` | File sinh, **được commit** (xem mục dưới) |
| `test/dto_parity_test.dart` | 66 test (47 ca JSON cho `TripDto`, 13 cho `QuoteDto`, thêm ca hai lỗi cùng lúc): cùng JSON vào bản viết tay và bản sinh mã, so kết quả |
| `test/naive_dto_test.dart` | Bản mặc định nhận gì mà bản viết tay từ chối (bảng trong bài viết) |
| `test/dto_behaviour_test.dart` | `==`, `copyWith`, `toString`, `toJson`, cầu nối domain |
| `test/fake_api_json_test.dart` | Body thật của `tools/fake_api` qua hai DTO; `DioQuoteSource` đổi lỗi thành `ApiFailure` |
| `test/quote_state_test.dart` | Union `sealed`: nhánh, `==`, `copyWith`, `when` |
| `test/providers_test.dart` | Provider sinh mã: thứ tự state, lỗi, dispose giữa chừng, auto-dispose, family |
| `test/quote_screen_test.dart` | Widget test màn hình, kể cả đi qua fake API trong tiến trình |
| `test/generated_files_test.dart` | File sinh có mặt và không bị `.gitignore` chặn |
| `test/support/` | Fixture JSON, bảng ca dùng chung, `FakeQuoteSource` điều khiển bằng `Completer` |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/26-codegen
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (134 test). **Không cần chạy `build_runner`**: file sinh đã nằm trong repo.

Mức kiểm chứng: **Kiểm chứng tự động**. Test không gọi mạng ngoài; báo giá đi qua `InProcessAdapter` vào fake API trong tiến trình.

## Sinh lại mã và kiểm file sinh không lệch nguồn

Sau khi sửa một file có `part '*.freezed.dart'` hay `part '*.g.dart'`:

```bash
dart run build_runner build   # sinh lại một lần
dart run build_runner watch   # sinh lại mỗi lần lưu file
```

`--delete-conflicting-outputs` (`-d`) đã bị gỡ ở `build_runner` 2.17.1: lệnh in `These options have been removed and were ignored` rồi vẫn chạy, nên không cần thêm.

Lần chạy đầu phải biên dịch các builder nên chậm (cỡ vài phút trên máy kiểm lúc nhiều tiến trình chạy cùng lúc); các lần sau nhanh hơn nhiều.

**Lệnh kiểm file sinh không lệch nguồn** (dùng trong CI; chạy từ thư mục lab):

```bash
dart run build_runner build --only-check
```

`--only-check` (từ `build_runner` 2.16) không ghi và không xoá gì, chỉ so kết quả mong đợi với file trên đĩa. Mã thoát 0: khớp. Mã thoát 1: nguồn hoặc file sinh đã đổi mà chưa sinh lại (kể cả sửa tay file sinh), và lệnh in tên file lệch. Không cần cây git sạch. Đã thử cả ba tình huống trên lab này.

Cách phụ dựa vào git: `dart run build_runner build && git diff --exit-code -- lib`. Nó ghi đè file sinh, chỉ có nghĩa khi cây làm việc sạch lúc bắt đầu, và `git diff` không thấy file **chưa được theo dõi** (lần đầu thêm file sinh mới, chạy thêm `git status --porcelain -- lib`).

**Rủi ro của lab**: `.gitignore` của lab bỏ `pubspec.lock` (quy ước của loạt bài) và các generator khai bằng caret, nên clone sạch có thể giải ra một bản vá generator mới mà code sinh khác đi, làm lệnh kiểm đỏ giả (suy từ changelog, chưa tái hiện). Dự án thật nên commit `pubspec.lock` và chạy `flutter pub get --enforce-lockfile` trong CI.

## Vì sao commit file sinh

Quy ước của lab: **commit** `*.freezed.dart` và `*.g.dart` để `flutter test` xanh ngay từ bản clone sạch, không cần cài builder (lần build đầu tốn nhiều phút) và để người đọc xem code sinh ra trong diff. Đánh đổi: diff dài (2.002 dòng sinh so với 633 dòng viết tay trong `lib/`), có thể xung đột khi hai nhánh cùng sửa nguồn, và có nguy cơ file sinh lệch nguồn, nên cần lệnh kiểm ở trên. Cách ngược lại (thêm `*.g.dart`, `*.freezed.dart` vào `.gitignore`, CI chạy `build_runner` trước `flutter test`) cũng hợp lệ; bài viết nêu khi nào nên chọn cách đó.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_26 --org vn.toinoi .
dart run bin/server.dart        # trong ../tools/fake_api, cổng 8787
flutter run
```

- `lib/main.dart` trỏ `http://localhost:8787`. iOS Simulator gọi được `localhost`; Android emulator phải đổi sang `10.0.2.2`; điện thoại thật dùng địa chỉ LAN của máy dev. Cấu hình theo môi trường là chuyện của bài 25.
- Ta **chưa chạy** app trên simulator, emulator hay thiết bị (máy kiểm không có Android SDK, CocoaPods).

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, Material 3.

| Gói | Phiên bản | Ghi chú |
|---|---|---|
| `freezed` / `freezed_annotation` | 4.0.3 / 3.1.0 | Lớp có `factory` phải khai `abstract` (một biến thể) hoặc `sealed` (union) |
| `json_serializable` / `json_annotation` | 6.15.2 / 4.12.0 | `checked: true` trong `build.yaml` |
| `riverpod_generator` / `riverpod_annotation` | 4.0.10 / 4.0.8 | |
| `flutter_riverpod` | 3.5.0 | `riverpod_annotation` 4.0.8 ghim `riverpod` đúng 3.5.0, nên `flutter_riverpod` phải ở 3.5.0 (khung chốt 3.4) |
| `build_runner` | 2.17.1 | |
| `analyzer` (gián tiếp) | 14.7.0 | pub giải được ngay, không phải ghim tay |
| `dio`, `flutter_lints`, `shelf` | 5.11.1, 6.0.0, 1.4.3 | |
| `toi_noi_core`, `toi_noi_api`, `toi_noi_fake_api` | path | Không sửa package chung |

## Chưa làm

- Chưa chạy trên simulator, emulator hay thiết bị; chưa có `android/`, `ios/`.
- Chưa sinh `QuoteRequestDto` và `TripSnapshot` (giữ bản viết tay của `toi_noi_api`) và chưa thay DTO trong `toi_noi_api` bằng bản sinh mã: package chung không đổi, lab chỉ chứng minh hai bản tương đương.
- Converter lặp lại quy tắc của `JsonReader` (viết tay) thay vì dùng chung code, vì `JsonReader` không nằm trong API công khai của `toi_noi_api`.
- `build_runner` mặc định quét cả `test/` (không sinh gì ở đó); chưa tinh chỉnh `generate_for` để chỉ quét `lib/`.
- Chưa dùng `riverpod_lint` (bài 32) và chưa có CI kiểm file sinh (bài 33).

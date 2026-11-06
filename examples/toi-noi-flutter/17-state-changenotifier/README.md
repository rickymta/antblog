# Lab 17: InheritedWidget, ChangeNotifier và ranh giới rebuild

Lab của bài **FL-17** trong loạt **Flutter qua Tới Nơi**: [Quản lý state: InheritedWidget, ChangeNotifier và ranh giới rebuild](../../../api/seed/content/flutter-state-inheritedwidget-changenotifier-bao-gia.md).

Bài 16 giữ `_trips` và `_replace` trong `State` của khung app rồi truyền callback qua nhiều tầng. Lab này đưa hai thứ đó ra khỏi `State`: `QuoteDraft` (dịch vụ, km, cước tính bằng `calculateFare`) và `TripBook` (danh sách chuyến), mỗi thứ là một `ChangeNotifier` đặt xuống cây bằng `InheritedNotifier`. Test đếm số lần build để chứng minh ranh giới rebuild: widget chỉ nghe cước không dựng lại khi đổi dịch vụ, và ca đối chứng khi truyền cả notifier thì dựng lại. Dùng [`toi_noi_core`](../packages/toi_noi_core) và [`toi_noi_ui`](../packages/toi_noi_ui) (theme, `MoneyText`, `TripStatusChip`) qua path dependency. Mọi con số cước do `calculateFare` tính; bảng giá là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/quote_draft.dart` | `QuoteDraft` (`ChangeNotifier`): dịch vụ, km, cước; `fareListenable` là `ValueListenable<Money>` |
| `lib/quote_scope.dart` | `QuoteScope` (`InheritedNotifier<QuoteDraft>`) với `of`, `maybeOf` (đăng ký phụ thuộc) và `read` (không đăng ký) |
| `lib/snapshot_scope.dart` | `QuoteSnapshotScope`: `InheritedWidget` viết tay với `updateShouldNotify`, và `QuoteSnapshotHost` giữ phần có state |
| `lib/fare_readers.dart` | Ba cách đọc cước (`FareFromScope`, `FareFromValue`, `FareFromWholeNotifier`) và `BuildCounts` để đếm build |
| `lib/trip_book.dart` | `TripBook` (`ChangeNotifier`) thay `_trips` và `_replace`; `TripBookScope` |
| `lib/quote_screen.dart`, `lib/history_screen.dart`, `lib/trip_detail_page.dart` | Màn hình đọc state từ scope, không nhận `trips` hay `onChanged` qua constructor |
| `lib/home_shell.dart`, `lib/app.dart`, `lib/main.dart`, `lib/sample_trips.dart` | Khung hai tab (tab đang mở vẫn là `setState`); `ToiNoiApp` sở hữu và huỷ hai notifier; `main()`; chuyến mẫu |
| `test/quote_draft_test.dart` (8) | Thông báo một lần mỗi thay đổi, không báo khi giá trị bằng, miền km, cước không báo khi vẫn 12.000, dispose, `ValueNotifier` sửa tại chỗ |
| `test/trip_book_test.dart` (7) | Đặt chuyến, huỷ, ca lỗi, danh sách chỉ đọc và bản cũ không đổi ngầm |
| `test/snapshot_scope_test.dart` (5) | `updateShouldNotify` đúng, luôn true, chỉ so cước (sai), host nghe và bỏ nghe draft |
| `test/quote_scope_test.dart` (9) | `of`, `maybeOf`, `read`, gộp thông báo, đổi danh tính notifier, vị trí đặt scope, `of` trong `initState`, tạo notifier trong `build` |
| `test/rebuild_boundary_test.dart` (6) | Ma trận số lần build của ba cách đọc cước theo từng loại thay đổi |
| `test/app_test.dart` (8) | Luồng chọn dịch vụ, đặt chuyến, huỷ ở trang chi tiết; widget nào bị dựng lại (`debugOnRebuildDirtyWidget`) |
| `test/support.dart` | `pumpApp`, `Probe` (đếm build), `ObservableDraft` (hỏi còn listener không) |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/17-state-changenotifier
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (43 test). Mức kiểm chứng: **Kiểm chứng tự động**.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_17 --org vn.toinoi .
flutter run -d <id>
```

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3. Lab không thêm package state nào: `provider`, `flutter_bloc` và `flutter_riverpod` chỉ được nhắc trong bài. Constructor dùng tham số tên private (`this._service`), cần Dart 3.12 trở lên.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật; số lần build đo trong test (debug), không phải số đo hiệu năng.
- Chưa có `InheritedModel` (đăng ký theo "khía cạnh"), chưa có provider, bloc hay Riverpod: Riverpod bắt đầu ở bài 18.
- State chỉ nằm trong bộ nhớ: không lưu cục bộ (bài 22), không gọi mạng (bài 19), không xử lý lỗi bất đồng bộ (bài 20).
- Chưa có accessibility, responsive và adaptive của bài 16 (màn hình trong lab này là bản rút gọn để tập trung vào state).

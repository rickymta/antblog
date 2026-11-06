# Lab 29: Golden test và Semantics test cho thẻ chuyến Tới Nơi

Lab của bài **FL-29** trong loạt **Flutter qua Tới Nơi**: [Golden test và Semantics test: khoá giao diện và accessibility](../../../api/seed/content/flutter-golden-test-semantics-test-giao-dien-on-dinh.md).

Lab khoá giao diện của `TripCard` (thẻ chuyến) và `TripStatusChip` (chip của `toi_noi_ui`) bằng ảnh chuẩn (golden) ở giao diện sáng và tối, kích thước cố định, và khoá cây semantics bằng `meetsGuideline` cùng `isSemantics`. Phần khó là golden phụ thuộc hệ điều hành: lab có comparator với ngưỡng nhỏ, tag `golden`, và cách chạy rõ ràng cho CI. Mọi con số cước do `calculateFare` tính (5 km là 25.500 VND); bảng giá là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/trip_card.dart` | `TripCard`: thân thẻ là một nút đọc trọn câu (`spokenTrip`), nút ẩn chuyến riêng có `tooltip` |
| `lib/broken_cards.dart` | Ca sai có chủ đích: icon 20 không nhãn, `IconButton` quên `tooltip`, nhãn trạng thái nhạt |
| `lib/sample_trips.dart` | `tripAt`: dựng chuyến bằng `Trip.request` rồi `apply` từng sự kiện |
| `test/flutter_test_config.dart` | Chạy trước mọi file test: nạp font icon thật, cài `TolerantGoldenComparator` |
| `test/support/tolerant_comparator.dart` | `measurePixelDiff` và `TolerantGoldenComparator` (ngưỡng kênh 4, ngưỡng pixel 0,1%) |
| `test/support/golden_harness.dart` | `pumpGolden`, `setGoldenView`, `capturePng`, `loadMaterialIcons`, `goldenSkip` |
| `test/support/sample_cards.dart` | Bốn thẻ mẫu, một thẻ, tám chip |
| `test/golden_trip_card_test.dart` | 5 golden (tag `golden`): thẻ sáng, tối, chữ gấp đôi; chip sáng, tối |
| `test/golden_proof_test.dart` | 6 test (tag `golden`): comparator bắt thẻ đổi trạng thái, mất nút; `takeException`; ngưỡng 0,5% nuốt lỗi icon |
| `test/comparator_test.dart` | 8 test: logic hai ngưỡng trên ảnh tổng hợp, bốn ảnh trong `failures/`, khác kích thước |
| `test/golden_setup_test.dart` | 9 test: cửa sổ 390 x 844, tắt bóng, font kiểm thử mặc định, đổi chữ ASCII cùng độ dài không đổi ảnh, thêm dấu thì đổi |
| `test/semantics_test.dart` | 23 test: nhãn đọc, nút ẩn chuyến, ba guideline, contrast tám trạng thái sáng/tối, ca sai |
| `test/goldens/*.png` | Ảnh chuẩn đã commit (sinh trên macOS) |

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/29-golden-semantics
flutter pub get
flutter analyze
flutter test                          # mặc định: ổn định trên macOS và Linux
flutter test --exclude-tags golden    # lệnh dành cho CI ubuntu
flutter test --tags golden            # chỉ golden, chạy ở máy khớp hệ điều hành baseline
```

Kỳ vọng: `No issues found!` và `All tests passed!` (51 test trên macOS; trên máy khác macOS, 11 test golden hiện là bỏ qua kèm lý do, còn 40 test chạy).

**Vì sao `flutter test` mặc định ổn định ở mọi hệ điều hành.** Ảnh chuẩn sinh trên macOS (xem `goldenBaselineOs` trong `test/support/golden_harness.dart`). Trên hệ điều hành khác, mọi test đụng tới ảnh chuẩn bị `skip` kèm lý do thay vì so với ảnh của máy khác. Test không đụng ảnh chuẩn (semantics, comparator trên ảnh tổng hợp, giả định về font) chạy ở mọi nơi.

**Chưa chạy trên Linux.** Lab chưa được chạy trên Linux (máy tác giả là macOS arm64). Do đó chưa biết ảnh sinh trên macOS lệch Linux bao nhiêu, và ngưỡng 0,1% chưa được hiệu chỉnh bằng số liệu hai hệ điều hành. Cách làm an toàn đã chọn: CI ubuntu chạy `flutter test --exclude-tags golden`, golden chạy ở job macOS, hoặc sinh lại ảnh chuẩn trên Linux (đổi `goldenBaselineOs` sang `linux` cùng commit) rồi chạy golden ở job ubuntu.

## Cập nhật ảnh chuẩn có chủ đích

```bash
flutter test --update-goldens --tags golden
git diff --stat test/goldens        # chỉ những ảnh dự định đổi mới được đổi
```

Mở từng ảnh đổi, đối chiếu với thay đổi giao diện trong cùng PR rồi mới commit. Khi golden trượt, `flutter test` ghi bốn ảnh vào `test/failures/` (`*_masterImage.png`, `*_testImage.png`, `*_isolatedDiff.png`, `*_maskedDiff.png`); thư mục này nằm trong `.gitignore`, không commit.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Lab này là test, không có `main()`; nếu muốn xem thẻ trên máy thì sinh runner rồi tự viết `main` dựng `TripCard`:

```bash
flutter create --platforms=ios,android --project-name toi_noi_29 --org vn.toinoi .
```

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3. Kiểm trên macOS (Apple silicon).

## Chưa làm

- Chưa chạy trên Linux và chưa chạy trên runner macOS của CI; chưa có số liệu lệch giữa hai hệ điều hành.
- Chữ trong ảnh là font kiểm thử mặc định của `flutter test` (tên `FlutterTest`; tài liệu `matchesGoldenFile` vẫn ghi là Ahem), không phải font thật: nhiều ký tự là ô vuông đặc, một số ký tự tiếng Việt là ô rỗng, nên golden chỉ thấy đổi chữ khi hình ô đổi (đổi chữ ASCII sang chữ ASCII cùng độ dài thì không đổi ảnh), và bố cục khác máy thật. Lab không commit file font nào; chỉ nạp font icon của Material từ asset do `uses-material-design: true` sinh ra.
- Chưa chạy trên simulator hay máy thật, chưa nghe bằng VoiceOver/TalkBack.
- Chưa có workflow GitHub Actions (bài 33).

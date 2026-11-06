# Lab 27: unit test với mocktail và fake

Lab của bài **FL-27** trong loạt **Flutter qua Tới Nơi**: [Unit test với mocktail và fake: quy tắc cước và use case](../../../api/seed/content/flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case.md).

Project Dart thuần (không Flutter). Ba việc: kiểm quy tắc cước của `toi_noi_core` bằng phân vùng tương đương, giá trị biên và vòng lặp; kiểm máy trạng thái chuyến bằng cách lặp đủ 56 cặp (trạng thái, sự kiện); kiểm use case `RequestTrip` (bản rút gọn của lab 21) bằng dummy, stub, spy, mock (mocktail) và fake. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/booking/` | Domain rút gọn của luồng đặt chuyến: `QuoteRequest`, `TripFailure` (`Unreachable`, `ServiceFault`, `Rejected`), `TripRepository`, `BookingLog`, `RequestTrip`. Chỉ import `toi_noi_core` |
| `test/support/doubles.dart` | Dummy (`DummyTripRepository`, `DummyBookingLog`), spy (`SpyBookingLog`), fake (`InMemoryTripRepository`), mock (`MockTripRepository`, `MockBookingLog`) và `registerFallbacks()` |
| `test/support/fare_cases.dart` | Bảng ca cước (đại diện và biên), `fareFailures`, `oracleFareVnd` |
| `test/support/transition_table.dart` | Bảng 9 cặp hợp lệ, `tripIn`, `transitionFailures` |
| `test/fare_rule_test.dart` | 18 test: ca đại diện, ca biên (0, 2, 2,001, 100, ngoài miền, NaN, vô cực), mọi mét từ 0 đến 100.000 so với phép tính độc lập |
| `test/fare_mutants_test.dart` | 6 test: cài 5 lỗi vào công thức, đếm bộ ca nào bắt được |
| `test/trip_state_machine_test.dart` | 64 test: 56 cặp sinh bằng vòng lặp, bảng đủ cỡ, hai lỗi cài vào |
| `test/request_trip_fake_test.dart` | 13 test use case với fake, spy và dummy |
| `test/request_trip_mock_test.dart` | 8 test use case với mock: `when`, `thenAnswer`, `verify`, `verifyNever`, `any(named:)`, `captureAny` |
| `test/in_memory_repository_test.dart` | 7 test cho chính fake |
| `test/mocktail_traps_test.dart` | 7 test: `thenThrow` với `Future`, phương thức chưa stub, `any()` thiếu `named:`, verify hai lần |
| `test/missing_fallback_test.dart` | 2 test: thiếu `registerFallbackValue` cho ra `StateError` thật |
| `test/refactor_brittle_test.dart` | 6 test: cùng một refactor làm gãy test viết kiểu mock mà không làm gãy test viết kiểu fake |

## Kiểm chứng (không cần simulator, không cần mạng)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/27-unit-test-mocktail
dart pub get
dart analyze
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (131 test). Chạy được cả khi đảo thứ tự: `dart test --test-randomize-ordering-seed=random`. Mức kiểm chứng: **Kiểm chứng tự động**.

## Chạy trên máy

Lab không có app để chạy. Không cần thư mục `android/`, `ios/`; nếu vẫn muốn sinh runner: `flutter create --platforms=ios,android --project-name toi_noi_27 --org vn.toinoi .`.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `test` 1.33.0, `mocktail` 1.0.5, `lints` 6.1.0, `toi_noi_core` 0.1.0 (path).

## Chưa làm

- `RequestTrip` ở lab là bản rút gọn của lab 21 (thêm `BookingLog` để có thứ mà mock phải canh); chưa nối vào app.
- Chưa có contract test chạy chung trên fake và repository thật; fake `InMemoryTripRepository` chỉ được kiểm bằng test của chính nó.
- Chưa đo coverage (bài 33), chưa có widget test (bài 28), chưa kiểm Riverpod, async và Stream (bài 30).
- Chưa chạy trên thiết bị: lab thuần Dart nên không cần.

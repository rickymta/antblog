# Lab 21: Clean Architecture cho luồng đặt chuyến

Lab của bài **FL-21** trong loạt **Flutter qua Tới Nơi**: [Clean Architecture cho mobile: domain, data, presentation và repository](../../../api/seed/content/flutter-clean-architecture-domain-data-presentation-mobile.md).

Cùng màn hình báo giá và đặt chuyến của lab 19 và 20, nhưng code được chia ba lớp với một chiều phụ thuộc: `presentation` và `data` cùng trỏ vào `domain`, còn `domain` không biết ai. Domain định nghĩa cổng `TripRepository`, lỗi `TripFailure` và use case `RequestTrip`; data thực thi cổng bằng `ToiNoiApiClient` của [`toi_noi_api`](../packages/toi_noi_api); presentation chỉ thấy domain. Một test đọc source của từng lớp và khẳng định chúng không import nhầm. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ; mọi con số cước do `calculateFare` của `toi_noi_core` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/domain/` | `QuoteRequest` (yêu cầu của khách, có `isWithinRange` và `requireWithinRange` dùng chung cho báo giá và đặt chuyến), `TripFailure` (sealed: `Unreachable`, `ServiceFault`, `Rejected`, `Unreadable`), `TripRepository` (interface), `RequestTrip` (use case), `domain.dart` (barrel). Chỉ import `toi_noi_core` |
| `lib/data/` | `TripRepositoryImpl` (gọi `ToiNoiApiClient`, bọc `guardApi`), `mappers.dart` (`QuoteRequest` sang DTO, `ApiFailure` sang `TripFailure`). Import `toi_noi_api`, không import Flutter |
| `lib/presentation/` | `providers.dart` (chỗ cắm `tripRepositoryProvider`, `requestTripProvider`, `QuoteController`, `BookingController`), `failure_text.dart`, `failure_view.dart`, `quote_screen.dart`, `app.dart`. Không import dio, `toi_noi_api` hay `data/` |
| `lib/composition.dart`, `lib/main.dart` | Composition root: nơi duy nhất nối ba lớp; ghi đè chỗ cắm bằng `TripRepositoryImpl` thật |
| `test/architecture_test.dart` | 15 test: đọc file trong `lib/domain`, `lib/data`, `lib/presentation` và kiểm import; kèm 8 test cho chính bộ kiểm (bắt `import`, `export`, import tương đối, `dart:io`; bỏ qua dòng chú thích) |
| `test/request_trip_test.dart` | 12 test use case với repository giả: biên 0 và 100 km, NaN và vô cực, `intentId` dùng lại sau lỗi, đổi yêu cầu, hai lần bấm đồng thời |
| `test/trip_repository_impl_test.dart` | 19 test: DTO sang `Trip` và `Money`, từng loại lỗi sang `TripFailure`, lỗi lạ không bị che (lên dưới dạng `DioException` kiểu `unknown` bọc `StateError`) |
| `test/booking_flow_test.dart` | 4 test: use case + data thật + fake API, mất response vẫn một chuyến; một test ghi lại giới hạn (đổi yêu cầu sau khi mất response thì có hai chuyến) |
| `test/controllers_test.dart`, `test/failure_text_test.dart`, `test/quote_screen_test.dart` | 9 + 4 + 6 test: provider, câu chữ lỗi, màn hình với repository giả và với data thật |
| `test/support.dart` | `FakeTripRepository` (thuần Dart), `Harness` (fake API + `FaultyAdapter` + `TripRepositoryImpl` thật, không có `RetryInterceptor`) |

## Kiểm chứng (không cần simulator, không cần mạng)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/21-clean-architecture
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (69 test). Mức kiểm chứng: **Kiểm chứng tự động**.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần:

```bash
cd examples/toi-noi-flutter/21-clean-architecture
flutter create --platforms=ios,android --project-name toi_noi_21 --org vn.toinoi .
```

Mở một terminal chạy fake API, terminal khác chạy app:

```bash
cd examples/toi-noi-flutter/tools/fake_api && dart pub get && dart run bin/server.dart
cd examples/toi-noi-flutter/21-clean-architecture && flutter run -d <id>
# Android emulator: máy dev nằm ở 10.0.2.2
flutter run -d <id> --dart-define=API_BASE_URL=http://10.0.2.2:8787
```

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `dio` 5.11.1 (qua `toi_noi_api` 0.2.0; thêm làm dev_dependency để test kiểu lỗi), `shelf` 1.4.3, `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0, `toi_noi_ui` 0.1.0, `toi_noi_fake_api` 0.1.0 (path), Material 3.

## Lab bỏ gì, giữ gì

Giữ: ba lớp, cổng `TripRepository` do domain sở hữu, lỗi của domain, **một** use case (`RequestTrip`, vì có hai quy tắc thật), composition root, test chiều phụ thuộc.

Bỏ có chủ ý:

- Không có use case cho báo giá và làm mới chuyến: quy tắc miền nằm ở `QuoteRequest`, còn lại chỉ là một lời gọi repository, controller gọi thẳng.
- Không có lớp cơ sở `UseCase<T, P>`, không có `Either`/`Result`: lỗi là `TripFailure` được ném, cùng cách `AsyncValue.guard` đang dùng.
- Không có entity riêng cho báo giá: dùng `Money` của `toi_noi_core`. Không có mapper `TripDto` sang `Trip`: `TripDto.toDomain` đã nằm trong `toi_noi_api`.
- Không có interface riêng cho từng nguồn dữ liệu (local, remote): chỉ có một nguồn tới bài 22.
- Không chia theo feature (`features/booking/...`): bài 44.
- Không có connectivity, nút Mở cài đặt, banner mất mạng của lab 20; không có `RetryInterceptor` trong test của repository (đã kiểm ở bài 20), chỉ trong `composition.dart`.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật (không có SDK Android và Xcode đầy đủ trên máy kiểm).
- Không test nào chạy `productionTripRepository` (composition root chưa được kiểm).
- `RequestTrip` giữ ý định trong bộ nhớ và chỉ nhớ một yêu cầu: mất response rồi đổi yêu cầu thì server có hai chuyến, quay lại yêu cầu đầu thành ba; tắt app thì mất khoá. Lưu khoá bền: bài 46.
- Bộ kiểm import đọc dòng `import`/`export` bằng biểu thức chính quy, không dùng analyzer: một directive viết URI ở dòng khác với từ khoá, hoặc import có điều kiện (`if (dart.library.io) 'package:dio/dio.dart'`), sẽ lọt. Lint theo lớp bằng công cụ chuyên dụng: bài 32.
- Chưa có lưu cục bộ (bài 22), điều hướng (bài 23), xác thực (bài 24), cấu hình theo flavor (bài 25), sinh mã (bài 26).

# Lab 23: go_router, deep link, ShellRoute và guard đăng nhập

Lab của bài **FL-23** trong loạt **Flutter qua Tới Nơi**: [go_router: deep link, ShellRoute, guard đăng nhập](../../../api/seed/content/flutter-go-router-deep-link-shellroute-guard.md).

App khách của Tới Nơi có ba tab (Đặt xe, Lịch sử, Tài khoản), màn hình chuyến `/trip/:id` toàn màn hình, màn hình đăng nhập và màn hình "không tìm thấy". Điều hướng bằng `go_router`; trạng thái đăng nhập nằm trong Riverpod và được nối vào router bằng `refreshListenable`. Link `toinoi://trip/{id}` và `https://toinoi.example/trip/{id}` mở thẳng chuyến. Dùng [`toi_noi_core`](../packages/toi_noi_core) (`Trip`, `Money`, `calculateFare`) và [`toi_noi_ui`](../packages/toi_noi_ui) (theme, `MoneyText`, `TripStatusChip`). Bảng giá là minh hoạ; mọi con số cước do `calculateFare` tính. Tên miền `toinoi.example`, Team ID `ABCDE12345` và vân tay chứng chỉ trong `native-config/` là giá trị hư cấu.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/paths.dart` | `AppPaths`: hằng đường dẫn, dựng `/trip/{id}`, `/login?from=...`, `/not-found?uri=...` |
| `lib/deep_links.dart` | `DeepLinks`: scheme, tên miền, `resolve(Uri)` đổi link vào app thành đường dẫn nội bộ |
| `lib/guard.dart` | `guardLocation` (hàm `redirect` thuần), `safeFrom`, `requiresLogin` |
| `lib/auth.dart` | `Session`, `SessionStore` (bản bộ nhớ có độ trễ và lỗi giả), `AuthController` (`AsyncNotifier`), `authProvider` |
| `lib/trips.dart` | `TripRepository`, `FakeTripRepository` (bốn chuyến mẫu), `tripListProvider`, `tripByIdProvider`, `TripNotFound` |
| `lib/router.dart` | `buildRoutes` (`StatefulShellRoute.indexedStack`, `/trip/:id`, `/login`, `/not-found`), `routerProvider`, `AuthRefresh` |
| `lib/shell_scaffold.dart` | `NavigationBar` + `StatefulNavigationShell.goBranch` |
| `lib/booking_screen.dart`, `history_screen.dart`, `account_screen.dart`, `trip_screen.dart`, `login_screen.dart`, `not_found_screen.dart` | Các màn hình |
| `native-config/` | Bản mẫu cấu hình native: `android/AndroidManifest.xml.example`, `ios/Info.plist.example`, `ios/Runner.entitlements.example`, `web/.well-known/apple-app-site-association.example`, `web/.well-known/assetlinks.json.example` |
| `test/guard_test.dart` | 12 test: `requiresLogin`, `safeFrom`, `guardLocation` |
| `test/deep_links_test.dart` | 8 test: `DeepLinks.resolve` cho custom scheme, universal link, host lạ |
| `test/router_guard_test.dart` | 15 test: chặn và quay lại `from`, đăng xuất (cả khi trang riêng đang hiện từ push), kho phiên đọc chậm hoặc hỏng, redirect async |
| `test/shell_test.dart` | 7 test: ba tab, state còn nguyên khi đổi tab, `push` giữ state còn `go` làm mất |
| `test/trip_route_test.dart` | 11 test: `/trip/:id`, id không tồn tại, nút Back, `onException` |
| `test/deep_link_flow_test.dart` | 15 test: khởi động nguội kiểu Android bằng `defaultRouteName`, nhận link bằng thông điệp `pushRouteInformation` (đường của iOS và của app đang chạy) |
| `test/pitfalls_test.dart` | 8 test: bốn lỗi thường gặp (quên ghép custom scheme, quên `refreshListenable`, `watch` trong provider router, `.future` cũ) |
| `test/native_config_test.dart` | 14 test: đọc các file trong `native-config/` và kiểm giá trị, nhất quán chéo với code Dart |
| `test/support.dart` | `pumpApp`, `receiveLink`, `launchWithLink`, `locationOf` |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/23-go-router
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (90 test). Mức kiểm chứng: **Kiểm chứng tự động**. Test không gọi mạng và không cần thiết bị.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_23 --org vn.toinoi .
flutter run -d <id>
```

Chạy xong thì ghép các đoạn trong `native-config/` vào file tương ứng của runner vừa sinh (xem chú thích đầu mỗi file). Các file `.example` không tự có hiệu lực. Với `flutter create --org vn.toinoi` và tên project `toi_noi_23`, application ID mặc định là `vn.toinoi.toi_noi_23`; đổi thành `vn.toinoi.customer` nếu muốn khớp `assetlinks.json` mẫu.

Thử custom scheme khi app đã chạy:

```bash
# Android (cần adb)
adb shell am start -a android.intent.action.VIEW -d "toinoi://trip/T-1002"
# iOS Simulator
xcrun simctl openurl booted "toinoi://trip/T-1002"
```

Bản mặc định của `main()` khởi động chưa đăng nhập: bấm "Đăng nhập" rồi mới thấy lịch sử. Kho phiên là bản trong bộ nhớ nên mỗi lần mở app lại là chưa đăng nhập.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `go_router` 18.1.3, `flutter_riverpod` 3.5.0 (`riverpod` 3.5.0; bài 18 kiểm ở 3.4.3), `xml` 7.1.0 (dev, chỉ để test đọc file cấu hình), `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3. `go_router` 18 kéo thêm `material_ui` và `cupertino_ui` làm dependency.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị: chưa kiểm hệ điều hành có mở app khi bấm link, chưa kiểm intent-filter và entitlement được chấp nhận, chưa kiểm xác minh App Link / Universal Link (cần tên miền thật, `assetlinks.json` và `apple-app-site-association` đặt trên server, chứng chỉ ký thật).
- Test mô phỏng đúng thông điệp `flutter/navigation` mà engine gửi và `defaultRouteName` của khởi động nguội trên Android, nhưng phần engine gửi gì chỉ được xác nhận bằng cách đọc mã nguồn engine 3.47.4 (Android: `Intent.getData().toString()`; iOS: `url.absoluteString`, gửi sau khung hình đầu tiên), không bằng chạy trên máy.
- Đăng nhập là giả (chỉ nhập tên); OTP, JWT, refresh token là bài 24. Kho phiên chưa dùng `flutter_secure_storage` (bài 22).
- Chưa có restoration state, `onEnter`, route có kiểu (`go_router_builder`), animation chuyển trang tuỳ biến.
- Ba flavor `dev`/`staging`/`prod` (bài 25): `native-config/` mới chỉ mô tả app `prod` (`vn.toinoi.customer`).

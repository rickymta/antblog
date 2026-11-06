# Lab 24: đăng nhập OTP, JWT, refresh token rotation và interceptor

Lab của bài **FL-24** trong loạt **Flutter qua Tới Nơi**: [Authentication: OTP, JWT, refresh token rotation và interceptor](../../../api/seed/content/flutter-authentication-otp-jwt-refresh-token-secure-storage.md).

App khách của Tới Nơi đăng nhập bằng số điện thoại và mã OTP, giữ cặp access/refresh token trong kho an toàn (`TokenStore` sao từ [lab 22](../22-persistence-drift)), gắn `Authorization: Bearer` vào mọi request bằng `AuthInterceptor`, và khi gặp 401 thì refresh **một lần** cho mọi request đang chờ rồi phát lại. Chạy trên `tools/fake_api` với `AuthModule` (mới ở bài này). Dùng [`toi_noi_core`](../packages/toi_noi_core), [`toi_noi_api`](../packages/toi_noi_api) và [`toi_noi_ui`](../packages/toi_noi_ui). Bảng giá là minh hoạ; mọi con số cước do `calculateFare` tính (5 km là 25.500 VND).

**Đây là đồ giả để học.** Mã OTP của fake là hằng `123456`; access token là JWT không ký (`alg: none`); số điện thoại trong lab (`+84000000001`) là số giả không có thật. Server thật sinh OTP ngẫu nhiên, gửi qua kênh riêng và ký token.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/auth/token_store.dart` | `AuthTokens`, `TokenStore`, `SecureTokenStore`, `InMemoryTokenStore` (sao từ lab 22, không đổi hành vi) |
| `lib/auth/unverified_claims.dart` | Đọc `sub`, `role`, `exp` từ payload JWT **không kiểm chữ ký**, chỉ để hiển thị |
| `lib/auth/auth_api.dart` | `AuthApi`: `requestOtp`, `verifyOtp`, `refresh`, `logout` trên một `Dio` riêng (không `AuthInterceptor`, không `RetryInterceptor`) |
| `lib/auth/session_manager.dart` | `SessionManager`: giữ token, một lần refresh dùng chung (`Future` chia sẻ), `epoch` phiên, lưu bộ nhớ rồi kho (kiểm lại `epoch` sau khi ghi, `onStoreError`), đăng xuất cục bộ trước rồi báo server |
| `lib/auth/auth_interceptor.dart` | `AuthInterceptor`: gắn Bearer và ghi `epoch` vào request, bắt 401, refresh qua `SessionManager`, phát lại đúng một lần (không phát lại 401 của phiên cũ) |
| `lib/auth/auth_controller.dart` | `AuthController` (`AsyncNotifier`) và `AuthSession` (chỉ để hiển thị) |
| `lib/providers.dart` | Dựng client: log, `AuthInterceptor`, `RetryInterceptor`; `BookingController` |
| `lib/login_screen.dart`, `home_screen.dart`, `app.dart`, `main.dart` | Đăng nhập hai bước, màn hình chính (đặt chuyến mẫu 5 km, đăng xuất), `AuthGate` |
| `test/` | Xem mục kiểm chứng |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/24-auth-jwt-refresh
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (79 test), cộng `cd examples/toi-noi-flutter/tools/fake_api && dart test` (91 test, trong đó 28 test mới của `AuthModule`). Mức kiểm chứng: **Kiểm chứng tự động**. Test chạy trên fake API trong tiến trình (`InProcessAdapter`), đồng hồ của server là biến trong test (`Rig.advance`), không gọi mạng, không cần thiết bị.

| Nhóm test | Chứng minh |
|---|---|
| `auth_interceptor_test.dart` | Gắn Bearer; 5 request song song cùng 401 chỉ refresh 1 lần; 401 về muộn không refresh lần hai; 401 của phiên cũ không bị phát lại bằng token phiên mới; phát lại giữ body và `Idempotency-Key`; phát lại vẫn 401 thì dừng; refresh bị từ chối thì đăng xuất; refresh gặp 5xx, mất mạng, 429 thì giữ phiên; thứ tự với `RetryInterceptor` (cả hai chiều); log không lộ token |
| `session_manager_test.dart` | Đăng xuất xoá kho (cả `InMemoryTokenStore` lẫn `SecureTokenStore` giả), kể cả khi mất mạng; refresh về muộn sau đăng xuất bị bỏ; đăng xuất hay đăng nhập lại đúng lúc đang ghi kho thì kho vẫn đúng; ghi kho lỗi không làm mất cặp mới; mất response refresh thì lần Thử lại bị đăng xuất (giới hạn đã biết); 401/403 kết thúc phiên còn 400/422/429 thì không |
| `pitfalls_test.dart` | Refresh riêng từng request bị rotation coi là kẻ trộm và đăng xuất oan; `QueuedInterceptor` refresh qua chính `Dio` bị treo, và vẫn treo nếu refresh trên `Dio` riêng mà phát lại qua `Dio` đó; thứ tự `onRequest`/`onError` của dio |
| `auth_api_test.dart`, `unverified_claims_test.dart` | Lỗi 422/429/401 thành `ClientFailure`; body sai hợp đồng thành `ParseFailure`; payload bị sửa vẫn đọc ra (nên client không được tin) |
| `app_test.dart` | Luồng đăng nhập trên giao diện, đặt chuyến khi access token hết hạn, đăng xuất, mở lại app, phiên bị thu hồi giữa chừng |

## Chạy trên máy (tự thử)

Bật server giả có xác thực (cửa sổ terminal riêng):

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart run bin/server.dart --with-auth
```

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
cd examples/toi-noi-flutter/24-auth-jwt-refresh
flutter create --platforms=ios,android --project-name toi_noi_24 --org vn.toinoi .
flutter run -d <id> --dart-define=API_BASE_URL=http://10.0.2.2:8787   # Android emulator
flutter run -d <id>                                                   # iOS Simulator (localhost:8787)
```

Gửi OTP, nhập `123456`, đặt chuyến mẫu: hiện 25.500 VND. Để thấy refresh: chờ 15 phút (hạn access token của fake) rồi đặt chuyến lần nữa, hoặc khởi động fake với `AuthModule(accessTokenTtl: Duration(seconds: 20))` bằng cách sửa `bin/server.dart` cục bộ. Android emulator gọi máy dev qua `10.0.2.2`; HTTP không mã hoá trên thiết bị thật cần cấu hình cleartext riêng (bài 25 và Phần 7).

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `dio` 5.11.1, `flutter_riverpod` 3.5.0, `flutter_secure_storage` 11.2.0, `flutter_lints` 6.0.0, `toi_noi_core`, `toi_noi_api` 0.2.0 và `toi_noi_ui` (path), `toi_noi_fake_api` 0.2.0 (path, dev), Material 3.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị: Keychain, Android Keystore và HTTP thật chưa được kiểm trong lab này. Test dùng kho trong bộ nhớ và nền tảng `flutter_secure_storage` giả.
- Fake API là đồ giả: OTP cố định, JWT không ký, không giới hạn tốc độ gửi OTP, không gắn refresh token với thiết bị.
- Mất response của lần refresh (server đã xoay token, client không nhận được) làm refresh token cũ thành "đã dùng": lần refresh sau bị coi là dùng lại và cả họ bị thu hồi. Server thật thường cho một khoảng ân hạn ngắn; lab và fake không có.
- Chưa refresh chủ động trước khi hết hạn (đọc `exp`), chưa có đăng nhập sinh trắc học, chưa chống chạy hai tiến trình app cùng lúc (nền và nổi) tranh một refresh token.
- Chưa nối với `go_router` của bài 23 (lab dùng `AuthGate` thay `redirect`); nối chỉ là đưa `signedOut` vào `refreshListenable`.
- Chưa có màn hình cho app tài xế và không có phân quyền theo vai trò: đó là việc của server.

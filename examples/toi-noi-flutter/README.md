# Flutter qua Tới Nơi: ứng dụng đặt xe iOS/Android

Khung biên tập loạt bài Flutter của Tech Knowledge Hub. Ứng dụng xuyên suốt là **Tới Nơi** — app khách và app tài xế viết bằng Flutter, chạy trên iOS và Android từ một mã nguồn. Chương trình đầy đủ 64 bài / 8 phần, điều kiện triển khai và quy trình viết nằm ở [`docs/flutter-master-curriculum.md`](../../docs/flutter-master-curriculum.md). Mỗi bài có câu hỏi mở đầu, hình minh hoạ giải thích cơ chế, ví dụ chạy được, lỗi thường gặp, bài tập có đáp án và nguồn chính thức. Tách rõ code minh hoạ với giải pháp production. Không tạo link cho bài chưa viết.

Đối tượng: người đã dùng được terminal và một editor; không cần biết Dart hay mobile. Phiên bản: **Flutter 3.47.x stable, Dart 3.13.x**, Material 3. Lab bài 01–42 là project nhỏ độc lập; từ bài 43 code dồn vào workspace `toi-noi-app`.

## Phần 1 · Dart cho Tới Nơi (cơ bản)

01. Flutter và Dart: từ source code đến ứng dụng Tới Nơi chạy trên iOS và Android — **đã viết, đã review**
02. Dart cơ bản: biến, kiểu dữ liệu và null safety khi tính cước Tới Nơi — **đã viết, đã review**
03. Hàm, class, enum và sealed class: mô hình trạng thái chuyến bằng Dart — **đã viết, đã review**
04. Collections, generics và extension: quản lý danh sách chuyến và tài xế — **đã viết, đã review**
05. Future, async/await và Stream: gọi báo giá và lắng nghe vị trí tài xế — **đã viết, đã review**
06. Package, pubspec, lint và test: thư viện toi_noi_core kiểm chứng được — **đã viết, đã review**

## Phần 2 · Flutter UI cơ bản (cơ bản → trung bình)

07. Widget, cây widget và hot reload: màn hình báo giá đầu tiên — kế hoạch
08. Layout và constraints: Row, Column, Expanded và thẻ chuyến — kế hoạch
09. ListView, Sliver và cuộn: danh sách lịch sử chuyến — kế hoạch
10. StatefulWidget, lifecycle và setState: chọn dịch vụ và đếm ngược tìm tài xế — kế hoạch
11. Form, TextFormField và validation: nhập điểm đón, điểm đến — kế hoạch
12. Navigator, route và truyền dữ liệu: từ báo giá sang xác nhận chuyến — kế hoạch
13. Theme Material 3, màu, chữ và dark mode: design system Tới Nơi — kế hoạch
14. Asset, intl và localization: hiển thị tiền VND, ngày giờ và song ngữ vi/en — kế hoạch
15. Animation cơ bản: implicit animation và Hero khi tìm tài xế — kế hoạch
16. Accessibility, responsive và adaptive: một giao diện cho iOS và Android — kế hoạch

## Phần 3 · Trạng thái, dữ liệu và kiến trúc (trung bình)

17. Quản lý state: InheritedWidget, ChangeNotifier và ranh giới rebuild — kế hoạch
18. Riverpod: Provider, Notifier và AsyncNotifier cho luồng đặt chuyến — kế hoạch
19. HTTP, dio và JSON: gọi API báo giá của trip-service — kế hoạch
20. Xử lý lỗi, retry, timeout và mất mạng: mạng di động không ổn định — kế hoạch
21. Clean Architecture cho mobile: domain, data, presentation và repository — kế hoạch
22. Lưu trữ cục bộ: drift, shared_preferences và secure storage — kế hoạch
23. go_router: deep link, ShellRoute, guard đăng nhập — kế hoạch
24. Authentication: OTP, JWT, refresh token rotation và interceptor — kế hoạch
25. Cấu hình môi trường: flavors, --dart-define-from-file và DI theo môi trường — kế hoạch
26. Code generation: freezed, json_serializable, riverpod_generator và build_runner — kế hoạch

## Phần 4 · Kiểm thử và chất lượng (trung bình → nâng cao)

27. Unit test với mocktail và fake: quy tắc cước và use case — kế hoạch
28. Widget test: pumpWidget, finder, matcher và màn hình báo giá — kế hoạch
29. Golden test và Semantics test: khoá giao diện và accessibility — kế hoạch
30. Kiểm thử Riverpod, async và Stream: ProviderContainer, fakeAsync, fake clock — kế hoạch
31. Integration test và patrol: luồng đặt chuyến trên simulator/emulator — kế hoạch
32. Lint, analyzer, dart format và custom_lint: chất lượng code có kiểm chứng — kế hoạch
33. CI với GitHub Actions: analyze, test, golden và cache pub — kế hoạch

## Phần 5 · Tính năng di động (trung bình → nâng cao)

34. Bản đồ với flutter_map: marker, polyline và chọn điểm đón — kế hoạch
35. Vị trí: geolocator, quyền foreground/background và cấu hình iOS, Android — kế hoạch
36. Realtime WebSocket: reconnect, backoff và theo dõi tài xế — kế hoạch
37. Push notification: FCM, APNs, local notification và deep link từ thông báo — kế hoạch
38. Tác vụ nền: workmanager, vị trí nền cho app tài xế và chính sách store — kế hoạch
39. Thanh toán qua VíXanh: WebView, deep link quay về và idempotency key — kế hoạch
40. Platform channel, Pigeon và FFI: gọi native khi plugin không đủ — kế hoạch
41. Hiệu năng: rebuild, const, RepaintBoundary, isolate và DevTools — kế hoạch
42. Bảo mật mobile: secure storage, pinning, obfuscation và OWASP MASVS — kế hoạch

## Phần 6 · Ứng dụng hoàn chỉnh: app khách và app tài xế (nâng cao)

43. Monorepo với pub workspaces: packages dùng chung cho app khách và app tài xế — kế hoạch
44. Feature-first + Clean Architecture + Riverpod: cấu trúc feature đặt chuyến — kế hoạch
45. Máy trạng thái chuyến trên client: đồng bộ server, optimistic UI và xác nhận — kế hoạch
46. Offline-first: outbox trên client, idempotency key và đồng bộ khi có mạng — kế hoạch
47. App tài xế: online/offline, nhận cuốc, đếm ngược và điều hướng — kế hoạch
48. Quan sát từ mobile: Crashlytics, Sentry và trace nối với backend — kế hoạch
49. Bài tổng hợp: đặt chuyến end-to-end giữa app khách, app tài xế và trip-service — kế hoạch

## Phần 7 · Build, phân phối và phát hành (nâng cao)

50. Flavors khi build: bundle ID, application ID, icon, tên app và Firebase theo môi trường — kế hoạch
51. Ký ứng dụng Android: keystore, Play App Signing, AAB và versionCode — kế hoạch
52. Ký ứng dụng iOS: Apple Developer Program, certificate, provisioning profile và capabilities — kế hoạch
53. fastlane cho Flutter: match, gym, pilot, supply — kế hoạch
54. CI/CD build: GitHub Actions macOS/ubuntu, secrets, build number và artifact — kế hoạch
55. Firebase App Distribution: phân phối nội bộ cho tester iOS và Android — kế hoạch
56. TestFlight: internal/external testing, Beta App Review và export compliance — kế hoạch
57. Google Play: internal, closed, open testing, production và staged rollout — kế hoạch
58. App Store production: metadata, privacy labels, App Review và phased release — kế hoạch
59. Vận hành sau phát hành: force update, remote config, Shorebird và rollback — kế hoạch

## Phần 8 · Chuyên sâu và tổng kết (nâng cao)

60. CustomPainter và RenderObject: vẽ tuyến đường và đồng hồ cước — kế hoạch
61. Explicit animation, Impeller và shader: hiệu ứng tìm tài xế — kế hoạch
62. Kích thước app, thời gian khởi động và deferred components — kế hoạch
63. Nâng cấp Flutter/Dart, breaking changes và quản lý dependency — kế hoạch
64. Bài tổng hợp: Tới Nơi Flutter từ Dart tới App Store và Google Play — kế hoạch

## Quy ước xuyên suốt

- **Phiên bản**: Flutter 3.47.x stable, Dart 3.13.x; không đặt `sdk` constraint vượt phiên bản đã kiểm. `flutter_lints` mới nhất; `flutter analyze` không còn cảnh báo. Không dùng API đã deprecated (ví dụ `WillPopScope` → `PopScope`, `MaterialStateProperty` → `WidgetStateProperty`).
- **Tiền và đơn vị**: VND là `int` đồng (`fareVnd`), không có phần lẻ; khoảng cách `double distanceKm` ở lớp nhập liệu, `int distanceMeters` khi lưu/truyền; thời điểm UTC (`DateTime.now().toUtc()`), hiển thị theo múi giờ thiết bị.
- **Bảng giá minh hoạ** (giống bài 01–03 lộ trình C#): 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền hợp lệ 0–100 km, ngoài miền ném `ArgumentError`; 2,001 km → 12.005 VND. Dạng chuẩn (từ bài 02 trở đi và trong `toi_noi_core`) tính bằng **mét nguyên** để không phụ thuộc sai số `double`: `extraMeters = max(0, (distanceKm * 1000).round() - 2000)`, `fareVnd = 12000 + (extraMeters * 4500 + 500) ~/ 1000`. Bài 01 dùng phép `double` trực tiếp để giữ đơn giản và chỉ ra rủi ro. Không phải bảng giá thực tế của sản phẩm.
- **Mô hình chung** (chốt từ bài 02–06, dùng lại ở mọi bài sau, tên tiếng Anh):
  - `enum ServiceKind { ride, food, parcel }`
  - `enum TripStatus { requested, searching, accepted, arriving, pickedUp, inTransit, completed, cancelled }` — 8 trạng thái của hồ sơ Tới Nơi; huỷ được ở `searching`, `accepted`, `arriving`.
  - `int calculateFareVnd(double distanceKm)` theo bảng giá minh hoạ.
  - `class Trip` bất biến: `id`, `serviceKind`, `status`, `distanceKm`, `fareVnd`, `driverId` (nullable), `requestedAt` (UTC); chuyển trạng thái trả về bản sao mới, không sửa tại chỗ.
  - `sealed class TripEvent` với 7 biến thể theo đúng 6 mũi tên đường đi bình thường + huỷ: `SearchStarted` (requested → searching), `DriverAccepted(driverId)` (searching → accepted), `DriverArrived` (accepted → arriving), `PickedUp` (arriving → pickedUp), `TripStarted` (pickedUp → inTransit), `Completed` (inTransit → completed), `Cancelled(by)` (từ searching/accepted/arriving → cancelled). Cặp (trạng thái, sự kiện) khác ném `StateError`.
  - `class Money` (bài 06, trong `toi_noi_core`): `const Money(int vnd)` (assert không âm), `Money.fromVnd(int)` kiểm bằng `ArgumentError` cho dữ liệu từ ngoài, `Money.zero`, toán tử `+`, `-`, `*` (nhân int), so sánh `<`/`<=`/`>`/`>=`, `Comparable<Money>`, `==`/`hashCode`, `toString()` dạng "25.500 VND". Từ bài 06 trở đi `Trip.fare` là `Money` (lấy số đồng bằng `trip.fare.vnd`); các lab 02–05 còn dùng `fareVnd: int` vì viết trước khi có package.
  - `toi_noi_core` còn có `FarePolicy` (hằng bảng giá), `calculateFare(double km) → Money`, `Trip.request(id, serviceKind, distanceKm, requestedAt)` tạo chuyến `requested`, `Trip ==` so cả 7 trường và `isSameTrip` so theo id, constructor kiểm `driverId` khớp trạng thái, `enum CancelledBy { customer, driver }`, extension `TripIterableX` (`totalFare`, `byStatus`, `active`). Mọi bài sau import package này thay vì định nghĩa lại.
- **Dữ liệu**: tên khách, tài xế, địa chỉ đều hư cấu; không có key, token, chứng chỉ, file `google-services.json`/`GoogleService-Info.plist` thật trong repo (chỉ có bản mẫu `.example`).
- **Lab**: mỗi thư mục có `README.md` ghi bài tương ứng, lệnh chạy, lệnh kiểm chứng, phiên bản đã kiểm và phạm vi chưa làm. Lab Dart thuần kiểm bằng `dart test`; lab Flutter kiểm bằng `flutter analyze` + `flutter test`. Không commit `build/`, `.dart_tool/`, `pubspec.lock` của lab (lab là ví dụ, không phải app phát hành; lockfile chỉ giữ từ `toi-noi-app`).
- **Bài viết** đặt tại `api/seed/content/<slug>.md`, khuôn theo §8 của master curriculum, ảnh bìa qua `apps/web/src/lib/article-covers.json`.

## Kiến trúc đích đã thống nhất

Hai app Flutter riêng (Customer App, Driver App) trong một workspace, dùng chung `toi_noi_core` (domain), `toi_noi_api` (client, DTO) và `toi_noi_ui` (design system). Trong mỗi app: feature-first, mỗi feature có domain/data/presentation; Riverpod giữ state và là ranh giới giữa presentation và use case; go_router điều hướng; drift + secure storage lưu cục bộ; realtime qua WebSocket tới Realtime Gateway; backend gọi qua BFF (lab dùng `tools/fake_api` hoặc trip-service .NET của bài 31–38 lộ trình C#). Ba flavor `dev`/`staging`/`prod` cố định từ bài 25. Chi tiết: `docs/toi-noi-overall-architecture.md` §3.1, §10, §12.

## Chạy bài 01–06

Cần Flutter 3.47 stable trên PATH. Từ thư mục gốc repository:

```bash
node examples/toi-noi-flutter/verify-01-06.mjs
```

Script chạy `flutter analyze` + `flutter test` cho lab 01 và `dart analyze` + `dart test` cho lab 02–05, `packages/toi_noi_core` và `06-dung-toi-noi-core` (tổng cộng hơn 350 test). Không cần simulator, emulator hay thiết bị; bài 01 mô tả cách chạy app trên iOS Simulator và Android emulator để tự thử.

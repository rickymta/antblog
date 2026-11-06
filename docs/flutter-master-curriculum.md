# Master Curriculum — Flutter qua Tới Nơi: ứng dụng đặt xe iOS/Android

Bản quản lý toàn bộ loạt bài **Flutter** của Tech Knowledge Hub (antblog.xyz). Ứng dụng xuyên suốt là **Tới Nơi** — ứng dụng đặt xe, giao đồ ăn và giao bưu kiện (hồ sơ: `api/seed/content/du-an-mau-toi-noi.md`, kiến trúc: `docs/toi-noi-overall-architecture.md`). Trong loạt này, hai ứng dụng di động của Tới Nơi — **app khách** và **app tài xế** — là ứng dụng **đa nền tảng iOS/Android viết bằng Flutter**, dùng chung các package Dart nội bộ, gọi backend qua API Gateway/BFF và nhận realtime qua Realtime Gateway đúng như hồ sơ kiến trúc §3, §10 và §12.

Loạt gồm **64 bài** chia **8 phần**, mỗi phần tối đa 10 bài, đi từ Dart cơ bản tới phát hành production trên App Store và Google Play. Mức bài: 14 cơ bản · 20 trung bình · 30 nâng cao.

## 1. Mục tiêu, đối tượng và kết quả

| Mục | Nội dung |
|---|---|
| Đối tượng | Người đã dùng được terminal và một editor; không cần biết Dart hay lập trình mobile. Người đã biết React/Swift/Kotlin có thể bắt đầu từ Phần 2 |
| Kết quả cuối | Tự dựng được app khách và app tài xế Tới Nơi bằng Flutter: kiến trúc rõ ràng, có test, gọi backend thật, realtime, bản đồ, push, thanh toán; đóng gói, ký, phân phối nội bộ, TestFlight, Google Play testing và phát hành production có quy trình |
| Liên kết với loạt khác | Backend: dùng lại `examples/toi-noi-dotnet/31-38-trip-service` (POST /trips, GET /trips/{id}, accept) của lộ trình C#/.NET 10; kiểm thử: khung ISTQB của chuyên mục Kiểm thử; design pattern: Strategy (chính sách cước), State (trạng thái chuyến), Adapter (cổng thanh toán) |
| Không nằm trong phạm vi | Flutter web/desktop làm sản phẩm chính (chỉ nhắc ở bài 62 với vai trò công cụ nội bộ); game; thiết kế UI/UX chuyên sâu; backend mới (chỉ có fake API phục vụ lab) |

## 2. Quy ước

| Mục | Quy ước |
|---|---|
| Article ID | `FL-01` … `FL-64`, tăng dần theo thứ tự đọc. Một bài chỉ thuộc một phần |
| Slug | Chữ thường ASCII, gạch nối, ≤ 80 ký tự, bắt đầu bằng `flutter-` (bài Flutter) hoặc `dart-` (bài Dart thuần). Duy nhất trong toàn blog |
| Frontmatter | `category: flutter`, `level: beginner \| intermediate \| advanced`, `tags` luôn có `flutter`, `dart`, `mobile`, `toi-noi`; thêm `ios`, `android`, `release`… theo bài. `summary` bắt đầu bằng `Bài NN:` |
| Lộ trình học | `learning-paths.json` slug `flutter-toi-noi-ios-android`, tên **Flutter qua Tới Nơi: ứng dụng đặt xe iOS/Android**, level `beginner`, category `flutter`. Chỉ liệt kê bài đã viết |
| Phần (Part) | P1 … P8. Mỗi phần có một bài kết thúc gắn kết các bài trong phần (bài lab tổng hợp hoặc bài bản lề sang phần sau) |
| Mức bài | `beginner` (cơ bản): đọc xong chạy được, không cần kiến thức trước ngoài bài trước; `intermediate` (trung bình): kết hợp nhiều cơ chế, cần hiểu trade-off; `advanced` (nâng cao): đụng tới nền tảng, vận hành, nhiều ràng buộc thật của store/OS |
| Thời lượng đọc | 8–14 phút (bài UI có nhiều code hơn bài Dart); bài tổng hợp có thể dài hơn |
| Phiên bản | Flutter **3.47.x stable**, Dart **3.13.x**, Material 3 mặc định. Package pin bằng caret theo phiên bản ở §4; không dùng channel beta/master. Ghi phiên bản đã kiểm trong README lab |
| Tiền và đơn vị | VND là `int` đồng (không có phần lẻ), khoảng cách là `int` mét hoặc `double` km có ghi rõ; thời điểm lưu/truyền UTC ISO-8601, hiển thị theo múi giờ thiết bị. Dữ liệu test giả; không có key, token, chứng chỉ thật trong repo |
| Bảng giá minh hoạ | 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền hợp lệ 0–100 km — cùng quy tắc với bài 01–03 lộ trình C# để hai loạt đối chiếu được nhau |
| Liên kết nội bộ | Chỉ link tới bài đã có trên blog (`/articles/<slug>`). Bài sau chưa viết thì nói "ở bài tiếp theo" không kèm link |

## 3. Môi trường học và điều kiện triển khai

### 3.1 Máy phát triển

| Thành phần | Bắt buộc từ bài | Ghi chú |
|---|---|---|
| Flutter SDK 3.47 stable + Dart 3.13 | 01 | `flutter doctor` phải xanh ở mục Flutter |
| Xcode đầy đủ + iOS Simulator | 07 (chạy iOS) | Chỉ trên macOS. Template Flutter 3.47 dùng Swift Package Manager; CocoaPods chỉ cần khi có plugin chưa hỗ trợ SPM. Không có macOS vẫn học được toàn bộ bằng Android/emulator; các bài iOS (52, 56, 58) cần macOS để build thật |
| Android Studio + Android SDK + emulator, JDK 17 | 07 (chạy Android) | Cần cho `flutter build apk/appbundle` |
| Node ≥ 22 | 06 | Chạy script `verify-*.mjs` của lab |
| .NET 10 SDK + PostgreSQL | 19 (tuỳ chọn) | Chỉ khi muốn gọi trip-service thật thay cho fake API |
| Ruby ≥ 3.1 + fastlane | 53 | Tự động hoá ký và upload |

### 3.2 Môi trường ứng dụng và kênh phân phối

Mỗi app Tới Nơi có ba flavor cố định từ bài 25; kênh phân phối đi theo flavor, không theo nhánh git.

| Flavor | Application ID / Bundle ID | Backend | Kênh phân phối | Ai nhận | Bài |
|---|---|---|---|---|---|
| `dev` | `vn.toinoi.customer.dev` / `vn.toinoi.driver.dev` | fake API local hoặc trip-service local | `flutter run`, simulator/emulator, thiết bị cắm dây | Lập trình viên | 01, 25, 50 |
| `staging` | `vn.toinoi.customer.staging` / `.driver.staging` | API staging | **Firebase App Distribution** (Android APK, iOS ad-hoc) · **TestFlight internal** · **Google Play internal testing** | QA, PM, tester nội bộ | 55, 56, 57 |
| `staging` (beta) | như trên | API staging hoặc production | **TestFlight external** (qua Beta App Review) · **Google Play closed/open testing** | Tester ngoài, tài xế thử nghiệm | 56, 57 |
| `prod` | `vn.toinoi.customer` / `vn.toinoi.driver` | API production | **App Store** (App Review, phased release) · **Google Play production** (staged rollout) | Người dùng | 58, 59 |

Điều kiện cần trước khi vào Phần 7:

- Tài khoản **Apple Developer Program** (trả phí hằng năm; cần tài khoản tổ chức hoặc cá nhân có quyền Admin/App Manager) và máy macOS để ký iOS.
- Tài khoản **Google Play Console** (phí một lần) với quyền Release manager; bật Play App Signing.
- Một **Firebase project** có ba app Android + ba app iOS tương ứng ba flavor (cho App Distribution, FCM, Crashlytics).
- Kho bí mật cho keystore, chứng chỉ, API key của store: GitHub Actions secrets hoặc `match` với repo riêng được mã hoá. Không bao giờ commit các file này.
- Quy trình phiên bản: `version: x.y.z+build` trong `pubspec.yaml`; `build` tăng tự động từ CI; `x.y.z` theo semver do người phát hành quyết.

### 3.3 Mức kiểm chứng của lab

Repo này **không** chứa tài khoản store hay chứng chỉ, nên mỗi bài ghi rõ mức kiểm chứng của lab đi kèm:

| Mức | Ý nghĩa | Áp dụng |
|---|---|---|
| **Kiểm chứng tự động** | `flutter analyze` + `flutter test`/`dart test` chạy xanh trên máy không cần thiết bị; script `verify-*.mjs` và CI ubuntu chạy được | Bài 01–33, 41, 43–49, 60–64 và phần logic của các bài còn lại |
| **Kiểm chứng trên thiết bị** | Cần simulator/emulator hoặc máy thật: integration test, quyền vị trí, push, nền | Bài 31, 34–40, 42 |
| **Kiểm chứng cấu hình** | File cấu hình (Gradle, Xcode project, Fastfile, workflow YAML, ExportOptions) được lint hoặc dry-run; bước upload thật mô tả theo tài liệu chính thức và cần tài khoản của người đọc | Bài 50–59 |

## 4. Bộ công nghệ đã chốt

| Nhu cầu | Lựa chọn | Phiên bản kiểm (11/2026) | Thay thế có nhắc trong bài |
|---|---|---|---|
| State management | `flutter_riverpod` + `riverpod_generator` | 3.4 / 4.0 | `provider`, `bloc` (so sánh ở bài 17–18) |
| Điều hướng | `go_router` | 18.1 | Navigator 1.0 (bài 12), `auto_route` |
| HTTP | `dio` | 5.11 | `http` |
| Model / JSON | `freezed` + `json_serializable` | 4.0 / 6.15 | viết tay (bài 19 viết tay trước, bài 26 sinh mã) |
| Lưu trữ cục bộ | `drift` (SQLite), `shared_preferences`, `flutter_secure_storage` | 2.35 / — / 11.2 | `isar`, `hive_ce` |
| Bản đồ | `flutter_map` (OpenStreetMap) | 8.3 | `google_maps_flutter` (cần API key; nêu khác biệt) |
| Vị trí | `geolocator`, `permission_handler` | 14.1 / — | — |
| Realtime | `web_socket_channel` | 3.0 | SSE, `socket_io_client` |
| Push | `firebase_messaging`, `flutter_local_notifications` | 17.0 / — | OneSignal |
| Nền | `workmanager` | 0.10 | `flutter_background_service` |
| Test | `flutter_test`, `mocktail`, `patrol`, `integration_test` | — / 1.0 / 4.11 / — | `mockito` |
| Lint | `flutter_lints`, analyzer plugin chính thức (`analysis_server_plugin`), `riverpod_lint` 3.2 | — | `very_good_analysis`; `custom_lint` không còn là đường khuyên dùng (kiểm 11/2026) |
| Build/phát hành | fastlane, GitHub Actions, Firebase App Distribution, TestFlight, Google Play Console, Shorebird (code push) | — | Codemagic, Bitrise |
| Quan sát | Firebase Crashlytics, Sentry, trace header W3C nối với OpenTelemetry của backend | — | — |
| Backend lab | `tools/fake_api` (Dart `shelf` + WebSocket) mô phỏng trip-service và Realtime Gateway; tuỳ chọn trip-service .NET thật | — | — |

Phiên bản ghi ở đây là mốc kiểm tại thời điểm chốt khung; mỗi lab pin phiên bản thật trong `pubspec.yaml` và README của bài.

## 5. Cấu trúc lab

```
examples/toi-noi-flutter/
  README.md                 khung biên tập, trạng thái từng bài, lệnh chạy
  packages/
    toi_noi_core/           bài 06: tiền, cước, trạng thái chuyến (Dart thuần)
    toi_noi_ui/             bài 13: theme, token màu, widget dùng chung
    toi_noi_api/            bài 19: client HTTP, DTO, lỗi mạng
  tools/
    fake_api/               bài 19+: server shelf mô phỏng trip-service + realtime
  01-flutter-va-dart/ … 42-mobile-security/   lab từng bài, tự chạy độc lập
  toi-noi-app/              bài 43+: workspace pub gồm apps/customer, apps/driver
  verify-01-06.mjs …        chạy analyze + test của một nhóm bài
```

Mỗi thư mục lab có `README.md` (bài tương ứng, lệnh chạy, lệnh kiểm chứng, phiên bản đã kiểm, phạm vi chưa làm). Từ bài 07, lab không commit thư mục `android/`, `ios/` (README ghi lệnh `flutter create` sinh runner một lần khi cần chạy trên máy); bài dạy cấu hình native chỉ commit đúng file cấu hình đang dạy. Lab bài 01–42 là project nhỏ độc lập; từ bài 43, code dồn vào `toi-noi-app` và các bài sau chỉ giải thích lát cắt của cùng một workspace (giống cách bài 31–38 lộ trình .NET dùng chung `31-38-trip-service`).

## 6. Chương trình chi tiết

Cột **Lab**: thư mục trong `examples/toi-noi-flutter/`. Cột **Trước**: bài cần đọc trước (phụ thuộc trực tiếp). Cột **KC**: mức kiểm chứng theo §3.3 (TĐ = tự động, TB = thiết bị, CH = cấu hình).

### Phần 1 · Dart cho Tới Nơi (cơ bản, 6 bài)

Mục tiêu: viết được Dart đủ để mô hình hoá cước, chuyến và luồng bất đồng bộ; có package `toi_noi_core` kiểm chứng bằng `dart test` trước khi chạm vào widget.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-01 | `flutter-va-dart-tu-source-code-den-ung-dung-toi-noi` | Flutter và Dart: từ source code đến ứng dụng Tới Nơi chạy trên iOS và Android | cơ bản | Flutter SDK, engine, framework, Dart; `flutter doctor`, `flutter create`, cấu trúc `lib/ android/ ios/ pubspec.yaml`; JIT khi dev, AOT khi release; hot reload; chạy trên simulator/emulator; app in cước 5 km | `01-flutter-va-dart` | — | TĐ |
| FL-02 | `dart-bien-kieu-du-lieu-null-safety-tinh-cuoc` | Dart cơ bản: biến, kiểu dữ liệu và null safety khi tính cước Tới Nơi | cơ bản | `int/double/num/String/bool`, `var/final/const/late`, VND là `int`, km và mét, chia nguyên, làm tròn; null safety `?`, `!`, `??`, `?.`, `required` | `02-dart-bien-kieu-null-safety` | 01 | TĐ |
| FL-03 | `dart-ham-class-enum-sealed-pattern-matching-trang-thai-chuyen` | Hàm, class, enum và sealed class: mô hình trạng thái chuyến bằng Dart | cơ bản | Hàm và tham số đặt tên, class/constructor, enhanced enum `TripStatus` (8 trạng thái), sealed class + `switch` vét cạn, record, `==`/`hashCode`, immutability | `03-ham-class-sealed-trang-thai` | 02 | TĐ |
| FL-04 | `dart-collections-generics-extension-danh-sach-chuyen` | Collections, generics và extension: quản lý danh sách chuyến và tài xế | cơ bản | `List/Set/Map`, `Iterable` lười, `where/map/fold/sorted`, generic class `Paged<T>`, extension methods, collection-if/for, spread | `04-collections-generics` | 03 | TĐ |
| FL-05 | `dart-future-async-await-stream-bao-gia-va-vi-tri` | Future, async/await và Stream: gọi báo giá và lắng nghe vị trí tài xế | cơ bản | Event loop và microtask, `Future`, `async/await`, lỗi và `try/catch`, `Stream`, `StreamController`, `await for`, huỷ subscription, timeout, `Isolate.run` sơ lược | `05-async-stream` | 04 | TĐ |
| FL-06 | `dart-package-pubspec-lint-test-toi-noi-core` | Package, pubspec, lint và test: thư viện toi_noi_core kiểm chứng được | cơ bản | `pubspec.yaml`, dependency/dev_dependency, `analysis_options.yaml`, `dart test`, `group/test/expect`, bố cục package, export, đưa code bài 02–05 vào `packages/toi_noi_core` | `packages/toi_noi_core` | 05 | TĐ |

### Phần 2 · Flutter UI cơ bản (cơ bản → trung bình, 10 bài)

Mục tiêu: dựng được các màn hình chính của app khách (báo giá, danh sách chuyến, nhập điểm, xác nhận) bằng widget chuẩn, theme Tới Nơi, song ngữ, có widget test.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-07 | `flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia` | Widget, cây widget và hot reload: màn hình báo giá đầu tiên | cơ bản | Widget/Element/RenderObject, `StatelessWidget`, `build`, `MaterialApp/Scaffold`, `Text/Icon/Container/Padding`, hot reload vs restart, widget test đầu tiên | `07-widget-man-hinh-bao-gia` | 06 | TĐ |
| FL-08 | `flutter-layout-constraints-row-column-the-chuyen` | Layout và constraints: Row, Column, Expanded và thẻ chuyến | cơ bản | "Constraints go down, sizes go up", `Row/Column/Flex`, `Expanded/Flexible`, `SizedBox`, `Align`, `Stack`, overflow, `LayoutBuilder`, thẻ chuyến | `08-layout-constraints` | 07 | TĐ |
| FL-09 | `flutter-listview-sliver-scroll-lich-su-chuyen` | ListView, Sliver và cuộn: danh sách lịch sử chuyến | cơ bản | `ListView.builder/separated`, `CustomScrollView` + `SliverAppBar/SliverList`, `RefreshIndicator`, `ScrollController`, tải thêm cuối trang, key trong danh sách | `09-listview-sliver` | 08 | TĐ |
| FL-10 | `flutter-statefulwidget-lifecycle-setstate-chon-dich-vu` | StatefulWidget, lifecycle và setState: chọn dịch vụ và đếm ngược tìm tài xế | cơ bản | `State`, `initState/didUpdateWidget/dispose`, `setState` và phạm vi rebuild, `Timer`, `mounted`, `GlobalKey`, chọn ride/food/parcel | `10-stateful-lifecycle` | 09 | TĐ |
| FL-11 | `flutter-form-validation-focus-nhap-diem-don-diem-den` | Form, TextFormField và validation: nhập điểm đón, điểm đến | cơ bản | `Form/FormState`, `TextEditingController`, `validator`, `FocusNode`, `TextInputAction`, `AutovalidateMode`, bàn phím và `resizeToAvoidBottomInset` | `11-form-validation` | 10 | TĐ |
| FL-12 | `flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen` | Navigator, route và truyền dữ liệu: từ báo giá sang xác nhận chuyến | cơ bản | `Navigator.push/pop` có kiểu trả về, named route, `PopScope`, `showModalBottomSheet`, dialog, truyền đối tượng không truyền state toàn cục | `12-navigator-route` | 11 | TĐ |
| FL-13 | `flutter-theme-material-3-dark-mode-design-system-toi-noi` | Theme Material 3, màu, chữ và dark mode: design system Tới Nơi | cơ bản | `ThemeData`, `ColorScheme.fromSeed`, `TextTheme`, `ThemeExtension`, dark mode theo hệ thống, component theme, package `toi_noi_ui` | `packages/toi_noi_ui` + `13-theme-design-system` | 12 | TĐ |
| FL-14 | `flutter-asset-intl-localization-tien-vnd-ngay-gio` | Asset, intl và localization: hiển thị tiền VND, ngày giờ và song ngữ vi/en | cơ bản | `pubspec` assets, ảnh và độ phân giải, font, `intl` `NumberFormat.currency` VND, `DateFormat` theo locale, `flutter_localizations` + ARB `gen-l10n` | `14-asset-intl-l10n` | 13 | TĐ |
| FL-15 | `flutter-animation-implicit-hero-tim-tai-xe` | Animation cơ bản: implicit animation và Hero khi tìm tài xế | trung bình | `AnimatedContainer/AnimatedSwitcher/AnimatedOpacity`, `TweenAnimationBuilder`, `Hero`, curve và duration, giảm chuyển động theo accessibility | `15-animation-co-ban` | 14 | TĐ |
| FL-16 | `flutter-accessibility-responsive-adaptive-ios-android` | Accessibility, responsive và adaptive: một giao diện cho iOS và Android | trung bình | `Semantics`, text scale, contrast, `MediaQuery/SafeArea`, breakpoint, `Platform`/`Theme.platform`, Cupertino vs Material có chủ đích, back gesture | `16-a11y-responsive-adaptive` | 15 | TĐ |

### Phần 3 · Trạng thái, dữ liệu và kiến trúc (trung bình, 10 bài)

Mục tiêu: app khách gọi được backend (fake API hoặc trip-service thật), có state rõ ràng bằng Riverpod, lưu cục bộ, đăng nhập, cấu hình ba môi trường và kiến trúc tách lớp.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-17 | `flutter-state-inheritedwidget-changenotifier-bao-gia` | Quản lý state: InheritedWidget, ChangeNotifier và ranh giới rebuild | trung bình | Ephemeral vs app state, `InheritedWidget`, `ChangeNotifier` + `ListenableBuilder`, `ValueNotifier`, vì sao cần thư viện, so sánh provider/bloc/riverpod | `17-state-changenotifier` | 16 | TĐ |
| FL-18 | `flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen` | Riverpod: Provider, Notifier và AsyncNotifier cho luồng đặt chuyến | trung bình | `ProviderScope`, `Provider/Notifier/AsyncNotifier`, `ref.watch/read/listen`, `AsyncValue`, auto-dispose, family, override để test | `18-riverpod` | 17 | TĐ |
| FL-19 | `flutter-http-dio-json-dto-goi-api-bao-gia` | HTTP, dio và JSON: gọi API báo giá của trip-service | trung bình | `dio` client, base URL theo môi trường, DTO `fromJson/toJson` viết tay, tách DTO khỏi domain, `tools/fake_api`, interceptor log; package `toi_noi_api` | `packages/toi_noi_api` + `tools/fake_api` + `19-http-dio-json` | 18 | TĐ |
| FL-20 | `flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong` | Xử lý lỗi, retry, timeout và mất mạng: mạng di động không ổn định | trung bình | Phân loại lỗi (mạng, 4xx, 5xx, parse), sealed `ApiFailure`, timeout, retry có backoff và chỉ cho request idempotent, `connectivity_plus`, hiển thị lỗi có hành động | `20-error-retry-connectivity` | 19 | TĐ |
| FL-21 | `flutter-clean-architecture-domain-data-presentation-mobile` | Clean Architecture cho mobile: domain, data, presentation và repository | trung bình | Ba lớp và chiều phụ thuộc, entity vs DTO, repository interface trong domain, use case, mapper, provider nối lớp, khi nào quá tay | `21-clean-architecture` | 20 | TĐ |
| FL-22 | `flutter-local-persistence-drift-secure-storage-lich-su-offline` | Lưu trữ cục bộ: drift, shared_preferences và secure storage | trung bình | Chọn kho theo loại dữ liệu, `shared_preferences` cho tuỳ chọn, `drift` cho lịch sử chuyến (bảng, DAO, migration, stream query), `flutter_secure_storage` cho token | `22-persistence-drift` | 21 | TĐ |
| FL-23 | `flutter-go-router-deep-link-shellroute-guard` | go_router: deep link, ShellRoute, guard đăng nhập | trung bình | Route khai báo, path param, `ShellRoute` + bottom navigation, `redirect` theo trạng thái đăng nhập, deep link `toinoi://trip/{id}` và universal/app link, test router | `23-go-router` | 22 | TĐ |
| FL-24 | `flutter-authentication-otp-jwt-refresh-token-secure-storage` | Authentication: OTP, JWT, refresh token rotation và interceptor | trung bình | Luồng OTP, access/refresh token, lưu an toàn, interceptor gắn token và refresh một lần khi 401, đăng xuất và revoke, không tin role từ client | `24-auth-jwt-refresh` | 23 | TĐ |
| FL-25 | `flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod` | Cấu hình môi trường: flavors, --dart-define-from-file và DI theo môi trường | trung bình | Ba flavor dev/staging/prod, `--dart-define-from-file`, `AppConfig`, Riverpod override theo môi trường, file cấu hình không chứa bí mật, chạy từng flavor | `25-flavors-config` | 24 | TĐ |
| FL-26 | `flutter-code-generation-freezed-json-serializable-riverpod-generator` | Code generation: freezed, json_serializable, riverpod_generator và build_runner | trung bình | Khi nào sinh mã đáng giá, `freezed` union/copyWith, `json_serializable`, `riverpod_generator`, `build_runner watch`, commit file sinh hay không | `26-codegen` | 25 | TĐ |

### Phần 4 · Kiểm thử và chất lượng (trung bình → nâng cao, 7 bài)

Mục tiêu: mỗi lớp của app có loại test phù hợp, chạy xanh trên CI, và quy tắc lint giữ code đồng nhất.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-27 | `flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case` | Unit test với mocktail và fake: quy tắc cước và use case | trung bình | Test domain thuần, test double (fake/stub/mock) bằng `mocktail`, `registerFallbackValue`, `verify`, ca biên, test use case qua repository giả | `27-unit-test-mocktail` | 26 | TĐ |
| FL-28 | `flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia` | Widget test: pumpWidget, finder, matcher và màn hình báo giá | trung bình | `WidgetTester`, `pump/pumpAndSettle`, `find.byKey/text/type`, `enterText/tap`, bọc `ProviderScope` override, test điều hướng | `28-widget-test` | 27 | TĐ |
| FL-29 | `flutter-golden-test-semantics-test-giao-dien-on-dinh` | Golden test và Semantics test: khoá giao diện và accessibility | trung bình | `matchesGoldenFile`, font và kích thước cố định, cập nhật golden có chủ đích, `meetsGuideline` (tap target, contrast, label), chạy golden trên CI Linux | `29-golden-semantics` | 28 | TĐ |
| FL-30 | `flutter-test-riverpod-async-stream-fake-async-providercontainer` | Kiểm thử Riverpod, async và Stream: ProviderContainer, fakeAsync, fake clock | trung bình | `ProviderContainer.test`, `listen`, `AsyncValue` các trạng thái, `fake_async`, `Clock` tiêm được, test Stream và timeout | `30-test-riverpod-async` | 29 | TĐ |
| FL-31 | `flutter-integration-test-patrol-luong-dat-chuyen-tren-thiet-bi` | Integration test và patrol: luồng đặt chuyến trên simulator/emulator | nâng cao | `integration_test`, `patrol` tương tác native (quyền, thông báo), chạy với fake API, chụp màn hình, chi phí và khi nào đáng chạy | `31-integration-test-patrol` | 30 | TB |
| FL-32 | `flutter-lint-analyzer-format-analyzer-plugin-chat-luong-code` | Lint, analyzer, dart format và analyzer plugin: chất lượng code có kiểm chứng | trung bình | `analysis_options.yaml`, `flutter_lints` mức strict, `riverpod_lint`, analyzer plugin chính thức (`plugins:`), lint tự viết, `dart format`, `dart fix`, pre-commit, ngân sách cảnh báo bằng 0 | `32-lint-analyzer` | 31 | TĐ |
| FL-33 | `flutter-ci-github-actions-analyze-test-golden-cache` | CI với GitHub Actions: analyze, test, golden và cache pub | trung bình | Workflow ubuntu: setup Flutter pin phiên bản, cache pub, `analyze`, `test --coverage`, golden chạy riêng trên macOS vì baseline sinh ở đó, artifact báo cáo, trigger theo path; `.github/workflows/toi-noi-flutter.yml` | `33-ci-github-actions` | 32 | TĐ |

### Phần 5 · Tính năng di động (trung bình → nâng cao, 9 bài)

Mục tiêu: các tính năng đặc thù mobile của Tới Nơi — bản đồ, vị trí, realtime, push, nền, thanh toán, native — với cấu hình iOS/Android đúng chính sách nền tảng.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-34 | `flutter-ban-do-flutter-map-marker-polyline-chon-diem-don` | Bản đồ với flutter_map: marker, polyline và chọn điểm đón | trung bình | Tile layer OSM và điều khoản dùng, `MapController`, marker tài xế, polyline tuyến, chọn điểm bằng kéo bản đồ, so với Google Maps SDK | `34-map-flutter-map` | 33 | TB |
| FL-35 | `flutter-vi-tri-geolocator-permission-foreground-background` | Vị trí: geolocator, quyền foreground/background và cấu hình iOS, Android | trung bình | `Info.plist` usage description, `AndroidManifest` permission, xin quyền đúng lúc, `getPositionStream`, độ chính xác và pin, mô phỏng vị trí trên simulator | `35-location-permission` | 34 | TB |
| FL-36 | `flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe` | Realtime WebSocket: reconnect, backoff và theo dõi tài xế | nâng cao | Kết nối sau xác thực, subscribe channel theo chuyến, reconnect có backoff + jitter, resume từ sequence, gộp vị trí, tách durable event và ephemeral telemetry, fake Realtime Gateway | `36-realtime-websocket` | 35 | TĐ |
| FL-37 | `flutter-push-notification-fcm-apns-deep-link` | Push notification: FCM, APNs, local notification và deep link từ thông báo | nâng cao | Cấu hình Firebase cho ba flavor, APNs key, token và đăng ký với backend, foreground/background/terminated, payload không chứa dữ liệu nhạy cảm, mở đúng màn hình | `37-push-notification` | 36 | TB |
| FL-38 | `flutter-background-work-workmanager-vi-tri-nen-app-tai-xe` | Tác vụ nền: workmanager, vị trí nền cho app tài xế và chính sách store | nâng cao | Giới hạn nền của iOS/Android, `workmanager` cho việc trì hoãn được, vị trí nền chỉ khi có chuyến hợp lệ, foreground service Android, pin, lý do bị từ chối review | `38-background-work` | 37 | TB |
| FL-39 | `flutter-thanh-toan-vi-xanh-webview-deep-link-idempotency` | Thanh toán qua VíXanh: WebView, deep link quay về và idempotency key | nâng cao | Không xử lý thẻ trong app, mở cổng thanh toán bằng WebView/Custom Tabs, deep link quay về, chờ xác nhận server (không tự coi là thành công), idempotency key, Apple IAP không áp dụng cho dịch vụ vật lý | `39-payment-flow` | 38 | TB |
| FL-40 | `flutter-platform-channel-pigeon-ffi-goi-native` | Platform channel, Pigeon và FFI: gọi native khi plugin không đủ | nâng cao | `MethodChannel` hai chiều, `Pigeon` sinh interface an toàn kiểu, `dart:ffi` với thư viện C, khi nào viết plugin riêng, kiểm thử channel bằng mock | `40-platform-channel-pigeon` | 39 | TB |
| FL-41 | `flutter-hieu-nang-rebuild-isolate-devtools` | Hiệu năng: rebuild, const, RepaintBoundary, isolate và DevTools | nâng cao | Đo trước khi sửa, DevTools Performance/Widget rebuild, `const`, tách widget, `RepaintBoundary`, `ListView` lười, cache ảnh, `Isolate.run` cho parse JSON lớn, jank trên profile mode | `41-performance` | 40 | TĐ |
| FL-42 | `flutter-bao-mat-mobile-secure-storage-pinning-obfuscation-masvs` | Bảo mật mobile: secure storage, pinning, obfuscation và OWASP MASVS | nâng cao | Mô hình đe doạ app khách/tài xế, lưu bí mật, pinning chỉ khi có quy trình rotation, `--obfuscate`, log không lộ dữ liệu, phát hiện root/jailbreak là tín hiệu không phải khoá, MASVS checklist | `42-mobile-security` | 41 | TB |

### Phần 6 · Ứng dụng hoàn chỉnh: app khách và app tài xế (nâng cao, 7 bài)

Mục tiêu: ghép mọi cơ chế thành workspace `toi-noi-app` gồm hai app dùng chung package, máy trạng thái chuyến đồng bộ với backend, offline-first và quan sát được.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-43 | `flutter-monorepo-pub-workspaces-packages-app-khach-app-tai-xe` | Monorepo với pub workspaces: packages dùng chung cho app khách và app tài xế | nâng cao | `workspace` trong pubspec, `apps/customer`, `apps/driver`, `packages/*`, một `analysis_options` gốc, script chạy test toàn workspace, ranh giới package | `toi-noi-app` | 42 | TĐ |
| FL-44 | `flutter-feature-first-clean-architecture-riverpod-feature-dat-chuyen` | Feature-first + Clean Architecture + Riverpod: cấu trúc feature đặt chuyến | nâng cao | Thư mục theo feature, mỗi feature có domain/data/presentation, provider là ranh giới, chia sẻ qua package core, quy tắc import kiểm bằng lint | `toi-noi-app` | 43 | TĐ |
| FL-45 | `flutter-may-trang-thai-chuyen-client-dong-bo-server-optimistic-ui` | Máy trạng thái chuyến trên client: đồng bộ server, optimistic UI và xác nhận | nâng cao | Sealed `TripState` 8 trạng thái + cancelled, nguồn sự thật là server, optimistic UI có hoàn tác, hợp nhất REST snapshot với realtime event theo version, màn hình theo trạng thái | `toi-noi-app` | 44 | TĐ |
| FL-46 | `flutter-offline-first-outbox-client-idempotency-dong-bo` | Offline-first: outbox trên client, idempotency key và đồng bộ khi có mạng | nâng cao | Outbox trong drift, thao tác được trì hoãn vs không được (thanh toán), idempotency key sinh ở client, đồng bộ khi có mạng, xung đột và thông báo người dùng | `toi-noi-app` | 45 | TĐ |
| FL-47 | `flutter-app-tai-xe-online-offline-nhan-cuoc-dieu-huong` | App tài xế: online/offline, nhận cuốc, đếm ngược và điều hướng | nâng cao | Trạng thái sẵn sàng, nhận offer qua realtime, đếm ngược accept có hạn, chỉ một tài xế thắng (409 từ server), gửi vị trí theo trạng thái chuyến, mở app dẫn đường | `toi-noi-app/apps/driver` | 46 | TĐ |
| FL-48 | `flutter-observability-crashlytics-sentry-opentelemetry-mobile` | Quan sát từ mobile: Crashlytics, Sentry và trace nối với backend | nâng cao | Crash và non-fatal, breadcrumb không chứa PII, `traceparent` gửi kèm request để nối với OpenTelemetry của trip-service, metric nghiệp vụ (thời gian tới báo giá), opt-in theo luật | `toi-noi-app` | 47 | TĐ |
| FL-49 | `flutter-tong-hop-phan-6-dat-chuyen-end-to-end-app-khach-tai-xe` | Bài tổng hợp: đặt chuyến end-to-end giữa app khách, app tài xế và trip-service | nâng cao | Chạy fake API + hai app, kịch bản đặt → tìm → nhận → hoàn thành, điều đã test tự động và điều phải thử tay, phạm vi chưa làm | `toi-noi-app` | 48 | TĐ |

### Phần 7 · Build, phân phối và phát hành (nâng cao, 10 bài)

Mục tiêu: từ source đến người dùng với quy trình lặp lại được: ba flavor, ký đúng, build trên CI, phân phối nội bộ, TestFlight, Google Play testing, production, và vận hành sau phát hành.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-50 | `flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build` | Flavors khi build: bundle ID, application ID, icon, tên app và Firebase theo môi trường | nâng cao | `productFlavors` Gradle, Xcode scheme/configuration, `--flavor`, icon và tên khác nhau để cài song song ba bản, `google-services.json`/`GoogleService-Info.plist` theo flavor, kiểm bằng script | `50-flavors-build` | 49 | CH |
| FL-51 | `flutter-ky-ung-dung-android-keystore-play-app-signing-aab` | Ký ứng dụng Android: keystore, Play App Signing, AAB và versionCode | nâng cao | Tạo upload keystore, `key.properties` ngoài git, `signingConfigs`, AAB vs APK, Play App Signing giữ khoá phát hành, `versionCode` tăng đơn điệu, kiểm chữ ký bằng `apksigner` | `51-android-signing` | 50 | CH |
| FL-52 | `flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities` | Ký ứng dụng iOS: Apple Developer Program, certificate, provisioning profile và capabilities | nâng cao | Vai trò tài khoản, certificate development/distribution, App ID và capabilities (push, background modes, associated domains), profile development/ad-hoc/app-store, `ExportOptions.plist`, lỗi ký hay gặp | `52-ios-signing` | 51 | CH |
| FL-53 | `flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build` | fastlane cho Flutter: match, gym, pilot, supply | nâng cao | `Fastfile` theo lane, `match` lưu chứng chỉ mã hoá, `gym` build IPA, `pilot` lên TestFlight, `supply` lên Play, App Store Connect API key thay mật khẩu, chạy local trước CI | `53-fastlane` | 52 | CH |
| FL-54 | `flutter-ci-cd-github-actions-macos-build-ios-android-secrets` | CI/CD build: GitHub Actions macOS/ubuntu, secrets, build number và artifact | nâng cao | Job Android trên ubuntu, job iOS trên macOS, giải mã keystore/profile từ secrets, build number từ `run_number`, artifact AAB/IPA, chi phí phút macOS, so với Codemagic | `54-ci-cd-build` | 53 | CH |
| FL-55 | `flutter-firebase-app-distribution-phan-phoi-noi-bo-tester` | Firebase App Distribution: phân phối nội bộ cho tester iOS và Android | nâng cao | Nhóm tester, upload APK/IPA ad-hoc từ CI, UDID thiết bị iOS, release notes tự sinh từ commit, feedback và crash từ bản nội bộ, giới hạn so với TestFlight | `55-app-distribution` | 54 | CH |
| FL-56 | `flutter-testflight-internal-external-testing-beta-review` | TestFlight: internal/external testing, Beta App Review và export compliance | nâng cao | Upload build, xử lý, internal tester (tới 100) không cần review, external cần Beta App Review, export compliance và mã hoá, hết hạn 90 ngày, thu thập feedback và crash | `56-testflight` | 55 | CH |
| FL-57 | `flutter-google-play-internal-closed-open-production-staged-rollout` | Google Play: internal, closed, open testing, production và staged rollout | nâng cao | Bốn track, promote giữa track, pre-launch report, Data safety và quyền nhạy cảm (vị trí nền), yêu cầu 12 tester cho tài khoản cá nhân, staged rollout và halt | `57-google-play` | 56 | CH |
| FL-58 | `flutter-app-store-production-metadata-privacy-review-phased-release` | App Store production: metadata, privacy labels, App Review và phased release | nâng cao | Bản ghi app, ảnh chụp theo kích thước, privacy nutrition labels, tài khoản demo cho reviewer, lý do bị từ chối hay gặp với app gọi xe, phased release 7 ngày, expedited review | `58-app-store-release` | 57 | CH |
| FL-59 | `flutter-van-hanh-sau-phat-hanh-force-update-remote-config-shorebird-rollback` | Vận hành sau phát hành: force update, remote config, Shorebird và rollback | nâng cao | Kiểm phiên bản tối thiểu từ server, feature flag/remote config, Shorebird code push cho Dart (giới hạn và chính sách store), rollback bằng halt rollout/phased release, hotfix nhánh release | `59-post-release-ops` | 58 | CH |

### Phần 8 · Chuyên sâu và tổng kết (nâng cao, 5 bài)

Mục tiêu: các chủ đề nền tảng còn lại và bài tổng kết toàn lộ trình.

| ID | Slug | Tiêu đề | Mức | Nội dung chính | Lab | Trước | KC |
|---|---|---|---|---|---|---|---|
| FL-60 | `flutter-custom-painter-renderobject-ve-tuyen-duong` | CustomPainter và RenderObject: vẽ tuyến đường và đồng hồ cước | nâng cao | `CustomPaint/CustomPainter`, `Canvas/Path`, `shouldRepaint`, khi nào xuống `RenderBox` (`performLayout/paint`), hit test, đo chi phí vẽ | `60-custom-painter` | 49 | TĐ |
| FL-61 | `flutter-explicit-animation-impeller-shader-hieu-ung-tim-tai-xe` | Explicit animation, Impeller và shader: hiệu ứng tìm tài xế | nâng cao | `AnimationController`, `Tween/Curve`, `AnimatedBuilder`, `TickerProvider`, Impeller và shader compile, fragment shader `.frag`, chuyển động tôn trọng reduce-motion | `61-explicit-animation-shader` | 60 | TĐ |
| FL-62 | `flutter-kich-thuoc-app-thoi-gian-khoi-dong-deferred-components` | Kích thước app, thời gian khởi động và deferred components | nâng cao | `--analyze-size`, tree shaking icon, nén asset, `--split-per-abi`, deferred components trên Android, startup trace, splash native và `flutter_native_splash` | `62-app-size-startup` | 61 | CH |
| FL-63 | `flutter-nang-cap-flutter-dart-breaking-changes-quan-ly-dependency` | Nâng cấp Flutter/Dart, breaking changes và quản lý dependency | nâng cao | Lịch phát hành, `flutter upgrade` có kiểm soát, đọc breaking changes, `dart fix`, `pub outdated/upgrade --major-versions`, pin bằng lockfile, kiểm CI trên hai phiên bản | `63-upgrade-dependency` | 62 | TĐ |
| FL-64 | `flutter-tong-hop-toi-noi-tu-dart-toi-store` | Bài tổng hợp: Tới Nơi Flutter từ Dart tới App Store và Google Play | nâng cao | Nhìn lại 8 phần theo một chuyến xe, bản đồ quyết định (state, điều hướng, lưu trữ, phát hành), checklist trước phát hành, phần chưa làm và hướng tiếp | `toi-noi-app` | 63 | TĐ |

## 7. Lộ trình đọc theo mục tiêu

| Mục tiêu | Thứ tự |
|---|---|
| Lần đầu học mobile | P1 → P2 → P3 → P4 → P5 → P6 → P7 → P8 |
| Đã biết Dart hoặc một ngôn ngữ OOP | 01 → P2 (bỏ 02–05, đọc 06) → tiếp theo thứ tự |
| Đã làm Flutter, muốn kiến trúc và test | 17, 18, 21, 25, 26 → P4 → P6 |
| Chỉ cần phát hành một app sẵn có | 25, 33 → P7 theo thứ tự 50 → 59 |
| Dev backend .NET muốn hiểu client | 01, 07, 18, 19, 20, 24, 36, 45, 46, 48 |

## 8. Khuôn bài viết

Mọi bài theo khuôn của lộ trình C#/.NET 10 để hai loạt đọc nối nhau được:

1. **Frontmatter** theo §2.
2. **Mở đầu** (2–3 đoạn): một tình huống của Tới Nơi dẫn vào cơ chế; câu "Đây là bài NN của lộ trình **Flutter qua Tới Nơi**"; cần biết gì trước (link bài trước).
3. **"Thử trước khi đọc"**: callout `tkh-callout tkh-callout--example` với một câu hỏi đúng/sai hoặc dự đoán và `<details>` đáp án.
4. **4–6 mục H2** giải thích cơ chế, mỗi mục có code; **ít nhất 2 hình SVG** theo bộ lớp của blog (`tkh-fig`, `f-phys-soft s-phys`, `f-pk-soft s-pk`, `f-warn-soft s-warn`, `t-b`, `t-small`, `t-mono`), `role="img"` + `aria-label`, `figcaption` "Hình N. …".
5. **Lỗi thường gặp** (callout `tkh-callout--warn` hoặc danh sách): 3–5 lỗi thật, mỗi lỗi có cách nhận ra và cách sửa.
6. **Chạy ví dụ và kiểm chứng**: lệnh từ gốc repository, test nào chứng minh điều gì, mức kiểm chứng (§3.3), phần lab chưa làm.
7. **Bài tập**: 3 câu, có `<details>` lời giải.
8. **Bài trước / bài tiếp** (chỉ link bài đã có) và **Tài liệu gốc**: docs.flutter.dev, dart.dev, api.flutter.dev, pub.dev, developer.apple.com, developer.android.com, firebase.google.com — không dẫn blog cá nhân thay tài liệu chính thức.

Giọng văn: tiếng Việt, xưng "ta", thuật ngữ kỹ thuật giữ tiếng Anh (widget, state, provider, build), câu ngắn, mỗi đoạn một ý. Mọi số liệu giá, thời gian, quy mô là minh hoạ và nói rõ như vậy.

## 9. Quy trình viết và review

| Bước | Ai | Việc |
|---|---|---|
| 1. Đề cương | Orchestrator | Chốt slug, tiêu đề, mục tiêu, lab theo bảng §6; chuẩn bị package dùng chung nếu bài cần |
| 2. Viết bài + lab | Sonnet | Viết `api/seed/content/<slug>.md` và thư mục lab; chạy `flutter analyze` và `flutter test`/`dart test` xanh; kiểm API bằng analyzer (không dùng API đã deprecated trong Flutter 3.47/Dart 3.13; khi không chắc, đọc source package trong `.pub-cache` hoặc changelog trên pub.dev) |
| 3. Review | Opus | Đối chiếu kỹ thuật (API, phiên bản, hành vi iOS/Android), code trong bài khớp lab, lab thực sự xanh (chạy lại), nhất quán với hồ sơ Tới Nơi (8 trạng thái, 3 dịch vụ, VND, bảng giá minh hoạ), khuôn bài §8, tiếng Việt, SVG dùng đúng lớp, không link bài chưa có, không lộ bí mật. Kết quả: danh sách vấn đề có mức (blocker/major/minor) và vị trí |
| 4. Sửa | Sonnet | Sửa theo danh sách; chạy lại kiểm chứng |
| 5. Tích hợp | Orchestrator | Ảnh bìa (`article-covers.json` + `generate-covers.mjs`), `learning-paths.json`, README lab, script verify, workflow CI; `npm run typecheck && npm run build`; commit |

Trạng thái từng bài theo dõi trong `examples/toi-noi-flutter/README.md` (Kế hoạch → Đang viết → Đã review → Đã xuất bản).

## 10. Điểm cần chủ blog quyết

1. **Thư viện state**: khung chọn Riverpod 3 làm trục; bloc chỉ so sánh ở bài 17. Nếu đội thực tế dùng bloc, bài 18 và Phần 6 cần viết lại theo bloc.
2. **Bản đồ**: lab dùng `flutter_map` + OpenStreetMap để không cần API key; sản phẩm thật nhiều khả năng dùng Google Maps/Mapbox. Bài 34 nêu khác biệt nhưng không có lab Google Maps.
3. **Backend cho lab**: mặc định `tools/fake_api` (Dart). Nếu muốn mọi bài từ 19 gọi trip-service .NET thật, cần bổ sung endpoint báo giá và realtime vào service đó (ngoài phạm vi loạt .NET hiện tại).
4. **Shorebird** (bài 59) là dịch vụ thương mại bên thứ ba; có thể thay bằng bài thuần "hotfix qua store" nếu không muốn giới thiệu dịch vụ trả phí.
5. **Hồ sơ kiến trúc** `toi-noi-overall-architecture.md` §3.1 và §12 đã đổi từ Swift/Kotlin sang Flutter trong cùng commit với khung này; nếu còn loạt bài native dự kiến, cần ghi hai lựa chọn song song.

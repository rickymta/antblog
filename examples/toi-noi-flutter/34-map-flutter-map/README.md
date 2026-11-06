# Lab 34: bản đồ flutter_map, marker, polyline và chọn điểm đón

Lab của bài **FL-34** trong loạt **Flutter qua Tới Nơi**: [Bản đồ với flutter_map: marker, polyline và chọn điểm đón](../../../api/seed/content/flutter-ban-do-flutter-map-marker-polyline-chon-diem-don.md).

Màn hình chọn điểm đón của app khách: bản đồ OpenStreetMap, một pin cố định giữa khung, người dùng **kéo bản đồ** để chọn điểm đón (đọc `MapCamera.center`). Có marker tài xế và marker điểm đến, hai polyline (tài xế tới điểm đón, điểm đón tới điểm đến), thẻ cước ước tính và nút xác nhận chỉ bật khi bản đồ đã đứng yên. Phần logic thuần (haversine ra mét, cước từ quãng đường bằng `calculateFare` của [`toi_noi_core`](../packages/toi_noi_core), kẹp vùng phục vụ) nằm riêng trong `lib/geo.dart` và có unit test. Bảng giá là minh hoạ: 5 km → 25.500 VND, 2,001 km → 12.005 VND, 100 km → 453.000 VND.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/geo.dart` | `haversineMeters`, `distanceMeters`, `routeMeters`, `fakeRoute`, `estimateFare` (`FareQuoted`/`FareOutOfRange`), `ServiceArea` (`contains`, `clamp`) |
| `lib/demo_data.dart` | Vùng phục vụ minh hoạ quanh TP. HCM và bốn toạ độ mẫu |
| `lib/pickup_map.dart` | `PickupMap`: `FlutterMap` + `TileLayer` OSM + `PolylineLayer` + `MarkerLayer` + `SimpleAttributionWidget`, pin giữa khung, debounce "đứng yên" |
| `lib/pickup_screen.dart` | `PickupScreen`: marker, polyline, thẻ cước, nút về điểm ban đầu, nút xác nhận |
| `lib/main.dart` | Điểm vào chạy trên máy (dùng tile OSM thật) |
| `test/support.dart` | `FakeTileProvider`, bộ đếm `HttpClient` (`CountingHttpOverrides`), vùng toàn quốc cho test vượt 100 km |
| `test/geo_test.dart` | Haversine đối chiếu `latlong2`, `Distance()` là Vincenty, toạ độ hỏng, `routeMeters`, `estimateFare` cho mọi mét 0..100000 |
| `test/service_area_test.dart` | `contains` và `clamp` ở biên, góc, toạ độ hỏng |
| `test/pickup_map_test.dart` | Không ra mạng, User-Agent, attribution, pin, kéo và fling, kẹp camera, tắt xoay |
| `test/pickup_screen_test.dart` | Vị trí marker, polyline, thẻ cước, nút xác nhận, kẹp điểm ban đầu |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/34-map-flutter-map
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (51 test). Test không gọi mạng: tile do `FakeTileProvider` trả từ bộ nhớ, và một test đếm số lần có mã tạo `HttpClient` (bằng 0). Mức kiểm chứng của bài là **trên thiết bị** (cột KC của khung); trên máy không có simulator, lab chỉ đạt phần **tự động** nói trên.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_34 --org vn.toinoi .
flutter run -d <id>
```

`main.dart` dùng tile OSM thật qua `tile.openstreetmap.org`. Mục đích chỉ là thử tay; **không** dùng cấu hình này cho app phát hành (xem bài và [Tile Usage Policy](https://operations.osmfoundation.org/policies/tiles/)). Ở chế độ debug flutter_map in cảnh báo về server tile công cộng; đó là chủ ý của thư viện.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_map` 8.3.2, `latlong2` 0.10.1, `http` 1.6.0 (dev, cho test đối chứng), `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 (path), Material 3. Bản `flutter_map` trong khung (§4) ghi 8.3; bản thật 8.3.2.

## Chưa làm

- Chưa chạy trên simulator, emulator hay máy thật: cử chỉ thật (kéo mượt, fling, pinch), hiệu năng khi nhiều marker, tải tile thật và hành vi thiếu mạng chưa kiểm.
- Chưa có vị trí thật của người dùng (bài 35) và chưa có vị trí tài xế realtime (bài 36); toạ độ tài xế là hằng.
- `fakeRoute` là đường giả hình chữ L, không phải dịch vụ định tuyến; cước ở đây chỉ là ước tính của client, cước chính thức do server tính.
- Chưa có geocoding ngược (toạ độ ra địa chỉ), chưa tải tile ngoại tuyến (bị cấm trên server OSM công cộng), chưa có tile server riêng hay nhà cung cấp tile trả phí.
- Chưa nối `onAttributionTap` với `url_launcher`; trang giấy phép chỉ được kiểm bằng callback.
- Chưa so sánh bằng lab với `google_maps_flutter` hay Mapbox (bài chỉ so sánh bằng văn bản).

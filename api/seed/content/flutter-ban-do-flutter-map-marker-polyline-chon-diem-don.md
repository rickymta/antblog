---
slug: flutter-ban-do-flutter-map-marker-polyline-chon-diem-don
title: "Bản đồ với flutter_map: marker, polyline và chọn điểm đón"
summary: "Bài 34: màn hình chọn điểm đón của Tới Nơi bằng flutter_map: tile OSM và điều khoản dùng (attribution, User-Agent, không chạy app lớn trên server cộng đồng), marker, polyline, kéo bản đồ đọc MapCamera.center, haversine, cước bằng calculateFare, test không ra mạng, so với Google Maps và Mapbox."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, ios, android, map, flutter-map, openstreetmap, testing]
language: vi
---

Chị Lan đứng ở cổng một khu chung cư ở TP. Hồ Chí Minh. GPS báo chị ở bên kia con hẻm, cách 80 m, nên tài xế sẽ phải chạy vòng. Chị muốn được đón ở cổng. Cách quen thuộc là một chiếc pin cố định giữa màn hình: chị kéo bản đồ cho tới khi pin đứng đúng cổng rồi bấm xác nhận, tuyến và cước cập nhật theo.

Đằng sau chiếc pin có ba câu hỏi: điểm đón trong code là gì, một `Marker` hay thứ khác? Làm sao tính cước từ hai toạ độ mà không lệch với `calculateFare`? Và bản đồ nền lấy ở đâu, khi app có 4 triệu khách mỗi tháng theo hồ sơ Tới Nơi?

Đây là bài 34 của lộ trình **Flutter qua Tới Nơi**, bài mở đầu Phần 5. Ta nên đọc [bài 33 về CI](/articles/flutter-ci-github-actions-analyze-test-golden-cache) trước, cùng [bài 28 về widget test](/articles/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia) và [bài 06 về `toi_noi_core`](/articles/dart-package-pubspec-lint-test-toi-noi-core), nơi có `calculateFare`. Bảng giá và mọi con số cước là minh hoạ; toạ độ không phải ranh giới dịch vụ thực.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta làm chiếc pin giữa màn hình bằng một <code>Marker</code> trong <code>MarkerLayer</code>, đặt tại toạ độ điểm đón hiện tại và cập nhật toạ độ đó bằng <code>setState</code> khi khách kéo. Thiết kế này đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p>Sai hướng. Một <code>Marker</code> gắn với toạ độ địa lý nên trôi theo bản đồ, và ta phải đuổi theo nó mỗi khung hình. Pin thuộc về <strong>màn hình</strong>: widget cố định giữa khung, còn điểm đón là <code>MapCamera.center</code>.</p></details>
</div>

## Tile raster và chính sách của server tile

`flutter_map` vẽ bản đồ **raster**: thế giới được cắt thành các ảnh vuông 256 px gọi là tile, ở mức zoom `z` có 2^z × 2^z tile, mỗi tile có toạ độ `(z, x, y)`. `TileLayer` tính tile nào nằm trong khung nhìn của camera, nhờ `TileProvider` lấy ảnh rồi xếp lại (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 1. Camera của FlutterMap cho biết khung nhìn. TileLayer cùng TileProvider ghép urlTemplate với z, x, y rồi gửi yêu cầu có header User-Agent tới tile.openstreetmap.org. Máy chủ trả ảnh PNG. Bên dưới là ba điều chính sách OSM yêu cầu: attribution luôn hiện, User-Agent nhận diện app, không tải hàng loạt hay dùng ngoại tuyến.">
<defs><marker id="fl34-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl34-1-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="30" width="210" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="64" class="t-b">Camera</text><text x="36" y="90" class="t-small">center, zoom, size</text><text x="36" y="112" class="t-small">tile nào trong khung</text>
<rect x="320" y="30" width="270" height="110" rx="10" class="f-pk-soft s-pk"/><text x="336" y="64" class="t-b">TileLayer + TileProvider</text><text x="336" y="90" class="t-mono">urlTemplate z/x/y</text><text x="336" y="112" class="t-small">header User-Agent</text>
<rect x="670" y="30" width="190" height="110" rx="10" class="f-warn-soft s-warn"/><text x="686" y="64" class="t-b">Server tile OSM</text><text x="686" y="90" class="t-small">cộng đồng tài trợ</text><text x="686" y="112" class="t-small">không cam kết SLA</text>
<path d="M232 70 L318 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl34-1-a)"/><text x="244" y="58" class="t-small">z, x, y</text>
<path d="M592 70 L668 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl34-1-a)"/><text x="600" y="58" class="t-small">GET</text>
<path d="M668 108 L592 108" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl34-1-m)"/><text x="600" y="128" class="t-small">PNG</text>
<text x="20" y="190" class="t-b">Ba điều chính sách OSM đòi (đọc lại 11/2026)</text>
<rect x="20" y="206" width="270" height="120" rx="10" class="f-surface s-muted"/><text x="36" y="236" class="t-b">Attribution</text><text x="36" y="262" class="t-small">© OpenStreetMap luôn hiện,</text><text x="36" y="284" class="t-small">không giấu sau nút bấm</text>
<rect x="305" y="206" width="270" height="120" rx="10" class="f-surface s-muted"/><text x="321" y="236" class="t-b">User-Agent</text><text x="321" y="262" class="t-mono">userAgentPackageName</text><text x="321" y="284" class="t-small">nêu tên app, không để unknown</text>
<rect x="590" y="206" width="270" height="120" rx="10" class="f-surface s-muted"/><text x="606" y="236" class="t-b">Dùng vừa phải</text><text x="606" y="262" class="t-small">không tải hàng loạt,</text><text x="606" y="284" class="t-small">không dùng ngoại tuyến</text>
</svg>
</div>
<figcaption>Hình 1. Yêu cầu tile đi từ camera, qua TileLayer, tới server OSM. Ba điều ở dưới là yêu cầu của chính sách tile.</figcaption>
</figure>

Dữ liệu OpenStreetMap miễn phí, nhưng **server tile công cộng thì không phải hạ tầng vô hạn**: [Tile Usage Policy](https://operations.osmfoundation.org/policies/tiles/) mô tả một dịch vụ do cộng đồng tài trợ, dung lượng có hạn. Tại thời điểm viết (11/2026), trang này đòi hỏi:

- HTTPS tới `tile.openstreetmap.org`, attribution hiện rõ trên bản đồ (không giấu sau nút bấm).
- `User-Agent` riêng nêu tên app; User-Agent mặc định của thư viện bị chặn.
- Tôn trọng header cache, hoặc giữ tile ít nhất 7 ngày.
- Không tải hàng loạt, không dựng kho tile, không dùng ngoại tuyến.

Chính sách nói thẳng: không có SLA, họ có thể chặn không báo trước, và dịch vụ thương mại nên biết quyền truy cập có thể bị rút bất kỳ lúc nào. Với Tới Nơi, lab dùng server này để học vì không cần key. App thật, với hàng triệu lượt mở bản đồ mỗi tháng, phải dùng nhà cung cấp tile trả phí hoặc tile server tự dựng, đặt `urlTemplate` ở một chỗ duy nhất. Cấu hình môi trường của [bài 25](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod) là hằng lúc biên dịch, nằm trong binary; muốn đổi nguồn tile mà không phát hành lại thì phải lấy URL từ server hoặc remote config.

Phần tile của `PickupMap`:

### pickup_map.dart · 34-map-flutter-map/lib/pickup_map.dart (trích)

```dart
/// Mẫu URL tile raster chuẩn của OpenStreetMap (HTTPS, không dùng subdomain).
const String osmTileUrl = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';

/// Tên package của app khách; flutter_map ghép thành
/// `User-Agent: flutter_map (vn.toinoi.customer)`.
const String tileUserAgentPackageName = 'vn.toinoi.customer';

// ...
            TileLayer(
              urlTemplate: osmTileUrl,
              userAgentPackageName: tileUserAgentPackageName,
              tileProvider: widget.tileProvider,
            ),
```

Theo source `flutter_map` 8.3.2, `TileLayer` ghép User-Agent từ `userAgentPackageName` khi không chạy trên web (trình duyệt không cho đổi header này); bỏ tham số thì là `flutter_map (unknown)`.

Hai widget attribution của `flutter_map` khác nhau ở đúng chỗ chính sách quan tâm. `RichAttributionWidget` để chữ trong popup mở bằng nút `i`, và theo source nút được làm mờ còn một nửa khi popup đóng và không có hover, tức gần như luôn mờ trên màn cảm ứng. `SimpleAttributionWidget` luôn hiện chữ, trên nền đặc:

### pickup_map.dart · 34-map-flutter-map/lib/pickup_map.dart (trích)

```dart
            // Luôn hiện, có nền đặc. RichAttributionWidget để chữ trong popup
            // sau nút và làm mờ nút một nửa khi không hover.
            SimpleAttributionWidget(
              source: const Text('OpenStreetMap contributors'),
              onTap: widget.onAttributionTap,
            ),
          ],
```

Widget này tự thêm tiền tố `flutter_map | © ` trước `source`. `onAttributionTap` để app thật mở trang giấy phép bằng `url_launcher`. Cách đọc chính sách này là của ta; trước khi phát hành, hãy đọc lại trang gốc.

## FlutterMap: camera, layer, marker và polyline

`FlutterMap` nhận `options` và danh sách `children`. Các layer xếp chồng như `Stack`, phần tử sau vẽ đè phần tử trước, và mọi layer đọc cùng một **camera** (tâm, zoom, xoay, kích thước) bằng `MapCamera.of(context)` (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 410" role="img" aria-label="Hình 2. Chồng layer trong FlutterMap, từ dưới lên: TileLayer, PolylineLayer, MarkerLayer, SimpleAttributionWidget. Ba layer đầu đọc MapCamera, attribution thì không. Chiếc pin giữa khung nằm ngoài FlutterMap, trong Stack của màn hình, bọc IgnorePointer, và không đọc camera.">
<defs><marker id="fl34-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="40" y="14" width="480" height="62" rx="10" class="f-warn-soft s-warn"/><text x="56" y="40" class="t-b">Pin giữa khung</text><text x="56" y="62" class="t-small">Center + IgnorePointer, ngoài FlutterMap</text>
<text x="545" y="50" class="t-small t-warn">không đọc camera: nó thuộc màn hình</text>
<rect x="20" y="96" width="520" height="304" rx="12" class="f-none s-muted" stroke-dasharray="6 5"/><text x="32" y="114" class="t-small">FlutterMap (children xếp chồng, sau vẽ đè trước)</text>
<rect x="40" y="126" width="480" height="56" rx="10" class="f-warn-soft s-warn"/><text x="56" y="160" class="t-b">4. SimpleAttributionWidget</text><text x="330" y="160" class="t-small">không đọc camera</text>
<rect x="40" y="194" width="480" height="56" rx="10" class="f-pk-soft s-pk"/><text x="56" y="228" class="t-b">3. MarkerLayer: tài xế, điểm đến</text>
<rect x="40" y="262" width="480" height="56" rx="10" class="f-pk-soft s-pk"/><text x="56" y="296" class="t-b">2. PolylineLayer: hai tuyến</text>
<rect x="40" y="330" width="480" height="56" rx="10" class="f-phys-soft s-phys"/><text x="56" y="364" class="t-b">1. TileLayer: tile OSM</text>
<rect x="640" y="130" width="220" height="250" rx="10" class="f-phys-soft s-phys"/><text x="656" y="166" class="t-b">MapCamera</text><text x="656" y="194" class="t-small">center, zoom</text><text x="656" y="216" class="t-small">rotation, size</text><text x="656" y="256" class="t-small">đọc bằng</text><text x="656" y="278" class="t-mono">MapCamera.of(context)</text>
<path d="M638 222 L524 222" class="f-none s-phys" stroke-width="2" marker-end="url(#fl34-2-a)"/>
<path d="M638 290 L524 290" class="f-none s-phys" stroke-width="2" marker-end="url(#fl34-2-a)"/>
<path d="M638 358 L524 358" class="f-none s-phys" stroke-width="2" marker-end="url(#fl34-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Ba layer đầu đọc camera; attribution và pin không đọc, pin nằm ngoài FlutterMap nên đứng yên khi camera đổi.</figcaption>
</figure>

`MapOptions` gom cấu hình bản đồ:

### pickup_map.dart · 34-map-flutter-map/lib/pickup_map.dart (trích)

```dart
        FlutterMap(
          mapController: widget.controller,
          options: MapOptions(
            initialCenter: widget.initialCenter,
            initialZoom: widget.initialZoom,
            minZoom: 11,
            maxZoom: 18,
            // Tâm bản đồ không ra khỏi vùng phục vụ.
            cameraConstraint: CameraConstraint.containCenter(
              bounds: LatLngBounds(area.southWest, area.northEast),
            ),
            // Xoay bản đồ làm "kéo tới điểm đón" khó đoán, tắt đi.
            interactionOptions: const InteractionOptions(
              flags: InteractiveFlag.all & ~InteractiveFlag.rotate,
            ),
            onPositionChanged: _onPositionChanged,
          ),
```

`containCenter` kẹp tâm camera vào hình chữ nhật. Theo source, `move()` kẹp camera trước rồi mới gọi `onPositionChanged`; test thấy kết quả trùng `ServiceArea.clamp` tới 1e-9. Hai cơ chế không thừa: `cameraConstraint` bảo vệ **bản đồ**, còn `ServiceArea` bảo vệ **dữ liệu** không đi qua bản đồ, như vị trí máy hay deep link. Nếu `initialCenter` nằm ngoài constraint, `flutter_map` ở debug dừng bằng assertion "MapCamera is no longer within the cameraConstraint"; test có đúng ca đó, nên màn hình kẹp điểm ban đầu trước khi dựng.

`MapController` là tay lái từ bên ngoài; nút "về điểm đón ban đầu" dùng `move(center, zoom)`, trả `false` nếu camera không đổi hoặc constraint từ chối.

**Marker** là vật chứa widget gắn với toạ độ, không tự là widget. Hai chi tiết hay sai. Một, `alignment` mặc định là `Alignment.center`, nên toạ độ nằm giữa marker; với biểu tượng ghim ta muốn đáy chạm toạ độ, dùng `Alignment.topCenter`. Hai, marker ngoài khung nhìn không được dựng, nên ở zoom 16 điểm đến cách 9 km không có widget nào để tìm. **Polyline** nhận danh sách toạ độ cùng nét vẽ, có thể là nét đứt:

### pickup_screen.dart · 34-map-flutter-map/lib/pickup_screen.dart (trích)

```dart
                    Marker(
                      key: dropoffMarkerKey,
                      point: widget.dropoff,
                      width: 40,
                      height: 40,
                      // Cả marker nằm phía trên điểm đến; đáy cờ chạm điểm.
                      alignment: Alignment.topCenter,
                      rotate: true,
                      child: _Badge(
                        icon: Icons.flag,
                        color: scheme.error,
                        label: 'Điểm đến',
                      ),
                    ),
                  ],
                  polylines: [
                    Polyline(
                      points: fakeRoute(widget.driver, _settled),
                      strokeWidth: 4,
                      color: scheme.tertiary,
                      pattern: StrokePattern.dashed(segments: const [10, 8]),
                    ),
                    Polyline(
                      points: fakeRoute(_settled, widget.dropoff),
                      strokeWidth: 5,
                      color: scheme.primary,
                      borderStrokeWidth: 2,
                      borderColor: scheme.surface,
                    ),
                  ],
```

`rotate: true` giữ marker thẳng đứng nếu bản đồ xoay. Test kiểm vị trí bằng `camera.latLngToScreenOffset`: tâm marker tài xế và đáy marker điểm đến trùng toạ độ, sai số dưới nửa pixel.

## Chọn điểm đón bằng cách kéo bản đồ

Pin đứng yên giữa khung, bản đồ trượt bên dưới, và điểm đón là `MapCamera.center` tại mỗi thời điểm. Pin là widget trong `Stack`, bọc `IgnorePointer` để cử chỉ kéo đi xuyên qua nó xuống bản đồ:

### pickup_map.dart · 34-map-flutter-map/lib/pickup_map.dart (trích)

```dart
        // Pin cố định giữa khung: nó thuộc màn hình, không thuộc toạ độ địa lý.
        // IgnorePointer để cử chỉ kéo đi xuyên qua pin xuống bản đồ.
        const IgnorePointer(child: Center(child: _CenterPin())),
```

Biểu tượng được dịch lên nửa chiều cao bằng `Transform.translate`, để **đáy hộp biểu tượng** trùng tâm khung; test đo đúng điều đó, và đo rằng kéo bắt đầu ngay trên pin vẫn làm bản đồ di chuyển.

Camera đổi liên tục khi kéo, nhưng việc tốn kém như định tuyến, báo giá, về sau là geocoding gọi mạng chỉ nên làm khi bản đồ đứng yên. Ta cần hai nhịp (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 3. Kéo, vuốt nhanh hoặc gọi move làm camera đổi. onPositionChanged chạy mỗi lần camera đổi. Nhánh trên gọi onCenterChanged cập nhật ValueNotifier _live, chỉ thẻ toạ độ và nút xác nhận dựng lại. Nhánh dưới huỷ và đặt lại Timer 250 ms; khi hết giờ gọi onPickupSettled, setState đặt _settled, rồi tính lại tuyến và cước.">
<defs><marker id="fl34-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="105" width="170" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="140" class="t-b">Camera đổi</text><text x="36" y="166" class="t-small">kéo hoặc move()</text>
<rect x="240" y="105" width="230" height="100" rx="10" class="f-pk-soft s-pk"/><text x="256" y="140" class="t-b">onPositionChanged</text><text x="256" y="166" class="t-small">camera, hasGesture</text>
<rect x="540" y="20" width="320" height="100" rx="10" class="f-pk-soft s-pk"/><text x="556" y="52" class="t-b">Mỗi lần camera đổi</text><text x="556" y="78" class="t-mono">_live.value = center</text><text x="556" y="100" class="t-small">chỉ thẻ toạ độ và nút dựng lại</text>
<rect x="540" y="215" width="150" height="115" rx="10" class="f-warn-soft s-warn"/><text x="556" y="245" class="t-b">Timer</text><text x="556" y="271" class="t-small">huỷ, đặt lại</text><text x="556" y="293" class="t-small">250 ms rồi gọi</text>
<rect x="730" y="215" width="130" height="115" rx="10" class="f-pk-soft s-pk"/><text x="744" y="245" class="t-b">setState</text><text x="744" y="271" class="t-mono">_settled</text><text x="744" y="293" class="t-small">tuyến và cước</text>
<path d="M192 155 L238 155" class="f-none s-pk" stroke-width="2" marker-end="url(#fl34-3-a)"/>
<path d="M472 135 L505 135 L505 70 L538 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl34-3-a)"/>
<path d="M472 175 L505 175 L505 272 L538 272" class="f-none s-pk" stroke-width="2" marker-end="url(#fl34-3-a)"/>
<path d="M692 272 L728 272" class="f-none s-pk" stroke-width="2" marker-end="url(#fl34-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Một callback, hai nhịp. Nhánh dưới chỉ chạy một lần khi camera đứng yên 250 ms (giá trị minh hoạ).</figcaption>
</figure>

### pickup_map.dart · 34-map-flutter-map/lib/pickup_map.dart (trích)

```dart
  void _onPositionChanged(MapCamera camera, bool hasGesture) {
    final center = camera.center;
    widget.onCenterChanged?.call(center);
    _settleTimer?.cancel();
    _settleTimer = Timer(widget.settleDelay, () {
      widget.onPickupSettled(center);
    });
  }
```

Vì sao debounce thay vì chờ sự kiện "kết thúc"? Theo source, kéo nhả tay phát `MapEventMoveEnd`, rồi `MapEventFlingAnimationNotStarted` hoặc `MapEventFlingAnimationStart` và `MapEventFlingAnimationEnd`. Cuộn chuột chỉ có `MapEventScrollWheelZoom`, còn `move()` chỉ có `MapEventMove`. Debounce phủ mọi nguồn bằng một `Timer`, cái giá là trễ `settleDelay`. Test vuốt nhanh xác nhận `onCenterChanged` chạy hơn năm lần trong lúc đà trôi, còn `onPickupSettled` đúng một lần, ở điểm cuối.

Phía màn hình dùng hai nhịp đó:

### pickup_screen.dart · 34-map-flutter-map/lib/pickup_screen.dart (trích)

```dart
  // Điểm đầu vào từ ngoài (vị trí máy, deep link) luôn đi qua ServiceArea.
  late final LatLng _start = widget.serviceArea.clamp(widget.initialPickup);

  // Đổi theo từng khung hình kéo; chỉ phần nhỏ của thẻ lắng nghe nó.
  late final ValueNotifier<LatLng> _live = ValueNotifier(_start);

  // Đổi một lần khi bản đồ đứng yên; tuyến và cước tính từ điểm này.
  late LatLng _settled = _start;

  void _onPickupSettled(LatLng pickup) => setState(() => _settled = pickup);
```

`_live` đi qua `ValueListenableBuilder`, nên mỗi khung hình kéo chỉ dựng lại dòng toạ độ và nút xác nhận; `setState` cả màn hình chỉ chạy khi `_settled` đổi. Nút xác nhận chỉ bật khi `center == settled`, để khách không xác nhận điểm mà cước hiển thị còn tính cho điểm cũ. Test kiểm ba pha: đang kéo thì toạ độ đổi, cước giữ nguyên, nút khoá; sau 300 ms cước đổi, nút mở.

## Logic thuần: mét, cước và vùng phục vụ

Mọi tính toán nằm trong `lib/geo.dart`, không import widget, nên unit test không cần `pumpWidget`. Khoảng cách giữa hai toạ độ là công thức haversine:

### geo.dart · 34-map-flutter-map/lib/geo.dart (trích)

```dart
double haversineMeters(
  LatLng a,
  LatLng b, {
  double radius = earthRadiusMeters,
}) {
  _checkCoordinate(a, 'a');
  _checkCoordinate(b, 'b');
  final sinDLat = math.sin((b.latitudeInRad - a.latitudeInRad) / 2);
  final sinDLng = math.sin((b.longitudeInRad - a.longitudeInRad) / 2);
  final h =
      sinDLat * sinDLat +
      sinDLng * sinDLng * math.cos(a.latitudeInRad) * math.cos(b.latitudeInRad);
  return 2 * radius * math.atan2(math.sqrt(h), math.sqrt(1 - h));
}
```

`latlong2` đã có lớp `Distance`, nên ta tự viết lại là có lý do, và test đối chiếu từng lý do. Một, `Distance()` mặc định dùng **Vincenty**, không phải haversine. Hai, `DistanceHaversine` ở bản 0.10.1 luôn dùng bán kính xích đạo 6.378.137 m (`withRadius` không đổi kết quả); với cùng bán kính hàm của lab khớp nó tới 1e-6 m, còn bán kính trung bình 6.371.008,8 m cho số nhỏ hơn 0,11%, chừng 10 m trên tuyến 9,4 km. Ba, `Distance.as` trả 0 âm thầm khi toạ độ là NaN: toạ độ hỏng thành "cách nhau 0 m", cước 12.000 VND. Hàm của lab ném `ArgumentError`.

Haversine là đường chim bay, không phải đường xe chạy, và với cước đó là chỗ dễ sai nhất. Lab dùng `fakeRoute`, tuyến giả hình chữ L thay cho dịch vụ định tuyến của backend. Từ điểm đón mặc định tới điểm đến, chim bay là 9.371 m (cước 45.170 VND), chữ L là 13.132 m (62.094 VND). Chữ L cố ý khắc nghiệt để thấy khoảng chênh; cước chính thức do server tính từ tuyến thật, và thẻ trên màn hình ghi rõ.

Quãng đường của tuyến là tổng các đoạn, làm tròn một lần ở cuối. `estimateFare` chỉ đổi mét sang km rồi gọi `calculateFare`:

### geo.dart · 34-map-flutter-map/lib/geo.dart (trích)

```dart
FareEstimate estimateFare(int meters) {
  if (meters < 0) {
    throw ArgumentError.value(meters, 'meters', 'không được âm');
  }
  if (meters > FarePolicy.maxKm * 1000) {
    return FareOutOfRange(meters);
  }
  return FareQuoted(meters, calculateFare(meters / 1000));
}
```

Kết quả là `sealed class` hai nhánh. Vượt 100 km là chuyện thường khi khách chọn nhầm điểm, nên trả `FareOutOfRange` để giao diện nói "chưa báo giá được" thay vì để `ArgumentError` rơi vào màn hình. Test chạy mọi giá trị mét từ 0 tới 100.000 và so với công thức nguyên của bài 02 viết lại độc lập, vì mét đi qua `double` km rồi về mét nguyên trong `calculateFare`; các mốc 5.000 m, 2.001 m, 100.000 m cho đúng 25.500, 12.005, 453.000 VND.

Vùng phục vụ là hình chữ nhật; `contains` tính cả biên, `clamp` kéo điểm về điểm gần nhất trong vùng:

### geo.dart · 34-map-flutter-map/lib/geo.dart (trích)

```dart
  /// Điểm gần [point] nhất nằm trong vùng; điểm đã nằm trong thì giữ nguyên.
  LatLng clamp(LatLng point) {
    _checkCoordinate(point, 'point');
    return LatLng(
      point.latitude.clamp(south, north),
      point.longitude.clamp(west, east),
    );
  }
```

`ServiceArea` lưu bốn số `double` thay vì hai `LatLng`: với `const` constructor, analyzer báo `const_eval_property_access` vì biểu thức hằng không đọc được `southWest.latitude` để `assert`. Cả `contains` và `clamp` ném `ArgumentError` với NaN, vì `NaN.clamp(south, north)` trả về `north`: toạ độ hỏng thành điểm hợp lệ âm thầm.

## Test không ra mạng

Test của màn hình bản đồ không được phụ thuộc server công cộng, cũng không được tạo tải cho OSM từ máy CI. `TileLayer` nhận `tileProvider` từ ngoài, nên ta thay bằng provider giả: ghi lại URL thư viện sẽ gọi và trả ảnh trong suốt có sẵn trong bộ nhớ.

### support.dart · 34-map-flutter-map/test/support.dart (trích)

```dart
class FakeTileProvider extends TileProvider {
  final requestedUrls = <String>[];

  @override
  ImageProvider getImage(TileCoordinates coordinates, TileLayer options) {
    requestedUrls.add(getTileUrl(coordinates, options));
    return MemoryImage(TileProvider.transparentImage);
  }
}
```

Thay provider vẫn chưa đủ để chứng minh "không có request thật". `flutter_test` thay `HttpClient` toàn cục bằng bản trả mã 400, nên dù dùng provider nào, test cũng không ra mạng thật; test xanh không nói được code **có cố gọi** hay không. Vì vậy lab cài một `HttpOverrides` đếm số lần có mã tạo `HttpClient`, rồi chuyển tiếp cho override của `flutter_test`:

### support.dart · 34-map-flutter-map/test/support.dart (trích)

```dart
class CountingHttpOverrides extends HttpOverrides {
  CountingHttpOverrides(this._inner);

  final HttpOverrides _inner;
  int clientsCreated = 0;

  @override
  HttpClient createHttpClient(SecurityContext? context) {
    clientsCreated++;
    return _inner.createHttpClient(context);
  }
}
```

### pickup_map_test.dart · 34-map-flutter-map/test/pickup_map_test.dart (trích)

```dart
    testWidgets('provider giả nhận đủ yêu cầu tile và không tạo HttpClient', (
      tester,
    ) async {
      final http = installHttpCounter();

      await pumpPickupMap(tester, controller: controller, tileProvider: tiles);
      await tester.pump(const Duration(seconds: 1));

      expect(tiles.requestedUrls, isNotEmpty);
      for (final url in tiles.requestedUrls) {
        expect(url, startsWith('https://tile.openstreetmap.org/16/'));
        expect(url, endsWith('.png'));
      }
      expect(http.clientsCreated, 0);
    });
```

Một test đối chứng dùng `NetworkTileProvider(httpClient: IOClient())`, tắt cache và nuốt lỗi, thấy bộ đếm lớn hơn 0; nhờ vậy con số 0 ở test trên có nghĩa. Giới hạn: bộ đếm chỉ thấy `HttpClient` của `dart:io`.

## So với Google Maps SDK và Mapbox

Sản phẩm thật nhiều khả năng không dừng ở tile cộng đồng; hai lựa chọn phổ biến là `google_maps_flutter` và `mapbox_maps_flutter`. Bài không có lab cho chúng; bảng chỉ ghi điều kiểm được từ trang chính thức (11/2026), giá và hạn mức đổi theo thời gian:

| | `flutter_map` + tile OSM | `google_maps_flutter` | `mapbox_maps_flutter` |
|---|---|---|---|
| Bản mới nhất lúc đọc | 8.3.2 | 2.19.0 (flutter.dev) | 3.34.2 (mapbox.com) |
| Khoá, tài khoản | Thư viện không cần; nhà cung cấp tile có thể cần | API key Google Maps Platform, bật Maps SDK từng nền tảng | Access token, `MapboxOptions.setAccessToken` trước khi tạo `MapWidget` |
| Chi phí | Thư viện miễn phí; tile tuỳ nhà cung cấp | Trang giá ghi SKU "Maps SDK" không giới hạn, không có giá; các SKU khác như Dynamic Maps tính theo mức dùng | Theo người dùng hoạt động hằng tháng (MAU); bậc đầu 25.000 MAU miễn phí |
| Gắn nhà cung cấp | Không, đổi bằng `urlTemplate` | Có | Có |

Nói công bằng, không bên nào "đúng" tuyệt đối. `flutter_map` cho ta chọn nguồn tile và không cần key cho thư viện, đổi lại ta tự lo nguồn tile, attribution và những thứ SDK trọn gói thường kèm như tìm địa điểm hay định tuyến. Google và Mapbox đầy đủ hơn, nhưng cần key hoặc token, tính phí và gắn app với nhà cung cấp. Với Tới Nơi, cách giữ cửa mở là giữ ranh giới: `PickupMap` là nơi duy nhất biết SDK bản đồ, còn `geo.dart` chỉ dùng `LatLng` của `latlong2`. Đổi SDK thì viết lại `PickupMap` và thêm một hàm chuyển `LatLng`; cước, vùng phục vụ và test logic không đổi. Chọn bên nào là quyết định của chủ sản phẩm.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp với bản đồ</span>
<ul>
<li><strong>Quên <code>userAgentPackageName</code>.</strong> Dấu hiệu: log debug cảnh báo, header là <code>flutter_map (unknown)</code>, về sau tile có thể bị chặn. Cách sửa: truyền tên app thật.</li>
<li><strong>Attribution nằm sau nút hoặc bị làm mờ.</strong> Dấu hiệu: không thấy chữ OpenStreetMap cho tới khi chạm, hoặc chữ quá nhạt. Cách sửa: <code>SimpleAttributionWidget</code> có nền đặc.</li>
<li><strong>Pin là <code>Marker</code>, hoặc pin không bọc <code>IgnorePointer</code>.</strong> Dấu hiệu: pin trôi theo bản đồ, hoặc kéo từ giữa màn hình không nhúc nhích. Cách sửa: pin là widget trong <code>Stack</code> ngoài <code>FlutterMap</code>, bọc <code>IgnorePointer</code>; toạ độ đọc từ <code>MapCamera.center</code>.</li>
<li><strong><code>setState</code> cả màn hình hoặc gọi API ở mỗi lần <code>onPositionChanged</code>.</strong> Dấu hiệu: giật khi kéo, hàng chục request mỗi giây. Cách sửa: <code>ValueNotifier</code> cho phần hiển thị, debounce cho việc tốn kém.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/34-map-flutter-map
flutter pub get
flutter analyze
flutter test --reporter expanded
```

Đã kiểm với Flutter 3.47.4 / Dart 3.13.3, `flutter_map` 8.3.2, `latlong2` 0.10.1: `flutter analyze` 0 issue, `flutter test` 51 test xanh. Unit test (16 cho `geo`, 12 cho `ServiceArea`) kiểm haversine đối chiếu `latlong2`, `estimateFare` cho mọi mét 0 tới 100.000 và kẹp vùng ở biên. Widget test (12 cho `PickupMap`, 11 cho `PickupScreen`) kiểm không ra mạng, attribution, pin, kéo và vuốt nhanh, kẹp camera, marker, polyline, thẻ cước.

Mức kiểm chứng của bài theo khung là **trên thiết bị**, nhưng máy viết bài **không có simulator hay emulator**, nên phần làm được là phần tự động nói trên. Chưa làm: chạy trên iOS hoặc Android; cử chỉ thật và hiệu năng; tải tile thật; `main.dart` chỉ được analyzer kiểm. Chưa có vị trí thật của khách (bài tiếp theo), tài xế realtime, geocoding ngược, tuyến thật.

## Bài tập

1. Vị trí máy ở Hà Nội, ngoài vùng phục vụ, được truyền thẳng làm `initialCenter`. Chuyện gì xảy ra ở debug, và sửa ở đâu?
2. Chim bay 5.000 m nhưng tuyến thật 6.500 m: cước ước tính chênh bao nhiêu so với cước theo tuyến? Tính bằng `calculateFare`.
3. Đồng nghiệp tắt cache và bật `silenceExceptions` cho `NetworkTileProvider`, test xanh, rồi kết luận "không ra mạng". Sai ở đâu, và kiểm đúng thế nào?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> <code>flutter_map</code> dừng bằng assertion "MapCamera is no longer within the cameraConstraint" ngay khi dựng. Assertion chỉ chạy ở debug; ở release camera không còn được đảm bảo nằm trong vùng. Sửa bằng <code>serviceArea.clamp(initialPickup)</code> trước khi dựng, như <code>_start</code> trong <code>PickupScreen</code>.</p>
<p><strong>2.</strong> <code>calculateFare(5)</code> cho 25.500 VND, <code>calculateFare(6.5)</code> cho 32.250 VND, chênh 6.750 VND.</p>
<p><strong>3.</strong> Xanh không chứng minh code không gọi mạng: <code>flutter_test</code> chặn request và trả 400, còn lỗi tile bị nuốt. Chạy cùng code trên <code>integration_test</code> hoặc máy thật là request ra thật. Cách đúng là inject provider giả và đếm số lần tạo <code>HttpClient</code> như lab, kèm test đối chứng cho thấy bộ đếm tăng khi dùng provider thật.</p>
</details>

Bài trước: [CI với GitHub Actions: analyze, test, golden và cache pub](/articles/flutter-ci-github-actions-analyze-test-golden-cache). Bài tiếp: [Vị trí: geolocator, quyền foreground/background và cấu hình iOS, Android](/articles/flutter-vi-tri-geolocator-permission-foreground-background).

## Tài liệu gốc

- [OSM Tile Usage Policy](https://operations.osmfoundation.org/policies/tiles/) (operations.osmfoundation.org).
- [flutter_map](https://pub.dev/packages/flutter_map) (pub.dev) và [tài liệu flutter_map](https://docs.fleaflet.dev/) (docs.fleaflet.dev).
- [latlong2](https://pub.dev/packages/latlong2) (pub.dev).
- [google_maps_flutter](https://pub.dev/packages/google_maps_flutter) (pub.dev) và [Google Maps Platform billing](https://developers.google.com/maps/billing-and-pricing/overview) (developers.google.com).
- [mapbox_maps_flutter](https://pub.dev/packages/mapbox_maps_flutter) (pub.dev) và [Mapbox pricing](https://www.mapbox.com/pricing) (mapbox.com).
- [Testing Flutter apps](https://docs.flutter.dev/testing/overview) và [`HttpOverrides`](https://api.flutter.dev/flutter/dart-io/HttpOverrides-class.html) (docs.flutter.dev, api.flutter.dev).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart';

import 'geo.dart';

/// Mẫu URL tile raster chuẩn của OpenStreetMap (HTTPS, không dùng subdomain).
const String osmTileUrl = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';

/// Tên package của app khách; flutter_map ghép thành
/// `User-Agent: flutter_map (vn.toinoi.customer)`.
const String tileUserAgentPackageName = 'vn.toinoi.customer';

/// Bản đồ chọn điểm đón: marker cố định ở giữa, người dùng kéo bản đồ.
///
/// Widget này chỉ báo toạ độ tâm bản đồ qua [onCenterChanged] (mỗi lần đổi)
/// và [onPickupSettled] (sau khi bản đồ đứng yên [settleDelay]). Việc tính
/// cước và vẽ tuyến thuộc về màn hình chứa nó.
class PickupMap extends StatefulWidget {
  const PickupMap({
    super.key,
    required this.initialCenter,
    required this.serviceArea,
    required this.onPickupSettled,
    this.onCenterChanged,
    this.markers = const [],
    this.polylines = const [],
    this.tileProvider,
    this.controller,
    this.initialZoom = 16,
    this.settleDelay = const Duration(milliseconds: 250),
    this.onAttributionTap,
  });

  final LatLng initialCenter;
  final ServiceArea serviceArea;
  final ValueChanged<LatLng> onPickupSettled;
  final ValueChanged<LatLng>? onCenterChanged;
  final List<Marker> markers;
  final List<Polyline> polylines;

  /// `null` dùng [NetworkTileProvider] mặc định của flutter_map. Test truyền
  /// provider giả để không có request nào ra mạng.
  final TileProvider? tileProvider;
  final MapController? controller;
  final double initialZoom;
  final Duration settleDelay;

  /// Mở trang giấy phép dữ liệu; app thật nối với `url_launcher`.
  final VoidCallback? onAttributionTap;

  @override
  State<PickupMap> createState() => _PickupMapState();
}

class _PickupMapState extends State<PickupMap> {
  Timer? _settleTimer;

  @override
  void dispose() {
    _settleTimer?.cancel();
    super.dispose();
  }

  void _onPositionChanged(MapCamera camera, bool hasGesture) {
    final center = camera.center;
    widget.onCenterChanged?.call(center);
    _settleTimer?.cancel();
    _settleTimer = Timer(widget.settleDelay, () {
      widget.onPickupSettled(center);
    });
  }

  @override
  Widget build(BuildContext context) {
    final area = widget.serviceArea;
    return Stack(
      children: [
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
          children: [
            TileLayer(
              urlTemplate: osmTileUrl,
              userAgentPackageName: tileUserAgentPackageName,
              tileProvider: widget.tileProvider,
            ),
            PolylineLayer(polylines: widget.polylines),
            MarkerLayer(markers: widget.markers),
            // Luôn hiện, có nền đặc. RichAttributionWidget để chữ trong popup
            // sau nút và làm mờ nút một nửa khi không hover.
            SimpleAttributionWidget(
              source: const Text('OpenStreetMap contributors'),
              onTap: widget.onAttributionTap,
            ),
          ],
        ),
        // Pin cố định giữa khung: nó thuộc màn hình, không thuộc toạ độ địa lý.
        // IgnorePointer để cử chỉ kéo đi xuyên qua pin xuống bản đồ.
        const IgnorePointer(child: Center(child: _CenterPin())),
      ],
    );
  }
}

class _CenterPin extends StatelessWidget {
  const _CenterPin();

  static const double _size = 48;

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme.primary;
    return Semantics(
      label: 'Điểm đón là vị trí giữa bản đồ',
      child: Transform.translate(
        // Đáy biểu tượng nằm đúng tâm khung, tức đúng `MapCamera.center`.
        offset: const Offset(0, -_size / 2),
        child: Icon(
          Icons.location_on,
          key: const ValueKey('center-pin'),
          size: _size,
          color: color,
        ),
      ),
    );
  }
}

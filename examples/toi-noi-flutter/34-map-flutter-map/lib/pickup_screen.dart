import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart';

import 'demo_data.dart';
import 'geo.dart';
import 'pickup_map.dart';

const driverMarkerKey = ValueKey('driver-marker');
const dropoffMarkerKey = ValueKey('dropoff-marker');
const myLocationButtonKey = ValueKey('my-location');
const confirmButtonKey = ValueKey('confirm-pickup');

/// Màn hình chọn điểm đón: kéo bản đồ, xem tuyến và cước ước tính.
class PickupScreen extends StatefulWidget {
  const PickupScreen({
    super.key,
    this.initialPickup = demoPickup,
    this.dropoff = demoDropoff,
    this.driver = demoDriver,
    this.serviceArea = demoServiceArea,
    this.tileProvider,
    this.onConfirm,
  });

  final LatLng initialPickup;
  final LatLng dropoff;
  final LatLng driver;
  final ServiceArea serviceArea;
  final TileProvider? tileProvider;
  final ValueChanged<LatLng>? onConfirm;

  @override
  State<PickupScreen> createState() => _PickupScreenState();
}

class _PickupScreenState extends State<PickupScreen> {
  final _controller = MapController();

  // Điểm đầu vào từ ngoài (vị trí máy, deep link) luôn đi qua ServiceArea.
  late final LatLng _start = widget.serviceArea.clamp(widget.initialPickup);

  // Đổi theo từng khung hình kéo; chỉ phần nhỏ của thẻ lắng nghe nó.
  late final ValueNotifier<LatLng> _live = ValueNotifier(_start);

  // Đổi một lần khi bản đồ đứng yên; tuyến và cước tính từ điểm này.
  late LatLng _settled = _start;

  @override
  void dispose() {
    _live.dispose();
    _controller.dispose();
    super.dispose();
  }

  void _onPickupSettled(LatLng pickup) => setState(() => _settled = pickup);

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(title: const Text('Chọn điểm đón')),
      body: Column(
        children: [
          Expanded(
            child: Stack(
              children: [
                PickupMap(
                  controller: _controller,
                  initialCenter: _start,
                  serviceArea: widget.serviceArea,
                  tileProvider: widget.tileProvider,
                  onCenterChanged: (center) => _live.value = center,
                  onPickupSettled: _onPickupSettled,
                  markers: [
                    Marker(
                      key: driverMarkerKey,
                      point: widget.driver,
                      width: 40,
                      height: 40,
                      rotate: true,
                      child: _Badge(
                        icon: Icons.two_wheeler,
                        color: scheme.tertiary,
                        label: 'Tài xế',
                      ),
                    ),
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
                ),
                Positioned(
                  right: 12,
                  top: 12,
                  child: FloatingActionButton.small(
                    key: myLocationButtonKey,
                    tooltip: 'Về điểm đón ban đầu',
                    onPressed: () => _controller.move(_start, 16),
                    child: const Icon(Icons.my_location),
                  ),
                ),
              ],
            ),
          ),
          _QuoteCard(
            live: _live,
            settled: _settled,
            driver: widget.driver,
            dropoff: widget.dropoff,
            serviceArea: widget.serviceArea,
            onConfirm: widget.onConfirm,
          ),
        ],
      ),
    );
  }
}

class _Badge extends StatelessWidget {
  const _Badge({required this.icon, required this.color, required this.label});

  final IconData icon;
  final Color color;
  final String label;

  @override
  Widget build(BuildContext context) => Semantics(
    label: label,
    child: DecoratedBox(
      decoration: BoxDecoration(
        color: color,
        shape: BoxShape.circle,
        border: Border.all(color: Colors.white, width: 2),
      ),
      child: Icon(icon, color: Colors.white, size: 22),
    ),
  );
}

class _QuoteCard extends StatelessWidget {
  const _QuoteCard({
    required this.live,
    required this.settled,
    required this.driver,
    required this.dropoff,
    required this.serviceArea,
    required this.onConfirm,
  });

  final ValueListenable<LatLng> live;
  final LatLng settled;
  final LatLng driver;
  final LatLng dropoff;
  final ServiceArea serviceArea;
  final ValueChanged<LatLng>? onConfirm;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            ValueListenableBuilder<LatLng>(
              valueListenable: live,
              builder: (context, center, _) => Text(
                'Điểm đón: ${center.latitude.toStringAsFixed(5)}, '
                '${center.longitude.toStringAsFixed(5)}',
                key: const ValueKey('live-coordinates'),
                style: textTheme.titleMedium,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              'Tài xế cách điểm đón khoảng '
              '${routeMeters(fakeRoute(driver, settled))} m',
              key: const ValueKey('driver-distance'),
            ),
            _FareLine(
              pickup: settled,
              dropoff: dropoff,
              serviceArea: serviceArea,
            ),
            const SizedBox(height: 12),
            ValueListenableBuilder<LatLng>(
              valueListenable: live,
              builder: (context, center, _) => FilledButton(
                key: confirmButtonKey,
                // Chỉ cho xác nhận khi bản đồ đã đứng yên và cước đã khớp.
                onPressed: center == settled && onConfirm != null
                    ? () => onConfirm!(settled)
                    : null,
                child: const Text('Xác nhận điểm đón'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _FareLine extends StatelessWidget {
  const _FareLine({
    required this.pickup,
    required this.dropoff,
    required this.serviceArea,
  });

  final LatLng pickup;
  final LatLng dropoff;
  final ServiceArea serviceArea;

  @override
  Widget build(BuildContext context) {
    if (!serviceArea.contains(dropoff)) {
      return const Text(
        'Điểm đến ngoài vùng phục vụ',
        key: ValueKey('fare-line'),
      );
    }
    final meters = routeMeters(fakeRoute(pickup, dropoff));
    final km = (meters / 1000).toStringAsFixed(1);
    final text = switch (estimateFare(meters)) {
      FareQuoted(:final fare) =>
        'Ước tính $km km, $fare (cước chính thức do server tính)',
      FareOutOfRange() => 'Quãng đường $km km vượt 100 km, chưa báo giá được',
    };
    return Text(text, key: const ValueKey('fare-line'));
  }
}

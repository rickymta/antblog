import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'driver_location_controller.dart';
import 'driver_location_policy.dart';

/// Mã chuyến dùng cho màn hình thử.
const demoTripId = 'T-DEMO';

/// Màn hình thử chính sách: chọn trạng thái chuyến và online/offline rồi xem
/// app tài xế có được gửi vị trí nền không. Khi có [controller], mỗi lần đổi
/// còn bật hoặc tắt luồng vị trí thật (để thử trên thiết bị).
class DriverApp extends StatelessWidget {
  const DriverApp({super.key, this.controller, this.onRequestPermission});

  final DriverLocationController? controller;
  final Future<void> Function()? onRequestPermission;

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Tới Nơi Tài xế',
    home: PolicyScreen(
      controller: controller,
      onRequestPermission: onRequestPermission,
    ),
    theme: ThemeData(colorSchemeSeed: Colors.teal),
  );
}

class PolicyScreen extends StatefulWidget {
  const PolicyScreen({
    super.key,
    this.policy = const DriverLocationPolicy(),
    this.controller,
    this.onRequestPermission,
  });

  final DriverLocationPolicy policy;
  final DriverLocationController? controller;
  final Future<void> Function()? onRequestPermission;

  @override
  State<PolicyScreen> createState() => _PolicyScreenState();
}

class _PolicyScreenState extends State<PolicyScreen> {
  TripStatus _status = TripStatus.searching;
  bool _online = true;

  void _change({TripStatus? status, bool? online}) {
    setState(() {
      _status = status ?? _status;
      _online = online ?? _online;
    });
    widget.controller?.onStateChanged(
      tripId: demoTripId,
      status: _status,
      driverOnline: _online,
    );
  }

  @override
  Widget build(BuildContext context) {
    final allowed = widget.policy.allowsBackgroundSharing(
      tripStatus: _status,
      driverOnline: _online,
    );
    return Scaffold(
      appBar: AppBar(title: const Text('Vị trí nền của tài xế')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            DropdownButton<TripStatus>(
              key: const Key('status-dropdown'),
              value: _status,
              items: [
                for (final s in TripStatus.values)
                  DropdownMenuItem(value: s, child: Text(s.label)),
              ],
              onChanged: (s) => _change(status: s),
            ),
            SwitchListTile(
              key: const Key('online-switch'),
              title: const Text('Tài xế online'),
              value: _online,
              onChanged: (v) => _change(online: v),
            ),
            if (widget.onRequestPermission != null)
              OutlinedButton(
                key: const Key('request-permission'),
                onPressed: widget.onRequestPermission,
                child: const Text('Xin quyền vị trí'),
              ),
            const SizedBox(height: 16),
            Text(
              allowed ? 'Gửi vị trí nền: CÓ' : 'Gửi vị trí nền: KHÔNG',
              key: const Key('decision'),
              style: Theme.of(context).textTheme.titleLarge,
            ),
          ],
        ),
      ),
    );
  }
}

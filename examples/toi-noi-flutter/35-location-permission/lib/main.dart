import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'geolocator_location_source.dart';
import 'location_access_controller.dart';
import 'location_models.dart';
import 'pickup_location_panel.dart';
import 'position_tracker.dart';

/// `flutter run --dart-define=APP_ROLE=driver` để thử vai tài xế.
const _roleName = String.fromEnvironment('APP_ROLE', defaultValue: 'customer');

void main() {
  final role = _roleName == 'driver' ? AppRole.driver : AppRole.customer;
  runApp(LocationLabApp(role: role));
}

class LocationLabApp extends StatefulWidget {
  const LocationLabApp({super.key, required this.role});

  final AppRole role;

  @override
  State<LocationLabApp> createState() => _LocationLabAppState();
}

class _LocationLabAppState extends State<LocationLabApp>
    with WidgetsBindingObserver {
  static const _source = GeolocatorLocationSource();
  late final LocationAccessController _access;
  late final PositionTracker _tracker;

  @override
  void initState() {
    super.initState();
    _access = LocationAccessController(
      source: _source,
      store: MemoryAskedStore(),
      role: widget.role,
    );
    _tracker = PositionTracker(source: _source, access: _access)
      ..setProfile(profileFor(widget.role, null));
    WidgetsBinding.instance.addObserver(this);
    _access.refresh();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    // Người dùng có thể đã đổi quyền trong Cài đặt.
    if (state == AppLifecycleState.resumed) {
      _access.refresh();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _tracker.dispose();
    _access.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: Text('Vị trí · ${widget.role.name}')),
        body: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              PickupLocationPanel(
                controller: _access,
                tripActive: widget.role == AppRole.driver,
              ),
              const SizedBox(height: 16),
              ValueListenableBuilder<LocationFix?>(
                valueListenable: _tracker.lastFix,
                builder: (context, fix, _) =>
                    Text(fix == null ? 'Chưa có vị trí' : fix.toString()),
              ),
              // Dùng TripStatus để nhắc profile phụ thuộc trạng thái chuyến.
              Text(
                'Profile khi đang ${TripStatus.inTransit.label}: '
                '${profileFor(widget.role, TripStatus.inTransit).name}',
              ),
            ],
          ),
        ),
      ),
    );
  }
}

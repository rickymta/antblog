import 'dart:async';

import 'package:flutter/material.dart';

import 'toi_noi_40.dart';

/// `--dart-define=BATTERY_BRIDGE=channel` chọn MethodChannel thuần;
/// mặc định là Pigeon.
const String _bridge = String.fromEnvironment(
  'BATTERY_BRIDGE',
  defaultValue: 'pigeon',
);

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final BatteryGateway gateway = _bridge == 'channel'
      ? MethodChannelBatteryGateway()
      : PigeonBatteryGateway();
  runApp(DriverBatteryApp(gateway: gateway));
}

class DriverBatteryApp extends StatelessWidget {
  const DriverBatteryApp({super.key, required this.gateway});

  final BatteryGateway gateway;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi tài xế',
      theme: ThemeData(colorSchemeSeed: Colors.teal),
      home: BatteryPanel(gateway: gateway),
    );
  }
}

/// Hiện pin và khoảng cách gửi vị trí đang áp dụng.
class BatteryPanel extends StatefulWidget {
  const BatteryPanel({super.key, required this.gateway});

  final BatteryGateway gateway;

  @override
  State<BatteryPanel> createState() => _BatteryPanelState();
}

class _BatteryPanelState extends State<BatteryPanel> {
  BatteryStatus? _status;
  String? _error;
  StreamSubscription<BatteryStatus>? _subscription;

  @override
  void initState() {
    super.initState();
    _subscription = widget.gateway.changes.listen(
      (status) => setState(() {
        _status = status;
        _error = null;
      }),
    );
    unawaited(_load());
  }

  Future<void> _load() async {
    try {
      final status = await widget.gateway.read();
      if (!mounted || _status != null) return;
      setState(() => _status = status);
    } on BatteryUnavailableException catch (e) {
      if (!mounted) return;
      setState(() => _error = e.code);
    }
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final status = _status;
    final interval = locationReportInterval(status);
    return Scaffold(
      appBar: AppBar(title: const Text('Pin và tần suất gửi vị trí')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (status != null) ...[
              Text('Mức pin: ${status.levelPercent}%'),
              Text('Đang sạc: ${status.isCharging ? 'có' : 'không'}'),
              Text('Tiết kiệm pin: ${status.isPowerSaveMode ? 'bật' : 'tắt'}'),
            ] else
              Text(
                _error == null
                    ? 'Đang đọc pin...'
                    : 'Không đọc được pin ($_error)',
              ),
            const SizedBox(height: 16),
            Text('Gửi vị trí mỗi ${interval.inSeconds} giây'),
          ],
        ),
      ),
    );
  }
}

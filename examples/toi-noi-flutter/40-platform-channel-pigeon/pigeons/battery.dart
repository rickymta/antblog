// Định nghĩa giao diện Dart <-> native cho app tài xế. File này nằm ngoài lib/
// vì chỉ pigeon đọc nó; chạy: dart run pigeon --input pigeons/battery.dart
import 'package:pigeon/pigeon.dart';

@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/src/generated/battery_api.g.dart',
    dartPackageName: 'toi_noi_40',
    kotlinOut:
        'native-config/android/app/src/main/kotlin/vn/toinoi/toi_noi_40/BatteryApi.g.kt',
    kotlinOptions: KotlinOptions(package: 'vn.toinoi.toi_noi_40'),
    swiftOut: 'native-config/ios/Runner/BatteryApi.g.swift',
  ),
)
/// Ảnh chụp trạng thái pin tại một thời điểm.
class BatteryStatusMessage {
  BatteryStatusMessage({
    required this.levelPercent,
    required this.isCharging,
    required this.isPowerSaveMode,
  });

  /// 0..100.
  int levelPercent;
  bool isCharging;

  /// Android: chế độ tiết kiệm pin; iOS: Low Power Mode.
  bool isPowerSaveMode;
}

/// Dart gọi sang native.
@HostApi()
abstract class BatteryHostApi {
  BatteryStatusMessage getBatteryStatus();
}

/// Native gọi sang Dart.
@FlutterApi()
abstract class BatteryFlutterApi {
  @asyncCallback
  void onBatteryChanged(BatteryStatusMessage status);
}

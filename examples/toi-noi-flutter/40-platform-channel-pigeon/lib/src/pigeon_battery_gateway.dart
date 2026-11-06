import 'dart:async';

import 'package:flutter/services.dart';

import 'battery_gateway.dart';
import 'battery_status.dart';
import 'generated/battery_api.g.dart';

/// Cùng hợp đồng với [MethodChannelBatteryGateway] nhưng tên channel, tên
/// method, codec và kiểu dữ liệu do Pigeon sinh từ `pigeons/battery.dart`.
final class PigeonBatteryGateway implements BatteryGateway, BatteryFlutterApi {
  PigeonBatteryGateway({BinaryMessenger? binaryMessenger})
    : _messenger = binaryMessenger,
      _host = BatteryHostApi(binaryMessenger: binaryMessenger) {
    BatteryFlutterApi.setUp(this, binaryMessenger: binaryMessenger);
  }

  final BinaryMessenger? _messenger;
  final BatteryHostApi _host;
  final StreamController<BatteryStatus> _changes =
      StreamController<BatteryStatus>.broadcast();

  @override
  Stream<BatteryStatus> get changes => _changes.stream;

  @override
  Future<BatteryStatus> read() async {
    try {
      return _fromMessage(await _host.getBatteryStatus());
    } on PlatformException catch (e) {
      // Gồm cả lỗi do native ném và `channel-error` khi chưa có handler.
      throw BatteryUnavailableException(e.code, e.message);
    } on FormatException catch (e) {
      throw BatteryUnavailableException('bad-payload', e.message);
    }
  }

  /// Pigeon gọi hàm này khi native gọi `BatteryFlutterApi.onBatteryChanged`.
  @override
  Future<void> onBatteryChanged(BatteryStatusMessage status) async {
    _changes.add(_fromMessage(status));
  }

  BatteryStatus _fromMessage(BatteryStatusMessage message) =>
      BatteryStatus.checked(
        levelPercent: message.levelPercent,
        isCharging: message.isCharging,
        isPowerSaveMode: message.isPowerSaveMode,
      );

  @override
  void dispose() {
    BatteryFlutterApi.setUp(null, binaryMessenger: _messenger);
    unawaited(_changes.close());
  }
}

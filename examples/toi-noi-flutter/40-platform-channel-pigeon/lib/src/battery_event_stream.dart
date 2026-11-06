import 'package:flutter/services.dart';

import 'battery_status.dart';

const String batteryEventsChannelName = 'vn.toinoi.driver/battery_events';

/// Dòng sự kiện một chiều native sang Dart. Khác `onBatteryChanged`: native chỉ
/// cần giữ receiver khi có người nghe, vì [EventChannel] báo `listen` và
/// `cancel` cho native. Lab chưa có code native cho channel này.
Stream<BatteryStatus> batteryEvents({BinaryMessenger? binaryMessenger}) {
  final channel = EventChannel(
    batteryEventsChannelName,
    const StandardMethodCodec(),
    binaryMessenger,
  );
  return channel.receiveBroadcastStream().map((event) {
    if (event is! Map<Object?, Object?>) {
      throw FormatException('Sự kiện pin phải là Map', event);
    }
    return BatteryStatus.fromMap(event);
  });
}

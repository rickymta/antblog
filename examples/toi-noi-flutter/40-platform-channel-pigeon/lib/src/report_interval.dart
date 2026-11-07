import 'dart:async';

import 'battery_gateway.dart';
import 'battery_status.dart';

/// Các mốc tần suất gửi vị trí. Số giây là số minh hoạ, không phải chính sách
/// của sản phẩm thật.
abstract final class ReportInterval {
  static const Duration normal = Duration(seconds: 5);
  static const Duration reduced = Duration(seconds: 10);
  static const Duration minimal = Duration(seconds: 30);
}

const int _lowLevelPercent = 20;
const int _mediumLevelPercent = 50;

/// Chọn khoảng cách giữa hai lần gửi vị trí theo pin.
///
/// `status == null` nghĩa là không đọc được pin: chọn mốc giữa, không đoán
/// bừa theo hướng nào. Chế độ tiết kiệm pin thắng việc đang sạc vì đó là ý
/// người dùng đã bật.
Duration locationReportInterval(BatteryStatus? status) {
  if (status == null) return ReportInterval.reduced;
  if (status.isPowerSaveMode) return ReportInterval.minimal;
  if (status.isCharging) return ReportInterval.normal;
  if (status.levelPercent < _lowLevelPercent) return ReportInterval.minimal;
  if (status.levelPercent < _mediumLevelPercent) return ReportInterval.reduced;
  return ReportInterval.normal;
}

/// Dòng khoảng cách gửi vị trí: giá trị đầu từ `read()`, các giá trị sau từ
/// `changes`. Chỉ phát khi khoảng cách đổi.
///
/// Nếu native đã báo một thay đổi trước khi `read()` trả về, kết quả `read()`
/// cũ hơn nên bị bỏ.
Stream<Duration> watchReportInterval(BatteryGateway gateway) {
  late final StreamController<Duration> controller;
  StreamSubscription<BatteryStatus>? subscription;
  controller = StreamController<Duration>(
    onListen: () {
      var sawChange = false;
      subscription = gateway.changes.listen((status) {
        sawChange = true;
        controller.add(locationReportInterval(status));
      });
      Future<void> publishInitial() async {
        BatteryStatus? status;
        try {
          status = await gateway.read();
        } on BatteryUnavailableException {
          status = null;
        }
        if (!sawChange && !controller.isClosed) {
          controller.add(locationReportInterval(status));
        }
      }

      unawaited(publishInitial());
    },
    onCancel: () => subscription?.cancel(),
  );
  return controller.stream.distinct();
}

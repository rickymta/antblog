import 'battery_status.dart';

/// Ranh giới tới native: phần còn lại của app chỉ biết interface này, không
/// biết bên dưới là MethodChannel hay Pigeon.
abstract interface class BatteryGateway {
  /// Đọc pin ngay bây giờ. Ném [BatteryUnavailableException] khi không đọc được.
  Future<BatteryStatus> read();

  /// Native chủ động báo khi pin hoặc chế độ tiết kiệm pin đổi.
  Stream<BatteryStatus> get changes;

  void dispose();
}

/// Không đọc được pin: máy không có pin (simulator), native báo lỗi, chưa có
/// code native đăng ký, hoặc native trả dữ liệu sai.
final class BatteryUnavailableException implements Exception {
  const BatteryUnavailableException(this.code, [this.message]);

  /// Mã lỗi của native (ví dụ `UNAVAILABLE`), hoặc mã do Dart đặt:
  /// `missing-plugin`, `bad-payload`, `null-reply`.
  final String code;
  final String? message;

  @override
  String toString() => 'BatteryUnavailableException($code, $message)';
}

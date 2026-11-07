import 'dart:convert';

/// Kết quả của việc đọc một payload push.
sealed class PushDecision {
  const PushDecision();
}

/// Payload hợp lệ: mở màn hình chuyến [tripId].
final class OpenTrip extends PushDecision {
  const OpenTrip(this.tripId);

  final String tripId;

  /// Đường dẫn go_router. [PushRouter.tripIdPattern] đã loại mọi ký tự có thể
  /// làm vỡ đường dẫn nên không cần mã hoá lại.
  String get location => '/trip/$tripId';

  /// Chuỗi lưu trong thông báo cục bộ: chỉ đúng hai trường ta hiểu.
  String toPayload() =>
      jsonEncode({'type': PushRouter.tripUpdateType, 'tripId': tripId});

  @override
  bool operator ==(Object other) => other is OpenTrip && other.tripId == tripId;

  @override
  int get hashCode => tripId.hashCode;

  @override
  String toString() => 'OpenTrip($tripId)';
}

/// Payload lạ: không làm gì cả. [reason] chỉ mô tả lý do, không chứa giá trị.
final class IgnorePush extends PushDecision {
  const IgnorePush(this.reason);

  final String reason;

  @override
  String toString() => 'IgnorePush($reason)';
}

/// Chuyển payload `{"type":"trip_update","tripId":"..."}` thành đường dẫn.
///
/// Hàm thuần: không đọc plugin, không đụng router, nên test được bằng map.
/// Payload tới từ ngoài app (qua máy chủ push, hoặc từ một thông báo còn nằm
/// trên máy từ phiên bản cũ) nên được kiểm như mọi dữ liệu vào khác.
final class PushRouter {
  const PushRouter();

  static const tripUpdateType = 'trip_update';

  /// Mã chuyến: chữ, số, gạch nối, gạch dưới, tối đa 64 ký tự, bắt đầu bằng
  /// chữ hoặc số. Không có `/`, `.`, `?`, `%`, khoảng trắng.
  static final tripIdPattern = RegExp(r'^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$');

  PushDecision decide(Map<String, Object?>? data) {
    if (data == null || data.isEmpty) {
      return const IgnorePush('payload rỗng');
    }
    final type = data['type'];
    if (type is! String) {
      return const IgnorePush('thiếu type');
    }
    if (type != tripUpdateType) {
      return const IgnorePush('type không được hỗ trợ');
    }
    final tripId = data['tripId'];
    if (tripId is! String || !tripIdPattern.hasMatch(tripId)) {
      return const IgnorePush('tripId thiếu hoặc sai định dạng');
    }
    return OpenTrip(tripId);
  }

  /// Giải mã chuỗi payload của thông báo cục bộ rồi áp dụng [decide].
  PushDecision decideFromPayload(String? payload) {
    if (payload == null || payload.isEmpty) {
      return const IgnorePush('payload rỗng');
    }
    final Object? decoded;
    try {
      decoded = jsonDecode(payload);
    } on FormatException {
      return const IgnorePush('payload không phải JSON');
    }
    if (decoded is! Map<String, Object?>) {
      return const IgnorePush('payload không phải object');
    }
    return decide(decoded);
  }

  /// Tóm tắt để ghi log: chỉ có tên khoá, không có giá trị nào.
  static String safeSummary(Map<String, Object?> data) {
    final keys = data.keys.toList()..sort();
    return 'push data keys=[${keys.join(', ')}]';
  }
}

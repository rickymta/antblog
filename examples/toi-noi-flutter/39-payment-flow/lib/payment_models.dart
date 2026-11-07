import 'package:toi_noi_core/toi_noi_core.dart';

/// Trạng thái của một phiên thanh toán theo SERVER. Đây là nguồn sự thật duy
/// nhất: không có trạng thái nào được suy ra từ deep link hay từ trình duyệt.
enum PaymentStatus {
  pending,
  succeeded,
  failed,
  cancelled,
  expired;

  /// Đã chốt: server sẽ không đổi nữa.
  bool get isFinal => this != pending;

  static PaymentStatus parse(Object? raw) {
    for (final status in values) {
      if (status.name == raw) return status;
    }
    throw FormatException('Trạng thái thanh toán lạ: $raw');
  }
}

/// Yêu cầu thanh toán của client: chuyến nào, bao nhiêu tiền, và khoá
/// idempotency của lần thử này. Khoá được sinh MỘT lần rồi giữ qua mọi lần thử
/// lại của cùng một yêu cầu.
final class PaymentRequest {
  const PaymentRequest({
    required this.tripId,
    required this.amount,
    required this.idempotencyKey,
  });

  factory PaymentRequest.fromJson(Map<String, Object?> json) => PaymentRequest(
    tripId: json['tripId']! as String,
    amount: Money.fromVnd(json['amountVnd']! as int),
    idempotencyKey: json['idempotencyKey']! as String,
  );

  final String tripId;
  final Money amount;
  final String idempotencyKey;

  Map<String, Object?> toJson() => {
    'tripId': tripId,
    'amountVnd': amount.vnd,
    'idempotencyKey': idempotencyKey,
  };

  @override
  bool operator ==(Object other) =>
      other is PaymentRequest &&
      other.tripId == tripId &&
      other.amount == amount &&
      other.idempotencyKey == idempotencyKey;

  @override
  int get hashCode => Object.hash(tripId, amount, idempotencyKey);
}

/// Phiên thanh toán do server tạo: mã, địa chỉ trang cổng và hạn.
final class PaymentSession {
  const PaymentSession({
    required this.id,
    required this.tripId,
    required this.amount,
    required this.checkoutUrl,
    required this.expiresAt,
  });

  factory PaymentSession.fromJson(Map<String, Object?> json) => PaymentSession(
    id: json['id']! as String,
    tripId: json['tripId']! as String,
    amount: Money.fromVnd(json['amountVnd']! as int),
    checkoutUrl: Uri.parse(json['checkoutUrl']! as String),
    expiresAt: DateTime.parse(json['expiresAt']! as String).toUtc(),
  );

  final String id;
  final String tripId;
  final Money amount;
  final Uri checkoutUrl;
  final DateTime expiresAt;

  Map<String, Object?> toJson() => {
    'id': id,
    'tripId': tripId,
    'amountVnd': amount.vnd,
    'checkoutUrl': checkoutUrl.toString(),
    'expiresAt': expiresAt.toIso8601String(),
  };

  @override
  bool operator ==(Object other) =>
      other is PaymentSession &&
      other.id == id &&
      other.tripId == tripId &&
      other.amount == amount &&
      other.checkoutUrl == checkoutUrl &&
      other.expiresAt == expiresAt;

  @override
  int get hashCode => Object.hash(id, tripId, amount, checkoutUrl, expiresAt);
}

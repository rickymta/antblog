import 'dart:convert';

import 'payment_models.dart';

/// Phần của một lần thanh toán phải sống sót qua việc app bị kill: yêu cầu
/// (kèm idempotency key) và, nếu đã tạo xong, phiên của server.
final class PendingPayment {
  const PendingPayment(this.request, [this.session]);

  factory PendingPayment.fromJson(Map<String, Object?> json) => PendingPayment(
    PaymentRequest.fromJson(json['request']! as Map<String, Object?>),
    json['session'] == null
        ? null
        : PaymentSession.fromJson(json['session']! as Map<String, Object?>),
  );

  final PaymentRequest request;
  final PaymentSession? session;

  Map<String, Object?> toJson() => {
    'request': request.toJson(),
    'session': session?.toJson(),
  };
}

/// Nơi giữ [PendingPayment] qua các lần mở app. Hiện thực thật dùng drift hoặc
/// secure storage (bài 22); giá trị chỉ là một chuỗi JSON, không có bí mật.
abstract interface class PaymentStore {
  Future<PendingPayment?> load();
  Future<void> save(PendingPayment pending);
  Future<void> clear();
}

/// Kho trong bộ nhớ. Lưu dạng chuỗi JSON để test chứng minh dữ liệu đi qua
/// được một kho bền thật; hai `PaymentFlow` dùng chung một đối tượng này mô
/// phỏng việc app bị kill rồi mở lại.
final class InMemoryPaymentStore implements PaymentStore {
  String? _json;

  /// Chuỗi đang lưu, để test soi.
  String? get raw => _json;

  @override
  Future<PendingPayment?> load() async {
    final json = _json;
    if (json == null) return null;
    return PendingPayment.fromJson(jsonDecode(json) as Map<String, Object?>);
  }

  @override
  Future<void> save(PendingPayment pending) async {
    _json = jsonEncode(pending.toJson());
  }

  @override
  Future<void> clear() async {
    _json = null;
  }
}

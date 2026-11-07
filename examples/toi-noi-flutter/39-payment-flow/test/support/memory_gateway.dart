import 'dart:async';

import 'package:toi_noi_39/payment_gateway.dart';
import 'package:toi_noi_39/payment_models.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Gateway trong bộ nhớ, chỉ dùng microtask (không Timer, không I/O) nên chạy
/// được ngay trong `testWidgets` mà không cần `runAsync`.
final class MemoryGateway implements PaymentGateway {
  PaymentStatus status = PaymentStatus.pending;
  bool offline = false;
  int created = 0;

  @override
  Future<PaymentSession> createSession({
    required String tripId,
    required Money amount,
    required String idempotencyKey,
  }) async {
    created++;
    return PaymentSession(
      id: 'mem-1',
      tripId: tripId,
      amount: amount,
      checkoutUrl: Uri.parse('https://pay.vixanh.example/checkout/mem-1'),
      expiresAt: DateTime.utc(2026, 11, 6, 9, 15),
    );
  }

  @override
  Future<PaymentStatus> fetchStatus(String sessionId) async {
    if (offline) {
      throw const PaymentGatewayException(
        GatewayErrorKind.unreachable,
        'offline',
      );
    }
    return status;
  }
}

/// Gateway chỉ trả lời khi test cho phép, để giữ flow ở `PaymentCreating`.
final class GatedGateway implements PaymentGateway {
  GatedGateway(this._inner);

  final PaymentGateway _inner;
  final Completer<void> _gate = Completer<void>();

  void open() => _gate.complete();

  @override
  Future<PaymentSession> createSession({
    required String tripId,
    required Money amount,
    required String idempotencyKey,
  }) async {
    await _gate.future;
    return _inner.createSession(
      tripId: tripId,
      amount: amount,
      idempotencyKey: idempotencyKey,
    );
  }

  @override
  Future<PaymentStatus> fetchStatus(String sessionId) =>
      _inner.fetchStatus(sessionId);
}

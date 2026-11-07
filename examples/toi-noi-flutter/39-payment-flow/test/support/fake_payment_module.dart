import 'dart:convert';

import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:toi_noi_39/payment_models.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

final _keyPattern = RegExp(r'^[A-Za-z0-9_-]{1,128}$');

final class _Session {
  _Session({
    required this.id,
    required this.tripId,
    required this.amountVnd,
    required this.expiresAt,
  });

  final String id;
  final String tripId;
  final int amountVnd;
  final DateTime expiresAt;
  PaymentStatus status = PaymentStatus.pending;
}

/// Server thanh toán giả của lab, gắn vào fake API bằng `FakeApiModule` (không
/// sửa `tools/fake_api`). Hai route:
///
/// - `POST /payments/sessions` với `Idempotency-Key`
/// - `GET /payments/sessions/{id}`
///
/// Trang cổng và webhook của VíXanh không có thật: test gọi [settle] để đóng
/// vai "khách đã trả" hoặc "khách bấm huỷ" ở phía cổng.
final class FakePaymentModule implements FakeApiModule {
  FakePaymentModule({this.sessionTtl = const Duration(minutes: 15)});

  /// Hạn của một phiên (15 phút, số minh hoạ).
  final Duration sessionTtl;

  late FakeApiContext _context;
  final Map<String, _Session> _sessions = {};
  final Map<String, ({String sessionId, String tripId, int amountVnd})> _byKey =
      {};

  /// Số phiên server đã tạo.
  int get sessionCount => _sessions.length;

  /// Mã phiên tạo gần nhất.
  String get lastSessionId => _sessions.keys.last;

  /// Idempotency key của từng request tạo phiên, theo thứ tự nhận.
  final List<String> keysReceived = [];

  /// Trạng thái hiện tại, đã áp dụng hết hạn theo đồng hồ của server.
  PaymentStatus statusOf(String sessionId) {
    final session = _sessions[sessionId]!;
    if (session.status == PaymentStatus.pending &&
        !_context.clock().isBefore(session.expiresAt)) {
      session.status = PaymentStatus.expired;
    }
    return session.status;
  }

  /// Phía cổng chốt kết quả. Trả `false` nếu phiên đã chốt trước đó (một
  /// phiên đã `expired` không thể thành `succeeded`).
  bool settle(String sessionId, PaymentStatus status) {
    assert(status.isFinal);
    if (statusOf(sessionId).isFinal) return false;
    _sessions[sessionId]!.status = status;
    return true;
  }

  @override
  void mount(Router router, FakeApiContext context) {
    _context = context;
    router.post('/payments/sessions', (Request request) async {
      final key = request.headers[idempotencyKeyHeader] ?? '';
      if (!_keyPattern.hasMatch(key)) {
        throw const ProblemException(
          422,
          'Unprocessable request',
          errors: {
            'Idempotency-Key': ['Use 1..128 ASCII letters, digits, - or _.'],
          },
        );
      }
      keysReceived.add(key);
      final body = jsonDecode(await request.readAsString());
      final tripId = body is Map ? body['tripId'] : null;
      final amountVnd = body is Map ? body['amountVnd'] : null;
      if (tripId is! String || amountVnd is! int) {
        throw const ProblemException(422, 'Unprocessable request');
      }

      final previous = _byKey[key];
      if (previous != null) {
        if (previous.tripId != tripId || previous.amountVnd != amountVnd) {
          throw const ProblemException(
            409,
            'Request conflicts with current state',
          );
        }
        return _json(200, _sessions[previous.sessionId]!);
      }

      final trip = context.store.trip(tripId);
      if (trip == null) throw const ProblemException(404, 'Trip not found');
      // Số tiền do server tính; client chỉ gửi để đối chiếu.
      if (trip.trip.fare.vnd != amountVnd) {
        throw const ProblemException(
          422,
          'Unprocessable request',
          errors: {
            'amountVnd': ['Amount does not match the trip fare.'],
          },
        );
      }
      final alreadyPaid = _sessions.values.any(
        (s) => s.tripId == tripId && statusOf(s.id) == PaymentStatus.succeeded,
      );
      if (alreadyPaid) {
        throw const ProblemException(
          409,
          'Request conflicts with current state',
        );
      }

      final session = _Session(
        id: context.newId(),
        tripId: tripId,
        amountVnd: amountVnd,
        expiresAt: context.clock().add(sessionTtl),
      );
      _sessions[session.id] = session;
      _byKey[key] = (
        sessionId: session.id,
        tripId: tripId,
        amountVnd: amountVnd,
      );
      return _json(201, session);
    });

    router.get('/payments/sessions/<id>', (Request request) {
      final id = request.params['id']!;
      if (!_sessions.containsKey(id)) {
        throw const ProblemException(404, 'Payment session not found');
      }
      return _json(200, _sessions[id]!);
    });
  }

  Response _json(int status, _Session s) => jsonResponse(status, {
    'id': s.id,
    'tripId': s.tripId,
    'amountVnd': s.amountVnd,
    'checkoutUrl': 'https://pay.vixanh.example/checkout/${s.id}',
    'expiresAt': s.expiresAt.toIso8601String(),
    'status': statusOf(s.id).name,
  });
}

import 'package:dio/dio.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'payment_models.dart';

/// Nguyên nhân một lần gọi server thanh toán thất bại.
enum GatewayErrorKind {
  /// Mất mạng, hết giờ, 5xx: chưa biết server có xử lý hay không.
  unreachable,

  /// Server trả lời rõ ràng là không (4xx): thử lại y nguyên cũng vậy.
  rejected,
}

final class PaymentGatewayException implements Exception {
  const PaymentGatewayException(this.kind, this.message, {this.statusCode});

  final GatewayErrorKind kind;
  final String message;
  final int? statusCode;

  /// Thử lại cùng yêu cầu (cùng idempotency key) có ý nghĩa.
  bool get retryable => kind == GatewayErrorKind.unreachable;

  @override
  String toString() => 'PaymentGatewayException($kind, $message)';
}

/// Cổng vào server Payment & Wallet của Tới Nơi. App chỉ nói chuyện với
/// server của mình, không bao giờ với VíXanh trực tiếp.
abstract interface class PaymentGateway {
  /// Tạo phiên thanh toán. Cùng [idempotencyKey] và cùng nội dung thì server
  /// trả lại đúng phiên cũ thay vì tạo phiên mới.
  Future<PaymentSession> createSession({
    required String tripId,
    required Money amount,
    required String idempotencyKey,
  });

  /// Trạng thái hiện tại của phiên theo server.
  Future<PaymentStatus> fetchStatus(String sessionId);
}

/// Hiện thực bằng `dio`.
final class HttpPaymentGateway implements PaymentGateway {
  HttpPaymentGateway(this._dio);

  final Dio _dio;

  @override
  Future<PaymentSession> createSession({
    required String tripId,
    required Money amount,
    required String idempotencyKey,
  }) => _guard(() async {
    final response = await _dio.post<Map<String, Object?>>(
      '/payments/sessions',
      data: {'tripId': tripId, 'amountVnd': amount.vnd},
      options: Options(headers: {'Idempotency-Key': idempotencyKey}),
    );
    return PaymentSession.fromJson(response.data!);
  });

  @override
  Future<PaymentStatus> fetchStatus(String sessionId) => _guard(() async {
    final response = await _dio.get<Map<String, Object?>>(
      '/payments/sessions/${Uri.encodeComponent(sessionId)}',
    );
    return PaymentStatus.parse(response.data!['status']);
  });

  Future<T> _guard<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on DioException catch (e) {
      final status = e.response?.statusCode;
      if (status != null && status >= 400 && status < 500) {
        throw PaymentGatewayException(
          GatewayErrorKind.rejected,
          'Server từ chối ($status)',
          statusCode: status,
        );
      }
      throw PaymentGatewayException(
        GatewayErrorKind.unreachable,
        status == null ? 'Không tới được server' : 'Server lỗi ($status)',
        statusCode: status,
      );
    } on FormatException catch (e) {
      throw PaymentGatewayException(
        GatewayErrorKind.unreachable,
        'Phản hồi không hợp lệ: ${e.message}',
      );
    }
  }
}

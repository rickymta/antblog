/// Link quay về từ cổng thanh toán: `toinoi://payment/return?session=...`.
///
/// `claimed` chỉ là LỜI KHẲNG ĐỊNH của người gọi link. Ai cũng gọi được
/// `toinoi://payment/return?session=...&status=success`, nên giá trị này
/// không bao giờ được dùng để kết luận thanh toán thành công; nó chỉ để ghi
/// log hoặc hiển thị "đang kiểm tra".
final class PaymentReturn {
  const PaymentReturn({required this.sessionId, this.claimed});

  static const scheme = 'toinoi';
  static const host = 'payment';
  static const path = '/return';

  final String sessionId;

  /// Giá trị tham số `status` nếu có (`success`, `cancel`, ...), chưa xác thực.
  final String? claimed;

  /// Đúng scheme, host và đường dẫn của link quay về (chưa xét tham số).
  static bool matches(Uri uri) =>
      uri.scheme == scheme && uri.host == host && uri.path == path;

  /// Trả `null` nếu không phải link quay về hoặc thiếu mã phiên.
  static PaymentReturn? tryParse(Uri uri) {
    if (!matches(uri)) return null;
    final session = uri.queryParameters['session'];
    if (session == null || session.isEmpty) return null;
    return PaymentReturn(
      sessionId: session,
      claimed: uri.queryParameters['status'],
    );
  }
}

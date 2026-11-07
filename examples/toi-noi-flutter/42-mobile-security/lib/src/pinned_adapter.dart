import 'dart:io';

import 'package:dio/io.dart';

import 'cert_pin.dart';

/// Adapter ghim chứng chỉ ở **bước bắt tay TLS**.
///
/// Client tạo ra không tin CA nào (`withTrustedRoots: false`), nên mọi chứng
/// chỉ đều bị coi là "xấu" và đi qua `badCertificateCallback`; callback chỉ
/// chấp nhận chứng chỉ lá khớp ghim của đúng host. Chứng chỉ sai bị từ chối
/// trước khi request nào được gửi đi.
///
/// Adapter này chỉ dành cho host của Tới Nơi: host ngoài [policy] luôn bị từ
/// chối. Request tới nơi khác (ví dụ tile bản đồ) dùng Dio riêng.
IOHttpClientAdapter createPinnedAdapter(PinPolicy policy) {
  return IOHttpClientAdapter(
    createHttpClient: () {
      final client = HttpClient(
        context: SecurityContext(withTrustedRoots: false),
      );
      client.badCertificateCallback = (certificate, host, port) =>
          _accepts(policy, certificate, host);
      return client;
    },
  );
}

/// Cách làm **không đủ**, giữ lại để so sánh: tin CA như bình thường rồi kiểm
/// ghim bằng `validateCertificate`.
///
/// Dio gọi `validateCertificate` sau `request.close()`, tức là khi request
/// (kể cả header `Authorization`) đã tới server. Chứng chỉ sai chỉ làm cuộc
/// gọi ném `DioException`, không ngăn dữ liệu đã gửi.
///
/// [context] cho test cấp chứng chỉ tin cậy cục bộ; app thật để `null` (dùng
/// kho CA của hệ thống).
IOHttpClientAdapter createCheckAfterSendAdapter(
  PinPolicy policy, {
  SecurityContext? context,
}) {
  return IOHttpClientAdapter(
    createHttpClient: () => HttpClient(context: context),
    validateCertificate: (certificate, host, port) =>
        _accepts(policy, certificate, host),
  );
}

bool _accepts(PinPolicy policy, X509Certificate? certificate, String host) {
  if (certificate == null) return false;
  return policy.accepts(
    host,
    certificate.der,
    startValidity: certificate.startValidity,
    endValidity: certificate.endValidity,
  );
}

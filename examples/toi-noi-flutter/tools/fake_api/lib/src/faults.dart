import 'package:shelf/shelf.dart';

import 'problem.dart';

/// Lỗi và độ trễ tiêm có chủ đích để thử client trước khi mạng thật làm hỏng.
///
/// `GET /health` luôn được miễn: nó dùng để biết server còn sống chứ không
/// phải để thử lỗi.
final class FaultConfig {
  const FaultConfig({
    this.latency = Duration.zero,
    this.failFirst = 0,
    this.failEvery = 0,
  }) : assert(failFirst >= 0, 'failFirst không được âm'),
       assert(failEvery >= 0, 'failEvery không được âm');

  /// Chờ chừng này trước khi xử lý mỗi request.
  final Duration latency;

  /// [failFirst] request đầu tiên trả 500.
  final int failFirst;

  /// Cứ mỗi [failEvery] request thì một request trả 500 (0 là tắt).
  final int failEvery;

  /// Request thứ [requestNumber] (đếm từ 1) có bị lỗi 500 không.
  bool shouldFail(int requestNumber) =>
      requestNumber <= failFirst ||
      (failEvery > 0 && requestNumber % failEvery == 0);
}

/// Middleware áp [FaultConfig]. Bộ đếm nằm trong closure nên mỗi handler có
/// bộ đếm riêng.
Middleware faultMiddleware(
  FaultConfig config,
  Future<void> Function(Duration) delay,
) {
  var requestNumber = 0;
  return (Handler inner) => (Request request) async {
    if (request.url.path == 'health') {
      return inner(request);
    }
    requestNumber++;
    if (config.latency > Duration.zero) {
      await delay(config.latency);
    }
    if (config.shouldFail(requestNumber)) {
      return problemResponse(request, 500, 'Unexpected server error');
    }
    return inner(request);
  };
}

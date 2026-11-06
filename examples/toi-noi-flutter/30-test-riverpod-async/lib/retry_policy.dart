/// Chính sách thử lại của Riverpod cho app: chỉ lỗi mạng tạm thời đáng thử.
///
/// Riverpod 3 mặc định thử lại mọi `Exception` tối đa 10 lần, kể cả 422 của
/// server vốn có gửi lại cũng vẫn 422. Hàm này chỉ thử lại [ApiFailure] mà
/// `retryable` là `true`, tối đa 3 lần, chờ 500 ms rồi 1 s rồi 2 s. Số liệu là
/// minh hoạ. App truyền hàm này cho `ProviderScope(retry: retryTransient)`;
/// test truyền `(_, _) => null` để tắt, hoặc chính hàm này để kiểm.
library;

import 'package:toi_noi_api/toi_noi_api.dart';

Duration? retryTransient(int retryCount, Object error) {
  if (error is! ApiFailure || !error.retryable || retryCount >= 3) return null;
  return Duration(milliseconds: 500 * (1 << retryCount));
}

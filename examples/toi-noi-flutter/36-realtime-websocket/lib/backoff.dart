import 'dart:math';

/// Thời gian chờ trước lần nối lại thứ n: exponential backoff có trần, với
/// **full jitter** (chọn đều ngẫu nhiên trong [0, trần]).
///
/// Trần của lần thử thứ `attempt` (0 là lần đầu) là
/// `min(cap, base * 2^attempt)`. Không có jitter, mọi client cùng mất kết nối
/// (gateway khởi động lại) sẽ cùng nối lại ở cùng một thời điểm, lần nào cũng
/// vậy.
final class Backoff {
  Backoff({
    this.base = const Duration(seconds: 1),
    this.cap = const Duration(seconds: 30),
    Random? random,
  }) : assert(base > Duration.zero, 'base phải dương'),
       assert(cap >= base, 'cap phải không nhỏ hơn base'),
       _random = random ?? Random();

  final Duration base;
  final Duration cap;
  final Random _random;

  /// Trần của lần thử thứ [attempt], tính từ 0.
  Duration ceiling(int attempt) {
    if (attempt < 0) throw RangeError.value(attempt, 'attempt');
    // 2^30 đã vượt mọi cap hợp lý; chặn để phép nhân không tràn int.
    if (attempt >= 30) return cap;
    final doubled = base * (1 << attempt);
    return doubled > cap ? cap : doubled;
  }

  /// Thời gian chờ thực tế của lần thử thứ [attempt]: đều trong [0, trần).
  Duration delay(int attempt) {
    final max = ceiling(attempt).inMicroseconds;
    return Duration(microseconds: (_random.nextDouble() * max).floor());
  }
}

import 'dart:async';
import 'dart:isolate';

/// Tổng 1..n bằng vòng lặp (cố ý tốn CPU để minh hoạ).
int sumUpTo(int n) {
  if (n < 0) throw ArgumentError.value(n, 'n', 'không được âm');
  var total = 0;
  for (var i = 1; i <= n; i++) {
    total += i;
  }
  return total;
}

/// Chạy [sumUpTo] ở isolate khác. Closure và dữ liệu nó bắt được (ở đây là n)
/// được copy sang isolate mới; kết quả trả về qua Isolate.exit nên được
/// chuyển sang isolate gọi mà không copy. Isolate mới không chia sẻ bộ nhớ
/// với isolate gọi, nên vòng lặp nặng không chặn event loop của UI.
Future<int> sumInIsolate(int n) => Isolate.run(() => sumUpTo(n));

/// Đếm số lần timer 20 ms kịp chạy trong lúc [work] thực hiện.
/// Nhiều tick nghĩa là event loop của isolate gọi vẫn rảnh.
Future<({int result, int ticks})> measureTicks(
  Future<int> Function() work,
) async {
  var ticks = 0;
  final timer = Timer.periodic(
    const Duration(milliseconds: 20),
    (_) => ticks++,
  );
  try {
    final result = await work();
    return (result: result, ticks: ticks);
  } finally {
    timer.cancel();
  }
}

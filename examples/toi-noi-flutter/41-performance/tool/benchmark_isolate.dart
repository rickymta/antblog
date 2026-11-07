// ignore_for_file: avoid_print
// Benchmark Dart thuần: `dart run tool/benchmark_isolate.dart [số chuyến] [số lần]`.
//
// MINH HOẠ, không phải số đo trên điện thoại: chạy trên máy phát triển bằng
// Dart VM (JIT), và kết quả đổi theo tải của máy. Vì vậy script đo thêm mốc
// "không làm gì" (nhiễu của chính Timer) và chạy nhiều lần để in khoảng giá trị.
import 'dart:async';

import 'package:toi_noi_41/trip_json.dart';

Future<void> main(List<String> args) async {
  final count = args.isEmpty ? 50000 : int.parse(args.first);
  final runs = args.length > 1 ? int.parse(args[1]) : 5;
  final json = generateTripsJson(count);
  print('JSON: $count chuyến, ${(json.length / 1e6).toStringAsFixed(1)} MB');

  // Làm nóng JIT một lần để lần đo đầu không gánh chi phí biên dịch.
  parseTrips(generateTripsJson(2000));

  final idle = <int>[];
  final sync = <int>[];
  final worker = <int>[];
  for (var i = 0; i < runs; i++) {
    idle.add(
      await maxGapMs(
        () => Future<void>.delayed(const Duration(milliseconds: 300)),
      ),
    );
    sync.add(await maxGapMs(() async => parseTrips(json)));
    worker.add(await maxGapMs(() => parseTripsInIsolate(json)));
  }

  print('');
  print('Khoảng cách lớn nhất giữa hai nhịp Timer 2 ms, $runs lần chạy (ms):');
  print('  không làm gì : ${_range(idle)}   <- nhiễu nền của máy');
  print('  main isolate : ${_range(sync)}');
  print('  Isolate.run  : ${_range(worker)}');
}

String _range(List<int> v) {
  final s = [...v]..sort();
  return '${s.first} - ${s.last}  (mọi lần: ${v.join(', ')})';
}

/// Chạy [work] trong khi một Timer.periodic 2 ms ghi lại khoảng cách lớn nhất
/// giữa các nhịp. Nếu main isolate bị chặn 80 ms, một khoảng cách sẽ xấp xỉ 80.
Future<int> maxGapMs(Future<void> Function() work) async {
  final clock = Stopwatch()..start();
  var last = clock.elapsed;
  var maxGap = Duration.zero;
  final timer = Timer.periodic(const Duration(milliseconds: 2), (_) {
    final now = clock.elapsed;
    if (now - last > maxGap) {
      maxGap = now - last;
    }
    last = now;
  });
  await Future<void>.delayed(const Duration(milliseconds: 20)); // vài nhịp đầu
  last = clock.elapsed;
  maxGap = Duration.zero;
  await work();
  // Chờ thêm vài nhịp: nhịp bị hoãn bởi work chỉ được ghi sau khi nó xong.
  await Future<void>.delayed(const Duration(milliseconds: 10));
  timer.cancel();
  return maxGap.inMilliseconds;
}

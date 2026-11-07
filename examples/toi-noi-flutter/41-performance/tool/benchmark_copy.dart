// ignore_for_file: avoid_print
// Chi phí chuyển dữ liệu qua ranh giới isolate, nhìn từ main isolate:
// `dart run tool/benchmark_copy.dart [số lần]`.
//
// MINH HOẠ trên máy phát triển (Dart VM, JIT), không phải số đo trên điện thoại.
// In khoảng giá trị của nhiều lần chạy cùng mốc "không làm gì".
import 'dart:async';
import 'dart:isolate';

import 'benchmark_isolate.dart' show maxGapMs;

Future<void> main(List<String> args) async {
  final runs = args.isEmpty ? 5 : int.parse(args.first);
  final bigList = List<Map<String, Object?>>.generate(
    1000000,
    (i) => {'id': i, 'fareVnd': 12000 + i % 1000},
  );
  final bigString = 'x' * 50000000;

  final results = <String, List<int>>{
    'không làm gì': [],
    'đối số List<Map> 1 triệu phần tử': [],
    'đối số String 50 MB': [],
    'closure chỉ dùng String, cùng scope với List lớn': [],
    'trả về 1 triệu Map (Isolate.exit)': [],
  };

  for (var i = 0; i < runs; i++) {
    results['không làm gì']!.add(
      await maxGapMs(
        () => Future<void>.delayed(const Duration(milliseconds: 300)),
      ),
    );
    results['đối số List<Map> 1 triệu phần tử']!.add(
      await maxGapMs(() => _listOnly(bigList)),
    );
    results['đối số String 50 MB']!.add(
      await maxGapMs(() => _stringOnly(bigString)),
    );
    results['closure chỉ dùng String, cùng scope với List lớn']!.add(
      await maxGapMs(() => _stringOnlyNextToList(bigString, bigList)),
    );
    results['trả về 1 triệu Map (Isolate.exit)']!.add(
      await maxGapMs(() => Isolate.run(_makeBigList)),
    );
  }

  print('Khoảng cách lớn nhất giữa hai nhịp Timer 2 ms, $runs lần chạy (ms):');
  results.forEach((name, v) {
    final s = [...v]..sort();
    print('  $name: ${s.first} - ${s.last}');
  });
}

// Mỗi closure nằm trong hàm riêng, chỉ thấy đối số của hàm đó: nếu đặt chúng
// chung trong `main`, closure sẽ dùng chung scope với mọi biến của `main`.
Future<int> _listOnly(List<Object?> list) => Isolate.run(() => list.length);

Future<int> _stringOnly(String text) => Isolate.run(() => text.length);

/// Closure chỉ dùng [text], nhưng [list] nằm cùng scope với một closure khác
/// bắt nó; theo tài liệu `Isolate.run`, closure có thể mang theo state ngoài ý
/// muốn (xem http://dartbug.com/36983).
Future<int> _stringOnlyNextToList(String text, List<Object?> list) {
  // ignore: unused_local_variable, prefer_function_declarations_over_variables
  final other = () => list.length;
  return Isolate.run(() => text.length);
}

List<Map<String, Object?>> _makeBigList() =>
    List.generate(1000000, (i) => {'id': i, 'fareVnd': 12000 + i % 1000});

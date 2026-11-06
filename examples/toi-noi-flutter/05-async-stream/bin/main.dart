// ignore_for_file: avoid_print
import 'dart:async';

import 'package:toi_noi_05/driver_location.dart';
import 'package:toi_noi_05/event_loop_demo.dart';
import 'package:toi_noi_05/heavy.dart';
import 'package:toi_noi_05/quote_service.dart';

Future<void> main() async {
  print('--- 1. Báo giá (độ trễ mô phỏng 300 ms) ---');
  final service = QuoteService();
  print('5 km => ${await service.fetchFareVnd(5)} VND');
  final parallel = await fetchFaresInParallel(service, [2, 5, 10]);
  print('2, 5, 10 km song song => $parallel');
  await service.fetchFareVnd(2).then((fare) => 'then: $fare VND').then(print);
  try {
    await service.fetchFareVnd(150);
  } on QuoteException catch (e) {
    print('150 km => $e');
  }

  print('--- 2. Timeout 100 ms ---');
  try {
    await fetchWithTimeout(
      service,
      5,
      timeout: const Duration(milliseconds: 100),
    );
  } on TimeoutException catch (e) {
    print('Quá hạn: ${e.message}');
  }

  print('--- 3. Lắng nghe 5 vị trí rồi huỷ ---');
  final start = DateTime.utc(2026, 11, 6, 8);
  final route = [
    for (var i = 0; i < 8; i++)
      Position(
        (210285 + i * 2) / 10000,
        105.8542,
        start.add(Duration(seconds: i)),
      ),
  ];
  final feed = DriverLocationFeed(route);
  final done = Completer<void>();
  late final StreamSubscription<Position> subscription;
  var received = 0;
  subscription = feed.positions(every: const Duration(milliseconds: 50)).listen(
    (p) async {
      received++;
      print('Vị trí #$received: $p');
      if (received == 5) {
        await subscription.cancel();
        print('Đã huỷ subscription, không nhận thêm.');
        done.complete();
      }
    },
  );
  await done.future;

  print('--- 3b. await for với take(3): tự huỷ khi đủ ---');
  final latitudes = feed
      .positions(every: const Duration(milliseconds: 30))
      .map((p) => p.lat.toStringAsFixed(4))
      .take(3);
  await for (final lat in latitudes) {
    print('lat $lat');
  }

  print('--- 4. Thứ tự event loop ---');
  print(await traceEventLoop());
  print(await traceAwaitOrder());

  print('--- 5. Việc nặng: main isolate và Isolate.run ---');
  const n = 600000000;
  final onMain = await measureTicks(() => Future(() => sumUpTo(n)));
  print(
    'Main isolate: kết quả ${onMain.result}, timer chạy ${onMain.ticks} lần',
  );
  final offMain = await measureTicks(() => sumInIsolate(n));
  print(
    'Isolate.run:  kết quả ${offMain.result}, timer chạy ${offMain.ticks} lần',
  );
}

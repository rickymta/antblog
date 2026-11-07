import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/history_page.dart';
import 'package:toi_noi_41/trip_json.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  testWidgets('bấm Tải: parser được gọi một lần và danh sách hiện kết quả', (
    tester,
  ) async {
    final seen = <int>[];
    Future<List<Trip>> fakeParser(String json) async {
      final trips = parseTrips(json);
      seen.add(trips.length);
      return trips;
    }

    await tester.pumpWidget(
      MaterialApp(home: HistoryPage(tripCount: 50, parser: fakeParser)),
    );
    expect(find.text('Đã nạp 0 chuyến'), findsOneWidget);

    await tester.tap(find.text('Tải 50 chuyến'));
    await tester.pump();
    await tester.pump();

    expect(seen, [50]);
    expect(find.text('Đã nạp 50 chuyến'), findsOneWidget);
    expect(find.text('T000001'), findsOneWidget);
  });

  testWidgets('không có parser tiêm vào thì chạy được trên isolate thật', (
    tester,
  ) async {
    await tester.pumpWidget(
      const MaterialApp(home: HistoryPage(tripCount: 30)),
    );

    await tester.tap(find.text('Tải 30 chuyến'));
    // Isolate thật chạy theo đồng hồ thật: chờ tới khi xong thay vì ngủ cố định.
    await tester.runAsync(() async {
      for (var i = 0; i < 500; i++) {
        await tester.pump();
        if (find.text('Đã nạp 30 chuyến').evaluate().isNotEmpty) {
          return;
        }
        await Future<void>.delayed(const Duration(milliseconds: 20));
      }
    });

    expect(find.text('Đã nạp 30 chuyến'), findsOneWidget);
  });
}

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show ScrollCacheExtent;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/build_log.dart';
import 'package:toi_noi_41/trip_json.dart';
import 'package:toi_noi_41/trip_list.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Khung nhìn của test mặc định là 800 x 600 pixel logic.
const double _viewport = 600;

/// Số dòng có phần nằm trong [extent] pixel đầu tiên của danh sách.
int _rowsWithin(double extent) => (extent / tripTileHeight).ceil();

void main() {
  late List<Trip> trips;

  setUpAll(() => trips = parseTrips(generateTripsJson(1000)));
  setUp(BuildLog.reset);

  Future<void> pumpList(
    WidgetTester tester,
    ListStrategy strategy, {
    ScrollCacheExtent? cacheExtent,
  }) => tester.pumpWidget(
    MaterialApp(
      theme: ToiNoiTheme.light(),
      home: Scaffold(
        body: TripHistoryList(
          trips: trips,
          strategy: strategy,
          cacheExtent: cacheExtent,
        ),
      ),
    ),
  );

  test('dữ liệu thử có 1000 chuyến', () {
    expect(trips, hasLength(1000));
  });

  group('SingleChildScrollView + Column: không lười', () {
    testWidgets('tạo và build cả 1000 dòng dù chỉ thấy 9 dòng', (tester) async {
      await pumpList(tester, ListStrategy.column);

      expect(BuildLog.of('tile-created'), 1000);
      expect(BuildLog.of('tile'), 1000);
    });
  });

  group('ListView(children:): tạo hết, build một phần', () {
    testWidgets('tạo 1000 widget nhưng chỉ build dòng gần khung nhìn', (
      tester,
    ) async {
      await pumpList(tester, ListStrategy.children);

      // 600 px khung nhìn + 250 px cacheExtent mặc định.
      final expected = _rowsWithin(_viewport + 250);
      expect(BuildLog.of('tile-created'), 1000);
      expect(BuildLog.of('tile'), expected);
      expect(expected, 12);
    });

    testWidgets('cha dựng lại thì tạo lại cả 1000 widget', (tester) async {
      await pumpList(tester, ListStrategy.children);
      BuildLog.reset();

      await pumpList(tester, ListStrategy.children);

      expect(BuildLog.of('tile-created'), 1000);
    });
  });

  group('ListView.builder: lười hoàn toàn', () {
    testWidgets('chỉ tạo và build 12 dòng', (tester) async {
      await pumpList(tester, ListStrategy.builder);

      expect(BuildLog.of('tile-created'), 12);
      expect(BuildLog.of('tile'), 12);
    });

    testWidgets('cha dựng lại chỉ tạo lại những dòng đang có', (tester) async {
      await pumpList(tester, ListStrategy.builder);
      BuildLog.reset();

      await pumpList(tester, ListStrategy.builder);

      expect(BuildLog.of('tile-created'), 12);
    });

    testWidgets('cuộn xuống thì dòng mới được build, dòng cũ bị bỏ', (
      tester,
    ) async {
      await pumpList(tester, ListStrategy.builder);
      BuildLog.reset();

      await tester.drag(find.byType(ListView), const Offset(0, -2000));
      await tester.pump();

      expect(find.text('T000001'), findsNothing);
      expect(BuildLog.of('tile'), greaterThan(0));
      expect(BuildLog.of('tile'), lessThan(40));
    });
  });

  group('scrollCacheExtent', () {
    testWidgets('pixels(0): chỉ build những dòng nhìn thấy', (tester) async {
      await pumpList(
        tester,
        ListStrategy.builder,
        cacheExtent: const ScrollCacheExtent.pixels(0),
      );

      expect(BuildLog.of('tile'), _rowsWithin(_viewport));
      expect(BuildLog.of('tile'), 9);
    });

    testWidgets('pixels(1000): build thêm phía trước', (tester) async {
      await pumpList(
        tester,
        ListStrategy.builder,
        cacheExtent: const ScrollCacheExtent.pixels(1000),
      );

      expect(BuildLog.of('tile'), _rowsWithin(_viewport + 1000));
    });

    testWidgets('viewport(1): vùng đệm bằng một khung nhìn', (tester) async {
      await pumpList(
        tester,
        ListStrategy.builder,
        cacheExtent: const ScrollCacheExtent.viewport(1),
      );

      expect(BuildLog.of('tile'), _rowsWithin(_viewport * 2));
    });

    testWidgets('viewport(0.5): 300 dp đệm, 13 dòng', (tester) async {
      await pumpList(
        tester,
        ListStrategy.builder,
        cacheExtent: const ScrollCacheExtent.viewport(0.5),
      );

      expect(BuildLog.of('tile'), _rowsWithin(_viewport * 1.5));
      expect(BuildLog.of('tile'), 13);
    });

    testWidgets('cacheExtent càng lớn thì càng build nhiều lúc dựng đầu', (
      tester,
    ) async {
      final counts = <int>[];
      for (final pixels in [0.0, 250.0, 1000.0]) {
        BuildLog.reset();
        await pumpList(
          tester,
          ListStrategy.builder,
          cacheExtent: ScrollCacheExtent.pixels(pixels),
        );
        counts.add(BuildLog.of('tile'));
        await tester.pumpWidget(const SizedBox());
      }

      expect(counts, [9, 12, 23]);
    });
  });
}

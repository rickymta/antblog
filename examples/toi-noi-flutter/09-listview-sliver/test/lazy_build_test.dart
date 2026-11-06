import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_09/recent_trips_list.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Phần tử đếm số lần `build` của chính nó, ghi lại chỉ số vào [log].
class _Probe extends StatelessWidget {
  const _Probe({required this.index, required this.log});

  final int index;
  final List<int> log;

  @override
  Widget build(BuildContext context) {
    log.add(index);
    return SizedBox(height: 72, child: Text('Dòng $index'));
  }
}

void main() {
  late List<Trip> trips500;

  // setUpAll chạy ngoài FakeAsync của testWidgets nên Future.delayed thật chạy được.
  setUpAll(() async => trips500 = await loadTrips(500));

  group('ListView.builder chỉ build phần tử trong vùng nhìn', () {
    testWidgets('500 phần tử nhưng chỉ vài chục lần build', (tester) async {
      final builds = <int>[];
      await tester.pumpWidget(
        app(
          ListView.builder(
            itemCount: 500,
            itemBuilder: (context, index) => _Probe(index: index, log: builds),
          ),
        ),
      );

      // Màn hình test cao 600 dp, mỗi dòng 72 dp, vùng đệm 250 dp.
      expect(builds, [for (var i = 0; i < 12; i++) i]);
      expect(find.text('Dòng 0'), findsOneWidget);
      expect(find.text('Dòng 400', skipOffstage: false), findsNothing);
    });

    testWidgets('cuộn tay vài màn hình: dòng đầu bị bỏ, số dòng đã build vẫn '
        'chỉ cỡ vài chục', (tester) async {
      final builds = <int>[];
      await tester.pumpWidget(
        app(
          ListView.builder(
            itemCount: 500,
            itemBuilder: (context, index) => _Probe(index: index, log: builds),
          ),
        ),
      );

      for (var i = 0; i < 5; i++) {
        await tester.drag(find.byType(ListView), const Offset(0, -500));
        await tester.pump();
      }

      expect(find.text('Dòng 0', skipOffstage: false), findsNothing);
      expect(find.text('Dòng 499', skipOffstage: false), findsNothing);
      expect(builds.toSet().length, lessThan(60));
      expect(builds.reduce(math.max), lessThan(60));
    });

    testWidgets('nhảy xa một phát khi chưa biết chiều cao: phải build lần lượt '
        'các dòng ở giữa', (tester) async {
      final builds = <int>[];
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        app(
          ListView.builder(
            controller: controller,
            itemCount: 500,
            itemBuilder: (context, index) => _Probe(index: index, log: builds),
          ),
        ),
      );

      controller.jumpTo(72.0 * 400);
      await tester.pump();

      // Trên màn hình vẫn chỉ vài dòng, nhưng muốn biết dòng 400 nằm ở đâu thì
      // framework phải đo các dòng trước nó.
      expect(find.text('Dòng 400'), findsOneWidget);
      expect(find.text('Dòng 0', skipOffstage: false), findsNothing);
      // 12 dòng của lần dựng đầu cộng 400 dòng dựng trong lúc đo tới đích.
      expect(builds, hasLength(412));
    });

    testWidgets('nhảy xa khi cho itemExtent: không build các dòng ở giữa', (
      tester,
    ) async {
      final builds = <int>[];
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        app(
          ListView.builder(
            controller: controller,
            itemExtent: 72,
            itemCount: 500,
            itemBuilder: (context, index) => _Probe(index: index, log: builds),
          ),
        ),
      );

      controller.jumpTo(72.0 * 400);
      await tester.pump();

      expect(find.text('Dòng 400'), findsOneWidget);
      expect(builds, hasLength(28)); // 12 lúc đầu + 16 sau cú nhảy
    });

    testWidgets('ListView.separated: separatorBuilder cũng chỉ chạy cho dòng '
        'đã build', (tester) async {
      await tester.pumpWidget(app(RecentTripsList(trips: trips500)));

      final tiles = tester.widgetList(find.byType(ListTile)).length;
      final dividers = tester.widgetList(find.byType(Divider)).length;
      expect(tiles, inInclusiveRange(1, 19));
      expect(dividers, inInclusiveRange(tiles - 1, tiles));
    });

    testWidgets('chuyến huỷ hiện trạng thái, chuyến hoàn thành hiện cước', (
      tester,
    ) async {
      await tester.pumpWidget(
        app(RecentTripsList(trips: trips500.take(5).toList())),
      );

      expect(find.text('25.500 VND'), findsOneWidget);
      expect(find.text('Đã huỷ'), findsOneWidget);
    });

    testWidgets('dòng cuối của danh sách 500 chuyến có thể cuộn tới', (
      tester,
    ) async {
      final trips = trips500;
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        app(
          ListView.builder(
            controller: controller,
            itemCount: trips.length,
            itemBuilder: (context, index) => Text(trips[index].id),
          ),
        ),
      );

      controller.jumpTo(controller.position.maxScrollExtent);
      await tester.pump();

      expect(find.text(trips.last.id), findsOneWidget);
    });
  });

  group('so với dựng sẵn mọi phần tử', () {
    testWidgets('ListView(children) khởi tạo đủ 500 widget, nhưng chỉ build '
        'phần tử trong vùng nhìn; builder không khởi tạo phần tử thừa', (
      tester,
    ) async {
      final builds = <int>[];
      var constructed = 0;

      // Cách 1: danh sách dựng sẵn. Vòng lặp chạy đủ 500 lần ngay lập tức.
      final children = [
        for (var i = 0; i < 500; i++)
          () {
            constructed++;
            return _Probe(index: i, log: builds);
          }(),
      ];
      await tester.pumpWidget(app(ListView(children: children)));
      expect(constructed, 500);
      expect(builds.length, lessThan(20));

      // Cách 2: builder. Chỉ những chỉ số được hỏi mới tạo widget.
      var built = 0;
      await tester.pumpWidget(
        app(
          ListView.builder(
            itemCount: 500,
            itemBuilder: (context, index) {
              built++;
              return SizedBox(height: 72, child: Text('Dòng $index'));
            },
          ),
        ),
      );
      expect(built, lessThan(20));
    });
  });

  group('hai cách làm mất tính lười', () {
    testWidgets('ListView trong Column không bọc Expanded: viewport không có '
        'chiều cao hữu hạn', (tester) async {
      final errors = <FlutterErrorDetails>[];
      final previous = FlutterError.onError;
      FlutterError.onError = errors.add;
      addTearDown(() => FlutterError.onError = previous);

      await tester.pumpWidget(
        app(
          Column(
            children: [
              ListView.builder(
                itemCount: 500,
                itemBuilder: (context, index) => Text('Dòng $index'),
              ),
            ],
          ),
        ),
      );

      expect(
        errors.map((e) => e.exceptionAsString()),
        contains(contains('Vertical viewport was given unbounded height')),
      );
    });

    testWidgets('shrinkWrap: true trong cuộn khác build đủ 500 phần tử', (
      tester,
    ) async {
      final builds = <int>[];
      await tester.pumpWidget(
        app(
          SingleChildScrollView(
            child: ListView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: 500,
              itemBuilder: (context, index) =>
                  _Probe(index: index, log: builds),
            ),
          ),
        ),
      );

      expect(builds.toSet(), hasLength(500));
    });
  });

  test('dữ liệu giả có 500 chuyến để các test trên có ý nghĩa', () {
    expect(trips500, hasLength(500));
    expect(trips500.map((Trip t) => t.id).toSet(), hasLength(500));
  });
}

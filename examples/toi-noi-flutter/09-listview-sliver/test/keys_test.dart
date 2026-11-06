import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_09/trip_history_tile.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Danh sách thử nghiệm: cùng một `TripHistoryTile` (có state "đang mở") nhưng
/// ta chọn có key hay không và có `findChildIndexCallback` hay không.
class _Harness extends StatelessWidget {
  const _Harness({
    required this.trips,
    required this.useKey,
    required this.useFindIndex,
  });

  final List<Trip> trips;
  final bool useKey;
  final bool useFindIndex;

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: trips.length,
      findChildIndexCallback: useFindIndex
          ? (key) {
              final index = trips.indexWhere(
                (trip) => key == ValueKey(trip.id),
              );
              return index < 0 ? null : index;
            }
          : null,
      itemBuilder: (context, index) => TripHistoryTile(
        key: useKey ? ValueKey(trips[index].id) : null,
        trip: trips[index],
      ),
    );
  }
}

/// Dòng của chuyến [id] đang mở (có dòng "Tài xế: ...") hay không.
bool isExpanded(String id) {
  final tile = find.ancestor(
    of: find.textContaining(id),
    matching: find.byType(TripHistoryTile),
  );
  return find
      .descendant(of: tile, matching: find.textContaining('Tài xế:'))
      .evaluate()
      .isNotEmpty;
}

void main() {
  late List<Trip> trips; // T-0047 ... T-0043, đều đã có tài xế hoặc đã huỷ

  setUpAll(() async => trips = (await loadTrips(47)).take(5).toList());

  Future<void> pumpOrder(
    WidgetTester tester,
    List<Trip> order, {
    required bool useKey,
    required bool useFindIndex,
  }) => tester.pumpWidget(
    app(_Harness(trips: order, useKey: useKey, useFindIndex: useFindIndex)),
  );

  group('đảo thứ tự danh sách đang có dòng được mở', () {
    testWidgets('không key: state ở lại vị trí, dòng mở sai chuyến', (
      tester,
    ) async {
      await pumpOrder(tester, trips, useKey: false, useFindIndex: false);
      await tester.tap(find.textContaining('T-0046'));
      await tester.pump();
      expect(isExpanded('T-0046'), isTrue);

      await pumpOrder(
        tester,
        trips.reversed.toList(),
        useKey: false,
        useFindIndex: false,
      );

      // Vị trí số 1 vẫn đang mở, nhưng giờ nó hiển thị chuyến T-0044.
      expect(isExpanded('T-0046'), isFalse);
      expect(isExpanded('T-0044'), isTrue);
    });

    testWidgets('ValueKey và findChildIndexCallback: state đi theo chuyến', (
      tester,
    ) async {
      await pumpOrder(tester, trips, useKey: true, useFindIndex: true);
      await tester.tap(find.textContaining('T-0046'));
      await tester.pump();
      expect(isExpanded('T-0046'), isTrue);

      await pumpOrder(
        tester,
        trips.reversed.toList(),
        useKey: true,
        useFindIndex: true,
      );

      expect(isExpanded('T-0046'), isTrue);
      expect(isExpanded('T-0044'), isFalse);
    });

    testWidgets('ghim T-0044 lên đầu: không key thì T-0047 mở nhầm, có key thì '
        'T-0046 vẫn mở', (tester) async {
      final pinned = [trips[3], trips[0], trips[1], trips[2], trips[4]];

      for (final keyed in [false, true]) {
        await pumpOrder(tester, trips, useKey: keyed, useFindIndex: keyed);
        await tester.tap(find.textContaining('T-0046'));
        await tester.pump();

        await pumpOrder(tester, pinned, useKey: keyed, useFindIndex: keyed);

        expect(isExpanded('T-0046'), keyed);
        expect(isExpanded('T-0047'), !keyed);

        // Dựng lại từ đầu cho vòng lặp kế.
        await tester.pumpWidget(const SizedBox());
      }
    });

    testWidgets(
      'chỉ ValueKey, thiếu findChildIndexCallback: không dính sai chuyến '
      'nhưng state bị mất',
      (tester) async {
        await pumpOrder(tester, trips, useKey: true, useFindIndex: false);
        await tester.tap(find.textContaining('T-0046'));
        await tester.pump();
        expect(isExpanded('T-0046'), isTrue);

        await pumpOrder(
          tester,
          trips.reversed.toList(),
          useKey: true,
          useFindIndex: false,
        );

        // T-0046 đổi vị trí nên phần tử cũ không được nối lại: dòng gập lại.
        expect(isExpanded('T-0046'), isFalse);
        // Và vị trí số 1 không còn "mở nhầm" T-0044 như trường hợp không key.
        expect(isExpanded('T-0044'), isFalse);
      },
    );

    testWidgets('dòng không đổi vị trí giữ state dù thiếu '
        'findChildIndexCallback', (tester) async {
      await pumpOrder(tester, trips, useKey: true, useFindIndex: false);
      await tester.tap(find.textContaining('T-0045'));
      await tester.pump();

      await pumpOrder(
        tester,
        trips.reversed.toList(),
        useKey: true,
        useFindIndex: false,
      );

      // T-0045 ở giữa danh sách 5 phần tử nên đảo xong vẫn ở vị trí số 2.
      expect(isExpanded('T-0045'), isTrue);
    });
  });
}

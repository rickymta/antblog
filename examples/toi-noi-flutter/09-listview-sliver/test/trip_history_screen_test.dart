import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_09/trip_history_screen.dart';
import 'package:toi_noi_09/trip_history_source.dart';
import 'package:toi_noi_09/trip_history_tile.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _bottom = Offset(0, -5000);

Finder get _scrollView => find.byType(CustomScrollView);

/// Cuộn xuống cuối rồi lắc nhẹ lên xuống trong vùng ngưỡng. Mỗi cú lắc đổi vị
/// trí cuộn nên gọi listener của ScrollController một lần nữa, giống ngón tay
/// người dùng còn đang trượt gần cuối danh sách.
Future<void> _scrollToEndRepeatedly(
  WidgetTester tester, {
  int times = 3,
}) async {
  await tester.drag(_scrollView, _bottom);
  await tester.pump();
  for (var i = 0; i < times; i++) {
    await tester.drag(_scrollView, const Offset(0, 60));
    await tester.pump();
    await tester.drag(_scrollView, const Offset(0, -60));
    await tester.pump();
  }
}

/// `RefreshIndicatorState.show` chạy một animation ngắn rồi mới gọi
/// `onRefresh`: frame đầu bắt đầu animation, frame sau kết thúc nó.
Future<void> _finishSnap(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(seconds: 1));
}

void main() {
  late List<Trip> all; // 47 chuyến, T-0047 mới nhất

  setUpAll(() async => all = await loadTrips(47));

  Future<ManualSource> pumpScreen(
    WidgetTester tester, {
    int pageSize = 20,
    List<Trip>? data,
  }) async {
    final source = ManualSource(data ?? all);
    await tester.pumpWidget(
      MaterialApp(
        home: TripHistoryScreen(source: source, pageSize: pageSize),
      ),
    );
    return source;
  }

  group('tải trang đầu', () {
    testWidgets('hiện vòng quay trong lúc chờ rồi hiện danh sách', (
      tester,
    ) async {
      final source = await pumpScreen(tester);

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(source.offsets, [0]);

      source.respond(0);
      await tester.pump();

      expect(find.byType(CircularProgressIndicator), findsNothing);
      expect(find.textContaining('T-0047'), findsOneWidget);
      expect(find.textContaining('Đã tải 20 chuyến'), findsOneWidget);
    });

    testWidgets('tải 40 chuyến nhưng chỉ dựng một phần nhỏ thành widget', (
      tester,
    ) async {
      final source = await pumpScreen(tester, pageSize: 40);
      source.respond(0);
      await tester.pump();

      final tiles = tester.widgetList(find.byType(TripHistoryTile)).length;
      expect(tiles, inInclusiveRange(1, 15));
      expect(find.textContaining('T-0007', skipOffstage: false), findsNothing);
    });

    testWidgets('tóm tắt cộng cước các chuyến hoàn thành bằng totalFare', (
      tester,
    ) async {
      final source = await pumpScreen(tester, pageSize: 5);
      source.respond(0);
      await tester.pump();

      // 5 chuyến đầu: 4 hoàn thành, 1 huỷ. Cước hoàn thành đúng công thức.
      final firstFive = all.take(5).toList();
      final expected = firstFive
          .where((t) => t.status == TripStatus.completed)
          .totalFare;
      expect(find.textContaining('4 hoàn thành'), findsOneWidget);
      expect(find.textContaining('tổng cước $expected'), findsOneWidget);
    });

    testWidgets('dòng của chuyến 5 km hiện 25.500 VND, chuyến huỷ hiện trạng '
        'thái', (tester) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();

      expect(find.text('25.500 VND'), findsOneWidget);
      // Chuyến thứ 5 (chỉ số 4) bị huỷ nên không tính cước.
      expect(find.text('Đã huỷ'), findsOneWidget);
    });

    testWidgets('nguồn rỗng hiện "Chưa có chuyến nào"', (tester) async {
      final source = await pumpScreen(tester);
      source.respondWith(0, const TripPage(trips: [], hasMore: false));
      await tester.pump();

      expect(find.text('Chưa có chuyến nào'), findsOneWidget);
      expect(find.byType(TripHistoryTile), findsNothing);
    });

    testWidgets('lỗi lần đầu hiện nút Thử lại, bấm thì gọi lại trang đầu', (
      tester,
    ) async {
      final source = await pumpScreen(tester);
      source.fail(0);
      await tester.pump();

      expect(find.text('Không tải được lịch sử chuyến'), findsOneWidget);

      await tester.tap(find.text('Thử lại'));
      await tester.pump();
      expect(source.offsets, [0, 0]);

      source.respond(1);
      await tester.pump();
      expect(find.textContaining('T-0047'), findsOneWidget);
    });
  });

  group('SliverAppBar', () {
    testWidgets('app bar ghim lại khi cuộn, phần tóm tắt thì trôi đi', (
      tester,
    ) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();
      expect(find.textContaining('Đã tải'), findsOneWidget);

      await tester.drag(_scrollView, const Offset(0, -1200));
      await tester.pump();

      // SliverToBoxAdapter luôn giữ child trong cây, nhưng nó đã trôi khỏi màn
      // hình: đáy của nó nằm trên cạnh trên của vùng nhìn.
      final summary = find.textContaining('Đã tải', skipOffstage: false);
      expect(tester.getBottomLeft(summary).dy, lessThan(kToolbarHeight));
      final title = tester.getTopLeft(find.text('Lịch sử chuyến'));
      expect(title.dy, lessThan(kToolbarHeight));
    });
  });

  group('tải thêm cuối trang', () {
    testWidgets('cuộn tới cuối gọi đúng một lần cho dù có rất nhiều sự kiện '
        'cuộn', (tester) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();
      expect(source.offsets, [0]);

      await _scrollToEndRepeatedly(tester);

      // Trang kế đang chờ nguồn trả lời: chỉ có đúng một lần gọi offset 20.
      expect(source.offsets, [0, 20]);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('mỗi trang đúng một lần gọi tới khi hết, không offset nào '
        'lặp lại', (tester) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();

      for (var call = 1; call <= 2; call++) {
        await _scrollToEndRepeatedly(tester);
        source.respond(call);
        await tester.pump();
      }
      // Sau trang thứ ba (7 chuyến, hasMore false) cuộn thêm cũng không gọi.
      await _scrollToEndRepeatedly(tester);

      expect(source.offsets, [0, 20, 40]);
      expect(source.pending, 0);
      expect(find.text('Đã hết lịch sử'), findsOneWidget);
      expect(
        find.textContaining('Đã tải 47 chuyến', skipOffstage: false),
        findsOneWidget,
      );
    });

    testWidgets('ngưỡng 0 không bao giờ kích hoạt, vì extentAfter không âm', (
      tester,
    ) async {
      final source = ManualSource(all);
      await tester.pumpWidget(
        MaterialApp(
          home: TripHistoryScreen(source: source, loadMoreThreshold: 0),
        ),
      );
      source.respond(0);
      await tester.pump();

      await _scrollToEndRepeatedly(tester);

      expect(source.offsets, [0]);
    });

    testWidgets('trang đầu không lấp đầy màn hình thì tự tải tiếp tới khi đủ', (
      tester,
    ) async {
      final source = await pumpScreen(tester, pageSize: 2);

      var answered = 0;
      while (source.pending > 0 && answered < 40) {
        source.respond(answered++);
        await tester.pump();
      }

      // Không có một lần cuộn nào của người dùng, nhưng màn hình đã được lấp
      // đầy vượt ngưỡng: nhiều hơn một trang, và dừng trước khi tải hết.
      expect(source.offsets.length, greaterThan(2));
      expect(source.offsets.length, lessThan(20));
      expect(source.pending, 0);
      // Mỗi offset đúng một lần, cách nhau đúng một trang.
      expect(source.offsets, [
        for (var i = 0; i < source.offsets.length; i++) i * 2,
      ]);
    });

    testWidgets('tải thêm lỗi: không tự gọi lại, bấm Thử lại gọi lại đúng '
        'trang đó', (tester) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();

      await _scrollToEndRepeatedly(tester, times: 1);
      source.fail(1);
      await tester.pump();
      await _scrollToEndRepeatedly(tester);

      expect(source.offsets, [0, 20]);
      expect(find.text('Không tải thêm được. Thử lại'), findsOneWidget);
      // Danh sách đã tải vẫn còn nguyên.
      expect(
        find.textContaining('Đã tải 20 chuyến', skipOffstage: false),
        findsOneWidget,
      );

      await tester.tap(find.text('Không tải thêm được. Thử lại'));
      await tester.pump();
      expect(source.offsets, [0, 20, 20]);

      source.respond(2);
      await tester.pump();
      expect(find.text('Không tải thêm được. Thử lại'), findsNothing);
    });
  });

  group('kéo để làm mới', () {
    testWidgets('kéo xuống gọi lại trang đầu và chuyến mới hiện lên đầu', (
      tester,
    ) async {
      final source = FakeTripHistorySource(
        total: 47,
        latency: const Duration(milliseconds: 100),
      );
      await tester.pumpWidget(
        MaterialApp(home: TripHistoryScreen(source: source)),
      );
      await tester.pump(const Duration(milliseconds: 100));
      expect(source.fetchCount, 1);
      expect(find.textContaining('T-0047'), findsOneWidget);
      expect(find.textContaining('T-0048'), findsNothing);

      // Trong lúc người dùng đang xem, một chuyến mới kết thúc.
      source.prepend(
        Trip.request(
          id: 'T-0048',
          serviceKind: ServiceKind.ride,
          distanceKm: 5,
          requestedAt: DateTime.utc(2026, 11, 6),
        ),
      );

      await tester.fling(_scrollView, const Offset(0, 300), 1000);
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));
      await tester.pumpAndSettle();

      expect(source.fetchCount, 2);
      expect(find.textContaining('T-0048'), findsOneWidget);
      expect(find.textContaining('Đã tải 20 chuyến'), findsOneWidget);
    });

    testWidgets('danh sách ngắn hơn màn hình vẫn kéo để làm mới được', (
      tester,
    ) async {
      final source = FakeTripHistorySource(
        total: 2,
        latency: const Duration(milliseconds: 100),
      );
      await tester.pumpWidget(
        MaterialApp(home: TripHistoryScreen(source: source)),
      );
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.byType(TripHistoryTile), findsNWidgets(2));
      expect(source.fetchCount, 1);

      await tester.fling(_scrollView, const Offset(0, 300), 1000);
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));
      await tester.pumpAndSettle();

      expect(source.fetchCount, 2);
    });

    testWidgets('kết quả của lượt tải thêm cũ bị bỏ khi đã làm mới', (
      tester,
    ) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();

      // Lượt 1: tải thêm trang offset 20, chưa trả lời.
      await _scrollToEndRepeatedly(tester, times: 1);
      expect(source.offsets, [0, 20]);

      // Lượt 2: người dùng làm mới trong lúc đó, gọi lại offset 0.
      final state = tester.state<RefreshIndicatorState>(
        find.byType(RefreshIndicator),
      );
      state.show();
      await _finishSnap(tester);
      expect(source.offsets, [0, 20, 0]);

      // Làm mới trả lời trước, lượt tải thêm cũ trả lời sau.
      source.respond(2);
      await tester.pump();
      source.respond(1);
      await tester.pump();

      // Ta đang ở cuối danh sách nên trang mới tự hỏi tiếp offset 20 (lần gọi
      // số 3). Lần gọi số 1 đã quá hạn: kết quả của nó không được nối vào.
      expect(source.offsets, [0, 20, 0, 20]);
      expect(
        find.textContaining('Đã tải 20 chuyến', skipOffstage: false),
        findsOneWidget,
      );

      source.respond(3);
      await tester.pump();
      expect(
        find.textContaining('Đã tải 40 chuyến', skipOffstage: false),
        findsOneWidget,
      );
    });

    testWidgets('làm mới lỗi giữ nguyên danh sách và báo bằng SnackBar', (
      tester,
    ) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();

      tester.state<RefreshIndicatorState>(find.byType(RefreshIndicator)).show();
      await _finishSnap(tester);
      source.fail(1);
      await tester.pumpAndSettle();

      expect(find.text('Không làm mới được'), findsOneWidget);
      expect(find.textContaining('T-0047'), findsOneWidget);
      expect(find.textContaining('Đã tải 20 chuyến'), findsOneWidget);
    });
  });

  group('ValueKey trong màn hình thật: ghim chuyến lên đầu', () {
    bool expanded(String id) {
      final tile = find.ancestor(
        of: find.textContaining(id),
        matching: find.byType(TripHistoryTile),
      );
      return find
          .descendant(of: tile, matching: find.textContaining('Tài xế:'))
          .evaluate()
          .isNotEmpty;
    }

    testWidgets('state mở/gập đi theo chuyến khi thứ tự đổi', (tester) async {
      final source = await pumpScreen(tester);
      source.respond(0);
      await tester.pump();

      // Mở T-0045 (vị trí 2) và T-0044 (vị trí 3).
      await tester.tap(find.textContaining('T-0045'));
      await tester.pump();
      await tester.tap(find.textContaining('T-0044'));
      await tester.pump();
      expect(expanded('T-0045'), isTrue);
      expect(expanded('T-0044'), isTrue);
      expect(expanded('T-0046'), isFalse);

      // Ghim T-0044 lên đầu: mọi dòng từ vị trí 0 đến 2 dịch xuống một nấc.
      await tester.tap(find.text('Ghim lên đầu').last);
      await tester.pump();

      final order = tester
          .widgetList<TripHistoryTile>(find.byType(TripHistoryTile))
          .take(4)
          .map((t) => t.trip.id);
      expect(order, ['T-0044', 'T-0047', 'T-0046', 'T-0045']);
      expect(expanded('T-0044'), isTrue);
      expect(expanded('T-0045'), isTrue);
      expect(expanded('T-0046'), isFalse);
      expect(expanded('T-0047'), isFalse);
    });
  });

  group('TripHistoryTile', () {
    test('formatTripTime định dạng dd/MM/yyyy HH:mm có số 0 đứng đầu', () {
      expect(formatTripTime(DateTime(2026, 11, 5, 9, 5)), '05/11/2026 09:05');
      expect(
        formatTripTime(DateTime(2026, 12, 31, 23, 59)),
        '31/12/2026 23:59',
      );
    });
  });
}

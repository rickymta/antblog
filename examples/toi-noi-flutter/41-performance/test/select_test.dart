import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/build_log.dart';
import 'package:toi_noi_41/trip_live.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  late ProviderContainer container;

  setUp(() {
    BuildLog.reset();
    container = ProviderContainer.test();
  });

  Future<void> pumpLabels(WidgetTester tester) => tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: const MaterialApp(
        home: Column(
          children: [
            FareLabelWatchAll(),
            FareLabelSelect(),
            FareLabelSelectList(),
            FareStatusLabel(),
          ],
        ),
      ),
    ),
  );

  Future<void> moveDriverTimes(WidgetTester tester, int times) async {
    final notifier = container.read(tripLiveProvider.notifier);
    for (var i = 1; i <= times; i++) {
      notifier.moveDriver(10.0 + i / 1000, 106.0 + i / 1000);
      await tester.pump();
    }
  }

  testWidgets('lần dựng đầu: mỗi nhãn build một lần, cùng hiện 25.500 VND', (
    tester,
  ) async {
    await pumpLabels(tester);

    expect(BuildLog.of('fare-all'), 1);
    expect(BuildLog.of('fare-select'), 1);
    expect(find.text('25.500 VND'), findsNWidgets(3));
  });

  group('tài xế di chuyển 5 lần, cước không đổi', () {
    testWidgets('watch cả state build lại 5 lần', (tester) async {
      await pumpLabels(tester);
      BuildLog.reset();

      await moveDriverTimes(tester, 5);

      expect(BuildLog.of('fare-all'), 5);
    });

    testWidgets('select cước không build lại lần nào', (tester) async {
      await pumpLabels(tester);
      BuildLog.reset();

      await moveDriverTimes(tester, 5);

      expect(BuildLog.of('fare-select'), 0);
    });

    testWidgets('select trả List mới mỗi lần: vẫn build lại 5 lần', (
      tester,
    ) async {
      await pumpLabels(tester);
      BuildLog.reset();

      await moveDriverTimes(tester, 5);

      expect(BuildLog.of('fare-select-list'), 5);
    });
  });

  group('cước đổi thật', () {
    testWidgets('mọi nhãn build lại đúng một lần và hiện số mới', (
      tester,
    ) async {
      await pumpLabels(tester);
      BuildLog.reset();

      container.read(tripLiveProvider.notifier).reroute(100);
      await tester.pump();

      expect(BuildLog.of('fare-all'), 1);
      expect(BuildLog.of('fare-select'), 1);
      expect(find.text('453.000 VND'), findsNWidgets(3));
    });

    testWidgets('reroute về cùng quãng đường: cước bằng nhau, select im', (
      tester,
    ) async {
      await pumpLabels(tester);
      BuildLog.reset();

      container.read(tripLiveProvider.notifier).reroute(5);
      await tester.pump();

      // Trip mới bằng Trip cũ theo giá trị nên cả state không đổi.
      expect(BuildLog.of('fare-all'), 0);
      expect(BuildLog.of('fare-select'), 0);
    });
  });

  group('đổi trạng thái chuyến nhưng cước giữ nguyên', () {
    testWidgets('watch cả state build lại, select cước thì không', (
      tester,
    ) async {
      await pumpLabels(tester);
      BuildLog.reset();

      container.read(tripLiveProvider.notifier).apply(const SearchStarted());
      await tester.pump();

      expect(BuildLog.of('fare-all'), 1);
      expect(BuildLog.of('fare-select'), 0);
    });
  });

  group('select trả về record', () {
    testWidgets('tài xế di chuyển: không build lại', (tester) async {
      await pumpLabels(tester);
      BuildLog.reset();

      await moveDriverTimes(tester, 5);

      expect(BuildLog.of('fare-status'), 0);
    });

    testWidgets('đổi trạng thái: build đúng một lần và hiện trạng thái mới', (
      tester,
    ) async {
      await pumpLabels(tester);
      BuildLog.reset();

      container.read(tripLiveProvider.notifier).apply(const SearchStarted());
      await tester.pump();

      expect(BuildLog.of('fare-status'), 1);
      expect(find.text('25.500 VND · searching'), findsOneWidget);
    });
  });

  test('state có == theo giá trị: hai TripLive giống nhau thì bằng nhau', () {
    final live = container.read(tripLiveProvider);

    expect(live.copyWith(), live);
    expect(live.copyWith(driverLat: 11), isNot(live));
  });
}

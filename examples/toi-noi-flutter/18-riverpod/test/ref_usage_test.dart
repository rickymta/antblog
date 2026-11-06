import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/trip_draft.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

ProviderContainer _container(WidgetTester tester) =>
    ProviderScope.containerOf(tester.element(find.byType(Consumer)));

void main() {
  group('select giới hạn phạm vi build lại', () {
    testWidgets('chỉ build lại khi phần được chọn đổi', (tester) async {
      var serviceBuilds = 0;
      var distanceBuilds = 0;
      await tester.pumpWidget(
        ProviderScope(
          child: Column(
            children: [
              Consumer(
                builder: (context, ref, _) {
                  serviceBuilds++;
                  ref.watch(tripDraftProvider.select((d) => d.serviceKind));
                  return const SizedBox();
                },
              ),
              Consumer(
                builder: (context, ref, _) {
                  distanceBuilds++;
                  ref.watch(tripDraftProvider.select((d) => d.distanceKm));
                  return const SizedBox();
                },
              ),
            ],
          ),
        ),
      );
      final container = ProviderScope.containerOf(
        tester.element(find.byType(Column)),
      );
      expect((serviceBuilds, distanceBuilds), (1, 1));

      container.read(tripDraftProvider.notifier).setDistanceKm(9);
      await tester.pump();
      expect((serviceBuilds, distanceBuilds), (1, 2));

      container
          .read(tripDraftProvider.notifier)
          .selectService(ServiceKind.food);
      await tester.pump();
      expect((serviceBuilds, distanceBuilds), (2, 2));
    });

    testWidgets('watch cả bản nháp thì đổi trường nào cũng build lại', (
      tester,
    ) async {
      var builds = 0;
      await tester.pumpWidget(
        ProviderScope(
          child: Consumer(
            builder: (context, ref, _) {
              builds++;
              ref.watch(tripDraftProvider);
              return const SizedBox();
            },
          ),
        ),
      );

      _container(tester)
          .read(tripDraftProvider.notifier)
          .selectService(ServiceKind.food);
      await tester.pump();

      expect(builds, 2);
    });
  });

  group('watch và read trong build', () {
    testWidgets('watch cập nhật giao diện, read trong build thì không', (
      tester,
    ) async {
      await tester.pumpWidget(
        ProviderScope(
          child: Directionality(
            textDirection: TextDirection.ltr,
            child: Column(
              children: [
                Consumer(
                  builder: (context, ref, _) =>
                      Text('watch ${ref.watch(tripDraftProvider).distanceKm}'),
                ),
                Consumer(
                  builder: (context, ref, _) =>
                      Text('read ${ref.read(tripDraftProvider).distanceKm}'),
                ),
              ],
            ),
          ),
        ),
      );
      final container = ProviderScope.containerOf(
        tester.element(find.byType(Column)),
      );

      container.read(tripDraftProvider.notifier).setDistanceKm(9);
      await tester.pump();

      expect(find.text('watch 9.0'), findsOneWidget);
      expect(find.text('read 5.0'), findsOneWidget); // đã cũ, không build lại
    });
  });

  group('dùng sai chỗ', () {
    testWidgets('ref.listen trong onPressed bị assert chặn', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Consumer(
              builder: (context, ref, _) => TextButton(
                onPressed: () => ref.listen(tripDraftProvider, (_, _) {}),
                child: const Text('nghe'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('nghe'));

      final error = tester.takeException();
      expect(error, isA<AssertionError>());
      expect(
        error.toString(),
        contains('ref.listen can only be used within the build method'),
      );
    });

    testWidgets('thiếu ProviderScope thì lỗi ngay khi build', (tester) async {
      await tester.pumpWidget(
        Consumer(
          builder: (context, ref, _) => Text('${ref.watch(tripDraftProvider)}'),
        ),
      );

      final error = tester.takeException();
      expect(error, isA<StateError>());
      expect(error.toString(), contains('No ProviderScope found'));
    });
  });
}

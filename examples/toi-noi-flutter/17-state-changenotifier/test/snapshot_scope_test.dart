import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/quote_draft.dart';
import 'package:toi_noi_17/snapshot_scope.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// InheritedWidget luôn đánh thức dependent: ca đối chứng cho `updateShouldNotify`.
class _AlwaysNotifyScope extends InheritedWidget {
  const _AlwaysNotifyScope({required this.snapshot, required super.child});

  final QuoteSnapshot snapshot;

  static QuoteSnapshot of(BuildContext context) => context
      .dependOnInheritedWidgetOfExactType<_AlwaysNotifyScope>()!
      .snapshot;

  @override
  bool updateShouldNotify(_AlwaysNotifyScope oldWidget) => true;
}

/// Chỉ so cước: sai khi có dependent đọc dịch vụ.
class _FareOnlyScope extends InheritedWidget {
  const _FareOnlyScope({required this.snapshot, required super.child});

  final QuoteSnapshot snapshot;

  static QuoteSnapshot of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<_FareOnlyScope>()!.snapshot;

  @override
  bool updateShouldNotify(_FareOnlyScope oldWidget) =>
      snapshot.fare != oldWidget.snapshot.fare;
}

QuoteSnapshot _snapshot({double km = 5, ServiceKind kind = ServiceKind.ride}) =>
    QuoteSnapshot(service: kind, km: km, fare: calculateFare(km));

void main() {
  group('updateShouldNotify', () {
    testWidgets(
      'cha dựng lại với ảnh chụp bằng nhau: dependent không dựng lại',
      (tester) async {
        var builds = 0;
        final probe = Probe(
          onBuild: () => builds++,
          read: (c) => '${QuoteSnapshotScope.of(c).fare}',
        );
        late StateSetter rebuild;
        var snapshot = _snapshot();

        await tester.pumpWidget(
          StatefulBuilder(
            builder: (context, setState) {
              rebuild = setState;
              // `probe` là cùng một đối tượng ở mọi lần dựng, nên framework
              // không có lý do dựng lại nó nếu không ai đánh thức nó.
              return QuoteSnapshotScope(snapshot: snapshot, child: probe);
            },
          ),
        );
        expect(builds, 1);

        // Một QuoteSnapshot mới tinh nhưng bằng giá trị cũ.
        rebuild(() => snapshot = _snapshot());
        await tester.pump();
        expect(builds, 1);

        // Đổi thật: dependent được đánh thức và thấy cước mới.
        rebuild(() => snapshot = _snapshot(km: 6));
        await tester.pump();
        expect(builds, 2);
        expect(find.text('30.000 VND'), findsOneWidget);
      },
    );

    testWidgets('updateShouldNotify luôn true: dựng lại dù không có gì đổi', (
      tester,
    ) async {
      var builds = 0;
      final probe = Probe(
        onBuild: () => builds++,
        read: (c) => '${_AlwaysNotifyScope.of(c).fare}',
      );
      late StateSetter rebuild;

      await tester.pumpWidget(
        StatefulBuilder(
          builder: (context, setState) {
            rebuild = setState;
            return _AlwaysNotifyScope(snapshot: _snapshot(), child: probe);
          },
        ),
      );
      expect(builds, 1);

      rebuild(() {});
      await tester.pump();
      expect(builds, 2);
    });
  });

  testWidgets(
    'updateShouldNotify chỉ so cước: dependent đọc dịch vụ bị bỏ sót',
    (tester) async {
      final probe = Probe(
        onBuild: () {},
        read: (c) => _FareOnlyScope.of(c).service.name,
      );
      late StateSetter rebuild;
      var snapshot = _snapshot();

      await tester.pumpWidget(
        StatefulBuilder(
          builder: (context, setState) {
            rebuild = setState;
            return _FareOnlyScope(snapshot: snapshot, child: probe);
          },
        ),
      );
      expect(find.text('ride'), findsOneWidget);

      // Đổi dịch vụ, cước vẫn 25.500: không ai được báo.
      rebuild(() => snapshot = _snapshot(kind: ServiceKind.food));
      await tester.pump();

      expect(find.text('ride'), findsOneWidget);
      expect(find.text('food'), findsNothing);
    },
  );

  group('QuoteSnapshotHost', () {
    testWidgets('draft đổi thì dependent dựng lại đúng một lần mỗi lần đổi', (
      tester,
    ) async {
      final draft = QuoteDraft();
      addTearDown(draft.dispose);
      var builds = 0;

      await tester.pumpWidget(
        QuoteSnapshotHost(
          draft: draft,
          child: Probe(
            onBuild: () => builds++,
            read: (c) => '${QuoteSnapshotScope.of(c).fare}',
          ),
        ),
      );
      expect(find.text('25.500 VND'), findsOneWidget);

      draft.setKm(2.001);
      await tester.pump();
      expect(builds, 2);
      expect(find.text('12.005 VND'), findsOneWidget);
    });

    testWidgets('gỡ khỏi cây thì huỷ đăng ký lắng nghe draft', (tester) async {
      final draft = ObservableDraft();
      addTearDown(draft.dispose);

      await tester.pumpWidget(
        QuoteSnapshotHost(draft: draft, child: const SizedBox()),
      );
      expect(draft.isListened, isTrue);

      await tester.pumpWidget(const SizedBox());
      expect(draft.isListened, isFalse);
    });
  });
}

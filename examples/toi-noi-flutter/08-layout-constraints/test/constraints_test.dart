import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_08/constraint_demos.dart';

void main() {
  group('constraints đi xuống, kích thước đi lên', () {
    testWidgets('cha ép chặt 300x300 thì con đòi 100x100 vẫn thành 300x300', (
      tester,
    ) async {
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Center(child: TightParentDemo(loosen: false)),
        ),
      );
      expect(
        tester.getSize(find.byKey(TightParentDemo.boxKey)),
        const Size(300, 300),
      );
    });

    testWidgets('chen Align nới lỏng thì con được 100x100 và nằm giữa cha', (
      tester,
    ) async {
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Center(child: TightParentDemo(loosen: true)),
        ),
      );
      final box = find.byKey(TightParentDemo.boxKey);
      expect(tester.getSize(box), const Size(100, 100));

      // Vị trí do cha quyết: tâm của con trùng tâm của cha 300x300.
      final parent = tester.getRect(find.byType(TightParentDemo));
      expect(tester.getCenter(box), parent.center);
    });

    testWidgets('Center truyền constraints lỏng, nên cha 300x300 chỉ là kích '
        'thước của SizedBox đó', (tester) async {
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Center(child: TightParentDemo(loosen: true)),
        ),
      );
      expect(
        tester.getSize(find.byType(TightParentDemo)),
        const Size(300, 300),
      );
    });
  });

  group('Row, Expanded và Flexible', () {
    testWidgets('60 cố định + flex 1 + flex 2 trong 360 px thành 60/100/200', (
      tester,
    ) async {
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Center(child: FlexRatioRow()),
        ),
      );
      expect(tester.getSize(find.byKey(FlexRatioRow.fixedKey)).width, 60);
      expect(tester.getSize(find.byKey(FlexRatioRow.oneKey)).width, 100);
      expect(tester.getSize(find.byKey(FlexRatioRow.twoKey)).width, 200);
    });

    testWidgets('Flexible (loose) giữ 50 px, Expanded (tight) lấy đủ 150 px', (
      tester,
    ) async {
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Center(child: FlexFitRow()),
        ),
      );
      expect(tester.getSize(find.byKey(FlexFitRow.flexibleKey)).width, 50);
      expect(tester.getSize(find.byKey(FlexFitRow.expandedKey)).width, 150);
    });
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/fare_readers.dart';
import 'package:toi_noi_17/quote_draft.dart';
import 'package:toi_noi_17/quote_scope.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Ba widget cùng đọc cước của một draft, mỗi cái một cách.
Future<(QuoteDraft, BuildCounts)> _pump(WidgetTester tester) async {
  final draft = QuoteDraft(km: 1); // cước 12.000, còn trong 2 km đầu
  addTearDown(draft.dispose);
  final counts = BuildCounts();
  await tester.pumpWidget(
    QuoteScope(
      draft: draft,
      child: MaterialApp(
        theme: ToiNoiTheme.light(),
        home: Column(
          children: [
            FareFromScope(counts: counts),
            FareFromValue(counts: counts),
            FareFromWholeNotifier(draft: draft, counts: counts),
          ],
        ),
      ),
    ),
  );
  return (draft, counts);
}

/// (scope, value, whole): số lần build của ba cách đọc.
(int, int, int) _tally(BuildCounts c) => (c['scope'], c['value'], c['whole']);

void main() {
  group('ranh giới rebuild của ba cách đọc cước', () {
    testWidgets('lần dựng đầu: mỗi cách build đúng một lần', (tester) async {
      final (_, counts) = await _pump(tester);

      expect(_tally(counts), (1, 1, 1));
      expect(counts['value:widget'], 1);
      expect(counts['caption'], 1);
    });

    testWidgets('đổi dịch vụ (thứ không liên quan tới cước)', (tester) async {
      final (draft, counts) = await _pump(tester);

      draft.selectService(ServiceKind.food);
      await tester.pump();

      // scope và whole bị đánh thức oan; value đứng yên.
      expect(_tally(counts), (2, 1, 2));
    });

    testWidgets('đổi km nhưng cước không đổi (1 km sang 1,5 km)', (
      tester,
    ) async {
      final (draft, counts) = await _pump(tester);

      draft.setKm(1.5);
      await tester.pump();

      expect(draft.fare.vnd, 12000);
      expect(_tally(counts), (2, 1, 2));
    });

    testWidgets('đổi km làm cước đổi (1 km sang 5 km)', (tester) async {
      final (draft, counts) = await _pump(tester);

      draft.setKm(5);
      await tester.pump();

      expect(_tally(counts), (2, 2, 2));
      // Cả ba hiện cùng một con số mới.
      expect(find.text('25.500 VND'), findsNWidgets(3));
    });

    testWidgets('chọn lại đúng dịch vụ đang chọn: không ai build lại', (
      tester,
    ) async {
      final (draft, counts) = await _pump(tester);

      draft.selectService(ServiceKind.ride);
      await tester.pump();

      expect(_tally(counts), (1, 1, 1));
    });

    testWidgets('widget đọc bằng read không bao giờ build lại phần của nó', (
      tester,
    ) async {
      final (draft, counts) = await _pump(tester);

      draft
        ..selectService(ServiceKind.parcel)
        ..setKm(1.5)
        ..setKm(5)
        ..setKm(20);
      await tester.pump();

      expect(counts['value:widget'], 1);
      // `child` của ValueListenableBuilder dựng một lần dù cước đổi.
      expect(counts['caption'], 1);
      expect(find.text(calculateFare(20).toString()), findsNWidgets(3));
    });
  });
}

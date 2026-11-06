import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_09/main.dart';
import 'package:toi_noi_09/trip_history_source.dart';

void main() {
  testWidgets('ToiNoiApp mở thẳng màn hình lịch sử và tải trang đầu', (
    tester,
  ) async {
    final source = FakeTripHistorySource(
      latency: const Duration(milliseconds: 400),
    );
    await tester.pumpWidget(ToiNoiApp(source: source));

    expect(find.text('Lịch sử chuyến'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    await tester.pump(const Duration(milliseconds: 400));

    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(find.textContaining('T-0047'), findsOneWidget);
    expect(source.fetchCount, 1);
    expect(
      Theme.of(tester.element(find.byType(Scaffold))).useMaterial3,
      isTrue,
    );
  });
}

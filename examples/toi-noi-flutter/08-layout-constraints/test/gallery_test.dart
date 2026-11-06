import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_08/main.dart';
import 'package:toi_noi_08/trip_card.dart';

import 'support.dart';

void main() {
  for (final width in [320.0, 390.0, 768.0]) {
    testWidgets('gallery ở $width px dựng đủ 4 thẻ, không lỗi layout', (
      tester,
    ) async {
      setScreen(tester, width);
      await tester.pumpWidget(const ToiNoiApp());

      expect(find.byType(TripCard), findsNWidgets(4));
      expect(find.text('25.500 VND'), findsOneWidget);
      expect(find.text('453.000 VND'), findsOneWidget);
      expect(tester.takeException(), isNull);
    });
  }
}

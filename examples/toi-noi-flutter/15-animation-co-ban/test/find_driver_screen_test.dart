import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_15/app.dart';
import 'package:toi_noi_15/driver.dart';
import 'package:toi_noi_15/driver_card.dart';
import 'package:toi_noi_15/search_status_panel.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

Trip _trip() => Trip.request(
  id: 'trip-test',
  serviceKind: ServiceKind.ride,
  distanceKm: 5,
  requestedAt: DateTime.utc(2026, 11, 6, 8),
);

final _next = find.byKey(const Key('next-step'));
final _cancel = find.byKey(const Key('cancel-trip'));

Future<void> _step(WidgetTester tester) async {
  await tester.tap(_next);
  await tester.pumpAndSettle();
}

double _ringValue(WidgetTester tester) => tester
    .widget<CircularProgressIndicator>(find.byType(CircularProgressIndicator))
    .value!;

void main() {
  group('FindDriverScreen', () {
    testWidgets('chuyến mới đặt: vòng 0%, chưa có thẻ tài xế', (tester) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));
      expect(_ringValue(tester), 0);
      expect(find.text(messageOf(TripStatus.requested)), findsOneWidget);
      expect(find.byType(DriverCard), findsNothing);
    });

    testWidgets('đi hết đường bình thường: vòng, câu trạng thái và thẻ đổi '
        'theo từng sự kiện', (tester) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));

      for (final status in [
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
        TripStatus.pickedUp,
        TripStatus.inTransit,
        TripStatus.completed,
      ]) {
        await _step(tester);
        expect(_ringValue(tester), closeTo(status.progress, 1e-9));
        expect(find.text(messageOf(status)), findsOneWidget);
      }

      // Ở trạng thái cuối, nút tiếp theo bị khoá và vòng đầy 100%.
      expect(tester.widget<FilledButton>(_next).onPressed, isNull);
      expect(find.text('100%'), findsOneWidget);
    });

    testWidgets('thẻ tài xế xuất hiện khi tài xế nhận chuyến', (tester) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));
      await _step(tester); // searching
      expect(find.byType(DriverCard), findsNothing);

      await _step(tester); // accepted
      expect(find.byType(DriverCard), findsOneWidget);
      expect(find.text(kDrivers.first.name), findsOneWidget);
    });

    testWidgets('huỷ chỉ bấm được ở searching, accepted, arriving', (
      tester,
    ) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));
      bool canCancel() => tester.widget<TextButton>(_cancel).onPressed != null;

      final expected = {
        TripStatus.requested: false,
        TripStatus.searching: true,
        TripStatus.accepted: true,
        TripStatus.arriving: true,
        TripStatus.pickedUp: false,
        TripStatus.inTransit: false,
        TripStatus.completed: false,
      };
      for (final entry in expected.entries) {
        expect(canCancel(), entry.value, reason: entry.key.name);
        if (entry.key != TripStatus.completed) await _step(tester);
      }
    });

    testWidgets('huỷ khi đang tìm: vòng về 0%, hiện "đã huỷ", thẻ biến mất', (
      tester,
    ) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));
      await _step(tester); // searching
      await _step(tester); // accepted
      await _step(tester); // arriving
      expect(find.byType(DriverCard), findsOneWidget);

      await tester.tap(_cancel);
      await tester.pumpAndSettle();

      expect(_ringValue(tester), 0);
      expect(find.text(messageOf(TripStatus.cancelled)), findsOneWidget);
      expect(find.byType(DriverCard), findsNothing);
      expect(tester.widget<FilledButton>(_next).onPressed, isNull);
    });

    testWidgets('bấm thẻ tài xế mở màn chi tiết, quay lại vẫn ở đúng trạng '
        'thái', (tester) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));
      await _step(tester);
      await _step(tester); // accepted

      await tester.tap(find.byType(DriverCard));
      await tester.pumpAndSettle();
      expect(find.text('Tài xế của bạn'), findsOneWidget);
      expect(find.text('Biển số: ${kDrivers.first.plate}'), findsOneWidget);

      await tester.pageBack();
      await tester.pumpAndSettle();
      expect(find.text('Tìm tài xế'), findsOneWidget);
      expect(find.text(messageOf(TripStatus.accepted)), findsOneWidget);
    });

    testWidgets('không có animation lặp vô hạn: pumpAndSettle kết '
        'thúc', (tester) async {
      await tester.pumpWidget(FindDriverApp(initialTrip: _trip()));
      await _step(tester);
      await _step(tester);
      await _step(tester); // arriving: đủ cả ba loại animation đã chạy
      expect(tester.hasRunningAnimations, isFalse);
    });
  });
}

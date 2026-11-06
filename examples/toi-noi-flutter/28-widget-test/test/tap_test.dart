import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/quote.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _place = Key('place-trip');

final _quote5km = Quote(
  serviceKind: ServiceKind.ride,
  distanceKm: 5,
  fare: calculateFare(5),
);

/// Dựng app ở màn hình điện thoại nhỏ với báo giá 5 km đã về.
Future<ControlledQuoteRepository> _pumpPhone(WidgetTester tester) async {
  usePhoneScreen(tester);
  final repository = ControlledQuoteRepository();
  await pumpApp(tester, repository: repository);
  repository.completeQuote();
  await tester.pump();
  return repository;
}

double _screenHeight(WidgetTester tester) =>
    tester.view.physicalSize.height / tester.view.devicePixelRatio;

void main() {
  group('tap, warnIfMissed, ensureVisible', () {
    testWidgets('nút Đặt chuyến nằm dưới đáy màn hình 360x640', (tester) async {
      await _pumpPhone(tester);

      final center = tester.getCenter(find.byKey(_place));

      expect(center.dy, greaterThan(_screenHeight(tester)));
    });

    testWidgets('tap ngoài màn hình: không có gì xảy ra', (tester) async {
      final repository = await _pumpPhone(tester);

      // warnIfMissed: false chỉ tắt cảnh báo; cú chạm vẫn rơi vào chỗ trống.
      await tester.tap(find.byKey(_place), warnIfMissed: false);
      await tester.pump();

      expect(repository.placeCount, 0);
    });

    testWidgets('hitTestWarningShouldBeFatal biến cảnh báo thành lỗi', (
      tester,
    ) async {
      await _pumpPhone(tester);
      WidgetController.hitTestWarningShouldBeFatal = true;
      addTearDown(() => WidgetController.hitTestWarningShouldBeFatal = false);

      expect(
        () => tester.tap(find.byKey(_place)),
        throwsA(
          isA<FlutterError>().having(
            (e) => e.toString(),
            'message',
            contains('would not receive pointer events'),
          ),
        ),
      );
    });

    testWidgets('ensureVisible cuộn tới nút rồi tap mới trúng', (tester) async {
      final repository = await _pumpPhone(tester);

      await tester.ensureVisible(find.byKey(_place));
      await tester.pump();
      expect(
        tester.getCenter(find.byKey(_place)).dy,
        lessThan(_screenHeight(tester)),
      );

      await tester.tap(find.byKey(_place));
      await tester.pump();

      expect(repository.placeCount, 1);
    });

    testWidgets('tap lần hai trúng ModalBarrier, không đặt trùng chuyến', (
      tester,
    ) async {
      final repository = await _pumpPhone(tester);
      await tester.ensureVisible(find.byKey(_place));
      await tester.pump();

      await tester.tap(find.byKey(_place));
      await tester.pump();
      expect(find.byKey(const Key('placing')), findsOneWidget);

      // Lớp phủ "đang đặt" chặn cú chạm: tap trúng ModalBarrier. Cảnh báo
      // là đúng, ta cố ý nên tắt nó.
      await tester.tap(find.byKey(_place), warnIfMissed: false);
      await tester.pump();

      expect(repository.placeCount, 1);

      // Dọn: cho yêu cầu đặt chuyến xong để test không còn việc treo.
      repository.completePlace(_quote5km);
      await tester.pumpAndSettle();
    });

    testWidgets('trong lúc đặt, tap bình thường báo ngay là trượt', (
      tester,
    ) async {
      final repository = await _pumpPhone(tester);
      await tester.ensureVisible(find.byKey(_place));
      await tester.pump();
      await tester.tap(find.byKey(_place));
      await tester.pump();
      WidgetController.hitTestWarningShouldBeFatal = true;
      addTearDown(() => WidgetController.hitTestWarningShouldBeFatal = false);

      expect(
        () => tester.tap(find.byKey(_place)),
        throwsA(isA<FlutterError>()),
      );

      repository.completePlace(_quote5km);
      await tester.pumpAndSettle();
    });

    testWidgets('tap nút bị vô hiệu hoá không gọi onPressed', (tester) async {
      var pressed = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Column(
              children: [
                FilledButton(
                  key: const Key('enabled'),
                  onPressed: () => pressed++,
                  child: const Text('Bật'),
                ),
                const FilledButton(
                  key: Key('disabled'),
                  onPressed: null,
                  child: Text('Tắt'),
                ),
              ],
            ),
          ),
        ),
      );

      // Nút tắt vẫn nhận chạm (hit test trúng), chỉ không làm gì. Bật chế độ
      // lỗi để chắc chắn không có cảnh báo trượt nào bị bỏ qua.
      WidgetController.hitTestWarningShouldBeFatal = true;
      addTearDown(() => WidgetController.hitTestWarningShouldBeFatal = false);

      await tester.tap(find.byKey(const Key('enabled')));
      await tester.tap(find.byKey(const Key('disabled')));

      expect(pressed, 1);
    });
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_10/find_driver_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

Finder _remaining(int seconds) =>
    find.text('Đang tìm tài xế, còn $seconds giây');

Widget _screen({Duration cancelDelay = Duration.zero}) =>
    MaterialApp(home: FindDriverScreen(cancelDelay: cancelDelay));

void main() {
  group('searchSecondsFor', () {
    test('mỗi dịch vụ có thời gian chờ riêng (số minh hoạ)', () {
      expect(searchSecondsFor(ServiceKind.ride), 30);
      expect(searchSecondsFor(ServiceKind.food), 45);
      expect(searchSecondsFor(ServiceKind.parcel), 60);
    });
  });

  group('FindDriverScreen', () {
    testWidgets('bấm Tìm tài xế bắt đầu đếm từ thời gian của dịch vụ', (
      tester,
    ) async {
      await tester.pumpWidget(_screen());

      await tester.tap(find.widgetWithText(FilledButton, 'Tìm tài xế'));
      await tester.pump();
      expect(_remaining(30), findsOneWidget);

      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(29), findsOneWidget);
    });

    testWidgets('đổi dịch vụ khi đang tìm thì đếm lại với thời gian mới', (
      tester,
    ) async {
      await tester.pumpWidget(_screen());
      await tester.tap(find.widgetWithText(FilledButton, 'Tìm tài xế'));
      await tester.pump();
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.tap(find.text('Giao đồ ăn'));
      await tester.pump();
      expect(_remaining(45), findsOneWidget);

      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(44), findsOneWidget);
    });

    testWidgets('lựa chọn dịch vụ sống qua lúc màn hình dựng lại', (
      tester,
    ) async {
      await tester.pumpWidget(_screen());
      await tester.tap(find.text('Giao bưu kiện'));
      await tester.pump();

      await tester.tap(find.widgetWithText(FilledButton, 'Tìm tài xế'));
      await tester.pump();

      // Màn hình đã dựng lại hai lần; ServicePicker giữ nguyên State.
      expect(_remaining(60), findsOneWidget);
      expect(find.text('Dịch vụ: Giao bưu kiện'), findsOneWidget);
    });

    testWidgets('hết giờ thì quay về nút tìm kèm thông báo', (tester) async {
      await tester.pumpWidget(_screen());
      await tester.tap(find.widgetWithText(FilledButton, 'Tìm tài xế'));
      await tester.pump();

      await tester.pump(const Duration(seconds: 30));

      expect(find.text('Chưa tìm được tài xế, hãy thử lại'), findsOneWidget);
      expect(find.text('Huỷ tìm'), findsNothing);
      expect(find.widgetWithText(FilledButton, 'Tìm tài xế'), findsOneWidget);
    });

    testWidgets('huỷ tìm: chờ cancelDelay rồi quay về nút tìm', (tester) async {
      await tester.pumpWidget(
        _screen(cancelDelay: const Duration(milliseconds: 400)),
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Tìm tài xế'));
      await tester.pump();

      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump(const Duration(milliseconds: 399));
      expect(find.text('Bạn đã huỷ tìm tài xế'), findsNothing);

      await tester.pump(const Duration(milliseconds: 1));
      await tester.pump();
      expect(find.text('Bạn đã huỷ tìm tài xế'), findsOneWidget);
      expect(find.text('Huỷ tìm'), findsNothing);
    });

    testWidgets('thoát màn hình khi đang huỷ: không setState sau dispose', (
      tester,
    ) async {
      await tester.pumpWidget(
        _screen(cancelDelay: const Duration(milliseconds: 400)),
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Tìm tài xế'));
      await tester.pump();
      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump(const Duration(milliseconds: 100));

      await tester.pumpWidget(const MaterialApp(home: SizedBox()));
      await tester.pump(const Duration(seconds: 1));

      expect(tester.takeException(), isNull);
    });
  });
}

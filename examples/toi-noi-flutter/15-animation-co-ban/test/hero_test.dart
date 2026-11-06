import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_15/driver.dart';
import 'package:toi_noi_15/driver_card.dart';
import 'package:toi_noi_15/driver_detail_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _status = TripStatus.accepted;
const _box = Key('driver-card-box');

/// Màn hình có một thẻ cho mỗi tài xế trong [drivers]; bấm thẻ nào thì mở màn
/// chi tiết của tài xế đó.
Widget _list(List<Driver> drivers, {bool reduceMotion = false}) => harness(
  Builder(
    builder: (context) => Scaffold(
      body: Column(
        children: [
          for (final driver in drivers)
            DriverCard(
              driver: driver,
              status: _status,
              onTap: () => Navigator.of(context).push(
                driverDetailRoute(context, driver: driver, status: _status),
              ),
            ),
        ],
      ),
    ),
  ),
  reduceMotion: reduceMotion,
);

Rect _boxRect(WidgetTester tester) => tester.getRect(find.byKey(_box));

void main() {
  final a = kDrivers[0];
  final b = kDrivers[1];

  group('Hero tag', () {
    test('mỗi tài xế một tag, không tag nào trùng', () {
      final tags = {for (final d in kDrivers) driverHeroTag(d)};
      expect(tags, hasLength(kDrivers.length));
    });

    testWidgets('hai thẻ của hai tài xế khác nhau: mở chi tiết không lỗi', (
      tester,
    ) async {
      await tester.pumpWidget(_list([a, b]));
      await tester.tap(find.text(a.name));
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
      expect(find.text('Tài xế của bạn'), findsOneWidget);
    });

    testWidgets('hai thẻ cùng một tài xế (trùng tag): framework ném lỗi khi '
        'chuyển trang', (tester) async {
      await tester.pumpWidget(_list([a, a]));
      await tester.tap(find.text(a.name).first);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(
        (error! as FlutterError).toString(),
        contains('multiple heroes that share the same tag'),
      );
    });

    testWidgets('trùng tag vẫn bị bắt khi bật giảm chuyển động: kiểm tag '
        'không phụ thuộc việc có bay hay không', (tester) async {
      await tester.pumpWidget(_list([a, a], reduceMotion: true));
      await tester.tap(find.text(a.name).first);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      expect(
        (tester.takeException()! as FlutterError).toString(),
        contains('multiple heroes that share the same tag'),
      );
    });
  });

  group('Hero chuyến bay', () {
    testWidgets('giữa chừng thẻ nằm giữa vị trí đầu và cuối; xong thì về đúng '
        'chỗ ở màn chi tiết', (tester) async {
      await tester.pumpWidget(_list([a]));
      final start = _boxRect(tester);

      await tester.tap(find.byType(DriverCard));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 150));

      // Một bản duy nhất: bản trong overlay của Hero; hai thẻ ở hai màn hình
      // đang được thay bằng chỗ giữ chỗ.
      expect(find.byKey(_box), findsOneWidget);
      final mid = _boxRect(tester);

      await tester.pumpAndSettle();
      final end = _boxRect(tester);

      expect(end.top, isNot(start.top));
      expect(mid.top, inExclusiveRange(start.top, end.top));
      expect(find.text('Tài xế của bạn'), findsOneWidget);
    });

    testWidgets('bật giảm chuyển động: thẻ ở đúng vị trí cuối ngay sau khung '
        'đầu, không bay', (tester) async {
      // Lần 1, có animation: đo vị trí cuối và vị trí ngay sau khung đầu.
      await tester.pumpWidget(_list([a]));
      await tester.tap(find.byType(DriverCard));
      await tester.pump();
      final firstFrame = _boxRect(tester);
      await tester.pumpAndSettle();
      final end = _boxRect(tester);
      expect(
        firstFrame,
        isNot(end),
        reason: 'có animation thì khung đầu chưa tới',
      );

      // Dựng cây mới hoàn toàn (Navigator mới) rồi chạy lại với giảm chuyển động.
      await tester.pumpWidget(const SizedBox());
      await tester.pumpWidget(_list([a], reduceMotion: true));
      await tester.tap(find.byType(DriverCard));
      await tester.pump();

      expect(find.text('Tài xế của bạn'), findsOneWidget);
      expect(_boxRect(tester), end);
    });

    testWidgets('quay lại: thẻ bay về chỗ cũ và màn danh sách còn nguyên', (
      tester,
    ) async {
      await tester.pumpWidget(_list([a]));
      final start = _boxRect(tester);
      await tester.tap(find.byType(DriverCard));
      await tester.pumpAndSettle();

      Navigator.of(tester.element(find.text('Tài xế của bạn'))).pop();
      await tester.pumpAndSettle();

      expect(find.text('Tài xế của bạn'), findsNothing);
      expect(_boxRect(tester), start);
    });
  });
}

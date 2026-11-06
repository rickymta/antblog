@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/golden_harness.dart';
import 'support/sample_cards.dart';
import 'support/tolerant_comparator.dart';

/// Chứng minh comparator bắt được lỗi giao diện thật.
///
/// Mỗi test dùng một thư mục tạm làm nơi đặt ảnh chuẩn và nơi ghi `failures/`,
/// nên ảnh chuẩn đã commit không bị đụng và repo không có file lạ.
void main() {
  late Directory dir;
  late GoldenFileComparator previous;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('toi_noi_29_');
    File('test/goldens/trip_card_light.png')
        .copySync('${dir.path}/baseline.png');
    previous = goldenFileComparator;
    goldenFileComparator = TolerantGoldenComparator(
      Uri.file('${dir.path}/probe_test.dart'),
    );
  });

  tearDown(() {
    goldenFileComparator = previous;
    dir.deleteSync(recursive: true);
  });

  group('so với ảnh chuẩn trip_card_light.png', () {
    // `matchesGoldenFile` chạy comparator trong `tester.runAsync`, nơi lỗi bị
    // bắt và báo bằng `FlutterError.reportError`; `expectLater` vẫn hoàn tất
    // bình thường nên `throwsA` không thấy gì. Lỗi nằm chờ ở binding và lấy
    // được bằng `tester.takeException()` (test cuối nhóm dùng cách đó). Ở
    // các ca trượt khác ta gọi thẳng comparator trên ảnh vừa chụp và tự bắt lỗi.
    Future<Object?> compareToBaseline(WidgetTester tester) async {
      final png = await capturePng(tester);
      return tester.runAsync<Object?>(() async {
        try {
          await goldenFileComparator.compare(png, Uri.parse('baseline.png'));
          return null;
        } on FlutterError catch (error) {
          return error;
        }
      });
    }

    testWidgets('dựng lại đúng thẻ cũ thì qua', (tester) async {
      await pumpGolden(tester, sampleCardColumn(), viewHeight: 1200);

      await expectLater(
        find.byKey(goldenKey),
        matchesGoldenFile('baseline.png'),
      );
    });

    testWidgets('đổi trạng thái thẻ đầu sang "Tài xế đã nhận" thì trượt', (
      tester,
    ) async {
      await pumpGolden(
        tester,
        sampleCardColumn(first: TripStatus.accepted),
        viewHeight: 1200,
      );

      final error = await compareToBaseline(tester);

      expect(error, isA<FlutterError>());
      expect((error! as FlutterError).message, contains('Pixel test failed'));
      expect(
        File('${dir.path}/failures/baseline_maskedDiff.png').existsSync(),
        isTrue,
      );
    });

    testWidgets('matchesGoldenFile trượt: lỗi nằm ở takeException', (
      tester,
    ) async {
      await pumpGolden(
        tester,
        sampleCardColumn(first: TripStatus.accepted),
        viewHeight: 1200,
      );

      await expectLater(
        find.byKey(goldenKey),
        matchesGoldenFile('baseline.png'),
      );

      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect((error! as FlutterError).message, contains('Pixel test failed'));
    });

    testWidgets('mất bốn nút ẩn chuyến thì trượt', (tester) async {
      await pumpGolden(
        tester,
        sampleCardColumn(dismissible: false),
        viewHeight: 1200,
      );

      expect(await compareToBaseline(tester), isA<FlutterError>());
    });
  }, skip: goldenCompareSkip);

  group('ngưỡng không được rộng hơn thay đổi nhỏ nhất ta muốn bắt', () {
    testWidgets('bỏ một icon X lệch khoảng 0,45% ảnh', (tester) async {
      await pumpGolden(tester, oneCard());
      final withIcon = await capturePng(tester);
      await pumpGolden(tester, oneCard(dismissible: false));
      final withoutIcon = await capturePng(tester);

      final diff = await tester.runAsync(
        () => measurePixelDiff(withIcon, withoutIcon, channelTolerance: 4),
      );

      expect(diff!.fraction, greaterThan(0.001));
      expect(diff.fraction, lessThan(0.005));
    });

    testWidgets('ngưỡng 0,5% nuốt mất lỗi đó, ngưỡng 0,1% của lab thì bắt', (
      tester,
    ) async {
      await pumpGolden(tester, oneCard());
      final withIcon = await capturePng(tester);
      await pumpGolden(tester, oneCard(dismissible: false));
      final withoutIcon = await capturePng(tester);
      final golden = Uri.parse('icon.png');
      final probe = Uri.file('${dir.path}/probe_test.dart');
      final loose = TolerantGoldenComparator(probe, pixelTolerance: 0.005);
      final lab = TolerantGoldenComparator(probe);

      final outcomes = await tester.runAsync(() async {
        await loose.update(golden, withIcon);
        final loosePassed = await loose.compare(withoutIcon, golden);
        Object? labError;
        try {
          await lab.compare(withoutIcon, golden);
        } on FlutterError catch (error) {
          labError = error;
        }
        return (loosePassed: loosePassed, labError: labError);
      });

      expect(outcomes!.loosePassed, isTrue);
      expect(outcomes.labError, isA<FlutterError>());
    });
  }, skip: goldenCompareSkip);
}

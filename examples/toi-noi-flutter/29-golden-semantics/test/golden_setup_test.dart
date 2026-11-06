import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/golden_harness.dart';
import 'support/sample_cards.dart';

/// Bằng chứng cho các giả định mà golden test dựa vào. Không so với ảnh
/// chuẩn nào, nên chạy ở mọi hệ điều hành.
void main() {
  group('cửa sổ test', () {
    testWidgets(
      'mặc định là 800 x 600 pixel logic, ta đặt lại thành 390 x 844',
      (tester) async {
        Size logical() =>
            tester.view.physicalSize / tester.view.devicePixelRatio;
        expect(logical(), const Size(800, 600));

        setGoldenView(tester);

        expect(logical(), const Size(390, 844));
        expect(tester.view.devicePixelRatio, 1);
      },
    );

    testWidgets('khôi phục sau test: test kế tiếp lại thấy 800 x 600', (
      tester,
    ) async {
      expect(
        tester.view.physicalSize / tester.view.devicePixelRatio,
        const Size(800, 600),
      );
    });

    testWidgets('ảnh chụp có đúng bề rộng cố định', (tester) async {
      await pumpGolden(tester, oneCard());

      final png = await capturePng(tester);
      final image = await tester.runAsync(() async {
        final codec = await ui.instantiateImageCodec(png);
        return (await codec.getNextFrame()).image;
      });
      expect(image!.width, goldenWidth.toInt());
      expect(
        image.height,
        tester.getSize(find.byKey(goldenKey)).height.toInt(),
      );
      image.dispose();
    });
  });

  testWidgets('testWidgets đã tắt bóng đổ sẵn', (tester) async {
    // Bóng mờ không đảm bảo vẽ giống nhau giữa các phiên bản, thậm chí giữa
    // các lần chạy, nên binding kiểm thử bật
    // cờ này cho mọi testWidgets.
    expect(debugDisableShadows, isTrue);
  });

  group('font kiểm thử', () {
    testWidgets(
      'font kiểm thử mặc định: mỗi ký tự của "Tới Nơi" rộng một cỡ chữ',
      (tester) async {
        await pumpGolden(
          tester,
          const Align(
            alignment: Alignment.topLeft,
            child: Text(
              'Tới Nơi',
              style: TextStyle(fontSize: 16, letterSpacing: 0),
            ),
          ),
        );

        expect(tester.getSize(find.text('Tới Nơi')).width, 7 * 16);
      },
    );

    testWidgets(
      'đổi chữ ASCII sang chữ ASCII cùng độ dài thì golden không đổi',
      (tester) async {
        await pumpGolden(
          tester,
          oneCard(pickup: 'Chung cư Hoa Sen, 12 Đường Mẫu'),
        );
        final before = await capturePng(tester);

        await pumpGolden(
          tester,
          oneCard(pickup: 'Chung cư Hoa Sun, 99 Đường Mẫu'),
        );
        final after = await capturePng(tester);

        expect(listEquals(before, after), isTrue);
        // Nhưng semantics thấy ngay.
        expect(find.bySemanticsLabel(RegExp('Hoa Sun, 99')), findsOneWidget);
      },
    );

    testWidgets('thêm dấu tiếng Việt (Sen thành Sến) thì golden đổi', (
      tester,
    ) async {
      await pumpGolden(
        tester,
        oneCard(pickup: 'Chung cư Hoa Sen, 12 Đường Mẫu'),
      );
      final before = await capturePng(tester);

      await pumpGolden(
        tester,
        oneCard(pickup: 'Chung cư Hoa Sến, 12 Đường Mẫu'),
      );
      final after = await capturePng(tester);

      expect(listEquals(before, after), isFalse);
    });

    testWidgets('đổi độ dài chữ thì golden đổi', (tester) async {
      await pumpGolden(
        tester,
        oneCard(pickup: 'Chung cư Hoa Sen, 12 Đường Mẫu'),
      );
      final before = await capturePng(tester);

      await pumpGolden(tester, oneCard(pickup: 'Chung cư Hoa Sen, 12'));
      final after = await capturePng(tester);

      expect(listEquals(before, after), isFalse);
    });

    testWidgets('đã nạp font icon: hai icon khác nhau cho hai ảnh khác nhau', (
      tester,
    ) async {
      await pumpGolden(tester, const Icon(Icons.check_circle, size: 48));
      final check = await capturePng(tester);

      await pumpGolden(tester, const Icon(Icons.cancel, size: 48));
      final cancel = await capturePng(tester);

      expect(listEquals(check, cancel), isFalse);
    });
  });
}

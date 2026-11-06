import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show BoxConstraints, RenderParagraph;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote_repository.dart';

import 'support.dart';

void main() {
  group('kích thước màn hình và font', () {
    testWidgets('mặc định 800x600 logical = 2400x1800 pixel, tỷ lệ 3', (
      tester,
    ) async {
      expect(tester.view.physicalSize, const Size(2400, 1800));
      expect(tester.view.devicePixelRatio, 3);

      late Size logical;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              logical = MediaQuery.sizeOf(context);
              return const SizedBox();
            },
          ),
        ),
      );

      expect(logical, const Size(800, 600));
    });

    testWidgets(
      'chữ trong test là font FlutterTest: mỗi ký tự rộng đúng cỡ chữ',
      (tester) async {
        await tester.pumpWidget(
          const MaterialApp(
            home: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('ABCDE', style: TextStyle(fontSize: 20)),
                Text('Đặt xe', style: TextStyle(fontSize: 10)),
              ],
            ),
          ),
        );

        // 5 ký tự x 20 px. Font thật (Roboto, SF) hẹp hơn nhiều, nên chỗ nào
        // vừa trong test chưa chắc vừa ngoài máy thật và ngược lại.
        expect(tester.getSize(find.text('ABCDE')), const Size(100, 20));
        // Ascent 0,75 em: baseline ở 15 px với cỡ 20 (font Ahem là 16).
        final paragraph = tester.renderObject<RenderParagraph>(
          find.text('ABCDE'),
        );
        expect(
          paragraph.getDryBaseline(
            BoxConstraints.loose(const Size(800, 600)),
            TextBaseline.alphabetic,
          ),
          15,
        );
        // Chữ tiếng Việt có dấu cũng rộng 1 em mỗi ký tự.
        expect(tester.getSize(find.text('Đặt xe')).width, 60);
      },
    );

    testWidgets('Row không Expanded tràn trên màn hình hẹp: takeException', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(960, 1440); // 320x480 logical
      tester.view.devicePixelRatio = 3;
      addTearDown(tester.view.reset);

      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: Row(
              children: [Text('Giá mở cửa (2 km đầu)'), Text('12.000 VND')],
            ),
          ),
        ),
      );

      // 21 + 10 ký tự x 14 px = 434 px > 320 px.
      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(error.toString(), contains('overflowed'));
    });

    for (final size in const [Size(320, 480), Size(360, 640), Size(800, 600)]) {
      for (final scale in const [1.0, 1.5]) {
        testWidgets('màn hình báo giá không tràn ở $size, chữ x$scale', (
          tester,
        ) async {
          tester.view.physicalSize = size * 3;
          tester.view.devicePixelRatio = 3;
          tester.platformDispatcher.textScaleFactorTestValue = scale;
          addTearDown(tester.view.reset);
          addTearDown(tester.platformDispatcher.clearAllTestValues);

          await pumpQuoteScreen(
            tester,
            overrides: [
              quoteRepositoryProvider.overrideWithValue(FakeQuoteRepository()),
            ],
          );
          await tester.pumpAndSettle();

          expect(find.byKey(const Key('quote-fare')), findsOneWidget);
          expect(tester.takeException(), isNull);
        });
      }
    }
  });

  testWidgets('kích thước đặt trong test còn nguyên tới lúc teardown', (
    tester,
  ) async {
    // Khi teardown chạy, test đã xong mà kích thước vẫn là của ta: framework
    // không tự trả lại, nên phải `addTearDown(tester.view.reset)`.
    addTearDown(() {
      expect(tester.view.physicalSize, const Size(1080, 1920));
      tester.view.reset();
      expect(tester.view.physicalSize, const Size(2400, 1800));
      expect(tester.view.devicePixelRatio, 3);
    });

    tester.view.physicalSize = const Size(1080, 1920);
    tester.view.devicePixelRatio = 3;
    await tester.pump();

    expect(tester.view.physicalSize, const Size(1080, 1920));
  });
}

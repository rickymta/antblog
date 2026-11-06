import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

const logo = 'assets/images/logo.png';

Future<ui.Image> decode(String key) async {
  final data = await rootBundle.load(key);
  final codec = await ui.instantiateImageCodec(data.buffer.asUint8List());
  return (await codec.getNextFrame()).image;
}

void main() {
  group('asset được khai báo trong pubspec', () {
    testWidgets('chỉ liệt kê bản 1.0x nhưng manifest có đủ ba bản', (
      tester,
    ) async {
      final manifest = await AssetManifest.loadFromAssetBundle(rootBundle);

      expect(manifest.listAssets(), [logo]);
      final variants = manifest.getAssetVariants(logo)!;
      expect(
        {for (final v in variants) v.key: v.targetDevicePixelRatio},
        {
          'assets/images/logo.png': null,
          'assets/images/2.0x/logo.png': 2.0,
          'assets/images/3.0x/logo.png': 3.0,
        },
      );
      expect(variants.where((v) => v.main).single.key, logo);
    });

    testWidgets('tên không khai báo thì manifest trả null', (tester) async {
      final manifest = await AssetManifest.loadFromAssetBundle(rootBundle);
      expect(manifest.getAssetVariants('assets/images/nope.png'), isNull);
    });

    test('pubspec.yaml có khoá assets và generate: true', () {
      final pubspec = File('pubspec.yaml').readAsStringSync();
      expect(pubspec, contains('generate: true'));
      expect(pubspec, contains('- assets/images/logo.png'));
    });
  });

  group('AssetImage chọn bản theo devicePixelRatio', () {
    // Quy tắc nằm trong AssetImage: trùng khớp thì lấy đúng bản; dưới 2.0 lấy
    // bản cao hơn gần nhất; từ 2.0 trở lên lấy bản gần nhất, hoà thì lấy bản thấp.
    Future<AssetBundleImageKey> keyFor(double dpr) {
      return const AssetImage(logo)
          .obtainKey(ImageConfiguration(devicePixelRatio: dpr));
    }

    for (final (dpr, name, scale) in [
      (1.0, 'assets/images/logo.png', 1.0),
      (1.5, 'assets/images/2.0x/logo.png', 2.0),
      (2.0, 'assets/images/2.0x/logo.png', 2.0),
      (2.5, 'assets/images/2.0x/logo.png', 2.0),
      (2.6, 'assets/images/3.0x/logo.png', 3.0),
      (3.0, 'assets/images/3.0x/logo.png', 3.0),
      (4.0, 'assets/images/3.0x/logo.png', 3.0),
    ]) {
      testWidgets('dpr $dpr chọn $name', (tester) async {
        final key = await keyFor(dpr);
        expect(key.name, name);
        expect(key.scale, scale);
      });
    }
  });

  group('file ảnh đọc và giải mã được', () {
    for (final (key, side) in [
      ('assets/images/logo.png', 48),
      ('assets/images/2.0x/logo.png', 96),
      ('assets/images/3.0x/logo.png', 144),
    ]) {
      testWidgets('$key là PNG ${side}x$side', (tester) async {
        final image = (await tester.runAsync(() => decode(key)))!;
        expect(image.width, side);
        expect(image.height, side);
      });
    }
  });

  group('Image.asset trong cây widget', () {
    Future<RenderImage> pumpLogo(WidgetTester tester, double dpr) async {
      tester.view.devicePixelRatio = dpr;
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        const MaterialApp(
          // Center nới ràng buộc chặt của màn hình để ảnh giữ được 48 dp.
          home: Center(
            child: Image(image: AssetImage(logo), width: 48, height: 48),
          ),
        ),
      );
      await tester.runAsync(() async {
        final context = tester.element(find.byType(Image));
        await precacheImage(const AssetImage(logo), context);
      });
      await tester.pump();
      return tester.renderObject<RenderImage>(find.byType(RawImage));
    }

    testWidgets('màn 3.0 dựng ảnh 144 px vào ô 48 dp, scale 3', (tester) async {
      final render = await pumpLogo(tester, 3);
      expect(render.image!.width, 144);
      expect(render.scale, 3.0);
      expect(render.size, const Size(48, 48));
    });

    testWidgets('màn 1.0 dựng ảnh 48 px, scale 1', (tester) async {
      final render = await pumpLogo(tester, 1);
      expect(render.image!.width, 48);
      expect(render.scale, 1.0);
    });
  });

  group('asset không tồn tại', () {
    testWidgets('rootBundle.load ném FlutterError', (tester) async {
      Object? error;
      await tester.runAsync(() async {
        try {
          await rootBundle.load('assets/images/nope.png');
        } on FlutterError catch (e) {
          error = e;
        }
      });
      expect(error, isA<FlutterError>());
      expect(error.toString(), contains('Unable to load asset'));
    });

    testWidgets('Image.asset dựng errorBuilder thay vì làm sập màn hình', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Image.asset(
            'assets/images/nope.png',
            errorBuilder: (context, error, stackTrace) =>
                const Text('không có ảnh'),
          ),
        ),
      );
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      await tester.pump();
      expect(find.text('không có ảnh'), findsOneWidget);
    });
  });
}

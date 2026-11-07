import 'dart:async';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/avatar.dart';

import 'support/png.dart';

void main() {
  // Ảnh gốc 512 x 512: giải mã đầy đủ tốn 512 * 512 * 4 byte.
  const sourceSide = 512;
  const fullBytes = sourceSide * sourceSide * 4;
  late Uint8List png;

  setUpAll(() => png = makePng(sourceSide, sourceSide));
  setUp(() {
    PaintingBinding.instance.imageCache.clear();
    PaintingBinding.instance.imageCache.clearLiveImages();
  });

  /// Giải mã [provider] thật và trả về ảnh đã giải mã.
  Future<ui.Image> decode(WidgetTester tester, ImageProvider provider) async {
    final image = await tester.runAsync(() {
      final completer = Completer<ui.Image>();
      final stream = provider.resolve(ImageConfiguration.empty);
      late ImageStreamListener listener;
      listener = ImageStreamListener(
        (info, _) {
          completer.complete(info.image);
          stream.removeListener(listener);
        },
        onError: (error, stack) {
          completer.completeError(error, stack);
          stream.removeListener(listener);
        },
      );
      stream.addListener(listener);
      return completer.future;
    });
    return image!;
  }

  test('decodeSizeFor đổi pixel logic sang pixel vật lý, làm tròn lên', () {
    expect(decodeSizeFor(48, 3), 144);
    expect(decodeSizeFor(48, 2.625), 126);
    expect(decodeSizeFor(48, 1), 48);
  });

  testWidgets('MemoryImage giải mã ở kích thước gốc', (tester) async {
    final image = await decode(tester, MemoryImage(png));

    expect(image.width, sourceSide);
    expect(image.height, sourceSide);
    expect(PaintingBinding.instance.imageCache.currentSizeBytes, fullBytes);
  });

  testWidgets('avatar 48 dp ở dpr 3 chỉ giải mã 144 px, giữ tỉ lệ', (
    tester,
  ) async {
    final provider = avatarProvider(
      MemoryImage(png),
      logicalSize: 48,
      devicePixelRatio: 3,
    );
    final image = await decode(tester, provider);

    expect(image.width, 144);
    expect(image.height, 144);
    expect(PaintingBinding.instance.imageCache.currentSizeBytes, 144 * 144 * 4);
    // Giảm gần 12,6 lần bộ nhớ ảnh giải mã so với giải mã đầy đủ.
    expect(fullBytes / (144 * 144 * 4), closeTo(12.64, 0.01));
  });

  testWidgets('ảnh chữ nhật: chỉ đặt width thì chiều cao theo tỉ lệ', (
    tester,
  ) async {
    final wide = makePng(400, 200);

    final image = await decode(
      tester,
      ResizeImage(MemoryImage(wide), width: 100),
    );

    expect(image.width, 100);
    expect(image.height, 50);
  });

  testWidgets('đòi giải mã lớn hơn ảnh gốc thì bị chặn ở kích thước gốc', (
    tester,
  ) async {
    final image = await decode(
      tester,
      ResizeImage(MemoryImage(png), width: 1024),
    );

    expect(image.width, sourceSide);
  });

  testWidgets('allowUpscaling: true cho phép giải mã lớn hơn ảnh gốc', (
    tester,
  ) async {
    final image = await decode(
      tester,
      ResizeImage(MemoryImage(png), width: 1024, allowUpscaling: true),
    );

    expect(image.width, 1024);
  });

  testWidgets('cùng một ảnh ở hai kích thước là hai mục trong ImageCache', (
    tester,
  ) async {
    await decode(tester, MemoryImage(png));
    await decode(tester, ResizeImage(MemoryImage(png), width: 144));

    final cache = PaintingBinding.instance.imageCache;
    expect(cache.currentSize, 2);
    expect(cache.currentSizeBytes, fullBytes + 144 * 144 * 4);
  });

  testWidgets('Image.memory với cacheWidth bọc nguồn trong ResizeImage', (
    tester,
  ) async {
    final widget = Image.memory(png, cacheWidth: 100);
    final plain = Image.memory(png);

    expect(widget.image, isA<ResizeImage>());
    expect((widget.image as ResizeImage).width, 100);
    expect(plain.image, isA<MemoryImage>());
  });

  testWidgets('DriverAvatar lấy dpr từ MediaQuery', (tester) async {
    await tester.pumpWidget(
      MediaQuery(
        data: const MediaQueryData(devicePixelRatio: 3),
        child: Directionality(
          textDirection: TextDirection.ltr,
          child: DriverAvatar(source: MemoryImage(png)),
        ),
      ),
    );

    final provider = tester.widget<Image>(find.byType(Image)).image;
    expect(provider, isA<ResizeImage>());
    expect((provider as ResizeImage).width, 144);
  });
}

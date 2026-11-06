import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/tolerant_comparator.dart';

/// Ảnh 100 x 100 một màu [base], thêm [dots] pixel đổi sang [dotColor].
///
/// Chỉ vẽ hình chữ nhật nguyên pixel, không khử răng cưa, nên kết quả giống
/// hệt trên mọi hệ điều hành: test comparator không phụ thuộc máy chạy.
Future<Uint8List> paintPng({
  Color base = const Color(0xFF808080),
  int dots = 0,
  Color dotColor = const Color(0xFFFFFFFF),
  int size = 100,
}) async {
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  final paint = Paint()
    ..isAntiAlias = false
    ..color = base;
  canvas.drawRect(Rect.fromLTWH(0, 0, size.toDouble(), size.toDouble()), paint);
  paint.color = dotColor;
  for (var i = 0; i < dots; i++) {
    canvas.drawRect(Rect.fromLTWH(i.toDouble(), 0, 1, 1), paint);
  }
  final image = await recorder.endRecording().toImage(size, size);
  try {
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    return data!.buffer.asUint8List();
  } finally {
    image.dispose();
  }
}

void main() {
  late Directory dir;
  late TolerantGoldenComparator comparator;
  final golden = Uri.parse('baseline.png');

  setUp(() async {
    dir = Directory.systemTemp.createTempSync('toi_noi_29_');
    comparator = TolerantGoldenComparator(
      Uri.file('${dir.path}/probe_test.dart'),
    );
    // Ngưỡng mặc định: lệch kênh <= 4, tối đa 0,1% pixel (10 trên 10.000).
    await comparator.update(golden, await paintPng());
  });

  tearDown(() => dir.deleteSync(recursive: true));

  test('ngưỡng mặc định của lab', () {
    expect(comparator.channelTolerance, 4);
    expect(comparator.pixelTolerance, 0.001);
  });

  test('ảnh giống hệt thì qua', () async {
    expect(await comparator.compare(await paintPng(), golden), isTrue);
  });

  test('nhiễu nhỏ trên mọi pixel (lệch 3 mức) vẫn qua', () async {
    final noisy = await paintPng(base: const Color(0xFF838383));

    // Comparator mặc định của Flutter thấy 100% pixel khác nhau...
    final strict = await GoldenFileComparator.compareLists(
      noisy,
      await File('${dir.path}/baseline.png').readAsBytes(),
    );
    expect(strict.passed, isFalse);
    expect(strict.diffPercent, 1.0);
    strict.dispose();

    // ...còn comparator của lab coi đó là nhiễu.
    expect(await comparator.compare(noisy, golden), isTrue);
  });

  test('lệch 10 mức trên mọi pixel thì trượt', () async {
    final shifted = await paintPng(base: const Color(0xFF8A8A8A));

    await expectLater(
      comparator.compare(shifted, golden),
      throwsA(isA<FlutterError>()),
    );
  });

  test('đổi hẳn 10 pixel (0,1%) qua, 11 pixel trượt', () async {
    expect(await comparator.compare(await paintPng(dots: 10), golden), isTrue);

    await expectLater(
      comparator.compare(await paintPng(dots: 11), golden),
      throwsA(isA<FlutterError>()),
    );
  });

  test('trượt thì báo số pixel và ghi bốn ảnh vào failures/', () async {
    Object? error;
    try {
      await comparator.compare(await paintPng(dots: 50), golden);
    } on FlutterError catch (e) {
      error = e;
    }

    final message = (error! as FlutterError).message;
    expect(message, contains('Pixel test failed'));
    expect(message, contains('Ngưỡng của lab'));
    expect(message, contains('50/10000'));
    final names = Directory(
      '${dir.path}/failures',
    ).listSync().map((e) => e.uri.pathSegments.last).toList()..sort();
    expect(names, [
      'baseline_isolatedDiff.png',
      'baseline_maskedDiff.png',
      'baseline_masterImage.png',
      'baseline_testImage.png',
    ]);
  });

  test('ảnh khác kích thước luôn trượt, ngưỡng nào cũng vậy', () async {
    final loose = TolerantGoldenComparator(
      Uri.file('${dir.path}/probe_test.dart'),
      channelTolerance: 255,
      pixelTolerance: 1,
    );

    await expectLater(
      loose.compare(await paintPng(size: 101), golden),
      throwsA(
        isA<FlutterError>().having(
          (e) => e.message,
          'message',
          contains('khác kích thước'),
        ),
      ),
    );
  });

  test(
    'thiếu ảnh chuẩn thì báo TestFailure, không phải qua mặc định',
    () async {
      await expectLater(
        comparator.compare(await paintPng(), Uri.parse('chua-co.png')),
        throwsA(isA<TestFailure>()),
      );
    },
  );
}

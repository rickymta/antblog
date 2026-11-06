import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';

/// Kết quả đo khác biệt giữa hai ảnh cùng kích thước.
typedef PixelDiff = ({int differing, int total, double fraction});

/// Đếm số pixel lệch hơn [channelTolerance] ở ít nhất một kênh R, G, B hoặc A.
///
/// Trả `null` khi hai ảnh khác kích thước: không có ngưỡng nào cứu được ca đó.
/// Hàm cần codec ảnh thật nên trong `testWidgets` phải gọi qua
/// `tester.runAsync`.
Future<PixelDiff?> measurePixelDiff(
  List<int> a,
  List<int> b, {
  int channelTolerance = 0,
}) async {
  final imageA = await _decode(a);
  final imageB = await _decode(b);
  try {
    if (imageA.width != imageB.width || imageA.height != imageB.height) {
      return null;
    }
    final rgbaA = (await imageA.toByteData())!.buffer.asUint8List();
    final rgbaB = (await imageB.toByteData())!.buffer.asUint8List();
    var differing = 0;
    for (var i = 0; i < rgbaA.length; i += 4) {
      for (var channel = 0; channel < 4; channel++) {
        if ((rgbaA[i + channel] - rgbaB[i + channel]).abs() >
            channelTolerance) {
          differing++;
          break;
        }
      }
    }
    final total = imageA.width * imageA.height;
    return (differing: differing, total: total, fraction: differing / total);
  } finally {
    imageA.dispose();
    imageB.dispose();
  }
}

Future<ui.Image> _decode(List<int> png) async {
  final codec = await ui.instantiateImageCodec(Uint8List.fromList(png));
  try {
    return (await codec.getNextFrame()).image;
  } finally {
    codec.dispose();
  }
}

/// `LocalFileComparator` có hai ngưỡng.
///
/// Pixel chỉ tính là "lệch" khi một kênh chênh hơn [channelTolerance] mức
/// (trên thang 0 đến 255), và ảnh chỉ trượt khi tỉ lệ pixel lệch vượt
/// [pixelTolerance]. Ảnh khác kích thước luôn trượt. Khi trượt, ảnh diff
/// vẫn được ghi vào `failures/` như comparator mặc định.
class TolerantGoldenComparator extends LocalFileComparator {
  TolerantGoldenComparator(
    super.testFile, {
    this.channelTolerance = 4,
    this.pixelTolerance = 0.001,
  }) : assert(channelTolerance >= 0 && channelTolerance <= 255),
       assert(pixelTolerance >= 0 && pixelTolerance <= 1);

  /// Chênh lệch tối đa mỗi kênh mà ta coi là nhiễu, thang 0 đến 255.
  final int channelTolerance;

  /// Tỉ lệ pixel lệch tối đa, 0 đến 1 (0,001 là 0,1%).
  final double pixelTolerance;

  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    final master = await getGoldenBytes(golden);
    final result = await GoldenFileComparator.compareLists(imageBytes, master);
    if (result.passed) {
      result.dispose();
      return true;
    }

    final diff = await measurePixelDiff(
      imageBytes,
      master,
      channelTolerance: channelTolerance,
    );
    if (diff != null && diff.fraction <= pixelTolerance) {
      debugPrint(
        'Golden "$golden": lệch ${diff.differing}/${diff.total} pixel '
        '(<= $pixelTolerance), chấp nhận.',
      );
      result.dispose();
      return true;
    }

    final error = await generateFailureOutput(result, golden, basedir);
    result.dispose();
    final detail = diff == null
        ? 'khác kích thước'
        : 'sau khi bỏ lệch kênh <= $channelTolerance còn '
              '${diff.differing}/${diff.total} pixel '
              '(${(diff.fraction * 100).toStringAsFixed(3)}%), '
              'ngưỡng ${(pixelTolerance * 100).toStringAsFixed(3)}%';
    throw FlutterError('$error\nNgưỡng của lab: $detail');
  }
}

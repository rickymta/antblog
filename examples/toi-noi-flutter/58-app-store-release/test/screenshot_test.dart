import 'dart:io';
import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

import 'support.dart';

PngInfo shot(int w, int h, {int colorType = 2, bool trns = false}) =>
    PngInfo.parse(png(w, h, colorType: colorType, trns: trns));

Set<String> check(
  List<PngInfo> iphone, {
  List<PngInfo> ipad = const [],
  bool supportsIpad = false,
}) => codes(
  checkScreenshots(iphone: iphone, ipad: ipad, supportsIpad: supportsIpad),
);

void main() {
  group('PngInfo', () {
    test('đọc rộng, cao và alpha theo colour type', () {
      final rgb = shot(1179, 2556);
      expect((rgb.width, rgb.height, rgb.hasAlpha), (1179, 2556, false));
      expect(shot(10, 20, colorType: 0).hasAlpha, isFalse);
      expect(shot(10, 20, colorType: 3).hasAlpha, isFalse);
      expect(shot(10, 20, colorType: 4).hasAlpha, isTrue);
      expect(shot(10, 20, colorType: 6).hasAlpha, isTrue);
    });

    test('chunk tRNS cũng là trong suốt dù colour type không có alpha', () {
      expect(shot(10, 20, colorType: 2, trns: true).hasAlpha, isTrue);
      expect(shot(10, 20, colorType: 3, trns: true).hasAlpha, isTrue);
    });

    test('không phải PNG hoặc bị cụt ném FormatException', () {
      expect(() => PngInfo.parse(Uint8List(0)), throwsFormatException);
      expect(() => PngInfo.parse(Uint8List(64)), throwsFormatException);
      final good = png(1, 1);
      expect(() => PngInfo.parse(good.sublist(0, 20)), throwsFormatException);
    });
  });

  group('ảnh iPhone', () {
    test('một ảnh đúng cỡ bắt buộc là đủ, cả dọc lẫn ngang', () {
      expect(check([shot(1179, 2556)]), isEmpty);
      expect(check([shot(1206, 2622)]), isEmpty);
      expect(check([shot(2556, 1179)]), isEmpty);
    });

    test('không có ảnh nào là lỗi', () {
      expect(check([]), {'screenshot-iphone-required-size'});
    });

    test('chỉ có cỡ khác (kể cả cỡ lớn hợp lệ) thì vẫn thiếu cỡ bắt buộc', () {
      expect(check([shot(1320, 2868)]), {'screenshot-iphone-required-size'});
    });

    test('cỡ lạ chỉ là cảnh báo', () {
      expect(check([shot(1179, 2556), shot(1000, 2000)]), {
        'screenshot-size-unknown',
      });
    });

    test('có alpha là lỗi, kể cả cỡ đúng', () {
      expect(check([shot(1179, 2556, colorType: 6)]), {'screenshot-alpha'});
      expect(check([shot(1179, 2556, trns: true)]), {'screenshot-alpha'});
    });

    test(
      '10 ảnh cùng cỡ được, 11 ảnh là lỗi; ngang và dọc tính chung một cỡ',
      () {
        expect(check([for (var i = 0; i < 10; i++) shot(1179, 2556)]), isEmpty);
        expect(check([for (var i = 0; i < 11; i++) shot(1179, 2556)]), {
          'screenshot-too-many',
        });
        expect(
          check([
            for (var i = 0; i < 6; i++) shot(1179, 2556),
            for (var i = 0; i < 5; i++) shot(2556, 1179),
          ]),
          {'screenshot-too-many'},
        );
      },
    );
  });

  group('ảnh iPad', () {
    test('app chạy được trên iPad thì cần ảnh 13 inch', () {
      final iphone = [shot(1179, 2556)];
      expect(check(iphone, supportsIpad: true), {
        'screenshot-ipad-required-size',
      });
      expect(
        check(iphone, ipad: [shot(2064, 2752)], supportsIpad: true),
        isEmpty,
      );
      expect(
        check(iphone, ipad: [shot(2048, 2732)], supportsIpad: true),
        isEmpty,
      );
    });

    test('ảnh iPad 11 inch không thay được ảnh 13 inch bắt buộc', () {
      expect(
        check([shot(1179, 2556)], ipad: [shot(1668, 2420)], supportsIpad: true),
        {'screenshot-ipad-required-size'},
      );
    });

    test('app chỉ iPhone mà có ảnh iPad là cảnh báo', () {
      expect(check([shot(1179, 2556)], ipad: [shot(2064, 2752)]), {
        'screenshot-ipad-unused',
      });
    });
  });

  group('splitByDevice', () {
    test('cỡ iPad vào nhóm iPad (cả ngang), còn lại vào iPhone', () {
      final split = splitByDevice([
        shot(1179, 2556),
        shot(2064, 2752),
        shot(2732, 2048),
        shot(1000, 2000),
      ]);
      expect([for (final s in split.iphone) s.width], [1179, 1000]);
      expect([for (final s in split.ipad) s.width], [2064, 2732]);
    });
  });

  group('đọc từ thư mục', () {
    test('loadScreenshots đọc các .png theo thứ tự tên và bỏ file khác', () {
      final dir = Directory.systemTemp.createTempSync('toi_noi_58_shots_');
      addTearDown(() => dir.deleteSync(recursive: true));
      File('${dir.path}/b.png').writeAsBytesSync(png(1206, 2622));
      File('${dir.path}/a.png').writeAsBytesSync(png(1179, 2556));
      File('${dir.path}/ghi-chu.txt').writeAsStringSync('không phải ảnh');
      final shots = loadScreenshots(dir);
      expect(
        [for (final s in shots) (s.width, s.height)],
        [(1179, 2556), (1206, 2622)],
      );
    });

    test('thư mục không tồn tại cho danh sách rỗng', () {
      expect(
        loadScreenshots(Directory('store/customer/screenshots/vi')),
        isEmpty,
      );
    });
  });
}

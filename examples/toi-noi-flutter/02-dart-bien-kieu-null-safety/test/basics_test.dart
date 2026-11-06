// Các test dưới đây là "thí nghiệm" kiểm chứng những khẳng định trong bài 02.
import 'package:test/test.dart';

void main() {
  group('phép chia và số nguyên', () {
    test('/ luôn trả double, ~/ chia lấy phần nguyên', () {
      expect(5 / 2, 2.5);
      expect(4 / 2, 2.0);
      expect(5 ~/ 2, 2);
      expect((5 / 2).runtimeType, double);
      expect((5 ~/ 2).runtimeType, int);
    });

    test('~/ cắt về phía 0, % luôn không âm với số chia dương', () {
      expect(-5 ~/ 2, -2);
      expect(-5 % 3, 1);
      expect((-5).remainder(3), -2);
    });
  });

  group('làm tròn', () {
    test('round() làm tròn điểm giữa ra xa số 0', () {
      expect(2.5.round(), 3);
      expect((-2.5).round(), -3);
      expect(2.4.round(), 2);
    });

    test('toStringAsFixed chỉ định dạng, không đổi giá trị', () {
      const km = 2.567;

      expect(km.toStringAsFixed(1), '2.6');
      expect(km, 2.567);
    });
  });

  group('double', () {
    test('0.1 + 0.2 không bằng 0.3 vì nhị phân không biểu diễn chính xác', () {
      expect(0.1 + 0.2 == 0.3, isFalse);
      expect((0.1 + 0.2 - 0.3).abs() < 1e-9, isTrue);
    });

    test('2.001 - 2 không đúng bằng 0.001', () {
      expect(2.001 - 2 == 0.001, isFalse);
    });

    test('làm tròn riêng phần vượt bằng double cho 12004 thay vì 12005', () {
      final extraFare = (2.001 - 2) * 4500; // lý thuyết là 4,5 đồng

      expect(extraFare, lessThan(4.5));
      expect(12000 + extraFare.round(), 12004);
    });

    test('NaN.round() và Infinity.round() ném UnsupportedError', () {
      expect(() => double.nan.round(), throwsUnsupportedError);
      expect(() => double.infinity.round(), throwsUnsupportedError);
    });

    test('NaN không bằng chính nó và mọi so sánh thứ tự đều false', () {
      const nan = double.nan;

      expect(nan == nan, isFalse);
      expect(nan >= 0, isFalse);
      expect(nan < 0, isFalse);
      expect(nan.isNaN, isTrue);
    });

    test('chia double cho 0 ra Infinity, không ném lỗi', () {
      expect(1 / 0, double.infinity);
    });
  });

  group('kiểu', () {
    test('int + double ra double (num)', () {
      final num mixed = 12000 + 4.5;

      expect(mixed, isA<double>());
    });

    test('final khởi tạo muộn nhưng chỉ gán một lần', () {
      final int total;
      total = 25500;

      expect(total, 25500);
    });
  });

  group('int 64-bit trên VM và mobile', () {
    test('chia nguyên cho 0 ném lỗi, còn chia double cho 0 thì không', () {
      expect(() => 1 ~/ 0, throwsUnsupportedError);
    });

    test('tràn số quay vòng quanh, không ném lỗi', () {
      const maxInt = 9223372036854775807;

      expect(maxInt + 1, -9223372036854775808);
    });
  });

  group('null safety', () {
    test('dấu ! ném lỗi lúc chạy khi giá trị là null', () {
      expect(_lengthOf('Tới Nơi'), 7);
      expect(() => _lengthOf(null), throwsA(isA<TypeError>()));
    });

    test('?. và ?? trên giá trị null không ném lỗi', () {
      const String? name = null;

      expect(name?.length, isNull);
      expect(name?.length ?? 0, 0);
    });

    test('kiểm null rồi dùng: biến cục bộ được promote sang String', () {
      expect(_labelOf(null), 'chưa có');
      expect(_labelOf('An'), 'An (2 ký tự)');
    });
  });

  group('dynamic và Object', () {
    test('dynamic dời lỗi sang lúc chạy', () {
      final dynamic anything = 'Tới Nơi';

      expect(anything.length, 7);
      expect(() => anything.fareVnd, throwsNoSuchMethodError);
    });

    test('Object giữ lại kiểm tra lúc biên dịch, is để thu hẹp kiểu', () {
      const Object anything = 'Tới Nơi';

      // anything.length; // không biên dịch: Object không có length
      expect(anything is String ? anything.length : -1, 7);
    });
  });
}

int _lengthOf(String? text) => text!.length;

String _labelOf(String? text) {
  if (text == null) {
    return 'chưa có';
  }
  // Tới đây text đã được promote từ String? thành String.
  return '$text (${text.length} ký tự)';
}

import 'package:test/test.dart';
import 'package:toi_noi_02/fare.dart';

void main() {
  group('calculateFareVnd', () {
    test('0 km vẫn thu giá cơ bản', () {
      expect(calculateFareVnd(0), 12000);
    });

    test('1 km và đúng 2 km nằm trong giá cơ bản', () {
      expect(calculateFareVnd(1), 12000);
      expect(calculateFareVnd(2), 12000);
    });

    test('2.001 km: 1 m vượt = 4,5 đồng, làm tròn nửa lên thành 12005', () {
      expect(calculateFareVnd(2.001), 12005);
    });

    test('độ phân giải 1 m: 2.0004 km tính là 2000 m, 2.0006 km là 2001 m', () {
      expect(calculateFareVnd(2.0004), 12000);
      expect(calculateFareVnd(2.0006), 12005);
    });

    test('2.3 km: 300 m vượt = 1.350 đồng', () {
      expect(calculateFareVnd(2.3), 13350);
    });

    test('5 km: 12.000 + 3 x 4.500 = 25.500', () {
      expect(calculateFareVnd(5), 25500);
    });

    test('100 km là cận trên hợp lệ: 453.000', () {
      expect(calculateFareVnd(100), 453000);
    });

    test('giá trị ngoài miền ném ArgumentError', () {
      expect(() => calculateFareVnd(-1), throwsArgumentError);
      expect(() => calculateFareVnd(-0.001), throwsArgumentError);
      expect(() => calculateFareVnd(100.5), throwsArgumentError);
      expect(() => calculateFareVnd(100.001), throwsArgumentError);
    });

    test('NaN và Infinity ném ArgumentError (NaN so sánh luôn false)', () {
      expect(() => calculateFareVnd(double.nan), throwsArgumentError);
      expect(() => calculateFareVnd(double.infinity), throwsArgumentError);
      expect(
        () => calculateFareVnd(double.negativeInfinity),
        throwsArgumentError,
      );
    });

    test('thông báo lỗi nêu tên tham số và giá trị sai', () {
      expect(
        () => calculateFareVnd(-1),
        throwsA(
          isA<ArgumentError>()
              .having((e) => e.name, 'name', 'distanceKm')
              .having((e) => e.invalidValue, 'invalidValue', -1),
        ),
      );
    });
  });

  group('formatVnd', () {
    test('chèn dấu chấm nghìn', () {
      expect(formatVnd(1234567), '1.234.567 VND');
      expect(formatVnd(25500), '25.500 VND');
    });

    test('số ngắn không có dấu chấm', () {
      expect(formatVnd(0), '0 VND');
      expect(formatVnd(999), '999 VND');
    });

    test('đúng bội của 1000 và số âm', () {
      expect(formatVnd(1000), '1.000 VND');
      expect(formatVnd(100000), '100.000 VND');
      expect(formatVnd(-1234), '-1.234 VND');
    });
  });
}

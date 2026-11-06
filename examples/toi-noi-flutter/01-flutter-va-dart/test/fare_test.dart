import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_01/fare.dart';

void main() {
  group('calculateFareVnd', () {
    test('0 km vẫn thu giá cơ bản', () {
      expect(calculateFareVnd(0), 12000);
    });

    test('2 km đầu nằm trong giá cơ bản', () {
      expect(calculateFareVnd(2), 12000);
    });

    test('5 km: 12.000 + 3 x 4.500', () {
      expect(calculateFareVnd(5), 25500);
    });

    test('2,001 km: phần lẻ làm tròn bằng round() thành 12.005', () {
      expect(calculateFareVnd(2.001), 12005);
    });

    test('100 km là biên trên hợp lệ', () {
      expect(calculateFareVnd(100), 12000 + 98 * 4500);
    });

    test('khoảng cách âm bị từ chối', () {
      expect(() => calculateFareVnd(-1), throwsArgumentError);
    });

    test('vượt 100 km bị từ chối', () {
      expect(() => calculateFareVnd(100.5), throwsArgumentError);
    });
  });
}

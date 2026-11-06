import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('calculateFareVnd', () {
    group('ca biên và ca thường', () {
      // km, cước kỳ vọng (VND). Số minh hoạ, không phải bảng giá thật.
      const cases = <(double, int)>[
        (0, 12000),
        (1.999, 12000),
        (2, 12000),
        (2.001, 12005),
        (2.499, 14246),
        (2.501, 14255),
        (5, 25500),
        (10, 48000),
        (100, 453000),
      ];

      for (final (km, expected) in cases) {
        test('$km km -> $expected VND', () {
          expect(calculateFareVnd(km), expected);
        });
      }
    });

    test('đơn điệu: đi xa hơn không bao giờ rẻ hơn', () {
      var previous = calculateFareVnd(0);
      for (var meters = 1; meters <= 100000; meters += 7) {
        final fare = calculateFareVnd(meters / 1000);
        expect(fare, greaterThanOrEqualTo(previous), reason: '$meters m');
        previous = fare;
      }
    });

    test('mỗi mét vượt 2 km cộng 4,5 đồng, làm tròn nửa lên', () {
      expect(calculateFareVnd(2.001), 12005); // 4,5 -> 5
      expect(calculateFareVnd(2.002), 12009); // 9
      expect(calculateFareVnd(2.003), 12014); // 13,5 -> 14
    });

    group('ngoài miền ném ArgumentError', () {
      const invalid = <double>[
        -1,
        -0.001,
        100.001,
        100.5,
        double.nan,
        double.infinity,
        double.negativeInfinity,
      ];

      for (final km in invalid) {
        test('$km km', () {
          expect(() => calculateFareVnd(km), throwsArgumentError);
        });
      }

      test('thông báo ghi miền 0..100 km', () {
        expect(
          () => calculateFareVnd(101),
          throwsA(
            isA<ArgumentError>().having(
              (e) => e.message,
              'message',
              contains('0..100 km'),
            ),
          ),
        );
      });

      test('thông báo nêu tên tham số', () {
        expect(
          () => calculateFareVnd(-1),
          throwsA(
            isA<ArgumentError>().having((e) => e.name, 'name', 'distanceKm'),
          ),
        );
      });
    });
  });

  group('calculateFare', () {
    test('cùng số với calculateFareVnd nhưng là Money', () {
      expect(calculateFare(5), const Money(25500));
      expect(calculateFare(5).vnd, calculateFareVnd(5));
      expect(calculateFare(5).toString(), '25.500 VND');
    });

    test('đầu vào sai cũng ném ArgumentError', () {
      expect(() => calculateFare(double.nan), throwsArgumentError);
    });
  });

  group('FarePolicy', () {
    test('các hằng khớp bảng giá minh hoạ', () {
      expect(FarePolicy.baseFareVnd, 12000);
      expect(FarePolicy.includedMeters, 2000);
      expect(FarePolicy.pricePerKmVnd, 4500);
      expect(FarePolicy.maxKm, 100);
      expect(FarePolicy.maxKm, isA<int>());
    });
  });
}

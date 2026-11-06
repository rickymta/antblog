import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:latlong2/latlong.dart';
import 'package:toi_noi_34/demo_data.dart';
import 'package:toi_noi_34/geo.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('haversine', () {
    test('hai điểm trùng nhau cách 0 m', () {
      expect(haversineMeters(demoPickup, demoPickup), 0);
    });

    test('một độ vĩ tuyến dài R·π/180 mét', () {
      const a = LatLng(10, 106);
      const b = LatLng(11, 106);
      final expected = earthRadiusMeters * math.pi / 180;
      expect(haversineMeters(a, b), closeTo(expected, 1e-6));
    });

    test('đối xứng và thoả bất đẳng thức tam giác', () {
      expect(
        haversineMeters(demoPickup, demoDropoff),
        closeTo(haversineMeters(demoDropoff, demoPickup), 1e-9),
      );
      final direct = haversineMeters(demoDriver, demoDropoff);
      final viaPickup =
          haversineMeters(demoDriver, demoPickup) +
          haversineMeters(demoPickup, demoDropoff);
      expect(viaPickup, greaterThanOrEqualTo(direct));
    });

    test('khớp DistanceHaversine của latlong2 khi dùng cùng bán kính', () {
      // latlong2 0.10.1 cố định bán kính xích đạo trong công thức haversine.
      const reference = DistanceHaversine(roundResult: false);
      final pairs = <(LatLng, LatLng)>[
        (demoPickup, demoDropoff),
        (demoDriver, demoPickup),
        (const LatLng(10.6, 106.55), const LatLng(10.9, 106.85)),
        (const LatLng(21.0285, 105.8542), demoPickup),
      ];
      for (final (a, b) in pairs) {
        expect(
          haversineMeters(a, b, radius: equatorRadius),
          closeTo(reference.distance(a, b), 1e-6),
        );
      }
    });

    test('bán kính trung bình cho số nhỏ hơn latlong2 khoảng 0,11%', () {
      const library = DistanceHaversine(roundResult: false);
      final ours = haversineMeters(demoPickup, demoDropoff);
      final theirs = library.distance(demoPickup, demoDropoff);
      expect(theirs / ours, closeTo(equatorRadius / earthRadiusMeters, 1e-9));
      expect((theirs - ours) / ours, closeTo(0.00112, 0.00001));
    });

    test('withRadius của latlong2 0.10.1 không đổi kết quả haversine', () {
      final custom = DistanceHaversine.withRadius(1000, roundResult: false);
      const standard = DistanceHaversine(roundResult: false);
      expect(
        custom.distance(demoPickup, demoDropoff),
        standard.distance(demoPickup, demoDropoff),
      );
    });

    test(
      'Distance() mặc định của latlong2 là Vincenty, không phải haversine',
      () {
        const distance = Distance(roundResult: false);
        expect(distance.calculator, isA<Vincenty>());
        final vincenty = distance.distance(demoPickup, demoDropoff);
        final haversine = haversineMeters(demoPickup, demoDropoff);
        expect(vincenty, isNot(closeTo(haversine, 1)));
        // Vẫn cùng cỡ: haversine trên mặt cầu lệch dưới 0,5% ở vĩ độ này.
        expect((vincenty - haversine).abs() / vincenty, lessThan(0.005));
      },
    );

    test('toạ độ hỏng ném ArgumentError, latlong2 trả 0 âm thầm', () {
      const bad = LatLng(double.nan, 106);
      expect(() => haversineMeters(bad, demoPickup), throwsArgumentError);
      expect(
        () => haversineMeters(demoPickup, const LatLng(91, 0)),
        throwsArgumentError,
      );
      expect(
        () => haversineMeters(demoPickup, const LatLng(0, 181)),
        throwsArgumentError,
      );
      expect(const Distance().as(LengthUnit.Meter, bad, demoPickup), 0);
    });
  });

  group('routeMeters và fakeRoute', () {
    test('ít hơn hai điểm dài 0 m', () {
      expect(routeMeters(const []), 0);
      expect(routeMeters([demoPickup]), 0);
    });

    test('cộng các đoạn rồi làm tròn một lần', () {
      final points = [demoDriver, demoPickup, demoDropoff];
      final exact =
          haversineMeters(points[0], points[1]) +
          haversineMeters(points[1], points[2]);
      expect(routeMeters(points), exact.round());
    });

    test('fakeRoute giữ đúng hai đầu và không ngắn hơn đường chim bay', () {
      final pairs = <(LatLng, LatLng)>[
        (demoDriver, demoPickup),
        (demoPickup, demoDropoff),
        (demoDropoff, demoDriver),
      ];
      for (final (a, b) in pairs) {
        final route = fakeRoute(a, b);
        expect(route.first, a);
        expect(route.last, b);
        expect(routeMeters(route), greaterThanOrEqualTo(distanceMeters(a, b)));
      }
    });

    test('đi đúng 5 km về phía bắc thì cước là 25.500 VND', () {
      const from = demoPickup;
      final degrees = 5000 / earthRadiusMeters * 180 / math.pi;
      final to = LatLng(from.latitude + degrees, from.longitude);
      expect(distanceMeters(from, to), 5000);
      final estimate = estimateFare(routeMeters([from, to]));
      expect(estimate, isA<FareQuoted>());
      expect((estimate as FareQuoted).fare, const Money(25500));
    });
  });

  group('estimateFare', () {
    FareQuoted quoted(int meters) => estimateFare(meters) as FareQuoted;

    test('các mốc của bảng giá minh hoạ', () {
      expect(quoted(0).fare, const Money(12000));
      expect(quoted(2000).fare, const Money(12000));
      expect(quoted(2001).fare, const Money(12005));
      expect(quoted(5000).fare, const Money(25500));
      expect(quoted(100000).fare, const Money(453000));
    });

    test('vượt 100 km trả FareOutOfRange, không ném', () {
      final estimate = estimateFare(100001);
      expect(estimate, isA<FareOutOfRange>());
      expect(estimate.meters, 100001);
    });

    test('mét âm bị từ chối', () {
      expect(() => estimateFare(-1), throwsArgumentError);
    });

    test('đổi mét sang km rồi về mét không lệch với mọi mét 0..100000', () {
      // Công thức nguyên của bài 02, viết lại độc lập với calculateFare.
      int expected(int meters) =>
          FarePolicy.baseFareVnd +
          (math.max(0, meters - FarePolicy.includedMeters) *
                      FarePolicy.pricePerKmVnd +
                  500) ~/
              1000;
      for (var meters = 0; meters <= 100000; meters++) {
        expect(quoted(meters).fare.vnd, expected(meters), reason: '$meters m');
      }
    });
  });
}

import 'package:flutter_test/flutter_test.dart';
import 'package:latlong2/latlong.dart';
import 'package:toi_noi_34/demo_data.dart';
import 'package:toi_noi_34/geo.dart';

void main() {
  const area = demoServiceArea;

  test('ba điểm minh hoạ đều nằm trong vùng phục vụ', () {
    for (final p in [demoPickup, demoDropoff, demoDriver]) {
      expect(area.contains(p), isTrue, reason: '$p');
    }
  });

  group('contains', () {
    test('biên tính là trong', () {
      expect(area.contains(area.southWest), isTrue);
      expect(area.contains(area.northEast), isTrue);
      expect(area.contains(LatLng(area.north, area.west)), isTrue);
    });

    test('sát biên ra ngoài là ngoài', () {
      expect(area.contains(LatLng(area.north + 1e-9, 106.7)), isFalse);
      expect(area.contains(LatLng(10.7, area.east + 1e-9)), isFalse);
      expect(area.contains(LatLng(area.south - 1e-9, 106.7)), isFalse);
      expect(area.contains(LatLng(10.7, area.west - 1e-9)), isFalse);
    });

    test('toạ độ hỏng ném ArgumentError thay vì trả false', () {
      expect(
        () => area.contains(const LatLng(double.nan, 106.7)),
        throwsArgumentError,
      );
    });
  });

  group('clamp', () {
    test('điểm trong vùng giữ nguyên', () {
      expect(area.clamp(demoPickup), demoPickup);
    });

    test('kẹp từng trục độc lập', () {
      expect(area.clamp(const LatLng(12, 106.7)), LatLng(area.north, 106.7));
      expect(area.clamp(const LatLng(10.7, 100)), LatLng(10.7, area.west));
    });

    test('góc ngoài kẹp về đúng góc', () {
      expect(area.clamp(const LatLng(0, 180)), LatLng(area.south, area.east));
      expect(area.clamp(const LatLng(60, 0)), LatLng(area.north, area.west));
    });

    test('kết quả luôn nằm trong vùng', () {
      for (var lat = -90.0; lat <= 90; lat += 30) {
        for (var lng = -180.0; lng <= 180; lng += 60) {
          expect(area.contains(area.clamp(LatLng(lat, lng))), isTrue);
        }
      }
    });

    test('toạ độ hỏng ném ArgumentError', () {
      expect(
        () => area.clamp(const LatLng(10.7, double.infinity)),
        throwsArgumentError,
      );
    });
  });

  test('NaN.clamp trả cận trên, nên kiểm toạ độ phải làm trước clamp', () {
    // Không có _checkCoordinate, toạ độ hỏng sẽ thành điểm hợp lệ âm thầm.
    expect(double.nan.clamp(area.south, area.north), area.north);
  });

  test('center là trung điểm hai góc', () {
    expect(area.center.latitude, closeTo(10.75, 1e-9));
    expect(area.center.longitude, closeTo(106.70, 1e-9));
  });

  test('vùng có south lớn hơn north bị assert chặn', () {
    expect(
      () => ServiceArea(south: 11, west: 106, north: 10, east: 107),
      throwsA(isA<AssertionError>()),
    );
  });
}

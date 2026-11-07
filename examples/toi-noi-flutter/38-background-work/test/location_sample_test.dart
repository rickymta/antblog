import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/location_sample.dart';

import 'test_helpers.dart';

void main() {
  test('JSON đi rồi về cho mẫu bằng nhau', () {
    final s = sample(3);
    expect(LocationSample.fromJson(s.toJson()), s);
  });

  test('dedupeKey gồm tripId và thời điểm UTC', () {
    expect(sample(0).dedupeKey, 'T1@2026-11-06T08:00:00.000Z');
    expect(sample(0, tripId: 'T2').dedupeKey, isNot(sample(0).dedupeKey));
  });

  group('fromJson ném FormatException', () {
    Map<String, Object?> good() => sample(0).toJson();

    test('không phải object', () {
      expect(() => LocationSample.fromJson([1]), throwsFormatException);
      expect(() => LocationSample.fromJson(null), throwsFormatException);
    });

    test('thiếu khoá', () {
      expect(
        () => LocationSample.fromJson(good()..remove('lat')),
        throwsFormatException,
      );
    });

    test('sai kiểu', () {
      expect(
        () => LocationSample.fromJson(good()..['lng'] = 'x'),
        throwsFormatException,
      );
    });

    test('thời điểm không phải UTC', () {
      expect(
        () => LocationSample.fromJson(good()..['at'] = '2026-11-06T08:00:00'),
        throwsFormatException,
      );
    });

    test('thời điểm không đọc được', () {
      expect(
        () => LocationSample.fromJson(good()..['at'] = 'hôm qua'),
        throwsFormatException,
      );
    });
  });

  test('assert chặn capturedAt không phải UTC', () {
    expect(
      () => LocationSample(
        tripId: 'T1',
        latitude: 0,
        longitude: 0,
        accuracyMeters: 1,
        capturedAt: DateTime(2026, 11, 6),
      ),
      throwsA(isA<AssertionError>()),
    );
  });
}

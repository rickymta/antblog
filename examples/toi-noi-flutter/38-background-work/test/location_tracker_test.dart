import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:geolocator/geolocator.dart';
import 'package:toi_noi_38/location_tracker.dart';

void main() {
  group('buildBackgroundLocationSettings', () {
    test('Android: chạy trong foreground service có thông báo bền', () {
      final settings = buildBackgroundLocationSettings(TargetPlatform.android);
      expect(settings, isA<AndroidSettings>());
      final config = (settings as AndroidSettings).foregroundNotificationConfig;
      expect(config, isNotNull);
      expect(config!.notificationTitle, isNotEmpty);
      expect(config.notificationText, isNotEmpty);
      expect(config.setOngoing, isTrue);
      expect(settings.accuracy, LocationAccuracy.high);
      expect(settings.distanceFilter, 25);
    });

    test('iOS: cho phép nhận vị trí ở nền và bật chỉ báo của hệ điều hành', () {
      final settings = buildBackgroundLocationSettings(TargetPlatform.iOS);
      expect(settings, isA<AppleSettings>());
      final apple = settings as AppleSettings;
      expect(apple.allowBackgroundLocationUpdates, isTrue);
      expect(apple.showBackgroundLocationIndicator, isTrue);
      expect(apple.pauseLocationUpdatesAutomatically, isFalse);
      expect(apple.activityType, ActivityType.automotiveNavigation);
    });

    test('nền tảng khác: cấu hình trơn, không có tính năng nền', () {
      final settings = buildBackgroundLocationSettings(TargetPlatform.macOS);
      expect(settings, isNot(isA<AndroidSettings>()));
      expect(settings, isNot(isA<AppleSettings>()));
    });
  });

  group('GeolocatorLocationTracker', () {
    Position position(double lat, DateTime at) => Position(
      latitude: lat,
      longitude: 106.7,
      timestamp: at,
      accuracy: 6,
      altitude: 0,
      altitudeAccuracy: 0,
      heading: 0,
      headingAccuracy: 0,
      speed: 0,
      speedAccuracy: 0,
    );

    test('đổi Position thành mẫu của chuyến, thời điểm UTC', () async {
      final controller = StreamController<Position>();
      final tracker = GeolocatorLocationTracker(
        positionStream: (_) => controller.stream,
        platform: TargetPlatform.android,
      );
      final future = tracker.positions(tripId: 'T9').first;
      // Thời điểm có múi giờ +07:00: mẫu phải được đổi sang UTC.
      controller.add(
        position(10.77, DateTime.parse('2026-11-06T15:00:00+07:00')),
      );
      final sample = await future;
      expect(sample.tripId, 'T9');
      expect(sample.latitude, 10.77);
      expect(sample.accuracyMeters, 6);
      expect(sample.capturedAt, DateTime.utc(2026, 11, 6, 8));
      expect(sample.capturedAt.isUtc, isTrue);
      await controller.close();
    });

    test('truyền đúng cấu hình nền của nền tảng cho nguồn vị trí', () async {
      LocationSettings? received;
      final tracker = GeolocatorLocationTracker(
        positionStream: (s) {
          received = s;
          return const Stream<Position>.empty();
        },
        platform: TargetPlatform.iOS,
      );
      await tracker.positions(tripId: 'T1').toList();
      expect(received, isA<AppleSettings>());
    });
  });
}

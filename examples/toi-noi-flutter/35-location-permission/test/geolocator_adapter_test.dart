import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show PlatformException;
import 'package:flutter_test/flutter_test.dart';
import 'package:geolocator/geolocator.dart';
import 'package:permission_handler/permission_handler.dart' as ph;
import 'package:permission_handler_platform_interface/permission_handler_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';
import 'package:toi_noi_35/geolocator_location_source.dart';
import 'package:toi_noi_35/location_models.dart';

/// `GeolocatorPlatform` giả: thay cho channel của plugin, để kiểm lớp bọc mà
/// không cần thiết bị.
class FakeGeolocatorPlatform extends GeolocatorPlatform
    with MockPlatformInterfaceMixin {
  LocationPermission permission = LocationPermission.denied;
  Object? requestError;
  bool serviceEnabled = true;
  LocationAccuracyStatus accuracy = LocationAccuracyStatus.precise;
  Object? accuracyError;
  final StreamController<Position> positions = StreamController();
  LocationSettings? streamSettings;
  String? temporaryPurposeKey;
  int requestCalls = 0;

  @override
  Future<LocationPermission> checkPermission() async => permission;

  @override
  Future<LocationPermission> requestPermission() async {
    requestCalls++;
    final error = requestError;
    if (error != null) {
      throw error;
    }
    return permission;
  }

  @override
  Future<bool> isLocationServiceEnabled() async => serviceEnabled;

  @override
  Future<LocationAccuracyStatus> getLocationAccuracy() async {
    final error = accuracyError;
    if (error != null) {
      throw error;
    }
    return accuracy;
  }

  @override
  Future<LocationAccuracyStatus> requestTemporaryFullAccuracy({
    required String purposeKey,
  }) async {
    temporaryPurposeKey = purposeKey;
    accuracy = LocationAccuracyStatus.precise;
    return accuracy;
  }

  @override
  Stream<Position> getPositionStream({LocationSettings? locationSettings}) {
    streamSettings = locationSettings;
    return positions.stream;
  }
}

class FakePermissionHandler extends PermissionHandlerPlatform
    with MockPlatformInterfaceMixin {
  final List<Permission> requested = [];

  @override
  Future<Map<Permission, PermissionStatus>> requestPermissions(
    List<Permission> permissions,
  ) async {
    requested.addAll(permissions);
    return {for (final p in permissions) p: PermissionStatus.granted};
  }
}

Position position({double lat = 10.7769, double lng = 106.7009}) => Position(
  latitude: lat,
  longitude: lng,
  timestamp: DateTime.utc(2026, 11, 6, 3),
  accuracy: 12,
  altitude: 0,
  altitudeAccuracy: 0,
  heading: 0,
  headingAccuracy: 0,
  speed: 0,
  speedAccuracy: 0,
);

void main() {
  late FakeGeolocatorPlatform platform;
  late GeolocatorPlatform original;
  const source = GeolocatorLocationSource();

  setUp(() {
    original = GeolocatorPlatform.instance;
    platform = FakeGeolocatorPlatform();
    GeolocatorPlatform.instance = platform;
  });

  tearDown(() {
    GeolocatorPlatform.instance = original;
    debugDefaultTargetPlatformOverride = null;
  });

  group('buildLocationSettings', () {
    test('Android khách: AndroidSettings, không có foreground service', () {
      final s = buildLocationSettings(
        TrackingProfile.customerPickup,
        platform: TargetPlatform.android,
      ) as AndroidSettings;
      expect(s.accuracy, LocationAccuracy.high);
      expect(s.distanceFilter, 10);
      expect(s.intervalDuration, const Duration(seconds: 5));
      expect(s.foregroundNotificationConfig, isNull);
    });

    test('Android tài xế đang có chuyến: có thông báo foreground service', () {
      final s = buildLocationSettings(
        TrackingProfile.driverOnTrip,
        platform: TargetPlatform.android,
      ) as AndroidSettings;
      final config = s.foregroundNotificationConfig;
      expect(config, isNotNull);
      expect(config!.setOngoing, isTrue);
      expect(config.notificationTitle, contains('Tới Nơi'));
      expect(s.distanceFilter, 25);
    });

    test(
      'Android tài xế online chưa có chuyến: chưa có foreground service',
      () {
        final s = buildLocationSettings(
          TrackingProfile.driverOnline,
          platform: TargetPlatform.android,
        ) as AndroidSettings;
        expect(s.foregroundNotificationConfig, isNull);
        expect(s.accuracy, LocationAccuracy.medium);
      },
    );

    test('iOS khách: tắt rõ ràng allowBackgroundLocationUpdates', () {
      final s = buildLocationSettings(
        TrackingProfile.customerPickup,
        platform: TargetPlatform.iOS,
      ) as AppleSettings;
      expect(s.allowBackgroundLocationUpdates, isFalse);
      expect(s.showBackgroundLocationIndicator, isFalse);
      expect(s.distanceFilter, 10);
    });

    test('iOS tài xế có chuyến: cho phép nền và hiện chỉ báo nền', () {
      final s = buildLocationSettings(
        TrackingProfile.driverOnTrip,
        platform: TargetPlatform.iOS,
      ) as AppleSettings;
      expect(s.allowBackgroundLocationUpdates, isTrue);
      expect(s.showBackgroundLocationIndicator, isTrue);
      expect(s.pauseLocationUpdatesAutomatically, isFalse);
      expect(s.activityType, ActivityType.automotiveNavigation);
    });

    test(
      'mặc định của plugin cho AppleSettings là cho phép nền (lý do ta đặt rõ)',
      () {
        expect(AppleSettings().allowBackgroundLocationUpdates, isTrue);
      },
    );

    test('nền tảng khác: LocationSettings thường', () {
      final s = buildLocationSettings(
        TrackingProfile.customerPickup,
        platform: TargetPlatform.linux,
      );
      expect(s, isNot(isA<AndroidSettings>()));
      expect(s, isNot(isA<AppleSettings>()));
      expect(s.distanceFilter, 10);
    });
  });

  group('quyền', () {
    test('map LocationPermission sang SystemPermission', () async {
      final expected = {
        LocationPermission.denied: SystemPermission.denied,
        LocationPermission.unableToDetermine: SystemPermission.denied,
        LocationPermission.deniedForever: SystemPermission.deniedForever,
        LocationPermission.whileInUse: SystemPermission.whileInUse,
        LocationPermission.always: SystemPermission.always,
      };
      for (final entry in expected.entries) {
        platform.permission = entry.key;
        expect(
          await source.checkPermission(),
          entry.value,
          reason: '${entry.key}',
        );
      }
    });

    test(
      'thiếu khai báo native thành LocationConfigurationException',
      () async {
        platform.requestError = const PermissionDefinitionsNotFoundException(
          'thiếu Info.plist',
        );
        await expectLater(
          source.requestForeground(),
          throwsA(isA<LocationConfigurationException>()),
        );
      },
    );

    test(
      'đang có hộp thoại khác thành LocationRequestInProgressException',
      () async {
        platform.requestError = const PermissionRequestInProgressException(
          'bận',
        );
        await expectLater(
          source.requestForeground(),
          throwsA(isA<LocationRequestInProgressException>()),
        );
      },
    );

    test('chỉ iOS có bước xin nền', () {
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      expect(source.canRequestBackground, isTrue);
      debugDefaultTargetPlatformOverride = TargetPlatform.android;
      expect(source.canRequestBackground, isFalse);
    });

    test('Android: requestBackground không gọi permission_handler', () async {
      debugDefaultTargetPlatformOverride = TargetPlatform.android;
      final handler = FakePermissionHandler();
      final original = PermissionHandlerPlatform.instance;
      PermissionHandlerPlatform.instance = handler;
      addTearDown(() => PermissionHandlerPlatform.instance = original);
      platform.permission = LocationPermission.whileInUse;
      expect(await source.requestBackground(), SystemPermission.whileInUse);
      expect(handler.requested, isEmpty);
    });

    test(
      'iOS: requestBackground đi qua permission_handler, không qua geolocator',
      () async {
        debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
        final handler = FakePermissionHandler();
        final original = PermissionHandlerPlatform.instance;
        PermissionHandlerPlatform.instance = handler;
        addTearDown(() => PermissionHandlerPlatform.instance = original);

        platform.permission = LocationPermission.always;
        final result = await source.requestBackground();

        expect(handler.requested, [ph.Permission.locationAlways]);
        expect(platform.requestCalls, 0);
        expect(result, SystemPermission.always);
      },
    );
  });

  group('độ chính xác', () {
    test('reduced và precise', () async {
      platform.accuracy = LocationAccuracyStatus.reduced;
      expect(await source.precision(), LocationPrecision.reduced);
      platform.accuracy = LocationAccuracyStatus.precise;
      expect(await source.precision(), LocationPrecision.precise);
    });

    test(
      'Android chưa có quyền ném PlatformException: coi là reduced',
      () async {
        platform.accuracyError = PlatformException(code: 'PERMISSION_DENIED');
        expect(await source.precision(), LocationPrecision.reduced);
      },
    );

    test(
      'iOS: nâng cấp bằng requestTemporaryFullAccuracy với đúng purposeKey',
      () async {
        debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
        platform.accuracy = LocationAccuracyStatus.reduced;
        final result = await source.requestPrecise();
        expect(platform.temporaryPurposeKey, preciseLocationPurposeKey);
        expect(result, LocationPrecision.precise);
      },
    );

    test('Android: nâng cấp bằng cách hỏi lại requestPermission', () async {
      debugDefaultTargetPlatformOverride = TargetPlatform.android;
      await source.requestPrecise();
      expect(platform.requestCalls, 1);
      expect(platform.temporaryPurposeKey, isNull);
    });
  });

  group('positionStream', () {
    test(
      'truyền đúng settings của profile và đổi Position thành LocationFix',
      () async {
        debugDefaultTargetPlatformOverride = TargetPlatform.android;
        final fixes = <LocationFix>[];
        final sub = source
            .positionStream(TrackingProfile.driverOnTrip)
            .listen(fixes.add);
        expect(platform.streamSettings, isA<AndroidSettings>());
        expect(platform.streamSettings!.distanceFilter, 25);

        platform.positions.add(position());
        await Future<void>.delayed(Duration.zero);
        expect(fixes.single.latitude, 10.7769);
        expect(fixes.single.accuracyMeters, 12);
        expect(fixes.single.timestamp.isUtc, isTrue);
        await sub.cancel();
      },
    );

    test(
      'LocationServiceDisabledException thành LocationServiceOffException',
      () async {
        final errors = <Object>[];
        final sub = source
            .positionStream(TrackingProfile.customerPickup)
            .listen((_) {}, onError: errors.add);
        platform.positions.addError(const LocationServiceDisabledException());
        await Future<void>.delayed(Duration.zero);
        expect(errors.single, isA<LocationServiceOffException>());
        await sub.cancel();
      },
    );
  });
}

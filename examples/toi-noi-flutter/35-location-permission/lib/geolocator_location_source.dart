import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show PlatformException;
import 'package:geolocator/geolocator.dart';
import 'package:permission_handler/permission_handler.dart' as ph;

import 'location_models.dart';
import 'location_source.dart';

/// Khoá trong `NSLocationTemporaryUsageDescriptionDictionary` của Info.plist.
const preciseLocationPurposeKey = 'PickupPrecise';

/// Dựng `LocationSettings` của plugin từ [profile] cho [platform].
///
/// Tách thành hàm để test được mà không cần thiết bị.
LocationSettings buildLocationSettings(
  TrackingProfile profile, {
  TargetPlatform? platform,
}) {
  final accuracy = switch (profile.accuracy) {
    TrackingAccuracy.balanced => LocationAccuracy.medium,
    TrackingAccuracy.high => LocationAccuracy.high,
  };
  return switch (platform ?? defaultTargetPlatform) {
    TargetPlatform.android => AndroidSettings(
      accuracy: accuracy,
      distanceFilter: profile.distanceFilterMeters,
      intervalDuration: profile.interval,
      // Có thông báo là chạy như foreground service; chỉ profile nền mới có.
      foregroundNotificationConfig: profile.background
          ? const ForegroundNotificationConfig(
              notificationTitle: 'Tới Nơi đang chia sẻ vị trí',
              notificationText:
                  'Khách đang theo dõi chuyến này. Hết chuyến, app sẽ dừng.',
              notificationChannelName: 'Vị trí trong chuyến đi',
              setOngoing: true,
            )
          : null,
    ),
    TargetPlatform.iOS || TargetPlatform.macOS => AppleSettings(
      accuracy: accuracy,
      distanceFilter: profile.distanceFilterMeters,
      activityType: ActivityType.automotiveNavigation,
      pauseLocationUpdatesAutomatically: false,
      // Mặc định của plugin là true; đặt rõ để app khách không phụ thuộc vào
      // việc Info.plist có UIBackgroundModes hay không.
      allowBackgroundLocationUpdates: profile.background,
      showBackgroundLocationIndicator: profile.background,
    ),
    TargetPlatform.fuchsia ||
    TargetPlatform.linux ||
    TargetPlatform.windows => LocationSettings(
      accuracy: accuracy,
      distanceFilter: profile.distanceFilterMeters,
    ),
  };
}

/// Lớp bọc mỏng quanh `geolocator` (và `permission_handler` cho bước xin nền).
final class GeolocatorLocationSource implements LocationSource {
  const GeolocatorLocationSource();

  @override
  Future<bool> isServiceEnabled() => Geolocator.isLocationServiceEnabled();

  @override
  Future<SystemPermission> checkPermission() async =>
      _map(await Geolocator.checkPermission());

  @override
  Future<SystemPermission> requestForeground() async {
    try {
      return _map(await Geolocator.requestPermission());
    } on PermissionDefinitionsNotFoundException catch (e) {
      throw LocationConfigurationException(e.toString());
    } on PermissionRequestInProgressException catch (e) {
      throw LocationRequestInProgressException(e.toString());
    }
  }

  @override
  bool get canRequestBackground => defaultTargetPlatform == TargetPlatform.iOS;

  @override
  Future<SystemPermission> requestBackground() async {
    // geolocator_apple chỉ gọi requestWhenInUseAuthorization khi Info.plist có
    // NSLocationWhenInUseUsageDescription, và trả về ngay khi quyền đã được
    // quyết định, nên không thể nâng lên "Always". Bước này đi qua
    // permission_handler (requestAlwaysAuthorization). Chỉ dùng trên iOS.
    if (!canRequestBackground) {
      return checkPermission();
    }
    await ph.Permission.locationAlways.request();
    return checkPermission();
  }

  @override
  Future<LocationPrecision> precision() async {
    try {
      final status = await Geolocator.getLocationAccuracy();
      return status == LocationAccuracyStatus.precise
          ? LocationPrecision.precise
          : LocationPrecision.reduced;
    } on PlatformException {
      // Android ném PERMISSION_DENIED khi chưa có quyền nào (geolocator 14.1.2).
      return LocationPrecision.reduced;
    }
  }

  @override
  Future<LocationPrecision> requestPrecise() async {
    switch (defaultTargetPlatform) {
      case TargetPlatform.iOS:
        await Geolocator.requestTemporaryFullAccuracy(
          purposeKey: preciseLocationPurposeKey,
        );
      case TargetPlatform.android:
        // Android: hỏi lại cặp COARSE + FINE; hệ thống hiện hộp thoại nâng cấp.
        // Chỉ chạy đúng khi manifest KHÔNG khai ACCESS_BACKGROUND_LOCATION: nếu
        // có, geolocator_android thêm quyền nền vào cùng yêu cầu.
        await Geolocator.requestPermission();
      case TargetPlatform.macOS ||
          TargetPlatform.fuchsia ||
          TargetPlatform.linux ||
          TargetPlatform.windows:
        break;
    }
    return precision();
  }

  @override
  Future<bool> openAppSettings() => Geolocator.openAppSettings();

  @override
  Future<bool> openLocationSettings() => Geolocator.openLocationSettings();

  @override
  Stream<LocationFix> positionStream(TrackingProfile profile) {
    return Geolocator.getPositionStream(
      locationSettings: buildLocationSettings(profile),
    ).map(_toFix).handleError((Object error) {
      throw const LocationServiceOffException(
        'Dịch vụ vị trí của thiết bị đã tắt',
      );
    }, test: (error) => error is LocationServiceDisabledException);
  }

  static LocationFix _toFix(Position p) => LocationFix(
    latitude: p.latitude,
    longitude: p.longitude,
    accuracyMeters: p.accuracy,
    timestamp: p.timestamp.toUtc(),
  );

  static SystemPermission _map(LocationPermission p) => switch (p) {
    LocationPermission.denied ||
    LocationPermission.unableToDetermine => SystemPermission.denied,
    LocationPermission.deniedForever => SystemPermission.deniedForever,
    LocationPermission.whileInUse => SystemPermission.whileInUse,
    LocationPermission.always => SystemPermission.always,
  };
}

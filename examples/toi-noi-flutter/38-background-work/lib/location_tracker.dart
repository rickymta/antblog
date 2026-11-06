import 'package:flutter/foundation.dart';
import 'package:geolocator/geolocator.dart';

import 'location_sample.dart';

/// Nguồn vị trí của tài xế trong một chuyến. Tách khỏi `geolocator` để logic
/// phía trên test được bằng một `Stream` giả.
abstract interface class LocationTracker {
  /// Huỷ subscription là dừng thu vị trí (và, trên Android, dừng foreground
  /// service đi kèm).
  Stream<LocationSample> positions({required String tripId});
}

/// Cấu hình luồng vị trí **nền** cho từng nền tảng.
///
/// - Android: có `foregroundNotificationConfig` thì plugin chạy luồng trong
///   một foreground service kèm thông báo.
/// - iOS: `allowBackgroundLocationUpdates` cho phép nhận vị trí khi app ở
///   nền (cần `UIBackgroundModes` có `location`). Với quyền "khi đang dùng"
///   hệ điều hành tự hiện chỉ báo xanh dương; `showBackgroundLocationIndicator`
///   chỉ tác động tới app đã có quyền "luôn luôn", ta đặt `true` cho minh bạch.
LocationSettings buildBackgroundLocationSettings(TargetPlatform platform) {
  const accuracy = LocationAccuracy.high;
  const distanceFilter = 25; // mét: xe đứng yên thì không phát thêm mẫu
  return switch (platform) {
    TargetPlatform.android => AndroidSettings(
      accuracy: accuracy,
      distanceFilter: distanceFilter,
      intervalDuration: const Duration(seconds: 5),
      foregroundNotificationConfig: const ForegroundNotificationConfig(
        notificationTitle: 'Tới Nơi đang chia sẻ vị trí',
        notificationText: 'Khách đang theo dõi xe của bạn trong chuyến này.',
        notificationChannelName: 'Vị trí chuyến đi',
        setOngoing: true,
      ),
    ),
    TargetPlatform.iOS => AppleSettings(
      accuracy: accuracy,
      distanceFilter: distanceFilter,
      activityType: ActivityType.automotiveNavigation,
      pauseLocationUpdatesAutomatically: false,
      allowBackgroundLocationUpdates: true,
      showBackgroundLocationIndicator: true,
    ),
    TargetPlatform.fuchsia ||
    TargetPlatform.linux ||
    TargetPlatform.macOS ||
    TargetPlatform.windows => const LocationSettings(
      accuracy: accuracy,
      distanceFilter: distanceFilter,
    ),
  };
}

typedef PositionStreamFactory = Stream<Position> Function(
  LocationSettings settings,
);

/// Lớp bọc mỏng quanh `Geolocator.getPositionStream`.
final class GeolocatorLocationTracker implements LocationTracker {
  GeolocatorLocationTracker({
    PositionStreamFactory? positionStream,
    TargetPlatform? platform,
  }) : _positionStream = positionStream ?? _geolocatorStream,
       _platform = platform ?? defaultTargetPlatform;

  final PositionStreamFactory _positionStream;
  final TargetPlatform _platform;

  static Stream<Position> _geolocatorStream(LocationSettings settings) =>
      Geolocator.getPositionStream(locationSettings: settings);

  @override
  Stream<LocationSample> positions({required String tripId}) {
    return _positionStream(buildBackgroundLocationSettings(_platform)).map(
      (p) => LocationSample(
        tripId: tripId,
        latitude: p.latitude,
        longitude: p.longitude,
        accuracyMeters: p.accuracy,
        capturedAt: p.timestamp.toUtc(),
      ),
    );
  }
}

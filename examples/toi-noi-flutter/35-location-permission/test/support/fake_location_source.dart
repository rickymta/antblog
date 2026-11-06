import 'dart:async';

import 'package:toi_noi_35/location_models.dart';
import 'package:toi_noi_35/location_source.dart';

/// Fake có thể điều khiển được: ta đặt quyền hệ điều hành sẽ trả lời khi hộp
/// thoại "hiện ra", và đếm số lần ứng dụng thật sự xin quyền.
class FakeLocationSource implements LocationSource {
  FakeLocationSource({
    this.serviceEnabled = true,
    this.permission = SystemPermission.denied,
    this.precisionValue = LocationPrecision.precise,
    this.androidLike = false,
    this.backgroundSupported = true,
  });

  /// Mô phỏng Android: `checkPermission` không bao giờ trả `deniedForever`;
  /// chỉ kết quả của chính lần xin mới trả (xem PermissionManager.java của
  /// geolocator_android).
  final bool androidLike;
  final bool backgroundSupported;

  bool serviceEnabled;
  SystemPermission permission;
  LocationPrecision precisionValue;

  /// Quyền sau khi người dùng trả lời hộp thoại tiếp theo (xếp hàng theo thứ tự).
  final List<SystemPermission> foregroundAnswers = [];
  final List<SystemPermission> backgroundAnswers = [];
  LocationPrecision? preciseAnswer;

  /// Nếu đặt, hộp thoại không trả lời cho tới khi `complete()`.
  Completer<void>? foregroundGate;

  Object? foregroundError;

  int foregroundRequests = 0;
  int backgroundRequests = 0;
  int preciseRequests = 0;
  int appSettingsOpened = 0;
  int locationSettingsOpened = 0;
  final List<TrackingProfile> streamsOpened = [];
  final List<StreamController<LocationFix>> controllers = [];

  @override
  Future<bool> isServiceEnabled() async => serviceEnabled;

  @override
  Future<SystemPermission> checkPermission() async =>
      androidLike && permission == SystemPermission.deniedForever
      ? SystemPermission.denied
      : permission;

  @override
  Future<LocationPrecision> precision() async => precisionValue;

  @override
  Future<SystemPermission> requestForeground() async {
    foregroundRequests++;
    final error = foregroundError;
    if (error != null) {
      throw error;
    }
    await foregroundGate?.future;
    if (foregroundAnswers.isNotEmpty) {
      permission = foregroundAnswers.removeAt(0);
    }
    return permission;
  }

  @override
  bool get canRequestBackground => backgroundSupported;

  @override
  Future<SystemPermission> requestBackground() async {
    backgroundRequests++;
    if (backgroundAnswers.isNotEmpty) {
      permission = backgroundAnswers.removeAt(0);
    }
    return permission;
  }

  @override
  Future<LocationPrecision> requestPrecise() async {
    preciseRequests++;
    precisionValue = preciseAnswer ?? precisionValue;
    return precisionValue;
  }

  @override
  Future<bool> openAppSettings() async {
    appSettingsOpened++;
    return true;
  }

  @override
  Future<bool> openLocationSettings() async {
    locationSettingsOpened++;
    return true;
  }

  @override
  Stream<LocationFix> positionStream(TrackingProfile profile) {
    streamsOpened.add(profile);
    final controller = StreamController<LocationFix>();
    controllers.add(controller);
    return controller.stream;
  }
}

LocationFix fixAt(double lat, double lng, {double accuracy = 8}) => LocationFix(
  latitude: lat,
  longitude: lng,
  accuracyMeters: accuracy,
  timestamp: DateTime.utc(2026, 11, 6, 3),
);

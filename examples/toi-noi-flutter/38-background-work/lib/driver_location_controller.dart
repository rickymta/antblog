import 'dart:async';

import 'package:toi_noi_core/toi_noi_core.dart';

import 'driver_location_policy.dart';
import 'location_sample.dart';
import 'location_tracker.dart';
import 'location_uploader.dart';

/// Nối chính sách với nguồn vị trí: bật luồng khi được phép, tắt khi hết phép.
///
/// Gọi [onStateChanged] mỗi khi trạng thái chuyến hoặc trạng thái online của
/// tài xế đổi. Controller không tự suy ra gì từ bản đồ hay từ thời gian.
final class DriverLocationController {
  DriverLocationController({
    required this.policy,
    required this.tracker,
    required this.uploader,
  });

  final DriverLocationPolicy policy;
  final LocationTracker tracker;
  final LocationUploader uploader;

  StreamSubscription<LocationSample>? _subscription;
  String? _activeTripId;
  Object? _lastError;

  bool get isTracking => _subscription != null;
  String? get activeTripId => _activeTripId;

  /// Lỗi gần nhất của nguồn vị trí, ví dụ người dùng thu hồi quyền.
  Object? get lastError => _lastError;

  Future<void> onStateChanged({
    required String? tripId,
    required TripStatus? status,
    required bool driverOnline,
  }) async {
    final allowed =
        tripId != null &&
        policy.allowsBackgroundSharing(
          tripStatus: status,
          driverOnline: driverOnline,
        );
    if (!allowed) {
      await _stop();
      return;
    }
    if (_activeTripId == tripId) return;
    await _stop(); // đổi sang chuyến khác: không trộn vị trí hai chuyến
    _start(tripId);
  }

  void _start(String tripId) {
    _activeTripId = tripId;
    _lastError = null;
    _subscription = tracker
        .positions(tripId: tripId)
        .listen(
          (sample) => unawaited(uploader.record(sample)),
          onError: (Object error) => _lastError = error,
        );
  }

  Future<void> _stop() async {
    final subscription = _subscription;
    if (subscription == null) return;
    _subscription = null;
    _activeTripId = null;
    await subscription.cancel();
    await uploader.discardPending();
  }

  Future<void> dispose() => _stop();
}

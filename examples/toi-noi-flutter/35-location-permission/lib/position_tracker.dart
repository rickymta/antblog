import 'dart:async';

import 'package:flutter/foundation.dart';

import 'location_access.dart';
import 'location_access_controller.dart';
import 'location_models.dart';
import 'location_source.dart';

/// Mở luồng vị trí khi quyền cho phép và đóng ngay khi quyền mất.
///
/// Luôn gọi [dispose]: subscription còn sống là cảm biến còn bật.
final class PositionTracker {
  PositionTracker({required this._source, required this._access}) {
    _access.addListener(_sync);
  }

  final LocationSource _source;
  final LocationAccessController _access;

  /// Lần đo gần nhất, `null` khi chưa có hoặc đã dừng.
  final ValueNotifier<LocationFix?> lastFix = ValueNotifier(null);

  /// Lỗi gần nhất của luồng (ví dụ dịch vụ vị trí bị tắt giữa chừng).
  final ValueNotifier<LocationSourceException?> lastError = ValueNotifier(null);

  TrackingProfile? _wantedProfile;
  TrackingProfile? _runningProfile;
  StreamSubscription<LocationFix>? _subscription;

  bool get isTracking => _subscription != null;

  /// Đặt profile mong muốn; `null` nghĩa là không cần theo dõi.
  void setProfile(TrackingProfile? profile) {
    _wantedProfile = profile;
    _sync();
  }

  void _sync() {
    final wanted = _access.value.canTrack ? _wantedProfile : null;
    if (wanted == _runningProfile) {
      return;
    }
    unawaited(_subscription?.cancel());
    _subscription = null;
    _runningProfile = wanted;
    if (wanted == null) {
      lastFix.value = null;
      return;
    }
    _subscription = _source
        .positionStream(wanted)
        .listen(
          (fix) {
            lastError.value = null;
            lastFix.value = fix;
          },
          onError: (Object error) {
            if (error is LocationSourceException) {
              lastError.value = error;
            }
          },
        );
  }

  void dispose() {
    _access.removeListener(_sync);
    unawaited(_subscription?.cancel());
    _subscription = null;
    lastFix.dispose();
    lastError.dispose();
  }
}

import 'package:firebase_remote_config/firebase_remote_config.dart';

import 'feature_flags.dart';

/// Lớp bọc mỏng quanh `firebase_remote_config`. Chưa chạy với Firebase project
/// thật ở lab này, chỉ kiểm bằng `flutter analyze`.
///
/// Không gọi `setDefaults` (cách chuẩn của SDK để đặt mặc định): `getBool` của
/// SDK trả `false` cho khoá không tồn tại, và ta muốn mặc định nằm cạnh kiểu
/// và khoảng hợp lệ trong [FlagDef], test được mà không cần plugin.
/// Chỉ lấy giá trị có nguồn `valueRemote`.
final class FirebaseRemoteFlagSource implements RemoteFlagSource {
  FirebaseRemoteFlagSource(this._config);

  final FirebaseRemoteConfig _config;

  /// Gọi một lần sau `Firebase.initializeApp()`.
  Future<void> configure({
    Duration fetchTimeout = const Duration(seconds: 10),
    Duration minimumFetchInterval = const Duration(hours: 1),
  }) {
    return _config.setConfigSettings(
      RemoteConfigSettings(
        fetchTimeout: fetchTimeout,
        minimumFetchInterval: minimumFetchInterval,
      ),
    );
  }

  @override
  Future<Map<String, Object?>> fetch() async {
    await _config.fetchAndActivate();
    return {
      for (final entry in _config.getAll().entries)
        if (entry.value.source == ValueSource.valueRemote)
          entry.key: entry.value.asString(),
    };
  }
}

import 'package:flutter/material.dart' show ThemeMode;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'auth/token_store.dart';
import 'db/app_database.dart';
import 'preferences/app_preferences.dart';
import 'trip_history_repository.dart';

/// Một `AppDatabase` cho cả app; đóng khi `ProviderScope` bị huỷ.
final appDatabaseProvider = Provider<AppDatabase>((ref) {
  final database = AppDatabase();
  ref.onDispose(database.close);
  return database;
});

final tripHistoryRepositoryProvider = Provider<TripHistoryRepository>(
  (ref) => DriftTripHistoryRepository(ref.watch(appDatabaseProvider).tripDao),
);

/// Lịch sử gần nhất. `StreamProvider` biến stream query của drift thành
/// `AsyncValue`: mỗi lần bảng đổi, màn hình tự cập nhật.
final recentTripsProvider = StreamProvider.autoDispose<List<Trip>>(
  (ref) => ref.watch(tripHistoryRepositoryProvider).watchRecent(),
);

final appPreferencesProvider = Provider<AppPreferences>(
  (ref) => AppPreferences(SharedPreferencesAsync()),
);

/// Chế độ sáng/tối: đọc từ `shared_preferences` một lần, ghi khi người dùng đổi.
class ThemeModeNotifier extends AsyncNotifier<ThemeMode> {
  @override
  Future<ThemeMode> build() =>
      ref.watch(appPreferencesProvider).readThemeMode();

  Future<void> select(ThemeMode mode) async {
    await ref.read(appPreferencesProvider).writeThemeMode(mode);
    // Provider có thể đã bị huỷ trong lúc chờ ghi.
    if (!ref.mounted) return;
    state = AsyncData(mode);
  }
}

final themeModeProvider = AsyncNotifierProvider<ThemeModeNotifier, ThemeMode>(
  ThemeModeNotifier.new,
);

final tokenStoreProvider = Provider<TokenStore>(
  (ref) => const SecureTokenStore(FlutterSecureStorage()),
);

/// Đã có token lưu hay chưa. Chỉ để màn hình cài đặt hiện trạng thái; không
/// bao giờ đưa giá trị token ra giao diện.
class SessionNotifier extends AsyncNotifier<bool> {
  @override
  Future<bool> build() async =>
      await ref.watch(tokenStoreProvider).read() != null;

  Future<void> signInDemo() async {
    await ref
        .read(tokenStoreProvider)
        .save(
          const AuthTokens(
            accessToken: 'demo-access-token',
            refreshToken: 'demo-refresh-token',
          ),
        );
    if (ref.mounted) state = const AsyncData(true);
  }

  Future<void> signOut() async {
    await ref.read(tokenStoreProvider).clear();
    if (ref.mounted) state = const AsyncData(false);
  }
}

final sessionProvider = AsyncNotifierProvider<SessionNotifier, bool>(
  SessionNotifier.new,
);

/// Đồng hồ tiêm được: test thay bằng giờ cố định.
final clockProvider = Provider<DateTime Function()>(
  (ref) =>
      () => DateTime.now().toUtc(),
);

import 'dart:math';

import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'auth/auth_api.dart';
import 'auth/auth_interceptor.dart';
import 'auth/session_manager.dart';
import 'auth/token_store.dart';

/// Địa chỉ fake API. Bài 25 thay chỗ này bằng cấu hình theo flavor.
///
/// Truyền lúc chạy: `--dart-define=API_BASE_URL=http://10.0.2.2:8787`
/// (Android emulator gọi máy dev qua 10.0.2.2, iOS Simulator dùng localhost).
final apiBaseUrlProvider = Provider<Uri>(
  (ref) => Uri.parse(
    const String.fromEnvironment(
      'API_BASE_URL',
      defaultValue: 'http://localhost:8787',
    ),
  ),
);

/// Nơi nhận dòng log của `SafeLogInterceptor`; test thay bằng `list.add`.
final logSinkProvider = Provider<void Function(String line)>(
  (ref) =>
      (line) => debugPrint(line),
);

/// Adapter gửi byte. `null` là adapter thật của dio; test truyền adapter giả.
final httpAdapterProvider = Provider<HttpClientAdapter?>((ref) => null);

/// Hàm chờ của backoff và nguồn jitter của `RetryInterceptor`; test thay.
final retrySleepProvider = Provider<Sleep>((ref) => Future<void>.delayed);
final retryRandomProvider = Provider<double Function()>(
  (ref) => Random().nextDouble,
);

/// Sinh `Idempotency-Key` cho một ý định đặt chuyến.
final idempotencyKeyProvider = Provider<String Function()>((ref) {
  final random = Random.secure();
  return () =>
      'book-${DateTime.now().toUtc().microsecondsSinceEpoch}-'
      '${random.nextInt(1 << 32).toRadixString(36)}';
});

/// Kho token: Keychain/Keystore qua `flutter_secure_storage` (bài 22).
final tokenStoreProvider = Provider<TokenStore>(
  (ref) => const SecureTokenStore(FlutterSecureStorage()),
);

/// `Dio` riêng cho `/auth/*`: không `AuthInterceptor`, không `RetryInterceptor`.
final authApiProvider = Provider<AuthApi>((ref) {
  final api = AuthApi.create(
    baseUrl: ref.watch(apiBaseUrlProvider),
    adapter: ref.watch(httpAdapterProvider),
    interceptors: [SafeLogInterceptor(ref.watch(logSinkProvider))],
  );
  ref.onDispose(api.dio.close);
  return api;
});

final sessionManagerProvider = Provider<SessionManager>((ref) {
  final manager = SessionManager(
    store: ref.watch(tokenStoreProvider),
    api: ref.watch(authApiProvider),
  );
  ref.onDispose(manager.dispose);
  return manager;
});

/// Client nghiệp vụ: log, `AuthInterceptor`, rồi `RetryInterceptor`.
///
/// `/quotes` là công khai nên không mang token. Cả `AuthInterceptor` và
/// `RetryInterceptor` cần chính `Dio` của client để phát lại request, vì vậy
/// được gắn sau khi tạo client.
final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final log = ref.watch(logSinkProvider);
  final client = ToiNoiApiClient(
    baseUrl: ref.watch(apiBaseUrlProvider),
    adapter: ref.watch(httpAdapterProvider),
    interceptors: [SafeLogInterceptor(log)],
  );
  client.dio.interceptors
    ..add(
      AuthInterceptor(
        dio: client.dio,
        session: ref.watch(sessionManagerProvider),
        publicPaths: const {'/quotes'},
      ),
    )
    ..add(
      RetryInterceptor(
        dio: client.dio,
        sleep: ref.watch(retrySleepProvider),
        random: ref.watch(retryRandomProvider),
      ),
    );
  ref.onDispose(client.close);
  return client;
});

/// Chuyến vừa đặt; `null` khi chưa đặt.
///
/// Mọi lỗi ra khỏi `guardApi` là một [ApiFailure]. Một request bị 401 do
/// access token hết hạn không hiện ra ở đây: `AuthInterceptor` refresh và phát
/// lại trước khi kết quả về tới controller.
class BookingController extends AsyncNotifier<Trip?> {
  String? _key;

  @override
  Trip? build() => null;

  Future<void> book() async {
    if (state.isLoading) return;
    // Cùng một khoá cho tới khi đặt thành công: bấm lại sau lỗi không tạo
    // chuyến thứ hai (bài 19).
    final key = _key ??= ref.read(idempotencyKeyProvider)();
    final client = ref.read(apiClientProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard(
      () => guardApi(() async {
        final dto = await client.createTrip(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
          idempotencyKey: key,
        );
        return dto.toDomain();
      }),
    );
    if (!ref.mounted) return;
    if (result is AsyncData<Trip?>) _key = null;
    state = result;
  }
}

final bookingControllerProvider =
    AsyncNotifierProvider<BookingController, Trip?>(BookingController.new);

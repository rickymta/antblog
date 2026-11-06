import 'dart:async';
import 'dart:convert';
import 'dart:io' show SocketException;
import 'dart:math';

import 'package:shelf/shelf.dart';
import 'package:toi_noi_24/auth/auth_api.dart';
import 'package:toi_noi_24/auth/auth_interceptor.dart';
import 'package:toi_noi_24/auth/session_manager.dart';
import 'package:toi_noi_24/auth/token_store.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final fixedNow = DateTime.utc(2026, 11, 6, 3, 30);

/// Số điện thoại giả, không phải số thật.
const testPhone = '+84000000001';

/// Chỗ của `RetryInterceptor` so với `AuthInterceptor` trong danh sách của client.
/// Log luôn đứng đầu.
enum RetryPlacement { none, afterAuth, beforeAuth }

/// Một "app" hoàn chỉnh dựng bằng tay (không Riverpod) trên fake API có xác
/// thực chạy trong tiến trình, để test từng lớp.
///
/// Đồng hồ của server là [now]: [advance] làm access token hết hạn mà không
/// chờ thật. Mọi request (cả `/auth/*` lẫn nghiệp vụ) đi qua cùng một
/// [adapter] nên [adapter.requests] cho thấy đúng thứ tự trên đường truyền.
final class Rig {
  Rig({
    Handler Function(Handler fake)? wrap,
    TokenStore? tokenStore,
    RetryPlacement retry = RetryPlacement.none,
    Set<String> publicPaths = const {'/quotes'},
    void Function(Object error)? onStoreError,
  }) : tokenStore = tokenStore ?? InMemoryTokenStore(),
       auth = AuthModule(random: Random(7)) {
    final fake = createFakeApiHandler(
      clock: () => now,
      newId: () => 'T-${++_trips}',
      store: store,
      modules: [auth],
      middleware: [auth.requireBearer],
    );
    adapter = InProcessAdapter(wrap == null ? fake : wrap(fake));
    final baseUrl = Uri.parse('http://fake.test');
    authApi = AuthApi.create(
      baseUrl: baseUrl,
      adapter: adapter,
      interceptors: [SafeLogInterceptor(logs.add)],
    );
    session = SessionManager(
      store: this.tokenStore,
      api: authApi,
      onStoreError: onStoreError,
    );
    session.signedOut.listen((_) => signedOutCount++);
    client = ToiNoiApiClient(
      baseUrl: baseUrl,
      adapter: adapter,
      interceptors: [SafeLogInterceptor(logs.add)],
    );
    final authInterceptor = AuthInterceptor(
      dio: client.dio,
      session: session,
      publicPaths: publicPaths,
    );
    final retryInterceptor = RetryInterceptor(
      dio: client.dio,
      sleep: (wait) async => waits.add(wait),
      random: () => 1.0,
    );
    client.dio.interceptors.addAll(switch (retry) {
      RetryPlacement.none => [authInterceptor],
      RetryPlacement.afterAuth => [authInterceptor, retryInterceptor],
      RetryPlacement.beforeAuth => [retryInterceptor, authInterceptor],
    });
  }

  DateTime now = fixedNow;
  int _trips = 0;
  int signedOutCount = 0;
  final FakeApiStore store = FakeApiStore();
  final TokenStore tokenStore;
  final AuthModule auth;
  final List<String> logs = [];
  final List<Duration> waits = [];
  late final InProcessAdapter adapter;
  late final AuthApi authApi;
  late final SessionManager session;
  late final ToiNoiApiClient client;

  /// Làm đồng hồ của server đi nhanh [duration].
  void advance(Duration duration) => now = now.add(duration);

  /// Làm access token vừa cấp hết hạn (hạn là 15 phút).
  void expireAccessTokens() => advance(const Duration(minutes: 16));

  /// Đăng nhập bằng OTP đúng, lưu phiên vào [session]; trả cặp token.
  Future<AuthTokens> signIn({String app = 'customer'}) async {
    final challenge = await authApi.requestOtp(phone: testPhone, app: app);
    final tokens = await authApi.verifyOtp(
      requestId: challenge.requestId,
      code: fakeOtpCode,
    );
    await session.signIn(tokens);
    return tokens;
  }

  /// Đặt chuyến 5 km (25.500 VND) với khoá [key], qua client có `AuthInterceptor`.
  Future<Trip> book(String key) async {
    final dto = await guardApi(
      () => client.createTrip(
        const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        idempotencyKey: key,
      ),
    );
    return dto.toDomain();
  }

  List<RecordedRequest> requestsTo(String path, {String? method}) => [
    for (final r in adapter.requests)
      if (r.uri.path == path && (method == null || r.method == method)) r,
  ];

  /// Số lời gọi `POST /auth/refresh` đã tới server.
  int get refreshCalls => requestsTo('/auth/refresh').length;

  /// Token trong header `Authorization` của từng request tới [path].
  List<String?> bearersTo(String path) => [
    for (final r in requestsTo(path))
      r.headers['authorization']?.replaceFirst('Bearer ', ''),
  ];
}

/// Giữ các request thoả [when] ở "server" cho tới khi đã có [count] request
/// cùng chờ, rồi thả tất cả: mô phỏng nhiều request cùng bay trên mạng chậm.
/// Request tới sau khi đã thả thì đi thẳng.
Handler holdUntil(
  Handler inner, {
  required int count,
  required bool Function(Request request) when,
}) {
  final released = Completer<void>();
  var arrived = 0;
  return (request) async {
    if (when(request) && !released.isCompleted) {
      if (++arrived >= count) released.complete();
      await released.future;
    }
    return inner(request);
  };
}

/// Response lỗi problem+json với mã [status].
Response problem(int status) => Response(
  status,
  body: jsonEncode({'title': 'injected', 'status': status}),
  headers: const {'content-type': 'application/problem+json'},
);

/// Handler ném `SocketException`: adapter đổi thành `DioException.connectionError`.
Never offline(Request request) =>
    throw const SocketException('mạng mất (giả lập)');

/// Cho vòng lặp sự kiện chạy tối đa [turns] lượt (mỗi lượt một timer 0 ms) và
/// báo [future] đã xong chưa. Dùng để chứng minh một lời gọi bị treo mà không
/// phải chờ hết giờ thật.
Future<bool> completesWithin(Future<Object?> future, {int turns = 200}) async {
  var done = false;
  unawaited(
    future.then(
      (_) => done = true,
      onError: (Object _, StackTrace _) => done = true,
    ),
  );
  for (var i = 0; i < turns && !done; i++) {
    await Future<void>.delayed(Duration.zero);
  }
  return done;
}

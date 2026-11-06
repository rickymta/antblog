/// Server giả của Tới Nơi: báo giá, đặt chuyến, nhận chuyến và lỗi có chủ đích.
///
/// Dùng trong test không cần socket:
///
/// ```dart
/// final handler = createFakeApiHandler();
/// final response = await handler(Request('GET', Uri.parse('http://x/health')));
/// ```
library;

export 'src/auth_module.dart' show AuthModule, fakeOtpCode;
export 'src/faults.dart' show FaultConfig;
export 'src/handler.dart' show createFakeApiHandler;
export 'src/module.dart' show FakeApiContext, FakeApiModule;
export 'src/problem.dart'
    show ProblemException, jsonResponse, problemResponse, traceIdContextKey;
export 'src/server_options.dart'
    show ServerOptions, parseServerOptions, serverUsage;
export 'src/store.dart' show Booking, FakeApiStore, StoredTrip, uuidGenerator;
export 'src/trips_module.dart'
    show fakeDriverIdHeader, idempotencyKeyHeader, tripToJson;

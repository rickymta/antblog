import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';

import 'faults.dart';
import 'module.dart';
import 'problem.dart';
import 'quotes_module.dart';
import 'store.dart';
import 'trips_module.dart';

/// Dựng handler của fake API. Không mở socket: `shelf_io.serve` ở
/// `bin/server.dart` bọc nó thành server, còn test gọi thẳng như một hàm
/// `Request -> Response`.
///
/// Thứ tự lớp: gắn mã theo dõi, bắt lỗi thành problem+json, tiêm lỗi/độ trễ,
/// các [middleware] thêm vào, rồi router.
Handler createFakeApiHandler({
  FaultConfig faults = const FaultConfig(),
  FakeApiStore? store,
  DateTime Function()? clock,
  String Function()? newId,
  Future<void> Function(Duration)? delay,
  List<Middleware> middleware = const [],
  List<FakeApiModule> modules = const [],
}) {
  final context = FakeApiContext(
    store: store ?? FakeApiStore(),
    clock: clock ?? () => DateTime.now().toUtc(),
    newId: newId ?? uuidGenerator(),
  );

  final router = Router(
    notFoundHandler: (Request request) =>
        problemResponse(request, 404, 'Route not found'),
  );
  router.get(
    '/health',
    (Request request) => jsonResponse(200, {'status': 'ok'}),
  );
  for (final module in <FakeApiModule>[
    const QuotesModule(),
    const TripsModule(),
    ...modules,
  ]) {
    module.mount(router, context);
  }

  var pipeline = const Pipeline()
      .addMiddleware(_traceMiddleware())
      .addMiddleware(_problemMiddleware())
      .addMiddleware(faultMiddleware(faults, delay ?? Future<void>.delayed));
  for (final extra in middleware) {
    pipeline = pipeline.addMiddleware(extra);
  }
  return pipeline.addHandler(router.call);
}

Middleware _traceMiddleware() {
  var counter = 0;
  return (Handler inner) => (Request request) {
    counter++;
    final traceId = 'fake-${counter.toString().padLeft(6, '0')}';
    return inner(request.change(context: {traceIdContextKey: traceId}));
  };
}

Middleware _problemMiddleware() {
  return (Handler inner) => (Request request) async {
    try {
      return await inner(request);
    } on ProblemException catch (e) {
      return problemResponse(request, e.status, e.title, errors: e.errors);
    } on Object {
      // Một lỗi lập trình trong fake không được làm sập server của lab.
      return problemResponse(request, 500, 'Unexpected server error');
    }
  };
}

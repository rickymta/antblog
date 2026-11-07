import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:toi_noi_39/checkout_launcher.dart';
import 'package:toi_noi_39/payment_flow.dart';
import 'package:toi_noi_39/payment_gateway.dart';
import 'package:toi_noi_39/payment_store.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'fake_payment_module.dart';

/// Mạng giả: tắt mạng, hoặc để server xử lý xong rồi làm rơi phản hồi.
final class ScriptedNetwork implements HttpClientAdapter {
  ScriptedNetwork(this.inner);

  final InProcessAdapter inner;

  /// Mất mạng hẳn: request không tới server.
  bool offline = false;

  /// Số phản hồi sắp tới bị rơi SAU KHI server đã xử lý request.
  int loseResponses = 0;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    if (offline) {
      throw DioException.connectionError(
        requestOptions: options,
        reason: 'offline',
      );
    }
    final response = await inner.fetch(options, requestStream, cancelFuture);
    if (loseResponses > 0) {
      loseResponses--;
      throw DioException.connectionError(
        requestOptions: options,
        reason: 'phản hồi bị rơi',
      );
    }
    return response;
  }

  @override
  void close({bool force = false}) => inner.close(force: force);
}

/// Launcher giả: ghi lại địa chỉ đã mở.
final class FakeLauncher implements CheckoutLauncher {
  final List<Uri> opened = [];
  bool result = true;
  int closed = 0;

  @override
  Future<void> close() async => closed++;

  @override
  Future<bool> open(Uri url) async {
    opened.add(url);
    return result;
  }
}

/// Fake API + module thanh toán + mạng giả + đồng hồ giả, và cách dựng
/// `PaymentFlow` trên đó. Nhiều flow từ cùng một harness dùng chung server,
/// kho và bộ sinh khoá, nên hai flow mô phỏng hai lần mở app.
final class PaymentHarness {
  PaymentHarness({this._poll = PollPolicy.standard}) {
    var next = 0;
    final handler = createFakeApiHandler(
      store: api,
      clock: () => now,
      newId: () => 'id-${++next}',
      modules: [server],
    );
    network = ScriptedNetwork(InProcessAdapter(handler));
    dio = Dio(BaseOptions(baseUrl: 'http://fake.test'))
      ..httpClientAdapter = network;
  }

  final PollPolicy _poll;
  final FakeApiStore api = FakeApiStore();
  final FakePaymentModule server = FakePaymentModule();
  late final ScriptedNetwork network;
  late final Dio dio;
  final InMemoryPaymentStore store = InMemoryPaymentStore();
  final FakeLauncher launcher = FakeLauncher();

  /// Đồng hồ giả của server và của app.
  DateTime now = DateTime.utc(2026, 11, 6, 9);

  /// Các khoảng chờ giữa hai lần hỏi; `delay` không chờ thật mà đẩy đồng hồ.
  final List<Duration> waited = [];
  int _keyCounter = 0;

  /// Dựng một chuyến 5 km (25.500 VND) trong server giả và trả `Trip`.
  Trip addTrip({double km = 5, String id = 'trip-1'}) {
    final trip = Trip.request(
      id: id,
      serviceKind: ServiceKind.ride,
      distanceKm: km,
      requestedAt: now,
    );
    api.putTrip(StoredTrip(trip, 0));
    return trip;
  }

  PaymentFlow flow({PaymentStore? withStore}) => PaymentFlow(
    gateway: HttpPaymentGateway(dio),
    launcher: launcher,
    store: withStore ?? store,
    newKey: () => 'key-${++_keyCounter}',
    now: () => now,
    poll: _poll,
    delay: (d) async {
      waited.add(d);
      now = now.add(d);
    },
  );

  /// Số request GET trạng thái mà server đã nhận.
  int get statusRequests => (network.inner.requests)
      .where((r) => r.method == 'GET' && r.uri.path.startsWith('/payments/'))
      .length;
}

/// `toinoi://payment/return?session=<id>&status=<claimed>`
Uri returnLink(String sessionId, {String? status}) => Uri(
  scheme: 'toinoi',
  host: 'payment',
  path: '/return',
  queryParameters: {'session': sessionId, 'status': ?status},
);

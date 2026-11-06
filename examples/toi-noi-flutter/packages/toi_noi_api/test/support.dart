import 'dart:convert';

import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final fixedNow = DateTime.utc(2026, 11, 6, 3, 30);

/// Client nối thẳng vào fake API chạy trong tiến trình: không socket.
final class Lab {
  Lab._(this.store, this.adapter, this.client);

  factory Lab({
    FaultConfig faults = const FaultConfig(),
    Iterable<Interceptor> interceptors = const [],
    Duration receiveTimeout = const Duration(seconds: 15),
  }) {
    final store = FakeApiStore();
    var seq = 0;
    final adapter = InProcessAdapter(
      createFakeApiHandler(
        faults: faults,
        store: store,
        clock: () => fixedNow,
        newId: () => 'T-${++seq}',
      ),
    );
    return Lab._(
      store,
      adapter,
      ToiNoiApiClient(
        baseUrl: Uri.parse('http://fake.test'),
        adapter: adapter,
        interceptors: interceptors,
        receiveTimeout: receiveTimeout,
      ),
    );
  }

  final FakeApiStore store;
  final InProcessAdapter adapter;
  final ToiNoiApiClient client;

  /// Client của tài xế [driverId]: cùng server, nhưng gắn header danh tính
  /// giả `x-fake-driver-id` (chỗ của claim `sub` trong JWT, bài 24).
  ToiNoiApiClient driver(String driverId) => driverClient(adapter, driverId);
}

/// Client của tài xế [driverId] nối vào [adapter].
ToiNoiApiClient driverClient(InProcessAdapter adapter, String driverId) =>
    ToiNoiApiClient(
      baseUrl: Uri.parse('http://fake.test'),
      adapter: adapter,
      interceptors: [
        InterceptorsWrapper(
          onRequest: (options, handler) {
            options.headers[fakeDriverIdHeader] = driverId;
            handler.next(options);
          },
        ),
      ],
    );

/// Client nối vào một [Handler] tự viết, cho các ca server "hư" mà fake API
/// không tạo ra (HTML, JSON sai hợp đồng, không bao giờ trả lời).
({ToiNoiApiClient client, InProcessAdapter adapter}) clientFor(
  Handler handler, {
  String baseUrl = 'http://stub.test',
  Duration receiveTimeout = const Duration(seconds: 15),
  Iterable<Interceptor> interceptors = const [],
}) {
  final adapter = InProcessAdapter(handler);
  return (
    client: ToiNoiApiClient(
      baseUrl: Uri.parse(baseUrl),
      adapter: adapter,
      receiveTimeout: receiveTimeout,
      interceptors: interceptors,
    ),
    adapter: adapter,
  );
}

/// Response JSON thành công với [body].
Response okJson(Object body, {int status = 200}) => Response(
  status,
  body: jsonEncode(body),
  headers: const {'content-type': 'application/json; charset=utf-8'},
);

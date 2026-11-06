import 'dart:convert';

import 'package:dio/dio.dart' hide Response;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:shelf/shelf.dart';
import 'package:toi_noi_19/providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final fixedNow = DateTime.utc(2026, 11, 6, 3, 30);

/// Một "máy chủ" cho test: fake API (hoặc handler tự viết) chạy trong tiến
/// trình, kèm các dòng log đã ghi và bộ sinh `Idempotency-Key` đếm tăng dần.
final class Harness {
  Harness._(this.store, this.adapter, this.logs, this._keyCount);

  /// Dùng `createFakeApiHandler` với đồng hồ và mã chuyến cố định (`T-1`,
  /// `T-2`...). [wrap] cho phép bọc handler để mô phỏng lỗi mạng.
  factory Harness({
    FaultConfig faults = const FaultConfig(),
    Handler Function(Handler fake)? wrap,
  }) {
    final store = FakeApiStore();
    var seq = 0;
    final fake = createFakeApiHandler(
      faults: faults,
      store: store,
      clock: () => fixedNow,
      newId: () => 'T-${++seq}',
    );
    return Harness._(
      store,
      InProcessAdapter(wrap == null ? fake : wrap(fake)),
      <String>[],
      _Counter(),
    );
  }

  /// Dùng một [handler] tự viết thay cho fake API.
  factory Harness.custom(Handler handler) => Harness._(
    FakeApiStore(),
    InProcessAdapter(handler),
    <String>[],
    _Counter(),
  );

  final FakeApiStore store;
  final InProcessAdapter adapter;
  final List<String> logs;
  final _Counter _keyCount;

  /// Số lần bộ sinh khoá đã được gọi.
  int get keysIssued => _keyCount.value;

  List<Override> get overrides => [
    httpAdapterProvider.overrideWithValue(adapter),
    logSinkProvider.overrideWithValue(logs.add),
    idempotencyKeyProvider.overrideWithValue(() => 'book-${++_keyCount.value}'),
  ];

  /// Client của tài xế [driverId]: cùng server, thêm header danh tính giả
  /// `x-fake-driver-id` bằng interceptor (bài 24 thay bằng JWT).
  ToiNoiApiClient driver(String driverId) => ToiNoiApiClient(
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

  ProviderContainer container() => ProviderContainer.test(overrides: overrides);
}

final class _Counter {
  int value = 0;
}

/// Response JSON thành công với [body].
Response okJson(Object body, {int status = 200}) => Response(
  status,
  body: jsonEncode(body),
  headers: const {'content-type': 'application/json; charset=utf-8'},
);

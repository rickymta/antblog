import 'dart:async';

import 'package:dio/dio.dart' show Interceptor;
import 'package:fake_async/fake_async.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

/// Client + fake API + `RetryInterceptor`, mọi thứ chạy trong tiến trình.
///
/// Mặc định khoảng chờ của backoff không chờ thật: [sleep] ghi lại vào
/// [waits] rồi trả ngay. Truyền `realSleep: true` để dùng `Future.delayed`
/// (kết hợp với `fakeAsync` khi muốn thấy thời gian trôi trên đồng hồ giả).
final class RetryLab {
  RetryLab._(
    this.store,
    this.inProcess,
    this.faulty,
    this.client,
    this.retry,
    this.waits,
    this.logs,
  );

  factory RetryLab({
    FaultConfig faults = const FaultConfig(),
    Iterable<NetworkFault?> script = const [],
    int maxAttempts = 3,
    double random = 1.0,
    Duration connectTimeout = const Duration(seconds: 10),
    Duration receiveTimeout = const Duration(seconds: 15),
    bool realSleep = false,
    Set<String> readOnlyPostPaths = const {'/quotes'},
    Duration? maxElapsed,
    Iterable<Interceptor> after = const [],
    void Function(RetryEvent event)? onRetry,
  }) {
    final store = FakeApiStore();
    var seq = 0;
    final inProcess = InProcessAdapter(
      createFakeApiHandler(
        faults: faults,
        store: store,
        clock: () => fixedNow,
        newId: () => 'T-${++seq}',
      ),
    );
    final faulty = FaultyAdapter(inProcess, script: script);
    final logs = <String>[];
    final client = ToiNoiApiClient(
      baseUrl: Uri.parse('http://fake.test'),
      adapter: faulty,
      connectTimeout: connectTimeout,
      receiveTimeout: receiveTimeout,
      interceptors: [SafeLogInterceptor(logs.add)],
    );
    final waits = <Duration>[];
    final retry = RetryInterceptor(
      dio: client.dio,
      maxAttempts: maxAttempts,
      maxElapsed: maxElapsed,
      readOnlyPostPaths: readOnlyPostPaths,
      random: () => random,
      sleep: realSleep ? null : (d) async => waits.add(d),
      onRetry: (event) {
        if (realSleep) {
          waits.add(event.wait);
        }
        onRetry?.call(event);
      },
    );
    client.dio.interceptors
      ..add(retry)
      ..addAll(after);
    return RetryLab._(store, inProcess, faulty, client, retry, waits, logs);
  }

  final FakeApiStore store;

  /// Chỉ ghi những request thật sự tới "server" (lỗi `refuse`, `hang` không tới).
  final InProcessAdapter inProcess;

  /// Mọi lần gọi tầng mạng, kể cả lần bị tiêm lỗi.
  final FaultyAdapter faulty;
  final ToiNoiApiClient client;
  final RetryInterceptor retry;

  /// Các khoảng chờ backoff đã dùng.
  final List<Duration> waits;
  final List<String> logs;
}

const ride5km = QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5);

/// Chạy [body] trên đồng hồ giả, tiến từng [step] cho tới khi xong hoặc hết [limit].
({T? value, Object? error, Duration elapsed, bool done}) runFake<T>(
  Future<T> Function() body, {
  Duration step = const Duration(milliseconds: 100),
  Duration limit = const Duration(minutes: 10),
}) {
  T? value;
  Object? error;
  var done = false;
  late Duration elapsed;
  fakeAsync((async) {
    unawaited(
      body().then<void>(
        (v) {
          value = v;
          done = true;
        },
        onError: (Object e) {
          error = e;
          done = true;
        },
      ),
    );
    while (!done && async.elapsed < limit) {
      async.elapse(step);
    }
    elapsed = async.elapsed;
  });
  return (value: value, error: error, elapsed: elapsed, done: done);
}

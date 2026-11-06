import 'dart:async';
import 'dart:convert';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:shelf/shelf.dart';
import 'package:toi_noi_20/connectivity.dart';
import 'package:toi_noi_20/providers.dart';
import 'package:toi_noi_20/system_settings.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final fixedNow = DateTime.utc(2026, 11, 6, 3, 30);

/// `ConnectivityMonitor` giả: test đặt tình trạng và phát thay đổi.
final class FakeConnectivity implements ConnectivityMonitor {
  FakeConnectivity([this.link = LinkType.wifi]);

  LinkType link;
  final _changes = StreamController<LinkType>.broadcast();

  @override
  Future<LinkType> current() async => link;

  @override
  Stream<LinkType> get changes => _changes.stream;

  /// Còn ai đang nghe [changes] không.
  bool get hasListeners => _changes.hasListener;

  void emit(LinkType next) {
    link = next;
    _changes.add(next);
  }
}

/// `SystemSettings` giả: đếm số lần được mở.
final class FakeSettings implements SystemSettings {
  int opened = 0;

  @override
  String get buttonLabel => 'Mở cài đặt';

  @override
  Future<void> openNetworkSettings() async => opened++;
}

/// Một "máy chủ" cho test: fake API (hoặc handler tự viết) chạy trong tiến
/// trình, bọc bởi [FaultyAdapter] để tiêm lỗi tầng mạng theo kịch bản.
///
/// Khoảng chờ của backoff không chờ thật: ghi vào [waits] rồi trả ngay. Với
/// `realSleep: true` nó vẫn ghi lại nhưng chờ bằng `Future.delayed`, dùng
/// cùng đồng hồ giả của `testWidgets` hoặc [runFake].
final class Harness {
  Harness._(
    this.store,
    this.inProcess,
    this.faulty,
    this.connectivity,
    this.settings,
    this.config,
    this.realSleep,
  );

  factory Harness({
    FaultConfig faults = const FaultConfig(),
    Iterable<NetworkFault?> script = const [],
    Handler Function(Handler fake)? wrap,
    LinkType link = LinkType.wifi,
    NetworkConfig config = const NetworkConfig(),
    bool realSleep = false,
  }) {
    final store = FakeApiStore();
    var seq = 0;
    final fake = createFakeApiHandler(
      faults: faults,
      store: store,
      clock: () => fixedNow,
      newId: () => 'T-${++seq}',
    );
    final inProcess = InProcessAdapter(wrap == null ? fake : wrap(fake));
    return Harness._(
      store,
      inProcess,
      FaultyAdapter(inProcess, script: script),
      FakeConnectivity(link),
      FakeSettings(),
      config,
      realSleep,
    );
  }

  /// Dùng một [handler] tự viết thay cho fake API.
  factory Harness.custom(
    Handler handler, {
    Iterable<NetworkFault?> script = const [],
    LinkType link = LinkType.wifi,
  }) {
    final inProcess = InProcessAdapter(handler);
    return Harness._(
      FakeApiStore(),
      inProcess,
      FaultyAdapter(inProcess, script: script),
      FakeConnectivity(link),
      FakeSettings(),
      const NetworkConfig(),
      false,
    );
  }

  final FakeApiStore store;

  /// Chỉ ghi những request thật sự tới "server".
  final InProcessAdapter inProcess;

  /// Mọi lần gọi tầng mạng, kể cả lần bị tiêm lỗi.
  final FaultyAdapter faulty;
  final FakeConnectivity connectivity;
  final FakeSettings settings;
  final NetworkConfig config;
  final bool realSleep;

  final List<String> logs = [];

  /// Các khoảng chờ backoff đã dùng.
  final List<Duration> waits = [];

  int _keys = 0;

  /// Số lần bộ sinh khoá đã được gọi.
  int get keysIssued => _keys;

  List<Override> get overrides => [
    httpAdapterProvider.overrideWithValue(faulty),
    logSinkProvider.overrideWithValue(logs.add),
    idempotencyKeyProvider.overrideWithValue(() => 'book-${++_keys}'),
    networkConfigProvider.overrideWithValue(config),
    // random = 1.0: khoảng chờ chạm trần, dễ tính tay.
    retryRandomProvider.overrideWithValue(() => 1.0),
    retrySleepProvider.overrideWithValue((wait) {
      waits.add(wait);
      return realSleep ? Future<void>.delayed(wait) : Future<void>.value();
    }),
    connectivityMonitorProvider.overrideWithValue(connectivity),
    systemSettingsProvider.overrideWithValue(settings),
  ];

  /// Container giống `ProviderScope` của `main`: không thử lại ở tầng provider.
  ProviderContainer container() =>
      ProviderContainer.test(overrides: overrides, retry: noProviderRetry);
}

/// Response JSON thành công với [body].
Response okJson(Object body, {int status = 200}) => Response(
  status,
  body: jsonEncode(body),
  headers: const {'content-type': 'application/json; charset=utf-8'},
);

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

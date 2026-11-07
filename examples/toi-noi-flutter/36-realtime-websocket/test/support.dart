import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:fake_async/fake_async.dart';
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_36/realtime_transport.dart';
import 'package:toi_noi_36/backoff.dart';

typedef Json = Map<String, Object?>;

/// Random cho Backoff: `nextDouble()` luôn trả [value], nên
/// `delay = trần * value` và test đọc được con số chính xác.
final class FixedRandom implements Random {
  FixedRandom(this.value);

  final double value;

  @override
  double nextDouble() => value;

  @override
  int nextInt(int max) => 0;

  @override
  bool nextBool() => false;
}

/// Backoff của test: base 1 s, trần 30 s, `delay = trần / 2`.
Backoff halfBackoff({
  Duration base = const Duration(seconds: 1),
  Duration cap = const Duration(seconds: 30),
}) => Backoff(base: base, cap: cap, random: FixedRandom(0.5));

/// Đầu bên kia của một kết nối giả: test đóng vai gateway.
final class FakeConnection implements RealtimeConnection {
  final StreamController<String> _incoming = StreamController<String>();

  /// Mọi frame client đã gửi, đã giải JSON.
  final List<Json> sent = [];

  /// Mã đóng do client yêu cầu (chỉ khi client gọi `close`).
  int? clientCloseCode;
  bool closedByClient = false;
  int? _closeCode;

  @override
  Stream<String> get incoming => _incoming.stream;

  @override
  int? get closeCode => _closeCode;

  @override
  void send(String text) {
    if (closedByClient || _incoming.isClosed) {
      throw StateError('Kết nối đã đóng');
    }
    sent.add(jsonDecode(text) as Json);
  }

  @override
  Future<void> close([int? code, String? reason]) async {
    // Không đóng `_incoming`: với một gateway đã chết, bắt tay đóng không bao
    // giờ hoàn tất, và socket "thây ma" vẫn có thể nhả frame muộn.
    closedByClient = true;
    clientCloseCode = code;
  }

  /// Client còn đang nghe kết nối này không.
  bool get hasListener => _incoming.hasListener;

  /// Các frame loại [type] client đã gửi.
  List<Json> sentOfType(String type) => [
    for (final frame in sent)
      if (frame['type'] == type) frame,
  ];

  // Phía gateway.

  void serverSend(Json frame) => _incoming.add(jsonEncode(frame));

  void serverSendRaw(String text) => _incoming.add(text);

  void serverWelcome() =>
      serverSend({'type': 'welcome', 'connectionId': 'c-1', 'userId': 'U-1'});

  /// Gateway đóng có mã (ví dụ 4401, 4001).
  void serverClose(int code) {
    _closeCode = code;
    unawaited(_incoming.close());
  }

  /// Mất kết nối đột ngột: không có mã đóng.
  void serverDrop() => unawaited(_incoming.close());

  /// Lỗi socket rồi đóng.
  void serverError() {
    _incoming.addError(const SocketException('Connection reset by peer'));
    unawaited(_incoming.close());
  }
}

/// Transport giả: mỗi `connect` tạo một [FakeConnection].
final class FakeTransport implements RealtimeTransport {
  /// Các kết nối đã tạo, theo thứ tự.
  final List<FakeConnection> connections = [];

  /// Token của từng lần `connect`.
  final List<String> tokens = [];

  /// Lần `connect` kế tiếp ném lỗi chừng này lần.
  int failures = 0;

  /// Lỗi ném ra khi còn `failures` (mặc định là mất kết nối).
  Object failWith = const SocketException('Connection refused');

  /// Khi khác null, `connect` đợi nó hoàn tất (giả lập bắt tay chậm).
  Completer<void>? gate;

  /// Tự gửi `welcome` ngay sau khi nối được.
  bool autoWelcome = true;

  int get attempts => tokens.length;

  FakeConnection get last => connections.last;

  @override
  Future<RealtimeConnection> connect(Uri url, {required String token}) async {
    tokens.add(token);
    if (failures > 0) {
      failures--;
      throw failWith;
    }
    final pending = gate;
    if (pending != null) await pending.future;
    final connection = FakeConnection();
    connections.add(connection);
    if (autoWelcome) scheduleMicrotask(connection.serverWelcome);
    return connection;
  }
}

/// Frame `event` đúng hợp đồng.
Json eventFrame(
  String tripId,
  int sequence, {
  String status = 'searching',
  Json event = const {'type': 'searchStarted'},
  int? version,
}) => {
  'type': 'event',
  'tripId': tripId,
  'sequence': sequence,
  'version': version ?? sequence,
  'occurredAt': '2026-11-06T03:30:00.000Z',
  'status': status,
  'event': event,
};

/// Frame `subscribed` đúng hợp đồng.
Json subscribedFrame(
  String tripId, {
  int latest = 0,
  int replayed = 0,
  String? logEpoch = 'epoch-1',
}) => {
  'type': 'subscribed',
  'tripId': tripId,
  'latestSequence': latest,
  'replayed': replayed,
  'logEpoch': ?logEpoch,
};

/// Frame `offer` đúng hợp đồng.
Json offerFrame(String offerId, {String tripId = 'T-1'}) => {
  'type': 'offer',
  'offerId': offerId,
  'tripId': tripId,
  'serviceKind': 'ride',
  'distanceMeters': 5000,
  'fare': 25500,
  'currency': 'VND',
  'expiresAt': '2026-11-06T03:30:20.000Z',
};

/// Frame `location` đúng hợp đồng; [second] là giây của `at`.
Json locationFrame(String tripId, double lat, {int second = 0}) => {
  'type': 'location',
  'tripId': tripId,
  'driverId': 'D-7',
  'lat': lat,
  'lng': 106.7,
  'at': DateTime.utc(2026, 11, 6, 3, 30, second).toIso8601String(),
};

/// Gom mọi giá trị của một stream broadcast vào danh sách.
List<T> collect<T>(Stream<T> stream) {
  final values = <T>[];
  stream.listen(values.add);
  return values;
}

/// Cho mọi microtask và Timer 0 giây chạy.
void settle(FakeAsync async) => async.elapse(Duration.zero);

/// Dựng client với transport giả, token cố định và các hằng thời gian của lab.
RealtimeClient buildClient(
  FakeTransport transport, {
  Backoff? backoff,
  Future<String?> Function()? tokenProvider,
  Duration heartbeatInterval = const Duration(seconds: 15),
  Duration heartbeatTimeout = const Duration(seconds: 5),
  Duration handshakeTimeout = const Duration(seconds: 10),
  Duration stableAfter = const Duration(seconds: 30),
  Duration locationInterval = const Duration(seconds: 1),
}) => RealtimeClient(
  transport: transport,
  url: Uri.parse('ws://gateway.test/realtime'),
  tokenProvider: tokenProvider ?? () async => 'token-1',
  backoff: backoff ?? halfBackoff(),
  heartbeatInterval: heartbeatInterval,
  heartbeatTimeout: heartbeatTimeout,
  handshakeTimeout: handshakeTimeout,
  stableAfter: stableAfter,
  locationInterval: locationInterval,
);

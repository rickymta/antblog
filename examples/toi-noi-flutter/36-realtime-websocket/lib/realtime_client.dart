import 'dart:async';

import 'backoff.dart';
import 'realtime_protocol.dart';
import 'realtime_transport.dart';

/// Bốn trạng thái của kết nối.
enum RealtimeStatus {
  /// Đang mở kết nối lần đầu (từ lúc gọi `connect`).
  connecting,

  /// Gateway đã xác nhận bằng `welcome`.
  connected,

  /// Mất kết nối (hoặc lần mở đầu thất bại), đang chờ hoặc đang thử lại.
  reconnecting,

  /// Không mở và không tự mở lại; xem [RealtimeState.cause].
  closed,
}

/// Vì sao kết nối ở trạng thái `closed`.
enum CloseCause {
  /// Người dùng của client gọi `close` hoặc `dispose`.
  client,

  /// App đi nền: `pause` đã ngắt, `resume` sẽ nối lại.
  background,

  /// Token bị gateway từ chối (mã 4401) hoặc không có token. Nối lại cũng sẽ
  /// bị từ chối, nên phải đăng nhập hoặc làm mới token rồi gọi `connect`.
  unauthorized,
}

/// Trạng thái kết nối tại một thời điểm.
final class RealtimeState {
  const RealtimeState(
    this.status, {
    this.attempt = 0,
    this.nextRetryIn,
    this.cause,
  });

  /// Chưa `connect` lần nào.
  static const initial = RealtimeState(RealtimeStatus.closed);

  final RealtimeStatus status;

  /// Số lần nối lại đã lên lịch trong đợt mất kết nối này (chỉ khi `reconnecting`).
  final int attempt;

  /// Thời gian chờ đã chọn cho lần thử kế tiếp; `null` khi đang thử.
  final Duration? nextRetryIn;

  /// Chỉ có khi `closed` sau một lần ngắt; `null` ở [initial].
  final CloseCause? cause;

  @override
  bool operator ==(Object other) =>
      other is RealtimeState &&
      other.status == status &&
      other.attempt == attempt &&
      other.nextRetryIn == nextRetryIn &&
      other.cause == cause;

  @override
  int get hashCode => Object.hash(status, attempt, nextRetryIn, cause);

  @override
  String toString() =>
      'RealtimeState(${status.name}, attempt: $attempt, '
      'nextRetryIn: $nextRetryIn, cause: ${cause?.name})';
}

/// Điều bất thường client thấy nhưng không làm rớt kết nối: `error` của
/// gateway, hoặc `malformed_frame`, `sequence_reset` do chính client phát hiện.
final class RealtimeIssue {
  const RealtimeIssue(this.code, this.message, {this.tripId});

  final String code;
  final String message;
  final String? tripId;

  @override
  String toString() => 'RealtimeIssue($code, $tripId, $message)';
}

final class _Live {
  _Live(this.connection);

  final RealtimeConnection connection;
  late final StreamSubscription<String> subscription;
}

final class _LocationSlot {
  Timer? timer;
  DriverLocation? pending;
}

/// Trạng thái nghe một chuyến: `sequence` cuối đã đưa ra và log mà nó thuộc về.
final class _Watch {
  _Watch(this.last, this.logEpoch);

  int last;
  String? logEpoch;
}

/// Client Realtime Gateway của Tới Nơi.
///
/// - Giữ một kết nối, tự nối lại bằng [Backoff] (full jitter) khi mất.
/// - Phát hiện kết nối chết bằng heartbeat ở tầng ứng dụng.
/// - Nhớ `sequence` cuối của từng chuyến đang nghe; sau khi nối lại, `subscribe`
///   lại từ đó, và khử trùng theo `sequence` nên mỗi sự kiện bền ra đúng một lần.
/// - Gộp vị trí tài xế: giữ bản mới nhất, tối đa một bản mỗi [locationInterval].
///
/// [events] là stream một người nghe và **đệm** sự kiện cho tới khi có người
/// `listen`: sự kiện đã tính vào `sequence` thì không bao giờ mất. Cần nhiều
/// người nghe thì dùng `events.asBroadcastStream()`. [locations] và [offers] là
/// broadcast: phát khi chưa có người nghe thì mất, vì chúng là dữ liệu tạm thời.
final class RealtimeClient {
  RealtimeClient({
    required this.transport,
    required this.url,
    required this.tokenProvider,
    Backoff? backoff,
    this.heartbeatInterval = const Duration(seconds: 15),
    this.heartbeatTimeout = const Duration(seconds: 5),
    this.handshakeTimeout = const Duration(seconds: 10),
    this.stableAfter = const Duration(seconds: 30),
    this.locationInterval = const Duration(seconds: 1),
  }) : backoff = backoff ?? Backoff();

  final RealtimeTransport transport;
  final Uri url;

  /// Trả access token **còn hạn**, hoặc `null` khi chưa đăng nhập. Gọi lại ở
  /// **mỗi** lần mở kết nối (kể cả khi `resume`), nên đây là chỗ làm mới token
  /// hết hạn. Trả token cũ thì gateway từ chối (4401 hoặc HTTP 401) và client
  /// dừng ở `closed(unauthorized)`; `resume()` sẽ không mở lại, phải `connect()`
  /// sau khi đăng nhập lại. Ném lỗi thì coi như lỗi tạm thời và thử lại. Token
  /// chỉ được kiểm lúc bắt tay: kết nối đang mở vẫn sống cho tới khi gateway
  /// tự đóng nó bằng 4401 lúc token hết hạn.
  final Future<String?> Function() tokenProvider;
  final Backoff backoff;

  /// Cứ chừng này gửi một `ping` khi đang `connected`. Số minh hoạ.
  final Duration heartbeatInterval;

  /// Sau `ping` (hoặc sau khi mở), im lặng quá chừng này thì coi kết nối đã chết.
  final Duration heartbeatTimeout;

  /// Chờ `welcome` tối đa chừng này sau khi bắt tay WebSocket xong.
  final Duration handshakeTimeout;

  /// Kết nối phải sống liên tục chừng này thì [Backoff] mới quay về lần đầu.
  final Duration stableAfter;

  /// Khoảng tối thiểu giữa hai vị trí đưa ra [locations] cho cùng một chuyến.
  final Duration locationInterval;

  final _states = StreamController<RealtimeState>.broadcast();
  final _events = StreamController<TripEventEnvelope>();
  final _locations = StreamController<DriverLocation>.broadcast();
  final _offers = StreamController<DriverOffer>.broadcast();
  final _issues = StreamController<RealtimeIssue>.broadcast();

  RealtimeState _state = RealtimeState.initial;
  _Live? _live;
  int _generation = 0;
  int _attempt = 0;
  int _pingSeq = 0;
  int _duplicates = 0;
  bool _disposed = false;

  Timer? _retryTimer;
  Timer? _handshakeTimer;
  Timer? _heartbeatTimer;
  Timer? _pongDeadline;
  Timer? _stableTimer;

  /// Chuyến đang nghe -> `sequence` cuối đã đưa ra [events] và log của nó.
  final Map<String, _Watch> _watching = {};
  final Map<String, int> _resyncing = {};
  final Set<String> _resyncAcked = {};
  bool _offersWanted = false;
  final Map<String, DateTime> _lastLocationAt = {};
  final Map<String, _LocationSlot> _slots = {};

  RealtimeState get state => _state;

  /// Mỗi lần [state] đổi. Không phát lại trạng thái hiện tại cho người nghe mới.
  Stream<RealtimeState> get states => _states.stream;

  /// Sự kiện bền của chuyến, theo thứ tự `sequence`, không trùng, không lỗ hổng.
  /// Một người nghe; sự kiện được đệm tới khi có người `listen`.
  Stream<TripEventEnvelope> get events => _events.stream;

  /// Lời mời cuốc gửi riêng cho tài xế đang đăng nhập (sau [subscribeOffers]).
  Stream<DriverOffer> get offers => _offers.stream;

  /// Vị trí tài xế đã gộp (bản mới nhất, không cũ hơn bản đã đưa ra).
  Stream<DriverLocation> get locations => _locations.stream;

  Stream<RealtimeIssue> get issues => _issues.stream;

  /// Số `event` bị bỏ vì `sequence` đã nhận rồi (để chẩn đoán và test).
  int get duplicatesDropped => _duplicates;

  /// `sequence` cuối đã nhận của [tripId], hoặc `null` nếu không đang nghe.
  int? lastSequence(String tripId) => _watching[tripId]?.last;

  /// `logEpoch` của log mà [lastSequence] thuộc về, khi đã biết.
  String? logEpoch(String tripId) => _watching[tripId]?.logEpoch;

  /// Bắt đầu mở kết nối. Không làm gì nếu đã mở hoặc đang mở.
  void connect() {
    _ensureAlive();
    if (_state.status != RealtimeStatus.closed) return;
    _generation++;
    _attempt = 0;
    unawaited(_open(first: true));
  }

  /// Đóng kết nối và không tự mở lại. `subscribe` đã đăng ký vẫn được nhớ.
  Future<void> close() {
    _ensureAlive();
    if (_state.status == RealtimeStatus.closed &&
        _state.cause == CloseCause.client) {
      return Future<void>.value();
    }
    return _stop(CloseCause.client);
  }

  /// App đi nền: ngắt kết nối nhưng nhớ để [resume] nối lại.
  void pause() {
    _ensureAlive();
    if (_state.status == RealtimeStatus.closed) return;
    unawaited(_stop(CloseCause.background));
  }

  /// App trở lại: nối lại nếu kết nối đang đóng vì [pause].
  void resume() {
    _ensureAlive();
    if (_state.status == RealtimeStatus.closed &&
        _state.cause == CloseCause.background) {
      connect();
    }
  }

  /// Nghe chuyến [tripId]. [lastSequence] và [logEpoch] là điểm app đã xử lý
  /// tới (ví dụ `lastSequence` và `logEpoch` của một bản chụp REST, hoặc đọc từ
  /// database cục bộ); mặc định là nhận từ sự kiện đầu tiên.
  ///
  /// Gọi lại cho chuyến đang nghe không làm gì. Chưa `connected` thì chỉ ghi
  /// nhớ; yêu cầu được gửi ngay khi gateway chào (`welcome`). Nếu gateway báo
  /// log đã đổi (`log_reset`) hoặc không phát lại được (`replay_unavailable`),
  /// client **thôi nghe** chuyến đó và báo qua [issues]; app tải lại bản chụp
  /// rồi `subscribe` lại.
  void subscribe(String tripId, {int lastSequence = 0, String? logEpoch}) {
    _ensureAlive();
    if (_watching.containsKey(tripId)) return;
    _watching[tripId] = _Watch(lastSequence, logEpoch);
    final live = _live;
    if (_state.status == RealtimeStatus.connected && live != null) {
      _send(live, encodeSubscribe(tripId, lastSequence, logEpoch: logEpoch));
    }
  }

  /// Thôi nghe [tripId] và quên `sequence` của nó: `subscribe` lại sau đó là
  /// một lượt nghe mới, bắt đầu từ `lastSequence` của lần gọi đó.
  void unsubscribe(String tripId) {
    _ensureAlive();
    if (_watching.remove(tripId) == null) return;
    _forget(tripId);
    final live = _live;
    if (_state.status == RealtimeStatus.connected && live != null) {
      _send(live, encodeUnsubscribe(tripId));
    }
  }

  /// Nhận lời mời cuốc gửi riêng cho tài xế (chỉ vai trò tài xế). Được đăng ký
  /// lại sau mỗi lần nối lại.
  void subscribeOffers() {
    _ensureAlive();
    if (_offersWanted) return;
    _offersWanted = true;
    final live = _live;
    if (_state.status == RealtimeStatus.connected && live != null) {
      _send(live, encodeSubscribeOffers());
    }
  }

  void unsubscribeOffers() {
    _ensureAlive();
    if (!_offersWanted) return;
    _offersWanted = false;
    final live = _live;
    if (_state.status == RealtimeStatus.connected && live != null) {
      _send(live, encodeUnsubscribeOffers());
    }
  }

  /// Giải phóng mọi thứ: đóng kết nối, huỷ Timer, đóng các stream. Gọi
  /// `connect` hay `subscribe` sau đó (kể cả ngay sau `dispose()` chưa `await`)
  /// là lỗi lập trình.
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true; // trước mọi await: chặn connect() chen vào giữa
    await _stop(CloseCause.client);
    _watching.clear();
    _resyncing.clear();
    _resyncAcked.clear();
    for (final slot in _slots.values) {
      slot.timer?.cancel();
    }
    _slots.clear();
    // `events` là stream một người nghe: close() chỉ xong khi có người nghe
    // rút hết đệm, nên không chờ nó.
    unawaited(_events.close());
    await Future.wait([
      _states.close(),
      _locations.close(),
      _offers.close(),
      _issues.close(),
    ]);
  }

  void _ensureAlive() {
    if (_disposed) throw StateError('RealtimeClient đã dispose');
  }

  void _emit(RealtimeState next) {
    if (next == _state) return;
    _state = next;
    if (!_states.isClosed) _states.add(next);
  }

  void _issue(RealtimeIssue issue) {
    if (!_issues.isClosed) _issues.add(issue);
  }

  // Mở kết nối ---------------------------------------------------------------

  Future<void> _open({required bool first}) async {
    final generation = _generation;
    _emit(
      first
          ? const RealtimeState(RealtimeStatus.connecting)
          : RealtimeState(RealtimeStatus.reconnecting, attempt: _attempt),
    );

    final String? token;
    try {
      token = await tokenProvider();
    } on Object {
      if (generation == _generation) _scheduleRetry();
      return;
    }
    if (generation != _generation) return;
    if (token == null) {
      unawaited(_stop(CloseCause.unauthorized));
      return;
    }

    final RealtimeConnection connection;
    try {
      connection = await transport.connect(url, token: token);
    } on RealtimeAuthRejected {
      // HTTP 401 ở bước nâng cấp: cùng một token thì lần nào cũng bị từ chối.
      if (generation == _generation) unawaited(_stop(CloseCause.unauthorized));
      return;
    } on Object {
      if (generation == _generation) _scheduleRetry();
      return;
    }
    if (generation != _generation) {
      // Bị huỷ (close, pause, dispose) trong lúc đang bắt tay: đừng để rò.
      unawaited(_closeQuietly(connection));
      return;
    }
    _attach(connection);
  }

  void _attach(RealtimeConnection connection) {
    final live = _Live(connection);
    _live = live;
    live.subscription = connection.incoming.listen(
      (text) => _onText(live, text),
      onError: (Object _) => _lost(live),
      onDone: () => _lost(live),
      cancelOnError: true,
    );
    _handshakeTimer = Timer(handshakeTimeout, () => _lost(live, dead: true));
  }

  void _scheduleRetry() {
    final delay = backoff.delay(_attempt);
    _attempt++;
    _emit(
      RealtimeState(
        RealtimeStatus.reconnecting,
        attempt: _attempt,
        nextRetryIn: delay,
      ),
    );
    _retryTimer = Timer(delay, () => unawaited(_open(first: false)));
  }

  // Mất kết nối --------------------------------------------------------------

  void _lost(_Live live, {bool dead = false}) {
    if (_live != live) return;
    _live = null;
    _cancelConnectionTimers();
    unawaited(live.subscription.cancel());
    if (dead) unawaited(_closeQuietly(live.connection));
    if (!dead && live.connection.closeCode == closeCodeUnauthorized) {
      unawaited(_stop(CloseCause.unauthorized));
      return;
    }
    _scheduleRetry();
  }

  Future<void> _stop(CloseCause cause) {
    _generation++;
    _cancelConnectionTimers();
    _retryTimer?.cancel();
    _retryTimer = null;
    for (final tripId in _slots.keys.toList()) {
      _dropSlot(tripId);
    }
    _resyncing.clear();
    _resyncAcked.clear();
    final live = _live;
    _live = null;
    _emit(RealtimeState(RealtimeStatus.closed, cause: cause));
    if (live == null) return Future<void>.value();
    unawaited(live.subscription.cancel());
    return _closeQuietly(live.connection, 1000, 'client close');
  }

  Future<void> _closeQuietly(
    RealtimeConnection connection, [
    int? code,
    String? reason,
  ]) async {
    try {
      await connection.close(code, reason);
    } on Object {
      // Đang đóng một kết nối hỏng: không còn gì để báo.
    }
  }

  void _cancelConnectionTimers() {
    _handshakeTimer?.cancel();
    _heartbeatTimer?.cancel();
    _pongDeadline?.cancel();
    _stableTimer?.cancel();
    _handshakeTimer = null;
    _heartbeatTimer = null;
    _pongDeadline = null;
    _stableTimer = null;
  }

  bool _send(_Live live, String text) {
    try {
      live.connection.send(text);
      return true;
    } on Object {
      _lost(live);
      return false;
    }
  }

  // Nhận frame ---------------------------------------------------------------

  void _onText(_Live live, String text) {
    if (_live != live) return;
    // Bất kỳ frame nào cũng chứng tỏ kết nối còn sống, không chỉ `pong`.
    _pongDeadline?.cancel();
    _pongDeadline = null;

    final ServerFrame frame;
    try {
      frame = parseServerFrame(text);
    } on FormatException catch (e) {
      _issue(RealtimeIssue('malformed_frame', e.message));
      return;
    }
    switch (frame) {
      case WelcomeFrame():
        _onWelcome(live);
      case SubscribedFrame(
        :final tripId,
        :final latestSequence,
        :final logEpoch,
      ):
        _onSubscribed(live, tripId, latestSequence, logEpoch);
      case EventFrame(:final envelope):
        _onEvent(live, envelope);
      case LocationFrame(:final location):
        _onLocation(location);
      case ErrorFrame():
        _onServerError(frame);
      case OfferFrame(:final offer):
        if (_offersWanted && !_offers.isClosed) _offers.add(offer);
      case OffersSubscribedFrame() ||
          UnsubscribedFrame() ||
          PongFrame() ||
          UnknownFrame():
        break;
    }
  }

  void _onWelcome(_Live live) {
    if (_state.status == RealtimeStatus.connected) return;
    _handshakeTimer?.cancel();
    _handshakeTimer = null;
    _resyncing.clear();
    _emit(const RealtimeState(RealtimeStatus.connected));
    // Chỉ khi sống đủ lâu mới coi là ổn định và xoá lịch sử thất bại.
    _stableTimer = Timer(stableAfter, () => _attempt = 0);
    _heartbeatTimer = Timer.periodic(heartbeatInterval, (_) {
      if (_live != live) return;
      if (!_send(live, encodePing('p-${++_pingSeq}'))) return;
      _pongDeadline ??= Timer(heartbeatTimeout, () => _lost(live, dead: true));
    });
    for (final entry in _watching.entries.toList()) {
      final watch = entry.value;
      final frame = encodeSubscribe(
        entry.key,
        watch.last,
        logEpoch: watch.logEpoch,
      );
      if (!_send(live, frame)) return;
    }
    if (_offersWanted && !_send(live, encodeSubscribeOffers())) return;
  }

  /// Gateway chào lại một chuyến: kiểm log có còn là log mà `sequence` của ta
  /// thuộc về không.
  void _onSubscribed(
    _Live live,
    String tripId,
    int latestSequence,
    String? logEpoch,
  ) {
    final watch = _watching[tripId];
    if (watch == null) return;
    final knownEpoch = watch.logEpoch;
    if ((knownEpoch != null && logEpoch != null && knownEpoch != logEpoch) ||
        latestSequence < watch.last) {
      // Gateway đã mất log (khởi động lại): mọi `sequence` mới thuộc về một
      // log khác, đưa ra hay bỏ đều sai. Thôi nghe và để app tải lại bản chụp.
      _issue(
        RealtimeIssue(
          'log_reset',
          'Log của gateway không còn là log mà client đã nhận tới #${watch.last}',
          tripId: tripId,
        ),
      );
      _watching.remove(tripId);
      _forget(tripId);
      _send(live, encodeUnsubscribe(tripId));
      return;
    }
    watch.logEpoch ??= logEpoch;
    if (_resyncing.containsKey(tripId)) _resyncAcked.add(tripId);
  }

  void _forget(String tripId) {
    _resyncing.remove(tripId);
    _resyncAcked.remove(tripId);
    _lastLocationAt.remove(tripId);
    _dropSlot(tripId);
  }

  void _onEvent(_Live live, TripEventEnvelope envelope) {
    final tripId = envelope.tripId;
    final watch = _watching[tripId];
    if (watch == null) return; // frame muộn của chuyến đã thôi nghe
    final last = watch.last;
    if (envelope.sequence <= last) {
      _duplicates++;
      return;
    }
    if (envelope.sequence > last + 1) {
      // Thiếu sự kiện giữa chừng: không đưa ra, xin lại từ `last`.
      if (_resyncAcked.contains(tripId)) {
        // Gateway đã trả lời lời xin lại mà vẫn không lấp được lỗ hổng.
        _issue(
          RealtimeIssue(
            'replay_unavailable',
            'Gateway không phát lại được các sự kiện sau #$last',
            tripId: tripId,
          ),
        );
        _watching.remove(tripId);
        _forget(tripId);
        _send(live, encodeUnsubscribe(tripId));
        return;
      }
      // Hỏi một lần cho mỗi mốc để frame tới dồn dập không sinh bão subscribe.
      if (_resyncing[tripId] != last) {
        _resyncing[tripId] = last;
        _send(live, encodeSubscribe(tripId, last, logEpoch: watch.logEpoch));
      }
      return;
    }
    watch.last = envelope.sequence;
    _resyncing.remove(tripId);
    _resyncAcked.remove(tripId);
    // `events` đệm khi chưa có người nghe nên không có sự kiện nào bị mất.
    if (!_events.isClosed) _events.add(envelope);
  }

  void _onServerError(ErrorFrame frame) {
    final tripId = frame.tripId;
    _issue(RealtimeIssue(frame.code, frame.message, tripId: tripId));
    if (tripId != null &&
        const {
          'trip_not_found',
          'forbidden',
          'log_reset',
          'replay_unavailable',
        }.contains(frame.code)) {
      // Lỗi vĩnh viễn của chuyến này: đừng xin lại mỗi lần nối lại. Với
      // `log_reset` và `replay_unavailable`, app tải lại bản chụp rồi subscribe.
      _watching.remove(tripId);
      _forget(tripId);
    }
    if (frame.channel == 'offers' && frame.code == 'forbidden') {
      _offersWanted = false;
    }
  }

  // Gộp vị trí ---------------------------------------------------------------

  void _onLocation(DriverLocation location) {
    final tripId = location.tripId;
    if (!_watching.containsKey(tripId)) return;
    final slot = _slots.putIfAbsent(tripId, _LocationSlot.new);
    // Chỉ so với bản đã đưa ra hoặc đang chờ; bản bị huỷ lúc đi nền hay
    // unsubscribe không để lại mốc nào.
    final newest = slot.pending?.at ?? _lastLocationAt[tripId];
    if (newest != null && !location.at.isAfter(newest)) return; // cũ hơn: bỏ

    if (locationInterval == Duration.zero) {
      _emitLocation(location);
      return;
    }
    if (slot.timer == null) {
      // Cửa sổ đang rảnh: đưa ra ngay, rồi đóng cửa sổ.
      _emitLocation(location);
      slot.timer = Timer(locationInterval, () => _flushLocation(tripId));
    } else {
      slot.pending = location; // thay bản đang chờ bằng bản mới hơn
    }
  }

  void _emitLocation(DriverLocation location) {
    _lastLocationAt[location.tripId] = location.at;
    if (!_locations.isClosed) _locations.add(location);
  }

  void _flushLocation(String tripId) {
    final slot = _slots[tripId];
    if (slot == null) return;
    final pending = slot.pending;
    slot.pending = null;
    if (pending == null) {
      slot.timer = null;
      return;
    }
    _emitLocation(pending);
    slot.timer = Timer(locationInterval, () => _flushLocation(tripId));
  }

  void _dropSlot(String tripId) {
    _slots.remove(tripId)?.timer?.cancel();
  }
}

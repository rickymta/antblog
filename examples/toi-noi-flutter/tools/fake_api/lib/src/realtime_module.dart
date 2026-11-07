import 'dart:async';
import 'dart:convert';

import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:shelf_web_socket/shelf_web_socket.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import 'input.dart';
import 'module.dart';
import 'problem.dart';
import 'store.dart';

/// Đường dẫn WebSocket của fake Realtime Gateway.
const realtimePath = '/realtime';

/// Mã đóng kết nối khi token thiếu, sai hoặc hết hạn (lúc bắt tay, hoặc khi
/// kết nối đang mở mà token đã quá hạn).
///
/// Nằm trong vùng 4000 đến 4999 mà RFC 6455 dành cho ứng dụng. Qua
/// `web_socket_channel` (qua package `web_socket` bên dưới) chỉ gửi được mã
/// 1000 hoặc 3000 đến 4999.
const realtimeCloseUnauthorized = 4401;

/// Mã đóng khi gateway chủ động ngắt mọi kết nối (khởi động lại, triển khai).
/// Client coi đây là mất kết nối bình thường và nối lại.
const realtimeCloseRestart = 4001;

/// Người đã xác thực của một kết nối. [expiresAt] là hạn của access token;
/// gateway đóng kết nối bằng mã [realtimeCloseUnauthorized] khi tới hạn.
typedef RealtimePrincipal = ({String userId, String role, DateTime? expiresAt});

/// Đổi access token thành người dùng, hoặc `null` khi token không dùng được.
typedef RealtimeTokenVerifier = RealtimePrincipal? Function(String token);

/// Quyết định [who] có được nghe kênh của [trip] không.
typedef RealtimeSubscribeAuthorizer = bool Function(
  RealtimePrincipal who,
  Trip trip,
);

/// Vị trí tài xế do bên phát (simulator) gửi vào gateway.
final class DriverLocationUpdate {
  const DriverLocationUpdate({
    required this.driverId,
    required this.lat,
    required this.lng,
    this.headingDegrees,
    this.speedMetersPerSecond,
  });

  final String driverId;
  final double lat;
  final double lng;
  final double? headingDegrees;
  final double? speedMetersPerSecond;
}

/// Một sự kiện bền đã được ghi vào log của chuyến.
final class DurableTripEvent {
  const DurableTripEvent({
    required this.sequence,
    required this.version,
    required this.event,
    required this.status,
    required this.occurredAt,
  });

  /// Số thứ tự trong log của chuyến, bắt đầu từ 1, tăng đúng 1 mỗi sự kiện.
  final int sequence;

  /// `version` của chuyến trong kho sau sự kiện này (cùng thước đo với
  /// `version` của REST `GET /trips/{id}`).
  final int version;
  final TripEvent event;

  /// Trạng thái của chuyến sau khi áp dụng [event].
  final TripStatus status;
  final DateTime occurredAt;
}

/// Lời mời một cuốc đã phát cho tài xế.
final class PublishedOffer {
  const PublishedOffer({
    required this.offerId,
    required this.delivered,
    required this.expiresAt,
  });

  final String offerId;

  /// Số kết nối của tài xế đã nhận.
  final int delivered;
  final DateTime expiresAt;
}

final class _Connection {
  _Connection(this.id, this.channel, this.principal);

  final String id;
  final WebSocketChannel channel;
  final RealtimePrincipal principal;
  final Set<String> trips = {};
  bool offers = false;
  bool closed = false;
  Timer? expiry;

  void send(Map<String, Object?> frame) {
    if (closed) return;
    try {
      channel.sink.add(jsonEncode(frame));
    } on Object {
      closed = true;
    }
  }
}

final class _TripChannel {
  /// Số sự kiện đã bị cắt khỏi đầu log (xem `maxLogPerTrip`).
  int trimmed = 0;
  final List<({DurableTripEvent event, Map<String, Object?> frame})> log = [];
  Map<String, Object?>? lastLocation;
  final Set<_Connection> subscribers = {};

  int get latest => trimmed + log.length;
}

/// Realtime Gateway giả: một WebSocket `GET /realtime` cho app khách và app
/// tài xế theo dõi chuyến và nhận lời mời cuốc.
///
/// **Chỉ dành cho lab.** Hợp đồng JSON đầy đủ nằm trong README của package.
/// Tóm tắt:
///
/// - Bắt tay: header `Authorization: Bearer <access token>`. Mặc định server
///   luôn nâng cấp lên WebSocket rồi mới kiểm; token xấu bị đóng bằng mã
///   [realtimeCloseUnauthorized]. Với `rejectBeforeUpgrade: true` server trả
///   HTTP 401 ngay ở bước nâng cấp (nhiều proxy và gateway thật làm vậy); client
///   `dart:io` thấy `WebSocketException.httpStatusCode == 401`. Hai đường đều
///   phải được client xử lý như lỗi xác thực.
/// - Client gửi `subscribe` (theo chuyến, hoặc `channel: offers`),
///   `unsubscribe`, `ping`.
/// - Server gửi `welcome`, `subscribed`, `event` (bền, có `sequence`,
///   `version`), `location` (tạm thời), `offer` (tạm thời, riêng từng tài xế),
///   `pong`, `error`.
/// - `subscribe` theo chuyến mang `lastSequence` và `logEpoch` mà client đã
///   thấy. Log nằm trong bộ nhớ nên gateway khởi động lại là sang một
///   `logEpoch` mới: gateway từ chối phát lại (`log_reset`) thay vì phát nhầm
///   `sequence` của log khác.
///
/// Đơn giản hoá so với gateway thật (xem hồ sơ kiến trúc, mục Realtime):
/// gateway thật nhận sự kiện từ Kafka hoặc pub/sub nội bộ và có thể là nhiều
/// node, nên "phát lại rồi đăng ký trong một lượt đồng bộ" ở đây chỉ đúng với
/// một tiến trình; quyền nghe mặc định là mọi người dùng đã xác thực.
final class RealtimeModule implements FakeApiModule {
  RealtimeModule({
    required this.verifyToken,
    this.authorize,
    String? logEpoch,
    this.maxLogPerTrip,
    this.rejectBeforeUpgrade = false,
    this.offerTtl = const Duration(seconds: 15),
  }) : assert(maxLogPerTrip == null || maxLogPerTrip > 0),
       _logEpoch = logEpoch ?? _newEpoch();

  /// Kiểm access token của bắt tay, thường là `AuthModule.verifyAccessToken`.
  final RealtimeTokenVerifier verifyToken;

  /// Quyền nghe kênh chuyến. Mặc định mọi người dùng đã xác thực nghe được mọi
  /// chuyến có thật: đơn giản hoá của lab. Gateway thật phải kiểm khách hoặc
  /// tài xế có thuộc chuyến này không (hồ sơ kiến trúc: kiểm quyền subscribe ở
  /// server). Kênh `offers` luôn là của chính người dùng trong token và chỉ cho
  /// vai trò `driver`.
  final RealtimeSubscribeAuthorizer? authorize;

  String _logEpoch;

  /// Định danh của log hiện tại. Đổi mỗi lần tiến trình khởi động, hoặc khi
  /// gọi [restart].
  String get logEpoch => _logEpoch;

  /// Giữ tối đa chừng này sự kiện cho mỗi chuyến; sự kiện cũ hơn bị cắt và
  /// `subscribe` xin phần đã cắt nhận `replay_unavailable`. `null` là giữ hết.
  final int? maxLogPerTrip;

  /// Trả HTTP 401 ở bước nâng cấp khi token xấu, thay vì nâng cấp rồi đóng 4401.
  final bool rejectBeforeUpgrade;

  /// Thời hạn mặc định của lời mời cuốc.
  final Duration offerTtl;

  final Map<String, _TripChannel> _channels = {};
  final Set<_Connection> _connections = {};
  FakeApiContext? _context;
  int _connectionSeq = 0;
  int _offerSeq = 0;

  /// Số kết nối đang mở (để test xem trạng thái phía server).
  int get connectionCount => _connections.length;

  /// Tổng số kết nối đã được chấp nhận từ lúc tạo module (kể cả đã đóng).
  int get connectionsOpened => _connectionSeq;

  /// Số kết nối đang nghe chuyến [tripId].
  int subscriberCount(String tripId) =>
      _channels[tripId]?.subscribers.length ?? 0;

  FakeApiContext get _ctx =>
      _context ?? (throw StateError('RealtimeModule chưa được mount'));

  @override
  void mount(Router router, FakeApiContext context) {
    _context = context;
    context.tripJsonExtras.add(
      (StoredTrip stored) => <String, Object?>{
        'lastSequence': latestSequence(stored.trip.id),
        'logEpoch': logEpoch,
      },
    );
    router.get(realtimePath, _upgrade);
    router.post('/realtime/trips/<id>/events', _postEvent);
    router.post('/realtime/trips/<id>/location', _postLocation);
    router.post('/realtime/drivers/<id>/offers', _postOffer);
  }

  /// Ghi một sự kiện bền vào chuyến [tripId], cập nhật chuyến trong kho (tăng
  /// `version` thêm 1) và phát cho người đang nghe.
  ///
  /// Ném [ProblemException] 404 khi chuyến không có trong kho, 409 khi cặp
  /// (trạng thái, sự kiện) không hợp lệ theo `Trip.apply` của `toi_noi_core`.
  DurableTripEvent publish(String tripId, TripEvent event) {
    final stored = _ctx.store.trip(tripId);
    if (stored == null) throw const ProblemException(404, 'Trip not found');
    final Trip next;
    try {
      next = stored.trip.apply(event);
    } on StateError {
      throw const ProblemException(409, 'Request conflicts with current state');
    }
    final version = stored.version + 1;
    _ctx.store.putTrip(StoredTrip(next, version));

    final channel = _channelFor(tripId);
    final durable = DurableTripEvent(
      sequence: channel.latest + 1,
      version: version,
      event: event,
      status: next.status,
      occurredAt: _ctx.clock(),
    );
    final frame = _eventFrame(tripId, durable);
    channel.log.add((event: durable, frame: frame));
    final limit = maxLogPerTrip;
    if (limit != null && channel.log.length > limit) {
      final excess = channel.log.length - limit;
      channel.log.removeRange(0, excess);
      channel.trimmed += excess;
    }
    for (final subscriber in channel.subscribers) {
      subscriber.send(frame);
    }
    return durable;
  }

  /// Phát vị trí tài xế cho người đang nghe chuyến [tripId] và nhớ bản mới
  /// nhất để gửi cho người nghe đến sau. Trả số kết nối đã nhận.
  int publishLocation(String tripId, DriverLocationUpdate update) {
    final channel = _channelFor(tripId);
    final frame = <String, Object?>{
      'type': 'location',
      'tripId': tripId,
      'driverId': update.driverId,
      'lat': update.lat,
      'lng': update.lng,
      'headingDegrees': ?update.headingDegrees,
      'speedMetersPerSecond': ?update.speedMetersPerSecond,
      'at': _ctx.clock().toUtc().toIso8601String(),
    };
    channel.lastLocation = frame;
    for (final subscriber in channel.subscribers) {
      subscriber.send(frame);
    }
    return channel.subscribers.length;
  }

  /// Phát lời mời cuốc [tripId] cho tài xế [userId] (mã người dùng trong
  /// token). Chỉ kết nối của đúng người đó đã `subscribe` kênh `offers` nhận
  /// được; lời mời không được lưu hay phát lại.
  ///
  /// Ném [ProblemException] 404 khi chuyến không có, 409 khi chuyến không còn
  /// ở `requested` hoặc `searching`.
  PublishedOffer publishOffer(String userId, String tripId, {Duration? ttl}) {
    final stored = _ctx.store.trip(tripId);
    if (stored == null) throw const ProblemException(404, 'Trip not found');
    final trip = stored.trip;
    if (trip.status != TripStatus.requested &&
        trip.status != TripStatus.searching) {
      throw const ProblemException(409, 'Request conflicts with current state');
    }
    final offerId = 'O-${(++_offerSeq).toString().padLeft(4, '0')}';
    final expiresAt = _ctx.clock().add(ttl ?? offerTtl);
    final frame = <String, Object?>{
      'type': 'offer',
      'offerId': offerId,
      'tripId': tripId,
      'serviceKind': trip.serviceKind.name,
      'distanceMeters': toMeters(trip.distanceKm),
      'fare': trip.fare.vnd,
      'currency': 'VND',
      'expiresAt': expiresAt.toUtc().toIso8601String(),
    };
    var delivered = 0;
    for (final connection in _connections) {
      if (connection.offers && connection.principal.userId == userId) {
        connection.send(frame);
        delivered++;
      }
    }
    return PublishedOffer(
      offerId: offerId,
      delivered: delivered,
      expiresAt: expiresAt,
    );
  }

  /// Đóng mọi kết nối như khi gateway khởi động lại, để thử reconnect.
  ///
  /// Log sự kiện và vị trí cuối của từng chuyến được giữ nguyên (cùng
  /// `logEpoch`): đây là "mất kết nối", không phải "mất log".
  Future<void> dropConnections({
    int code = realtimeCloseRestart,
    String reason = 'server restart',
  }) async {
    final open = _connections.toList();
    _connections.clear();
    for (final connection in open) {
      _detach(connection);
    }
    await Future.wait([
      for (final connection in open)
        connection.channel.sink.close(code, reason),
    ]);
  }

  /// Đóng (mã [realtimeCloseUnauthorized]) mọi kết nối mà token đã quá hạn
  /// theo đồng hồ của fake API. Mỗi kết nối cũng có một Timer tự làm việc này
  /// đúng hạn. Trả số kết nối đã đóng.
  Future<int> closeExpired() async {
    final now = _ctx.clock();
    final expired = [
      for (final connection in _connections)
        if (connection.principal.expiresAt case final at?
            when !now.isBefore(at))
          connection,
    ];
    for (final connection in expired) {
      _closeUnauthorized(connection);
    }
    return expired.length;
  }

  /// Giả lập gateway khởi động lại và **mất log**: xoá log sự kiện và vị trí
  /// cuối của mọi chuyến, sang một `logEpoch` mới, rồi đóng mọi kết nối (mã
  /// [realtimeCloseRestart]). Chuyến trong kho (trạng thái, `version`) giữ
  /// nguyên; `sequence` của sự kiện kế tiếp bắt đầu lại từ 1.
  Future<void> restart({String? logEpoch}) async {
    _logEpoch = logEpoch ?? _newEpoch();
    _channels.clear();
    await dropConnections();
  }

  /// Số sự kiện bền đã ghi cho [tripId] (0 khi chưa có kênh).
  int latestSequence(String tripId) => _channels[tripId]?.latest ?? 0;

  _TripChannel _channelFor(String tripId) {
    if (_ctx.store.trip(tripId) == null) {
      throw const ProblemException(404, 'Trip not found');
    }
    return _channels.putIfAbsent(tripId, _TripChannel.new);
  }

  FutureOr<Response> _upgrade(Request request) {
    final header = request.headers['authorization'] ?? '';
    final token = RegExp(
      r'^Bearer (\S+)$',
      caseSensitive: false,
    ).firstMatch(header)?.group(1);
    final principal = token == null ? null : verifyToken(token);
    if (principal == null && rejectBeforeUpgrade) {
      return problemResponse(request, 401, 'Unauthorized').change(
        headers: const {'www-authenticate': 'Bearer error="invalid_token"'},
      );
    }
    return webSocketHandler((WebSocketChannel channel, String? _) {
      _onConnection(channel, principal);
    })(request);
  }

  void _onConnection(WebSocketChannel channel, RealtimePrincipal? principal) {
    if (principal == null) {
      unawaited(channel.sink.close(realtimeCloseUnauthorized, 'unauthorized'));
      return;
    }
    final connection = _Connection(
      'c-${(++_connectionSeq).toString().padLeft(4, '0')}',
      channel,
      principal,
    );
    _connections.add(connection);
    final expiresAt = principal.expiresAt;
    if (expiresAt != null) {
      final wait = expiresAt.difference(_ctx.clock());
      connection.expiry = Timer(
        wait.isNegative ? Duration.zero : wait,
        () => _closeUnauthorized(connection),
      );
    }
    connection.send({
      'type': 'welcome',
      'connectionId': connection.id,
      'userId': principal.userId,
    });
    channel.stream.listen(
      (Object? data) => _onFrame(connection, data),
      onDone: () => _drop(connection),
      onError: (Object _) => _drop(connection),
      cancelOnError: true,
    );
  }

  void _closeUnauthorized(_Connection connection) {
    if (connection.closed) return;
    _drop(connection);
    unawaited(
      connection.channel.sink.close(realtimeCloseUnauthorized, 'token expired'),
    );
  }

  void _drop(_Connection connection) {
    _connections.remove(connection);
    _detach(connection);
  }

  void _detach(_Connection connection) {
    connection.closed = true;
    connection.expiry?.cancel();
    for (final tripId in connection.trips) {
      _channels[tripId]?.subscribers.remove(connection);
    }
    connection.trips.clear();
    connection.offers = false;
  }

  void _onFrame(_Connection connection, Object? data) {
    final Object? decoded;
    try {
      decoded = data is String ? jsonDecode(data) : null;
    } on FormatException {
      return _error(connection, 'bad_frame', 'Expected a JSON object');
    }
    if (decoded is! Map<String, Object?>) {
      return _error(connection, 'bad_frame', 'Expected a JSON object');
    }
    switch (decoded['type']) {
      case 'ping':
        connection.send({'type': 'pong', 'id': decoded['id']});
      case 'subscribe':
        if (decoded['channel'] == 'offers') {
          _subscribeOffers(connection);
        } else {
          _subscribe(connection, decoded);
        }
      case 'unsubscribe':
        if (decoded['channel'] == 'offers') {
          connection.offers = false;
          connection.send({'type': 'unsubscribed', 'channel': 'offers'});
          return;
        }
        final tripId = decoded['tripId'];
        if (tripId is! String || tripId.isEmpty) {
          return _error(connection, 'bad_frame', 'tripId must be a string');
        }
        _leave(connection, tripId);
        connection.send({'type': 'unsubscribed', 'tripId': tripId});
      default:
        _error(connection, 'unknown_type', 'Unknown frame type');
    }
  }

  void _leave(_Connection connection, String tripId) {
    if (connection.trips.remove(tripId)) {
      _channels[tripId]?.subscribers.remove(connection);
    }
  }

  void _subscribeOffers(_Connection connection) {
    if (connection.principal.role != 'driver') {
      return _error(
        connection,
        'forbidden',
        'Only drivers have offers',
        channel: 'offers',
      );
    }
    connection.offers = true;
    connection.send({'type': 'subscribed', 'channel': 'offers'});
  }

  void _subscribe(_Connection connection, Map<String, Object?> frame) {
    final tripId = frame['tripId'];
    if (tripId is! String || tripId.isEmpty) {
      return _error(connection, 'bad_frame', 'tripId must be a string');
    }
    final last = frame['lastSequence'] ?? 0;
    if (last is! int || last < 0) {
      return _error(
        connection,
        'bad_frame',
        'lastSequence must be a nonnegative integer',
        tripId: tripId,
      );
    }
    final clientEpoch = frame['logEpoch'];
    if (clientEpoch != null && clientEpoch is! String) {
      return _error(
        connection,
        'bad_frame',
        'logEpoch must be a string',
        tripId: tripId,
      );
    }
    final stored = _ctx.store.trip(tripId);
    if (stored == null) {
      return _error(
        connection,
        'trip_not_found',
        'Trip not found',
        tripId: tripId,
      );
    }
    final authorizer = authorize;
    if (authorizer != null && !authorizer(connection.principal, stored.trip)) {
      return _error(connection, 'forbidden', 'Not allowed', tripId: tripId);
    }

    final channel = _channelFor(tripId);
    if ((clientEpoch != null && clientEpoch != logEpoch) ||
        last > channel.latest) {
      // Client đang nói về một log khác (gateway đã khởi động lại): phát lại
      // theo `last` của log cũ sẽ đưa nhầm sự kiện của log mới.
      _leave(connection, tripId);
      return _error(
        connection,
        'log_reset',
        'The client sequence belongs to another log',
        tripId: tripId,
      );
    }
    if (last < channel.trimmed) {
      _leave(connection, tripId);
      return _error(
        connection,
        'replay_unavailable',
        'Events after $last were trimmed from the log',
        tripId: tripId,
      );
    }

    channel.subscribers.add(connection);
    connection.trips.add(tripId);
    // Phát lại và đăng ký chạy trong một lượt đồng bộ (đúng với một tiến
    // trình): sự kiện mới không thể chen vào giữa các sự kiện được phát lại.
    final replay = [
      for (final entry in channel.log)
        if (entry.event.sequence > last) entry.frame,
    ];
    connection.send({
      'type': 'subscribed',
      'tripId': tripId,
      'latestSequence': channel.latest,
      'replayed': replay.length,
      'logEpoch': logEpoch,
    });
    replay.forEach(connection.send);
    final location = channel.lastLocation;
    if (location != null) connection.send(location);
  }

  void _error(
    _Connection connection,
    String code,
    String message, {
    String? tripId,
    String? channel,
  }) {
    connection.send({
      'type': 'error',
      'code': code,
      'message': message,
      'tripId': ?tripId,
      'channel': ?channel,
    });
  }

  Future<Response> _postEvent(Request request) async {
    final tripId = request.params['id']!;
    final event = _readEvent(await readJsonObject(request));
    final durable = publish(tripId, event);
    return jsonResponse(201, {
      'tripId': tripId,
      'sequence': durable.sequence,
      'version': durable.version,
      'status': durable.status.name,
    });
  }

  Future<Response> _postLocation(Request request) async {
    final tripId = request.params['id']!;
    final update = _readLocation(await readJsonObject(request));
    final delivered = publishLocation(tripId, update);
    return jsonResponse(202, {'tripId': tripId, 'delivered': delivered});
  }

  Future<Response> _postOffer(Request request) async {
    final userId = request.params['id']!;
    final body = await readJsonObject(request);
    final errors = <String, List<String>>{};
    final tripId = body['tripId'];
    if (tripId is! String || tripId.isEmpty) {
      errors['tripId'] = ['Required non-empty string'];
    }
    Duration? ttl;
    final rawTtl = body['ttlSeconds'];
    if (rawTtl != null) {
      if (rawTtl is int && rawTtl >= 1 && rawTtl <= 600) {
        ttl = Duration(seconds: rawTtl);
      } else {
        errors['ttlSeconds'] = ['Expected an integer from 1 to 600'];
      }
    }
    if (errors.isNotEmpty) {
      throw ProblemException(422, 'Unprocessable request', errors: errors);
    }
    final offer = publishOffer(userId, tripId! as String, ttl: ttl);
    return jsonResponse(202, {
      'offerId': offer.offerId,
      'delivered': offer.delivered,
      'expiresAt': offer.expiresAt.toUtc().toIso8601String(),
    });
  }
}

Map<String, Object?> _eventFrame(String tripId, DurableTripEvent durable) =>
    <String, Object?>{
      'type': 'event',
      'tripId': tripId,
      'sequence': durable.sequence,
      'version': durable.version,
      'occurredAt': durable.occurredAt.toUtc().toIso8601String(),
      'status': durable.status.name,
      'event': tripEventToJson(durable.event),
    };

/// JSON của một [TripEvent]: `{"type": "driverAccepted", "driverId": "D-7"}`.
///
/// Tên `type` là tên lớp theo camelCase: `searchStarted`, `driverAccepted`,
/// `driverArrived`, `pickedUp`, `tripStarted`, `completed`, `cancelled`.
Map<String, Object?> tripEventToJson(TripEvent event) => switch (event) {
  SearchStarted() => {'type': 'searchStarted'},
  DriverAccepted(:final driverId) => {
    'type': 'driverAccepted',
    'driverId': driverId,
  },
  DriverArrived() => {'type': 'driverArrived'},
  PickedUp() => {'type': 'pickedUp'},
  TripStarted() => {'type': 'tripStarted'},
  Completed() => {'type': 'completed'},
  Cancelled(:final by) => {'type': 'cancelled', 'by': by.name},
};

TripEvent _readEvent(Map<String, Object?> body) {
  final type = body['type'];
  switch (type) {
    case 'searchStarted':
      return const SearchStarted();
    case 'driverAccepted':
      final driverId = body['driverId'];
      if (driverId is! String || driverId.trim().isEmpty) {
        throw const ProblemException(
          422,
          'Unprocessable request',
          errors: {
            'driverId': ['Required non-empty string'],
          },
        );
      }
      return DriverAccepted(driverId);
    case 'driverArrived':
      return const DriverArrived();
    case 'pickedUp':
      return const PickedUp();
    case 'tripStarted':
      return const TripStarted();
    case 'completed':
      return const Completed();
    case 'cancelled':
      final by = CancelledBy.values.asNameMap()[body['by']];
      if (by == null) {
        throw const ProblemException(
          422,
          'Unprocessable request',
          errors: {
            'by': ['Expected one of: customer, driver'],
          },
        );
      }
      return Cancelled(by: by);
    default:
      throw const ProblemException(
        422,
        'Unprocessable request',
        errors: {
          'type': [
            'Expected one of: searchStarted, driverAccepted, driverArrived, '
                'pickedUp, tripStarted, completed, cancelled',
          ],
        },
      );
  }
}

DriverLocationUpdate _readLocation(Map<String, Object?> body) {
  final errors = <String, List<String>>{};
  final driverId = body['driverId'];
  if (driverId is! String || driverId.trim().isEmpty) {
    errors['driverId'] = ['Required non-empty string'];
  }
  double? coordinate(String name, double limit) {
    final raw = body[name];
    // Viết dạng "điều kiện hợp lệ" rồi phủ định để NaN cũng bị chặn.
    if (raw is! num || !(raw >= -limit && raw <= limit)) {
      errors[name] = ['Expected a number from -$limit to $limit'];
      return null;
    }
    return raw.toDouble();
  }

  final lat = coordinate('lat', 90);
  final lng = coordinate('lng', 180);
  double? optional(String name) {
    final raw = body[name];
    if (raw == null) return null;
    if (raw is! num || !raw.isFinite) {
      errors[name] = ['Expected a finite number'];
      return null;
    }
    return raw.toDouble();
  }

  final heading = optional('headingDegrees');
  final speed = optional('speedMetersPerSecond');
  if (errors.isNotEmpty || lat == null || lng == null) {
    throw ProblemException(422, 'Unprocessable request', errors: errors);
  }
  return DriverLocationUpdate(
    driverId: (driverId as String).trim(),
    lat: lat,
    lng: lng,
    headingDegrees: heading,
    speedMetersPerSecond: speed,
  );
}

int _epochSeq = 0;

String _newEpoch() =>
    'e${DateTime.now().microsecondsSinceEpoch.toRadixString(36)}-${++_epochSeq}';

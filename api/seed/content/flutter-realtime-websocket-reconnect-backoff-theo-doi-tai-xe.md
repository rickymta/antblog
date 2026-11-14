---
slug: flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe
title: "Realtime WebSocket: reconnect, backoff và theo dõi tài xế"
summary: "Bài 36: viết RealtimeClient cho Tới Nơi trên WebSocket: kết nối sau xác thực bằng token, tách sự kiện bền (sequence, logEpoch) khỏi vị trí tạm thời, reconnect với backoff full jitter, heartbeat, resume không mất và không trùng, gộp vị trí, ngắt khi app đi nền."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, realtime, websocket, ios, android]
language: vi
---

Khách đặt xe, màn hình theo dõi hiện chấm xanh của tài xế đang chạy tới. Xe vào hầm gửi xe, mạng đứt vài giây rồi có lại. Có hai kiểu hỏng. Kiểu thứ nhất: app vẫn ghi "đã kết nối" trong khi socket đã chết lặng, chấm xanh đứng yên mãi. Kiểu thứ hai: app nối lại được, nhưng sự kiện "tài xế đã tới" xảy ra đúng lúc mất mạng không bao giờ tới nơi, hoặc tới hai lần và làm bước đón khách chạy hai lần.

Bài này viết `RealtimeClient` cho cả hai app Tới Nơi để tránh cả hai. Đây là bài 36 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 35 về vị trí](/articles/flutter-vi-tri-geolocator-permission-foreground-background), [bài 05 về Stream](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri), [bài 24 về token](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage) và [bài 30 về fakeAsync](/articles/flutter-test-riverpod-async-stream-fake-async-providercontainer) trước. Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); mọi khoảng thời gian trong bài là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Client mất kết nối, nối lại và gửi <code>subscribe</code> cho chuyến đang theo dõi, không kèm gì khác. Gateway chỉ phát sự kiện mới từ lúc đó. Trong lúc mất mạng, tài xế đã bấm "đã tới điểm đón". Khách sẽ thấy gì, và có lỗi nào được báo không?</p>
<details><summary>Xem đáp án</summary><p>Khách thấy trạng thái cũ ("tài xế đã nhận") cho tới khi có sự kiện kế tiếp, và không có lỗi nào: <code>subscribe</code> thành công, chỉ là một sự kiện đã trôi qua mà không ai kể lại. Chữa bằng cách đánh số sự kiện bền theo chuyến (<code>sequence</code>) để client nói "tôi đã nhận tới số N" khi nối lại.</p></details>
</div>

## Hai loại tin trên một kết nối

Hồ sơ kiến trúc chia tin từ Realtime Gateway làm hai loại (Hình 1). **Sự kiện bền** như "tài xế đã nhận" phải truy vết và khôi phục được. **Dữ liệu tạm thời** như vị trí mới nhất có thể gộp hoặc bỏ khi quá tải.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 390" role="img" aria-label="Hình 1. Backend gửi sự kiện bền và vị trí vào Realtime Gateway. Gateway giữ log có sequence cho sự kiện bền và chỉ giữ bản mới nhất cho vị trí. Cả hai chảy tới RealtimeClient; client gửi ngược subscribe kèm lastSequence."><defs><marker id="fl36-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl36-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="120" width="170" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="158" class="t-b">Backend</text><text x="36" y="184" class="t-small">trip-service,</text><text x="36" y="206" class="t-small">vị trí tài xế</text>
<rect x="290" y="20" width="300" height="300" rx="10" class="f-surface s-line"/><text x="306" y="50" class="t-b">Realtime Gateway</text>
<rect x="310" y="70" width="260" height="110" rx="10" class="f-pk-soft s-pk"/><text x="326" y="100" class="t-b">Sự kiện bền</text><text x="326" y="128" class="t-small">log theo chuyến</text><text x="326" y="150" class="t-small">sequence, logEpoch</text>
<rect x="310" y="200" width="260" height="100" rx="10" class="f-warn-soft s-warn"/><text x="326" y="230" class="t-b">Vị trí tạm thời</text><text x="326" y="258" class="t-small">chỉ nhớ bản mới nhất</text><text x="326" y="280" class="t-small">không có sequence</text>
<rect x="690" y="120" width="170" height="110" rx="10" class="f-phys-soft s-phys"/><text x="706" y="158" class="t-b">App Tới Nơi</text><text x="706" y="186" class="t-mono">RealtimeClient</text>
<path d="M192 160 L308 125" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-1-a)"/>
<path d="M192 195 L308 250" class="f-none s-warn" stroke-width="2" marker-end="url(#fl36-1-w)"/>
<path d="M572 125 L688 160" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-1-a)"/><text x="618" y="132" class="t-small">event #n</text>
<path d="M572 250 L688 195" class="f-none s-warn" stroke-width="2" marker-end="url(#fl36-1-w)"/><text x="612" y="252" class="t-small">location</text>
<path d="M775 232 L775 352 L440 352 L440 322" class="f-none s-pk" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl36-1-a)"/><text x="470" y="378" class="t-small">subscribe(tripId, lastSequence)</text>
</svg>
</div>
<figcaption>Hình 1. Hai đường đi qua cùng một kết nối. Sự kiện bền đi qua log có số thứ tự nên phát lại được; vị trí chỉ còn bản mới nhất. Đường nét đứt là lời xin của client khi nối lại.</figcaption>
</figure>

Lab dùng Realtime Gateway giả `RealtimeModule` trong `tools/fake_api` (bật bằng `--with-realtime`); hợp đồng JSON nằm trong README của `fake_api` và ổn định vì bài 45 và 47 dựa vào. Mỗi tin là một text frame chứa một JSON object có `type`:

```json
{"type":"event","tripId":"T-1","sequence":2,"version":2,"occurredAt":"2026-11-06T03:30:12.000Z","status":"accepted","event":{"type":"driverAccepted","driverId":"D-7"}}
{"type":"location","tripId":"T-1","driverId":"D-7","lat":10.7769,"lng":106.7009,"at":"2026-11-06T03:30:15.000Z"}
```

`event.event` là một trong bảy `TripEvent` của `toi_noi_core`; `sequence` bắt đầu từ 1 và tăng đúng 1 mỗi sự kiện của chuyến; `version` là `version` của chuyến sau sự kiện, cùng thước đo với REST. Frame `location` không có `sequence`. Frame hỏng ném `FormatException`; loại frame lạ bị bỏ qua để gateway thêm loại mới mà app cũ không vỡ.

### realtime_protocol.dart · 36-realtime-websocket/lib/realtime_protocol.dart (trích)

```dart
final type = _string(decoded, 'type');
return switch (type) {
  'welcome' => WelcomeFrame(
    connectionId: _string(decoded, 'connectionId'),
    userId: _string(decoded, 'userId'),
  ),
  'subscribed' when decoded['channel'] == 'offers' =>
    const OffersSubscribedFrame(),
  'subscribed' => SubscribedFrame(
    tripId: _string(decoded, 'tripId'),
    latestSequence: _int(decoded, 'latestSequence'),
    replayed: _int(decoded, 'replayed'),
    logEpoch: _optionalString(decoded, 'logEpoch'),
  ),
  'offer' => OfferFrame(_offer(decoded)),
  'unsubscribed' => UnsubscribedFrame(_string(decoded, 'tripId')),
  'event' => EventFrame(_envelope(decoded)),
  'location' => LocationFrame(_location(decoded)),
  'pong' => const PongFrame(),
  'error' => ErrorFrame(
    code: _string(decoded, 'code'),
    message: _string(decoded, 'message'),
    tripId: _optionalString(decoded, 'tripId'),
    channel: _optionalString(decoded, 'channel'),
  ),
  _ => UnknownFrame(type),
};
```

## Kết nối sau xác thực và bốn trạng thái

Client gửi access token trong header `Authorization: Bearer`, không đặt vào URL vì URL dễ bị ghi vào log. Token lấy từ `tokenProvider`, gọi lại ở **mỗi** lần mở kết nối, kể cả khi `resume`: đây là chỗ làm mới token hết hạn (bài 24), nên nó phải trả token còn hạn.

Client nói chuyện qua interface `RealtimeTransport` để test thay bằng transport giả. Bản thật bọc `WebSocket` của `dart:io`:

### realtime_transport.dart · 36-realtime-websocket/lib/realtime_transport.dart

```dart
final class WebSocketTransport implements RealtimeTransport {
  const WebSocketTransport({this.connectTimeout = const Duration(seconds: 10)});

  /// Chờ bắt tay tối đa chừng này; quá hạn thì `connect` ném `TimeoutException`
  /// và socket nào tới muộn sau đó bị đóng ngay.
  final Duration connectTimeout;

  @override
  Future<RealtimeConnection> connect(Uri url, {required String token}) async {
    final connecting = WebSocket.connect(
      url.toString(),
      headers: {'Authorization': 'Bearer $token'},
    );
    final WebSocket socket;
    try {
      socket = await connecting.timeout(connectTimeout);
    } on TimeoutException {
      // `Future.timeout` không huỷ việc đang chạy: nếu không đóng, socket tới
      // muộn sẽ mở mà không ai giữ.
      unawaited(
        connecting.then(
          (lateSocket) => lateSocket.close(),
          onError: (Object _) {},
        ),
      );
      rethrow;
    } on WebSocketException catch (e) {
      if (e.httpStatusCode == 401) throw const RealtimeAuthRejected();
      rethrow;
    }
    return _ChannelConnection(IOWebSocketChannel(socket));
  }
}
```

Gateway có hai cách từ chối token xấu, và client phải coi cả hai là lỗi xác thực. Một: HTTP 401 ngay ở bước nâng cấp; `dart:io` báo qua `WebSocketException.httpStatusCode` (test kiểm trên Dart 3.13.3) và transport đổi thành `RealtimeAuthRejected`. Hai: nâng cấp xong rồi đóng bằng mã **4401** (vùng 4000 đến 4999 dành cho ứng dụng), kiểu gateway giả dùng mặc định; `rejectBeforeUpgrade` bật kiểu một. Transport còn một việc: `Future.timeout` không huỷ việc đang chạy, nên socket nâng cấp tới sau `connectTimeout` phải bị đóng ngay, nếu không nó mở mà không ai giữ.

Trạng thái kết nối chỉ có bốn giá trị (Hình 2). `connected` chỉ đến khi gateway gửi `welcome`: bắt tay WebSocket xong mới là có đường ống, chưa chắc token được chấp nhận.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Bốn trạng thái: closed chuyển sang connecting khi gọi connect; connecting sang connected khi nhận welcome; connected sang reconnecting khi mất kết nối hoặc heartbeat im lặng; reconnecting sang connected khi nhận welcome; từ mọi trạng thái có thể về closed bằng close, pause hoặc mã 4401."><defs><marker id="fl36-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl36-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="30" y="40" width="190" height="70" rx="10" class="f-warn-soft s-warn"/><text x="46" y="70" class="t-b">closed</text><text x="46" y="94" class="t-small">có lý do (cause)</text>
<rect x="345" y="40" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="361" y="70" class="t-b">connecting</text><text x="361" y="94" class="t-small">mở lần đầu</text>
<rect x="660" y="40" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="676" y="70" class="t-b">connected</text><text x="676" y="94" class="t-small">đã nhận welcome</text>
<rect x="345" y="220" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="361" y="250" class="t-b">reconnecting</text><text x="361" y="274" class="t-small">chờ backoff, thử lại</text>
<path d="M222 75 L343 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-2-a)"/><text x="244" y="64" class="t-mono">connect()</text>
<path d="M537 75 L658 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-2-a)"/><text x="560" y="64" class="t-mono">welcome</text>
<path d="M440 112 L440 218" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-2-a)"/><text x="452" y="160" class="t-small">lỗi, hết giờ</text><text x="452" y="178" class="t-small">bắt tay</text>
<path d="M820 112 L820 266 L537 266" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-2-a)"/><text x="548" y="304" class="t-small">socket đóng hoặc heartbeat im lặng</text>
<path d="M537 242 L700 242 L700 112" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-2-a)"/><text x="562" y="232" class="t-mono">welcome</text>
<path d="M343 255 L125 255 L125 112" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl36-2-w)"/><text x="138" y="243" class="t-small t-warn">close(), pause(), mã 4401</text><text x="138" y="278" class="t-small">từ mọi trạng thái</text>
</svg>
</div>
<figcaption>Hình 2. Bốn trạng thái của <code>RealtimeClient</code>. Đường nét đứt về <code>closed</code> không bao giờ tự quay lại: <code>closed</code> chỉ rời đi khi người dùng client gọi <code>connect()</code> hoặc <code>resume()</code>.</figcaption>
</figure>

Hai kiểu thất bại phải được tách. Mất mạng, lỗi socket, gateway khởi động lại: nối lại. Lỗi xác thực: **không** nối lại, vì cùng một token thì lần nào cũng bị từ chối. Client dừng ở `closed` với `cause: unauthorized`; `resume()` không mở lại, phải `connect()` sau khi đăng nhập lại. Gateway giả cũng đóng kết nối đang mở bằng 4401 khi token hết hạn, và client xử lý như vậy.

### realtime_client.dart · 36-realtime-websocket/lib/realtime_client.dart (trích)

```dart
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
```

`_lost` nhận mọi cách kết nối chấm dứt; chỉ mã 4401 rẽ sang `closed`. Test kiểm chỉ một lần gọi transport sau một giờ giả.

## Mất kết nối: backoff full jitter và heartbeat

Nối lại ngay nghe hợp lý, cho tới khi gateway khởi động lại và hàng chục nghìn máy cùng nối. **Exponential backoff** gấp đôi thời gian chờ sau mỗi lần thất bại, có trần. Nhưng nếu mọi máy đều chờ đúng 1 giây rồi đúng 2 giây, chúng vẫn đồng bộ với nhau. **Full jitter** chọn thời gian chờ đều ngẫu nhiên từ 0 tới trần, nên đám đông bị rải ra.

### backoff.dart · 36-realtime-websocket/lib/backoff.dart

```dart
final class Backoff {
  Backoff({
    this.base = const Duration(seconds: 1),
    this.cap = const Duration(seconds: 30),
    Random? random,
  }) : assert(base > Duration.zero, 'base phải dương'),
       assert(cap >= base, 'cap phải không nhỏ hơn base'),
       _random = random ?? Random();

  final Duration base;
  final Duration cap;
  final Random _random;

  /// Trần của lần thử thứ [attempt], tính từ 0.
  Duration ceiling(int attempt) {
    if (attempt < 0) throw RangeError.value(attempt, 'attempt');
    // 2^30 đã vượt mọi cap hợp lý; chặn để phép nhân không tràn int.
    if (attempt >= 30) return cap;
    final doubled = base * (1 << attempt);
    return doubled > cap ? cap : doubled;
  }

  /// Thời gian chờ thực tế của lần thử thứ [attempt]: đều trong [0, trần).
  Duration delay(int attempt) {
    final max = ceiling(attempt).inMicroseconds;
    return Duration(microseconds: (_random.nextDouble() * max).floor());
  }
}
```

Với `base` 1 giây và `cap` 30 giây, trần là 1, 2, 4, 8, 16, 30, 30 giây. Test dựng 1000 client mất kết nối cùng lúc ở lần thử thứ tư (trần 8 giây), chia vào các ô 500 ms: có jitter thì mỗi ô trung bình khoảng 63 client và ô đông nhất dưới 100; không jitter thì cả 1000 rơi vào một ô.

Khi nào backoff quay về lần đầu? Nếu reset ngay khi vừa kết nối được, một gateway nhận kết nối rồi đóng sau vài trăm mili giây sẽ bị nối lại với thời gian chờ ngắn nhất mãi mãi. Nên client chỉ xoá lịch sử thất bại khi kết nối **sống đủ lâu** (`stableAfter`, 30 giây).

### realtime_client.dart · 36-realtime-websocket/lib/realtime_client.dart (trích)

```dart
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
```

Test `kết nối chập chờn` cho lịch 500, 1000, 2000, 4000 ms; test `sống đủ stableAfter` kiểm biên: thiếu 1 ms thì chưa reset.

**Heartbeat.** Một socket có thể chết mà không ai báo: stream không kết thúc, không có lỗi, chỉ là không có gì đến nữa. Cứ 15 giây client gửi `ping`; nếu sau 5 giây không nhận được **bất kỳ** frame nào, kết nối bị coi là chết. `dart:io` có sẵn `pingInterval`, nhưng ping ở tầng ứng dụng test được bằng `fakeAsync` qua `RealtimeTransport` và cho biết code của gateway còn chạy, không chỉ tầng WebSocket.

### client_heartbeat_test.dart · 36-realtime-websocket/test/client_heartbeat_test.dart (trích)

```dart
test('gateway im lặng sau ping: đúng heartbeatTimeout thì coi là chết, đóng và nối lại', () {
  fakeAsync((async) {
    final transport = FakeTransport();
    final client = buildClient(transport);
    client.connect();
    settle(async);
    final first = transport.last;

    async.elapse(const Duration(seconds: 15)); // ping gửi đi
    async.elapse(
      const Duration(seconds: 5) - const Duration(milliseconds: 1),
    );
    expect(client.state.status, RealtimeStatus.connected);

    async.elapse(const Duration(milliseconds: 1));
    expect(client.state.status, RealtimeStatus.reconnecting);
    expect(first.closedByClient, isTrue);

    async.elapse(client.state.nextRetryIn!);
    expect(transport.connections, hasLength(2));
    expect(client.state.status, RealtimeStatus.connected);
  });
});
```

## Resume: không mất, không trùng

Quay lại câu hỏi mở đầu. Client nhớ `sequence` cuối đã đưa ra cho app và gửi `subscribe` kèm `lastSequence` đó mỗi lần nối lại. Gateway phát lại mọi sự kiện lớn hơn, theo thứ tự, rồi mới tới sự kiện mới (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 3. Log của gateway có sự kiện 1 đến 5. Client nhận 1 và 2 trực tiếp, mất kết nối, nối lại và gửi subscribe với lastSequence bằng 2. Gateway phát lại 3 và 4, sự kiện 5 đến trực tiếp. Mỗi sự kiện ra đúng một lần."><defs><marker id="fl36-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl36-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="12" y="62" class="t-b">Log gateway</text><text x="12" y="84" class="t-small">đủ 1 đến 5</text>
<text x="12" y="202" class="t-b">Client nhận</text><text x="12" y="224" class="t-small">ra cho app</text>
<rect x="150" y="45" width="90" height="56" rx="10" class="f-phys-soft s-phys"/><text x="166" y="70" class="t-b">#1</text><text x="166" y="90" class="t-small">bền</text>
<rect x="255" y="45" width="90" height="56" rx="10" class="f-phys-soft s-phys"/><text x="271" y="70" class="t-b">#2</text><text x="271" y="90" class="t-small">bền</text>
<rect x="360" y="45" width="90" height="56" rx="10" class="f-phys-soft s-phys"/><text x="376" y="70" class="t-b">#3</text><text x="376" y="90" class="t-small">bền</text>
<rect x="465" y="45" width="90" height="56" rx="10" class="f-phys-soft s-phys"/><text x="481" y="70" class="t-b">#4</text><text x="481" y="90" class="t-small">bền</text>
<rect x="570" y="45" width="90" height="56" rx="10" class="f-phys-soft s-phys"/><text x="586" y="70" class="t-b">#5</text><text x="586" y="90" class="t-small">bền</text>
<rect x="150" y="185" width="90" height="56" rx="10" class="f-pk-soft s-pk"/><text x="166" y="210" class="t-b">#1</text><text x="166" y="230" class="t-small">trực tiếp</text>
<rect x="255" y="185" width="90" height="56" rx="10" class="f-pk-soft s-pk"/><text x="271" y="210" class="t-b">#2</text><text x="271" y="230" class="t-small">trực tiếp</text>
<rect x="360" y="185" width="90" height="56" rx="10" class="f-warn-soft s-warn"/><text x="376" y="210" class="t-b">#3</text><text x="376" y="230" class="t-small">phát lại</text>
<rect x="465" y="185" width="90" height="56" rx="10" class="f-warn-soft s-warn"/><text x="481" y="210" class="t-b">#4</text><text x="481" y="230" class="t-small">phát lại</text>
<rect x="570" y="185" width="90" height="56" rx="10" class="f-pk-soft s-pk"/><text x="586" y="210" class="t-b">#5</text><text x="586" y="230" class="t-small">trực tiếp</text>
<path d="M195 103 L195 183" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-3-a)"/>
<path d="M300 103 L300 183" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-3-a)"/>
<path d="M405 103 L405 183" class="f-none s-warn" stroke-width="2" marker-end="url(#fl36-3-w)"/>
<path d="M510 103 L510 183" class="f-none s-warn" stroke-width="2" marker-end="url(#fl36-3-w)"/>
<path d="M615 103 L615 183" class="f-none s-pk" stroke-width="2" marker-end="url(#fl36-3-a)"/>
<path d="M352 20 L352 275" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6"/>
<text x="360" y="300" class="t-small t-warn">mất kết nối, nối lại, gửi subscribe với lastSequence = 2</text>
<text x="680" y="100" class="t-small">mỗi #n ra</text><text x="680" y="120" class="t-small">đúng một lần,</text><text x="680" y="140" class="t-small">đúng thứ tự</text>
</svg>
</div>
<figcaption>Hình 3. Sự kiện 3 và 4 xảy ra khi client mất kết nối. Nhờ <code>lastSequence = 2</code>, gateway phát lại đúng hai sự kiện đó; sự kiện 5 đến trực tiếp sau khi phát lại xong.</figcaption>
</figure>

Điều này chỉ đúng với hai điều kiện. Thứ nhất, `sequence` chỉ so được trong **cùng một log**: log của fake nằm trong bộ nhớ, gateway khởi động lại là sang log mới và `sequence` bắt đầu lại từ 1. Mỗi log có một `logEpoch`; client gửi kèm cái đã thấy, và gateway từ chối phát lại (`log_reset`) nếu khác, hoặc nếu `lastSequence` vượt log hiện có. Thứ hai, sự kiện phải còn trong log: nếu đã bị cắt, gateway trả `replay_unavailable`.

### realtime_module.dart · tools/fake_api/lib/src/realtime_module.dart (trích)

```dart
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
```

Gateway giả phát lại và đăng ký người nghe trong một lượt đồng bộ, nhưng gateway thật có thể nhiều node, nên client không tin mà tự kiểm. Nếu `logEpoch` trong `subscribed` đổi, hoặc `latestSequence` nhỏ hơn số đã nhận, client **thôi nghe** chuyến đó và báo `log_reset`. Chỉ báo mà vẫn nghe tiếp là sai: sự kiện 3, 4, 5 của log mới sẽ bị đưa ra như của log cũ, hoặc bị coi là trùng.

### realtime_client.dart · 36-realtime-websocket/lib/realtime_client.dart (trích)

```dart
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
```

Còn ba nhánh so `sequence` với số cuối đã nhận `last`: nhỏ hơn hoặc bằng là bản trùng, đúng bằng `last + 1` thì đưa ra, lớn hơn là thiếu giữa chừng và client xin lại từ `last`. Nếu gateway trả lời mà vẫn không lấp được lỗ hổng, client báo `replay_unavailable` và thôi nghe thay vì bỏ mọi sự kiện sau đó trong im lặng.

### realtime_client.dart · 36-realtime-websocket/lib/realtime_client.dart (trích)

```dart
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
```

`events` là stream một người nghe và **đệm** cho tới khi có người `listen`, nên không có sự kiện nào được tính là đã nhận mà chưa tới app.

REST thì sao? `publish` của gateway giả ghi vào kho nên `GET /trips/{id}` thấy cùng trạng thái, và response REST mang thêm `lastSequence` và `logEpoch`. Bài 45 hợp nhất theo `version`: lấy bản chụp REST, `subscribe` với `lastSequence` và `logEpoch` của nó, bỏ mọi `event` có `version` không lớn hơn bản chụp.

Test socket thật: gateway ngắt kết nối, hai sự kiện xảy ra lúc client chưa nối lại, thêm một sự kiện sau khi nối.

### real_socket_test.dart · 36-realtime-websocket/test/real_socket_test.dart (trích)

```dart
await gateway.realtime.dropConnections();
// Sự kiện xảy ra trong lúc client chưa kịp nối lại.
gateway.realtime
  ..publish(tripId, const DriverArrived())
  ..publish(tripId, const PickedUp());
await waitForState(client, (s) => s.status == RealtimeStatus.connected);
await eventually(() => events.length == 4);
gateway.realtime.publish(
  tripId,
  const TripStarted(),
); // sự kiện trực tiếp sau resume
await eventually(() => events.length == 5);

expect(events.map((e) => e.sequence), [1, 2, 3, 4, 5]);
expect(client.duplicatesDropped, 0); // gateway chỉ phát lại phần thiếu
expect(transport.connects, 2);
expect(gateway.realtime.subscriberCount(tripId), 1);
```

## Vị trí tài xế và lời mời cuốc

Tài xế có thể gửi vài vị trí mỗi giây; bản cũ giao muộn không được đè bản mới. Client áp dụng hai luật: bản có `at` cũ hơn hoặc bằng bản đã đưa ra (hay đang chờ) thì bỏ; mỗi chuyến có một cửa sổ `locationInterval` (1 giây), bản đầu ra ngay, các bản đến trong cửa sổ chỉ còn bản cuối, ra khi cửa sổ khép. Mốc `at` chỉ ghi khi bản đó thực sự ra, nên bản bị huỷ lúc đi nền hay `unsubscribe` không để lại mốc ảo chặn bản gateway gửi lại.

### realtime_client.dart · 36-realtime-websocket/lib/realtime_client.dart (trích)

```dart
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
```

Với 50 bản trong 10 giây, test kiểm client đưa ra đúng 11 bản và bản cuối là bản thứ 50: tối đa một lần dựng lại bản đồ mỗi giây, nhưng vị trí cuối không bị nuốt.

App tài xế (bài 47) cần thêm kênh theo **người dùng** `driver:{id}:offers` của hồ sơ kiến trúc. Gateway giả dành sẵn frame `offer` (chuyến, cước, hạn) và client có `subscribeOffers()`. Kênh luôn là của người dùng trong token (client không gửi mã tài xế), chỉ vai trò `driver`, và không phát lại. Simulator có `POST /realtime/drivers/{userId}/offers`.

## Quyền, nền và huỷ

Hồ sơ kiến trúc yêu cầu kiểm quyền subscribe ở server: không cho client tự đoán mã chuyến rồi nghe dữ liệu người khác. Gateway giả có chỗ cắm `authorize(người dùng, chuyến)` (test kiểm `forbidden`), nhưng mặc định cho mọi người dùng đã xác thực nghe mọi chuyến có thật vì kho của fake không có chủ chuyến. Đó là đơn giản hoá của lab, cũng như việc fake giữ log (hồ sơ chỉ giao gateway lo kết nối và fan-out); gateway thật phải kiểm khách hoặc tài xế có thuộc chuyến.

Khi khách chuyển sang app khác, hệ điều hành có thể không để socket yên. Với Android, tài liệu về Doze nói thiết bị rảnh bị tạm cắt quyền dùng mạng của app (kiểm tại 11/2026) và khuyên dùng FCM thay vì tự giữ kết nối lâu dài (bài 37). Với iOS, ta cũng không nên dựa vào socket còn sống ở nền (xem tài liệu vòng đời app của Apple ở cuối bài). Lab chỉ làm phần app kiểm soát được: ngắt chủ động khi `paused`, nối lại khi `resumed`.

### realtime_lifecycle.dart · 36-realtime-websocket/lib/realtime_lifecycle.dart

```dart
final class RealtimeLifecycleBinding with WidgetsBindingObserver {
  RealtimeLifecycleBinding(this.client, {WidgetsBinding? binding})
    : _binding = binding ?? WidgetsBinding.instance {
    _binding.addObserver(this);
  }

  final RealtimeClient client;
  final WidgetsBinding _binding;

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.paused || AppLifecycleState.detached:
        client.pause();
      case AppLifecycleState.resumed:
        client.resume();
      case AppLifecycleState.inactive || AppLifecycleState.hidden:
        break;
    }
  }

  /// Gỡ khỏi binding. Không dispose [client].
  void dispose() => _binding.removeObserver(this);
}
```

`inactive` và `hidden` là trạng thái thoáng qua nên không ngắt; theo tài liệu `AppLifecycleState`, trên Android `Activity.onPause` là `inactive` còn `onStop` mới là `paused`. `pause()` nhớ chuyến và `sequence`, nên `resume()` subscribe tiếp đúng chỗ. `lifecycle_test.dart` giả lập vòng đời bằng `handleAppLifecycleStateChanged`; vòng đời thật chưa được kiểm.

Cuối cùng là huỷ, và nguy hiểm nhất là chuyện giữa chừng: người dùng rời màn hình đúng lúc đang bắt tay. Mỗi lần `connect`, `close`, `pause` và `dispose` tăng bộ đếm `_generation`; mọi bước bất đồng bộ kiểm lại nó sau mỗi `await`. Riêng `dispose` đặt cờ đã huỷ **trước** `await`: nếu đặt sau, `connect()` gọi ngay sau `dispose()` vẫn mở được kết nối sống cùng hai Timer.

### realtime_client.dart · 36-realtime-websocket/lib/realtime_client.dart (trích)

```dart
if (generation != _generation) {
  // Bị huỷ (close, pause, dispose) trong lúc đang bắt tay: đừng để rò.
  unawaited(_closeQuietly(connection));
  return;
}
_attach(connection);
```

**Test với socket thật.** Khi có `TestWidgetsFlutterBinding`, `HttpClient()` là bản giả luôn trả 400 (có test kiểm). Nhưng `WebSocket.connect` mặc định dùng một `HttpClient` ở biến static của `dart:io`; suy từ source, `flutter test` gọi `WebSocket.connect` lúc khởi động, trước khi binding đặt override, nên client đó là client thật. Đó là chi tiết cài đặt, không phải hợp đồng, nên test socket thật nằm ở `real_socket_test.dart`, file Dart thuần không khởi tạo binding, vẫn chạy trong `flutter test`.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp với WebSocket di động</span>
<ul>
<li><strong>Nối lại ngay, không backoff hoặc không jitter.</strong> Dấu hiệu: sau mỗi lần deploy gateway, số kết nối mới tăng vọt. Nguyên nhân: mọi máy chờ cùng một khoảng. Cách sửa: backoff có trần với full jitter, chỉ reset khi kết nối đã sống ổn định.</li>
<li><strong><code>subscribe</code> lại mà không kèm <code>lastSequence</code> và <code>logEpoch</code>.</strong> Dấu hiệu: màn hình kẹt ở trạng thái cũ sau khi mạng chập chờn, hoặc sự kiện của log khác được đưa ra như của log này. Cách sửa: nhớ cả hai theo chuyến, gửi lại khi nối, khử trùng lặp khi nhận, thôi nghe khi log đổi.</li>
<li><strong>Nối lại vô hạn khi token hết hạn.</strong> Dấu hiệu: log đầy các kết nối bị đóng mã 4401 hoặc HTTP 401. Cách sửa: dừng ở <code>closed(unauthorized)</code>, làm mới token trong <code>tokenProvider</code> rồi mới <code>connect()</code>.</li>
<li><strong>Không có heartbeat.</strong> Dấu hiệu: vẫn "connected" nhưng không có dữ liệu sau khi đổi mạng. Cách sửa: ping định kỳ, coi im lặng quá hạn là chết.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/36-realtime-websocket
flutter pub get
flutter analyze
flutter test

cd ../tools/fake_api
dart pub get
dart analyze
dart test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3, `web_socket_channel` 3.0.3 (mới nhất lúc kiểm), `shelf_web_socket` 3.0.0, `fake_async` 1.3.3: lab báo 0 issue và chạy 131 test xanh; `fake_api` 0.3.0 chạy 132 test xanh (41 test mới). Mức: **Kiểm chứng tự động**. Logic chạy trên `fake_async` và transport giả; test socket thật chỉ chờ sự kiện trên `127.0.0.1` (riêng test `connectTimeout` dùng độ trễ 300 ms của fake API). Test client dùng transport giả (70 test), giao thức và backoff (39), vòng đời app (6), socket thật (14); phía `fake_api`, `realtime_test.dart` kiểm hợp đồng từ phía gateway.

**Chưa làm**: chưa chạy trên simulator, emulator hay thiết bị, nên hành vi socket khi app đi nền thật và qua mạng di động chưa được kiểm. Chưa có foreground service (bài 38), push (bài 37), nối vào Riverpod hay màn hình và đếm ngược lời mời cuốc (bài 45 và 47). `WebSocketTransport` dùng `dart:io` nên chưa hỗ trợ Flutter web.

## Bài tập

1. Đổi `_stableTimer` để reset `_attempt` ngay khi `welcome` đến. Gateway nhận kết nối rồi đóng sau 200 ms, lặp lại. Thời gian chờ giữa các lần thử trở thành thế nào, và test nào đỏ?
2. Gateway khởi động lại và mất log. Client đã nhận tới `sequence` 2; log mới đã có 5 sự kiện. Client có đưa 3, 4, 5 ra không? App nên làm gì?
3. Bản chụp REST của một chuyến có `version` 4, `lastSequence` 3, `logEpoch` `e1`. App `subscribe` với `lastSequence: 0` thay vì 3. Điều gì khác đi, và vì sao nên dùng 3?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Mỗi lần kết nối lại coi như lần đầu nên trần luôn là 1 giây: client dội vào gateway đang gặp sự cố. Hai test <code>kết nối chập chờn</code> và <code>sống đủ stableAfter</code> đỏ.</p>
<p><strong>2.</strong> Không. <code>subscribed</code> mang <code>logEpoch</code> khác cái client đã biết nên client phát <code>RealtimeIssue('log_reset')</code> và thôi nghe chuyến (test <code>logEpoch đổi dù log mới đã dài hơn</code>); gateway cũng tự trả <code>log_reset</code> khi nhận <code>logEpoch</code> cũ. App tải lại bản chụp REST rồi <code>subscribe</code> với <code>lastSequence</code> và <code>logEpoch</code> của bản chụp.</p>
<p><strong>3.</strong> Gateway phát lại #1 đến #3, đều có <code>version</code> không lớn hơn 4 nên phải bị bỏ theo <code>version</code>. Với <code>lastSequence: 3</code>, <code>replayed</code> là 0 và sự kiện kế tiếp có <code>sequence</code> 4 (test <code>snapshot REST + lastSequence</code> trong <code>fake_api</code>).</p>
</details>

Bài trước: [Vị trí: geolocator, quyền foreground/background và cấu hình iOS, Android](/articles/flutter-vi-tri-geolocator-permission-foreground-background). Bài tiếp: [Push notification: FCM, APNs, local notification và deep link từ thông báo](/articles/flutter-push-notification-fcm-apns-deep-link).

## Tài liệu gốc

- [Work with WebSockets](https://docs.flutter.dev/cookbook/networking/web-sockets) (docs.flutter.dev) và [web_socket_channel](https://pub.dev/packages/web_socket_channel), [shelf_web_socket](https://pub.dev/packages/shelf_web_socket), [fake_async](https://pub.dev/packages/fake_async) (pub.dev).
- [WebSocket](https://api.dart.dev/stable/dart-io/WebSocket-class.html) và [WebSocketException](https://api.dart.dev/stable/dart-io/WebSocketException-class.html) (api.dart.dev).
- [AppLifecycleState](https://api.flutter.dev/flutter/dart-ui/AppLifecycleState.html) và [WidgetsBindingObserver](https://api.flutter.dev/flutter/widgets/WidgetsBindingObserver-class.html) (api.flutter.dev).
- [Optimize for Doze and App Standby](https://developer.android.com/training/monitoring-device-state/doze-standby) (developer.android.com), kiểm tại 11/2026.
- [Managing your app's life cycle](https://developer.apple.com/documentation/uikit/app_and_environment/managing_your_app_s_life_cycle) (developer.apple.com); kiểm lại tại thời điểm phát hành.

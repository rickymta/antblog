---
slug: flutter-thanh-toan-vi-xanh-webview-deep-link-idempotency
title: "Thanh toán qua VíXanh: WebView, deep link quay về và idempotency key"
summary: "Bài 39: trả cước bằng ví VíXanh mà app không chạm vào thẻ. Chọn trình duyệt trong app thay vì WebView, idempotency key giữ qua các lần thử, deep link chỉ là tín hiệu, máy trạng thái hỏi server có giới hạn, trạng thái bền khi app bị kill, đối chiếu chính sách Apple và Google."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, payment, deep-link, idempotency, riverpod, ios, android]
language: vi
---

Khách đặt chuyến 5 km và trả bằng ví VíXanh: 25.500 VND. App mở trang của ví, khách xác nhận rồi quay lại app. Có ba chỗ dễ làm tiền đi lệch. Mạng chập chờn khiến app tạo thanh toán hai lần. Ai đó mở link giả `toinoi://payment/return?status=success` để app tưởng đã trả. App bị kill đúng lúc khách đang ở trang ví, và lần mở sau không biết gì.

Bài này dựng `PaymentFlow`, máy trạng thái Dart thuần cho luồng đó: server giữ sự thật, app chỉ giữ một phiên thanh toán và một khoá idempotency. Đây là bài 39 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 38 về tác vụ nền](/articles/flutter-background-work-workmanager-vi-tri-nen-app-tai-xe), [bài 20 về retry và mất mạng](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong), [bài 22 về lưu trữ cục bộ](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline) và [bài 23 về deep link](/articles/flutter-go-router-deep-link-shellroute-guard) trước. VíXanh là ví hư cấu dùng trong các ví dụ của blog (có mặt ở [bài về design pattern](/articles/design-pattern-la-gi) và các bài kiểm thử), không phải nhà cung cấp thật; hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi) chỉ nói tới "Adapter cho cổng thanh toán". Thời hạn phiên, lịch hỏi và số tiền trong bài đều là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Khách trả xong ở trang VíXanh. Trình duyệt mở <code>toinoi://payment/return?session=S1&amp;status=success</code> và app nhận được link. App có nên hiện "Thanh toán thành công" ngay không?</p>
<details><summary>Xem đáp án</summary><p><strong>Không.</strong> Link tuỳ chỉnh ai cũng gọi được, nên <code>status=success</code> chỉ là lời khẳng định của người gọi. App chỉ dùng link để biết phiên nào vừa quay về, rồi hỏi server. Test <code>status=success nhưng server nói chưa</code> chứng minh state không phải thành công trong trường hợp đó.</p></details>
</div>

## Không xử lý thẻ trong app: app chỉ giữ một phiên thanh toán

Hồ sơ kiến trúc của Tới Nơi (§4.9) có miền Payment & Wallet, chung cho mọi cổng chứ không riêng VíXanh: ledger, webhook của cổng, đối soát; không lưu số thẻ, dùng trang thanh toán do cổng host. App theo đúng ranh giới đó: không bao giờ thấy số thẻ, mật khẩu hay OTP của ví. App xin server một **phiên thanh toán** (mã phiên, địa chỉ trang cổng, hạn), mở trang đó, rồi hỏi lại server kết quả. `PaymentGateway` là chỗ Adapter mà hồ sơ nói tới ([nhóm cấu trúc](/articles/nhom-cau-truc-structural)): phần còn lại của app không biết VíXanh hay dio.

### payment_gateway.dart · 39-payment-flow/lib/payment_gateway.dart

```dart
/// Cổng vào server Payment & Wallet của Tới Nơi. App chỉ nói chuyện với
/// server của mình, không bao giờ với VíXanh trực tiếp.
abstract interface class PaymentGateway {
  /// Tạo phiên thanh toán. Cùng [idempotencyKey] và cùng nội dung thì server
  /// trả lại đúng phiên cũ thay vì tạo phiên mới.
  Future<PaymentSession> createSession({
    required String tripId,
    required Money amount,
    required String idempotencyKey,
  });

  /// Trạng thái hiện tại của phiên theo server.
  Future<PaymentStatus> fetchStatus(String sessionId);
}
```

Còn chính sách store? Mục 3.1.3(e) của App Store Review Guidelines nói hàng hoá hoặc dịch vụ dùng ngoài app phải thu tiền bằng phương thức khác in-app purchase, ví dụ Apple Pay hoặc nhập thẻ; mục 3.1.1 (bắt buộc IAP) dành cho việc mở khoá tính năng trong app. Chuyến xe và giao đồ ăn diễn ra ngoài đời, nên VíXanh hợp lệ và không qua IAP. Mục 3.1.5 hiện nói về tiền mã hoá, không liên quan. Google Play Payments policy tương tự: billing của Play không dùng cho hàng hoá hoặc dịch vụ vật lý, và trang chính sách ghi vận chuyển, giao đồ ăn là dịch vụ vật lý. Ghi nhận ở 11/2026; kiểm lại tại thời điểm phát hành.

Hình 1 là cả luồng. Chú ý mũi tên 5 và 6: kết quả thật đến server qua webhook, còn app chỉ nhận một đường link.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 1. Luồng thanh toán giữa bốn bên. 1: app gửi yêu cầu tạo phiên kèm Idempotency-Key tới server Tới Nơi. 2: server trả mã phiên và địa chỉ trang cổng. 3: app mở địa chỉ đó trong trình duyệt trong app. 4: khách làm việc với trang VíXanh trong trình duyệt. 5: VíXanh gửi webhook có chữ ký tới server, đây là kết quả thật. 6: trình duyệt mở link toinoi://payment/return về app, chỉ là tín hiệu. 7: app hỏi lại server trạng thái phiên. 8: server trả trạng thái.">
<defs>
<marker id="f39a-n" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker>
<marker id="f39a-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker>
<marker id="f39a-t" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="25" y="10" width="170" height="50" rx="8" class="f-pk-soft s-pk"/><text x="110" y="31" text-anchor="middle" class="t-b">App Tới Nơi</text><text x="110" y="49" text-anchor="middle" class="t-small">Flutter</text>
<rect x="255" y="10" width="170" height="50" rx="8" class="f-phys-soft s-phys"/><text x="340" y="31" text-anchor="middle" class="t-b">Server Tới Nơi</text><text x="340" y="49" text-anchor="middle" class="t-small">Payment &amp; Wallet</text>
<rect x="485" y="10" width="170" height="50" rx="8" class="f-pk-soft s-pk"/><text x="570" y="31" text-anchor="middle" class="t-b">Trình duyệt trong app</text><text x="570" y="49" text-anchor="middle" class="t-small">Custom Tabs, Safari VC</text>
<rect x="715" y="10" width="150" height="50" rx="8" class="f-warn-soft s-warn"/><text x="790" y="31" text-anchor="middle" class="t-b">VíXanh</text><text x="790" y="49" text-anchor="middle" class="t-small">ví hư cấu</text>
<line x1="110" y1="60" x2="110" y2="425" class="s-line dash"/>
<line x1="340" y1="60" x2="340" y2="425" class="s-line dash"/>
<line x1="570" y1="60" x2="570" y2="425" class="s-line dash"/>
<line x1="790" y1="60" x2="790" y2="425" class="s-line dash"/>
<text x="120" y="94" class="t-small">1. tạo phiên (Idempotency-Key)</text>
<line x1="112" y1="100" x2="338" y2="100" class="s-pk" stroke-width="2" marker-end="url(#f39a-n)"/>
<text x="136" y="134" class="t-small">2. mã phiên + địa chỉ trang cổng</text>
<line x1="338" y1="140" x2="112" y2="140" class="s-pk" stroke-width="2" marker-end="url(#f39a-n)"/>
<text x="124" y="179" class="t-small">3. mở địa chỉ trang cổng (https)</text>
<line x1="112" y1="185" x2="568" y2="185" class="s-pk" stroke-width="2" marker-end="url(#f39a-n)"/>
<text x="680" y="219" text-anchor="middle" class="t-small">4. khách trả trên trang ví</text>
<line x1="572" y1="225" x2="788" y2="225" class="s-pk" stroke-width="2" marker-start="url(#f39a-n)" marker-end="url(#f39a-n)"/>
<text x="372" y="259" class="t-small">5. webhook có chữ ký: kết quả thật</text>
<line x1="788" y1="265" x2="342" y2="265" class="s-phys dash" stroke-width="2" marker-end="url(#f39a-t)"/>
<text x="455" y="297" text-anchor="middle" class="t-small t-warn">6. link toinoi://payment/return</text>
<line x1="568" y1="305" x2="112" y2="305" class="s-warn" stroke-width="2" marker-end="url(#f39a-w)"/>
<text x="455" y="324" text-anchor="middle" class="t-small t-warn">chỉ là tín hiệu</text>
<text x="124" y="354" class="t-small">7. hỏi lại trạng thái phiên</text>
<line x1="112" y1="360" x2="338" y2="360" class="s-pk" stroke-width="2" marker-end="url(#f39a-n)"/>
<text x="136" y="394" class="t-small">8. trạng thái theo server</text>
<line x1="338" y1="400" x2="112" y2="400" class="s-phys" stroke-width="2" marker-end="url(#f39a-t)"/>
</svg>
</div>
<figcaption>Hình 1. Bốn bên của một lần thanh toán. App chỉ nói chuyện với server của Tới Nơi và với trình duyệt; kết quả thật đi từ VíXanh tới server (nét đứt), còn link quay về (mũi tên 6) chỉ cho app biết nên hỏi lại.</figcaption>
</figure>

## Mở cổng: WebView hay trình duyệt trong app

`webview_flutter` nhúng Android WebView hoặc iOS WKWebView vào widget của app. `url_launcher` với `LaunchMode.inAppBrowserView` mở Custom Tabs (Android) hoặc SFSafariViewController (iOS). Bảng chỉ ghi điều đã đối chiếu với tài liệu hoặc mã nguồn plugin.

| | WebView (`webview_flutter`) | Trình duyệt trong app (`url_launcher`) |
|---|---|---|
| App thấy gì | Mọi điều hướng, chạy được JavaScript của app | Apple: tương tác không hiện với app, app không đọc được AutoFill, lịch sử, dữ liệu web; có ô địa chỉ chỉ đọc kèm chỉ báo an toàn |
| Cookie, đăng nhập sẵn | Android: WebView không chia sẻ trạng thái với trình duyệt | Android: Custom Tabs chia sẻ cookie jar và autofill. iOS: chia sẻ dữ liệu với Safari thì tài liệu chỉ sang `ASWebAuthenticationSession` |
| Việc app phải làm | Tự chặn điều hướng ra ngoài, tự xử lý lỗi TLS | Tự đóng tab sau khi quay về: plugin đóng được trên iOS, không đóng được Custom Tabs trên Android |

Ta chọn **trình duyệt trong app**. Một: với WebView, mã của app và mọi thư viện trong app đọc được điều khách gõ vào trang ví; với SFSafariViewController thì không. Hai: khách thấy địa chỉ thật nên khó bị lừa bằng trang giả. Ba, riêng Android: khách đã đăng nhập ví trên trình duyệt thì khỏi đăng nhập lại, và Custom Tabs dùng Safe Browsing. Tài liệu Android nói WebView hợp khi app chỉ chứa nội dung của mình hoặc cần chèn JavaScript; ta không thuộc hai trường hợp đó.

Lớp bọc chỉ mở địa chỉ `https` của host đã biết và chuyển lỗi của plugin thành giá trị `false`.

### checkout_launcher.dart · 39-payment-flow/lib/checkout_launcher.dart (trích)

```dart
final class CheckoutUrlPolicy {
  const CheckoutUrlPolicy({required this.trustedHosts});

  /// Host hư cấu của cổng VíXanh.
  static const vixanh = CheckoutUrlPolicy(trustedHosts: {'pay.vixanh.example'});

  final Set<String> trustedHosts;

  bool allows(Uri url) =>
      url.scheme == 'https' && trustedHosts.contains(url.host);
}
// ...
final class UrlLauncherCheckout implements CheckoutLauncher {
  UrlLauncherCheckout({
    this.policy = CheckoutUrlPolicy.vixanh,
    this._launch = _defaultLaunch,
    this._supportsClose = supportsCloseForLaunchMode,
    this._closeView = closeInAppWebView,
  });

  final CheckoutUrlPolicy policy;
  final LaunchFn _launch;
  final SupportsCloseFn _supportsClose;
  final Future<void> Function() _closeView;

  @override
  Future<bool> open(Uri url) async {
    if (!policy.allows(url)) return false;
    try {
      return await _launch(url, mode: LaunchMode.inAppBrowserView);
    } on PlatformException {
      // Không mở được thì plugin ném PlatformException thay vì trả false:
      // Android (ACTIVITY_NOT_FOUND, sau khi đã thử cả Custom Tabs lẫn
      // WebViewActivity) và iOS (SFSafariViewController load lỗi).
      return false;
    }
  }

  /// iOS đóng được SFSafariViewController; Android thì plugin chỉ hỗ trợ đóng
  /// cho `inAppWebView`, không đóng được Custom Tabs.
  @override
  Future<void> close() async {
    try {
      if (await _supportsClose(LaunchMode.inAppBrowserView)) await _closeView();
    } on PlatformException {
      // Đóng là việc làm cho gọn: lỗi ở đây không được làm hỏng luồng.
    }
  }
}
```

Từ mã nguồn plugin: trên Android, không mở được Custom Tab thì plugin rơi về `WebViewActivity` của nó (một WebView, không phải trình duyệt; `supportsLaunchMode` cho biết trước nhưng cần `<queries>`), và chỉ khi cả hai thất bại `launchUrl` mới ném `PlatformException`. iOS cũng ném `PlatformException` khi SFSafariViewController load lỗi. `supportsCloseForLaunchMode` trả `true` trên iOS, `false` trên Android cho Custom Tabs, nên `close()` chỉ đóng khi nền tảng cho phép; `handleReturn` gọi nó trước khi hỏi server để tab không che màn hình kết quả.

Nếu vẫn dùng WebView, app tự gánh việc chặn. Trong `webview_flutter_android` 4.14.1 chỉ điều hướng của frame chính chặn được, nên iframe trong trang cổng không bị kiểm. `judgeNavigation` là phần thuần của phương án này, có test; `WebViewCheckoutPage` chỉ được analyze.

### webview_checkout.dart · 39-payment-flow/lib/webview_checkout.dart (trích)

```dart
/// Với WebView, chính app phải tự chặn điều hướng ra ngoài danh sách tin cậy.
/// Hàm thuần để test được mà không cần nền tảng.
WebViewVerdict judgeNavigation(Uri target, CheckoutUrlPolicy policy) {
  if (PaymentReturn.matches(target)) return WebViewVerdict.returnToApp;
  return policy.allows(target) ? WebViewVerdict.allow : WebViewVerdict.block;
}
```

## Idempotency key: sinh ở client, giữ qua các lần thử

Bài 20 đã nói: hết thời gian chờ không có nghĩa server chưa làm. Client không phân biệt được "chưa tới" với "đã tới, phản hồi rơi", thử lại, và server tạo phiên thứ hai ([phía server](/articles/dotnet10-transaction-concurrency-idempotency) có bài riêng). Hình 2 vẽ trường hợp đó và cách khoá chặn lỗi này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 395" role="img" aria-label="Hình 2. Phản hồi tạo phiên bị rơi. App sinh key-1 và lưu vào kho rồi gửi POST kèm key-1. Server chưa thấy key-1 nên tạo phiên S1 và trả 201, nhưng phản hồi bị rơi. App chuyển sang CreateFailed và giữ key-1. App gửi lại POST với cùng key-1. Server đã thấy key-1 nên trả lại S1 với mã 200 và không tạo phiên mới. Nếu mỗi lần thử dùng khoá mới, server sẽ tạo hai phiên S1 và S2.">
<defs>
<marker id="f39c-n" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker>
<marker id="f39c-t" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="140" y="10" width="180" height="44" rx="8" class="f-pk-soft s-pk"/><text x="230" y="30" text-anchor="middle" class="t-b">App Tới Nơi</text><text x="230" y="46" text-anchor="middle" class="t-small">PaymentFlow + kho</text>
<rect x="560" y="10" width="180" height="44" rx="8" class="f-phys-soft s-phys"/><text x="650" y="30" text-anchor="middle" class="t-b">Server Tới Nơi</text><text x="650" y="46" text-anchor="middle" class="t-small">Payment &amp; Wallet</text>
<line x1="230" y1="54" x2="230" y2="325" class="s-line dash"/>
<line x1="650" y1="54" x2="650" y2="325" class="s-line dash"/>
<rect x="10" y="72" width="205" height="46" rx="8" class="f-pk-soft s-pk"/><text x="20" y="92" class="t-b">Sinh key-1, lưu vào kho</text><text x="20" y="108" class="t-small">trước khi gửi request</text>
<text x="440" y="89" text-anchor="middle" class="t-small">POST /payments/sessions</text>
<line x1="232" y1="95" x2="648" y2="95" class="s-pk" stroke-width="2" marker-end="url(#f39c-n)"/>
<text x="440" y="112" text-anchor="middle" class="t-mono">Idempotency-Key: key-1</text>
<rect x="670" y="128" width="200" height="46" rx="8" class="f-phys-soft s-phys"/><text x="680" y="148" class="t-b">Chưa thấy key-1</text><text x="680" y="164" class="t-small">tạo phiên S1, trả 201</text>
<line x1="648" y1="165" x2="440" y2="165" class="s-warn dash" stroke-width="2"/>
<text x="424" y="171" text-anchor="middle" class="t-b t-warn">×</text>
<text x="440" y="188" text-anchor="middle" class="t-small t-warn">201 + S1: phản hồi bị rơi</text>
<rect x="10" y="198" width="205" height="46" rx="8" class="f-warn-soft s-warn"/><text x="20" y="218" class="t-b">CreateFailed</text><text x="20" y="234" class="t-small">giữ key-1 (đã lưu trong kho)</text>
<text x="440" y="244" text-anchor="middle" class="t-small">POST /payments/sessions</text>
<line x1="232" y1="250" x2="648" y2="250" class="s-pk" stroke-width="2" marker-end="url(#f39c-n)"/>
<text x="440" y="267" text-anchor="middle" class="t-mono">Idempotency-Key: key-1</text>
<rect x="670" y="272" width="200" height="46" rx="8" class="f-phys-soft s-phys"/><text x="680" y="292" class="t-b">Thấy key-1 rồi</text><text x="680" y="308" class="t-small">trả lại S1, không tạo S2</text>
<text x="440" y="304" text-anchor="middle" class="t-small">200 + S1 (cùng phiên)</text>
<line x1="648" y1="310" x2="232" y2="310" class="s-phys" stroke-width="2" marker-end="url(#f39c-t)"/>
<rect x="10" y="340" width="860" height="44" rx="8" class="f-warn-soft s-warn"/><text x="24" y="360" class="t-b">Nếu mỗi lần thử sinh khoá mới (key-1, rồi key-2)</text><text x="24" y="376" class="t-small">server tạo cả S1 lẫn S2 và khách có thể bị thu hai lần cho một chuyến.</text>
</svg>
</div>
<figcaption>Hình 2. Phản hồi tạo phiên bị rơi. Khoá sinh một lần, lưu trước khi gửi và dùng lại ở lần thử thứ hai nên server trả lại đúng phiên S1.</figcaption>
</figure>

Lab tuân theo ba quy tắc. Khoá sinh ở client **trước** khi gọi và lưu vào kho ngay, để app chết giữa chừng vẫn còn khoá. Khoá chỉ dùng lại khi lần trước chưa nhận được phiên cho cùng chuyến và số tiền. Sau một kết quả cuối không thành công, thanh toán lại là lần mới với khoá mới: khoá cũ chỉ nhận lại phiên đã huỷ (test của server giả).

### payment_flow.dart · 39-payment-flow/lib/payment_flow.dart (trích)

```dart
  /// Bắt đầu thanh toán một chuyến.
  ///
  /// Khoá idempotency chỉ được dùng lại khi lần trước chưa nhận được phiên
  /// (mất mạng, app bị đóng) cho cùng chuyến và số tiền. Sau một kết quả
  /// cuối không thành công, khách thử lại là một lần thanh toán mới và có khoá
  /// mới.
  Future<void> start({required String tripId, required Money amount}) async {
    if (_starting) return;
    _starting = true;
    try {
      await restore();
      if (!_canStart) return;

      final stored = await _store.load();
      final reuse =
          stored != null &&
          stored.session == null &&
          stored.request.tripId == tripId &&
          stored.request.amount == amount;
      final request = reuse
          ? stored.request
          : PaymentRequest(
              tripId: tripId,
              amount: amount,
              idempotencyKey: _newKey(),
            );
      await _store.save(PendingPayment(request));
      _set(PaymentCreating(request));

      final PaymentSession session;
      try {
        session = await _gateway.createSession(
          tripId: tripId,
          amount: amount,
          idempotencyKey: request.idempotencyKey,
        );
      } on PaymentGatewayException catch (e) {
        if (!e.retryable) await _store.clear();
        _set(PaymentCreateFailed(request, e));
        return;
      }
      await _store.save(PendingPayment(request, session));
      await _openCheckout(session);
    } finally {
      _starting = false;
    }
  }
```

Server giả của lab (một `FakeApiModule` trong test, không sửa `tools/fake_api`) dùng header `Idempotency-Key` như `POST /trips` ở bài 19: cùng khoá và nội dung thì trả phiên cũ, khác nội dung thì 409. Server tự tính tiền từ chuyến, client chỉ gửi số tiền để đối chiếu (lệch thì 422), và chuyến đã trả xong thì 409 dù khoá mới. Lỗi 4xx không thử lại được nên `start` xoá bản dang dở; lỗi mạng thì giữ lại.

### payment_flow_test.dart · 39-payment-flow/test/payment_flow_test.dart (trích)

```dart
    test(
      'phản hồi tạo phiên bị rơi: thử lại cùng khoá, server chỉ có một phiên',
      () async {
        h.network.loseResponses = 1;

        await flow.start(tripId: trip.id, amount: trip.fare);

        expect(flow.state, isA<PaymentCreateFailed>());
        expect((flow.state as PaymentCreateFailed).retryable, isTrue);
        expect(h.server.sessionCount, 1, reason: 'server đã tạo trước khi rơi');

        await flow.start(tripId: trip.id, amount: trip.fare);

        expect(h.server.keysReceived, ['key-1', 'key-1']);
        expect(h.server.sessionCount, 1);
        expect(flow.state, isA<PaymentAwaitingCheckout>());
        expect(flow.state.session!.id, h.server.lastSessionId);
      },
    );
```

## Deep link quay về: tín hiệu, không phải bằng chứng

Custom scheme không chứng minh người gọi là VíXanh, và ai cũng đăng ký được. Android: app khác đăng ký cùng scheme vẫn kích hoạt được hộp chọn app. Apple: nhiều app cùng scheme thì không xác định được app nào nhận link, và tài liệu khuyên mạnh dùng universal link. Bài 23 cũng chỉ khuyên dùng custom scheme cho việc không nhạy cảm. Hậu quả: một app lạ có thể nhận mã phiên và `status`, nên chỉ có mã phiên thì chưa đủ để xem kết quả: `GET` trạng thái phải kèm token đăng nhập của khách (bài 24; server giả của lab chưa có bước này). `PaymentReturn` chỉ nhận ra đúng `toinoi://payment/return` và lấy mã phiên; `status` nằm trong `claimed` để ghi log, không dòng code nào dùng nó để kết luận.

### payment_return.dart · 39-payment-flow/lib/payment_return.dart (trích)

```dart
  static const scheme = 'toinoi';
  static const host = 'payment';
  static const path = '/return';

  final String sessionId;

  /// Giá trị tham số `status` nếu có (`success`, `cancel`, ...), chưa xác thực.
  final String? claimed;

  /// Đúng scheme, host và đường dẫn của link quay về (chưa xét tham số).
  static bool matches(Uri uri) =>
      uri.scheme == scheme && uri.host == host && uri.path == path;

  /// Trả `null` nếu không phải link quay về hoặc thiếu mã phiên.
  static PaymentReturn? tryParse(Uri uri) {
    if (!matches(uri)) return null;
    final session = uri.queryParameters['session'];
    if (session == null || session.isEmpty) return null;
    return PaymentReturn(
      sessionId: session,
      claimed: uri.queryParameters['status'],
    );
  }
```

`handleReturn` coi link là yêu cầu "hãy hỏi lại". Link không phải của thanh toán, hoặc của phiên lạ (kể cả khi chưa có phiên nào, như lúc còn đang tạo), bị bỏ qua, không gọi mạng. Link đến hai lần chỉ chạy một vòng hỏi, vì vòng đang chạy được dùng chung. Link đến sau khi đã chốt không làm gì.

### payment_flow.dart · 39-payment-flow/lib/payment_flow.dart (trích)

```dart
  /// Nhận link quay về từ cổng. Tham số của link chỉ dùng để tìm đúng phiên;
  /// kết quả luôn do server quyết.
  Future<ReturnHandling> handleReturn(Uri link) async {
    final ret = PaymentReturn.tryParse(link);
    if (ret == null) return ReturnHandling.notPaymentLink;
    await restore();
    final session = _state.session;
    if (session == null || session.id != ret.sessionId) {
      return ReturnHandling.unknownSession;
    }
    switch (_state) {
      case PaymentAwaitingCheckout() || PaymentPendingConfirmation():
        // Tab trình duyệt (nếu đóng được) che màn hình kết quả: đóng trước.
        await _launcher.close();
        await _confirm(session);
      case PaymentConfirming():
        await _confirming;
      default:
        break; // đã chốt: link thứ hai không làm gì
    }
    return ReturnHandling.handled;
  }

  Future<void> _confirm(PaymentSession session) =>
      _confirming ??= _runConfirm(session).whenComplete(() {
        _confirming = null;
      });
```

Phía native: Android cần bộ lọc `VIEW` kèm `DEFAULT` và `BROWSABLE` (thiếu `BROWSABLE` thì link bấm từ trình duyệt không tới app, theo tài liệu Android); iOS đăng ký scheme trong `CFBundleURLTypes`. `native_config_test.dart` kiểm các file trong `native-config/`: scheme, host, path trùng hằng của `PaymentReturn`, mỗi bộ lọc một `<data>`, `MAIN`/`LAUNCHER` không lẫn `<data>`.

### AndroidManifest.xml · 39-payment-flow/native-config/android/app/src/main/AndroidManifest.xml (trích)

```xml
            <!-- Link quay về từ cổng thanh toán: toinoi://payment/return.
                 BROWSABLE là bắt buộc để link bấm từ trình duyệt (kể cả
                 Custom Tabs) tới được app. Bộ lọc riêng, không trộn với
                 toinoi://trip của bài 23. -->
            <intent-filter>
                <action android:name="android.intent.action.VIEW"/>
                <category android:name="android.intent.category.DEFAULT"/>
                <category android:name="android.intent.category.BROWSABLE"/>
                <data android:scheme="toinoi" android:host="payment" android:path="/return"/>
            </intent-filter>
```

**Khuyến nghị cho production**: dùng Universal Link (iOS) và App Link (Android), ví dụ `https://toinoi.example/payment/return?session=...`, gắn với tên miền đã được hệ điều hành xác minh (cấu hình như bài 23). Lab dùng custom scheme để minh hoạ, vì xác minh cần tên miền thật. Link xác minh chỉ chứng minh nó đến từ đâu, không chứng minh khách đã trả.

## Máy trạng thái: hỏi server có giới hạn, mất mạng thì chờ

`PaymentState` là sealed class. Chỉ bốn lớp con của `PaymentFinished` là kết quả; mọi trạng thái khác nghĩa là ta chưa biết (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 390" role="img" aria-label="Hình 3. Các trạng thái của PaymentFlow. Idle chuyển sang Creating khi gọi start. Creating chuyển sang AwaitingCheckout khi có phiên, hoặc sang CreateFailed khi lỗi mạng; từ CreateFailed thử lại với cùng khoá quay về Creating. AwaitingCheckout chuyển sang Confirming khi có deep link hoặc app trở lại foreground. Confirming chuyển sang PendingConfirmation khi mất mạng hoặc hết lượt hỏi; từ PendingConfirmation, kiểm tra lại quay về Confirming. Khi server đã chốt, Confirming chuyển sang một trong bốn trạng thái cuối: Succeeded, Failed, Cancelled, Expired.">
<defs>
<marker id="f39b-n" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker>
<marker id="f39b-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker>
<marker id="f39b-t" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="40" width="130" height="50" rx="8" class="f-pk-soft s-pk"/><text x="35" y="62" class="t-mono t-b">Idle</text><text x="35" y="79" class="t-small">chưa thanh toán</text>
<rect x="205" y="40" width="140" height="50" rx="8" class="f-pk-soft s-pk"/><text x="220" y="62" class="t-mono t-b">Creating</text><text x="220" y="79" class="t-small">xin phiên</text>
<rect x="400" y="40" width="170" height="50" rx="8" class="f-pk-soft s-pk"/><text x="415" y="62" class="t-mono t-b">AwaitingCheckout</text><text x="415" y="79" class="t-small">đã mở cổng, đợi khách</text>
<rect x="625" y="40" width="150" height="50" rx="8" class="f-pk-soft s-pk"/><text x="640" y="62" class="t-mono t-b">Confirming</text><text x="640" y="79" class="t-small">đang hỏi server</text>
<line x1="152" y1="65" x2="203" y2="65" class="s-pk" stroke-width="2" marker-end="url(#f39b-n)"/><text x="177" y="33" text-anchor="middle" class="t-small">start()</text>
<line x1="347" y1="65" x2="398" y2="65" class="s-pk" stroke-width="2" marker-end="url(#f39b-n)"/><text x="372" y="33" text-anchor="middle" class="t-small">có phiên</text>
<line x1="572" y1="65" x2="623" y2="65" class="s-pk" stroke-width="2" marker-end="url(#f39b-n)"/><text x="597" y="33" text-anchor="middle" class="t-small">link, resume</text>
<rect x="205" y="170" width="140" height="50" rx="8" class="f-warn-soft s-warn"/><text x="220" y="192" class="t-mono t-b">CreateFailed</text><text x="220" y="209" class="t-small">chưa có phiên</text>
<line x1="235" y1="92" x2="235" y2="168" class="s-warn" stroke-width="2" marker-end="url(#f39b-w)"/><text x="227" y="135" text-anchor="end" class="t-small">lỗi mạng</text>
<line x1="315" y1="168" x2="315" y2="92" class="s-pk" stroke-width="2" marker-end="url(#f39b-n)"/><text x="323" y="135" class="t-small">thử lại, cùng khoá</text>
<rect x="380" y="170" width="200" height="50" rx="8" class="f-warn-soft s-warn"/><text x="395" y="192" class="t-mono t-b">PendingConfirmation</text><text x="395" y="209" class="t-small">đang chờ xác nhận</text>
<path d="M650 92 L650 195 L582 195" class="f-none s-warn" stroke-width="2" marker-end="url(#f39b-w)"/><text x="642" y="135" text-anchor="end" class="t-small">mất mạng hoặc hết lượt hỏi</text>
<path d="M582 210 L700 210 L700 92" class="f-none s-pk" stroke-width="2" marker-end="url(#f39b-n)"/><text x="708" y="135" class="t-small">Kiểm tra lại</text>
<path d="M777 65 L840 65 L840 268" class="f-none s-phys" stroke-width="2" marker-end="url(#f39b-t)"/><text x="832" y="170" text-anchor="end" class="t-small">server đã chốt</text>
<rect x="20" y="270" width="840" height="105" rx="10" class="f-bg s-line"/>
<text x="36" y="292" class="t-b">Trạng thái cuối do server quyết, không đổi nữa</text>
<rect x="36" y="302" width="190" height="36" rx="8" class="f-phys-soft s-phys"/><text x="50" y="325" class="t-mono t-b">Succeeded</text>
<rect x="240" y="302" width="190" height="36" rx="8" class="f-warn-soft s-warn"/><text x="254" y="325" class="t-mono t-b">Failed</text>
<rect x="444" y="302" width="190" height="36" rx="8" class="f-warn-soft s-warn"/><text x="458" y="325" class="t-mono t-b">Cancelled</text>
<rect x="648" y="302" width="190" height="36" rx="8" class="f-warn-soft s-warn"/><text x="662" y="325" class="t-mono t-b">Expired</text>
<text x="36" y="362" class="t-small">Failed, Cancelled, Expired: bấm thanh toán lại là lần mới, khoá mới.</text>
</svg>
</div>
<figcaption>Hình 3. Các trạng thái của PaymentFlow. Chỉ hàng dưới là kết quả; PendingConfirmation là "chưa biết", và chỉ đi tiếp qua Confirming.</figcaption>
</figure>

Vòng hỏi nằm trong `_runConfirm`. `PollPolicy.standard` hỏi ngay một lần rồi chờ 1, 2, 4, 8 giây giữa các lần tiếp: tối đa 5 lần, 15 giây (số minh hoạ). Trạng thái cuối của server thì chốt luôn. Ba lối ra còn lại đều là "chưa biết", không lối nào là thất bại. Hết lượt mà server vẫn nói `pending`: ví có thể xử lý chậm, webhook có thể đến trễ (hồ sơ §8.3 cho phép `PENDING`). Mất mạng hoặc server lỗi: `unreachable`. Server từ chối trả lời về phiên: `rejected`, màn hình mời liên hệ hỗ trợ.

### payment_flow.dart · 39-payment-flow/lib/payment_flow.dart (trích)

```dart
  Future<void> _runConfirm(PaymentSession session) async {
    _set(PaymentConfirming(session));
    for (var check = 0; ; check++) {
      final PaymentStatus status;
      try {
        status = await _gateway.fetchStatus(session.id);
      } on PaymentGatewayException catch (e) {
        _set(
          PaymentPendingConfirmation(
            session,
            e.retryable ? PendingReason.unreachable : PendingReason.rejected,
          ),
        );
        return;
      }
      if (status.isFinal) {
        _set(switch (status) {
          PaymentStatus.succeeded => PaymentSucceeded(session),
          PaymentStatus.failed => PaymentFailed(session),
          PaymentStatus.cancelled => PaymentCancelled(session),
          PaymentStatus.expired => PaymentExpired(session),
          PaymentStatus.pending => throw StateError('không thể tới đây'),
        });
        await _store.clear();
        return;
      }
      if (check >= poll.waits.length) {
        _set(PaymentPendingConfirmation(session, PendingReason.stillPending));
        return;
      }
      await _delay(poll.waits[check]);
    }
  }
}
```

Thoát trạng thái chờ bằng nút "Kiểm tra lại" (có cả ở `AwaitingCheckout`) hoặc khi app trở lại foreground. Cách sau quan trọng: đóng tab bằng Done hoặc Back thì không có link nào được gửi, và nếu chỉ đợi link, app kẹt ở "đợi khách". `PaymentScreen` nối `AppLifecycleListener` với `onAppResumed`; một test đẩy chuỗi vòng đời tới `resumed` và kiểm màn hình đổi. Lab chưa nghe thay đổi kết nối: mất mạng thì khách bấm nút.

### payment_screen.dart · 39-payment-flow/lib/payment_screen.dart (trích)

```dart
    // Khách có thể đóng tab bằng Done hoặc Back: không có deep link nào được
    // gửi, nên khi app trở lại foreground ta hỏi lại server.
    _lifecycle = AppLifecycleListener(
      onStateChange: (state) {
        if (state == AppLifecycleState.resumed) {
          unawaited(
            ref.read(paymentControllerProvider.notifier).onAppResumed(),
          );
        }
      },
    );
  }
```

Test sau chính là test case ở đầu bài.

### payment_flow_test.dart · 39-payment-flow/test/payment_flow_test.dart (trích)

```dart
    test(
      'status=success nhưng server nói chưa: hỏi có giới hạn rồi chờ xác nhận',
      () async {
        final session = await startAndGetSession(flow);

        final handling = await flow.handleReturn(
          returnLink(session.id, status: 'success'),
        );

        expect(handling, ReturnHandling.handled);
        expect(
          flow.state,
          PaymentPendingConfirmation(session, PendingReason.stillPending),
        );
        // Hỏi ngay một lần rồi chờ theo lịch: 1 + 4 lần, không hơn.
        expect(h.statusRequests, PollPolicy.standard.waits.length + 1);
        expect(h.waited, PollPolicy.standard.waits);
      },
    );
```

## App bị kill: trạng thái bền qua PaymentStore

Hệ điều hành có thể kill app khi khách đang ở trang ví, và khách có thể trả xong lúc đó. `PaymentStore` ghi yêu cầu kèm khoá (trước khi gọi server) và phiên (ngay khi nhận), rồi xoá khi có kết quả cuối. Giá trị là chuỗi JSON không có bí mật; bản thật dùng drift hoặc secure storage của bài 22. Lab dùng bản trong bộ nhớ nhưng vẫn lưu chuỗi JSON, để test chứng minh dữ liệu đi qua được kho bền.

### payment_store.dart · 39-payment-flow/lib/payment_store.dart (trích)

```dart
abstract interface class PaymentStore {
  Future<PendingPayment?> load();
  Future<void> save(PendingPayment pending);
  Future<void> clear();
}
```

`restore()` đọc kho. Chỉ có yêu cầu mà chưa có phiên thì ta không biết server đã tạo hay chưa: state là `CreateFailed`, thử lại bằng đúng khoá cũ. Đã có phiên thì hỏi server ngay. `start` và `handleReturn` đều `await restore()` trước, nên app mở lại bằng chính deep link vẫn đi đúng đường: nạp kho, khớp mã phiên, hỏi server.

### payment_flow_test.dart · 39-payment-flow/test/payment_flow_test.dart (trích)

```dart
    test('khách trả xong lúc app không chạy: mở lại thấy thành công', () async {
      final session = await startAndGetSession(flow);
      flow.dispose(); // kill
      h.server.settle(session.id, PaymentStatus.succeeded);

      final reopened = h.flow();
      await reopened.restore();

      expect(reopened.state, PaymentSucceeded(session));
      expect(h.store.raw, isNull);
    });
```

Lớp Riverpod là cầu nối mỏng: `build()` gọi `restore()` khi app mở. `build()` chạy lại khi provider bị `invalidate` hoặc dependency được watch đổi, nên `_flow` là `late` chứ không phải `late final`; hai test `invalidate` kiểm điều này (với `late final` cả hai ném `LateInitializationError`).

### payment_providers.dart · 39-payment-flow/lib/payment_providers.dart (trích)

```dart
class PaymentController extends Notifier<PaymentState> {
  // Không phải `late final`: `build()` chạy lại khi provider bị invalidate
  // hoặc khi một dependency được watch đổi.
  late PaymentFlow _flow;

  @override
  PaymentState build() {
    _flow = ref.watch(paymentFlowProvider);
    final sub = _flow.changes.listen((next) => state = next);
    ref.onDispose(sub.cancel);
    // Có lần thanh toán dang dở từ lần mở app trước thì nạp và hỏi server.
    unawaited(_flow.restore());
    return _flow.state;
  }
  // ...
  /// Gọi khi app trở lại foreground: khách có thể đóng tab bằng nút Done hoặc
  /// Back, khi đó không có deep link nào được gửi.
  Future<void> onAppResumed() => _flow.refresh();
```

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi làm thanh toán qua cổng ngoài</span>
<ul>
<li><strong>Coi <code>status=success</code> trên link là thành công.</strong> Dấu hiệu: tự mở link bằng <code>adb</code> là app báo thành công. Nguyên nhân: ai cũng tạo được link. Cách sửa: link chỉ để tìm phiên, kết quả lấy từ server.</li>
<li><strong>Sinh khoá idempotency mới ở mỗi lần thử.</strong> Dấu hiệu: một chuyến có hai phiên sau một lần mất mạng. Nguyên nhân: khoá sinh trong hàm gọi API. Cách sửa: sinh một lần, lưu vào kho, giữ tới khi có phiên.</li>
<li><strong>Giữ khoá cũ sau khi huỷ hoặc hết hạn.</strong> Dấu hiệu: bấm thanh toán lại thì nhận ngay phiên đã huỷ. Nguyên nhân: cùng khoá thì server trả phiên cũ. Cách sửa: khi có kết quả cuối không thành công thì kết thúc lần thanh toán đó; lần sau dùng khoá mới.</li>
<li><strong>Chỉ chờ deep link.</strong> Dấu hiệu: khách bấm Done, app đứng ở "đợi khách". Nguyên nhân: đóng tab không gửi link. Cách sửa: hỏi server khi app trở lại foreground và có nút Kiểm tra lại.</li>
<li><strong>Báo thất bại khi mất mạng.</strong> Dấu hiệu: khách thấy "thất bại", trả lần hai và bị trừ hai lần. Nguyên nhân: lẫn "server nói không" với "không hỏi được". Sửa: mất mạng và hết lượt hỏi là <code>PaymentPendingConfirmation</code>; chỉ server mới làm ra <code>PaymentFailed</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/39-payment-flow
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `dio` 5.11.1, `url_launcher` 6.3.3, `webview_flutter` 4.14.1: `flutter analyze` báo 0 issue, `flutter test` chạy 84 test xanh, không test nào chờ thời gian thật hay gọi mạng.

Mức kiểm chứng theo §3.3 là **trên thiết bị**, nhưng máy viết bài không có thiết bị, nên chỉ phần logic được test tự động. Nhóm `PaymentFlow` (34 test) chứng minh đường chính, link giả, huỷ, hết hạn, link đến hai lần, mất mạng, khoá giữ qua các lần thử, đóng tab và mở lại sau khi bị kill. Nhóm server giả (14 test) chứng minh idempotency và đối chiếu số tiền. Các nhóm còn lại kiểm đọc link, `UrlLauncherCheckout` với hàm giả, `judgeNavigation`, controller và màn hình Riverpod, `native-config/`.

`WebViewCheckoutPage` chỉ được analyze (kiểm kiểu và tên API, không chạy). **Chưa chạy trên thiết bị**: mở Custom Tabs hoặc SFSafariViewController thật; hệ điều hành đưa `toinoi://payment/return` về app; đóng SFSafariViewController có phát `resumed` hay không (chưa rõ, nên nút Kiểm tra lại không bỏ được); `closeInAppWebView` có đóng đúng tab trên iOS; `launchMode="singleTop"` (mặc định của template Flutter) có tạo thêm `MainActivity` khi Custom Tab nằm trên stack hay không (tài liệu Android: instance cũ chỉ nhận intent khi ở đỉnh stack; cần thử `singleTask`); trang cổng của ví thật. **Chưa làm**: server và webhook thật, kho bền thật, Universal Link và App Link, hoàn tiền, Apple Pay và Google Pay.

## Bài tập

1. Server thêm trạng thái `authorized` (đã giữ tiền, chờ thu). Bạn phải sửa những chỗ nào trong lab, và trình biên dịch giúp được đến đâu?
2. Khách bấm Done trên tab thanh toán, không có link nào được gửi. App ở trạng thái nào, và điều gì đưa nó đi tiếp?
3. Link quay về tới đúng lúc `PaymentFlow` còn ở `PaymentCreating`. Kết quả là gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Thêm giá trị vào <code>PaymentStatus</code>. Analyzer báo <code>non_exhaustive_switch_expression</code> ở <code>switch</code> trong <code>_runConfirm</code> (đã thử). Nhưng <code>isFinal</code> là <code>this != pending</code>, nên <code>authorized</code> mặc nhiên trở thành trạng thái cuối và không có lỗi biên dịch: phải sửa tay, rồi quyết định nó vào <code>PendingConfirmation</code> hay một trạng thái riêng.</p>
<p><strong>2.</strong> <code>PaymentAwaitingCheckout</code>. Có hai lối đi tiếp: <code>AppLifecycleListener</code> trong <code>PaymentScreen</code> gọi <code>refresh()</code> khi app về <code>resumed</code> (test <code>đóng tab bằng nút Done</code> trong <code>payment_screen_test.dart</code> đẩy chuỗi vòng đời), hoặc khách bấm Kiểm tra lại. Trên iOS thật, tab đóng có phát <code>resumed</code> hay không thì chưa kiểm.</p>
<p><strong>3.</strong> <code>ReturnHandling.unknownSession</code>, state vẫn là <code>PaymentCreating</code> vì chưa có phiên để khớp mã; phiên vẫn được tạo bình thường sau đó (test <code>link đến khi còn đang tạo phiên</code>).</p>
</details>

Bài trước: [Tác vụ nền: workmanager, vị trí nền cho app tài xế và chính sách store](/articles/flutter-background-work-workmanager-vi-tri-nen-app-tai-xe). Bài tiếp: [Platform channel, Pigeon và FFI: gọi native khi plugin không đủ](/articles/flutter-platform-channel-pigeon-ffi-goi-native).

## Tài liệu gốc

- [App Store Review Guidelines](https://developer.apple.com/app-store/review/guidelines/) (developer.apple.com), mục 3.1.1 và 3.1.3(e); [SFSafariViewController](https://developer.apple.com/documentation/safariservices/sfsafariviewcontroller) (developer.apple.com).
- [Payments policy](https://support.google.com/googleplay/android-developer/answer/9858738) (support.google.com/googleplay).
- [Overview of Android Custom Tabs](https://developer.android.com/develop/ui/views/layout/webapps/overview-of-android-custom-tabs) và [Create deep links to app content](https://developer.android.com/training/app-links/deep-linking) (developer.android.com).
- [url_launcher](https://pub.dev/packages/url_launcher) và [webview_flutter](https://pub.dev/packages/webview_flutter) (pub.dev).
- [Deep linking](https://docs.flutter.dev/ui/navigation/deep-linking) (docs.flutter.dev).
- [Defining a custom URL scheme for your app](https://developer.apple.com/documentation/xcode/defining-a-custom-url-scheme-for-your-app) (developer.apple.com) và [Activity element, launchMode](https://developer.android.com/guide/topics/manifest/activity-element) (developer.android.com).
- [Providers](https://riverpod.dev/docs/concepts2/providers) (riverpod.dev).

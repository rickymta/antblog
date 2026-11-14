---
slug: flutter-authentication-otp-jwt-refresh-token-secure-storage
title: "Authentication: OTP, JWT, refresh token rotation và interceptor"
summary: "Bài 24: đăng nhập app khách Tới Nơi bằng OTP, access và refresh token JWT, rotation phát hiện dùng lại, SessionManager chỉ refresh một lần cho nhiều request song song, AuthInterceptor của dio, đăng xuất và vì sao client không tin role trong token."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, authentication, jwt, dio, riverpod, security, ios, android]
language: vi
---

Khách đứng trong hầm gửi xe, mở app Tới Nơi sau hai tiếng. Màn hình chính gọi cùng lúc năm API. Access token đã hết hạn, nên cả năm request nhận 401. Nếu mỗi request tự đi xin token mới, server thấy cùng một refresh token được đưa ra năm lần, kết luận có kẻ dùng token bị đánh cắp, và thu hồi cả phiên. Khách bị đăng xuất giữa lúc đặt xe về nhà.

Bài 23 đã có guard đăng nhập nhưng đăng nhập vẫn là giả. Bài này làm phần thật: OTP, hai loại token, interceptor của `dio` và đăng xuất. Đây là bài 24 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 23 về go_router](/articles/flutter-go-router-deep-link-shellroute-guard), [bài 22](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline) về `TokenStore` và [bài 20](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong) về `RetryInterceptor`. Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); thời hạn token và bảng giá là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Năm request cùng nhận 401. Mỗi request chạy <code>onError</code> riêng: đọc refresh token trong kho, gọi <code>POST /auth/refresh</code>, lưu cặp mới, rồi gửi lại. Server dùng rotation. Mọi thứ có chạy đúng không?</p>
<details><summary>Xem đáp án</summary><p><strong>Không.</strong> Khi mạng chậm, cả năm request cùng đưa một refresh token cũ. Lần đầu thành công, bốn lần sau bị coi là dùng lại, server thu hồi cả họ token, và code ngây thơ xoá kho. Test <code>refresh riêng từng request + rotation</code>: <code>refreshCalls</code> là 5, <code>revokedFamilies</code> là 1, kho rỗng. Mục 4 cho cách sửa.</p></details>
</div>

## 1. Luồng OTP và hai loại token

Khách nhập số điện thoại, nhận mã OTP, nhập mã; server trả hai token (Hình 1). **Access token** đi kèm mọi request, sống ngắn. **Refresh token** chỉ dùng để đổi lấy cặp mới, sống dài, và là thứ đáng bảo vệ nhất trên máy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 1. Luồng đăng nhập OTP. App gửi POST /auth/otp/request với số điện thoại và loại app. Server trả requestId và hạn của mã. Khách nhập mã, app gửi POST /auth/otp/verify với requestId và mã. Server trả cặp access token và refresh token. App ghi cả hai vào kho an toàn bằng một khoá JSON duy nhất.">
<defs><marker id="fl24-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="16" width="230" height="44" rx="10" class="f-phys-soft s-phys"/><text x="36" y="43" class="t-b">App Tới Nơi</text>
<rect x="630" y="16" width="230" height="44" rx="10" class="f-phys-soft s-phys"/><text x="646" y="43" class="t-b">Server (fake API)</text>
<path d="M135 62 L135 300" class="f-none s-muted" stroke-dasharray="4 5"/>
<path d="M745 62 L745 300" class="f-none s-muted" stroke-dasharray="4 5"/>
<text x="160" y="96" class="t-mono">POST /auth/otp/request</text>
<path d="M135 106 L745 106" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-1-a)"/>
<text x="160" y="124" class="t-small">phone, app: customer</text>
<text x="160" y="158" class="t-mono">requestId, hạn của mã</text>
<path d="M745 168 L135 168" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-1-a)"/>
<text x="160" y="186" class="t-small">mã OTP tới khách qua kênh riêng, không nằm trong response</text>
<text x="160" y="220" class="t-mono">POST /auth/otp/verify</text>
<path d="M135 230 L745 230" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-1-a)"/>
<text x="160" y="248" class="t-small">requestId, code</text>
<text x="160" y="282" class="t-mono">access JWT + refresh token</text>
<path d="M745 292 L135 292" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-1-a)"/>
<rect x="20" y="308" width="840" height="40" rx="10" class="f-pk-soft s-pk"/><text x="36" y="333" class="t-small">App ghi cả hai token vào kho an toàn (một khoá JSON, bài 22) rồi mới đổi state</text>
</svg>
</div>
<figcaption>Hình 1. Đăng nhập OTP. Mũi tên chỉ chiều của request và response. Server fake dùng mã cố định để lab chạy lặp lại; server thật sinh mã ngẫu nhiên cho từng yêu cầu.</figcaption>
</figure>

Phần client nằm trong `AuthApi`, bốn lời gọi trên một `Dio` riêng (lý do ở mục 5). Lỗi ra khỏi lớp này là `ApiFailure` của bài 20: mã sai là `ClientFailure` 422, sai ba lần là 429.

### auth_api.dart · 24-auth-jwt-refresh/lib/auth/auth_api.dart (trích)

```dart
Future<AuthTokens> verifyOtp({
  required String requestId,
  required String code,
}) => guardApi(
  () async => _tokens(
    await _post('/auth/otp/verify', {'requestId': requestId, 'code': code}),
  ),
);

Future<AuthTokens> refresh(String refreshToken) => guardApi(
  () async =>
      _tokens(await _post('/auth/refresh', {'refreshToken': refreshToken})),
);
```

Để lab chạy được, ta thêm `AuthModule` vào `tools/fake_api` (một `FakeApiModule` như ở bài 19) với bốn route `/auth/otp/request`, `/auth/otp/verify`, `/auth/refresh`, `/auth/logout`, bật bằng `--with-auth`; không bật thì fake chạy y như cũ. Mã OTP là hằng `123456` và số điện thoại trong lab là số giả, cả hai **chỉ dành cho fake**. Mã sống 5 phút, dùng một lần, sai ba lần thì bị huỷ.

| Token | Dạng | Hạn (minh hoạ) | Server kiểm bằng |
|---|---|---|---|
| Access | JWT, gắn vào mọi request | 15 phút | chữ ký và `exp` (fake tra bảng đã phát) |
| Refresh | chuỗi opaque, chỉ gửi tới `/auth/refresh` | 30 ngày | bảng token và họ token của server |

## 2. JWT: đọc được, nhưng không tin được

Một JWT là ba đoạn base64url nối bằng dấu chấm: header, payload, chữ ký. Payload là JSON như `{"sub":"U-1001","role":"customer","exp":...}`. Client đọc được nó mà không cần khoá, và đó chính là vấn đề: **ai cũng sửa được payload**. Chỉ bên giữ khoá bí mật mới ký được, và app nằm trong tay người dùng nên kiểm chữ ký ở đó không thay được việc kiểm ở server.

JWT của fake cố ý **không ký**: header ghi `alg: none`, chữ ký để trống. Server fake tra nguyên chuỗi token trong bảng đã phát chứ không đọc payload, nên sửa một ký tự là bị từ chối. Client chỉ đọc payload để hiển thị:

### unverified_claims.dart · 24-auth-jwt-refresh/lib/auth/unverified_claims.dart (trích)

```dart
static UnverifiedClaims? tryParse(String jwt) {
  final parts = jwt.split('.');
  if (parts.length != 3) return null;
  try {
    final json = jsonDecode(
      utf8.decode(base64Url.decode(base64Url.normalize(parts[1]))),
    );
    if (json is! Map<String, dynamic>) return null;
    final sub = json['sub'];
    if (sub is! String || sub.isEmpty) return null;
    final role = json['role'];
    // ... exp, rồi dựng UnverifiedClaims
  } on FormatException {
    return null;
  }
}
```

Test `payload bị sửa (role thành driver) vẫn đọc ra bình thường` cho thấy client đọc ra `driver` từ token bị sửa; test của fake cho thấy server từ chối đúng token đó với 401. Vì vậy `HomeScreen` chỉ in vai trò ra chữ, không có `if (role == 'driver')` nào mở chức năng. Quyền là việc của server.

## 3. Refresh token rotation: dùng lại là báo động

Refresh token sống 30 ngày mà không đổi thì kẻ trộm có 30 ngày. **Rotation** thu hẹp cửa sổ đó: mỗi lần refresh thành công, server phát cặp mới và đánh dấu token vừa dùng là "đã dùng". Mọi token sinh ra từ một lần đăng nhập OTP thuộc một **họ**. Token đã dùng mà xuất hiện lại thì có nghĩa là hai bên cùng cầm nó; server không biết ai là ai nên thu hồi cả họ (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Một họ refresh token. Đăng nhập OTP cấp token R1. Refresh thành công đổi R1 lấy R2 và R1 thành đã dùng. Refresh tiếp đổi R2 lấy R3 và R2 thành đã dùng. R3 là token hiện tại. Nếu ai đó đưa lại R1 đã dùng, server thu hồi cả họ: R3 cũng không còn dùng được, trả 401 và người dùng phải đăng nhập lại bằng OTP.">
<defs><marker id="fl24-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl24-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="16" y="16" width="848" height="150" rx="12" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 5"/><text x="32" y="38" class="t-small">Một họ token: cùng một lần đăng nhập OTP</text>
<rect x="40" y="60" width="200" height="86" rx="10" class="f-surface s-line"/><text x="56" y="86" class="t-b">R1</text><text x="56" y="110" class="t-small">cấp lúc đăng nhập</text><text x="56" y="130" class="t-small">đã dùng</text>
<rect x="340" y="60" width="200" height="86" rx="10" class="f-surface s-line"/><text x="356" y="86" class="t-b">R2</text><text x="356" y="110" class="t-small">cấp khi refresh lần 1</text><text x="356" y="130" class="t-small">đã dùng</text>
<rect x="640" y="60" width="200" height="86" rx="10" class="f-pk-soft s-pk"/><text x="656" y="86" class="t-b">R3</text><text x="656" y="110" class="t-small">cấp khi refresh lần 2</text><text x="656" y="130" class="t-small">hiện tại, app đang giữ</text>
<path d="M242 103 L338 103" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-2-a)"/><text x="256" y="94" class="t-small">refresh</text>
<path d="M542 103 L638 103" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-2-a)"/><text x="556" y="94" class="t-small">refresh</text>
<path d="M140 148 L140 214 L338 214" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl24-2-w)"/><text x="156" y="200" class="t-small t-warn">ai đó đưa lại R1</text>
<rect x="340" y="190" width="500" height="124" rx="10" class="f-warn-soft s-warn"/><text x="356" y="218" class="t-b t-warn">Server: dùng lại token đã xoay</text><text x="356" y="244" class="t-small">thu hồi cả họ, trả 401 "reuse detected"</text><text x="356" y="266" class="t-small">R3 cũng hỏng dù chưa ai dùng lại</text><text x="356" y="288" class="t-small">người dùng phải đăng nhập lại bằng OTP</text>
</svg>
</div>
<figcaption>Hình 2. Mũi tên liền là refresh bình thường, mũi tên đứt là lần đưa lại token cũ. Cái giá của rotation: app phải lưu cặp mới trước khi làm gì khác và không bao giờ đưa một refresh token đi hai lần.</figcaption>
</figure>

Phía fake chỉ có bốn nhánh:

### auth_module.dart · tools/fake_api/lib/src/auth_module.dart (trích)

```dart
final family = record.family;
if (family.revoked) {
  throw const ProblemException(401, 'Refresh token revoked');
}
if (record.used) {
  // Hai bên cùng giữ một refresh token đã xoay: thu hồi cả họ.
  family.revoked = true;
  throw const ProblemException(401, 'Refresh token reuse detected');
}
if (!_ctx.clock().isBefore(record.expiresAt)) {
  throw const ProblemException(401, 'Refresh token expired');
}
record.used = true;
return jsonResponse(200, _issue(family));
```

Hai điểm cần nhớ. Access token **không** bị thu hồi khi họ bị thu hồi hay khi đăng xuất, vì JWT hợp lệ tới `exp` (test `access token vẫn dùng được tới hết hạn`); đó là lý do hạn của nó ngắn. Cách làm này theo mục refresh token rotation của OAuth 2.0 Security Best Current Practice (RFC 9700): server không phân biệt được bên nào gửi token đã xoay nên thu hồi token đang hoạt động.

## 4. SessionManager: một lần refresh cho mọi request

Lỗi ở đầu bài sinh ra từ việc mỗi request tự quyết định refresh. Ta dồn quyết định đó vào `SessionManager`, nơi giữ cặp token trong bộ nhớ sau lần đọc kho đầu tiên và là nơi duy nhất thay đổi cặp token đó. Request bị 401 gọi vào đây và nhận một `Future` chung (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 3. Năm request cùng nhận 401. Cả năm gọi refreshAfterUnauthorized. Request đầu tiên tạo ra Future _inFlight và gửi một lời gọi POST /auth/refresh. Bốn request còn lại nhận lại đúng Future đó. Khi refresh xong, SessionManager lưu cặp mới vào kho rồi cập nhật bộ nhớ. Cuối cùng cả năm request được phát lại với access token mới.">
<defs><marker id="fl24-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="24" class="t-small">Năm request, cùng token hết hạn</text>
<rect x="20" y="36" width="190" height="40" rx="8" class="f-phys-soft s-phys"/><text x="36" y="61" class="t-small">request 1 nhận 401</text>
<rect x="20" y="90" width="190" height="40" rx="8" class="f-phys-soft s-phys"/><text x="36" y="115" class="t-small">request 2 nhận 401</text>
<rect x="20" y="144" width="190" height="40" rx="8" class="f-phys-soft s-phys"/><text x="36" y="169" class="t-small">request 3 nhận 401</text>
<rect x="20" y="198" width="190" height="40" rx="8" class="f-phys-soft s-phys"/><text x="36" y="223" class="t-small">request 4 nhận 401</text>
<rect x="20" y="252" width="190" height="40" rx="8" class="f-phys-soft s-phys"/><text x="36" y="277" class="t-small">request 5 nhận 401</text>
<path d="M212 56 L318 56" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M212 110 L318 76" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M212 164 L318 96" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M212 218 L318 114" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M212 272 L318 128" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<rect x="320" y="36" width="240" height="100" rx="10" class="f-pk-soft s-pk"/><text x="336" y="62" class="t-mono">refreshAfterUnauthorized</text><text x="336" y="86" class="t-small">có _inFlight thì dùng chung</text><text x="336" y="106" class="t-small">token đã đổi thì trả luôn</text><text x="336" y="126" class="t-small">còn lại: tạo _inFlight mới</text>
<rect x="320" y="162" width="240" height="56" rx="10" class="f-pk-soft s-pk"/><text x="336" y="188" class="t-b">_inFlight</text><text x="336" y="208" class="t-small">một Future, năm người chờ</text>
<rect x="320" y="242" width="240" height="52" rx="10" class="f-surface s-line"/><text x="336" y="266" class="t-mono">POST /auth/refresh</text><text x="336" y="284" class="t-small">đúng một lời gọi</text>
<rect x="320" y="314" width="240" height="64" rx="10" class="f-surface s-line"/><text x="336" y="340" class="t-b">Lưu cặp mới</text><text x="336" y="362" class="t-small">bộ nhớ, rồi kho</text>
<path d="M440 138 L440 160" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M440 220 L440 240" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M440 296 L440 312" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<rect x="670" y="36" width="190" height="40" rx="8" class="f-surface s-line"/><text x="686" y="61" class="t-small">phát lại request 1</text>
<rect x="670" y="90" width="190" height="40" rx="8" class="f-surface s-line"/><text x="686" y="115" class="t-small">phát lại request 2</text>
<rect x="670" y="144" width="190" height="40" rx="8" class="f-surface s-line"/><text x="686" y="169" class="t-small">phát lại request 3</text>
<rect x="670" y="198" width="190" height="40" rx="8" class="f-surface s-line"/><text x="686" y="223" class="t-small">phát lại request 4</text>
<rect x="670" y="252" width="190" height="40" rx="8" class="f-surface s-line"/><text x="686" y="277" class="t-small">phát lại request 5</text>
<path d="M562 346 L620 346 L620 56 L668 56" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M620 110 L668 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M620 164 L668 164" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M620 218 L668 218" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<path d="M620 272 L668 272" class="f-none s-pk" stroke-width="2" marker-end="url(#fl24-3-a)"/>
<text x="590" y="26" class="t-small">token mới, một lần</text>
</svg>
</div>
<figcaption>Hình 3. Mũi tên chỉ chiều của luồng điều khiển. Năm request hội tụ về một lời gọi refresh, rồi cả năm đi tiếp với token mới.</figcaption>
</figure>

### session_manager.dart · 24-auth-jwt-refresh/lib/auth/session_manager.dart (trích)

```dart
Future<AuthTokens> refreshAfterUnauthorized(
  String rejectedAccessToken,
) async {
  await _load();
  // Từ đây tới khi gán _inFlight không có await: không ai chen vào được.
  final pending = _inFlight;
  if (pending != null) return pending;
  final current = _tokens;
  if (current == null) throw const SessionExpired();
  if (current.accessToken != rejectedAccessToken) return current;
  return _inFlight = _refresh(current).whenComplete(() => _inFlight = null);
}
```

Từ `final pending` tới phép gán `_inFlight` không có `await`, nên không ai chen vào được; vì thế token được giữ trong bộ nhớ: nếu phải đọc kho (bất đồng bộ) thì hai request có thể cùng qua kiểm tra. Nhánh `current.accessToken != rejectedAccessToken` cứu trường hợp 401 về chậm, sau khi người khác đã refresh xong (test `401 về muộn khi token đã đổi`). Nhưng nhánh này không biết 401 thuộc phiên nào: request của phiên A về muộn sau khi khách đăng nhập lại thành B cũng mang token khác, và phát lại nó bằng token B là làm việc của A dưới danh nghĩa B. Vì vậy `AuthInterceptor` ghi `epoch` vào từng request (mục 5).

### session_manager.dart · 24-auth-jwt-refresh/lib/auth/session_manager.dart (trích)

```dart
try {
  next = await _api.refresh(current.refreshToken);
} on ClientFailure catch (failure) {
  if (failure.statusCode == 401 || failure.statusCode == 403) {
    if (epoch == _epoch) await _endSession();
    throw const SessionExpired();
  }
  rethrow; // 429, 400...: lỗi của lời gọi, không phải của phiên.
}
if (epoch != _epoch) {
  throw const SessionExpired();
}
_tokens = next;
try {
  await _store.save(next);
} on Object catch (error) {
  onStoreError?.call(error);
}
if (epoch != _epoch) {
  // Đăng xuất xảy ra ngay trong lúc ghi kho: lệnh ghi có thể đến sau lệnh xoá.
  await _restoreStore();
  throw const SessionExpired();
}
return next;
```

"Refresh thất bại thì đăng xuất" cần nói chính xác. 401 hoặc 403 từ `/auth/refresh` nghĩa là phiên đã chết: ta xoá kho và phát `signedOut`. Đó là hợp đồng của Tới Nơi; server theo RFC 6749 §5.2 trả 400 `invalid_grant`, và đổi server thì phải đổi nhánh này. Mất sóng, timeout hay 5xx của chính lời gọi refresh không nói gì về phiên, nên ta giữ phiên và để request nhận lỗi thật. Giới hạn: nếu server đã xoay mà response bị mất, phiên "còn" nhưng lần Thử lại đưa token đã xoay và bị đăng xuất (test `mất response của lần refresh`). Chỉ server sửa được, bằng khoảng ân hạn ngắn mà một số server có; fake không có.

Hai nơi có thể làm sống lại phiên đã đăng xuất, và `epoch` (tăng mỗi lần phiên kết thúc hoặc bắt đầu) chặn cả hai. Kết quả refresh về muộn sau khi khách bấm Đăng xuất bị bỏ qua (test `đăng xuất trong lúc refresh đang bay`). Lệnh ghi kho cũng có thể đến **sau** lệnh xoá, nên sau khi ghi ta kiểm lại `epoch` và đưa kho về đúng trạng thái hiện tại (test `đăng xuất đúng lúc đang ghi kho`). Cặp mới vào bộ nhớ trước rồi mới ghi kho, vì server đã xoay token nên ghi kho lỗi (`PlatformException`) không được làm mất nó; lỗi báo qua `onStoreError`, nhưng kho còn cặp cũ nên lần mở app sau phải đăng nhập lại.

## 5. AuthInterceptor: gắn token, bắt 401, phát lại một lần

`AuthInterceptor` làm hai việc: ở `onRequest` gắn `Authorization: Bearer` bằng token hiện tại, ở `onError` bắt 401, hỏi `SessionManager`, rồi phát lại.

### auth_interceptor.dart · 24-auth-jwt-refresh/lib/auth/auth_interceptor.dart (trích)

```dart
final AuthTokens fresh;
try {
  fresh = await session.refreshAfterUnauthorized(rejected);
} on SessionExpired {
  handler.next(err);
  return;
} on ApiFailure catch (failure) {
  handler.next(
    DioException(
      requestOptions: options,
      type: DioExceptionType.unknown,
      error: failure,
    ),
  );
  return;
}

// ... nếu người gọi đã huỷ (cancelToken) trong lúc chờ refresh thì dừng, không phát lại
try {
  final response = await dio.fetch<dynamic>(
    options.copyWith(
      headers: {
        ...options.headers,
        'authorization': 'Bearer ${fresh.accessToken}',
      },
      extra: {...options.extra, _replayedKey: true},
    ),
  );
  handler.resolve(response);
} on DioException catch (e) {
  handler.next(e);
}
```

Cờ `_replayedKey` bảo đảm "đúng một lần": phát lại mà vẫn 401 thì lỗi đi tiếp. Request chưa từng mang token hoặc tới đường dẫn công khai như `/quotes` bị bỏ qua. `onRequest` ghi `session.epoch` vào request, và `onError` bỏ qua 401 của request mang `epoch` khác phiên hiện tại: đó là request của người đã đăng xuất, không được phát lại bằng token người vừa đăng nhập (test `401 của phiên cũ về sau khi đã đăng nhập lại`). Ba chi tiết khác đều có test:

- **Interceptor thường, không phải `QueuedInterceptor`.** Tài liệu của `dio` ghi interceptor thường chạy đồng thời, còn `QueuedInterceptor` cho truy cập tuần tự. Mã nguồn `dio` 5.11.1: `QueuedInterceptor` có hàng đợi riêng cho `onError`, task kế tiếp chỉ chạy khi `handler` của task trước hoàn tất. Nếu `onError` chờ một request đi qua chính `Dio` đó và request ấy cũng lỗi, lỗi mới xếp sau task đang chờ nó (test `QueuedInterceptor refresh qua chính Dio`: không bao giờ xong). Refresh trên `Dio` riêng chưa đủ nếu lần phát lại vẫn đi qua `Dio` đó và cũng 401 (test `refresh trên Dio riêng vẫn treo`).
- **`Dio` riêng cho `/auth/*`**, không `AuthInterceptor`, không `RetryInterceptor`. Lời gọi refresh không an toàn để lặp lại: gửi lại sau khi response mất bị coi là dùng lại. Test `refresh gặp 5xx` đếm đúng một lời gọi refresh dù client có `RetryInterceptor`.
- **`onRequest` ghi đè header mỗi lần.** Lần phát lại sau refresh và lần thử lại của `RetryInterceptor` đều gọi `dio.fetch`, tức đi lại qua cả danh sách, nên luôn mang token hiện tại. Request đến giữa lúc đang refresh thì chờ (`waitForRefresh`) và đi với token mới ngay lần đầu.

### providers.dart · 24-auth-jwt-refresh/lib/providers.dart (trích)

```dart
client.dio.interceptors
  ..add(
    AuthInterceptor(
      dio: client.dio,
      session: ref.watch(sessionManagerProvider),
      publicPaths: const {'/quotes'},
    ),
  )
  ..add(
    RetryInterceptor(
      dio: client.dio,
      sleep: ref.watch(retrySleepProvider),
      random: ref.watch(retryRandomProvider),
    ),
  );
```

Lab dùng thứ tự log (đã có từ lúc tạo client), `AuthInterceptor`, rồi `RetryInterceptor`. Test `dio chạy onRequest và onError theo cùng một thứ tự` xác nhận `onError` chạy theo đúng thứ tự trong danh sách, nên log đứng đầu thấy 401 trước `AuthInterceptor`. Log thật của một lần đặt chuyến với token hết hạn là:

### Log thật (rút gọn: bỏ thời gian và header; test log thấy 401 trước khi AuthInterceptor xử lý)

```text
--> POST /trips
<-- ERROR badResponse 401 POST /trips
--> POST /auth/refresh
<-- 200 POST /auth/refresh
--> POST /trips
<-- 201 POST /trips
```

Thứ tự log, `RetryInterceptor`, `AuthInterceptor` cũng được hỗ trợ (README của `toi_noi_api`: interceptor đứng sau Retry chỉ thấy `onError` một lần, với lỗi cuối). Với cả hai thứ tự, test chạy ba test case: năm request song song cùng 401 vẫn một lần refresh và Retry không chờ lần nào; 503 rồi 401 thì refresh một lần và chuyến chỉ tạo một lần; lần phát lại sau refresh gặp 503 thì Retry vẫn thử lại. Lý do chung: 401 là 4xx nên Retry không đụng tới, còn mọi lần gửi lại đều qua `onRequest` của ta. Ta chọn log, Auth, Retry vì đặt log sau `AuthInterceptor` thì dòng lỗi 401 biến mất: lỗi đã được xử lý trước khi tới log.

## 6. Đăng xuất và nối vào giao diện

Đăng xuất làm ba việc theo thứ tự: xoá kho và bộ nhớ, phát `signedOut`, rồi mới báo server thu hồi họ token.

### session_manager.dart · 24-auth-jwt-refresh/lib/auth/session_manager.dart (trích)

```dart
await _endSession();
try {
  await _api.logout(tokens.refreshToken);
} on ApiFailure {
  // Họ token ở server sẽ tự hết hạn; thu hồi sớm chỉ là thêm một lớp an toàn.
}
```

Báo server trước thì khách không có sóng sẽ bấm Đăng xuất mà app đứng im, token vẫn nằm trong kho. Làm cục bộ trước thì dù mạng ra sao, thiết bị cũng không còn giữ gì. Test `mất mạng khi báo server` kiểm kho sạch và không ném lỗi; test chạy chung cho `InMemoryTokenStore` và `SecureTokenStore` (nền tảng giả).

`AuthController` là một `AsyncNotifier` nghe luồng `signedOut`; đăng xuất ở đâu cũng đổi state thành `null`, kể cả khi server từ chối refresh giữa chừng một request. `AuthGate` chọn màn hình theo state nên không có `Navigator.push` nào để quên; với `go_router` ở bài 23, ta đưa luồng này vào `refreshListenable`. Test `server thu hồi phiên giữa chừng` bấm đặt chuyến khi họ token đã bị thu hồi, và màn hình tự về đăng nhập.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với token và interceptor</span>

<p><strong>1. Mỗi request tự refresh.</strong> Dấu hiệu: nhiều lời gọi <code>/auth/refresh</code> cùng lúc, rồi người dùng cũng bị đăng xuất. Nguyên nhân: không dùng chung kết quả, rotation coi lần đưa lại là trộm. Cách sửa: một <code>Future</code> chung và so token bị từ chối với token hiện tại (mục 4).</p>

<p><strong>2. Treo vì <code>QueuedInterceptor</code> và <code>Dio</code> dùng chung.</strong> Dấu hiệu: request không bao giờ xong khi refresh hỏng. Nguyên nhân: lỗi của lời gọi refresh xếp hàng sau task đang chờ nó. Cách sửa: <code>Interceptor</code> thường cộng <code>Dio</code> riêng cho refresh (mục 5).</p>

<p><strong>3. Refresh lỗi mạng cũng đăng xuất.</strong> Dấu hiệu: khách ra khỏi hầm là phải nhập OTP lại. Nguyên nhân: coi mọi lỗi refresh là phiên chết. Cách sửa: chỉ 401 và 403 kết thúc phiên, lỗi khác giữ phiên (test <code>refresh gặp 5xx, mất mạng, 429</code>), trừ trường hợp mất response đã nói ở mục 4.</p>

<p><strong>4. Tin <code>role</code> trong JWT.</strong> Dấu hiệu: đổi một chữ trong payload là mở được màn hình tài xế. Nguyên nhân: client không kiểm được chữ ký. Cách sửa: client chỉ hiển thị; server kiểm từng request (mục 2).</p>

<p><strong>5. Đăng xuất chờ mạng, hoặc để kết quả muộn làm sống lại phiên.</strong> Dấu hiệu: nút Đăng xuất đứng im khi mất sóng, hoặc vừa đăng xuất lại thấy đã đăng nhập. Cách sửa: xoá cục bộ trước, và kiểm <code>epoch</code> sau mỗi <code>await</code> (mục 4 và 6).</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get && dart analyze && dart test
cd ../../24-auth-jwt-refresh
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: fake API `No issues found!` và **91 test** (28 test mới của `AuthModule`). Lab `No issues found!` và `All tests passed!` với **79 test**. Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `dio` 5.11.1, `flutter_riverpod` 3.5.0, `flutter_secure_storage` 11.2.0. Mức kiểm chứng: **Kiểm chứng tự động**.

Test dựng một "app" trên fake API chạy trong tiến trình, đồng hồ của server là biến trong test nên access token hết hạn mà không chờ thật. Ta cũng thử phá: bỏ bước dùng chung `_inFlight` làm test năm request song song không đạt, bỏ cờ phát lại làm test phát lại vẫn 401 lặp vô hạn.

**Chưa kiểm**: Keychain và Android Keystore thật, HTTP thật qua mạng di động, JWT có chữ ký, OTP qua SMS, ân hạn khi mất response refresh, hai tiến trình app cùng tranh một refresh token, refresh chủ động theo `exp`, và nối với `go_router` của bài 23.

## Bài tập

1. Server thật cho dùng lại refresh token vừa xoay trong 10 giây (khoảng ân hạn). Trường hợp nào trong bài bớt đáng sợ hơn, và client có cần đổi gì không?
2. Thêm refresh chủ động khi `exp` còn dưới 60 giây. Đặt ở đâu, và vì sao vẫn giữ nhánh bắt 401?
3. Một bạn viết `if (claims.role == 'driver')` để hiện màn hình dành cho tài xế. Sai ở đâu, và nên làm gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Trường hợp "mất response refresh" không còn kéo theo việc thu hồi cả họ: lần gửi lại trong ân hạn nhận cặp mới. Client vẫn phải lưu cặp mới trước khi dùng, và năm request vẫn nên dùng chung một lần refresh.</p>
<p><strong>2.</strong> Trong <code>AuthInterceptor.onRequest</code>: đọc <code>UnverifiedClaims.tryParse(token)?.expiresAt</code>, nếu gần hạn thì gọi cùng <code>refreshAfterUnauthorized</code> để vẫn dùng chung <code>Future</code>. Nhánh 401 vẫn cần: đồng hồ máy có thể lệch, server có thể thu hồi sớm.</p>
<p><strong>3.</strong> Payload sửa được, nên ai sửa token cũng mở được màn hình đó. Ẩn lối vào theo <code>role</code> chỉ để giao diện gọn; mọi API của tài xế phải trả 403 cho token không phải tài xế, vì server mới là nơi kiểm quyền.</p>
</details>

Bài trước: [go_router: deep link, ShellRoute, guard đăng nhập](/articles/flutter-go-router-deep-link-shellroute-guard). Bài tiếp: [Cấu hình môi trường: flavors, --dart-define-from-file và DI theo môi trường](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod).

## Tài liệu gốc

- [dio](https://pub.dev/packages/dio) và [tài liệu API](https://pub.dev/documentation/dio/latest/dio/dio-library.html) (pub.dev), gồm `Interceptor`, `QueuedInterceptor`, `RequestOptions.copyWith`.
- [flutter_secure_storage](https://pub.dev/packages/flutter_secure_storage) (pub.dev) và [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) (pub.dev).
- [RFC 9700: OAuth 2.0 Security Best Current Practice](https://www.rfc-editor.org/rfc/rfc9700.html) (mục refresh token protection) và [RFC 6749 mục 5.2](https://www.rfc-editor.org/rfc/rfc6749.html#section-5.2) (rfc-editor.org).
- [OWASP MASVS](https://mas.owasp.org/MASVS/) (mas.owasp.org), nhóm MASVS-AUTH và MASVS-STORAGE.
- [Keychain services](https://developer.apple.com/documentation/security/keychain-services) (developer.apple.com) và [Android Keystore system](https://developer.android.com/privacy-and-security/keystore) (developer.android.com).
- [Networking](https://docs.flutter.dev/cookbook/networking) (docs.flutter.dev).

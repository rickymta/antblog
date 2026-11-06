---
slug: python-fastapi-jwt-oauth2-authorization
title: "Authentication và authorization: khách, tài xế, điều phối"
summary: "Bài 34: bảo vệ API FastAPI bằng OAuth2 password và bearer: mật khẩu băm argon2, JWT HS256 ký bằng PyJWT, scope customer/driver/dispatcher, 401 khác 403. Có chín ca token xấu và lab chạy thật."
category: programming
level: intermediate
tags: [python, toi-noi, fastapi, jwt, security]
language: vi
---

Ở bài 33, API của [Tới Nơi](/articles/du-an-mau-toi-noi) biết «khách hiện tại» là ai bằng một header `X-Customer-Id` do chính client gửi: ai gõ `CU-001` cũng đọc được chuyến của khách 1. Trong khi đó API có ba loại người gọi: khách báo giá và xem chuyến của mình, tài xế xem chuyến được gán, điều phối viên xem hàng đợi và mọi chuyến. Bài này thay «tin header» bằng danh tính đã được chứng minh: đăng nhập bằng mật khẩu, nhận một token có chữ ký, mỗi route kiểm token rồi kiểm quyền.

Đây là bài 34 của lộ trình **Python trung bình qua Tới Nơi**. Bài dựng trên `Depends` và `Settings` của [bài 33](/articles/python-fastapi-dependency-middleware-settings-logging), nên bạn nên đọc nó trước. Lab tự chứa, không cần hạ tầng; nó mang theo Problem Details và hợp đồng `POST /v1/quotes` của [bài 32](/articles/python-fastapi-validation-problem-details-openapi) và `request_id` của bài 33.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Token của khách <code>CU-001</code> có payload <code>"scope": "customer"</code>. Khách này giải mã phần payload bằng base64, sửa thành <code>"scope": "dispatcher"</code>, ghép lại với phần chữ ký cũ rồi gửi. Server có chấp nhận không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Chữ ký được tính trên header và payload, sửa một ký tự là chữ ký không còn khớp, và server trả 401. Nhưng chiều ngược lại đáng chú ý: <b>đọc</b> payload thì ai cũng làm được, vì JWT mặc định chỉ ký chứ không mã hóa. Bài chạy đúng ca này.</p></details>
</div>

## Ba người gọi, ba quyền khác nhau

**Authentication** (xác thực) trả lời «bạn là ai»: server kiểm một bằng chứng (mật khẩu, token) rồi mới tin danh tính. **Authorization** (phân quyền) trả lời «bạn được làm gì». Chúng cho hai mã lỗi khác nhau: **401** khi chưa chứng minh được danh tính, **403** khi biết là ai nhưng không đủ quyền.

| Người gọi | Scope trong token | Được làm |
| --- | --- | --- |
| Khách `CU-001` | `customer` | Báo giá, xem chuyến của mình |
| Tài xế `DR-001` | `driver` | Xem khả năng nhận chuyến, xem chuyến được gán |
| Điều phối `DP-001` | `dispatcher` | Xem hàng đợi và mọi chuyến |

Một **scope** là một quyền ghi trong token. Lab đặt tên scope trùng tên vai trò cho gọn; hệ thống lớn tách vai trò (người này là ai) khỏi scope (token này được làm gì).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Bốn cổng liên tiếp: có token, token hợp lệ, đủ scope, đúng chuyến. Hai cổng đầu trả 401, cổng thứ ba trả 403, cổng thứ tư trả 404. Qua đủ bốn cổng thì handler trả 200."><defs><marker id="py34-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="56" class="t-b">1. Có token?</text><text x="36" y="80" class="t-small">header Authorization</text><text x="36" y="98" class="t-small">scheme Bearer</text><path d="M170 75 L220 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py34-0-arrow)"/><rect x="220" y="30" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="236" y="56" class="t-b">2. Hợp lệ?</text><text x="236" y="80" class="t-small">chữ ký, hạn,</text><text x="236" y="98" class="t-small">aud, iss, claim</text><path d="M370 75 L420 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py34-0-arrow)"/><rect x="420" y="30" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="436" y="56" class="t-b">3. Đủ scope?</text><text x="436" y="80" class="t-small">scope token chứa</text><text x="436" y="98" class="t-small">scope route cần</text><path d="M570 75 L620 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py34-0-arrow)"/><rect x="620" y="30" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="636" y="56" class="t-b">4. Đúng chuyến?</text><text x="636" y="80" class="t-small">chủ chuyến, tài xế</text><text x="636" y="98" class="t-small">được gán, điều phối</text><path d="M770 75 L790 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py34-0-arrow)"/><rect x="790" y="30" width="70" height="90" rx="10" class="f-phys-soft s-phys"/><text x="806" y="70" class="t-b">200</text><text x="802" y="92" class="t-small">handler</text><path d="M95 120 L95 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py34-0-arrow)"/><path d="M295 120 L295 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py34-0-arrow)"/><path d="M495 120 L495 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py34-0-arrow)"/><path d="M695 120 L695 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py34-0-arrow)"/><rect x="20" y="170" width="150" height="62" rx="10" class="f-warn-soft s-warn"/><text x="36" y="196" class="t-b">401</text><text x="36" y="216" class="t-small">thách thức Bearer</text><rect x="220" y="170" width="150" height="62" rx="10" class="f-warn-soft s-warn"/><text x="236" y="196" class="t-b">401</text><text x="236" y="216" class="t-small">invalid_token</text><rect x="420" y="170" width="150" height="62" rx="10" class="f-warn-soft s-warn"/><text x="436" y="196" class="t-b">403</text><text x="436" y="216" class="t-small">insufficient_scope</text><rect x="620" y="170" width="150" height="62" rx="10" class="f-warn-soft s-warn"/><text x="636" y="196" class="t-b">404</text><text x="636" y="216" class="t-small">như không tồn tại</text><text x="20" y="262" class="t-small">Cổng 1 và 2 trả lời «bạn là ai»; cổng 3 và 4 trả lời «bạn được làm gì». Cổng nào đóng thì dừng ở đó.</text></svg>
</div>
<figcaption>Hình 1. Bốn cổng một request phải qua, mỗi cổng có mã lỗi riêng. Cổng 4 là lớp kiểm theo từng chuyến, bên cạnh lớp kiểm theo vai trò.</figcaption>
</figure>

Cổng 4 cần giải thích. Scope `customer` chỉ nói «được xem chuyến», không nói chuyến nào; thiếu cổng này, mọi khách có token hợp lệ đọc được chuyến của khách khác (lỗi kiểm soát truy cập cấp đối tượng, rất hay gặp). Cổng 3 dựa vào token, cổng 4 dựa vào dữ liệu server tin cậy (chủ chuyến lưu ở kho, không do client khai). Phía C#: [bài authentication và authorization](/articles/dotnet10-authentication-authorization).

## Mật khẩu: băm bằng argon2, không tự viết mật mã

Server không lưu mật khẩu, chỉ lưu **hash**. Hàm băm phải *cố ý chậm* và tốn bộ nhớ, để kẻ trộm được bảng hash không thử hàng tỷ mật khẩu mỗi giây như với SHA-256. Lab dùng `pwdlib` với cấu hình khuyến nghị, tức **argon2id**, thay vì tự ghép hàm băm:

```python
hasher = PasswordHash.recommended()  # argon2id
app.state.users = build_users(hasher)
```

Hash mang theo mọi tham số, nên về sau đổi tham số vẫn kiểm được hash cũ (output của `main.py`, cắt bớt):

```text
hash lưu trữ : $argon2id$v=19$m=65536,t=3,p=4$…
verify một mật khẩu: khoảng 32 ms (cố ý chậm; máy của bạn sẽ khác)
```

`m=65536` là 64 MiB bộ nhớ mỗi lần băm, `t=3` số vòng, `p=4` mức song song; kế đó là muối ngẫu nhiên rồi tới hash, nên cùng mật khẩu băm hai lần cho hai chuỗi khác nhau (lab kiểm). Hai chi tiết trong `POST /v1/auth/token` (trích `app/routes.py`):

```python
    user = users.get(form.username)
    # Tên không tồn tại vẫn tốn một lần băm, để thời gian trả lời không tiết lộ tài khoản có thật.
    hash_to_check = user.password_hash if user else request.app.state.dummy_hash
    # Băm argon2 tốn CPU cỡ vài chục mili giây: đưa sang thread để không chặn event loop (bài 21).
    password_ok = await asyncio.to_thread(hasher.verify, form.password, hash_to_check)
    if user is None or not password_ok:
        log.warning("auth.login.failed known_user=%s request_id=%s", user is not None, request.state.request_id)
        raise OAuthError("invalid_grant", "sai tên đăng nhập hoặc mật khẩu")  # không log tên và mật khẩu đã gõ
```

Trả lời ngay khi tài khoản không tồn tại thì kẻ dò thấy «không có tài khoản» nhanh hơn «sai mật khẩu» và liệt kê được người dùng; nên băm một hash giả và dùng **cùng một thông báo** cho hai trường hợp (lab kiểm). `asyncio.to_thread` cần vì `verify` tốn CPU: gọi thẳng trong `async def` sẽ chặn event loop (bài 21). Log chỉ ghi `known_user=True/False`, vì tên gõ vào ô đăng nhập đôi khi là mật khẩu gõ nhầm chỗ.

Lỗi đăng nhập là 400, không phải 401: RFC 6749 mục 5.2 quy định sai thông tin đăng nhập của người dùng là `invalid_grant`, trả 400 kèm thành viên `error`; 401 dành cho việc xác thực *client* OAuth (ở đây không có). Lab giữ Problem Details của cả loạt và thêm đúng thành viên `error`.

## JWT: chữ ký chống sửa, không chống đọc

Sau khi mật khẩu đúng, server phát một **JWT** (JSON Web Token): chuỗi gồm ba phần cách nhau bằng dấu chấm, mỗi phần là base64url.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Một JWT gồm ba phần: header khai thuật toán HS256, payload chứa các claim sub, scope, iss, aud, iat, exp, và chữ ký tính từ header, payload và khóa. Header và payload đọc được bằng base64; chữ ký chống việc sửa nội dung."><defs><marker id="py34-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="36" y="58" class="t-b">Header</text><text x="36" y="84" class="t-mono">{"alg":"HS256",</text><text x="36" y="102" class="t-mono"> "typ":"JWT"}</text><text x="279" y="92" class="t-b">.</text><rect x="290" y="30" width="340" height="110" rx="10" class="f-pk-soft s-pk"/><text x="306" y="58" class="t-b">Payload (các claim)</text><text x="306" y="82" class="t-mono">"sub":"CU-001", "scope":"customer"</text><text x="306" y="100" class="t-mono">"iss", "aud", "iat", "exp"</text><text x="306" y="122" class="t-small">tối thiểu để biết ai, quyền gì, hết hạn khi nào</text><text x="639" y="92" class="t-b">.</text><rect x="650" y="30" width="210" height="110" rx="10" class="f-warn-soft s-warn"/><text x="666" y="58" class="t-b">Chữ ký</text><text x="666" y="84" class="t-small">HMAC-SHA256 của</text><text x="666" y="102" class="t-small">header.payload + khóa</text><path d="M20 164 L630 164" class="f-none s-pk" stroke-width="2"/><text x="20" y="188" class="t-small">Đọc được bằng base64, không cần khóa. Đừng đặt bí mật hay số giấy tờ ở đây.</text><path d="M650 164 L860 164" class="f-none s-warn" stroke-width="2"/><text x="650" y="188" class="t-small">Chỉ bên có khóa mới tạo lại được.</text><text x="20" y="226" class="t-small">Server nhận token thì tính lại chữ ký từ header.payload bằng khóa của mình rồi so; lệch một ký tự là 401.</text></svg>
</div>
<figcaption>Hình 2. Ba phần của một JWT. Chữ ký bảo vệ tính nguyên vẹn của hai phần đầu, không che giấu chúng.</figcaption>
</figure>

Payload thật của khách `CU-001` do lab sinh (cắt token ở hai mươi bốn ký tự đầu):

```text
POST /v1/auth/token -> 200 {"access_token": "eyJhbGciOiJIUzI1NiIsInR5…", "token_type": "bearer", "expires_in": 900, "scope": "customer"}
header : {'alg': 'HS256', 'typ': 'JWT'}
payload: {'iss': 'toinoi-lab34', 'aud': 'toinoi-api', 'sub': 'CU-001', 'iat': 1793969250, 'exp': 1793970150, 'scope': 'customer'}
(đọc được bằng base64: chữ ký chống SỬA, không chống ĐỌC)
```

Sáu claim, không hơn: `sub` chủ thể, `scope` quyền (chuỗi cách nhau bằng dấu cách theo RFC 6749), `iat` và `exp` lúc phát và hết hạn (cách nhau 900 giây), `iss` bên phát, `aud` dịch vụ được nhận. Không có tên, số điện thoại, giấy tờ hay hash, vì ai cũng đọc được payload. Phát và kiểm token do PyJWT làm (trích `app/security.py`):

```python
    def decode(self, token: str) -> Principal:
        try:
            # algorithms=[...] luôn truyền tường minh: token tự khai alg=none hay alg khác bị từ chối.
            claims: dict[str, Any] = jwt.decode(
                token,
                self._key,
                algorithms=[ALGORITHM],
                audience=self._audience,
                issuer=self._issuer,
                options={"require": REQUIRED_CLAIMS},
            )
        except jwt.ExpiredSignatureError as exc:
            raise TokenError("expired") from exc
        ...
        scope = claims.get("scope", "")
        if not isinstance(scope, str):
            raise TokenError("malformed")  # scope phải là chuỗi cách nhau bởi dấu cách, không phải danh sách
        return Principal(claims["sub"], frozenset(scope.split()))
```

Ba chi tiết cần nhớ. Hai chi tiết đầu là khuyến nghị của RFC 8725 (JWT Best Current Practices): **luôn khai danh sách thuật toán** khi kiểm (mục 3.1), vì header do người gửi viết; và **kiểm `iss`, `aud`** (mục 3.8, 3.9), vì token phát cho dịch vụ khác không được dùng ở đây dù chữ ký đúng. Chi tiết thứ ba là quyết định của bài, không phải của RFC: **đòi đủ claim cần thiết** (`require`), vì thiếu `exp` mà không đòi thì token sống mãi. Lab còn từ chối token có `scope` không phải chuỗi.

**Khóa ký là chỗ dễ sai nhất.** HS256 dùng một khóa chung để ký và kiểm. RFC 7518 mục 3.2 đòi khóa ít nhất 32 byte; RFC 8725 mục 3.5 đòi thêm rằng mật khẩu con người nhớ được **không** được dùng trực tiếp làm khóa HS256. Cách đúng là sinh ngẫu nhiên, ví dụ `secrets.token_urlsafe(32)` (43 ký tự). `Settings` của lab bắt cả ba lỗi và **không có giá trị mặc định** cho `environment` lẫn `jwt_secret`. Bản đầu của bài mặc định là khóa công khai của lab và chỉ báo lỗi khi đặt `environment="prod"`: quên đặt biến thì dịch vụ mở ra với khóa cả thế giới biết. Giờ thiếu biến là không khởi động (output của `uvicorn --factory` thật):

```text
pydantic_core._pydantic_core.ValidationError: 2 validation errors for Settings
environment
  Field required [type=missing]
jwt_secret
  Field required [type=missing]
```

Khóa sai cũng không được lộ. Với `TOINOI_JWT_SECRET=my-real-short-key`, `ValidationError` mặc định của pydantic in cả `input_value`, tức khóa thật nằm ngay trong log lúc khởi động. `hide_input_in_errors=True` tắt việc đó, và `--verify` kiểm chuỗi lỗi không chứa khóa (trích):

```text
thiếu cả hai biến    -> environment: Field required; jwt_secret: Field required
khóa ngắn 17 byte    -> model: Value error, TOINOI_JWT_SECRET ngắn hơn 32 byte
khóa lặp 48 ký tự    -> model: Value error, TOINOI_JWT_SECRET quá đơn điệu; sinh khóa bằng secrets.token_urlsafe(32)
prod + khóa của lab  -> model: Value error, TOINOI_JWT_SECRET là khóa của lab, không dùng được ở prod
dev + khóa của lab   -> OK
prod + khóa sinh bằng secrets.token_urlsafe(32) (43 ký tự) -> OK, repr=SecretStr('**********')
```

Kiểm «quá đơn điệu» (dưới 12 ký tự khác nhau) chỉ chặn khóa kiểu `xxxx…`, không chứng minh được entropy. Khóa `lab34-dev-secret-not-for-production` là khóa công khai: chỉ chạy được khi đặt tường minh `TOINOI_ENVIRONMENT=dev`, còn `--verify` dùng khóa sinh mới mỗi lần.

## OAuth2 password flow và Bearer trong FastAPI

FastAPI có sẵn hai mảnh ghép của OAuth2. `OAuth2PasswordRequestForm` đọc form `application/x-www-form-urlencoded` với các trường `username`, `password`, `scope` (cần gói `python-multipart` để parse form). `OAuth2PasswordBearer` đọc header `Authorization: Bearer <token>` và khai với OpenAPI rằng API dùng luồng này, nên nút **Authorize** của `/docs` hoạt động:

```python
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/v1/auth/token", scopes=SCOPES, auto_error=False)
```

`auto_error=False` để lab tự trả 401 theo hợp đồng riêng (Problem Details, tiếng Việt) thay vì chuỗi mặc định. Server thật chạy bằng `uvicorn --factory` với `TOINOI_ENVIRONMENT=dev` và một khóa sinh ngẫu nhiên; đoạn tương tác dưới đây là output của nó (bỏ các header `date`, `server`, `content-length`):

```text
$ curl -i localhost:8034/v1/me
HTTP/1.1 401 Unauthorized
www-authenticate: Bearer
content-type: application/problem+json
x-request-id: 790c1d7831d246aa9ea72ff1ef9dae3a

{"type":"about:blank","title":"Unauthorized","status":401,"detail":"cần đăng nhập","instance":"/v1/me","request_id":"790c1d7831d246aa9ea72ff1ef9dae3a"}
```

Response của endpoint cấp token mang hai header mà RFC 6749 mục 5.1 bắt buộc cho mọi response chứa token, để proxy và trình duyệt không lưu token vào cache; lab kiểm cả hai (token cắt bớt):

```text
$ curl -i -X POST localhost:8034/v1/auth/token -d 'username=CU-001&password=lab34-cu001-pass'
HTTP/1.1 200 OK
content-type: application/json
cache-control: no-store
pragma: no-cache

{"access_token":"eyJhbGciOiJIUzI1NiIsInR5…","token_type":"bearer","expires_in":900,"scope":"customer"}
```

Sai mật khẩu cho `invalid_grant` ở dạng Problem Details, vẫn kèm `no-store`:

```text
HTTP/1.1 400 Bad Request
cache-control: no-store

{"type":"https://toinoi.example/problems/oauth-invalid-grant","title":"Không cấp được token","status":400,"detail":"sai tên đăng nhập hoặc mật khẩu","instance":"/v1/auth/token","request_id":"e417f577db1a4588bbc843208a6541fe","error":"invalid_grant"}
```

Header `WWW-Authenticate` là một phần của hợp đồng HTTP, không phải trang trí: theo RFC 6750, 401 phải kèm nó để client biết cần gửi bearer token. `OAuth2PasswordBearer` còn chấp nhận đúng scheme `Bearer`; một header `Basic …` bị coi như không có token (lab kiểm điều này).

Một lưu ý về luồng: «password flow» bắt client nhận mật khẩu của người dùng rồi đổi lấy token. RFC 9700 (OAuth 2.0 Security BCP, mục 2.4) ghi luồng này **không được dùng**, vì client nhìn thấy mật khẩu và nó không đi cùng xác thực hai bước. Lab chọn nó vì cho thấy trọn vòng mật khẩu, token, kiểm token trong một service. Hệ thống thật dùng IdP với authorization code kèm PKCE; API chỉ còn việc kiểm token của IdP.

## Scope và require_scope: chọn 401 hay 403

Dependency trung tâm là `current_principal`, và `require_scope` chỉ là cách khai báo ngắn gọn dựa trên `Security` và `SecurityScopes` của FastAPI (trích):

```python
async def current_principal(request, security_scopes: SecurityScopes, token: Annotated[str | None, Depends(oauth2_scheme)]) -> Principal:
    if token is None:
        raise HTTPException(status_code=401, detail="cần đăng nhập", headers=_bearer_challenge())
    ...
    missing = [s for s in security_scopes.scopes if s not in principal.scopes]
    if missing:
        raise HTTPException(status_code=403, detail=f"thiếu scope: {' '.join(missing)}",
                            headers=_bearer_challenge(error="insufficient_scope", scope=" ".join(security_scopes.scopes)))


def require_scope(*scopes: str):
    return Security(current_principal, scopes=list(scopes))
```

Route chỉ cần `principal: Annotated[Principal, require_scope("dispatcher")]`. Vì FastAPI biết scope qua `Security`, OpenAPI tự ghi route này yêu cầu `dispatcher` (`security: [{"OAuth2PasswordBearer": ["dispatcher"]}]`, lab kiểm). Route `get_trip` khác: nó dùng `Authenticated`, chỉ đòi token hợp lệ và **không** đòi scope nào (không có cổng 3), vì quyền ở đây phụ thuộc quan hệ giữa người gọi và từng chuyến (cổng 4):

```python
    trip = request.app.state.trips.get(trip_id)
    allowed = trip is not None and (
        "dispatcher" in principal.scopes
        or ("customer" in principal.scopes and trip.customer_id == principal.user_id)
        or ("driver" in principal.scopes and trip.driver_id == principal.user_id)
    )
    if not allowed:
        if trip is not None:
            log.warning("auth.trip.denied user=%s trip=%s request_id=%s", ...)
        raise HTTPException(status_code=404, detail="không tìm thấy chuyến")
```

Chuyến của người khác trả 404 thay vì 403 là quyết định có cân nhắc. RFC 9110 mục 15.5.4 cho phép máy chủ muốn «giấu» tài nguyên bị cấm trả 404: với 403, kẻ dò `TN-0001`, `TN-0002`… biết chuyến nào có thật; với 404, chuyến của người khác và chuyến không có là một. Đổi lại, gỡ lỗi khó hơn; nơi mã chuyến không bí mật và cần thông báo rõ, 403 như bài C# lại hợp hơn. Vì 404 xóa dấu vết với client, lab **ghi log** `auth.trip.denied` cho chuyến có thật bị từ chối. `--verify` kiểm body 404 của `TN-0002` (khách khác) và `TN-9999` (không có) trùng nhau, và chỉ chuyến có thật có dòng log từ chối. Bảng quyền (output của `main.py`; hàng là người gọi, cột là route; `TN-0001` của `CU-001` do `DR-001` chạy, `TN-0002` của `CU-002`):

```text
               me   quotes  TN-0001  TN-0002   driver    queue
ẩn danh       401      401      401      401      401      401
CU-001        200      201      200      404      403      403
DR-001        200      403      200      404      200      403
DP-001        200      403      200      200      403      200
WWW-Authenticate của 401: Bearer
WWW-Authenticate của 403: Bearer error="insufficient_scope", scope="dispatcher"
```

Đọc bảng theo cột: `quotes` chỉ khách báo giá được (201); `queue` chỉ điều phối; `TN-0002` khách và tài xế không liên quan nhận 404, điều phối nhận 200. Phần lớn giá trị của bảng nằm ở các ô bị từ chối, nên `--verify` kiểm đủ cả 24 ô.

## Token xấu: mười ca đều ra 401

Phần quan trọng nhất của kiểm token là những thứ nó phải từ chối. Lab dựng mười token xấu từ một token hợp lệ, gọi `/v1/me` với từng cái, và đọc lý do thật trong log của server:

```text
hết hạn 1 giây             -> 401 | log: auth.token.rejected reason=expired
sửa payload, giữ chữ ký    -> 401 | log: auth.token.rejected reason=bad_signature
ký bằng khóa khác          -> 401 | log: auth.token.rejected reason=bad_signature
alg=none, không chữ ký     -> 401 | log: auth.token.rejected reason=bad_algorithm
sai audience               -> 401 | log: auth.token.rejected reason=bad_audience
sai issuer                 -> 401 | log: auth.token.rejected reason=bad_issuer
thiếu exp                  -> 401 | log: auth.token.rejected reason=missing_claim:exp
thiếu sub                  -> 401 | log: auth.token.rejected reason=missing_claim:sub
scope là danh sách         -> 401 | log: auth.token.rejected reason=malformed
không phải JWT             -> 401 | log: auth.token.rejected reason=malformed
còn 30 giây hạn            -> 200
```

Dòng thứ hai trả lời câu hỏi đầu bài. `alg=none` là tấn công kinh điển (token tự khai «không cần chữ ký»); nhờ `algorithms=[ALGORITHM]`, PyJWT từ chối ngay. Cả mười ca nhận **cùng một** response (401, `invalid_token`, cùng `detail`): lý do chi tiết chỉ nằm trong log, vì nói «chữ ký sai» hay «sai audience» chỉ giúp kẻ tấn công chỉnh token. Dòng cuối là phép đối chứng: token còn 30 giây hạn vẫn dùng được, nên ca «hết hạn» đỏ đúng vì hết hạn.

Lab còn kiểm: log không chứa token hay mật khẩu nào, và xin scope ngoài quyền (khách xin `dispatcher`) trả 400 với `error=invalid_scope`. TTL 900 giây là cân bằng: ngắn thì token lộ ít hại, nhưng phải lấy lại thường hơn. Giữa các máy lệch đồng hồ vài giây, PyJWT có tham số `leeway`; lab để 0.

## Chạy và kiểm chứng

Không cần hạ tầng. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/34-fastapi-jwt-authorization examples/toi-noi-python/34-fastapi-jwt-authorization/main.py
uv run --project examples/toi-noi-python/34-fastapi-jwt-authorization examples/toi-noi-python/34-fastapi-jwt-authorization/main.py --verify
```

Lệnh đầu in các khối ở trên (trích từng phần). `--verify` chạy cùng kịch bản và so 73 kết quả kỳ vọng: argon2 và token, `Cache-Control`, `invalid_grant`, bảng quyền 4 hàng 6 cột, hợp đồng `quotes` và Problem Details, mười token xấu (mỗi ca kiểm mã, lý do log, nội dung response và việc token không lọt vào log), cấu hình khóa không mặc định và không lộ khóa:

```text
OK 73/73
```

Để biết bộ kiểm có thể đỏ, ta phá có chủ ý trên một bản sao: thêm `"verify_signature": False` vào `options` của `jwt.decode`. `--verify` báo (trích dòng đầu và dòng cuối):

```text
SAI: token xấu 'hết hạn 1 giây': 401 và lý do log: nhận (200, ''), kỳ vọng (401, 'auth.token.rejected reason=expired')
FAIL 61/73
```

Phá kiểm scope (`if False`) cho `SAI: ma trận quyền: CU-001: …` rồi `FAIL 68/73`; bỏ `hide_input_in_errors` thì ca «không lộ khóa» đỏ; cho `jwt_secret` một mặc định thì ca «không có giá trị mặc định» đỏ. Môi trường: Python 3.14.7, FastAPI 0.162.0, PyJWT 2.15.1, pwdlib 0.3.1, python-multipart 0.0.33, pydantic-settings 2.16.2 (`uv.lock`). Lần chạy cuối in `load average 32.64 / 47.56` (máy Apple M1 Max đang bận), nên 32 ms của argon2 chỉ để tham khảo; mã trạng thái và lý do từ chối lặp lại được.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Service không lên, báo `Field required` | Đã đặt `TOINOI_ENVIRONMENT` và `TOINOI_JWT_SECRET` chưa; không có mặc định là chủ ý |
| Mọi token đều bị 401 sau khi đổi khóa | Mọi replica có cùng khóa không; token ký bằng khóa cũ hỏng khi xoay khóa |
| 401 dù token vừa phát | `aud`, `iss` trong token và trong `Settings` có khớp không; `exp` có tính theo UTC không |
| Token hợp lệ nhưng 403 | Claim `scope` là chuỗi cách nhau bằng dấu cách; route cần đủ **mọi** scope được liệt kê |
| Đăng nhập chậm, request khác bị treo theo | `verify` argon2 đang chạy thẳng trong `async def`; dùng `asyncio.to_thread` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>HS256 dùng chung một khóa: mọi service kiểm token đều ký được token. Production dùng IdP và khóa bất đối xứng (RS256 hoặc ES256) công bố qua JWKS, để service chỉ giữ khóa công khai. Bản lab không có refresh token, thu hồi token (token đã phát sống tới <code>exp</code> kể cả khi tài khoản bị khóa), giới hạn đăng nhập sai (bài 27 có token bucket), xác thực hai bước, TLS và CORS. Kho người dùng là bốn tài khoản giả trong bộ nhớ, mật khẩu công khai trong mã lab; kho báo giá và bộ đếm <code>quote_id</code> cũng chỉ nằm trong một tiến trình.</p>
</div>

**Bài tập:** (1) Thêm route `GET /v1/dispatch/trips/{trip_id}/driver` chỉ dành cho `dispatcher`. Cần sửa những chỗ nào, và test nào trong bảng quyền phải thêm? (2) Một khách xin token bằng `scope=customer dispatcher`. Lab trả gì, và vì sao kiểm điều này ở **lúc phát token** chưa đủ nếu token do hệ thống khác ký?

<details><summary>Xem lời giải</summary><p>(1) Một route mới với <code>principal: Annotated[Principal, require_scope("dispatcher")]</code>; không sửa <code>current_principal</code>, OpenAPI tự ghi scope mới. Bảng quyền thêm một cột: ẩn danh 401, khách 403, tài xế 403, điều phối 200, trong đó ba ô là ô từ chối. (2) Lab trả 400 với <code>invalid_scope</code>. Nhưng kiểm lúc phát chỉ bảo vệ khi chính ta phát token: với token do IdP khác ký, API không thấy bước xin scope, nên phải tin claim <code>scope</code> sau khi kiểm chữ ký, <code>iss</code>, <code>aud</code> như <code>decode</code> làm.</p></details>

Bài trước: [bài 33](/articles/python-fastapi-dependency-middleware-settings-logging). Bài tiếp: [bài 35](/articles/python-fastapi-upload-background-task-streaming).

## Tài liệu gốc

- [FastAPI: OAuth2 scopes](https://fastapi.tiangolo.com/advanced/security/oauth2-scopes/).
- [FastAPI: Simple OAuth2 with Password and Bearer](https://fastapi.tiangolo.com/tutorial/security/simple-oauth2/).
- [PyJWT: Usage Examples](https://pyjwt.readthedocs.io/en/stable/usage.html).
- [pwdlib: tài liệu](https://frankie567.github.io/pwdlib/).
- [RFC 8725: JSON Web Token Best Current Practices](https://datatracker.ietf.org/doc/html/rfc8725).
- [RFC 6750: Bearer Token Usage](https://datatracker.ietf.org/doc/html/rfc6750).

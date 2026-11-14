---
slug: azure-entra-external-id-xac-thuc-khach-tai-xe
title: "Entra External ID: xác thực khách và tài xế, JWT cho API"
summary: "Bài 19: dựng external tenant Microsoft Entra External ID cho khách và tài xế của Tới Nơi, đi qua authorization code với PKCE, cấu hình AddJwtBearer cho API trips và phân biệt bốn loại danh tính trong hệ thống."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, identity, oauth2, jwt, external-id]
language: vi
---

Đây là **bài 19** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 9/9 của phần).

Bài này dựng đăng nhập cho khách và tài xế bằng Microsoft Entra External ID, rồi nối access token vào `trip-service` của loạt C#. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tài xế đăng nhập app Tới Nơi bằng Microsoft Entra ID giống nhân viên, chỉ khác là thêm họ vào một group riêng. Cách này có ổn không?</p>
<details><summary>Xem đáp án</summary><p>Không. Khách và tài xế là danh tính bên ngoài, số lượng lớn và tự đăng ký. Họ thuộc một external tenant riêng, tách khỏi thư mục nhân viên và quyền quản trị.</p></details>
</div>

## Khách và tài xế cần một tenant riêng, không phải thư mục nhân viên

Nhân viên Tới Nơi đăng nhập portal bằng tài khoản công ty trong workforce tenant (bài 11). Khách và tài xế tự đăng ký, số lượng lên hàng trăm nghìn, và không nên nằm chung thư mục với nhân viên. Vì vậy Tới Nơi dùng một **external tenant** của Entra External ID, chỉ chứa tài khoản khách hàng và app đối diện họ.

Tên gọi dễ gây rối. Azure AD B2C là sản phẩm đời trước cùng mục đích, với endpoint `b2clogin.com` và custom policy XML; theo thông báo trên Microsoft Learn, B2C đã ngừng bán cho khách hàng mới từ 05/2025. External tenant dùng endpoint `ciamlogin.com` và user flow trong Entra admin center. Authority chép từ ví dụ cũ phải kiểm lại loại nào.

Trải nghiệm đăng ký và đăng nhập được mô tả bằng **user flow** sign-up/sign-in. User flow quyết định ba thứ:

- Cách xác thực: email kèm mã một lần hoặc email kèm mật khẩu, cộng tài khoản mạng xã hội nếu bạn cấu hình làm identity provider. Đăng nhập bằng số điện thoại phụ thuộc tính năng hiện hành, kiểm tra tài liệu.
- MFA: mã qua email hoặc SMS làm yếu tố thứ hai. SMS tính phí riêng.
- Thuộc tính thu thập lúc đăng ký: tên hiển thị, và custom attribute như `driver_id`. Custom attribute xuất hiện trong token với tên dạng `extension_<app-id>_driver_id` (app-id là của app `b2c-extensions-app` trong tenant, bỏ dấu gạch) và chỉ có trong token khi bạn thêm nó vào cấu hình claim của app.

Mỗi app chỉ gắn với một user flow, còn một user flow dùng được cho nhiều app. Tới Nơi tách một user flow cho app khách và một cho app tài xế, để tài xế qua bước thu thập thêm thuộc tính và MFA chặt hơn, đổi lại là thêm cấu hình phải bảo trì.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 1. App di động đăng nhập với external tenant bằng authorization code và PKCE, nhận access token có audience là API trips, rồi gửi token cho trip-service."><defs><marker id="az19-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="185" height="110" rx="10" class="f-pk-soft s-pk"/><text x="34" y="72" class="t-b">App di động</text><text x="34" y="100" class="t-small">Authorization code</text><text x="34" y="124" class="t-small">+ PKCE, không secret</text><path d="M205 95 L240 95" class="f-none s-phys" stroke-width="2" marker-end="url(#az19-0-a)"/><rect x="240" y="40" width="185" height="110" rx="10" class="f-phys-soft s-phys"/><text x="254" y="72" class="t-b">External tenant</text><text x="254" y="100" class="t-small">Đăng ký, đăng nhập, MFA</text><text x="254" y="124" class="t-small">Cấp code rồi token</text><path d="M425 95 L460 95" class="f-none s-phys" stroke-width="2" marker-end="url(#az19-0-a)"/><rect x="460" y="40" width="185" height="110" rx="10" class="f-warn-soft s-warn"/><text x="474" y="72" class="t-b">Access token</text><text x="474" y="100" class="t-small">aud = API trips</text><text x="474" y="124" class="t-small">Sống ngắn, không log</text><path d="M645 95 L680 95" class="f-none s-phys" stroke-width="2" marker-end="url(#az19-0-a)"/><rect x="680" y="40" width="185" height="110" rx="10" class="f-pk-soft s-pk"/><text x="694" y="72" class="t-b">trip-service</text><text x="694" y="100" class="t-small">AddJwtBearer</text><text x="694" y="124" class="t-small">Kiểm iss, aud, exp, ký</text></svg>
</div>
<figcaption>Hình 1. App di động đăng nhập với external tenant bằng authorization code và PKCE, nhận access token có audience là API trips, rồi gửi token cho trip-service.</figcaption>
</figure>

## App di động không giữ được secret, nên dùng authorization code kèm PKCE

App Tới Nơi trên điện thoại là **public client**: ai cũng giải nén được bản cài và đọc mọi chuỗi bên trong, nên client secret nhúng trong app không còn là bí mật. App không có secret, và luồng phù hợp là **authorization code với PKCE**:

1. App sinh một chuỗi ngẫu nhiên `code_verifier`, băm SHA-256 thành `code_challenge`, rồi mở trình duyệt hệ thống tới endpoint authorize kèm challenge.
2. Người dùng đăng ký hoặc đăng nhập ngay trên trang của external tenant. App không bao giờ thấy mật khẩu.
3. Tenant chuyển hướng về redirect URI của app kèm một `code` dùng một lần.
4. App gọi endpoint token, gửi `code` cùng `code_verifier` gốc. Tenant băm lại và so với challenge đã nhận.

Bước 4 là chỗ PKCE có tác dụng: app độc hại chặn được `code` ở bước 3 vẫn không đổi được token vì thiếu `code_verifier`. Thực tế MSAL lo sinh verifier, mở trình duyệt, lưu token. Đừng tự dựng form đăng nhập trong WebView.

Scope `offline_access` cho app một **refresh token** để lấy access token mới mà không bắt tài xế đăng nhập lại giữa ca. Mặt trái: access token là JWT, API kiểm bằng chữ ký chứ không hỏi lại tenant, nên khoá tài khoản hay thu hồi refresh token không làm access token đang lưu hành chết ngay.

## Access token phải nhắm đúng API trips

Tới Nơi cần ba app registration trong external tenant (khái niệm xem bài 11):

| App registration | Loại | Vai trò |
| --- | --- | --- |
| `app-toinoi-trips-api` | Web API | Khai báo scope `trips.read`, `trips.write` và app role `customer`, `driver` |
| `app-toinoi-customer-mobile` | Public client | App khách, xin scope của API trips |
| `app-toinoi-driver-mobile` | Public client | App tài xế, cùng API nhưng redirect URI và branding riêng |

App di động xin scope dạng `api://<api-client-id>/trips.read`. Token trả về có `aud` trỏ về API trips, nên API phải kiểm `aud`: token hợp lệ của app khác trong cùng tenant không được đi qua cổng của bạn. Với token v2 (thường là mặc định trong external tenant), `aud` là client ID của API; với token v1 là Application ID URI. Hãy decode token lab một lần rồi cấu hình Audience đúng giá trị đó.

Các claim bạn sẽ gặp trong token:

| Claim | Ý nghĩa | Dùng làm gì ở `trip-service` |
| --- | --- | --- |
| `iss` | Tenant phát hành, dạng `https://<tenant-id>.ciamlogin.com/<tenant-id>/v2.0` | Handler so với issuer trong metadata |
| `aud` | API đích | Handler kiểm với `Auth:Audience` |
| `scp` | Scope app đã được cấp | Policy kiểm quyền gọi theo ứng dụng |
| `roles` | App role gán cho người dùng | Policy `Customer`, `Driver` |
| `oid` | Id người dùng trong tenant | Khoá chủ sở hữu chuyến, ổn định giữa các app |
| `sub` | Id người dùng theo cặp (user, app) | Không dùng làm khoá dữ liệu chung nhiều app |

Bẫy ở hai dòng cuối: `sub` khác nhau giữa các app cho cùng một người và không phải GUID, nên bảng `trips` dùng chung phải khoá theo `oid`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Custom attribute thu lúc đăng ký là dữ liệu người dùng tự gõ. Tin claim <code>driver_id</code> để mở quyền nhận chuyến thì ai cũng tự nhận mình là tài xế. Quyền tài xế phải do Tới Nơi cấp sau khi xác minh hồ sơ (ví dụ gán app role từ quy trình onboarding); thuộc tính tự khai chỉ để đối chiếu.</p>
</div>

## ASP.NET Core chỉ cần kiểm token, không cần biết đăng nhập diễn ra thế nào

Bài 19 loạt C# ([Authentication và authorization](/articles/dotnet10-authentication-authorization)) dựng `AddJwtBearer`, policy theo role và ownership, nhưng chứng minh bằng token tự ký trong test. Bài này thay nguồn token bằng một issuer thật. Phía API đổi authority, audience và hai claim type:

```csharp
builder.Services
    .AddAuthentication(JwtBearerDefaults.AuthenticationScheme)
    .AddJwtBearer(o =>
    {
        // https://<tenant-subdomain>.ciamlogin.com/<tenant-id>/v2.0
        o.Authority = builder.Configuration["Auth:Authority"];
        // Giá trị aud quan sát được trong token của API trips
        o.Audience = builder.Configuration["Auth:Audience"];
        o.MapInboundClaims = false;
        o.TokenValidationParameters.NameClaimType = "oid";
        o.TokenValidationParameters.RoleClaimType = "roles";
    });

builder.Services.AddAuthorizationBuilder()
    .AddPolicy("Customer", p => p.RequireAuthenticatedUser().RequireRole("customer"))
    .AddPolicy("Driver", p => p.RequireAuthenticatedUser().RequireRole("driver"));
```

Handler lấy metadata và khoá công khai từ authority, kiểm chữ ký, issuer, audience và hạn. Nó không chứng minh ownership: khách A có xem được chuyến của khách B hay không vẫn kiểm trong handler nghiệp vụ như bài 19 loạt C#. `Auth__Authority` và `Auth__Audience` chính là hai biến môi trường mà Bicep của Container Apps ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment) truyền vào.

Ba điểm cần để ý. Khoá ký lấy từ metadata nên tenant xoay khoá thì API không cần deploy lại. Đừng gõ cứng `ValidIssuer`: authority dùng subdomain, còn `iss` trong token và metadata (`/.well-known/openid-configuration` dưới authority) dùng tenant ID làm host; nếu nhận 401, so hai giá trị đó trước. Cuối cùng, mapping claim phải có chủ đích: code `trip-service` hiện đọc `sub` và parse thành GUID làm chủ sở hữu chuyến, cùng `RoleClaimType = "role"` cho token test. Với Entra phải chuyển sang `oid` và `roles`, nếu không mọi request sẽ bị từ chối dù token hợp lệ.

## Bốn danh tính trong Tới Nơi, bốn cổng kiểm tra khác nhau

Sau bài 11 tới bài 19, Tới Nơi có đủ bốn loại danh tính, và nhầm chúng là nguồn lỗi phổ biến:

| Danh tính | Dịch vụ cấp | Ví dụ ở Tới Nơi | Cổng kiểm tra |
| --- | --- | --- | --- |
| Nhân viên | Entra ID (workforce tenant) | Kỹ sư vào portal, chạy `az` | Azure RBAC trên scope (bài 12) |
| Khách và tài xế | Entra External ID (external tenant) | Khách gọi `GET /trips` | JWT bearer và policy trong API |
| Workload | Managed identity | `ca-toinoi-trips-dev` đọc Key Vault | RBAC của từng resource (bài 13, 14) |
| Pipeline | Federated credential (OIDC) | GitHub Actions deploy lên dev | RBAC của identity được liên kết |

Token của khách không có quyền gì trên Azure Resource Manager, và managed identity của container không thành tài khoản khách trong JWT. Hai thế giới gặp nhau ở `trip-service`: nó là workload (dùng managed identity đọc `kv-toinoi-dev-sea`) đồng thời là API nhận token của người dùng ngoài.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Bốn danh tính của Tới Nơi đi qua bốn cổng khác nhau: RBAC cho tài nguyên Azure và policy trong code cho API."><defs><marker id="az19-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="12" width="400" height="64" rx="10" class="f-phys-soft s-phys"/><text x="34" y="38" class="t-b">Nhân viên</text><text x="34" y="64" class="t-small">Entra ID, vào portal và ARM</text><rect x="20" y="88" width="400" height="64" rx="10" class="f-pk-soft s-pk"/><text x="34" y="114" class="t-b">Khách và tài xế</text><text x="34" y="140" class="t-small">External ID, vào API bằng JWT</text><rect x="20" y="164" width="400" height="64" rx="10" class="f-phys-soft s-phys"/><text x="34" y="190" class="t-b">Workload</text><text x="34" y="216" class="t-small">Managed identity, Key Vault</text><rect x="20" y="240" width="400" height="64" rx="10" class="f-phys-soft s-phys"/><text x="34" y="266" class="t-b">Pipeline</text><text x="34" y="292" class="t-small">Federated credential, deploy</text><rect x="560" y="95" width="300" height="130" rx="10" class="f-surface s-line"/><text x="576" y="130" class="t-b">Mỗi danh tính một cổng</text><text x="576" y="160" class="t-small">RBAC cho tài nguyên Azure</text><text x="576" y="188" class="t-small">Policy trong code cho API</text><path d="M420 44 L560 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az19-1-a)"/><path d="M420 120 L560 150" class="f-none s-pk" stroke-width="2" marker-end="url(#az19-1-a)"/><path d="M420 196 L560 172" class="f-none s-phys" stroke-width="2" marker-end="url(#az19-1-a)"/><path d="M420 272 L560 190" class="f-none s-phys" stroke-width="2" marker-end="url(#az19-1-a)"/></svg>
</div>
<figcaption>Hình 2. Bốn danh tính của Tới Nơi đi qua bốn cổng khác nhau: RBAC cho tài nguyên Azure và policy trong code cho API.</figcaption>
</figure>

## Chi phí theo MAU, di chuyển người dùng và chuyện không tự lưu mật khẩu

External ID tính tiền chủ yếu theo **MAU** (monthly active users), có một hạn mức miễn phí rồi tính theo bậc; SMS cho MFA và add-on có giá riêng. Tra Pricing Calculator. Hai hệ quả thiết kế:

- MAU đếm người dùng hoạt động, không đếm lượt gọi API: tài xế mở app 300 lần trong tháng vẫn là một MAU. Nhưng bot tự đăng ký hàng loạt làm MAU phình ra, nên cần chặn ở biên (bài 18) và theo dõi số tài khoản mới theo ngày.
- Tenant gắn với một subscription để thanh toán, nên đưa nó vào budget và cảnh báo chi phí ([bài về Cost Management và budget](/articles/azure-chi-phi-pricing-cost-management-budget)).

Người dùng cũ từ hệ thống trước là bài toán migration. Hash mật khẩu cũ thường không nhập thẳng được vì thuật toán băm khác nhau. Hai cách hay dùng: tạo tài khoản hàng loạt qua Microsoft Graph rồi buộc đặt lại mật khẩu hoặc đăng nhập bằng mã email ở lần đầu; hoặc migration dần khi người dùng đăng nhập lần đầu bằng custom authentication extension. Kiểm tra tài liệu hiện hành về hạn chế của từng cách.

Nguyên tắc cuối: Tới Nơi không viết form đăng nhập, không lưu hash mật khẩu. Băm, chống brute force, đặt lại mật khẩu là việc của dịch vụ danh tính; tự viết lại chỉ thêm bề mặt tấn công.

## Lab: lấy token bằng trình duyệt và gọi GET /trips

Lab mô tả cách làm trên một external tenant dùng thử; ghi lại `<tenant-id>` và `<tenant-subdomain>`. Tên menu portal có thể đổi.

1. Trong Entra admin center, tạo tenant loại External, liên kết subscription và resource group `rg-toinoi-identity-dev-sea` để thanh toán.
2. Tạo user flow sign-up/sign-in, chọn email kèm mã một lần, thu thập tên hiển thị.
3. Đăng ký `app-toinoi-trips-api`: đặt Application ID URI, thêm scope `trips.read`, thêm app role `customer`.
4. Đăng ký `app-toinoi-customer-mobile` với platform «Mobile and desktop applications», redirect URI `http://localhost`, xin quyền `trips.read` rồi cấp admin consent.
5. Gắn app khách vào user flow, tự đăng ký một người dùng thử, gán app role `customer` cho người đó qua enterprise application của API.

Khối dưới sinh cặp PKCE và in URL authorize; nó chưa lấy token, và trang localhost báo lỗi kết nối sau khi đăng nhập là bình thường:

```bash
TENANT=<tenant-id>
SUB=<tenant-subdomain>
CLIENT_ID=<customer-mobile-client-id>
API_ID=<api-client-id>

VERIFIER=$(openssl rand -base64 48 | tr '+/' '-_' | tr -d '=\n' | cut -c1-64)
CHALLENGE=$(printf '%s' "$VERIFIER" | openssl dgst -sha256 -binary \
  | openssl base64 -A | tr '+/' '-_' | tr -d '=')

echo "https://$SUB.ciamlogin.com/$TENANT/oauth2/v2.0/authorize?client_id=$CLIENT_ID\
&response_type=code&redirect_uri=http%3A%2F%2Flocalhost&response_mode=query\
&scope=api%3A%2F%2F$API_ID%2Ftrips.read%20openid%20offline_access\
&code_challenge=$CHALLENGE&code_challenge_method=S256"
```

Mở URL, đăng nhập, copy giá trị `code=` trên thanh địa chỉ (code sống vài phút). Khối sau đổi code lấy token và gọi API dev; `200` còn phụ thuộc token có đúng `aud` và `roles` hay không:

```bash
CODE=<code-tu-thanh-dia-chi>

TOKEN=$(curl -s "https://$SUB.ciamlogin.com/$TENANT/oauth2/v2.0/token" \
  -d "client_id=$CLIENT_ID" -d grant_type=authorization_code \
  --data-urlencode code="$CODE" \
  --data-urlencode redirect_uri="http://localhost" \
  --data-urlencode code_verifier="$VERIFIER" \
  --data-urlencode scope="api://$API_ID/trips.read openid offline_access" \
  | jq -r .access_token)

# Không gửi header: kỳ vọng 401
curl -i https://<fqdn-ca-toinoi-trips-dev>/trips
# Có Bearer: kỳ vọng 200 nếu aud, scp, roles đều đúng
curl -i -H "Authorization: Bearer $TOKEN" https://<fqdn-ca-toinoi-trips-dev>/trips
```

`403` nghĩa là token hợp lệ nhưng thiếu role: kiểm claim `roles`. `401` thì nghi `aud`, `iss` hoặc authority. Đây chính là ranh giới authentication và authorization ở bài 19 loạt C#. Token là credential: đừng dán token thật vào công cụ decode trực tuyến, đừng để nó trong lịch sử lệnh máy dùng chung, và xoá tenant lab khi xong.

**Bài tập:** tài xế bị khoá lúc 10:00 trong khi access token của họ còn hạn tới 10:40. Lúc 10:20 `trip-service` có từ chối request của họ không? Nếu muốn từ chối ngay, bạn làm gì?

<details><summary>Xem lời giải</summary><p>Không tự động. Handler JWT chỉ kiểm chữ ký, issuer, audience và hạn, nên token vẫn hợp lệ tới 10:40; khoá tài khoản và thu hồi refresh token chỉ ngăn việc lấy token mới. Muốn chặn ngay, `trip-service` kiểm thêm trạng thái tài xế do server giữ (cache ngắn theo `oid`) và từ chối ở policy; giữ access token sống ngắn để thu hẹp cửa sổ rủi ro.</p></details>

## Học tiếp từ đây

Bạn vừa nối ba mảnh: external tenant cho khách và tài xế, authorization code với PKCE cho app di động, và `AddJwtBearer` trong `trip-service` kiểm token do tenant đó phát, cùng bảng bốn danh tính. Đây là bài cuối của Phần 2.

Phần 3 (Compute và ứng dụng) bắt đầu bằng [bài 20](/articles/azure-app-service-deploy-slot-scale): đưa API .NET lên App Service với deployment slot và scale; authority và audience ở đây sẽ thành app setting.

## Tài liệu gốc

- [Microsoft Entra External ID](https://learn.microsoft.com/en-us/entra/external-id/)
- [External ID cho khách hàng (CIAM) overview](https://learn.microsoft.com/en-us/entra/external-id/customers/overview-customers-ciam)
- [OAuth 2.0 authorization code flow và PKCE](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow)
- [Access token của Microsoft identity platform](https://learn.microsoft.com/en-us/entra/identity-platform/access-tokens)
- [Policy-based authorization trong ASP.NET Core](https://learn.microsoft.com/en-us/aspnet/core/security/authorization/policies)
- [Azure Active Directory B2C (tên cũ)](https://learn.microsoft.com/en-us/azure/active-directory-b2c/)

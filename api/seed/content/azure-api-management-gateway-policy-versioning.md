---
slug: azure-api-management-gateway-policy-versioning
title: "API Management: gateway, policy, versioning và product"
summary: "Bài 39: đặt Azure API Management làm cổng cho đối tác của Tới Nơi, import OpenAPI của trip-service, tạo product Partner với quota, viết policy validate-jwt và rate-limit rồi quản lý version, revision và log."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, api-management, gateway, api, security]
language: vi
---

Đây là **bài 39** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 10/10 của phần).

Bài này dựng cổng API cho đối tác của Tới Nơi (quán ăn, doanh nghiệp đặt xe cho nhân viên) phía trước `trip-service`, đặt khoá truy cập, quota và kiểm JWT ở cổng thay vì trong từng service. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã có API Management kiểm JWT ở cổng, vậy trip-service bên trong có thể bỏ AddJwtBearer cho đỡ kiểm hai lần?</p>
<details><summary>Xem đáp án</summary><p>Không nên. Gateway chỉ bảo vệ đường đi qua nó; nếu ai đó gọi thẳng được vào ingress của service (cấu hình sai, mạng nội bộ bị xâm nhập), service không có lớp phòng thủ nào. Kiểm hai lần rẻ hơn nhiều so với một lần lọt.</p></details>
</div>

## Ba phần của API Management và chỗ nó đứng trong Tới Nơi

API Management (APIM) gồm ba phần. **Gateway** nhận mọi request, chạy policy, chuyển tới backend rồi trả về. **Management plane** là nơi khai báo API, product, policy qua portal, CLI hoặc ARM. **Developer portal** là trang web cho bên gọi API đọc tài liệu, đăng ký và lấy key. Tách ba phần giúp khoanh lỗi: gateway chậm là chuyện runtime và scale, portal không hiện API là chuyện cấu hình và publish.

Tới Nơi không cần APIM cho mọi thứ. App khách và app tài xế đã có JWT từ Entra External ID (bài 19) và đi thẳng vào Container Apps qua Front Door. APIM đáng tiền ở **API cho đối tác**: quán ăn tạo đơn từ hệ thống riêng, doanh nghiệp đặt xe hàng loạt cho nhân viên. Đó là những bên bạn không kiểm soát code, cần cấp và thu hồi quyền theo từng đối tác, giới hạn lượt gọi và có tài liệu tự phục vụ. Làm hết trong `trip-service` là trộn luật thương mại với luật nghiệp vụ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Request của đối tác đi qua Front Door và WAF, tới API Management rồi tới trip-service; APIM không thay WAF."><defs><marker id="az39-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Hệ thống đối tác</text><text x="34" y="123" class="t-small">Gọi bằng key và token</text><path d="M210 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az39-0-a)"/><rect x="240" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="254" y="95" class="t-b">Front Door + WAF</text><text x="254" y="123" class="t-small">Chặn bot, TLS, miền</text><path d="M430 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az39-0-a)"/><rect x="460" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="474" y="95" class="t-b">API Management</text><text x="474" y="123" class="t-small">Key, quota, JWT, policy</text><path d="M650 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az39-0-a)"/><rect x="680" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="694" y="95" class="t-b">trip-service</text><text x="694" y="123" class="t-small">Vẫn kiểm JWT lần nữa</text></svg>
</div>
<figcaption>Hình 1. Request của đối tác đi qua Front Door và WAF, tới API Management rồi tới trip-service; APIM không thay WAF.</figcaption>
</figure>

APIM bán theo tier, khác nhau ở mô hình tính tiền, tích hợp mạng và tính năng:

| Tier | Mô hình | Hợp khi |
| --- | --- | --- |
| Consumption | Serverless, tính theo lượt gọi, tạo nhanh | Lab, API ít gọi; không có developer portal, cache dựng sẵn và các policy `*-by-key` |
| Basic v2, Standard v2 | Instance dành riêng, thế hệ v2 | Production cỡ vừa, cần capacity ổn định |
| Premium (classic), Premium v2 | Instance dành riêng, tính năng đầy đủ nhất | Yêu cầu mạng và độ sẵn sàng cao nhất |

Danh sách tính năng theo tier thay đổi theo thời gian; đọc trang so sánh tier trên Microsoft Learn trước khi chọn. **Self-hosted gateway** là gateway chạy bằng container ở nơi bạn chọn (datacenter riêng, cụm Kubernetes khác), vẫn nhận cấu hình từ APIM trên Azure; nó hợp khi backend không ở trên Azure và chỉ có ở một số tier dedicated. Tới Nơi chạy trên Container Apps nên chưa cần.

## Import OpenAPI, backend, product và subscription key

Đưa API vào APIM bằng cách import đặc tả OpenAPI của `trip-service`; APIM tạo sẵn các operation như `GET /trips/{id}`, `POST /trips`. **Backend** là địa chỉ thật (`https://<fqdn-ca-toinoi-trips-dev>`) mà mọi operation chuyển tiếp tới. Đặc tả này là hợp đồng công khai, nên chỉ mô tả những route đối tác được dùng và bạn sẵn sàng giữ ổn định.

**Product** là gói cấp quyền: gom một hay nhiều API, kèm điều khoản và policy riêng. Tới Nơi có product `Partner` cho đối tác thương mại. **Subscription** là cặp key (primary, secondary) gắn với product, đại diện cho một đối tác. Client gửi key trong header `Ocp-Apim-Subscription-Key`; APIM tra ra subscription, biết đối tác nào gọi và áp quota nào. Hai key cho phép xoay vòng không gián đoạn: đối tác chuyển sang secondary rồi bạn mới tạo lại primary.

Subscription key chỉ nhận diện bên gọi, không chứng minh người dùng cuối là ai; key lộ thì ai cầm cũng gọi được dưới danh nghĩa đối tác cho tới khi bạn thu hồi. Với API tạo chuyến, key chỉ là lớp đầu, lớp sau là JWT.

## Policy là pipeline XML chạy theo scope và theo giai đoạn

Policy thay đổi hành vi request mà không đụng code backend. Mỗi định nghĩa policy là XML chia bốn giai đoạn theo thứ tự: `inbound` trước khi chuyển tiếp, `backend` quanh lúc gọi backend, `outbound` xử lý phản hồi, `on-error` khi có lỗi ở bất kỳ bước nào. Phần lớn công việc nằm ở `inbound`.

Policy gắn được ở bốn **scope**: global, product, API và operation (thêm workspace nếu dùng). Chúng lồng nhau chứ không đè nhau: thẻ `<base />` đánh dấu chỗ chèn policy của scope cha trong cùng giai đoạn. Thứ tự thực thi vì thế do vị trí `<base />` quyết định; đặt nó ở đầu mọi khối thì inbound chạy global, product, API, operation. Quên `<base />` ở một scope là bỏ mất toàn bộ policy của scope cha mà không có lỗi nào báo.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Khi mỗi scope đặt base ở đầu khối, policy inbound chạy từ scope ngoài vào trong: global, product, API, operation."><defs><marker id="az39-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">1. Global</text><text x="34" y="123" class="t-small">Mọi API, mọi product</text><path d="M210 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az39-1-a)"/><rect x="240" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="254" y="95" class="t-b">2. Product Partner</text><text x="254" y="123" class="t-small">Quota, giới hạn gói</text><path d="M430 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az39-1-a)"/><rect x="460" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">3. API trips</text><text x="474" y="123" class="t-small">validate-jwt, header</text><path d="M650 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az39-1-a)"/><rect x="680" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">4. Operation</text><text x="694" y="123" class="t-small">rewrite-uri, cache</text></svg>
</div>
<figcaption>Hình 2. Khi mỗi scope đặt base ở đầu khối, policy inbound chạy từ scope ngoài vào trong: global, product, API, operation.</figcaption>
</figure>

Khối dưới là policy ở scope API `trips-api`. Nó kiểm JWT của Entra External ID (bài 19) với đúng audience và role, giới hạn 60 lượt mỗi phút cho mỗi subscription, gắn tên đối tác vào header gửi backend và bỏ key khỏi request chuyển tiếp. Khối không chứng minh thứ tự đúng ở mọi scope; hãy xem policy hiệu lực bằng trace.

```xml
<policies>
  <inbound>
    <base />
    <validate-jwt header-name="Authorization" failed-validation-httpcode="401"
        failed-validation-error-message="Token không hợp lệ">
      <openid-config url="{{entra-openid-config}}" />
      <audiences>
        <audience>API_CLIENT_ID</audience>
        <audience>api://API_CLIENT_ID</audience>
      </audiences>
      <required-claims>
        <claim name="roles" match="any"><value>partner</value></claim>
      </required-claims>
    </validate-jwt>
    <rate-limit calls="60" renewal-period="60" />
    <set-header name="X-Partner" exists-action="override">
      <value>@(context.Subscription.Name)</value>
    </set-header>
    <set-header name="Ocp-Apim-Subscription-Key" exists-action="delete" />
  </inbound>
  <backend><base /></backend>
  <outbound><base /></outbound>
  <on-error><base /></on-error>
</policies>
```

Quota ngày đặt ở scope product `Partner`, vì policy `quota` chỉ dùng được ở scope product và đếm theo subscription:

```xml
<policies>
  <inbound>
    <base />
    <quota calls="1000" renewal-period="86400" />
  </inbound>
  <backend><base /></backend>
  <outbound><base /></outbound>
  <on-error><base /></on-error>
</policies>
```

Vài điểm cần đọc kỹ. `rate-limit` và `quota` đếm theo subscription nên mỗi đối tác có bộ đếm riêng, và cả hai có ở mọi tier kể cả Consumption. Bản `rate-limit-by-key` và `quota-by-key` cho tự chọn `counter-key` (ví dụ một claim trong token) nhưng **không có ở Consumption**, chỉ ở tier dedicated; kiểm dòng Gateways trên trang từng policy. Chọn khoá đếm sai, như IP, khiến đối tác sau NAT chung bị chặn oan hoặc một đối tác lách giới hạn. Vượt rate limit trả `429`, vượt quota trả `403`, nên ghi rõ trong tài liệu cho đối tác. Bộ đếm không chính xác tuyệt đối khi gateway có nhiều instance: dùng để bảo vệ backend và giới hạn thương mại, không để tính tiền. Cuối cùng, access token v2 của Entra thường có `aud` là client ID dạng GUID chứ không phải `api://...`, nên khối trên chấp nhận cả hai.

`{{entra-openid-config}}` là **named value**: giá trị đặt tên dùng lại trong nhiều policy, có thể là chuỗi thường, giá trị bí mật, hoặc tham chiếu secret trong Key Vault ([bài 14](/articles/azure-key-vault-secret-key-certificate)). Với loại sau, managed identity của APIM cần quyền đọc secret ([bài 13](/articles/azure-managed-identity-workload-identity)); đổi secret ở Key Vault thì APIM tự lấy giá trị mới sau một khoảng, không cần sửa policy.

Hai policy khác hợp với scope hẹp hơn. `rewrite-uri` đổi đường dẫn gửi backend khi route công khai khác route nội bộ, chẳng hạn đối tác gọi `GET /trips/{id}/receipt` còn backend nhận `/internal/receipts?tripId={id}`; tiền tố `partner` của API thì APIM đã tự bỏ khi chuyển tiếp. `cache-lookup` cùng `cache-store` lưu phản hồi của operation ít đổi như danh sách thành phố; Consumption không có cache dựng sẵn nên phải gắn Azure Cache for Redis bên ngoài. Đừng cache phản hồi phụ thuộc người gọi nếu khoá cache không phân biệt người gọi.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>APIM không phải WAF. Nó kiểm key, token và đếm lượt, nhưng không lọc payload tấn công hay chặn một đợt bot gửi hàng triệu request. Ở tier tính theo lượt gọi, request rác có thể vẫn bị tính tiền dù cuối cùng bị từ chối. Đặt Front Door hoặc Application Gateway kèm WAF ở phía trước (<a href="/articles/azure-front-door-cdn-custom-domain-tls">bài về Front Door</a> và <a href="/articles/azure-firewall-waf-ddos-bao-ve-bien">bài về WAF</a>), và giới hạn để backend chỉ nhận traffic từ gateway.</p>
</div>

## Version là hợp đồng mới, revision là sửa không phá hợp đồng

Hai khái niệm này hay bị nhầm. **Version** dành cho thay đổi phá vỡ tương thích: đổi tên hay bỏ field, đổi nghĩa mã lỗi. Các version nằm trong một version set, client chọn version qua path (`/partner/v1/trips`), header hoặc query string. Path dễ thấy và dễ cache nhất; header giữ URL ổn định nhưng khó thử bằng trình duyệt. **Revision** dành cho thay đổi không phá vỡ như thêm field tuỳ chọn, thêm operation: sửa trên revision riêng, thử qua URL của revision (dạng `;rev=2`), rồi đặt làm bản hiện hành kèm ghi chú thay đổi.

Bỏ một version cần lộ trình, không phải một lệnh xoá: công bố ngày ngừng, thêm header `Deprecation` và `Sunset` ở `outbound` của version cũ để client tự phát hiện, theo dõi lượt gọi còn lại theo subscription qua log, liên hệ từng đối tác, và chỉ gỡ khi lượt gọi về gần không.

**Developer portal** cho đối tác tự đăng ký, đọc tài liệu sinh từ OpenAPI, thử API và lấy key. Portal phải tuỳ biến và publish trước khi dùng, và **không có ở Consumption**, nên production cho đối tác cần tier dedicated (v2 trở lên). Bật phê duyệt thủ công cho product `Partner` vì đối tác nhận quyền tạo chuyến thật.

## Quan sát: log ở gateway, trace xuyên service

APIM gửi log và metric tới Application Insights và Log Analytics ([bài 10](/articles/azure-monitor-metrics-activity-log-alert-dau-tien)). Application Insights cho trace từng request kèm phụ thuộc; Log Analytics với bảng gateway log cho thống kê theo API, subscription, mã phản hồi. Khối KQL dưới đếm lỗi theo đối tác trong một giờ, giả định diagnostic setting ở chế độ resource-specific; nó không cho biết lỗi nằm ở gateway hay backend.

```kusto
ApiManagementGatewayLogs
| where TimeGenerated > ago(1h)
| summarize requests = count(), errors = countif(ResponseCode >= 400)
    by ApiId, SubscriptionId = tostring(ApimSubscriptionId), ResponseCode
| order by errors desc
```

Để nối trace, logger Application Insights của APIM truyền header `traceparent` (W3C) tới backend; service có OpenTelemetry ([bài 37 loạt C#](/articles/dotnet10-opentelemetry-traces-metrics-logs)) nối span vào cùng trace. Tên tuỳ chọn correlation khác nhau giữa các phiên bản, hãy kiểm tài liệu. Để debug từng request, **trace** của APIM trả về chuỗi policy đã chạy, hữu ích nhất khi tìm policy nào chặn request. Cách bật trace đã đổi (Test console trong portal, hoặc debug credential ngắn hạn thay cho header cũ), và trace lộ chi tiết nội bộ nên không bật bừa trên production.

## Lab: APIM Consumption, product Partner, key và token

Lab mô tả cách làm; tên APIM phải duy nhất toàn cầu. Khối đầu tạo instance Consumption, import OpenAPI, tạo product `Partner`, gắn API và tạo named value. Nó chưa gắn policy và chưa tạo subscription:

```bash
RG=rg-toinoi-apim-dev-sea
APIM=apim-toinoi-dev-sea

az group create -n $RG -l southeastasia
az apim create -g $RG -n $APIM -l southeastasia \
  --publisher-email <email-lien-he> --publisher-name "Toi Noi Lab" \
  --sku-name Consumption

az apim api import -g $RG --service-name $APIM \
  --api-id trips-api --path partner --display-name "Trips API" \
  --specification-format OpenApiJson --specification-path ./trips-openapi.json \
  --service-url https://<fqdn-ca-toinoi-trips-dev>

az apim product create -g $RG --service-name $APIM \
  --product-id partner --product-name "Partner" \
  --subscription-required true --approval-required true --state published
az apim product api add -g $RG --service-name $APIM \
  --product-id partner --api-id trips-api
az apim nv create -g $RG --service-name $APIM --named-value-id entra-openid-config \
  --display-name entra-openid-config \
  --value "https://<tenant-subdomain>.ciamlogin.com/<tenant-id>/v2.0/.well-known/openid-configuration"
```

Các bước còn lại làm trong portal hoặc ARM REST (bài 40 đưa chúng vào Bicep): dán policy API vào `trips-api` và policy quota vào product `Partner`; thêm app role `partner` vào `app-toinoi-trips-api` và gán cho người dùng thử như bài 19; tạo subscription thuộc `Partner`, lấy primary key vào biến `KEY`. Rồi kiểm bốn tình huống bằng `curl`, với `TOKEN` lấy theo bài 19:

```bash
GW=https://apim-toinoi-dev-sea.azure-api.net
H="Ocp-Apim-Subscription-Key"

curl -i $GW/partner/trips/<trip-id>                                  # thiếu key: 401
curl -i -H "$H: key-sai" -H "Authorization: Bearer $TOKEN" \
  $GW/partner/trips/<trip-id>                                        # key sai: 401
curl -i -H "$H: $KEY" $GW/partner/trips/<trip-id>                    # có key, thiếu token: 401
curl -i -H "$H: $KEY" -H "Authorization: Bearer $TOKEN" \
  $GW/partner/trips/<trip-id>                                        # đủ cả hai: 200 nếu backend ổn
```

Ba lệnh đầu cùng trả `401` vì ba lý do khác nhau, body cho biết lý do. Lệnh cuối kỳ vọng `200` nếu `aud` và `roles` đúng; nếu vẫn `401`, so audience trong policy với claim `aud` đã decode ở bài 19. Kiểm quota bằng cách tạm hạ `calls` xuống vài đơn vị rồi gọi vượt, kỳ vọng `403`. Cuối cùng xem trace một request và chạy khối KQL. Dọn lab bằng `az group delete -n $RG`.

**Bài tập:** đối tác muốn đổi `fare` từ số nguyên sang object `{amount, currency}` trong phản hồi của `GET /trips/{id}`. Dùng version hay revision? Và bạn làm gì với đối tác cũ?

<details><summary>Xem lời giải</summary><p>Đây là thay đổi phá vỡ tương thích nên cần version mới (`v2`), không phải revision. Giữ `v1` chạy song song, thêm header `Deprecation` và `Sunset` ở outbound của `v1`, theo dõi lượt gọi còn lại theo subscription trong log và liên hệ từng đối tác. Chỉ gỡ `v1` khi lượt gọi gần như về không.</p></details>

## Học tiếp từ đây

Bạn vừa đặt API Management làm cổng cho đối tác của Tới Nơi: import OpenAPI, product Partner với subscription key và quota, policy validate-jwt, cùng cách quản lý version, revision và log. Đây là bài cuối của Phần 4.

Phần 5 (DevOps trên Azure) bắt đầu bằng [bài 40](/articles/azure-bicep-co-ban-module-parameter-what-if): Bicep, để mô tả toàn bộ những thứ bạn vừa tạo bằng tay (resource group, APIM, product, named value) thành code có thể review và chạy lại. Nếu cần xem lại luồng dữ liệu phía sau cổng, bài trước là [bài 38](/articles/azure-data-lake-data-factory-bao-cao).

## Tài liệu gốc

- [Azure API Management documentation](https://learn.microsoft.com/en-us/azure/api-management/)
- [API Management key concepts](https://learn.microsoft.com/en-us/azure/api-management/api-management-key-concepts)
- [Policies trong API Management](https://learn.microsoft.com/en-us/azure/api-management/api-management-howto-policies)
- [Versions và revisions](https://learn.microsoft.com/en-us/azure/api-management/api-management-versions)
- [Products và subscriptions](https://learn.microsoft.com/en-us/azure/api-management/api-management-howto-add-products)
- [Named values trong API Management](https://learn.microsoft.com/en-us/azure/api-management/api-management-howto-properties)

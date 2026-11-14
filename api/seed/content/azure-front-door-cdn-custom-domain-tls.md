---
slug: azure-front-door-cdn-custom-domain-tls
title: "Front Door, CDN, custom domain và TLS"
summary: "Bài 28: đặt Azure Front Door trước trip-service của Tới Nơi, dựng endpoint, origin group, route, custom domain api.toinoi.example với managed certificate, chuyển WAF sang Prevention và khoá origin để không ai đi vòng qua edge."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, front-door, cdn, tls, networking]
language: vi
---

Đây là **bài 28** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 9/10 của phần).

Bài này đặt một cửa vào toàn cầu trước API của Tới Nơi: Front Door nhận kết nối ở điểm gần khách, kết thúc TLS, chạy WAF rồi mới chuyển request về Container App. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn đặt Front Door trước Container App và gắn WAF Prevention. Từ giờ mọi request tới API chắc chắn đã đi qua WAF, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Container App vẫn có FQDN công khai của riêng nó. Ai biết địa chỉ đó có thể gọi thẳng và bỏ qua toàn bộ edge. Muốn «mọi request đi qua WAF» thì phải khoá origin, và đó là mục 4.</p></details>
</div>

## Front Door là cửa vào toàn cầu, không phải Application Gateway đặt xa hơn

Azure có ba dịch vụ hay bị nhầm lẫn khi cần «đứng trước ứng dụng». Chúng khác nhau ở tầng hoạt động và phạm vi, không ở độ mạnh.

| | Front Door | Application Gateway | Traffic Manager |
| --- | --- | --- | --- |
| Phạm vi | Toàn cầu, nhiều POP | Một region, nằm trong VNet | Toàn cầu, chỉ ở tầng DNS |
| Hoạt động ở | L7, HTTP/HTTPS | L7, HTTP/HTTPS | DNS, không thấy request |
| Thấy traffic của khách | Có, TLS kết thúc tại edge | Có, TLS kết thúc tại gateway | Không, chỉ trả về một địa chỉ |
| Cache nội dung tĩnh | Có | Không | Không |
| WAF | Có | Có | Không |
| Đi tới origin riêng tư | Có, qua Private Link (Premium) | Có, vì ở trong VNet | Không áp dụng |

Front Door dùng địa chỉ anycast: cùng một IP được quảng bá từ nhiều POP, nên kết nối của khách tự rơi vào POP gần nhất theo định tuyến mạng. TCP và TLS handshake diễn ra ở POP đó. Từ POP về origin, Front Door giữ sẵn kết nối để dùng lại trên mạng backbone của Microsoft. Với app tài xế ở Đà Nẵng gọi API đặt ở Singapore, phần lợi thường nằm ở handshake ngắn hơn và đường về origin ổn định hơn, không phải ở việc origin chạy nhanh hơn.

Traffic Manager chỉ trả lời truy vấn DNS rồi biến mất khỏi đường đi. Application Gateway (bài 27, [mạng cho ứng dụng](/articles/azure-vnet-integration-application-gateway)) nằm trong VNet nên hợp để đứng trước workload riêng tư trong một region. Hai dịch vụ này ghép được với nhau: Front Door ở ngoài cùng, Application Gateway ở trong. Tới Nơi mới có một region chính nên chỉ dùng Front Door là đủ cho bài này.

Về CDN: các SKU Azure CDN cũ và cả Front Door (classic) đều đã có lịch ngừng hỗ trợ; hướng hiện hành là Front Door Standard/Premium, nơi caching là một tính năng của route. Kiểm tra thông báo retirement trong tài liệu trước khi tạo profile CDN độc lập mới.

## Bốn tầng cấu hình: endpoint, origin group, origin, route

Một request đi qua bốn khái niệm, theo đúng thứ tự dưới đây.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Đường đi của một request: khách vào POP gần nhất, route chọn origin group, health probe lọc origin khoẻ, rồi tới Container App."><defs><marker id="az28-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Khách / app tài xế</text><text x="34" y="123" class="t-small">api.toinoi.example</text><path d="M210 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az28-0-a)"/><rect x="240" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="95" class="t-b">POP + endpoint</text><text x="254" y="123" class="t-small">TLS, WAF, cache</text><path d="M430 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az28-0-a)"/><rect x="460" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">Route + origin group</text><text x="474" y="123" class="t-small">Path, probe, weight</text><path d="M650 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az28-0-a)"/><rect x="680" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="694" y="95" class="t-b">Origin</text><text x="694" y="123" class="t-small">ca-toinoi-trips-prod</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một request: khách vào POP gần nhất, route chọn origin group, health probe lọc origin khoẻ, rồi tới Container App.</figcaption>
</figure>

- **Profile** là vỏ ngoài chứa mọi thứ và quyết định SKU (Standard hoặc Premium).
- **Endpoint** là tên miền `*.azurefd.net` do Front Door cấp. Domain riêng của bạn được gắn vào route.
- **Origin** là nơi thật sự xử lý request: Container App, App Service, Blob, hay bất kỳ host nào có địa chỉ công khai.
- **Origin group** gom các origin cùng vai trò và mang **health probe**. Probe là request định kỳ tới một path, thường `/health/ready` của `trip-service`. Origin không qua được probe sẽ bị loại khỏi danh sách được chọn. Probe khác với giám sát của khách: nếu cả group cùng không qua được probe, Front Door vẫn thử gửi request chứ không tự trả lỗi thay bạn.
- **Route** nối domain và pattern path với một origin group, quyết định giao thức, redirect HTTP sang HTTPS và có bật cache hay không.

Có hai cấu hình dễ chọn sai. **Session affinity** ghim một khách vào một origin qua cookie. Chỉ bật khi origin giữ state trong bộ nhớ. `trip-service` stateless nên để tắt, nếu không một replica nóng sẽ giữ khách cố định và phá cân bằng tải. **Rule set** là nơi viết rewrite: thêm hoặc đổi header request và response, đổi URL. Một use case hợp lý là thêm header `Strict-Transport-Security` vào mọi response.

## Custom domain: CNAME, TXT và managed certificate

Domain `api.toinoi.example` phải được chứng minh là của bạn trước khi Front Door cấp chứng chỉ. `toinoi.example` là tên dành riêng cho ví dụ, nên khi chạy lab hãy thay bằng domain bạn sở hữu.

Có hai bản ghi DNS. Bản ghi **CNAME** trỏ `api` về hostname endpoint để traffic đi vào Front Door. Bản ghi **TXT** tên `_dnsauth.api`, giá trị là token do Front Door sinh, để chứng minh quyền sở hữu khi cấp **managed certificate**. Managed certificate do Front Door phát hành và tự gia hạn, bạn không giữ private key. Muốn tự quản chứng chỉ thì dùng certificate trong Key Vault (bài 14, [Key Vault](/articles/azure-key-vault-secret-key-certificate)) và Front Door đọc từ đó. Đổi lại, bạn phải lo gia hạn và quyền truy cập.

Hai thiết lập cuối làm cho TLS thực sự có tác dụng: phiên bản TLS tối thiểu 1.2 (Front Door đã ngừng nhận TLS 1.0 và 1.1, nhưng vẫn nên ghi rõ trong cấu hình), và route redirect HTTP sang HTTPS. Redirect không bảo vệ lần gọi HTTP đầu tiên của một client viết ẩu, nên với API hãy từ chối HTTP hẳn ở client mobile và để redirect chỉ phục vụ trình duyệt.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Chuyển WAF từ Detection sang Prevention nghĩa là rule khớp sẽ chặn request thật. Một false positive trên ghi chú giao hàng của app tài xế là đơn bị mất, không còn là một dòng log. Ở bài 18 bạn đã đọc log vài ngày; đừng bỏ bước đó. Ngoài ra Front Door tính tiền theo giờ cấu hình, theo lượng request và theo dữ liệu truyền ra, nên lab nào cũng phải xoá profile khi xong. Tra Pricing Calculator cho SKU bạn chọn.</p>
</div>

## Khoá origin: tránh bypass Front Door

Đây là lỗ hổng phổ biến nhất của kiến trúc có edge. Container App có FQDN `ca-toinoi-trips-prod.<hash>.southeastasia.azurecontainerapps.io` truy cập được từ Internet. Nếu không làm gì thêm, kẻ tấn công gọi thẳng địa chỉ đó và không gặp WAF, rate limit hay cache nào.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Hai con đường tới origin: qua Front Door được phép, gọi thẳng FQDN của origin phải bị từ chối."><defs><marker id="az28-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="az28-1-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-dead"/></marker></defs><rect x="20" y="40" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="72" class="t-b">Khách thật</text><text x="34" y="98" class="t-small">api.toinoi.example</text><path d="M210 80 L300 80" class="f-none s-phys" stroke-width="2" marker-end="url(#az28-1-a)"/><rect x="300" y="40" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="314" y="72" class="t-b">Front Door + WAF</text><text x="314" y="98" class="t-small">Gắn header X-Azure-FDID</text><path d="M550 80 L640 80" class="f-none s-phys" stroke-width="2" marker-end="url(#az28-1-a)"/><rect x="640" y="40" width="220" height="220" rx="10" class="f-phys-soft s-phys"/><text x="654" y="72" class="t-b">Container App</text><text x="654" y="98" class="t-small">Ingress công khai</text><text x="654" y="150" class="t-small">Chỉ nhận request có</text><text x="654" y="174" class="t-small">đúng FDID và IP của</text><text x="654" y="198" class="t-small">Front Door</text><rect x="20" y="180" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="34" y="212" class="t-b">Kẻ bỏ qua WAF</text><text x="34" y="238" class="t-small">Biết FQDN của origin</text><path d="M210 220 L630 220" class="f-none s-dead" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#az28-1-b)"/><text x="330" y="210" class="t-small">Gọi thẳng origin</text><text x="330" y="246" class="t-dead">Bị từ chối (403)</text></svg>
</div>
<figcaption>Hình 2. Hai con đường tới origin: qua Front Door được phép, gọi thẳng FQDN của origin phải bị từ chối.</figcaption>
</figure>

Có hai cách, độ chắc khác nhau:

| Cách | Cơ chế | Điểm yếu |
| --- | --- | --- |
| Private Link (Premium) | Front Door nối tới origin qua kết nối riêng tư; origin tắt truy cập công khai | Cần SKU Premium; origin phải thuộc loại được hỗ trợ, kiểm tra bảng trong tài liệu; có bước phê duyệt kết nối |
| Header `X-Azure-FDID` + IP restriction | Front Door gắn ID profile vào header; origin chỉ nhận IP của Front Door và đúng ID | Header đứng một mình có thể bị giả; IP của Front Door là dải chung, nên phải kèm kiểm tra ID |

Hai lớp của cách thứ hai bổ sung cho nhau. IP restriction loại phần lớn Internet nhưng chia sẻ dải với mọi profile Front Door khác trên thế giới. Kiểm tra `X-Azure-FDID` loại những profile khác. Thiếu một trong hai là chưa khoá. Lấy ID bằng `az afd profile show --query frontDoorId`.

Lệnh dưới đặt IP restriction trên ingress của Container Apps. Ingress này nhận CIDR, không nhận tên service tag, nên mỗi dải của `AzureFrontDoor.Backend` là một rule (lấy danh sách bằng `az network list-service-tags --location southeastasia` hoặc file Service Tags tải về). Lệnh không chứng minh dải bạn nhập là dải hiện hành: danh sách thay đổi, cần kế hoạch cập nhật định kỳ và kiểm tra giới hạn số rule. App Service thì gọn hơn: access restriction nhận thẳng `--service-tag AzureFrontDoor.Backend` kèm `--http-header x-azure-fdid=<id>`.

```bash
az containerapp ingress access-restriction set \
  --name ca-toinoi-trips-prod --resource-group rg-toinoi-trips-prod-sea \
  --rule-name afd-backend --ip-address <cidr-trong-service-tag-AzureFrontDoor.Backend> \
  --action Allow
```

Phần kiểm tra header nằm trong ứng dụng. Đoạn dưới từ chối request không mang đúng ID, trừ path `/health` để có thể kiểm tra sức khoẻ trực tiếp khi chẩn đoán. Đoạn này dừng app ngay lúc khởi động nếu thiếu `FrontDoor:Id`, vì so sánh với giá trị rỗng thì chẳng kiểm tra được gì; đoạn này không thay thế IP restriction.

```csharp
var fdid = builder.Configuration["FrontDoor:Id"]
    ?? throw new InvalidOperationException("Thiếu FrontDoor:Id");

app.Use(async (ctx, next) =>
{
    var isHealth = ctx.Request.Path.StartsWithSegments("/health");
    if (!isHealth && ctx.Request.Headers["X-Azure-FDID"] != fdid)
    {
        ctx.Response.StatusCode = StatusCodes.Status403Forbidden;
        return;
    }
    await next();
});
```

Còn một lối vòng nữa: domain mặc định `*.azurefd.net` của endpoint. Nếu WAF chỉ gắn vào custom domain, khách gọi qua domain mặc định vẫn né rule. Gắn security policy cho cả hai, hoặc tắt liên kết với domain mặc định ở route khi custom domain đã chạy.

## Ảnh tĩnh nên đi qua cache của edge, không qua trip-service

Logo quán và ảnh món nằm trong Blob (bài [Storage account](/articles/azure-storage-account-blob-tier-sas)) không cần đi qua `trip-service`. Tạo origin group thứ hai trỏ tới `sttoinoiassetsprod.blob.core.windows.net` và một route cho path `/img/*`. Nên dùng storage account riêng chỉ chứa tài nguyên công khai: bật anonymous read trên account chứa dữ liệu nghiệp vụ là một quyết định tồi. Muốn đóng hẳn truy cập công khai thì dùng Private Link của Premium.

Cơ chế cache như sau: Front Door lưu response ở POP, trả lại cho khách tiếp theo trong thời gian hết hạn do header `Cache-Control` của origin hoặc do rule của bạn. Hai tham số quyết định hiệu quả là **TTL** và **query string**. Với ảnh bất biến, để query string bị bỏ qua khi tính khoá cache và đặt TTL dài. Với API động thì không cache gì cả; `POST /quotes` không bao giờ nên vào cache.

Hậu quả của TTL dài là ảnh cũ nằm lại ở POP cho đến khi hết hạn, kể cả khi bạn đã thay file ở origin. Có hai cách: **purge** để xoá cache theo path, hoặc đặt tên file có hash nội dung (`logo.3fa9c1.webp`) để bản mới là URL mới. Cách thứ hai rẻ và chắc hơn, purge nên giữ cho trường hợp khẩn cấp.

## Lab: Front Door Standard trước Container App

Lab dựng profile Standard trước Container App môi trường dev (environment external, khác lab internal của bài 27). Khối đầu tạo profile, endpoint, origin group với probe, origin và route:

```bash
RG=rg-toinoi-edge-dev-sea; P=afd-toinoi-dev; EP=toinoi-api-dev
FQDN=$(az containerapp show -n ca-toinoi-trips-dev \
  -g rg-toinoi-trips-dev-sea \
  --query properties.configuration.ingress.fqdn -o tsv)

az group create -n $RG -l southeastasia
az afd profile create -g $RG --profile-name $P \
  --sku Standard_AzureFrontDoor
az afd endpoint create -g $RG --profile-name $P \
  --endpoint-name $EP --enabled-state Enabled
az afd origin-group create -g $RG --profile-name $P \
  --origin-group-name og-trips --probe-request-type GET \
  --probe-protocol Https --probe-path /health/ready \
  --probe-interval-in-seconds 30 --sample-size 4 \
  --successful-samples-required 3 \
  --additional-latency-in-milliseconds 50
az afd origin create -g $RG --profile-name $P \
  --origin-group-name og-trips --origin-name ca-trips \
  --host-name $FQDN --origin-host-header $FQDN \
  --http-port 80 --https-port 443 --priority 1 --weight 1000
az afd route create -g $RG --profile-name $P \
  --endpoint-name $EP --route-name r-api \
  --origin-group og-trips --supported-protocols Https Http \
  --https-redirect Enabled --forwarding-protocol HttpsOnly \
  --link-to-default-domain Enabled --patterns-to-match "/*"
```

Khối này mô tả cách làm, không chứng minh origin đã khoẻ: sau khi chạy, kiểm tra trạng thái probe trong portal rồi `curl` endpoint. Tên tham số có thể khác theo phiên bản extension `afd`, nên chạy `az afd route create -h` khi gặp lỗi. Tiếp theo là custom domain:

```bash
az afd custom-domain create -g $RG --profile-name $P \
  --custom-domain-name api-toinoi \
  --host-name api.toinoi.example \
  --minimum-tls-version TLS12 \
  --certificate-type ManagedCertificate

# Lấy token cho bản ghi TXT _dnsauth.api
az afd custom-domain show -g $RG --profile-name $P \
  --custom-domain-name api-toinoi \
  --query validationProperties -o json

# Sau khi tạo CNAME và TXT, gắn domain vào route
az afd route update -g $RG --profile-name $P \
  --endpoint-name $EP --route-name r-api \
  --custom-domains api-toinoi
```

Trạng thái validation chuyển từ pending sang approved sau khi DNS lan truyền, có thể mất một lúc. Managed certificate chỉ được triển khai khi domain đã approved và đã gắn vào route; trước đó gọi HTTPS tới domain sẽ lỗi chứng chỉ. Cuối cùng là WAF. Policy của bài 18 được tạo cho tier Premium, mà SKU của WAF policy phải khớp SKU của profile, nên với profile Standard ta tạo một policy Standard ở Detection, gắn vào cả domain mặc định lẫn custom domain, chép custom rule rate-limit của bài 18 sang (không hiện ở đây), rồi chỉ chuyển Prevention khi đã đọc log Detection:

```bash
az network front-door waf-policy create -g $RG -n waftoinoistddev \
  --sku Standard_AzureFrontDoor --mode Detection
WAF_ID=$(az network front-door waf-policy show \
  -g $RG -n waftoinoistddev --query id -o tsv)

az afd security-policy create -g $RG --profile-name $P \
  --security-policy-name sp-api --waf-policy $WAF_ID \
  --domains $(az afd endpoint show -g $RG \
      --profile-name $P --endpoint-name $EP --query id -o tsv) \
    $(az afd custom-domain show -g $RG --profile-name $P \
      --custom-domain-name api-toinoi --query id -o tsv)

# Chép custom rule của bài 18, đọc log Detection vài ngày, rồi:
az network front-door waf-policy update -g $RG \
  -n waftoinoistddev --mode Prevention
```

Về tier: WAF trên Standard chỉ có custom rule như rate limit của bài 18; managed rule set (chặn SQLi, XSS) và bot protection cần Premium, cũng như Private Link tới origin. Nếu Tới Nơi cần managed rule ở prod, chọn Premium và dùng thẳng policy của bài 18; kiểm tra bảng so sánh tier trước khi quyết định.

Đo TTFB từ hai vị trí, ví dụ Cloud Shell hoặc một VM ở Southeast Asia và một VM ở region xa như West Europe. Chạy lệnh dưới nhiều lần ở mỗi nơi, so FQDN origin với domain qua Front Door, rồi lấy trung vị. Kết quả phụ thuộc vị trí và thời điểm; với API động bạn kỳ vọng thấy chênh lệch ở máy xa origin, chứ chưa thể đoán trước con số.

```bash
for i in 1 2 3 4 5; do
  curl -s -o /dev/null -w "%{time_connect} %{time_appconnect} %{time_starttransfer}\n" \
    https://api.toinoi.example/health/ready
done
```

Ba cột là thời gian TCP, TLS handshake và byte đầu tiên. Xong lab, xoá cả resource group `rg-toinoi-edge-dev-sea` để dừng tính tiền.

**Bài tập:** chủ quán đổi logo nhưng giữ nguyên tên `/img/q123/logo.webp`, route đặt TTL một ngày. Khách vẫn thấy logo cũ. Bạn xử lý ngay thế nào, và đổi quy ước gì để không gặp lại?

<details><summary>Xem lời giải</summary><p>Ngay lập tức: purge path `/img/q123/*` bằng `az afd endpoint purge`, vì xoá file ở origin không làm POP xoá cache. Về lâu dài: đặt tên file kèm hash nội dung để mỗi lần đổi là một URL mới, khi đó TTL dài không còn là vấn đề và purge chỉ còn dùng cho trường hợp khẩn.</p></details>

## Học tiếp từ đây

Bạn đã có đường vào hoàn chỉnh: Front Door nhận request ở POP gần khách, kết thúc TLS với domain `api.toinoi.example`, chạy WAF Prevention, định tuyến theo path, cache ảnh tĩnh và khoá origin để không ai đi vòng. Nhớ rằng lab chỉ mô tả cách làm, bạn cần tự kiểm tra probe, validation và việc origin từ chối truy cập trực tiếp.

Bài kế tiếp là bài cuối của Phần 3, [Bài 29](/articles/azure-app-configuration-feature-flag-key-vault-reference): App Configuration, nơi đặt cờ tính năng và cấu hình đổi được mà không phải build lại image. Bạn có thể xem lại [Bài 18](/articles/azure-firewall-waf-ddos-bao-ve-bien) về WAF và [Bài 22](/articles/azure-container-apps-revision-ingress-scale) về ingress của Container Apps.

## Tài liệu gốc

- [Azure Front Door](https://learn.microsoft.com/en-us/azure/frontdoor/)
- [Front Door: origins và origin groups](https://learn.microsoft.com/en-us/azure/frontdoor/origin)
- [Add a custom domain to Azure Front Door](https://learn.microsoft.com/en-us/azure/frontdoor/standard-premium/how-to-add-custom-domain)
- [Secure your origin with Private Link in Front Door Premium](https://learn.microsoft.com/en-us/azure/frontdoor/private-link)
- [Caching with Azure Front Door](https://learn.microsoft.com/en-us/azure/frontdoor/front-door-caching)
- [Azure Web Application Firewall](https://learn.microsoft.com/en-us/azure/web-application-firewall/overview)

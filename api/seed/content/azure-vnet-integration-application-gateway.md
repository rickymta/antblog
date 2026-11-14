---
slug: azure-vnet-integration-application-gateway
title: "Mạng cho ứng dụng: VNet integration, Private Endpoint và Application Gateway"
summary: "Bài 27: nối ứng dụng vào mạng theo hai chiều (outbound tới PostgreSQL private, inbound qua private endpoint hoặc internal ingress), rồi dựng Application Gateway v2 với probe /health/ready phía trước Container Apps environment nội bộ của Tới Nơi."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, networking, application-gateway, container-apps, dns]
language: vi
---

Đây là **bài 27** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 8/10 của phần).

Bài này tách việc nối mạng cho ứng dụng thành hai chiều, rồi dựng mẫu Internet, Application Gateway, Container Apps nội bộ, PostgreSQL private cho `trip-service`. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn bật VNet integration cho một App Service để gọi PostgreSQL private. Sau đó khách ngoài Internet có bị chặn truy cập vào chính App Service đó không?</p>
<details><summary>Xem đáp án</summary><p>Không. VNet integration chỉ điều khiển chiều đi ra của ứng dụng. Chiều đi vào cần một cơ chế khác: private endpoint, access restriction hoặc một cổng phía trước.</p></details>
</div>

## Mạng cho ứng dụng có hai chiều và hai bộ công cụ

Một ứng dụng giao tiếp theo hai hướng, và mỗi hướng cần câu trả lời riêng. **Outbound**: `trip-service` gọi PostgreSQL, Key Vault, nhà cung cấp bên ngoài. **Inbound**: khách, tài xế và các service khác gọi vào `trip-service`.

Với outbound, App Service dùng **VNet integration**: instance của app được gắn vào một subnet được delegate cho App Service, nên lời gọi ra tới địa chỉ private trong VNet đi từ subnet đó. Nó không biến app thành thứ nằm trong VNet và không đóng cổng vào. Container Apps đơn giản hơn: environment được đặt thẳng vào subnet bạn chọn, nên traffic ra của mọi app trong environment đều đi từ subnet đó mà không cần thêm bước.

Với inbound, App Service dùng **private endpoint** (bài 17) để app có một IP private trong VNet, hoặc access restriction để lọc theo IP. Container Apps chọn lúc tạo environment, và không đổi được sau đó: **external** (có IP công cộng) hoặc **internal** (chỉ có IP private trong VNet). Environment workload profiles còn cho gắn private endpoint như App Service; kiểm trạng thái hỗ trợ trong tài liệu trước khi dựa vào nó. Trong environment internal, ingress đặt `external` nghĩa là «mở cho toàn VNet», còn ingress `internal` nghĩa là chỉ app khác trong cùng environment gọi được. Hai chữ trùng tên nhưng khác phạm vi, đây là chỗ dễ đọc nhầm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Hai chiều của mạng ứng dụng: inbound đi từ Internet qua Application Gateway vào environment nội bộ, outbound đi từ ứng dụng qua subnet tới PostgreSQL private."><defs><marker id="az27-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">Inbound</text><rect x="20" y="36" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="34" y="66" class="t-b">Internet</text><text x="34" y="94" class="t-small">Khách, tài xế, 443</text><path d="M280 78 L305 78" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-0-a)"/><rect x="305" y="36" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="66" class="t-b">Application Gateway</text><text x="319" y="94" class="t-small">Listener, WAF, probe</text><path d="M565 78 L590 78" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-0-a)"/><rect x="590" y="36" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="66" class="t-b">Environment internal</text><text x="604" y="94" class="t-small">Chỉ IP private trong VNet</text><text x="20" y="168" class="t-b">Outbound</text><rect x="20" y="180" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="210" class="t-b">trip-service</text><text x="34" y="238" class="t-small">Chạy trong snet-app</text><path d="M280 222 L305 222" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-0-a)"/><rect x="305" y="180" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="210" class="t-b">Private DNS</text><text x="319" y="238" class="t-small">Tên ra IP private</text><path d="M565 222 L590 222" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-0-a)"/><rect x="590" y="180" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="210" class="t-b">PostgreSQL private</text><text x="604" y="238" class="t-small">psql-toinoi-dev-sea</text></svg>
</div>
<figcaption>Hình 1. Hai chiều của mạng ứng dụng: inbound đi từ Internet qua Application Gateway vào environment nội bộ, outbound đi từ ứng dụng qua subnet tới PostgreSQL private.</figcaption>
</figure>

## Application Gateway v2 ghép sáu thành phần thành một đường vào

Application Gateway là bộ cân bằng tải ở tầng L7: nó hiểu HTTP, đọc host và path, rồi quyết định gửi tới đâu. Cấu hình của nó gồm các mảnh nối vào nhau, và hầu hết lỗi 502 đến từ việc hai mảnh không khớp:

| Thành phần | Việc nó làm | Lỗi thường gặp |
| --- | --- | --- |
| Frontend IP | Public IP (hoặc private) nhận request | Gateway v2 cần public IP Standard tĩnh nếu mở ra Internet |
| Listener | Cổng, giao thức, host name; nơi kết thúc TLS | Chứng chỉ sai host name |
| Routing rule | Nối listener với backend pool; basic hoặc theo path | Thiếu priority của rule |
| Backend pool | Danh sách đích: IP hoặc FQDN | FQDN không phân giải được trong VNet |
| HTTP setting | Cổng, giao thức, timeout, host header gửi tới backend | Host header sai nên backend trả 404 |
| Health probe | Gọi đường dẫn kiểm tra để quyết định backend khoẻ hay không | Probe tới `/` trong khi chỉ `/health/ready` trả 200 |

**URL path routing** cho phép một gateway phục vụ nhiều service: ví dụ `/api/quotes/*` tới pool của Pricing & Quote, `/api/trips/*` tới pool của Trip. Mỗi pool có HTTP setting và probe riêng. **WAF_v2** là SKU gắn thêm một WAF policy (bài 18); nguyên tắc vẫn là chạy Detection trước, đọc log, rồi mới chuyển Prevention.

Hai chi tiết cơ chế đáng nhớ. Thứ nhất, probe là nguồn sự thật về sức khoẻ backend: nếu mọi backend bị đánh dấu unhealthy, gateway trả 502 cho khách dù các app vẫn đang chạy. Dùng đúng `/health/ready` của `trip-service`, thứ đã kiểm database và migration (xem [bài triển khai AWS và Azure](/articles/dotnet10-aws-azure-deployment)), thay vì trang chủ. Thứ hai, **host header** quyết định Container Apps ingress route tới app nào, nên HTTP setting phải gửi đúng tên miền của app, thường bằng cách lấy host name từ backend pool.

### TLS: kết thúc ở gateway hay đi hết tới ứng dụng

Có hai mô hình. **TLS termination**: khách nói HTTPS với gateway, gateway giải mã, WAF đọc được nội dung, rồi gửi HTTP tới backend. Đơn giản, nhưng đoạn gateway tới backend không mã hoá, dù đoạn đó nằm trong VNet. **End-to-end TLS**: gateway giải mã để WAF kiểm tra, rồi mã hoá lại tới backend. Gateway cần tin chứng chỉ của backend (CA được tin, hoặc root certificate bạn nạp), và tên trong chứng chỉ phải khớp host header.

Ở Tới Nơi, dữ liệu đặt chuyến chứa vị trí và số điện thoại, nên staging và prod nên chọn end-to-end TLS. Dev có thể chọn termination cho nhanh, với điều kiện ghi rõ đó là khác biệt có chủ đích. Chứng chỉ phía listener nên nằm ở Key Vault và lấy bằng managed identity ([bài 14](/articles/azure-key-vault-secret-key-certificate)), không nằm trong file `.pfx` trên laptop.

## Mẫu Internet, gateway, Container Apps nội bộ, PostgreSQL private

Mẫu cho Tới Nơi: Internet vào Application Gateway (WAF) ở `snet-gateway`; gateway chuyển tới Container Apps environment internal ở `snet-app`; app gọi PostgreSQL qua mạng private. Nếu cần kiểm soát đường ra, mô hình hub-spoke ([bài 18](/articles/azure-firewall-waf-ddos-bao-ve-bien)) gắn route `0.0.0.0/0` trên subnet app vào Azure Firewall ở hub.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. Mẫu đầy đủ: Internet, Application Gateway WAF, Container Apps internal, PostgreSQL private; bên dưới là egress qua Firewall và private DNS."><defs><marker id="az27-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="34" y="50" class="t-b">Internet</text><text x="34" y="78" class="t-small">Khách, tài xế</text><path d="M205 62 L238 62" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-1-a)"/><rect x="238" y="20" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="252" y="50" class="t-b">App Gateway WAF_v2</text><text x="252" y="78" class="t-small">snet-gateway</text><path d="M423 62 L456 62" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-1-a)"/><rect x="456" y="20" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="470" y="50" class="t-b">Container Apps</text><text x="470" y="78" class="t-small">internal, snet-app</text><path d="M641 62 L674 62" class="f-none s-phys" stroke-width="2" marker-end="url(#az27-1-a)"/><rect x="674" y="20" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="688" y="50" class="t-b">PostgreSQL</text><text x="688" y="78" class="t-small">Private, không public</text><rect x="20" y="150" width="400" height="75" rx="10" class="f-surface s-line"/><text x="34" y="180" class="t-b">Egress</text><text x="34" y="206" class="t-small">UDR 0.0.0.0/0 tới Firewall ở hub</text><rect x="456" y="150" width="403" height="75" rx="10" class="f-surface s-line"/><text x="470" y="180" class="t-b">DNS</text><text x="470" y="206" class="t-small">Zone của environment, link vào VNet</text></svg>
</div>
<figcaption>Hình 2. Mẫu đầy đủ: Internet, Application Gateway WAF, Container Apps internal, PostgreSQL private; bên dưới là egress qua Firewall và private DNS.</figcaption>
</figure>

Hai bẫy khi ghép mẫu. Subnet của gateway v2 phải dành riêng cho gateway và cần NSG cho phép các cổng quản lý của Azure; route `0.0.0.0/0` của spoke không nên gắn nguyên xi vào subnet này, vì gateway có yêu cầu riêng về đường ra (kiểm tra tài liệu). Ở phía Container Apps, UDR trên subnet environment chỉ được hỗ trợ với environment loại workload profiles (loại mặc định khi tạo mới bằng CLI hiện nay), không với environment Consumption-only cũ; Firewall cũng phải cho phép các FQDN mà nền tảng cần, nên đọc tài liệu networking của Container Apps trước khi gắn route.

## DNS quyết định gateway có tìm thấy backend không

Environment internal không có DNS công khai. Nó có một **default domain** dạng `<tên>.<vùng>.azurecontainerapps.io` và một IP tĩnh trong subnet. Để tên của app phân giải ra IP đó trong VNet, bạn tạo **private DNS zone** mang đúng tên default domain, thêm bản ghi A wildcard `*` trỏ về IP tĩnh, và link zone vào VNet (như bài 17).

Nếu quên bước này, app vẫn chạy khoẻ nhưng gateway không phân giải được FQDN của backend, probe fail và khách thấy 502. Với tên nội bộ đẹp hơn như `trips.dev.toinoi.example`, bạn làm một private zone riêng và thêm custom domain cho app, kèm chứng chỉ khớp tên. Cũng như bài 17, câu hỏi chẩn đoán đầu tiên là: tên này đang ra IP nào, nhìn từ subnet của gateway.

## Chi phí cố định và khi nào dùng Front Door

Application Gateway v2 tính tiền theo thời gian tồn tại cộng đơn vị dung lượng (capacity unit), nên có một khoản cố định kể cả khi không có request. Tra Pricing Calculator thay vì nhớ số. Vì vậy dev thường không cần gateway: dùng environment external, hoặc chạy lab rồi xoá ngay. Staging và prod mới đáng trả khoản đó.

Front Door (bài 28) là lựa chọn ở rìa toàn cầu: edge gần khách, tính tiền theo mô hình khác, tích hợp WAF. Application Gateway là bộ cân bằng tải theo vùng nằm trong VNet của bạn. Hai thứ không thay thế nhau hoàn toàn; có kiến trúc dùng cả hai, nhưng khi đó đừng đặt hai lớp WAF trùng nhau mà không có lý do. Với Tới Nơi sáu thành phố ở một region, một gateway theo vùng là điểm khởi đầu hợp lý.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Gateway v2 tính tiền cả khi rảnh, nên lab này phải xoá resource sau khi làm, kể cả public IP. Đừng bật `--allow-insecure` hay termination không mã hoá ở staging và prod chỉ vì probe chưa healthy; hãy sửa chứng chỉ và host header trước.</p>
</div>

## Lab: Application Gateway trước Container Apps internal

Mục tiêu: environment `cae-toinoi-dev` ở chế độ internal trong `snet-app`, Application Gateway v2 trong `snet-gateway` với probe `/health/ready`, kiểm bằng curl qua public IP của gateway. Giả sử đã có `rg-toinoi-net-dev-sea` và `vnet-toinoi-dev-sea` (bài 6), và `snet-app` (`/24`, đủ so với mức tối thiểu `/27` của environment workload profiles) đang trống để dành riêng cho environment (xoá VM lab cũ nếu còn). Khối đầu tạo environment internal và private DNS zone cho nó; nó không chứng minh DNS đã đúng.

```bash
RG=rg-toinoi-net-dev-sea
VNET=vnet-toinoi-dev-sea

az network vnet subnet update -g $RG --vnet-name $VNET --name snet-app \
  --delegations Microsoft.App/environments
SUBNET_ID=$(az network vnet subnet show -g $RG --vnet-name $VNET \
  --name snet-app --query id -o tsv)

az containerapp env create -g $RG --name cae-toinoi-dev \
  --location southeastasia \
  --infrastructure-subnet-resource-id $SUBNET_ID --internal-only true

DOMAIN=$(az containerapp env show -g $RG -n cae-toinoi-dev \
  --query properties.defaultDomain -o tsv)
STATIC_IP=$(az containerapp env show -g $RG -n cae-toinoi-dev \
  --query properties.staticIp -o tsv)

az network private-dns zone create -g $RG --name $DOMAIN
az network private-dns link vnet create -g $RG --zone-name $DOMAIN \
  --name link-vnet-toinoi-dev-sea --virtual-network $VNET \
  --registration-enabled false
az network private-dns record-set a add-record -g $RG \
  --zone-name $DOMAIN --record-set-name '*' --ipv4-address $STATIC_IP
```

Tiếp theo tạo app `ca-toinoi-trips-dev` trong environment này. Ở lab dev, ingress `external` trong environment internal chỉ mở cho VNet; `--allow-insecure` để gateway gọi HTTP ở bước termination của lab, không dùng cho staging và prod.

```bash
az containerapp create -g $RG --name ca-toinoi-trips-dev \
  --environment cae-toinoi-dev \
  --image <acr-login-server>/trip-service:<tag> \
  --registry-server <acr-login-server> \
  --registry-identity <identity-resource-id> \
  --target-port 8080 --ingress external --allow-insecure

APP_FQDN=$(az containerapp show -g $RG -n ca-toinoi-trips-dev \
  --query properties.configuration.ingress.fqdn -o tsv)
```

Bây giờ tạo gateway. Lab dùng `Standard_v2`; để có WAF, đổi sang `WAF_v2` và thêm `--waf-policy` trỏ tới một policy tạo bằng `az network application-gateway waf-policy create`. Policy ở bài 18 là loại của Front Door, không gắn được vào Application Gateway; chỉ nguyên tắc Detection trước, Prevention sau là dùng lại được. Lệnh dưới dùng `snet-gateway` đã có từ bài 6 và mô tả cách làm, chưa chứng minh gateway healthy.

```bash
az network public-ip create -g $RG --name pip-toinoi-agw-dev-sea \
  --sku Standard --allocation-method Static

az network application-gateway create -g $RG \
  --name agw-toinoi-dev-sea --location southeastasia \
  --sku Standard_v2 --capacity 1 --priority 100 \
  --vnet-name $VNET --subnet snet-gateway \
  --public-ip-address pip-toinoi-agw-dev-sea \
  --frontend-port 80 --http-settings-port 80 \
  --http-settings-protocol Http --servers $APP_FQDN
```

Cuối cùng gắn probe `/health/ready`, cho HTTP setting lấy host header từ backend, rồi kiểm sức khoẻ và gọi thử bằng curl:

```bash
az network application-gateway probe create -g $RG \
  --gateway-name agw-toinoi-dev-sea --name probe-ready \
  --protocol Http --path /health/ready \
  --host-name-from-http-settings true \
  --interval 15 --timeout 30 --threshold 3

az network application-gateway http-settings update -g $RG \
  --gateway-name agw-toinoi-dev-sea --name appGatewayBackendHttpSettings \
  --probe probe-ready --host-name-from-backend-pool true

az network application-gateway show-backend-health -g $RG \
  --name agw-toinoi-dev-sea --query "backendAddressPools[].backendHttpSettingsCollection[].servers[].health"

PIP=$(az network public-ip show -g $RG -n pip-toinoi-agw-dev-sea \
  --query ipAddress -o tsv)
curl -i http://$PIP/health/ready
```

Kỳ vọng: backend health là `Healthy` và curl nhận `200`. Điều này chỉ chứng minh đường gateway tới app và probe hoạt động; nó chưa chứng minh WAF chặn đúng, TLS đúng, hay app gọi được PostgreSQL private (cần kiểm riêng ở bài 17 và bài PostgreSQL). Xong lab, xoá resource group để dừng tính tiền gateway.

**Bài tập:** `show-backend-health` trả `Unhealthy`, curl trả 502, trong khi từ VM trong cùng VNet `curl` thẳng tới FQDN của app thì thành công. Bạn kiểm gì trước?

<details><summary>Xem lời giải</summary><p>Kiểm DNS từ góc nhìn gateway: private zone đã link vào đúng VNet chưa, wildcard có trỏ về IP tĩnh chưa. Sau đó kiểm host header của HTTP setting (đã lấy từ backend pool chưa) và đường dẫn probe (app có trả 200 ở `/health/ready` không, và có redirect HTTP sang HTTPS làm probe fail không). NSG của `snet-gateway` và `snet-app` là chỗ kiểm cuối.</p></details>

## Học tiếp từ đây

Bạn đã phân biệt outbound và inbound, hiểu từng mảnh của Application Gateway v2, và có một lab đưa gateway tới Container Apps nội bộ qua probe `/health/ready`. Bước tiếp theo là đặt lớp toàn cầu phía trước: edge, custom domain và TLS cho khách thật.

Bài sau là [Bài 28](/articles/azure-front-door-cdn-custom-domain-tls): Front Door, CDN, custom domain và TLS. Nếu cần ôn việc chọn compute, quay lại [Bài 26](/articles/azure-chon-compute-cho-tung-service-toi-noi).

## Tài liệu gốc

- [Azure Application Gateway](https://learn.microsoft.com/en-us/azure/application-gateway/)
- [Application Gateway cho Container Apps nội bộ](https://learn.microsoft.com/en-us/azure/container-apps/waf-app-gateway)
- [Container Apps: mạng và environment](https://learn.microsoft.com/en-us/azure/container-apps/networking)
- [App Service: tích hợp VNet](https://learn.microsoft.com/en-us/azure/app-service/overview-vnet-integration)
- [Azure App Service](https://learn.microsoft.com/en-us/azure/app-service/)
- [Azure Private Endpoint](https://learn.microsoft.com/en-us/azure/private-link/private-endpoint-overview)

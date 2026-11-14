---
slug: azure-ha-dr-zone-region-rto-rpo
title: "HA và DR: zone, region pair, RTO/RPO và failover"
summary: "Bài 51: phân biệt HA và DR cho Tới Nơi, đặt RTO/RPO cho sáu service, dựng PostgreSQL geo-replica ở East Asia cùng Front Door hai origin group rồi mô phỏng tắt origin chính để đo thời gian failover."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, high-availability, disaster-recovery, front-door, postgresql]
language: vi
---

Đây là **bài 51** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 2/9 của phần).

Bài 50 chỉ ra khoảng trống lớn nhất của Tới Nơi: mọi thứ nằm trong Southeast Asia. Bài này lấp khoảng trống đó bằng bảng RTO/RPO, thiết kế zone-redundant, phương án DR sang East Asia và một buổi mô phỏng sự cố có đo thời gian. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã bật zone-redundant HA cho PostgreSQL của Tới Nơi thì khi cả region Southeast Asia gặp sự cố, dịch vụ có tự chuyển sang region khác không?</p>
<details><summary>Xem đáp án</summary><p>Không. Zone-redundant chỉ bảo vệ khi một zone mất, mọi zone vẫn nằm trong cùng một region. Mất cả region cần bản sao ở region khác và quy trình chuyển do bạn tập.</p></details>
</div>

## HA là tự động trong vài phút, DR là quyết định trong vài giờ

**HA** (high availability) giữ dịch vụ chạy khi một máy, một replica hay một zone hỏng. HA tự động, diễn ra trong một region, đo bằng phút. **DR** (disaster recovery) dựng lại dịch vụ khi cả region hỏng, hoặc khi bạn xoá nhầm cả resource group. DR thường có bước thủ công, nằm ở region khác, đo bằng giờ trừ khi bạn trả tiền để rút ngắn.

Hai con số quyết định thiết kế. **RTO** là thời gian tối đa chấp nhận được từ lúc sự cố đến lúc dịch vụ chạy lại. **RPO** là lượng dữ liệu tối đa chấp nhận mất, tính bằng thời gian: RPO 5 phút nghĩa là mất tối đa 5 phút giao dịch cuối. Hai con số này do nghiệp vụ chốt; kỹ sư nói cái giá của từng mức. Đặt xe cần RTO 15 phút vì mỗi phút gián đoạn là khách bỏ đi. Báo cáo thu nhập tài xế chịu được RTO 24 giờ, dựng lại từ backup là đủ.

## Bảng RTO/RPO cho sáu service của Tới Nơi

Bảng dưới là mục tiêu đề xuất, không phải cam kết của Azure, cho hai kịch bản: mất một zone và mất cả Southeast Asia. Pricing & Quote không có dòng riêng vì không giữ dữ liệu chính; DR của service này chỉ là chạy lại IaC.

| Service | Dữ liệu chính | Mất một zone (RTO / RPO) | Mất cả region (RTO / RPO) | Cách đạt |
| --- | --- | --- | --- | --- |
| Booking | PostgreSQL, bảng `trips` | 5 phút / 0 | 15 phút / tối đa 5 phút | Zone-redundant HA, geo-replica, standby ấm |
| Dispatch | Trạng thái ghép tài xế đang diễn ra | 5 phút / 0 | 15 phút / dựng lại từ Trip | Compute đa zone, trạng thái dựng lại được |
| Trip | PostgreSQL, vòng đời chuyến | 5 phút / 0 | 30 phút / tối đa 5 phút | Geo-replica, promote theo runbook |
| Location & Tracking | Điểm GPS, hết giá trị sau vài giây | 5 phút / vài giây GPS | 30 phút / chấp nhận mất | Luồng sự kiện đa zone, không cần bản sao |
| Payment | Giao dịch, cần đối soát | 5 phút / 0 | 60 phút / tối đa 5 phút cộng đối soát | Geo-replica, idempotency key, đối soát với cổng thanh toán |
| Notification | Hàng đợi Service Bus | 15 phút / 0 | 4 giờ / chấp nhận gửi lại | Service Bus Premium đa zone, dựng lại ở region phụ |

RPO 0 chỉ có khi sao chép đồng bộ, và PostgreSQL Flexible Server chỉ làm điều đó giữa các zone trong một region. Sang region khác, replica là bất đồng bộ nên RPO luôn lớn hơn 0 (bài 30). Với Payment, phần chênh lệch phải được đối soát với cổng thanh toán, kèm idempotency key để không thu tiền hai lần. RPO bằng 0 xuyên region là một thiết kế khác hẳn với giá khác hẳn, không phải một công tắc.

## Zone-redundant: mỗi lớp một cách, và có lớp không có

Availability zone là các nhóm datacenter tách nhau trong một region, có điện, làm mát và mạng riêng. Mỗi dịch vụ trải bản sao qua zone theo cách riêng, và nhiều lựa chọn chỉ đặt được lúc tạo.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Zone-redundant trong Southeast Asia: Front Door ở ngoài region, ba zone chứa replica trip-service và cặp primary, standby của PostgreSQL."><defs><marker id="az51-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="840" height="60" rx="10" class="f-pk-soft s-pk"/><text x="34" y="46" class="t-b">Front Door (toàn cầu)</text><text x="34" y="68" class="t-small">Không thuộc zone nào</text><path d="M440 80 L440 100" class="f-none s-phys" stroke-width="2" marker-end="url(#az51-0-a)"/><rect x="20" y="100" width="840" height="200" rx="12" class="f-phys-soft s-phys"/><text x="36" y="126" class="t-b t-phys">Southeast Asia (một region)</text><rect x="40" y="145" width="250" height="135" rx="10" class="f-surface s-phys"/><text x="54" y="172" class="t-b">Zone 1</text><text x="54" y="204" class="t-small">Replica trip-service</text><text x="54" y="232" class="t-small t-phys">PostgreSQL primary</text><rect x="310" y="145" width="250" height="135" rx="10" class="f-surface s-phys"/><text x="324" y="172" class="t-b">Zone 2</text><text x="324" y="204" class="t-small">Replica trip-service</text><text x="324" y="232" class="t-small t-warn">PostgreSQL standby</text><rect x="580" y="145" width="260" height="135" rx="10" class="f-surface s-phys"/><text x="594" y="172" class="t-b">Zone 3</text><text x="594" y="204" class="t-small">Replica trip-service</text><text x="594" y="232" class="t-small">Không có bản sao DB</text></svg>
</div>
<figcaption>Hình 1. Zone-redundant trong Southeast Asia: Front Door ở ngoài region, ba zone chứa replica trip-service và cặp primary, standby của PostgreSQL.</figcaption>
</figure>

| Lớp | Cách zone-redundant | Lưu ý |
| --- | --- | --- |
| Container Apps | Environment bật zone redundancy, replica trải qua các zone | Chỉ chọn lúc tạo environment, cần VNet; min replica từ 2 |
| AKS | Node pool trải qua các zone | Disk LRS khoá pod vào một zone; cân nhắc disk ZRS |
| PostgreSQL Flexible Server | HA zone-redundant, standby đồng bộ ở zone khác | HA không hỗ trợ tier Burstable; chi tiết ở bài 30 |
| Service Bus | Tier Premium, zone redundancy bật sẵn ở region có zone | Kiểm tra tài liệu hiện hành cho tier khác và region của bạn |
| Storage | Redundancy ZRS | Đổi LRS sang ZRS là quá trình chuyển đổi chạy nền, có điều kiện, không tức thì |
| Redis | Azure Managed Redis có zone redundancy ở region có zone | Azure Cache for Redis đang ngừng hỗ trợ (bài về Redis); với SKU, hãy kiểm tra tài liệu |

Với mỗi dòng, hãy kiểm tra xem SKU đó có hỗ trợ zone tại Southeast Asia lúc bạn đọc không. Nếu không, ghi lại lựa chọn: chấp nhận rủi ro, đổi SKU, hoặc bù bằng thiết kế.

## DR sang East Asia: mỗi lớp một cơ chế, và dữ liệu là phần khó

Cặp region của Southeast Asia là East Asia. Compute dễ vì không giữ trạng thái: dựng lại từ Bicep (bài 40, 45) và image theo digest. Dữ liệu mới là phần khó.

| Lớp | Cơ chế sang East Asia | Chuyển bằng | RPO |
| --- | --- | --- | --- |
| PostgreSQL | Read replica khác region, hoặc geo-redundant backup | Promote replica, hoặc restore từ backup (chậm hơn) | Bất đồng bộ, lớn hơn 0 |
| Storage | GRS hoặc RA-GRS | Failover tài khoản do bạn khởi động | Bất đồng bộ, có độ trễ sao chép |
| ACR | Geo-replication (tier Premium) | Pull từ bản sao gần nhất, không cần chuyển | Image đã đẩy sẽ được nhân bản |
| Front Door | Origin group có origin ưu tiên 2 ở East Asia | Tự động theo health probe | Không có dữ liệu |
| Compute | Container Apps dựng sẵn ở region phụ, hoặc từ IaC | Bật lên hoặc chạy pipeline | Không có dữ liệu |

Có hai kiểu vận hành. **Active/passive**: region phụ giữ bản sao dữ liệu, compute tắt hoặc chạy tối thiểu, mọi traffic về region chính cho tới khi chuyển. Rẻ và đơn giản, đổi lại RTO dài hơn. **Active/active**: cả hai region nhận traffic. RTO thấp nhưng phải giải bài toán ghi ở hai nơi: xung đột dữ liệu, thứ tự sự kiện, một chuyến không bị ghép tài xế hai lần. Với `trips` nằm trong một PostgreSQL có một primary, active/active chưa làm được nếu không đổi kiến trúc dữ liệu (bài 57).

Hai cạm bẫy hay bị bỏ qua. Thứ nhất, **Service Bus geo-disaster recovery (tier Premium) chỉ sao chép metadata** (queue, topic, subscription, rule), không sao chép message đang chờ. Tin chưa xử lý ở namespace chính không sang region phụ, nên producer phải gửi lại được từ Outbox và consumer phải xử lý được tin trùng lặp (bài 53). Tính năng **geo-replication** mới hơn sao chép cả message; kiểm tra trạng thái hỗ trợ trong tài liệu trước khi dựa vào. Thứ hai, **DNS và TTL**: sau khi promote, PostgreSQL ở East Asia có hostname khác. Hoặc cập nhật connection string, hoặc dùng virtual endpoint của Flexible Server (endpoint ghi tự trỏ về primary mới khi promote kiểu switchover), hoặc CNAME TTL ngắn trong Private DNS; pool kết nối phải bỏ connection cũ. TTL 1 giờ nghĩa là một phần client còn tìm server cũ cả giờ sau khi chuyển.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. DR active/passive: Front Door dùng health probe để chuyển từ origin Southeast Asia sang East Asia, còn PostgreSQL sao chép bất đồng bộ sang replica."><defs><marker id="az51-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="60" class="t-b">Origin Southeast Asia</text><text x="34" y="88" class="t-small">Ưu tiên 1, nhận traffic</text><path d="M305 72 L280 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az51-1-a)"/><rect x="305" y="30" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="60" class="t-b">Front Door</text><text x="319" y="88" class="t-small">Health probe /health/ready</text><path d="M565 72 L590 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az51-1-a)"/><rect x="590" y="30" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="60" class="t-b">Origin East Asia</text><text x="604" y="88" class="t-small">Ưu tiên 2, standby ấm</text><path d="M150 115 L150 175" class="f-none s-line" stroke-width="2"/><path d="M720 115 L720 175" class="f-none s-line" stroke-width="2"/><rect x="20" y="175" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="205" class="t-b">PostgreSQL primary</text><text x="34" y="233" class="t-small">psql-toinoi-prod-sea</text><text x="318" y="205" class="t-small t-warn">Sao chép bất đồng bộ</text><path d="M280 218 L590 218" class="f-none s-warn" stroke-width="2" marker-end="url(#az51-1-a)"/><rect x="590" y="175" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="205" class="t-b">Geo-replica</text><text x="604" y="233" class="t-small">psql-toinoi-prod-ea-rr</text></svg>
</div>
<figcaption>Hình 2. DR active/passive: Front Door dùng health probe để chuyển từ origin Southeast Asia sang East Asia, còn PostgreSQL sao chép bất đồng bộ sang replica.</figcaption>
</figure>

Hình 2 cho thấy điểm khác biệt: Front Door tự chuyển traffic, PostgreSQL thì không. Khi database vẫn là replica chỉ đọc thì mọi lệnh ghi đều lỗi. Bước promote phải nằm trong runbook và thường chậm nhất.

## Chưa tập thì chưa có RTO

RTO trong bảng mới là lời hứa; RTO thật là con số đo được khi tập. Một buổi DR drill có bốn phần: kịch bản («Southeast Asia không tới được từ phút 0»), runbook từng bước có người thực hiện, bấm giờ tới lúc một chuyến đặt thử thành công ở region phụ, và biên bản so sánh mục tiêu với kết quả đo. Tập mỗi quý trên `staging` và sau mỗi thay đổi kiến trúc lớn. Nếu kết quả đo vượt mục tiêu thì chọn một trong hai hướng: trả tiền để rút ngắn (standby ấm hơn, tự động hoá promote) hoặc thương lượng lại RTO với nghiệp vụ.

Standby là khoản chi thường trực: replica, ACR nhân bản, storage GRS và compute tối thiểu ở region phụ đều tính tiền cả khi không có sự cố. Tra Pricing Calculator và để người giữ ngân sách ký quyết định.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Promote kiểu standalone là một chiều: replica tách khỏi primary và không nối lại được. Chỉ tập trên staging. Khi region chính hồi phục, app còn ghi vào cả hai database sẽ gây split-brain. Failback là một dự án riêng: dựng lại replication theo chiều ngược, không phải chỉ đổi lại priority.</p></div>

## Lab: geo-replica, Front Door hai origin group và mô phỏng tắt origin chính

Lab dùng `staging`, giả định đã có `psql-toinoi-staging-sea` ở tier General Purpose (replica không hỗ trợ Burstable), `ca-toinoi-trips-staging` ở Southeast Asia và một bản `ca-toinoi-trips-staging-ea` ở East Asia (cùng image digest, trong `cae-toinoi-staging-ea`). Bước 1: điền bảng RTO/RPO của bạn. Bước 2 tạo geo-replica. Nếu primary dùng VNet integration, replica cần thêm `--vnet`, `--subnet`, `--private-dns-zone` ở East Asia.

```bash
RG=rg-toinoi-data-staging-sea
SERVER=psql-toinoi-staging-sea

az postgres flexible-server replica create \
  --resource-group "$RG" --name psql-toinoi-staging-ea-rr \
  --source-server "$SERVER" --location eastasia

az postgres flexible-server show \
  --resource-group "$RG" --name psql-toinoi-staging-ea-rr \
  --query "{state:state, role:replicationRole}" --output table
```

Lệnh trên không chứng minh replica đã bắt kịp primary: theo dõi metric replica lag trước khi tin vào RPO. Bước 3 là Front Door. Một route chỉ trỏ tới một origin group, và failover tự động chạy giữa các origin trong cùng group theo priority. Vì vậy `og-trips` chứa cả hai origin (priority 1 ở SEA, 2 ở EA) và nhận traffic thật. Group thứ hai `og-trips-standby` chỉ có origin EA, để kiểm tra standby qua một route riêng.

```bash
FD_RG=rg-toinoi-edge-staging-sea
P=afd-toinoi-staging
SEA_FQDN=$(az containerapp show -g rg-toinoi-trips-staging-sea \
  -n ca-toinoi-trips-staging --query properties.configuration.ingress.fqdn -o tsv)
EA_FQDN=$(az containerapp show -g rg-toinoi-trips-staging-ea \
  -n ca-toinoi-trips-staging-ea --query properties.configuration.ingress.fqdn -o tsv)

az afd profile create -g "$FD_RG" --profile-name "$P" --sku Standard_AzureFrontDoor
az afd endpoint create -g "$FD_RG" --profile-name "$P" --endpoint-name ep-toinoi-staging

az afd origin-group create -g "$FD_RG" --profile-name "$P" --origin-group-name og-trips \
  --probe-request-type GET --probe-protocol Https --probe-path /health/ready \
  --probe-interval-in-seconds 30 --sample-size 4 --successful-samples-required 3 \
  --additional-latency-in-milliseconds 50

az afd origin create -g "$FD_RG" --profile-name "$P" --origin-group-name og-trips \
  --origin-name sea --host-name "$SEA_FQDN" --origin-host-header "$SEA_FQDN" \
  --priority 1 --weight 1000 --https-port 443 --enabled-state Enabled
az afd origin create -g "$FD_RG" --profile-name "$P" --origin-group-name og-trips \
  --origin-name ea --host-name "$EA_FQDN" --origin-host-header "$EA_FQDN" \
  --priority 2 --weight 1000 --https-port 443 --enabled-state Enabled
```

Probe `/health/ready` đi qua readiness nên origin bị coi là hỏng khi database không sẵn sàng. Tiếp theo là group thứ hai và hai route; `--origin-path /` ở route standby bỏ tiền tố `/standby-check` trước khi chuyển tới origin.

```bash
az afd origin-group create -g "$FD_RG" --profile-name "$P" --origin-group-name og-trips-standby \
  --probe-request-type GET --probe-protocol Https --probe-path /health/ready \
  --probe-interval-in-seconds 30 --sample-size 4 --successful-samples-required 3
az afd origin create -g "$FD_RG" --profile-name "$P" --origin-group-name og-trips-standby \
  --origin-name ea --host-name "$EA_FQDN" --origin-host-header "$EA_FQDN" \
  --priority 1 --weight 1000 --https-port 443 --enabled-state Enabled

az afd route create -g "$FD_RG" --profile-name "$P" --endpoint-name ep-toinoi-staging \
  --route-name r-trips --origin-group og-trips --supported-protocols Https \
  --forwarding-protocol HttpsOnly --link-to-default-domain Enabled --patterns-to-match "/*"
az afd route create -g "$FD_RG" --profile-name "$P" --endpoint-name ep-toinoi-staging \
  --route-name r-standby --origin-group og-trips-standby --supported-protocols Https \
  --forwarding-protocol HttpsOnly --link-to-default-domain Enabled --patterns-to-match "/standby-check/*" \
  --origin-path /
```

`https://<endpoint>/standby-check/health/ready` luôn đi vào East Asia, nên bạn kiểm tra standby mà không phải chờ sự cố. Hai route chưa chứng minh certificate, tên miền riêng hay WAF đã đúng (bài 28). Bước 4 mô phỏng sự cố: tắt ingress của origin chính để probe thất bại, rồi đo. Giả định mỗi bản deploy trả header `x-toinoi-region` từ biến môi trường (middleware vài dòng, không trình bày ở đây).

```bash
HOST=$(az afd endpoint show -g "$FD_RG" --profile-name "$P" \
  --endpoint-name ep-toinoi-staging --query hostName -o tsv)

az containerapp ingress disable -g rg-toinoi-trips-staging-sea \
  -n ca-toinoi-trips-staging
date -u +"T0 %H:%M:%S"

while true; do
  OUT=$(curl -s -D - -o /dev/null -m 5 "https://$HOST/health/ready" \
    | tr -d '\r' | grep -iE '^(HTTP|x-toinoi-region)' | paste -sd' ')
  echo "$(date -u +%H:%M:%S) $OUT"
  sleep 5
done
```

Vòng lặp in thời gian, mã trạng thái và region phục vụ mỗi 5 giây. RTO của Front Door là khoảng từ T0 tới dòng đầu tiên trả `200` từ East Asia. Với probe 30 giây, kỳ vọng là vài phút gồm phát hiện và lan truyền; hãy lấy số thật từ buổi chạy. Bật lại bằng `az containerapp ingress enable` với `--type external` và `--target-port` cũ. Bước cuối của runbook là promote database.

```bash
az postgres flexible-server replica promote \
  --resource-group "$RG" --name psql-toinoi-staging-ea-rr \
  --promote-mode standalone --promote-option forced
```

`standalone` tách replica thành server độc lập nhận ghi, `forced` không chờ đồng bộ nốt dữ liệu (đúng cho kịch bản primary đã mất). Kiểu `switchover` đảo vai trò primary và replica nhưng đòi replica đã gán virtual endpoint. Còn phải cập nhật secret ở region phụ, tạo revision mới và đặt thử một chuyến. Để ước lượng RPO, ghi `select max(created_at) from trips` trên primary trước khi tắt (giả định có cột `created_at`), rồi so với replica sau khi promote. Xong lab, xoá replica, profile Front Door và resource group của region phụ.

**Bài tập:** Front Door đã chuyển traffic sang origin East Asia trong 3 phút, nhưng khách vẫn không đặt được xe suốt 25 phút. Liệt kê hai nguyên nhân khả dĩ và nói số RTO nào bạn báo cáo cho nghiệp vụ.

<details><summary>Xem lời giải</summary><p>Hai nguyên nhân thường gặp: database chưa được promote nên origin EA chỉ đọc được, và connection string hoặc pool của bản EA vẫn trỏ server cũ hoặc giữ DNS cũ theo TTL. RTO báo cho nghiệp vụ là 25 phút, thời gian tới khi khách đặt được xe, không phải 3 phút của Front Door.</p></details>

## Học tiếp từ đây

Bạn đã có bảng RTO/RPO, biết lớp nào zone-redundant, và có quy trình dựng geo-replica cùng Front Door để đo thời gian chuyển.

[Bài 52](/articles/azure-kien-truc-tham-chieu-microservices-toi-noi) ghép mọi mảnh của loạt thành kiến trúc tham chiếu microservices cho Tới Nơi.

## Tài liệu gốc

- [Azure reliability documentation](https://learn.microsoft.com/en-us/azure/reliability/)
- [Azure availability zones overview](https://learn.microsoft.com/en-us/azure/reliability/availability-zones-overview)
- [Azure Front Door documentation](https://learn.microsoft.com/en-us/azure/frontdoor/)
- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
- [Azure Service Bus documentation](https://learn.microsoft.com/en-us/azure/service-bus-messaging/)
- [Azure Container Apps documentation](https://learn.microsoft.com/en-us/azure/container-apps/)

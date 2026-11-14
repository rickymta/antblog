---
slug: azure-tong-ket-go-live-checklist-chung-chi
title: "Tổng kết: checklist go-live và bản đồ chứng chỉ AZ-900, AZ-104, AZ-204, AZ-305"
summary: "Bài 58: nhìn lại trip-service của Tới Nơi từ máy ảo tới Container Apps canary, gom 40 mục go-live theo năm trụ cột WAF, bốn runbook mẫu, bản đồ chứng chỉ Azure và những việc loạt bài chưa làm."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, well-architected, go-live, runbook, certification]
language: vi
---

Đây là **bài 58** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 9/9 của phần).

Bài cuối không dựng thêm resource mà gom những gì đã dựng thành checklist go-live, bốn runbook cho sự cố hay gặp, bản đồ chứng chỉ, và nói thẳng loạt bài chưa làm gì. Vài lệnh `az` trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn, trước hết ở staging.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Checklist go-live có 40 mục và cả 40 đều đã tick xanh. Điều đó có nghĩa hệ thống chịu được ngày cao điểm không?</p>
<details><summary>Xem đáp án</summary><p>Không. Dấu tick chỉ nói mục đó đã có bằng chứng, ví dụ đã có alert hay đã bật HA. Ngày cao điểm cần thêm số đo tải thật và một lần diễn tập sự cố, thứ mà checklist chỉ nhắc chứ không thay được.</p></details>
</div>

## Từ một máy ảo tới canary: mỗi phần gỡ đi một việc làm tay

Ở bài 7, `trip-service` chạy trên một máy ảo: SSH vào để cài, chép file để deploy, nhìn log bằng mắt, và Tới Nơi tự vá hệ điều hành. Năm mươi bài sau, cùng service đó chạy thành revision trong Container Apps, đọc secret qua managed identity, gọi PostgreSQL qua private endpoint và được pipeline OIDC đẩy lên. Mỗi phần thay một việc làm tay bằng một cơ chế kiểm tra được.

| Phần | Trip-service trông thế nào khi xong phần | Bài tiêu biểu |
| --- | --- | --- |
| 1. Nền tảng | Một VM, một VNet, Blob cho ảnh giấy tờ, PostgreSQL managed, alert đầu tiên | [Bài 7](/articles/azure-virtual-machine-linux-chay-api), [bài 9](/articles/azure-database-postgresql-flexible-server-co-ban), [bài 10](/articles/azure-monitor-metrics-activity-log-alert-dau-tien) |
| 2. Danh tính và bảo mật | Không còn mật khẩu trong cấu hình, database không mở ra Internet | [Bài 13](/articles/azure-managed-identity-workload-identity), [bài 14](/articles/azure-key-vault-secret-key-certificate), [bài 17](/articles/azure-private-endpoint-private-dns-service-endpoint) |
| 3. Compute và ứng dụng | Image có digest trong registry, chạy thành revision có ingress và scale | [Bài 21](/articles/azure-container-registry-build-push-digest), [bài 22](/articles/azure-container-apps-revision-ingress-scale), [bài 28](/articles/azure-front-door-cdn-custom-domain-tls) |
| 4. Dữ liệu và tích hợp | PostgreSQL có HA và PITR, báo giá cache trong Redis, sự kiện chuyến đi qua Service Bus | [Bài 30](/articles/azure-postgresql-ha-read-replica-pitr-entra-auth), [bài 33](/articles/azure-cache-for-redis-cache-geospatial-rate-limit), [bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter) |
| 5. DevOps | Hạ tầng bằng Bicep, deploy bằng OIDC, canary theo revision, SLO có alert | [Bài 40](/articles/azure-bicep-co-ban-module-parameter-what-if), [bài 42](/articles/azure-github-actions-oidc-federated-credential), [bài 44](/articles/azure-cd-environment-approval-blue-green-canary), [bài 48](/articles/azure-alert-action-group-slo-workbook) |
| 6. Kiến trúc và vận hành | Có mục tiêu RTO/RPO, Saga/Outbox, kế hoạch chi phí, Sentinel, load test và chaos | [Bài 51](/articles/azure-ha-dr-zone-region-rto-rpo), [bài 53](/articles/azure-saga-outbox-service-bus-postgresql), [bài 55](/articles/azure-zero-trust-network-segmentation-sentinel), [bài 56](/articles/azure-load-testing-autoscale-chaos-studio) |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Sáu phần của loạt xếp thành một đường: mỗi phần thêm một lớp lên cùng trip-service, và phần cuối là thứ phải kiểm trước go-live."><defs><marker id="az58-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="34" y="50" class="t-b">Phần 1 · Nền tảng</text><text x="34" y="76" class="t-small">VM chạy API, VNet, Blob</text><text x="34" y="98" class="t-small">PostgreSQL, alert đầu tiên</text><path d="M280 67 L305 67" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-0-a)"/><rect x="305" y="20" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="319" y="50" class="t-b">Phần 2 · Danh tính, mạng</text><text x="319" y="76" class="t-small">Managed identity, Key Vault</text><text x="319" y="98" class="t-small">Private endpoint, Policy</text><path d="M565 67 L590 67" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-0-a)"/><rect x="590" y="20" width="260" height="95" rx="10" class="f-pk-soft s-pk"/><text x="604" y="50" class="t-b">Phần 3 · Container Apps</text><text x="604" y="76" class="t-small">Image ở ACR, revision</text><text x="604" y="98" class="t-small">Front Door, App Config</text><path d="M720 115 L720 145 L150 145 L150 178" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-0-a)"/><rect x="20" y="180" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="34" y="210" class="t-b">Phần 4 · Dữ liệu</text><text x="34" y="236" class="t-small">PostgreSQL HA, Service Bus</text><text x="34" y="258" class="t-small">Redis, Cosmos DB, APIM</text><path d="M280 227 L305 227" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-0-a)"/><rect x="305" y="180" width="260" height="95" rx="10" class="f-pk-soft s-pk"/><text x="319" y="210" class="t-b">Phần 5 · Pipeline</text><text x="319" y="236" class="t-small">Bicep, OIDC, canary</text><text x="319" y="258" class="t-small">Insights, SLO alert</text><path d="M565 227 L590 227" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-0-a)"/><rect x="590" y="180" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="604" y="210" class="t-b">Phần 6 · Vận hành</text><text x="604" y="236" class="t-small">HA/DR, Saga, FinOps</text><text x="604" y="258" class="t-small">Checklist go-live</text></svg>
</div>
<figcaption>Hình 1. Sáu phần của loạt xếp thành một đường: mỗi phần thêm một lớp lên cùng trip-service, và phần cuối là thứ phải kiểm trước go-live.</figcaption>
</figure>

Cái giá là hệ thống có nhiều bộ phận chuyển động hơn. Máy ảo đơn lẻ chỉ có vài thứ hỏng; hệ thống cuối có identity, private DNS, revision, hàng đợi, pipeline, alert, mỗi thứ hỏng theo cách riêng. Checklist dưới đây là phần trách nhiệm chia sẻ còn lại của Tới Nơi sau khi Azure đã lo hạ tầng.

## Bốn mươi mục go-live, mỗi mục phải có bằng chứng chứ không phải cảm giác

Checklist theo năm trụ cột của [Well-Architected Framework](/articles/azure-well-architected-framework-5-tru-cot): bảo mật, độ tin cậy, vận hành, chi phí và hiệu năng. Các nhóm identity, network, data, deploy, observe, runbook, on-call, DR drill và security review được xếp vào trụ cột gần nhất. Một mục chỉ được tick khi có thứ để chỉ vào: đầu ra lệnh, link pipeline, báo cáo, biên bản diễn tập. Bằng chứng riêng của từng mục ghi trong ngoặc.

### Bảo mật (mục 1 đến 8)

| # | Mục |
| --- | --- |
| 1 | Workload dùng managed identity, quét repo và biến môi trường không thấy secret |
| 2 | Quyền người dùng cấp theo RBAC, quyền admin qua PIM, role assignment đã rà theo scope |
| 3 | API kiểm tra issuer, audience và chữ ký JWT (test từ chối token sai audience) |
| 4 | PostgreSQL và Key Vault chỉ qua private endpoint (`publicNetworkAccess` tắt, DNS private đúng) |
| 5 | Traffic vào đi qua Front Door, WAF ở chế độ Prevention chứ không chỉ Detection |
| 6 | Egress của subnet ứng dụng được kiểm soát, route và rule firewall đã review |
| 7 | Khuyến nghị mức cao của Defender for Cloud đã xử lý hoặc có người nhận |
| 8 | Threat model và security review đã ký (biên bản có ngày, người duyệt, việc còn mở) |

### Độ tin cậy (mục 9 đến 16)

| # | Mục |
| --- | --- |
| 9 | PostgreSQL bật HA zone-redundant ở prod |
| 10 | Thời gian giữ backup khớp RPO, đã restore thử và đo thời gian |
| 11 | RTO và RPO từng service được viết ra và người phụ trách đồng ý |
| 12 | Failover đã diễn tập ở staging (biên bản kèm thời gian gián đoạn thực tế) |
| 13 | Kế hoạch DR sang East Asia đã review, nêu rõ phần nào chưa tự động |
| 14 | Migration theo mẫu expand rồi contract, rollback code không cần rollback schema |
| 15 | Consumer idempotent, hàng đợi có dead-letter, alert và người theo dõi |
| 16 | Probe liveness và readiness tách nhau, readiness kiểm tra database |

### Vận hành (mục 17 đến 24)

| # | Mục |
| --- | --- |
| 17 | Toàn bộ hạ tầng được dựng bằng Bicep hoặc Terraform, what-if hoặc plan trước khi áp |
| 18 | Pipeline đăng nhập OIDC bằng federated credential, không secret dài hạn |
| 19 | Deploy theo canary, lệnh rollback đã thử ở staging |
| 20 | Trace, metric, log nối bằng correlation qua OpenTelemetry, một request truy được tới database |
| 21 | SLO có alert burn-rate, đã kích hoạt thử |
| 22 | Ít nhất bốn runbook, mỗi cái có người phụ trách và người trực đã đọc |
| 23 | Lịch on-call có người thay thế và kênh leo thang |
| 24 | Có mẫu post-incident review, đã dùng cho một sự cố giả lập |

### Chi phí (mục 25 đến 32)

| # | Mục |
| --- | --- |
| 25 | Budget có alert cho từng subscription |
| 26 | Tag `env`, `workload`, `owner` bắt buộc bằng Policy, báo cáo compliance không còn vi phạm |
| 27 | Dev và staging tự tắt hoặc scale về tối thiểu ngoài giờ |
| 28 | Reservation hay savings plan chỉ mua sau khi có baseline nhiều tuần |
| 29 | Log Analytics có retention và daily cap |
| 30 | Chi phí băng thông liên region đã ước tính bằng Pricing Calculator |
| 31 | Rà resource mồ côi hằng tháng |
| 32 | Chi phí thực tế so với ước tính mỗi tháng trong Cost Management |

### Hiệu năng (mục 33 đến 40)

| # | Mục |
| --- | --- |
| 33 | Đã load test với mô hình đặt xe thật, không chỉ một endpoint |
| 34 | Autoscale có min và max, đã quan sát replica khi tải đổi |
| 35 | Tổng connection PostgreSQL của mọi replica dưới giới hạn server (phép tính kèm số đo) |
| 36 | Index cho truy vấn nóng trên `trips` đã kiểm tra bằng `EXPLAIN` trên dữ liệu đủ lớn |
| 37 | Báo giá lặp lại được cache, có TTL và cách vô hiệu |
| 38 | p95 của đường đặt chuyến có mục tiêu và đang được đo |
| 39 | Quota subscription ở Southeast Asia đủ cho đỉnh dự kiến |
| 40 | Có ít nhất một thí nghiệm Chaos Studio ở staging, kèm việc sửa sau đó |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Tick vì «chắc là có rồi» tệ hơn không có checklist, vì cách này tạo cảm giác an toàn giả. Mục 10, 12, 33 và 40 chỉ có bằng chứng khi bạn thật sự restore, failover, tạo tải và gây lỗi. Làm ở staging trước; failover cưỡng bức hay chaos ở prod giữa giờ cao điểm là tự tạo sự cố.</p>
</div>

## Bốn runbook mẫu cho bốn sự cố hay gặp nhất

Runbook tốt đủ ngắn để đọc lúc hai giờ sáng và trả lời bốn câu: dấu hiệu, kiểm tra gì trước, làm gì, không làm gì. Sửa tên resource và ngưỡng theo hệ thống của bạn.

**1. PostgreSQL failover.** Dấu hiệu: lỗi kết nối tăng vọt, readiness trả 503. Kiểm tra trạng thái HA, Resource Health và Activity Log. Chờ pool mở kết nối mới, vì tên host giữ nguyên sau failover. Không restart đồng loạt mọi replica ứng dụng, không đổi connection string sang IP. Sau đó ghi thời gian gián đoạn và so với RTO ở mục 11.

**2. Dead-letter tăng.** Dấu hiệu: alert dead-letter vượt ngưỡng. Đọc `DeadLetterReason` và `DeliveryCount` của vài message. Dependency tạm hỏng thì gửi lại sau khi sửa; poison message (message không thể xử lý thành công) thì sửa consumer trước. Chỉ gửi lại khi consumer idempotent theo [bài 26 của loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer), nếu không một khoản thanh toán có thể bị xử lý hai lần. Không xoá dead-letter trước khi xuất một bản sao.

**3. Revision lỗi cần rollback.** Dấu hiệu: canary đốt error budget nhanh. Đưa 100% traffic về revision ổn định rồi mới điều tra. Giữ revision lỗi (0% traffic) để đọc log, đừng xoá ngay. Rollback code không hoàn tác schema; nếu migration đã chạy, xem mục 14 và [bài 46](/articles/azure-database-migration-trong-pipeline).

**4. Secret hết hạn.** Dấu hiệu: lỗi xác thực tới một dependency, đúng ngày hết hạn. Xem `expires` trong Key Vault, tạo phiên bản mới, rồi restart hoặc tạo revision mới vì replica đang chạy có thể chưa đọc giá trị mới ([bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment)). Lâu dài, đổi sang managed identity, và nhận sự kiện `SecretNearExpiry` qua [Event Grid](/articles/azure-event-grid-system-custom-topic) để được báo trước.

Khối lệnh sau gom bước kiểm tra đầu tiên của bốn runbook. Khối lệnh này chỉ đọc trạng thái và đổi traffic, không chứng minh runbook đúng; phải diễn tập từng cái ở staging.

```bash
# 1) PostgreSQL: trạng thái server và HA
az postgres flexible-server show \
  --resource-group rg-toinoi-data-prod-sea \
  --name psql-toinoi-prod-sea \
  --query "{state:state, ha:highAvailability.state, mode:highAvailability.mode}" \
  --output table

# 2) Service Bus: số message trong dead-letter của một queue
az servicebus queue show \
  --resource-group rg-toinoi-msg-prod-sea \
  --namespace-name sb-toinoi-prod-sea --name <queue> \
  --query countDetails.deadLetterMessageCount

# 3) Container Apps: xem revision, đưa traffic về bản ổn định
az containerapp revision list \
  --resource-group rg-toinoi-trips-prod-sea \
  --name ca-toinoi-trips-prod --output table
az containerapp ingress traffic set \
  --resource-group rg-toinoi-trips-prod-sea \
  --name ca-toinoi-trips-prod \
  --revision-weight <stable-revision>=100

# 4) Key Vault: ngày hết hạn của các secret (không đọc giá trị)
az keyvault secret list --vault-name kv-toinoi-prod-sea \
  --query "[].{name:name, expires:attributes.expires}" --output table
```

**Bài tập:** lúc 2 giờ sáng alert dead-letter nổ. Đồng nghiệp muốn gửi lại toàn bộ message vào queue chính ngay cho nhanh. Bạn hỏi lại hai điều gì trước khi đồng ý?

<details><summary>Xem lời giải</summary><p>Hỏi message chết vì lý do gì (dependency tạm thời hay poison message, xem `DeadLetterReason` và `DeliveryCount`), và consumer đã idempotent chưa. Gửi lại poison message sẽ quay lại dead-letter, còn gửi lại vào consumer không idempotent có thể nhân đôi tác dụng phụ như trừ tiền hai lần. Xuất một bản sao dead-letter trước khi làm gì tiếp.</p></details>

## Bản đồ chứng chỉ chỉ ra bài nào gần mục thi nào, không thay tài liệu thi

Chứng chỉ Azure đo kiến thức theo vai trò, còn loạt bài đi theo một hệ thống. Hai thứ giao nhau nhiều nhưng không trùng. Mã thi và nội dung đổi theo thời gian, có kỳ thi được thay thế hoặc ngừng, nên trước khi ôn hãy đọc study guide hiện hành trên Microsoft Learn. AZ-305 và AZ-400 dẫn tới chứng chỉ Expert, cần thêm một chứng chỉ Associate làm điều kiện (AZ-104 cho AZ-305; AZ-104 hoặc AZ-204 cho AZ-400).

| Chứng chỉ | Vai trò | Phần của loạt | Chủ đề thi loạt chưa phủ sâu |
| --- | --- | --- | --- |
| AZ-900 | Nền tảng cloud và Azure | Phần 1: bài 1 đến 5 | Azure Arc và hybrid, công cụ governance và compliance |
| AZ-104 | Quản trị Azure | Phần 1 và 2, cộng bài vận hành ở phần 3 và 5 | Azure Files, Backup cho VM, scale set, VPN Gateway, Load Balancer |
| AZ-204 | Lập trình viên Azure | Phần 3 và 4 từ góc nhìn dev | SDK chi tiết, MSAL và Microsoft Graph |
| AZ-400 | DevOps engineer | Phần 5 | Quy trình quản lý work item và chiến lược branching |
| AZ-305 | Kiến trúc sư giải pháp | Phần 6, dựa trên Phần 2 và 4 | Migration và hybrid |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Hai bậc loạt bài đã đi qua, và bốn bậc bạn phải tự làm trên subscription thật trước khi go-live."><defs><marker id="az58-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b t-phys">Loạt bài dừng ở đây</text><text x="590" y="24" class="t-b t-warn">Việc của bạn</text><rect x="20" y="40" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">Soạn theo tài liệu</text><text x="34" y="96" class="t-small">Lệnh và Bicep theo</text><text x="34" y="116" class="t-small">Microsoft Learn 11/2026</text><path d="M280 87 L305 87" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-1-a)"/><rect x="305" y="40" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="319" y="70" class="t-b">Compile Bicep sang ARM</text><text x="319" y="96" class="t-small">Kiểm cú pháp, chưa gọi</text><text x="319" y="116" class="t-small">subscription nào</text><path d="M565 87 L590 87" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-1-a)"/><rect x="590" y="40" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="604" y="70" class="t-b">Chạy trên subscription</text><text x="604" y="96" class="t-small">Lỗi quyền, quota, DNS</text><text x="604" y="116" class="t-small">chỉ lộ ra ở bước này</text><path d="M720 135 L720 165 L150 165 L150 188" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-1-a)"/><rect x="20" y="190" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="34" y="220" class="t-b">Đo tải thật</text><text x="34" y="246" class="t-small">Mô hình đặt xe, giờ</text><text x="34" y="266" class="t-small">cao điểm, nhiều thành phố</text><path d="M280 237 L305 237" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-1-a)"/><rect x="305" y="190" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="319" y="220" class="t-b">Diễn tập failover, DR</text><text x="319" y="246" class="t-small">Đo RTO và RPO thực tế</text><text x="319" y="266" class="t-small">thay vì con số mục tiêu</text><path d="M565 237 L590 237" class="f-none s-phys" stroke-width="2" marker-end="url(#az58-1-a)"/><rect x="590" y="190" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="604" y="220" class="t-b">Go-live có người trực</text><text x="604" y="246" class="t-small">Checklist đủ bằng chứng</text><text x="604" y="266" class="t-small">và runbook đã diễn tập</text></svg>
</div>
<figcaption>Hình 2. Hai bậc loạt bài đã đi qua, và bốn bậc bạn phải tự làm trên subscription thật trước khi go-live.</figcaption>
</figure>

## Những gì loạt bài chưa làm, nói thẳng để bạn khỏi phải đoán

Loạt bài có những giới hạn bạn nên biết trước khi tin vào bất kỳ lệnh nào.

- **Chưa chạy trên subscription thật khi soạn.** Bicep của `trip-service` đã compile sang ARM, nhưng compile không chứng minh identity đọc được secret hay ứng dụng tới được PostgreSQL trong mạng của bạn. Lỗi quyền, quota, tên toàn cục bị trùng và DNS private chỉ hiện khi chạy thật.
- **Chưa có số đo tải thật.** [Bài 56](/articles/azure-load-testing-autoscale-chaos-studio) mô tả cách đo, không đưa con số. Đừng lấy replica hay SKU trong bài làm kích thước production.
- **Chưa diễn tập failover hay DR.** RTO và RPO trong các bài là mục tiêu để thiết kế, chưa phải số đo.
- **Không đi qua Fabric, nhóm AI services và Azure Local.** Đó là chủ đề riêng, Tới Nơi chưa cần ở giai đoạn này. Báo cáo ở bài 38 dùng Data Lake và Data Factory, không thay một nền tảng dữ liệu hoàn chỉnh.
- **Trip-service là một lát cắt.** Như bài tổng hợp của [loạt C#](/articles/dotnet10-tong-hop-service-dat-chuyen) đã nói, nó chưa phải toàn bộ luồng đặt xe, giao đồ ăn và giao bưu kiện. Phần pháp lý về dữ liệu cá nhân ở bài 57 không phải tư vấn pháp lý.
- **Chi tiết dễ đổi.** Tên tier, giới hạn SKU, danh mục region và nội dung kỳ thi thay đổi; bài tránh ghi giá và giới hạn cụ thể.

**Bài tập:** một đội đọc xong cả 58 bài rồi tick 40 mục bằng cách mở lại các bài. Hai mục nào trong checklist họ không thể tick bằng cách đó, và vì sao?

<details><summary>Xem lời giải</summary><p>Ví dụ mục 10 (đã restore thử từ backup) và mục 33 (đã load test với mô hình thật). Cả hai cần một hành động trên hệ thống của chính họ, tạo ra biên bản và số đo riêng. Đọc bài cho biết phải làm gì, không chứng minh rằng hệ thống của họ đã làm.</p></details>

## Học tiếp từ đây

Đây là bài cuối của loạt. Hãy quay lại câu hỏi ở [bài 1](/articles/azure-la-gi-cloud-region-trach-nhiem-chia-se): ai lo phần nào. Mang `trip-service` lên một subscription riêng có budget thấp, đi qua checklist ở staging, restore một bản backup, kích hoạt từng runbook một lần, rồi mới nghĩ tới prod. Có số đo thật rồi, quay lại [bài 50](/articles/azure-well-architected-framework-5-tru-cot) và [bài 54](/articles/azure-toi-uu-chi-phi-reservation-savings-plan-finops) để quyết định bằng dữ liệu.

Muốn mở rộng thì đi theo nhu cầu: chọn một chứng chỉ ở bảng trên làm khung ôn, hoặc học Fabric, dịch vụ AI, Azure Local khi sản phẩm cần. Phía code, [loạt C# và .NET 10](/articles/dotnet10-tong-hop-service-dat-chuyen) vẫn là nơi giải thích service đã đưa lên cloud.

## Tài liệu gốc

- [Azure Well-Architected Framework](https://learn.microsoft.com/en-us/azure/well-architected/)
- [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
- [Microsoft Certifications](https://learn.microsoft.com/en-us/credentials/certifications/)
- [Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/)
- [Azure Database for PostgreSQL](https://learn.microsoft.com/en-us/azure/postgresql/)
- [Azure Service Bus](https://learn.microsoft.com/en-us/azure/service-bus-messaging/)

---
slug: azure-kien-truc-tham-chieu-microservices-toi-noi
title: "Kiến trúc tham chiếu microservices Tới Nơi trên Azure"
summary: "Bài 52: ánh xạ từng thành phần của kiến trúc tổng thể Tới Nơi (BFF, domain service, Kafka, Redis, Saga, Realtime Gateway) sang dịch vụ Azure, dựng mạng hub-spoke, chốt identity, observability và ghi rõ những chỗ cố tình chưa tách."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, architecture, networking, microservices]
language: vi
---

Đây là **bài 52** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 3/9 của phần).

Bài này ghép các mảnh của năm phần trước thành một bản vẽ tham chiếu: thành phần nào nằm ở dịch vụ Azure nào, hub-spoke nối chúng ra sao, ba loại danh tính đi qua đâu, và quyết định nào cố tình để mở. Đây là bản vẽ để tranh luận, không phải template đã triển khai. Cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã đặt mọi database vào private endpoint và không gán public IP cho service thì hệ thống có còn public IP nào không?</p>
<details><summary>Xem đáp án</summary><p>Vẫn còn. Front Door là điểm vào công khai có chủ đích, Firewall cần public IP cho egress, Bastion thường cũng cần (trừ các cấu hình private-only). Điều bạn loại bỏ là public IP ở data plane, nơi dữ liệu và service nghiệp vụ nằm.</p></details>
</div>

## Kiến trúc tổng thể trung lập, Azure là một cách hiện thực nó

Tài liệu kiến trúc tổng thể của Tới Nơi mô tả vai trò, không mô tả sản phẩm: lớp edge, các BFF theo nhóm client, 14 domain service, event backbone, datastore do từng service sở hữu và một Realtime Gateway. Bảng dưới gán mỗi vai trò vào một dịch vụ Azure; cột cuối nói lý do, phần còn dùng được khi dịch vụ đổi tên.

| Thành phần trong kiến trúc tổng thể | Dịch vụ Azure | Lý do và giới hạn |
| --- | --- | --- |
| CDN, WAF, DDoS | Front Door với WAF policy | Điểm vào toàn cầu duy nhất; tier Premium mới nối được origin bằng Private Link |
| Load balancer | Ingress của Container Apps environment | Không cần thêm lớp riêng khi Front Door đã chia tải |
| API Gateway | API Management | JWT, rate limit, versioning, không chứa luật nghiệp vụ. Muốn APIM trong VNet: Developer, Premium hoặc Premium v2 (VNet injection); Standard v2 có VNet integration chiều ra, phần inbound private kiểm tra tài liệu |
| Customer, Driver, Admin, Public BFF | Container Apps, mỗi BFF một app | Scale riêng theo kênh; BFF chỉ gom dữ liệu |
| Domain service (Booking, Dispatch, Trip, Payment...) | Container Apps | Mặc định cho nhóm nhỏ; xem mục «chỗ chưa tách» cho AKS |
| Kafka cho sự kiện nghiệp vụ | Service Bus topic và subscription | Session, dead-letter, duplicate detection hợp Saga |
| Kafka cho luồng GPS và analytics | Event Hubs, endpoint Kafka từ tier Standard | Throughput cao, replay theo partition, nhiều consumer group |
| Schema Registry | Hợp đồng trong repository, hoặc Event Hubs Schema Registry | Chọn một nơi duy nhất; đừng để mỗi service tự giữ một bản |
| Outbox relay và CDC | Relay trong code (bài 53) | CDC chưa dùng; tránh thêm một thành phần phải vận hành |
| Database của từng service | PostgreSQL Flexible Server | Transaction local, PITR, private endpoint |
| Location store, dữ liệu theo vùng | Cosmos DB | Partition key theo thành phố; chọn theo mẫu truy cập |
| Redis | Azure Managed Redis | Cache, lease, geospatial; dữ liệu phải dựng lại được. Azure Cache for Redis đang nghỉ hưu |
| Object storage | Storage Account (Blob) | Giấy tờ tài xế, ảnh; SAS ngắn hạn hoặc identity |
| Realtime Gateway | Azure SignalR Service hoặc Web PubSub | SignalR Service cần client theo giao thức ASP.NET Core SignalR; Web PubSub dùng WebSocket thuần hoặc subprotocol pub/sub |
| Identity Provider OIDC | Entra External ID (khách, tài xế), Entra ID (nhân viên) | Hai tenant khác nhau, hai vòng đời khác nhau |
| Secrets, KMS | Key Vault | Truy cập bằng managed identity |
| Push APNs, FCM | Notification Hubs | Fallback khi app không foreground |
| OpenTelemetry | Application Insights, Log Analytics, Managed Grafana | Một chỗ truy vết và một chỗ vẽ dashboard |
| CI/CD, registry | GitHub Actions, Container Registry | Đã dựng ở Phần 5 |

Service mesh không có dòng riêng vì Container Apps đã đóng gói ingress, revision và scale. APIM đứng trước BFF chứ không thay BFF, vì tài liệu tổng thể cấm gateway giữ business rule.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Đường đi của một request từ Front Door qua APIM và BFF tới domain service trên Container Apps, rồi tới bus sự kiện, datastore và kênh realtime."><defs><marker id="az52-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="170" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="138" class="t-b">Front Door + WAF</text><text x="34" y="164" class="t-small">Edge, TLS, chặn bot</text><path d="M190 150 L215 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-0-a)"/><rect x="215" y="105" width="170" height="90" rx="10" class="f-pk-soft s-pk"/><text x="229" y="138" class="t-b">APIM + BFF</text><text x="229" y="164" class="t-small">JWT, rate limit</text><path d="M385 150 L410 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-0-a)"/><rect x="410" y="105" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="424" y="138" class="t-b">Container Apps</text><text x="424" y="164" class="t-small">Domain service</text><path d="M600 130 L640 55" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-0-a)"/><path d="M600 150 L640 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-0-a)"/><path d="M600 170 L640 245" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-0-a)"/><rect x="640" y="15" width="220" height="80" rx="10" class="f-phys-soft s-phys"/><text x="654" y="46" class="t-b">Bus sự kiện</text><text x="654" y="72" class="t-small">Service Bus, Event Hubs</text><rect x="640" y="110" width="220" height="80" rx="10" class="f-phys-soft s-phys"/><text x="654" y="141" class="t-b">Datastore</text><text x="654" y="167" class="t-small">PostgreSQL, Cosmos, Redis</text><rect x="640" y="205" width="220" height="80" rx="10" class="f-phys-soft s-phys"/><text x="654" y="236" class="t-b">Realtime</text><text x="654" y="262" class="t-small">SignalR hoặc Web PubSub</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một request từ Front Door qua APIM và BFF tới domain service trên Container Apps, rồi tới bus sự kiện, datastore và kênh realtime.</figcaption>
</figure>

## Hai bus cho hai loại sự kiện

Kafka trong bản tổng thể gánh hai việc khác tính chất, và Azure cho phép tách chúng. Sự kiện nghiệp vụ như `BookingCreated` hay `DriverAssigned` ít, quan trọng, cần thứ tự theo chuyến và cần biết khi nào một message hỏng. Đó là việc của Service Bus: session theo `tripId`, dead-letter, duplicate detection (xem [bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter)). Luồng vị trí GPS thì nhiều, ngắn hạn, mất một điểm không sao. Đó là việc của Event Hubs: partition theo `driverId`, consumer group cho tracking và analytics (xem [bài về Event Hubs](/articles/azure-event-hubs-stream-gps-consumer-group)).

Tách có giá: hai dịch vụ, hai mô hình tính tiền, đội phải nhớ message nào đi đường nào. Đổi lại, một đợt GPS dồn dập không làm chậm Saga đặt chuyến. Với Realtime Gateway, dịch vụ quản lý giữ kết nối thay bạn; phần bạn vẫn làm là cấp token kết nối có claim theo người dùng và kiểm quyền vào group ở server, vì tài liệu tổng thể cấm client tự đoán channel ID. App iOS và Android chưa muốn kéo client SignalR thì Web PubSub với WebSocket thuần dễ tích hợp hơn.

## Hub-spoke đặt kiểm soát ở một nơi, workload ở nơi khác

Hub-spoke chia mạng làm hai loại vnet. **Hub** chứa thứ dùng chung cần kiểm soát tập trung: Azure Firewall cho egress, Bastion cho quản trị, DNS Private Resolver và các private DNS zone. **Spoke** chứa workload, mỗi môi trường một vnet. Peering không bắc cầu: spoke prod và staging không thấy nhau trực tiếp; muốn đi qua nhau phải route qua Firewall và có rule cho phép. Đó là ranh giới bạn muốn giữa môi trường thử và thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Hub chứa Firewall, Bastion và DNS; hai spoke prod và staging peering với hub, không peering với nhau; Front Door vào thẳng spoke prod."><defs><marker id="az52-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="125" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="157" class="t-b">Front Door</text><text x="34" y="183" class="t-small">Origin qua Private Link</text><rect x="300" y="15" width="300" height="110" rx="10" class="f-phys-soft s-phys"/><text x="314" y="45" class="t-b">Hub</text><text x="314" y="72" class="t-mono">vnet-toinoi-hub-sea</text><text x="314" y="98" class="t-small">Firewall, Bastion, DNS</text><path d="M400 125 L400 195" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-1-a)"/><path d="M520 125 L690 195" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-1-a)"/><path d="M190 175 L220 235" class="f-none s-phys" stroke-width="2" marker-end="url(#az52-1-a)"/><rect x="220" y="195" width="320" height="120" rx="10" class="f-pk-soft s-pk"/><text x="234" y="225" class="t-b">Spoke prod</text><text x="234" y="252" class="t-mono">vnet-toinoi-prod-sea</text><text x="234" y="278" class="t-small">Container Apps, subnet PE</text><rect x="600" y="195" width="260" height="120" rx="10" class="f-warn-soft s-warn"/><text x="614" y="225" class="t-b">Spoke staging</text><text x="614" y="252" class="t-mono">vnet-toinoi-staging-sea</text><text x="614" y="278" class="t-small">Không peering với prod</text></svg>
</div>
<figcaption>Hình 2. Hub chứa Firewall, Bastion và DNS; hai spoke prod và staging peering với hub, không peering với nhau; Front Door vào thẳng spoke prod.</figcaption>
</figure>

Trong spoke, mọi data store (PostgreSQL, Cosmos DB, Redis, Key Vault, Storage, Service Bus Premium) đặt private endpoint vào một subnet riêng, và zone DNS tương ứng được link vào vnet cần phân giải. Thiếu DNS thì service vẫn phân giải ra địa chỉ công khai và bị chặn. Phần chi tiết ở [bài về Private Endpoint](/articles/azure-private-endpoint-private-dns-service-endpoint); bài [Landing Zone](/articles/azure-landing-zone-subscription-vending) bàn việc đặt hub và spoke vào subscription nào.

Khối lệnh dưới nối spoke prod với hub bằng peering hai chiều, rồi gắn route mặc định qua Firewall vào subnet của Container Apps environment. UDR trên subnet này chỉ được hỗ trợ cho environment kiểu workload profiles. Khối lệnh chưa tạo Firewall, chưa mở rule egress mà Container Apps cần (registry, Entra ID), và không chứng minh DNS phân giải đúng.

```bash
HUB_RG=rg-toinoi-net-hub-sea;  HUB=vnet-toinoi-hub-sea
SPK_RG=rg-toinoi-net-prod-sea; SPK=vnet-toinoi-prod-sea

az network vnet peering create -g $HUB_RG -n hub-to-prod --vnet-name $HUB \
  --remote-vnet $(az network vnet show -g $SPK_RG -n $SPK --query id -o tsv) \
  --allow-vnet-access --allow-forwarded-traffic
az network vnet peering create -g $SPK_RG -n prod-to-hub --vnet-name $SPK \
  --remote-vnet $(az network vnet show -g $HUB_RG -n $HUB --query id -o tsv) \
  --allow-vnet-access --allow-forwarded-traffic

az network route-table create -g $SPK_RG -n rt-toinoi-prod-sea
az network route-table route create -g $SPK_RG --route-table-name rt-toinoi-prod-sea \
  -n default-via-firewall --address-prefix 0.0.0.0/0 \
  --next-hop-type VirtualAppliance --next-hop-ip-address <firewall-private-ip>
az network vnet subnet update -g $SPK_RG --vnet-name $SPK -n snet-app \
  --route-table rt-toinoi-prod-sea
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Firewall và Bastion tính tiền theo giờ tồn tại, kể cả khi không có traffic; tra Pricing Calculator và xoá hub sau lab. Dùng chung hub là dùng chung bán kính ảnh hưởng: rule Firewall sai làm cả prod lẫn staging mất egress cùng lúc. Route 0.0.0.0/0 qua Firewall cũng chặn kéo image hay đọc Key Vault nếu rule chưa mở.</p></div>

## Ba loại danh tính, ba cơ chế, không trộn

Tới Nơi có ba nhóm chủ thể và mỗi nhóm có cơ chế riêng:

- **Khách và tài xế** đăng nhập bằng Entra External ID. Token của họ mang claim nghiệp vụ như vai trò và customer ID, và là thứ APIM và BFF kiểm (xem [bài về Entra External ID](/articles/azure-entra-external-id-xac-thuc-khach-tai-xe)).
- **Service** dùng managed identity để lấy secret từ Key Vault, kéo image, đọc blob và gọi Service Bus. Không có password nào nằm trong biến môi trường (xem [bài về managed identity](/articles/azure-managed-identity-workload-identity)).
- **Con người vận hành** dùng tài khoản Entra ID của tổ chức, và quyền cao đi qua PIM: kích hoạt có thời hạn, có lý do, có phê duyệt (xem [bài về RBAC và PIM](/articles/azure-rbac-role-scope-assignment-pim)).

Ranh giới giữa ba loại là chỗ dễ nhầm nhất. Managed identity của `ca-toinoi-trips-prod` không phải danh tính của khách; service phải truyền token hoặc claim của khách đi tiếp, không «đóng vai» khách. Ngược lại, token của khách không bao giờ được dùng để vào Key Vault. Bài [Zero Trust](/articles/azure-zero-trust-network-segmentation-sentinel) sẽ đi sâu hơn vào phân đoạn này.

## Một workspace, nhiều nguồn, trace đi xuyên message

Mỗi môi trường có một workspace Log Analytics dùng chung, để log prod không lẫn staging về quyền và thời gian lưu. Mỗi service có Application Insights riêng ở chế độ workspace-based trỏ vào workspace đó, nên lọc được theo service mà vẫn truy vấn chéo bằng một KQL. Azure Managed Grafana đọc từ Azure Monitor để dựng dashboard SLO.

Điểm khó nhất là trace đi qua bus. SDK .NET của Service Bus tự gắn context W3C vào application property `Diagnostic-Id` khi gửi trong một Activity đang chạy. Nhưng relay của Outbox chạy sau, ngoài request gốc, nên context phải được lưu vào cột của dòng outbox lúc ghi (giá trị `traceparent`) rồi relay đặt lại khi gửi. Thiếu bước đó, bạn có hai trace rời nhau cho cùng một chuyến. Hãy kiểm một trace thật từ `POST /trips` tới Dispatch rồi mới tin. Bài [OpenTelemetry của loạt C#](/articles/dotnet10-opentelemetry-traces-metrics-logs) và [bài Application Insights](/articles/azure-application-insights-opentelemetry-kql) cho phần cấu hình.

## Những chỗ cố tình chưa tách

Bản vẽ tham chiếu trung thực phải ghi cả những thứ chưa quyết, kèm tiêu chí đổi quan sát được.

| Quyết định | Mặc định hôm nay | Đổi khi nào | Cái giá của việc đổi |
| --- | --- | --- | --- |
| Một AKS hay nhiều Container Apps | Container Apps cho mọi service | Cần quyền kiểm soát node, operator hay CRD, mạng pod đặc thù, hoặc đội đã có người trực Kubernetes | Thêm upgrade cluster, node pool, bảo mật pod phải tự vận hành |
| Kafka thật hay Event Hubs | Service Bus cho sự kiện nghiệp vụ, Event Hubs cho GPS | Cần tính năng Kafka mà endpoint tương thích không có, hệ sinh thái connector hoặc stream processing của Kafka, hoặc retention và replay vượt khả năng | Tự vận hành broker, replication, DR; tính vào chi phí nhân sự |
| Một server PostgreSQL hay mỗi service một server | Một server, mỗi service một database và một role riêng | Một service gây noisy neighbor, cần restore riêng, hoặc có yêu cầu tuân thủ cho Payment | Nhiều server, nhiều chi phí cố định, nhiều private endpoint |
| SignalR Service hay Web PubSub hay tự dựng | Dịch vụ quản lý | Cần giao thức hoặc hành vi mà dịch vụ không hỗ trợ, hoặc kết nối vượt hạn mức đã đo | Tự giữ kết nối, tự scale, tự vá lỗi |
| Service mesh | Không có | Cần mTLS đồng nhất giữa service mà Container Apps hay Dapr chưa đáp ứng | Thêm một lớp phải vận hành và gỡ lỗi |

Chưa đo được điều kiện ở cột «Đổi khi nào» thì chưa đổi, vì cột cuối là nợ vận hành thật. Bài [chọn compute](/articles/azure-chon-compute-cho-tung-service-toi-noi) đã đặt nền cho dòng đầu tiên, và bài [chọn messaging](/articles/azure-chon-messaging-storage-queue-service-bus-event-hubs) cho dòng thứ hai.

### Vẽ lại với hai region để thấy chỗ còn một bản

Mọi thứ trên đây nằm ở Southeast Asia, cặp region là East Asia ([bài 51](/articles/azure-ha-dr-zone-region-rto-rpo)). Thêm region thứ hai không tự cho mọi thành phần hai bản.

**Bài tập:** vẽ lại Hình 2 với hai region, mỗi region có một hub và một spoke prod. Chỉ ra ba điểm single-point-of-failure còn lại và với mỗi điểm nói tại sao replica hay Front Door không che được.

<details><summary>Xem lời giải</summary><p>Đáp án không duy nhất; ba điểm hợp lý là: (1) PostgreSQL chỉ có một primary ghi ở một region, replica sang East Asia là bất đồng bộ nên failover có thể mất vài giao dịch cuối và cần quyết định promote; (2) namespace Service Bus chỉ ở một region trừ khi bạn cấu hình geo-DR hay geo-replication, và geo-DR không sao chép message đang chờ; (3) đường điều khiển dùng chung như Entra tenant của khách, pipeline triển khai hoặc registry nếu không có bản thứ hai. Front Door chia traffic tới app ở hai region nhưng không đồng bộ dữ liệu giúp bạn.</p></details>

## Học tiếp từ đây

Bạn vừa có bảng ánh xạ sang dịch vụ Azure, mạng hub-spoke với ranh giới prod và staging, ba cơ chế danh tính và danh sách quyết định còn mở. Bài này chưa chứng minh mọi mảnh chạy đúng khi ghép; đó là việc của test và diễn tập. Bài kế tiếp, [bài 53](/articles/azure-saga-outbox-service-bus-postgresql), đi vào luồng đặt chuyến xuyên service: Saga điều phối Booking, Payment và Dispatch trên Service Bus và PostgreSQL, với Outbox và compensation.

## Tài liệu gốc

- [Hub-spoke network topology in Azure](https://learn.microsoft.com/en-us/azure/architecture/networking/architecture/hub-spoke)
- [Azure Container Apps documentation](https://learn.microsoft.com/en-us/azure/container-apps/)
- [Azure Service Bus documentation](https://learn.microsoft.com/en-us/azure/service-bus-messaging/)
- [Azure Event Hubs documentation](https://learn.microsoft.com/en-us/azure/event-hubs/)
- [Azure Front Door documentation](https://learn.microsoft.com/en-us/azure/frontdoor/)
- [Azure SignalR Service documentation](https://learn.microsoft.com/en-us/azure/azure-signalr/)

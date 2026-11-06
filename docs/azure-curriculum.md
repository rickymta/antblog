# Khung chương trình — Azure từ cơ bản tới nâng cao qua Tới Nơi

Loạt bài Azure của Tech Knowledge Hub (antblog.xyz), chủ đề `devops-azure`. Gồm **58 bài** trong **6 phần** (mỗi phần tối đa 10 bài), đi từ người chưa dùng cloud tới người thiết kế và vận hành hệ thống đặt xe Tới Nơi trên Azure. Mỗi phần là một lộ trình học riêng trong `api/seed/learning-paths.json`; bài viết nằm ở `api/seed/content/azure-*.md`; ảnh bìa sinh từ `apps/web/src/lib/article-covers.json`.

## Trạng thái (06/11/2026)

Cả 58 bài đã viết và đã qua review kỹ thuật; frontmatter hợp lệ theo schema seed; 6 lộ trình và 58 ảnh bìa đã sinh. Chưa seed lên MongoDB production và chưa chạy lab trên subscription thật. Những điểm chủ blog nên quyết trước khi đăng:

- Bài 33: Azure Cache for Redis (Basic/Standard/Premium) đã chặn tạo mới với khách hàng mới từ 04/2026 và có lịch nghỉ hưu; bài giữ lab Standard C1 theo đề cương và kèm phương án Azure Managed Redis. Có thể đổi tiêu đề và lab chính sang Azure Managed Redis.
- Bài 46, 48: lab giả định role `toinoi_migrator`/`toinoi_app` và Dockerfile cho project Migrator, hiện chưa có trong `examples/toi-noi-dotnet/31-38-trip-service`.
- Bài 47, 48: mô tả thêm gói `Azure.Monitor.OpenTelemetry.AspNetCore` và `Npgsql.OpenTelemetry`, repo chưa tham chiếu.
- Bài 58: kiểm lại danh sách kỳ thi còn hiệu lực (AZ-204) trước khi đăng.
- Các chi tiết lệnh `az` được đối chiếu với tài liệu, chưa chạy thật: hành vi `az group create` lần hai không có `--tags` (bài 3), tên cờ của một số extension (`alertsmanagement`, `costmanagement`, `load`, `sentinel`).

## Nguyên tắc biên tập

- Một hệ thống mẫu xuyên suốt: Tới Nơi, với `trip-service` của loạt C# và .NET 10 làm workload đưa lên Azure. Region chính Southeast Asia, cặp East Asia (Azure không có region tại Việt Nam).
- Mỗi bài: câu hỏi «Thử trước khi đọc», 5–8 mục giải thích cơ chế và cái giá, 2 hình SVG, ít nhất một bảng, lab bằng Azure CLI/Bicep, bài tập có lời giải, link bài kế tiếp và tài liệu Microsoft Learn.
- Lab được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026), chưa chạy trên một subscription thật trong quá trình biên tập; bài không ghi giá cụ thể và không khẳng định giới hạn SKU dễ thay đổi.
- Quy trình: Sonnet viết theo brief của bài, Opus review và sửa trực tiếp, sau đó seed vào MongoDB bằng `npm run seed` (api) và sinh ảnh bìa bằng `npm run covers:generate` (apps/web).

## Tổng quan các phần

| Phần | Lộ trình (slug) | Mức | Số bài | Nội dung |
|---|---|---|---:|---|
| Phần 1 · Nền tảng Azure | `azure-p1-nen-tang-azure` | Cơ bản | 10 | Mười bài đầu để một người chưa từng dùng cloud đứng được trên Azure: region và trách nhiệm chia sẻ, tenant/subscription/resource group, Portal và CLI, Resource Manager, chi phí, VNet, VM, Storage, PostgreSQL Flexible Server và Azure Monitor — mỗi bài dựng một mảnh hạ tầng đầu tiên cho dịch vụ đặt xe Tới Nơi. |
| Phần 2 · Danh tính và bảo mật | `azure-p2-danh-tinh-bao-mat` | Trung cấp | 9 | Chín bài về lớp danh tính và bảo vệ: Microsoft Entra ID, RBAC và PIM, managed identity, Key Vault, Azure Policy, Defender for Cloud, Private Endpoint và Private DNS, Firewall/WAF/DDoS, và xác thực khách–tài xế bằng Entra External ID — để hạ tầng Tới Nơi không còn mật khẩu trong cấu hình và không còn database mở ra Internet. |
| Phần 3 · Compute và ứng dụng | `azure-p3-compute-ung-dung` | Trung cấp | 10 | Mười bài đưa trip-service và các service Tới Nơi lên đúng nền tảng: App Service, Container Registry, Container Apps, AKS cơ bản và nâng cao, Functions, cách chọn compute cho từng service, mạng cho ứng dụng với Application Gateway, Front Door và custom domain, và App Configuration với feature flag. |
| Phần 4 · Dữ liệu và tích hợp | `azure-p4-du-lieu-tich-hop` | Trung cấp | 10 | Mười bài về nơi dữ liệu Tới Nơi sống và cách các service nói chuyện: PostgreSQL Flexible Server nâng cao, chọn Azure SQL/PostgreSQL/Cosmos DB, Cosmos DB cho vị trí tài xế, Redis, Service Bus, Event Hubs, Event Grid, cách chọn messaging, Data Lake và Data Factory cho báo cáo, và API Management. |
| Phần 5 · DevOps trên Azure | `azure-p5-devops-tren-azure` | Nâng cao | 10 | Mười bài biến hạ tầng Tới Nơi thành mã và đường ống: Bicep, Terraform, đăng nhập OIDC từ GitHub Actions/Azure DevOps, CI build–test–scan–push, CD với approval và canary, Azure Developer CLI và deployment stacks, migration database trong pipeline, Application Insights với OpenTelemetry và KQL, alert và SLO, và landing zone cho nhiều môi trường. |
| Phần 6 · Kiến trúc nâng cao và vận hành | `azure-p6-kien-truc-nang-cao` | Nâng cao | 9 | Chín bài khép lại loạt: Well-Architected Framework, HA/DR với zone và region pair, kiến trúc tham chiếu microservices Tới Nơi trên Azure, Saga và Outbox trên Service Bus, tối ưu chi phí và FinOps, Zero Trust và Sentinel, Load Testing và Chaos Studio, multi-region và data residency, và tổng kết với checklist go-live cùng bản đồ chứng chỉ. |

## Chuỗi bài

### Phần 1 · Nền tảng Azure

| # | Bài | Slug | Mức |
|---:|---|---|---|
| 1 | Azure là gì: cloud, region, availability zone và trách nhiệm chia sẻ | `azure-la-gi-cloud-region-trach-nhiem-chia-se` | Cơ bản |
| 2 | Tenant, subscription, management group và resource group | `azure-tenant-subscription-resource-group` | Cơ bản |
| 3 | Azure Portal, Azure CLI và Cloud Shell: tạo resource group đầu tiên | `azure-portal-cli-cloud-shell-resource-group-dau-tien` | Cơ bản |
| 4 | Azure Resource Manager: resource provider, resource ID, tag, lock và naming | `azure-resource-manager-resource-id-tag-lock` | Cơ bản |
| 5 | Chi phí Azure: pricing, Cost Management, budget và hoá đơn tháng đầu của Tới Nơi | `azure-chi-phi-pricing-cost-management-budget` | Cơ bản |
| 6 | Mạng cơ bản: VNet, subnet, NSG và public IP | `azure-vnet-subnet-nsg-public-ip` | Cơ bản |
| 7 | Virtual Machine Linux: chạy API Tới Nơi, SSH, disk và image | `azure-virtual-machine-linux-chay-api` | Cơ bản |
| 8 | Storage Account và Blob: tier, SAS và ảnh giấy tờ tài xế | `azure-storage-account-blob-tier-sas` | Cơ bản |
| 9 | Azure Database for PostgreSQL Flexible Server: bảng trips lên cloud | `azure-database-postgresql-flexible-server-co-ban` | Cơ bản |
| 10 | Azure Monitor cơ bản: metrics, Activity Log và alert đầu tiên | `azure-monitor-metrics-activity-log-alert-dau-tien` | Cơ bản |

### Phần 2 · Danh tính và bảo mật

| # | Bài | Slug | Mức |
|---:|---|---|---|
| 11 | Microsoft Entra ID: tenant, user, group và app registration | `azure-entra-id-tenant-user-group-app-registration` | Cơ bản |
| 12 | Azure RBAC: role, scope, assignment và PIM | `azure-rbac-role-scope-assignment-pim` | Trung cấp |
| 13 | Managed identity: system-assigned, user-assigned và workload identity | `azure-managed-identity-workload-identity` | Trung cấp |
| 14 | Key Vault: secret, key, certificate và quyền truy cập | `azure-key-vault-secret-key-certificate` | Trung cấp |
| 15 | Azure Policy: deny, audit, initiative và compliance | `azure-policy-deny-audit-initiative` | Trung cấp |
| 16 | Defender for Cloud và Secure Score | `azure-defender-for-cloud-secure-score` | Trung cấp |
| 17 | Private Endpoint, Private DNS và service endpoint | `azure-private-endpoint-private-dns-service-endpoint` | Trung cấp |
| 18 | Azure Firewall, WAF và DDoS Protection: bảo vệ biên của Tới Nơi | `azure-firewall-waf-ddos-bao-ve-bien` | Trung cấp |
| 19 | Entra External ID: xác thực khách và tài xế, JWT cho API | `azure-entra-external-id-xac-thuc-khach-tai-xe` | Trung cấp |

### Phần 3 · Compute và ứng dụng

| # | Bài | Slug | Mức |
|---:|---|---|---|
| 20 | App Service: deploy API .NET, deployment slot và scale | `azure-app-service-deploy-slot-scale` | Trung cấp |
| 21 | Azure Container Registry: build, push, digest và ACR Tasks | `azure-container-registry-build-push-digest` | Trung cấp |
| 22 | Azure Container Apps: environment, revision, ingress và scale theo KEDA | `azure-container-apps-revision-ingress-scale` | Trung cấp |
| 23 | AKS cơ bản: cluster, node pool và deploy service đầu tiên | `azure-aks-co-ban-cluster-node-pool-deploy` | Trung cấp |
| 24 | AKS nâng cao: ingress, workload identity, autoscaler và upgrade | `azure-aks-nang-cao-ingress-workload-identity-upgrade` | Nâng cao |
| 25 | Azure Functions: trigger, binding và Durable Functions | `azure-functions-trigger-binding-durable` | Trung cấp |
| 26 | Chọn compute cho từng service Tới Nơi: VM, App Service, Container Apps, AKS, Functions | `azure-chon-compute-cho-tung-service-toi-noi` | Trung cấp |
| 27 | Mạng cho ứng dụng: VNet integration, Private Endpoint và Application Gateway | `azure-vnet-integration-application-gateway` | Trung cấp |
| 28 | Front Door, CDN, custom domain và TLS | `azure-front-door-cdn-custom-domain-tls` | Trung cấp |
| 29 | App Configuration: feature flag và Key Vault reference | `azure-app-configuration-feature-flag-key-vault-reference` | Trung cấp |

### Phần 4 · Dữ liệu và tích hợp

| # | Bài | Slug | Mức |
|---:|---|---|---|
| 30 | PostgreSQL Flexible Server nâng cao: HA, read replica, PITR và Entra auth | `azure-postgresql-ha-read-replica-pitr-entra-auth` | Nâng cao |
| 31 | Chọn database: Azure SQL, PostgreSQL hay Cosmos DB cho Tới Nơi | `azure-chon-database-sql-postgresql-cosmos` | Trung cấp |
| 32 | Cosmos DB: partition key, RU, consistency và vị trí tài xế | `azure-cosmos-db-partition-key-ru-consistency` | Nâng cao |
| 33 | Azure Cache for Redis: cache báo giá, geospatial và rate limit | `azure-cache-for-redis-cache-geospatial-rate-limit` | Trung cấp |
| 34 | Service Bus: queue, topic, session và dead-letter cho sự kiện chuyến | `azure-service-bus-queue-topic-session-dead-letter` | Trung cấp |
| 35 | Event Hubs: stream vị trí GPS, consumer group và capture | `azure-event-hubs-stream-gps-consumer-group` | Nâng cao |
| 36 | Event Grid: sự kiện hệ thống và custom topic | `azure-event-grid-system-custom-topic` | Trung cấp |
| 37 | Chọn messaging: Storage Queue, Service Bus, Event Hubs hay Event Grid | `azure-chon-messaging-storage-queue-service-bus-event-hubs` | Trung cấp |
| 38 | Data Lake, Data Factory và Synapse: báo cáo thu nhập tài xế | `azure-data-lake-data-factory-bao-cao` | Nâng cao |
| 39 | API Management: gateway, policy, versioning và product | `azure-api-management-gateway-policy-versioning` | Nâng cao |

### Phần 5 · DevOps trên Azure

| # | Bài | Slug | Mức |
|---:|---|---|---|
| 40 | Bicep: resource, module, parameter và what-if | `azure-bicep-co-ban-module-parameter-what-if` | Trung cấp |
| 41 | Terraform trên Azure: azurerm, state trong Storage và workspace | `azure-terraform-azurerm-state-workspace` | Trung cấp |
| 42 | GitHub Actions và Azure DevOps: đăng nhập OIDC bằng federated credential | `azure-github-actions-oidc-federated-credential` | Trung cấp |
| 43 | CI: build, test, scan image và push ACR | `azure-ci-pipeline-build-test-scan-push-acr` | Trung cấp |
| 44 | CD: environment, approval, blue/green và canary bằng revision | `azure-cd-environment-approval-blue-green-canary` | Nâng cao |
| 45 | Azure Developer CLI, deployment stacks và môi trường dev/staging/prod | `azure-developer-cli-azd-deployment-stacks-moi-truong` | Trung cấp |
| 46 | Migration database trong pipeline: thứ tự, role và rollback | `azure-database-migration-trong-pipeline` | Nâng cao |
| 47 | Application Insights, OpenTelemetry và Log Analytics với KQL | `azure-application-insights-opentelemetry-kql` | Nâng cao |
| 48 | Alert, action group, SLO và workbook | `azure-alert-action-group-slo-workbook` | Nâng cao |
| 49 | Landing zone và subscription vending cho nhiều môi trường | `azure-landing-zone-subscription-vending` | Nâng cao |

### Phần 6 · Kiến trúc nâng cao và vận hành

| # | Bài | Slug | Mức |
|---:|---|---|---|
| 50 | Well-Architected Framework: năm trụ cột áp vào Tới Nơi | `azure-well-architected-framework-5-tru-cot` | Nâng cao |
| 51 | HA và DR: zone, region pair, RTO/RPO và failover | `azure-ha-dr-zone-region-rto-rpo` | Nâng cao |
| 52 | Kiến trúc tham chiếu microservices Tới Nơi trên Azure | `azure-kien-truc-tham-chieu-microservices-toi-noi` | Nâng cao |
| 53 | Saga và Outbox trên Service Bus và PostgreSQL | `azure-saga-outbox-service-bus-postgresql` | Nâng cao |
| 54 | Tối ưu chi phí: reservation, savings plan, spot và FinOps | `azure-toi-uu-chi-phi-reservation-savings-plan-finops` | Nâng cao |
| 55 | Bảo mật nâng cao: Zero Trust, phân đoạn mạng và Sentinel | `azure-zero-trust-network-segmentation-sentinel` | Nâng cao |
| 56 | Hiệu năng: Azure Load Testing, autoscale và Chaos Studio | `azure-load-testing-autoscale-chaos-studio` | Nâng cao |
| 57 | Multi-region, data residency và multi-tenant | `azure-multi-region-data-residency-multi-tenant` | Nâng cao |
| 58 | Tổng kết: checklist go-live và bản đồ chứng chỉ AZ-900, AZ-104, AZ-204, AZ-305 | `azure-tong-ket-go-live-checklist-chung-chi` | Nâng cao |

## Đề cương từng bài

### Bài 1 · Azure là gì: cloud, region, availability zone và trách nhiệm chia sẻ

- IaaS/PaaS/SaaS và mô hình trách nhiệm chia sẻ, đặt vào câu hỏi: Tới Nơi tự lo gì khi chạy API trên VM, trên App Service, trên PostgreSQL managed
- Region, region pair, availability zone; Azure không có region ở Việt Nam ⇒ chọn Southeast Asia (Singapore) làm chính, East Asia làm cặp; độ trễ, data residency, dịch vụ không có ở mọi region
- Các nhóm dịch vụ chính sẽ gặp trong loạt bài (compute, network, storage, database, identity, monitor, devops) và vị trí của chúng trong kiến trúc Tới Nơi
- SLA, credit, khái niệm 99,9% nghĩa là bao nhiêu phút/tháng; SLA của một dịch vụ không cộng dồn thành SLA của hệ thống
- Giới thiệu cấu trúc 6 phần của loạt bài và lab xuyên suốt: đưa trip-service của loạt C# lên Azure

### Bài 2 · Tenant, subscription, management group và resource group

- Bốn cấp scope: tenant Entra ID → management group → subscription → resource group → resource; cái gì kế thừa xuống (RBAC, policy, tag không kế thừa)
- Subscription là ranh giới billing và quota; Tới Nơi chia sub theo môi trường dev/staging/prod và theo nền tảng (platform-shared)
- Resource group: nhóm theo vòng đời; quy ước đặt tên rg-toinoi-<workload>-<env>-<region>; ví dụ rg-toinoi-trips-prod-sea
- Resource group không phải ranh giới mạng hay bảo mật; resource trong RG khác region vẫn được
- Lab: vẽ sơ đồ phân cấp cho Tới Nơi với 3 subscription và quyết định RG cho trip-service, PostgreSQL, monitoring

### Bài 3 · Azure Portal, Azure CLI và Cloud Shell: tạo resource group đầu tiên

- Ba cách thao tác: Portal (khám phá), Azure CLI (lặp lại được, dùng trong bài), Cloud Shell (CLI không cần cài); nhắc PowerShell Az tồn tại nhưng loạt bài dùng az
- az login, az account list/set, az configure defaults; đăng nhập bằng tài khoản người dùng khác với service principal (học ở Phần 2 và 5)
- az group create/list/show/delete với tag; --output table/json/tsv và --query JMESPath để lọc
- Mọi lệnh az là một request tới ARM REST API; Portal cũng vậy ⇒ cùng một mô hình quyền và log
- Lab: tạo rg-toinoi-lab-dev-sea với tag env=dev owner=platform cost-center=toinoi, liệt kê, rồi xoá; lưu ý xoá RG xoá toàn bộ resource bên trong

### Bài 4 · Azure Resource Manager: resource provider, resource ID, tag, lock và naming

- ARM là control plane: mọi request được xác thực, kiểm policy, ghi Activity Log rồi chuyển tới resource provider; data plane (đọc blob, query PostgreSQL) đi đường khác
- Resource provider và namespace (Microsoft.Compute, Microsoft.DBforPostgreSQL); đăng ký provider; API version
- Cấu trúc resource ID /subscriptions/…/resourceGroups/…/providers/…; dùng ID làm scope cho RBAC, diagnostic, policy
- Tag: mục đích (chi phí, owner, env), giới hạn, không kế thừa; policy có thể bắt buộc tag (học sau)
- Lock CanNotDelete/ReadOnly: bảo vệ PostgreSQL prod khỏi xoá nhầm; lock không thay cho RBAC
- Quy ước đặt tên Tới Nơi cho từng loại resource (ràng buộc độ dài/ký tự khác nhau: storage account không có gạch nối, ≤ 24 ký tự)

### Bài 5 · Chi phí Azure: pricing, Cost Management, budget và hoá đơn tháng đầu của Tới Nơi

- Mô hình tính tiền: theo giờ/giây compute, GB-tháng storage, egress mạng, request; free tier và tín dụng dùng thử; không ghi giá cụ thể, chỉ hướng dẫn tra Pricing Calculator
- Ước lượng hoá đơn môi trường dev của Tới Nơi: 1 VM nhỏ hoặc Container Apps, PostgreSQL Flexible Burstable, Storage, Log Analytics — liệt kê hạng mục và biến số thay vì con số
- Cost Management: cost analysis theo RG/tag/service, budget với alert ở 50/80/100%, export hàng ngày ra Storage
- Những chi phí dễ quên: public IP giữ chỗ, disk của VM đã tắt, snapshot, Log Analytics ingestion, egress giữa region
- Thói quen: tắt/xoá resource lab, dùng auto-shutdown cho VM dev, tag cost-center bắt buộc

### Bài 6 · Mạng cơ bản: VNet, subnet, NSG và public IP

- VNet là mạng riêng theo region; address space CIDR; subnet chia vai trò: snet-app, snet-db, snet-gateway; không trùng dải với VNet khác hoặc mạng văn phòng
- NSG: rule theo priority, source/destination, service tag (VirtualNetwork, Internet, AzureLoadBalancer); default rule; gắn vào subnet hay NIC
- Public IP: Basic đã nghỉ, dùng Standard; static; cân nhắc không mở SSH ra Internet, dùng Bastion hoặc just-in-time
- Luồng gói tin từ Internet tới VM: public IP → NSG subnet → NSG NIC; so sánh với luồng nội bộ VNet
- Lab: tạo vnet-toinoi-dev-sea 10.10.0.0/16 với 3 subnet, NSG chỉ mở 443 từ Internet vào snet-app và 5432 từ snet-app vào snet-db

### Bài 7 · Virtual Machine Linux: chạy API Tới Nơi, SSH, disk và image

- Các thành phần của một VM: size (họ B/D/E, vCPU, RAM), image (Ubuntu LTS), OS disk, data disk, NIC, public IP, NSG; mỗi thứ là resource riêng
- az vm create với SSH key, cloud-init cài .NET runtime và chạy trip-service; managed disk Standard SSD/Premium; không lưu dữ liệu PostgreSQL trên disk VM ở bài này
- Availability: single VM không có SLA cao; availability set, zone; scale set nhắc để sau
- Vận hành: auto-shutdown, update, boot diagnostics, serial console, snapshot; dừng (deallocate) mới ngừng tính tiền compute
- Lab: tạo vm-toinoi-api-dev chạy trip-service lắng nghe 8080 sau nginx; curl /health/live qua public IP; nêu rõ đây là bước học, Phần 3 sẽ thay bằng Container Apps

### Bài 8 · Storage Account và Blob: tier, SAS và ảnh giấy tờ tài xế

- Storage account gồm Blob, Files, Queue, Table; redundancy LRS/ZRS/GRS; performance Standard/Premium; tên toàn cầu duy nhất
- Blob container, block blob, access tier Hot/Cool/Cold/Archive và lifecycle management; ảnh giấy tờ tài xế: Hot 90 ngày rồi Cool, không public
- Xác thực: account key (tránh), SAS (user delegation SAS có thời hạn), Entra RBAC (Storage Blob Data Contributor) — chuẩn bị cho managed identity ở Phần 2
- Upload trực tiếp từ app tài xế bằng SAS do API cấp; CORS; soft delete và versioning
- Lab: tạo sttoinoidriverdocsdev, container driver-docs private, bật soft delete, cấp SAS 15 phút để upload bằng az storage blob upload

### Bài 9 · Azure Database for PostgreSQL Flexible Server: bảng trips lên cloud

- Managed PostgreSQL: Microsoft lo OS, patch, backup; bạn lo schema, index, query, connection; so sánh với PostgreSQL trên VM ở bài 7
- Compute tier Burstable/General Purpose/Memory Optimized, storage và IOPS, version PostgreSQL, backup retention mặc định và PITR cơ bản
- Kết nối: public access với firewall rule vs private access (VNet integration) — bài này dùng firewall rule cho IP lab, Phần 2 chuyển sang private
- Tạo database toi_noi, chạy migration của trip-service, bắt buộc TLS (sslmode=verify-full), tham số server (max_connections, log_min_duration_statement)
- Lab: az postgres flexible-server create tier Burstable B1ms, psql tạo bảng trips, chạy dotnet ef database update từ máy lab; nhắc chi phí và xoá sau lab

### Bài 10 · Azure Monitor cơ bản: metrics, Activity Log và alert đầu tiên

- Ba nguồn dữ liệu: platform metrics (tự có, 93 ngày), Activity Log (ai làm gì ở control plane), resource logs (phải bật diagnostic setting gửi về Log Analytics)
- Metric explorer: CPU của VM, connections của PostgreSQL, Ingress/Egress của Storage; aggregation, time grain
- Alert rule: signal, condition, action group (email/SMS/webhook), severity; alert processing rule tắt tiếng khi bảo trì
- Log Analytics workspace: tạo một workspace cho môi trường, giá theo GB ingest; KQL học sâu ở Phần 5
- Lab: alert CPU VM > 80% trong 5 phút và alert PostgreSQL storage > 85%, action group gửi email đội platform

### Bài 11 · Microsoft Entra ID: tenant, user, group và app registration

- Entra ID là identity provider của tenant; phân biệt Entra ID với Azure RBAC (bài sau) và với Entra External ID cho khách (bài 9 của phần)
- User, guest (B2B), group (security, M365, dynamic); đặt quyền cho group chứ không cho user; MFA và Conditional Access cơ bản
- App registration vs enterprise application (service principal): ứng dụng trip-service đăng ký để nhận token, client secret vs certificate vs federated credential
- Token: ID token, access token, audience, scope, app role; đọc một JWT do Entra phát hành
- Lab: tạo group grp-toinoi-platform, app registration api-toinoi-trips expose scope Trips.Read, lấy token bằng az rest/az account get-access-token và giải mã

### Bài 12 · Azure RBAC: role, scope, assignment và PIM

- Ba thành phần của một assignment: security principal, role definition (Actions/NotActions/DataActions), scope; kế thừa từ MG xuống resource; deny assignment
- Built-in role hay gặp: Owner, Contributor, Reader, User Access Administrator, các role data plane (Storage Blob Data Reader, Key Vault Secrets User); vì sao Contributor không đọc được secret
- Custom role khi built-in quá rộng: ví dụ role chỉ được restart Container App
- PIM: quyền just-in-time có thời hạn, phê duyệt, lý do; Owner prod chỉ eligible, không permanent; access review
- Lab: gán Reader cho group dev ở subscription dev, Contributor ở RG trips-dev; kiểm bằng az role assignment list --all và thử một thao tác bị từ chối

### Bài 13 · Managed identity: system-assigned, user-assigned và workload identity

- Vấn đề: trip-service cần đọc Key Vault và Blob mà không có secret trong config; managed identity = service principal do Azure quản lý, token lấy từ IMDS/endpoint nội bộ
- System-assigned (sống chết cùng resource) vs user-assigned (dùng chung cho nhiều revision/replica, tạo trước khi deploy, dễ gán quyền sớm); Tới Nơi chọn user-assigned id-toinoi-trips-<env>
- Trong code .NET: DefaultAzureCredential/ManagedIdentityCredential với client ID; luồng fallback khi chạy local bằng az login
- Workload identity federation: GitHub Actions, AKS pod dùng OIDC token đổi lấy token Entra, không secret; nguyên lý federated credential (issuer, subject, audience)
- Nhắc lại ranh giới của bài 36 loạt C#: identity workload không thay JWT của khách/tài xế
- Lab: tạo user-assigned identity, gán Storage Blob Data Reader ở container driver-docs, gắn vào VM bài 7 và đọc blob bằng az login --identity

### Bài 14 · Key Vault: secret, key, certificate và quyền truy cập

- Ba loại object: secret (connection string), key (mã hoá, ký), certificate (TLS kèm private key); versioning, soft delete, purge protection
- Quyền: RBAC (Key Vault Secrets User/Officer) được khuyến nghị thay cho access policy; data plane vs control plane
- Tham chiếu secret từ Container Apps/App Service bằng URI; rotation và lý do app cần restart/revision mới để nhận giá trị
- Network: public với firewall, private endpoint (bài 7 của phần); logging ai đọc secret nào qua diagnostic setting
- Không dùng Key Vault như config store cho mọi thứ: App Configuration ở Phần 3; không log giá trị secret
- Lab: tạo kv-toinoi-dev-sea, đặt secret Postgres--Trips với connection string sslmode=verify-full, gán quyền cho identity bài trước, đọc bằng az keyvault secret show qua identity

### Bài 15 · Azure Policy: deny, audit, initiative và compliance

- Policy chạy ở control plane: effect Deny, Audit, AuditIfNotExists, DeployIfNotExists, Modify; assignment ở MG/sub/RG; exemption
- Built-in hay dùng: giới hạn region (chỉ Southeast Asia/East Asia), bắt buộc tag cost-center, cấm storage public access, bắt buộc TLS 1.2, cấm public IP cho PostgreSQL
- Initiative (policy set) như Microsoft cloud security benchmark; compliance dashboard; remediation task cho DeployIfNotExists
- Viết custom policy JSON: if/then với field và alias; test bằng Audit trước khi Deny
- Lab: gán initiative tự tạo toinoi-guardrails gồm 4 policy ở subscription dev, thử tạo storage ở West US và quan sát bị từ chối

### Bài 16 · Defender for Cloud và Secure Score

- Hai nửa: CSPM (tư thế bảo mật, free tier Foundational) và CWPP (Defender plan cho Servers, Containers, Databases, Storage, Key Vault — có phí)
- Secure Score và recommendation: đọc một recommendation (PostgreSQL chưa bật private access), fix bằng tay hay bằng policy; đừng chạy theo điểm số mù quáng
- Security alert: ví dụ brute-force SSH trên VM, truy cập Key Vault bất thường; workflow automation gửi về Teams/Logic App
- Vulnerability assessment cho image trong ACR và VM; liên hệ Phần 5 CI scan
- Lab: bật Foundational CSPM, bật Defender for Servers Plan 1 trên subscription dev cho VM lab, đọc 5 recommendation đầu và chọn 2 cái để sửa

### Bài 17 · Private Endpoint, Private DNS và service endpoint

- Vấn đề: PostgreSQL và Key Vault đang mở public với firewall; mục tiêu chỉ reach được từ VNet
- Service endpoint: route tối ưu + firewall theo subnet, resource vẫn có public IP; Private Endpoint: NIC riêng trong subnet, IP private, tắt public access
- Private DNS zone privatelink.postgres.database.azure.com, privatelink.vaultcore.azure.net: vì sao cần, link với VNet, DNS resolve ra IP private; lỗi hay gặp khi DNS vẫn trả public IP
- PostgreSQL Flexible Server có hai lựa chọn: VNet integration (delegated subnet) hoặc Private Endpoint; trade-off
- Chi phí Private Endpoint theo giờ và theo GB; không cần cho mọi resource dev
- Lab: tạo private endpoint cho Key Vault, private DNS zone, tắt public network access, kiểm nslookup từ VM lab và từ máy ngoài

### Bài 18 · Azure Firewall, WAF và DDoS Protection: bảo vệ biên của Tới Nơi

- Phân lớp: DDoS Protection (L3/4, Network Protection vs IP Protection), WAF trên Front Door/Application Gateway (L7, OWASP CRS, bot rule, rate limit), Azure Firewall (egress, FQDN rule, threat intel), NSG/ASG (phân đoạn trong VNet)
- ASG gom NIC theo vai trò thay vì IP; hub-spoke với Firewall ở hub, UDR 0.0.0.0/0 → Firewall
- WAF cho API đặt xe: rule chặn SQLi/XSS, custom rule rate-limit theo IP cho POST /quotes, mode Detection trước Prevention, đọc log false positive
- Không phải mọi thứ đều cần: Tới Nơi dev chỉ WAF trên Front Door; Firewall và DDoS Network Protection ở prod theo đánh giá rủi ro và chi phí
- Lab: tạo WAF policy ở Detection, gắn vào Front Door/App Gateway (mô tả, cấu hình chi tiết ở Phần 3 bài 8–9), gửi request chứa chuỗi SQLi và đọc log

### Bài 19 · Entra External ID: xác thực khách và tài xế, JWT cho API

- Khách và tài xế không phải nhân viên: cần tenant external (Entra External ID, kế thừa Azure AD B2C) tách khỏi tenant workforce; đăng ký bằng email/phone, social login, MFA
- OAuth2/OIDC flow cho app di động: authorization code + PKCE; access token audience là API trips; refresh token; sign-up/sign-in user flow và custom attribute (driver_id)
- Phía ASP.NET Core: AddJwtBearer với authority/audience, claim mapping, policy theo role khách/tài xế; liên kết tới bài 19 loạt C#
- Phân biệt bốn danh tính trong Tới Nơi: nhân viên (Entra ID), khách/tài xế (External ID), workload (managed identity), pipeline (federated credential)
- Giới hạn và chi phí theo MAU; migration người dùng; không tự viết lưu mật khẩu
- Lab: tạo external tenant, user flow sign-up/sign-in, app registration cho mobile và API, lấy token bằng trình duyệt và gọi GET /trips với Bearer

### Bài 20 · App Service: deploy API .NET, deployment slot và scale

- App Service plan (SKU, OS Linux, scale unit) và web app; plan là nơi tính tiền; nhiều app một plan
- Deploy trip-service: az webapp up/zip deploy, hoặc container từ ACR; app settings và connection string từ Key Vault reference; health check path /health/ready
- Deployment slot staging → swap: warm-up, sticky setting, rollback bằng swap ngược; không có slot ở SKU thấp
- Scale up (SKU) vs scale out (instance) với autoscale rule theo CPU/HTTP queue; Always On; ARR affinity tắt cho API stateless
- Khi nào App Service đủ cho Tới Nơi và khi nào chuyển sang Container Apps (bài 3)
- Lab: tạo plan P0v3 Linux, deploy container trip-service, bật health check, tạo slot staging, swap và quan sát zero-downtime bằng vòng curl

### Bài 21 · Azure Container Registry: build, push, digest và ACR Tasks

- Registry SKU Basic/Standard/Premium (geo-replication, private link, content trust); tên toàn cầu; đăng nhập bằng Entra thay admin user
- Tag vs digest: tag :latest trôi, deploy bằng @sha256 digest như bài 36 loại C#; retention policy và xoá image cũ
- ACR Tasks: az acr build trên cloud (không cần Docker local), quick task vs task trigger theo commit/base image update
- Quyền: AcrPull cho identity runtime, AcrPush cho pipeline; token scope map
- Quét lỗ hổng image với Defender for Containers; liên kết Phần 5 CI
- Lab: tạo acrtoinoidev, az acr build image trips:1.0 từ examples/toi-noi-dotnet/31-38-trip-service, lấy digest, gán AcrPull cho identity id-toinoi-trips-dev

### Bài 22 · Azure Container Apps: environment, revision, ingress và scale theo KEDA

- Managed environment (VNet, Log Analytics, workload profile Consumption/Dedicated); container app bên trong; Dapr nhắc là tùy chọn
- Revision: mỗi thay đổi template tạo revision mới; single vs multiple revision mode; traffic splitting cho canary (chi tiết Phần 5 bài 5); label
- Ingress external/internal, target port, TLS tự có, IP restriction; service-to-service bằng tên nội bộ trong environment
- Scale rule KEDA: HTTP concurrency, CPU, Service Bus queue length; min replica 0 (cold start) vs 1; max replica và chi phí
- Secret từ Key Vault qua managed identity; probe startup/readiness/liveness như bài 36 loạt C#
- Lab: triển khai Bicep main.bicep của trip-service lên cae-toinoi-dev, đọc log bằng az containerapp logs, scale bằng HTTP rule 50 concurrent, đổi image digest tạo revision mới

### Bài 23 · AKS cơ bản: cluster, node pool và deploy service đầu tiên

- AKS là Kubernetes managed: control plane do Azure lo (free/standard tier), node pool là VMSS bạn trả tiền; system vs user node pool; Azure CNI overlay vs kubenet
- az aks create với managed identity, Azure CNI overlay, attach ACR; az aks get-credentials và kubectl; namespace toinoi
- Deployment, Service ClusterIP, ConfigMap cho trip-service; resource requests/limits; không dùng LoadBalancer public trực tiếp ở prod
- So với Container Apps: AKS cho khi cần CRD, operator, sidecar, nhiều service, kiểm soát mạng; giá phải trả là vận hành cluster
- Lab: cluster 1 system pool B-series + 1 user pool, deploy trip-service từ ACR bằng manifest, port-forward kiểm /health/ready; nhắc xoá cluster sau lab

### Bài 24 · AKS nâng cao: ingress, workload identity, autoscaler và upgrade

- Ingress: Application Routing add-on (managed NGINX) hoặc Application Gateway for Containers; TLS bằng cert từ Key Vault; internal ingress cho service nội bộ
- Workload identity: OIDC issuer, service account annotate client-id, federated credential; pod đọc Key Vault không secret; Key Vault CSI driver
- Scale: HPA theo CPU/custom metric, KEDA add-on, cluster autoscaler theo node pool, node auto-provisioning; PodDisruptionBudget khi scale down
- Upgrade: Kubernetes version lifecycle, auto-upgrade channel, node image upgrade, surge, maintenance window; vì sao prod cần 2 user pool để blue/green node
- Observability: Container Insights, Managed Prometheus và Grafana; liên hệ Phần 5
- Lab: bật workload identity + OIDC, tạo federated credential cho SA trips, deploy trip-service đọc secret Key Vault, bật cluster autoscaler 1–3 node và quan sát scale khi tăng replica

### Bài 25 · Azure Functions: trigger, binding và Durable Functions

- Hosting: Consumption, Flex Consumption, Premium, trên Container Apps; cold start; storage account bắt buộc; isolated worker model cho .NET
- Trigger và binding: HTTP, Timer, Service Bus, Blob, Event Grid; ví dụ Tới Nơi: timer đóng ngày đối soát thu nhập tài xế; blob trigger resize ảnh giấy tờ
- Durable Functions: orchestrator, activity, entity; pattern function chaining, fan-out/fan-in, human interaction; ví dụ gửi SMS nhắc tài xế nộp giấy tờ với timeout 48 giờ
- Idempotency và retry: at-least-once, poison message; không đặt luật nghiệp vụ cốt lõi của chuyến vào function rời rạc
- Lab: function app .NET isolated với Service Bus trigger nhận trip.completed từ queue (Phần 4 bài 5 tạo), ghi vào Blob; chạy local bằng Core Tools rồi deploy

### Bài 26 · Chọn compute cho từng service Tới Nơi: VM, App Service, Container Apps, AKS, Functions

- Ma trận tiêu chí: mức kiểm soát, vận hành, scale-to-zero, cold start, mạng riêng, chi phí tối thiểu, mô hình trả tiền, hỗ trợ sidecar/CRD, giới hạn thời gian chạy
- Áp cho từng service trong kiến trúc Tới Nơi: Pricing & Quote (Container Apps), Booking/Trip (Container Apps hoặc AKS), Dispatch (AKS vì stateful matching + custom scaler), Location & Tracking (AKS/Event Hubs consumer), Notification (Functions), Admin Portal (App Service hoặc Static Web Apps), batch đối soát (Functions timer/Container Apps jobs)
- Sai lầm phổ biến: chọn AKS từ ngày đầu; chọn Functions cho API latency-sensitive; dùng VM vì quen
- Lộ trình di chuyển: VM (bài 7 P1) → Container Apps → AKS khi điều kiện đủ; không bắt buộc đích cuối là AKS
- Bài tập: quyết định cho hai service mới (Rating, Support) và biện hộ

### Bài 27 · Mạng cho ứng dụng: VNet integration, Private Endpoint và Application Gateway

- Hai chiều: outbound (VNet integration của App Service/Container Apps để tới PostgreSQL private) và inbound (Private Endpoint cho app, internal ingress)
- Application Gateway v2: listener, backend pool, HTTP setting, health probe, URL path routing, WAF_v2; TLS termination và end-to-end TLS
- Mẫu: Internet → Application Gateway (WAF) → Container Apps internal environment → PostgreSQL private; hub-spoke với Firewall egress
- DNS: private DNS zone cho Container Apps environment nội bộ; custom domain nội bộ
- Chi phí cố định của App Gateway và khi dùng Front Door thay thế (bài sau)
- Lab: Container Apps environment internal trong snet-app, App Gateway v2 ở snet-gateway với probe /health/ready, kiểm bằng curl qua public IP của gateway

### Bài 28 · Front Door, CDN, custom domain và TLS

- Front Door là global entry: anycast, POP gần người dùng, TLS termination, caching tĩnh, WAF, routing theo path/health; so với Application Gateway (regional) và Traffic Manager (DNS)
- Endpoint, origin group, origin, route; health probe; session affinity; rule set rewrite header
- Custom domain api.toinoi.example với managed certificate; CNAME/validation TXT; HTTP→HTTPS; TLS 1.2+
- Khoá origin: Private Link tới origin (Premium) hoặc header X-Azure-FDID + IP restriction; tránh bypass Front Door
- CDN cho ảnh tĩnh của app (logo quán, ảnh món) từ Blob; cache rule và purge
- Lab: Front Door Standard với origin là Container App, custom domain, WAF policy bài 8 P2 chuyển sang Prevention; đo TTFB từ hai vị trí

### Bài 29 · App Configuration: feature flag và Key Vault reference

- Tách config khỏi image: key-value với label theo môi trường (dev/staging/prod), snapshot; Key Vault reference để config trỏ sang secret mà không copy giá trị
- Tích hợp ASP.NET Core: AddAzureAppConfiguration với managed identity, refresh theo sentinel key, không gọi mỗi request
- Feature flag: filter theo phần trăm, theo nhóm (tài xế thành phố thử nghiệm), theo time window; ví dụ bật thuật toán giá mới cho 10% chuyến ở Đà Nẵng
- Quy trình: ai được đổi config prod, audit, so với biến môi trường và Key Vault
- Lab: tạo appcs-toinoi-dev, key Pricing:SurgeMultiplier với label dev/prod, flag NewPricingAlgorithm 10%, trip-service đọc và đổi giá trị không restart

### Bài 30 · PostgreSQL Flexible Server nâng cao: HA, read replica, PITR và Entra auth

- Zone-redundant HA: standby đồng bộ ở zone khác, failover tự động, giá gấp đôi compute, RPO 0 và RTO phút; same-zone HA; khi nào dev không cần HA
- Read replica bất đồng bộ cùng/khác region cho báo cáo và DR; replica lag; promote; không dùng replica để scale ghi
- Backup và PITR: retention 7–35 ngày, geo-redundant backup, restore ra server mới (không ghi đè); tập restore thử mỗi quý
- Entra authentication: user database map tới identity; trip-service lấy token làm password với Npgsql; vì sao cần token refresh; liên kết bài 36 loạt C#
- Vận hành: maintenance window, major version upgrade, connection pooling PgBouncer built-in, tham số và metric cần xem (connections, IOPS, replication lag)
- Lab: bật zone-redundant HA trên server staging, tạo read replica, restore PITR về 10 phút trước, bật Entra admin và kết nối psql bằng token

### Bài 31 · Chọn database: Azure SQL, PostgreSQL hay Cosmos DB cho Tới Nơi

- Tiêu chí: mô hình dữ liệu, giao dịch, truy vấn, scale ghi, phân tán địa lý, độ trễ, chi phí, kỹ năng đội, ecosystem (EF Core, extension PostGIS)
- Azure SQL (Database, Managed Instance, Hyperscale) vs PostgreSQL Flexible Server: khi nào mỗi bên; Tới Nơi đã chọn PostgreSQL cho trips theo loạt C#
- Cosmos DB (NoSQL, MongoDB vCore, PostgreSQL/Citus): điểm mạnh ghi phân tán, TTL, change feed; điểm yếu join, giao dịch xuyên partition, chi phí RU khó đoán
- Quyết định theo bounded context: trips/payment → PostgreSQL; driver location/live tracking → Cosmos DB hoặc Redis + Event Hubs; catalog quán → PostgreSQL; analytics → Data Lake
- Polyglot persistence và cái giá: nhiều backup, nhiều kỹ năng, nhiều runbook
- Bài tập: chọn database cho Rating & Review và Promotion, viết lý do 5 dòng

### Bài 32 · Cosmos DB: partition key, RU, consistency và vị trí tài xế

- Container, item, logical/physical partition; partition key cho driver_location: driverId (ghi phân tán, đọc theo tài xế) vs cityId+geohash (đọc theo vùng nhưng hot partition); hierarchical partition key
- RU: ví dụ chi phí point read vs query; provisioned vs autoscale vs serverless; 429 và retry; index policy loại bớt trường
- Năm mức consistency; Session đủ cho vị trí tài xế; Strong có giá về độ trễ và multi-region
- TTL cho điểm vị trí 24 giờ; change feed đẩy sang Event Hubs/Functions; multi-region write và conflict
- SDK .NET: singleton CosmosClient, Direct mode, bulk; không dùng cross-partition query cho đường nóng
- Lab: tạo account serverless, container driver-locations partition /driverId, ghi 10k điểm bằng script, đo RU của point read vs query theo city qua header x-ms-request-charge

### Bài 33 · Azure Cache for Redis: cache báo giá, geospatial và rate limit

- Tier Basic/Standard/Premium và Azure Managed Redis (Enterprise); persistence, clustering, zone redundancy, private endpoint; chọn cho Tới Nơi
- Cache báo giá theo (city, vehicleType, distanceBucket) TTL 60 giây; cache-aside, stampede và lock; invalidation khi đổi bảng giá (feature flag bài 10 P3)
- Geospatial: GEOADD/GEOSEARCH cho tài xế rảnh quanh điểm đón trong 3 km; so với Cosmos DB bài trước; dữ liệu mất khi restart Basic
- Rate limit sliding window bằng INCR/EXPIRE hoặc Lua cho POST /quotes theo khách; so với rate limiter in-process bài 25 loạt C#
- Vận hành: maxmemory policy, eviction, metric server load, connection count; StackExchange.Redis multiplexer singleton, timeouts
- Lab: Standard C1, trip-service cache quote với IDistributedCache, GEOSEARCH 3 km bằng redis-cli, rate limit 10 req/phút, đọc metric Cache Hits/Misses

### Bài 34 · Service Bus: queue, topic, session và dead-letter cho sự kiện chuyến

- Namespace tier Basic/Standard/Premium (Premium có VNet, dự đoán throughput); queue vs topic/subscription với filter (SQL/correlation)
- Ngữ nghĩa: peek-lock vs receive-and-delete, lock duration, renew, max delivery count → dead-letter; at-least-once ⇒ consumer idempotent (bài 26 loạt C#)
- Session để giữ thứ tự theo tripId; duplicate detection theo MessageId; scheduled message cho nhắc 15 phút; transaction giữa hai entity trong namespace
- Outbox của trip-service đẩy sang topic trip-events; subscription cho Notification, Payment, Analytics với filter theo eventType
- Observability: dead-letter queue cần alert; metric active/dead-lettered messages; Service Bus Explorer trong Portal
- Lab: Standard namespace, topic trip-events, 2 subscription có filter, gửi 100 message bằng script, consumer .NET với session và cố ý throw để xem dead-letter sau 10 lần

### Bài 35 · Event Hubs: stream vị trí GPS, consumer group và capture

- Event Hubs là log phân vùng kiểu Kafka: namespace, event hub, partition, retention, throughput unit/processing unit; Kafka endpoint tương thích để dùng client Kafka
- Ước lượng tải Tới Nơi: tài xế online × 1 điểm/3 giây; partition key = driverId để giữ thứ tự theo tài xế; số partition không đổi được dễ dàng ở tier thấp
- Consumer group và checkpoint trong Blob (EventProcessorClient); hai consumer độc lập: Dispatch (realtime) và Analytics (batch); replay từ offset
- Capture tự ghi Avro/Parquet vào Data Lake theo cửa sổ thời gian — đầu vào cho bài 9 của phần
- So với Service Bus: stream vs message, không dead-letter, không per-message lock; schema registry
- Lab: event hub driver-gps 4 partition, producer .NET gửi 10k điểm, hai consumer group, bật Capture vào Blob, xem file sinh ra

### Bài 36 · Event Grid: sự kiện hệ thống và custom topic

- Event Grid là router sự kiện push: system topic (Blob created, resource write succeeded, Key Vault secret near expiry), custom topic, namespace topic (MQTT/pull); CloudEvents schema
- Handler: Functions, Webhook, Service Bus, Event Hubs, Storage Queue; retry và dead-letter vào Blob; filter theo subject/data
- Ví dụ Tới Nơi: Blob created ở driver-docs → Function xác minh giấy tờ; Key Vault secret near expiry → ticket; ARM event khi có ai tạo public IP ở prod
- Webhook validation handshake; bảo mật bằng Entra hoặc SAS key; schema versioning
- Khi KHÔNG dùng Event Grid: cần thứ tự, cần stream khối lượng lớn, cần transaction — chuyển về Service Bus/Event Hubs
- Lab: system topic trên storage driver-docs, subscription gửi tới Function bài 6 P3 lọc .jpg, upload ảnh và theo dõi delivery trong metric

### Bài 37 · Chọn messaging: Storage Queue, Service Bus, Event Hubs hay Event Grid

- Ba loại việc: command/queue (làm việc này), event notification (đã xảy ra), stream/telemetry (dòng dữ liệu); mỗi dịch vụ sinh ra cho một loại
- Bảng so sánh: thứ tự, kích thước message, retention, dead-letter, transaction, throughput, chi phí, protocol (AMQP/Kafka/HTTP/MQTT), VNet
- Áp vào Tới Nơi: trip-events (Service Bus topic), driver-gps (Event Hubs), blob/ARM events (Event Grid), job resize ảnh (Storage Queue đủ), notification fan-out (Service Bus hoặc Event Grid)
- Mẫu kết hợp: Event Grid → Service Bus; Event Hubs → Functions → Service Bus; outbox/inbox với mọi broker; không dùng Kafka thay request/response (nguyên tắc 2 trong kiến trúc tổng thể)
- Bài tập: thiết kế đường đi của sự kiện trip.completed tới 4 consumer và chọn broker, giải thích thứ tự và idempotency

### Bài 38 · Data Lake, Data Factory và Synapse: báo cáo thu nhập tài xế

- Tách OLTP khỏi báo cáo: PostgreSQL không nên chạy query tổng hợp 30 ngày ở giờ cao điểm; Data Lake Storage Gen2 (hierarchical namespace) với vùng raw/curated/serving
- Data Factory: pipeline, copy activity từ PostgreSQL (read replica) sang Parquet theo ngày, trigger lịch, integration runtime, managed identity, retry; Synapse serverless SQL hoặc Fabric để query Parquet (nhắc Fabric là hướng mới, không đi sâu)
- Mô hình dữ liệu báo cáo: fact_trips, dim_driver, dim_city; tính thu nhập tài xế theo ngày; PII và masking
- Capture của Event Hubs (bài 6) đổ vào raw; nối với trips để tính km thực tế
- Chi phí: DIU-giờ, lưu trữ, query theo TB; lifecycle sang Cool/Archive
- Lab: ADLS Gen2 với 3 container, pipeline copy trips hôm qua sang curated/trips/dt=YYYY-MM-DD/*.parquet, query bằng Synapse serverless OPENROWSET tổng thu nhập theo tài xế

### Bài 39 · API Management: gateway, policy, versioning và product

- APIM gồm gateway, developer portal, management plane; tier Consumption/Basic v2/Standard v2/Premium; self-hosted gateway; vị trí trong kiến trúc Tới Nơi: API Gateway cho đối tác (quán, doanh nghiệp) chứ không bắt buộc cho app nội bộ
- Import OpenAPI của trip-service; backend; product và subscription key; rate-limit-by-key, quota, validate-jwt với Entra External ID, cache-lookup, rewrite-uri, set-header
- Versioning (path/header/query) và revision; deprecate có lộ trình; developer portal cho đối tác tự lấy key
- Policy XML theo scope global/product/API/operation; thứ tự inbound/backend/outbound/on-error; dùng named value từ Key Vault
- Observability: Application Insights, diagnostic log, trace header; APIM không thay WAF (Front Door/App Gateway phía trước)
- Lab: APIM Consumption, import OpenAPI, product Partner với quota 1000/ngày, validate-jwt, gọi bằng key đúng/sai, xem trace

### Bài 40 · Bicep: resource, module, parameter và what-if

- Bicep compile sang ARM JSON; declarative, idempotent; deployment scope resourceGroup/subscription/managementGroup; deployment mode Incremental vs Complete
- Cú pháp: param với @allowed/@secure, var, resource với symbolic name, output, existing, loop, condition; decorators; bicepparam file theo môi trường
- Module: tách network/postgres/containerapp thành module; Azure Verified Modules từ registry công khai; dependency ngầm qua tham chiếu
- what-if trước deploy; az deployment group create; đọc lỗi deployment; linter bicep lint; không đưa secret vào output
- Mở rộng main.bicep của bài 36 loạt C# thành bộ module cho môi trường dev Tới Nơi
- Lab: viết modules/network.bicep, modules/postgres.bicep, main.bicep tham số hoá env, chạy what-if rồi deploy dev; đổi SKU và xem what-if báo thay đổi gì

### Bài 41 · Terraform trên Azure: azurerm, state trong Storage và workspace

- Khi nào Terraform thay Bicep: đa cloud, đội đã quen HCL, ecosystem provider; azurerm vs azapi; Bicep và Terraform không trộn trên cùng resource
- Backend azurerm: state trong Storage container với lease lock, versioning blob, RBAC cho pipeline; state chứa secret ⇒ bảo vệ
- Cấu trúc: module network/postgres/containerapp, variables, tfvars theo env; workspace vs thư mục env; remote module từ registry
- plan/apply, import resource có sẵn, moved block, lifecycle prevent_destroy cho PostgreSQL prod; drift detection
- Đăng nhập bằng OIDC (bài sau) với use_oidc; không dùng client secret trong CI
- Lab: dựng lại network + PostgreSQL dev bằng Terraform với backend Storage, plan/apply, cố ý đổi tag ngoài Terraform rồi plan để thấy drift

### Bài 42 · GitHub Actions và Azure DevOps: đăng nhập OIDC bằng federated credential

- So sánh ngắn GitHub Actions và Azure Pipelines (YAML, environment, approval, marketplace); loạt bài dùng GitHub Actions, nêu tương đương trên Azure DevOps
- Federated credential: app registration hoặc user-assigned identity, issuer token.actions.githubusercontent.com, subject repo:org/repo:environment:prod; audience api://AzureADTokenExchange; không secret dài hạn
- azure/login với client-id/tenant-id/subscription-id và permissions id-token: write; Azure DevOps dùng service connection workload identity federation
- Quyền tối thiểu cho identity pipeline: Contributor ở RG, AcrPush, Key Vault không cần; tách identity dev/prod theo environment
- Hiểu workflow toi-noi-dotnet.yml có sẵn trong repo và mở rộng
- Lab: tạo app registration gh-toinoi-deploy với 2 federated credential (environment dev, prod), workflow chạy az account show và az group list không secret

### Bài 43 · CI: build, test, scan image và push ACR

- Giai đoạn CI của trip-service: restore/build/test với PostgreSQL service container (như workflow có sẵn), coverage, kết quả test lên Actions summary
- Build image multi-stage, tag bằng git sha, SBOM (syft/dotnet CycloneDX), scan lỗ hổng (Trivy hoặc Defender for Containers sau push), chặn khi CRITICAL
- Push ACR bằng OIDC + az acr login; lưu digest ra output để CD dùng; immutable tag; cache layer
- Cache NuGet, concurrency group, path filter; chạy nhanh dưới 10 phút; branch protection yêu cầu CI
- Ký image (cosign/notation) và policy chỉ cho deploy image đã ký — giới thiệu, không bắt buộc
- Lab: workflow ci.yml build/test/scan/push acrtoinoidev với output image-digest, PR cố ý fail test để thấy chặn merge

### Bài 44 · CD: environment, approval, blue/green và canary bằng revision

- Pipeline CD nhận digest từ CI: deploy dev tự động → staging với smoke test → prod với approval (GitHub environment required reviewers)
- Container Apps multiple revision mode: revision mới 10% traffic, quan sát error rate/latency qua Application Insights 15 phút, tăng 50% rồi 100%, rollback bằng đổi traffic weight; label canary
- App Service slot swap (blue/green) và AKS rolling/canary với ingress weight — tương đương
- Health gate tự động: query KQL trả lỗi > 1% thì job fail và tự rollback; không deploy thứ 6 chiều — ghi thành rule môi trường
- Migration database chạy trước revision mới (bài 7) và tương thích ngược
- Lab: cd.yml ba job dev/staging/prod, prod có approval, canary 10%→100% bằng az containerapp ingress traffic set, tạo lỗi cố ý ở revision mới để thấy rollback

### Bài 45 · Azure Developer CLI, deployment stacks và môi trường dev/staging/prod

- azd: azure.yaml khai báo service, infra/ Bicep, azd env theo môi trường, azd up/provision/deploy/down; hook; template; phù hợp dev inner loop, không thay CD prod
- Deployment stacks: nhóm resource do một deployment quản, xoá resource bị bỏ khỏi template (actionOnUnmanage), deny setting chống sửa tay; thay cho Complete mode
- Chiến lược môi trường: subscription riêng theo env, RG theo workload, naming/tag thống nhất, dữ liệu prod không copy xuống dev (mask)
- Ephemeral environment cho PR: azd env mới + Container Apps + PostgreSQL Burstable, xoá khi đóng PR; chi phí và giới hạn
- Lab: khởi tạo azd cho trip-service, azd up lên env dev, chuyển infra thành deployment stack với denySettings DenyDelete, thử xoá tay một resource

### Bài 46 · Migration database trong pipeline: thứ tự, role và rollback

- Vấn đề: ai chạy EF Core migration, bằng quyền gì, trước hay sau revision mới; không chạy migration lúc app start ở prod
- Expand/contract: thêm cột nullable → deploy code đọc cả hai → backfill → đổi NOT NULL → xoá cột cũ ở release sau; ví dụ đổi trips.fare thành fare_amount + fare_currency
- Job migration riêng trong CD: Container Apps job hoặc GitHub job qua private runner/VNet, dùng role migrator (DDL) khác role app (DML), script SQL sinh bằng dotnet ef migrations script --idempotent để review
- Rollback: code quay lại được vì schema tương thích; không rollback schema tự động; backup/PITR trước migration lớn; lock và thời gian ALTER trên bảng lớn
- Kiểm thử migration trên bản restore của prod ở staging; đo thời gian
- Lab: pipeline sinh script idempotent, chạy job migration bằng role migrator trước canary, mô phỏng expand/contract cho cột fare qua 2 release

### Bài 47 · Application Insights, OpenTelemetry và Log Analytics với KQL

- Application Insights workspace-based trên Log Analytics; Azure Monitor OpenTelemetry distro cho .NET thay SDK cũ; liên kết bài 35 loạt C# (traces/metrics/logs đã có)
- Bảng dữ liệu: requests, dependencies, exceptions, traces, customMetrics; sampling và chi phí; connection string từ Key Vault/App Configuration
- KQL: where/summarize/bin/join/render; truy vấn p95 latency POST /trips theo 5 phút; tỷ lệ lỗi theo revision; dependency PostgreSQL chậm; correlation operation_Id xuyên service qua Service Bus
- Application Map, Live Metrics, transaction search; log của Container Apps (ContainerAppConsoleLogs) và PostgreSQL (AzureDiagnostics) trong cùng workspace
- Daily cap, retention, commitment tier; không log PII (số điện thoại khách)
- Lab: bật distro trong trip-service, gửi tải bằng script, viết 5 truy vấn KQL và lưu thành function; tìm một request lỗi và lần theo trace tới PostgreSQL

### Bài 48 · Alert, action group, SLO và workbook

- SLI/SLO cho Tới Nơi từ kiến trúc tổng thể §14.1: availability POST /trips 99,9%, p95 < 500 ms; error budget tháng; alert theo burn rate thay vì ngưỡng tĩnh
- Loại alert: metric, log (KQL scheduled), activity log, smart detection; severity 0–4; action group email/SMS/push/webhook/ITSM; alert processing rule
- Thiết kế alert không ồn: cửa sổ đánh giá, số lần vi phạm, auto-resolve, nhóm theo service; runbook link trong description
- Workbook/dashboard: một trang cho on-call: traffic, error, latency, saturation, dead-letter, replica, DB connections; Managed Grafana như lựa chọn
- Availability test (standard test) từ nhiều vùng tới /health/ready qua Front Door
- Lab: 2 alert burn-rate (fast 1h, slow 6h) từ requests, action group tới webhook giả, workbook 6 ô, availability test; kích hoạt bằng tải lỗi để xem alert fire và resolve

### Bài 49 · Landing zone và subscription vending cho nhiều môi trường

- Cloud Adoption Framework và Azure Landing Zone: management group hierarchy (Platform: identity/management/connectivity; Landing zones: corp/online), policy ở MG, hub-spoke connectivity, Log Analytics trung tâm
- Kích thước phù hợp cho Tới Nơi: không cần bản enterprise đầy đủ; MG toinoi với platform và workloads, 3 subscription workload + 1 platform
- Subscription vending: tạo subscription mới bằng IaC (Bicep/Terraform module) kèm RG, RBAC, policy, VNet peering, budget — một PR cho một môi trường mới
- Identity: group theo vai trò (platform-admin, trips-dev), PIM cho prod; break-glass account
- Quản trị liên tục: policy compliance, cost by subscription, tag governance, decommission
- Lab: Bicep ở scope managementGroup tạo MG, assign 3 policy, vending module tạo RG + budget + RBAC cho subscription dev; what-if ở scope tenant/MG

### Bài 50 · Well-Architected Framework: năm trụ cột áp vào Tới Nơi

- Năm trụ cột: Reliability, Security, Cost Optimization, Operational Excellence, Performance Efficiency; trade-off giữa chúng; Well-Architected Review và Azure Advisor
- Đi qua từng trụ cột với trip-service đã dựng ở Phần 1–5: cái gì đã có, cái gì còn thiếu, điểm trade-off (HA PostgreSQL vs chi phí; private endpoint vs độ phức tạp)
- Design review checklist rút gọn 20 câu cho một service Tới Nơi trước go-live
- Azure Advisor recommendation: đọc, phân loại, quyết định bỏ qua có lý do
- Bài tập: đánh giá Notification service (Functions + Service Bus) theo 5 trụ cột, liệt kê 3 rủi ro cao nhất

### Bài 51 · HA và DR: zone, region pair, RTO/RPO và failover

- Phân biệt HA (zone, replica, tự động, phút) và DR (region, backup/replica, có thể thủ công, giờ); RTO/RPO đặt theo nghiệp vụ: đặt xe RTO 15 phút, báo cáo RTO 24 giờ
- Zone-redundant cho từng lớp: Container Apps/AKS zones, PostgreSQL HA, Service Bus Premium, Storage ZRS, Redis zone; cái gì không có zone ở region đã chọn
- DR sang East Asia: PostgreSQL geo-replica/geo-backup, Storage GRS/RA-GRS, ACR geo-replication, Front Door failover theo health probe, IaC để dựng lại
- Active/passive vs active/active; dữ liệu là phần khó; Service Bus geo-DR chỉ metadata; DNS TTL
- DR drill: kịch bản, runbook, đo RTO thực, lịch tập mỗi quý; chi phí standby
- Lab: thiết kế bảng RTO/RPO cho 6 service, dựng PostgreSQL geo-replica + Front Door 2 origin group, mô phỏng tắt origin chính và đo thời gian chuyển

### Bài 52 · Kiến trúc tham chiếu microservices Tới Nơi trên Azure

- Ánh xạ kiến trúc tổng thể (docs/toi-noi-overall-architecture.md: BFF, domain services, Kafka, Redis, Saga, Realtime Gateway) sang dịch vụ Azure: Front Door + WAF → APIM/BFF trên Container Apps → services trên Container Apps/AKS; Service Bus + Event Hubs thay Kafka hoặc Event Hubs Kafka endpoint; PostgreSQL, Cosmos DB, Redis; SignalR Service/Web PubSub cho realtime
- Sơ đồ mạng hub-spoke: hub (Firewall, Bastion, DNS), spoke prod/staging; private endpoint cho mọi data store; không public IP ở data plane
- Identity: Entra External ID cho khách/tài xế, managed identity cho service, PIM cho người
- Observability chung: một Log Analytics, Application Insights per service, trace xuyên Service Bus, Managed Grafana
- Những chỗ cố tình chưa tách: một AKS hay nhiều Container Apps; Kafka thật trên HDInsight/Confluent vs Event Hubs — ghi rõ tiêu chí quyết định
- Bài tập: vẽ lại sơ đồ với 2 region và chỉ ra 3 điểm single-point-of-failure còn lại

### Bài 53 · Saga và Outbox trên Service Bus và PostgreSQL

- Luồng đặt chuyến xuyên service: Booking tạo trip → Payment giữ tiền → Dispatch gán tài xế; không transaction phân tán; Saga choreography vs orchestration
- Outbox trong PostgreSQL của mỗi service (bài 26 loạt C#) + relay đẩy lên Service Bus topic với MessageId = outbox id để duplicate detection; inbox ở consumer
- Orchestrator bằng Durable Functions hoặc một service riêng với state machine; timeout và compensation (hoàn tiền, huỷ chuyến); session theo tripId
- Idempotency key từ client cho POST /trips; trạng thái pending nhìn thấy được với khách; eventual consistency và UX
- Kiểm thử: tắt Payment giữa chừng, replay message, message trùng; quan sát dead-letter và compensation
- Lab: orchestrator Durable Functions điều phối 3 bước qua Service Bus với outbox/inbox, cố ý fail bước Payment và kiểm compensation hoàn tất trong PostgreSQL

### Bài 54 · Tối ưu chi phí: reservation, savings plan, spot và FinOps

- FinOps: inform (tag, show-back theo team/service), optimize, operate; chi phí là metric kỹ thuật, review hàng tuần
- Công cụ cam kết: reserved instance (VM, PostgreSQL compute, Redis), savings plan for compute, Azure Hybrid Benefit; spot VM cho batch/CI runner; khi nào không nên cam kết (dev, workload sắp đổi)
- Right-size theo metric: Advisor cost recommendation, scale-to-zero dev ngoài giờ, Container Apps consumption vs dedicated, PostgreSQL Burstable cho dev, Log Analytics commitment tier, giảm retention
- Những khoản ẩn: egress liên region, private endpoint, Premium tier không cần, snapshot/backup giữ mãi, Defender plan bật toàn bộ
- Lab: báo cáo chi phí 30 ngày theo tag service, tìm 3 khoản lớn nhất, mô phỏng tiết kiệm bằng reservation 1 năm cho PostgreSQL prod trên Pricing Calculator, viết đề xuất 1 trang

### Bài 55 · Bảo mật nâng cao: Zero Trust, phân đoạn mạng và Sentinel

- Ba nguyên tắc Zero Trust áp vào Tới Nơi: xác minh mọi request (Conditional Access cho nhân viên, JWT cho khách, managed identity cho service), quyền tối thiểu (PIM, RBAC data plane, scope hẹp), giả định bị xâm nhập (phân đoạn, log, phát hiện)
- Phân đoạn: hub-spoke, NSG/ASG theo vai trò, private endpoint, Firewall egress allowlist FQDN, không SSH/RDP công khai (Bastion), JIT access
- Bảo vệ dữ liệu: encryption at rest mặc định, customer-managed key với Key Vault khi cần, TLS, Purview/classification cho PII, masking trong báo cáo
- Microsoft Sentinel: SIEM trên Log Analytics, data connector (Entra, Activity, Defender, Firewall), analytics rule, incident, playbook (Logic Apps); chi phí ingest; bắt đầu với 5 rule có ý nghĩa
- Supply chain: image ký, dependency scan, secret scanning, branch protection
- Lab: bật Sentinel trên workspace, connector Entra + Activity, rule phát hiện tạo public IP ở prod và role assignment Owner mới, playbook gửi Teams; kích hoạt và đóng incident

### Bài 56 · Hiệu năng: Azure Load Testing, autoscale và Chaos Studio

- Mục tiêu từ kiến trúc tổng thể: 70 chuyến/giây cao điểm, p95 < 500 ms; test trên staging giống prod; không tạo số liệu từ máy dev
- Azure Load Testing: JMeter/Locust script, engine instance, tham số, so sánh run, server-side metric của Container Apps/PostgreSQL trong cùng kết quả, pass/fail criteria trong CI
- Autoscale: Container Apps HTTP/CPU/queue rule, PostgreSQL không autoscale compute (chuẩn bị IOPS/connection pool), Redis; đọc kết quả để chỉnh min/max replica và cooldown
- Chaos Studio: fault (tắt zone, CPU pressure, network latency, kill pod AKS, Cosmos failover) trong experiment có mục tiêu và điều kiện dừng; chạy ở staging, có người trực
- Kết nối với SLO bài 9 P5: load test + chaos là cách kiểm error budget trước khi khách kiểm giúp
- Lab: load test POST /quotes 500 user ảo 10 phút với tiêu chí p95 < 500 ms, chỉnh scale rule, chạy experiment Chaos tắt 1 replica và network latency 200 ms tới PostgreSQL, ghi nhận SLI

### Bài 57 · Multi-region, data residency và multi-tenant

- Khi Tới Nơi mở sang nước khác: yêu cầu dữ liệu cá nhân ở trong nước (data residency), độ trễ, pháp lý; Azure không có region Việt Nam ⇒ nêu vấn đề và cách tiếp cận (Southeast Asia + cam kết hợp đồng; theo dõi region mới) không khẳng định pháp lý
- Mô hình: stamp per region (mỗi nước một bộ hạ tầng độc lập + control plane chung) vs active/active dữ liệu phân tán; routing theo địa lý ở Front Door; dữ liệu nào global (catalog, config), dữ liệu nào regional (trips, PII)
- Multi-tenant cho khách hàng doanh nghiệp (đội xe thuê): tách theo subscription/RG/database/schema/row; cái giá về cách ly, noisy neighbour, chi phí; Deployment stamps pattern
- Identity nhiều region: Entra là global; External ID tenant theo thị trường
- Chi phí và vận hành nhân đôi; IaC bắt buộc; observability tập trung với workspace theo region
- Bài tập: thiết kế stamp cho thị trường thứ hai và liệt kê dữ liệu được/không được rời khỏi region

### Bài 58 · Tổng kết: checklist go-live và bản đồ chứng chỉ AZ-900, AZ-104, AZ-204, AZ-305

- Nhìn lại trip-service từ VM bài 7 P1 tới Container Apps canary, PostgreSQL HA, private network, pipeline OIDC, SLO alert: bảng những gì đã dựng theo từng phần
- Checklist go-live 40 mục nhóm theo 5 trụ cột WAF: identity, network, data, deploy, observe, cost, runbook, on-call, DR drill, security review
- Runbook mẫu: sự cố PostgreSQL failover, dead-letter tăng, revision lỗi cần rollback, secret hết hạn
- Bản đồ chứng chỉ: AZ-900 (Phần 1), AZ-104 (Phần 1–2 + vận hành), AZ-204 (Phần 3–4 góc nhìn dev), AZ-400 (Phần 5), AZ-305 (Phần 6); loạt bài không thay tài liệu thi, chỉ ra mục nào tương ứng
- Những gì loạt bài chưa làm: chưa chạy trên subscription thật khi soạn, chưa có số đo tải thật, Fabric/AI services, Azure Local; hướng học tiếp

## Bản đồ chứng chỉ (tham khảo)

| Chứng chỉ | Phần tương ứng |
|---|---|
| AZ-900 Azure Fundamentals | Phần 1 |
| AZ-104 Azure Administrator | Phần 1–2, bài vận hành ở Phần 3 và 5 |
| AZ-204 Azure Developer | Phần 3–4 |
| AZ-400 DevOps Engineer | Phần 5 |
| AZ-305 Solutions Architect | Phần 6 |

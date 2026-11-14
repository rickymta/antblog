---
slug: azure-zero-trust-network-segmentation-sentinel
title: "Bảo mật nâng cao: Zero Trust, phân đoạn mạng và Sentinel"
summary: "Bài 55: ánh xạ ba nguyên tắc Zero Trust vào các lớp Tới Nơi đã dựng, thêm phân đoạn mạng, bảo vệ dữ liệu và supply chain, rồi bật Microsoft Sentinel với hai analytics rule KQL và một playbook Logic Apps báo vào Teams."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, security, zero-trust, sentinel, kql]
language: vi
---

Đây là **bài 55** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 6/9 của phần).

Các bài trước đã dựng từng lớp bảo mật riêng lẻ. Bài này gom chúng lại dưới ba nguyên tắc Zero Trust, chỉ ra chỗ còn thiếu, rồi thêm lớp cuối cùng là phát hiện: Microsoft Sentinel. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Mọi service của Tới Nơi đã nằm trong VNet riêng và sau WAF. Vậy một service bị chiếm quyền có còn gọi được Key Vault và PostgreSQL không?</p>
<details><summary>Xem đáp án</summary><p>Có thể có. Mạng riêng chỉ chặn đường vào từ ngoài. Service bị chiếm vẫn ở bên trong và mang managed identity của nó, nên quyền của identity và việc phát hiện hành vi lạ mới là thứ giới hạn thiệt hại.</p></details>
</div>

## Zero Trust là ba câu hỏi đặt lên mọi request, không phải một sản phẩm

Mô hình cũ coi bên trong mạng là tin cậy. Zero Trust bỏ giả định đó. Đây không phải một dịch vụ để mua mà là ba câu hỏi, và mỗi câu đã có lời đáp trong các bài trước.

| Nguyên tắc | Câu hỏi | Tới Nơi trả lời ở đâu |
| --- | --- | --- |
| Xác minh mọi request | Ai đang gọi, từ thiết bị nào, bằng cách nào? | Nhân viên: Conditional Access trên [Entra ID](/articles/azure-entra-id-tenant-user-group-app-registration) (cần license phù hợp, hãy kiểm tra trang tài liệu). Khách và tài xế: JWT từ [Entra External ID](/articles/azure-entra-external-id-xac-thuc-khach-tai-xe), API kiểm tra issuer và audience ([bài 19 loạt C#](/articles/dotnet10-authentication-authorization)). Service: [managed identity](/articles/azure-managed-identity-workload-identity) |
| Quyền tối thiểu | Identity này cần gì, ở scope nào, trong bao lâu? | [RBAC, scope hẹp và PIM](/articles/azure-rbac-role-scope-assignment-pim); RBAC data plane trên [Key Vault](/articles/azure-key-vault-secret-key-certificate); guardrail bằng [Policy](/articles/azure-policy-deny-audit-initiative) |
| Giả định bị xâm nhập | Nếu một thành phần đã thất thủ, thiệt hại dừng ở đâu và ai biết? | Phân đoạn ([NSG, ASG](/articles/azure-vnet-subnet-nsg-public-ip), [private endpoint](/articles/azure-private-endpoint-private-dns-service-endpoint), [Firewall](/articles/azure-firewall-waf-ddos-bao-ve-bien)); log; phát hiện ([Defender](/articles/azure-defender-for-cloud-secure-score) và Sentinel ở bài này) |

Ba câu hỏi này không thay thế được nhau. Xác minh tốt mà quyền rộng thì token bị đánh cắp vẫn làm được nhiều việc. Quyền hẹp mà không có log thì không ai biết token bị dùng sai. Dùng bảng như danh sách kiểm: service nào còn ô trống?

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Ba nguyên tắc Zero Trust bổ sung nhau: xác minh danh tính, giới hạn quyền, và phát hiện khi một thành phần đã bị xâm nhập."><defs><marker id="az55-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Xác minh mọi request</text><text x="34" y="123" class="t-small">Entra, JWT, managed identity</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az55-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Quyền tối thiểu</text><text x="319" y="123" class="t-small">RBAC, PIM, scope hẹp</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az55-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Giả định bị xâm nhập</text><text x="604" y="123" class="t-small">Phân đoạn, log, Sentinel</text></svg>
</div>
<figcaption>Hình 1. Ba nguyên tắc Zero Trust bổ sung nhau: xác minh danh tính, giới hạn quyền, và phát hiện khi một thành phần đã bị xâm nhập.</figcaption>
</figure>

## Phân đoạn nghĩa là mỗi bước đi ngang đều phải có lý do

Phân đoạn là thực hành của nguyên tắc thứ ba. Nếu kẻ tấn công vào được Notification, từ đó nó không được chạm tới Payment hay database. Các biện pháp dưới đây đã gặp ở bài 6, 17 và 18; ở đây ta nhìn chúng như một chuỗi các lớp chặn liên tiếp.

| Biện pháp | Chặn gì | Giới hạn cần biết |
| --- | --- | --- |
| Hub-spoke, route `0.0.0.0/0` về Firewall | Egress ra đích không nằm trong allowlist FQDN | Chỉ hiệu lực với subnet có route; dịch vụ PaaS truy cập qua endpoint riêng không đi qua đường này |
| NSG và ASG theo vai trò | Đi ngang giữa subnet, ví dụ app tới db ngoài cổng 5432 | ASG gắn vào NIC của VM, với PaaS thường vẫn phải viết rule theo subnet |
| Private endpoint và tắt public access | Truy cập PostgreSQL, Key Vault, Storage từ Internet | Cần private DNS đúng; quên DNS thì service không resolve được |
| Bastion thay SSH/RDP công khai | Quét và brute force cổng quản trị | Các SKU trả phí tính tiền theo giờ; vẫn cần quyền tối thiểu cho ai được dùng |
| JIT access (Defender for Servers Plan 2) | Cổng quản trị mở cố định | Cần plan có phí, và chỉ phủ VM |

Quy tắc đơn giản: không cổng 22 hay 3389 nào mở ra Internet ở bất kỳ môi trường nào; rule NSG như vậy là lỗi cấu hình đáng được Policy từ chối. Tới Nơi chạy chủ yếu trên Container Apps, nhưng VM jump box hoặc runner cũ là chỗ quy tắc này hay bị vi phạm.

## Bảo vệ dữ liệu: mặc định đã mã hoá, câu hỏi là ai giữ khoá

Dữ liệu lưu trữ (data at rest) trong PostgreSQL, Storage và hầu hết dịch vụ Azure được mã hoá mặc định bằng khoá do nền tảng quản lý. Lớp này có sẵn, nhưng không phải bằng chứng bạn kiểm soát dữ liệu.

**Customer-managed key** (khoá nằm trong Key Vault của bạn) chỉ cần khi có yêu cầu tuân thủ hoặc hợp đồng đòi bạn kiểm soát vòng đời khoá. Cái giá thực: bạn phải xoay khoá, bảo vệ Key Vault, và nếu khoá bị vô hiệu hoặc xoá thì dữ liệu mã hoá bằng nó không đọc được. Bật soft delete và purge protection trước khi dùng. Các bước còn lại ngắn gọn hơn: TLS cho mọi kết nối (với PostgreSQL, client dùng chế độ `VerifyFull` để xác minh cả chứng chỉ server, như [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment)), và phân loại dữ liệu để biết đâu là PII.

Với PII của Tới Nơi (số điện thoại khách, ảnh giấy tờ tài xế, địa chỉ nhận hàng), Microsoft Purview giúp quét và gắn nhãn phân loại ở các nguồn được hỗ trợ; kiểm tra danh sách nguồn trước khi lên kế hoạch. Phần khó hơn là báo cáo: dữ liệu chảy sang data lake hay dashboard nên được che hoặc bỏ cột nhạy cảm từ lớp truy vấn, vì sao chép dữ liệu thô sang chỗ ít bảo vệ hơn là đường rò rỉ rất phổ biến.

## Supply chain: kẻ tấn công cũng đi qua pipeline của bạn

Nếu pipeline đưa được image vào prod thì chiếm pipeline là chiếm prod. Bốn kiểm soát cho Tới Nơi:

- **Image ký.** Ký digest sau khi build (dùng công cụ mã nguồn mở như Notation hoặc cosign) và kiểm tra chữ ký ở bước deploy. Container Apps không tự ép chữ ký, nên bước kiểm tra nằm trong pipeline; trên AKS có thể ép ở admission (Ratify, Image Integrity). Tag có thể bị ghi đè, digest thì không ([bài về ACR và digest](/articles/azure-container-registry-build-push-digest)).
- **Dependency scan.** Quét thư viện trong CI để chặn CVE đã biết trước khi image tới registry ([bài về CI](/articles/azure-ci-pipeline-build-test-scan-push-acr)); Defender quét tiếp lúc image đã nằm trong ACR.
- **Secret scanning.** Bật quét secret trên repository và push protection để chặn commit chứa khoá. Secret đã lộ phải xoay, không chỉ xoá khỏi lịch sử.
- **Branch protection.** Bắt review và check đạt trước khi merge vào nhánh deploy.

Bốn thứ này không chứng minh code hết lỗi; chúng khiến việc chèn thay đổi độc hại không thể diễn ra mà không bị phát hiện.

## Sentinel đọc log của bạn để tìm hành vi mà cấu hình không chặn được

**Microsoft Sentinel** là SIEM chạy trên Log Analytics. Cơ chế: **data connector** đưa log vào workspace (Entra, Azure Activity, Defender, Firewall); **analytics rule** là truy vấn KQL chạy theo lịch; khi rule trả về dòng kết quả, rule đó tạo **alert** và gộp thành **incident**; **automation rule** gắn **playbook** (Logic Apps) vào incident để thông báo hoặc xử lý.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Luồng Sentinel: connector đưa log vào workspace, analytics rule tạo incident, playbook Logic Apps báo vào Teams."><defs><marker id="az55-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Connector</text><text x="34" y="123" class="t-small">Entra, Activity, Firewall</text><path d="M210 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az55-1-a)"/><rect x="235" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="249" y="95" class="t-b">Analytics rule</text><text x="249" y="123" class="t-small">KQL chạy theo lịch</text><path d="M425 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az55-1-a)"/><rect x="450" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="464" y="95" class="t-b">Incident</text><text x="464" y="123" class="t-small">Có chủ, có trạng thái</text><path d="M640 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az55-1-a)"/><rect x="665" y="65" width="195" height="85" rx="10" class="f-phys-soft s-phys"/><text x="679" y="95" class="t-b">Playbook</text><text x="679" y="123" class="t-small">Logic Apps báo Teams</text></svg>
</div>
<figcaption>Hình 2. Luồng Sentinel: connector đưa log vào workspace, analytics rule tạo incident, playbook Logic Apps báo vào Teams.</figcaption>
</figure>

Chi phí chủ yếu là **ingest**: tính theo GB log vào workspace, cộng phần Sentinel; một số nguồn có chính sách riêng, nên kiểm tra trang giá trước khi bật connector. Đừng bật mọi thứ. Bắt đầu với năm rule có ý nghĩa, mỗi rule có người nhận:

| Rule | Nguồn | Câu hỏi nó trả lời |
| --- | --- | --- |
| Public IP được tạo ở prod | AzureActivity | Có ai mở thêm đường vào không? |
| Role assignment Owner mới | AzureActivity | Có ai tự nâng quyền không? |
| Đăng nhập từ nhiều quốc gia trong thời gian ngắn | SigninLogs | Tài khoản nhân viên có bị lộ hoặc dùng chung không? |
| Thay đổi Conditional Access hoặc PIM | AuditLogs | Có ai nới lỏng lớp xác minh không? |
| Firewall chặn lặp lại tới cùng một đích | AZFWApplicationRule, AZFWNetworkRule | Có workload nào đang cố ra ngoài không? |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Mỗi connector bật lên là tiền ingest mỗi ngày, và mỗi rule không ai đọc là tiếng ồn che mất rule quan trọng. Playbook tự xử lý (xoá IP, vô hiệu tài khoản) còn nguy hiểm hơn: một alert sai hoặc bị kẻ tấn công cố tình kích hoạt sẽ làm gián đoạn prod. Bắt đầu bằng thông báo cho người, chỉ tự động hoá hành động đảo ngược được.</p>
</div>

## Lab: bật Sentinel, hai rule, một playbook, một incident

Dùng workspace lab riêng, không dùng workspace prod. Bước 1, tạo workspace và onboard Sentinel. `az sentinel` là extension của Azure CLI (đang ở trạng thái preview, có thể đổi); hãy kiểm tra `az sentinel onboarding-state create -h` trước khi chạy. Cách bền hơn cho IaC là khai báo resource `Microsoft.SecurityInsights/onboardingStates` trong Bicep.

```bash
az group create --name rg-toinoi-sec-dev-sea --location southeastasia
az monitor log-analytics workspace create \
  --resource-group rg-toinoi-sec-dev-sea \
  --workspace-name log-toinoi-sec-dev-sea --location southeastasia

az extension add --name sentinel
az sentinel onboarding-state create \
  --resource-group rg-toinoi-sec-dev-sea \
  --workspace-name log-toinoi-sec-dev-sea \
  --name default
```

Lệnh cuối chỉ đăng ký Sentinel trên workspace, không bật connector, không tạo rule. Microsoft đang chuyển Sentinel sang Defender portal và đã công bố lộ trình dừng trải nghiệm trên Azure portal, nên tên menu bên dưới có thể khác.

Bước 2, cài solution từ **Content hub** rồi bật connector: **Microsoft Entra ID** (gửi SigninLogs, AuditLogs; cần Security Administrator hoặc vai trò cao hơn ở tenant, và SigninLogs cần license Entra ID P1/P2) và **Azure Activity** (cấu hình diagnostic setting của subscription, thường qua Azure Policy). Kiểm tra dữ liệu bằng `AzureActivity | take 5`; dữ liệu có thể trễ vài phút.

Bước 3, tạo hai analytics rule dạng Scheduled. Rule 1 bắt tạo hoặc sửa public IP trong resource group thuộc prod (tên có `-prod-`). Rule này cũng khớp khi cập nhật vì tạo và sửa dùng chung operation `write`. Giá trị trạng thái thành công trong `ActivityStatusValue` là `Success` ở schema hiện tại; lọc thêm `Succeeded` để xử lý được dữ liệu cũ.

```kusto
AzureActivity
| where TimeGenerated > ago(15m)
| where ingestion_time() > ago(5m)
| where OperationNameValue =~ "MICROSOFT.NETWORK/PUBLICIPADDRESSES/WRITE"
| where ActivityStatusValue in~ ("Success", "Succeeded")
| where ResourceGroup contains "-prod-"
| project TimeGenerated, Caller, CallerIpAddress, ResourceGroup, _ResourceId
```

Rule 2 bắt role assignment mới có role Owner (GUID bên dưới là role definition ID cố định của Owner). Chi tiết assignment nằm trong chuỗi JSON `Properties`: sự kiện Start mang `requestbody`, sự kiện Success mang `responseBody`, và chữ hoa chữ thường của khoá khác nhau. Vì vậy lọc bằng `has` trên cả chuỗi, rồi lấy một event mẫu để chỉnh đường dẫn `principalId` trước khi bật.

```kusto
AzureActivity
| where TimeGenerated > ago(15m)
| where ingestion_time() > ago(5m)
| where OperationNameValue =~ "MICROSOFT.AUTHORIZATION/ROLEASSIGNMENTS/WRITE"
| where ActivityStatusValue in~ ("Success", "Succeeded")
| where Properties has "8e3af657-a8ff-443c-a75c-2fe8c4bcb635"
| extend body = parse_json(tostring(parse_json(Properties).responseBody))
| extend principalId = tostring(body.properties.principalId)
| project TimeGenerated, Caller, CallerIpAddress, principalId, _ResourceId
```

Hai truy vấn chỉ là điểm bắt đầu: chưa lọc ngoại lệ hợp lệ (pipeline, PIM kích hoạt có chủ đích) và chưa chứng minh bắt được mọi kênh cấp quyền. Cho mỗi rule: chạy mỗi 5 phút, nhìn lại 15 phút. Cửa sổ `TimeGenerated` rộng để không sót event đến trễ; điều kiện `ingestion_time()` có độ rộng bằng tần suất chạy để mỗi event chỉ bắn một lần. Bật gộp alert thành incident và ánh xạ entity Account vào `Caller`, IP vào `CallerIpAddress`.

Bước 4, playbook. Tạo một Logic App (Consumption) với trigger **Microsoft Sentinel incident**. Hành động: gửi tin vào kênh Teams (tiêu đề, mức nghiêm trọng, link incident), rồi thêm comment vào incident cho biết đã báo ai. Cấp cho Sentinel quyền chạy playbook (vai trò Microsoft Sentinel Automation Contributor trên resource group chứa Logic App) và cấp cho managed identity của Logic App vai trò Microsoft Sentinel Responder để ghi comment. Cuối cùng tạo automation rule: khi incident được tạo từ hai rule trên, chạy playbook này. Phần mô tả này chưa chứng minh connector Teams đã được uỷ quyền; hãy kiểm tra bằng cách chạy thử playbook thủ công trên một incident mẫu.

Bước 5, kích hoạt. Tạo resource group giả prod chỉ để thử rồi một public IP trong đó:

```bash
az group create --name rg-toinoi-lab-prod-sea --location southeastasia
az network public-ip create \
  --resource-group rg-toinoi-lab-prod-sea \
  --name pip-toinoi-lab-prod --sku Standard
```

Chờ lịch rule chạy (có thể mất vài phút cộng độ trễ ingest). Kỳ vọng: một incident xuất hiện ở trang Incidents và một tin trong Teams. Mở incident, gán chủ, ghi kết luận, rồi đóng với phân loại phù hợp (ở đây là benign positive vì bạn tự tạo). Cuối lab: xoá `rg-toinoi-lab-prod-sea` và `rg-toinoi-sec-dev-sea` bằng `az group delete`, gỡ diagnostic setting của subscription trỏ về workspace lab, và xoá Logic App, để dừng tính tiền ingest.

**Bài tập:** Rule public IP ở prod bắn mỗi lần pipeline triển khai chạy Bicep, dù không có gì thay đổi. Giảm nhiễu thế nào mà không tắt rule? Và có nên cho playbook xoá IP tự động không?

<details><summary>Xem lời giải</summary><p>Loại trừ caller là danh tính pipeline đã biết (principal của federated credential) hoặc chỉ báo khi public IP mới xuất hiện, không phải mỗi lần ghi. Danh sách loại trừ phải có chủ và hạn rà lại, vì kẻ tấn công cũng muốn mượn danh tính đó. Không nên cho playbook xoá IP tự động ở giai đoạn đầu: có thể xoá nhầm một thay đổi hợp lệ, hoặc bị lợi dụng để gây gián đoạn. Hãy báo cho người trước.</p></details>

## Học tiếp từ đây

Bạn vừa đặt các lớp bảo mật đã có dưới ba nguyên tắc Zero Trust, thấy chỗ phân đoạn, dữ liệu và supply chain còn cần siết, và dựng một đường phát hiện từ Activity log tới Teams qua Sentinel. Duy trì rule và trực incident là việc hằng ngày, không phải cấu hình một lần.

Bài tiếp theo là [bài 56](/articles/azure-load-testing-autoscale-chaos-studio): đo hiệu năng bằng Azure Load Testing, autoscale và Chaos Studio, để kiểm tra hệ thống chịu tải và chịu lỗi ra sao.

## Tài liệu gốc

- [Microsoft Sentinel documentation](https://learn.microsoft.com/en-us/azure/sentinel/)
- [Zero Trust guidance center](https://learn.microsoft.com/en-us/security/zero-trust/)
- [Azure Bastion](https://learn.microsoft.com/en-us/azure/bastion/bastion-overview)
- [Automate threat response with playbooks in Microsoft Sentinel](https://learn.microsoft.com/en-us/azure/sentinel/automate-responses-with-playbooks)
- [Azure encryption at rest](https://learn.microsoft.com/en-us/azure/security/fundamentals/encryption-atrest)
- [Microsoft Purview](https://learn.microsoft.com/en-us/purview/)

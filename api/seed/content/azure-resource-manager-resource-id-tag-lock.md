---
slug: azure-resource-manager-resource-id-tag-lock
title: "Azure Resource Manager: resource provider, resource ID, tag, lock và naming"
summary: "Bài 4: đi vào Azure Resource Manager của Tới Nơi: control plane và data plane, resource provider, cấu trúc resource ID, tag không kế thừa, lock bảo vệ PostgreSQL prod và bảng quy ước đặt tên theo từng loại resource."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, arm, resource-id, naming]
language: vi
---

Đây là **bài 4** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 4/10 của phần).

Bài 3 cho thấy mọi lệnh `az` là một request tới Azure Resource Manager (ARM). Bài này mở cánh cửa đó ra: ARM làm gì với request, resource được định danh thế nào và Tới Nơi dùng tag, lock và quy ước đặt tên để giữ mọi thứ gọn gàng. Lệnh trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã đặt lock CanNotDelete lên PostgreSQL server, một người có thể chạy <code>DROP TABLE trips</code> được không?</p>
<details><summary>Xem đáp án</summary><p>Được, nếu người đó có quyền trong database. Lock chỉ chặn thao tác trên control plane của Azure, như xoá cả server. Nó không nhìn vào SQL chạy trên data plane.</p></details>
</div>

## ARM là control plane, dữ liệu của bạn đi đường khác

Khi bạn tạo, sửa hoặc xoá một resource, request được gửi tới ARM và xử lý theo một chuỗi chung: xác thực danh tính, kiểm tra quyền RBAC, đánh giá policy và lock, ghi Activity Log, rồi mới chuyển cho **resource provider** phụ trách loại resource đó. Thứ tự chi tiết do ARM quyết định; điều cần nhớ là mọi thao tác quản lý đều qua cổng chung này, nên chính sách đặt ở đây áp dụng được cho mọi công cụ.

Đó là **control plane**: quản lý bản thân resource (tạo server, đổi cấu hình, xoá). **Data plane** là đường đi của dữ liệu: đọc blob, `SELECT` từ bảng `trips`. Data plane không đi qua ARM. Ứng dụng của Tới Nơi kết nối thẳng tới endpoint của PostgreSQL hoặc Blob Storage, và dùng cơ chế xác thực riêng của dịch vụ đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Control plane đi qua ARM với xác thực, RBAC, policy, lock rồi tới resource provider; data plane đi thẳng từ ứng dụng tới endpoint dữ liệu."><defs><marker id="az04-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="170" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Request</text><text x="34" y="82" class="t-small">az, Portal, Bicep</text><path d="M190 67 L225 67" class="f-none s-phys" stroke-width="2" marker-end="url(#az04-0-a)"/><rect x="230" y="25" width="200" height="85" rx="10" class="f-phys-soft s-phys"/><text x="244" y="55" class="t-b">Xác thực + RBAC</text><text x="244" y="82" class="t-small">Ai? Có quyền không?</text><path d="M430 67 L465 67" class="f-none s-phys" stroke-width="2" marker-end="url(#az04-0-a)"/><rect x="470" y="25" width="200" height="85" rx="10" class="f-warn-soft s-warn"/><text x="484" y="55" class="t-b">Policy + Lock</text><text x="484" y="82" class="t-small">Rồi ghi Activity Log</text><path d="M670 67 L705 67" class="f-none s-phys" stroke-width="2" marker-end="url(#az04-0-a)"/><rect x="710" y="25" width="160" height="85" rx="10" class="f-phys-soft s-phys"/><text x="724" y="55" class="t-b">Resource</text><text x="724" y="82" class="t-small">provider thực thi</text><rect x="20" y="185" width="240" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="215" class="t-b">trip-service</text><text x="34" y="242" class="t-small">Chạy trên Container Apps</text><path d="M260 227 L405 227" class="f-none s-pk" stroke-width="2" marker-end="url(#az04-0-a)"/><rect x="410" y="185" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="424" y="215" class="t-b">Endpoint dữ liệu</text><text x="424" y="242" class="t-small">PostgreSQL, Blob: data plane</text></svg>
</div>
<figcaption>Hình 1. Control plane đi qua ARM với xác thực, RBAC, policy, lock rồi tới resource provider; data plane đi thẳng từ ứng dụng tới endpoint dữ liệu.</figcaption>
</figure>

Hệ quả cho Tới Nơi: một người có quyền quản lý PostgreSQL server (control plane) chưa chắc đọc được bảng `trips`, và ngược lại. Bài về RBAC và identity ở Phần 2 sẽ tách hai lớp quyền này rõ hơn.

## Resource provider: mỗi loại resource có một người phụ trách

Mỗi loại resource thuộc một **resource provider**, gọi bằng **namespace** có dạng `Microsoft.<Dịch vụ>`. Trong Tới Nơi, bạn sẽ gặp:

| Namespace | Loại resource Tới Nơi dùng |
| --- | --- |
| `Microsoft.Network` | VNet, NSG, public IP |
| `Microsoft.Compute` | Virtual Machine, disk |
| `Microsoft.Storage` | Storage account, Blob |
| `Microsoft.DBforPostgreSQL` | PostgreSQL Flexible Server |
| `Microsoft.App` | Container Apps environment, Container App |
| `Microsoft.KeyVault` | Key Vault |
| `Microsoft.ContainerRegistry` | Azure Container Registry |
| `Microsoft.ManagedIdentity` | User-assigned managed identity |

Trước khi tạo được resource thuộc một namespace, subscription phải **đăng ký** (register) namespace đó. Portal hay tự đăng ký giúp bạn khi tạo lần đầu, còn CLI, Bicep hoặc pipeline có thể gặp lỗi `MissingSubscriptionRegistration` nếu chưa đăng ký. Quyền đăng ký thường nằm ở Owner hoặc Contributor trên subscription; một role hẹp hơn có thể không làm được, và lúc đó phải nhờ người có quyền.

```bash
az provider show \
  --namespace Microsoft.DBforPostgreSQL \
  --query registrationState \
  --output tsv

az provider register --namespace Microsoft.DBforPostgreSQL --wait

az provider show \
  --namespace Microsoft.DBforPostgreSQL \
  --query "resourceTypes[?resourceType=='flexibleServers'] | [0].apiVersions[:3]"
```

Khối này kiểm tra trạng thái đăng ký, đăng ký namespace và chờ xong rồi in vài API version của loại `flexibleServers`. Nó không chứng minh bạn đủ quota hoặc đủ quyền để tạo server ở region đã chọn; đăng ký chỉ mở khả năng gọi provider.

**API version** là điểm dễ bị bỏ qua. Mỗi request tới ARM kèm một `api-version`, và provider hỗ trợ nhiều version song song. Bicep và template luôn ghi version trong khai báo, ví dụ `Microsoft.App/containerApps@2025-01-01` ở bài 36 loạt C#. Nhờ ghi cố định version, template cũ vẫn chạy ổn định còn template mới dùng được tính năng mới; nâng version có thể thêm hoặc đổi thuộc tính. Nên chọn version theo trang schema của resource trên Microsoft Learn.

## Resource ID là địa chỉ đầy đủ của mọi resource

Mỗi resource có một **resource ID**, chuỗi duy nhất toàn Azure. Ví dụ ID của PostgreSQL server production của Tới Nơi:

`/subscriptions/<subscription-id>/resourceGroups/rg-toinoi-data-prod-sea/providers/Microsoft.DBforPostgreSQL/flexibleServers/psql-toinoi-prod-sea`

Đọc từ trái sang phải, ID là một đường dẫn phân cấp:

| Đoạn | Ý nghĩa |
| --- | --- |
| `/subscriptions/<subscription-id>` | Subscription chứa resource |
| `/resourceGroups/rg-toinoi-data-prod-sea` | Resource group |
| `/providers/Microsoft.DBforPostgreSQL` | Namespace của resource provider |
| `/flexibleServers/psql-toinoi-prod-sea` | Loại resource và tên cụ thể |
| `/databases/trips` (nếu thêm) | Resource con nằm trong server |

ID quan trọng vì nó chính là **scope**. Gán role RBAC, cấu hình diagnostic, áp policy hay đặt lock đều nhận một ID làm phạm vi. ID của subscription, của resource group và của resource đều là scope hợp lệ, và gán ở scope càng cao thì phạm vi ảnh hưởng càng rộng. Tên thì có thể trùng giữa hai resource group (với loại có phạm vi duy nhất là resource group), còn ID thì không bao giờ trùng.

```bash
PG_ID=$(az resource show \
  --resource-group rg-toinoi-data-prod-sea \
  --resource-type Microsoft.DBforPostgreSQL/flexibleServers \
  --name psql-toinoi-prod-sea \
  --query id --output tsv)

az resource show --ids "$PG_ID" \
  --query "{Name:name, Type:type, Region:location}" \
  --output table

az role assignment list --scope "$PG_ID" --output table
```

Khối này lấy ID của server, dùng lại ID đó để đọc resource và liệt kê role gán đúng ở scope này. Hãy thay tên cho khớp lab của bạn. Lệnh cuối chỉ liệt kê gán trực tiếp ở scope server trong ngữ cảnh bạn xem; nó không tự tổng hợp mọi quyền kế thừa từ resource group hay subscription, nên chưa trả lời câu "ai thực sự đụng được vào server này".

## Tag nối chi phí và trách nhiệm, nhưng không kế thừa

**Tag** là cặp tên và giá trị gắn lên resource, resource group hoặc subscription. Tới Nơi dùng ba tag nền: `env` (dev, staging, prod), `owner` (đội chịu trách nhiệm) và `cost-center`. Mục đích: lọc chi phí theo đội, biết gọi ai khi resource lạ xuất hiện, và tìm tất cả resource của một môi trường.

Ba điểm cần nhớ:

- **Tag không kế thừa.** Gắn tag lên resource group không tự đưa tag xuống resource bên trong. Muốn đồng bộ, bạn gắn từng resource, hoặc dùng policy để kế thừa/bắt buộc (nội dung của bài về governance sau này). Ngược lại, lock và RBAC thì kế thừa xuống scope con.
- **Có giới hạn** về số tag trên mỗi resource và độ dài tên, giá trị; một số loại resource không hỗ trợ tag. Số cụ thể thay đổi theo thời gian, hãy đọc trang giới hạn tag trong tài liệu trước khi thiết kế quy ước dài.
- **Tag không phải chỗ cất bí mật.** Ai đọc được resource cũng đọc được tag, và tag xuất hiện trong báo cáo chi phí. Không đặt mật khẩu, token hay dữ liệu cá nhân của khách vào đó.

```bash
az tag update \
  --resource-id "$PG_ID" \
  --operation Merge \
  --tags env=prod owner=platform cost-center=toinoi

az tag list --resource-id "$PG_ID" --output json

az resource list --tag env=prod \
  --query "[].{Name:name, Type:type, Group:resourceGroup}" \
  --output table
```

Khối này gộp ba tag vào resource mà không xoá tag có sẵn (`Merge`), đọc lại tag rồi liệt kê mọi resource có `env=prod`. Khối này không bắt buộc ai phải gắn tag; muốn ép quy ước thì cần policy.

Hình 2 gom lại phần phân cấp: scope nào truyền xuống scope con, và thứ gì không truyền.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Phân cấp scope từ subscription tới resource con; RBAC, policy và lock kế thừa xuống, còn tag phải gắn riêng cho từng resource."><defs><marker id="az04-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="50" class="t-b">Subscription</text><text x="34" y="77" class="t-small">/subscriptions/&lt;id&gt;</text><path d="M200 62 L240 62" class="f-none s-phys" stroke-width="2" marker-end="url(#az04-1-a)"/><rect x="245" y="20" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="259" y="50" class="t-b">Resource group</text><text x="259" y="77" class="t-small">rg-toinoi-data-prod-sea</text><path d="M425 62 L465 62" class="f-none s-phys" stroke-width="2" marker-end="url(#az04-1-a)"/><rect x="470" y="20" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="484" y="50" class="t-b">PostgreSQL server</text><text x="484" y="77" class="t-small">flexibleServers</text><path d="M650 62 L690 62" class="f-none s-phys" stroke-width="2" marker-end="url(#az04-1-a)"/><rect x="695" y="20" width="175" height="85" rx="10" class="f-pk-soft s-pk"/><text x="709" y="50" class="t-b">Database trips</text><text x="709" y="77" class="t-small">Resource con</text><rect x="20" y="150" width="850" height="48" rx="10" class="f-phys-soft s-phys"/><text x="36" y="180" class="t-b">RBAC, policy, lock: kế thừa xuống mọi scope con</text><rect x="20" y="215" width="850" height="48" rx="10" class="f-warn-soft s-warn"/><text x="36" y="245" class="t-b">Tag: gắn riêng từng resource, không kế thừa</text></svg>
</div>
<figcaption>Hình 2. Phân cấp scope từ subscription tới resource con; RBAC, policy và lock kế thừa xuống, còn tag phải gắn riêng cho từng resource.</figcaption>
</figure>

## Lock chặn xoá nhầm, không thay cho RBAC

**Lock** có hai mức. `CanNotDelete`: vẫn đọc và sửa được nhưng không xoá được. `ReadOnly`: chỉ đọc, và nó chặn mọi thay đổi qua control plane. Lock áp dụng cho mọi người, kể cả Owner, và kế thừa xuống resource con. Tới Nơi đặt `CanNotDelete` lên PostgreSQL server prod để một lệnh `az group delete` hay một lần bấm nhầm trên Portal không xoá được dữ liệu.

```bash
az lock create \
  --name lock-psql-no-delete \
  --lock-type CanNotDelete \
  --resource-group rg-toinoi-data-prod-sea \
  --resource-name psql-toinoi-prod-sea \
  --resource-type Microsoft.DBforPostgreSQL/flexibleServers

az lock list \
  --resource-group rg-toinoi-data-prod-sea \
  --output table
```

Khối này tạo lock mức CanNotDelete trên server rồi liệt kê lock của resource group. Lock không bảo vệ dữ liệu bên trong, như đã thấy ở câu hỏi mở đầu.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Lock không thay cho RBAC. Ai có quyền <code>Microsoft.Authorization/locks/*</code> (Owner và User Access Administrator có) vẫn gỡ được lock rồi xoá. Còn <code>ReadOnly</code> dễ gây bất ngờ: một số thao tác trông như "đọc" thực chất là POST tới control plane (ví dụ liệt kê khoá của storage account) và bị chặn. Lock cũng làm pipeline xoá resource group lab bị thất bại; hãy gỡ lock có chủ đích, không gỡ hàng loạt.</p>
</div>

## Quy ước đặt tên: mỗi loại resource một luật

Tên resource thường **không đổi được** sau khi tạo; muốn đổi phải tạo lại và di chuyển dữ liệu. Vì vậy Tới Nơi chốt quy ước từ đầu: `<loại>-toinoi-<workload>-<env>-<region>`, trong đó `<loại>` là tiền tố ngắn (`rg`, `vnet`, `kv`, `psql`, `id`, `ca`), `<env>` là dev/staging/prod và `<region>` là `sea` cho Southeast Asia. Resource dùng chung cho cả môi trường (VNet, Key Vault, PostgreSQL) bỏ phần `<workload>`; managed identity và Container App bỏ phần `<region>` theo quy ước của loạt, vì resource group chứa chúng đã mang hậu tố region.

Nhưng mỗi resource provider có luật riêng, nên một quy ước chung phải co lại cho vừa loại khó tính nhất. Storage account không cho gạch nối và chỉ nhận chữ thường, nên `sttoinoi<mucdich><env>` bỏ cả dấu gạch lẫn region.

| Resource | Tên Tới Nơi | Độ dài | Ký tự cho phép | Phạm vi duy nhất |
| --- | --- | --- | --- | --- |
| Resource group | `rg-toinoi-data-prod-sea` | 1–90 | Chữ, số, `_`, `-`, `.`, `()`; không kết thúc bằng `.` | Subscription |
| Virtual network | `vnet-toinoi-prod-sea` | 2–64 | Chữ, số, `_`, `-`, `.`; bắt đầu bằng chữ hoặc số, kết thúc bằng chữ, số hoặc `_` | Resource group |
| Network security group | `nsg-toinoi-app-prod-sea` | 1–80 | Chữ, số, `_`, `-`, `.`; bắt đầu bằng chữ hoặc số, kết thúc bằng chữ, số hoặc `_` | Resource group |
| Storage account | `sttoinoidocsprod` | 3–24 | Chỉ chữ thường và số | Toàn Azure |
| Key Vault | `kv-toinoi-prod-sea` | 3–24 | Chữ, số, `-`; bắt đầu bằng chữ, kết thúc bằng chữ hoặc số; không hai `-` liền nhau | Toàn Azure |
| Container registry | `acrtoinoiprod` | 5–50 | Chỉ chữ và số | Toàn Azure |
| Managed identity | `id-toinoi-trips-prod` | 3–128 | Chữ, số, `_`, `-`; bắt đầu bằng chữ hoặc số | Resource group |
| Container App | `ca-toinoi-trips-prod` | 2–32 | Chữ thường, số, `-`; bắt đầu bằng chữ, kết thúc bằng chữ hoặc số; không hai `-` liền nhau | Resource group |
| PostgreSQL server | `psql-toinoi-prod-sea` | 3–63 | Chữ thường, số, `-`; không bắt đầu/kết thúc bằng `-` | Toàn Azure |
| Log Analytics workspace | `log-toinoi-prod-sea` | 4–63 | Chữ, số, `-`; bắt đầu và kết thúc bằng chữ hoặc số | Resource group |

Bảng này tóm tắt luật tại thời điểm viết; hãy đối chiếu với trang quy tắc đặt tên và hạn chế của Microsoft Learn trước khi chốt, vì provider có thể đổi. Vài hệ quả đáng ghi nhớ:

- Tên cho resource có endpoint DNS (storage, Key Vault, registry, PostgreSQL) phải **duy nhất toàn Azure**. Kiểm tra trước khi tạo, ví dụ `az storage account check-name` hoặc `az acr check-name`.
- Giới hạn 24 ký tự của storage: sau `sttoinoi` (8 ký tự) còn tối đa 16 cho phần mục đích và môi trường. Với `staging` (7 ký tự), phần mục đích chỉ còn khoảng 9 ký tự.
- Key Vault bị xoá mềm vẫn **giữ tên** trong thời gian lưu giữ, nên tên của vault vừa xoá có thể chưa dùng lại được ngay.
- Tên đẹp mà thiếu tag vẫn không trả lời được "ai chịu chi phí". Tên và tag bổ sung cho nhau.

```bash
az storage account check-name \
  --name sttoinoidocsprod \
  --query "{Available:nameAvailable, Reason:reason}" \
  --output table

az acr check-name \
  --name acrtoinoiprod \
  --query "{Available:nameAvailable, Reason:reason}" \
  --output table
```

Khối này hỏi Azure xem tên storage account và registry đã bị chiếm chưa. Kết quả "available" chỉ đúng tại thời điểm hỏi, chưa giữ chỗ cho bạn; người khác vẫn có thể tạo tên đó trước khi bạn tạo.

**Bài tập:** Tới Nơi cần storage account cho môi trường staging, mục đích lưu ảnh giấy tờ tài xế. Tên `sttoinoi-giaytotaixe-staging` có hợp lệ không? Đề xuất một tên đúng luật.

<details><summary>Xem lời giải</summary><p>Không hợp lệ: có dấu gạch nối (storage chỉ nhận chữ thường và số) và dài 28 ký tự, vượt giới hạn 24. Một tên đúng luật là <code>sttoinoidocsstaging</code>, gồm 8 + 4 + 7 = 19 ký tự toàn chữ thường. Sau đó vẫn phải chạy <code>az storage account check-name</code> vì tên phải duy nhất toàn Azure.</p></details>

## Học tiếp từ đây

Bạn đã biết ARM là cổng chung của control plane, resource provider cần đăng ký, resource ID chính là scope, tag không kế thừa, lock chặn xoá nhưng không thay RBAC, và mỗi loại resource có luật đặt tên riêng nên bảng quy ước cần được kiểm tra thực tế trước khi dùng.

Bài tiếp theo là [Bài 5](/articles/azure-chi-phi-pricing-cost-management-budget): tag và tên bạn vừa chốt sẽ được dùng để đọc chi phí, đặt budget và hiểu hoá đơn tháng đầu của Tới Nơi. Quay lại [Bài 3](/articles/azure-portal-cli-cloud-shell-resource-group-dau-tien) nếu cần ôn cách chạy `az` và lọc kết quả bằng `--query`.

## Tài liệu gốc

- [Azure Resource Manager overview](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/overview)
- [Azure resource providers and types](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-providers-and-types)
- [Use tags to organize your Azure resources](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/tag-resources)
- [Lock your resources to protect your infrastructure](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/lock-resources)
- [Naming rules and restrictions for Azure resources](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules)

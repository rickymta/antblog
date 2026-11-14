---
slug: azure-tenant-subscription-resource-group
title: "Tenant, subscription, management group và resource group"
summary: "Bài 2: vẽ phân cấp tenant, management group, subscription và resource group cho Tới Nơi, tách ba subscription dev, staging, prod và chọn resource group cho trip-service, PostgreSQL và monitoring theo vòng đời."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, governance, resource-group, subscription]
language: vi
---

Đây là **bài 2** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 2/10 của phần).

Bài này dựng khung tổ chức mà mọi resource sau này sẽ nằm trong: bốn cấp scope, cái gì kế thừa xuống, và cách Tới Nơi chia subscription và resource group. Đoạn Bicep trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource group sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai resource nằm chung một resource group thì có tự nói chuyện được với nhau qua mạng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Resource group là nhóm để quản lý và gán quyền. Khả năng kết nối do mạng quyết định: VNet, subnet, NSG và các quy tắc truy cập của từng resource.</p></details>
</div>

## Bốn cấp scope và thứ tự kế thừa

Azure tổ chức mọi thứ thành một cây. Trên cùng là **tenant**, một thư mục Microsoft Entra ID chứa người dùng, nhóm và service principal của tổ chức. Dưới tenant có thể có các **management group** để gom nhiều subscription. Mỗi **subscription** là một vùng chứa riêng, và bên trong nó là các **resource group**, nơi cuối cùng chứa **resource** như máy ảo, database, storage.

Cây này không phải để vẽ cho đẹp. Các cấp trong cây chính là các **scope**: chỗ bạn gán quyền và policy. Quyền (RBAC role assignment) gán ở scope nào thì có hiệu lực với mọi thứ bên dưới scope đó. **Azure Policy** cũng vậy: policy gán ở management group áp xuống mọi subscription con. Resource lock cũng kế thừa xuống. Có một ngoại lệ đáng nhớ là **tag**: tag gắn ở subscription hay resource group không tự chảy xuống resource. Muốn resource nhận tag của resource group thì phải gắn riêng hoặc dùng policy để sao chép.

Một chi tiết kỹ thuật: Azure RBAC nhận bốn cấp scope là management group, subscription, resource group và resource; bản thân tenant không nằm trong danh sách đó. Cấp cao nhất thường dùng là root management group (Tenant Root Group), nằm ngay dưới tenant. Số cấp management group lồng nhau cũng có giới hạn, hãy kiểm tra tài liệu trước khi thiết kế cây sâu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 285" role="img" aria-label="Hình 1. Phân cấp scope từ tenant xuống resource; RBAC, Policy và lock kế thừa xuống, tag thì không."><defs><marker id="az02-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="15" width="380" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="43" class="t-b">Tenant</text><text x="190" y="43" class="t-small">Thư mục Entra ID</text><rect x="60" y="67" width="380" height="44" rx="10" class="f-phys-soft s-phys"/><text x="74" y="95" class="t-b">Management group</text><text x="230" y="95" class="t-small">Gom nhiều subscription</text><rect x="100" y="119" width="380" height="44" rx="10" class="f-pk-soft s-pk"/><text x="114" y="147" class="t-b">Subscription</text><text x="270" y="147" class="t-small">Billing, quota, ranh giới</text><rect x="140" y="171" width="380" height="44" rx="10" class="f-pk-soft s-pk"/><text x="154" y="199" class="t-b">Resource group</text><text x="310" y="199" class="t-small">Nhóm theo vòng đời</text><rect x="180" y="223" width="380" height="44" rx="10" class="f-surface s-line"/><text x="194" y="251" class="t-b">Resource</text><text x="350" y="251" class="t-small">VM, database, storage</text><path d="M610 20 L610 255" class="f-none s-pk" stroke-width="2" marker-end="url(#az02-0-a)"/><text x="630" y="60" class="t-small">RBAC, Policy, Lock</text><text x="630" y="82" class="t-small">đi từ trên xuống dưới</text><rect x="630" y="150" width="230" height="95" rx="10" class="f-warn-soft s-warn"/><text x="644" y="180" class="t-b t-warn">Tag</text><text x="644" y="205" class="t-small">Không tự kế thừa xuống,</text><text x="644" y="225" class="t-small">tự gắn hoặc dùng Policy</text></svg>
</div>
<figcaption>Hình 1. Phân cấp scope từ tenant xuống resource; RBAC, Policy và lock kế thừa xuống, tag thì không.</figcaption>
</figure>

Kế thừa là con dao hai lưỡi. Gán Contributor ở subscription nghĩa là người đó sửa được mọi resource group bên trong, kể cả cái chưa tạo. Nguyên tắc gán quyền ở scope thấp nhất đủ dùng sẽ được nói kỹ ở phần Danh tính và bảo mật.

## Subscription là ranh giới billing và quota

Subscription là đơn vị tính hoá đơn: chi phí được gom và báo cáo theo subscription. Subscription cũng là đơn vị của **quota** và giới hạn dịch vụ, ví dụ số vCPU được dùng ở mỗi region. Vì vậy tách subscription là cách rẻ nhất để dựng một ranh giới thật: một đợt load test ở dev có thể ăn hết quota vCPU của subscription dev, và prod không bị ảnh hưởng vì quota của nó tính riêng.

Tới Nơi chia theo môi trường: `sub-toinoi-dev`, `sub-toinoi-staging`, `sub-toinoi-prod`. Tên subscription là nhãn hiển thị, đặt tuỳ ý. Khi hệ thống lớn lên, đội sẽ thêm một subscription theo nền tảng (platform-shared) để chứa thứ dùng chung như registry, DNS và logging tập trung. Bài 49 về landing zone và subscription vending (Phần 5) sẽ quay lại chuyện này, còn lab hôm nay chỉ vẽ ba subscription.

| Tiêu chí | Một subscription chung | Subscription theo môi trường |
| --- | --- | --- |
| Hoá đơn | Một cục, phải tag mới tách được | Tách sẵn theo môi trường |
| Quota | Dev và prod tranh nhau một hạn mức | Mỗi môi trường một hạn mức riêng |
| Quyền | Dễ cấp thừa vì mọi thứ chung chỗ | Dev rộng quyền, prod chặt, tách bạch |
| Rủi ro xoá nhầm | Dev xoá nhầm có thể chạm prod | Lệnh ở dev không chạm resource prod |
| Công sức vận hành | Ít việc hơn | Nhiều subscription, cần chuẩn hoá bằng template |

Cái giá nằm ở dòng cuối. Ba subscription nghĩa là ba bộ quyền, ba bộ policy và ba lần tạo resource lặp lại. Đó là lý do management group và Infrastructure as Code (Bicep) xuất hiện sớm trong loạt.

**Management group** giải đúng bài toán đó. Tới Nơi bắt đầu với một `mg-toinoi` chứa cả ba subscription và gán ở đó các luật chung. Ví dụ điển hình là policy giới hạn region được phép tạo resource chỉ còn `southeastasia` và `eastasia`, để một dev lỡ chọn region mặc định khác trong Portal không tạo được resource ở đó. Lưu ý policy dựng sẵn «Allowed locations» chỉ áp cho resource; resource group có một policy riêng là «Allowed locations for resource groups», nên thường phải gán cả hai. Khi cần luật khác nhau cho prod và non-prod, mới tách thêm tầng con.

## Resource group gom theo vòng đời, không theo loại resource

**Resource group** (RG) là nơi chứa các resource có chung vòng đời: triển khai cùng nhau, cập nhật cùng nhau, và xoá cùng nhau. Đó là tiêu chí gom đúng. Gom theo loại (một RG cho mọi database, một RG cho mọi VM) nghe gọn nhưng buộc những thứ có vòng đời khác nhau vào cùng một nhóm, và lúc xoá hay phân quyền sẽ phải làm việc với cả đống thứ không liên quan.

Quy ước đặt tên của Tới Nơi là `rg-toinoi-<workload>-<env>-<region>`. Ví dụ `rg-toinoi-trips-prod-sea` đọc ra: nhóm của workload `trips`, môi trường prod, region Southeast Asia. Đặt tên có cấu trúc giúp lọc bằng lệnh và đọc hoá đơn mà không phải mở từng resource.

Bicep sau tạo ba RG cho một môi trường ở scope subscription. Bicep này chỉ tạo RG rỗng, không chứng minh bạn có quyền tạo RG ở subscription đó, và không kiểm tra policy chặn tên hay region.

```bicep
targetScope = 'subscription'

@allowed(['dev', 'staging', 'prod'])
param environmentName string
// Hậu tố -sea trong tên khớp location mặc định; đổi location thì đổi cả hậu tố.
param location string = 'southeastasia'

var workloads = ['trips', 'data', 'monitor']

resource rgs 'Microsoft.Resources/resourceGroups@2024-03-01' = [for w in workloads: {
  name: 'rg-toinoi-${w}-${environmentName}-sea'
  location: location
}]
```

Triển khai ở scope subscription dùng lệnh `az deployment sub create`, khác với `az deployment group create` dùng cho RG. Khối dưới chạy trên subscription dev đang chọn. Khối này không chứng minh template không có lỗi với mọi API version, nên hãy đọc kết quả của lệnh.

```bash
az deployment sub create \
  --location southeastasia \
  --template-file rgs.bicep \
  --parameters environmentName=dev
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Xoá một resource group là xoá tất cả resource bên trong, và lệnh xoá không phân biệt cái nào quan trọng. Nếu dev và prod chung một RG, hoặc PostgreSQL chung RG với app hay xoá đi dựng lại, một lệnh dọn dẹp là đủ để mất dữ liệu. Vì lý do đó Tới Nơi đặt database ở RG riêng và đặt resource lock (bài 4) lên prod.</p>
</div>

## Resource group không phải ranh giới mạng hay bảo mật

Đây là ngộ nhận phổ biến nhất của người mới. Hai điều cần nhớ.

Thứ nhất, **RG không cô lập mạng**. Một VM ở `rg-toinoi-trips-dev-sea` hoàn toàn có thể gọi PostgreSQL ở `rg-toinoi-data-dev-sea` nếu đường mạng và quyền cho phép, và ngược lại hai resource cùng RG vẫn bị chặn nếu NSG không cho. Ranh giới mạng là VNet, subnet, NSG và firewall của từng dịch vụ, sẽ gặp ở [bài 6](/articles/azure-vnet-subnet-nsg-public-ip). RG cũng không phải ranh giới bảo mật. RG chỉ tiện cho việc gán quyền, vì quyền gán ở RG áp xuống mọi resource trong nhóm.

Thứ hai, **resource trong RG có thể ở region khác với RG**. RG có một location, nhưng đó là nơi lưu metadata của nhóm, không bắt resource bên trong cùng region. Điều này linh hoạt, nhưng có hệ quả: nếu region của RG gặp sự cố, bạn có thể không cập nhật được resource trong RG đó dù resource ở region còn khoẻ, vì metadata không truy cập được. Quy tắc thực dụng cho Tới Nơi là giữ RG cùng region với resource chính. Khi dựng phương án thảm hoạ ở East Asia, tạo RG riêng, chẳng hạn `rg-toinoi-trips-prod-ea`, thay vì nhét resource Hồng Kông vào RG Singapore.

Một điểm nữa: mỗi resource thuộc đúng một RG. Di chuyển resource giữa các RG làm được với nhiều loại nhưng không phải mọi loại, nên chọn RG cho đúng ngay từ đầu rẻ hơn là chuyển sau.

## Lab: sơ đồ ba subscription và RG cho trip-service, PostgreSQL, monitoring

Lab này làm trên giấy hoặc công cụ vẽ, không cần tạo resource. Bước 1: vẽ `mg-toinoi` ở trên và ba subscription bên dưới. Bước 2: liệt kê resource mà mỗi môi trường cần, gồm VNet, ứng dụng `trip-service`, PostgreSQL, Key Vault, registry và Log Analytics workspace. Bước 3: gán từng resource vào RG theo vòng đời, rồi tự hỏi «nếu xoá RG này thì mất gì».

| Resource | Resource group | Lý do |
| --- | --- | --- |
| `ca-toinoi-trips-<env>` và `cae-toinoi-<env>` | `rg-toinoi-trips-<env>-sea` | Deploy lại thường xuyên, xoá dựng lại được |
| `psql-toinoi-<env>-sea` | `rg-toinoi-data-<env>-sea` | Chứa dữ liệu, vòng đời dài, cần lock ở prod |
| Log Analytics workspace | `rg-toinoi-monitor-<env>-sea` | Log phải còn lại khi app bị xoá để điều tra |
| `vnet-toinoi-<env>-sea` | `rg-toinoi-network-<env>-sea` | Nhiều workload dùng chung, thay đổi hiếm |

Tên RG cho mạng không nằm trong ba RG của đoạn Bicep, vì bài 6 mới dựng mạng. Bạn có thể thêm `network` vào mảng `workloads` khi tới lúc đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. mg-toinoi chứa ba subscription dev, staging, prod; mỗi subscription có cùng ba resource group trips, data, monitor."><defs><marker id="az02-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="15" width="840" height="60" rx="10" class="f-phys-soft s-phys"/><text x="34" y="42" class="t-b">mg-toinoi</text><text x="34" y="62" class="t-small">Policy: chỉ southeastasia, eastasia</text><path d="M150 75 L150 105" class="f-none s-phys" stroke-width="2" marker-end="url(#az02-1-a)"/><path d="M440 75 L440 105" class="f-none s-phys" stroke-width="2" marker-end="url(#az02-1-a)"/><path d="M730 75 L730 105" class="f-none s-phys" stroke-width="2" marker-end="url(#az02-1-a)"/><rect x="20" y="108" width="260" height="180" rx="10" class="f-pk-soft s-pk"/><text x="34" y="138" class="t-b">sub-toinoi-dev</text><text x="34" y="170" class="t-mono">rg-toinoi-trips-dev-sea</text><text x="34" y="198" class="t-mono">rg-toinoi-data-dev-sea</text><text x="34" y="226" class="t-mono">rg-toinoi-monitor-dev-sea</text><text x="34" y="262" class="t-small">Quyền rộng, dễ xoá dựng lại</text><rect x="310" y="108" width="260" height="180" rx="10" class="f-pk-soft s-pk"/><text x="324" y="138" class="t-b">sub-toinoi-staging</text><text x="324" y="170" class="t-mono">rg-toinoi-trips-staging-sea</text><text x="324" y="198" class="t-mono">rg-toinoi-data-staging-sea</text><text x="324" y="226" class="t-mono">rg-toinoi-monitor-staging-sea</text><text x="324" y="262" class="t-small">Giống prod để thử release</text><rect x="600" y="108" width="260" height="180" rx="10" class="f-warn-soft s-warn"/><text x="614" y="138" class="t-b">sub-toinoi-prod</text><text x="614" y="170" class="t-mono">rg-toinoi-trips-prod-sea</text><text x="614" y="198" class="t-mono">rg-toinoi-data-prod-sea</text><text x="614" y="226" class="t-mono">rg-toinoi-monitor-prod-sea</text><text x="614" y="262" class="t-small">Quyền chặt, có lock</text></svg>
</div>
<figcaption>Hình 2. mg-toinoi chứa ba subscription dev, staging, prod; mỗi subscription có cùng ba resource group trips, data, monitor.</figcaption>
</figure>

Sơ đồ này là một quyết định thiết kế, không phải đáp án duy nhất. Một đội nhỏ hơn có thể gộp staging vào dev trong giai đoạn đầu. Điều quan trọng là mỗi lựa chọn có lý do viết ra được, và bạn đổi nó có chủ đích khi nhu cầu đổi.

**Bài tập:** Tới Nơi muốn xoá sạch môi trường dev cuối tuần để tiết kiệm chi phí, nhưng muốn giữ log của tuần đó để điều tra lỗi. Với sơ đồ trên thì xoá RG nào, giữ RG nào? Nếu mọi resource dev nằm chung một RG thì chuyện gì xảy ra?

<details><summary>Xem lời giải</summary><p>Xoá `rg-toinoi-trips-dev-sea` và, nếu dữ liệu dev có thể dựng lại từ seed, cả `rg-toinoi-data-dev-sea`. Giữ `rg-toinoi-monitor-dev-sea` để còn log. Nếu mọi thứ chung một RG, xoá RG là mất cả log, và bạn phải xoá từng resource thủ công, dễ sót hoặc xoá nhầm. Vòng đời khác nhau thì nên ở RG khác nhau.</p></details>

## Học tiếp từ đây

Bạn đã vẽ được cây tenant, management group, subscription, resource group cho Tới Nơi, biết quyền và policy kế thừa còn tag thì không, và biết RG gom theo vòng đời chứ không cô lập mạng. Bài tiếp theo chuyển từ sơ đồ sang tay làm: [bài 3](/articles/azure-portal-cli-cloud-shell-resource-group-dau-tien) dùng Azure Portal, Azure CLI và Cloud Shell để tạo resource group đầu tiên. Nếu cần ôn nền tảng, quay lại [bài 1](/articles/azure-la-gi-cloud-region-trach-nhiem-chia-se).

## Tài liệu gốc

- [Azure Resource Manager documentation](https://learn.microsoft.com/en-us/azure/azure-resource-manager/)
- [Azure management groups](https://learn.microsoft.com/en-us/azure/governance/management-groups/)
- [Azure role-based access control](https://learn.microsoft.com/en-us/azure/role-based-access-control/)
- [Azure Policy](https://learn.microsoft.com/en-us/azure/governance/policy/)
- [Microsoft Entra documentation](https://learn.microsoft.com/en-us/entra/)
- [Cloud Adoption Framework for Azure](https://learn.microsoft.com/en-us/azure/cloud-adoption-framework/)

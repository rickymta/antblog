---
slug: azure-landing-zone-subscription-vending
title: "Landing zone và subscription vending cho nhiều môi trường"
summary: "Bài 49: thu nhỏ Azure Landing Zone cho Tới Nơi thành mg-toinoi với platform và workloads, viết Bicep ở scope management group gán policy, rồi dựng module vending tạo resource group, budget và RBAC cho một môi trường mới bằng một pull request."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, governance, landing-zone, bicep, management-group]
language: vi
---

Đây là **bài 49** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 10/10 của phần).

[Bài 2](/articles/azure-tenant-subscription-resource-group) vẽ cây tenant, management group, subscription trên giấy. Bài này biến nó thành code: một cấu trúc landing zone vừa cỡ Tới Nơi, và một module để môi trường mới ra đời bằng pull request thay vì bằng một buổi click portal. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Có một subscription riêng cho từng môi trường thì dev đã tách khỏi prod, nên chuyện quản trị coi như xong?</p>
<details><summary>Xem đáp án</summary><p>Chưa. Subscription chỉ là ranh giới. Nếu mỗi subscription được dựng tay theo một kiểu, bạn có ba môi trường lệch nhau về policy, quyền và ngân sách. Việc cần làm là dựng chúng từ cùng một định nghĩa.</p></details>
</div>

## Landing zone là khung quản trị dựng sẵn trước khi workload tới

Cloud Adoption Framework là bộ hướng dẫn của Microsoft về cách một tổ chức đưa workload lên Azure. Azure Landing Zone là phần hiện thực hoá nó: một môi trường đã có sẵn management group, policy, mạng, danh tính và logging, để đội ứng dụng chỉ việc đặt workload vào.

Kiến trúc tham chiếu chia management group thành hai nhánh chính. **Platform** chứa các subscription dùng chung, thường tách ba vai trò: identity, management (Log Analytics trung tâm, automation) và connectivity (hub-spoke, DNS, firewall). **Landing zones** chứa subscription cho workload, thường tách `corp` (chỉ nối nội bộ qua mạng riêng) và `online` (hướng ra internet). Ngoài ra thường có thêm nhóm sandbox cho thử nghiệm và decommissioned cho subscription sắp xoá.

Lý do của hình dạng này là **policy gán ở management group, kế thừa xuống subscription**. Gán một lần ở trên thì mọi subscription mới đặt dưới đó tự mang luật: region được phép, tag bắt buộc, cấm public IP cho VM. Log Analytics trung tâm cho phép đội bảo mật hỏi một câu KQL xuyên mọi môi trường. Hub-spoke cho phép mọi spoke đi chung một chỗ kiểm soát lối ra, thay vì mỗi VNet tự xoay.

Điểm yếu là kích thước. Bản tham chiếu đầy đủ viết cho tổ chức có hàng chục đội, mạng lai on-premises và đội bảo mật riêng. Dựng nguyên bản đó cho một sản phẩm đặt xe và giao hàng sáu thành phố là trả giá vận hành cho thứ chưa cần.

## Tới Nơi chỉ cần một cây nhỏ với ba subscription workload và một platform

Tới Nơi chọn cây tối thiểu vẫn giữ đúng nguyên lý: policy ở trên, workload ở dưới, tách platform khỏi workload.

| Thành phần | Có trong bản của Tới Nơi | Lý do |
| --- | --- | --- |
| `mg-toinoi` | Có | Một chỗ gán policy chung: region, tag |
| `mg-platform` + `sub-toinoi-platform` | Có, một subscription gộp | Chứa Log Analytics trung tâm và registry dùng chung; identity do Entra ID lo |
| `mg-workloads` + 3 subscription | Có: dev, staging, prod | Quota, billing và quyền tách theo môi trường |
| Tách `corp` và `online` | Chưa | Mọi workload hiện ra public qua Front Door; tách khi có service nội bộ thuần |
| Hub-spoke và firewall | Chưa | Thêm khi có nhu cầu kiểm soát lối ra tập trung; peering vẫn nằm trong module vending để bật sau |

Cái giá của bản nhỏ là khi lớn lên bạn phải tách nhóm và di chuyển subscription, điều làm được nhưng cần kế hoạch. Đổi lại, bạn không vận hành mười MG chưa có việc làm. Nguyên tắc: thêm tầng khi có một luật thật sự khác nhau giữa hai nhóm, không thêm cho đẹp sơ đồ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Cây management group của Tới Nơi: mg-toinoi gán policy chung, nhánh platform một subscription và nhánh workloads ba subscription dev, staging, prod."><defs><marker id="az49-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="280" y="15" width="320" height="65" rx="10" class="f-pk-soft s-pk"/><text x="294" y="43" class="t-b">mg-toinoi</text><text x="294" y="66" class="t-small">Policy: region, tag bắt buộc</text><path d="M380 80 L170 110" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-0-a)"/><path d="M500 80 L610 110" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-0-a)"/><rect x="20" y="110" width="300" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="140" class="t-b">mg-platform</text><text x="34" y="164" class="t-mono">sub-toinoi-platform</text><text x="34" y="186" class="t-small">Log trung tâm, registry</text><rect x="360" y="110" width="500" height="60" rx="10" class="f-phys-soft s-phys"/><text x="374" y="136" class="t-b">mg-workloads</text><text x="374" y="158" class="t-small">Luật riêng cho workload</text><path d="M440 170 L440 225" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-0-a)"/><path d="M610 170 L610 225" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-0-a)"/><path d="M780 170 L780 225" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-0-a)"/><rect x="360" y="225" width="160" height="85" rx="10" class="f-pk-soft s-pk"/><text x="374" y="258" class="t-mono">sub-toinoi-dev</text><text x="374" y="284" class="t-small">Quyền rộng</text><rect x="530" y="225" width="160" height="85" rx="10" class="f-pk-soft s-pk"/><text x="544" y="258" class="t-mono">sub-toinoi-staging</text><text x="544" y="284" class="t-small">Giống prod</text><rect x="700" y="225" width="160" height="85" rx="10" class="f-warn-soft s-warn"/><text x="714" y="258" class="t-mono">sub-toinoi-prod</text><text x="714" y="284" class="t-small">Quyền chặt, PIM</text></svg>
</div>
<figcaption>Hình 1. Cây management group của Tới Nơi: mg-toinoi gán policy chung, nhánh platform một subscription và nhánh workloads ba subscription dev, staging, prod.</figcaption>
</figure>

## Lab: Bicep ở scope management group tạo MG và gán ba policy

File `mg-toinoi.bicep` có `targetScope = 'managementGroup'` và được deploy ở root management group. Nó tạo `mg-toinoi` (management group là resource ở scope tenant, nên khai báo `scope: tenant()`), rồi gọi module gán ba built-in policy lên chính MG đó: allowed locations cho resource, allowed locations cho resource group, và bắt buộc resource group có tag `env`. Xác nhận ID built-in bằng `az policy definition show --name <id>` trước khi tin. Template cũng chưa chứng minh bạn có quyền: tạo MG và gán policy ở gần root cần role cao ở scope đó.

```bicep
targetScope = 'managementGroup'

param parentMgName string = tenant().tenantId
param regions array = ['southeastasia', 'eastasia']

resource mg 'Microsoft.Management/managementGroups@2021-04-01' = {
  scope: tenant()
  name: 'mg-toinoi'
  properties: {
    displayName: 'Toi Noi'
    details: { parent: { id: tenantResourceId('Microsoft.Management/managementGroups', parentMgName) } }
  }
}

module policies 'mg-policies.bicep' = {
  name: 'toinoi-policies'
  scope: managementGroup(mg.name)
  params: { regions: regions }
}
```

Module `mg-policies.bicep` chạy ở scope `mg-toinoi`, nên assignment không cần thuộc tính `scope`.

```bicep
targetScope = 'managementGroup'
param regions array

var policies = [
  { key: 'loc', defId: 'e56962a6-4747-49cd-b67b-bf8b01975c4c', args: { listOfAllowedLocations: { value: regions } } }
  { key: 'rgloc', defId: 'e765b5de-1225-4ba3-bd56-1ac6695af988', args: { listOfAllowedLocations: { value: regions } } }
  { key: 'tag', defId: '96670d01-0a4d-4649-9c89-2d3abc0a5025', args: { tagName: { value: 'env' } } }
]

resource assign 'Microsoft.Authorization/policyAssignments@2024-04-01' = [for p in policies: {
  name: 'toinoi-${p.key}'
  properties: {
    displayName: 'Toi Noi ${p.key}'
    policyDefinitionId: tenantResourceId('Microsoft.Authorization/policyDefinitions', p.defId)
    parameters: p.args
    enforcementMode: 'DoNotEnforce'
  }
}]
```

Bài không có kết quả what-if thật vì cần tenant của bạn. Template có `targetScope = 'managementGroup'` nên dùng `az deployment mg`; `az deployment tenant what-if` chỉ dành cho template `targetScope = 'tenant'`. Lệnh đầu xem trước thay đổi, lệnh sau mới tạo thật.

```bash
az deployment mg what-if \
  --management-group-id <tenant-id> --location southeastasia \
  --template-file mg-toinoi.bicep

az deployment mg create \
  --management-group-id <tenant-id> --location southeastasia \
  --template-file mg-toinoi.bicep
```

Hai điểm khi đọc what-if. Thứ nhất, nó cho thấy MG sẽ được tạo; phần module nhắm vào một MG chưa tồn tại nên lần đầu có thể hiện không đủ, và what-if không dự đoán được policy sau này chặn resource nào. Thứ hai, MG mới tạo có thể cần một lúc để hiển thị nhất quán, nên nếu assignment lỗi ngay sau lần tạo đầu, chạy lại là cách thử đầu tiên. Cờ `DoNotEnforce` khiến policy chỉ đánh giá và báo compliance, chưa chặn gì. Đây là cách ra mắt policy an toàn: xem kết quả vài ngày, rồi đổi sang enforce. Built-in Allowed locations đã bỏ qua resource có location `global` (Front Door, action group), nhưng resource group chứa chúng vẫn phải ở region được phép.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Policy deny ở management group gần root ảnh hưởng mọi subscription dưới nó, kể cả những cái bạn quên. Gán ở chế độ enforce mà chưa xem compliance có thể chặn một deployment đang chạy của đội khác lúc nửa đêm. Hãy gán bằng DoNotEnforce, đọc kết quả, rồi mới bật chặn. Tương tự, xoá một management group chỉ làm được khi nó rỗng, còn xoá subscription là việc có hậu quả dữ liệu, không làm trong lab này.</p>
</div>

## Subscription vending: một pull request cho một môi trường mới

**Subscription vending** là việc cấp subscription như một sản phẩm tự phục vụ: đội xin, pipeline dựng, đội nhận về một subscription đã đúng chuẩn. Với Tới Nơi, "một pull request cho một môi trường mới" nghĩa là thêm một file tham số cho `sub-toinoi-<env>`, review nó như code, và pipeline lo phần còn lại.

Việc tạo chính subscription làm được bằng IaC thông qua resource alias của Microsoft.Subscription, nhưng cần quyền ở billing scope của hợp đồng thanh toán. Quyền này khác với quyền Azure RBAC thường và thường nằm với bộ phận tài chính, nên hãy kiểm tài liệu và hỏi người quản lý billing trước khi hứa với đội rằng "PR là có subscription". Phần sau khi subscription tồn tại thì hoàn toàn nằm trong tầm tay của đội platform, và đó là phần module dưới đây làm.

| Việc vending làm | Vì sao đưa vào module |
| --- | --- |
| Đặt subscription vào đúng management group | Kế thừa policy ngay từ phút đầu |
| Tạo resource group theo quy ước và tag `env` | Không có resource group lệch tên hoặc thiếu tag |
| Gán RBAC cho group, không cho người | Đổi người trong group, không sửa template |
| Tạo budget kèm cảnh báo | Chi phí môi trường mới có chủ từ ngày đầu |
| Peering tới hub (khi có hub) | Mạng đúng chuẩn, không mở thủ công |

Module dưới chạy ở scope subscription. Nó tạo ba resource group, một budget tháng cảnh báo khi chi thực đạt 80%, và chỉ gán Contributor cho group dev khi `env` là dev; staging và prod dùng quyền eligible qua PIM. Số tiền budget là tham số: tra Pricing Calculator và dữ liệu thực tế, đừng đoán. Module chưa kiểm tên group đúng object ID và chưa chứng minh email nhận cảnh báo.

```bicep
targetScope = 'subscription'

@allowed(['dev', 'staging', 'prod'])
param env string
param budgetAmount int
param startDate string
param alertEmail string
param devGroupObjectId string

resource rgs 'Microsoft.Resources/resourceGroups@2024-03-01' = [for w in ['trips', 'data', 'monitor']: {
  name: 'rg-toinoi-${w}-${env}-sea'
  location: 'southeastasia'
  tags: { env: env, workload: w }
}]

resource budget 'Microsoft.Consumption/budgets@2023-05-01' = {
  name: 'budget-toinoi-${env}'
  properties: {
    category: 'Cost'
    amount: budgetAmount
    timeGrain: 'Monthly'
    timePeriod: { startDate: startDate }
    notifications: {
      actual80: { enabled: true, operator: 'GreaterThan', threshold: 80, thresholdType: 'Actual', contactEmails: [alertEmail] }
    }
  }
}

resource dev 'Microsoft.Authorization/roleAssignments@2022-04-01' = if (env == 'dev') {
  name: guid(subscription().id, devGroupObjectId, 'contributor')
  properties: {
    principalId: devGroupObjectId
    principalType: 'Group'
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', 'b24988ac-6180-42a0-ab88-20f7382dd24c')
  }
}
```

Ngày bắt đầu budget phải là ngày đầu tháng. Chạy what-if trên đúng subscription trước, rồi mới tạo:

```bash
az deployment sub what-if --subscription <subscription-id> \
  --location southeastasia --template-file sub-baseline.bicep \
  --parameters env=dev budgetAmount=<amount> startDate=2026-12-01 \
               alertEmail=platform@toinoi.example devGroupObjectId=<group-object-id>
```

Đặt subscription vào management group nằm ngoài module scope subscription; trong lab dùng `az account management-group subscription add --name mg-toinoi --subscription <subscription-id>` (cây đầy đủ đặt vào `mg-workloads`). Cùng module dùng cho staging và prod, chỉ khác tham số. Đó là điều làm ba môi trường giống nhau có chủ đích: khác nhau ở tham số nằm trong Git, không ở cái ai đó đã click.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Luồng vending: PR đổi tham số, what-if và duyệt, tạo hoặc nhận subscription, đặt vào management group, module baseline tạo resource group, budget, RBAC, policy kế thừa từ MG."><defs><marker id="az49-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">1. Pull request</text><text x="34" y="90" class="t-small">File tham số mới</text><path d="M270 70 L300 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-1-a)"/><rect x="300" y="30" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="314" y="62" class="t-b">2. What-if và duyệt</text><text x="314" y="90" class="t-small">Review như code</text><path d="M550 70 L580 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-1-a)"/><rect x="580" y="30" width="280" height="80" rx="10" class="f-warn-soft s-warn"/><text x="594" y="62" class="t-b">3. Có subscription</text><text x="594" y="90" class="t-small">Cần quyền billing</text><path d="M720 110 L720 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-1-a)"/><rect x="580" y="170" width="280" height="80" rx="10" class="f-phys-soft s-phys"/><text x="594" y="202" class="t-b">4. Đặt vào mg-workloads</text><text x="594" y="230" class="t-small">Policy kế thừa xuống</text><path d="M580 210 L550 210" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-1-a)"/><rect x="300" y="170" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="314" y="202" class="t-b">5. Module baseline</text><text x="314" y="230" class="t-small">RG, budget, RBAC</text><path d="M300 210 L270 210" class="f-none s-phys" stroke-width="2" marker-end="url(#az49-1-a)"/><rect x="20" y="170" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="202" class="t-b">6. Đội nhận môi trường</text><text x="34" y="230" class="t-small">Đúng chuẩn từ ngày 1</text></svg>
</div>
<figcaption>Hình 2. Luồng vending: PR đổi tham số, what-if và duyệt, tạo hoặc nhận subscription, đặt vào management group, module baseline tạo resource group, budget, RBAC, policy kế thừa từ MG.</figcaption>
</figure>

## Danh tính: nhóm theo vai trò, PIM cho prod, break-glass cho ngày tồi tệ

Module trên gán quyền cho **group**, không cho từng người. Người vào hay ra đội thì đổi thành viên group trong Entra ID, template không đổi. Tới Nơi bắt đầu với vài group theo vai trò: `platform-admin` (đội dựng nền tảng, quyền rộng ở platform và management group), `trips-dev` (đội viết `trip-service`, Contributor ở dev), `trips-ops` (vận hành, Reader thường trực ở prod) và một group chỉ đọc chi phí cho người quản lý.

Với prod, Contributor thường trực là rủi ro: tài khoản bị chiếm là kẻ tấn công có quyền ghi. **Privileged Identity Management (PIM)** đổi quyền thường trực thành quyền đủ điều kiện (eligible): người dùng phải kích hoạt, nêu lý do, có thể cần duyệt, và quyền tự hết sau một khoảng. Chi tiết về role, scope và PIM nằm ở [bài về RBAC](/articles/azure-rbac-role-scope-assignment-pim); hãy kiểm yêu cầu license của PIM trước khi hứa với đội.

Cuối cùng là **break-glass account**: hai tài khoản chỉ nằm trong Entra ID (không đồng bộ từ nơi khác), có quyền cao nhất, dùng khi cơ chế đăng nhập thông thường hỏng, chẳng hạn một Conditional Access policy khoá nhầm toàn bộ admin. Theo khuyến nghị của Microsoft, chúng cần phương thức xác thực mạnh và chống phishing, thông tin lưu ngoại tuyến ở nơi an toàn, được thử định kỳ, và mọi lần đăng nhập đều phải kích hoạt alert, vì bình thường không ai được dùng chúng. Bạn đã có hạ tầng alert từ [bài 48](/articles/azure-alert-action-group-slo-workbook) để làm việc đó.

## Quản trị liên tục: dựng xong mới là một nửa

Landing zone mục ruỗng nếu không ai nhìn. Bốn việc định kỳ sau giữ nó đúng:

| Việc | Công cụ | Dấu hiệu xấu |
| --- | --- | --- |
| Policy compliance | Azure Policy, `az policy state summarize` | Số resource non-compliant tăng dần |
| Chi phí theo subscription | Cost Management, budget | Một môi trường vượt budget nhiều tháng liền |
| Tag governance | Policy bắt buộc tag, Resource Graph | Resource không có `env` hoặc không có chủ |
| Decommission | Quy trình có checklist | Subscription chạy tiếp dù đội đã giải tán |

Khối dưới đếm resource non-compliant theo từng assignment ở `mg-toinoi`. Nó cho bạn một con số để theo dõi theo thời gian, không cho biết resource nào cần sửa trước; muốn vậy hãy mở chi tiết từng assignment. Chi phí và budget đã nói ở [bài về Cost Management](/articles/azure-chi-phi-pricing-cost-management-budget), tag và lock ở [bài về Resource Manager](/articles/azure-resource-manager-resource-id-tag-lock).

```bash
az policy state summarize --management-group mg-toinoi \
  --query "policyAssignments[].{assignment:policyAssignmentId, nonCompliant:results.nonCompliantResources}" \
  -o table
```

**Decommission** là bước người ta hay quên. Khi một môi trường hết vai trò, hãy chuyển subscription vào nhóm decommissioned, gỡ quyền, xuất hoặc giữ log theo thời gian lưu trữ cần thiết, rồi mới huỷ subscription. Subscription bị huỷ thường có thời gian chờ trước khi xoá hẳn, hãy xem tài liệu hiện hành. Việc này rẻ khi làm có kế hoạch và đắt khi phát hiện sau một năm vẫn có VM chạy.

**Bài tập:** Policy "Require a tag on resource groups" ở `mg-toinoi` đang DoNotEnforce. Bạn đổi sang enforce; module vending cho staging vẫn chạy được, nhưng activity log báo một resource group bị từ chối dù bạn không tạo RG nào ngoài module. Đoán nguyên nhân và cách xác nhận.

<details><summary>Xem lời giải</summary><p>Ba resource group của module có `tags: { env: env }` nên qua được. Thứ bị chặn là resource group do Azure tự tạo, như `NetworkWatcherRG` khi có VNet đầu tiên hay resource group hạ tầng của một số dịch vụ managed, vốn không mang tag của bạn. Xác nhận bằng sự kiện deny trong activity log (nêu assignment và resource) và mục compliance. Cách xử lý: exemption cho các RG đó, hoặc dùng policy effect modify để thêm tag thay vì deny.</p></details>

## Học tiếp từ đây

Bạn vừa có một cây management group vừa cỡ cho Tới Nơi, một template Bicep tạo MG và gán ba policy, một module vending dựng resource group, budget và RBAC cho từng môi trường, cùng cách giữ nó đúng theo thời gian. Lab mô tả cách làm, kết quả what-if và compliance bạn cần tự kiểm trên tenant của mình.

Đây là bài cuối của Phần 5. Phần 6 là Kiến trúc nâng cao và vận hành, bắt đầu bằng [bài 50 về Well-Architected Framework](/articles/azure-well-architected-framework-5-tru-cot), nơi ta chấm điểm toàn bộ những gì đã dựng theo năm trụ cột. Nếu cần ôn việc kiểm trước khi deploy, xem lại [bài 40 về Bicep và what-if](/articles/azure-bicep-co-ban-module-parameter-what-if).

## Tài liệu gốc

- [Cloud Adoption Framework for Azure](https://learn.microsoft.com/en-us/azure/cloud-adoption-framework/)
- [Azure landing zones](https://learn.microsoft.com/en-us/azure/cloud-adoption-framework/ready/landing-zone/)
- [Azure management groups](https://learn.microsoft.com/en-us/azure/governance/management-groups/)
- [Azure Policy](https://learn.microsoft.com/en-us/azure/governance/policy/)
- [Bicep documentation](https://learn.microsoft.com/en-us/azure/azure-resource-manager/bicep/)
- [Microsoft Entra Privileged Identity Management](https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/)

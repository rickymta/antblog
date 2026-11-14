---
slug: azure-rbac-role-scope-assignment-pim
title: "Azure RBAC: role, scope, assignment và PIM"
summary: "Bài 12: đọc một role assignment gồm principal, role và scope, phân biệt control plane với data plane, viết custom role restart Container App của Tới Nơi và đưa Owner production sang dạng eligible qua PIM."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, rbac, pim, security]
language: vi
---

Đây là **bài 12** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 2/9 của phần).

Bài 11 cho ta biết ai đang gọi; bài này quyết định người đó được làm gì trên resource nào. Ta đi từ cấu trúc của một role assignment, qua các role hay gặp, đến custom role và PIM, rồi làm lab gán quyền cho group dev của Tới Nơi. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một kỹ sư có role Contributor trên Key Vault. Họ có đọc được giá trị secret trong vault không?</p>
<details><summary>Xem đáp án</summary><p>Với vault dùng mô hình quyền RBAC thì không. Contributor là quyền control plane (quản lý resource), còn đọc giá trị secret là quyền data plane, cần role như Key Vault Secrets User.</p></details>
</div>

## Một assignment gồm ba mảnh, và quyền được kế thừa xuống dưới

Mỗi role assignment ghép đúng ba thứ:

- **Security principal**: user, group, service principal hoặc managed identity nhận quyền.
- **Role definition**: tập quyền, gồm `Actions` và `NotActions` cho control plane, `DataActions` và `NotDataActions` cho data plane.
- **Scope**: phạm vi áp dụng, từ management group, subscription, resource group đến từng resource.

Assignment ở scope cha được **kế thừa** xuống mọi scope con. Gán Reader ở subscription nghĩa là đọc được mọi resource group trong đó, kể cả cái tạo sau này. Muốn quyền hẹp thì gán ở scope hẹp, không có cách «trừ bớt» ở scope con.

Hai điểm hay hiểu sai. Thứ nhất, quyền hiệu lực là **phép hợp** của mọi assignment áp lên principal: nhiều assignment chỉ cộng thêm, không bao giờ bớt đi. `NotActions` chỉ loại một quyền khỏi chính role đó, nó không phải lệnh cấm; nếu role khác cho quyền ấy thì principal vẫn có. Thứ hai, **deny assignment** mới là thứ thực sự chặn: Azure đánh giá deny trước, và deny thắng mọi allow. Nhưng bạn không tự tạo deny assignment trực tiếp được; chúng do nền tảng sinh ra, ví dụ khi bảo vệ resource của một managed application hoặc từ deny settings của deployment stack (bài 45). Vì vậy deny assignment không phải công cụ phân quyền hằng ngày; xem deny assignment đang có trên một scope ở mục Access control (IAM) của portal.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Phân cấp scope: Reader gán ở subscription dev, Contributor gán ở resource group trips-dev, cả hai kế thừa xuống resource."><defs><marker id="az12-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Management group</text><text x="34" y="123" class="t-small">Chính sách chung</text><path d="M200 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az12-0-a)"/><rect x="240" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="95" class="t-b">Subscription dev</text><text x="254" y="123" class="t-small">Reader: grp dev</text><path d="M420 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az12-0-a)"/><rect x="460" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">rg-toinoi-trips-dev</text><text x="474" y="123" class="t-small">Contributor: grp dev</text><path d="M640 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az12-0-a)"/><rect x="680" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="694" y="95" class="t-b">Resource</text><text x="694" y="123" class="t-small">Kế thừa cả hai</text></svg>
</div>
<figcaption>Hình 1. Phân cấp scope: Reader gán ở subscription dev, Contributor gán ở resource group trips-dev, cả hai kế thừa xuống resource.</figcaption>
</figure>

## Bốn role nền tảng và hai role data plane

Đa số nhu cầu hằng ngày nằm gọn trong vài role built-in:

| Role | Làm được | Không làm được | Dùng cho Tới Nơi |
| --- | --- | --- | --- |
| Owner | Toàn quyền quản lý resource và gán role | Không có `DataActions`: không tự đọc secret hay blob, dù tự gán được role data | Chỉ eligible qua PIM ở prod |
| Contributor | Tạo, sửa, xoá resource | Gán role cho người khác | Đội platform ở RG của workload |
| Reader | Xem resource và cấu hình | Mọi thao tác ghi | Dev xem toàn subscription dev |
| User Access Administrator | Quản lý role assignment | Không sửa resource khác | Người được uỷ quyền gán quyền |
| Storage Blob Data Reader | Đọc blob (data plane) | Ghi hay xoá blob | Service đọc ảnh chứng từ |
| Key Vault Secrets User | Đọc giá trị secret (data plane) | Tạo hay sửa secret | `trip-service` đọc connection string |

Hai role đầu dễ gây hiểu lầm nhất. Contributor có thể làm hầu như mọi thứ trừ chuyện gán quyền, vì quyền ghi `Microsoft.Authorization` bị loại khỏi nó. User Access Administrator thì ngược lại: không đụng được vào resource nhưng gán được quyền, tức có thể tự nâng mình thành Owner. Vì vậy hãy xem role này nguy hiểm ngang Owner.

## Control plane và data plane là hai cánh cổng khác nhau

**Control plane** là các thao tác qua Azure Resource Manager: tạo vault, đổi cấu hình, xoá account. **Data plane** là thao tác trên dữ liệu bên trong: đọc secret, đọc blob. Role Contributor khai báo `Actions` gồm mọi thứ nhưng **không có `DataActions`**, nên role này quản lý được vault mà không đọc được secret.

Từ đó có hai hệ quả bảo mật:

- Key Vault có hai mô hình quyền: RBAC và access policy cũ. Với access policy, quyền ghi vào vault đủ để Contributor tự thêm policy cho chính mình rồi đọc secret. Vì vậy ta chọn mô hình RBAC cho vault mới (chi tiết ở bài 14).
- Storage account có đường vòng: Contributor gọi được thao tác lấy account key, và key đó cho toàn quyền data plane qua shared key. Tắt shared key và dùng Entra khi có thể.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Contributor không có nghĩa «chỉ quản lý hạ tầng, không chạm dữ liệu». Nếu một resource cung cấp khoá truy cập qua thao tác control plane, người có Contributor vẫn lấy được dữ liệu. Khi nghi ngờ, đọc danh sách <code>Actions</code> của role và của resource provider tương ứng thay vì tin vào tên role.</p>
</div>

## Custom role khi built-in quá rộng

Giả sử đội on-call của Tới Nơi cần restart `ca-toinoi-trips-prod` khi service treo, nhưng không được sửa cấu hình hay xoá app. Contributor quá rộng, Reader thì quá hẹp. Đó là lúc viết custom role. Khối JSON dưới đây là một thiết kế khả dĩ; nó **không chứng minh** tên operation đúng với phiên bản provider hiện tại.

```json
{
  "Name": "Toi Noi Trips Restarter",
  "IsCustom": true,
  "Description": "Xem Container App và restart revision, không sửa cấu hình.",
  "Actions": [
    "Microsoft.App/containerApps/read",
    "Microsoft.App/containerApps/revisions/read",
    "Microsoft.App/containerApps/revisions/restart/action",
    "Microsoft.Resources/subscriptions/resourceGroups/read"
  ],
  "NotActions": [],
  "DataActions": [],
  "NotDataActions": [],
  "AssignableScopes": [
    "/subscriptions/<subscription-id>/resourceGroups/rg-toinoi-trips-prod-sea"
  ]
}
```

Trước khi tạo, hãy tra operation thật bằng `az provider operation show --namespace Microsoft.App` và tìm dòng `revisions/restart/action`. Rồi tạo và gán:

```bash
az role definition create --role-definition @restarter.json

az role assignment create \
  --assignee-object-id <oncall-group-object-id> \
  --assignee-principal-type Group \
  --role "Toi Noi Trips Restarter" \
  --scope /subscriptions/<subscription-id>/resourceGroups/rg-toinoi-trips-prod-sea
```

Người on-call sau đó restart bằng `az containerapp revision restart --name ca-toinoi-trips-prod --resource-group rg-toinoi-trips-prod-sea --revision <revision-name>`. `AssignableScopes` giới hạn nơi role này được gán, nên nó không lan sang RG khác. Cái giá của custom role: bạn phải tự bảo trì danh sách operation khi provider thêm thao tác mới, và role chỉ xem được log nếu bạn thêm quyền đọc tương ứng, phần mà đoạn JSON trên cố tình bỏ qua.

## PIM: quyền Owner chỉ tồn tại khi có người cần và có lý do

Quyền thường trực (permanent) là rủi ro nằm yên: tài khoản bị chiếm thì kẻ tấn công có ngay quyền Owner. **Privileged Identity Management (PIM)** đổi mô hình này thành **just-in-time**:

- **Eligible**: principal đủ điều kiện nhưng chưa có quyền thật.
- **Kích hoạt**: người dùng yêu cầu, nhập lý do, có thể phải xác thực MFA, và tuỳ cấu hình cần người phê duyệt.
- **Active có thời hạn**: assignment hiệu lực trong số giờ cho phép rồi tự thu hồi.

Với Tới Nơi, quy tắc là **Owner trên subscription prod chỉ ở dạng eligible**, thời hạn kích hoạt ngắn, bắt buộc có lý do và người duyệt. Mọi lần kích hoạt để lại dấu vết kiểm toán. Kèm theo là **access review**: định kỳ yêu cầu chủ group hoặc quản lý xác nhận ai còn cần quyền, và gỡ những assignment không còn ai cần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Luồng PIM: eligible, yêu cầu kích hoạt với lý do và MFA, phê duyệt, rồi active có thời hạn và tự thu hồi."><defs><marker id="az12-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Eligible</text><text x="34" y="123" class="t-small">Chưa có quyền thật</text><path d="M200 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az12-1-a)"/><rect x="240" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="95" class="t-b">Yêu cầu kích hoạt</text><text x="254" y="123" class="t-small">Lý do và MFA</text><path d="M420 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az12-1-a)"/><rect x="460" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">Phê duyệt</text><text x="474" y="123" class="t-small">Chấp thuận hoặc từ chối</text><path d="M640 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az12-1-a)"/><rect x="680" y="65" width="180" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">Active có thời hạn</text><text x="694" y="123" class="t-small">Hết giờ tự thu hồi</text></svg>
</div>
<figcaption>Hình 2. Luồng PIM: eligible, yêu cầu kích hoạt với lý do và MFA, phê duyệt, rồi active có thời hạn và tự thu hồi.</figcaption>
</figure>

PIM cần license Entra phù hợp, hãy kiểm tra tài liệu trước khi lên kế hoạch. Nó cũng không thay RBAC: về bản chất PIM tạo và xoá role assignment có thời hạn thay bạn, nên mọi khái niệm ở trên vẫn đúng. Sau khi kích hoạt, token cũ có thể chưa phản ánh quyền mới; đăng nhập lại nếu thao tác vẫn bị từ chối.

## Lab: Reader ở subscription dev, Contributor ở RG trips-dev

Lab giả định bạn đã có group `grp-toinoi-dev` (tạo theo cách của bài 11), một user thử nghiệm là thành viên của nó và resource group `rg-toinoi-trips-dev-sea`. Khối dưới gán hai role ở hai scope khác nhau.

```bash
GROUP_ID=$(az ad group show --group grp-toinoi-dev --query id -o tsv)

az role assignment create \
  --assignee-object-id "$GROUP_ID" \
  --assignee-principal-type Group \
  --role Reader \
  --scope /subscriptions/<subscription-id>

az role assignment create \
  --assignee-object-id "$GROUP_ID" \
  --assignee-principal-type Group \
  --role Contributor \
  --scope /subscriptions/<subscription-id>/resourceGroups/rg-toinoi-trips-dev-sea

az role assignment list --all --assignee "$GROUP_ID" --output table
```

Lệnh `list --all` liệt kê assignment trên toàn subscription, và bạn kỳ vọng thấy hai dòng với hai scope. Việc gán có thể mất vài phút mới có hiệu lực. Sau đó đăng nhập bằng user thử nghiệm và thử một thao tác ghi ngoài RG được phép:

```bash
az login --tenant <tenant-id>   # đăng nhập bằng user thử nghiệm

az group create --name rg-toinoi-denied-dev-sea --location southeastasia
az tag update --resource-id \
  /subscriptions/<subscription-id>/resourceGroups/rg-toinoi-trips-dev-sea \
  --operation Merge --tags lab=rbac
```

Kỳ vọng: lệnh đầu thất bại với lỗi `AuthorizationFailed`, vì Reader ở subscription không cho tạo resource group. Lệnh thứ hai thành công vì Contributor ở RG `trips-dev`. Dùng `az tag update --operation Merge` thay vì `az tag create`, vì `az tag create` thay toàn bộ tag hiện có của resource group, kể cả tag `cost-center` mà bài 15 bắt buộc. Đây là hành vi mong đợi theo tài liệu, không phải kết quả tôi đã chạy; nếu khác, kiểm tra assignment kế thừa bằng `az role assignment list --scope <scope> --include-inherited`. Role assignment không tính tiền, nhưng sau lab hãy gỡ chúng bằng `az role assignment delete --assignee "$GROUP_ID" --role Reader --scope /subscriptions/<subscription-id>` (và tương tự cho Contributor) để không để lại quyền thừa.

**Bài tập:** `grp-toinoi-dev` được Contributor ở `rg-toinoi-trips-dev-sea` và Reader ở subscription. Một admin thêm `NotActions` vào custom role gán cho nhóm khác để «cấm» xoá Container App, nhưng user vẫn xoá được. Vì sao, và cách chặn thật là gì?

<details><summary>Xem lời giải</summary><p>Quyền hiệu lực là hợp của mọi assignment, nên nếu Contributor của group dev cho phép xoá thì quyền đó vẫn còn dù một role khác loại nó bằng <code>NotActions</code>. Muốn chặn phải bỏ quyền xoá ở mọi role áp lên user, hoặc dùng cơ chế kiểm soát khác như resource lock CanNotDelete hay Azure Policy với effect DenyAction (bài 15). Deny assignment thì bạn không tạo trực tiếp được; cách gần nhất là deny settings của deployment stack.</p></details>

## Học tiếp từ đây

Bạn đã đọc được một assignment qua ba mảnh principal, role và scope, biết vì sao Contributor không đọc secret, có một custom role restart Container App và quy tắc Owner prod chỉ eligible. Quyền cho người đã rõ. Còn ứng dụng thì sao: `trip-service` cũng là một principal cần quyền, và [Bài 13](/articles/azure-managed-identity-workload-identity) cho nó danh tính managed identity để nhận role mà không cần mật khẩu nào.

## Tài liệu gốc

- [Azure role-based access control documentation](https://learn.microsoft.com/en-us/azure/role-based-access-control/)
- [Understand Azure role definitions](https://learn.microsoft.com/en-us/azure/role-based-access-control/role-definitions)
- [Understand scope for Azure RBAC](https://learn.microsoft.com/en-us/azure/role-based-access-control/scope-overview)
- [Azure built-in roles](https://learn.microsoft.com/en-us/azure/role-based-access-control/built-in-roles)
- [Azure custom roles](https://learn.microsoft.com/en-us/azure/role-based-access-control/custom-roles)
- [What is Privileged Identity Management?](https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/pim-configure)

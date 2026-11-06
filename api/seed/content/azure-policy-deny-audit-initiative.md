---
slug: azure-policy-deny-audit-initiative
title: "Azure Policy: deny, audit, initiative và compliance"
summary: "Bài 15: dựng guardrail toinoi-guardrails cho subscription dev của Tới Nơi bằng Azure Policy: hiểu effect Deny và Audit, viết custom policy bắt buộc tag cost-center, gom thành initiative rồi quan sát storage ở West US bị từ chối."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, policy, governance, security]
language: vi
---

Đây là **bài 15** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 5/9 của phần).

Bài này dựng bộ guardrail `toinoi-guardrails` cho subscription dev của Tới Nơi: viết một custom policy bắt buộc tag `cost-center`, gom bốn policy thành một initiative, gán xuống subscription rồi thử tạo storage ở West US để xem Azure từ chối. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một người có role Owner trên subscription có tạo được storage account ở West US khi subscription đã gán policy chỉ cho phép Southeast Asia và East Asia không?</p>
<details><summary>Xem đáp án</summary><p>Không. RBAC trả lời «ai được làm gì», còn Policy trả lời «thứ được tạo ra có hợp lệ không». Owner có quyền tạo resource nhưng request vẫn bị Policy chặn.</p></details>
</div>

## Policy chặn request ở control plane, không phải ở quyền của người gọi

Mọi thao tác tạo hoặc sửa resource đi qua Azure Resource Manager (control plane). Sau khi RBAC cho phép, Resource Manager đưa nội dung request cho Azure Policy đánh giá trước khi chuyển tới resource provider. Policy so sánh các thuộc tính của resource với điều kiện trong định nghĩa. Nếu khớp, effect của policy quyết định chuyện gì xảy ra.

Hệ quả có hai mặt. Thứ nhất, Policy không phụ thuộc người gọi: portal, `az`, Bicep hay pipeline CI đều bị đối xử như nhau. Thứ hai, Policy chủ yếu nhìn control plane. Nó biết storage account bật public access hay không, nhưng không biết ai đang đọc blob. Một số resource provider mode (Kubernetes, Key Vault) mở rộng phạm vi đánh giá, và đó là chủ đề riêng của từng dịch vụ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Request tạo resource đi qua Resource Manager, Azure Policy đánh giá rồi hoặc chặn bằng Deny, hoặc cho tạo và ghi non-compliant bằng Audit."><defs><marker id="az15-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="240" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">Request qua ARM</text><text x="34" y="98" class="t-small">Portal, az, Bicep, CI</text><path d="M260 82 L320 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az15-0-a)"/><rect x="320" y="40" width="240" height="85" rx="10" class="f-pk-soft s-pk"/><text x="334" y="70" class="t-b">Azure Policy</text><text x="334" y="98" class="t-small">Đánh giá if / then</text><path d="M560 82 L620 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az15-0-a)"/><rect x="620" y="40" width="240" height="85" rx="10" class="f-phys-soft s-phys"/><text x="634" y="70" class="t-b">Resource provider</text><text x="634" y="98" class="t-small">Tạo hoặc cập nhật</text><path d="M400 125 L400 175" class="f-none s-warn" stroke-width="2" marker-end="url(#az15-0-a)"/><rect x="320" y="175" width="240" height="85" rx="10" class="f-warn-soft s-warn"/><text x="334" y="205" class="t-b">Deny: bị từ chối</text><text x="334" y="233" class="t-small">RequestDisallowedByPolicy</text><path d="M480 125 L480 150 L740 150 L740 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az15-0-a)"/><rect x="620" y="175" width="240" height="85" rx="10" class="f-surface s-line"/><text x="634" y="205" class="t-b">Audit: vẫn tạo</text><text x="634" y="233" class="t-small">Ghi non-compliant</text></svg>
</div>
<figcaption>Hình 1. Policy đứng giữa Resource Manager và resource provider; Deny chặn, Audit chỉ ghi nhận.</figcaption>
</figure>

## Effect quyết định mức độ can thiệp

Một policy có phần `if` (điều kiện trên resource) và phần `then` (effect). Chọn effect là chọn mức can thiệp.

| Effect | Làm gì | Khi nào dùng |
| --- | --- | --- |
| Audit | Cho tạo, đánh dấu non-compliant | Giai đoạn thử, đo mức độ vi phạm |
| Deny | Từ chối request tạo hoặc sửa | Quy tắc đã chắc chắn, không có ngoại lệ ngầm |
| AuditIfNotExists | Báo khi resource liên quan không tồn tại | Ví dụ VM thiếu extension giám sát |
| DeployIfNotExists | Triển khai resource phụ nếu thiếu | Tự bật diagnostic settings |
| Modify | Thêm, sửa tag hoặc thuộc tính | Gán tag mặc định cho resource mới |

Hai điều hay bị hiểu nhầm. Deny chỉ có tác dụng với request mới: resource đã tồn tại trước khi gán policy không bị xoá, chỉ bị đánh dấu non-compliant. Và `DeployIfNotExists` với `Modify` cần một managed identity trên assignment kèm role đủ quyền, vì chính Policy phải thực hiện thay đổi. Với resource cũ, bạn cần một remediation task chạy thủ công; assignment không tự sửa chúng.

## Assignment kế thừa xuống dưới, exemption là lối thoát có kiểm soát

Định nghĩa policy chỉ là «luật trên giấy». Luật có hiệu lực khi bạn tạo assignment ở một scope: management group, subscription hoặc resource group. Mọi thứ bên dưới scope đó kế thừa assignment. Tới Nơi gán `toinoi-guardrails` ở subscription dev trước. Khi đã tin cậy, cùng định nghĩa đó được gán ở scope cao hơn cho staging và prod, thường với effect nghiêm hơn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Assignment ở management group hoặc subscription được kế thừa xuống resource group; exemption là ngoại lệ có hạn dùng."><defs><marker id="az15-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Management group</text><text x="34" y="123" class="t-small">Gán cho nhiều sub</text><path d="M205 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az15-1-a)"/><rect x="240" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="95" class="t-b">Subscription dev</text><text x="254" y="123" class="t-small">Initiative gán ở đây</text><path d="M425 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az15-1-a)"/><rect x="460" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">Resource group</text><text x="474" y="123" class="t-small">Kế thừa từ tầng trên</text><path d="M645 108 L680 108" class="f-none s-warn" stroke-width="2" marker-end="url(#az15-1-a)"/><rect x="680" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">Exemption</text><text x="694" y="123" class="t-small">Ngoại lệ có hạn dùng</text></svg>
</div>
<figcaption>Hình 2. Assignment đi từ trên xuống; ngoại lệ phải được khai báo rõ thay vì sửa luật.</figcaption>
</figure>

Khi một resource group có lý do chính đáng để lệch luật (ví dụ vùng thử nghiệm cần region khác), có hai lối. `--not-scopes` loại hẳn scope con khỏi assignment. Exemption tinh tế hơn: nó giữ assignment, đánh dấu resource hoặc scope được miễn, ghi lý do (waiver hoặc mitigated) và có thể đặt ngày hết hạn. Ưu tiên exemption có hạn, vì ngoại lệ không ai nhớ thường thành lỗ hổng vĩnh viễn.

## Built-in policy đủ cho guardrail đầu tiên của Tới Nơi

Trước khi viết custom, hãy xem built-in có sẵn chưa. Tên hiển thị có thể đổi theo thời gian, nên tra bằng `az policy definition list` và đọc mô tả thay vì chép ID từ bài viết.

| Mục tiêu của Tới Nơi | Hướng dùng | Ghi chú |
| --- | --- | --- |
| Chỉ chạy ở Southeast Asia và East Asia | «Allowed locations», tham số `listOfAllowedLocations` | Effect cố định là Deny; bỏ qua location `global` và không áp lên resource group (có policy riêng «Allowed locations for resource groups») |
| Bắt buộc tag `cost-center` | Custom policy (bên dưới) | Có built-in yêu cầu tag, nhưng custom cho phép chạy Audit rồi mới Deny với cùng một định nghĩa |
| Cấm public access của storage | «Storage account public access should be disallowed» | Kiểm `allowBlobPublicAccess` (truy cập blob ẩn danh), không phải public network access; có tham số effect |
| Bắt buộc TLS 1.2 | «Storage accounts should have the specified minimum TLS version» | Tham số `minimumTlsVersion` (giá trị như `TLS1_2`) và effect |
| Cấm public network cho PostgreSQL | Policy về public network access của PostgreSQL flexible server (tra tên hiện hành) | Chỉ bật Deny sau khi có private access ở bài 17 |

Dòng cuối là ví dụ điển hình của việc đi quá nhanh. Nếu bạn Deny public network cho `psql-toinoi-dev-sea` mà chưa có private endpoint hoặc VNet integration, `trip-service` không còn đường kết nối. Audit trước, sửa hạ tầng, rồi mới Deny.

## Custom policy: if/then trên field, thử bằng Audit trước

Policy đọc thuộc tính resource qua `field`. Field có thể là thuộc tính chung (`type`, `location`, `tags['cost-center']`) hoặc một alias, tên chuẩn hoá cho thuộc tính riêng của từng resource type, ví dụ `Microsoft.Storage/storageAccounts/allowBlobPublicAccess`. Muốn biết alias nào có thể dùng, hỏi resource provider bằng `az provider show --namespace Microsoft.Storage --expand "resourceTypes/aliases"`.

Policy bắt buộc tag của Tới Nơi chỉ cần một điều kiện: tag không tồn tại thì áp effect. Effect được tham số hoá để cùng một định nghĩa chạy Audit ở dev và Deny ở prod. Mode `Indexed` giới hạn đánh giá ở resource type hỗ trợ tag và location; resource group và subscription không nằm trong đó, nên policy này không bắt tag trên chính resource group. Khối đầu là phần tham số:

```json
{
  "effect": {
    "type": "String",
    "metadata": {
      "displayName": "Effect",
      "description": "Audit hoac Deny khi thieu tag cost-center"
    },
    "allowedValues": ["Audit", "Deny", "Disabled"],
    "defaultValue": "Audit"
  }
}
```

Lưu thành `params.json`. Khối tiếp theo là luật, lưu thành `rule.json`. Hai khối này chỉ chứng minh cú pháp đúng với CLI; chúng không chứng minh policy cover được mọi resource type, vì loại không hỗ trợ tag nằm ngoài mode `Indexed`.

```json
{
  "if": {
    "field": "tags['cost-center']",
    "exists": "false"
  },
  "then": {
    "effect": "[parameters('effect')]"
  }
}
```

Tạo định nghĩa ở subscription dev bằng CLI. `--rules` và `--params` nhận đường dẫn file hoặc chuỗi JSON:

```bash
az policy definition create \
  --name require-cost-center-tag \
  --display-name "Toi Noi: bat buoc tag cost-center" \
  --description "Audit hoac Deny resource thieu tag cost-center" \
  --mode Indexed \
  --rules rule.json \
  --params params.json
```

Quy trình an toàn cho mọi custom policy là Audit trước, Deny sau. Gán với effect Audit, đọc danh sách non-compliant, sửa các resource hợp lệ hoặc tạo exemption, rồi mới đổi tham số sang Deny. Nếu bỏ qua bước này, bạn sẽ phát hiện luật sai khi pipeline deploy của đồng đội bị chặn lúc nửa đêm.

## Initiative gom policy lại, compliance cho thấy bức tranh

Gán từng policy một thì nhanh chóng thành chuỗi assignment khó quản lý. Initiative (policy set) gom nhiều policy thành một đơn vị, có tham số riêng và một điểm compliance chung. Microsoft cloud security benchmark là một initiative lớn như vậy; thường nó đã được gán sẵn ở subscription dùng Defender for Cloud (bài 16). `toinoi-guardrails` là initiative nhỏ của riêng Tới Nơi, gồm bốn policy: Allowed locations, custom tag `cost-center`, storage không public access, storage tối thiểu TLS 1.2.

Khối dưới tra ID built-in theo tên hiển thị rồi dựng `initiative.json`. Nó không kiểm tra tên hiển thị còn đúng hay không: nếu biến rỗng, hãy `az policy definition list` và tìm lại. Nó cũng không kiểm `allowedValues` của tham số effect trong từng built-in; đọc bằng `az policy definition show --name <id> --query parameters` trước khi truyền chung một giá trị `effect` cho nhiều policy.

```bash
LOC=$(az policy definition list \
  --query "[?displayName=='Allowed locations'].name | [0]" -o tsv)
PUB=$(az policy definition list \
  --query "[?displayName=='Storage account public access should be disallowed'].name | [0]" -o tsv)
TLS=$(az policy definition list \
  --query "[?displayName=='Storage accounts should have the specified minimum TLS version'].name | [0]" -o tsv)

cat > initiative.json <<EOF
[
  { "policyDefinitionReferenceId": "locations",
    "policyDefinitionId": "/providers/Microsoft.Authorization/policyDefinitions/$LOC",
    "parameters": { "listOfAllowedLocations": { "value": "[parameters('allowedLocations')]" } } },
  { "policyDefinitionReferenceId": "cost-center",
    "policyDefinitionId": "/subscriptions/<subscription-id>/providers/Microsoft.Authorization/policyDefinitions/require-cost-center-tag",
    "parameters": { "effect": { "value": "[parameters('effect')]" } } },
  { "policyDefinitionReferenceId": "storage-public",
    "policyDefinitionId": "/providers/Microsoft.Authorization/policyDefinitions/$PUB",
    "parameters": { "effect": { "value": "[parameters('effect')]" } } },
  { "policyDefinitionReferenceId": "storage-tls",
    "policyDefinitionId": "/providers/Microsoft.Authorization/policyDefinitions/$TLS",
    "parameters": { "effect": { "value": "[parameters('effect')]" },
                    "minimumTlsVersion": { "value": "TLS1_2" } } }
]
EOF

az policy set-definition create \
  --name toinoi-guardrails \
  --display-name "Toi Noi guardrails" \
  --definitions initiative.json \
  --params '{"effect":{"type":"String","defaultValue":"Audit"},"allowedLocations":{"type":"Array"}}'
```

Compliance dashboard (Policy, mục Compliance trong portal) hiển thị tỷ lệ resource tuân thủ theo assignment, rồi đi sâu xuống từng policy và từng resource. Đánh giá cho resource đã có chạy theo chu kỳ, và với assignment mới có thể mất khá lâu mới thấy kết quả; bạn có thể ép một lượt quét cho subscription hiện hành (hoặc một resource group với `--resource-group`) bằng `az policy state trigger-scan`. Nếu một resource bị đánh dấu non-compliant, có ba hướng xử lý: sửa resource, tạo remediation task (với `DeployIfNotExists` hoặc `Modify`), hoặc tạo exemption.

## Lab: gán toinoi-guardrails và quan sát West US bị từ chối

Lab chạy trên subscription dev, với resource group `rg-toinoi-lab-dev-sea`. Đầu tiên gán initiative ở chế độ `DoNotEnforce`: Policy vẫn đánh giá và báo compliance nhưng không chặn request.

```bash
az policy assignment create \
  --name toinoi-guardrails-dev \
  --display-name "Toi Noi guardrails (dev)" \
  --policy-set-definition toinoi-guardrails \
  --scope /subscriptions/<subscription-id> \
  --enforcement-mode DoNotEnforce \
  --params '{"effect":{"value":"Audit"},"allowedLocations":{"value":["southeastasia","eastasia"]}}'

az policy state trigger-scan --no-wait
az policy state list \
  --filter "complianceState eq 'NonCompliant'" \
  --query "[].{resource:resourceId, policy:policyDefinitionReferenceId}" -o table
```

`--no-wait` trả về ngay trong khi lượt quét còn chạy, nên `az policy state list` lúc đó có thể chưa có kết quả; chờ rồi chạy lại. Đọc danh sách, xử lý resource hợp lệ rồi bật enforcement bằng `az policy assignment update --name toinoi-guardrails-dev --scope /subscriptions/<subscription-id> --enforcement-mode Default` (hoặc chạy lại đúng lệnh `create` ở trên với `--enforcement-mode Default`, vì `create` ghi đè assignment cùng tên). Sau đó tạo resource group và thử storage ở West US:

```bash
az group create \
  --name rg-toinoi-lab-dev-sea \
  --location southeastasia \
  --tags cost-center=lab

az storage account create \
  --name sttoinoilabdev \
  --resource-group rg-toinoi-lab-dev-sea \
  --location westus \
  --sku Standard_LRS \
  --tags cost-center=lab
```

Kỳ vọng: lệnh thứ hai thất bại với lỗi `RequestDisallowedByPolicy`, trong thông báo có tên assignment và policy vi phạm. Assignment mới hoặc vừa đổi enforcement cần một khoảng thời gian (có thể tới vài chục phút) mới có hiệu lực, nên nếu storage vẫn được tạo, hãy xoá nó, chờ rồi thử lại. Effect của Allowed locations cố định là Deny, nên lệnh bị chặn dù tham số `effect` của initiative đang là Audit. Đổi `--location` thành `southeastasia` thì request qua được phần Allowed locations. Bài này mô tả cách làm; kết quả cụ thể phụ thuộc subscription và các policy đã có sẵn của bạn.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Deny ở scope cao sẽ chặn cả pipeline deploy, không chỉ lập trình viên gõ tay. Đừng gán Deny cho management group chung khi chưa chạy Audit và chưa báo cho các team. Cuối lab, xoá theo thứ tự: <code>az policy assignment delete</code>, rồi <code>az policy set-definition delete --name toinoi-guardrails</code> và <code>az policy definition delete --name require-cost-center-tag</code>, cuối cùng <code>az group delete</code>, để guardrail thử nghiệm không ảnh hưởng việc khác.</p>
</div>

**Bài tập:** `toinoi-guardrails` đang Deny location ở dev. Một storage account tạo ở West US từ tuần trước có bị xoá không, và bạn xử lý nó thế nào?

<details><summary>Xem lời giải</summary><p>Không bị xoá. Deny chỉ chặn request tạo hoặc sửa mới; resource cũ chỉ hiện non-compliant trong dashboard. Region của storage account không đổi tại chỗ, nên bạn phải tạo lại ở Southeast Asia rồi chuyển dữ liệu, hoặc tạo exemption có ngày hết hạn nếu cần giữ tạm.</p></details>

## Học tiếp từ đây

Bạn vừa có một guardrail đầy đủ vòng đời: custom policy với effect tham số hoá, initiative gom bốn policy, assignment ở scope subscription, và cách đi từ Audit tới Deny mà không phá pipeline. Bài tiếp theo là [Bài 16](/articles/azure-defender-for-cloud-secure-score): Defender for Cloud dùng cùng cơ chế Policy, qua initiative Microsoft cloud security benchmark, để chấm Secure Score và đưa ra recommendation, nhưng phần phát hiện đe doạ thì có phí. Nếu muốn nối với danh tính workload, đọc lại [bài 14 về Key Vault](/articles/azure-key-vault-secret-key-certificate).

## Tài liệu gốc

- [Azure Policy overview](https://learn.microsoft.com/en-us/azure/governance/policy/overview)
- [Azure Policy effects](https://learn.microsoft.com/en-us/azure/governance/policy/concepts/effect-basics)
- [Tutorial: create a custom policy definition](https://learn.microsoft.com/en-us/azure/governance/policy/tutorials/create-custom-policy-definition)
- [Remediate non-compliant resources](https://learn.microsoft.com/en-us/azure/governance/policy/how-to/remediate-resources)
- [Policy exemption structure](https://learn.microsoft.com/en-us/azure/governance/policy/concepts/exemption-structure)
- [az policy (Azure CLI)](https://learn.microsoft.com/en-us/cli/azure/policy)

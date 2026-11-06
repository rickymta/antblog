---
slug: azure-bicep-co-ban-module-parameter-what-if
title: "Bicep: resource, module, parameter và what-if"
summary: "Bài 40: tách main.bicep của trip-service thành module network và PostgreSQL cho môi trường dev Tới Nơi, tham số hoá bằng bicepparam, rồi dùng what-if để đọc trước thay đổi khi đổi SKU của psql-toinoi-dev-sea."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, bicep, iac, postgresql, networking]
language: vi
---

Đây là **bài 40** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 1/10 của phần).

Bài này dựng một bộ Bicep nhỏ gồm module network, module PostgreSQL và `main.bicep` chọn cấu hình theo môi trường, rồi chạy `what-if` trước khi deploy. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

Phần 5 gồm 10 bài và đi theo một đường: định nghĩa hạ tầng bằng code (bài 40 và 41), cho pipeline đăng nhập không cần mật khẩu (bài 42), build và deploy có kiểm soát (bài 43 đến 46), quan sát và cảnh báo (bài 47, 48), rồi landing zone cho nhiều môi trường (bài 49). Mọi thứ phía sau đều giả định hạ tầng đã nằm trong file, nên bài này đi trước.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn xoá một resource khỏi file Bicep rồi chạy lại <code>az deployment group create</code>. Resource đó có bị xoá trên Azure không?</p>
<details><summary>Xem đáp án</summary><p>Mặc định thì không. Deployment ở chế độ Incremental chỉ tạo hoặc cập nhật những gì có trong template, còn resource thừa được để yên. Chỉ chế độ Complete mới xoá resource không có trong template, và đó là chế độ dễ gây tai nạn.</p></details>
</div>

## Bicep chỉ là cú pháp dễ đọc hơn trên Azure Resource Manager

Bicep không có engine riêng. Khi bạn deploy, Azure CLI compile file `.bicep` thành ARM JSON rồi gửi cho Azure Resource Manager. Mọi hành vi quan trọng đều là hành vi của ARM: nó nhận một mô tả trạng thái mong muốn, so với trạng thái hiện có, rồi tạo hoặc sửa cho khớp. Đó là nghĩa của **declarative** (bạn mô tả đích, không viết từng bước) và **idempotent** (chạy lại cùng template cho cùng kết quả, không tạo bản sao).

Hệ quả thực tế: Bicep không nhớ gì giữa hai lần chạy. Không có file state như Terraform ở bài 41. ARM chính là nơi giữ trạng thái, nên "hiện trạng" luôn là cái đang có trên Azure, kể cả khi ai đó sửa tay trên portal. Lần deploy kế tiếp áp lại toàn bộ định nghĩa của resource, không cộng dồn từng thuộc tính: thuộc tính template không khai báo có thể bị đặt về giá trị mặc định. Vì vậy template phải mô tả trạng thái cuối đầy đủ của resource, và subnet nên khai báo trong `subnets` của VNet để deploy lại không làm mất subnet.

Hai quyết định đi kèm mỗi lần deploy là **scope** và **mode**.

| Thuộc tính | Giá trị | Ý nghĩa với Tới Nơi |
| --- | --- | --- |
| Scope | `resourceGroup` | Mặc định. Một workload trong một môi trường, ví dụ `rg-toinoi-trips-dev-sea` |
| Scope | `subscription` | Tạo resource group, gán policy hoặc role ở mức subscription (bài 49 dùng nhiều) |
| Scope | `managementGroup` | Áp chính sách cho nhiều subscription cùng lúc |
| Mode | Incremental | Chỉ động vào resource có trong template. Mặc định của `az deployment group create` |
| Mode | Complete | Xoá mọi resource trong resource group mà template không nhắc tới. Chỉ có ở scope resource group, đang được thay dần bằng deployment stack |

Scope khai báo bằng dòng `targetScope` (bỏ trống thì là `resourceGroup`) và phải khớp với lệnh bạn gọi (`az deployment group`, `az deployment sub`, `az deployment mg`). Gọi sai lệnh sẽ báo lỗi ngay, không làm hỏng gì.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Bicep và file tham số được compile thành ARM JSON, what-if so sánh với hiện trạng trước khi Resource Manager áp dụng."><defs><marker id="az40-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="195" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Bicep + bicepparam</text><text x="34" y="123" class="t-small">Code + giá trị theo env</text><path d="M215 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-0-a)"/><rect x="235" y="65" width="195" height="85" rx="10" class="f-phys-soft s-phys"/><text x="249" y="95" class="t-b">ARM JSON</text><text x="249" y="123" class="t-small">Kết quả compile</text><path d="M430 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-0-a)"/><rect x="450" y="65" width="195" height="85" rx="10" class="f-pk-soft s-pk"/><text x="464" y="95" class="t-b">what-if</text><text x="464" y="123" class="t-small">So hiện trạng, chưa ghi</text><path d="M645 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-0-a)"/><rect x="665" y="65" width="195" height="85" rx="10" class="f-warn-soft s-warn"/><text x="679" y="95" class="t-b">Deploy</text><text x="679" y="123" class="t-small">ARM thật sự thay đổi</text></svg>
</div>
<figcaption>Hình 1. Bicep và file tham số được compile thành ARM JSON, what-if so sánh với hiện trạng trước khi Resource Manager áp dụng.</figcaption>
</figure>

## Parameter, biến và symbolic name quyết định file có tái dùng được không

Một file Bicep gồm vài loại khai báo. **`param`** nhận giá trị từ ngoài vào. **`var`** tính giá trị từ param. **`resource`** khai báo một resource với **symbolic name** (tên dùng trong code, không phải tên trên Azure). **`output`** trả giá trị ra cho người gọi hoặc module cha. Symbolic name là chỗ Bicep thắng ARM JSON rõ nhất: bạn viết `vnet.id` thay vì ghép chuỗi `resourceId(...)`, và compiler tự suy ra thứ tự tạo.

Đoạn dưới là bản tóm tắt cú pháp, không phải một phần của lab. Nó cho thấy decorator, `existing`, vòng lặp và điều kiện trong một chỗ.

```bicep
@allowed(['dev', 'staging', 'prod'])
param env string

@description('Số replica tối đa, prod cao hơn dev.')
@minValue(1)
param maxReplicas int = 3

@secure()
param adminPassword string

var isProd = env == 'prod'
var services = ['pricing', 'booking', 'dispatch', 'trips']

resource kv 'Microsoft.KeyVault/vaults@2024-11-01' existing = {
  name: 'kv-toinoi-${env}-sea'
}

resource ids 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = [for s in services: {
  name: 'id-toinoi-${s}-${env}'
  location: resourceGroup().location
}]

resource lock 'Microsoft.Authorization/locks@2020-05-01' = if (isProd) {
  name: 'lock-kv'
  scope: kv
  properties: { level: 'CanNotDelete' }
}

output identityIds array = [for i in range(0, length(services)): ids[i].id]
```

Khối này cho thấy cú pháp, nhưng nó không chứng minh template deploy được: tên API version và quyền của bạn trên vault vẫn phải kiểm khi chạy. `@secure()` che giá trị khỏi log và lịch sử deployment, còn param thường được lưu trong lịch sử deployment của resource group, nên đừng nhét mật khẩu vào param không có `@secure()`.

Giá trị theo môi trường nên nằm trong file **`.bicepparam`**, mỗi môi trường một file. Dòng `using` trỏ về template, và Bicep kiểm kiểu ngay lúc compile.

```bicep
using 'main.bicep'

param env = 'dev'
param addressPrefix = '10.20.0.0/16'
param pgSku = 'Standard_B1ms'
param deployApp = false
```

File này khiến `staging.bicepparam` và `prod.bicepparam` chỉ khác vài dòng, trong khi code hạ tầng chỉ có một bản. Nó không chứa bí mật nào, và đó là chủ đích: mật khẩu admin sẽ đi qua Key Vault ở phần lab.

## Module tách theo vòng đời và dependency đi theo tham chiếu

Một file `main.bicep` dài vài trăm dòng thì không ai dám sửa. **Module** là một file Bicep khác được gọi như một resource, có `params` đầu vào và `outputs` đầu ra. Tới Nơi tách ba module theo vòng đời thay đổi: `network` (hiếm đổi, đổi sai là mất kết nối), `postgres` (đổi SKU và sao lưu, chứa dữ liệu) và `containerapp` (đổi mỗi lần release, chính là `main.bicep` của bài 36 loạt C#).

Dependency giữa các module không cần khai báo thủ công. Khi `postgres` nhận `network.outputs.pgSubnetId`, compiler thấy giá trị đó chỉ có sau khi `network` xong, và tự đặt thứ tự. `dependsOn` chỉ cần khi hai resource phụ thuộc nhau mà không có tham chiếu nào nối chúng. Lạm dụng `dependsOn` làm deployment chạy tuần tự không cần thiết và che mất quan hệ thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 2. main.bicep gọi ba module; output của network là đầu vào của postgres nên ARM tự sắp thứ tự."><defs><marker id="az40-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="118" width="220" height="74" rx="10" class="f-surface s-line"/><text x="34" y="148" class="t-b">main.bicep + dev.bicepparam</text><text x="34" y="174" class="t-small">Chọn module, truyền giá trị</text><path d="M240 140 L330 52" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-1-a)"/><path d="M240 155 L330 155" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-1-a)"/><path d="M240 170 L330 258" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-1-a)"/><rect x="330" y="15" width="300" height="74" rx="10" class="f-phys-soft s-phys"/><text x="344" y="45" class="t-b">modules/network.bicep</text><text x="344" y="71" class="t-small">VNet, subnet, DNS riêng</text><path d="M480 89 L480 118" class="f-none s-phys" stroke-width="2" marker-end="url(#az40-1-a)"/><text x="494" y="108" class="t-small">pgSubnetId, zoneId</text><rect x="330" y="118" width="300" height="74" rx="10" class="f-warn-soft s-warn"/><text x="344" y="148" class="t-b">modules/postgres.bicep</text><text x="344" y="174" class="t-small">Dữ liệu, SKU, sao lưu</text><rect x="330" y="221" width="300" height="74" rx="10" class="f-pk-soft s-pk"/><text x="344" y="251" class="t-b">modules/containerapp.bicep</text><text x="344" y="277" class="t-small">trip-service, đổi mỗi release</text></svg>
</div>
<figcaption>Hình 2. main.bicep gọi ba module; output của network là đầu vào của postgres nên ARM tự sắp thứ tự.</figcaption>
</figure>

Bạn không buộc phải tự viết mọi module. **Azure Verified Modules** (AVM) là thư viện module do Microsoft và cộng đồng duy trì, công bố trên registry công khai `br/public`. Gọi chúng như sau.

```bicep
module vnet 'br/public:avm/res/network/virtual-network:<phiên-bản>' = {
  name: 'vnet-avm'
  params: {
    name: 'vnet-toinoi-dev-sea'
    addressPrefixes: ['10.20.0.0/16']
  }
}
```

Hai điều cần cân nhắc. Một là phải ghim phiên bản cụ thể; tra số hiện hành trong tài liệu AVM, đừng dùng số bạn đoán. Hai là module dựng sẵn bao phủ rất nhiều tuỳ chọn, nên khi review bạn đọc một giao diện rộng thay vì ba mươi dòng của riêng mình. Với network của Tới Nơi, module tự viết ngắn thì dễ hiểu hơn. Với resource nhiều ràng buộc như Key Vault hay Log Analytics, AVM tiết kiệm công.

## Lab: viết module network và postgres cho môi trường dev

Lab giả định resource group `rg-toinoi-trips-dev-sea` và `kv-toinoi-dev-sea` đã có từ bài 14, trong vault có secret `pg-admin-password`, và vault bật `enabledForTemplateDeployment`. Người deploy cần quyền `Microsoft.KeyVault/vaults/deploy/action` trên vault (có sẵn trong Owner và Contributor), không phải quyền đọc secret qua data plane. Tạo thư mục `infra/bicep` với `modules/network.bicep` trước.

```bicep
param env string
param location string
param addressPrefix string

resource vnet 'Microsoft.Network/virtualNetworks@2024-05-01' = {
  name: 'vnet-toinoi-${env}-sea'
  location: location
  properties: {
    addressSpace: { addressPrefixes: [addressPrefix] }
    subnets: [
      { name: 'snet-pg', properties: { addressPrefix: cidrSubnet(addressPrefix, 24, 1), delegations: [{ name: 'pg', properties: { serviceName: 'Microsoft.DBforPostgreSQL/flexibleServers' } }] } }
      { name: 'snet-apps', properties: { addressPrefix: cidrSubnet(addressPrefix, 23, 1) } }
    ]
  }
}

resource zone 'Microsoft.Network/privateDnsZones@2024-06-01' = {
  name: 'toinoi-${env}.private.postgres.database.azure.com'
  location: 'global'
}

resource link 'Microsoft.Network/privateDnsZones/virtualNetworkLinks@2024-06-01' = {
  parent: zone
  name: 'link-${env}'
  location: 'global'
  properties: { virtualNetwork: { id: vnet.id }, registrationEnabled: false }
}

output pgSubnetId string = '${vnet.id}/subnets/snet-pg'
output zoneId string = zone.id
```

Module này tạo mạng riêng để PostgreSQL không có địa chỉ công khai: subnet `snet-pg` được delegate cho dịch vụ, còn zone DNS riêng được nối vào VNet để tên server phân giải ra IP nội bộ. Nó không chứng minh container app tới được database, vì environment từ bài 22 tạo không gắn VNet này. Subnet `snet-apps` chỉ là chỗ để dành cho environment tạo sau.

Module thứ hai là `modules/postgres.bicep`.

```bicep
param env string
param location string
@allowed(['Standard_B1ms', 'Standard_B2s', 'Standard_D2ds_v5'])
param skuName string
param subnetId string
param zoneId string
param adminLogin string
@secure()
param adminPassword string

var tier = startsWith(skuName, 'Standard_B') ? 'Burstable' : 'GeneralPurpose'

resource pg 'Microsoft.DBforPostgreSQL/flexibleServers@2024-08-01' = {
  name: 'psql-toinoi-${env}-sea'
  location: location
  sku: { name: skuName, tier: tier }
  properties: {
    version: '16'
    administratorLogin: adminLogin
    administratorLoginPassword: adminPassword
    storage: { storageSizeGB: 32 }
    backup: { backupRetentionDays: 7 }
    network: { delegatedSubnetResourceId: subnetId, privateDnsZoneArmResourceId: zoneId }
  }
}

output serverName string = pg.name
```

`@allowed` chặn gõ nhầm SKU ngay lúc compile. Biến `tier` suy ra từ tên SKU để hai giá trị không lệch nhau. Output chỉ trả tên server, không trả mật khẩu hay chuỗi kết nối. Danh sách SKU còn bán ở region thay đổi theo thời gian, nên kiểm bằng `az postgres flexible-server list-skus --location southeastasia`. Nếu bạn đã tạo tay `psql-toinoi-dev-sea` ở bài trước, hãy xoá hoặc đổi tên trước, vì Bicep sẽ cố sửa resource đó thay vì tạo mới.

Cuối cùng là `main.bicep`.

```bicep
targetScope = 'resourceGroup'

@allowed(['dev', 'staging', 'prod'])
param env string
param location string = resourceGroup().location
param addressPrefix string
param pgSku string
param pgAdminLogin string = 'toinoiadmin'
param deployApp bool = false

resource kv 'Microsoft.KeyVault/vaults@2024-11-01' existing = {
  name: 'kv-toinoi-${env}-sea'
}

module network 'modules/network.bicep' = {
  name: 'network'
  params: { env: env, location: location, addressPrefix: addressPrefix }
}

module postgres 'modules/postgres.bicep' = {
  name: 'postgres'
  params: {
    env: env
    location: location
    skuName: pgSku
    subnetId: network.outputs.pgSubnetId
    zoneId: network.outputs.zoneId
    adminLogin: pgAdminLogin
    adminPassword: kv.getSecret('pg-admin-password')
  }
}
```

`kv.getSecret(...)` đọc secret lúc deploy và chuyền thẳng vào param `@secure()`. Hàm này chỉ dùng được trong `params` của một module và chỉ cho param có `@secure()`; muốn lấy secret ngay trong file tham số thì dùng `az.getSecret(...)` trong `.bicepparam`. Nhờ vậy mật khẩu không đi qua dòng lệnh hay file tham số. Module thứ ba, `containerapp`, là file `cloud/main.bicep` của `trip-service` chép vào `modules/containerapp.bicep`. Bạn gọi nó như hai module trên với điều kiện `= if (deployApp)`, truyền `environmentResourceId` lấy từ `cae-toinoi-dev` qua một khai báo `existing`. Trong lab giữ `deployApp = false` để không chạm vào app từ bài 22. Khi nào đưa app vào luồng này là chuyện của bài 45.

## what-if đọc trước thay đổi, không kiểm chứng ứng dụng

Chạy lint, rồi what-if, rồi mới deploy. Ba lệnh này nằm trong cùng một thói quen.

```bash
RG=rg-toinoi-trips-dev-sea

az bicep lint --file main.bicep

az deployment group what-if \
  --resource-group $RG \
  --parameters dev.bicepparam

az deployment group create \
  --resource-group $RG \
  --name platform-dev \
  --parameters dev.bicepparam
```

Không cần `--template-file`, vì dòng `using` trong `dev.bicepparam` đã chỉ đúng template. Linter bắt lỗi kiểu: param không dùng (`no-unused-params`), API version cũ (`use-recent-api-versions`, mặc định tắt, bật trong `bicepconfig.json`), và quan trọng nhất là output chứa secret (quy tắc `outputs-should-not-contain-secrets`). Ai có quyền đọc deployment đều đọc được output, nên không bao giờ đưa mật khẩu, khoá hay chuỗi kết nối vào đó. Kỳ vọng ở lần chạy đầu: what-if liệt kê bốn resource sẽ được tạo (VNet, zone, link, server). Deployment PostgreSQL thường mất vài phút.

what-if cũng có giới hạn. Nó so template với hiện trạng nên không biết quyền của identity, không biết quota còn đủ, và đôi khi báo thay đổi giả ở các thuộc tính ARM tự điền giá trị mặc định. Đọc nó như bản xem trước, không như lời cam kết. Khi deployment thất bại, bước đầu là đọc lỗi từng bước, đừng đoán.

```bash
az deployment group show -g $RG -n platform-dev --query properties.error

az deployment operation group list -g $RG -n platform-dev \
  --query "[?properties.provisioningState=='Failed'].properties.statusMessage"
```

Mỗi module là một deployment lồng nhau mang tên trong thuộc tính `name` (ở đây `network`, `postgres`). Lệnh thứ hai trên `platform-dev` thường chỉ báo module nào hỏng; chạy lại nó với `-n postgres` để thấy nguyên nhân thật như quota, subnet chưa delegate hay thiếu quyền `deploy/action` trên vault.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Thêm <code>--mode Complete</code> vào <code>create</code> trên resource group chung sẽ xoá mọi resource mà template không nhắc tới, kể cả Key Vault hay database ai đó tạo tay. Hãy chạy <code>what-if</code> với cùng <code>--mode</code> trước khi dùng, và chỉ bật Complete cho resource group chỉ chứa thứ do template này quản. Microsoft khuyên dùng deployment stack (bài 45) thay cho Complete khi cần xoá resource. PostgreSQL xoá nhầm là mất dữ liệu; khoá <code>CanNotDelete</code> ở <a href="/articles/azure-resource-manager-resource-id-tag-lock">bài 4</a> vẫn là lớp chặn cuối.</p>
</div>

## Đổi SKU và xem what-if báo gì

Sửa `dev.bicepparam` từ `Standard_B1ms` sang `Standard_B2s`, rồi chạy lại `what-if`. Kỳ vọng, dạng đầu ra rút gọn (không phải kết quả chạy thật):

```text
~ Microsoft.DBforPostgreSQL/flexibleServers/psql-toinoi-dev-sea [2024-08-01]
    ~ sku.name: "Standard_B1ms" => "Standard_B2s"

Resource changes: 1 to modify, 3 no change.
```

Chỉ một thuộc tính đổi, còn VNet, zone và link chỉ hiện dòng `=` không đổi (ẩn được bằng `--exclude-change-types NoChange`). Đó là điều bạn muốn thấy: thay đổi nhỏ cho ra diff nhỏ. Nhưng what-if chỉ nói "sẽ sửa `sku.name`", không nói việc sửa đó có thể làm server khởi động lại và ngắt kết nối trong lúc đó. Hậu quả vận hành nằm ở tài liệu PostgreSQL Flexible Server và [bài 9](/articles/azure-database-postgresql-flexible-server-co-ban), không nằm trong diff. Nếu đổi sang SKU thuộc tier khác, diff cũng chỉ hiện hai dòng đổi, còn khả năng bị từ chối hay phải tạo lại thì chỉ biết khi deploy thật. Chạy thử ở dev trước khi sửa `staging.bicepparam`.

**Bài tập:** bạn chạy `what-if` thấy dòng `~ Microsoft.Network/virtualNetworks/vnet-toinoi-dev-sea` và `addressSpace` đổi từ `10.20.0.0/16` sang `10.30.0.0/16`. Nên deploy ngay không?

<details><summary>Xem lời giải</summary><p>Không nên. Subnet và DNS link phụ thuộc vào address space, còn subnet đã được delegate cho PostgreSQL đang chứa server. Đổi dải địa chỉ có thể bị ARM từ chối hoặc buộc tạo lại. Nếu muốn đổi, hãy lên kế hoạch tạo VNet mới và chuyển dữ liệu. Diff nhỏ về chữ không có nghĩa tác động nhỏ.</p></details>

## Học tiếp từ đây

Bạn đã tách hạ tầng Tới Nơi thành module network và PostgreSQL, tham số hoá theo môi trường bằng `.bicepparam`, và biết đọc what-if trước khi deploy. Bicep chỉ là một trong hai ngôn ngữ phổ biến để làm việc này trên Azure. [Bài 41](/articles/azure-terraform-azurerm-state-workspace) dựng lại cùng network và PostgreSQL dev bằng Terraform, với state lưu trong Storage, để bạn thấy khi nào chọn cái nào. Nếu cần ôn, xem lại [bài 17](/articles/azure-private-endpoint-private-dns-service-endpoint) về private DNS, [bài 39](/articles/azure-api-management-gateway-policy-versioning) về gateway phía trước, hoặc [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment) về `main.bicep` gốc.

## Tài liệu gốc

- [Bicep documentation](https://learn.microsoft.com/en-us/azure/azure-resource-manager/bicep/)
- [Bicep modules](https://learn.microsoft.com/en-us/azure/azure-resource-manager/bicep/modules)
- [Deployment what-if operation](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/deploy-what-if)
- [Deployment modes](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/deployment-modes)
- [Azure Verified Modules](https://learn.microsoft.com/en-us/community/content/azure-verified-modules)

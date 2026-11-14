---
slug: azure-terraform-azurerm-state-workspace
title: "Terraform trên Azure: azurerm, state trong Storage và workspace"
summary: "Bài 41: dựng lại network và psql-toinoi-dev-sea của Tới Nơi bằng Terraform với backend azurerm trên Storage, đăng nhập use_oidc, bảo vệ PostgreSQL prod bằng prevent_destroy và đọc drift khi ai đó sửa tag ngoài code."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, terraform, iac, state, postgresql]
language: vi
---

Đây là **bài 41** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 2/10 của phần).

Bài này dựng lại network và PostgreSQL dev của Tới Nơi bằng Terraform, với state lưu trong Azure Storage. Trọng tâm là chỗ Terraform khác Bicep: nó có file state, và file đó chứa dữ liệu nhạy cảm. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn và tài liệu provider tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Biến Terraform đã đánh dấu <code>sensitive = true</code> thì mật khẩu có còn nằm trong file state không?</p>
<details><summary>Xem đáp án</summary><p>Có. <code>sensitive</code> chỉ che giá trị khỏi output của <code>plan</code> và <code>apply</code>. State vẫn lưu giá trị ở dạng văn bản thường, nên quyền đọc state ngang quyền đọc secret.</p></details>
</div>

## Chọn Terraform vì đội và hệ sinh thái, không phải vì nó mạnh hơn

Bài 40 đã cho thấy Bicep đủ để dựng hạ tầng Azure. Terraform chỉ đáng chọn khi có lý do cụ thể. Lý do thứ nhất là đa cloud hoặc cần quản lý thứ không thuộc Azure bằng cùng một quy trình, ví dụ bản ghi DNS ở nhà cung cấp khác hay cấu hình của hệ thống bên thứ ba có provider. Lý do thứ hai là đội đã quen **HCL** và đã có pipeline, review, module nội bộ viết bằng nó. Lý do thứ ba là hệ sinh thái provider và module sẵn có.

Cái giá là bạn tự vận hành **state**. Bicep không có state vì ARM là nơi giữ hiện trạng; Terraform ghi lại những gì nó đã tạo vào một file, rồi dùng file đó để biết phải sửa gì. Hỏng, mất hoặc lộ file này là sự cố thật.

| Tiêu chí | Bicep | Terraform |
| --- | --- | --- |
| Hiện trạng nằm ở đâu | Azure Resource Manager | File state do bạn lưu và bảo vệ |
| Resource mới hoặc API version mới | Dùng được ngay khi Azure công bố | Phải đợi provider hỗ trợ, hoặc dùng azapi |
| Xem trước thay đổi | `what-if` | `terraform plan` |
| Phạm vi | Chỉ Azure | Nhiều provider, nhiều cloud |
| Khoá chống chạy đồng thời | ARM tự xử lý | Lease trên blob state |

Trong Terraform có hai provider cho Azure. **azurerm** bọc từng resource thành một khái niệm ổn định, có kiểm tra giá trị và tên thuộc tính riêng (ví dụ `sku_name = "B_Standard_B1ms"`, khác cặp `sku.name` và `tier` của Bicep). **azapi** gọi thẳng ARM REST API, nên dùng được ngay thuộc tính hoặc resource mới mà azurerm chưa bọc, đổi lại bạn viết gần với JSON của ARM hơn. Cách dùng hợp lý: azurerm cho phần lớn resource, azapi chỉ cho phần azurerm còn thiếu.

Một quy tắc cứng: **Bicep và Terraform không được trộn trên cùng một resource.** Mỗi công cụ tin rằng nó sở hữu resource đó. Bicep không biết state của Terraform, Terraform không biết deployment của Bicep, nên hai bên ghi đè thuộc tính của nhau, và `plan` hiện drift mãi không dứt. Tệ hơn, deployment Bicep ở chế độ Complete sẽ xoá resource Terraform tạo trong cùng resource group. Chia ranh giới theo resource hoặc theo tầng: ví dụ Bicep dựng storage chứa state và phần nền của subscription, Terraform dựng workload.

## State nằm trong Storage nên Storage là thứ phải bảo vệ nhất

Backend **azurerm** lưu state thành một blob trong container của Storage account. Nó cho bạn ba thứ không có khi để state trên máy cá nhân: nhiều người và pipeline dùng chung một nguồn sự thật, khoá chống hai lần `apply` chạy cùng lúc, và lịch sử để quay lại. Khoá thực hiện bằng **lease** trên blob: khi `plan` hoặc `apply` bắt đầu, Terraform giữ lease, người thứ hai thấy lỗi "state locked" và phải chờ. **Blob versioning** giữ các phiên bản cũ của file state, nên một lần ghi hỏng vẫn khôi phục được.

Quan trọng hơn cả là quyền. State chứa mọi thuộc tính resource, kể cả mật khẩu administrator của PostgreSQL và chuỗi kết nối. Mỗi phiên bản cũ cũng chứa chúng. Vì vậy storage state phải tách khỏi storage ứng dụng, tắt truy cập công khai, tắt shared key để buộc dùng Entra ID, và chỉ gán `Storage Blob Data Contributor` cho đúng identity của pipeline trên đúng container.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Pipeline đăng nhập OIDC, Terraform đọc ghi state trong Storage bằng lease và gọi ARM để tạo resource."><defs><marker id="az41-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="230" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="135" class="t-b">Pipeline</text><text x="34" y="163" class="t-small">Token OIDC, không secret</text><path d="M250 150 L320 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-0-a)"/><rect x="320" y="105" width="230" height="90" rx="10" class="f-pk-soft s-pk"/><text x="334" y="135" class="t-b">terraform plan/apply</text><text x="334" y="163" class="t-small">So code với state</text><path d="M550 130 L620 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-0-a)"/><path d="M550 170 L620 230" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-0-a)"/><rect x="620" y="25" width="240" height="90" rx="10" class="f-warn-soft s-warn"/><text x="634" y="55" class="t-b">Blob toinoi-dev.tfstate</text><text x="634" y="83" class="t-small">Lease + versions + secret</text><rect x="620" y="185" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="634" y="215" class="t-b">Resource trên Azure</text><text x="634" y="243" class="t-small">VNet, psql-toinoi-dev-sea</text></svg>
</div>
<figcaption>Hình 1. Pipeline đăng nhập OIDC, Terraform đọc ghi state trong Storage bằng lease và gọi ARM để tạo resource.</figcaption>
</figure>

Storage chứa state phải tồn tại trước khi Terraform chạy, nên không thể do chính Terraform đó tạo. Tạo nó bằng Azure CLI hoặc Bicep ở bài 40, một lần, ở subscription quản trị.

```bash
RG=rg-toinoi-tfstate-prod-sea
ST=sttoinoitfstateprod

az group create --name $RG --location southeastasia

az storage account create --name $ST --resource-group $RG \
  --sku Standard_ZRS --min-tls-version TLS1_2 \
  --allow-blob-public-access false --allow-shared-key-access false

az storage account blob-service-properties update \
  --account-name $ST --resource-group $RG --enable-versioning true

az storage container-rm create --name tfstate \
  --storage-account $ST --resource-group $RG
```

Container tạo bằng `container-rm` (qua ARM) vì shared key đã tắt, còn tài khoản Owner chưa có role data plane nên `az storage container create --auth-mode login` sẽ bị từ chối. Khối này dựng chỗ chứa và bật versioning, nhưng chưa cấp quyền cho pipeline và chưa đặt firewall hay private endpoint cho Storage. Hai việc đó vẫn phải làm: gán `Storage Blob Data Contributor` ở scope container cho identity của pipeline (bài 42 tạo identity này), và nếu pipeline chạy trong mạng riêng thì thêm private endpoint như bài 17.

## Cấu trúc: module dùng chung, mỗi môi trường một thư mục

Khối cấu hình dưới khai báo provider và backend. Backend không nhận biến, vì Terraform cần đọc nó trước khi đánh giá bất cứ biến nào. Do đó giá trị cố định nằm thẳng trong block, còn giá trị đổi theo ngữ cảnh (như `use_oidc`) truyền bằng `-backend-config` lúc `init`, hoặc đặt biến môi trường `ARM_USE_OIDC=true` mà cả backend lẫn provider cùng đọc. Provider azurerm 4.x còn bắt buộc có subscription, qua `subscription_id` hoặc `ARM_SUBSCRIPTION_ID`; nó không tự lấy subscription mặc định của `az login`.

```hcl
terraform {
  required_version = ">= 1.9"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }
  }

  backend "azurerm" {
    resource_group_name  = "rg-toinoi-tfstate-prod-sea"
    storage_account_name = "sttoinoitfstateprod"
    container_name       = "tfstate"
    key                  = "toinoi-dev.tfstate"
    use_azuread_auth     = true
  }
}

provider "azurerm" {
  features {}
  use_oidc = var.use_oidc
}
```

`key` là tên blob state, và mỗi môi trường phải có key riêng. Có hai cách tách môi trường. **Workspace** giữ một bộ code và một backend, mỗi workspace có blob state riêng. **Thư mục theo môi trường** (`envs/dev`, `envs/prod`) cho mỗi môi trường backend, tfvars và state của nó, còn code chung nằm trong `modules/`.

| Tiêu chí | Workspace | Thư mục `envs/<env>` |
| --- | --- | --- |
| Backend và credential | Dùng chung cho mọi môi trường | Tách riêng, prod có thể ở subscription khác |
| Nguy cơ nhầm môi trường | Cao, chỉ khác tên workspace đang chọn | Thấp, bạn đang đứng trong thư mục nào là rõ |
| Khác biệt giữa môi trường | Khó, `lifecycle` không đổi được theo biến | Dễ, mỗi thư mục có code riêng |
| Phù hợp | Nhiều bản sao giống hệt nhau, tạm thời | Dev, staging, prod của Tới Nơi |

Tới Nơi chọn thư mục theo môi trường. Lý do quyết định là prod phải có `prevent_destroy` còn dev thì không, và `lifecycle` chỉ nhận giá trị viết thẳng, không nhận biến, nên workspace không diễn tả được khác biệt đó.

Module `modules/network/main.tf` dưới đây tạo VNet, subnet delegate cho PostgreSQL và private DNS zone. Nó giống phần Bicep ở bài 40 về ý.

```hcl
variable "env" { type = string }
variable "location" { type = string }
variable "resource_group_name" { type = string }
variable "address_space" { type = string }

resource "azurerm_virtual_network" "this" {
  name                = "vnet-toinoi-${var.env}-sea"
  location            = var.location
  resource_group_name = var.resource_group_name
  address_space       = [var.address_space]
}

resource "azurerm_subnet" "pg" {
  name                 = "snet-pg"
  resource_group_name  = var.resource_group_name
  virtual_network_name = azurerm_virtual_network.this.name
  address_prefixes     = [cidrsubnet(var.address_space, 8, 1)]

  delegation {
    name = "pg"
    service_delegation {
      name    = "Microsoft.DBforPostgreSQL/flexibleServers"
      actions = ["Microsoft.Network/virtualNetworks/subnets/join/action"]
    }
  }
}

resource "azurerm_private_dns_zone" "pg" {
  name                = "toinoi-${var.env}.private.postgres.database.azure.com"
  resource_group_name = var.resource_group_name
}

resource "azurerm_private_dns_zone_virtual_network_link" "pg" {
  name                  = "link-${var.env}"
  resource_group_name   = var.resource_group_name
  private_dns_zone_name = azurerm_private_dns_zone.pg.name
  virtual_network_id    = azurerm_virtual_network.this.id
}
output "pg_subnet_id" { value = azurerm_subnet.pg.id }
output "dns_zone_id" { value = azurerm_private_dns_zone.pg.id }
```

Module này chưa kiểm tra gì về kết nối thật từ app tới database. Nó chỉ mô tả tài nguyên. Bài giữ server PostgreSQL ở root cho ngắn; khi tách `modules/postgres` và `modules/containerapp` theo cùng khuôn (biến vào, output ra), bạn cần block `moved` ở phần sau. Ngoài module tự viết, bạn có thể dùng module từ registry công khai, nhưng nên ghim phiên bản và đọc kỹ biến đầu vào, vì mỗi module ngoài là code bạn chạy với quyền của pipeline.

Trong `envs/dev/main.tf`, root gọi module và khai báo server PostgreSQL. Giá trị riêng của môi trường đặt trong `dev.tfvars`, còn mật khẩu admin chuyền qua biến môi trường `TF_VAR_pg_admin_password`, không viết vào file.

```hcl
data "azurerm_resource_group" "this" {
  name = "rg-toinoi-trips-dev-sea"
}

module "network" {
  source              = "../../modules/network"
  env                 = var.env
  location            = data.azurerm_resource_group.this.location
  resource_group_name = data.azurerm_resource_group.this.name
  address_space       = var.address_space
}

resource "azurerm_postgresql_flexible_server" "main" {
  name                          = "psql-toinoi-${var.env}-sea"
  resource_group_name           = data.azurerm_resource_group.this.name
  location                      = data.azurerm_resource_group.this.location
  version                       = "16"
  sku_name                      = var.pg_sku
  storage_mb                    = 32768
  delegated_subnet_id           = module.network.pg_subnet_id
  private_dns_zone_id           = module.network.dns_zone_id
  public_network_access_enabled = false
  administrator_login           = "toinoiadmin"
  administrator_password        = var.pg_admin_password
  tags                          = { env = var.env, workload = "trips" }

  depends_on = [module.network]
}
```

`depends_on` buộc server chờ cả DNS link, thứ mà server không tham chiếu trực tiếp. File `dev.tfvars` chỉ có ba dòng: `env = "dev"`, `address_space = "10.20.0.0/16"`, `pg_sku = "B_Standard_B1ms"`. Bạn cũng khai báo các biến này trong `variables.tf`, trong đó `pg_admin_password` có `sensitive = true` và `use_oidc` là `bool` mặc định `false`. Nhớ rằng `sensitive` không đưa mật khẩu ra khỏi state; tuỳ phiên bản provider có thể có thuộc tính write-only cho mật khẩu, hãy kiểm tra tài liệu azurerm trước khi dùng.

## plan, apply và đăng nhập OIDC thay cho client secret

`terraform plan` đọc state, hỏi Azure hiện trạng thật rồi in ra khác biệt với code. `terraform apply` thực hiện khác biệt đó. Trong CI, bạn không đưa client secret vào pipeline. Pipeline xin một token OIDC ngắn hạn và Terraform đổi nó lấy quyền truy cập, đó là `use_oidc`. Bài 42 trình bày cách tạo federated credential; ở đây chỉ cần biết phía Terraform cần gì.

```bash
export ARM_CLIENT_ID=<client-id>
export ARM_TENANT_ID=<tenant-id>
export ARM_SUBSCRIPTION_ID=<subscription-id>

terraform init -backend-config="use_oidc=true"
terraform plan -var-file=dev.tfvars -var="use_oidc=true" -out=dev.plan
```

Các biến `ARM_*` chỉ là định danh, không phải bí mật. Token OIDC do runner của pipeline cấp, bài này không chứng minh việc cấp đó. Với `-out`, file plan cũng chứa giá trị nhạy cảm, nên coi nó như state: không đưa vào artifact công khai. Để phát hiện drift định kỳ, chạy `terraform plan -detailed-exitcode` trong lịch; mã thoát 2 nghĩa là có khác biệt.

## Import, moved và prevent_destroy cho resource đã có

Hạ tầng thật thường có trước code. Khối `import` đưa một resource có sẵn vào state mà không tạo lại, và `moved` báo cho Terraform rằng resource đổi địa chỉ trong code chứ không phải bị xoá rồi tạo mới.

```hcl
import {
  to = azurerm_postgresql_flexible_server.main
  id = "/subscriptions/<subscription-id>/resourceGroups/rg-toinoi-trips-dev-sea/providers/Microsoft.DBforPostgreSQL/flexibleServers/psql-toinoi-dev-sea"
}

moved {
  from = azurerm_postgresql_flexible_server.main
  to   = module.database.azurerm_postgresql_flexible_server.main
}
```

Đây là hai ví dụ độc lập, không dùng cùng lúc cho một server. Sau khi import, chạy `plan` và sửa code cho tới khi diff gần như rỗng, vì import chỉ ghi vào state, không viết giúp bạn phần HCL. Quên `moved` khi tách resource vào module sẽ khiến `plan` đòi xoá server cũ và tạo server mới, tức mất dữ liệu.

Với PostgreSQL prod, thêm lớp chặn trong `envs/prod`:

```hcl
resource "azurerm_postgresql_flexible_server" "main" {
  # ... các thuộc tính như bản dev ...

  lifecycle {
    prevent_destroy = true
  }
}
```

Với `prevent_destroy`, mọi plan có bước xoá server này đều lỗi. Nó chỉ bảo vệ trước lệnh Terraform. Ai xoá tay trên portal hoặc bằng `az` vẫn xoá được, nên cần thêm khoá `CanNotDelete` của bài 4 và quyền RBAC chặt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Mỗi thư mục môi trường dùng chung module nhưng có file state riêng; prod có thêm prevent_destroy."><defs><marker id="az41-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="230" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="50" class="t-b">envs/dev</text><text x="34" y="76" class="t-small">dev.tfvars, không lock</text><rect x="20" y="200" width="230" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="230" class="t-b">envs/prod</text><text x="34" y="256" class="t-small">prevent_destroy</text><path d="M250 90 L340 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-1-a)"/><path d="M250 210 L340 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-1-a)"/><rect x="340" y="110" width="220" height="80" rx="10" class="f-phys-soft s-phys"/><text x="354" y="140" class="t-b">modules/network</text><text x="354" y="166" class="t-small">Code dùng chung</text><path d="M250 60 L630 60" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-1-a)"/><path d="M250 240 L630 240" class="f-none s-phys" stroke-width="2" marker-end="url(#az41-1-a)"/><rect x="630" y="20" width="230" height="80" rx="10" class="f-warn-soft s-warn"/><text x="644" y="50" class="t-b">toinoi-dev.tfstate</text><text x="644" y="76" class="t-small">Chứa secret của dev</text><rect x="630" y="200" width="230" height="80" rx="10" class="f-warn-soft s-warn"/><text x="644" y="230" class="t-b">toinoi-prod.tfstate</text><text x="644" y="256" class="t-small">Chứa secret của prod</text></svg>
</div>
<figcaption>Hình 2. Mỗi thư mục môi trường dùng chung module nhưng có file state riêng; prod có thêm prevent_destroy.</figcaption>
</figure>

## Lab: đổi tag ngoài Terraform rồi xem drift

Giả định `rg-toinoi-trips-dev-sea` và `kv-toinoi-dev-sea` có từ bài 14 và Storage state đã dựng. Chạy local bằng `az login`, không cần `use_oidc`, nhưng tài khoản của bạn cần `Storage Blob Data Contributor` trên container `tfstate` vì backend dùng `use_azuread_auth` (Owner của subscription không tự có quyền data plane). Nếu bạn đã tạo tay `psql-toinoi-dev-sea` ở bài 9, hãy xoá hoặc đổi tên, hoặc dùng `import` ở trên.

```bash
az login
export ARM_SUBSCRIPTION_ID=$(az account show --query id -o tsv)
export TF_VAR_pg_admin_password=$(az keyvault secret show \
  --vault-name kv-toinoi-dev-sea --name pg-admin-password \
  --query value -o tsv)

cd envs/dev
terraform init
terraform plan -var-file=dev.tfvars -out=dev.plan
terraform apply dev.plan
```

Kỳ vọng: `plan` liệt kê năm resource sẽ tạo (VNet, subnet, DNS zone, link, server) và `apply` tạo chúng, mất vài phút ở bước PostgreSQL. Lúc này một blob `toinoi-dev.tfstate` xuất hiện trong container. Tiếp theo cố ý tạo drift bằng cách thêm tag ngoài Terraform.

```bash
PG_ID=$(az postgres flexible-server show \
  -g rg-toinoi-trips-dev-sea -n psql-toinoi-dev-sea --query id -o tsv)

az tag update --resource-id $PG_ID \
  --operation Merge --tags costcenter=manual

terraform plan -var-file=dev.tfvars
```

Kỳ vọng: plan báo `1 to change`, với dòng `tags` có `- "costcenter" = "manual"` nghĩa là Terraform sẽ gỡ tag đó, vì code không khai báo. Đây không phải lỗi mà là bằng chứng drift được phát hiện. Nếu tag hợp lệ, thêm nó vào `tags` trong code; nếu không, chạy `apply` để về đúng. `terraform apply -refresh-only` chỉ ghi hiện trạng vào state; lần `plan` thường kế tiếp vẫn đòi gỡ tag vì code không khai báo. Nếu tag do hệ thống khác quản (ví dụ policy gắn tag), hãy dùng `lifecycle { ignore_changes = [tags["costcenter"]] }` để Terraform bỏ qua đúng khoá đó. Dọn lab bằng `terraform destroy` (dev không có `prevent_destroy`) và xoá state container khi không còn cần.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Chạy <code>terraform destroy</code> hoặc đổi tên resource trong code (không có <code>moved</code>) trên môi trường chung có thể xoá database thật. Luôn đọc <code>plan</code> tìm chữ <code>destroy</code> và <code>replace</code> trước khi <code>apply</code>. Và đừng bao giờ dùng <code>force-unlock</code> khi chưa chắc không còn lần chạy nào đang giữ lease, vì hai lần ghi đồng thời có thể làm hỏng state.</p>
</div>

**Bài tập:** `envs/prod` đã có `prevent_destroy = true` trên server PostgreSQL. Một đồng nghiệp xoá toàn bộ block `resource` của server khỏi code rồi chạy `terraform apply`. Chuyện gì xảy ra, và làm sao chặn?

<details><summary>Xem lời giải</summary><p>`prevent_destroy` là một phần của block resource, nên xoá cả block thì lớp chặn biến mất cùng nó, và plan sẽ đề nghị xoá server thật. Phải chặn ở tầng quy trình: bắt buộc review plan có chữ destroy, dùng khoá CanNotDelete ở Azure, và giới hạn quyền xoá của identity pipeline prod.</p></details>

## Học tiếp từ đây

Bạn đã dựng network và PostgreSQL dev bằng Terraform, hiểu state là tài sản cần bảo vệ như secret, biết chọn thư mục theo môi trường thay cho workspace khi môi trường khác nhau về lifecycle, và đọc được drift qua `plan`. Cả Bicep ở [bài 40](/articles/azure-bicep-co-ban-module-parameter-what-if) lẫn Terraform đều cần một pipeline đăng nhập vào Azure mà không giữ mật khẩu. [Bài 42](/articles/azure-github-actions-oidc-federated-credential) tạo federated credential và đưa cách đăng nhập `use_oidc` ở trên vào pipeline thật.

## Tài liệu gốc

- [Terraform on Azure](https://learn.microsoft.com/en-us/azure/developer/terraform/)
- [Store Terraform state in Azure Storage](https://learn.microsoft.com/en-us/azure/developer/terraform/store-state-in-azure-storage)
- [Authenticate Terraform to Azure](https://learn.microsoft.com/en-us/azure/developer/terraform/authenticate-to-azure)
- [Blob versioning overview](https://learn.microsoft.com/en-us/azure/storage/blobs/versioning-overview)
- [Azure Database for PostgreSQL](https://learn.microsoft.com/en-us/azure/postgresql/)

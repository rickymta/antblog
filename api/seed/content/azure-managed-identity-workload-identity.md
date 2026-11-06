---
slug: azure-managed-identity-workload-identity
title: "Managed identity: system-assigned, user-assigned và workload identity"
summary: "Bài 13: cho trip-service của Tới Nơi đọc Blob và Key Vault mà không giữ secret nào, so sánh system-assigned với user-assigned, dùng DefaultAzureCredential trong .NET, hiểu federated credential và gán Storage Blob Data Reader cho một VM."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, identity, managed-identity, security]
language: vi
---

Đây là **bài 13** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 3/9 của phần).

Bài này giải thích cách một workload như `trip-service` chứng minh mình là ai với Azure mà không cần mật khẩu, rồi dựng một lab nhỏ: tạo user-assigned identity, gán quyền đọc một container Blob và dùng nó từ một VM. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gán managed identity cho một VM thì VM đó có tự đọc được mọi storage account trong subscription không?</p>
<details><summary>Xem đáp án</summary><p>Không. Identity chỉ là danh tính. Quyền đến từ role assignment ở bài trước, và mặc định identity mới tạo chưa có quyền nào.</p></details>
</div>

## Secret trong config là thứ ta muốn xoá

`trip-service` cần đọc connection string trong Key Vault và giấy tờ tài xế trong container Blob `driver-docs`. Cách cũ là tạo một app registration, sinh client secret, dán vào biến môi trường. Secret đó hết hạn, bị copy vào pipeline, lọt vào log, và không ai nhớ ai đang giữ bản nào.

Managed identity bỏ bước đó. Về bản chất nó là một **service principal trong Entra ID do Azure tạo và quản lý**: Azure tự sinh và tự xoay credential, bạn không bao giờ nhìn thấy nó. Khi code cần token, nó gọi một endpoint nội bộ của nền tảng (IMDS trên VM, endpoint do runtime cung cấp trong Container Apps). Endpoint này chỉ truy cập được từ bên trong workload, nên kẻ bên ngoài không lấy được token bằng cách biết tên identity. Mặt trái: mọi tiến trình chạy trong VM hay container đều xin được token đó, nên một lỗ SSRF hay một thư viện độc trong workload là đủ để mượn quyền của identity. Token trả về có hạn ngắn, SDK tự cache và tự xin lại.

Điều cần nhớ: managed identity giải quyết bài toán **chứng minh danh tính** (authentication). Còn **được làm gì** (authorization) vẫn do RBAC quyết định, nên bài 12 là điều kiện đọc trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Workload xin token từ endpoint nội bộ, Entra ID cấp token ngắn hạn, token được gửi tới Blob hoặc Key Vault để kiểm role assignment."><defs><marker id="az13-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">trip-service</text><text x="34" y="123" class="t-small">Code gọi SDK, không có secret</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az13-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Endpoint nội bộ + Entra</text><text x="319" y="123" class="t-small">Cấp token ngắn hạn</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az13-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Blob / Key Vault</text><text x="604" y="123" class="t-small">Kiểm role assignment</text></svg>
</div>
<figcaption>Hình 1. Workload xin token từ endpoint nội bộ, Entra ID cấp token ngắn hạn, token được gửi tới Blob hoặc Key Vault để kiểm role assignment.</figcaption>
</figure>

## System-assigned gắn vào resource, user-assigned đứng riêng

Azure có hai loại managed identity. Khác biệt nằm ở vòng đời.

| Tiêu chí | System-assigned | User-assigned |
| --- | --- | --- |
| Tạo ra | Bật trên một resource | Tạo như resource độc lập |
| Vòng đời | Xoá resource thì identity mất | Tồn tại cho tới khi bạn xoá |
| Số resource dùng chung | Đúng một | Nhiều resource, nhiều revision |
| Gán quyền trước khi deploy | Không được, vì identity chưa tồn tại | Được, tạo identity và role assignment trước |
| Rủi ro | Tạo lại resource là mất quyền, phải gán lại | Một identity dùng chung nghĩa là quyền dùng chung |

Tới Nơi chọn **user-assigned**: `id-toinoi-trips-<env>`, mỗi môi trường một cái. Lý do thực tế là thứ tự triển khai. Container App cần pull image từ ACR và đọc secret Key Vault ngay khi revision đầu tiên khởi động. Nếu identity chỉ sinh ra cùng Container App thì role assignment đến sau, và revision đầu fail đúng ở bước nó cần quyền. Với user-assigned, pipeline tạo identity, gán quyền, rồi mới deploy app. Đây cũng là lý do Bicep ở bài 36 loạt C# nhận `identityResourceId` như một tham số có sẵn.

Cái giá là phạm vi quyền. Nếu bạn gắn `id-toinoi-trips-prod` cho cả `trip-service` lẫn một job dọn dữ liệu, mọi quyền của cái này thành quyền của cái kia. Quy ước: một identity cho một workload có cùng nhu cầu quyền, không gom tiện tay.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Xoá user-assigned identity không dọn role assignment của nó. Assignment trở thành bản ghi mồ côi hiện là «Unknown» trong danh sách, và nếu sau này có identity khác trùng tên thì nó vẫn là principal mới, không thừa hưởng quyền cũ. Gán quyền ở scope nhỏ nhất đủ dùng, ở đây là container `driver-docs` chứ không phải cả storage account.</p>
</div>

## Trong code .NET chỉ cần credential, không cần secret

Với thư viện Azure.Identity, `DefaultAzureCredential` thử lần lượt nhiều nguồn: biến môi trường, workload identity, managed identity, Visual Studio, Azure CLI và vài nguồn khác. Chạy local, nó rơi xuống phiên `az login` của bạn. Chạy trên Azure, nó dùng managed identity. Cùng một dòng code cho cả hai nơi là cái lợi, nhưng chuỗi fallback cũng là cái hại: khi lỗi, rất khó biết nguồn nào đã được thử và vì sao thất bại, và trên production có thể vô tình dùng một nguồn bạn không định dùng.

Với user-assigned identity, bạn phải nói rõ **client ID**. Nếu không, SDK không biết chọn identity nào khi resource gắn nhiều identity. Với `DefaultAzureCredential`, client ID đi qua `DefaultAzureCredentialOptions.ManagedIdentityClientId` hoặc biến môi trường `AZURE_CLIENT_ID`; với `ManagedIdentityCredential`, nó đi thẳng vào constructor như dưới.

```csharp
using Azure.Core;
using Azure.Identity;
using Azure.Storage.Blobs;

var clientId = builder.Configuration["Azure:ManagedIdentityClientId"];

// Local: dùng phiên az login. Azure: chỉ dùng đúng identity đã gắn.
TokenCredential credential = builder.Environment.IsDevelopment()
    ? new DefaultAzureCredential()
    : new ManagedIdentityCredential(
        ManagedIdentityId.FromUserAssignedClientId(clientId!));

var docs = new BlobContainerClient(
    new Uri("https://sttoinoidriverdocsdev.blob.core.windows.net/driver-docs"),
    credential);
```

Đoạn này cho thấy cách chọn credential, nhưng nó không chứng minh identity có role đúng; quyền chỉ lộ ra khi lần gọi đầu tiên trả 403 hoặc 200. Production dùng `ManagedIdentityCredential` hẹp vì lỗi đọc là lỗi identity, không bị che bởi chuỗi fallback. Client ID không phải secret, nên để trong cấu hình thường là được.

Ranh giới đã nêu ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment) vẫn đứng: identity này là danh tính của **workload**. Nó không thay JWT của khách hay tài xế. API vẫn xác thực người dùng bằng token riêng của họ, còn managed identity chỉ để service nói chuyện với Blob, Key Vault, ACR.

## Workload identity federation: đổi token bên ngoài lấy token Entra

Managed identity chỉ tồn tại với workload chạy trong Azure. Còn pipeline GitHub Actions, hay một pod trong AKS, thì sao? Cách cũ vẫn là client secret. Cách mới là **federated credential**: bạn nói trước với Entra «tin token do nơi kia phát hành, nếu nó khớp các điều kiện này», và workload đổi token của nó lấy token Entra, không secret nào được lưu.

Một federated credential gồm ba trường, và cả ba phải khớp:

| Trường | Ý nghĩa | Ví dụ |
| --- | --- | --- |
| issuer | Ai phát hành token ngoài | `https://token.actions.githubusercontent.com`, hoặc URL OIDC issuer của cluster AKS |
| subject | Workload cụ thể nào | `repo:<org>/<repo>:environment:prod`, hoặc `system:serviceaccount:<namespace>:<service-account>` |
| audience | Token này dành cho ai | `api://AzureADTokenExchange` |

Subject là phần quan trọng nhất về bảo mật. Entra so khớp subject chính xác, nên chọn subject nào là chọn ai được đổi token: đặt `repo:<org>/<repo>:pull_request` nghĩa là mọi workflow chạy từ pull request, kể cả pull request chưa ai review, đều đổi được token. Với Tới Nơi, subject của pipeline deploy prod nên gắn với environment `prod` của repo, nơi đã đặt approval. Federated credential có thể gắn vào app registration hoặc vào chính user-assigned identity; gắn vào identity là gọn khi bạn đã có `id-toinoi-trips-<env>`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Token OIDC từ GitHub Actions hoặc pod AKS được Entra kiểm issuer, subject, audience rồi đổi lấy token Entra để gọi Azure."><defs><marker id="az13-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Token OIDC ngoài</text><text x="34" y="123" class="t-small">GitHub Actions hoặc pod AKS</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az13-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Entra kiểm 3 trường</text><text x="319" y="123" class="t-small">issuer, subject, audience</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az13-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Token Entra</text><text x="604" y="123" class="t-small">Gọi Azure theo RBAC</text></svg>
</div>
<figcaption>Hình 2. Token OIDC từ GitHub Actions hoặc pod AKS được Entra kiểm issuer, subject, audience rồi đổi lấy token Entra để gọi Azure.</figcaption>
</figure>

Lệnh tạo federated credential trên identity cho pipeline môi trường `prod` có dạng sau. Nó chỉ đăng ký niềm tin; pipeline vẫn cần role assignment cho identity mới làm được gì.

```bash
az identity federated-credential create \
  --name fc-github-toinoi-prod \
  --identity-name id-toinoi-trips-prod \
  --resource-group rg-toinoi-trips-prod-sea \
  --issuer https://token.actions.githubusercontent.com \
  --subject "repo:<org>/<repo>:environment:prod" \
  --audiences api://AzureADTokenExchange
```

## Lab: identity đọc được một container, và chỉ container đó

Lab dùng VM `vm-toinoi-api-dev` đã tạo ở [bài 7](/articles/azure-virtual-machine-linux-chay-api), storage account `sttoinoidriverdocsdev` có container `driver-docs` từ [bài 8](/articles/azure-storage-account-blob-tier-sas), và giả định bạn có quyền tạo role assignment (Owner hoặc User Access Administrator ở scope tương ứng). Trước hết tạo identity và lấy các giá trị cần dùng.

```bash
RG=rg-toinoi-trips-dev-sea

az identity create --name id-toinoi-trips-dev \
  --resource-group $RG --location southeastasia

PRINCIPAL_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query principalId -o tsv)
CLIENT_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query clientId -o tsv)
IDENTITY_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query id -o tsv)

SA_ID=$(az storage account show -n sttoinoidriverdocsdev \
  -g rg-toinoi-storage-dev-sea --query id -o tsv)
SCOPE="$SA_ID/blobServices/default/containers/driver-docs"

az role assignment create \
  --assignee-object-id $PRINCIPAL_ID \
  --assignee-principal-type ServicePrincipal \
  --role "Storage Blob Data Reader" \
  --scope "$SCOPE"
```

Gán quyền theo `--assignee-object-id` kèm `ServicePrincipal` tránh việc CLI phải tra Graph ngay sau khi identity vừa sinh, vốn dễ lỗi do độ trễ replicate. Role assignment cũng có thể mất vài phút mới có hiệu lực. Tiếp theo gắn identity vào VM, SSH vào VM, rồi đăng nhập bằng chính identity đó.

```bash
# Trên máy của bạn
az vm identity assign -g rg-toinoi-api-dev-sea -n vm-toinoi-api-dev \
  --identities $IDENTITY_ID

# Trong VM (đã cài Azure CLI); <client-id> là giá trị $CLIENT_ID ở trên
az login --identity --client-id <client-id>
az storage blob list --account-name sttoinoidriverdocsdev \
  --container-name driver-docs --auth-mode login -o table
```

Kỳ vọng: lệnh `blob list` thành công. Hãy thử thêm hai phép âm tính để biết quyền đúng cỡ: tải lên một blob phải bị từ chối vì chỉ có role Reader, và đọc một container khác trong cùng storage account cũng bị từ chối vì assignment chỉ ở scope `driver-docs`. `--client-id` là cờ của Azure CLI hiện hành; bản CLI cũ dùng `--username <client-id>` cho cùng việc, nên kiểm `az login --help` nếu lệnh trên báo lỗi. `--auth-mode login` bắt buộc ở đây vì account bài 8 đã tắt shared key. Dọn lab theo đúng thứ tự: `az role assignment delete` cho assignment ở `$SCOPE`, `az vm identity remove` để gỡ identity khỏi VM, rồi mới `az identity delete`, để không để lại assignment mồ côi.

**Bài tập:** pipeline deploy dev và prod dùng chung một federated credential với subject `repo:<org>/<repo>:ref:refs/heads/main`. Vấn đề nằm ở đâu, và sửa thế nào?

<details><summary>Xem lời giải</summary><p>Mọi lần chạy trên nhánh main đều đổi được token, bất kể môi trường, nên không có approval gate nào chặn quyền prod. Tách hai identity (dev, prod), mỗi identity một federated credential có subject theo environment (`environment:dev`, `environment:prod`), và đặt required reviewer trên environment prod.</p></details>

## Học tiếp từ đây

Bạn đã có cách cho workload chứng minh danh tính mà không giữ secret: user-assigned identity cho dịch vụ chạy trên Azure, federated credential cho pipeline và pod ngoài Azure. Quyền vẫn do RBAC quyết định, nên luôn gán ở scope hẹp nhất. Bài kế tiếp, [bài 14](/articles/azure-key-vault-secret-key-certificate), dùng đúng identity này để đọc connection string trong Key Vault. Nếu cần ôn phần nền, xem lại [bài 12](/articles/azure-rbac-role-scope-assignment-pim) về role và scope, hoặc [bài 11](/articles/azure-entra-id-tenant-user-group-app-registration) về service principal.

## Tài liệu gốc

- [Managed identities for Azure resources](https://learn.microsoft.com/en-us/entra/identity/managed-identities-azure-resources/overview)
- [Workload identity federation](https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation)
- [Azure Identity client library for .NET](https://learn.microsoft.com/en-us/dotnet/api/overview/azure/identity-readme)
- [Azure Container Apps: managed identity](https://learn.microsoft.com/en-us/azure/container-apps/managed-identity)
- [Azure RBAC documentation](https://learn.microsoft.com/en-us/azure/role-based-access-control/)

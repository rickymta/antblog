---
slug: azure-developer-cli-azd-deployment-stacks-moi-truong
title: "Azure Developer CLI, deployment stacks và môi trường dev/staging/prod"
summary: "Bài 45: khởi tạo azd cho trip-service của Tới Nơi, dùng azd up cho env dev, chuyển infra thành deployment stack có deny setting DenyDelete, rồi bàn chiến lược môi trường và môi trường tạm cho từng pull request."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, azd, deployment-stacks, bicep, environments]
language: vi
---

Đây là **bài 45** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 6/10 của phần).

Bài này giới thiệu hai công cụ giải hai bài toán khác nhau. Azure Developer CLI (`azd`) rút ngắn vòng lặp dựng và chạy thử một môi trường. Deployment stack giữ vòng đời của các resource do Bicep tạo ra. Sau đó bài bàn cách tổ chức dev, staging, prod và dựng môi trường tạm cho pull request. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn xoá một resource khỏi file Bicep rồi chạy lại <code>az deployment group create</code>. Resource đó có bị xoá khỏi Azure không?</p>
<details><summary>Xem đáp án</summary><p>Không. Chế độ Incremental mặc định chỉ tạo hoặc cập nhật những gì có trong template, nên resource bị bỏ khỏi file vẫn còn nguyên và tiếp tục tốn tiền. Đây chính là khoảng trống mà deployment stack được tạo ra để lấp.</p></details>
</div>

## azd gói vòng lặp dev, không thay pipeline prod

`azd` nhìn một repository như một tập gồm hai thứ: code của các service và hạ tầng Bicep. File `azure.yaml` ở gốc khai báo mối nối giữa chúng, thư mục `infra/` chứa Bicep, và mỗi **azd environment** là một bộ giá trị (tên môi trường, subscription, location, output của lần provision) lưu cục bộ trong `.azure/<tên>/`. Một lệnh `azd up` gộp `package`, `provision` và `deploy`: dựng hạ tầng từ Bicep, build và push image, cập nhật container app. `azd down` xoá resource group của môi trường.

Với trip-service, `azure.yaml` có thể ngắn như sau.

```yaml
name: toinoi-trips
infra:
  provider: bicep
  path: infra
services:
  trips:
    project: ./examples/toi-noi-dotnet/31-38-trip-service
    language: dotnet
    host: containerapp
    docker:
      path: ./Dockerfile
hooks:
  postprovision:
    shell: sh
    run: ./scripts/grant-db-roles.sh
```

Khối này chỉ khai báo ý định. Nó không chứng minh `Dockerfile` nằm đúng chỗ hay `infra/` đã tồn tại. Cơ chế nối service với hạ tầng là một **tag**: với `host: containerapp`, `azd` tìm trong resource group resource Container Apps mang tag `azd-service-name` trùng tên service (ở đây là `trips`). Module `cloud/main.bicep` (bài 22) chưa nhận tham số tag, nên phải thêm cho nó một tham số `tags` để gắn tag này lên container app; `infra/main.bicep` của azd bọc module đó và tự tạo registry, identity, environment mà module nhận làm tham số. Module còn đòi `imageDigest` ngay lúc provision, khi image của service chưa được build, nên lần provision đầu cần một image tạm để `azd deploy` thay sau. Hook `postprovision` là chỗ gắn việc không phải Bicep, như cấp role database cho identity, nhưng nhớ rằng hook là shell script chạy trên máy người gọi, nên nó phải chạy lại được nhiều lần.

Vòng lặp hằng ngày của một dev trông như sau.

```bash
azd env new dev
azd env set AZURE_LOCATION southeastasia
azd up
azd env list
azd deploy trips
azd down --purge
```

`azd env new` tạo một môi trường và hỏi subscription, location, `azd up` dựng toàn bộ, `azd deploy trips` chỉ build lại và cập nhật service sau khi bạn sửa code, và `azd down --purge` xoá resource group kèm việc xoá hẳn các resource có soft delete như Key Vault. Khối này không chứng minh tên resource sinh ra khớp quy ước `rg-toinoi-trips-dev-sea`. Quy ước đó phải nằm trong Bicep của bạn, chứ `azd` không tự áp.

Template của `azd` là một repository mẫu có sẵn `azure.yaml` và `infra/`, dùng được qua `azd init --template`. Nó hợp để khởi động nhanh, nhưng với Tới Nơi bạn nên giữ Bicep của riêng mình để naming và tag nhất quán.

Giới hạn của `azd` cần nói thẳng. Nó tối ưu cho inner loop của một dev hoặc một môi trường tạm. Mặc định `azd deploy` đẩy image với một tag do công cụ sinh và cập nhật app, thay vì deploy theo digest bất biến như bài 44 đòi hỏi, nên bạn hãy kiểm output để biết chính xác nó dùng gì. Nó cũng không có approval, canary hay health gate. Quy tắc gọn là: dev dùng `azd`, staging và prod đi qua pipeline CD của bài 44.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. azd nối azure.yaml, Bicep trong infra, azd environment và resource group của một môi trường."><defs><marker id="az45-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">azure.yaml</text><text x="34" y="123" class="t-small">Service và hook</text><path d="M200 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az45-0-a)"/><rect x="240" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="254" y="95" class="t-b">infra/ Bicep</text><text x="254" y="123" class="t-small">Tag azd-service-name</text><path d="M420 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az45-0-a)"/><rect x="460" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">azd env dev</text><text x="474" y="123" class="t-small">Subscription, location</text><path d="M640 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az45-0-a)"/><rect x="680" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="694" y="95" class="t-b">rg-toinoi-...-dev</text><text x="694" y="123" class="t-small">Resource thật</text></svg>
</div>
<figcaption>Hình 1. azd nối azure.yaml, Bicep trong infra, azd environment và resource group của một môi trường.</figcaption>
</figure>

## Deployment stack quản vòng đời, không chỉ lần deploy

Deployment thường là một sự kiện: nó chạy, ghi lại lịch sử, rồi không còn gắn gì với resource nó đã tạo. **Deployment stack** là một đối tượng của Azure sở hữu một tập resource. Mỗi lần bạn cập nhật stack với template mới, Azure so sánh tập resource mới với tập đã quản lý và xử lý phần chênh lệch theo cách bạn chọn.

Trước đây cách duy nhất để dọn resource bị bỏ khỏi template là **Complete mode** của deployment. Nó xoá mọi thứ trong phạm vi deploy mà template không nhắc, kể cả resource do người khác thêm bằng tay vào resource group. Stack khác ở chỗ nó chỉ động vào những gì chính nó quản lý. Hai cấu hình quyết định hành vi:

- **`actionOnUnmanage`** (tham số CLI `--action-on-unmanage`): resource bị bỏ khỏi template thì sao. `detachAll` giữ lại nhưng thôi quản lý, `deleteResources` xoá resource, `deleteAll` xoá cả resource group con hay management group con nếu stack có chúng.
- **Deny setting** (`--deny-settings-mode`): `none`, `denyDelete` hoặc `denyWriteAndDelete`. Azure tạo một deny assignment trên các resource được quản lý, từ chối thao tác tương ứng của mọi principal, kể cả Owner, trừ các principal bạn khai báo ngoại lệ (`--deny-settings-excluded-principals`).

| Tiêu chí | Deployment Incremental | Complete mode | Deployment stack |
| --- | --- | --- | --- |
| Resource bị bỏ khỏi template | Còn nguyên | Bị xoá, kể cả thứ không do template tạo | Theo `actionOnUnmanage` |
| Biết resource nào thuộc về mình | Không | Không, dựa vào phạm vi | Có, danh sách managed resources |
| Chặn sửa tay | Không | Không | Có, qua deny settings |
| Rủi ro chính | Resource mồ côi tốn tiền | Xoá nhầm resource không thuộc template | Chặn luôn cả pipeline nếu cấu hình sai |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Stack chỉ sở hữu resource có trong template; deny assignment chặn xoá tay container app, còn identity tạo ngoài template không được bảo vệ."><defs><marker id="az45-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="15" width="560" height="270" rx="14" class="f-surface s-line"/><text x="40" y="45" class="t-b">Stack: rg-toinoi-trips-dev-sea</text><rect x="40" y="65" width="240" height="85" rx="10" class="f-pk-soft s-pk"/><text x="54" y="95" class="t-b">ca-toinoi-trips-dev</text><text x="54" y="123" class="t-small">Managed resource</text><rect x="300" y="65" width="260" height="85" rx="10" class="f-muted s-muted" stroke-dasharray="5 4"/><text x="314" y="95" class="t-b">id-toinoi-trips-dev</text><text x="314" y="123" class="t-small">Ngoài template, không chặn</text><rect x="40" y="180" width="520" height="85" rx="10" class="f-phys-soft s-phys"/><text x="54" y="210" class="t-b">Deny assignment: denyDelete</text><text x="54" y="238" class="t-small">Từ chối xoá, kể cả với Owner</text><rect x="640" y="105" width="220" height="85" rx="10" class="f-warn-soft s-warn"/><text x="654" y="135" class="t-b">az containerapp delete</text><text x="654" y="163" class="t-small">Xoá tay, bị từ chối</text><path d="M640 148 L585 148" class="f-none s-warn" stroke-width="2" marker-end="url(#az45-1-a)"/></svg>
</div>
<figcaption>Hình 2. Stack chỉ sở hữu resource có trong template; deny assignment chặn xoá tay container app, còn identity tạo ngoài template không được bảo vệ.</figcaption>
</figure>

Có một tương tác với bài 44 cần nhớ. Pipeline canary ghi trực tiếp lên container app (tạo revision, đổi traffic weight) bằng `az containerapp`, tức là ghi ngoài stack. Với `denyWriteAndDelete`, những thao tác đó bị từ chối trừ khi bạn đưa identity của pipeline vào danh sách ngoại lệ. Với `denyDelete` thì chỉ thao tác xoá bị chặn, nên đây là lựa chọn dễ sống chung với CD hơn. Đường được thiết kế để sửa resource được bảo vệ là cập nhật qua chính stack (chạy lại `az stack group create` với template mới); nếu pipeline gặp lỗi quyền, kiểm phần deny settings trong tài liệu deployment stacks.

## Môi trường: subscription theo env, resource group theo workload

Tới Nơi chạy ba môi trường, và ranh giới mạnh nhất Azure cung cấp để tách chúng là subscription. Quota, chi phí, Azure Policy và phân quyền đều bám theo subscription, nên dev bị hết quota hay một role gán nhầm sẽ không chạm được prod. Bên trong mỗi subscription, resource group chia theo workload (`rg-toinoi-trips-<env>-sea`, `rg-toinoi-data-<env>-sea`) để vòng đời và quyền truy cập đi cùng nhau. Một resource group chứa mọi thứ của một môi trường là tiện lúc đầu nhưng sẽ làm cho quyền và việc xoá trở nên quá rộng.

Tên và tag là thứ giúp bạn tìm ra mọi thứ khi có sự cố hoặc khi dọn dẹp, vì vậy chúng phải thống nhất bằng Bicep chứ không bằng trí nhớ. Một bộ tag tối thiểu là `env`, `workload`, `owner` và `expires` (với môi trường tạm). Azure Policy có thể bắt buộc tag này trên resource group, nhưng cách cấu hình nằm ngoài phạm vi bài.

Dữ liệu là chỗ dễ sai nhất. Bảng `trips` chứa điểm đón, điểm đến và số điện thoại, tức là dữ liệu cá nhân của khách. Không restore bản sao lưu của prod xuống dev hay staging để «cho giống thật». Môi trường thấp hơn có ít kiểm soát hơn và nhiều người truy cập hơn, nên bản sao đó thành rủi ro rò rỉ. Dùng dữ liệu tổng hợp, hoặc xuất từ prod qua một bước che (mask, ví dụ băm số điện thoại, làm tròn toạ độ) do một quy trình có kiểm soát thực hiện và được xem lại trước khi dùng.

## Môi trường tạm cho pull request: tạo khi mở, xoá khi đóng

Với `azd`, một môi trường theo pull request chỉ là `azd env new pr123` rồi `azd up`. Mỗi môi trường tạm có Container Apps của nó và một PostgreSQL ở tier Burstable với kích thước nhỏ nhất, đủ để chạy migration và vài request kiểm thử. Workflow dưới đây tạo khi pull request mở hoặc cập nhật, và xoá khi nó đóng.

```yaml
name: pr-env
on:
  pull_request: { types: [opened, synchronize, closed] }
permissions: { id-token: write, contents: read }
jobs:
  up:
    if: github.event.action != 'closed'
    runs-on: ubuntu-latest
    environment: dev
    steps:
      - uses: actions/checkout@v4
      - uses: Azure/setup-azd@v2
      - run: azd auth login --client-id "${{ vars.AZURE_CLIENT_ID }}" --tenant-id "${{ vars.AZURE_TENANT_ID }}" --federated-credential-provider github
      - run: >
          azd env new pr${{ github.event.number }} --no-prompt
          --subscription "${{ vars.AZURE_SUBSCRIPTION_ID }}" --location southeastasia
          && azd up --no-prompt
  down:
    if: github.event.action == 'closed'
    runs-on: ubuntu-latest
    environment: dev
    steps:
      - uses: actions/checkout@v4
      - uses: Azure/setup-azd@v2
      - run: azd auth login --client-id "${{ vars.AZURE_CLIENT_ID }}" --tenant-id "${{ vars.AZURE_TENANT_ID }}" --federated-credential-provider github
      - run: >
          azd env new pr${{ github.event.number }} --no-prompt
          --subscription "${{ vars.AZURE_SUBSCRIPTION_ID }}" --location southeastasia
          && azd down --force --purge
```

Khối này chỉ minh hoạ vòng đời. Job `down` cần checkout vì `azd` đọc `azure.yaml` và `infra/`, và nó tạo lại môi trường cùng tên trên runner mới để tìm lại resource đã dựng. Khối không truyền các tham số Bicep riêng của bạn (qua `azd env set`) và không chứng minh `azd down` tìm đúng resource group; hãy đọc tài liệu `azd` về CI trước khi dùng.

Có ba giới hạn chắc chắn gặp. Thứ nhất, quota vCPU của Container Apps và số PostgreSQL server trên mỗi subscription và region là hữu hạn, nên nhiều pull request mở cùng lúc sẽ chạm trần. Thứ hai, tên một số resource có độ dài tối đa (Key Vault, storage account), nên `pr123` cộng tiền tố phải còn vừa. Thứ ba, workflow `down` có thể không chạy khi pull request đóng bất thường hoặc runner lỗi, vì vậy cần một job định kỳ xoá resource group có tag `expires` đã quá hạn. Chi phí của môi trường tạm là chi phí của những thứ chạy khi không ai dùng. Hãy tra Pricing Calculator cho PostgreSQL và Container Apps theo kích thước bạn chọn, và đặt `minReplicas` thấp.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p><code>azd down</code> xoá cả resource group của môi trường, và <code>--purge</code> xoá hẳn Key Vault đã soft delete, tức là mất secret không lấy lại được. Một <code>azd env</code> trỏ nhầm sang subscription hay tên của môi trường khác sẽ xoá nhầm thứ đó. Với stack, <code>deleteAll</code> xoá cả resource group con. Luôn chạy <code>azd env list</code> và kiểm subscription đang chọn trước mọi lệnh xoá, và đừng bao giờ để prod cùng subscription với môi trường tạm.</p>
</div>

## Lab: azd up lên dev, rồi chuyển thành deployment stack

Lab mô tả cách làm, các kỳ vọng dưới đây chưa phải kết quả đã chạy. Giả định bạn đã có Azure CLI, `azd`, quyền Contributor trên một subscription lab, và Bicep ở `infra/` bọc `cloud/main.bicep` như phần đầu bài.

1. Chạy `azd init` ở gốc repository, chọn dùng code có sẵn, kiểm `azure.yaml` có service `trips` rồi tạo môi trường bằng `azd env new dev`. Chạy `azd up`. Kỳ vọng: có `rg-toinoi-trips-dev-sea` với container app đang chạy và output cho biết endpoint.
2. Chuyển hạ tầng thành deployment stack. Lệnh dưới tạo stack ở phạm vi resource group từ `cloud/main.bicep`, xoá resource bị bỏ khỏi template và chặn xoá tay. Khi `appName` trùng tên container app azd đã tạo, stack nhận quản resource có sẵn đó chứ không tạo cái mới.

```bash
az stack group create \
  --name stack-toinoi-trips-dev \
  --resource-group rg-toinoi-trips-dev-sea \
  --template-file examples/toi-noi-dotnet/31-38-trip-service/cloud/main.bicep \
  --parameters params.dev.json \
  --action-on-unmanage deleteAll \
  --deny-settings-mode denyDelete \
  --yes
```

   Khối này không chứng minh `params.dev.json` đủ tham số. File đó phải chứa các giá trị như `appName` và `imageDigest` mà `main.bicep` yêu cầu. Cũng lưu ý stack này chỉ quản các resource trong template, nên environment và identity tạo ngoài template không nằm trong tập được bảo vệ.
3. Thử xoá tay container app. Kỳ vọng: lệnh bị từ chối với thông báo liên quan tới deny assignment, và app vẫn còn.

```bash
az containerapp delete -g rg-toinoi-trips-dev-sea -n ca-toinoi-trips-dev --yes
az stack group show -g rg-toinoi-trips-dev-sea -n stack-toinoi-trips-dev \
  --query "resources[].id" -o tsv
```

4. Dọn lab bằng lệnh xoá stack, thay vì xoá tay từng resource, để deny assignment cũng biến mất. Sau đó `azd down` cho môi trường.

```bash
az stack group delete -g rg-toinoi-trips-dev-sea -n stack-toinoi-trips-dev \
  --action-on-unmanage deleteAll --yes
```

**Bài tập:** bạn đổi stack sang `denyWriteAndDelete` rồi chạy pipeline canary của bài 44 lên cùng container app. Chuyện gì xảy ra, và có hai cách xử lý nào?

<details><summary>Xem lời giải</summary><p>Lệnh `az containerapp update` và `traffic set` của pipeline là thao tác ghi ngoài stack, nên bị deny assignment từ chối. Cách thứ nhất là hạ xuống `denyDelete` để chỉ chặn xoá. Cách thứ hai là giữ `denyWriteAndDelete` nhưng thêm identity của pipeline vào danh sách principal ngoại lệ, đánh đổi bằng việc identity đó lại sửa được app mà stack không biết.</p></details>

## Học tiếp từ đây

Bạn đã biết `azd` rút ngắn vòng lặp dev, deployment stack thay cho Complete mode để dọn resource bị bỏ và chặn xoá tay, và vì sao môi trường nên tách theo subscription với dữ liệu prod không đi xuống. Bước tiếp theo trong phần này là [bài 46](/articles/azure-database-migration-trong-pipeline), nơi migration database được đặt vào pipeline với thứ tự, role và rollback rõ ràng. Nếu cần ôn cách pipeline đẩy bản mới lên prod, xem lại [bài 44](/articles/azure-cd-environment-approval-blue-green-canary).

## Tài liệu gốc

- [Azure Developer CLI documentation](https://learn.microsoft.com/en-us/azure/developer/azure-developer-cli/)
- [Deployment stacks](https://learn.microsoft.com/en-us/azure/azure-resource-manager/bicep/deployment-stacks)
- [Azure Resource Manager deployment modes](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/deployment-modes)
- [Azure Container Apps overview](https://learn.microsoft.com/en-us/azure/container-apps/overview)
- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
- [Azure Policy overview](https://learn.microsoft.com/en-us/azure/governance/policy/overview)

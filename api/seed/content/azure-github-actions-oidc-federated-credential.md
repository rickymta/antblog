---
slug: azure-github-actions-oidc-federated-credential
title: "GitHub Actions và Azure DevOps: đăng nhập OIDC bằng federated credential"
summary: "Bài 42: cho pipeline GitHub Actions của Tới Nơi đăng nhập Azure không cần secret dài hạn, tạo app registration gh-toinoi-deploy với federated credential theo environment dev và prod, cấp quyền tối thiểu và đối chiếu với service connection của Azure DevOps."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, github-actions, oidc, federated-credential, security]
language: vi
---

Đây là **bài 42** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 3/10 của phần).

Bài này nối pipeline GitHub Actions của `trip-service` với Azure mà không để lại client secret nào trong repository. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Pipeline đăng nhập Azure bằng OIDC thì repository có còn phải giữ một secret nào để Azure nhận ra nó không?</p>
<details><summary>Xem đáp án</summary><p>Không còn secret dài hạn. Repository chỉ giữ ba định danh (client ID, tenant ID, subscription ID) và chúng không phải bí mật. Cái chứng minh danh tính là token ngắn hạn do GitHub ký cho từng lần chạy.</p></details>
</div>

## Hai nền tảng pipeline, cùng một bài toán đăng nhập

Loạt bài này dùng GitHub Actions vì mã nguồn Tới Nơi nằm trên GitHub. Nhiều đội Azure lại quen Azure Pipelines. Hai bên giống nhau ở phần lớn thứ quan trọng: pipeline là file YAML nằm trong repository, có khái niệm environment để gắn approval, và có kho tác vụ dựng sẵn để không phải tự viết mọi bước.

| Tiêu chí | GitHub Actions | Azure Pipelines |
| --- | --- | --- |
| Định nghĩa | `.github/workflows/*.yml` | `azure-pipelines.yml` |
| Đơn vị chạy | job trong workflow, mỗi job một runner | job trong stage, mỗi job một agent |
| Environment và approval | `environment` với required reviewer, giới hạn nhánh | Environment với approval và check |
| Kho tác vụ | Marketplace action, ví dụ `azure/login` | Task, ví dụ `AzureCLI@2` |
| Cách đăng nhập Azure không secret | OIDC token đổi lấy token Entra | Service connection kiểu workload identity federation |

Phần dưới đây đi sâu vào GitHub Actions và chỉ ra chỗ tương đương trên Azure DevOps. Cơ chế bên dưới là một: nền tảng pipeline phát hành token cho từng lần chạy, Entra ID kiểm token đó theo điều kiện bạn đăng ký trước, rồi đổi lấy token truy cập Azure.

## Federated credential thay client secret bằng một lời hứa có điều kiện

Cách cũ là tạo app registration, sinh client secret, dán vào secret của repository. Secret đó có hạn, nằm ở hai nơi, và ai đọc được biến môi trường của runner đều mang nó đi được. Bài 13 đã giới thiệu federated credential cho workload ngoài Azure. Ở đây ta áp dụng cho pipeline.

Mỗi lần job chạy, GitHub có thể phát một token OIDC ký bằng khoá của GitHub, sống ngắn và mang các claim mô tả job: repository nào, nhánh nào, environment nào. Entra ID đã được bạn dặn trước qua federated credential gắn vào app registration. Khi token tới, Entra kiểm chữ ký theo issuer, kiểm audience, và so khớp subject. Khớp cả ba thì Entra cấp token Entra để gọi Azure Resource Manager. Không khớp thì dừng ngay, kể cả khi token hợp lệ về chữ ký.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Job GitHub Actions xin token OIDC, Entra kiểm issuer, subject, audience rồi đổi lấy token Entra để gọi Azure Resource Manager theo role assignment."><defs><marker id="az42-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Job trong environment prod</text><text x="34" y="123" class="t-small">permissions: id-token: write</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az42-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Entra kiểm 3 trường</text><text x="319" y="123" class="t-small">issuer, subject, audience</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az42-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Token Entra ngắn hạn</text><text x="604" y="123" class="t-small">Gọi ARM theo role</text></svg>
</div>
<figcaption>Hình 1. Job GitHub Actions xin token OIDC, Entra kiểm issuer, subject, audience rồi đổi lấy token Entra để gọi Azure Resource Manager theo role assignment.</figcaption>
</figure>

Ba giá trị cho GitHub Actions là issuer `https://token.actions.githubusercontent.com`, audience `api://AzureADTokenExchange` và một subject. Credential có thể gắn vào app registration hoặc vào user-assigned managed identity. Lab dưới đây dùng app registration `gh-toinoi-deploy`. Azure DevOps có hai biến thể tương tự: service connection do Azure DevOps tự tạo app và credential, hoặc bạn tự tạo rồi khai báo. Issuer và subject lúc đó do Azure DevOps quy định và đã từng đổi định dạng (dạng cũ là issuer theo organization với subject `sc://<organization>/<project>/<tên service connection>`, dạng mới do Entra phát hành). Khi tự tạo, hãy chép đúng issuer và subject mà trang tạo service connection hiển thị thay vì tự gõ.

## Subject quyết định ai đổi được token

Mặc định Entra so khớp subject chính xác, từng ký tự (biến thể cho phép biểu thức khớp mẫu còn ở dạng preview, kiểm tài liệu nếu muốn dùng). Vì vậy chọn subject là chọn ai được vào. GitHub sinh subject theo ngữ cảnh job, và ngữ cảnh này đổi theo cách job được kích hoạt.

| Ngữ cảnh job | Subject GitHub phát hành |
| --- | --- |
| Job khai `environment: prod` | `repo:<org>/<repo>:environment:prod` |
| Job không có environment, chạy trên nhánh main | `repo:<org>/<repo>:ref:refs/heads/main` |
| Job chạy do pull request | `repo:<org>/<repo>:pull_request` |
| Job chạy trên tag | `repo:<org>/<repo>:ref:refs/tags/<tag>` |

Có một hệ quả dễ hiểu sai. Khi job khai `environment`, subject chỉ chứa tên environment, không chứa nhánh. Entra không biết nhánh nào đang chạy và cũng không biết approval đã được duyệt chưa. Thứ chặn là phía GitHub: job tới environment `prod` phải qua required reviewer và quy tắc giới hạn nhánh trước khi runner bắt đầu và token được phát. Nếu bạn quên đặt hai quy tắc đó, bất kỳ ai push được workflow vào repository đều khai được `environment: prod` và đổi token prod. Federated credential chỉ đúng khi cấu hình environment đứng sau nó cũng đúng.

Lệnh tạo credential cho environment `prod` trên app registration dùng một file JSON mô tả ba trường:

```json
{
  "name": "fc-github-toinoi-prod",
  "issuer": "https://token.actions.githubusercontent.com",
  "subject": "repo:<org>/<repo>:environment:prod",
  "audiences": ["api://AzureADTokenExchange"]
}
```

```bash
az ad app federated-credential create \
  --id <app-object-id> \
  --parameters credential-prod.json
```

Hai khối trên chỉ đăng ký niềm tin. Chúng không cho pipeline quyền làm gì, vì quyền đến từ role assignment ở phần sau. Mỗi app registration cũng có giới hạn số federated credential, nên kiểm tài liệu nếu bạn định gom nhiều repository vào một app.

## Workflow đăng nhập: ba định danh và một quyền id-token

Phía workflow chỉ cần hai thứ. Thứ nhất là `permissions: id-token: write`, nếu thiếu thì job không được phép xin token OIDC. Thứ hai là action `azure/login` với ba định danh. Chúng nằm ở environment variable của GitHub thay vì secret, vì không có gì cần giấu: biết client ID mà không có token GitHub hợp lệ thì không đăng nhập được.

```yaml
name: oidc-smoke
on:
  workflow_dispatch:
    inputs:
      env:
        type: choice
        options: [dev, prod]
permissions:
  id-token: write
  contents: read
jobs:
  whoami:
    runs-on: ubuntu-latest
    environment: ${{ inputs.env }}
    steps:
      - uses: azure/login@v2
        with:
          client-id: ${{ vars.AZURE_CLIENT_ID }}
          tenant-id: ${{ vars.AZURE_TENANT_ID }}
          subscription-id: ${{ vars.AZURE_SUBSCRIPTION_ID }}
      - run: az account show --query "{sub:name,user:user.name}" -o table
      - run: az group list --query "[].name" -o tsv
```

Khối này chạy cùng một file cho dev và prod, vì mỗi environment giữ giá trị `AZURE_CLIENT_ID` riêng và subject đổi theo tên environment. Nó không chứng minh bất cứ thứ gì về an toàn của prod. Phiên bản action `v2` ở đây chỉ để minh hoạ, nên hãy kiểm bản hiện hành và ghim vào một commit SHA trong repository thật, vì action bên thứ ba chạy với quyền của job.

Trên Azure DevOps, tương đương là service connection loại Azure Resource Manager chọn workload identity federation, rồi task `AzureCLI@2` trỏ vào tên service connection qua tham số `azureSubscription`. Approval và check nằm ở environment Azure DevOps, cùng vai trò với required reviewer của GitHub.

## Quyền tối thiểu, và hai identity thay vì một

Token Entra chỉ làm được những gì role assignment cho phép. Với pipeline Tới Nơi, nguyên tắc là gán ở scope hẹp nhất đủ cho đúng việc pipeline làm.

| Việc của pipeline | Role | Scope |
| --- | --- | --- |
| Tạo hoặc cập nhật Container App, chạy Bicep | Contributor | `rg-toinoi-trips-<env>-sea` |
| Push image (bài 43) | AcrPush | `acrtoinoi<env>` |
| Đọc secret Key Vault | Không cấp | Runtime đọc bằng identity của nó |
| Ghi state Terraform (bài 41) | Storage Blob Data Contributor | Container chứa state |

Dòng Key Vault đáng dừng lại. Pipeline không cần đọc connection string production. Container App tự đọc nó qua `id-toinoi-trips-<env>`, nên chỗ duy nhất giữ secret là Key Vault và chỗ duy nhất đọc nó là runtime. Pipeline bị chiếm quyền vẫn không lấy được secret.

Contributor còn một ranh giới nữa: nó không được tạo role assignment. Nếu Bicep của bạn gán AcrPull cho runtime identity, lần chạy pipeline sẽ trả lỗi thiếu quyền ghi role assignment. Có hai hướng. Hoặc gán sẵn các role đó bằng tay một lần ngoài pipeline, hoặc cấp thêm role `Role Based Access Control Administrator` kèm condition chỉ cho gán đúng vài role như AcrPull. Đừng chữa bằng cách nâng pipeline lên Owner.

Mô hình đúng là hai identity, một cho mỗi môi trường. Credential của dev chỉ ra token cho app dev, app dev chỉ có quyền ở resource group dev. Credential prod làm tương tự cho prod và đứng sau approval.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Hai nhánh tách biệt: environment dev dùng app dev với quyền ở resource group dev; environment prod có approval, dùng app prod với quyền ở resource group prod."><defs><marker id="az42-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="34" y="55" class="t-b">Environment dev</text><text x="34" y="85" class="t-small">Không cần approval</text><path d="M270 70 L315 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az42-1-a)"/><rect x="315" y="20" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="329" y="55" class="t-b">App gh-toinoi-deploy-dev</text><text x="329" y="85" class="t-small">Credential: environment:dev</text><path d="M565 70 L610 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az42-1-a)"/><rect x="610" y="20" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="624" y="55" class="t-b">rg-toinoi-trips-dev-sea</text><text x="624" y="85" class="t-small">Contributor + AcrPush dev</text><rect x="20" y="170" width="250" height="100" rx="10" class="f-warn-soft s-warn"/><text x="34" y="205" class="t-b">Environment prod</text><text x="34" y="235" class="t-small">Required reviewer</text><path d="M270 220 L315 220" class="f-none s-phys" stroke-width="2" marker-end="url(#az42-1-a)"/><rect x="315" y="170" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="329" y="205" class="t-b">App gh-toinoi-deploy-prod</text><text x="329" y="235" class="t-small">Credential: environment:prod</text><path d="M565 220 L610 220" class="f-none s-phys" stroke-width="2" marker-end="url(#az42-1-a)"/><rect x="610" y="170" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="624" y="205" class="t-b">rg-toinoi-trips-prod-sea</text><text x="624" y="235" class="t-small">Contributor + AcrPush prod</text></svg>
</div>
<figcaption>Hình 2. Hai nhánh tách biệt: environment dev dùng app dev với quyền ở resource group dev; environment prod có approval, dùng app prod với quyền ở resource group prod.</figcaption>
</figure>

## Workflow có sẵn trong repo và chỗ mở rộng

Repository đã có `.github/workflows/toi-noi-dotnet.yml`. Nó chạy khi có thay đổi trong thư mục service hoặc chính file workflow, với `permissions: contents: read` ở cấp workflow. Job `verify` dựng PostgreSQL 18 làm service container có health check, chạy `dotnet test` với `TOI_NOI_PG` trỏ vào nó, rồi dùng `PublishContainer` để xuất image ra file OCI archive, không push đâu cả. Cuối cùng nó upload các artifact kể cả khi test lỗi.

Workflow này chưa đăng nhập Azure, và đó là chủ ý của một lab học: nó không cần bất kỳ credential nào ngoài mật khẩu PostgreSQL chỉ dùng trong runner. Khi mở rộng, có một bẫy. Vì quyền `id-token` ở cấp workflow đang không được cấp, job nào gọi `azure/login` sẽ lỗi lúc xin token. Cách sửa đúng không phải sửa quyền ở cấp workflow, mà khai riêng ở job cần đăng nhập, để job test vẫn chỉ đọc nội dung:

```yaml
  push-image:
    needs: verify
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: dev
    permissions:
      id-token: write
      contents: read
    steps:
      - uses: azure/login@v2
        with:
          client-id: ${{ vars.AZURE_CLIENT_ID }}
          tenant-id: ${{ vars.AZURE_TENANT_ID }}
          subscription-id: ${{ vars.AZURE_SUBSCRIPTION_ID }}
```

Đoạn này là phần thêm vào dưới `jobs:`, chưa có bước build hay push thật; bài 43 điền các bước đó. Điều kiện `if` loại pull request và các nhánh khác main khỏi job này. Lý do không chỉ là pull request từ fork không xin được token OIDC: pull request từ nhánh trong cùng repository mà khai `environment: dev` sẽ nhận đúng subject `environment:dev` và đổi được token dev. Vì vậy environment `dev` cũng nên giới hạn nhánh được deploy là `main`. Với pull request, job `verify` đã đủ để chặn merge khi test hỏng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Đừng đặt subject theo <code>pull_request</code> hoặc theo cả nhánh main cho credential có quyền ghi. Pull request chưa ai review sẽ đổi được token. Đừng cấp Contributor ở cả subscription cho tiện: một workflow lỗi hoặc một action bị chiếm quyền sẽ sửa được mọi resource group. Và environment prod không có required reviewer thì credential prod chỉ còn là một cái tên.</p>
</div>

## Lab: app gh-toinoi-deploy, hai credential, không secret

Lab mô tả cách làm trên subscription và repository của bạn. Để ngắn gọn, nó dùng một app registration với hai credential. Ở production bạn nên tách thành hai app như hình 2, vì với một app dùng chung, quyền của dev và prod lẫn vào nhau. Trước hết tạo app, service principal và hai credential:

```bash
APP_ID=$(az ad app create --display-name gh-toinoi-deploy \
  --query appId -o tsv)
SP_ID=$(az ad sp create --id $APP_ID --query id -o tsv)

for ENV in dev prod; do
  az ad app federated-credential create --id $APP_ID --parameters "{
    \"name\": \"fc-github-toinoi-$ENV\",
    \"issuer\": \"https://token.actions.githubusercontent.com\",
    \"subject\": \"repo:<org>/<repo>:environment:$ENV\",
    \"audiences\": [\"api://AzureADTokenExchange\"]
  }"
done

az role assignment create --assignee-object-id $SP_ID \
  --assignee-principal-type ServicePrincipal --role Contributor \
  --scope /subscriptions/<subscription-id>/resourceGroups/rg-toinoi-trips-dev-sea
```

Role chỉ được gán ở resource group dev. Credential prod vẫn đăng nhập được, nhưng nó không thấy gì ở prod: có token hợp lệ không có nghĩa là có quyền. Vì dùng chung một app, job prod vẫn thấy resource group dev, đúng chỗ quyền lẫn vào nhau mà hai app sẽ tách ra. Tiếp theo tạo hai environment `dev` và `prod` trong Settings của repository, đặt required reviewer cho `prod`, rồi lưu ba định danh thành variable của từng environment:

```bash
for ENV in dev prod; do
  gh variable set AZURE_CLIENT_ID --env $ENV --repo <org>/<repo> --body "$APP_ID"
  gh variable set AZURE_TENANT_ID --env $ENV --repo <org>/<repo> --body "<tenant-id>"
  gh variable set AZURE_SUBSCRIPTION_ID --env $ENV --repo <org>/<repo> --body "<subscription-id>"
done
```

Kỳ vọng khi chạy workflow `oidc-smoke` với `env = dev`: bước `azure/login` thành công, `az account show` in ra subscription và danh tính là service principal của app, `az group list` liệt kê `rg-toinoi-trips-dev-sea`. Với `env = prod`, job dừng chờ reviewer duyệt, rồi đăng nhập thành công, danh sách vẫn chỉ có resource group dev và không có gì của prod. Lab chưa chứng minh pipeline deploy được app, vì Contributor mới chỉ được thử bằng lệnh đọc. Dọn lab bằng cách xoá role assignment, rồi xoá app bằng `az ad app delete --id $APP_ID`.

**Bài tập:** workflow khai `environment: prod` nhưng bước `azure/login` báo không có federated identity record nào khớp. Bạn kiểm gì đầu tiên, và vì sao?

<details><summary>Xem lời giải</summary><p>Đọc thông báo lỗi của Entra, vì nó in ra subject mà GitHub vừa phát hành. So khớp từng ký tự với subject trong credential: sai chữ hoa chữ thường của tên environment, thiếu dòng `environment:` ở job (khi đó subject là theo nhánh), hoặc sai tên repository thường gặp nhất. Entra so khớp chính xác nên chỉ lệch một ký tự là từ chối.</p></details>

## Học tiếp từ đây

Bạn vừa mô tả cách cho GitHub Actions đăng nhập Azure bằng federated credential theo environment, cấp quyền ở scope hẹp, và nối vào workflow có sẵn của repo mà không đụng tới secret dài hạn. Bài kế tiếp, [bài 43](/articles/azure-ci-pipeline-build-test-scan-push-acr), dùng danh tính này để build image, quét lỗ hổng và push vào `acrtoinoidev`. Nếu cần ôn cơ chế federated credential, xem lại [bài 13](/articles/azure-managed-identity-workload-identity), còn phần hạ tầng bằng code nằm ở [bài 41](/articles/azure-terraform-azurerm-state-workspace).

## Tài liệu gốc

- [Workload identity federation](https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation)
- [Configure an app to trust an external identity provider](https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation-create-trust)
- [Deploy to Azure from GitHub Actions](https://learn.microsoft.com/en-us/azure/developer/github/connect-from-azure-openid-connect)
- [Connect to Azure with an Azure Resource Manager service connection](https://learn.microsoft.com/en-us/azure/devops/pipelines/library/connect-to-azure)
- [Azure RBAC documentation](https://learn.microsoft.com/en-us/azure/role-based-access-control/)

---
slug: azure-ci-pipeline-build-test-scan-push-acr
title: "CI: build, test, scan image và push ACR"
summary: "Bài 43: dựng ci.yml cho trip-service của Tới Nơi, test với PostgreSQL service container, build image gắn git sha, quét Trivy chặn CRITICAL, push acrtoinoidev bằng OIDC và xuất image digest cho bước CD, kèm cache, concurrency và branch protection."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, ci, github-actions, container-registry, trivy]
language: vi
---

Đây là **bài 43** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 4/10 của phần).

Bài này dựng giai đoạn CI cho `trip-service`: test với PostgreSQL thật, build image, quét lỗ hổng, đẩy vào `acrtoinoidev` và trả ra digest cho bước deploy. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Pipeline đã push image lên registry và quét lỗ hổng sau đó. Nếu quét ra lỗi CRITICAL thì image xấu đã nằm trong registry chưa?</p>
<details><summary>Xem đáp án</summary><p>Có, nếu quét đặt sau bước push. Registry không tự từ chối image có lỗ hổng, nên cổng chặn phải đứng trước push. Quét sau push vẫn có ích, nhưng là tín hiệu thứ hai chứ không phải cổng.</p></details>
</div>

## CI là một chuỗi cổng, mỗi cổng chặn một loại lỗi

CI của Tới Nơi chạy bốn việc theo thứ tự rẻ trước, đắt sau: test, build image, quét, push. Test hỏng thì không build. Quét ra CRITICAL thì không push. Mỗi bước sau tốn thời gian và đụng tài nguyên dùng chung hơn bước trước, nên lỗi sớm nhất dừng cả chuỗi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Bốn cổng của CI theo thứ tự: test với PostgreSQL, build image gắn git sha, quét Trivy chặn CRITICAL, push ACR và xuất digest."><defs><marker id="az43-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Test</text><text x="34" y="123" class="t-small">PostgreSQL thật</text><path d="M205 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-0-a)"/><rect x="235" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="249" y="95" class="t-b">Build image</text><text x="249" y="123" class="t-small">Tag bằng git sha</text><path d="M420 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-0-a)"/><rect x="450" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="464" y="95" class="t-b">Quét Trivy</text><text x="464" y="123" class="t-small">Chặn khi có CRITICAL</text><path d="M635 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-0-a)"/><rect x="665" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="679" y="95" class="t-b">Push ACR</text><text x="679" y="123" class="t-small">Xuất digest cho CD</text></svg>
</div>
<figcaption>Hình 1. Bốn cổng của CI theo thứ tự: test với PostgreSQL, build image gắn git sha, quét Trivy chặn CRITICAL, push ACR và xuất digest.</figcaption>
</figure>

Workflow `toi-noi-dotnet.yml` có sẵn đã làm tốt cổng đầu: PostgreSQL 18 là service container có health check, `dotnet test` chạy với `TOI_NOI_PG` trỏ vào nó. Nó cũng đã build image nhưng chỉ ra file OCI archive, không push. Bài này giữ cùng khung và thêm ba cổng còn lại. Phần đăng nhập Azure dùng đúng `gh-toinoi-deploy` và environment ở [bài 42](/articles/azure-github-actions-oidc-federated-credential).

## Một file ci.yml cho cả bốn cổng

Dưới đây là toàn bộ `ci.yml` (workflow lấy tên từ tên file khi không khai `name`). Job `test` chạy cho cả pull request lẫn push. Job `image` chỉ chạy khi push vào main, vì pull request từ fork không có token OIDC và vì không ai muốn mỗi pull request đẩy image vào registry.

```yaml
on:
  pull_request: { paths: ['examples/toi-noi-dotnet/31-38-trip-service/**'] }
  push: { branches: [main], paths: ['examples/toi-noi-dotnet/31-38-trip-service/**'] }
concurrency: { group: 'ci-${{ github.ref }}', cancel-in-progress: "${{ github.event_name == 'pull_request' }}" }
permissions: { contents: read }
env: { SVC: examples/toi-noi-dotnet/31-38-trip-service, ACR: acrtoinoidev.azurecr.io }
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    services:
      postgres:
        image: postgres:18
        env: { POSTGRES_DB: toi_noi_dotnet_lab, POSTGRES_USER: lab_user, POSTGRES_PASSWORD: ci-lab-only }
        ports: ['5432:5432']
        options: --health-cmd "pg_isready -U lab_user -d toi_noi_dotnet_lab" --health-interval 5s --health-timeout 5s --health-retries 12
    env: { TOI_NOI_PG: 'Host=127.0.0.1;Port=5432;Database=toi_noi_dotnet_lab;Username=lab_user;Password=ci-lab-only' }
    steps:
      - uses: actions/checkout@v7.0.1
      - uses: actions/setup-dotnet@v6.1.0
        with: { dotnet-version: '10.0.x', cache: true, cache-dependency-path: '**/packages.lock.json' }
      - run: dotnet test $SVC/Tests -c Release --collect:"XPlat Code Coverage"
  image:
    needs: test
    if: github.event_name == 'push'
    runs-on: ubuntu-latest
    environment: dev
    permissions: { id-token: write, contents: read }
    outputs: { image-digest: '${{ steps.push.outputs.image-digest }}' }
    steps:
      - uses: actions/checkout@v7.0.1
      - uses: azure/login@v2
        with: { client-id: '${{ vars.AZURE_CLIENT_ID }}', tenant-id: '${{ vars.AZURE_TENANT_ID }}', subscription-id: '${{ vars.AZURE_SUBSCRIPTION_ID }}' }
      - run: az acr login -n acrtoinoidev && docker build -t $ACR/trips:${{ github.sha }} $SVC
      - uses: aquasecurity/trivy-action@0.33.1
        with: { image-ref: 'acrtoinoidev.azurecr.io/trips:${{ github.sha }}', severity: CRITICAL, exit-code: '1', ignore-unfixed: true }
      - id: push
        run: |
          docker push $ACR/trips:${{ github.sha }}
          echo "image-digest=$(az acr repository show -n acrtoinoidev --image trips:${{ github.sha }} --query digest -o tsv)" >> "$GITHUB_OUTPUT"
```

Khối này mô tả cách ghép, nó không chứng minh pipeline xanh. Ba chi tiết cần kiểm trên repo của bạn: `--collect:"XPlat Code Coverage"` chỉ ra số liệu nếu project test tham chiếu `coverlet.collector` (project `Tests` hiện chưa có); cache của `setup-dotnet` khoá theo `packages.lock.json`, nên phải bật `RestorePackagesWithLockFile` và commit file lock, nếu không bước setup sẽ báo thiếu file; và phiên bản các action ở đây là minh hoạ, nên hãy ghim vào commit SHA đã kiểm trước khi dùng thật.

## Test thật, và kết quả phải đọc được

Service container PostgreSQL cho phép test chạm database thật thay vì mock, nhờ đó bắt được lỗi SQL, lỗi mapping và lỗi migration mà test đơn vị bỏ sót. Cái giá là thời gian khởi động database, nên health check phải đủ kiên nhẫn: lệnh `pg_isready` với số lần thử đủ lớn và `--health-interval` ngắn. Bỏ interval thì Docker dùng mặc định 30 giây giữa hai lần kiểm, job chờ lâu vô ích. Thiếu bước này, test bắt đầu khi database chưa nhận kết nối và lỗi nhìn như lỗi code.

Mật khẩu `ci-lab-only` nằm trong YAML vì database sống và chết trong runner. Quy tắc đó chỉ đúng với database tạm, không áp dụng cho thứ gì nằm ngoài runner.

Coverage cho biết dòng nào chưa được test chạm tới, không cho biết test có đúng không. Dùng nó để tìm vùng chưa kiểm, đừng biến ngưỡng phần trăm thành mục tiêu. Để kết quả hiện ngay trên trang run thay vì trong log dài, bạn có thể thêm logger cho Actions (có package mã nguồn mở in annotation và summary) hoặc ghi báo cáo vào `$GITHUB_STEP_SUMMARY`. Hai việc này cần thêm package vào project test, và bài không kiểm chúng trên repo.

## Image: tag theo git sha, quét trước khi push

Dockerfile của service dùng multi-stage: một stage SDK để restore và publish, một stage runtime `aspnet:10.0` chỉ chứa kết quả publish. Stage cuối không mang SDK, nên image nhỏ hơn và có ít gói hơn để bị báo lỗ hổng. Tag là `github.sha`, nên mỗi commit có đúng một tag, và từ tag nhìn ngược ra commit ngay. Tag kiểu `latest` hay `1.0` bị bài 21 chỉ ra là tag trôi.

Quét bằng Trivy đặt giữa build và push. Cờ `exit-code: 1` kèm `severity: CRITICAL` làm bước đỏ khi có lỗ hổng nghiêm trọng, và image chưa bao giờ vào registry. Cờ `ignore-unfixed` bỏ qua lỗ hổng chưa có bản vá, vì build lại cũng không sửa được và pipeline chỉ đỏ vô ích. Cái giá là bạn chấp nhận rủi ro đó có chủ ý, nên ghi nó vào quy ước của đội. Cũng nhớ rằng nguồn lỗi hay là base image, không phải code của bạn. Khi đó cách sửa là build lại trên base image đã vá.

Trivy không thay Defender for Containers đã nói ở bài 21. Trivy là cổng tại thời điểm build. Defender quét tiếp image đã nằm trong registry khi cơ sở dữ liệu lỗ hổng cập nhật, nên bắt được lỗ hổng công bố sau khi bạn đã push. Hai cái bổ sung nhau.

SBOM là danh sách thành phần của image. Có thể sinh bằng syft trên image, bằng công cụ CycloneDX cho .NET ở mức package, hoặc để Trivy xuất. Lưu SBOM cùng digest để khi một thư viện bị công bố lỗ hổng, bạn tra được image nào đang chứa nó mà không phải mở từng cái.

## Push bằng OIDC, ghi lại digest, khoá tag

`azure/login` đổi token OIDC lấy token Entra, rồi `az acr login` dùng phiên đó để lấy quyền docker với registry. Không có mật khẩu registry, không bật admin user. Identity cần đúng một role: AcrPush ở scope registry, và nó cũng kéo được. Nếu registry bật chế độ phân quyền theo repository (ABAC), AcrPush không còn hiệu lực và bạn cần role kiểu Container Registry Repository Writer; kiểm chế độ của `acrtoinoidev` trước. Phần còn lại của việc deploy không cần quyền này.

Điểm quan trọng nhất của bước push là digest. Sau `docker push`, job hỏi chính registry digest của tag vừa đẩy và ghi giá trị `sha256:...` vào output `image-digest`, thay vì đọc `RepoDigests` của Docker local vốn có thể chứa nhiều mục. Workflow CD ở [bài 44](/articles/azure-cd-environment-approval-blue-green-canary) nhận giá trị này qua input: hoặc `ci.yml` gọi `cd.yml` như reusable workflow và truyền `needs.image.outputs.image-digest`, hoặc người chạy dán vào khi kích hoạt tay. Ghép với tên registry, ví dụ `acrtoinoidev.azurecr.io/trips@sha256:...`, nó chính là giá trị `imageDigest` của Bicep trong bài 36 loạt C#. Nhờ vậy thứ được test, được quét và thứ được deploy là cùng một dãy byte, không có chỗ nào quay lại tag để tra.

Tag theo sha vẫn có thể bị ghi đè nếu có người push lại. Để tag không đổi sau khi push, khoá nó:

```bash
az acr repository update -n acrtoinoidev \
  --image trips:$GITHUB_SHA \
  --write-enabled false
```

Lệnh này khoá ghi cho một tag cụ thể, đúng như bài 21 mô tả. Nó không chứng minh identity của pipeline đủ quyền thực hiện; hãy chạy thử và đọc lỗi nếu có. Nếu AcrPush không đủ, đừng nâng pipeline lên Owner chỉ để khoá tag. Hãy xem tài liệu về role của registry và cân nhắc quyền hẹp nhất cho việc này.

Về cache layer: runner GitHub là máy mới mỗi lần chạy, nên `docker build` ở trên luôn build lại mọi layer. Nếu restore và publish làm pipeline chậm, chuyển sang `docker/build-push-action` với cache kiểu `gha` hoặc cache vào registry. Đổi lại bạn có thêm một nơi chứa layer cần dọn và cần tin. Chỉ làm khi số đo cho thấy cần.

## Chạy dưới 10 phút và để branch protection làm việc

Ba cơ chế giữ pipeline nhanh nằm ngay trong file: cache NuGet qua `setup-dotnet`, `concurrency` huỷ lần chạy cũ khi cùng pull request có commit mới, và path filter bỏ qua mọi thay đổi ngoài thư mục service. `timeout-minutes: 10` biến mục tiêu thành giới hạn cứng, để một lần treo không giữ runner cả giờ. Hãy xem thời gian từng bước trên trang run để biết chỗ nào đáng tối ưu trước khi đoán.

| Cơ chế | Chặn được | Cái giá |
| --- | --- | --- |
| Cache NuGet | Restore lặp lại mỗi lần chạy | Cần file `packages.lock.json`; khoá cache theo hash của nó |
| `concurrency` + cancel | Lãng phí khi pull request có commit mới | Lần chạy bị huỷ không cho kết quả; file trên chỉ huỷ với pull request, không huỷ lần push đang đẩy image |
| Path filter | Chạy CI khi không đổi gì liên quan | Gây kẹt required check, xem dưới |
| `timeout-minutes` | Job treo | Phải đặt dư so với thời gian thường |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Hai đường chạy của ci.yml: pull request chỉ chạy job test và bị branch protection chặn nếu đỏ; push vào main chạy tiếp job image và xuất digest cho CD."><defs><marker id="az43-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="100" rx="10" class="f-surface s-line"/><text x="34" y="55" class="t-b">Pull request</text><text x="34" y="85" class="t-small">Chỉ chạy job test</text><path d="M270 70 L315 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-1-a)"/><rect x="315" y="20" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="329" y="55" class="t-b">Job test</text><text x="329" y="85" class="t-small">Postgres thật + coverage</text><path d="M565 70 L610 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-1-a)"/><rect x="610" y="20" width="250" height="100" rx="10" class="f-warn-soft s-warn"/><text x="624" y="55" class="t-b">Branch protection</text><text x="624" y="85" class="t-small">Đỏ thì không merge</text><rect x="20" y="170" width="250" height="100" rx="10" class="f-surface s-line"/><text x="34" y="205" class="t-b">Push vào main</text><text x="34" y="235" class="t-small">Test rồi mới tới image</text><path d="M270 220 L315 220" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-1-a)"/><rect x="315" y="170" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="329" y="205" class="t-b">Job image</text><text x="329" y="235" class="t-small">Build, Trivy, push ACR</text><path d="M565 220 L610 220" class="f-none s-phys" stroke-width="2" marker-end="url(#az43-1-a)"/><rect x="610" y="170" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="624" y="205" class="t-b">Output digest</text><text x="624" y="235" class="t-small">CD bài 44 dùng lại</text></svg>
</div>
<figcaption>Hình 2. Hai đường chạy của ci.yml: pull request chỉ chạy job test và bị branch protection chặn nếu đỏ; push vào main chạy tiếp job image và xuất digest cho CD.</figcaption>
</figure>

Branch protection (hoặc ruleset) mới biến CI thành cổng thật: đặt job `test` là required status check cho nhánh main, thì pull request đỏ không merge được. Có một bẫy với path filter. Nếu workflow không chạy vì pull request không đụng thư mục service, check bắt buộc không bao giờ được báo và pull request kẹt ở trạng thái chờ. Hãy xử lý bằng cách cho workflow luôn chạy và lọc đường dẫn bên trong, hoặc dùng một job tổng hợp luôn chạy làm check bắt buộc duy nhất.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Đừng ghi log hay output chứa token registry. Đừng để job <code>image</code> chạy trên pull request từ fork rồi cấp cho nó quyền AcrPush. Và nhớ rằng ignore-unfixed là quyết định chấp nhận rủi ro: nếu không ai xem lại, danh sách lỗ hổng chưa vá sẽ phình lên mà pipeline vẫn xanh.</p>
</div>

## Ký image: bước tiếp theo, chưa bắt buộc

Digest chứng minh image không đổi sau khi build, nhưng không chứng minh image do pipeline của bạn tạo ra. Chữ ký làm việc thứ hai: pipeline ký digest bằng khoá giữ trong Key Vault, và bên deploy chỉ chấp nhận digest có chữ ký hợp lệ. Hai công cụ phổ biến là cosign và notation. ACR hỗ trợ lưu chữ ký cùng image, và bài 21 đã nhắc Microsoft hướng sang Notary Project cho phần này.

Phần khó là chính sách. Nền tảng có admission control (như AKS) có thể từ chối image chưa ký ngay lúc tạo pod. Theo hiểu biết lúc viết, Container Apps không có tầng tương đương, nên cách thực tế là một bước verify trong job CD trước khi đổi revision. Hãy kiểm tài liệu hiện hành của từng dịch vụ trước khi dựa vào một cơ chế cụ thể. Với Tới Nơi ở giai đoạn này, chỉ cần biết vị trí của nó trong chuỗi, chưa cần dựng.

## Lab: ci.yml đẩy được image, và PR hỏng bị chặn

Lab mô tả cách làm. Bạn cần `acrtoinoidev` từ bài 21, app `gh-toinoi-deploy` và các variable của environment `dev` từ bài 42. Trước hết cấp AcrPush cho service principal của app ở scope registry:

```bash
ACR_ID=$(az acr show -n acrtoinoidev --query id -o tsv)
az role assignment create \
  --assignee-object-id <sp-object-id> \
  --assignee-principal-type ServicePrincipal \
  --role AcrPush --scope $ACR_ID
```

Sau đó commit `ci.yml` vào `.github/workflows/` và push vào main. Kỳ vọng là job `test` xanh, job `image` xanh, và log của `docker push` in một dòng có `digest: sha256:...`. Đối chiếu giá trị đó với registry:

```bash
az acr repository show -n acrtoinoidev \
  --image trips:<commit-sha> \
  --query "{digest:digest,writeEnabled:changeableAttributes.writeEnabled}" -o json
```

Digest phải trùng với log. Lab này chưa chứng minh image chạy đúng, vì chưa có container nào khởi động từ nó; bước đó là việc của CD. Phần chặn merge: trong Settings của repository, đặt `test` là required check cho main. Rồi mở một nhánh, cố ý sửa một assertion trong test cho sai và mở pull request. Kỳ vọng là check `test` đỏ và nút merge bị khoá. Sửa lại assertion thì check xanh và merge mở. Dọn lab bằng `az acr repository delete -n acrtoinoidev --image trips:<commit-sha>`, lệnh này xoá cả manifest mà tag trỏ tới cùng mọi tag khác của nó (chỉ gỡ tag thì dùng `az acr repository untag`), rồi bỏ role AcrPush nếu không còn dùng.

**Bài tập:** đội bật required check `test` cho main và giữ path filter trên workflow. Một pull request chỉ sửa tài liệu ngoài thư mục service không merge được. Chuyện gì xảy ra và sửa thế nào?

<details><summary>Xem lời giải</summary><p>Workflow không chạy vì path filter loại pull request đó, nên check `test` không bao giờ được báo và GitHub chờ mãi. Cách sửa: bỏ path filter ở cấp workflow và lọc đường dẫn bên trong, hoặc thêm một job tổng hợp luôn chạy và đặt nó làm required check duy nhất, các job nặng bên dưới chạy có điều kiện.</p></details>

## Học tiếp từ đây

Bạn vừa mô tả cách chạy CI cho `trip-service`: test với PostgreSQL thật, build image gắn git sha, quét Trivy trước khi push, đẩy vào `acrtoinoidev` bằng OIDC và trả ra digest. Bài kế tiếp, [bài 44](/articles/azure-cd-environment-approval-blue-green-canary), nhận digest này để deploy qua environment có approval, rồi chia traffic giữa các revision theo kiểu blue/green và canary. Nếu cần xem lại cách đăng nhập không secret, đọc [bài 42](/articles/azure-github-actions-oidc-federated-credential), còn các khái niệm tag và digest nằm ở [bài 21](/articles/azure-container-registry-build-push-digest).

## Tài liệu gốc

- [Azure Container Registry documentation](https://learn.microsoft.com/en-us/azure/container-registry/)
- [Authenticate with an Azure container registry](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-authentication)
- [Lock a container image in Azure Container Registry](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-image-lock)
- [Sign container images with Notation and Azure Key Vault](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-tutorial-sign-build-push)
- [Deploy to Azure from GitHub Actions](https://learn.microsoft.com/en-us/azure/developer/github/connect-from-azure-openid-connect)
- [Introduction to Microsoft Defender for Containers](https://learn.microsoft.com/en-us/azure/defender-for-cloud/defender-for-containers-introduction)

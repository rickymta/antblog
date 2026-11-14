---
slug: azure-container-registry-build-push-digest
title: "Azure Container Registry: build, push, digest và ACR Tasks"
summary: "Bài 21: dựng acrtoinoidev, build image trips:1.0 từ trip-service bằng az acr build không cần Docker local, lấy digest để deploy theo @sha256, gán AcrPull cho id-toinoi-trips-dev và phân biệt tag, retention, token, quét lỗ hổng."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, container-registry, acr, docker]
language: vi
---

Đây là **bài 21** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 2/10 của phần).

Bài trước chạy container trên App Service. Container Apps và AKS ở các bài sau cũng cần cùng một thứ: một nơi chứa image mà runtime kéo về. Bài này dựng nơi đó bằng Azure Container Registry (ACR), build `trip-service` ngay trên cloud và lấy digest để deploy đúng từng byte. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hôm nay bạn push <code>trips:1.0</code>. Tuần sau bạn push một image khác với cùng tag <code>1.0</code>. Container App đang cấu hình <code>trips:1.0</code> có chạy bản mới ngay không?</p>
<details><summary>Xem đáp án</summary><p>Không ngay: replica đang chạy giữ bản đã kéo. Nhưng replica mới (scale out, restart, node mới) kéo lại theo tag và có thể nhận byte khác, nên cùng một deployment chạy hai bản khác nhau mà không ai ra lệnh. Digest không cho phép điều đó.</p></details>
</div>

## Registry là kho dùng chung, và tên của nó là toàn cầu

ACR là một registry theo chuẩn OCI. Tên registry thành hostname `<tên>.azurecr.io`, nên tên phải duy nhất trên toàn Azure và chỉ gồm chữ và số. Quy ước của Tới Nơi là `acrtoinoi<env>`, ví dụ `acrtoinoidev`. Nếu bị trùng, thêm hậu tố riêng của bạn. Nhiều đội dùng một registry cho cả ba môi trường để build một lần rồi promote. Tới Nơi tách theo môi trường cho dễ phân quyền, đổi lại phải copy image giữa các registry khi promote.

SKU quyết định tính năng, không chỉ dung lượng:

| SKU | Có gì | Dùng cho Tới Nơi |
| --- | --- | --- |
| Basic | Registry cơ bản, tài nguyên nhỏ | Học và thử nghiệm |
| Standard | Nhiều dung lượng và băng thông hơn Basic | Dev, staging (lab này) |
| Premium | Geo-replication, Private Link, content trust, các tính năng bảo vệ cho production | Prod, khi cần private endpoint hoặc bản sao ở East Asia |

Con số cụ thể về dung lượng và throughput thay đổi, hãy đọc trang SKUs hiện hành. Hai tính năng Premium đáng nói ngay. **Private Link** đưa registry vào mạng riêng, nối với [bài 17 về Private Endpoint](/articles/azure-private-endpoint-private-dns-service-endpoint). **Geo-replication** giữ bản sao ở nhiều region để pull gần runtime và sống sót khi mất region, và phải bật trước khi bạn cần chứ không sau sự cố. Content trust (Docker Content Trust) cũng thuộc Premium, nhưng đã bị đánh dấu deprecated; Microsoft hướng sang ký image bằng Notary Project. Hãy kiểm tra tài liệu trước khi xây quy trình ký.

Về đăng nhập: registry có sẵn một **admin user** với username/password dùng chung. Hãy tắt nó (`--admin-enabled false`). Mật khẩu chung không biết ai đã dùng, không thu hồi riêng cho từng người, và là bí mật nằm ngoài Key Vault. Thay vào đó, người dùng chạy `az acr login` (xác thực bằng Entra) và runtime dùng managed identity có role trên registry. Mọi lần kéo và đẩy đều có danh tính trong log.

## Tag trôi, digest thì không

Tag là nhãn trỏ tới một manifest, và nhãn đó gỡ ra gắn lại được. Digest là SHA-256 của chính manifest, nên đổi một byte sẽ ra digest khác. Registry không cho hai image khác nhau cùng digest.

| | Tag (`trips:1.0`) | Digest (`trips@sha256:...`) |
| --- | --- | --- |
| Bản chất | Nhãn, có thể trỏ sang image khác | Định danh theo nội dung |
| Ghi đè được không | Được, nếu registry cho phép | Không |
| Rollback về «bản cũ» | Không đảm bảo là cùng byte | Đúng byte đã chạy trước đó |
| Dễ đọc | Có | Không, cần công cụ |
| Dùng để | Build, nói chuyện giữa người với nhau | Deploy |

Đây là lý do Bicep ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment) nhận tham số `imageDigest` thay vì tag. Quy ước Tới Nơi: build đánh tag cho dễ đọc, deploy chỉ ghi digest. Cần nhớ giới hạn: digest chứng minh image không bị đổi sau khi build. Nó không chứng minh image an toàn hay không chứa lỗ hổng.

Image tích tụ theo thời gian và registry tính tiền theo dung lượng lưu. Có hai việc cần làm. Thứ nhất là dọn: lệnh `acr purge` chạy qua `az acr run` xoá theo tuổi và bộ lọc, và bản retention policy cho manifest không còn tag là tính năng của Premium. Thứ hai là khoá những image đang chạy:

```bash
PURGE="acr purge --filter 'trips:.*' --ago 30d --keep 10 --untagged --dry-run"
az acr run -r acrtoinoidev --cmd "$PURGE" /dev/null

az acr repository update -n acrtoinoidev --image trips:1.0 \
  --write-enabled false --delete-enabled false
```

Cờ `--dry-run` chỉ liệt kê những gì sẽ bị xoá. Khối này không chứng minh danh sách đó an toàn, vì purge theo tuổi không hề biết image nào đang chạy ở production. Hãy đối chiếu danh sách với digest trong cấu hình deploy trước khi bỏ `--dry-run`. Image bị khoá ghi và xoá sẽ không bị purge động tới, nhưng khoá cũng chặn chính bạn push lại cùng tag.

## ACR Tasks build trên cloud, không cần Docker local

Lệnh `az acr build` gửi thư mục nguồn lên ACR, build bằng Dockerfile ở đó trên hạ tầng của Azure rồi đẩy kết quả vào registry. Bạn không cần Docker daemon trên máy, và image không đi qua mạng của bạn hai lần. Lệnh này là một **quick task**: chạy một lần, theo yêu cầu. Một số loại subscription hạn chế ACR Tasks. Nếu lệnh báo lỗi về điều này, hãy build local rồi push, và kiểm trang Tasks overview.

Quick task khác **task** theo kích hoạt:

| | Quick task (`az acr build`) | Task (`az acr task create`) |
| --- | --- | --- |
| Khi nào chạy | Bạn gõ lệnh | Commit, base image đổi hoặc lịch |
| Nguồn | Thư mục local hoặc git URL | Git repo |
| Lưu cấu hình | Không | Có, task là resource |
| Dùng cho | Thử nhanh, lab | Build lặp lại có kiểm soát |

Base image trigger đáng giá nhất. Dockerfile của trip-service dựa trên `mcr.microsoft.com/dotnet/aspnet:10.0`. Khi base image được vá, một task có trigger này tự build lại và sinh digest mới mà bạn không cần đụng code. Điều đó tốt cho bản vá bảo mật, nhưng cũng nghĩa là digest mới nhất của registry không còn là digest đã kiểm. Việc promote lên production vẫn phải là quyết định của bạn. Tài liệu nêu rõ registry nào của base image được theo dõi, hãy kiểm với trường hợp của bạn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Từ thư mục nguồn trip-service tới image trong ACR: build trên cloud, tag để đọc, digest để deploy."><defs><marker id="az21-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-surface s-line"/><text x="34" y="95" class="t-b">Thư mục nguồn</text><text x="34" y="123" class="t-small">Dockerfile + 5 project</text><path d="M205 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az21-0-a)"/><rect x="235" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="249" y="95" class="t-b">ACR Tasks</text><text x="249" y="123" class="t-small">Build trên cloud</text><path d="M420 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az21-0-a)"/><rect x="450" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="464" y="95" class="t-b">acrtoinoidev</text><text x="464" y="123" class="t-small">Tag trips:1.0 + digest</text><path d="M635 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az21-0-a)"/><rect x="665" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="679" y="95" class="t-b">Runtime kéo</text><text x="679" y="123" class="t-small">Theo @sha256, cần AcrPull</text></svg>
</div>
<figcaption>Hình 1. Từ thư mục nguồn trip-service tới image trong ACR: build trên cloud, tag để đọc, digest để deploy.</figcaption>
</figure>

Khối dưới tạo một task theo commit và theo base image. Tag dùng `{{.Run.ID}}` nên mỗi lần build có tag riêng, không ghi đè nhau. Biến `$GIT_PAT` là token của repository, đặt trong môi trường, không đưa vào source. Commit trigger cần ACR tạo webhook trên repository, nên token phải có thêm quyền đó chứ không chỉ đọc code; tra đúng scope tối thiểu trong tài liệu ACR Tasks:

```bash
az acr task create -r acrtoinoidev -n build-trips \
  --image 'trips:{{.Run.ID}}' \
  --context '<git-repo-url>#main:examples/toi-noi-dotnet/31-38-trip-service' \
  --file Dockerfile \
  --git-access-token "$GIT_PAT" \
  --commit-trigger-enabled true \
  --base-image-trigger-enabled true
```

Khối này không chứng minh task chạy đúng. Bạn chỉ biết sau khi push một commit và xem `az acr task list-runs`. Nó cũng không quyết định image nào được lên production.

## AcrPull cho runtime, AcrPush cho pipeline

Quyền trên registry chia theo việc, và Tới Nơi cấp theo nguyên tắc người kéo không được đẩy:

| Danh tính | Role | Làm được |
| --- | --- | --- |
| `id-toinoi-trips-dev` (runtime) | AcrPull | Chỉ kéo image |
| Pipeline CI (Phần 5) | AcrPush | Đẩy image, và cũng kéo được |
| Người vận hành | Role quản trị registry theo nhu cầu | Cấu hình, xoá |

Vì sao tách? AcrPush cho phép ghi đè tag. Nếu runtime có AcrPush và bị chiếm quyền, kẻ tấn công thay image bằng bản của họ rồi chờ replica tiếp theo kéo về. Với AcrPull, runtime bị chiếm vẫn không sửa được image. Registry mới tạo gần đây có thể dùng chế độ phân quyền theo repository với role khác. Hãy kiểm chế độ của registry bạn tạo trong tài liệu roles.

Có một nhóm không dùng Entra: công cụ hay đối tác chỉ hỗ trợ username/password. Với họ có **token và scope map**. Scope map liệt kê repository và hành động được phép, ví dụ chỉ `content/read` trên `trips`. Token gắn với một scope map, có mật khẩu riêng thu hồi được độc lập, và hết hạn được. Đây là lối thoát hẹp, tốt hơn nhiều so với admin user.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Ba danh tính vào registry với quyền hẹp khác nhau: pipeline đẩy, runtime kéo, token scope map chỉ đọc; admin user bị tắt."><defs><marker id="az21-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="15" width="300" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="47" class="t-b">Pipeline CI</text><text x="34" y="75" class="t-small">AcrPush: đẩy image</text><rect x="20" y="110" width="300" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="142" class="t-b">id-toinoi-trips-dev</text><text x="34" y="170" class="t-small">AcrPull: chỉ kéo</text><rect x="20" y="205" width="300" height="80" rx="10" class="f-surface s-line"/><text x="34" y="237" class="t-b">Token cho đối tác</text><text x="34" y="265" class="t-small">Scope map: content/read</text><path d="M320 55 L540 85" class="f-none s-phys" stroke-width="2" marker-end="url(#az21-1-a)"/><path d="M320 150 L540 110" class="f-none s-phys" stroke-width="2" marker-end="url(#az21-1-a)"/><path d="M320 245 L540 135" class="f-none s-phys" stroke-width="2" marker-end="url(#az21-1-a)"/><rect x="540" y="30" width="320" height="140" rx="10" class="f-phys-soft s-phys"/><text x="554" y="68" class="t-b">acrtoinoidev</text><text x="554" y="96" class="t-small">Chỉ nhận Entra hoặc token</text><text x="554" y="124" class="t-small">Mọi lần kéo, đẩy có danh tính</text><rect x="540" y="205" width="320" height="80" rx="10" class="f-dead s-dead"/><text x="554" y="237" class="t-b t-dead">Admin user</text><text x="554" y="265" class="t-small t-dead">Tắt, mật khẩu dùng chung</text></svg>
</div>
<figcaption>Hình 2. Ba danh tính vào registry với quyền hẹp khác nhau: pipeline đẩy, runtime kéo, token scope map chỉ đọc; admin user bị tắt.</figcaption>
</figure>

## Quét lỗ hổng không chặn deploy

Defender for Containers quét image trong ACR để tìm lỗ hổng đã biết và đưa kết quả vào Defender for Cloud dưới dạng khuyến nghị, cùng chỗ với [secure score ở bài 16](/articles/azure-defender-for-cloud-secure-score). Quét chạy khi image được đẩy và định kỳ với image còn được dùng, theo mô hình tính tiền riêng của Defender. Hãy đọc tài liệu để biết chính xác chu kỳ và phạm vi.

Hai điểm cần nhớ. Một: cơ sở dữ liệu lỗ hổng cập nhật liên tục, nên một digest sạch hôm nay có thể bị báo lỗi tuần sau mà không đổi một byte. Đó là lý do «đã deploy» không có nghĩa là «xong». Hai: việc quét tự nó không chặn ai kéo hay deploy image có lỗ hổng. Nó là tín hiệu. Muốn chặn, bạn phải đặt cổng trong pipeline, và đó là việc của Phần 5 về CI.

## Lab: build trips:1.0, lấy digest, gán AcrPull

Lab mô tả cách làm trên subscription của bạn. Khối đầu tạo registry (không bật admin user) và build image từ thư mục service. Chạy từ gốc repository để context trỏ đúng thư mục có Dockerfile:

```bash
RG=rg-toinoi-trips-dev-sea
ACR=acrtoinoidev
az acr create -g $RG -n $ACR -l southeastasia \
  --sku Standard --admin-enabled false
az acr build -r $ACR -t trips:1.0 --platform linux/amd64 \
  examples/toi-noi-dotnet/31-38-trip-service
```

Kỳ vọng: log build chạy `dotnet restore`, `dotnet publish` rồi push, kết thúc bằng thông tin run thành công. Tệp `.dockerignore` trong thư mục được dùng để loại phần thừa khỏi context. Khối này không chứng minh image chạy đúng, vì chưa có container nào khởi động. Bước đó là việc của bài 20 hoặc bài 22.

Khối tiếp theo lấy digest. Đây là giá trị bạn truyền cho `imageDigest` của Bicep và cho biến `IMAGE` ở bài 20:

```bash
DIGEST=$(az acr repository show -n $ACR --image trips:1.0 \
  --query digest -o tsv)
echo "$ACR.azurecr.io/trips@$DIGEST"
az acr manifest list-metadata -r $ACR -n trips \
  --query "[].{digest:digest,tags:tags,pushed:createdTime}" -o table
```

Lệnh thứ hai liệt kê mọi manifest của repository `trips` cùng tag. Sau một lần push lại cùng tag, bạn sẽ thấy digest cũ mất tag và trở thành manifest không tag, bằng chứng rằng tag đã trôi còn digest cũ thì vẫn còn đó. Cuối cùng, gán AcrPull cho identity ở bài 13:

```bash
ACR_ID=$(az acr show -n $ACR --query id -o tsv)
PRINCIPAL_ID=$(az identity show -g $RG -n id-toinoi-trips-dev \
  --query principalId -o tsv)
az role assignment create --assignee-object-id $PRINCIPAL_ID \
  --assignee-principal-type ServicePrincipal \
  --role AcrPull --scope $ACR_ID
az role assignment list --assignee $PRINCIPAL_ID --scope $ACR_ID \
  --query "[].roleDefinitionName" -o tsv
```

Dòng cuối chỉ chứng minh role assignment tồn tại. Nó chưa chứng minh runtime kéo được image: role mới có thể mất vài phút để có hiệu lực, và runtime còn phải được cấu hình dùng đúng identity (`registries[].identity` trong Bicep bài 36, hoặc `acrUserManagedIdentityID` ở bài 20). Phép thử thật là một lần kéo từ runtime.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Registry tính tiền theo SKU và dung lượng lưu, kể cả khi không ai kéo image. Lab này không cần giữ lại sau khi xong: xoá cả resource group lab bằng <code>az group delete -n rg-toinoi-trips-dev-sea</code>, và chỉ chạy khi chắc đây đúng resource group lab. Đừng xoá bằng <code>acr purge</code> khi chưa xem dry-run: image đang chạy ở production mà bị xoá sẽ khiến replica mới không kéo được.</p>
</div>

**Bài tập:** (1) Bạn muốn xoá mọi image cũ hơn 30 ngày, nhưng production đang chạy digest của một image 45 ngày tuổi. Purge theo tuổi sẽ ảnh hưởng gì? (2) Có người đề nghị cấp AcrPush cho `id-toinoi-trips-dev` «cho tiện, vì AcrPush gồm cả kéo». Có nên không?

<details><summary>Xem lời giải</summary><p>(1) Purge theo tuổi không biết image nào đang chạy, nên sẽ xoá manifest đó. Replica hiện tại vẫn sống, nhưng scale out hoặc restart sẽ không kéo được image và bị kẹt ở lỗi pull. Cách tránh: chạy dry-run, đối chiếu với digest trong cấu hình deploy, và khoá image đang chạy bằng cờ delete-enabled. (2) Không. AcrPush đúng là gồm kéo, nhưng nó cho phép ghi đè tag, nên runtime bị chiếm quyền có thể thay image. Runtime chỉ cần AcrPull.</p></details>

## Học tiếp từ đây

Bạn vừa mô tả cách dựng `acrtoinoidev`, build `trips:1.0` bằng `az acr build`, lấy digest và gán AcrPull cho `id-toinoi-trips-dev`, đồng thời hiểu vì sao deploy phải theo digest và vì sao runtime không nên có quyền đẩy. Quét lỗ hổng và cổng chặn sẽ quay lại ở Phần 5.

Bài sau dùng chính digest này: [Bài 22](/articles/azure-container-apps-revision-ingress-scale) dựng Container Apps environment, revision, ingress và scale theo KEDA cho `trip-service`. Nếu muốn xem lại nơi image vừa được chạy, đọc [bài 20](/articles/azure-app-service-deploy-slot-scale) về App Service.

## Tài liệu gốc

- [Azure Container Registry documentation](https://learn.microsoft.com/en-us/azure/container-registry/)
- [Azure Container Registry service tiers](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-skus)
- [ACR Tasks overview](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-tasks-overview)
- [Authenticate with an Azure container registry](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-authentication)
- [Azure Container Registry roles and permissions](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-roles)
- [Introduction to Microsoft Defender for Containers](https://learn.microsoft.com/en-us/azure/defender-for-cloud/defender-for-containers-introduction)

---
slug: azure-container-apps-revision-ingress-scale
title: "Azure Container Apps: environment, revision, ingress và scale theo KEDA"
summary: "Bài 22: đưa trip-service của Tới Nơi lên cae-toinoi-dev bằng main.bicep, hiểu environment, revision, ingress và scale rule KEDA, rồi đọc log, scale theo 50 request đồng thời và đổi image digest để tạo revision mới."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, container-apps, keda, bicep, containers]
language: vi
---

Đây là **bài 22** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 3/10 của phần).

Bài này giải thích bốn khái niệm quyết định cách Container Apps vận hành (environment, revision, ingress, scale rule), rồi triển khai `main.bicep` của `trip-service` lên một environment dev. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đặt <code>minReplicas: 0</code> thì container app của bạn có «miễn phí» khi không ai gọi không?</p>
<details><summary>Xem đáp án</summary><p>Không hẳn. Khi không có replica nào chạy thì phần compute thôi tính tiền, nhưng request đầu tiên sau đó phải chờ một replica khởi động. Environment, Log Analytics và các resource đi kèm vẫn tồn tại và có thể tính tiền theo cách riêng.</p></details>
</div>

## Environment là ranh giới mạng và log, không phải ranh giới ứng dụng

Một **managed environment** là lớp nền mà nhiều container app cùng chạy trên đó. Mọi app trong một environment dùng chung một VNet (hoặc VNet do Azure quản lý nếu bạn không chỉ định), cùng một nơi gửi log (thường là một Log Analytics workspace) và nhìn thấy nhau bằng tên nội bộ. Tới Nơi dùng `cae-toinoi-<env>`: một environment cho mỗi môi trường dev, staging, prod, và các service Booking, Dispatch, Trip, Payment đều là container app riêng bên trong.

Hệ quả thực tế của việc gom chung: quyết định mạng được đưa ra ở cấp environment, lúc tạo. Muốn container app gọi được PostgreSQL qua private endpoint thì subnet của environment phải đi được tới đó. Chọn sai VNet lúc tạo là chuyện khó sửa về sau, nên bài 27 sẽ quay lại phần mạng; ở đây chỉ cần nhớ environment là nơi bạn đưa ra quyết định đó.

Environment còn có khái niệm **workload profile**. Profile **Consumption** chạy theo mô hình serverless: Azure xếp replica lên hạ tầng chung, bạn trả theo tài nguyên mà replica sử dụng. Profile **Dedicated** dành riêng một nhóm máy cho bạn, hợp với workload cần ổn định hoặc kích thước lớn, và bạn trả cho cả nhóm máy dù replica ít hay nhiều. Tới Nơi bắt đầu bằng Consumption cho cả ba môi trường. Chỉ cân nhắc Dedicated khi số liệu đo được cho thấy việc đó cần thiết. Dapr (sidecar cho service invocation, pub/sub) là tính năng tuỳ chọn bật theo từng app. Tới Nơi chưa cần vì đã có outbox và HTTP thường ở bài 26 loạt C#.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Environment cae-toinoi-dev chứa nhiều container app và dùng chung VNet cùng Log Analytics."><rect x="20" y="15" width="840" height="290" rx="14" class="f-surface s-line"/><text x="40" y="45" class="t-b">cae-toinoi-dev (managed environment)</text><rect x="40" y="65" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="54" y="95" class="t-b">ca-toinoi-trips-dev</text><text x="54" y="123" class="t-small">Revision, ingress, scale</text><rect x="315" y="65" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="329" y="95" class="t-b">ca-toinoi-dispatch-dev</text><text x="329" y="123" class="t-small">Gọi nhau bằng tên nội bộ</text><rect x="590" y="65" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="604" y="95" class="t-b">ca-toinoi-payment-dev</text><text x="604" y="123" class="t-small">Ingress internal</text><rect x="40" y="185" width="390" height="95" rx="10" class="f-phys-soft s-phys"/><text x="54" y="215" class="t-b">VNet + subnet</text><text x="54" y="243" class="t-small">Đường tới PostgreSQL, Key Vault</text><rect x="450" y="185" width="390" height="95" rx="10" class="f-phys-soft s-phys"/><text x="464" y="215" class="t-b">Log Analytics workspace</text><text x="464" y="243" class="t-small">Log console và system của mọi app</text></svg>
</div>
<figcaption>Hình 1. Environment cae-toinoi-dev chứa nhiều container app và dùng chung VNet cùng Log Analytics.</figcaption>
</figure>

## Revision là bản chụp bất biến của template

Mỗi lần bạn đổi **template** của container app (image, biến môi trường, CPU/memory, scale rule, probe), Azure tạo một **revision** mới. Revision không sửa được sau khi tạo. Đây là lý do rollback ở Container Apps rẻ: revision cũ vẫn còn đó, chỉ cần chuyển traffic về.

Cần phân biệt hai tầng cấu hình. Thay đổi ở **template** sinh revision mới. Thay đổi ở **configuration** (ingress, secret, danh sách registry, revision mode) áp dụng cho app và không sinh revision. Hậu quả dễ vấp: đổi giá trị secret không tự làm replica đang chạy nhận giá trị mới. Với secret trỏ Key Vault không kèm version, nền tảng định kỳ lấy version mới, nhưng biến môi trường chỉ được nạp lúc replica khởi động, nên vẫn cần restart revision hoặc tạo revision mới. Cùng bài học với bài 36 loạt C#: đừng mặc định rằng cấu hình sẽ tự hot reload.

Có hai **revision mode**:

| Tiêu chí | Single | Multiple |
| --- | --- | --- |
| Revision nhận traffic | Một, bản mới nhất ở trạng thái healthy | Nhiều, chia theo phần trăm |
| Khi đổi template | Revision mới thay revision cũ sau khi sẵn sàng | Revision mới được tạo, bạn tự quyết traffic |
| Hợp với | Dev, staging, service đơn giản | Canary, blue/green, rollback có kiểm soát |
| Bạn phải tự dọn | Không | Có, revision cũ vẫn chạy và tính tiền cho tới khi tắt |

`main.bicep` của `trip-service` dùng `activeRevisionsMode: 'Single'`, hợp với lab. Ở chế độ Multiple, bạn gán **label** (tên cố định như `canary`) cho một revision để có URL riêng kiểm thử trước khi chuyển traffic, và dùng `az containerapp ingress traffic set --revision-weight` để chia phần trăm. Bài 5 của Phần 5 đi sâu vào canary; ở đây chỉ cần biết cơ chế nằm sẵn trong revision.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Mỗi replica của <code>trip-service</code> mở connection pool riêng tới PostgreSQL. Nếu <code>maxReplicas</code> nhân với kích thước pool vượt giới hạn connection của <code>psql-toinoi-dev-sea</code>, lúc tải tăng bạn gặp lỗi kết nối chứ không có thêm công suất. Ở chế độ Multiple, revision cũ còn chạy cũng giữ connection và tính tiền. Đặt <code>maxReplicas</code> dựa trên số liệu đo được, và tắt revision cũ khi đã chuyển hết traffic.</p>
</div>

## Ingress quyết định ai gọi được app

Ingress là cổng HTTP vào container app. Bốn thứ cần quyết định:

- **External hay internal.** External cho phép gọi từ Internet. Internal chỉ nhận traffic từ bên trong environment. Trong Tới Nơi, chỉ API nhận request từ app khách hàng và tài xế mới cần external; Payment, Dispatch, Notification nên internal.
- **Target port.** Cổng mà container thực sự lắng nghe. Bicep của bài đặt `targetPort: 8080`, khớp `ASPNETCORE_HTTP_PORTS`. Sai cổng không báo lỗi lúc deploy, mà lộ ra dưới dạng probe fail hoặc 502 lúc chạy.
- **TLS.** Với domain mặc định của environment, nền tảng cấp sẵn chứng chỉ và `allowInsecure: false` buộc HTTP chuyển sang HTTPS. Domain riêng và chứng chỉ riêng thuộc bài 28.
- **IP restriction.** Bạn có thể giới hạn dải IP gọi vào, ví dụ chỉ cho IP của Application Gateway ở bài 27. Đây là lớp lọc thô, không thay cho xác thực JWT.

Service-to-service đi theo tên: app khác trong cùng environment gọi `http://ca-toinoi-trips-dev` mà không qua Internet. Lưu ý đường nội bộ này không tự có xác thực giữa các service. Dù người gọi đã tới được app thì vẫn phải qua bước kiểm tra token như bài 19 loạt C#.

## Scale rule: KEDA đo tải rồi quyết định số replica

Container Apps dùng **KEDA** để scale. Bạn khai báo một hoặc nhiều **scale rule**; KEDA đo nguồn tải tương ứng, rồi tăng hoặc giảm replica trong khoảng `minReplicas` tới `maxReplicas`. Khi có nhiều rule, rule nào đòi nhiều replica nhất sẽ thắng.

| Rule | Đo cái gì | Scale về 0 được không | Hợp với service nào |
| --- | --- | --- | --- |
| HTTP | Số request đồng thời mỗi replica | Được | API nhận request, như trip-service |
| CPU / memory | Mức dùng CPU hoặc bộ nhớ | Không | Worker tính toán nặng |
| Service Bus queue | Độ dài queue hoặc số message chờ | Được | Notification, consumer của outbox |

Hai điểm cần hiểu kỹ. Một là **min replica**. `minReplicas: 0` tiết kiệm khi nhàn rỗi nhưng gây **cold start**: request đầu tiên chờ pull image, khởi động .NET, qua startup probe. Với endpoint đặt xe, người dùng thấy độ trễ đó. Bicep của bài đặt `minReplicas: 1` để tránh. Hai là **max replica**: giá trị này là trần chi phí và cũng là trần số connection tới database như callout ở trên.

Rule HTTP đặt ngưỡng số request đồng thời. Với ngưỡng 50, khi trung bình mỗi replica phải xử lý quá 50 request cùng lúc thì KEDA thêm replica. Con số 50 trong lab chỉ là điểm bắt đầu để quan sát cơ chế. Ngưỡng thật phải lấy từ load test, vì một request đặt xe chờ PostgreSQL khác hẳn một request đọc cache.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Ingress chia traffic giữa revision cũ và revision mới, trong khi rule HTTP của KEDA quyết định số replica."><defs><marker id="az22-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="210" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="135" class="t-b">Ingress HTTPS</text><text x="34" y="163" class="t-small">Cổng 8080, chia traffic</text><path d="M230 140 L300 80" class="f-none s-phys" stroke-width="2" marker-end="url(#az22-1-a)"/><path d="M230 160 L300 220" class="f-none s-phys" stroke-width="2" marker-end="url(#az22-1-a)"/><rect x="300" y="40" width="270" height="85" rx="10" class="f-pk-soft s-pk"/><text x="314" y="70" class="t-b">Revision cũ</text><text x="314" y="98" class="t-small">Image digest A, 90% traffic</text><rect x="300" y="175" width="270" height="85" rx="10" class="f-warn-soft s-warn"/><text x="314" y="205" class="t-b">Revision mới</text><text x="314" y="233" class="t-small">Image digest B, 10% traffic</text><path d="M570 82 L640 140" class="f-none s-phys" stroke-width="2" marker-end="url(#az22-1-a)"/><path d="M570 218 L640 160" class="f-none s-phys" stroke-width="2" marker-end="url(#az22-1-a)"/><rect x="640" y="105" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="654" y="135" class="t-b">KEDA, rule HTTP 50</text><text x="654" y="163" class="t-small">Min 1, max 3 replica</text></svg>
</div>
<figcaption>Hình 2. Ingress chia traffic giữa revision cũ và revision mới, trong khi rule HTTP của KEDA quyết định số replica.</figcaption>
</figure>

## Secret và probe đi theo identity và endpoint của service

Bicep của bài không chứa mật khẩu. Block `secrets` trỏ tới Key Vault bằng `keyVaultUrl` kèm `identity`, và biến `ConnectionStrings__Trips` lấy giá trị qua `secretRef`. Identity là `id-toinoi-trips-<env>` ở bài 13. Identity này cần quyền đọc secret trong `kv-toinoi-dev-sea` (với Key Vault dùng RBAC thì thường là role Key Vault Secrets User) và quyền `AcrPull` trên registry. Thiếu một trong hai, deployment hoặc revision đầu tiên thất bại vì không lấy được secret hay image. User-assigned identity được chọn chính vì có thể cấp quyền trước khi app tồn tại; system-assigned chỉ có sau khi app được tạo.

Ba probe lấy từ bài 36 loạt C#. **Startup** dành cho .NET một khoảng thời gian khởi động riêng. **Liveness** chỉ hỏi `/health/live`, nghĩa là process còn trả lời. **Readiness** hỏi `/health/ready`, kiểm cả database và migration. Không đưa truy vấn DB vào liveness, vì khi PostgreSQL gặp sự cố bạn không muốn nền tảng restart mọi replica cùng lúc.

## Lab: triển khai trip-service lên cae-toinoi-dev

Lab giả định bạn đã có resource group `rg-toinoi-trips-dev-sea`, identity và quyền ở bài 13, secret chuỗi kết nối trong Key Vault ở bài 14, và image đã push vào `acrtoinoidev` ở bài 21 (cần hai digest khác nhau, A và B, ví dụ từ hai lần build). Đầu tiên tạo environment. Lệnh này thường tự tạo một Log Analytics workspace nếu bạn không chỉ định; kiểm tra `az containerapp env create --help` cho các tham số mạng.

```bash
RG=rg-toinoi-trips-dev-sea

az containerapp env create \
  --name cae-toinoi-dev \
  --resource-group $RG \
  --location southeastasia
```

Lệnh trên mới dựng nền, chưa có app nào và chưa chứng minh environment tới được PostgreSQL. Bước tiếp theo triển khai Bicep có sẵn trong repository, truyền digest A.

```bash
ENV_ID=$(az containerapp env show -g $RG -n cae-toinoi-dev --query id -o tsv)
IDENTITY_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query id -o tsv)

az deployment group create \
  --resource-group $RG \
  --template-file examples/toi-noi-dotnet/31-38-trip-service/cloud/main.bicep \
  --parameters appName=ca-toinoi-trips-dev \
    environmentResourceId=$ENV_ID \
    identityResourceId=$IDENTITY_ID \
    registryServer=acrtoinoidev.azurecr.io \
    imageDigest=acrtoinoidev.azurecr.io/trips@sha256:<digest-a> \
    postgresSecretUrl=https://kv-toinoi-dev-sea.vault.azure.net/secrets/Postgres--Trips \
    authAuthority=<https-authority> authAudience=<audience>
```

Kỳ vọng: deployment thành công và output `fqdn` trả về tên miền của app. Nếu thất bại, nguyên nhân thường gặp là thiếu quyền của identity trên Key Vault hoặc ACR, hoặc PostgreSQL không tới được từ subnet. Compile Bicep thành công không chứng minh những điều đó. Đọc log để xem app có khởi động được không.

```bash
az containerapp logs show -g $RG -n ca-toinoi-trips-dev --follow --tail 30
az containerapp logs show -g $RG -n ca-toinoi-trips-dev --type system --tail 30
```

Log mặc định là log console của container. Với `--type system`, bạn thấy log của nền tảng như pull image, probe fail, replica được thêm. Hai nguồn trả lời hai câu hỏi khác nhau: app nói gì, và nền tảng đã làm gì với app.

Tiếp theo thêm scale rule HTTP với ngưỡng 50 request đồng thời. `main.bicep` hiện chưa khai báo rule nào, nên lệnh `az` dưới đây là thao tác thử nghiệm. Sau khi hài lòng, hãy đưa rule vào Bicep. Nếu không, lần deploy Bicep kế tiếp sẽ ghi đè và làm rule biến mất.

```bash
az containerapp update -g $RG -n ca-toinoi-trips-dev \
  --min-replicas 1 --max-replicas 3 \
  --scale-rule-name http-50 \
  --scale-rule-type http \
  --scale-rule-http-concurrency 50
```

Lệnh này tự nó sinh một revision mới vì scale thuộc template. Để quan sát, gửi hơn 50 request song song vào một endpoint (một vòng lặp `curl` chạy nền là đủ), rồi liệt kê replica. Kỳ vọng là số replica tăng tới tối đa 3, nhưng độ trễ scale phụ thuộc chu kỳ đo của KEDA nên có thể mất vài chục giây.

```bash
az containerapp replica list -g $RG -n ca-toinoi-trips-dev -o table
```

Cuối cùng, đổi image sang digest B để thấy revision mới. Digest là định danh bất biến của image (bài 21), nên đây là cách chắc chắn nhất để nói «đúng bản này đang chạy».

```bash
az containerapp update -g $RG -n ca-toinoi-trips-dev \
  --image acrtoinoidev.azurecr.io/trips@sha256:<digest-b>

az containerapp revision list -g $RG -n ca-toinoi-trips-dev \
  --query "[].{name:name, active:properties.active, traffic:properties.trafficWeight}" -o table
```

Kỳ vọng: xuất hiện thêm một revision và ở chế độ Single, traffic chuyển sang revision mới khi nó qua readiness probe. Nếu readiness không bao giờ xanh thì revision mới không nhận traffic và bản cũ tiếp tục phục vụ, nhưng bài này không chứng minh điều đó bằng một lần chạy thật. Dọn lab bằng `az group delete` cho resource group dev khi xong, và chỉ làm vậy khi nhóm này không chứa tài nguyên bạn còn cần.

**Bài tập:** bạn đổi biến `Auth__Audience` và đổi `targetPort` trong ingress. Việc nào tạo revision mới, việc nào không, và vì sao?

<details><summary>Xem lời giải</summary><p>Đổi biến môi trường sửa template nên tạo revision mới. Đổi ingress sửa configuration nên áp dụng ngay cho app mà không tạo revision. Vì vậy một thay đổi ingress sai có thể ảnh hưởng mọi revision đang nhận traffic cùng lúc, và không có revision cũ nào để rollback theo kiểu đổi traffic.</p></details>

## Học tiếp từ đây

Bạn đã đưa `trip-service` lên Container Apps, biết environment gom mạng và log ở đâu, revision sinh ra từ thay đổi nào, ingress đặt theo ai gọi ai, và scale rule gắn với chi phí lẫn số connection database. Container Apps ẩn đi gần hết phần vận hành cluster. Khi cần CRD, operator, sidecar tuỳ biến hoặc kiểm soát mạng sâu hơn, bạn phải trả giá bằng việc vận hành Kubernetes. [Bài 23](/articles/azure-aks-co-ban-cluster-node-pool-deploy) dựng một cluster AKS nhỏ và triển khai cùng service đó bằng manifest. Nếu cần ôn phần image, xem lại [bài 21](/articles/azure-container-registry-build-push-digest) về digest và ACR, hoặc [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment) về Bicep và probe.

## Tài liệu gốc

- [Azure Container Apps overview](https://learn.microsoft.com/en-us/azure/container-apps/overview)
- [Azure Container Apps environments](https://learn.microsoft.com/en-us/azure/container-apps/environment)
- [Revisions in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/revisions)
- [Set scaling rules in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/scale-app)
- [Ingress in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/ingress-overview)
- [Container Apps Bicep schema](https://learn.microsoft.com/en-us/azure/templates/microsoft.app/containerapps)

---
slug: azure-app-service-deploy-slot-scale
title: "App Service: deploy API .NET, deployment slot và scale"
summary: "Bài 20: tạo asp-toinoi-dev-sea (P0v3 Linux), chạy container trip-service từ ACR bằng managed identity, bật health check /health/ready, thử slot staging, swap và rollback, rồi chỉnh scale để biết App Service đủ dùng tới đâu cho Tới Nơi."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, app-service, deployment-slot, scaling]
language: vi
---

Đây là **bài 20** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 1/10 của phần).

Phần 3 gồm 10 bài đưa `trip-service` từ máy ảo ở [bài 7](/articles/azure-virtual-machine-linux-chay-api) lên những nền tảng phù hợp hơn: App Service, Container Apps, AKS, Functions, rồi chọn đúng chỗ cho từng service. Bài này mở đầu bằng App Service, nơi bạn gánh ít việc vận hành nhất để chạy một API .NET đóng gói thành container. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai web app cùng nằm trong một App Service plan. Bạn xoá một app. Hoá đơn của plan có giảm không?</p>
<details><summary>Xem đáp án</summary><p>Không. Plan tính tiền theo SKU và số instance, không theo số app. Tiền chỉ giảm khi bạn đổi SKU thấp hơn, giảm số instance hoặc xoá plan.</p></details>
</div>

## Plan là nơi tính tiền, web app chỉ thuê chỗ trên đó

App Service có hai loại resource tách biệt. **App Service plan** là một nhóm máy do Azure quản lý, với hệ điều hành, SKU và số instance bạn chọn. **Web app** là cấu hình và code chạy trên các instance đó. Mọi app trong plan, kể cả slot, chia nhau CPU và RAM của cùng những instance ấy.

Hậu quả có hai mặt. Mặt tiền: plan chạy là tính tiền, dù app bên trong đã dừng hay chưa nhận request nào. Mặt hiệu năng: một app ngốn RAM sẽ làm chậm những app cùng plan, và không có ranh giới tài nguyên nào chặn việc đó. Với Tới Nơi, dev có thể gom `trip-service` cùng vài service nhỏ vào một plan. Production nên tách plan theo mức tải.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Plan là đơn vị tính tiền và cấp tài nguyên; web app và slot staging chạy chung instance của plan."><defs><marker id="az20-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="34" y="58" class="t-b">App Service plan</text><text x="34" y="86" class="t-small">Tính tiền theo SKU, instance</text><path d="M280 72 L310 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az20-0-a)"/><rect x="310" y="25" width="260" height="95" rx="10" class="f-pk-soft s-pk"/><text x="324" y="58" class="t-b">app-toinoi-trips-dev</text><text x="324" y="86" class="t-small">Slot production</text><rect x="600" y="25" width="260" height="95" rx="10" class="f-surface s-line"/><text x="614" y="58" class="t-b">Web app khác</text><text x="614" y="86" class="t-small">Cùng plan, chia CPU, RAM</text><path d="M150 120 L150 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az20-0-a)"/><rect x="20" y="175" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="34" y="208" class="t-b">Instance 1..N</text><text x="34" y="236" class="t-small">Máy Linux Azure quản lý</text><path d="M440 120 L440 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az20-0-a)"/><rect x="310" y="175" width="260" height="95" rx="10" class="f-pk-soft s-pk"/><text x="324" y="208" class="t-b">Slot staging</text><text x="324" y="236" class="t-small">Chạy trên cùng instance</text></svg>
</div>
<figcaption>Hình 1. Plan là đơn vị tính tiền và cấp tài nguyên; web app và slot staging chạy chung instance của plan.</figcaption>
</figure>

Hệ điều hành gắn với plan: Linux và Windows không chung plan. Tới Nơi chọn Linux vì image `trip-service` là image Linux. SKU quyết định bạn có những tính năng nào:

| Nhóm SKU | Deployment slot | Autoscale | Dùng cho Tới Nơi |
| --- | --- | --- | --- |
| Free, Shared | Không | Không | Chỉ để thử, không có Always On |
| Basic | Không | Chỉ scale tay | Lab nhỏ, không cần swap |
| Standard | Có, số lượng giới hạn | Có | Dev và staging có swap |
| Premium v3 (P0v3 trở lên) | Có, nhiều hơn | Có | Production, lab của bài này |

Tên SKU, số slot tối đa và giới hạn instance thay đổi theo thời gian. Hãy đối chiếu trang giá và trang limits hiện hành thay vì tin vào bảng này khi chốt thiết kế.

## Container từ ACR: identity kéo image, không có mật khẩu registry

Có hai cách đưa code lên. Zip deploy hoặc `az webapp up` đẩy code đã build lên và chạy trên runtime .NET do nền tảng cung cấp. Container chạy đúng image bạn đã build. Tới Nơi chọn container vì cùng image đó chạy được trên Container Apps ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment), nên artifact không đổi khi nền tảng đổi. Zip deploy gắn bạn với phiên bản runtime của App Service.

Dockerfile của trip-service đặt `ASPNETCORE_HTTP_PORTS=8080` và `EXPOSE 8080`. App Service tự dò một số cổng phổ biến, nhưng đừng dựa vào cơ chế dò: khai báo tường minh app setting `WEBSITES_PORT=8080`. Dò sai cổng thì nền tảng báo container không khởi động.

Khối lệnh dưới tạo plan, web app và cho web app dùng `id-toinoi-trips-dev` (bài 13) để kéo image. Giả định `acrtoinoidev` và image đã có từ [bài 21](/articles/azure-container-registry-build-push-digest). Tên web app phải duy nhất toàn cầu vì nó thành hostname `azurewebsites.net`, nên hãy thêm hậu tố nếu bị trùng.

```bash
RG=rg-toinoi-trips-dev-sea
PLAN=asp-toinoi-dev-sea
APP=app-toinoi-trips-dev
IMAGE=acrtoinoidev.azurecr.io/trips@sha256:<digest-tu-bai-21>

az appservice plan create -g $RG -n $PLAN -l southeastasia \
  --is-linux --sku P0V3
IDENTITY_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query id -o tsv)
CLIENT_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query clientId -o tsv)

az webapp create -g $RG -p $PLAN -n $APP --container-image-name $IMAGE
az webapp identity assign -g $RG -n $APP --identities $IDENTITY_ID
az webapp config set -g $RG -n $APP --generic-configurations \
  "{\"acrUseManagedIdentityCreds\": true, \"acrUserManagedIdentityID\": \"$CLIENT_ID\"}"
az webapp config appsettings set -g $RG -n $APP --settings WEBSITES_PORT=8080
```

Khối này chưa chứng minh app đã chạy. Nếu `id-toinoi-trips-dev` chưa có AcrPull trên registry, lần kéo image đầu sẽ thất bại và app kẹt ở trạng thái khởi động; xem log bằng `az webapp log tail`. CLI bản cũ dùng `--deployment-container-image-name`, nay đã thay bằng `--container-image-name`; chạy `--help` nếu CLI của bạn không nhận. Nếu nền tảng từ chối reference có `@sha256`, hãy dùng một tag không bao giờ bị ghi đè và ghi lại digest tương ứng.

## Secret qua Key Vault reference, readiness qua health check

App setting có thể là một tham chiếu dạng `@Microsoft.KeyVault(SecretUri=...)`. Nền tảng tự phân giải tham chiếu bằng identity của app, rồi đưa giá trị thật vào biến môi trường. Code của trip-service thấy một connection string bình thường tại `ConnectionStrings__Trips`, không biết Key Vault tồn tại. Hai điều kiện: identity phải có quyền đọc secret (**Key Vault Secrets User** ở bài 14), và app phải biết dùng identity nào. Mặc định App Service dùng identity hệ thống, nên với identity do người dùng tạo bạn đặt thêm thuộc tính `keyVaultReferenceIdentity`.

Có một thất bại khó nhìn: nếu tham chiếu không phân giải được, app nhận chuỗi `@Microsoft.KeyVault(...)` nguyên văn làm giá trị. Host production sẽ từ chối khởi động vì chuỗi đó không phải connection string hợp lệ. Secret `Postgres--Trips` phải là connection string đầy đủ với `SslMode=VerifyFull`. URI không kèm version thì nền tảng lấy phiên bản mới nhất, nhưng không tức thì. Bài 29 quay lại chu kỳ làm mới.

Health check nói cho nền tảng biết app nào đang khoẻ. App Service gọi định kỳ vào path bạn khai báo. Với plan có từ hai instance trở lên, instance trả lỗi liên tiếp bị loại khỏi vòng chia tải và có thể bị thay. Với một instance thì không còn chỗ để chuyển traffic, nên health check chỉ mang tính báo hiệu. Tới Nơi dùng `/health/ready`, vì đó là endpoint kiểm kết nối PostgreSQL và migration. Cái giá: khi database sập, mọi instance cùng báo lỗi một lúc. Hãy đọc tài liệu Health check để biết nền tảng làm gì khi mọi instance cùng fail. Đây cũng là cuộc tranh luận liveness và readiness ở bài 36.

Khối dưới đặt setting còn lại, bật health check, Always On và tắt ARR affinity:

```bash
APP_ID=$(az webapp show -g $RG -n $APP --query id -o tsv)
az rest --method PATCH --uri "${APP_ID}?api-version=2022-03-01" \
  --body "{\"properties\":{\"keyVaultReferenceIdentity\":\"$IDENTITY_ID\"}}"

az webapp config appsettings set -g $RG -n $APP --settings \
  ASPNETCORE_ENVIRONMENT=Production \
  "ConnectionStrings__Trips=@Microsoft.KeyVault(SecretUri=https://kv-toinoi-dev-sea.vault.azure.net/secrets/Postgres--Trips/)" \
  "Auth__Authority=<https-issuer-tu-bai-19>" Auth__Audience=toinoi-trips \
  WEBSITE_SWAP_WARMUP_PING_PATH=/health/ready \
  WEBSITE_SWAP_WARMUP_PING_STATUSES=200

az webapp config set -g $RG -n $APP --always-on true \
  --generic-configurations '{"healthCheckPath": "/health/ready"}'
az webapp update -g $RG -n $APP --client-affinity-enabled false
```

Khối này chưa chứng minh app tới được PostgreSQL: nếu mạng giữa App Service và `psql-toinoi-dev-sea` chưa thông (bài 27 sẽ làm VNet integration), `/health/ready` trả 503 và cả health check lẫn warm-up của swap sẽ coi app là chưa sẵn sàng. Đó là hành vi đúng, không phải lỗi cấu hình. **Always On** giữ process luôn được nạp, nếu không App Service dỡ app đang nhàn rỗi và request đầu tiên sau đó phải chờ khởi động lại. **ARR affinity** là cookie dính một client vào một instance. `trip-service` không giữ trạng thái trong bộ nhớ (idempotency nằm trong database), nên tắt affinity để tải chia đều sau khi scale out.

## Slot staging: swap đổi hướng traffic, không đổi dữ liệu

Deployment slot là một web app đầy đủ với hostname riêng, ví dụ `app-toinoi-trips-dev-staging.azurewebsites.net`, chạy trên cùng plan. Bạn đưa bản mới vào staging, kiểm tra ở đó, rồi swap. Cơ chế swap: App Service áp cấu hình của slot đích lên slot nguồn, khởi động lại nguồn, gọi warm-up path, và chỉ khi nhận được phản hồi hợp lệ mới đổi hướng routing. Bản cũ lúc này nằm ở staging, vẫn đang nóng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Luồng swap: staging chạy bản mới và warm-up, swap đổi hướng traffic, bản cũ nằm lại staging nên rollback là swap ngược."><defs><marker id="az20-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Staging: digest mới</text><text x="34" y="123" class="t-small">Kiểm bằng hostname riêng</text><path d="M205 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az20-1-a)"/><rect x="235" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="249" y="95" class="t-b">Warm-up</text><text x="249" y="123" class="t-small">Gọi /health/ready</text><path d="M420 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az20-1-a)"/><rect x="450" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="464" y="95" class="t-b">Swap</text><text x="464" y="123" class="t-small">Đổi hướng traffic</text><path d="M635 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az20-1-a)"/><rect x="665" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="679" y="95" class="t-b">Bản cũ ở staging</text><text x="679" y="123" class="t-small">Rollback = swap ngược</text></svg>
</div>
<figcaption>Hình 2. Luồng swap: staging chạy bản mới và warm-up, swap đổi hướng traffic, bản cũ nằm lại staging nên rollback là swap ngược.</figcaption>
</figure>

Cấu hình đi theo slot theo hai kiểu. Setting thường **đi theo code** khi swap: giá trị ở staging sang production cùng bản mới. Setting đánh dấu **slot setting** (hay sticky) thì **ở lại slot**. Ví dụ, staging nên tắt xuất telemetry, và cờ đó không được theo code sang production. Chi tiết dễ quên: slot là một app riêng, nên identity do người dùng tạo và `keyVaultReferenceIdentity` phải được đặt cho slot (URI `${APP_ID}/slots/staging`). Hãy kiểm lại, đừng giả định nó thừa kế.

```bash
az webapp deployment slot create -g $RG -n $APP --slot staging \
  --configuration-source $APP
az webapp identity assign -g $RG -n $APP --slot staging \
  --identities $IDENTITY_ID
az webapp config appsettings set -g $RG -n $APP --slot staging \
  --slot-settings Telemetry__Export=false

az webapp config container set -g $RG -n $APP --slot staging \
  --container-image-name acrtoinoidev.azurecr.io/trips@sha256:<digest-moi>
az webapp deployment slot swap -g $RG -n $APP \
  --slot staging --target-slot production
```

Lệnh `swap` cuối cũng là lệnh rollback: chạy lại đúng nó sẽ đưa bản cũ trở lại production. Khối này không chứng minh swap không làm rớt request, và nó càng không đụng tới database. Dữ liệu và schema dùng chung cho cả hai slot.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Swap ngược chỉ hoàn tác code, không hoàn tác migration. Nếu bản mới đã đổi schema theo cách bản cũ không đọc được, rollback đưa bản cũ về với database mà nó không hiểu. Hãy giữ migration tương thích hai chiều trong một chu kỳ release, đúng như bài 36 khuyên. Ngoài ra, plan tính tiền suốt thời gian slot tồn tại: xoá slot khi xong lab, và xoá cả resource group nếu bạn không dùng nữa.</p>
</div>

### Quan sát swap bằng vòng curl

Chạy vòng lặp dưới ở terminal thứ hai rồi swap. Dùng `/health/live` vì nó không phụ thuộc database:

```bash
URL=https://$APP.azurewebsites.net/health/live
while true; do
  curl -s -o /dev/null -w "%{http_code} %{time_total}s\n" $URL
  sleep 0.5
done
```

Kỳ vọng là một chuỗi mã 200 liền mạch trong lúc swap, vì routing chỉ đổi sau khi slot nguồn đã nóng. Đây là kỳ vọng theo cơ chế, không phải kết quả đo. Vòng lặp chỉ cho biết có request nào hỏng, không cho biết bản nào đang trả lời; muốn phân biệt thì thêm header phiên bản. Request đang chạy dở tại thời điểm đổi routing vẫn có thể bị cắt, nên client cần retry kèm `Idempotency-Key` như API tạo chuyến yêu cầu.

## Scale up thay đổi máy, scale out thay đổi số máy

Hai hướng scale giải hai bài toán khác nhau:

| | Scale up | Scale out |
| --- | --- | --- |
| Thay đổi gì | SKU của plan (CPU, RAM mỗi instance) | Số instance của plan |
| Gỡ nút thắt nào | Một request cần nhiều CPU hoặc RAM | Quá nhiều request đồng thời |
| Điều kiện | Có thể gián đoạn ngắn khi đổi SKU, hãy kiểm tài liệu | App phải stateless, ARR affinity tắt |
| Chi phí | Đơn giá mỗi instance tăng | Nhân số instance |

Autoscale là một tập luật chạy trên metric, thường là metric của plan (`CpuPercentage`, `MemoryPercentage`, `HttpQueueLength`). Luật scale out và scale in phải có ngưỡng cách nhau đủ xa, nếu không plan sẽ dao động liên tục. `trip-service` chủ yếu chờ PostgreSQL, nên CPU có thể thấp trong khi latency đã tăng. Khối dưới thử `HttpQueueLength`, nhưng hãy xem metric này có dữ liệu thật trên plan Linux của bạn trước khi dựa vào nó; nếu không, quay về CPU kèm cảnh báo latency.

```bash
PLAN_ID=$(az appservice plan show -g $RG -n $PLAN --query id -o tsv)
az monitor autoscale create -g $RG --resource $PLAN_ID \
  --resource-type Microsoft.Web/serverfarms \
  -n autoscale-toinoi-dev --min-count 1 --max-count 3 --count 1
az monitor autoscale rule create -g $RG --autoscale-name autoscale-toinoi-dev \
  --condition "HttpQueueLength > 50 avg 5m" --scale out 1
az monitor autoscale rule create -g $RG --autoscale-name autoscale-toinoi-dev \
  --condition "HttpQueueLength < 5 avg 10m" --scale in 1
```

Ngưỡng 50 và 5 chỉ là điểm khởi đầu, phải load-test rồi mới chốt. Hệ quả hay bị bỏ sót: mỗi instance mới mang theo connection pool riêng tới PostgreSQL. Scale out từ 1 lên 3 instance nhân đôi hoặc gấp ba số kết nối, và Flexible Server ở bài 9 có giới hạn kết nối. Tính `max-count` nhân với kích thước pool trước khi bật autoscale.

## App Service đủ cho tới khi bạn cần thứ nó không có

App Service hợp với API HTTP không giữ trạng thái, tải tương đối đều và đội muốn ít việc vận hành nhất. Pricing & Quote và Booking của Tới Nơi nằm trong nhóm này. Hãy cân nhắc chuyển sang Container Apps ở [bài 22](/articles/azure-container-apps-revision-ingress-scale) khi gặp một trong các dấu hiệu sau:

| Dấu hiệu | Vì sao App Service chưa vừa |
| --- | --- |
| Worker nền tiêu thụ hàng đợi (Dispatch) | Không scale về 0; scale theo queue phải mượn metric của resource khác |
| Tải bùng nổ theo từng đợt, nhàn rỗi phần lớn thời gian | Plan tính tiền cả khi nhàn rỗi |
| Cần nhiều phiên bản chia traffic theo phần trăm | Slot có định tuyến theo phần trăm, nhưng mỗi phiên bản chiếm một slot trên cùng plan |
| Nhiều service nhỏ cần ranh giới tài nguyên riêng | Các app trong plan chia chung CPU và RAM |

Bài 26 đặt từng service Tới Nơi lên một bảng so sánh đầy đủ giữa VM, App Service, Container Apps, AKS và Functions.

**Bài tập:** staging đặt `Auth__Audience=toinoi-trips-staging` nhưng bạn quên đánh dấu slot setting. Sau swap, production dùng audience nào và điều gì xảy ra với JWT của khách? Rollback bằng swap ngược có sửa được toàn bộ không?

<details><summary>Xem lời giải</summary><p>Setting thường đi theo code, nên production nhận `toinoi-trips-staging`. Token cấp cho audience `toinoi-trips` sẽ bị API từ chối, và khách thấy 401 cho tới khi bạn swap lại. Swap ngược trả lại setting cũ cùng code cũ, nên sửa được lỗi cấu hình này. Nó không sửa được dữ liệu hay schema nếu bản mới đã ghi thay đổi không tương thích.</p></details>

## Học tiếp từ đây

Bạn vừa mô tả được cách chạy `trip-service` trên App Service: plan Linux P0v3, container kéo từ ACR bằng identity, secret qua Key Vault reference, health check, slot staging với swap và rollback, và autoscale theo metric của plan. Cái giá lớn nhất cần nhớ là plan tính tiền theo máy chứ không theo app, và swap không hoàn tác dữ liệu.

Bài sau trả lời câu hỏi image đến từ đâu: [Bài 21](/articles/azure-container-registry-build-push-digest) dựng `acrtoinoidev`, build bằng `az acr build`, lấy digest và gán AcrPull cho identity.

## Tài liệu gốc

- [App Service documentation](https://learn.microsoft.com/en-us/azure/app-service/)
- [Azure App Service plan overview](https://learn.microsoft.com/en-us/azure/app-service/overview-hosting-plans)
- [Set up staging environments in App Service](https://learn.microsoft.com/en-us/azure/app-service/deploy-staging-slots)
- [Monitor App Service instances by using Health check](https://learn.microsoft.com/en-us/azure/app-service/monitor-instances-health-check)
- [Use Key Vault references as app settings](https://learn.microsoft.com/en-us/azure/app-service/app-service-key-vault-references)
- [Configure a custom container for App Service](https://learn.microsoft.com/en-us/azure/app-service/configure-custom-container)

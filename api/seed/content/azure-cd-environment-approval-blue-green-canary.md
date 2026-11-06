---
slug: azure-cd-environment-approval-blue-green-canary
title: "CD: environment, approval, blue/green và canary bằng revision"
summary: "Bài 44: dựng cd.yml ba job dev, staging, prod cho trip-service của Tới Nơi, gắn approval vào GitHub environment, đẩy canary 10%, 50%, 100% bằng revision Container Apps và để KQL trên Application Insights quyết định rollback."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, cd, container-apps, canary, github-actions]
language: vi
---

Đây là **bài 44** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 5/10 của phần).

Bài này nối bước CI của bài 43 với ba môi trường: dev tự động, staging có smoke test, prod có người duyệt và canary theo revision, rồi để một truy vấn KQL quyết định tiếp tục hay quay lại. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>CI đã chạy test và scan image, tất cả đều xanh. Deploy thẳng 100% lên prod có an toàn không?</p>
<details><summary>Xem đáp án</summary><p>Chưa. CI chứng minh artifact build đúng và không có lỗ hổng đã biết. Nó không chứng minh artifact chạy đúng với dữ liệu, cấu hình và lượng request thật của prod. Canary và health gate sinh ra để bắt đúng loại lỗi đó trước khi nó chạm toàn bộ khách.</p></details>
</div>

## CD nhận digest từ CI và không build lại

Bài 43 kết thúc bằng output `image-digest` dạng `sha256:...`. Pipeline CD nhận digest đó làm input và chỉ đưa đúng artifact ấy qua ba môi trường. Build lại ở staging hay prod là sai về cơ chế. Cùng một commit vẫn có thể cho image khác nếu base image hoặc package đã đổi giữa hai lần build, và thứ test ở staging không còn là thứ chạy ở prod.

Tới Nơi dùng một registry cho mỗi môi trường (`acrtoinoidev`, `acrtoinoistaging`, `acrtoinoiprod`). Đưa image sang registry kế tiếp bằng `az acr import` thì manifest được sao chép nguyên, và bạn nên so lại digest sau khi import thay vì tin. Thứ tự là: dev tự động mỗi khi CI xanh trên `main`, staging chạy sau dev kèm smoke test, prod chỉ bắt đầu khi có người bấm duyệt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Một digest từ CI đi qua dev tự động, staging có smoke test, rồi dừng ở approval trước prod."><defs><marker id="az44-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Digest từ CI</text><text x="34" y="123" class="t-small">sha256 bất biến</text><path d="M200 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az44-0-a)"/><rect x="240" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="95" class="t-b">dev</text><text x="254" y="123" class="t-small">Tự động, không duyệt</text><path d="M420 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az44-0-a)"/><rect x="460" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">staging</text><text x="474" y="123" class="t-small">Smoke test</text><path d="M640 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az44-0-a)"/><rect x="680" y="65" width="180" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">prod</text><text x="694" y="123" class="t-small">Chờ reviewer duyệt</text></svg>
</div>
<figcaption>Hình 1. Một digest từ CI đi qua dev tự động, staging có smoke test, rồi dừng ở approval trước prod.</figcaption>
</figure>

Workflow dưới đây dựng đúng khung đó. `workflow_call` cho phép `ci.yml` gọi nó ngay sau job `image` (job gọi phải cấp `id-token: write`), nên dev được deploy tự động; `workflow_dispatch` để chạy tay khi cần. Mỗi job gắn một `environment`, còn logic deploy nằm trong `cd/*.sh` để YAML ngắn. Khối này chỉ mô tả luồng. Nó không chứng minh federated credential đã được tạo cho từng environment hay các script đã tồn tại.

```yaml
name: cd
on:
  workflow_call:
    inputs:
      digest: { type: string, required: true }
  workflow_dispatch:
    inputs:
      digest: { description: "sha256:<hex> từ output image-digest của CI", required: true }
permissions: { id-token: write, contents: read }
jobs:
  dev:
    runs-on: ubuntu-latest
    environment: dev
    steps:
      - uses: actions/checkout@v7.0.1
      - uses: azure/login@v2
        with: { client-id: "${{ vars.AZURE_CLIENT_ID }}", tenant-id: "${{ vars.AZURE_TENANT_ID }}", subscription-id: "${{ vars.AZURE_SUBSCRIPTION_ID }}" }
      - run: ./cd/deploy.sh dev "${{ inputs.digest }}"
  staging:
    needs: dev
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v7.0.1
      - uses: azure/login@v2
        with: { client-id: "${{ vars.AZURE_CLIENT_ID }}", tenant-id: "${{ vars.AZURE_TENANT_ID }}", subscription-id: "${{ vars.AZURE_SUBSCRIPTION_ID }}" }
      - run: ./cd/deploy.sh staging "${{ inputs.digest }}" && ./cd/smoke.sh staging
  prod:
    needs: staging
    runs-on: ubuntu-latest
    environment: prod
    steps:
      - uses: actions/checkout@v7.0.1
      - uses: azure/login@v2
        with: { client-id: "${{ vars.AZURE_CLIENT_ID }}", tenant-id: "${{ vars.AZURE_TENANT_ID }}", subscription-id: "${{ vars.AZURE_SUBSCRIPTION_ID }}" }
      - run: ./cd/canary.sh "${{ inputs.digest }}"
```

## Approval là luật của environment, và Azure cũng phải đồng ý

Dòng `environment: prod` không tự có người duyệt. Người duyệt cấu hình ở Settings, mục Environments: required reviewers, tuỳ chọn cấm tự duyệt, và giới hạn nhánh được deploy. Khi job trỏ tới `prod`, runner không bắt đầu bước nào cho tới khi reviewer chấp thuận. Biến và secret của environment cũng chỉ được nạp sau đó, nên `vars.AZURE_CLIENT_ID` của prod không lộ cho job staging. Với repository private, required reviewers phụ thuộc gói GitHub, hãy kiểm tài liệu.

Phần quan trọng hơn nằm phía Azure. Ở bài 42, federated credential có `subject` chứa tên environment, dạng `repo:<org>/<repo>:environment:prod`. Hệ quả: Microsoft Entra chỉ phát token cho identity prod khi job chạy trong environment `prod`, tức là sau khi qua cổng duyệt. Ai sửa workflow để bỏ dòng `environment:` sẽ có subject khác và bị từ chối. Approval vì vậy là cổng để lấy danh tính, nhưng chỉ khi identity prod là identity riêng và role của nó gắn ở `rg-toinoi-trips-prod-sea`, không phải cả subscription.

| Luật | dev | staging | prod |
| --- | --- | --- | --- |
| Reviewer | Không | Không | Bắt buộc, cấm tự duyệt |
| Nhánh được deploy | `main` | `main` | `main` |
| Identity và scope | App riêng, RG dev | App riêng, RG staging | App riêng, RG prod |
| Smoke test | Health ready | Smoke có xác thực | Gate KQL ở mỗi bước canary |
| Cửa sổ deploy | Bất kỳ lúc nào | Giờ làm việc | Giờ làm việc, không chiều thứ 6 |

## Canary là chia weight giữa hai revision đang chạy

Ở chế độ **Multiple**, Container Apps giữ nhiều revision cùng hoạt động và ingress chia request giữa chúng theo phần trăm weight bạn đặt. Revision cũ vẫn nguyên, nên rollback chỉ là trả weight về cho nó, không cần build hay pull gì.

Có một cái bẫy cụ thể với repository này. `main.bicep` của trip-service đang cố định `activeRevisionsMode: 'Single'`. Muốn canary ở prod bạn phải tham số hoá giá trị này hoặc đổi bằng `az containerapp revision set-mode --mode multiple`, và nhớ rằng lần deploy Bicep kế tiếp sẽ ghi đè nó về Single. Một bẫy nữa: mặc định ingress gửi 100% traffic cho revision mới nhất (`latestRevision`), nên revision mới vừa sẵn sàng là nhận hết traffic, trừ khi bạn đã ghim traffic vào revision cũ theo tên.

Script `canary.sh` ghi nhớ revision đang nhận 100%, ghim traffic vào nó theo tên, tạo revision mới từ digest với suffix theo số lần chạy, rồi đẩy weight 10, 50, 100 và gọi gate sau mỗi bước quan sát. Gate fail thì hàm `rollback` trả weight về bản cũ và làm job thất bại.

```bash
set -euo pipefail
RG=rg-toinoi-trips-prod-sea; APP=ca-toinoi-trips-prod
SOAK=${SOAK:-900}   # giây quan sát mỗi bước
SUFFIX=c$GITHUB_RUN_NUMBER-$GITHUB_RUN_ATTEMPT; NEW=$APP--$SUFFIX
STABLE=$(az containerapp revision list -g $RG -n $APP \
  --query "[?properties.trafficWeight==\`100\`].name | [0]" -o tsv)

rollback() {
  az containerapp ingress traffic set -g $RG -n $APP \
    --revision-weight $STABLE=100 $NEW=0
  echo "Rollback về $STABLE" >&2; exit 1
}

# Ghim traffic theo tên để revision mới khởi đầu ở 0%
az containerapp ingress traffic set -g $RG -n $APP --revision-weight $STABLE=100
az containerapp update -g $RG -n $APP --revision-suffix $SUFFIX \
  --image acrtoinoiprod.azurecr.io/trips@$1

for W in 10 50 100; do
  az containerapp ingress traffic set -g $RG -n $APP \
    --revision-weight $NEW=$W $STABLE=$((100 - W))
  sleep "$SOAK"
  ./cd/gate.sh "$NEW" || rollback
done
az containerapp revision deactivate -g $RG -n $APP --revision "$STABLE"
```

Suffix có thêm `GITHUB_RUN_ATTEMPT` để chạy lại job không đụng tên revision đã có. Khối này không chứng minh revision mới thật sự giữ 0% trước bước 10%, hãy kiểm bằng lệnh liệt kê ở lab. Dòng cuối tắt revision cũ, nên sau khi qua gate cuối bạn mất đường lui một lệnh; muốn giữ, dời bước tắt sang một job dọn dẹp chạy sau vài giờ. Bạn cũng có thể gán label `canary` (`az containerapp revision label add`) cho revision mới để có URL riêng cho smoke test trước khi chia weight.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Canary tăng weight 10%, 50%, 100% qua gate KQL; gate fail đưa weight về revision cũ."><defs><marker id="az44-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="250" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="70" class="t-b">Canary 10%</text><text x="34" y="98" class="t-small">Gate KQL sau 15 phút</text><path d="M270 82 L315 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az44-1-a)"/><rect x="315" y="40" width="250" height="85" rx="10" class="f-pk-soft s-pk"/><text x="329" y="70" class="t-b">Canary 50%</text><text x="329" y="98" class="t-small">Gate KQL sau 15 phút</text><path d="M565 82 L610 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az44-1-a)"/><rect x="610" y="40" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="624" y="70" class="t-b">100% revision mới</text><text x="624" y="98" class="t-small">Tắt revision cũ sau</text><path d="M440 125 L440 185" class="f-none s-warn" stroke-width="2" marker-end="url(#az44-1-a)"/><rect x="315" y="185" width="250" height="85" rx="10" class="f-warn-soft s-warn"/><text x="329" y="215" class="t-b">Gate fail: rollback</text><text x="329" y="243" class="t-small">Weight về revision cũ</text></svg>
</div>
<figcaption>Hình 2. Canary tăng weight 10%, 50%, 100% qua gate KQL; gate fail đưa weight về revision cũ.</figcaption>
</figure>

## Health gate: KQL trả error rate, script quyết định

Gate là một truy vấn KQL trên Application Insights (bài 47 đi sâu hơn) trả về ba con số cho riêng revision mới: tổng request, tỷ lệ lỗi và p95 latency. Tên replica của Container Apps bắt đầu bằng tên revision, nên lọc theo tiền tố `cloud_RoleInstance` là cách đơn giản; thêm dấu `-` sau tên để revision `--c4` không khớp nhầm replica của `--c42`. Đây là giả định cần kiểm trên bảng `requests` của bạn, vì nó phụ thuộc cách OpenTelemetry gán resource attribute ở bài 37 loạt C#.

```kusto
let rev = "__REV__";
requests
| where timestamp > ago(15m)
| where cloud_RoleInstance startswith strcat(rev, "-")
| summarize total = count(),
            failed = countif(success == false),
            p95 = percentile(duration, 95)
| extend errorRate = iff(total == 0, 0.0, 100.0 * failed / total)
```

Script `gate.sh` thay `__REV__` bằng tên revision, chạy truy vấn bằng `az monitor app-insights query` (cần extension `application-insights`) và áp hai điều kiện. Điều kiện thứ nhất là `errorRate` không vượt 1%. Điều kiện thứ hai dễ bị quên: `total` phải đủ lớn, vì «0% lỗi trên 4 request» không có nghĩa gì. Nếu `total` dưới ngưỡng, gate phải coi là chưa đủ dữ liệu, kéo dài thời gian quan sát hoặc fail, chứ không được pass im lặng. Truy vấn trên cũng chưa so p95 với revision cũ; muốn vậy hãy chạy nó cho cả `STABLE`.

Rollback chỉ trả traffic, không hoàn tác dữ liệu. Gate bắt lỗi kỹ thuật, không bắt lỗi nghiệp vụ như trip bị tính sai giá nhưng vẫn trả HTTP 200.

## Blue/green ở App Service và canary ở AKS có cùng ý tưởng

Không phải workload nào của Tới Nơi cũng ở Container Apps. Cơ chế tương đương ở hai dịch vụ còn lại:

| Tiêu chí | Container Apps | App Service | AKS |
| --- | --- | --- | --- |
| Đơn vị phiên bản mới | Revision | Deployment slot | ReplicaSet của Deployment |
| Chia traffic | Phần trăm weight giữa revision | Swap slot, hoặc phần trăm bằng traffic routing | Rolling update mặc định; canary theo weight cần ingress hoặc Gateway API |
| Rollback | Đặt weight về revision cũ | Swap ngược lại | `kubectl rollout undo` hoặc đổi weight |
| Cái giá | Revision cũ còn chạy cho tới khi tắt | Slot thứ hai tốn tài nguyên; setting phải đánh dấu sticky | Phải tự dựng lớp chia traffic |

Với App Service, blue/green là deploy vào slot `staging`, warm-up ở đó, rồi `az webapp deployment slot swap`; slot cũ giữ bản trước nên swap ngược là rollback. Với AKS, rolling update thay pod dần nhưng không kiểm soát phần trăm traffic. Canary thật cần lớp định tuyến có weight, ví dụ Gateway API với `HTTPRoute` có `backendRefs` kèm weight, hoặc service mesh. Đừng chọn ingress-nginx cho thiết kế mới, vì dự án này đã ngừng được duy trì từ 2026.

## Migration chạy trước revision mới và phải tương thích ngược

Trong canary, revision cũ và mới cùng đọc ghi vào `psql-toinoi-prod-sea` khá lâu. Vì vậy migration phải chạy trước khi tạo revision mới, bằng role riêng ở một job riêng (bài 46 sẽ làm kỹ), và schema sau migration phải phục vụ được cả hai phiên bản code. Kiểu mẫu an toàn là expand rồi contract. Bước expand thêm cột `pickup_zone` kiểu nullable vào bảng `trips`, code cũ bỏ qua cột này còn code mới ghi vào nó. Chỉ sau khi revision mới đã chạy 100% và ổn định qua một chu kỳ release, bước contract mới ràng buộc NOT NULL hoặc xoá cột cũ.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Rollback revision không hoàn tác migration. Nếu migration đã đổi tên cột hay xoá dữ liệu, revision cũ quay lại sẽ chạy trên schema nó không hiểu. Mỗi revision đang hoạt động cũng giữ connection pool riêng tới PostgreSQL, nên trong lúc canary tổng connection là của cả hai revision cộng lại. Và canary lỗi lúc chiều thứ 6 sẽ do ít người nhất xử lý.</p>
</div>

## Không deploy chiều thứ 6: ghi thành luật, không ghi thành thói quen

Quy ước truyền miệng sẽ bị phá ngay lần đầu có áp lực deadline. Hãy biến nó thành bước đầu của `deploy.sh` và `canary.sh`: đọc giờ theo múi giờ đội vận hành và fail trong cửa sổ cấm, trừ khi có biến override được ghi vào log.

```bash
freeze_check() {  # $1 = tên môi trường
  local DOW HOUR
  DOW=$(TZ=Asia/Ho_Chi_Minh date +%u); HOUR=$(TZ=Asia/Ho_Chi_Minh date +%H)
  if [ "$1" = prod ] && { [ "$DOW" -ge 6 ] || { [ "$DOW" -eq 5 ] && [ "$HOUR" -ge 14 ]; }; }; then
    [ "${FREEZE_OVERRIDE:-}" = "incident-fix" ] || { echo "Ngoài cửa sổ deploy prod" >&2; exit 1; }
  fi
}
```

`deploy.sh` truyền tên môi trường nó nhận, còn `canary.sh` gọi `freeze_check prod` vì tham số đầu của nó là digest. Khối này chỉ chặn từ phía script, người sửa được workflow vẫn gỡ được, nên `cd/` và `.github/workflows` cần quy tắc review riêng. Muốn luật nằm ở chính environment, GitHub có custom deployment protection rule (một GitHub App trả lời cho phép hay không); kiểm gói GitHub của bạn.

## Lab: canary 10% tới 100% và rollback do lỗi cố ý

Lab mô tả cách làm, kỳ vọng bên dưới chưa phải kết quả đã chạy. Chuẩn bị: một environment `prod` có required reviewer, `ca-toinoi-trips-prod` ở chế độ Multiple và có sẵn một revision đang nhận 100%, và hai digest trong `acrtoinoiprod`. Digest A là bản bình thường. Digest B build từ nhánh lab không merge, có middleware trả HTTP 500 cho khoảng 30% request vào một endpoint mà traffic lab chắc chắn gọi.

1. Đặt `SOAK=60` để lab không phải chờ 15 phút mỗi bước. Chạy workflow `cd` với digest A. Kỳ vọng: dev và staging chạy xong, job `prod` dừng chờ duyệt, sau khi duyệt weight đi 10, 50, 100 và gate pass ở mỗi bước.
2. Trong lúc canary, gửi request liên tục vào FQDN bằng vòng lặp `curl` để gate có dữ liệu. Liệt kê revision và weight:

```bash
az containerapp revision list -g rg-toinoi-trips-prod-sea -n ca-toinoi-trips-prod \
  --query "[].{name:name, active:properties.active, weight:properties.trafficWeight}" -o table
```

3. Chạy lại workflow với digest B. Kỳ vọng: ở bước 10% revision mới nhận một phần traffic, `errorRate` vượt 1%, `gate.sh` trả mã khác 0, `rollback` đặt `STABLE=100` và `NEW=0`, job fail. Chạy lệnh liệt kê ở trên để xác nhận weight đã về revision cũ. Gate thấy lỗi sau độ trễ ingestion của Application Insights, có thể vài phút.
4. Dọn lab: deactivate revision lỗi, xoá nhánh lab chứa middleware lỗi, và `az group delete` cho resource group lab nếu nó không chứa thứ bạn còn cần.

**Bài tập:** lúc 3 giờ sáng gate báo `errorRate` bằng 0 ở bước 10% sau 15 phút, nhưng `total` chỉ có 6 request. Bạn có nên cho lên 50% không? Cách khắc phục không cần chờ người thức dậy là gì?

<details><summary>Xem lời giải</summary><p>Không nên. Sáu request không đủ để nói revision mới khỏe, vì một lỗi chỉ xảy ra ở 5% request vẫn có thể không xuất hiện. Gate cần ngưỡng `total` tối thiểu và coi «chưa đủ dữ liệu» là fail hoặc kéo dài quan sát. Với khung giờ ít traffic, có thể chạy thêm smoke test chủ động có xác thực qua label `canary` để tạo request, hoặc đơn giản hơn là không bắt đầu canary ngoài giờ có lưu lượng đủ.</p></details>

## Học tiếp từ đây

Bạn đã dựng được luồng CD đi từ một digest qua dev, staging và prod có người duyệt, đẩy canary bằng weight của revision, và có một gate KQL biến lỗi thành rollback tự động thay cho người nhìn dashboard. Phần còn thiếu là cách khai báo cả ba môi trường một cách nhất quán và dựng nhanh một môi trường thử, việc của [bài 45](/articles/azure-developer-cli-azd-deployment-stacks-moi-truong) với Azure Developer CLI và deployment stack. Sau đó [bài 46](/articles/azure-database-migration-trong-pipeline) quay lại thứ tự và role của migration trong pipeline.

## Tài liệu gốc

- [Revisions in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/revisions)
- [Traffic splitting in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/traffic-splitting)
- [Blue-green deployment in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/blue-green-deployment)
- [Set up staging environments in Azure App Service](https://learn.microsoft.com/en-us/azure/app-service/deploy-staging-slots)
- [Azure Monitor Application Insights overview](https://learn.microsoft.com/en-us/azure/azure-monitor/app/app-insights-overview)
- [Azure Kubernetes Service documentation](https://learn.microsoft.com/en-us/azure/aks/)

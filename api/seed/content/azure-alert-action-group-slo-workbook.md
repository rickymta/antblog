---
slug: azure-alert-action-group-slo-workbook
title: "Alert, action group, SLO và workbook"
summary: "Bài 48: đặt SLO 99,9% cho POST /trips của Tới Nơi, dựng hai alert burn-rate fast 1h và slow 6h bằng log alert KQL, action group tới webhook, availability test qua Front Door và workbook sáu ô cho người trực."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, monitoring, alerting, slo, kql]
language: vi
---

Đây là **bài 48** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 9/10 của phần).

[Bài 10](/articles/azure-monitor-metrics-activity-log-alert-dau-tien) dạy alert theo ngưỡng tĩnh. Bài này biến SLO của `POST /trips` thành error budget, cảnh báo theo tốc độ đốt budget, rồi nối alert tới người trực và một workbook để họ nhìn. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>SLO là 99,9% thì alert "tỉ lệ lỗi trên 0,1%" là alert đúng nhất, phải không?</p>
<details><summary>Xem đáp án</summary><p>Không. Ngưỡng 0,1% nổ vì một phút xấu và im lặng khi lỗi nhỏ kéo dài cả tuần. Budget được tính trên cả tháng, nên alert phải đo tốc độ tiêu budget, không đo từng điểm tỉ lệ lỗi.</p></details>
</div>

## SLO là con số, error budget là thứ bạn được phép tiêu

Kiến trúc tổng thể của Tới Nơi (mục 14.1) gợi ý hai mục tiêu ban đầu cho API đặt xe: availability 99,9% theo tháng và p95 dưới 500 ms, không tính thời gian tìm tài xế. Đó là con số để thảo luận, chưa phải cam kết khi chưa có load test.

Cần tách ba khái niệm. **SLI** là phép đo: tỉ lệ request `POST /trips` không trả 5xx. **SLO** là mục tiêu đặt lên SLI: 99,9% trong 30 ngày. **Error budget** là phần còn lại: 0,1% request được phép lỗi. Quy ra thời gian là khoảng 43 phút hỏng hoàn toàn mỗi tháng, nhưng tính theo request mới đúng vì lỗi một phần cũng tiêu budget.

Định nghĩa SLI quyết định alert có đáng tin không. Chỉ 5xx là lỗi hệ thống; 4xx (token hết hạn, body sai) là lỗi client, không đốt budget. Latency cũng là SLI: "p95 dưới 500 ms" nghĩa là được phép 5% request chậm, và dùng được cùng kiểu alert.

Từ đó ra **burn rate**: tốc độ tiêu budget so với tốc độ "vừa hết đúng cuối tháng". Burn rate 14,4 duy trì một giờ tiêu 2% budget tháng, để nguyên thì hết sạch sau khoảng hai ngày. Burn rate 6 duy trì sáu giờ tiêu 5% budget, hết sau khoảng năm ngày. Hai cặp này là cặp phổ biến trong sách SRE về cảnh báo theo SLO, không phải hằng số của Azure.

| Alert | Cửa sổ | Burn rate | Tỉ lệ 5xx kích hoạt (SLO 99,9%) | Severity | Ý nghĩa |
| --- | --- | --- | --- | --- | --- |
| Fast burn | 1 giờ | 14,4 | trên 1,44% | 1 | Gọi người trực ngay |
| Slow burn | 6 giờ | 6 | trên 0,6% | 2 | Mở ticket, xử lý trong ngày |

Lỗi lớn làm fast burn nổ nhanh; lỗi nhỏ rỉ rả làm slow burn nổ chậm hơn nhưng vẫn trước khi budget cạn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Từ telemetry requests tới SLI, hai log alert burn-rate, action group và người trực; alert processing rule điều chỉnh thông báo."><defs><marker id="az48-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Bảng requests</text><text x="34" y="90" class="t-small">Telemetry của trips</text><path d="M210 72 L240 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-0-a)"/><rect x="240" y="30" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="62" class="t-b">Log alert KQL</text><text x="254" y="90" class="t-small">Fast 1h, slow 6h</text><path d="M430 72 L460 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-0-a)"/><rect x="460" y="30" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="62" class="t-b">Action group</text><text x="474" y="90" class="t-small">Email, webhook</text><path d="M650 72 L680 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-0-a)"/><rect x="680" y="30" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="694" y="62" class="t-b">Người trực</text><text x="694" y="90" class="t-small">Mở runbook</text><rect x="240" y="160" width="410" height="70" rx="10" class="f-warn-soft s-warn"/><text x="254" y="190" class="t-b">Alert processing rule</text><text x="254" y="214" class="t-small">Tắt tiếng, thêm action group</text><path d="M555 160 L555 125 L555 115" class="f-none s-warn" stroke-width="2" stroke-dasharray="5 4" marker-end="url(#az48-0-a)"/></svg>
</div>
<figcaption>Hình 1. Từ telemetry requests tới SLI, hai log alert burn-rate, action group và người trực; alert processing rule điều chỉnh thông báo.</figcaption>
</figure>

## Bốn loại alert, mỗi loại trả lời một câu hỏi khác

| Loại | Nguồn | Hợp với | Ví dụ ở Tới Nơi |
| --- | --- | --- | --- |
| Metric alert | Platform metrics | Con số đơn giản, độ trễ thấp | `active_connections` của `psql-toinoi-prod-sea` |
| Log alert (scheduled query) | KQL trên Log Analytics hoặc Application Insights | Phép tính tuỳ ý: tỉ lệ, percentile, join | Burn rate từ bảng `requests` |
| Activity log alert | Control plane | Ai đó đổi hoặc xoá resource | Xoá `ca-toinoi-trips-prod` |
| Smart detection | Application Insights tự học baseline | Bất thường mà bạn chưa nghĩ tới | Tỉ lệ failed request tăng đột biến |

Burn rate cần tỉ lệ trên hai cửa sổ thời gian, metric alert không biểu diễn gọn được, nên bài dùng log alert. Cái giá: mỗi lần đánh giá chạy một truy vấn, chịu độ trễ ingest, và tính tiền theo số rule và tần suất. Smart detection là lưới thứ hai, nhưng nó không biết SLO nên không thay alert burn-rate.

Severity chạy từ 0 (critical) tới 4 (verbose). Azure không tự hành động theo con số này; quy ước của Tới Nơi là 0 và 1 gọi người, 2 mở ticket, 3 và 4 chỉ để xem. Hành động nằm ở action group gắn vào rule: email, SMS, push tới Azure mobile app, voice, webhook, Logic App, Azure Function, ITSM. SMS và voice bị giới hạn tần suất.

## Lab: hai alert burn-rate, action group và kích hoạt bằng tải lỗi

Lab dựng trên môi trường dev, giả định bạn đã có Application Insights nhận telemetry của `trip-service` từ [bài 47](/articles/azure-application-insights-opentelemetry-kql).

### Action group tới webhook giả

Webhook giả là một địa chỉ HTTP bạn kiểm soát để xem payload; thực tế đó là công cụ trực hoặc ITSM. Khối dưới tạo action group với một email và một webhook dùng common alert schema để payload thống nhất cho mọi loại alert. Nó không chứng minh webhook nhận được: dùng chức năng test action group trong portal.

```bash
RG=rg-toinoi-monitor-dev-sea
AG_NAME=ag-toinoi-oncall-dev

az monitor action-group create \
  --resource-group "$RG" --name "$AG_NAME" --short-name oncall \
  --action email oncall-mail oncall@toinoi.example \
  --action webhook fake-pager "https://<fake-webhook-host>/hooks/toinoi" usecommonalertschema
```

Coi URL webhook là bí mật: ai biết nó đều gửi được payload giả vào công cụ trực.

### Truy vấn burn-rate từ bảng requests

Truy vấn dưới trả về một dòng khi tỉ lệ 5xx của `POST /trips` vượt 14,4 lần budget trong 1 giờ và vẫn vượt trong 5 phút gần nhất. Cửa sổ ngắn giúp alert tự resolve sớm khi sự cố qua; vì ingest trễ vài phút, nếu cửa sổ này hay trống thì nới ra. `total >= 100` chặn báo giả khi gần như không có traffic, và đếm bằng `sum(itemCount)` để đúng cả khi bật sampling (bài 47). Lọc `name ==` vì `POST /trips/{id}/accept` cũng bắt đầu bằng `POST /trips`.

```kusto
let sloBudget = 0.001;
let burn = 14.4;
let window = 1h;
let guard = 5m;
requests
| where timestamp > ago(window)
| where name == "POST /trips"
| summarize total = sum(itemCount),
            failed = sumif(itemCount, toint(resultCode) >= 500),
            totalShort = sumif(itemCount, timestamp > ago(guard)),
            failedShort = sumif(itemCount, timestamp > ago(guard)
                                and toint(resultCode) >= 500)
| where total >= 100
| where failed > total * sloBudget * burn
| where failedShort > totalShort * sloBudget * burn
```

Alert slow đổi `burn = 6`, `window = 6h`, `guard = 30m` và nên nâng ngưỡng `total`. Lưu thành `fast-burn.kql` và `slow-burn.kql`, rồi tạo hai rule bằng một hàm nhỏ; `--condition "count 'Burn' > 0"` nghĩa là nổ khi truy vấn trả ít nhất một dòng.

```bash
AI_ID=$(az monitor app-insights component show -g "$RG" \
  --app appi-toinoi-dev-sea --query id -o tsv)
AG_ID=$(az monitor action-group show -g "$RG" -n "$AG_NAME" --query id -o tsv)

create_burn_alert() {  # $1 tên  $2 cửa sổ  $3 tần suất  $4 severity  $5 file KQL
  az monitor scheduled-query create \
    --resource-group "$RG" --name "$1" --scopes "$AI_ID" \
    --condition "count 'Burn' > 0" \
    --condition-query Burn="$(cat "$5")" \
    --window-size "$2" --evaluation-frequency "$3" \
    --severity "$4" --action-groups "$AG_ID" --auto-mitigate true \
    --description "SLO POST /trips. Runbook: https://<wiki-noi-bo>/runbooks/trips-slo-burn"
}

create_burn_alert alert-slo-burn-fast-dev 1h 5m 1 fast-burn.kql
create_burn_alert alert-slo-burn-slow-dev 6h 15m 2 slow-burn.kql
```

`az monitor scheduled-query` nằm trong extension cùng tên, CLI sẽ hỏi cài. Description chứa link runbook vì người nhận alert lúc 2 giờ sáng cần biết mở gì đầu tiên.

### Kích hoạt, rồi xem alert nổ và tự đóng

Cách gây 5xx ở dev mà replica vẫn ready là thu quyền INSERT của role app trên bảng `trips` ([bài 46](/articles/azure-database-migration-trong-pipeline)): `/health/ready` chỉ kiểm kết nối và migration nên vẫn xanh, còn `POST /trips` lỗi khi ghi và trả 500. Đừng dừng PostgreSQL: readiness đỏ làm replica bị rút khỏi ingress, request có thể hỏng ngay ở ingress mà không vào `requests`, và alert im lặng. Request thiếu token hay thiếu `Idempotency-Key` chỉ ra 4xx, không đốt budget. Kỳ vọng là hai alert fired sau vài chu kỳ đánh giá, chưa phải kết quả đã chạy.

```bash
# $MIGRATOR_CONN: connection string của toinoi_migrator (chủ bảng) ở dev
psql "$MIGRATOR_CONN" -c 'REVOKE INSERT ON TABLE trips FROM toinoi_app;'

for i in $(seq 1 400); do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST "$DEV_URL/trips" \
    -H "Authorization: Bearer $DEV_TOKEN" -H "Idempotency-Key: $(uuidgen)" \
    -H "Content-Type: application/json" -d '{"distanceKm":5}'
  sleep 1
done | sort | uniq -c

psql "$MIGRATOR_CONN" -c 'GRANT INSERT ON TABLE trips TO toinoi_app;'
```

Sau khi GRANT lại và gửi vài chục request thành công, cửa sổ ngắn hết vượt ngưỡng, truy vấn không còn trả dòng và rule tự resolved nhờ auto-mitigate. Xem trạng thái ở Alerts của Azure Monitor và đọc payload webhook.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Thu quyền role app chỉ làm ở dev, và lệnh GRANT lại phải nằm ngay trong cùng script; chạy nhầm ở prod là một sự cố thật. Log alert tính tiền theo số rule, tần suất đánh giá và số chuỗi thời gian chia theo dimension: nhân bản mỗi alert cho mọi service và mọi vùng làm hoá đơn tăng mà không tăng độ tin cậy. Cuối lab hãy xoá rule, action group, web test và resource group lab.</p>
</div>

## Thiết kế alert để người trực còn tin nó

Alert tốt ít, hiếm, và cái nào cũng cần hành động.

- **Cửa sổ và tần suất.** Cửa sổ quyết định cần bao nhiêu bằng chứng, tần suất quyết định bao lâu mới thấy.
- **Số lần vi phạm.** Log alert cho phép yêu cầu "ít nhất X lần trong Y điểm" trước khi nổ. Dùng cho tín hiệu dao động như độ trễ, để một điểm xấu không đánh thức ai.
- **Tự đóng.** Alert không tự resolve để lại hàng đống mục mở và người trực học cách phớt lờ. Bật auto-mitigate kèm cửa sổ ngắn như trên.
- **Nhóm theo service.** Chia theo dimension như `cloud_RoleName` thì một service lỗi không che service khác, và thông báo vẫn gom về một rule.
- **Alert processing rule.** Thay vì sửa từng rule, một processing rule ở scope subscription dev có thể tắt mọi thông báo theo lịch ngoài giờ làm việc (suppression), hoặc thêm một action group cho mọi alert trong scope.

Mọi alert phải trả lời được câu "người nhận làm gì". Nếu câu trả lời là "xem thử", đó là nội dung của workbook, không phải của alert.

**Bài tập:** Ban đêm dev chỉ có 40 request `POST /trips` trong một giờ, 5 request trả 5xx. Fast burn có nổ không? Có nên bỏ điều kiện `total >= 100` để bắt cả trường hợp này?

<details><summary>Xem lời giải</summary><p>Không nổ, vì tổng 40 nhỏ hơn 100. Bỏ điều kiện này thì chỉ cần 1 lỗi trong 3 request đã vượt ngưỡng 1,44% rất xa và alert nổ giả mỗi đêm. Hãy giữ ngưỡng traffic tối thiểu, để alert slow 6h gom nhiều mẫu hơn, và dùng availability test để biết service còn sống khi traffic thật quá thấp.</p></details>

## Availability test nhìn Tới Nơi từ bên ngoài

Bảng `requests` chỉ có dữ liệu khi request tới được ứng dụng. Nếu DNS, TLS, Front Door hỏng hoặc không replica nào ready, `requests` im lặng và burn-rate alert im theo. **Availability test** gửi request từ nhiều vùng tới URL công khai theo chu kỳ và ghi vào bảng `availabilityResults`. Dùng **standard test** (kiểm status code, TLS, nội dung); URL ping test cổ điển đã có lịch nghỉ hưu.

Tới Nơi trỏ test tới `/health/ready` qua endpoint [Front Door](/articles/azure-front-door-cdn-custom-domain-tls), không tới thẳng container app, để test chạm cả đường vào của khách: DNS, TLS, WAF, định tuyến. Readiness kiểm cả database ([bài 36](/articles/dotnet10-aws-azure-deployment)), nên DB hỏng làm test đỏ dù process còn sống. Nó không chứng minh `POST /trips` có xác thực chạy đúng.

Khối dưới tạo standard test từ Southeast Asia, East Asia và Japan East (mã vùng theo danh sách trong tài liệu). Lệnh không tạo alert kèm theo: bạn tự thêm rule trên `availabilityResults`.

```bash
az monitor app-insights web-test create \
  --resource-group "$RG" --name wt-toinoi-trips-ready-dev \
  --defined-web-test-name wt-toinoi-trips-ready-dev \
  --location southeastasia --kind standard --enabled true \
  --frequency 300 --timeout 30 --retry-enabled true \
  --locations Id=apac-sg-sin-azr Id=apac-hk-hkn-azr Id=apac-jp-kaw-edge \
  --synthetic-monitor-id wt-toinoi-trips-ready-dev \
  --http-verb GET --request-url "https://<front-door-endpoint>/health/ready" \
  --expected-status-code 200 \
  --tags "hidden-link:$AI_ID=Resource"
```

Thẻ `hidden-link` nối test với Application Insights. Quy tắc nổ là "thất bại ở ít nhất 2 trong 3 vùng", vì một vùng đỏ lẻ thường là mạng của điểm test. Tài liệu khuyên dùng ít nhất năm vùng cho prod.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Availability test từ ba vùng đi qua Front Door tới /health/ready; kết quả vào availabilityResults và nuôi alert yêu cầu từ hai vùng trở lên thất bại."><defs><marker id="az48-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="200" height="60" rx="10" class="f-phys-soft s-phys"/><text x="34" y="46" class="t-b">Singapore</text><text x="34" y="68" class="t-small">Điểm test 1</text><rect x="20" y="120" width="200" height="60" rx="10" class="f-phys-soft s-phys"/><text x="34" y="146" class="t-b">Hồng Kông</text><text x="34" y="168" class="t-small">Điểm test 2</text><rect x="20" y="220" width="200" height="60" rx="10" class="f-phys-soft s-phys"/><text x="34" y="246" class="t-b">Nhật Bản</text><text x="34" y="268" class="t-small">Điểm test 3</text><path d="M220 50 L290 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-1-a)"/><path d="M220 150 L290 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-1-a)"/><path d="M220 250 L290 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-1-a)"/><rect x="290" y="105" width="220" height="90" rx="10" class="f-pk-soft s-pk"/><text x="304" y="140" class="t-b">Front Door</text><text x="304" y="166" class="t-small">TLS, WAF, định tuyến</text><path d="M510 150 L560 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az48-1-a)"/><rect x="560" y="105" width="300" height="90" rx="10" class="f-pk-soft s-pk"/><text x="574" y="140" class="t-b">/health/ready</text><text x="574" y="166" class="t-small">ca-toinoi-trips-dev</text><rect x="560" y="215" width="300" height="65" rx="10" class="f-warn-soft s-warn"/><text x="574" y="242" class="t-b">availabilityResults</text><text x="574" y="266" class="t-small">Alert khi 2 vùng thất bại</text></svg>
</div>
<figcaption>Hình 2. Availability test từ ba vùng đi qua Front Door tới /health/ready; kết quả vào availabilityResults và nuôi alert yêu cầu từ hai vùng trở lên thất bại.</figcaption>
</figure>

## Workbook một trang cho người trực

Alert gọi người dậy, workbook là thứ họ mở ra đầu tiên. Một trang, sáu ô, đọc từ trên xuống. Saturation và replica gộp vào một ô vì chúng giải thích nhau: CPU cao mà replica đã chạm trần nghĩa là autoscale hết đường.

| Ô | Nguồn | Câu hỏi nó trả lời |
| --- | --- | --- |
| 1. Traffic | Logs: `requests` | `POST /trips` mỗi phút có bất thường không |
| 2. Error | Logs: `requests` | Tỉ lệ 5xx và burn rate hiện tại |
| 3. Latency | Logs: `requests` | p95 so với đường 500 ms |
| 4. Saturation và replica | Metrics: Container Apps | CPU, memory và số replica có chạm trần không |
| 5. Dead-letter | Metrics: Service Bus | Message chết có đang tăng không ([Service Bus](/articles/azure-service-bus-queue-topic-session-dead-letter)) |
| 6. DB connections | Metrics: PostgreSQL | `active_connections` còn xa giới hạn không |

Ba ô đầu dùng chung truy vấn dưới, mỗi ô vẽ một cột. Nó đếm bằng `sum(itemCount)` nên chịu được sampling, nhưng không chứng minh telemetry đủ dày để p95 ổn định.

```kusto
requests
| where timestamp > ago(6h) and name == "POST /trips"
| summarize total = sum(itemCount),
            failed = sumif(itemCount, toint(resultCode) >= 500),
            p95 = percentile(duration, 95)
          by bin(timestamp, 5m)
| extend errorRatePct = round(100.0 * failed / total, 3)
| project timestamp, total, errorRatePct, p95
```

Workbook có tham số (môi trường, khoảng thời gian), gộp được Logs, Metrics và Resource Graph, và deploy được bằng IaC (resource `Microsoft.Insights/workbooks`) cho từng môi trường. **Azure Managed Grafana** hợp khi đội quen Grafana hoặc cần gộp nguồn ngoài Azure, nhưng đó là một instance riêng để vận hành và tính tiền. Với đội nhỏ như Tới Nơi, workbook là điểm bắt đầu rẻ hơn.

Description của alert nên chứa link workbook và runbook: nhận alert, mở workbook, thấy ô đỏ, mở runbook tương ứng.

## Học tiếp từ đây

Bạn vừa đặt SLO cho `POST /trips`, đổi nó thành hai alert burn-rate fast 1h và slow 6h, nối alert tới action group có webhook, thêm availability test từ ba vùng và phác workbook sáu ô. Lab mô tả cách làm; việc alert nổ và tự đóng là kỳ vọng bạn cần tự kiểm trên subscription dev.

Bài cuối của Phần 5 là [bài 49](/articles/azure-landing-zone-subscription-vending): khi đã có ba môi trường và cả tá rule như thế này, làm sao dựng môi trường mới bằng một pull request mà vẫn đúng policy, quyền và ngân sách. Nếu cần ôn telemetry ở phía code, xem lại [bài 37 về OpenTelemetry](/articles/dotnet10-opentelemetry-traces-metrics-logs).

## Tài liệu gốc

- [Azure Monitor documentation](https://learn.microsoft.com/en-us/azure/azure-monitor/)
- [Azure Monitor alerts overview](https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/alerts-overview)
- [Create or edit a log search alert rule](https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/alerts-create-log-alert-rule)
- [Action groups](https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/action-groups)
- [Application Insights availability tests](https://learn.microsoft.com/en-us/azure/azure-monitor/app/availability)
- [Azure Workbooks overview](https://learn.microsoft.com/en-us/azure/azure-monitor/visualize/workbooks-overview)

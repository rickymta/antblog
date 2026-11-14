---
slug: azure-application-insights-opentelemetry-kql
title: "Application Insights, OpenTelemetry và Log Analytics với KQL"
summary: "Bài 47: bật Azure Monitor OpenTelemetry distro trong trip-service của Tới Nơi, đọc telemetry bằng năm truy vấn KQL trên Log Analytics, lần theo một request lỗi tới PostgreSQL và kiểm soát sampling, daily cap, PII."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, monitoring, opentelemetry, kql]
language: vi
---

Đây là **bài 47** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 8/10 của phần).

Bài này nối [bài 37 của loạt C#](/articles/dotnet10-opentelemetry-traces-metrics-logs) (traces, metrics, logs đã có trong code) với backend của Azure: Application Insights trên Log Analytics, rồi đọc dữ liệu bằng KQL. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Application Insights có lưu telemetry ở một kho riêng, tách khỏi Log Analytics không?</p>
<details><summary>Xem đáp án</summary><p>Không với resource workspace-based, là loại bạn nên dùng. Dữ liệu nằm ngay trong Log Analytics workspace, cùng chỗ với log Container Apps và log PostgreSQL, nên truy vấn chéo được.</p></details>
</div>

## Distro thay SDK cũ, và dữ liệu nằm trong một workspace

Application Insights hiện là một resource đứng trên **Log Analytics workspace**. Cách này cho phép đặt telemetry ứng dụng, console log của `ca-toinoi-trips-<env>` và log của `psql-toinoi-<env>-sea` vào cùng một nơi, dùng cùng một ngôn ngữ truy vấn, cùng retention và quyền RBAC. Trước đây telemetry ứng dụng nằm riêng, nên so sánh một lỗi 500 với log container là việc thủ công.

Phía code, hướng hiện tại là **Azure Monitor OpenTelemetry distro** (package `Azure.Monitor.OpenTelemetry.AspNetCore`) thay cho Application Insights SDK cũ. Distro dựa trên OpenTelemetry nên `ActivitySource`/`Meter` `ToiNoi.Trips` của bài 37 loạt C# dùng lại được, không phải viết lại điểm đo. Hạ tầng tạo bằng CLI như sau.

```bash
az monitor log-analytics workspace create \
  --resource-group rg-toinoi-ops-staging-sea \
  --workspace-name log-toinoi-staging-sea \
  --location southeastasia --quota 2 --retention-time 30

az monitor app-insights component create \
  --app appi-toinoi-staging-sea \
  --resource-group rg-toinoi-ops-staging-sea \
  --location southeastasia --kind web \
  --workspace <log-analytics-workspace-resource-id>
```

Hai lệnh tạo workspace (kèm daily cap và retention) và một Application Insights gắn vào nó. Chúng không gắn Container Apps environment vào workspace và không cho `trip-service` gửi dữ liệu; lệnh `app-insights` cần extension của `az`, bản CLI sẽ hỏi cài. Giá trị `--quota 2` (GB mỗi ngày) và `30` ngày chỉ là điểm bắt đầu cho dev, bạn tự đặt theo lượng thật.

## Bật distro trong trip-service

Một dòng `UseAzureMonitor()` bật xuất trace, metric và log. Phần còn lại là đăng ký nguồn của riêng ta, gắn tên role và phiên bản để tách revision khi truy vấn.

```csharp
var revision = Environment.GetEnvironmentVariable("CONTAINER_APP_REVISION") ?? "local";

builder.Services.AddOpenTelemetry()
    .UseAzureMonitor(o => o.SamplingRatio = 0.5f)
    .ConfigureResource(r => r.AddService("toi-noi-trips", serviceVersion: revision))
    .WithTracing(t => t.AddSource("ToiNoi.Trips").AddNpgsql())
    .WithMetrics(m => m.AddMeter("ToiNoi.Trips"));
```

Khối này chỉ khai báo exporter, nguồn đo và sampling. Đoạn này thay khối `AddOpenTelemetry()` hiện có trong `Api/Program.cs` (vốn chỉ xuất OTLP khi bật `Telemetry:Export`), giữ tên service `toi-noi-trips` để `cloud_RoleName` không đổi. `AddNpgsql()` đến từ package `Npgsql.OpenTelemetry`, repo hiện chưa tham chiếu nên phải thêm; package này tạo ra dependency PostgreSQL, thiếu package này bạn chỉ thấy span `db.commit` tự viết của bài 37. Connection string lấy từ biến `APPLICATIONINSIGHTS_CONNECTION_STRING`, không hard-code trong source. `SamplingRatio = 0.5f` ở đây chỉ để minh hoạ; các bản distro mới có thêm sampler giới hạn số trace mỗi giây và mặc định sampling có thể đổi giữa các phiên bản, hãy đọc changelog của bản bạn dùng.

Connection string cho phép *gửi* telemetry vào resource, không cho đọc ra. Vẫn nên đưa nó vào Container App qua secret tham chiếu Key Vault hoặc lấy từ App Configuration (xem các bài Key Vault và App Configuration), để mỗi môi trường trỏ vào resource của mình và xoay được mà không build lại image. Muốn chặt hơn, bật xác thực Entra cho ingestion bằng identity của app và tắt local auth, kiểm tra cấu hình theo tài liệu.

## Năm bảng dữ liệu và cái giá của sampling

Trong màn Logs của Application Insights, telemetry nằm trong năm bảng chính. Nếu truy vấn từ workspace, cùng dữ liệu có tên khác (`AppRequests`, `AppDependencies`, `AppExceptions`, `AppTraces`, `AppMetrics`) và tên cột cũng đổi; bài này dùng tên ở màn Application Insights.

| Bảng | Chứa gì | Ví dụ ở Tới Nơi |
| --- | --- | --- |
| `requests` | Request đến service | `POST /trips`, `resultCode`, `duration` |
| `dependencies` | Lời gọi đi ra | INSERT vào PostgreSQL, gửi Service Bus |
| `exceptions` | Exception đã ghi nhận | Lỗi concurrency khi accept chuyến |
| `traces` | Log có cấu trúc | Log `ILogger` theo mức lọc đã cấu hình |
| `customMetrics` | Metric tự định nghĩa | `trips.requests`, `trips.request.duration` |

Mọi dòng có `operation_Id` (chính là trace id), `cloud_RoleName`, và `appVersion` nếu bạn đặt `service.version`. Vì vậy một request sinh ra các dòng ở nhiều bảng cùng `operation_Id`.

**Sampling** là cái giá lớn nhất về dữ liệu. Giữ 50% trace nghĩa là một nửa request không có dòng tương ứng, nhưng mỗi dòng giữ lại mang `itemCount` để bù. Hệ quả cho KQL: đếm phải dùng `sum(itemCount)`, không dùng `count()`. Percentile trên mẫu ngẫu nhiên vẫn gần đúng, nhưng một lỗi hiếm có thể không có dòng nào để tìm. Tỷ lệ giữ là đánh đổi giữa chi phí ingestion và khả năng điều tra; chi phí tính theo lượng dữ liệu nạp vào, tra Pricing Calculator.

## KQL: p95 theo 5 phút và tỷ lệ lỗi theo revision

KQL là chuỗi toán tử nối bằng `|`: `where` lọc, `summarize ... by bin(...)` gom theo nhóm thời gian, `join` nối bảng, `render` vẽ đồ thị. Hai truy vấn đầu trả lời "chậm không" và "bản nào lỗi".

```kusto
// Q1: p95 của POST /trips theo 5 phút
requests
| where timestamp > ago(6h) and name == "POST /trips"
| summarize p95_ms = percentile(duration, 95), n = sum(itemCount)
    by bin(timestamp, 5m)
| render timechart

// Q2: tỷ lệ lỗi 5xx theo revision
requests
| where timestamp > ago(1h)
| summarize total = sum(itemCount),
            failed = sumif(itemCount, toint(resultCode) >= 500)
    by appVersion
| extend error_rate_pct = round(100.0 * failed / total, 2)
| order by error_rate_pct desc
```

Q1 cho đường p95 và số request mỗi bucket; Q2 so error rate giữa revision đang canary và revision cũ, thông tin bạn cần để quyết định tăng traffic ở bài 44. Hai truy vấn này không cho biết nguyên nhân, và `appVersion` chỉ có giá trị khi bạn đã đặt `service.version`. Bài dùng `toint(resultCode) >= 500` thay vì `success == false` vì `success` có thể gồm cả các mã 4xx do client gửi sai.

## Dependency PostgreSQL chậm và lần theo operation_Id

Khi p95 tăng, câu hỏi tiếp theo là thời gian mất ở đâu. Dependency của Npgsql cho biết từng lời gọi database tốn bao lâu, còn `operation_Id` nối chúng với request.

```kusto
// Q3: dependency PostgreSQL chậm
dependencies
| where timestamp > ago(1h) and type =~ "postgresql"
| summarize p95_ms = percentile(duration, 95), calls = sum(itemCount)
    by target, name
| where p95_ms > 200
| top 10 by p95_ms desc

// Q4: request lỗi nối với dependency PostgreSQL cùng operation_Id
requests
| where timestamp > ago(1h) and toint(resultCode) >= 500
| join kind=leftouter (
    dependencies | where type =~ "postgresql"
    | project operation_Id, db_ms = duration, db_name = name
  ) on operation_Id
| project timestamp, operation_Id, duration, db_ms, db_name
| top 20 by duration desc
```

Giá trị `type` do exporter đặt; chạy `dependencies | summarize count() by type` một lần để xem tên thật trước khi tin vào bộ lọc `"postgresql"`. Ngưỡng 200 ms là số ví dụ, không phải SLO. Hai truy vấn này không chứng minh database là nguyên nhân: nếu `db_ms` nhỏ so với `duration`, thời gian nằm ở code hoặc ở lời gọi khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Một operation_Id nối request, message Service Bus và dependency PostgreSQL; message chỉ nối được khi propagation được bật."><defs><marker id="az47-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="60" width="200" height="110" rx="10" class="f-pk-soft s-pk"/><text x="34" y="92" class="t-b">POST /trips</text><text x="34" y="120" class="t-small">Bảng requests</text><text x="34" y="144" class="t-small">operation_Id = A</text><path d="M220 115 L243 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-0-a)"/><rect x="243" y="60" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="257" y="92" class="t-b">INSERT PostgreSQL</text><text x="257" y="120" class="t-small">Bảng dependencies</text><text x="257" y="144" class="t-small">operation_Id = A</text><path d="M443 115 L466 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-0-a)"/><rect x="466" y="60" width="200" height="110" rx="10" class="f-warn-soft s-warn"/><text x="480" y="92" class="t-b">Message Service Bus</text><text x="480" y="120" class="t-small">Cần bật propagation</text><text x="480" y="144" class="t-small">Nếu không, mất dấu</text><path d="M666 115 L689 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-0-a)"/><rect x="689" y="60" width="171" height="110" rx="10" class="f-pk-soft s-pk"/><text x="703" y="92" class="t-b">Consumer</text><text x="703" y="120" class="t-small">request + dependency</text><text x="703" y="144" class="t-small">operation_Id = A</text></svg>
</div>
<figcaption>Hình 1. Một operation_Id nối request, message Service Bus và dependency PostgreSQL; message chỉ nối được khi propagation được bật.</figcaption>
</figure>

Việc nối xuyên Service Bus (bài 26 của loạt C#, outbox rồi consumer) không tự xảy ra. Context phải đi theo message qua thuộc tính của nó, và nguồn tracing của Azure SDK có thể phải bật thủ công tuỳ phiên bản SDK. Hãy kiểm tra bằng cách gửi một request thật và chạy truy vấn sau với `operation_Id` của nó; nếu consumer ra `operation_Id` khác, propagation chưa nối.

```kusto
// Q5a: toàn bộ dấu vết của một request
let opId = "<operation_Id>";
union requests, dependencies, exceptions, traces
| where operation_Id == opId
| project timestamp, itemType, cloud_RoleName, name, resultCode, duration, message
| order by timestamp asc
```

## Log Container Apps và PostgreSQL cùng workspace

Telemetry của ứng dụng chưa đủ khi container chết trước khi kịp gửi, hay khi lỗi nằm ở database. Container Apps environment (`cae-toinoi-<env>`) gửi console log vào Log Analytics, và Flexible Server gửi log của nó sang workspace khi bạn bật diagnostic settings.

```kusto
// Q5: console log của trip-service, lọc dòng lỗi
ContainerAppConsoleLogs_CL
| where TimeGenerated > ago(30m)
| where ContainerAppName_s == "ca-toinoi-trips-staging"
| where Log_s has_any ("fail", "error", "exception")
| project TimeGenerated, RevisionName_s, Log_s
| order by TimeGenerated desc
| take 100
```

Tên bảng phụ thuộc đích log của Container Apps environment. Đích **Log Analytics** ghi vào `ContainerAppConsoleLogs_CL` với cột có hậu tố kiểu (`ContainerAppName_s`, `Log_s`) như truy vấn trên. Đích **Azure Monitor** (định tuyến bằng diagnostic settings) ghi vào bảng `ContainerAppConsoleLogs` với cột không hậu tố (`ContainerAppName`, `RevisionName`, `Log`), nên phải sửa truy vấn theo. Log PostgreSQL vào `AzureDiagnostics` (lọc theo `ResourceProvider` và `Category`) hoặc bảng riêng nếu bạn chọn chế độ resource-specific. Truy vấn này không chứng minh log đã đầy đủ: console log bị cắt khi container bị kill sớm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Ba nguồn dữ liệu vào cùng một Log Analytics workspace, nơi KQL, Application Map và function đọc chung."><defs><marker id="az47-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="260" height="55" rx="10" class="f-pk-soft s-pk"/><text x="34" y="64" class="t-b">trip-service + distro</text><rect x="20" y="95" width="260" height="55" rx="10" class="f-phys-soft s-phys"/><text x="34" y="129" class="t-b">Console log Container Apps</text><rect x="20" y="160" width="260" height="55" rx="10" class="f-phys-soft s-phys"/><text x="34" y="194" class="t-b">Log PostgreSQL</text><path d="M280 58 L335 110" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-1-a)"/><path d="M280 122 L335 122" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-1-a)"/><path d="M280 187 L335 135" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-1-a)"/><rect x="335" y="75" width="270" height="95" rx="10" class="f-phys-soft s-phys"/><text x="349" y="108" class="t-b">Log Analytics workspace</text><text x="349" y="136" class="t-small">Daily cap + retention</text><path d="M605 122 L630 122" class="f-none s-phys" stroke-width="2" marker-end="url(#az47-1-a)"/><rect x="630" y="75" width="230" height="95" rx="10" class="f-pk-soft s-pk"/><text x="644" y="108" class="t-b">KQL, Application Map</text><text x="644" y="136" class="t-small">Function dùng chung</text></svg>
</div>
<figcaption>Hình 2. Ba nguồn dữ liệu vào cùng một Log Analytics workspace, nơi KQL, Application Map và function đọc chung.</figcaption>
</figure>

Ngoài truy vấn, portal có ba công cụ đọc nhanh. **Application Map** vẽ service và dependency từ `cloud_RoleName` và `target`, nên phải đặt tên role đúng. **Transaction search** tìm một request và mở cây span của nó. **Live Metrics** cho thấy request, lỗi và CPU gần như theo thời gian thực khi bạn vừa deploy canary, nhưng không thay truy vấn lịch sử.

## Daily cap, retention và không log PII

Chi phí telemetry tỷ lệ với lượng dữ liệu nạp vào workspace. Có ba thông số điều chỉnh: sampling (ở app), **daily cap** (ở workspace) và retention. Khi đạt cap, workspace ngừng nhận dữ liệu cho tới chu kỳ sau, kể cả lúc có sự cố. Resource Application Insights còn có daily cap riêng; đặt cả hai thì cap thấp hơn quyết định. Nếu lượng dữ liệu ổn định ở mức cao, **commitment tier** tính theo dung lượng cam kết thay cho trả theo từng GB; so sánh bằng Pricing Calculator.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Daily cap thấp làm mất đúng dữ liệu bạn cần nhất khi lỗi bùng lên, vì sự cố cũng làm tăng lượng log. Hãy đặt cảnh báo khi dùng gần hết cap (bài 48). Và số điện thoại khách hay địa chỉ đón lọt vào log hoặc tag span thì đã nằm trong workspace, xoá lại rất khó. Hãy lọc trước khi gửi, đừng đợi đến khi đã nạp.</p>
</div>

Cách lọc PII: không log số điện thoại, không gắn nó thành tag; nếu cần nối sự kiện với khách, dùng trace id và id nội bộ. Kiểm tra các thuộc tính tự động như URL có query string và câu SQL (Npgsql dùng tham số, nhưng log tự viết thì không có gì bảo vệ). Retention đặt ở workspace, mặc định theo tài liệu và có thể chỉnh theo bảng; giữ ngắn cho bảng nhiều dữ liệu.

## Lab: gửi tải, lưu truy vấn thành function, lần theo một lỗi

Lab mô tả cách làm trên staging. Bước 1: bật distro như trên, đặt connection string vào Container App bằng secret. Bước 2: gửi tải bằng script, trộn vài request body sai để có lỗi 400.

```bash
FQDN=$(az containerapp show --name ca-toinoi-trips-staging \
  --resource-group rg-toinoi-trips-staging-sea \
  --query properties.configuration.ingress.fqdn -o tsv)

for i in $(seq 1 200); do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST "https://$FQDN/trips" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Idempotency-Key: $(uuidgen)" \
    -H "Content-Type: application/json" \
    -d "{\"distanceKm\":$(( i % 20 == 0 ? -1 : 5 ))}"
done | sort | uniq -c
```

Script chỉ cho thấy phân bố mã HTTP; `$TOKEN` bạn tự lấy từ identity provider của lab. Mỗi request thứ 20 gửi `distanceKm` âm nên validation trả 400, đủ để thấy request lỗi phía client trong `requests`. Vòng lặp này không tạo lỗi 5xx; muốn có, bạn phải gây lỗi này có chủ đích ở staging (ví dụ chặn tạm kết nối database) và nhớ khôi phục. Bước 3: chạy Q1 đến Q5, rồi lưu mỗi truy vấn thành function trong workspace để cả đội gọi lại bằng tên.

```bash
az monitor log-analytics workspace saved-search create \
  --resource-group rg-toinoi-ops-staging-sea \
  --workspace-name log-toinoi-staging-sea \
  --name toinoi_p95_post_trips --category "Tới Nơi" \
  --display-name "p95 POST /trips" \
  --func-alias toinoi_p95_post_trips \
  --saved-query 'AppRequests | where Name == "POST /trips" | summarize p95_ms = percentile(DurationMs, 95) by bin(TimeGenerated, 5m)'
```

Lệnh lưu một truy vấn thành function, gọi được bằng `toinoi_p95_post_trips()`. Function nằm trong workspace, nên dùng tên bảng và cột của workspace (`AppRequests`, `Name`, `DurationMs`, `TimeGenerated`), không dùng `requests` như ở màn Application Insights. Lệnh không lưu các truy vấn còn lại; hãy kiểm tra tham số bằng `-h` trên phiên bản CLI của bạn. Bước 4: lấy một `operation_Id` từ Q4, chạy Q5a, đọc cây từ request tới dependency PostgreSQL và ghi lại phần nào chiếm nhiều thời gian nhất.

**Bài tập:** sampling cố định 10%, một bucket 5 phút trả `count() = 120` còn `sum(itemCount)` trả khoảng 1200. Con số nào phản ánh tải thật, và tại sao một lỗi hiếm vẫn có thể không thấy?

<details><summary>Xem lời giải</summary><p>`sum(itemCount)` phản ánh tải thật vì mỗi dòng giữ lại đại diện cho khoảng 10 request. `count()` chỉ đếm số dòng còn lại. Một lỗi xảy ra vài lần có thể rơi hết vào phần bị bỏ, nên sampling phù hợp để đo xu hướng, không phải để chắc chắn tìm ra mọi lỗi hiếm.</p></details>

## Học tiếp từ đây

Bạn vừa có telemetry của `trip-service` trong Application Insights, năm truy vấn KQL cho latency, lỗi theo revision, database chậm, correlation và log container, cùng ba thông số kiểm soát chi phí: sampling, daily cap, retention. Dữ liệu này là nền cho việc đặt ngưỡng. [Bài 48](/articles/azure-alert-action-group-slo-workbook) biến chúng thành alert, action group, SLO và workbook; nếu cần xem lại phần rollout, quay về [bài 46](/articles/azure-database-migration-trong-pipeline).

## Tài liệu gốc

- [Azure Monitor OpenTelemetry cho .NET](https://learn.microsoft.com/en-us/azure/azure-monitor/app/opentelemetry-enable)
- [Application Insights overview](https://learn.microsoft.com/en-us/azure/azure-monitor/app/app-insights-overview)
- [Kusto Query Language (KQL)](https://learn.microsoft.com/en-us/kusto/query/)
- [Log Analytics workspace overview](https://learn.microsoft.com/en-us/azure/azure-monitor/logs/log-analytics-workspace-overview)
- [Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/)

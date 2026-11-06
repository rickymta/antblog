---
slug: azure-monitor-metrics-activity-log-alert-dau-tien
title: "Azure Monitor cơ bản: metrics, Activity Log và alert đầu tiên"
summary: "Bài 10: phân biệt metrics, Activity Log và resource logs của Azure Monitor, rồi tạo action group và hai metric alert đầu tiên cho VM và PostgreSQL của Tới Nơi để đội platform nhận email trước khi khách phản ánh."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, monitoring, alerting, log-analytics]
language: vi
---

Đây là **bài 10** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 10/10 của phần).

Bài này khép Phần 1 bằng việc làm cho các resource đã dựng ở bài 7 (VM) và bài 9 (PostgreSQL) tự báo khi có chuyện. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tạo xong một PostgreSQL Flexible Server thì mọi câu lệnh chậm của nó đã tự có sẵn để xem trong Azure Monitor, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Các con số như CPU hay số connection thì có sẵn. Nhưng log chi tiết, ví dụ log câu lệnh chậm, chỉ có khi bạn bật diagnostic setting và chọn nơi nhận.</p></details>
</div>

## Ba nguồn dữ liệu, ba cách thu thập khác nhau

Azure Monitor không phải một thứ duy nhất mà là nơi gom nhiều loại tín hiệu. Người mới hay nhầm vì giao diện đặt chúng cạnh nhau, trong khi cơ chế thu thập khác hẳn.

| Nguồn | Trả lời câu hỏi | Có sẵn không | Lưu bao lâu |
| --- | --- | --- | --- |
| Platform metrics | Resource đang khoẻ cỡ nào: CPU, connections, dung lượng | Tự có, không cần cấu hình | 93 ngày |
| Activity Log | Ai làm gì ở control plane: tạo, sửa, xoá resource | Tự có, ở phạm vi subscription | 90 ngày, muốn lâu hơn phải xuất qua diagnostic setting |
| Resource logs | Bên trong resource xảy ra gì: query chậm, lỗi đăng nhập | Không. Phải bật diagnostic setting | Tuỳ nơi bạn gửi tới |

Platform metrics là các con số theo thời gian, thu liên tục, nhẹ và rẻ. Chúng trả lời "có vấn đề không", chưa trả lời "vì sao". Activity Log ghi thao tác quản trị đi qua Azure Resource Manager (bài 4), ví dụ ai đó xoá VM `vm-toinoi-api-dev` lúc nào và bằng danh tính nào. Nó **không** ghi một request HTTP nào của `trip-service`, vì đó là data plane.

Resource logs mới là chỗ chứa chi tiết bên trong, nhưng chúng không tự đi đâu cả. Diagnostic setting là cấu hình nối "category log này của resource này" với một đích: Log Analytics workspace, storage account hoặc Event Hubs. Chưa có diagnostic setting thì log chưa được ai giữ.

Activity Log đáng để mở ngay ở tuần đầu, không phải khi có sự cố. Câu hỏi "ai đã xoá firewall rule của PostgreSQL sáng nay" hay "ai đổi kích cỡ VM" có đáp án trong đó, kèm thời điểm và danh tính. Vì nó ghi cả thao tác thất bại, bạn cũng thấy được ai bị từ chối quyền. Phần 2 sẽ dùng nó để kiểm chứng role assignment hoạt động đúng như thiết kế.

Cũng cần chống một ngộ nhận khác: "có số liệu" không bằng "có người xem". Dữ liệu metric nằm đó 93 ngày nhưng sẽ không ai mở lúc 2 giờ sáng. Đó là lý do tồn tại của alert: biến một con số thụ động thành một thông báo chủ động tới đúng người.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Ba nguồn dữ liệu của Azure Monitor: metrics và Activity Log tự có, resource logs cần diagnostic setting để tới Log Analytics."><defs><marker id="az10-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="270" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="58" class="t-b">Platform metrics</text><text x="34" y="82" class="t-small">Tự có, 93 ngày</text><rect x="20" y="115" width="270" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="143" class="t-b">Activity Log</text><text x="34" y="167" class="t-small">Control plane, tự có</text><rect x="330" y="72" width="230" height="86" rx="10" class="f-pk-soft s-pk"/><text x="344" y="105" class="t-b">Azure Monitor</text><text x="344" y="131" class="t-small">Metric explorer, alert</text><path d="M290 65 L330 100" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-0-a)"/><path d="M290 150 L330 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-0-a)"/><rect x="600" y="30" width="260" height="70" rx="10" class="f-warn-soft s-warn"/><text x="614" y="58" class="t-b">Diagnostic setting</text><text x="614" y="82" class="t-small">Bạn phải bật thủ công</text><rect x="600" y="115" width="260" height="70" rx="10" class="f-phys-soft s-phys"/><text x="614" y="143" class="t-b">Log Analytics workspace</text><text x="614" y="167" class="t-small">Tính tiền theo GB ingest</text><path d="M730 100 L730 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-0-a)"/><path d="M600 150 L560 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-0-a)"/></svg>
</div>
<figcaption>Hình 1. Ba nguồn dữ liệu của Azure Monitor: metrics và Activity Log tự có, resource logs cần diagnostic setting để tới Log Analytics.</figcaption>
</figure>

## Đọc metric đúng: aggregation và time grain

Metric explorer trong portal vẽ biểu đồ từ platform metrics. Với Tới Nơi bạn sẽ mở ba chỗ đầu tiên:

- VM `vm-toinoi-api-dev`: metric `Percentage CPU`.
- `psql-toinoi-dev-sea`: `active_connections` và `storage_percent`.
- Storage account `sttoinoidriverdocsdev`: `Ingress`, `Egress` và `Transactions`, dùng để xem ảnh giấy tờ tài xế (bài 8) được tải lên bao nhiêu.

Mỗi biểu đồ có hai núm dễ bị bỏ qua. **Aggregation** quyết định cách gộp nhiều mẫu thành một điểm: Average, Min, Max, Sum, Count. **Time grain** là độ dài mỗi điểm, ví dụ 1 phút hoặc 1 giờ. Hai núm này đổi hoàn toàn câu chuyện: CPU vọt lên 95% trong 20 giây rồi về lại sẽ gần như biến mất nếu bạn nhìn Average theo grain 1 giờ, nhưng lộ rõ nếu nhìn Max theo grain 1 phút.

Quy tắc dùng được ngay: với tài nguyên có thể cạn (CPU, connections, dung lượng) hãy nhìn Max để thấy đỉnh, còn Average để thấy xu hướng. Không có đáp án chung, chỉ có câu hỏi bạn đang hỏi.

Các con số platform metrics trả lời "bao nhiêu" nhưng không nói câu lệnh nào gây ra. Muốn biết câu SQL chậm trên bảng `trips`, bạn cần resource logs của PostgreSQL, kết hợp với `log_min_duration_statement` đã đặt ở bài 9. Ngưỡng log đó và diagnostic setting là hai nửa của cùng một việc: một nửa quyết định log nào sinh ra, nửa kia quyết định nó đi đâu.

## Alert rule: tín hiệu, điều kiện và người nhận

Một alert rule gồm bốn mảnh ghép:

1. **Scope**: resource nào bị theo dõi.
2. **Condition**: metric nào, aggregation nào, so với ngưỡng nào, trong cửa sổ bao lâu (window size), và cứ bao lâu thì đánh giá một lần (evaluation frequency).
3. **Action group**: nhóm hành động khi alert nổ: email, SMS, push, webhook, và các hình thức khác.
4. **Severity**: từ 0 (nghiêm trọng nhất) tới 4 (thông tin). Nó chỉ là nhãn để bạn phân loại, Azure không tự hành động theo severity.

Action group là thứ dùng lại được. Bạn tạo một lần `ag-toinoi-platform-dev` với email đội platform, rồi gắn cho nhiều alert. Điều này tránh việc mỗi alert có danh sách người nhận lệch nhau.

Có thêm **alert processing rule** để tắt tiếng có chủ đích. Khi đội platform nâng cấp PostgreSQL vào đêm thứ Bảy, CPU và connections biến động là điều bình thường. Processing rule với lịch cụ thể sẽ chặn thông báo trong khung giờ đó, thay vì bạn xoá rồi tạo lại các alert rule. Alert vẫn được ghi nhận, chỉ không gửi email.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Luồng alert: metric đi qua alert rule tới action group và email đội platform; processing rule có thể chặn thông báo khi bảo trì."><defs><marker id="az10-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="72" class="t-b">Metric</text><text x="34" y="100" class="t-small">CPU, storage_percent</text><path d="M210 82 L240 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-1-a)"/><rect x="240" y="40" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="254" y="72" class="t-b">Alert rule</text><text x="254" y="100" class="t-small">Ngưỡng + cửa sổ</text><path d="M430 82 L460 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-1-a)"/><rect x="460" y="40" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="72" class="t-b">Action group</text><text x="474" y="100" class="t-small">Email, SMS, webhook</text><path d="M650 82 L680 82" class="f-none s-phys" stroke-width="2" marker-end="url(#az10-1-a)"/><rect x="680" y="40" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="694" y="72" class="t-b">Đội platform</text><text x="694" y="100" class="t-small">Nhận email</text><rect x="240" y="165" width="410" height="75" rx="10" class="f-warn-soft s-warn"/><text x="254" y="197" class="t-b">Alert processing rule</text><text x="254" y="223" class="t-small">Chặn thông báo khi bảo trì</text><path d="M445 165 L445 135 L500 135 L500 125" class="f-none s-warn" stroke-width="2" stroke-dasharray="5 4" marker-end="url(#az10-1-a)"/></svg>
</div>
<figcaption>Hình 2. Luồng alert: metric đi qua alert rule tới action group và email đội platform; processing rule có thể chặn thông báo khi bảo trì.</figcaption>
</figure>

## Log Analytics workspace: nơi log đi tới

Log Analytics workspace là kho lưu log và là nơi bạn truy vấn bằng KQL. Với Tới Nơi, quy ước đơn giản là một workspace cho mỗi môi trường (dev, staging, prod), đặt cùng region Southeast Asia, đặt tên kiểu `log-toinoi-dev-sea`. Tách theo môi trường để quyền truy cập và chi phí của prod không lẫn với dev.

Workspace tính tiền chủ yếu theo lượng dữ liệu đưa vào (GB ingest) và thời gian giữ. Điều này có hậu quả thực tế: bật diagnostic setting cho mọi category của mọi resource rồi đặt `log_min_duration_statement` về 0 trên PostgreSQL sẽ biến mỗi câu SQL thành một dòng log và hoá đơn tăng theo tải. Hãy chọn đúng category cần dùng và tra Pricing Calculator trước khi bật rộng. Tới Phần 5 ta học KQL và cách thiết kế log; bài này chỉ dựng chỗ chứa. Các lệnh dưới giả định server PostgreSQL của bài 9 (và ở mục sau, VM của bài 7) vẫn còn; nếu bạn đã xoá chúng sau lab, dựng lại hoặc thay ID bằng một resource lab đang có.

```bash
RG=rg-toinoi-data-dev-sea

az monitor log-analytics workspace create \
  --resource-group "$RG" \
  --workspace-name log-toinoi-dev-sea \
  --location southeastasia

PG_ID=$(az postgres flexible-server show -g "$RG" -n psql-toinoi-dev-sea --query id -o tsv)
WS_ID=$(az monitor log-analytics workspace show -g "$RG" \
  --workspace-name log-toinoi-dev-sea --query id -o tsv)

az monitor diagnostic-settings create \
  --name diag-psql-to-log \
  --resource "$PG_ID" --workspace "$WS_ID" \
  --logs '[{"category":"PostgreSQLLogs","enabled":true}]'
```

Khối này tạo workspace và nối log PostgreSQL vào đó. Tên category có thể khác theo version của dịch vụ: liệt kê bằng `az monitor diagnostic-settings categories list --resource "$PG_ID"` trước khi tin tên trong bài. Khối này cũng chưa chứng minh có dữ liệu về: log chỉ xuất hiện sau khi server thực sự sinh ra dòng log.

## Lab: action group và hai alert đầu tiên

Mục tiêu: nhận email khi CPU của VM vượt 80% liên tục trong 5 phút, và khi storage của PostgreSQL vượt 85%. Địa chỉ email dưới đây là placeholder, hãy thay bằng hộp thư thật của đội.

```bash
AG_NAME=ag-toinoi-platform-dev

az monitor action-group create \
  --resource-group "$RG" --name "$AG_NAME" \
  --short-name platform \
  --action email platform-team platform@toinoi.example

AG_ID=$(az monitor action-group show -g "$RG" -n "$AG_NAME" --query id -o tsv)
VM_ID=$(az vm show -g rg-toinoi-api-dev-sea -n vm-toinoi-api-dev --query id -o tsv)

az monitor metrics alert create \
  --resource-group "$RG" --name alert-vm-cpu-high-dev \
  --scopes "$VM_ID" \
  --condition "avg Percentage CPU > 80" \
  --window-size 5m --evaluation-frequency 1m \
  --severity 2 --action "$AG_ID" \
  --description "CPU VM API trung binh tren 80% trong 5 phut"
```

Lệnh này gắn điều kiện "trung bình CPU trên 80% trong cửa sổ 5 phút, kiểm mỗi phút" vào VM của bài 7. Nó chưa chứng minh email sẽ tới: nếu thư vào spam hoặc người nhận đã bấm unsubscribe trong email thông báo của action group, bạn vẫn không biết. Hãy dùng chức năng test action group trong portal, hoặc tạo tải CPU thật trên VM lab đủ lâu (hơn 5 phút) để kiểm chứng.

```bash
az monitor metrics alert create \
  --resource-group "$RG" --name alert-psql-storage-high-dev \
  --scopes "$PG_ID" \
  --condition "avg storage_percent > 85" \
  --window-size 15m --evaluation-frequency 5m \
  --severity 1 --action "$AG_ID" \
  --description "Dung luong PostgreSQL tren 85%"
```

Storage đầy là sự cố nghiêm trọng hơn CPU cao: theo tài liệu, khi dung lượng dùng chạm 95% (hoặc còn dưới 5 GiB), server tự chuyển sang chế độ chỉ đọc và `POST /trips` sẽ thất bại, nên severity được đặt 1. Bật storage autogrow (`--storage-auto-grow Enabled`) là lớp phòng thủ bổ sung, không thay alert. Cửa sổ dài hơn vì storage thay đổi chậm, không cần kiểm từng phút. Chọn con số 85% là một quyết định vận hành: với database tăng nhanh, 85% có thể không đủ thời gian để bạn kịp mở rộng. Tên metric và đơn vị hãy xác nhận trong Metric explorer của chính server của bạn.

Nếu muốn thêm một alert về Activity Log, ví dụ báo khi ai đó xoá VM, dùng `az monitor activity-log alert create` với điều kiện theo `category` và `operationName`. Đây là alert theo sự kiện, không theo ngưỡng số.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Alert quá nhạy sinh ra thói quen bỏ qua email, và khi sự cố thật tới thì không ai đọc. Alert rule và workspace cũng tính tiền (rule theo số tín hiệu theo dõi, workspace theo GB ingest), nên đừng nhân bản rule cho mọi metric. Cuối lab hãy xoá: <code>az monitor metrics alert delete</code> cho từng rule, action group, diagnostic setting và workspace, hoặc xoá resource group lab. Email platform dùng trong lab phải là hộp thư bạn kiểm soát, không dùng danh sách người thật của một đội nếu chưa báo trước.</p>
</div>

## Tạm dừng thông báo mà không xoá alert

Khi cần bảo trì, tạo alert processing rule với loại bỏ action group trong một khung giờ. Lệnh `az monitor alert-processing-rule` thuộc extension `alertsmanagement`; CLI tự cài extension ở lần chạy đầu. Thời gian theo dạng `YYYY-MM-DD HH:MM:SS`, múi giờ dùng tên kiểu Windows (`SE Asia Standard Time` là UTC+7). Tên tham số có thể đổi theo phiên bản extension, hãy đối chiếu bằng `--help`:

```bash
az monitor alert-processing-rule create \
  --resource-group "$RG" --name apr-maintenance-dev \
  --rule-type RemoveAllActionGroups \
  --scopes "$PG_ID" \
  --schedule-start-datetime "2026-11-21 22:00:00" \
  --schedule-end-datetime "2026-11-22 02:00:00" \
  --schedule-time-zone "SE Asia Standard Time"
```

Đây chỉ là mô tả cách làm, chưa kèm bằng chứng đã chạy. Điều cần nhớ là cơ chế: alert vẫn được đánh giá và ghi lại, chỉ phần gửi thông báo bị chặn, nên sau bảo trì bạn vẫn đối chiếu được lịch sử.

**Bài tập:** VM đột ngột lên 95% CPU trong đúng 2 phút rồi trở về 20%. Alert `alert-vm-cpu-high-dev` ở trên (Average, cửa sổ 5 phút) có chắc chắn nổ không? Nếu muốn bắt cả đỉnh ngắn đó thì đổi gì và đánh đổi gì?

<details><summary>Xem lời giải</summary><p>Không, và thực tế là sẽ không nổ. Mọi cửa sổ 5 phút chứa đỉnh chỉ có tối đa 2 phút 95%; trung bình cao nhất là (2 x 95 + 3 x 20) / 5 = 50%, thấp hơn 80% nên alert im lặng. Đổi aggregation sang Max sẽ bắt được đỉnh, nhưng đánh đổi là alert nổ nhiều hơn vì mọi đỉnh ngắn vô hại cũng gây thông báo. Chọn theo việc đỉnh ngắn có gây hậu quả cho `trip-service` hay không.</p></details>

## Học tiếp từ đây

Bạn vừa có đủ ba lớp quan sát cơ bản: metrics tự có, Activity Log cho thao tác quản trị, và đường dẫn resource logs về Log Analytics. Hai alert đầu tiên cùng một action group sẽ báo cho đội platform khi VM quá tải hoặc `trips` sắp hết chỗ.

Đây là bài cuối của Phần 1. Bạn đã đi từ khái niệm cloud và resource group, qua mạng, VM, storage, PostgreSQL tới giám sát. Phần 2 bắt đầu bằng [bài 11 về Microsoft Entra ID](/articles/azure-entra-id-tenant-user-group-app-registration), nơi ta trả lời "ai được làm gì" trước khi thêm managed identity và mạng riêng cho `trip-service`. Về mặt telemetry trong code, xem lại [bài 35 về OpenTelemetry](/articles/dotnet10-opentelemetry-traces-metrics-logs).

## Tài liệu gốc

- [Azure Monitor documentation](https://learn.microsoft.com/en-us/azure/azure-monitor/)
- [Azure Monitor overview](https://learn.microsoft.com/en-us/azure/azure-monitor/fundamentals/overview)
- [Azure Monitor metrics overview](https://learn.microsoft.com/en-us/azure/azure-monitor/metrics/data-platform-metrics)
- [Azure Monitor alerts overview](https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/alerts-overview)
- [Diagnostic settings in Azure Monitor](https://learn.microsoft.com/en-us/azure/azure-monitor/data-collection/diagnostic-settings)
- [Log Analytics workspace overview](https://learn.microsoft.com/en-us/azure/azure-monitor/logs/log-analytics-workspace-overview)

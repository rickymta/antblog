---
slug: aws-observability-nang-cao-xray-adot-logs-insights-slo
title: "Quan sát nâng cao: X-Ray và ADOT, Logs Insights, anomaly detection, Application Signals và SLO"
summary: "Bài 55: từ metric sang câu hỏi vận hành: trace yêu cầu đặt chuyến chậm qua ALB, ECS, Lambda, Aurora bằng X-Ray và ADOT; log JSON có trace_id, Logs Insights, metric filter và Embedded Metric Format; anomaly detection và composite alarm giảm ồn; Application Signals định nghĩa SLO, error budget và burn rate; Synthetics, RUM và chi phí log."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, observability, x-ray, opentelemetry, logs-insights, slo, application-signals, synthetics]
language: vi
---

17:10 chiều thứ sáu, giờ cao điểm. Khách ở TP.HCM nhắn "đặt xe quay vòng tám giây rồi mới ra tài xế". Bảng điều khiển hiển thị CPU của service `booking` ở 45%, tỉ lệ 5xx 0,05%, mọi alarm đều xanh. Metric cho biết *có chuyện*, nhưng không cho biết request nào chậm, chậm ở đâu, và có đủ tệ để gọi người dậy hay không. Bài này dựng ba thứ còn thiếu: truy vết một request, hỏi log một câu cụ thể, và định nghĩa "đủ tệ" bằng SLO.

Đây là **bài 55** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần xong [bài 10](/articles/aws-cloudwatch-budgets-tagging-co-ban) (alarm, metric filter) và nên đọc [bài 37 loạt .NET](/articles/dotnet10-opentelemetry-traces-metrics-logs) cùng [bài 47](/articles/aws-eks-observability-container-insights-adot-prometheus), vì bài này không lặp lại cách service phát trace mà tập trung vào phía AWS.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Alarm "CPU lớn hơn 80%" kêu mỗi giờ cao điểm nhưng khách không phàn nàn. Nên làm gì với nó?</p>
<details><summary>Xem đáp án</summary><p>Đừng dùng nó để gọi người. CPU là <b>nguyên nhân tiềm năng</b>, không phải <b>triệu chứng</b> khách nhìn thấy. Giữ nó trên dashboard để chẩn đoán, và gọi on-call theo triệu chứng: lỗi hoặc chậm đo từ góc nhìn khách, tức SLO.</p></details>
</div>

## Trace: một request đi qua những đâu

Một lần đặt chuyến đi qua ALB, service `booking` trên ECS, Lambda `dispatch-assign` và truy vấn Aurora. **Trace** là cây các **span**, mỗi span là một đoạn công việc có thời gian bắt đầu và kết thúc; nhìn trace là thấy đoạn nào chiếm nhiều thời gian nhất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Biểu đồ thác nước của một trace đặt chuyến chậm: ALB bao ngoài, booking trên ECS, Lambda dispatch-assign và truy vấn Aurora chiếm khoảng 1,2 giây trong tổng 2,4 giây.">
<text x="200" y="26" class="t-small">0 s</text><text x="475" y="26" class="t-small">1 s</text><text x="750" y="26" class="t-small">2 s</text>
<path d="M200 34 L200 190 M475 34 L475 190 M750 34 L750 190" class="f-none s-line dash" stroke-width="1"/>
<text x="20" y="60" class="t-b">ALB</text><rect x="200" y="42" width="660" height="26" rx="6" class="f-surface s-line"/><text x="210" y="60" class="t-small">2,40 s tổng</text>
<text x="20" y="102" class="t-b">booking (ECS)</text><rect x="205" y="84" width="643" height="26" rx="6" class="f-phys-soft s-phys"/><text x="215" y="102" class="t-small">POST /v1/trips</text>
<text x="20" y="144" class="t-b">dispatch-assign (Lambda)</text><rect x="282" y="126" width="522" height="26" rx="6" class="f-pk-soft s-pk"/><text x="292" y="144" class="t-small">gán tài xế</text>
<text x="20" y="186" class="t-b">Aurora (truy vấn)</text><rect x="447" y="168" width="330" height="26" rx="6" class="f-warn-soft s-warn"/><text x="457" y="186" class="t-small t-warn">SELECT tài xế gần nhất: 1,2 s</text>
<text x="20" y="232" class="t-small">Số liệu minh hoạ. Đoạn truy vấn chiếm một nửa thời gian: câu hỏi tiếp theo là plan và index (Phần 04), không phải CPU của booking.</text>
<text x="20" y="252" class="t-small">Annotation city = HCM, service_type = ride cho phép lọc những trace như thế này trong X-Ray.</text>
</svg>
</div>
<figcaption>Hình 1. Trace cho biết thời gian đi đâu; metric của riêng từng thành phần không làm được điều này.</figcaption>
</figure>

Service phát span bằng **OpenTelemetry SDK**; **ADOT (AWS Distro for OpenTelemetry)** là collector nhận span qua OTLP và xuất sang X-Ray bằng exporter `awsxray`. SDK và daemon X-Ray riêng của AWS đã vào chế độ bảo trì từ 25/02/2026 (chỉ vá lỗi nghiêm trọng) và hết hỗ trợ ngày 25/02/2027, nên dự án mới dùng OpenTelemetry; trace vẫn hiện trong console X-Ray như cũ. Trace chỉ nối được nếu mọi chặng **truyền ngữ cảnh** đúng:

| Chặng | Cách truyền | Ghi chú |
|---|---|---|
| Client tới ALB tới service | ALB thêm header `X-Amzn-Trace-Id`; OpenTelemetry mặc định dùng `traceparent` (W3C) | Cấu hình propagator X-Ray của OpenTelemetry để đọc header của ALB |
| Service tới service (HTTP) | `traceparent` hoặc header X-Ray | SDK HTTP client tự gắn khi đã instrument |
| Service tới SQS tới Lambda | Thuộc tính hệ thống `AWSTraceHeader` của message | Lambda cần bật active tracing |
| Lambda tới Aurora | Span của driver hoặc span thủ công | Không có propagation vào database; đoạn này nằm trong span của Lambda |

**Sampling rule** của X-Ray quyết định giữ bao nhiêu trace, tập trung ở một chỗ thay vì sửa code. Với 70 chuyến mỗi giây, ghi mọi trace là phí tiền: lab đặt thanh toán 100% (luồng không được mất dấu), đặt chuyến 1 trace chắc chắn mỗi giây cộng 5% phần còn lại. Chọn **annotation** (được đánh chỉ mục, lọc được) cho thuộc tính cần tìm như `city`, `service_type`; để thứ khác ở metadata. Với ADOT, khai báo chúng bằng `indexed_attributes` của exporter. `trip_id` có hàng triệu giá trị nên để ở log, nối với trace qua `trace_id`.

## Log: hỏi một câu cụ thể

Log phải là **JSON một dòng** có `trace_id`, `trip_id`, `city`, `duration_ms`, `level`, như tệp `samples/booking-log.json` trong lab. Khi trace chỉ ra một request chậm, bạn lấy `trace_id` và tìm đúng log của nó trong CloudWatch Logs. **Logs Insights** là ngôn ngữ truy vấn cho việc đó:

```
fields @timestamp, city, duration_ms, trace_id
| filter service = "toinoi-booking" and duration_ms > 800
| stats count() as slow_requests, pct(duration_ms, 99) as p99_ms by city
| sort slow_requests desc
```

Truy vấn lọc request chậm hơn 800 ms, đếm theo thành phố và tính p99. `parse` tách trường từ log chưa có cấu trúc (`parse @message "trip=* took=*ms" as trip, took_ms`), `bin(5m)` gom theo thời gian. Truy vấn tính tiền theo GB log quét, nên giới hạn khoảng thời gian và log group.

Hai cách biến log thành metric. **Metric filter** quét log đã ghi và đếm hoặc lấy giá trị từ trường JSON (lab đếm `level = ERROR` và lấy `duration_ms`). **Embedded Metric Format (EMF)** là cách app tự ghi dòng JSON có khối `_aws.CloudWatchMetrics`, CloudWatch trích metric ngay khi nhận (`samples/emf-example.json`); EMF cho dimension tuỳ ý mà không cần gọi API metric từng lần, và dòng log vẫn còn nguyên để truy vấn. Cẩn thận với dimension nhiều giá trị: mỗi tổ hợp là một metric riêng, tính tiền riêng.

Log chứa dữ liệu khách. **Data protection policy** của log group tự phát hiện và che giá trị khớp mẫu (email, số điện thoại bằng regex tuỳ chỉnh) khi người dùng không có quyền `logs:Unmask`. Policy chỉ che log nhận *sau* khi áp dụng, không sửa log cũ. Đặt retention cho mọi log group; với log hiếm truy vấn, log class `INFREQUENT_ACCESS` giảm chi phí nạp, đổi lại bớt tính năng. Giả định mỗi chuyến sinh 20 dòng log, mỗi dòng 0,5 KB: 2 triệu chuyến mỗi ngày là khoảng 20 GB log mỗi ngày chỉ riêng `booking`.

## Alarm: gọi người đúng lúc

| Loại | Dùng khi | Rủi ro |
|---|---|---|
| Ngưỡng tĩnh trên percentile | Latency p99 vượt ngưỡng SLO (`extended_statistic = p99`) | Dùng trung bình sẽ che mất đuôi chậm |
| Anomaly detection | Lưu lượng có chu kỳ ngày và tuần; báo khi vọt ra ngoài **dải bình thường** học từ lịch sử | Cần vài tuần dữ liệu; sự kiện đặc biệt (Tết, khuyến mãi) làm dải lệch |
| Burn rate | Mức độ đốt error budget của SLO | Cần SLO đã định nghĩa rõ |
| Composite | Chỉ báo khi cả lỗi lẫn chậm cùng xấu: `ALARM(a) AND ALARM(b)` | Điều kiện quá chặt có thể bỏ sót sự cố một chiều |

**Composite alarm** ghép nhiều alarm con bằng biểu thức logic; chỉ composite gửi SNS tới on-call, còn alarm con chỉ đổi trạng thái. Đây là công cụ chính chống **alert fatigue**: ba alarm con cùng nháy khi một dependency chết sẽ thành một cuộc gọi. Nguyên tắc: gọi người theo triệu chứng khách thấy, mỗi alarm gọi người phải có runbook và chủ, và alarm không ai hành động thì sửa hoặc xoá.

## SLO với Application Signals

**CloudWatch Application Signals** thu metric chuẩn của từng service và operation (`Latency`, `Error`, `Fault`) từ service đã instrument bằng ADOT hoặc CloudWatch agent, dựng service map và cho phép định nghĩa SLO trực tiếp trên chúng. Một **SLI** (service level indicator) là phép đo, ví dụ tỉ lệ request tạo chuyến thành công. **SLO** là mục tiêu cho SLI trong một cửa sổ, ví dụ 99,9% trong 30 ngày. **Error budget** là phần còn lại để được phép hỏng: 100% trừ mục tiêu.

Application Signals có hai kiểu SLO: **request-based** (tỉ lệ request tốt trên tổng request) và **period-based** (tỉ lệ khoảng thời gian trong đó SLI đạt ngưỡng). Lab dùng request-based cho availability và period-based cho latency p99 dưới 800 ms:

```json
{
  "Name": "toinoi-booking-availability",
  "RequestBasedSliConfig": {
    "RequestBasedSliMetricConfig": {
      "KeyAttributes": { "Type": "Service", "Name": "toinoi-booking", "Environment": "ecs:toinoi-lab" },
      "OperationName": "POST /v1/trips",
      "MetricType": "AVAILABILITY"
    }
  },
  "Goal": {
    "Interval": { "RollingInterval": { "DurationUnit": "DAY", "Duration": 30 } },
    "AttainmentGoal": 99.9
  },
  "BurnRateConfigurations": [{ "LookBackWindowMinutes": 60 }, { "LookBackWindowMinutes": 360 }]
}
```

Tệp này (cắt bớt) tạo qua `aws application-signals create-service-level-objective --cli-input-json file://...`. `Environment` phải khớp thuộc tính `deployment.environment` mà task ECS khai báo trong `OTEL_RESOURCE_ATTRIBUTES` (không khai báo thì CloudWatch agent tự lấy theo tên cluster), nên lab đặt rõ giá trị này thay vì đoán. `BurnRateConfigurations` cho Application Signals tính metric burn rate trên các cửa sổ 1 giờ và 6 giờ; alarm trên các metric đó tạo riêng (console cho tạo cùng lúc với SLO). Với SLI availability, chỉ lỗi 5xx bị tính là hỏng, 4xx vẫn là thành công. **Burn rate** bằng 1 nghĩa là đốt hết budget đúng vào cuối cửa sổ; bằng 14,4 nghĩa là hết sau 30 ngày chia 14,4, tức khoảng 50 giờ.

| Cửa sổ | Burn rate | Budget bị đốt trong cửa sổ | Hành động |
|---|---|---|---|
| 1 giờ | 14,4 | 2% | Gọi on-call ngay |
| 6 giờ | 6 | 5% | Gọi on-call |
| 3 ngày | 1 | 10% | Mở ticket, không gọi giữa đêm |

Các con số là mức khởi điểm phổ biến trong cộng đồng SRE, hãy chỉnh theo độ nhạy mà Tới Nơi chấp nhận. Lab có thêm một alarm burn rate nhanh dựng bằng metric math trên metric của ALB để có tín hiệu độc lập với Application Signals; đó là xấp xỉ (12 chu kỳ 5 phút liên tiếp), không phải cửa sổ trượt chuẩn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="Hình 2. Từ SLI tới SLO, error budget, burn rate alarm, composite alarm rồi on-call; canary và RUM cấp thêm SLI từ bên ngoài.">
<defs><marker id="aws55-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="10" y="50" width="130" height="80" rx="10" class="f-surface s-line"/><text x="22" y="80" class="t-b">SLI</text><text x="22" y="102" class="t-small">request tốt / tổng</text>
<rect x="165" y="50" width="130" height="80" rx="10" class="f-phys-soft s-phys"/><text x="177" y="80" class="t-b">SLO 99,9%</text><text x="177" y="102" class="t-small">cửa sổ 30 ngày</text>
<rect x="320" y="50" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="332" y="80" class="t-b">Error budget</text><text x="332" y="102" class="t-small">43,2 phút</text>
<rect x="475" y="50" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="487" y="80" class="t-b">Burn rate</text><text x="487" y="102" class="t-small">1 giờ, 6 giờ</text>
<rect x="630" y="50" width="130" height="80" rx="10" class="f-warn-soft s-warn"/><text x="642" y="80" class="t-b t-warn">Composite</text><text x="642" y="102" class="t-small">gom, giảm ồn</text>
<rect x="780" y="50" width="90" height="80" rx="10" class="f-warn-soft s-warn"/><text x="792" y="80" class="t-b t-warn">On-call</text><text x="792" y="102" class="t-small">SNS</text>
<path d="M140 90 L163 90 M295 90 L318 90 M450 90 L473 90 M605 90 L628 90 M760 90 L778 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws55-2-a)"/>
<text x="10" y="170" class="t-small">Synthetics canary (đặt chuyến thử mỗi 5 phút) và RUM (người dùng thật trên web) bổ sung SLI từ phía khách,</text>
<text x="10" y="190" class="t-small">bắt được cả lỗi nằm ngoài tầm nhìn của service như DNS, chứng chỉ, CDN.</text>
</svg>
</div>
<figcaption>Hình 2. Chuỗi từ phép đo tới người trực; mỗi khâu giảm bớt tiếng ồn trước khi tới người.</figcaption>
</figure>

**Synthetics canary** là script chạy theo lịch từ bên ngoài, gọi API như một khách: lab có canary `POST /v1/quotes` mỗi 5 phút kiểm `price > 0`, cho biết ngay cả khi không có khách nào đang dùng. **RUM** thu dữ liệu từ trình duyệt thật (thời gian tải, lỗi JavaScript) cho web đặt chuyến. Cả hai tính phí theo lần chạy hoặc event.

## Cái giá phải trả

- **Log là khoản lớn nhất**: đặt retention, giảm mức log, dùng log class phù hợp, và sampling log `Information` (giữ mọi lỗi).
- **Cardinality**: dimension EMF hoặc nhãn metric như `trip_id`, `driver_id` nổ số metric (bài 47).
- **SLO quá chặt hoặc đo sai**: health check của ALB và canary chiếm phần lớn request có thể làm SLI đẹp giả; loại chúng khỏi SLI hoặc đo theo operation thật.
- **Trace không nối**: một chặng không truyền header là trace đứt đôi; kiểm propagator trước khi tin service map.

## Bài tập

**Bài tập:** SLO availability 99,9% cho một tháng 30 ngày cho phép bao nhiêu phút lỗi toàn phần? Burn rate 14 nghĩa là gì, và vì sao nên gọi người ngay?

<details><summary>Xem đáp án</summary><p>30 × 24 × 60 = 43.200 phút; 0,1% của số đó là <b>43,2 phút</b> (kiểm bằng <code>node slo-math.mjs 99.9 30</code> trong lab). Burn rate 14 nghĩa là đang đốt error budget nhanh gấp 14 lần mức vừa đủ để cạn đúng cuối tháng: ngay cả khi không đổi gì, budget cạn sau 720 giờ chia 14, khoảng 51 giờ, và mỗi giờ ở mức này đã tiêu khoảng 2% budget cả tháng. Giữ nguyên tới sáng thứ hai nghĩa là SLO tháng này coi như thua, nên đây là mức đáng gọi người, trong khi burn rate 1 chỉ cần ticket.</p></details>

## Lab: log, alarm, composite và SLO

Lab ở `examples/toi-noi-aws/p06-55-slo` có Terraform dựng log group kèm data protection và metric filter, bốn alarm và một composite alarm gọi SNS, luật lấy mẫu và group X-Ray, canary và RUM tuỳ chọn; thư mục `slo/` có hai SLO để tạo bằng CLI (mặc định chỉ in lệnh); `slo-math.mjs` tính budget trên máy. Mọi dịch vụ tính phí theo khối lượng (log, alarm nâng cao, canary, RUM); README liệt kê từng khoản, và dọn bằng `terraform destroy` cùng xoá SLO.

```bash
cd examples/toi-noi-aws/p06-55-slo
node slo-math.mjs 99.9 30
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
./create-slos.sh toinoi-lab
```

Terraform đã qua `fmt` và `validate` bằng Docker; JSON đã parse được; mã canary qua `node --check`; hai tệp SLO được kiểm với mô hình `CreateServiceLevelObjective` của botocore. Provider Terraform `~> 6` (bản 6.72) chưa có resource cho SLO của Application Signals nên SLO tạo bằng CLI. Canary dùng runtime `syn-nodejs-puppeteer-18.0` với namespace mới `@aws/synthetics-puppeteer`. **Chưa chạy `apply` hay lệnh AWS thật.**

Bài trước: [bài 54](/articles/aws-cloudtrail-config-security-hub-guardduty). Bài tiếp: [bài 56](/articles/aws-resilience-multi-region-route53-failover-fis).

## Tài liệu gốc

- [AWS X-Ray Developer Guide](https://docs.aws.amazon.com/xray/latest/devguide/aws-xray.html).
- [AWS Distro for OpenTelemetry](https://aws.amazon.com/otel/).
- [CloudWatch Logs Insights query syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CWL_QuerySyntax.html).
- [Application Signals](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Application-Monitoring-Sections.html).
- [Composite alarms](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/Create_Composite_Alarm.html).
- [CloudWatch Synthetics canaries](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries.html).

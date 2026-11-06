---
slug: aws-cloudwatch-budgets-tagging-co-ban
title: "CloudWatch, Budgets và tagging: nhìn thấy hệ thống và chi phí trước khi mở rộng"
summary: "Bài 10: metric, log group, metric filter, alarm và dashboard CloudWatch cho hạ tầng đã dựng ở phần 01; chiến lược tag Project/Env/Service, cost allocation tag, Cost Explorer và Budgets; checklist dọn tài nguyên để không mất tiền."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, cloudwatch, alarm, logs, tagging, cost-explorer, budgets]
language: vi
---

11:20 sáng thứ Ba, service `quote` bắt đầu trả lỗi 5xx cho một phần khách. Không ai trong đội biết cho tới khi bộ phận chăm sóc khách hàng nhận cuộc gọi đầu tiên. Cuối tháng, hoá đơn tăng bất ngờ vì một NAT Gateway của lab bài 04 vẫn chạy suốt hai tuần sau khi cả đội đã quên nó. Cùng một gốc: ta chưa **nhìn thấy** hệ thống và chưa **đếm được** tiền. Bài này khép Phần 01 bằng CloudWatch để nhìn, tag và Budgets để đếm, rồi một checklist dọn sạch mọi thứ đã dựng.

Đây là **bài 10** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần ALB, ASG và RDS đã dựng ở [bài 07](/articles/aws-rds-postgresql-multi-az-backup) và [bài 08](/articles/aws-alb-target-group-auto-scaling), nhưng đọc bài này vẫn hiểu nếu chưa dựng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một alarm đang ở trạng thái <code>INSUFFICIENT_DATA</code>. Có nghĩa là hệ thống đang hỏng không?</p>
<details><summary>Xem đáp án</summary><p>Không nhất thiết. Nó nghĩa là CloudWatch chưa đủ dữ liệu để kết luận, ví dụ metric chưa có điểm nào vì ALB không có request. Cách xử lý là chọn <code>treat-missing-data</code> phù hợp với từng metric, không phải coi mọi lần thiếu dữ liệu là sự cố.</p></details>
</div>

## Metric: namespace, dimension, period và statistic

**CloudWatch** thu **metric**: chuỗi số theo thời gian. Mỗi metric thuộc một **namespace** (ví dụ `AWS/ApplicationELB`, `AWS/RDS`; metric tự đặt như `ToiNoi/Quote`) và được phân biệt bằng **dimension** (cặp tên-giá trị như `LoadBalancer=app/toinoi-lab-quote-alb/...`). Khi đọc, bạn chọn **period** (độ dài mỗi khoảng, tính bằng giây) và **statistic** (Sum, Average, Maximum, hoặc phân vị như p95) để gộp các điểm trong khoảng đó.

| Câu hỏi về `quote` | Namespace | Metric | Statistic |
| --- | --- | --- | --- |
| Bao nhiêu request mỗi phút? | `AWS/ApplicationELB` | `RequestCount` | Sum |
| Chậm không? | `AWS/ApplicationELB` | `TargetResponseTime` | p95 |
| Service có lỗi không? | `AWS/ApplicationELB` | `HTTPCode_Target_5XX_Count` | Sum |
| Máy có quá tải không? | `AWS/EC2` (dimension ASG) | `CPUUtilization` | Average |
| Database còn chỗ không? | `AWS/RDS` | `DatabaseConnections` | Maximum |

Cần phân biệt `HTTPCode_Target_5XX_Count` (lỗi do chính ứng dụng trả) với `HTTPCode_ELB_5XX_Count` (lỗi do ALB sinh, ví dụ không còn target khoẻ). Metric EC2 mặc định lấy mẫu 5 phút, bật detailed monitoring thì 1 phút và tính thêm tiền. Bộ nhớ và dung lượng đĩa **không** có sẵn: phải cài agent (bên dưới) và chúng là metric tuỳ chỉnh, tính tiền theo số metric.

## Alarm và thông báo

**Alarm** theo dõi một metric và có ba trạng thái: `OK`, `ALARM` và `INSUFFICIENT_DATA`. Điều kiện gồm period, ngưỡng, số khoảng cần đánh giá (`evaluation-periods`) và số khoảng phải vi phạm (`datapoints-to-alarm`), nhờ đó "3 trong 5 phút" lọc được nhiễu thoáng qua. Khi đổi trạng thái, alarm có thể gọi một **SNS topic**; topic gửi email tới người đã đăng ký và xác nhận.

```bash
aws sns create-topic --name toinoi-lab-alerts
aws sns subscribe --topic-arn arn:aws:sns:ap-southeast-1:123456789012:toinoi-lab-alerts \
  --protocol email --notification-endpoint oncall@toinoi.example
aws cloudwatch put-metric-alarm --alarm-name toinoi-lab-quote-target-5xx \
  --namespace AWS/ApplicationELB --metric-name HTTPCode_Target_5XX_Count \
  --dimensions Name=LoadBalancer,Value=app/toinoi-lab-quote-alb/0123456789abcdef \
  --statistic Sum --period 60 --evaluation-periods 5 --datapoints-to-alarm 3 \
  --threshold 10 --comparison-operator GreaterThanOrEqualToThreshold \
  --treat-missing-data notBreaching \
  --alarm-actions arn:aws:sns:ap-southeast-1:123456789012:toinoi-lab-alerts
```

Alarm trên báo ALARM khi trong 5 phút có ít nhất 3 phút mà số lỗi 5xx từ target đạt 10 trở lên. `notBreaching` coi lúc không có request là bình thường, vì metric đếm không có điểm dữ liệu khi không có gì để đếm. Đội thật nối topic với kênh chat hoặc công cụ on-call thay vì email.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Metric 5xx của target đi vào alarm có ba trạng thái; khi chuyển sang ALARM, alarm gọi SNS topic và SNS gửi email."><defs><marker id="aws10-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="60" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="90" class="t-b">Metric</text><text x="34" y="112" class="t-small t-mono">HTTPCode_Target_5XX_Count</text><text x="34" y="134" class="t-small">Sum mỗi 60 giây</text><rect x="270" y="30" width="340" height="170" rx="10" class="f-surface s-pk"/><text x="284" y="56" class="t-b">Alarm toinoi-lab-quote-target-5xx</text><text x="284" y="76" class="t-small">3 trong 5 phút có Sum ≥ 10 thì ALARM</text><rect x="285" y="100" width="70" height="34" rx="8" class="f-phys-soft s-phys"/><text x="320" y="122" text-anchor="middle" class="t-small t-b">OK</text><rect x="365" y="100" width="80" height="34" rx="8" class="f-warn-soft s-warn"/><text x="405" y="122" text-anchor="middle" class="t-small t-b t-warn">ALARM</text><rect x="455" y="100" width="140" height="34" rx="8" class="f-surface s-muted dash"/><text x="525" y="122" text-anchor="middle" class="t-small t-b">INSUFFICIENT_DATA</text><text x="284" y="170" class="t-small">treat-missing-data: notBreaching</text><rect x="660" y="30" width="200" height="64" rx="10" class="f-pk-soft s-pk"/><text x="674" y="58" class="t-b">SNS topic</text><text x="674" y="78" class="t-small">toinoi-lab-alerts</text><rect x="660" y="136" width="200" height="64" rx="10" class="f-phys-soft s-phys"/><text x="674" y="164" class="t-b">Email on-call</text><text x="674" y="184" class="t-small">phải xác nhận đăng ký</text><path d="M220 110 H268" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-1-pk)"/><path d="M610 62 H658" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-1-pk)"/><path d="M760 94 V134" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-1-pk)"/></svg>
</div>
<figcaption>Hình 1. Đường đi của một cảnh báo. Alarm cũng có thể báo khi trở lại <code>OK</code> bằng <code>--ok-actions</code>, để người trực biết sự cố đã qua.</figcaption>
</figure>

## Logs: agent, retention và metric filter

Metric cho biết **có** vấn đề, log cho biết **vì sao**. **CloudWatch Logs** gom log theo **log group** (một nhóm, ví dụ `/toinoi/lab/quote`) và **log stream** (một nguồn, ví dụ một instance). Log của ứng dụng trên EC2 không tự lên đây: cài **CloudWatch agent**, cấp cho role của instance policy `CloudWatchAgentServerPolicy`, rồi khai báo file log cần đẩy trong file cấu hình JSON. Lab lưu cấu hình vào Parameter Store và cài lên máy bằng SSM Run Command; instance mới của ASG phải có agent ngay từ launch template hoặc AMI, nếu không chúng lặng lẽ không có log.

Điều chỉnh quan trọng nhất: **retention**. Log group mặc định giữ log **mãi mãi** và tính tiền lưu trữ; lab đặt 14 ngày. Chi phí còn gồm dung lượng log nhận vào và dung lượng mỗi truy vấn Logs Insights quét.

**Metric filter** biến log thành metric: mỗi dòng khớp mẫu cộng vào một metric tuỳ chỉnh. Lab đếm dòng chứa `ERROR` thành `ToiNoi/Quote` / `QuoteErrorCount`, rồi đặt alarm trên metric đó như mọi metric khác.

```bash
aws logs put-metric-filter --log-group-name /toinoi/lab/quote --filter-name quote-error-count \
  --filter-pattern "ERROR" \
  --metric-transformations metricName=QuoteErrorCount,metricNamespace=ToiNoi/Quote,metricValue=1,defaultValue=0
```

**Logs Insights** là ngôn ngữ truy vấn log theo thời gian. Truy vấn cơ bản lấy 20 dòng lỗi gần nhất:

```text
fields @timestamp, @message
| filter @message like /ERROR/
| sort @timestamp desc
| limit 20
```

Log dạng JSON (mỗi dòng một object có `level`, `traceId`, `tripId`) cho phép lọc theo trường thay vì khớp chuỗi. Đừng ghi dữ liệu nhạy cảm như số điện thoại hay token vào log.

## Dashboard cho service báo giá

**Dashboard** gom nhiều widget vào một màn hình, định nghĩa bằng JSON nên có thể đưa vào repo. Dashboard của `quote` có sáu widget: request mỗi phút, latency p95, lỗi 5xx, số target khoẻ và không khoẻ, CPU của ASG, `DatabaseConnections` của RDS. Một widget trông như sau:

```json
{
  "type": "metric", "x": 12, "y": 0, "width": 12, "height": 6,
  "properties": {
    "title": "Quote: latency p95 (giây)", "view": "timeSeries",
    "region": "ap-southeast-1", "period": 60, "stat": "p95",
    "metrics": [["AWS/ApplicationELB", "TargetResponseTime", "LoadBalancer", "app/toinoi-lab-quote-alb/0123456789abcdef"]]
  }
}
```

Dashboard tính tiền theo tháng (có hạn mức miễn phí, xem pricing). Giá trị của nó là đặt số liệu cạnh nhau: nếu request tăng mà latency không đổi, ASG đang làm đúng việc; nếu `DatabaseConnections` tăng theo số instance, đã đến lúc xem connection pool.

## Tag và chi phí: ai tiêu tiền vào việc gì

**Tag** là cặp key-value gắn vào tài nguyên. Không có tag, hoá đơn chỉ cho biết "EC2 tốn bao nhiêu", không cho biết cho ai.

| Tag | Ví dụ | Mục đích |
| --- | --- | --- |
| `Project` | `ToiNoi` | Tách mọi thứ của dự án khỏi tài khoản dùng chung |
| `Env` | `lab`, `dev`, `staging`, `prod` | Biết môi trường nào tốn tiền |
| `Service` | `quote`, `booking` | Chi phí theo service |
| `Owner` | `platform` | Ai chịu trách nhiệm khi cần dọn |
| `CostCenter` | `CC-100` | Gắn với đơn vị chịu chi phí (giá trị giả định của bài) |

Một tag chỉ xuất hiện trong báo cáo chi phí sau khi bạn **kích hoạt cost allocation tag** trong Billing, thường mất tới khoảng một ngày, và **không áp ngược** cho chi phí đã phát sinh trước đó. Kích hoạt chỉ làm được ở tài khoản quản lý (hoặc tài khoản đơn lẻ). Sau khi kích hoạt, **Cost Explorer** cho nhóm chi phí theo tag (`group by Tag: Project`). Mỗi lần gọi API Cost Explorer có thể bị tính phí nhỏ, nên đừng đặt vào vòng lặp.

**AWS Budgets** đặt hạn mức theo tháng và gửi cảnh báo khi chi phí thực tế hoặc dự báo vượt phần trăm đã chọn. Lab dùng lại budget `toinoi-lab-monthly` của bài 02 nếu đã có, chưa có thì tạo với hạn mức do bạn chọn, cảnh báo qua email ở 80% thực tế và 100% dự báo. Budget có thể lọc theo tag. API Budgets hiện khuyên dùng `FilterExpression` cùng `Metrics` thay cho `CostFilters` đã deprecated; lọc theo tag `Project` trông như sau (cách ghi khoá tag chưa kiểm trên tài khoản thật):

```json
"FilterExpression": { "Tags": { "Key": "Project", "Values": ["ToiNoi"] } },
"Metrics": ["UNBLENDED_COST"]
```

Budgets còn có **action**: khi vượt ngưỡng, tự gắn một IAM policy chặn tạo thêm tài nguyên, hoặc dừng EC2/RDS chỉ định. Nó cần role riêng và quyền thao tác, nên bài này chỉ mô tả, chưa chạy. Điều cần nhớ: dữ liệu chi phí cập nhật chậm hơn thực tế vài giờ tới cả ngày, nên Budgets là lưới báo động trễ, không phải cầu dao tức thì.

## Dọn tài nguyên: checklist toàn Phần 01

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Thứ tự dọn tài nguyên Phần 01: giám sát, ASG và ALB, RDS, S3, EC2, NAT Gateway rồi Elastic IP, VPC, cuối cùng IAM."><defs><marker id="aws10-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="58" class="t-b">① Giám sát, SNS</text><text x="34" y="78" class="t-small">alarm, dashboard, log, budget</text><rect x="237" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="251" y="58" class="t-b">② ASG và ALB</text><text x="251" y="78" class="t-small">tính tiền theo giờ</text><rect x="454" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="468" y="58" class="t-b">③ RDS và snapshot</text><text x="468" y="78" class="t-small">snapshot không tự hết hạn</text><rect x="671" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="685" y="58" class="t-b">④ S3 và mọi version</text><text x="685" y="78" class="t-small">version cũ vẫn tính tiền</text><rect x="20" y="160" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="188" class="t-b">⑤ EC2, EBS, AMI</text><text x="34" y="208" class="t-small">stopped vẫn trả tiền đĩa</text><rect x="237" y="160" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="251" y="188" class="t-b t-warn">⑥ NAT Gateway, rồi EIP</text><text x="251" y="208" class="t-small">NAT chạy theo giờ</text><rect x="454" y="160" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="468" y="188" class="t-b">⑦ Subnet, IGW, VPC</text><text x="468" y="208" class="t-small">xoá VPC sau cùng</text><rect x="671" y="160" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="685" y="188" class="t-b">⑧ IAM, Route 53</text><text x="685" y="208" class="t-small">role, policy, hosted zone</text><path d="M210 70 H235" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/><path d="M427 70 H452" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/><path d="M644 70 H669" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/><path d="M766 110 V135 H115 V158" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/><path d="M210 200 H235" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/><path d="M427 200 H452" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/><path d="M644 200 H669" class="f-none s-pk" stroke-width="2" marker-end="url(#aws10-2-pk)"/></svg>
</div>
<figcaption>Hình 2. Dọn theo thứ tự ngược với lúc dựng, vì tài nguyên phụ thuộc nhau: security group chưa xoá được khi còn instance, Elastic IP chưa release được khi còn gắn vào NAT, VPC chưa xoá được khi còn subnet.</figcaption>
</figure>

Những thứ tính tiền theo giờ hoặc tồn tại âm thầm, theo thứ tự nên kiểm:

- **NAT Gateway**: tính theo giờ tồn tại và theo dữ liệu đi qua, kể cả khi không có traffic. Xoá NAT rồi mới release Elastic IP của nó.
- **ALB** (bài 08): tính theo giờ. **RDS instance** (bài 07): tính theo giờ, kể cả khi bạn không truy vấn.
- **Elastic IP**: mọi địa chỉ IPv4 public đều tính tiền, kể cả EIP không gắn vào gì.
- **Snapshot** (RDS thủ công, EBS) và **version cũ trong S3**: không tự hết hạn.
- **Log group**, VPC endpoint loại Interface, và hosted zone Route 53 nếu bạn đã tạo ở bài 09.

Lab bài này có hai script cho việc đó. `90-audit-leftovers.sh` **chỉ đọc**, liệt kê những gì còn sót ở region hiện tại; `99-cleanup-all.sh` xoá alarm, dashboard, log, SNS, budget rồi gọi script dọn của lab 08, 07, 06 theo đúng thứ tự trên, và in sẵn lệnh dọn của lab 05, 04, 03 cùng các bước tay cho bài 09. Mọi thao tác xoá hỏi xác nhận tài khoản trước.

```bash
cd examples/toi-noi-aws/p01-10-cloudwatch-cleanup
cp env.sh.example env.sh && ./01-alarms-logs.sh && ./02-dashboard.sh
./90-audit-leftovers.sh      # chỉ đọc
./99-cleanup-all.sh          # xoá thật, chỉ chạy trong tài khoản lab
```

Script đã qua kiểm tra cú pháp bash, shellcheck và kiểm tên tham số ở chế độ không mạng; chưa chạy trên tài khoản thật. Cách ghi khoá tag trong `FilterExpression` của Budgets và cú pháp cấu hình agent cần bạn xác nhận khi chạy. Audit chỉ nhìn một region; nếu từng thử ở region khác, đặt `AWS_REGION` rồi chạy lại.

## Cái giá phải trả

- **Metric tuỳ chỉnh tính tiền theo từng tổ hợp dimension.** Đặt dimension có số giá trị lớn (như `tripId`) sẽ tạo hàng triệu metric; dùng log và Logs Insights cho dữ liệu chi tiết.
- **Log mặc định giữ mãi.** Quên retention là khoản phí lưu trữ tăng đều mỗi tháng.
- **Alarm quá nhạy tạo mệt mỏi.** Đội bắt đầu bỏ qua email; ưu tiên ít alarm có ý nghĩa, mỗi alarm có người chịu trách nhiệm và hướng xử lý.
- **Tag không áp ngược.** Tài nguyên tạo trước khi có chiến lược tag không xuất hiện đúng trong báo cáo; gắn tag ngay từ launch template và IaC.
- **Budgets báo trễ.** Hạn mức thấp cùng với cảnh báo dự báo giúp bắt sớm, nhưng không thay cho việc dọn tài nguyên ngay khi học xong.

**Bài tập:** khi một instance EC2 chuyển sang `stopped`, những tài nguyên nào liên quan vẫn tiếp tục tính tiền?

<details><summary>Xem đáp án</summary><p>Tiền chạy instance (theo giây) dừng, nhưng <b>EBS volume</b> gắn kèm vẫn tính theo GB-tháng, <b>snapshot và AMI</b> đã tạo vẫn tính, và <b>Elastic IP</b> gắn vào instance vẫn tính tiền địa chỉ IPv4 public. Địa chỉ public tự cấp thì bị thu hồi khi dừng. Các tài nguyên đứng riêng như NAT Gateway, ALB, RDS không liên quan tới trạng thái của EC2 và vẫn tính theo giờ.</p></details>

Bài trước: [bài 09](/articles/aws-route53-acm-https). Bài tiếp: [bài 11](/articles/aws-ecr-image-scan-lifecycle), mở đầu Phần 02 về container và serverless.

## Tài liệu gốc

- [Amazon CloudWatch User Guide](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/WhatIsCloudWatch.html).
- [CloudWatch alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html).
- [Amazon CloudWatch Logs](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/WhatIsCloudWatchLogs.html).
- [Cú pháp truy vấn Logs Insights](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CWL_QuerySyntax.html).
- [Cost allocation tag](https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/cost-alloc-tags.html).
- [AWS Budgets](https://docs.aws.amazon.com/cost-management/latest/userguide/budgets-managing-costs.html).

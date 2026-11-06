---
slug: aws-finops-savings-plans-spot-right-sizing
title: "FinOps trên AWS: cost allocation, Savings Plans, Reserved, Spot, right-sizing và kiến trúc tiết kiệm"
summary: "Bài 59: đọc hoá đơn Tới Nơi theo tag với Cost Explorer, CUR 2.0 và Athena, chi phí mỗi chuyến làm KPI; cam kết bằng Savings Plans và Reserved từ mức nền, Spot cho worker; right-sizing với Compute Optimizer và Cost Optimization Hub; chi phí ẩn của NAT, log, snapshot; budget, anomaly detection và quy trình hằng tháng."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, finops, cost-optimization, savings-plans, spot, right-sizing, cost-explorer]
language: vi
---

Hoá đơn AWS của Tới Nơi tăng 30% sau hai tháng mở thêm thành phố, trong khi số chuyến chỉ tăng 20%. Ban lãnh đạo hỏi: "Tăng vì cần, hay vì lãng phí?". Trả lời "EC2 và NAT tốn nhất" chưa cho biết phải sửa gì; cần **chi phí mỗi chuyến** tách theo service và owner, rồi mới tới cam kết hay Spot.

Đây là **bài 59** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần bài về tag và Budgets ([bài 10](/articles/aws-cloudwatch-budgets-tagging-co-ban)), và nên đã biết ECS (bài 12, 13) cùng autoscaling của EKS (bài 45).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tải của <code>booking</code> đạt đỉnh lúc 11:00 và 17:00. Bạn mua Savings Plans đúng bằng mức đỉnh để "chắc chắn được giảm giá". Có hợp lý không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Cam kết trả tiền theo giờ dù dùng hay không. Ngoài giờ cao điểm, phần cam kết vượt mức dùng bị bỏ phí. Mua theo mức <b>nền</b> (luôn có 24/7), để phần đỉnh chạy on-demand hoặc Spot.</p></details>
</div>

## Mô hình giá và mua bao nhiêu

Cùng một tải có thể trả theo nhiều cách:

| Mô hình | Cam kết | Áp cho | Đánh đổi |
| --- | --- | --- | --- |
| On-demand | Không | Mọi thứ | Linh hoạt nhất, đắt nhất |
| Compute Savings Plans | 1 hoặc 3 năm, theo $/giờ | EC2 mọi family và region, Fargate, Lambda | Giảm nhiều, giữ linh hoạt đổi kiến trúc |
| EC2 Instance Savings Plans | 1 hoặc 3 năm | Một family trong một region | Giảm sâu hơn, kém linh hoạt |
| Database Savings Plans (từ 12/2025) | 1 năm, theo $/giờ, no upfront | Aurora, RDS, DynamoDB, ElastiCache Valkey và vài DB khác | Linh hoạt giữa engine, cỡ và region; không cộng dồn với Reserved cho cùng tải |
| Reserved (RDS, ElastiCache, OpenSearch, Redshift) | 1 hoặc 3 năm | Đúng loại và cỡ node | Giảm sâu hơn nhưng gắn chặt engine và cỡ |
| Spot | Không | EC2, Fargate Spot, worker chịu gián đoạn | Giảm rất lớn nhưng có thể bị thu hồi sau khoảng 2 phút báo trước |

Compute, EC2 Instance Savings Plans và Reserved có ba cách trả: **no upfront**, **partial upfront**, **all upfront**; trả trước nhiều thì giảm nhiều hơn nhưng khoá vốn. Hai chỉ số theo dõi sau khi mua:

- **Coverage**: phần chi tiêu đủ điều kiện được cam kết chi trả.
- **Utilization**: phần cam kết thực sự được dùng. Dưới 100% nghĩa là đang trả cho thứ không dùng.

Ví dụ với đơn vị quy ước (tính theo giá on-demand, đơn giản hoá). Tải của `booking` và `quote` là **20 đơn vị/giờ** cả ngày, cộng thêm 30 đơn vị trong 4 giờ cao điểm. Tổng một ngày: `20×24 + 30×4 = 600`.

| Mức cam kết | Phần dùng được phủ | Coverage | Utilization |
| --- | --- | --- | --- |
| 20 đơn vị/giờ (nền) | 480 | 80% | 100% |
| 35 đơn vị/giờ | 540 | 90% | khoảng 64% |

Cam kết 35 phủ nhiều hơn, nhưng 20 giờ mỗi ngày chỉ dùng 20 nên phần dư bị bỏ phí. Quy tắc: bắt đầu bằng mức nền đã ổn định vài tháng rồi tăng dần; phần đỉnh để on-demand hoặc Spot. Và **right-size trước, cam kết sau**, nếu không bạn cam kết cho cả sự lãng phí.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Tải một ngày của booking và quote: nền 20 đơn vị 24 giờ được Savings Plans phủ, hai khung cao điểm 11 đến 13 giờ và 17 đến 19 giờ chạy on-demand hoặc Spot."><rect x="20" y="20" width="840" height="230" rx="12" class="f-surface s-line"/><rect x="80" y="136" width="760" height="64" class="f-phys-soft s-phys"/><text x="96" y="172" class="t-b">Savings Plans: nền 20 đơn vị/giờ, 24/7</text><rect x="428" y="40" width="63" height="96" class="f-pk-soft s-pk"/><rect x="618" y="40" width="63" height="96" class="f-pk-soft s-pk"/><text x="376" y="86" text-anchor="end" class="t-small">Cao điểm 11:00 - 13:00</text><text x="722" y="86" class="t-small">Cao điểm 17:00 - 19:00</text><text x="376" y="106" text-anchor="end" class="t-small">on-demand hoặc Spot</text><text x="722" y="106" class="t-small">on-demand hoặc Spot</text><path d="M80 200 H840" class="f-none s-muted" stroke-width="1"/><text x="80" y="220" text-anchor="middle" class="t-small t-mono">0h</text><text x="270" y="220" text-anchor="middle" class="t-small t-mono">6h</text><text x="460" y="220" text-anchor="middle" class="t-small t-mono">12h</text><text x="650" y="220" text-anchor="middle" class="t-small t-mono">18h</text><text x="840" y="220" text-anchor="middle" class="t-small t-mono">24h</text><text x="36" y="46" class="t-b">Cam kết ở mức nền, đỉnh để linh hoạt</text><text x="36" y="240" class="t-small">Đơn vị quy ước, không phải số liệu thật</text></svg>
</div>
<figcaption>Hình 1. Savings Plans phủ phần luôn chạy. Hai khung giờ cao điểm là phần biến đổi nên không cam kết, chạy on-demand hoặc đưa worker sang Spot.</figcaption>
</figure>

**Spot** chỉ nên gánh những việc chịu được bị ngắt: worker xử lý hàng đợi `dispatch` và đối soát (bài 16, 57), job build. ECS dùng capacity provider `FARGATE_SPOT`, EKS dùng Karpenter (bài 45) với nhiều instance type để giảm khả năng bị thu hồi cùng lúc. Task nhận tín hiệu dừng phải thoát nhanh; message chưa xử lý quay lại queue nhờ visibility timeout. Đừng chạy `booking` hoàn toàn trên Spot.

## Nhìn thấy: tag, category và chi phí mỗi chuyến

Cam kết hay Spot đều vô nghĩa nếu không biết ai tiêu gì. Các lớp dữ liệu:

- **Cost allocation tag**: kích hoạt trong Billing (bài 10) cho `Project`, `Env`, `Service`, `Owner`. Chỉ áp từ lúc kích hoạt, không áp ngược.
- **Cost Categories**: nhóm chi phí theo quy tắc, ví dụ lớp "compute", "data", "edge" cho báo cáo lãnh đạo.
- **CUR 2.0 qua Data Exports**: xuất chi phí chi tiết từng dòng usage ra S3, truy vấn bằng Athena (bài 38).
- **Cost Explorer và API**: xem nhanh, nhóm theo tag; API tính phí theo lần gọi nên đừng đặt vào vòng lặp.
- **Split Cost Allocation Data** (bật trong Cost Management preferences): chia chi phí compute chung của ECS (theo task) và EKS (theo pod) về từng service, hiện trong CUR 2.0. Thiếu nó, cluster EKS dùng chung chỉ là một dòng lớn.

KPI trung tâm là **chi phí mỗi chuyến**: chi phí chia cho số chuyến cùng kỳ, tính bằng Athena trên CUR 2.0 và bảng chuyến của lakehouse (tên bảng, cột là giả định, kiểm bằng `DESCRIBE`):

```sql
WITH cost AS (
  SELECT date_trunc('month', line_item_usage_start_date) AS month,
         sum(line_item_unblended_cost) AS cost
  FROM cur_db.cur2
  WHERE element_at(resource_tags, 'user_project') = 'ToiNoi'
  GROUP BY 1
), trips AS (
  SELECT date_trunc('month', trip_date) AS month, sum(trip_count) AS trips
  FROM analytics.trips_daily GROUP BY 1
)
SELECT c.month, c.cost, t.trips, c.cost / t.trips AS cost_per_trip
FROM cost c JOIN trips t ON c.month = t.month ORDER BY 1;
```

Truy vấn cộng chi phí của mọi tài nguyên gắn `Project=ToiNoi` theo tháng rồi chia cho số chuyến. `resource_tags` là map với khoá dạng `user_project` (kiểm bằng `map_keys`); dùng `element_at` vì `resource_tags['...']` báo lỗi ở dòng thiếu khoá. Xu hướng quan trọng hơn con số tuyệt đối: chi phí mỗi chuyến tăng theo số chuyến nghĩa là kiến trúc đang mất hiệu quả theo quy mô.

## Giảm: nơi tiền thường rò rỉ

| Nguồn lãng phí | Cách phát hiện | Cách sửa |
| --- | --- | --- |
| Tài nguyên cỡ thừa | Compute Optimizer, Cost Optimization Hub | Right-size theo đề xuất, sau khi xem đỉnh thật |
| Chưa dùng Graviton | Khuyến nghị trong Hub | Image arm64, task `ARM64` (bài 12), NodePool arm64 (bài 45) |
| Ổ đĩa gp2 | Cost Explorer theo usage type | `modify-volume --volume-type gp3`, không cần dừng |
| Log không retention, snapshot cũ | Trusted Advisor, script | Retention theo loại log; lifecycle cho snapshot và S3 (bài 36) |
| NAT Gateway xử lý traffic S3/DynamoDB | Cost Explorer theo usage type | Gateway endpoint (không tính tiền theo giờ), interface endpoint khi cần (bài 53) |
| Truyền dữ liệu giữa AZ và region | CUR theo `usage_type` | Gom thành phần hay nói chuyện cùng AZ khi chịu được, cache |
| Load balancer, public IPv4 và EBS không dùng | Hub, Trusted Advisor | Xoá; mọi public IPv4 đều tính phí theo giờ |
| Môi trường dev chạy 24/7 | Tag `Env=dev` theo giờ | Scheduled scaling về 0 ngoài giờ, tắt DB tạm thời |

**Compute Optimizer** phân tích metric (EC2, EBS, Lambda, ECS trên Fargate, RDS, và nhiều loại khác) rồi đề xuất cỡ phù hợp; cần bật opt-in và vài ngày đến vài tuần dữ liệu. **Cost Optimization Hub** (endpoint `us-east-1`, cần opt-in) gom khuyến nghị của nhiều dịch vụ vào một danh sách, ước tính tiết kiệm sau giảm giá của bạn, kể cả khuyến nghị mua Savings Plans và Reserved; đó là điểm xuất phát, không phải lệnh. Với Aurora Serverless v2, DynamoDB on-demand hay Lambda, "right-size" là giới hạn dung lượng và cấu hình bộ nhớ, không phải đổi instance.

Kiến trúc tiết kiệm có hai mặt: giảm lượng việc (cache báo giá ở bài 15, nén Parquet và phân vùng toạ độ lịch sử ở bài 35, lifecycle dữ liệu cũ) và chọn dịch vụ khớp mẫu tải. Kinesis hay MSK, ECS hay EKS còn là quyết định chi phí vận hành: con người cũng là chi phí.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 2. Vòng FinOps hằng tháng: Inform cho thấy chi phí theo tag và theo chuyến, Optimize right-size trước rồi cam kết, Operate đặt budget, anomaly detection và review tháng; kết quả quay lại Inform."><defs><marker id="aws59-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="50" width="240" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="80" class="t-b">Inform</text><text x="36" y="104" class="t-small">tag, Cost Category, CUR 2.0</text><text x="36" y="124" class="t-small">chi phí mỗi chuyến theo service</text><text x="36" y="144" class="t-small">owner nhìn thấy số của mình</text><rect x="320" y="50" width="240" height="110" rx="10" class="f-pk-soft s-pk"/><text x="336" y="80" class="t-b">Optimize</text><text x="336" y="104" class="t-small">right-size trước</text><text x="336" y="124" class="t-small">rồi Savings Plans, Reserved</text><text x="336" y="144" class="t-small">Spot cho worker, dọn rác</text><rect x="620" y="50" width="240" height="110" rx="10" class="f-phys-soft s-phys"/><text x="636" y="80" class="t-b">Operate</text><text x="636" y="104" class="t-small">Budgets, Cost Anomaly Detection</text><text x="636" y="124" class="t-small">review hằng tháng với owner</text><text x="636" y="144" class="t-small">đưa vào quy trình thiết kế</text><path d="M260 105 H318" class="f-none s-pk" stroke-width="2" marker-end="url(#aws59-2-pk)"/><path d="M560 105 H618" class="f-none s-pk" stroke-width="2" marker-end="url(#aws59-2-pk)"/><path d="M740 160 V200 H140 V162" class="f-none s-muted dash" stroke-width="2" marker-end="url(#aws59-2-pk)"/><text x="440" y="194" text-anchor="middle" class="t-small">đo lại sau mỗi thay đổi</text></svg>
</div>
<figcaption>Hình 2. Ba pha của FinOps lặp lại hằng tháng. Cam kết nằm ở pha Optimize, sau khi right-size, và được theo dõi tiếp bằng coverage, utilization ở pha Operate.</figcaption>
</figure>

## Quy trình FinOps hằng tháng

FinOps là thói quen, không phải một dự án: **Inform** (nhìn thấy), **Optimize** (giảm), **Operate** (giữ kỷ luật). Với Tới Nơi:

- **Owner theo service**: con số không có người nhận sẽ không giảm.
- **Budgets** theo `Project` và theo môi trường với cảnh báo 80% thực tế, 100% dự báo (bài 10). Thêm budget cho **Savings Plans utilization** và **coverage** sau khi đã mua.
- **Cost Anomaly Detection** dùng học máy báo chi tiêu bất thường theo dịch vụ hoặc tag (NAT tăng gấp ba vì một service gọi S3 qua Internet): lưới bắt lỗi cấu hình, còn Budgets chỉ báo khi tổng vượt ngưỡng.
- **Review hằng tháng** 30 phút: đọc chi phí mỗi chuyến, xem anomaly, chốt hai ba việc giảm. Mua cam kết mới chỉ ở buổi review, kèm giả định (nền tải, thời hạn), vì mua xong khó huỷ.

## Cái giá phải trả

- **Cam kết là khoản nợ**: Savings Plans và Reserved gần như không huỷ được; đổi kiến trúc lớn (đổi engine DB, đổi họ instance với EC2 Instance SP) có thể bỏ phí cam kết.
- **Cắt giảm quá tay làm hại độ tin cậy**: bỏ dư thừa của bài 56 để "tiết kiệm" thì lần sự cố sau trả giá đắt hơn. Một số "lãng phí" là bảo hiểm.
- **Tag thiếu nghĩa là báo cáo sai**: tài nguyên không có tag rơi vào nhóm "không rõ", và con số mỗi chuyến vô nghĩa.
- **Spot không có SLA**: worker phải chịu bị ngắt giữa chừng và xử lý idempotent.

**Bài tập:** tải 24/7 là 20 đơn vị/giờ, cộng 30 đơn vị trong 4 giờ cao điểm mỗi ngày. Tính coverage và utilization nếu cam kết 30 đơn vị/giờ, và cho biết bạn có chọn mức đó không.

<details><summary>Xem đáp án</summary><p>Mỗi ngày: 20 giờ dùng 20, 4 giờ dùng 50. Phần được phủ: <code>20×20 + 4×30 = 520</code>. Tổng dùng 600, nên coverage = 520/600 ≈ <b>86,7%</b>. Cam kết cả ngày là <code>30×24 = 720</code>, nên utilization = 520/720 ≈ <b>72,2%</b>, tức gần 28% cam kết bị bỏ phí. So với mức 20, thêm 6,7 điểm coverage nhưng trả tiền cho 200 đơn vị không dùng. Thường không chọn, trừ khi tải nền sắp tăng; hợp lý hơn là cam kết 20 rồi tăng khi nền thật sự lên.</p></details>

## Lab: budget và anomaly monitor bằng Terraform

Thư mục `examples/toi-noi-aws/p06-59-finops` có Terraform dựng budget theo tag `Project` (cảnh báo thực tế và dự báo), hai anomaly monitor (theo dịch vụ và theo tag) cùng subscription hằng ngày, một Cost Category, budget coverage và utilization của Savings Plans (tắt mặc định), cộng `queries/cost-per-trip.sql` và `scripts/finops-review.sh` (chỉ lệnh đọc: chi phí theo tag, coverage, utilization, khuyến nghị mua).

```bash
cd examples/toi-noi-aws/p06-59-finops
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/finops-review.sh
```

Terraform đã qua `fmt` và `validate` nhưng **chưa `apply` trên tài khoản thật**. Budget lọc tag bằng `filter_expression` như `FilterExpression` ở bài 10 (tag phải đã kích hoạt cost allocation); tên bảng và cột CUR 2.0 trong SQL là giả định, kiểm schema trước khi chạy. Budget, anomaly monitor và Cost Category không tính tiền riêng; chi phí lab chủ yếu là các lệnh Cost Explorer API trong script.

Bài trước: [bài 58](/articles/aws-well-architected-review-toi-noi). Bài tiếp: [bài 60](/articles/aws-tong-hop-kien-truc-production-toi-noi).

## Tài liệu gốc

- [Savings Plans User Guide](https://docs.aws.amazon.com/savingsplans/latest/userguide/what-is-savings-plans.html).
- [AWS Cost Optimization Hub](https://docs.aws.amazon.com/cost-management/latest/userguide/cost-optimization-hub.html).
- [AWS Compute Optimizer](https://docs.aws.amazon.com/compute-optimizer/latest/ug/what-is-compute-optimizer.html).
- [Data Exports và CUR 2.0](https://docs.aws.amazon.com/cur/latest/userguide/what-is-data-exports.html).
- [AWS Cost Anomaly Detection](https://docs.aws.amazon.com/cost-management/latest/userguide/manage-ad.html).
- [FinOps Framework](https://www.finops.org/framework/).

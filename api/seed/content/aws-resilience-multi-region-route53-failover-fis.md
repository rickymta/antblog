---
slug: aws-resilience-multi-region-route53-failover-fis
title: "Resilience: multi-AZ tới multi-region, Route 53 failover và ARC, Global Accelerator, Fault Injection Service"
summary: "Bài 56: Tới Nơi sống qua mất một AZ nhờ static stability, cell theo thành phố, retry có jitter; dữ liệu đa region với Aurora Global Database và DynamoDB Global Tables; Route 53 failover, ARC routing control, zonal shift, Global Accelerator; diễn tập bằng Fault Injection Service có stop condition."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, resilience, multi-region, route53, application-recovery-controller, fault-injection, chaos-engineering]
language: vi
---

Lúc 17:40, giờ cao điểm, một availability zone của `ap-southeast-1` bị nghẽn mạng. Một phần ba task của `booking` không gọi được Aurora, ALB đánh dấu chúng lỗi, và 70 chuyến mỗi giây dồn về hai AZ còn lại. Nếu hai AZ đó đã chạy ở 90% công suất thì chúng sập nốt trong vài phút. Câu hỏi là ta đã chừa sẵn chỗ trống để chịu mất một phần hay chưa, và đã thử điều đó bao giờ chưa.

Đây là **bài 56** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần nắm RPO/RTO và các chiến lược DR ở [bài 37](/articles/aws-backup-disaster-recovery-rpo-rto), cùng DNS và health check của [bài 09](/articles/aws-route53-acm-https).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Region chính mất, bạn đổi bản ghi Route 53 sang region phụ bằng console. Việc đổi bản ghi đó có thể chính nó cũng hỏng trong sự cố không?</p>
<details><summary>Xem đáp án</summary><p>Có thể. Đổi bản ghi là thao tác của <b>control plane</b>, còn việc trả lời DNS là <b>data plane</b>. Thiết kế tốt không gọi control plane lúc sự cố: dùng health check tự động hoặc routing control của ARC (data plane riêng) và dựng sẵn công suất ở nơi nhận tải.</p></details>
</div>

## Chọn miền lỗi cần sống sót

Mỗi cơ chế chống một **miền lỗi** (failure domain) khác nhau, và mỗi lớp bảo vệ tốn thêm tiền.

| Miền lỗi | Cơ chế chính | Độ khó |
| --- | --- | --- |
| Một task hoặc instance | Health check, auto scaling (bài 08, 13) | Thấp |
| Một AZ | Multi-AZ, static stability, zonal shift | Vừa |
| Một dependency chậm | Timeout, retry có jitter, circuit breaker | Vừa |
| Một region | Dữ liệu đa region, Route 53 + ARC, Global Accelerator | Cao |
| Một lỗi triển khai | Canary, rollback, cell (bài 26, 27) | Vừa |

Mất cả region hiếm hơn nhiều so với các dòng khác, nhưng chỉ ai chuẩn bị trước mới phục hồi nhanh.

## Static stability, cell và retry

**Static stability** nghĩa là hệ thống giữ nguyên hoạt động khi một phần hạ tầng mất, mà không cần gọi API nào để sửa. Chờ Auto Scaling bật thêm task sau khi AZ mất nghĩa là phụ thuộc EC2, ECS và quota đúng lúc cả khu vực đang rối. Cách vững hơn là cấp **dư sẵn**: với N AZ, mỗi AZ gánh được `1/(N-1)` tải đỉnh; 3 AZ thì mỗi AZ bằng 50% tải đỉnh, tổng 150%. Đó là khoản bảo hiểm trả liên tục để không phải bật gì lúc sự cố.

**Cell-based architecture** chia hệ thống thành nhiều bản sao độc lập gọi là cell, mỗi cell phục vụ một phần khách hàng. Với Tới Nơi, cell tự nhiên là **thành phố**: `toinoi-prod-booking-hcm`, `toinoi-prod-booking-hn`... có service, queue và (ở mức dữ liệu) partition riêng. Lỗi triển khai hay tải bất thường chỉ chạm một cell, blast radius còn khoảng một phần sáu; cái giá là sáu bản để vận hành và một lớp định tuyến mỏng phía trước.

Ba thói quen phần mềm quyết định một sự cố nhỏ có lan hay không:

- **Timeout cho mọi lời gọi**, thấp hơn timeout của bên gọi; không có thì một dependency chậm giữ hết thread.
- **Retry với exponential backoff và jitter**; thiếu jitter, mọi client thử lại cùng lúc và đánh sập service vừa hồi phục.
- **Circuit breaker**: quá ngưỡng lỗi thì ngừng gọi một lúc, trả lỗi nhanh hoặc dữ liệu cache. SDK AWS có retry mode `standard` và `adaptive` (`AWS_RETRY_MODE`); lời gọi sang service nội bộ phải tự cài.

```text
sleep = random(0, min(cap, base * 2^attempt))
```

Chọn `base` 100 ms, `cap` vài giây và giới hạn tổng số lần thử: nếu 3 tầng mỗi tầng thử 3 lần, một request lỗi thành 27 lời gọi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. Ba availability zone, mỗi zone có task booking và NAT; zone c bị mất, hai zone còn lại tiếp nhận tải vì đã được cấp dư sẵn."><defs><marker id="aws56-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="20" width="840" height="220" rx="12" class="f-surface s-line"/><text x="36" y="44" class="t-b">ap-southeast-1</text><rect x="40" y="62" width="240" height="150" rx="10" class="f-phys-soft s-phys"/><text x="56" y="88" class="t-b">AZ a</text><text x="56" y="112" class="t-small">booking: đủ 50% tải đỉnh</text><text x="56" y="132" class="t-small">NAT, subnet private</text><text x="56" y="160" class="t-mono">đang gánh 50%</text><rect x="320" y="62" width="240" height="150" rx="10" class="f-phys-soft s-phys"/><text x="336" y="88" class="t-b">AZ b</text><text x="336" y="112" class="t-small">booking: đủ 50% tải đỉnh</text><text x="336" y="132" class="t-small">NAT, subnet private</text><text x="336" y="160" class="t-mono">đang gánh 50%</text><rect x="600" y="62" width="240" height="150" rx="10" class="f-warn-soft s-warn dash"/><text x="616" y="88" class="t-b t-warn">AZ c (mất)</text><text x="616" y="112" class="t-small">tất cả task không phục vụ</text><text x="616" y="132" class="t-small">ALB bỏ khỏi target</text><text x="616" y="160" class="t-mono t-warn">0%</text><path d="M600 187 H564" class="f-none s-pk" stroke-width="2" marker-end="url(#aws56-1-pk)"/><text x="582" y="180" text-anchor="middle" class="t-small">dồn</text><text x="56" y="196" class="t-small">Không cần bật gì thêm lúc sự cố</text></svg>
</div>
<figcaption>Hình 1. Static stability: ba AZ, mỗi AZ đủ 50% tải đỉnh. Mất một AZ thì hai AZ còn lại vừa đủ 100% mà không phụ thuộc scale-up trong sự cố.</figcaption>
</figure>

## Dữ liệu và bí mật ở region thứ hai

Chuyển traffic sang region phụ vô nghĩa nếu dữ liệu không có ở đó:

| Kho | Cơ chế đa region | Điều cần biết |
| --- | --- | --- |
| Aurora PostgreSQL | **Global Database**: cluster phụ chỉ đọc, nhân bản ở tầng storage | RPO tính bằng giây. **Switchover** có kế hoạch không mất dữ liệu; **failover** không kế hoạch có thể mất phần chưa nhân bản |
| DynamoDB | **Global Tables**: nhiều region ghi được | Mặc định (MREC) bất đồng bộ, last-writer-wins. Chế độ **MRSC** (GA 2025) cho RPO bằng 0 nhưng cần đúng ba region (hoặc hai cộng witness), không có TTL, LSI, transaction |
| S3 | Cross-Region Replication (bài 36) | Bất đồng bộ, theo object |
| Secrets Manager, KMS, ECR | Replica secret, multi-Region key (bài 52), replication image | Thiếu một món là task ở region phụ không khởi động được |

Với Tới Nơi: ví và `trips` ghi ở một region tại một thời điểm (Aurora Global), vị trí tài xế tự dựng lại sau 4 giây nên không nhân bản, ảnh chứng từ dùng S3 replication. Global Tables ghi hai nơi chỉ hợp dữ liệu chịu được ghi cuối thắng, không phải số dư ví.

Hạ tầng region B là code (bài 22, 23): cùng module, khác `region`; **quota** của region phụ (vCPU Fargate, ENI) không tự nhân đôi. Lệnh dưới thuộc runbook, không chạy trong lab:

```bash
aws rds switchover-global-cluster \
  --global-cluster-identifier toinoi-prod-trips-global \
  --target-db-cluster-identifier arn:aws:rds:ap-northeast-1:123456789012:cluster:toinoi-prod-trips-b
```

Switchover chuyển vai trò ghi sang `ap-northeast-1` khi cả hai vùng còn khoẻ; sự cố thật dùng `aws rds failover-global-cluster --allow-data-loss`, chấp nhận mất phần chưa nhân bản, và bước này do người quyết.

## Điều hướng traffic: Route 53, ARC và Global Accelerator

**Route 53 failover record** gồm bản ghi PRIMARY và SECONDARY; health check của PRIMARY lỗi thì Route 53 trả SECONDARY. `TTL` và cache của client quyết định bao lâu client mới thật sự đổi đích.

**Amazon Application Recovery Controller (ARC)**, trước đây mang tên Route 53 ARC, thêm các thứ sau:

- **Routing control**: công tắc Bật/Tắt do con người điều khiển, gắn vào health check kiểu `RECOVERY_CONTROL`. Công tắc nằm trên một **cluster** có endpoint ở nhiều region nên vẫn đổi được khi một region hỏng; **safety rule** chặn thao tác nguy hiểm như tắt cả hai region.
- **Region switch**: kế hoạch chuyển region chạy thành từng bước (failover Aurora Global, scale ECS ở region đích, đổi routing control), có plan evaluation định kỳ. **Readiness check** cũ đã đóng với khách hàng mới từ 2026; AWS khuyên dùng Region switch thay thế.
- **Zonal shift** và **zonal autoshift**: dời traffic của ALB/NLB (và một số tài nguyên được hỗ trợ) khỏi một AZ. Zonal shift tự hết hạn, tối đa vài ngày và gia hạn được; autoshift để AWS tự dời khi phát hiện AZ suy giảm.

```bash
aws arc-zonal-shift start-zonal-shift \
  --resource-identifier arn:aws:elasticloadbalancing:ap-southeast-1:123456789012:loadbalancer/app/toinoi-prod-booking/0123456789abcdef \
  --away-from apse1-az3 --expires-in 30m --comment "AZ c nghẽn mạng, drill"
```

Lệnh dời traffic của ALB khỏi `apse1-az3` trong 30 phút; dùng AZ ID vì tên như `ap-southeast-1c` ánh xạ khác nhau giữa các tài khoản.

**Global Accelerator** cấp hai IP anycast tĩnh, đưa client vào mạng AWS ở edge gần nhất rồi chuyển tới endpoint group ở region khoẻ. Failover không phụ thuộc DNS cache, hợp app di động giữ kết nối lâu; `traffic dial` và endpoint weight để dời dần.

| Tiêu chí | Route 53 failover | Route 53 + ARC | Global Accelerator |
| --- | --- | --- | --- |
| Cách đổi đích | Health check tự động | Công tắc do người điều khiển | Health check hoặc dial/weight |
| Phụ thuộc DNS cache | Có, theo TTL | Có, theo TTL | Không |
| Dùng control plane lúc sự cố | Không | Không (data plane cluster) | Chỉnh dial dùng control plane |
| Hợp với | Web, API đơn giản | Quyết định có chủ ý, tránh flapping | Mobile, TCP/UDP, cần IP tĩnh |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Route 53 trả về region chính hoặc phụ theo health check; health check gắn vào routing control của ARC; người vận hành đổi công tắc qua endpoint của cluster; Aurora Global Database nhân bản từ region chính sang region phụ."><defs><marker id="aws56-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws56-2-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="110" width="170" height="70" rx="10" class="f-surface s-line"/><text x="105" y="140" text-anchor="middle" class="t-b">Khách</text><text x="105" y="160" text-anchor="middle" class="t-small t-mono">api.toinoi.example</text><rect x="260" y="110" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="355" y="138" text-anchor="middle" class="t-b">Route 53 failover</text><text x="355" y="158" text-anchor="middle" class="t-small">PRIMARY / SECONDARY</text><rect x="260" y="230" width="190" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="355" y="254" text-anchor="middle" class="t-b t-warn">ARC routing control</text><text x="355" y="272" text-anchor="middle" class="t-small">Bật / Tắt từng region</text><rect x="560" y="30" width="300" height="100" rx="10" class="f-phys-soft s-phys"/><text x="576" y="56" class="t-b">ap-southeast-1 (chính)</text><text x="576" y="78" class="t-small">ALB, booking, Aurora ghi</text><text x="576" y="98" class="t-small">công suất 100%</text><rect x="560" y="160" width="300" height="100" rx="10" class="f-phys-soft s-phys"/><text x="576" y="186" class="t-b">ap-northeast-1 (phụ)</text><text x="576" y="208" class="t-small">ALB, booking, Aurora chỉ đọc</text><text x="576" y="228" class="t-small">warm standby, quota đã nâng</text><path d="M190 145 H258" class="f-none s-phys" stroke-width="2" marker-end="url(#aws56-2-phys)"/><path d="M450 130 L558 90" class="f-none s-pk" stroke-width="2" marker-end="url(#aws56-2-pk)"/><path d="M450 160 L558 200" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws56-2-pk)"/><path d="M355 230 V182" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws56-2-pk)"/><path d="M710 130 V158" class="f-none s-phys" stroke-width="2" marker-end="url(#aws56-2-phys)"/><text x="722" y="150" class="t-small">Global Database</text></svg>
</div>
<figcaption>Hình 2. Route 53 chọn đích theo health check; health check đọc routing control của ARC nên con người đổi được đích mà không cần sửa bản ghi DNS lúc sự cố.</figcaption>
</figure>

## Fault Injection Service và game day

Kế hoạch failover chưa diễn tập chỉ là giả thuyết. **AWS Fault Injection Service (FIS)** chạy lỗi có kiểm soát theo **experiment template** gồm:

- **Action**: hành động gây lỗi như dừng instance (`aws:ec2:stop-instances`), dừng task ECS (`aws:ecs:stop-task`), xoá pod EKS (`aws:eks:pod-delete`), cắt kết nối subnet (`aws:network:disrupt-connectivity`), thêm độ trễ mạng. Thư viện **scenario** có sẵn kịch bản mất điện một AZ.
- **Target**: tài nguyên bị tác động, chọn theo tag, bộ lọc AZ và tỉ lệ (`COUNT(1)`, `PERCENT(30)`).
- **Stop condition**: một CloudWatch alarm; alarm vào trạng thái ALARM thì FIS dừng thí nghiệm. Không có stop condition thì đừng chạy nơi có người dùng thật.
- **Role** riêng, chỉ có quyền cho đúng các action đó.

FIS tính tiền theo phút action chạy. Game day mỗi quý: đặt giả thuyết ("mất 30% task `booking`, p99 đặt chuyến vẫn dưới 800 ms"), chạy ở staging trước, một người chỉ huy và một người ghi giờ, alarm SLO của bài 55 làm stop condition, rồi biến kết quả thành việc cần sửa. Game day DR kèm bước restore của bài 37 và đo thời gian từ lúc bấm công tắc ARC tới request đầu tiên thành công ở region phụ.

## Cái giá phải trả

- **Multi-region gần như nhân đôi hạ tầng** và thêm lỗi mới (replication lag, split-brain).
- **Failover chưa phải là xong**: cache nguội, connection pool trỏ DB cũ, secret chưa nhân bản, quota thiếu đều kéo dài RTO.
- **Health check quá nhạy gây flapping**: failover rồi failback liên tục; phần lớn trường hợp nên để người quyết qua routing control.
- **FIS là tải thật**: target viết sai có thể dừng nhầm production. Luôn có stop condition, tag riêng cho tài nguyên được thử.

**Bài tập:** DNS failover mất 60 giây nhưng người dùng vẫn thấy lỗi 10 phút. Hãy kể ít nhất bốn nguyên nhân có thể.

<details><summary>Xem đáp án</summary><p>(1) TTL và cache của resolver hoặc client (kể cả connection pool giữ kết nối cũ). (2) Region phụ thiếu công suất: `desiredCount` thấp, quota Fargate thiếu. (3) Database chưa được promote nên ghi đặt chuyến vẫn lỗi. (4) Thiếu secret, khoá KMS hoặc image ở region phụ nên task không khởi động. (5) Cache nguội làm DB quá tải. Chữa bằng checklist region phụ, diễn tập đo giờ thật và bước promote DB trong runbook.</p></details>

## Lab: FIS và failover bằng Terraform

Thư mục `examples/toi-noi-aws/p06-56-resilience` có Terraform dựng ARC cluster với hai routing control và safety rule, hai health check `RECOVERY_CONTROL`, cặp bản ghi failover cho `api.toinoi.example`, alarm 5xx và experiment template FIS có stop condition, cùng script chạy thí nghiệm và đổi công tắc (chỉ gọi endpoint data plane đã lưu sẵn).

```bash
cd examples/toi-noi-aws/p06-56-resilience
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Terraform đã qua `fmt` và `validate` nhưng **chưa `apply` trên tài khoản thật**; target `aws:ecs:task` (tham số `cluster`, `service`), action và quyền FIS đã đối chiếu tài liệu, API cấu hình ARC chỉ có ở `us-west-2`. Tiền lab đến từ ARC cluster tính theo giờ, health check và phút action của FIS; README ghi cách dọn.

Bài trước: [bài 55](/articles/aws-observability-nang-cao-xray-adot-logs-insights-slo). Bài tiếp: [bài 57](/articles/aws-step-functions-saga-dat-chuyen).

## Tài liệu gốc

- [Reliability Pillar của AWS Well-Architected Framework](https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/welcome.html).
- [Amazon Application Recovery Controller (ARC)](https://docs.aws.amazon.com/r53recovery/latest/dg/what-is-route-53-recovery.html).
- [AWS Fault Injection Service User Guide](https://docs.aws.amazon.com/fis/latest/userguide/what-is.html).
- [Aurora Global Database](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/aurora-global-database.html).
- [AWS Global Accelerator Developer Guide](https://docs.aws.amazon.com/global-accelerator/latest/dg/what-is-global-accelerator.html).
- [Disaster recovery of workloads on AWS (whitepaper)](https://docs.aws.amazon.com/whitepapers/latest/disaster-recovery-workloads-on-aws/disaster-recovery-workloads-on-aws.html).

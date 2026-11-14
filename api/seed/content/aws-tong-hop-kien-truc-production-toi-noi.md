---
slug: aws-tong-hop-kien-truc-production-toi-noi
title: "Tổng hợp: kiến trúc production Tới Nơi trên AWS, luồng một chuyến đi qua từng dịch vụ và runbook"
summary: "Bài 60: ghép 59 bài thành ba bản vẽ theo lớp (tài khoản và mạng, compute và dữ liệu, phân phối và quan sát); đi theo một chuyến từ lúc khách bấm đặt tới lúc đối soát, chỉ ra chỗ idempotent và chỗ có thể mất; runbook 5 sự cố, SLO, bảo mật, bảng chi phí theo lớp (thấp/vừa/cao) và lộ trình học tiếp."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, architecture, production, reference-architecture, runbook, tong-hop]
language: vi
---

Lúc 17:42 một khách ở Quận 7 bấm "Đặt xe". Trong vài giây, yêu cầu đi qua CloudFront và WAF, service `booking`, một transaction Aurora, một sự kiện MSK, một saga Step Functions và một lượt tìm tài xế trên DynamoDB; mỗi dịch vụ là một bài trong lộ trình. Bài cuối không dạy dịch vụ mới mà ghép chúng lại: một chuyến đi đâu, hỏng ở đâu, và ai làm gì lúc 02:00 sáng.

Đây là **bài 60**, bài kết của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn nên đã đọc các bài trước, hoặc dùng số bài trong ngoặc làm mục lục để quay lại.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Booking ghi chuyến vào Aurora rồi gửi sự kiện sang MSK. Nếu service crash giữa hai việc đó, chuyến có bị "mất tăm" khỏi dispatch không?</p>
<details><summary>Xem đáp án</summary><p><b>Không, nếu dùng outbox.</b> Chuyến và dòng outbox ghi trong cùng transaction Aurora, connector đẩy outbox sang MSK sau. Crash chỉ làm sự kiện đến muộn hoặc hai lần, nên consumer phải idempotent.</p></details>
</div>

## Bản vẽ tổng, đọc theo ba lớp

Kiến trúc đích của Tới Nơi hiện thực trên AWS thành ba lớp. Lớp nền là tài khoản và mạng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 1. Lớp tài khoản và mạng: Organizations chia tài khoản quản lý, bảo mật, mạng dùng chung, dev, staging, prod; tài khoản prod có edge (Route 53, CloudFront, WAF, API Gateway), một VPC ba AZ chia subnet public, private, data, endpoint riêng và region phụ warm standby."><defs><marker id="aws60-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws60-1-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws60-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><text x="30" y="24" class="t-b">AWS Organizations và SCP (bài 28): mỗi môi trường một tài khoản</text><rect x="30" y="36" width="130" height="54" rx="10" class="f-phys-soft s-phys"/><text x="95.0" y="60.0" text-anchor="middle" class="t-b">Management</text><text x="95.0" y="77.0" text-anchor="middle" class="t-small">chỉ thanh toán, SCP</text><rect x="168" y="36" width="130" height="54" rx="10" class="f-phys-soft s-phys"/><text x="233.0" y="60.0" text-anchor="middle" class="t-b">Security</text><text x="233.0" y="77.0" text-anchor="middle" class="t-small">log archive, audit (54)</text><rect x="306" y="36" width="130" height="54" rx="10" class="f-phys-soft s-phys"/><text x="371.0" y="60.0" text-anchor="middle" class="t-b">Network</text><text x="371.0" y="77.0" text-anchor="middle" class="t-small">TGW, endpoint (53)</text><rect x="444" y="36" width="130" height="54" rx="10" class="f-phys-soft s-phys"/><text x="509.0" y="60.0" text-anchor="middle" class="t-b">Dev</text><text x="509.0" y="77.0" text-anchor="middle" class="t-small">lab tự do có giới hạn</text><rect x="582" y="36" width="130" height="54" rx="10" class="f-phys-soft s-phys"/><text x="647.0" y="60.0" text-anchor="middle" class="t-b">Staging</text><text x="647.0" y="77.0" text-anchor="middle" class="t-small">bản sao gần prod</text><rect x="720" y="36" width="130" height="54" rx="10" class="f-pk-soft s-pk"/><text x="785.0" y="60.0" text-anchor="middle" class="t-b">Prod</text><text x="785.0" y="77.0" text-anchor="middle" class="t-small">dữ liệu khách thật</text><rect x="20" y="110" width="840" height="318" rx="12" class="f-bg s-line"/><text x="34" y="132" class="t-b">Tài khoản prod · ap-southeast-1</text><rect x="36" y="142" width="170" height="54" rx="10" class="f-phys-soft s-phys"/><text x="121.0" y="166.0" text-anchor="middle" class="t-b">Route 53</text><text x="121.0" y="183.0" text-anchor="middle" class="t-small">DNS, health check (09)</text><rect x="224" y="142" width="230" height="54" rx="10" class="f-phys-soft s-phys"/><text x="339.0" y="166.0" text-anchor="middle" class="t-b">CloudFront + WAF + Shield</text><text x="339.0" y="183.0" text-anchor="middle" class="t-small">biên, chặn bot (20)</text><rect x="472" y="142" width="180" height="54" rx="10" class="f-phys-soft s-phys"/><text x="562.0" y="166.0" text-anchor="middle" class="t-b">API Gateway · ALB</text><text x="562.0" y="183.0" text-anchor="middle" class="t-small">HTTP, WebSocket (19, 13)</text><path d="M206 169 L222 169" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-1-pk)"/><path d="M454 169 L470 169" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-1-pk)"/><rect x="36" y="210" width="600" height="206" rx="12" class="f-surface s-line"/><text x="50" y="232" class="t-b">VPC 10.20.0.0/16, ba AZ (04, 23)</text><rect x="50" y="256" width="572" height="34" rx="10" class="f-phys-soft s-phys"/><text x="336.0" y="277.0" text-anchor="middle" class="t-b">Subnet public: ALB, NAT mỗi AZ (04, 56)</text><rect x="50" y="298" width="572" height="34" rx="10" class="f-phys-soft s-phys"/><text x="336.0" y="319.0" text-anchor="middle" class="t-b">Subnet private: ECS Fargate (12, 13), EKS tuỳ chọn (41), Lambda (18)</text><rect x="50" y="340" width="572" height="34" rx="10" class="f-pk-soft s-pk"/><text x="336.0" y="361.0" text-anchor="middle" class="t-b">Subnet data: Aurora, RDS Proxy, ElastiCache, MSK (31, 33, 34)</text><text x="50" y="396" class="t-small">Mỗi subnet có một bản ở mỗi AZ; security group theo service, DB không ra Internet (04)</text><rect x="656" y="210" width="194" height="92" rx="10" class="f-surface s-line"/><text x="753.0" y="253.0" text-anchor="middle" class="t-b">VPC endpoint, PrivateLink</text><text x="753.0" y="270.0" text-anchor="middle" class="t-small">gateway endpoint S3, DynamoDB (53)</text><rect x="656" y="314" width="194" height="96" rx="10" class="f-warn-soft s-warn dash"/><text x="753.0" y="359.0" text-anchor="middle" class="t-b">Region phụ ap-northeast-1</text><text x="753.0" y="376.0" text-anchor="middle" class="t-small">Aurora Global, standby (37, 56)</text><path d="M562 196 L562 254" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-1-pk)"/></svg>
</div>
<figcaption>Hình 1. Lớp tài khoản và mạng; số trong ngoặc là bài tham chiếu, mạng hybrid và Network Firewall ở bài 53.</figcaption>
</figure>

Mỗi môi trường một tài khoản, SCP đặt trần quyền (bài 28); prod có VPC ba AZ, ba tầng subnet và endpoint để traffic S3, DynamoDB không qua NAT; region phụ chạy warm standby. Lớp thứ hai là code và dữ liệu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 2. Lớp compute và dữ liệu: API vào các service booking, quote, dispatch, tracking, payment trên ECS Fargate (EKS là phương án tuỳ chọn); saga Step Functions; dữ liệu ở Aurora, DynamoDB, ElastiCache, S3, OpenSearch; luồng bất đồng bộ qua SQS, EventBridge, MSK, Kinesis và lakehouse."><defs><marker id="aws60-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws60-2-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws60-2-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="60" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="105.0" y="87.0" text-anchor="middle" class="t-b">API Gateway · ALB</text><text x="105.0" y="104.0" text-anchor="middle" class="t-small">vào hệ thống (19, 13)</text><rect x="20" y="150" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="105.0" y="177.0" text-anchor="middle" class="t-b">Webhook VíXanh</text><text x="105.0" y="194.0" text-anchor="middle" class="t-small">Lambda, SQS (18, 16)</text><rect x="230" y="40" width="320" height="250" rx="12" class="f-bg s-line"/><text x="244" y="62" class="t-b">ECS Fargate (12, 13) · EKS tuỳ chọn (41)</text><rect x="244" y="66" width="142" height="42" rx="10" class="f-phys-soft s-phys"/><text x="315.0" y="84.0" text-anchor="middle" class="t-b">booking</text><text x="315.0" y="101.0" text-anchor="middle" class="t-small">ghi trips, outbox</text><rect x="396" y="66" width="142" height="42" rx="10" class="f-phys-soft s-phys"/><text x="467.0" y="84.0" text-anchor="middle" class="t-b">quote</text><text x="467.0" y="101.0" text-anchor="middle" class="t-small">báo giá, cache</text><rect x="244" y="114" width="142" height="42" rx="10" class="f-phys-soft s-phys"/><text x="315.0" y="132.0" text-anchor="middle" class="t-b">dispatch</text><text x="315.0" y="149.0" text-anchor="middle" class="t-small">tìm và ghép tài xế</text><rect x="396" y="114" width="142" height="42" rx="10" class="f-phys-soft s-phys"/><text x="467.0" y="132.0" text-anchor="middle" class="t-b">tracking</text><text x="467.0" y="149.0" text-anchor="middle" class="t-small">vị trí, WebSocket</text><rect x="244" y="162" width="142" height="42" rx="10" class="f-phys-soft s-phys"/><text x="315.0" y="180.0" text-anchor="middle" class="t-b">payment</text><text x="315.0" y="197.0" text-anchor="middle" class="t-small">ví, đối soát</text><rect x="244" y="214" width="294" height="46" rx="10" class="f-pk-soft s-pk"/><text x="391.0" y="234.0" text-anchor="middle" class="t-b">Step Functions: saga đặt chuyến (57)</text><text x="391.0" y="251.0" text-anchor="middle" class="t-small">giữ tiền, callback tài xế, bù trừ</text><text x="244" y="280" class="t-small">Sizing và Spot: Savings Plans phủ nền (59)</text><rect x="590" y="40" width="270" height="250" rx="12" class="f-bg s-line"/><text x="604" y="62" class="t-b">Dữ liệu</text><rect x="604" y="68" width="242" height="38" rx="10" class="f-pk-soft s-pk"/><text x="725.0" y="91.0" text-anchor="middle" class="t-b">Aurora + RDS Proxy: trips, outbox (31)</text><rect x="604" y="112" width="242" height="38" rx="10" class="f-pk-soft s-pk"/><text x="725.0" y="135.0" text-anchor="middle" class="t-b">DynamoDB: vị trí, ai nhận chuyến (32)</text><rect x="604" y="156" width="242" height="38" rx="10" class="f-pk-soft s-pk"/><text x="725.0" y="179.0" text-anchor="middle" class="t-b">ElastiCache / MemoryDB: cache (15, 33)</text><rect x="604" y="200" width="242" height="38" rx="10" class="f-pk-soft s-pk"/><text x="725.0" y="223.0" text-anchor="middle" class="t-b">S3: ảnh chứng từ, lake (06, 36)</text><rect x="604" y="244" width="242" height="34" rx="10" class="f-pk-soft s-pk"/><text x="725.0" y="265.0" text-anchor="middle" class="t-b">OpenSearch: tìm quán (39)</text><path d="M190 90 L228 100" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-2-pk)"/><path d="M190 180 L228 160" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-2-pk)"/><path d="M550 160 L588 160" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-2-pk)"/><rect x="20" y="312" width="840" height="112" rx="12" class="f-surface s-line"/><text x="34" y="334" class="t-b">Bất đồng bộ và phân tích</text><rect x="34" y="342" width="150" height="66" rx="10" class="f-phys-soft s-phys"/><text x="109.0" y="372.0" text-anchor="middle" class="t-b">SQS · SNS</text><text x="109.0" y="389.0" text-anchor="middle" class="t-small">việc nền, fan-out (16)</text><rect x="198" y="342" width="160" height="66" rx="10" class="f-phys-soft s-phys"/><text x="278.0" y="372.0" text-anchor="middle" class="t-b">EventBridge</text><text x="278.0" y="389.0" text-anchor="middle" class="t-small">TripConfirmed, lịch (17)</text><rect x="372" y="342" width="150" height="66" rx="10" class="f-phys-soft s-phys"/><text x="447.0" y="372.0" text-anchor="middle" class="t-b">MSK: sự kiện chuyến</text><text x="447.0" y="389.0" text-anchor="middle" class="t-small">outbox, replay (34)</text><rect x="536" y="342" width="150" height="66" rx="10" class="f-phys-soft s-phys"/><text x="611.0" y="372.0" text-anchor="middle" class="t-b">Kinesis → Firehose</text><text x="611.0" y="389.0" text-anchor="middle" class="t-small">6.250 toạ độ/giây (35)</text><rect x="700" y="342" width="146" height="66" rx="10" class="f-phys-soft s-phys"/><text x="773.0" y="372.0" text-anchor="middle" class="t-b">Glue · Athena</text><text x="773.0" y="389.0" text-anchor="middle" class="t-small">lakehouse (38)</text><path d="M380 290 L447 340" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-2-pk)"/><path d="M300 290 L277 340" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-2-pk)"/><path d="M686 375 L698 375" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-2-pk)"/></svg>
</div>
<figcaption>Hình 2. Lớp compute và dữ liệu: sự kiện nghiệp vụ qua MSK, toạ độ GPS qua Kinesis (lý do ở bảng quyết định).</figcaption>
</figure>

Năm service chạy trên ECS Fargate, Lambda lo webhook thanh toán, Step Functions lo saga. Lớp thứ ba chạy xuyên mọi tài khoản: phân phối, quan sát và bảo vệ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 3. Lớp phân phối, quan sát và bảo mật: pipeline GitHub Actions OIDC, ECR, CodeDeploy hoặc Argo CD, Terraform và policy as code; CloudWatch, ADOT, X-Ray, SLO; IAM, KMS, Secrets Manager, GuardDuty, Backup, FIS, Budgets và Well-Architected review."><defs><marker id="aws60-3-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws60-3-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws60-3-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="16" width="840" height="122" rx="12" class="f-bg s-line"/><text x="34" y="38" class="t-b">Phân phối: từ commit tới production; hạ tầng là Terraform và policy as code (22, 23, 30)</text><rect x="34" y="48" width="150" height="76" rx="10" class="f-phys-soft s-phys"/><text x="109.0" y="83.0" text-anchor="middle" class="t-b">GitHub Actions</text><text x="109.0" y="100.0" text-anchor="middle" class="t-small">OIDC, không key (25)</text><rect x="204" y="48" width="130" height="76" rx="10" class="f-phys-soft s-phys"/><text x="269.0" y="83.0" text-anchor="middle" class="t-b">ECR</text><text x="269.0" y="100.0" text-anchor="middle" class="t-small">digest bất biến (11)</text><rect x="354" y="48" width="170" height="76" rx="10" class="f-phys-soft s-phys"/><text x="439.0" y="83.0" text-anchor="middle" class="t-b">CodeDeploy · Argo CD</text><text x="439.0" y="100.0" text-anchor="middle" class="t-small">blue/green, GitOps (26, 46)</text><rect x="544" y="48" width="150" height="76" rx="10" class="f-phys-soft s-phys"/><text x="619.0" y="83.0" text-anchor="middle" class="t-b">ECS · EKS</text><text x="619.0" y="100.0" text-anchor="middle" class="t-small">canary, rollback (27)</text><rect x="712" y="48" width="134" height="76" rx="10" class="f-pk-soft s-pk"/><text x="779.0" y="83.0" text-anchor="middle" class="t-b">AppConfig</text><text x="779.0" y="100.0" text-anchor="middle" class="t-small">flag, bảng giá (29)</text><path d="M184 86 L202 86" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-3-pk)"/><path d="M334 86 L352 86" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-3-pk)"/><path d="M524 86 L542 86" class="f-none s-pk" stroke-width="2" marker-end="url(#aws60-3-pk)"/><rect x="20" y="152" width="840" height="126" rx="12" class="f-bg s-line"/><text x="34" y="174" class="t-b">Quan sát: biết khi nào hỏng trước khách</text><rect x="34" y="184" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="129.0" y="221.0" text-anchor="middle" class="t-b">CloudWatch</text><text x="129.0" y="238.0" text-anchor="middle" class="t-small">metric, alarm, log, retention (10)</text><rect x="240" y="184" width="200" height="80" rx="10" class="f-phys-soft s-phys"/><text x="340.0" y="221.0" text-anchor="middle" class="t-b">ADOT · X-Ray</text><text x="340.0" y="238.0" text-anchor="middle" class="t-small">trace, metric chuẩn OTLP (47, 55)</text><rect x="456" y="184" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="551.0" y="221.0" text-anchor="middle" class="t-b">SLO · Application Signals</text><text x="551.0" y="238.0" text-anchor="middle" class="t-small">error budget, burn rate (55)</text><rect x="662" y="184" width="184" height="80" rx="10" class="f-phys-soft s-phys"/><text x="754.0" y="221.0" text-anchor="middle" class="t-b">Prometheus · Grafana</text><text x="754.0" y="238.0" text-anchor="middle" class="t-small">EKS, dashboard chung (47)</text><rect x="20" y="292" width="840" height="136" rx="12" class="f-bg s-line"/><text x="34" y="314" class="t-b">Bảo mật, khôi phục và kỷ luật</text><rect x="34" y="326" width="152" height="84" rx="10" class="f-phys-soft s-phys"/><text x="110.0" y="365.0" text-anchor="middle" class="t-b">IAM · SCP</text><text x="110.0" y="382.0" text-anchor="middle" class="t-small">quyền tối thiểu (03, 28, 51)</text><rect x="198" y="326" width="152" height="84" rx="10" class="f-phys-soft s-phys"/><text x="274.0" y="365.0" text-anchor="middle" class="t-b">KMS · Secrets</text><text x="274.0" y="382.0" text-anchor="middle" class="t-small">mã hoá, secret (14, 52)</text><rect x="362" y="326" width="152" height="84" rx="10" class="f-phys-soft s-phys"/><text x="438.0" y="365.0" text-anchor="middle" class="t-b">GuardDuty · Config</text><text x="438.0" y="382.0" text-anchor="middle" class="t-small">phát hiện, audit (54)</text><rect x="526" y="326" width="152" height="84" rx="10" class="f-warn-soft s-warn"/><text x="602.0" y="365.0" text-anchor="middle" class="t-b">Backup · FIS</text><text x="602.0" y="382.0" text-anchor="middle" class="t-small">restore, game day (37, 56)</text><rect x="690" y="326" width="152" height="84" rx="10" class="f-phys-soft s-phys"/><text x="766.0" y="365.0" text-anchor="middle" class="t-b">Budgets · WA</text><text x="766.0" y="382.0" text-anchor="middle" class="t-small">chi phí, review (58, 59)</text></svg>
</div>
<figcaption>Hình 3. Lớp phân phối, quan sát và bảo mật; tín hiệu quan sát quyết định canary đi tiếp hay dừng.</figcaption>
</figure>

Ba quyết định lớn nhất, kèm điều kiện để đổi quyết định:

| Quyết định | Chọn | Lý do | Đổi khi |
| --- | --- | --- | --- |
| Chạy container | ECS Fargate (bài 12, 13) | Ít khái niệm, AWS lo nâng cấp, đủ cho đội nhỏ | Nhiều service, cần operator, KEDA, canary theo metric (bài 41, 45, 49) |
| Sự kiện nghiệp vụ | MSK với outbox (bài 34) | Replay, nhiều consumer group, thứ tự theo khoá | Chỉ cần "giao cho một worker": SQS (bài 16) |
| Luồng toạ độ GPS | Kinesis, rồi Firehose ra S3 (bài 35) | Tải đều, không gánh cluster | Đã có MSK và muốn một nền tảng: topic riêng |

## Một chuyến đi qua từng dịch vụ

Bảng dưới đi theo thứ tự thời gian, ghi chỗ nào bảo đảm không xử lý hai lần (idempotent) và chỗ nào chấp nhận mất.

| Bước | Dịch vụ | Idempotent nhờ | Có thể mất? |
| --- | --- | --- | --- |
| 1. Báo giá | CloudFront, WAF, API Gateway hoặc ALB, `quote`, ElastiCache | GET không đổi trạng thái | Cache miss thì tính lại; báo giá có hạn |
| 2. Đặt chuyến | `booking` ghi `trips` và outbox vào Aurora | Idempotency-Key duy nhất theo khách | Không: một transaction |
| 3. Phát sự kiện | MSK Connect (Debezium) đọc outbox sang MSK | Inbox ở consumer, khoá `eventId` | Không, nhưng có thể trùng |
| 4. Saga bắt đầu, giữ tiền | Consumer của sự kiện gọi Step Functions `StartExecution` (bài 57) | Tên execution là `bookingId`, khoá `bookingId#hold` | Không: Standard có lịch sử và redrive |
| 5. Tìm tài xế | Saga gửi yêu cầu kèm task token qua SQS; `dispatch` tìm trong DynamoDB, đẩy offer qua WebSocket | Offer có TTL và `bookingId` | Offer có thể mất, re-dispatch sau timeout |
| 6. Nhận chuyến | DynamoDB `TransactWriteItems`, rồi `SendTaskSuccess` | Điều kiện trạng thái, khoá `IDEMP#` | Không: một người thắng |
| 7. Xác nhận, thu tiền | Step Functions, bù trừ khi lỗi | Khoá `bookingId#bước` | Không: có hàng đợi xử lý thủ công |
| 8. Webhook VíXanh | API Gateway, Lambda xác minh chữ ký, SQS | Khử trùng lặp theo mã giao dịch | Webhook có thể đến trễ; đối soát bù |
| 9. Sự kiện hoàn tất | EventBridge, rule tới thông báo và thống kê | `eventId`, DLQ của rule | Có archive để replay |
| 10. Toạ độ GPS | Tracking, Kinesis, Firehose, S3 | Theo `driverId` và thời điểm | Vị trí hiện tại tự dựng lại; lịch sử có thể lấy mẫu |
| 11. Đối soát đêm | Step Functions distributed map đọc S3, Athena | Theo `tripId` và ngày | Không: lô lỗi ghi lại để xem |

Toàn chuỗi là **at-least-once**: "đúng một lần" là kết quả của idempotency ở từng nơi, không phải của hạ tầng (xem bài về [Outbox và Inbox](/articles/dotnet10-outbox-inbox-idempotent-consumer) và [transaction cùng idempotency](/articles/dotnet10-transaction-concurrency-idempotency)). Chỉ hai thứ được phép mất có chủ đích: vị trí hiện tại và offer chưa nhận, vì cả hai tự dựng lại sau vài giây.

## Vận hành: SLO, alarm và năm sự cố

SLO ban đầu là mục tiêu khởi tạo, chưa phải cam kết, chỉ đổi sau khi có dữ liệu thật (bài 55):

| Năng lực | Mục tiêu khởi tạo | Alarm gắn với |
| --- | --- | --- |
| API đặt chuyến khả dụng | 99,9% mỗi tháng | Burn rate nhanh và chậm của error budget |
| Độ trễ đặt chuyến | p99 của `POST /v1/trips` dưới 800 ms, không tính chờ tài xế | SLO latency, alarm p99 |
| Độ tươi vị trí tài xế | p95 dưới 5 đến 10 giây | Tuổi bản ghi mới nhất, Kinesis iterator age |
| Thanh toán hội tụ | Theo cam kết của cổng, có đối soát | Số saga ở hàng đợi xử lý thủ công |

Runbook ngắn cho năm sự cố hay gặp, mỗi mục gắn với bài để đọc sâu:

| Sự cố | Kiểm đầu tiên | Hành động | Bài |
| --- | --- | --- | --- |
| Aurora failover | Writer endpoint, kết nối của RDS Proxy | Chờ failover, xác nhận ứng dụng nối lại; không restart hàng loạt | 31 |
| Spot thu hồi hàng loạt | Số task theo capacity provider (`FARGATE_SPOT`) | Tăng tạm phần `FARGATE` on-demand; trên EKS đa dạng instance type | 59, 45 |
| MSK consumer lag tăng | Lag theo consumer group, partition lệch | Scale consumer tới số partition; kiểm message độc | 34 |
| WAF chặn nhầm | Sampled requests, rule gây chặn | Chuyển rule sang Count, đưa ngoại lệ, rồi mới siết lại | 20 |
| Deploy lỗi | Alarm canary, error rate theo phiên bản | Rollback theo digest; kiểm migration tương thích hai bản | 26, 27 |

Cả năm theo cùng thứ tự: xác nhận ảnh hưởng, giảm thiệt hại trước (rollback, chuyển Count, dời traffic), tìm nguyên nhân sau. Mỗi quý chạy game day bằng FIS và diễn tập restore (bài 37, 56).

Bảo mật theo lớp: biên có WAF và Shield (bài 20); mạng chia subnet, không mở dữ liệu ra Internet (bài 04, 53); danh tính tạm thời, quyền tối thiểu có boundary (bài 03, 25, 51); mã hoá bằng KMS, secret trong Secrets Manager (bài 14, 52); phát hiện bằng CloudTrail, GuardDuty, Security Hub (bài 54); image có quét và digest bất biến (bài 11).

## Chi phí: nhìn theo lớp, không theo con số

Bảng này **không ghi tiền** vì giá đổi theo thời gian và region. Mức thấp, vừa, cao chỉ để so sánh tương đối giữa các lớp; con số thật lấy từ AWS Pricing Calculator và hoá đơn (bài 59).

| Lớp | Dev nhỏ | Prod | Thứ làm chi phí tăng chính |
| --- | --- | --- | --- |
| Biên (CloudFront, WAF, Shield) | Thấp | Vừa | Lượng request, quy tắc WAF, băng thông ra |
| Compute (ECS Fargate, Lambda) | Thấp | Cao | Số giờ vCPU và GB, phần cam kết và Spot |
| Cơ sở dữ liệu (Aurora, RDS Proxy) | Thấp | Cao | Instance, I/O, số reader, backup |
| NoSQL và cache | Thấp | Vừa | Dung lượng đọc ghi, số node |
| Streaming (MSK, Kinesis) | Thấp | Vừa đến cao | Broker-giờ hoặc shard-giờ, retention |
| Lưu trữ và phân tích (S3, Athena) | Thấp | Vừa | Dung lượng, byte quét |
| Quan sát (log, metric, trace) | Thấp | Vừa đến cao | Lượng log nhận vào, cardinality metric |
| Mạng (NAT, endpoint, truyền giữa AZ) | Thấp | Vừa | GB qua NAT, số interface endpoint |

Mỗi dòng theo công thức `chi phí ≈ số đơn vị × số giờ × đơn giá + dữ liệu truyền`; dev nhỏ khác prod chủ yếu ở số đơn vị (một AZ, instance nhỏ, tắt ngoài giờ), không ở kiến trúc.

## Lộ trình tiếp

Chứng chỉ chỉ là mốc tham chiếu: Phần 01 ứng CLF-C02 và nửa đầu SAA-C03; Phần 02 DVA-C02, SAA-C03; Phần 03 DOP-C02, DVA-C02; Phần 04 SAA-C03, SAP-C02, DEA-C01; Phần 05 DOP-C02, CKA; Phần 06 SCS-C02, SAP-C02. Chủ đề chưa phủ: machine learning và Bedrock, IoT cho thiết bị trên xe, media cho video chứng từ, kết nối hybrid sâu với Direct Connect.

Mã nằm ở `examples/toi-noi-aws/`; mỗi lab có README ghi chi phí và cách dọn, **không có lệnh nào tự tạo tài nguyên**. Cách học tốt nhất: dựng một lab trong tài khoản riêng, phá có chủ đích như bài 56, đo, rồi dọn.

## Cái giá phải trả

- **Nhiều dịch vụ là nhiều thứ phải vận hành**: mỗi khối trong ba hình cần owner, alarm và runbook.
- **Bản vẽ này là điểm xuất phát, không phải đích bắt buộc**: đội nhỏ có thể bỏ EKS, MSK hay region phụ cho tới khi số liệu đòi hỏi.
- **Mọi con số ở đây là giả định**: SLO, mức chi phí và quy mô phải đo lại bằng tải và hoá đơn thật.

**Bài tập:** vẽ lại luồng **huỷ chuyến** với compensation. Khách huỷ sau khi tài xế đã nhận nhưng trước khi đón. Ghi rõ từng bước, bù trừ của nó và khoá idempotency.

<details><summary>Xem đáp án</summary><p>Một saga huỷ riêng: (1) <code>booking</code> ghi <code>CANCEL_REQUESTED</code> cùng outbox trong một transaction Aurora, khoá <code>bookingId#cancel</code>. (2) Dispatch giải phóng tài xế bằng cập nhật DynamoDB có điều kiện, khoá <code>bookingId#release-driver</code>. (3) Payment hoàn giữ tiền hoặc thu phí huỷ theo chính sách, khoá <code>bookingId#release-or-fee</code>. (4) Booking chốt <code>CANCELLED</code> và phát <code>TripCancelled</code> qua EventBridge. Bước 3 lỗi sau khi retry thì đẩy vào hàng đợi xử lý thủ công, không chốt <code>CANCELLED</code> khi tiền chưa nhả. Chuyến đã chạy thì sang luồng hoàn tiền.</p></details>

Bài trước: [bài 59](/articles/aws-finops-savings-plans-spot-right-sizing). Đây là bài cuối; muốn ôn lại thì bắt đầu từ [bài 01](/articles/aws-la-gi-region-az-shared-responsibility) cùng [hồ sơ dự án Tới Nơi](/articles/du-an-mau-toi-noi).

## Tài liệu gốc

- [AWS Well-Architected Framework](https://docs.aws.amazon.com/wellarchitected/latest/framework/welcome.html).
- [AWS Architecture Center](https://aws.amazon.com/architecture/).
- [Disaster recovery of workloads on AWS (whitepaper)](https://docs.aws.amazon.com/whitepapers/latest/disaster-recovery-workloads-on-aws/disaster-recovery-workloads-on-aws.html).
- [AWS Prescriptive Guidance: patterns cho Saga và Outbox](https://docs.aws.amazon.com/prescriptive-guidance/latest/modernization-data-persistence/saga-pattern.html).
- [AWS Pricing và Pricing Calculator](https://aws.amazon.com/pricing/).

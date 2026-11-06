---
slug: aws-well-architected-review-toi-noi
title: "Well-Architected Framework: review sáu trụ cột cho Tới Nơi và biến finding thành backlog"
summary: "Bài 58: chạy một Well-Architected review trên kiến trúc Tới Nơi đã dựng qua 57 bài: sáu trụ cột, câu hỏi trọng tâm, high-risk issue, Well-Architected Tool với lens, milestone và improvement plan; ví dụ finding thật, cách ưu tiên và biến thành backlog có owner, ước lượng, chỉ số; review lại sau 90 ngày."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, well-architected, architecture-review, best-practices, operational-excellence, cost-optimization, sustainability]
language: vi
---

Tới Nơi sắp mở thêm hai thành phố, tải dự kiến tăng gấp đôi. Đội hạ tầng đã làm theo từng bài: VPC, ECS, Aurora, MSK, pipeline, SLO. Người ký ngân sách hỏi: "Đã đủ vững chưa, chỗ nào còn rủi ro, làm gì trước?". Không ai trả lời được bằng cảm giác. Well-Architected review đưa ra bộ câu hỏi chung, buộc đội nhìn lại bằng chứng và ra danh sách việc có thứ tự.

Đây là **bài 58** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn nên đã đọc các bài 51 đến 57, vì bài này dùng chúng làm bằng chứng. Bài không có lab: sản phẩm là một bảng đánh giá và một backlog.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một hệ thống có đủ mọi dịch vụ "đúng chuẩn" (Multi-AZ, WAF, backup). Có cần review kiến trúc nữa không?</p>
<details><summary>Xem đáp án</summary><p><b>Có.</b> Có dịch vụ khác với dùng đúng: NAT chỉ ở một AZ làm Multi-AZ mất ý nghĩa, backup chưa từng restore thử thì chưa phải backup, log không có retention âm thầm tăng hoá đơn. Review tìm khoảng cách giữa "đã bật tính năng" và "đạt mục tiêu".</p></details>
</div>

## Sáu trụ cột và điều chúng hỏi

**AWS Well-Architected Framework** là tập thực hành tốt chia thành sáu trụ cột. Mỗi trụ cột có nguyên tắc thiết kế và bộ câu hỏi; trả lời xong bạn biết mình đang ở đâu, không phải để được điểm.

| Trụ cột | Hỏi gì | Từ khoá của Tới Nơi |
| --- | --- | --- |
| Operational Excellence | Chạy, quan sát và cải tiến hệ thống ra sao | Pipeline, runbook, game day |
| Security | Bảo vệ dữ liệu, danh tính và phát hiện bất thường | IAM, KMS, GuardDuty |
| Reliability | Chịu và phục hồi từ lỗi | Multi-AZ, DR, retry |
| Performance Efficiency | Dùng đúng tài nguyên cho đúng tải | Chọn DB, cache, autoscaling |
| Cost Optimization | Tránh chi tiêu không tạo giá trị | Tag, Savings Plans, retention |
| Sustainability | Giảm tài nguyên tiêu thụ cho cùng một khối lượng việc | Graviton, scale về 0 ngoài giờ |

Các trụ cột kéo nhau: Reliability muốn dư thừa, Cost muốn bớt; Security thêm lớp kiểm, Performance muốn bớt độ trễ. Review không tối ưu riêng từng trụ cột mà làm rõ đánh đổi nào được chấp nhận và đã ghi lại chưa.

## Chấm điểm bằng bằng chứng, không bằng cảm giác

Bảng dưới là một lượt review của Tới Nơi vào cuối lộ trình. Câu hỏi diễn đạt lại theo ngữ cảnh của dự án, không chép nguyên văn bộ câu hỏi chính thức. **Trạng thái là minh hoạ** dựa trên những gì các bài đã dựng; trên hệ thống thật, mỗi ô phải dẫn tới một bằng chứng cụ thể (dashboard, runbook, kết quả diễn tập).

| Trụ cột | Câu hỏi | Bài liên quan | Trạng thái |
| --- | --- | --- | --- |
| Op. Excellence | Deploy có tự động, có rollback và cổng migration không? | [25](/articles/aws-github-actions-oidc-ecr-deploy), [26](/articles/aws-codepipeline-codebuild-codedeploy-blue-green), [27](/articles/aws-chien-luoc-deploy-rolling-canary-rollback) | Đạt |
| Op. Excellence | Có metric, log, trace và SLO để biết khi nào hỏng? | [10](/articles/aws-cloudwatch-budgets-tagging-co-ban), [55](/articles/aws-observability-nang-cao-xray-adot-logs-insights-slo) | Một phần: staging chưa có SLO |
| Op. Excellence | Có diễn tập sự cố định kỳ không? | [56](/articles/aws-resilience-multi-region-route53-failover-fis) | Chưa |
| Security | Quyền tối thiểu, danh tính tạm thời cho người và máy? | [03](/articles/aws-iam-user-role-policy), [25](/articles/aws-github-actions-oidc-ecr-deploy), [51](/articles/aws-iam-nang-cao-permission-boundary-abac-access-analyzer) | Đạt |
| Security | Dữ liệu được mã hoá, khoá do ai quản lý? | [52](/articles/aws-kms-envelope-encryption-key-policy), [36](/articles/aws-s3-nang-cao-storage-class-replication-object-lock) | Đạt |
| Security | Phát hiện và điều tra bất thường, kể cả PII trong S3? | [54](/articles/aws-cloudtrail-config-security-hub-guardduty) | Một phần: chưa có Macie |
| Reliability | Sống qua mất một AZ? | [04](/articles/aws-vpc-subnet-route-security-group), [13](/articles/aws-ecs-alb-rolling-deploy-autoscaling), [31](/articles/aws-aurora-postgresql-rds-proxy-failover) | Một phần: NAT một AZ |
| Reliability | Backup đã restore thử, DR có RPO/RTO đo được? | [37](/articles/aws-backup-disaster-recovery-rpo-rto) | Đạt |
| Reliability | Giới hạn blast radius, quota và retry có jitter? | [56](/articles/aws-resilience-multi-region-route53-failover-fis), [57](/articles/aws-step-functions-saga-dat-chuyen) | Một phần |
| Performance | Mỗi access pattern có kho dữ liệu phù hợp? | [15](/articles/aws-elasticache-redis-cache-rate-limit), [31](/articles/aws-aurora-postgresql-rds-proxy-failover), [32](/articles/aws-dynamodb-single-table-gsi-ttl-streams) | Đạt |
| Performance | Sizing dựa trên load test, không đoán? | [13](/articles/aws-ecs-alb-rolling-deploy-autoscaling), [55](/articles/aws-observability-nang-cao-xray-adot-logs-insights-slo) | Một phần |
| Cost | Thấy chi phí theo service và owner? | [10](/articles/aws-cloudwatch-budgets-tagging-co-ban), [59](/articles/aws-finops-savings-plans-spot-right-sizing) | Một phần: tag chưa đủ |
| Cost | Log và snapshot có vòng đời? | [10](/articles/aws-cloudwatch-budgets-tagging-co-ban), [37](/articles/aws-backup-disaster-recovery-rpo-rto) | Chưa: log không retention |
| Cost | Cam kết và Spot đặt đúng chỗ? | [59](/articles/aws-finops-savings-plans-spot-right-sizing) | Chưa |
| Sustainability | Dùng tài nguyên vừa đủ, Graviton, tắt ngoài giờ? | [12](/articles/aws-ecs-fargate-task-definition-roles), [45](/articles/aws-eks-hpa-karpenter-spot), [59](/articles/aws-finops-savings-plans-spot-right-sizing) | Chưa: Graviton chưa dùng |
| Sustainability | Dữ liệu cũ được chuyển lớp hoặc xoá? | [36](/articles/aws-s3-nang-cao-storage-class-replication-object-lock), [38](/articles/aws-glue-athena-redshift-lakehouse) | Một phần |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Tình trạng minh hoạ của 16 câu hỏi theo sáu trụ cột: mỗi hàng là một trụ cột, ô xanh là đạt, ô tím là một phần, ô cam là chưa. Cost Optimization và Sustainability có nhiều ô cam nhất."><rect x="20" y="20" width="840" height="250" rx="12" class="f-surface s-line"/><text x="36" y="46" class="t-b">Tình trạng minh hoạ theo trụ cột</text><text x="36" y="84" class="t-small">Operational Excellence</text><rect x="260" y="68" width="70" height="26" rx="6" class="f-phys-soft s-phys"/><rect x="334" y="68" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><rect x="408" y="68" width="70" height="26" rx="6" class="f-warn-soft s-warn"/><text x="36" y="118" class="t-small">Security</text><rect x="260" y="102" width="70" height="26" rx="6" class="f-phys-soft s-phys"/><rect x="334" y="102" width="70" height="26" rx="6" class="f-phys-soft s-phys"/><rect x="408" y="102" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><text x="36" y="152" class="t-small">Reliability</text><rect x="260" y="136" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><rect x="334" y="136" width="70" height="26" rx="6" class="f-phys-soft s-phys"/><rect x="408" y="136" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><text x="36" y="186" class="t-small">Performance Efficiency</text><rect x="260" y="170" width="70" height="26" rx="6" class="f-phys-soft s-phys"/><rect x="334" y="170" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><text x="36" y="220" class="t-small">Cost Optimization</text><rect x="260" y="204" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><rect x="334" y="204" width="70" height="26" rx="6" class="f-warn-soft s-warn"/><rect x="408" y="204" width="70" height="26" rx="6" class="f-warn-soft s-warn"/><text x="36" y="254" class="t-small">Sustainability</text><rect x="260" y="238" width="70" height="26" rx="6" class="f-warn-soft s-warn"/><rect x="334" y="238" width="70" height="26" rx="6" class="f-pk-soft s-pk"/><rect x="540" y="68" width="300" height="130" rx="10" class="f-bg s-line"/><rect x="556" y="84" width="40" height="18" rx="4" class="f-phys-soft s-phys"/><text x="606" y="98" class="t-small">Đạt: có bằng chứng</text><rect x="556" y="114" width="40" height="18" rx="4" class="f-pk-soft s-pk"/><text x="606" y="128" class="t-small">Một phần: có khoảng trống đã biết</text><rect x="556" y="144" width="40" height="18" rx="4" class="f-warn-soft s-warn"/><text x="606" y="158" class="t-small">Chưa: rủi ro cần việc cụ thể</text><text x="556" y="184" class="t-small">Mỗi ô là một câu hỏi của bảng trên</text></svg>
</div>
<figcaption>Hình 1. Bức tranh một lượt review: cột bảo mật và vận hành khá chắc, còn chi phí và bền vững là nơi dồn nhiều việc nhất. Đây là số liệu minh hoạ, không phải đánh giá của một đội thật.</figcaption>
</figure>

## Well-Architected Tool, lens và milestone

**AWS Well-Architected Tool** (miễn phí, trong console) lưu một **workload**, dẫn bạn qua từng câu hỏi và cho chọn các thực hành đã làm. Từ lựa chọn đó nó gán mức rủi ro cho từng câu: **HRI** (high-risk issue), **MRI** (medium-risk issue) hoặc không rủi ro, rồi sinh **improvement plan**. Các khái niệm cần biết:

- **Lens**: bộ câu hỏi chuyên biệt thêm vào Framework gốc. Với Tới Nơi hợp nhất là **Serverless Applications Lens** (bài 18, 19, 57) và **Container Build Lens** (bài 11, 25, 26). Lens Catalog còn có **DevOps** và **Connected Mobility** (hệ thống giao thông, sát nghiệp vụ Tới Nơi); không có lens riêng cho Operational Readiness Review, vốn là một tài liệu hướng dẫn. Tiêu chuẩn nội bộ thì viết **custom lens** bằng JSON.
- **Milestone**: ảnh chụp trạng thái tại một thời điểm, dùng để so sánh trước và sau khi cải tiến.
- **Pillar priorities**: bạn xếp trụ cột nào quan trọng hơn cho workload này, để kế hoạch cải tiến ưu tiên theo đó.

```bash
aws wellarchitected create-workload \
  --workload-name toinoi-prod-booking \
  --description "Dat chuyen: ECS, Aurora, MSK, Step Functions" \
  --environment PRODUCTION --review-owner platform-lead@toinoi.example \
  --aws-regions ap-southeast-1 \
  --lenses wellarchitected serverless \
  --pillar-priorities reliability security costOptimization
```

Lệnh này tạo workload cho nhóm `booking` ở production, gắn Framework gốc (alias `wellarchitected`) cùng Serverless Lens (`serverless`) và xếp ba trụ cột ưu tiên; alias của lens khác tra bằng `aws wellarchitected list-lenses`. Sau khi trả lời, `aws wellarchitected list-lens-review-improvements --workload-id <id> --lens-alias wellarchitected` liệt kê các cải tiến, và `aws wellarchitected create-milestone --workload-id <id> --milestone-name 2026-q4-baseline` chụp mốc.

Tool không thấy hệ thống, chỉ dựa vào câu trả lời của bạn. Phần "nhìn thấy thật" đến từ **Trusted Advisor** (kiểm chi phí, bảo mật, khả năng chịu lỗi, giới hạn dịch vụ; phạm vi tuỳ support plan) và **AWS Health Dashboard** (sự kiện của AWS ảnh hưởng tới tài khoản).

## Finding thật và cách đọc chúng

Một finding tốt gồm bằng chứng, hậu quả và cách sửa. Năm finding mẫu từ lượt review trên:

| Finding | Trụ cột | Hậu quả | Hướng sửa |
| --- | --- | --- | --- |
| NAT Gateway chỉ ở một AZ | Reliability | Mất AZ đó thì mọi subnet private mất đường ra Internet | NAT mỗi AZ, thêm VPC endpoint (bài 04, 53) |
| Log group không có retention | Cost | Log nằm vô thời hạn, hoá đơn tăng đều | Đặt retention theo loại log (bài 10) |
| Chưa có game day | Operational Excellence | Không biết runbook có chạy được | Lịch game day quý với FIS (bài 56) |
| Ảnh chứng từ có thể chứa PII, chưa quét | Security | Rò rỉ dữ liệu khách không phát hiện được | Bật Macie cho bucket chứng từ (bài 54) |
| Service `quote` chạy x86, chưa thử Graviton | Sustainability | Tốn năng lượng và tiền hơn cho cùng việc | Build image arm64, task definition `ARM64`, canary (bài 11, 12) |

Không phải finding nào cũng sửa ngay. Chấm hai trục: **tác động** nếu xảy ra và **công sửa**. NAT một AZ tác động lớn, công vừa, nên đứng đầu; Graviton tác động vừa nhưng công lớn hơn (thư viện native, test hiệu năng) nên xếp sau.

## Biến finding thành backlog có đo được

Review thất bại khi danh sách kết quả nằm trong một tài liệu không ai mở lại. Mỗi finding phải thành một việc trong công cụ quản lý công việc của đội, với năm thứ:

| Trường | Ví dụ |
| --- | --- |
| Owner | Một người, không phải "team hạ tầng" |
| Ước lượng | 2 ngày, hoặc "cần thiết kế trước" |
| Chỉ số hoàn thành | "Mọi log group có `retentionInDays`", kiểm bằng script AWS Config |
| Hạn | Theo mức rủi ro: HRI trong quý, MRI trong hai quý |
| Liên kết | Câu hỏi và bằng chứng trong Tool |

Review thành "checkbox" theo ba cách: chọn đáp án cho điểm đẹp, một người chấm thay cả đội, và không bao giờ review lại. Cách tránh: mỗi câu trả lời kèm bằng chứng (dashboard, kết quả diễn tập), có cả người vận hành lẫn người phát triển, và **review lại sau 90 ngày** bằng milestone mới để thấy HRI nào đã đóng. Review trước mỗi mốc lớn (mở thành phố, đổi kiến trúc) để phát hiện khi còn rẻ.

## Cái giá phải trả

- **Review tốn thời gian của người giỏi**: một lượt cho workload vừa mất vài buổi; chọn workload quan trọng nhất.
- **Câu trả lời tự chấm dễ lạc quan**: kiểm chéo với dữ liệu thật như Trusted Advisor, Config, kết quả diễn tập.
- **Framework không biết nghiệp vụ**: nó không nói chuyến đặt xe cần bao nhiêu nine. SLO của bạn vẫn phải do sản phẩm đặt ra.
- **Backlog phình ra**: nếu mọi MRI đều thành việc, đội ngập. Chấp nhận có chủ đích một số rủi ro, ghi lại lý do và hạn xem lại.

**Bài tập:** tự chấm năm câu hỏi Reliability cho hệ thống bạn đang vận hành: (1) mất một AZ thì sao, (2) backup đã restore thử chưa, (3) retry có jitter và circuit breaker chưa, (4) có giới hạn blast radius khi một khách hoặc lỗi triển khai gây hại không, (5) quota đã đủ để mở rộng gấp đôi chưa. Với mỗi câu ghi bằng chứng và một hành động.

<details><summary>Xem đáp án</summary><p>Không có đáp án chung; một bản chấm tốt có dạng "câu, trạng thái, bằng chứng, việc". Ví dụ: (1) Một phần, có Multi-AZ cho DB nhưng NAT một AZ (finding đầu bảng trên), việc: thêm NAT thứ hai, hạn hai tuần. (2) Đạt, restore thử hằng tuần bằng AWS Backup, RTO đo 38 phút, việc: lặp lại mỗi quý. (3) Chưa, hai service gọi nhau không có timeout, việc: thêm timeout và backoff có jitter, hạn một sprint. Nếu cả năm câu bạn đều ghi "Đạt" mà không có bằng chứng thì đó là dấu hiệu review hình thức.</p></details>

Bài trước: [bài 57](/articles/aws-step-functions-saga-dat-chuyen). Bài tiếp: [bài 59](/articles/aws-finops-savings-plans-spot-right-sizing).

## Tài liệu gốc

- [AWS Well-Architected Framework](https://docs.aws.amazon.com/wellarchitected/latest/framework/welcome.html).
- [AWS Well-Architected Tool User Guide](https://docs.aws.amazon.com/wellarchitected/latest/userguide/intro.html).
- [AWS Well-Architected Lenses](https://docs.aws.amazon.com/wellarchitected/latest/userguide/lenses.html).
- [AWS Trusted Advisor](https://docs.aws.amazon.com/awssupport/latest/user/trusted-advisor.html).
- [AWS Health Dashboard](https://docs.aws.amazon.com/health/latest/ug/what-is-aws-health.html).

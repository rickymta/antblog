# Master Curriculum — AWS từ cơ bản tới nâng cao qua dự án Tới Nơi

Bản quản lý toàn bộ lộ trình **AWS qua Tới Nơi** của Tech Knowledge Hub (antblog.xyz): **60 bài** trong **6 phần**, mỗi phần tối đa 10 bài, đi từ tài khoản đầu tiên tới kiến trúc production nhiều region. Chủ đề trên site: `devops-aws` (con của `devops`). Mỗi phần là một lộ trình học (`learning-paths.json`), mỗi bài là một tệp `api/seed/content/<slug>.md`.

Mọi ví dụ dùng hệ thống hư cấu [Tới Nơi](../api/seed/content/du-an-mau-toi-noi.md) (đặt xe, giao đồ ăn, giao bưu kiện ở 6 thành phố; 2 triệu chuyến/ngày, 70 chuyến/giây và 6.250 toạ độ GPS/giây lúc cao điểm) và bám [kiến trúc tổng thể](toi-noi-overall-architecture.md) (microservices, Kafka, PostgreSQL, Redis, Kubernetes tuỳ chọn, Terraform, GitOps).

## 1. Lộ trình theo phần

| Phần | Lộ trình (slug) | Cấp độ | Bài | Bám theo chứng chỉ |
|---|---|---|---:|---|
| Phần 01 · Nền tảng AWS | `aws-p01-nen-tang-aws` | Beginner | 10 | CLF-C02, nửa đầu SAA-C03 |
| Phần 02 · Container, serverless và dịch vụ ứng dụng | `aws-p02-container-serverless` | Intermediate | 10 | DVA-C02, SAA-C03 |
| Phần 03 · Infrastructure as Code và CI/CD | `aws-p03-iac-ci-cd` | Intermediate | 10 | DOP-C02, DVA-C02 |
| Phần 04 · Dữ liệu, lưu trữ và messaging | `aws-p04-du-lieu-messaging` | Advanced | 10 | SAA-C03, SAP-C02, DEA-C01 |
| Phần 05 · Kubernetes trên AWS với EKS | `aws-p05-eks` | Advanced | 10 | DOP-C02, CKA |
| Phần 06 · Bảo mật, resilience và vận hành ở quy mô | `aws-p06-bao-mat-van-hanh` | Advanced | 10 | SCS-C02, SAP-C02 |

Chứng chỉ chỉ là mốc tham chiếu để người đọc biết phần nào tương đương kiến thức nào; bài viết không chép câu hỏi thi và không bị giới hạn bởi đề cương thi.

### Đối tượng và tiền đề

- **Phần 01** cho người chưa từng mở console AWS: chỉ cần biết dùng terminal, hiểu HTTP và SQL ở mức cơ bản. Có thể đọc song song loạt [C# và .NET 10](../examples/toi-noi-dotnet/README.md) nhưng không bắt buộc.
- **Phần 02–03** cần xong Phần 01 và biết Docker ở mức build/run image.
- **Phần 04–06** cần xong Phần 02–03; Phần 05 cần hiểu Kubernetes cơ bản (pod, deployment, service) — bài 41 ôn lại tối thiểu.

## 2. Quy ước xuyên suốt

| Mục | Quy ước |
|---|---|
| Region mặc định | `ap-southeast-1` (Singapore). Khi cần region thứ hai dùng `ap-northeast-1`; CloudFront/ACM cho CloudFront dùng `us-east-1`. |
| Tài khoản, ARN | Account ID giả `123456789012`; ARN ví dụ `arn:aws:iam::123456789012:role/toinoi-booking-task`. Không bao giờ dùng account, key, secret, tên miền hay IP thật. |
| Tên tài nguyên | Prefix `toinoi-<env>-<service>`, ví dụ `toinoi-prod-booking`; bucket `toinoi-lab-delivery-photos`; tên miền minh hoạ `toinoi.example`, `api.toinoi.example`. |
| Tag bắt buộc | `Project=ToiNoi`, `Env=lab|dev|staging|prod`, `Service=<tên>`, `Owner=<team>`. |
| Chi phí | Mỗi bài có lab ghi rõ tài nguyên nào tính tiền theo giờ và có bước **dọn tài nguyên**. Không ghi giá cụ thể (đổi theo thời gian); nếu buộc phải nêu thì ghi "số liệu minh hoạ, xem Pricing Calculator". |
| Số liệu | Lấy từ hồ sơ Tới Nơi (70 chuyến/giây, 560 UPDATE/giây, 6.250 toạ độ/giây, 30.000 đọc/giây, 730 triệu dòng/năm). Mọi con số khác ghi rõ là giả định. |
| Dịch vụ đã ngừng | Không dùng CodeCommit cho khách mới, App Mesh (hết hỗ trợ 09/2026), Cloud9, lock Terraform bằng DynamoDB (dùng `use_lockfile`). Khi nhắc tới phải ghi rõ trạng thái. |
| Công cụ | AWS CLI v2 với profile SSO, Terraform ≥ 1.10 (hoặc OpenTofu), CDK v2, kubectl/Helm/eksctl ở Phần 05. Lệnh viết đúng cú pháp hiện hành; option không chắc thì tra tài liệu trước khi viết. |
| Mã ví dụ | Nằm ở `examples/toi-noi-aws/<pNN-slug>/` khi bài có lab (cột **Lab** bên dưới). Terraform phải qua `terraform fmt -check` và `terraform validate`; YAML/JSON phải parse được. Không có lệnh nào trong repo tự tạo tài nguyên cloud. |

## 3. Khung một bài

Theo đúng khung các loạt đã xuất bản (xem `api/seed/content/csharp-va-dotnet-10-tu-source-code-den-chuong-trinh.md` và `dotnet10-aws-azure-deployment.md`):

1. **Frontmatter**: `slug`, `title`, `summary` (bắt đầu bằng "Bài NN: …", ≤ 400 ký tự), `category: devops-aws`, `level`, `tags` (≤ 10, chữ thường, luôn có `aws`, `toi-noi`, `devops`), `language: vi`.
2. **Mở bài** bằng một tình huống của Tới Nơi (2–4 câu), rồi câu "Đây là **bài NN** của lộ trình **AWS qua Tới Nơi · Phần 0X: <tên phần>**" kèm yêu cầu tiền đề.
3. **Callout "Thử trước khi đọc"** (`tkh-callout tkh-callout--example`) với một câu hỏi và đáp án trong `<details>`.
4. **4–7 mục H2**: cơ chế → cách dựng cho Tới Nơi → cái giá/giới hạn. Ít nhất một bảng so sánh. Lệnh CLI, HCL, YAML, JSON trong code block có ngôn ngữ.
5. **1–2 hình SVG** viết tay trong `<figure class="tkh-fig">` (viewBox rộng 880, lớp `f-phys-soft s-phys t-b t-small f-warn-soft s-warn t-warn f-pk-soft s-pk`…, marker id duy nhất theo bài, ví dụ `aws12-0-a`), có `aria-label` và `<figcaption>Hình N. …</figcaption>`.
6. **Lỗi thường gặp** hoặc **Cái giá phải trả**: 3–5 gạch đầu dòng từ kinh nghiệm vận hành.
7. **Bài tập** in đậm `**Bài tập:**` và đáp án trong `<details><summary>Xem đáp án</summary>…</details>`.
8. **Lab / Code và kiểm chứng** (nếu có): đường dẫn `examples/toi-noi-aws/...`, lệnh kiểm trên máy, bước dọn tài nguyên.
9. **Bài trước / Bài tiếp** dạng `[bài NN](/articles/<slug>)` — chỉ link tới bài có trong bảng này.
10. **## Tài liệu gốc**: 3–6 link tới `docs.aws.amazon.com` / `aws.amazon.com` / tài liệu chính thức của công cụ (Terraform, Kubernetes, Argo). Không bịa URL; khi không chắc đường dẫn thì link tới trang user guide gốc của dịch vụ.

Độ dài 1.300–1.900 từ (không tính SVG). Giọng văn: tiếng Việt, thuật ngữ AWS giữ nguyên tiếng Anh, giải thích cơ chế trước khi đưa lệnh; nói rõ điều gì **chưa được kiểm chứng trên tài khoản thật** khi bài chỉ có template.

## 4. Danh sách bài

### Phần 01 · Nền tảng AWS — `aws-p01-nen-tang-aws` (Beginner)

Bắt đầu từ con số không: region và availability zone, tài khoản và IAM, VPC, EC2, S3, RDS PostgreSQL, ALB và Auto Scaling, Route 53 với ACM, CloudWatch và chi phí. Mỗi bài dựng một mảnh hạ tầng đầu tiên của Tới Nơi bằng console và AWS CLI, kết thúc bằng bước dọn tài nguyên.

| # | Slug | Tiêu đề | Lab |
|---:|---|---|---|
| 01 | `aws-la-gi-region-az-shared-responsibility` | AWS là gì: region, availability zone, edge location và trách nhiệm chia sẻ | — |
| 02 | `aws-tai-khoan-iam-identity-center-cli` | Tài khoản AWS, IAM Identity Center và AWS CLI: thiết lập an toàn ngày đầu | `p01-02-cli-setup` |
| 03 | `aws-iam-user-role-policy` | IAM: user, group, role và policy, quyền tối thiểu cho Tới Nơi | `p01-03-iam-policies` |
| 04 | `aws-vpc-subnet-route-security-group` | VPC: subnet public và private, route table, NAT, security group và NACL | `p01-04-vpc` |
| 05 | `aws-ec2-ami-ebs-user-data` | EC2: chạy service đầu tiên của Tới Nơi bằng AMI, EBS, user data và Session Manager | `p01-05-ec2` |
| 06 | `aws-s3-bucket-versioning-presigned-url` | S3: bucket, versioning, lifecycle và presigned URL cho ảnh chứng từ giao hàng | `p01-06-s3` |
| 07 | `aws-rds-postgresql-multi-az-backup` | RDS PostgreSQL: tạo instance, Multi-AZ, backup và kết nối từ private subnet | `p01-07-rds` |
| 08 | `aws-alb-target-group-auto-scaling` | ALB, target group và Auto Scaling Group: chịu tải giờ cao điểm 11:00 và 17:00 | `p01-08-alb-asg` |
| 09 | `aws-route53-acm-https` | Route 53 và ACM: tên miền, chứng chỉ và HTTPS end-to-end cho api.toinoi.example | — |
| 10 | `aws-cloudwatch-budgets-tagging-co-ban` | CloudWatch, Budgets và tagging: nhìn thấy hệ thống và chi phí trước khi mở rộng | `p01-10-cloudwatch-cleanup` |

### Phần 02 · Container, serverless và dịch vụ ứng dụng — `aws-p02-container-serverless` (Intermediate)

Đưa service đặt chuyến của Tới Nơi lên ECS Fargate với ECR, ALB và auto scaling; quản lý secret bằng Secrets Manager và Parameter Store; thêm Redis, SQS, SNS, EventBridge, Lambda, API Gateway, CloudFront và WAF để hệ thống có cache, hàng đợi, sự kiện và lớp chắn ở biên.

| # | Slug | Tiêu đề | Lab |
|---:|---|---|---|
| 11 | `aws-ecr-image-scan-lifecycle` | ECR: đẩy image, quét lỗ hổng, tag bất biến và lifecycle policy | `p02-11-ecr` |
| 12 | `aws-ecs-fargate-task-definition-roles` | ECS Fargate: cluster, task definition, service, execution role và task role | `p02-12-ecs-fargate` |
| 13 | `aws-ecs-alb-rolling-deploy-autoscaling` | ECS với ALB: rolling deploy, deployment circuit breaker và service auto scaling | `p02-13-ecs-deploy-scaling` |
| 14 | `aws-secrets-manager-parameter-store` | Secrets Manager và Parameter Store: cấu hình, secret và rotation cho service Tới Nơi | `p02-14-secrets-parameters` |
| 15 | `aws-elasticache-redis-cache-rate-limit` | ElastiCache: cache báo giá, session và rate limit bằng Redis hoặc Valkey | `p02-15-elasticache` |
| 16 | `aws-sqs-sns-dlq-fan-out` | SQS và SNS: hàng đợi, dead-letter queue và fan-out thông báo khi có chuyến mới | `p02-16-sqs-sns` |
| 17 | `aws-eventbridge-event-bus-rule-schedule` | EventBridge: event bus, rule, schedule và sự kiện chuyến của Tới Nơi | `p02-17-eventbridge` |
| 18 | `aws-lambda-webhook-thanh-toan-cold-start` | Lambda: hàm xử lý webhook thanh toán, trigger từ S3 và SQS, cold start và quyền | `p02-18-lambda-webhook` |
| 19 | `aws-api-gateway-authorizer-websocket` | API Gateway: HTTP API, JWT authorizer, throttling và WebSocket theo dõi chuyến | `p02-19-api-gateway` |
| 20 | `aws-cloudfront-waf-shield` | CloudFront, WAF và Shield: CDN và lớp chắn cho web và API của Tới Nơi | `p02-20-cloudfront-waf` |

### Phần 03 · Infrastructure as Code và CI/CD — `aws-p03-iac-ci-cd` (Intermediate)

Mô tả hạ tầng bằng CloudFormation, Terraform và CDK; pipeline GitHub Actions với OIDC, CodePipeline với blue/green; chiến lược rolling, canary, rollback có migration; nhiều tài khoản với Organizations và Control Tower; AppConfig và policy as code để pipeline tự kiểm hạ tầng trước khi áp dụng.

| # | Slug | Tiêu đề | Lab |
|---:|---|---|---|
| 21 | `aws-cloudformation-stack-change-set-drift` | CloudFormation: template, stack, change set, drift và vì sao hạ tầng phải là code | `p03-21-cloudformation` |
| 22 | `aws-terraform-provider-state-s3-lock` | Terraform trên AWS: provider, resource, remote state trên S3 và lock | `p03-22-terraform-basics` |
| 23 | `aws-terraform-module-vpc-ecs-rds` | Terraform module: dựng VPC, ECS Fargate và RDS cho Tới Nơi theo từng môi trường | `p03-23-terraform-modules` |
| 24 | `aws-cdk-construct-synth-diff` | AWS CDK: construct, stack, synth và diff bằng TypeScript cho service báo giá | `p03-24-cdk` |
| 25 | `aws-github-actions-oidc-ecr-deploy` | GitHub Actions và OIDC: assume role không cần access key, build, đẩy ECR và deploy ECS | `p03-25-github-actions-oidc` |
| 26 | `aws-codepipeline-codebuild-codedeploy-blue-green` | CodePipeline, CodeBuild và CodeDeploy: blue/green cho ECS với traffic shifting | `p03-26-codedeploy-blue-green` |
| 27 | `aws-chien-luoc-deploy-rolling-canary-rollback` | Chiến lược triển khai: rolling, blue/green, canary, rollback và cổng migration database | — |
| 28 | `aws-organizations-scp-control-tower-landing-zone` | Organizations, SCP và Control Tower: nhiều tài khoản cho dev, staging, prod và bảo mật | — |
| 29 | `aws-appconfig-feature-flag-cau-hinh-dong` | AppConfig và feature flag: đổi bảng giá và bật tính năng mới không cần deploy | `p03-29-appconfig` |
| 30 | `aws-policy-as-code-checkov-infracost-drift` | Policy as code trong pipeline: cfn-guard, Checkov, tflint, Infracost và kiểm drift | `p03-30-policy-as-code` |

### Phần 04 · Dữ liệu, lưu trữ và messaging — `aws-p04-du-lieu-messaging` (Advanced)

Aurora PostgreSQL và RDS Proxy, DynamoDB cho vị trí tài xế, ElastiCache và MemoryDB, Amazon MSK cho outbox, Kinesis cho GPS, S3 nâng cao, AWS Backup và DR, lakehouse với Glue, Athena, Redshift Serverless, OpenSearch và DMS: dữ liệu của 2 triệu chuyến mỗi ngày đi đâu và được giữ an toàn thế nào.

| # | Slug | Tiêu đề | Lab |
|---:|---|---|---|
| 31 | `aws-aurora-postgresql-rds-proxy-failover` | Aurora PostgreSQL và RDS Proxy: reader endpoint, failover, Serverless v2 và PITR | `p04-31-aurora` |
| 32 | `aws-dynamodb-single-table-gsi-ttl-streams` | DynamoDB: single-table design cho vị trí tài xế và trạng thái chuyến, GSI, TTL và Streams | `p04-32-dynamodb` |
| 33 | `aws-elasticache-nang-cao-memorydb-ha` | ElastiCache nâng cao và MemoryDB: HA, failover, cluster mode, eviction và pub/sub | `p04-33-elasticache-ha` |
| 34 | `aws-msk-kafka-iam-auth-msk-connect` | Amazon MSK: Kafka managed, IAM auth, MSK Connect và outbox của Tới Nơi | `p04-34-msk` |
| 35 | `aws-kinesis-data-streams-firehose-gps` | Kinesis Data Streams và Data Firehose: nuốt 6.250 toạ độ GPS mỗi giây | `p04-35-kinesis` |
| 36 | `aws-s3-nang-cao-storage-class-replication-object-lock` | S3 nâng cao: storage class, replication, Object Lock, event notification và S3 Tables | `p04-36-s3-advanced` |
| 37 | `aws-backup-disaster-recovery-rpo-rto` | AWS Backup và disaster recovery: RPO, RTO, pilot light, warm standby và diễn tập restore | `p04-37-backup` |
| 38 | `aws-glue-athena-redshift-lakehouse` | Glue, Athena và Redshift Serverless: lakehouse phân tích 2 triệu chuyến mỗi ngày | `p04-38-athena` |
| 39 | `aws-opensearch-tim-kiem-log-analytics` | OpenSearch Service: tìm quán và địa chỉ, geo query và phân tích log tập trung | `p04-39-opensearch` |
| 40 | `aws-dms-migration-postgresql-zero-downtime` | DMS và cắt chuyển gần như không downtime: từ PostgreSQL tự vận hành sang Aurora | `p04-40-dms` |

### Phần 05 · Kubernetes trên AWS với EKS — `aws-p05-eks` (Advanced)

Dựng và vận hành EKS cho Tới Nơi: control plane, node group, Pod Identity, VPC CNI và Load Balancer Controller, EBS/EFS CSI, HPA và Karpenter, GitOps với Argo CD, quan sát bằng ADOT và Managed Prometheus, bảo mật pod, progressive delivery với Argo Rollouts và VPC Lattice, nâng cấp cluster và chi phí.

| # | Slug | Tiêu đề | Lab |
|---:|---|---|---|
| 41 | `aws-eks-tong-quan-control-plane-node-group` | EKS tổng quan: control plane, managed node group, Fargate profile, Auto Mode và Pod Identity | — |
| 42 | `aws-eks-terraform-eksctl-kubectl-namespace` | Dựng EKS bằng Terraform và eksctl: cluster, node group, kubectl và namespace cho Tới Nơi | `p05-42-eks-terraform` |
| 43 | `aws-eks-vpc-cni-load-balancer-controller-ingress` | Mạng EKS: VPC CNI, prefix delegation, AWS Load Balancer Controller, Ingress và Gateway API | `p05-43-eks-ingress` |
| 44 | `aws-eks-ebs-efs-csi-statefulset` | Lưu trữ EKS: EBS CSI, EFS CSI, StorageClass, StatefulSet và snapshot | `p05-44-eks-storage` |
| 45 | `aws-eks-hpa-karpenter-spot` | Autoscaling EKS: HPA, VPA, KEDA, Karpenter và Spot cho worker giờ cao điểm | `p05-45-eks-autoscaling` |
| 46 | `aws-eks-gitops-argocd-helm-kustomize` | GitOps trên EKS: Argo CD, Helm và Kustomize cho dev, staging và prod | `p05-46-eks-gitops` |
| 47 | `aws-eks-observability-container-insights-adot-prometheus` | Quan sát EKS: Container Insights, ADOT, Managed Prometheus và Grafana, log và trace | `p05-47-eks-observability` |
| 48 | `aws-eks-bao-mat-pod-identity-external-secrets` | Bảo mật EKS: Pod Identity, External Secrets, Pod Security Standards, network policy và ký image | `p05-48-eks-security` |
| 49 | `aws-eks-progressive-delivery-argo-rollouts-vpc-lattice` | Progressive delivery trên EKS: Argo Rollouts, canary theo metric, VPC Lattice và giao tiếp giữa service | `p05-49-eks-rollouts` |
| 50 | `aws-eks-van-hanh-upgrade-addon-chi-phi` | Vận hành EKS: nâng cấp cluster theo phiên bản, add-on, chi phí, multi-cluster và runbook | — |

### Phần 06 · Bảo mật, resilience và vận hành ở quy mô — `aws-p06-bao-mat-van-hanh` (Advanced)

IAM nâng cao, KMS, PrivateLink và Transit Gateway, CloudTrail với Security Hub và GuardDuty, quan sát nâng cao với SLO, multi-region và Fault Injection Service, Step Functions cho Saga đặt chuyến, Well-Architected review, FinOps và bài tổng hợp kiến trúc production Tới Nơi trên AWS kèm runbook.

| # | Slug | Tiêu đề | Lab |
|---:|---|---|---|
| 51 | `aws-iam-nang-cao-permission-boundary-abac-access-analyzer` | IAM nâng cao: permission boundary, condition key, ABAC, session policy và Access Analyzer | `p06-51-iam-advanced` |
| 52 | `aws-kms-envelope-encryption-key-policy` | KMS: envelope encryption, key policy, grant, multi-region key và mã hoá S3, RDS, EBS, Secrets | `p06-52-kms` |
| 53 | `aws-privatelink-transit-gateway-network-firewall` | PrivateLink, VPC endpoint, Transit Gateway, Network Firewall và kết nối hybrid | `p06-53-private-network` |
| 54 | `aws-cloudtrail-config-security-hub-guardduty` | CloudTrail, Config, Security Hub, GuardDuty và Inspector: audit và phát hiện cho dữ liệu khách | `p06-54-detection` |
| 55 | `aws-observability-nang-cao-xray-adot-logs-insights-slo` | Quan sát nâng cao: X-Ray và ADOT, Logs Insights, anomaly detection, Application Signals và SLO | `p06-55-slo` |
| 56 | `aws-resilience-multi-region-route53-failover-fis` | Resilience: multi-AZ tới multi-region, Route 53 failover và ARC, Global Accelerator, Fault Injection Service | `p06-56-resilience` |
| 57 | `aws-step-functions-saga-dat-chuyen` | Step Functions: Saga đặt chuyến, compensation, distributed map và các pattern serverless nâng cao | `p06-57-step-functions` |
| 58 | `aws-well-architected-review-toi-noi` | Well-Architected Framework: review sáu trụ cột cho Tới Nơi và biến finding thành backlog | — |
| 59 | `aws-finops-savings-plans-spot-right-sizing` | FinOps trên AWS: cost allocation, Savings Plans, Reserved, Spot, right-sizing và kiến trúc tiết kiệm | `p06-59-finops` |
| 60 | `aws-tong-hop-kien-truc-production-toi-noi` | Tổng hợp: kiến trúc production Tới Nơi trên AWS, luồng một chuyến đi qua từng dịch vụ và runbook | — |

## 5. Đề cương từng bài

Mỗi mục là đề cương bắt buộc cho người viết (và là checklist cho người review). Tags liệt kê là tag riêng của bài, cộng thêm ba tag chung `aws`, `toi-noi`, `devops`.

### Bài 01 · AWS là gì: region, availability zone, edge location và trách nhiệm chia sẻ

- **Slug**: `aws-la-gi-region-az-shared-responsibility` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: không
- **Tóm tắt**: Bài 01: AWS cung cấp gì, region và availability zone khác nhau ở đâu, edge location để làm gì, mô hình shared responsibility chia việc giữa AWS và Tới Nơi ra sao, và chọn region nào cho một ứng dụng chạy ở Việt Nam.
- **Tags riêng**: `region`, `availability-zone`, `shared-responsibility`, `cloud-co-ban`
- **Nội dung bắt buộc**:
  - Giải thích IaaS/PaaS/SaaS và mô hình trả theo dùng; AWS Global Infrastructure: region, AZ (nhiều data center, độc lập điện/mạng), Local Zone, edge location/CloudFront POP.
  - Chọn region cho Tới Nơi: độ trễ tới người dùng Việt Nam, dịch vụ sẵn có, chi phí, quy định dữ liệu; dùng ap-southeast-1 (Singapore) làm region mặc định của cả loạt, nhắc Local Zone/Region mới cần kiểm tra trên trang chính thức thay vì giả định.
  - Shared responsibility: AWS chịu 'security of the cloud', khách chịu 'security in the cloud'; bảng ví dụ với EC2, RDS, S3, Lambda.
  - Bản đồ dịch vụ sẽ gặp trong cả 6 phần theo lớp: compute, storage, database, network, security, observability, IaC.
  - Không có lab tốn tiền; bài tập: xếp 8 trách nhiệm vận hành Tới Nơi vào đúng bên.

### Bài 02 · Tài khoản AWS, IAM Identity Center và AWS CLI: thiết lập an toàn ngày đầu

- **Slug**: `aws-tai-khoan-iam-identity-center-cli` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-02-cli-setup`
- **Tóm tắt**: Bài 02: khoá root user bằng MFA, tạo người dùng qua IAM Identity Center, đặt Budgets để không giật mình khi có hoá đơn, cài AWS CLI v2 và đăng nhập bằng aws configure sso; kiểm tra bằng aws sts get-caller-identity.
- **Tags riêng**: `iam-identity-center`, `aws-cli`, `mfa`, `budgets`, `root-user`
- **Nội dung bắt buộc**:
  - Root user chỉ dùng cho vài việc; bật MFA, không tạo access key cho root; billing alert và AWS Budgets với ngưỡng nhỏ cho tài khoản học.
  - IAM Identity Center (tên cũ AWS SSO): user, group, permission set; vì sao ưu tiên hơn IAM user có access key dài hạn.
  - AWS CLI v2: cài đặt, profile, `aws configure sso`, `aws sso login`, biến AWS_PROFILE, output json/table, `--query` với JMESPath.
  - Quy ước của loạt bài: account id giả 123456789012, region ap-southeast-1, tag Project=ToiNoi, Env=lab; không bao giờ dán access key vào bài.
  - Bài tập: viết lệnh liệt kê region và kiểm tra identity; đáp án trong details.

### Bài 03 · IAM: user, group, role và policy, quyền tối thiểu cho Tới Nơi

- **Slug**: `aws-iam-user-role-policy` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-03-iam-policies`
- **Tóm tắt**: Bài 03: đọc một IAM policy JSON (Effect, Action, Resource, Condition), phân biệt identity-based và resource-based policy, hiểu role và trust policy, rồi viết policy quyền tối thiểu cho service upload ảnh giao hàng lên S3.
- **Tags riêng**: `iam`, `policy`, `role`, `least-privilege`, `trust-policy`
- **Nội dung bắt buộc**:
  - Cấu trúc policy: Version, Statement, Effect, Action, Resource (ARN), Condition; explicit deny thắng allow; đánh giá policy khi có nhiều policy.
  - User/group/role; role có trust policy (ai được assume) và permission policy (được làm gì); STS AssumeRole, temporary credential.
  - Managed policy vs inline, AWS managed vs customer managed; vì sao không dùng AdministratorAccess cho workload.
  - Ví dụ: policy cho service delivery-photo chỉ PutObject/GetObject trên prefix s3://toinoi-lab-delivery-photos/proof/*; policy simulator và CloudTrail để debug AccessDenied.
  - Bài tập: sửa một policy quá rộng (s3:* trên *) thành quyền tối thiểu.

### Bài 04 · VPC: subnet public và private, route table, NAT, security group và NACL

- **Slug**: `aws-vpc-subnet-route-security-group` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-04-vpc`
- **Tóm tắt**: Bài 04: dựng VPC 10.20.0.0/16 cho Tới Nơi với subnet public/private ở hai AZ, Internet Gateway, NAT Gateway, route table; phân biệt security group có trạng thái với NACL không trạng thái và đặt luật để chỉ ALB tới được app.
- **Tags riêng**: `vpc`, `subnet`, `nat-gateway`, `security-group`, `nacl`, `networking`
- **Nội dung bắt buộc**:
  - CIDR, subnet per AZ, public subnet = route 0.0.0.0/0 tới IGW; private subnet qua NAT Gateway (chi phí giờ + dữ liệu); IPv6 nhắc qua.
  - Security group: stateful, chỉ allow, tham chiếu security group khác (sg-alb → sg-app port 8080 → sg-db port 5432); NACL stateless, có deny, theo subnet.
  - VPC Flow Logs để debug 'không kết nối được'; Reachability Analyzer.
  - Sơ đồ SVG: 2 AZ × (public, private app, private data) = 6 subnet; đường đi của một request từ Internet tới app và từ app ra Internet qua NAT.
  - Lab CLI tạo VPC bằng aws ec2 create-vpc/create-subnet/...; cleanup script; bài tập: vì sao RDS trong private subnet vẫn cập nhật được patch?

### Bài 05 · EC2: chạy service đầu tiên của Tới Nơi bằng AMI, EBS, user data và Session Manager

- **Slug**: `aws-ec2-ami-ebs-user-data` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-05-ec2`
- **Tóm tắt**: Bài 05: chọn instance type và AMI, gắn IAM instance profile, dùng user data để cài và chạy service báo giá, truy cập bằng Systems Manager Session Manager thay vì mở SSH, hiểu EBS, snapshot và IMDSv2.
- **Tags riêng**: `ec2`, `ami`, `ebs`, `user-data`, `session-manager`, `imdsv2`
- **Nội dung bắt buộc**:
  - Instance family (t, m, c, r), Graviton, burstable credit; AMI Amazon Linux 2023; pricing model On-Demand/Spot/Savings Plans chỉ giới thiệu.
  - Instance profile = role cho EC2; IMDSv2 bắt buộc; không đặt access key lên máy.
  - User data cài Docker và chạy container service báo giá Tới Nơi (image minh hoạ ghcr.io/toinoi/quote:lab hoặc nginx placeholder) nghe 8080.
  - Session Manager thay SSH: không mở port 22, log phiên; EC2 Instance Connect Endpoint nhắc qua.
  - EBS gp3, snapshot, root volume delete-on-termination; bài tập: instance dừng/stop/terminate khác nhau về chi phí và dữ liệu.

### Bài 06 · S3: bucket, versioning, lifecycle và presigned URL cho ảnh chứng từ giao hàng

- **Slug**: `aws-s3-bucket-versioning-presigned-url` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-06-s3`
- **Tóm tắt**: Bài 06: thiết kế bucket ảnh chứng từ giao hàng của Tới Nơi với Block Public Access, mã hoá mặc định, versioning, lifecycle sang lớp lưu trữ rẻ hơn; tài xế upload bằng presigned URL và khách xem bằng URL có hạn.
- **Tags riêng**: `s3`, `presigned-url`, `versioning`, `lifecycle`, `object-storage`
- **Nội dung bắt buộc**:
  - Object storage khác file system: key, prefix, consistency; bucket name toàn cục; Block Public Access bật mặc định, ACL tắt mặc định, SSE-S3 mặc định.
  - Versioning và lifecycle: chuyển Standard → Standard-IA → Glacier Instant Retrieval sau N ngày, xoá version cũ; Intelligent-Tiering khi không biết pattern.
  - Presigned URL PUT/GET với thời hạn, tạo từ service có role; bucket policy chỉ cho role app; CORS cho web.
  - Static website hosting chỉ nhắc, CloudFront ở bài 20.
  - Lab CLI: tạo bucket, bật versioning, tạo presigned URL bằng `aws s3 presign`; bài tập: xoá object có versioning thì chuyện gì xảy ra?

### Bài 07 · RDS PostgreSQL: tạo instance, Multi-AZ, backup và kết nối từ private subnet

- **Slug**: `aws-rds-postgresql-multi-az-backup` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-07-rds`
- **Tóm tắt**: Bài 07: tạo RDS PostgreSQL cho bảng trips của Tới Nơi trong private subnet với DB subnet group, parameter group, Multi-AZ, automated backup và snapshot; kết nối từ EC2 bằng security group và secret trong Secrets Manager.
- **Tags riêng**: `rds`, `postgresql`, `multi-az`, `backup`, `parameter-group`
- **Nội dung bắt buộc**:
  - RDS quản lý gì thay bạn (patch, backup, failover) và không quản lý gì (schema, query, index); instance class, storage gp3, storage autoscaling.
  - DB subnet group, security group chỉ nhận từ sg-app; không public; managed master password trong Secrets Manager.
  - Multi-AZ instance vs Multi-AZ DB cluster (2 reader); failover đổi DNS, ứng dụng phải reconnect; automated backup, retention, PITR, manual snapshot.
  - Parameter group: log_min_duration_statement, max_connections; Performance Insights/Database Insights; CloudWatch metric CPU, FreeStorageSpace, DatabaseConnections.
  - Lab: tạo instance nhỏ, psql từ EC2 qua Session Manager, nạp vài dòng trips; bài tập: Multi-AZ có thay thế backup không?

### Bài 08 · ALB, target group và Auto Scaling Group: chịu tải giờ cao điểm 11:00 và 17:00

- **Slug**: `aws-alb-target-group-auto-scaling` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-08-alb-asg`
- **Tóm tắt**: Bài 08: đặt Application Load Balancer trước service báo giá, cấu hình target group với health check, launch template và Auto Scaling Group giãn theo CPU hoặc request/target, lịch scale theo hai khung giờ cao điểm của Tới Nơi.
- **Tags riêng**: `alb`, `target-group`, `auto-scaling`, `health-check`, `launch-template`
- **Nội dung bắt buộc**:
  - ALB layer 7: listener, rule theo path/host, target group, health check path /health/ready, deregistration delay; NLB chỉ so sánh ngắn.
  - Launch template thay launch configuration; ASG min/desired/max, nhiều AZ, instance refresh khi đổi AMI.
  - Scaling policy: target tracking (ALBRequestCountPerTarget, CPU), step, scheduled cho 11:00–13:00 và 17:00–19:00 (giờ Việt Nam = UTC+7, cron UTC).
  - Sticky session và vì sao service nên stateless; access log ALB vào S3.
  - Lab: ALB + ASG 2 instance từ bài 05; bài tập: health check fail một instance thì ALB và ASG làm gì khác nhau?

### Bài 09 · Route 53 và ACM: tên miền, chứng chỉ và HTTPS end-to-end cho api.toinoi.example

- **Slug**: `aws-route53-acm-https` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: không
- **Tóm tắt**: Bài 09: tạo hosted zone, bản ghi alias trỏ về ALB, xin chứng chỉ ACM xác thực bằng DNS, bật listener 443 và chuyển hướng 80 sang 443; routing policy weighted, latency, failover với health check.
- **Tags riêng**: `route53`, `acm`, `https`, `dns`, `tls`
- **Nội dung bắt buộc**:
  - Hosted zone public/private, record A/AAAA/CNAME, alias record tới ALB/CloudFront không tính phí query; TTL.
  - ACM: certificate miễn phí cho tài nguyên AWS, xác thực DNS tự gia hạn, region của certificate phải trùng ALB (CloudFront cần us-east-1).
  - Listener HTTPS, security policy TLS 1.2/1.3, redirect 80→443, SNI nhiều certificate.
  - Routing policy: simple, weighted (canary), latency, failover + health check, geolocation; nhắc Route 53 Resolver cho private DNS.
  - Lab: dùng tên miền giả toinoi.example minh hoạ, giải thích cần tên miền thật để chạy; bài tập: vì sao không nên dùng CNAME ở apex?

### Bài 10 · CloudWatch, Budgets và tagging: nhìn thấy hệ thống và chi phí trước khi mở rộng

- **Slug**: `aws-cloudwatch-budgets-tagging-co-ban` · **Phần**: 1 · **Cấp độ**: beginner · **Lab**: `examples/toi-noi-aws/p01-10-cloudwatch-cleanup`
- **Tóm tắt**: Bài 10: metric, log group, metric filter, alarm và dashboard CloudWatch cho hạ tầng đã dựng ở phần 01; chiến lược tag Project/Env/Service, cost allocation tag, Cost Explorer và Budgets; checklist dọn tài nguyên để không mất tiền.
- **Tags riêng**: `cloudwatch`, `alarm`, `logs`, `tagging`, `cost-explorer`, `budgets`
- **Nội dung bắt buộc**:
  - Metric namespace, dimension, period, statistic; alarm 3 trạng thái OK/ALARM/INSUFFICIENT_DATA; SNS email khi ALB 5xx tăng.
  - CloudWatch Logs: agent trên EC2, log group, retention, metric filter đếm ERROR, Logs Insights query cơ bản.
  - Dashboard cho service báo giá: request count, latency p95, 5xx, CPU ASG, DB connections.
  - Tag chuẩn: Project=ToiNoi, Env, Service, Owner, CostCenter; cost allocation tag phải kích hoạt; Cost Explorer group by tag; Budgets với action.
  - Checklist cleanup toàn phần 01 (NAT Gateway, ALB, RDS, EIP, snapshot); bài tập: tài nguyên nào vẫn tính tiền khi EC2 đã stop?

### Bài 11 · ECR: đẩy image, quét lỗ hổng, tag bất biến và lifecycle policy

- **Slug**: `aws-ecr-image-scan-lifecycle` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-11-ecr`
- **Tóm tắt**: Bài 11: tạo private repository ECR cho service đặt chuyến, đăng nhập và push image từ máy, bật immutable tag và scan on push với Inspector, dùng digest thay tag khi deploy, lifecycle policy dọn image cũ, pull-through cache cho image công khai.
- **Tags riêng**: `ecr`, `container`, `docker`, `image-scan`, `lifecycle-policy`
- **Nội dung bắt buộc**:
  - Registry/repository/image/tag/digest; `aws ecr get-login-password | docker login`; push image service đặt chuyến (build từ examples/toi-noi-dotnet/31-38-trip-service hoặc image minh hoạ).
  - Immutable tag + deploy theo digest; scan basic vs enhanced (Inspector), đọc finding; ký image với Signer chỉ nhắc.
  - Lifecycle policy: giữ 10 image mới nhất, xoá untagged sau 7 ngày; repository policy cho tài khoản khác pull; pull-through cache cho ECR Public/Docker Hub.
  - Chi phí storage và data transfer; VPC endpoint cho ECR (dkr, api) và S3 để Fargate pull không qua NAT.
  - Bài tập: tag latest bị đẩy đè thì chuyện gì xảy ra với task đang chạy?

### Bài 12 · ECS Fargate: cluster, task definition, service, execution role và task role

- **Slug**: `aws-ecs-fargate-task-definition-roles` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-12-ecs-fargate`
- **Tóm tắt**: Bài 12: đưa service đặt chuyến lên ECS Fargate: task definition với CPU/memory, port, log awslogs, secret từ Secrets Manager; phân biệt execution role với task role; service giữ số task mong muốn trong private subnet và kết nối RDS.
- **Tags riêng**: `ecs`, `fargate`, `task-definition`, `execution-role`, `task-role`, `container`
- **Nội dung bắt buộc**:
  - Cluster, task definition (family, revision), task, service; Fargate vs EC2 launch type; awsvpc network mode, mỗi task một ENI.
  - Execution role (pull ECR, ghi log, đọc secret lúc khởi động) khác task role (quyền runtime của app như S3 PutObject); liên hệ bài 36 loạt .NET.
  - Secret inject qua `secrets.valueFrom` ARN Secrets Manager/Parameter Store; env không chứa password; health check container.
  - Task trong private subnet, VPC endpoint hoặc NAT; ECS Exec để vào container debug; Service Connect/Cloud Map nhắc qua.
  - Lab: task definition JSON và lệnh register/create-service; bài tập: task bị STOPPED với lý do CannotPullContainerError thì sửa ở role nào?

### Bài 13 · ECS với ALB: rolling deploy, deployment circuit breaker và service auto scaling

- **Slug**: `aws-ecs-alb-rolling-deploy-autoscaling` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-13-ecs-deploy-scaling`
- **Tóm tắt**: Bài 13: gắn service ECS vào ALB target group kiểu ip, cấu hình minimumHealthyPercent và maximumPercent cho rolling update, bật deployment circuit breaker tự rollback, service auto scaling theo request/target và lịch giờ cao điểm.
- **Tags riêng**: `ecs`, `alb`, `rolling-update`, `circuit-breaker`, `auto-scaling`, `deployment`
- **Nội dung bắt buộc**:
  - Target group target type ip, health check grace period, deregistration delay và connection draining với task Fargate.
  - Rolling update: desired 4, min 50%, max 200% ⇒ thứ tự thay task; circuit breaker + rollback; CloudWatch alarm làm điều kiện rollback.
  - Service auto scaling: Application Auto Scaling target tracking ALBRequestCountPerTarget/ECSServiceAverageCPUUtilization, scheduled action 11:00 và 17:00 giờ Việt Nam, cooldown.
  - Blue/green của ECS và CodeDeploy để sang bài 26; so sánh ngắn trong bảng.
  - Lab: update service với image digest mới và quan sát events; bài tập: tính số task tối đa đồng thời khi deploy với các tham số đã cho.

### Bài 14 · Secrets Manager và Parameter Store: cấu hình, secret và rotation cho service Tới Nơi

- **Slug**: `aws-secrets-manager-parameter-store` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-14-secrets-parameters`
- **Tóm tắt**: Bài 14: chia cấu hình thường vào Parameter Store (chuẩn, phân cấp /toinoi/prod/...) và secret vào Secrets Manager; rotation tự động cho mật khẩu RDS, cache secret trong ứng dụng, quyền đọc theo resource và tag, chi phí mỗi secret và mỗi lần gọi API.
- **Tags riêng**: `secrets-manager`, `parameter-store`, `rotation`, `configuration`, `kms`
- **Nội dung bắt buộc**:
  - Parameter Store: String/SecureString, phân cấp, GetParametersByPath, standard vs advanced, miễn phí tier; Secrets Manager: JSON secret, versioning AWSCURRENT/AWSPREVIOUS, rotation bằng Lambda hoặc managed rotation cho RDS.
  - Ai đọc được gì: policy theo ARN và tag; KMS key riêng cho secret; CloudTrail ghi mọi GetSecretValue.
  - Ứng dụng đọc secret lúc khởi động và cache (Secrets Manager caching library, Parameters and Secrets Lambda extension); ECS inject như bài 12.
  - Rotation làm connection string đổi: ứng dụng reconnect thế nào, cửa sổ hai mật khẩu cùng hợp lệ.
  - Lab CLI tạo parameter và secret, đọc bằng role; bài tập: tại sao không đặt secret vào biến môi trường của task definition dạng plain?

### Bài 15 · ElastiCache: cache báo giá, session và rate limit bằng Redis hoặc Valkey

- **Slug**: `aws-elasticache-redis-cache-rate-limit` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-15-elasticache`
- **Tóm tắt**: Bài 15: dựng ElastiCache (Redis OSS hoặc Valkey) trong private subnet cho ba việc của Tới Nơi: cache báo giá theo thành phố và km, lưu session tài xế, rate limit API; so sánh node-based với Serverless, TLS và AUTH, TTL và cache stampede.
- **Tags riêng**: `elasticache`, `redis`, `valkey`, `cache`, `rate-limit`, `ttl`
- **Nội dung bắt buộc**:
  - Khi nào cache: 30.000 đọc/giây giờ cao điểm của Tới Nơi; cache-aside, TTL, key design quote:{city}:{service}:{km}; invalidation khi đổi bảng giá.
  - ElastiCache engine Redis OSS vs Valkey vs Memcached; node-based (replication group, cluster mode) vs Serverless; subnet group, security group, in-transit/at-rest encryption, AUTH/RBAC.
  - Rate limit bằng INCR + EXPIRE hoặc sliding window; session tài xế với TTL; Lua/atomic để tránh race.
  - Cache stampede, hot key, eviction policy, maxmemory; metric CurrConnections, Evictions, CacheHitRate; không coi cache là nguồn sự thật.
  - Lab: tạo cache nhỏ và test bằng redis-cli từ EC2; bài tập: TTL ngẫu nhiên hoá để làm gì?

### Bài 16 · SQS và SNS: hàng đợi, dead-letter queue và fan-out thông báo khi có chuyến mới

- **Slug**: `aws-sqs-sns-dlq-fan-out` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-16-sqs-sns`
- **Tóm tắt**: Bài 16: khi một chuyến được tạo, SNS topic fan-out tới nhiều SQS queue (gửi push cho tài xế, cập nhật thống kê, gửi email); visibility timeout, long polling, DLQ và redrive, FIFO với deduplication, idempotent consumer để không xử lý trùng.
- **Tags riêng**: `sqs`, `sns`, `dead-letter-queue`, `fan-out`, `fifo`, `messaging`
- **Nội dung bắt buộc**:
  - SQS standard vs FIFO (ordering, dedup id, group id, throughput); at-least-once ⇒ consumer idempotent (liên hệ bài outbox/inbox loạt .NET).
  - Visibility timeout, long polling WaitTimeSeconds=20, batch 10, message retention; DLQ với maxReceiveCount, redrive to source.
  - SNS topic, subscription SQS/Lambda/HTTPS/email, message filtering theo attribute (city, service), raw message delivery; access policy cho SNS gửi vào SQS.
  - Sơ đồ fan-out BookingCreated; so sánh SQS/SNS với EventBridge (bài 17) và MSK (bài 34) trong một bảng.
  - Lab CLI: tạo topic + 2 queue + DLQ, publish và receive; bài tập: tin nhắn bị xử lý hai lần do đâu và chặn thế nào?

### Bài 17 · EventBridge: event bus, rule, schedule và sự kiện chuyến của Tới Nơi

- **Slug**: `aws-eventbridge-event-bus-rule-schedule` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-17-eventbridge`
- **Tóm tắt**: Bài 17: đưa sự kiện TripCompleted và TripCancelled lên custom event bus, viết rule với event pattern để định tuyến tới SQS, Lambda hoặc API destination, EventBridge Scheduler cho job đối soát 02:00, schema registry, archive và replay sự kiện.
- **Tags riêng**: `eventbridge`, `event-bus`, `event-pattern`, `scheduler`, `event-driven`
- **Nội dung bắt buộc**:
  - Default bus (sự kiện AWS) vs custom bus (sự kiện Tới Nơi) vs partner bus; envelope sự kiện: source, detail-type, detail.
  - Event pattern: match theo detail.city, prefix, numeric; input transformer; target SQS/Lambda/Step Functions/API destination với auth; retry policy và DLQ của rule.
  - EventBridge Scheduler (khác rule schedule): cron theo múi giờ Asia/Ho_Chi_Minh, flexible window, one-time schedule.
  - Schema registry và discovery; archive + replay để xử lý lại 1 giờ sự kiện; Pipes nối SQS → enrichment → bus.
  - Lab: put-events và rule tới SQS; bài tập: khi nào chọn EventBridge thay SNS?

### Bài 18 · Lambda: hàm xử lý webhook thanh toán, trigger từ S3 và SQS, cold start và quyền

- **Slug**: `aws-lambda-webhook-thanh-toan-cold-start` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-18-lambda-webhook`
- **Tóm tắt**: Bài 18: viết Lambda nhận webhook từ cổng thanh toán VíXanh (giả lập), xác minh chữ ký, ghi SQS; trigger từ S3 khi có ảnh giao hàng và từ SQS với batch; execution role, biến môi trường, timeout, memory, concurrency, cold start và SnapStart; Lambda trong VPC.
- **Tags riêng**: `lambda`, `serverless`, `webhook`, `cold-start`, `event-source-mapping`, `concurrency`
- **Nội dung bắt buộc**:
  - Mô hình Lambda: handler, runtime (Node.js/Python/.NET), invocation sync/async/poll, execution environment tái dùng, cold start; memory quyết định CPU.
  - Execution role tối thiểu, biến môi trường + Secrets, timeout, reserved/provisioned concurrency, SnapStart cho runtime hỗ trợ.
  - Trigger: Function URL hoặc API Gateway cho webhook, S3 event notification, SQS event source mapping với batch size, partial batch response, DLQ/destination.
  - Lambda trong VPC để tới RDS: ENI, cần NAT hoặc VPC endpoint để gọi AWS API; RDS Proxy nhắc tới bài 31.
  - Lab: code handler Node.js ngắn + deploy bằng CLI zip; bài tập: webhook gửi lại 3 lần thì hệ thống phải làm gì?

### Bài 19 · API Gateway: HTTP API, JWT authorizer, throttling và WebSocket theo dõi chuyến

- **Slug**: `aws-api-gateway-authorizer-websocket` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-19-api-gateway`
- **Tóm tắt**: Bài 19: đặt API Gateway trước Lambda và ALB nội bộ: HTTP API vs REST API, JWT authorizer với identity provider của Tới Nơi, usage plan và throttling, stage và custom domain; WebSocket API để khách xem vị trí tài xế theo thời gian thực.
- **Tags riêng**: `api-gateway`, `http-api`, `jwt-authorizer`, `throttling`, `websocket`
- **Nội dung bắt buộc**:
  - HTTP API (rẻ, nhanh, JWT authorizer, VPC link tới ALB) vs REST API (usage plan, API key, request validation, WAF, caching); chọn cho từng BFF của Tới Nơi.
  - JWT authorizer với issuer/audience; Lambda authorizer khi cần logic; IAM auth cho nội bộ; CORS.
  - Throttling theo stage/route, burst và rate, usage plan + API key cho đối tác quán; access log JSON vào CloudWatch; custom domain + ACM.
  - WebSocket API: $connect/$disconnect/$default route, connection id lưu DynamoDB, post_to_connection đẩy vị trí; khi nào dùng AppSync Events/IoT thay vì tự dựng.
  - Lab: HTTP API tới Lambda bài 18; bài tập: 429 từ API Gateway và từ service khác nhau ở đâu?

### Bài 20 · CloudFront, WAF và Shield: CDN và lớp chắn cho web và API của Tới Nơi

- **Slug**: `aws-cloudfront-waf-shield` · **Phần**: 2 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p02-20-cloudfront-waf`
- **Tóm tắt**: Bài 20: CloudFront distribution cho web tĩnh trên S3 (Origin Access Control) và API qua ALB, cache policy và origin request policy, CloudFront Functions chuyển hướng, WAF với managed rule group, rate-based rule chống brute force đăng nhập, Shield Standard và khi nào cần Advanced.
- **Tags riêng**: `cloudfront`, `waf`, `shield`, `cdn`, `origin-access-control`, `ddos`
- **Nội dung bắt buộc**:
  - Distribution, origin (S3 với OAC, ALB, VPC origin), behavior theo path, cache policy/origin request policy/response headers policy, TTL và invalidation; ACM phải ở us-east-1.
  - CloudFront Functions vs Lambda@Edge: viết redirect và header bảo mật; signed URL/cookie cho nội dung riêng tư thay presigned S3 khi cần CDN.
  - WAF web ACL: managed rule group (Core, Known Bad Inputs, SQLi), rate-based rule trên /auth/login, geo match, custom response, log vào S3/Firehose; gắn WAF vào CloudFront, ALB hoặc API Gateway.
  - Shield Standard mặc định, Shield Advanced và DDoS response team; nguyên tắc ALB chỉ nhận từ CloudFront (prefix list + custom header).
  - Lab: CloudFront trước S3 web tĩnh; bài tập: cache-control nào khiến API bị cache nhầm?

### Bài 21 · CloudFormation: template, stack, change set, drift và vì sao hạ tầng phải là code

- **Slug**: `aws-cloudformation-stack-change-set-drift` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-21-cloudformation`
- **Tóm tắt**: Bài 21: viết template CloudFormation YAML cho VPC và S3 của Tới Nơi với Parameters, Mappings, Conditions, Outputs và intrinsic function; tạo stack, xem change set trước khi áp dụng, phát hiện drift, stack policy và deletion policy để không mất dữ liệu.
- **Tags riêng**: `cloudformation`, `iac`, `change-set`, `drift`, `stack`
- **Nội dung bắt buộc**:
  - Vì sao IaC: lặp lại, review, nhiều môi trường, audit; so sánh click console ở phần 01 với template.
  - Template anatomy: Parameters, Mappings, Conditions, Resources, Outputs, Ref/GetAtt/Sub/ImportValue; nested stack và cross-stack export.
  - Stack lifecycle: create/update/delete, change set, rollback khi lỗi, UPDATE_ROLLBACK_FAILED và cách thoát; DeletionPolicy Retain/Snapshot cho RDS/S3; stack policy.
  - Drift detection và vì sao sửa tay là nguồn gốc sự cố; StackSets cho nhiều tài khoản/region nhắc qua; cfn-lint.
  - Lab: template vpc.yaml + deploy bằng `aws cloudformation deploy`; bài tập: đổi tên logical ID của RDS thì chuyện gì xảy ra?

### Bài 22 · Terraform trên AWS: provider, resource, remote state trên S3 và lock

- **Slug**: `aws-terraform-provider-state-s3-lock` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-22-terraform-basics`
- **Tóm tắt**: Bài 22: cài Terraform, cấu hình provider AWS với profile SSO, viết resource VPC và S3 cho Tới Nơi, hiểu state và vì sao state không nằm trên máy cá nhân: backend S3 với use_lockfile, mã hoá, versioning; plan, apply, import và moved block.
- **Tags riêng**: `terraform`, `iac`, `remote-state`, `s3-backend`, `provider`
- **Nội dung bắt buộc**:
  - HCL: provider, resource, data, variable, output, locals; `terraform init/plan/apply/destroy`; lockfile .terraform.lock.hcl và pin version provider.
  - State: ánh xạ resource ↔ ID thật; chứa secret ⇒ mã hoá; backend S3 với `use_lockfile = true` (Terraform ≥ 1.10, thay DynamoDB lock), versioning, bucket policy chỉ role CI; state per môi trường.
  - Vòng đời đổi code: plan đọc kỹ (+/-/~/-+ replace), lifecycle prevent_destroy cho RDS/S3, moved/import block khi đổi tên hoặc nhận tài nguyên có sẵn.
  - So sánh CloudFormation/Terraform/OpenTofu/CDK trong bảng, tiêu chí chọn; Tới Nơi chọn Terraform theo tài liệu kiến trúc §16.
  - Lab: examples/toi-noi-aws/p03-22-terraform-basics với backend; kiểm bằng `terraform fmt -check` và `validate`; bài tập: hai người apply cùng lúc không có lock thì sao?

### Bài 23 · Terraform module: dựng VPC, ECS Fargate và RDS cho Tới Nơi theo từng môi trường

- **Slug**: `aws-terraform-module-vpc-ecs-rds` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-23-terraform-modules`
- **Tóm tắt**: Bài 23: tổ chức repo Terraform theo module network, database, service và thư mục môi trường dev/staging/prod; input/output, for_each, dynamic block, data source; module registry chính thức vs tự viết; cách thêm một service mới mà không copy-paste.
- **Tags riêng**: `terraform`, `module`, `ecs`, `rds`, `vpc`, `environments`
- **Nội dung bắt buộc**:
  - Cấu trúc: modules/{network,ecs-service,rds-postgres}, envs/{dev,staging,prod} với backend và tfvars riêng; không dùng workspace cho prod/dev khi khác tài khoản.
  - Module interface: variable có validation, output ARN/ID, versioning module qua git tag; dùng terraform-aws-modules/vpc khi hợp lý và khi nào tự viết.
  - for_each cho nhiều service (quote, booking, dispatch), dynamic block cho security group rule, data aws_caller_identity/aws_region, depends_on hiếm khi cần.
  - Ghép: network → rds → ecs-service với secret ARN; secret master password do RDS quản lý, không viết vào tfvars.
  - Lab: examples/toi-noi-aws/p03-23-terraform-modules validate được; bài tập: thay đổi nào gây replace RDS và chặn thế nào?

### Bài 24 · AWS CDK: construct, stack, synth và diff bằng TypeScript cho service báo giá

- **Slug**: `aws-cdk-construct-synth-diff` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-24-cdk`
- **Tóm tắt**: Bài 24: viết CDK app TypeScript dựng ECS Fargate service với ALB bằng L2/L3 construct, cdk synth ra CloudFormation, cdk diff trước deploy, bootstrap và quyền của role CDK, aspects và cdk-nag để kiểm tuân thủ; khi nào CDK hợp hơn Terraform.
- **Tags riêng**: `cdk`, `typescript`, `iac`, `construct`, `cloudformation`
- **Nội dung bắt buộc**:
  - CDK v2: app → stack → construct L1/L2/L3; `cdk bootstrap`, `synth`, `diff`, `deploy`; asset (Docker image, Lambda code) và bucket bootstrap.
  - Ví dụ: ApplicationLoadBalancedFargateService cho quote service với secret, log, autoscaling; props an toàn mặc định (removalPolicy, encryption).
  - Aspects và cdk-nag (AwsSolutions pack) để chặn S3 public, DB không mã hoá; snapshot test với assertions.
  - Bootstrap role và trust giữa tài khoản; context và environment; CDK Pipelines nhắc; so sánh với Terraform CDK (CDKTF) và Pulumi.
  - Lab: examples/toi-noi-aws/p03-24-cdk với `npx cdk synth`; bài tập: đổi logical ID do refactor construct gây replace resource thế nào?

### Bài 25 · GitHub Actions và OIDC: assume role không cần access key, build, đẩy ECR và deploy ECS

- **Slug**: `aws-github-actions-oidc-ecr-deploy` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-25-github-actions-oidc`
- **Tóm tắt**: Bài 25: cấu hình IAM OIDC identity provider cho GitHub, trust policy giới hạn repo và branch, workflow build image, push ECR theo digest và cập nhật task definition ECS; quyền tối thiểu cho role CI, environment protection và approval cho prod.
- **Tags riêng**: `github-actions`, `oidc`, `ci-cd`, `ecr`, `ecs`, `least-privilege`
- **Nội dung bắt buộc**:
  - Vì sao không dùng access key dài hạn trong secret CI; OIDC provider token.actions.githubusercontent.com, audience sts.amazonaws.com, condition sub `repo:org/toi-noi:ref:refs/heads/main` hoặc environment.
  - Workflow: configure-aws-credentials (pin version), docker build, push, lấy digest, render task definition, deploy service và wait-for-service-stability; cache layer.
  - Role CI tách build (ECR push) và deploy (ECS UpdateService, PassRole chỉ cho execution/task role cụ thể).
  - Environment prod với required reviewers; concurrency group tránh hai deploy chồng; artifact và SBOM nhắc.
  - Lab: workflow YAML trong examples/toi-noi-aws/p03-25-github-actions-oidc (không chạy thật trong repo CI); bài tập: trust policy dùng `repo:org/*` có rủi ro gì?

### Bài 26 · CodePipeline, CodeBuild và CodeDeploy: blue/green cho ECS với traffic shifting

- **Slug**: `aws-codepipeline-codebuild-codedeploy-blue-green` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-26-codedeploy-blue-green`
- **Tóm tắt**: Bài 26: pipeline native AWS: source từ GitHub qua CodeConnections, CodeBuild với buildspec và quyền ECR, CodeDeploy ECS blue/green với hai target group, listener test, traffic shifting linear/canary, hook Lambda kiểm smoke test và rollback theo alarm; so với blue/green tích hợp sẵn của ECS.
- **Tags riêng**: `codepipeline`, `codebuild`, `codedeploy`, `blue-green`, `ecs`, `ci-cd`
- **Nội dung bắt buộc**:
  - CodeConnections (GitHub app) thay CodeCommit (không còn cho khách mới); pipeline stage/action, artifact S3 mã hoá; CodeBuild buildspec, compute type, cache, privileged mode cho docker.
  - CodeDeploy ECS: appspec.yaml, taskdef, hai target group, production và test listener, deployment config Linear10PercentEvery1Minute/Canary; lifecycle hook AfterAllowTestTraffic gọi Lambda smoke test; rollback theo CloudWatch alarm.
  - ECS blue/green tích hợp (2025) vs CodeDeploy: bảng so sánh, khi nào chọn gì.
  - Approval action thủ công trước prod; EventBridge notification tới Slack/SNS; pipeline per service hay monorepo.
  - Lab: buildspec + appspec trong examples/toi-noi-aws/p03-26-codedeploy-blue-green; bài tập: khi canary 10% lỗi 5xx thì ai rollback và dữ liệu ghi bởi bản xanh đi đâu?

### Bài 27 · Chiến lược triển khai: rolling, blue/green, canary, rollback và cổng migration database

- **Slug**: `aws-chien-luoc-deploy-rolling-canary-rollback` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: không
- **Tóm tắt**: Bài 27: so sánh rolling, blue/green, canary và feature flag theo rủi ro, chi phí và thời gian rollback; expand/contract migration để schema tương thích hai bản; cổng kiểm trước khi chuyển traffic; runbook rollback cho service đặt chuyến khi 5xx tăng sau deploy.
- **Tags riêng**: `deployment-strategy`, `canary`, `blue-green`, `rollback`, `database-migration`, `release`
- **Nội dung bắt buộc**:
  - Bảng so sánh chiến lược: số bản cùng chạy, chi phí hạ tầng, thời gian rollback, yêu cầu tương thích API/DB, phù hợp service nào của Tới Nơi.
  - Expand/contract: thêm cột nullable → deploy code ghi cả hai → backfill → đổi đọc → xoá cột; không rollback migration phá dữ liệu; liên hệ bài 35/36 loạt .NET.
  - Cổng trước khi shift traffic: readiness, smoke test có xác thực, synthetic canary (CloudWatch Synthetics), alarm 5xx/latency/DB; bake time.
  - Rollback là quyết định có điều kiện: image cũ + schema mới vẫn chạy nhờ expand/contract; runbook từng bước và ai được quyết.
  - Không có lab tốn tiền; bài tập: thiết kế deploy cho thay đổi đổi kiểu cột price từ integer sang numeric.

### Bài 28 · Organizations, SCP và Control Tower: nhiều tài khoản cho dev, staging, prod và bảo mật

- **Slug**: `aws-organizations-scp-control-tower-landing-zone` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: không
- **Tóm tắt**: Bài 28: vì sao Tới Nơi cần nhiều tài khoản thay vì một tài khoản nhiều VPC; Organizations với OU, SCP chặn region và dịch vụ, Resource Control Policy; Control Tower dựng landing zone với log archive và audit account; truy cập chéo tài khoản bằng Identity Center và role.
- **Tags riêng**: `organizations`, `scp`, `control-tower`, `landing-zone`, `multi-account`
- **Nội dung bắt buộc**:
  - Giới hạn của một tài khoản: blast radius, quota, hoá đơn, IAM rối; mô hình OU: Security (log archive, audit), Infrastructure (network, shared services), Workloads (dev/staging/prod per service group), Sandbox.
  - SCP: guardrail chặn region ngoài ap-southeast-1/us-east-1, chặn tắt CloudTrail/GuardDuty, chặn root; RCP cho S3/KMS/STS; SCP không cấp quyền.
  - Control Tower: landing zone, controls preventive/detective/proactive, Account Factory; AWS Config aggregator và CloudTrail org trail vào log archive.
  - Cross-account: Identity Center permission set vào từng tài khoản, role assume từ CI, RAM chia sẻ subnet/Transit Gateway; consolidated billing và cost allocation.
  - Không lab (cần nhiều tài khoản); bài tập: SCP deny s3:PutObject nếu không mã hoá đặt ở OU nào?

### Bài 29 · AppConfig và feature flag: đổi bảng giá và bật tính năng mới không cần deploy

- **Slug**: `aws-appconfig-feature-flag-cau-hinh-dong` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-29-appconfig`
- **Tóm tắt**: Bài 29: AppConfig application, environment, configuration profile dạng feature flag và freeform, deployment strategy có bake time và rollback theo alarm, validator JSON Schema/Lambda, AppConfig Agent cho ECS/Lambda; dùng flag để bật giá giờ cao điểm mới cho 5% khách ở một thành phố.
- **Tags riêng**: `appconfig`, `feature-flag`, `configuration`, `progressive-delivery`, `parameter-store`
- **Nội dung bắt buộc**:
  - Khác biệt: Parameter Store (giá trị), Secrets Manager (bí mật), AppConfig (cấu hình có deploy, validate, rollback); khi nào cần AppConfig.
  - Feature flag có attribute và variant; targeting theo thành phố/percentage; freeform config cho bảng giá; validator chặn giá âm.
  - Deployment strategy: growth linear/exponential, bake time, CloudWatch alarm rollback; environment dev/prod; hosted config vs S3/SSM source.
  - Lấy cấu hình: AppConfig Agent sidecar/extension, poll interval, cache; không gọi API mỗi request; quyền appconfig:GetLatestConfiguration.
  - Lab: tạo flag bằng CLI và đọc từ Lambda bài 18; bài tập: flag khác A/B test ở đâu?

### Bài 30 · Policy as code trong pipeline: cfn-guard, Checkov, tflint, Infracost và kiểm drift

- **Slug**: `aws-policy-as-code-checkov-infracost-drift` · **Phần**: 3 · **Cấp độ**: intermediate · **Lab**: `examples/toi-noi-aws/p03-30-policy-as-code`
- **Tóm tắt**: Bài 30: thêm cổng kiểm hạ tầng vào pipeline Terraform/CloudFormation của Tới Nơi: fmt và validate, tflint, Checkov hoặc cfn-guard với rule riêng (S3 phải mã hoá, RDS không public, tag bắt buộc), Infracost ước chi phí trong pull request, plan có review, kiểm drift định kỳ bằng Config và terraform plan.
- **Tags riêng**: `policy-as-code`, `checkov`, `cfn-guard`, `infracost`, `tflint`, `drift`, `ci-cd`
- **Nội dung bắt buộc**:
  - Pipeline IaC: fmt → validate → lint → policy → plan → review → apply; plan artifact và apply đúng plan đã review; least privilege cho role plan (read) vs apply (write).
  - Checkov/tfsec/cfn-guard: rule có sẵn và rule riêng cho Tới Nơi (tag Project/Env, không 0.0.0.0/0 port 5432, log bật); suppress có lý do và hạn.
  - Infracost diff trong PR; AWS Config rules và conformance pack để phát hiện drift sau apply; terraform plan -detailed-exitcode theo lịch.
  - OPA/Rego và Sentinel nhắc; proactive control của Control Tower liên hệ bài 28.
  - Lab: workflow + rule mẫu trong examples/toi-noi-aws/p03-30-policy-as-code chạy được trên máy với Docker; bài tập: rule nào bắt được RDS publicly_accessible=true?

### Bài 31 · Aurora PostgreSQL và RDS Proxy: reader endpoint, failover, Serverless v2 và PITR

- **Slug**: `aws-aurora-postgresql-rds-proxy-failover` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-31-aurora`
- **Tóm tắt**: Bài 31: Aurora khác RDS ở lớp lưu trữ phân tán 6 bản sao 3 AZ; cluster endpoint, reader endpoint, custom endpoint cho báo cáo; failover và thời gian; Aurora Serverless v2 cho dev; RDS Proxy gom kết nối từ Lambda và ECS; Blue/Green deployment nâng phiên bản; PITR, clone và diễn tập restore.
- **Tags riêng**: `aurora`, `postgresql`, `rds-proxy`, `failover`, `serverless-v2`, `pitr`
- **Nội dung bắt buộc**:
  - Kiến trúc Aurora: storage tách compute, 6 copy/3 AZ, writer + tới 15 reader, replication lag thấp; so với RDS Multi-AZ; chi phí I/O-Optimized vs Standard.
  - Endpoint: cluster (writer), reader (round-robin DNS), custom (reporting readers), instance; ứng dụng tách read/write (liên hệ CQRS loạt .NET); failover priority tier.
  - RDS Proxy: pool, pinning, IAM auth, tránh cạn max_connections khi Lambda bùng; TLS.
  - Aurora Serverless v2 ACU, scale to zero; Blue/Green deployments để nâng major version; Global Database nhắc tới bài 56; PITR, fast clone cho test, restore drill với RTO đo được.
  - Lab: Terraform cluster + proxy trong examples/toi-noi-aws/p04-31-aurora (validate); bài tập: query báo cáo tháng chạy ở endpoint nào và vì sao?

### Bài 32 · DynamoDB: single-table design cho vị trí tài xế và trạng thái chuyến, GSI, TTL và Streams

- **Slug**: `aws-dynamodb-single-table-gsi-ttl-streams` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-32-dynamodb`
- **Tóm tắt**: Bài 32: khi 6.250 toạ độ mỗi giây không nên vào PostgreSQL: mô hình DynamoDB với partition key và sort key, access pattern trước schema, GSI cho tra cứu theo thành phố, TTL xoá vị trí cũ, on-demand vs provisioned, transaction và conditional write, Streams kích hoạt Lambda.
- **Tags riêng**: `dynamodb`, `nosql`, `single-table`, `gsi`, `ttl`, `streams`
- **Nội dung bắt buộc**:
  - Mô hình: table, item, PK/SK, partition và hot partition, item 400 KB, consistency eventual/strong; capacity on-demand vs provisioned và warm throughput.
  - Access pattern của Tới Nơi: vị trí mới nhất của tài xế, lịch sử 10 phút, tài xế đang rảnh trong thành phố; thiết kế key PK=DRIVER#id SK=TS#..., GSI1 PK=CITY#hcm#AVAILABLE; TTL 24h.
  - Conditional write cho nhận chuyến (chỉ một tài xế thắng), TransactWriteItems, idempotency token; Streams + Lambda cập nhật thống kê; DAX và Global Tables nhắc.
  - Chi phí theo RCU/WCU, item size; khi DynamoDB không hợp (join, báo cáo ad hoc) ⇒ export to S3 + Athena (bài 38).
  - Lab: tạo bảng và put/query bằng CLI; bài tập: hot partition do PK=CITY, sửa thế nào?

### Bài 33 · ElastiCache nâng cao và MemoryDB: HA, failover, cluster mode, eviction và pub/sub

- **Slug**: `aws-elasticache-nang-cao-memorydb-ha` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-33-elasticache-ha`
- **Tóm tắt**: Bài 33: khi cache trở thành phụ thuộc quan trọng: replication group với Multi-AZ và auto failover, cluster mode với slot và resharding, eviction policy và memory, Global Datastore, MemoryDB khi cần durability cho dữ liệu dispatch, pub/sub và stream cho realtime, metric và sự cố thường gặp.
- **Tags riêng**: `elasticache`, `memorydb`, `redis`, `valkey`, `high-availability`, `cluster-mode`
- **Nội dung bắt buộc**:
  - Replication group: primary + replica, Multi-AZ, auto failover, thời gian và tác động tới client (reconnect, reader endpoint); backup/snapshot.
  - Cluster mode enabled: 16384 slot, shard, resharding online, key hash tag {trip:42}; client phải hỗ trợ cluster; Serverless tự scale.
  - Eviction policy allkeys-lru vs volatile-ttl, maxmemory-policy, reserved memory, swap; hot key và big key; pipeline/mget.
  - MemoryDB: Multi-AZ transaction log, durable ⇒ dùng làm primary store cho trạng thái dispatch; so sánh chi phí/latency với ElastiCache; pub/sub và Streams cho vị trí realtime, Global Datastore nhắc.
  - Lab: Terraform replication group trong examples/toi-noi-aws/p04-33-elasticache-ha; bài tập: failover 30 giây thì rate limit ở bài 15 hỏng thế nào?

### Bài 34 · Amazon MSK: Kafka managed, IAM auth, MSK Connect và outbox của Tới Nơi

- **Slug**: `aws-msk-kafka-iam-auth-msk-connect` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-34-msk`
- **Tóm tắt**: Bài 34: khi kiến trúc Tới Nơi cần Kafka: MSK Provisioned với broker 3 AZ, Express broker và Serverless; topic, partition, replication factor, min.insync.replicas, acks=all; xác thực IAM và ACL; MSK Connect chạy Debezium đọc outbox PostgreSQL; consumer lag, retention và chi phí.
- **Tags riêng**: `msk`, `kafka`, `event-driven`, `outbox`, `debezium`, `msk-connect`
- **Nội dung bắt buộc**:
  - Khi nào Kafka thay SQS/EventBridge: replay, nhiều consumer group, ordering theo key, throughput; liên hệ tài liệu kiến trúc §6 và bài outbox loạt .NET.
  - MSK Provisioned (broker type, storage, 3 AZ), Express broker, Serverless; cấu hình topic: partition theo tripId, RF=3, min.insync.replicas=2, acks=all, retention.
  - Bảo mật: IAM access control (SASL/IAM) với policy theo cluster/topic/group, TLS, private only; MSK không có public endpoint mặc định.
  - MSK Connect + Debezium PostgreSQL connector đọc bảng outbox (logical replication, publication, slot); schema registry Glue; consumer trên ECS/EKS/Lambda event source mapping.
  - Giám sát: consumer lag, UnderReplicatedPartitions, disk; Lab: Terraform cluster tối thiểu trong examples/toi-noi-aws/p04-34-msk (validate); bài tập: partition theo cityId thay tripId gây gì?

### Bài 35 · Kinesis Data Streams và Data Firehose: nuốt 6.250 toạ độ GPS mỗi giây

- **Slug**: `aws-kinesis-data-streams-firehose-gps` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-35-kinesis`
- **Tóm tắt**: Bài 35: pipeline vị trí tài xế: Kinesis Data Streams với shard hoặc on-demand, partition key theo driverId, producer KPL/SDK và PutRecords, consumer Lambda với enhanced fan-out, Data Firehose ghi Parquet vào S3 theo giờ; so với MSK, SQS; retention, replay và chi phí.
- **Tags riêng**: `kinesis`, `data-streams`, `firehose`, `streaming`, `gps`, `parquet`
- **Nội dung bắt buộc**:
  - Bài toán: 25.000 tài xế × 1 vị trí/4 giây; yêu cầu realtime (dispatch) và lưu lịch sử (phân tích); kiến trúc stream → Lambda cập nhật DynamoDB bài 32 → Firehose → S3.
  - Data Streams: shard 1 MB/s in, 2 MB/s out, on-demand mode, partition key và hot shard, retention 24h–365 ngày, enhanced fan-out, checkpoint; Lambda event source mapping batch window, parallelization factor, bisect on error.
  - Amazon Data Firehose: buffer theo size/time, chuyển đổi Parquet với Glue schema, dynamic partitioning theo city/date, lỗi vào error prefix; Firehose từ MSK/Streams.
  - So sánh Kinesis vs MSK vs SQS trong bảng theo ordering, replay, throughput, vận hành, chi phí.
  - Lab: stream + Firehose bằng CLI/Terraform (p04-35-kinesis); bài tập: tăng gấp đôi tài xế thì cần bao nhiêu shard?

### Bài 36 · S3 nâng cao: storage class, replication, Object Lock, event notification và S3 Tables

- **Slug**: `aws-s3-nang-cao-storage-class-replication-object-lock` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-36-s3-advanced`
- **Tóm tắt**: Bài 36: tối ưu kho 730 triệu chứng từ và log mỗi năm: chọn storage class theo pattern truy cập, Intelligent-Tiering, Cross-Region Replication và Same-Region Replication, Object Lock cho hoá đơn phải giữ 10 năm, event notification tới EventBridge, S3 Batch Operations, Access Points, S3 Tables cho Iceberg.
- **Tags riêng**: `s3`, `storage-class`, `replication`, `object-lock`, `s3-tables`, `event-notification`
- **Nội dung bắt buộc**:
  - Storage class so sánh: Standard, IA, One Zone-IA, Intelligent-Tiering, Glacier Instant/Flexible/Deep Archive, Express One Zone; chi phí lưu vs truy xuất vs retrieval time; lifecycle với filter tag/prefix/size.
  - Replication CRR/SRR: versioning bắt buộc, role, RTC, replica modification sync, batch replication cho object cũ; dùng cho DR hoá đơn.
  - Object Lock governance/compliance, retention và legal hold; không xoá được kể cả root; S3 Inventory và Storage Lens.
  - Event notification → EventBridge (filter, nhiều target) vs SQS/SNS/Lambda trực tiếp; S3 Batch Operations; Access Point và VPC-only; S3 Tables (Iceberg) để Athena query trực tiếp.
  - Lab: lifecycle + replication Terraform (p04-36-s3-advanced); bài tập: Deep Archive có phù hợp cho ảnh chứng từ khiếu nại không?

### Bài 37 · AWS Backup và disaster recovery: RPO, RTO, pilot light, warm standby và diễn tập restore

- **Slug**: `aws-backup-disaster-recovery-rpo-rto` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-37-backup`
- **Tóm tắt**: Bài 37: xác định RPO/RTO cho từng dữ liệu của Tới Nơi (trips, ví, vị trí), AWS Backup với backup plan, vault, cross-region và cross-account copy, vault lock; bốn chiến lược DR từ backup-restore tới multi-site active/active; Elastic Disaster Recovery; runbook và game day restore có đo thời gian.
- **Tags riêng**: `aws-backup`, `disaster-recovery`, `rpo-rto`, `pilot-light`, `warm-standby`, `restore`
- **Nội dung bắt buộc**:
  - RPO/RTO theo dữ liệu: ví và giao dịch (RPO ≈ 0), trips (phút), vị trí (có thể mất); chi phí tăng theo mức DR; ma trận quyết định.
  - AWS Backup: backup plan, rule, vault, tag-based selection, cross-region/cross-account copy, vault lock (WORM), restore testing plan tự động; hỗ trợ RDS/Aurora/DynamoDB/EBS/EFS/S3.
  - Bốn chiến lược: backup & restore, pilot light, warm standby, multi-site; cần gì ở region thứ hai (IaC, image, secret, DNS); Route 53 ARC nhắc tới bài 56.
  - Elastic Disaster Recovery cho EC2/on-prem; diễn tập restore: thời gian thực tế, kiểm toàn vẹn, ai làm; backup không kiểm = không có backup.
  - Lab: backup plan Terraform (p04-37-backup); bài tập: snapshot RDS cùng region có cứu được khi mất region không?

### Bài 38 · Glue, Athena và Redshift Serverless: lakehouse phân tích 2 triệu chuyến mỗi ngày

- **Slug**: `aws-glue-athena-redshift-lakehouse` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-38-athena`
- **Tóm tắt**: Bài 38: đưa dữ liệu chuyến từ Aurora, DynamoDB và Kinesis vào S3 dạng Parquet/Iceberg, Glue Data Catalog và crawler, Glue ETL hoặc zero-ETL, Athena truy vấn theo partition, Redshift Serverless cho dashboard nặng, Lake Formation phân quyền cột PII; chi phí quét dữ liệu.
- **Tags riêng**: `glue`, `athena`, `redshift`, `lakehouse`, `iceberg`, `lake-formation`, `analytics`
- **Nội dung bắt buộc**:
  - Vì sao không chạy báo cáo trên Aurora writer: tách OLTP/OLAP; kiến trúc lake: raw/curated/serving, Parquet, partition theo dt=YYYY-MM-DD/city.
  - Glue Data Catalog, crawler, Glue ETL (Spark) vs Glue DataBrew; zero-ETL Aurora → Redshift; DynamoDB export to S3; Iceberg table và S3 Tables cho upsert/time travel.
  - Athena: CTAS, partition projection, cost per TB scanned, workgroup limit; ví dụ query doanh thu theo thành phố/giờ cao điểm.
  - Redshift Serverless RPU, Spectrum đọc S3, materialized view cho dashboard; QuickSight nhắc; Lake Formation column-level permission che số điện thoại khách.
  - Lab: Athena table trên Parquet mẫu (p04-38-athena); bài tập: query không có partition filter tốn bao nhiêu với 1 năm dữ liệu?

### Bài 39 · OpenSearch Service: tìm quán và địa chỉ, geo query và phân tích log tập trung

- **Slug**: `aws-opensearch-tim-kiem-log-analytics` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-39-opensearch`
- **Tóm tắt**: Bài 39: hai việc cho OpenSearch ở Tới Nơi: tìm kiếm quán theo tên có dấu/không dấu với geo_distance, và log analytics từ CloudWatch/Firehose; domain vs Serverless, index, mapping, analyzer tiếng Việt, shard và replica, UltraWarm và ISM xoá index cũ, fine-grained access control, chi phí.
- **Tags riêng**: `opensearch`, `search`, `geo-query`, `log-analytics`, `index-state-management`
- **Nội dung bắt buộc**:
  - Khi nào cần search engine thay LIKE/pg_trgm trong PostgreSQL; index, document, mapping, analyzer (asciifolding cho tiếng Việt không dấu), geo_point và geo_distance 3 km.
  - Domain managed (instance, EBS, 3 AZ, dedicated master) vs OpenSearch Serverless (collection search/time-series, OCU); VPC access, fine-grained access control, SAML/Cognito cho Dashboards.
  - Đồng bộ dữ liệu quán từ Aurora qua DMS/Lambda/OpenSearch Ingestion; index alias và reindex không downtime.
  - Log analytics: Firehose/CloudWatch subscription → OpenSearch, index theo ngày, ISM hot → UltraWarm → delete sau 30 ngày; so với CloudWatch Logs Insights (bài 55).
  - Lab: domain nhỏ hoặc Serverless collection bằng Terraform (p04-39-opensearch); bài tập: shard quá nhiều gây gì?

### Bài 40 · DMS và cắt chuyển gần như không downtime: từ PostgreSQL tự vận hành sang Aurora

- **Slug**: `aws-dms-migration-postgresql-zero-downtime` · **Phần**: 4 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p04-40-dms`
- **Tóm tắt**: Bài 40: kế hoạch chuyển database trips 730 triệu dòng từ PostgreSQL trên EC2 sang Aurora: đánh giá bằng DMS Schema Conversion và Fleet Advisor, replication instance hoặc DMS Serverless, full load + CDC với logical replication, kiểm đối soát dữ liệu, cutover với Route 53/connection string, rollback; so với pg_dump, native logical replication và Blue/Green.
- **Tags riêng**: `dms`, `migration`, `postgresql`, `aurora`, `cdc`, `cutover`
- **Nội dung bắt buộc**:
  - Chiến lược di chuyển: rehost/replatform/refactor; chọn homogeneous PostgreSQL → Aurora PostgreSQL; khi nào pg_dump/pg_restore đủ, khi nào cần CDC.
  - DMS: source/target endpoint, replication instance vs Serverless, task full-load + CDC, table mapping/transformation, LOB mode, giới hạn (sequence, DDL); yêu cầu wal_level=logical, slot.
  - Kiểm tra: DMS validation, đếm dòng/checksum theo partition, so sánh query mẫu; theo dõi CDC latency.
  - Cutover runbook: freeze ghi ngắn, đợi lag=0, đổi secret/endpoint, bật ghi, giữ replication ngược để rollback; diễn tập trên staging.
  - Lab: task definition DMS JSON và Terraform (p04-40-dms); bài tập: sequence bị lệch sau migration xử lý thế nào?

### Bài 41 · EKS tổng quan: control plane, managed node group, Fargate profile, Auto Mode và Pod Identity

- **Slug**: `aws-eks-tong-quan-control-plane-node-group` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: không
- **Tóm tắt**: Bài 41: EKS quản lý control plane, bạn quản lý gì ở data plane: managed node group, self-managed, Fargate profile, EKS Auto Mode; authentication bằng access entries, Pod Identity thay IRSA để pod gọi AWS API; khi nào Tới Nơi cần Kubernetes thay ECS và cái giá vận hành.
- **Tags riêng**: `eks`, `kubernetes`, `control-plane`, `node-group`, `pod-identity`, `auto-mode`
- **Nội dung bắt buộc**:
  - Kubernetes tối thiểu cần biết: cluster, node, pod, deployment, service, namespace; EKS quản lý API server/etcd nhiều AZ, phiên bản và chu kỳ hỗ trợ (standard/extended).
  - Data plane: managed node group (AMI, launch template, update), self-managed, Fargate profile (giới hạn: không DaemonSet, không EBS), EKS Auto Mode (node, LB, storage do EKS quản lý).
  - Truy cập: access entries và access policies thay aws-auth ConfigMap; kubectl với `aws eks update-kubeconfig`; RBAC.
  - Pod Identity (agent add-on, association role ↔ service account) vs IRSA (OIDC provider); add-on: VPC CNI, CoreDNS, kube-proxy, EBS CSI.
  - Quyết định ECS hay EKS cho Tới Nơi (bảng); không lab tốn tiền; bài tập: pod cần ghi S3 thì cấu hình gì?

### Bài 42 · Dựng EKS bằng Terraform và eksctl: cluster, node group, kubectl và namespace cho Tới Nơi

- **Slug**: `aws-eks-terraform-eksctl-kubectl-namespace` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-42-eks-terraform`
- **Tóm tắt**: Bài 42: dựng cluster EKS trong VPC phần 03 bằng module terraform-aws-modules/eks (so với eksctl), managed node group Graviton với Spot, add-on, access entry cho role CI và dev; kubectl context, namespace theo bounded context (booking, dispatch, tracking), ResourceQuota, LimitRange và deploy service đặt chuyến đầu tiên.
- **Tags riêng**: `eks`, `terraform`, `eksctl`, `kubectl`, `namespace`, `node-group`
- **Nội dung bắt buộc**:
  - Terraform module eks: cluster version, endpoint private/public, cluster_addons, managed node group (instance type, capacity type SPOT/ON_DEMAND, taints/labels), access_entries; eksctl YAML tương đương để so sánh.
  - Subnet tag cho LB, cluster security group vs node security group, control plane logging (api, audit) vào CloudWatch.
  - kubectl: update-kubeconfig, context nhiều cluster, namespace per bounded context, ResourceQuota/LimitRange, Deployment + Service ClusterIP cho booking service với image ECR, Pod Identity association.
  - Node update: AMI mới, max unavailable, PDB để không rớt service; chi phí cluster + node + NAT.
  - Lab: examples/toi-noi-aws/p05-42-eks-terraform (validate) + manifest; bài tập: Spot node bị thu hồi thì pod đi đâu?

### Bài 43 · Mạng EKS: VPC CNI, prefix delegation, AWS Load Balancer Controller, Ingress và Gateway API

- **Slug**: `aws-eks-vpc-cni-load-balancer-controller-ingress` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-43-eks-ingress`
- **Tóm tắt**: Bài 43: pod nhận IP thật trong VPC qua VPC CNI, giới hạn ENI/IP và prefix delegation, custom networking khi hết IP; AWS Load Balancer Controller tạo ALB từ Ingress và NLB từ Service, IngressGroup, target type ip, TLS bằng ACM, Gateway API; security group for pods; Service Connect so sánh.
- **Tags riêng**: `eks`, `vpc-cni`, `load-balancer-controller`, `ingress`, `gateway-api`, `networking`
- **Nội dung bắt buộc**:
  - VPC CNI: IP per pod, ENI/IP theo instance type, prefix delegation tăng mật độ, warm pool, custom networking (secondary CIDR 100.64/16) khi VPC hết IP; IPv6 cluster nhắc.
  - AWS Load Balancer Controller: cài bằng Helm với Pod Identity, IngressClass alb, annotation scheme/target-type/certificate-arn/healthcheck-path, IngressGroup gộp nhiều Ingress một ALB; Service type LoadBalancer → NLB.
  - Gateway API (GatewayClass/Gateway/HTTPRoute) hướng thay Ingress; ExternalDNS tạo record Route 53; cert-manager không cần khi dùng ACM.
  - Security group for pods, NetworkPolicy (VPC CNI network policy) chặn booking → payment DB; kube-proxy/ipvs nhắc.
  - Lab: Ingress cho booking service (p05-43-eks-ingress); bài tập: ALB health check fail dù pod Ready, nguyên nhân nào?

### Bài 44 · Lưu trữ EKS: EBS CSI, EFS CSI, StorageClass, StatefulSet và snapshot

- **Slug**: `aws-eks-ebs-efs-csi-statefulset` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-44-eks-storage`
- **Tóm tắt**: Bài 44: pod cần đĩa: EBS CSI driver với StorageClass gp3 và WaitForFirstConsumer, PVC/PV, volume theo AZ; EFS CSI cho nhiều pod cùng đọc ảnh tạm; StatefulSet với volumeClaimTemplates cho worker có trạng thái; snapshot bằng VolumeSnapshot; vì sao database vẫn nên ở RDS/Aurora.
- **Tags riêng**: `eks`, `ebs-csi`, `efs-csi`, `statefulset`, `persistent-volume`, `storage`
- **Nội dung bắt buộc**:
  - Mô hình PV/PVC/StorageClass, dynamic provisioning, reclaim policy; EBS CSI add-on + Pod Identity, gp3 với iops/throughput parameter, volumeBindingMode WaitForFirstConsumer vì EBS theo AZ; resize online.
  - EFS CSI: access point, ReadWriteMany, dùng cho cache ảnh tạm/model; chi phí và hiệu năng khác EBS; Mountpoint for S3 CSI khi chỉ đọc object.
  - StatefulSet: ordinal, headless service, volumeClaimTemplates, PDB; ví dụ worker đối soát giữ checkpoint; rolling update StatefulSet.
  - VolumeSnapshot + snapshot class, restore PVC từ snapshot; AWS Backup cho EBS; chạy PostgreSQL trên EKS vs RDS: khi nào không nên.
  - Lab: manifest StorageClass + StatefulSet (p05-44-eks-storage); bài tập: pod pending vì PVC ở AZ khác node, sửa thế nào?

### Bài 45 · Autoscaling EKS: HPA, VPA, KEDA, Karpenter và Spot cho worker giờ cao điểm

- **Slug**: `aws-eks-hpa-karpenter-spot` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-45-eks-autoscaling`
- **Tóm tắt**: Bài 45: hai tầng scale: pod bằng HPA theo CPU và custom metric (độ sâu hàng đợi SQS qua KEDA), VPA gợi ý request; node bằng Karpenter với NodePool, consolidation, Spot và Graviton; so với Cluster Autoscaler; PodDisruptionBudget, topology spread để không dồn một AZ; đo chi phí.
- **Tags riêng**: `eks`, `hpa`, `keda`, `karpenter`, `spot`, `autoscaling`
- **Nội dung bắt buộc**:
  - Metrics server, HPA v2 theo CPU/memory và external metric; KEDA ScaledObject theo SQS queue length cho dispatch worker; VPA recommendation mode; request/limit đúng thì scale mới đúng.
  - Karpenter: NodePool (instance family, capacity type spot/on-demand, Graviton), EC2NodeClass, consolidation, disruption budget, interruption handling qua SQS; so với Cluster Autoscaler và EKS Auto Mode.
  - Spot: diversification nhiều instance type, taint cho worker chịu được gián đoạn, PDB, graceful shutdown (SIGTERM, terminationGracePeriodSeconds) cho consumer Kafka/SQS.
  - Topology spread constraints và pod anti-affinity theo AZ; scheduled scale giờ cao điểm; chi phí và Compute Optimizer.
  - Lab: HPA + Karpenter NodePool manifest (p05-45-eks-autoscaling); bài tập: HPA scale lên nhưng pod Pending, ai chịu trách nhiệm tiếp?

### Bài 46 · GitOps trên EKS: Argo CD, Helm và Kustomize cho dev, staging và prod

- **Slug**: `aws-eks-gitops-argocd-helm-kustomize` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-46-eks-gitops`
- **Tóm tắt**: Bài 46: chuyển từ kubectl apply sang GitOps: repo cấu hình tách khỏi repo code, Helm chart cho service Tới Nơi với values theo môi trường, Kustomize overlay, Argo CD Application và ApplicationSet, sync policy tự động với prune và self-heal, image updater, RBAC và SSO cho Argo CD, promote dev → staging → prod bằng pull request.
- **Tags riêng**: `eks`, `gitops`, `argocd`, `helm`, `kustomize`, `ci-cd`
- **Nội dung bắt buộc**:
  - GitOps: Git là nguồn sự thật, controller pull, drift tự sửa; cấu trúc repo toi-noi-deploy với apps/<service>/{base,overlays/dev,staging,prod}.
  - Helm: chart cho service chuẩn (Deployment, Service, Ingress, HPA, PDB, ServiceAccount với Pod Identity), values per env, helm template để review; Kustomize patch khi không cần template.
  - Argo CD cài bằng Helm, Application/ApplicationSet (generator theo thư mục/cluster), sync wave, hook, automated sync + prune + selfHeal, health check; Argo CD Image Updater hoặc CI commit tag mới.
  - RBAC project, SSO qua Identity Center (OIDC), notification; multi-cluster quản lý từ một Argo CD; Flux so sánh ngắn.
  - Lab: chart + ApplicationSet (p05-46-eks-gitops); bài tập: ai đó kubectl edit replicas trên prod thì chuyện gì xảy ra?

### Bài 47 · Quan sát EKS: Container Insights, ADOT, Managed Prometheus và Grafana, log và trace

- **Slug**: `aws-eks-observability-container-insights-adot-prometheus` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-47-eks-observability`
- **Tóm tắt**: Bài 47: ba tín hiệu trên EKS: metric bằng CloudWatch Container Insights hoặc Amazon Managed Prometheus với ADOT collector, dashboard Managed Grafana, alert; log bằng Fluent Bit tới CloudWatch Logs hoặc OpenSearch; trace OpenTelemetry từ service .NET tới X-Ray; chi phí metric cardinality và log volume.
- **Tags riêng**: `eks`, `observability`, `prometheus`, `grafana`, `adot`, `opentelemetry`, `fluent-bit`
- **Nội dung bắt buộc**:
  - Container Insights enhanced observability (add-on CloudWatch agent) vs Prometheus: metric node/pod/container, kube-state-metrics; Managed Prometheus workspace, remote write từ ADOT/Prometheus agent, PromQL ví dụ p95 latency booking.
  - Managed Grafana: data source AMP/CloudWatch/X-Ray, SSO Identity Center, dashboard as code; alertmanager rule → SNS.
  - Log: Fluent Bit DaemonSet → CloudWatch Logs (log group per namespace, retention) hoặc OpenSearch bài 39; log JSON có trace_id; sampling.
  - Trace: ADOT collector Deployment/sidecar, OTLP từ service .NET (liên hệ bài 37 loạt .NET) → X-Ray/AMP; Application Signals SLO nhắc tới bài 55; cardinality và chi phí.
  - Lab: ADOT collector + ServiceMonitor manifest (p05-47-eks-observability); bài tập: metric có label userId gây gì?

### Bài 48 · Bảo mật EKS: Pod Identity, External Secrets, Pod Security Standards, network policy và ký image

- **Slug**: `aws-eks-bao-mat-pod-identity-external-secrets` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-48-eks-security`
- **Tóm tắt**: Bài 48: danh tính pod với Pod Identity và quyền tối thiểu, External Secrets Operator đồng bộ Secrets Manager vào Secret, Pod Security Standards restricted, runAsNonRoot và read-only root, NetworkPolicy chặn ngang hàng, image ký bằng Signer hoặc cosign và policy admission bằng Kyverno, GuardDuty EKS runtime monitoring, audit log.
- **Tags riêng**: `eks`, `security`, `pod-identity`, `external-secrets`, `kyverno`, `pod-security`, `network-policy`
- **Nội dung bắt buộc**:
  - Lớp bảo mật: cluster (endpoint private, access entries, audit log), node (IMDS hop limit, SSM), pod (PSS restricted, securityContext), workload identity (Pod Identity mỗi service account một role).
  - Secret: External Secrets Operator với ClusterSecretStore Secrets Manager/Parameter Store, refresh interval, Secret mã hoá etcd bằng KMS; không commit secret vào repo GitOps (SOPS so sánh).
  - NetworkPolicy VPC CNI: default deny trong namespace, cho phép booking → pricing, chặn tới metadata; security group for pods cho RDS.
  - Image: scan ECR, ký với Signer/cosign, Kyverno verifyImages + policy bắt buộc label/limit/no latest tag; GuardDuty EKS protection và Inspector; Pod Security Admission.
  - Lab: manifest ESO + Kyverno policy (p05-48-eks-security); bài tập: pod cần ghi S3 nhưng gọi được RDS secret của service khác, lỗi ở đâu?

### Bài 49 · Progressive delivery trên EKS: Argo Rollouts, canary theo metric, VPC Lattice và giao tiếp giữa service

- **Slug**: `aws-eks-progressive-delivery-argo-rollouts-vpc-lattice` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p05-49-eks-rollouts`
- **Tóm tắt**: Bài 49: thay Deployment bằng Argo Rollouts canary với bước 5/25/50%, AnalysisTemplate đọc Prometheus/CloudWatch để tự promote hoặc abort, traffic routing qua ALB Ingress hoặc Gateway API; VPC Lattice cho service-to-service có auth IAM giữa EKS, ECS và Lambda; App Mesh đã ngừng, khi nào cần Istio.
- **Tags riêng**: `eks`, `argo-rollouts`, `canary`, `vpc-lattice`, `service-mesh`, `progressive-delivery`
- **Nội dung bắt buộc**:
  - Argo Rollouts: Rollout CRD, strategy canary/blueGreen, steps setWeight/pause, AnalysisTemplate với provider Prometheus/CloudWatch (5xx rate < 1%), auto promote/abort, kubectl argo rollouts; so với Flagger.
  - Traffic split: ALB Ingress weighted target group qua annotation, Gateway API HTTPRoute weight, hoặc service mesh; header-based routing cho tester nội bộ.
  - VPC Lattice: service network, service, target group (EKS, ECS, Lambda, IP), auth policy IAM, Gateway API controller cho EKS; kết nối booking (EKS) → pricing (ECS) → fraud (Lambda) xuyên VPC/tài khoản.
  - App Mesh đã kết thúc hỗ trợ (09/2026): lựa chọn hiện tại là Lattice, Istio/Cilium trên EKS khi cần mTLS/retry/timeout mức mesh; cái giá vận hành sidecar.
  - Lab: Rollout + AnalysisTemplate manifest (p05-49-eks-rollouts); bài tập: canary 5% không đủ request để metric có ý nghĩa, làm gì?

### Bài 50 · Vận hành EKS: nâng cấp cluster theo phiên bản, add-on, chi phí, multi-cluster và runbook

- **Slug**: `aws-eks-van-hanh-upgrade-addon-chi-phi` · **Phần**: 5 · **Cấp độ**: advanced · **Lab**: không
- **Tóm tắt**: Bài 50: lịch phát hành Kubernetes và cửa sổ hỗ trợ EKS, nâng cấp control plane rồi node group và add-on, kiểm API deprecated bằng EKS upgrade insights và pluto, PDB và surge; chi phí cluster, node, NAT, LB, log và cách đo per namespace với Kubecost/Split Cost Allocation; multi-cluster theo môi trường hay region; runbook sự cố thường gặp.
- **Tags riêng**: `eks`, `upgrade`, `add-on`, `cost`, `multi-cluster`, `operations`, `runbook`
- **Nội dung bắt buộc**:
  - Phiên bản: 3 release/năm, standard support 14 tháng, extended support có phí; quy trình nâng cấp control plane (1 minor mỗi lần) → add-on → node group (surge, PDB) → kiểm app; upgrade insights, pluto/kubent quét API removed.
  - Add-on EKS quản lý vs Helm tự cài: VPC CNI, CoreDNS, kube-proxy, EBS CSI, Pod Identity agent, CloudWatch agent; conflict resolution và version compatibility.
  - Chi phí: control plane giờ, node (Spot/Graviton/Savings Plans), NAT và data transfer giữa AZ, LB, CloudWatch log; Split Cost Allocation Data per pod, Kubecost/OpenCost; right-size request.
  - Multi-cluster: per env, per region, blast radius; Argo CD quản lý nhiều cluster; backup cluster state bằng Velero; DR cluster (liên hệ bài 37/56).
  - Runbook: node NotReady, pod CrashLoopBackOff, IP exhaustion, CoreDNS quá tải, etcd size; không lab tốn tiền; bài tập: lập kế hoạch nâng 2 minor version cho prod.

### Bài 51 · IAM nâng cao: permission boundary, condition key, ABAC, session policy và Access Analyzer

- **Slug**: `aws-iam-nang-cao-permission-boundary-abac-access-analyzer` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-51-iam-advanced`
- **Tóm tắt**: Bài 51: logic đánh giá policy đầy đủ với SCP, RCP, permission boundary, session policy và resource policy; condition key toàn cục (aws:PrincipalTag, aws:ResourceTag, aws:SourceVpce, aws:MultiFactorAuthPresent) để làm ABAC cho team Tới Nơi; role chaining và external id; Access Analyzer tìm quyền thừa, unused access và sinh policy từ CloudTrail.
- **Tags riêng**: `iam`, `abac`, `permission-boundary`, `access-analyzer`, `condition-key`, `security`
- **Nội dung bắt buộc**:
  - Sơ đồ đánh giá: explicit deny → SCP/RCP → resource policy → permission boundary → session policy → identity policy; cross-account cần cả hai bên.
  - ABAC với tag: principal tag Team=dispatch chỉ chạm resource Team=dispatch; tag policy và quyền gắn tag; so với RBAC nhiều role.
  - Permission boundary để dev tự tạo role cho Lambda mà không vượt quyền; session policy khi assume role; external id cho đối tác; confused deputy.
  - Access Analyzer: external access findings, unused access, policy validation, policy generation từ CloudTrail; CloudTrail để trả lời 'ai làm gì'; credential report.
  - Lab: policy ABAC + boundary (p06-51-iam-advanced, kiểm bằng policy simulator CLI); bài tập: tại sao role có AdministratorAccess vẫn bị AccessDenied?

### Bài 52 · KMS: envelope encryption, key policy, grant, multi-region key và mã hoá S3, RDS, EBS, Secrets

- **Slug**: `aws-kms-envelope-encryption-key-policy` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-52-kms`
- **Tóm tắt**: Bài 52: KMS key là gì và ai được dùng (key policy khác IAM policy), envelope encryption với data key, encryption context, grant cho dịch vụ AWS, key rotation, multi-region key cho DR, customer managed vs AWS managed vs AWS owned; bật mã hoá cho S3 (SSE-KMS, bucket key), RDS, EBS, Secrets Manager; chi phí request và giới hạn.
- **Tags riêng**: `kms`, `encryption`, `key-policy`, `envelope-encryption`, `security`, `compliance`
- **Nội dung bắt buộc**:
  - Key type và ownership; key policy là gốc (kms:ViaService, kms:EncryptionContext), IAM chỉ hiệu lực khi key policy cho phép; key admin vs key user; grant tạm thời cho EBS/RDS.
  - Envelope encryption: GenerateDataKey, mã hoá local, lưu data key mã hoá cạnh dữ liệu; Encryption SDK; encryption context làm AAD và xuất hiện trong CloudTrail.
  - Rotation tự động (material mới, key id giữ), import key material, multi-region key cho bucket replication và secret DR; alias.
  - Dịch vụ: S3 SSE-KMS + bucket key giảm chi phí, RDS/EBS mã hoá tại tạo (không bật sau), Secrets Manager, CloudWatch Logs; chia sẻ snapshot mã hoá qua tài khoản.
  - Lab: key + policy + S3 SSE-KMS Terraform (p06-52-kms); bài tập: xoá key có lịch 7 ngày thì dữ liệu RDS ra sao?

### Bài 53 · PrivateLink, VPC endpoint, Transit Gateway, Network Firewall và kết nối hybrid

- **Slug**: `aws-privatelink-transit-gateway-network-firewall` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-53-private-network`
- **Tóm tắt**: Bài 53: giữ lưu lượng Tới Nơi trong mạng riêng: gateway endpoint S3/DynamoDB, interface endpoint cho ECR, Secrets Manager, STS với endpoint policy; PrivateLink xuất service cho đối tác; Transit Gateway nối VPC dev/staging/prod và hub inspection với Network Firewall; Site-to-Site VPN và Direct Connect tới văn phòng; Cloud WAN và chi phí data transfer.
- **Tags riêng**: `privatelink`, `vpc-endpoint`, `transit-gateway`, `network-firewall`, `direct-connect`, `networking`
- **Nội dung bắt buộc**:
  - Gateway endpoint (S3, DynamoDB, miễn phí, route table) vs interface endpoint (ENI, DNS private, giờ + GB); endpoint policy chỉ cho bucket của Tới Nơi; aws:SourceVpce trong bucket policy.
  - PrivateLink: endpoint service sau NLB, xuất API đối soát cho đối tác thanh toán mà không peering; so với VPC peering (không transitive).
  - Transit Gateway: attachment, route table, segment dev/prod không thấy nhau, hub-and-spoke, inspection VPC với Network Firewall (stateful rule, domain allowlist cho egress), Gateway Load Balancer nhắc.
  - Hybrid: Site-to-Site VPN, Direct Connect, Route 53 Resolver inbound/outbound cho DNS hai chiều; Cloud WAN cho nhiều region; chi phí data transfer giữa AZ/region/Internet.
  - Lab: endpoint + TGW Terraform (p06-53-private-network); bài tập: ECS task không pull được ECR sau khi bỏ NAT, thiếu endpoint nào?

### Bài 54 · CloudTrail, Config, Security Hub, GuardDuty và Inspector: audit và phát hiện cho dữ liệu khách

- **Slug**: `aws-cloudtrail-config-security-hub-guardduty` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-54-detection`
- **Tóm tắt**: Bài 54: lớp phát hiện của Tới Nơi: CloudTrail org trail với data event S3 và Lake truy vấn SQL, Config rule và remediation tự động, Security Hub gom finding theo chuẩn CIS/FSBP, GuardDuty phân tích CloudTrail/VPC Flow/DNS/EKS/RDS, Inspector quét EC2/ECR/Lambda, Detective điều tra; quy trình xử lý sự cố và Macie cho PII trong S3.
- **Tags riêng**: `cloudtrail`, `aws-config`, `security-hub`, `guardduty`, `inspector`, `incident-response`, `macie`
- **Nội dung bắt buộc**:
  - CloudTrail: management vs data event (S3 object, Lambda invoke), org trail vào log archive, log file validation, CloudTrail Lake query 'ai đọc secret X tuần qua'; CloudWatch alarm từ CloudTrail (root login, policy change).
  - Config: recorder, rule managed/custom (Lambda/Guard), conformance pack, remediation SSM Automation (bật mã hoá, tắt public); timeline tài nguyên.
  - Security Hub: chuẩn CIS/AWS FSBP/PCI, severity, automation rule, tích hợp finding từ GuardDuty/Inspector/Macie/Access Analyzer; aggregation nhiều tài khoản/region.
  - GuardDuty: nguồn dữ liệu, finding type (credential exfiltration, crypto mining, EKS runtime, RDS login anomaly), Malware Protection; Inspector cho ECR/EC2/Lambda; Macie quét PII (số điện thoại, CCCD) trong bucket.
  - Quy trình incident response theo NIST (chuẩn bị, phát hiện, cô lập, diệt, phục hồi, rút kinh nghiệm) với playbook 'access key lộ'; lab: Config rule + Security Hub Terraform (p06-54-detection); bài tập: tìm ai đã tắt VPC Flow Logs.

### Bài 55 · Quan sát nâng cao: X-Ray và ADOT, Logs Insights, anomaly detection, Application Signals và SLO

- **Slug**: `aws-observability-nang-cao-xray-adot-logs-insights-slo` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-55-slo`
- **Tóm tắt**: Bài 55: từ metric sang câu hỏi vận hành: trace một yêu cầu đặt chuyến chậm qua ALB, ECS, Lambda, Aurora bằng X-Ray/ADOT và service map; log có cấu trúc với trace id, Logs Insights query và metric filter, embedded metric format; CloudWatch anomaly detection và composite alarm giảm ồn; Application Signals định nghĩa SLO cho API đặt chuyến; Synthetics canary và RUM; chi phí log.
- **Tags riêng**: `observability`, `x-ray`, `opentelemetry`, `logs-insights`, `slo`, `application-signals`, `synthetics`
- **Nội dung bắt buộc**:
  - Trace: OTel SDK/ADOT → X-Ray, propagation header giữa ALB/ECS/Lambda/SQS, sampling rule, service map, annotation city/service để lọc; liên hệ bài 37 loạt .NET.
  - Log: JSON có trace_id/trip_id, Logs Insights (stats, parse, filter), metric filter → metric, Embedded Metric Format từ app, retention và data protection policy che PII.
  - Alarm: anomaly detection band, composite alarm (5xx AND latency), alarm theo percentile, SNS → on-call; tránh alert fatigue.
  - Application Signals: service, SLI/SLO (availability 99.9%, latency p99 < 800 ms), error budget và burn rate alarm; Synthetics canary đặt chuyến mỗi 5 phút, RUM cho web; dashboard vận hành.
  - Lab: SLO + composite alarm Terraform (p06-55-slo); bài tập: SLO 99.9% tháng cho phép bao nhiêu phút lỗi và burn rate 14 nghĩa là gì?

### Bài 56 · Resilience: multi-AZ tới multi-region, Route 53 failover và ARC, Global Accelerator, Fault Injection Service

- **Slug**: `aws-resilience-multi-region-route53-failover-fis` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-56-resilience`
- **Tóm tắt**: Bài 56: thiết kế Tới Nơi sống qua mất một AZ và có kế hoạch cho mất region: static stability, cell-based, quota và retry với backoff và jitter; Aurora Global Database, DynamoDB Global Tables, S3 replication; Route 53 health check và failover, Application Recovery Controller readiness và routing control, Global Accelerator; diễn tập bằng Fault Injection Service và game day.
- **Tags riêng**: `resilience`, `multi-region`, `route53`, `application-recovery-controller`, `fault-injection`, `chaos-engineering`
- **Nội dung bắt buộc**:
  - Nguyên tắc: AZ độc lập, static stability (không phụ thuộc control plane khi sự cố), cell-based theo thành phố, giới hạn blast radius; quota và throttling; retry với exponential backoff + jitter, circuit breaker; dependency timeout.
  - Dữ liệu đa region: Aurora Global Database (RPO giây, failover managed/unplanned), DynamoDB Global Tables (conflict last-writer-wins), S3 CRR, ElastiCache Global Datastore; secret và KMS multi-region; IaC deploy region B.
  - Điều hướng: Route 53 health check + failover record, ARC readiness check, routing control (tắt region chủ động), zonal shift; Global Accelerator anycast IP và endpoint weight; DNS TTL và client caching.
  - Fault Injection Service: experiment template (stop instance, AZ power interruption, network latency, EKS pod delete), stop condition bằng alarm, game day runbook; liên hệ DR bài 37.
  - Lab: FIS experiment + failover record Terraform (p06-56-resilience); bài tập: failover DNS 60 giây nhưng app vẫn lỗi 10 phút, vì sao?

### Bài 57 · Step Functions: Saga đặt chuyến, compensation, distributed map và các pattern serverless nâng cao

- **Slug**: `aws-step-functions-saga-dat-chuyen` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-57-step-functions`
- **Tóm tắt**: Bài 57: điều phối luồng đặt chuyến nhiều bước (giữ tiền, tìm tài xế, xác nhận, thu tiền) bằng Step Functions Standard với compensation khi thất bại, retry/catch, timeout và heartbeat cho bước chờ tài xế, callback task token; Express workflow cho luồng ngắn; distributed map xử lý 2 triệu chuyến đối soát; JSONata và variables; so với Saga tự viết bằng SQS và với EventBridge.
- **Tags riêng**: `step-functions`, `saga`, `serverless`, `workflow`, `compensation`, `distributed-map`
- **Nội dung bắt buộc**:
  - Saga trong Tới Nơi: các bước và compensation (hoàn giữ tiền, huỷ chuyến), idempotency từng bước; orchestration (Step Functions) vs choreography (EventBridge/MSK) trong bảng; liên hệ tài liệu kiến trúc §8–9.
  - State machine ASL: Task (Lambda, SDK integration gọi DynamoDB/SQS trực tiếp), Choice, Parallel, Wait, Map; Retry/Catch, TimeoutSeconds, HeartbeatSeconds; wait for callback với task token khi tài xế bấm nhận.
  - Standard vs Express (thời lượng, giá theo transition vs duration, exactly-once vs at-least-once); execution history, redrive từ bước lỗi; versioning và alias.
  - Distributed map đọc S3 manifest, concurrency, batch, tolerated failure cho job đối soát đêm; JSONata và variables giảm Lambda keo dán; SAM/CDK định nghĩa.
  - Lab: ASL JSON + Terraform (p06-57-step-functions, kiểm bằng CLI validate-state-machine-definition); bài tập: compensation thất bại thì làm gì?

### Bài 58 · Well-Architected Framework: review sáu trụ cột cho Tới Nơi và biến finding thành backlog

- **Slug**: `aws-well-architected-review-toi-noi` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: không
- **Tóm tắt**: Bài 58: chạy một Well-Architected review thật trên kiến trúc Tới Nơi đã dựng qua 57 bài: sáu trụ cột, câu hỏi trọng tâm và high-risk issue, Well-Architected Tool với lens (Serverless, Container Build), cách chấm, ưu tiên và lập kế hoạch cải tiến có đo được; review định kỳ trước mỗi mốc lớn.
- **Tags riêng**: `well-architected`, `architecture-review`, `best-practices`, `operational-excellence`, `cost-optimization`, `sustainability`
- **Nội dung bắt buộc**:
  - Sáu trụ cột và nguyên tắc thiết kế chung; mỗi trụ cột 2–3 câu hỏi áp vào Tới Nơi với bằng chứng từ bài trước (bảng: câu hỏi → bài liên quan → trạng thái).
  - Well-Architected Tool: workload, lens (Serverless, Container Build, Operational Readiness), milestone, HRI/MRI, improvement plan; Trusted Advisor và Health Dashboard bổ sung.
  - Ví dụ finding thật: NAT single-AZ (Reliability), log không có retention (Cost), thiếu game day (OpEx), PII chưa Macie (Security), Graviton chưa dùng (Sustainability).
  - Biến finding thành backlog có owner, ước lượng và chỉ số; review lại sau 90 ngày; tránh 'checkbox'.
  - Không lab; bài tập: tự chấm 5 câu hỏi Reliability cho hệ thống bạn đang vận hành.

### Bài 59 · FinOps trên AWS: cost allocation, Savings Plans, Reserved, Spot, right-sizing và kiến trúc tiết kiệm

- **Slug**: `aws-finops-savings-plans-spot-right-sizing` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: `examples/toi-noi-aws/p06-59-finops`
- **Tóm tắt**: Bài 59: đọc hoá đơn Tới Nơi theo tag và tài khoản với Cost Explorer, CUR 2.0 và Athena, unit economics chi phí mỗi chuyến; cam kết với Savings Plans (Compute, EC2 Instance) và Reserved cho RDS/ElastiCache/OpenSearch, Spot cho worker; right-sizing với Compute Optimizer và Cost Optimization Hub; data transfer, NAT, log và snapshot ẩn; budget, anomaly detection và quy trình FinOps hằng tháng.
- **Tags riêng**: `finops`, `cost-optimization`, `savings-plans`, `spot`, `right-sizing`, `cost-explorer`
- **Nội dung bắt buộc**:
  - Mô hình giá: on-demand, Savings Plans (Compute vs EC2 Instance, 1/3 năm, no/partial/all upfront, coverage và utilization), Reserved Instance cho RDS/ElastiCache/OpenSearch/Redshift, Spot; mua bao nhiêu từ baseline.
  - Nhìn thấy: cost allocation tag, Cost Categories, CUR 2.0 → Athena, Cost Explorer API, Split Cost Allocation cho ECS/EKS; chi phí mỗi chuyến làm KPI.
  - Giảm: Compute Optimizer, Cost Optimization Hub, Graviton, gp2→gp3, S3 lifecycle, log retention, snapshot cũ, NAT → VPC endpoint, cross-AZ transfer, idle LB/EIP, dev tắt ngoài giờ.
  - Quy trình FinOps: inform/optimize/operate, budget và Cost Anomaly Detection, review tháng, owner theo service; tránh cam kết trước khi right-size.
  - Lab: Budget + anomaly monitor Terraform (p06-59-finops); bài tập: tính coverage Savings Plans cho tải 24/7 và giờ cao điểm.

### Bài 60 · Tổng hợp: kiến trúc production Tới Nơi trên AWS, luồng một chuyến đi qua từng dịch vụ và runbook

- **Slug**: `aws-tong-hop-kien-truc-production-toi-noi` · **Phần**: 6 · **Cấp độ**: advanced · **Lab**: không
- **Tóm tắt**: Bài 60: ghép 59 bài thành một bản vẽ: tài khoản và mạng, edge, compute ECS/EKS/Lambda, dữ liệu Aurora/DynamoDB/ElastiCache/MSK/Kinesis/S3, pipeline và GitOps, bảo mật và quan sát; đi theo một chuyến từ lúc khách bấm đặt tới lúc đối soát để thấy từng dịch vụ làm gì; runbook on-call, SLO, chi phí ước tính theo lớp và lộ trình học tiếp (chứng chỉ, chủ đề chưa phủ).
- **Tags riêng**: `architecture`, `production`, `reference-architecture`, `runbook`, `tong-hop`
- **Nội dung bắt buộc**:
  - Bản vẽ tổng: 3 SVG theo lớp (account/network, compute/data, delivery/observability) nhất quán với tài liệu kiến trúc §19 và các bài trước; mỗi khối ghi số bài tham chiếu.
  - Luồng một chuyến: CloudFront/WAF → API Gateway/ALB → booking (ECS/EKS) → pricing cache → Aurora + outbox → MSK → dispatch (DynamoDB vị trí) → Step Functions thanh toán → EventBridge → Kinesis/Firehose → S3/Athena; chỗ nào idempotent, chỗ nào có thể mất.
  - Vận hành: SLO và alarm, on-call runbook 5 sự cố (Aurora failover, Spot thu hồi, MSK lag, WAF chặn nhầm, deploy lỗi), game day và DR; bảo mật tóm tắt theo lớp.
  - Chi phí: bảng ước lượng theo lớp và theo môi trường (dev nhỏ, prod), nhấn mạnh con số minh hoạ cần Pricing Calculator; quyết định ECS vs EKS, Kinesis vs MSK đã chọn và lý do.
  - Lộ trình tiếp: chứng chỉ theo phần (CLF, SAA, DVA, DOP, SCS, SAP), chủ đề chưa phủ (ML/Bedrock, IoT, media), cách dùng repo examples; không lab; bài tập: tự vẽ lại luồng huỷ chuyến với compensation.

## 6. Trạng thái

Cập nhật khi bài được viết/review/xuất bản. Bài được tính "đã viết" khi tệp `.md` qua review và `npm run seed` nạp không lỗi.

| Phần | Viết | Review | Ghi chú |
|---|---|---|---|
| Phần 01 · Nền tảng AWS | chưa | xong 2026-11-06 | Lab 05/07/08/10 nối nhau qua tag Name của lab 04 (env.sh để trống ID). Chưa chạy trên tài khoản thật. Cần quyết: viện dẫn pháp lý VN ở bài 01; khoá tag trong `FilterExpression` của Budgets (bài 10) |
| Phần 02 · Container, serverless và dịch vụ ứng dụng | chưa | xong 2026-11-06 | Cần chốt: secret DB dạng connection string (bài 12, bài 36 .NET) hay JSON có rotation (bài 14); `healthCheck` dùng curl trong khi image `aspnet` không có curl. |
| Phần 03 · Infrastructure as Code và CI/CD | chưa | chưa | |
| Phần 04 · Dữ liệu, lưu trữ và messaging | chưa | chưa | |
| Phần 05 · Kubernetes trên AWS với EKS | chưa | chưa | |
| Phần 06 · Bảo mật, resilience và vận hành ở quy mô | chưa | chưa | |

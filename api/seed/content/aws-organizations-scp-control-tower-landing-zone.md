---
slug: aws-organizations-scp-control-tower-landing-zone
title: "Organizations, SCP và Control Tower: nhiều tài khoản cho dev, staging, prod và bảo mật"
summary: "Bài 28: vì sao Tới Nơi cần nhiều tài khoản thay vì một tài khoản nhiều VPC; Organizations với OU, SCP chặn region và dịch vụ, Resource Control Policy; Control Tower dựng landing zone với log archive và audit account; truy cập chéo tài khoản bằng Identity Center và role."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, organizations, scp, control-tower, landing-zone, multi-account]
language: vi
---

Giả sử Tới Nơi vẫn chạy mọi thứ trong một tài khoản AWS: ba VPC cho dev, staging và prod, mọi kỹ sư dùng chung một nhóm role. Một chiều thứ Sáu, ai đó chạy `terraform destroy` với profile sai và bucket ảnh chứng từ của môi trường prod đi cùng. Không có lỗ hổng nào bị khai thác, chỉ có một ranh giới quá mỏng: ranh giới giữa các môi trường chỉ là một tên tag và một dòng trong file cấu hình.

Đây là **bài 28** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bài này không có lab vì cần nhiều tài khoản, và đưa ra chính sách ở cấp tổ chức, nên mọi JSON dưới đây **chưa được thử trên một AWS Organizations thật**. Bạn nên nắm IAM (bài 03) và role cho CI ([bài 25](/articles/aws-github-actions-oidc-ecr-deploy)) trước.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gắn một SCP có <code>"Effect": "Allow"</code> cho <code>s3:*</code> lên một OU. Người dùng trong tài khoản thuộc OU đó đã ghi được S3 chưa?</p>
<details><summary>Xem đáp án</summary><p>Chưa. SCP không cấp quyền mà chỉ đặt trần. Người dùng vẫn cần IAM policy hoặc resource policy cho phép; quyền hiệu lực là phần giao của hai bên.</p></details>
</div>

## Giới hạn của một tài khoản

Tài khoản AWS là ranh giới cứng nhất mà AWS cung cấp: ranh giới quyền, quota, hoá đơn và dữ liệu. Nhiều VPC trong một tài khoản chỉ tạo ranh giới mạng.

| Vấn đề | Một tài khoản nhiều VPC | Nhiều tài khoản |
|---|---|---|
| Blast radius | Một quyền admin sai chạm cả prod | Lỗi ở dev không chạm được prod về mặt quyền |
| Quota | Dev chạy thử tải và ăn quota của prod | Mỗi tài khoản có quota riêng theo region |
| Hoá đơn | Phải tách chi phí bằng tag, tag thiếu là mất dấu | Mỗi tài khoản có dòng chi phí riêng |
| IAM | Policy ngày càng phức tạp với điều kiện theo tag | Role đơn giản, mỗi tài khoản một việc |
| Kiểm toán | Log chung với dữ liệu người dùng cuối | Log archive nằm ở tài khoản người vận hành không sửa được |

Với Tới Nơi, cây OU đề xuất gồm: **Security** (log archive, audit), **Infrastructure** (network, shared services), **Workloads** (dev, staging, prod; khi lớn thì tách theo nhóm service, ví dụ thanh toán riêng) và **Sandbox** cho thử nghiệm cá nhân có budget.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Cây OU của Tới Nơi: Root với bốn OU là Security, Infrastructure, Workloads và Sandbox, mỗi OU chứa các tài khoản tương ứng."><rect x="340" y="15" width="200" height="50" rx="10" class="f-pk-soft s-pk"/><text x="354" y="38" class="t-b">Root</text><text x="354" y="55" class="t-small">Management account</text><path d="M440 65 L440 85 M105 85 L780 85 M105 85 L105 110 M330 85 L330 110 M555 85 L555 110 M780 85 L780 110" class="f-none s-line" stroke-width="1.6"/><rect x="10" y="110" width="190" height="50" rx="10" class="f-pk-soft s-pk"/><text x="22" y="140" class="t-b">OU Security</text><rect x="235" y="110" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="247" y="140" class="t-b">OU Infrastructure</text><rect x="460" y="110" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="472" y="140" class="t-b">OU Workloads</text><rect x="685" y="110" width="190" height="50" rx="10" class="f-warn-soft s-warn"/><text x="697" y="140" class="t-b t-warn">OU Sandbox</text><path d="M105 160 L105 180 M330 160 L330 180 M555 160 L555 180 M780 160 L780 180" class="f-none s-line" stroke-width="1.6"/><rect x="10" y="180" width="190" height="64" rx="8" class="f-surface s-line"/><text x="22" y="204" class="t-small t-mono">toinoi-log-archive</text><text x="22" y="222" class="t-small t-mono">toinoi-audit</text><rect x="235" y="180" width="190" height="64" rx="8" class="f-surface s-line"/><text x="247" y="204" class="t-small t-mono">toinoi-network</text><text x="247" y="222" class="t-small t-mono">toinoi-shared-services</text><rect x="460" y="180" width="190" height="64" rx="8" class="f-surface s-line"/><text x="472" y="198" class="t-small t-mono">toinoi-dev</text><text x="472" y="214" class="t-small t-mono">toinoi-staging</text><text x="472" y="230" class="t-small t-mono">toinoi-prod</text><rect x="685" y="180" width="190" height="64" rx="8" class="f-surface s-line"/><text x="697" y="204" class="t-small t-mono">toinoi-sandbox-xxx</text><text x="697" y="222" class="t-small">có budget, tự dọn</text><text x="10" y="276" class="t-small">SCP chặn region và root user gắn ở Root; SCP về mã hoá gắn ở OU Workloads; tài khoản quản lý không chạy workload.</text></svg>
</div>
<figcaption>Hình 1. Chính sách gắn vào Root, OU hoặc từng tài khoản và được kế thừa xuống dưới.</figcaption>
</figure>

## SCP: guardrail đặt trần quyền

**Service control policy** áp lên mọi principal trong tài khoản thành viên, kể cả root user của tài khoản đó, nhưng không áp lên tài khoản quản lý và không áp lên service-linked role. Quyền hiệu lực là giao của SCP và IAM policy; SCP không bao giờ cấp quyền. Điều kiện để dùng là tổ chức bật đủ tính năng (all features).

SCP đầu tiên nên có là chặn region ngoài `ap-southeast-1` và `us-east-1` (CloudFront và ACM của CloudFront nằm ở `us-east-1`). Dịch vụ toàn cầu được loại bằng `NotAction`; danh sách bên dưới là ví dụ cần đối chiếu với các dịch vụ bạn dùng. Nếu dùng Control Tower (phần sau), bật control Region deny có sẵn thay vì tự viết SCP này, vì control đó đã chừa sẵn các role quản trị của landing zone:

```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Sid": "DenyOutsideAllowedRegions",
    "Effect": "Deny",
    "NotAction": ["iam:*", "organizations:*", "route53:*", "route53domains:*", "cloudfront:*",
                  "support:*", "sts:*", "budgets:*", "ce:*", "waf:*", "wafv2:*", "account:*"],
    "Resource": "*",
    "Condition": {"StringNotEquals": {"aws:RequestedRegion": ["ap-southeast-1", "us-east-1"]}}
  }]
}
```

SCP thứ hai bảo vệ dấu vết và chặn root user. Nó phải loại trừ role mà Control Tower dùng để quản lý, nếu không chính landing zone sẽ bị chặn:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "ProtectAuditTrail",
      "Effect": "Deny",
      "Action": ["cloudtrail:StopLogging", "cloudtrail:DeleteTrail", "cloudtrail:UpdateTrail",
                 "guardduty:DeleteDetector", "guardduty:DisassociateFromAdministratorAccount",
                 "guardduty:DisassociateFromMasterAccount"],
      "Resource": "*",
      "Condition": {"ArnNotLike": {"aws:PrincipalArn": "arn:aws:iam::*:role/AWSControlTowerExecution"}}
    },
    {
      "Sid": "DenyRootUser",
      "Effect": "Deny",
      "Action": "*",
      "Resource": "*",
      "Condition": {"StringLike": {"aws:PrincipalArn": "arn:aws:iam::*:root"}}
    }
  ]
}
```

Tạo, bật và gắn bằng CLI (ID dưới đây là giá trị minh hoạ):

```bash
aws organizations list-roots
aws organizations create-organizational-unit --parent-id r-ab12 --name Workloads
aws organizations create-policy --name toinoi-deny-outside-regions --type SERVICE_CONTROL_POLICY \
  --description "Chan region ngoai ap-southeast-1 va us-east-1" --content file://scp-regions.json
aws organizations attach-policy --policy-id p-EXAMPLE1 --target-id r-ab12
```

Hãy gắn thử lên OU Sandbox trước. Một SCP sai có thể khoá cả đội ra khỏi tài khoản; luôn giữ một đường vào qua tài khoản quản lý và kiểm bằng IAM policy simulator hoặc thử trên tài khoản thử nghiệm.

## Resource Control Policy

**Resource control policy (RCP)** là nửa còn lại: nếu SCP đặt trần cho principal, RCP đặt trần cho **tài nguyên** trong tài khoản thành viên, kể cả khi principal gọi từ ngoài tổ chức. RCP cần được bật bằng `enable-policy-type` với `RESOURCE_CONTROL_POLICY`. Danh sách dịch vụ hỗ trợ RCP mở rộng dần (ban đầu gồm S3, STS, KMS, SQS, Secrets Manager); hãy kiểm trang user guide trước khi dựa vào.

| | SCP | RCP |
|---|---|---|
| Giới hạn | Hành động của principal | Truy cập vào tài nguyên |
| Principal ngoài tổ chức | Không kiểm soát | Có thể chặn |
| Hiệu lực | Đặt trần, không cấp quyền | Đặt trần, không cấp quyền |
| Ví dụ cho Tới Nơi | Chặn region, chặn tắt trail | Bắt buộc TLS cho S3, chặn truy cập ngoài tổ chức |

```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Sid": "EnforceTlsOnS3",
    "Effect": "Deny",
    "Principal": "*",
    "Action": "s3:*",
    "Resource": "*",
    "Condition": {"BoolIfExists": {"aws:SecureTransport": "false"}}
  }]
}
```

RCP đòi `Principal` là `*` và chỉ dùng `Deny`. Một RCP kiểu perimeter theo `aws:PrincipalOrgID` phải chừa đường cho service principal của AWS, nếu không CloudTrail hay Config sẽ không ghi được vào bucket.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 2. Một yêu cầu phải qua SCP, được IAM policy cho phép, rồi qua RCP của tài nguyên; mọi lớp chỉ Deny hoặc đặt trần, quyền chỉ được cấp bởi Allow ở IAM hoặc resource policy."><defs><marker id="aws28-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="45" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="22" y="74" class="t-b">Yêu cầu</text><text x="22" y="96" class="t-small">Principal trong</text><text x="22" y="112" class="t-small">tài khoản workload</text><path d="M160 85 L190 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws28-2-a)"/><rect x="190" y="45" width="165" height="80" rx="10" class="f-warn-soft s-warn"/><text x="202" y="74" class="t-b t-warn">SCP</text><text x="202" y="96" class="t-small">Trần quyền của principal</text><text x="202" y="112" class="t-small">Root, OU, account</text><path d="M355 85 L385 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws28-2-a)"/><rect x="385" y="45" width="165" height="80" rx="10" class="f-pk-soft s-pk"/><text x="397" y="74" class="t-b">IAM policy</text><text x="397" y="96" class="t-small">Cấp quyền (Allow),</text><text x="397" y="112" class="t-small">permission boundary</text><path d="M550 85 L580 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws28-2-a)"/><rect x="580" y="45" width="150" height="80" rx="10" class="f-warn-soft s-warn"/><text x="592" y="74" class="t-b t-warn">RCP</text><text x="592" y="96" class="t-small">Trần quyền trên</text><text x="592" y="112" class="t-small">tài nguyên</text><path d="M730 85 L760 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws28-2-a)"/><rect x="760" y="45" width="110" height="80" rx="10" class="f-phys-soft s-phys"/><text x="772" y="74" class="t-b">Cho phép</text><text x="772" y="96" class="t-small">Có Allow và không</text><text x="772" y="112" class="t-small">lớp nào chặn</text><text x="10" y="165" class="t-small">Một Deny ở bất kỳ lớp nào đều thắng. Resource policy trong cùng tài khoản cũng có thể cấp quyền, tuỳ loại tài nguyên.</text></svg>
</div>
<figcaption>Hình 2. Hai lớp trần (SCP và RCP) bao quanh lớp cấp quyền (IAM hoặc resource policy).</figcaption>
</figure>

## Control Tower: landing zone dựng sẵn

**Control Tower** dựng một landing zone theo thực hành đã đúc kết: tổ chức Organizations, OU Security với hai tài khoản **log archive** và **audit**, IAM Identity Center, AWS Config ghi cấu hình từng tài khoản, một **Config aggregator** ở tài khoản audit để xem tuân thủ của cả tổ chức, và **CloudTrail của tổ chức** ghi vào bucket ở log archive. Home region chọn khi dựng và không đổi được sau đó, nên chọn `ap-southeast-1` có chủ đích.

Control Tower triển khai ba loại **control**: *preventive* (thực thi bằng SCP, chặn hành động), *detective* (AWS Config rule, phát hiện sai lệch) và *proactive* (CloudFormation hook, chặn tài nguyên không hợp lệ trước khi tạo; bài 30 sẽ đi tiếp theo hướng này bằng policy as code). Mỗi control thuộc nhóm mandatory, strongly recommended hoặc elective. **Account Factory** tạo tài khoản mới đã được đưa vào OU, có baseline và control sẵn, thay cho việc tạo tay từng account.

## Truy cập chéo tài khoản

Con người vào bằng **IAM Identity Center**: mỗi permission set gán cho nhóm với từng tài khoản, và Identity Center tạo một role tương ứng trong tài khoản đó. Máy vào bằng role: mỗi tài khoản có role CI tin cậy OIDC của GitHub (bài 25) hoặc một role triển khai cho phép pipeline từ tài khoản shared services assume. **AWS RAM** chia sẻ subnet và Transit Gateway từ tài khoản network cho tài khoản workload. Với hoá đơn, tài khoản quản lý nhận hoá đơn gộp; tag phân bổ chi phí phải được kích hoạt ở đó.

## Lỗi thường gặp

- **Chạy workload trong tài khoản quản lý**: SCP không áp lên nó, nên không còn guardrail nào.
- **SCP deny quá rộng mà không loại trừ role quản trị**: chính Control Tower hoặc pipeline bị khoá.
- **Gắn thử thẳng lên Root**: sai một dòng là cả tổ chức bị ảnh hưởng. Thử ở OU Sandbox rồi mới nâng lên.
- **Nhầm SCP với quyền**: cần cả Allow ở IAM; SCP chỉ là trần.
- **Quên kiểm tra service-linked role và dịch vụ toàn cầu** khi viết deny theo region.

**Bài tập:** muốn chặn `s3:PutObject` khi không mã hoá thì gắn SCP đó ở OU nào, và nó có gì cần thận trọng?

<details><summary>Xem đáp án</summary><p>Gắn ở OU Workloads (kế thừa xuống dev, staging, prod), thử trước ở OU Sandbox hoặc nhóm non-prod, không gắn Root để tránh chặn dịch vụ ghi log của tài khoản log archive. Cần thận trọng: từ năm 2023 S3 tự mã hoá object mới bằng SSE-S3, nên điều kiện kiểu "thiếu header mã hoá" có thể chặn cả client vốn dựa vào mã hoá mặc định. Thường tốt hơn là yêu cầu khoá KMS cụ thể hoặc bắt buộc bằng RCP, bucket policy và Config rule.</p></details>

Bài trước: [bài 27](/articles/aws-chien-luoc-deploy-rolling-canary-rollback). Bài tiếp: [bài 29](/articles/aws-appconfig-feature-flag-cau-hinh-dong).

## Tài liệu gốc

- [AWS Organizations User Guide](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_introduction.html)
- [Service control policies](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_scps.html)
- [Resource control policies](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_rcps.html)
- [AWS Control Tower User Guide](https://docs.aws.amazon.com/controltower/latest/userguide/what-is-control-tower.html)
- [Whitepaper Organizing Your AWS Environment Using Multiple Accounts](https://docs.aws.amazon.com/whitepapers/latest/organizing-your-aws-environment/organizing-your-aws-environment.html)

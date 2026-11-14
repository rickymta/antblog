---
slug: aws-cloudtrail-config-security-hub-guardduty
title: "CloudTrail, Config, Security Hub, GuardDuty và Inspector: audit và phát hiện cho dữ liệu khách"
summary: "Bài 54: lớp phát hiện của Tới Nơi: CloudTrail với data event S3 và truy vấn log, Config rule và remediation, Security Hub (phiên bản mới và CSPM), GuardDuty kể cả Extended Threat Detection, Inspector, Macie cho PII, cùng quy trình xử lý sự cố theo NIST với playbook cho access key bị lộ."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, cloudtrail, aws-config, security-hub, guardduty, inspector, incident-response, macie]
language: vi
---

Đối tác thanh toán hỏi Tới Nơi một câu rất cụ thể: nếu access key của một kỹ sư bị đẩy nhầm lên một repository công khai lúc hai giờ sáng, bao lâu thì các bạn biết, và biết nó đã làm những gì? Câu trả lời thật thà lúc này là "khi ai đó tình cờ nhìn hoá đơn". Phần 06 đã có quyền tối thiểu ([bài 51](/articles/aws-iam-nang-cao-permission-boundary-abac-access-analyzer)), mã hoá ([bài 52](/articles/aws-kms-envelope-encryption-key-policy)) và mạng riêng ([bài 53](/articles/aws-privatelink-transit-gateway-network-firewall)); còn thiếu **phát hiện và truy vết**.

Đây là **bài 54** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần nắm tài khoản log archive và tài khoản security ở [bài 28](/articles/aws-organizations-scp-control-tower-landing-zone). Bài này đi qua từng dịch vụ theo câu hỏi nó trả lời, rồi ghép thành quy trình xử lý sự cố.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Kẻ tấn công dùng key lộ gọi <code>GetObject</code> hàng loạt trên bucket ảnh chứng từ. CloudTrail event history mặc định của tài khoản có ghi các lần gọi đó không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Event history chỉ chứa management event trong 90 ngày. <code>GetObject</code> là data event: phải tạo trail có bật data event cho bucket đó, và chấp nhận chi phí theo số event.</p></details>
</div>

## Năm dịch vụ, năm câu hỏi khác nhau

Bạn dễ nhầm đây là "năm công cụ giám sát"; thực ra mỗi cái trả lời một câu hỏi riêng:

| Dịch vụ | Câu hỏi nó trả lời | Dữ liệu đầu vào | Kết quả |
|---|---|---|---|
| CloudTrail | Ai gọi API nào, lúc nào, từ đâu | Lời gọi API | Log sự kiện |
| Config | Tài nguyên cấu hình ra sao, có tuân thủ không, đổi lúc nào | Trạng thái tài nguyên | Configuration item, đánh giá rule |
| GuardDuty | Có hành vi đáng ngờ không | CloudTrail, VPC Flow Logs, DNS, S3, EKS, runtime… | Finding có mức nghiêm trọng |
| Inspector | Phần mềm có lỗ hổng đã biết không | EC2, image ECR, Lambda | Finding theo CVE |
| Macie | Dữ liệu nhạy cảm nằm ở đâu trong S3 | Nội dung object | Finding PII |

**Security Hub** đứng trên cùng: gom finding từ các dịch vụ trên (và từ Access Analyzer ở bài 51) về một chỗ để xếp hạng và chuyển cho người xử lý. **Detective** dựng đồ thị từ cùng nguồn log để điều tra một finding.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Nguồn dữ liệu (CloudTrail, VPC Flow Logs và DNS, trạng thái tài nguyên, nội dung S3) đi vào GuardDuty, Config, Inspector, Macie, Access Analyzer; tất cả gửi finding về Security Hub, rồi EventBridge và SNS báo cho on-call.">
<defs><marker id="aws54-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="20" width="170" height="48" rx="10" class="f-surface s-line"/><text x="34" y="49" class="t-b">CloudTrail</text>
<rect x="20" y="82" width="170" height="48" rx="10" class="f-surface s-line"/><text x="34" y="111" class="t-b">Flow Logs, DNS</text>
<rect x="20" y="144" width="170" height="48" rx="10" class="f-surface s-line"/><text x="34" y="173" class="t-b">Trạng thái tài nguyên</text>
<rect x="20" y="206" width="170" height="48" rx="10" class="f-surface s-line"/><text x="34" y="235" class="t-b">Object trong S3</text>
<rect x="290" y="20" width="190" height="48" rx="10" class="f-pk-soft s-pk"/><text x="304" y="49" class="t-b">GuardDuty (+ETD)</text>
<rect x="290" y="82" width="190" height="48" rx="10" class="f-pk-soft s-pk"/><text x="304" y="111" class="t-b">Config rules</text>
<rect x="290" y="144" width="190" height="48" rx="10" class="f-pk-soft s-pk"/><text x="304" y="173" class="t-b">Inspector, Access Analyzer</text>
<rect x="290" y="206" width="190" height="48" rx="10" class="f-pk-soft s-pk"/><text x="304" y="235" class="t-b">Macie</text>
<rect x="580" y="60" width="150" height="150" rx="10" class="f-phys-soft s-phys"/><text x="594" y="105" class="t-b">Security Hub</text><text x="594" y="128" class="t-small">xếp hạng, gom,</text><text x="594" y="146" class="t-small">automation rule</text>
<rect x="770" y="100" width="100" height="70" rx="10" class="f-warn-soft s-warn"/><text x="782" y="130" class="t-b t-warn">On-call</text><text x="782" y="150" class="t-small">SNS</text>
<path d="M190 44 L288 44 M190 106 L288 52 M190 168 L288 106 M190 168 L288 168 M190 230 L288 230" class="f-none s-phys" stroke-width="2" marker-end="url(#aws54-1-a)"/>
<path d="M480 44 L578 100 M480 106 L578 120 M480 168 L578 150 M480 230 L578 180" class="f-none s-phys" stroke-width="2" marker-end="url(#aws54-1-a)"/>
<path d="M730 135 L768 135" class="f-none s-phys" stroke-width="2" marker-end="url(#aws54-1-a)"/>
</svg>
</div>
<figcaption>Hình 1. Mỗi dịch vụ phát hiện nhìn vào một loại dữ liệu; Security Hub là nơi hội tụ để người trực không phải mở năm giao diện.</figcaption>
</figure>

## CloudTrail: ai làm gì

**Management event** (tạo, sửa, xoá tài nguyên, `AssumeRole`, `GetSecretValue`) được ghi mặc định vào event history 90 ngày. **Data event** (`GetObject`/`PutObject` của S3, `Invoke` của Lambda) tắt mặc định và tính phí theo số event, nên Tới Nơi chỉ bật cho bucket nhạy cảm như `toinoi-prod-delivery-photos`. Giữ lâu hơn 90 ngày hoặc có data event thì cần **trail**. Với nhiều tài khoản, tạo **organization trail** từ management account để mọi tài khoản đẩy log về bucket ở tài khoản log archive, nơi người vận hành workload không xoá được. Bật **log file validation** để CloudTrail ký digest file, cho phép chứng minh log chưa bị sửa.

Trail lưu file trên S3 nên khó truy vấn nhanh. Trước đây câu trả lời là **CloudTrail Lake** (event data store truy vấn bằng SQL), nhưng Lake **đóng với khách mới từ 31/05/2026**: tài khoản đã dùng vẫn chạy tiếp, Lake chỉ còn nhận bản vá nghiêm trọng, và AWS khuyên chuyển sang CloudWatch. Tới Nơi vì thế đẩy trail sang CloudWatch Logs và truy vấn bằng **Logs Insights**, kiểu "ai đọc secret của VíXanh tuần qua" (lab có `queries/who-read-secret.logsinsights`), đổi lại bị tính phí theo dung lượng log được nạp vào và lưu trữ. Cùng log group đó phục vụ cảnh báo tức thời: metric filter và alarm cho vài mẫu kinh điển (root đăng nhập, đổi IAM policy, `StopLogging`, `DeleteFlowLogs`), như lab đã dựng.

## Config: trạng thái và tuân thủ

**AWS Config** ghi lại cấu hình mỗi tài nguyên theo thời gian (configuration item) qua một **recorder**. Hai việc nó làm mà CloudTrail không làm: cho xem **timeline** ("security group này mở cổng 22 từ 14:03 tới 14:40") và **đánh giá rule**. Rule managed như `S3_BUCKET_PUBLIC_READ_PROHIBITED`, `RDS_STORAGE_ENCRYPTED`, `VPC_FLOW_LOGS_ENABLED`, `KMS_CMK_NOT_SCHEDULED_FOR_DELETION` chạy sẵn; rule tự viết dùng Lambda hoặc CloudFormation Guard (bài 30). **Conformance pack** đóng gói nhiều rule thành một bộ cài đồng loạt. Khi rule báo không tuân thủ, **remediation** chạy một **SSM Automation** document để sửa, ví dụ đóng bucket public.

```hcl
resource "aws_config_remediation_configuration" "s3_public_read" {
  config_rule_name = aws_config_config_rule.managed["s3-bucket-public-read-prohibited"].name
  resource_type    = "AWS::S3::Bucket"
  target_type      = "SSM_DOCUMENT"
  target_id        = "AWS-DisableS3BucketPublicReadWrite"
  target_version   = "1"
  automatic        = false
  # parameter AutomationAssumeRole (static) và S3BucketName (RESOURCE_ID) xem trong lab
}
```

`automatic = false` nghĩa là người duyệt bấm chạy. Chỉ tự động hoá những sửa chữa đã diễn tập, vì remediation tự đóng một bucket đang phục vụ ảnh cho app là sự cố do chính công cụ bảo mật gây ra.

## Security Hub: hai nửa

Từ cuối năm 2025 tên **Security Hub** dùng để chỉ phiên bản mới, còn dịch vụ cũ được gọi là **Security Hub CSPM** (cloud security posture management). Hai nửa phục vụ hai việc, và bạn nên tra trang tài liệu gốc để biết tính năng hiện hành vì phần này thay đổi nhanh:

| | Security Hub CSPM | Security Hub (phiên bản mới) |
|---|---|---|
| Việc chính | Kiểm cấu hình theo **chuẩn** (AWS Foundational Security Best Practices, CIS, PCI DSS, NIST 800-53) bằng các control dựa trên Config | Tương quan tín hiệu từ GuardDuty, Inspector, Macie và CSPM thành rủi ro có thứ tự ưu tiên |
| Đầu ra | Finding theo control, điểm tuân thủ | Finding rủi ro tổng hợp, định dạng OCSF |
| Cấu hình | Bật chuẩn, tắt control không phù hợp, automation rule | Bật dịch vụ, nối tài khoản và region, automation rule |
| Terraform | `aws_securityhub_account`, `aws_securityhub_standards_subscription` | `aws_securityhub_account_v2` |

Cả hai hỗ trợ **automation rule**: tự đổi mức độ, thêm ghi chú hoặc gắn nhãn cho finding khớp tiêu chí. Lab có một rule nâng finding `HIGH` trên tài nguyên gắn `Env=prod` lên `CRITICAL`. Với nhiều tài khoản và region, chỉ định một **delegated administrator** và bật tổng hợp (aggregation) về một region để một người trực nhìn một danh sách.

## GuardDuty, Inspector và Macie

**GuardDuty** đọc **CloudTrail management event, VPC Flow Logs và DNS query log** qua đường nội bộ mà bạn không cần bật log. Thêm các **protection plan** khi cần thêm tín hiệu: S3 (data event), EKS audit log, **Runtime Monitoring** (EKS, ECS, EC2), Malware Protection cho EBS và S3, đăng nhập bất thường vào RDS, Lambda network activity. Tên finding cho biết loại hành vi, ví dụ `UnauthorizedAccess:IAMUser/InstanceCredentialExfiltration.OutsideAWS` (credential của instance bị dùng từ ngoài AWS), `CryptoCurrency:EC2/BitcoinTool.B!DNS` (đào coin), `Stealth:IAMUser/CloudTrailLoggingDisabled`.

**Extended Threat Detection** liên kết nhiều tín hiệu rời rạc thành một **attack sequence**, một finding duy nhất mô tả chuỗi hành vi theo thời gian kèm kỹ thuật MITRE ATT&CK (loại finding bắt đầu bằng `AttackSequence:`). Tính năng này tự bật cùng GuardDuty nhưng bao phủ rộng hơn khi bạn bật thêm các protection plan, vì có thêm nguồn để nối. Tra danh sách finding type hiện hành trước khi lọc theo tên.

**Inspector** quét EC2, image ECR và hàm Lambda để tìm lỗ hổng đã biết (CVE) và xếp hạng theo điểm; với ECR, Inspector quét khi push và quét liên tục, bổ sung cho quét cơ bản ở [bài 11](/articles/aws-ecr-image-scan-lifecycle). **Macie** tìm dữ liệu nhạy cảm trong S3 bằng identifier có sẵn và **custom data identifier** do bạn viết bằng regex. Với Tới Nơi, lab thêm hai identifier: số căn cước công dân (12 chữ số bắt đầu bằng 0) và số điện thoại Việt Nam (đầu 0 hoặc +84, theo sau 9 chữ số). Cả hai regex dễ khớp nhầm, nên dùng kèm `keywords` và kiểm tra trên mẫu thật trước khi tin kết quả.

## Quy trình xử lý sự cố

Phát hiện chỉ có ý nghĩa khi có người biết làm gì tiếp. Tới Nơi dùng vòng đời của NIST (SP 800-61): chuẩn bị, phát hiện và phân tích, cô lập, loại bỏ mối đe doạ, phục hồi, rút kinh nghiệm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Sáu bước xử lý sự cố theo NIST, với hành động cụ thể cho playbook access key bị lộ.">
<defs><marker id="aws54-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="10" y="40" width="130" height="100" rx="10" class="f-surface s-line"/><text x="22" y="66" class="t-b">1 Chuẩn bị</text><text x="22" y="88" class="t-small">alarm, quyền break-glass,</text><text x="22" y="104" class="t-small">playbook, liên lạc</text>
<rect x="154" y="40" width="130" height="100" rx="10" class="f-pk-soft s-pk"/><text x="166" y="66" class="t-b">2 Phát hiện</text><text x="166" y="88" class="t-small">GuardDuty báo key</text><text x="166" y="104" class="t-small">dùng từ IP lạ</text>
<rect x="298" y="40" width="130" height="100" rx="10" class="f-warn-soft s-warn"/><text x="310" y="66" class="t-b t-warn">3 Cô lập</text><text x="310" y="88" class="t-small">đặt key Inactive,</text><text x="310" y="104" class="t-small">chặn bằng Deny</text>
<rect x="442" y="40" width="130" height="100" rx="10" class="f-warn-soft s-warn"/><text x="454" y="66" class="t-b t-warn">4 Loại bỏ</text><text x="454" y="88" class="t-small">xoá user, role, key</text><text x="454" y="104" class="t-small">do kẻ tấn công tạo</text>
<rect x="586" y="40" width="130" height="100" rx="10" class="f-phys-soft s-phys"/><text x="598" y="66" class="t-b">5 Phục hồi</text><text x="598" y="88" class="t-small">xoay secret, cấp</text><text x="598" y="104" class="t-small">credential mới</text>
<rect x="730" y="40" width="130" height="100" rx="10" class="f-phys-soft s-phys"/><text x="742" y="66" class="t-b">6 Rút kinh nghiệm</text><text x="742" y="88" class="t-small">postmortem, thu hẹp</text><text x="742" y="104" class="t-small">quyền, thêm rule</text>
<path d="M140 90 L152 90 M284 90 L296 90 M428 90 L440 90 M572 90 L584 90 M716 90 L728 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws54-2-a)"/>
<text x="10" y="190" class="t-small">Khoanh vùng trước khi xoá: key chỉ đặt Inactive để còn điều tra; xoá sau khi đóng sự cố.</text>
<text x="10" y="212" class="t-small">Script access-key-triage.sh trong lab làm bước 2 và 3: lần dùng cuối, sự kiện CloudTrail, lệnh cô lập (mặc định chỉ in).</text>
</svg>
</div>
<figcaption>Hình 2. Vòng đời xử lý sự cố; bước 3 thường làm trong vài phút, bước 4 và 5 mới tốn thời gian.</figcaption>
</figure>

Playbook cho **access key bị lộ** gồm các bước sau, theo thứ tự: (1) xác nhận key có thật và của ai bằng `aws iam get-access-key-last-used`; (2) đặt key `Inactive` bằng `aws iam update-access-key`, không xoá; (3) tra CloudTrail theo `AccessKeyId` để biết nó đã gọi gì, từ IP nào, ở region nào, có tạo user, role hay tài nguyên lạ không; (4) gỡ những thứ kẻ tấn công đã tạo; (5) xoay mọi secret mà key đó từng đọc được (bài 14); (6) viết postmortem và thu hẹp quyền (bài 51). Mỗi bước cần có người phụ trách, và phải diễn tập playbook một lần trước khi cần dùng.

## Cái giá phải trả

- **Log là dữ liệu nhạy cảm**: bucket log ở tài khoản riêng, hạn chế người đọc, bật Object Lock (bài 36) nếu cần bất biến.
- **Chi phí theo khối lượng**: data event, finding và đánh giá rule đều tính theo số lượng. Bật data event cho bucket cần thiết, không phải mọi bucket.
- **Finding fatigue**: bật mọi chuẩn và control sẽ tạo hàng nghìn finding không ai đọc. Bắt đầu bằng control nghiêm trọng, giảm bớt nhiễu bằng automation rule.
- **Remediation tự động có thể gây sự cố**: hãy để ở chế độ chờ duyệt cho tới khi đã diễn tập.
- **Chỉ bật một region**: kẻ tấn công thích region bạn không nhìn. Bật ở mọi region đang dùng và dùng SCP chặn region không dùng (bài 28).

## Bài tập

**Bài tập:** sáng nay bạn phát hiện VPC Flow Logs của VPC `prod` đã bị xoá. Tìm ai làm, từ đâu, và đề xuất hai biện pháp để chuyện này không lặp lại.

<details><summary>Xem đáp án</summary><p><code>DeleteFlowLogs</code> là management event của EC2. Chạy <code>aws cloudtrail lookup-events --lookup-attributes AttributeKey=EventName,AttributeValue=DeleteFlowLogs</code> ở từng region (hoặc Logs Insights trên log group của trail nếu quá 90 ngày; lab có <code>who-disabled-flow-logs.sh</code> và <code>queries/who-deleted-flow-logs.logsinsights</code>): kết quả cho biết thời gian, principal, access key, IP nguồn và user agent. Rule Config <code>VPC_FLOW_LOGS_ENABLED</code> cho biết VPC chuyển sang không tuân thủ vào lúc nào. Principal quen thuộc thì hỏi người phụ trách role; IP và region lạ thì chuyển sang playbook cho access key bị lộ. Phòng ngừa: SCP Deny <code>ec2:DeleteFlowLogs</code> trừ role platform (bài 28), alarm CloudTrail cho <code>DeleteFlowLogs</code> như trong lab, và remediation Config bật lại Flow Logs.</p></details>

## Lab: lớp phát hiện cho một tài khoản

Lab ở `examples/toi-noi-aws/p06-54-detection` có Terraform dựng trail, bốn cảnh báo, Config với chín rule và một remediation chờ duyệt, GuardDuty, Security Hub (mới và CSPM), Inspector, Macie tuỳ chọn, cùng truy vấn Logs Insights và hai script điều tra. Các dịch vụ này tính phí theo khối lượng; README liệt kê từng khoản và công tắc tắt. Dọn bằng `terraform destroy`.

```bash
cd examples/toi-noi-aws/p06-54-detection
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
./access-key-triage.sh <access-key-id> <iam-user> toinoi-lab
```

Terraform đã qua `fmt` và `validate` bằng Docker; script đã qua `bash -n` và chạy thử với lệnh `aws` giả lập. **Chưa chạy `apply` hay bất kỳ lệnh AWS thật nào.** Đã đối chiếu tài liệu: tham số của runbook `AWS-DisableS3BucketPublicReadWrite`, và việc Security Hub mới với CSPM cùng tồn tại trong một tài khoản. Org trail và delegated admin chỉ được mô tả.

Bài trước: [bài 53](/articles/aws-privatelink-transit-gateway-network-firewall). Bài tiếp: [bài 55](/articles/aws-observability-nang-cao-xray-adot-logs-insights-slo).

## Tài liệu gốc

- [AWS CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-user-guide.html).
- [AWS Config Developer Guide](https://docs.aws.amazon.com/config/latest/developerguide/WhatIsConfig.html).
- [AWS Security Hub (phiên bản mới)](https://docs.aws.amazon.com/securityhub/latest/userguide/what-is-securityhub-v2.html) và [Security Hub CSPM](https://docs.aws.amazon.com/securityhub/latest/userguide/what-is-securityhub.html).
- [Amazon GuardDuty User Guide](https://docs.aws.amazon.com/guardduty/latest/ug/what-is-guardduty.html).
- [Amazon Inspector User Guide](https://docs.aws.amazon.com/inspector/latest/user/what-is-inspector.html).
- [Amazon Macie User Guide](https://docs.aws.amazon.com/macie/latest/user/what-is-macie.html).

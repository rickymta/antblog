---
slug: aws-kms-envelope-encryption-key-policy
title: "KMS: envelope encryption, key policy, grant, multi-region key và mã hoá S3, RDS, EBS, Secrets"
summary: "Bài 52: KMS key và ai được dùng nó (key policy khác IAM policy), envelope encryption với data key và encryption context, grant cho dịch vụ AWS, rotation, multi-region key cho DR; bật mã hoá S3 SSE-KMS với bucket key, RDS, EBS, Secrets Manager, CloudWatch Logs; chi phí request và giới hạn."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, kms, encryption, key-policy, envelope-encryption, security, compliance]
language: vi
---

Kiểm toán viên của đối tác thanh toán hỏi Tới Nơi ba câu: ảnh chứng từ và hoá đơn của khách được mã hoá bằng khoá của ai, ai xem được dữ liệu thô, và nếu một tài xế bị lộ tài khoản thì có biết nó đã giải mã những gì không. Mã hoá mặc định của S3 trả lời được câu đầu một nửa, còn hai câu sau cần khoá do chính Tới Nơi kiểm soát: tách người quản trị khoá khỏi người dùng khoá, ghi từng lần giải mã vào CloudTrail, và có nút thu hồi.

Đây là **bài 52** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần xong [bài 51](/articles/aws-iam-nang-cao-permission-boundary-abac-access-analyzer) và đã gặp KMS key ở [bài 14](/articles/aws-secrets-manager-parameter-store). Bài này giải thích cơ chế của **AWS KMS (Key Management Service)** rồi bật mã hoá cho S3, RDS, EBS, Secrets Manager và CloudWatch Logs.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Role <code>booking</code> có IAM policy <code>Allow kms:Decrypt</code> trên một key, nhưng key policy của key đó không nhắc gì tới tài khoản hay role. Role giải mã được không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Key policy là gốc của quyền trên KMS key. IAM policy chỉ có hiệu lực khi key policy cho phép tài khoản (thường bằng statement uỷ quyền cho IAM) hoặc nêu đích danh principal.</p></details>
</div>

## Ba loại key và ai sở hữu

| Loại | Ai tạo và quản | Bạn thấy gì | Dùng khi |
|---|---|---|---|
| AWS owned key | AWS, dùng chung nhiều khách | Không thấy, không có log CloudTrail | Mã hoá mặc định của một số dịch vụ |
| AWS managed key (`aws/s3`, `aws/rds`...) | AWS tạo trong tài khoản bạn | Thấy key, đọc được log; không sửa policy, tự xoay vòng hằng năm | Mã hoá cơ bản, không cần chia sẻ |
| Customer managed key | Bạn | Toàn quyền: key policy, rotation, alias, xoá, dùng chéo tài khoản | Dữ liệu khách của Tới Nơi |

Tới Nơi dùng customer managed key (CMK) cho dữ liệu khách vì chỉ loại này cho phép tách quyền, chia sẻ snapshot hoặc secret sang tài khoản khác và thu hồi. Key đối xứng (AES-256-GCM) không bao giờ rời KMS ở dạng plaintext; KMS còn có key bất đối xứng (RSA, ECC để ký và mã hoá) và HMAC key cho các việc khác.

## Key policy là gốc, IAM là phụ

Mọi KMS key đều có **key policy**, một resource policy gắn thẳng vào key. Quyền dùng key đến theo ba đường:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Request tới KMS luôn qua key policy; quyền đến từ ba đường: key policy nêu trực tiếp principal, key policy uỷ quyền cho IAM kèm IAM policy, hoặc grant tạm thời cho dịch vụ AWS.">
<defs><marker id="aws52-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="85" width="130" height="70" rx="10" class="f-surface s-line"/><text x="34" y="115" class="t-b">Request</text><text x="34" y="137" class="t-small">Decrypt, Generate…</text>
<rect x="200" y="85" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="214" y="115" class="t-b">Key policy</text><text x="214" y="137" class="t-small">luôn được xét</text>
<rect x="420" y="20" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="434" y="46" class="t-b">1. Nêu trực tiếp principal</text><text x="434" y="66" class="t-small">kèm ViaService, EncryptionContext</text>
<rect x="420" y="95" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="434" y="121" class="t-b">2. Uỷ quyền cho IAM</text><text x="434" y="141" class="t-small">cần thêm Allow ở IAM policy của role</text>
<rect x="420" y="170" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="434" y="196" class="t-b">3. Grant tạm thời</text><text x="434" y="216" class="t-small">EBS, RDS: tạo tự động, thu hồi được</text>
<rect x="730" y="85" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="744" y="115" class="t-b t-phys">Dùng key</text><text x="744" y="137" class="t-small">và ghi CloudTrail</text>
<path d="M150 120 L198 120" class="f-none s-phys" stroke-width="2" marker-end="url(#aws52-1-a)"/>
<path d="M350 105 L418 55 M350 120 L418 125 M350 135 L418 200" class="f-none s-phys" stroke-width="2" marker-end="url(#aws52-1-a)"/>
<path d="M670 50 L728 105 M670 125 L728 120 M670 200 L728 140" class="f-none s-phys" stroke-width="2" marker-end="url(#aws52-1-a)"/>
</svg>
</div>
<figcaption>Hình 1. Ba đường cấp quyền trên KMS key; mọi đường đều bắt đầu từ key policy, và một Deny ở bất kỳ policy nào vẫn thắng.</figcaption>
</figure>

Key policy mặc định của console có statement `Principal: arn:aws:iam::123456789012:root, Action: kms:*`. Đây không phải "user root" mà là **uỷ quyền cho IAM** (đường 2). Bỏ statement này thì IAM policy của cả tài khoản không còn tác dụng với key, và nếu key policy không nêu ai khác thì key có thể thành không ai quản trị được. Lab giữ statement đó rồi thêm ba lớp kiểm soát hẹp hơn:

```hcl
statement {
  sid       = "UseKeyOnlyViaS3AndSecrets"
  effect    = "Allow"
  actions   = ["kms:Encrypt", "kms:Decrypt", "kms:ReEncrypt*", "kms:GenerateDataKey*", "kms:DescribeKey"]
  resources = ["*"]

  principals {
    type        = "AWS"
    identifiers = ["*"]
  }

  condition {
    test     = "StringEquals"
    variable = "kms:CallerAccount"
    values   = [local.account_id]
  }

  condition {
    test     = "StringEquals"
    variable = "kms:ViaService"
    values   = ["s3.ap-southeast-1.amazonaws.com", "secretsmanager.ap-southeast-1.amazonaws.com"]
  }
}
```

Statement này cho mọi principal trong tài khoản dùng key, nhưng chỉ khi request đi qua S3 hoặc Secrets Manager (`kms:ViaService`), không gọi KMS trực tiếp. Hai statement còn lại trong lab tách **key administrator** (tạo, tắt, đặt lịch xoá, sửa policy, không có `Encrypt` hay `Decrypt`) khỏi **key user** của service booking (`GenerateDataKey` và `Decrypt` chỉ khi `kms:EncryptionContext:Service` bằng `booking`). Người quản trị khoá vì thế không đọc được dữ liệu, đúng điều kiểm toán viên muốn.

## Envelope encryption và encryption context

KMS `Encrypt` chỉ nhận tối đa 4 KB plaintext, và gửi cả ảnh 3 MB qua mạng tới KMS cho mỗi lần mã hoá là lãng phí. Giải pháp là **envelope encryption**: KMS giữ khoá bọc (key-encrypting key), ứng dụng mã hoá dữ liệu bằng một **data key** sinh theo từng đối tượng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Envelope encryption: ứng dụng xin data key từ KMS, mã hoá dữ liệu cục bộ, xoá plaintext key, lưu ciphertext cùng data key đã mã hoá; đọc lại thì nhờ KMS giải mã data key với đúng encryption context.">
<defs><marker id="aws52-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="50" width="190" height="110" rx="10" class="f-surface s-line"/><text x="34" y="80" class="t-b">Service booking</text><text x="34" y="104" class="t-small">mã hoá và giải mã</text><text x="34" y="122" class="t-small">dữ liệu tại chỗ (AES-GCM)</text>
<rect x="350" y="30" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="364" y="60" class="t-b">KMS key (CMK)</text><text x="364" y="82" class="t-small">không rời KMS</text>
<rect x="680" y="50" width="180" height="110" rx="10" class="f-phys-soft s-phys"/><text x="694" y="80" class="t-b">S3</text><text x="694" y="104" class="t-small">ciphertext của ảnh</text><text x="694" y="122" class="t-small">+ data key đã mã hoá</text>
<path d="M210 70 L348 60" class="f-none s-phys" stroke-width="2" marker-end="url(#aws52-2-a)"/>
<text x="228" y="52" class="t-small">1 GenerateDataKey + context</text>
<path d="M348 90 L212 100" class="f-none s-phys dash" stroke-width="2" marker-end="url(#aws52-2-a)"/>
<text x="228" y="122" class="t-small">2 plaintext key + bản mã hoá</text>
<path d="M210 140 L678 140" class="f-none s-phys" stroke-width="2" marker-end="url(#aws52-2-a)"/>
<text x="330" y="158" class="t-small">3 xoá plaintext key; 4 lưu hai thứ cạnh nhau</text>
<text x="20" y="205" class="t-b">Đọc lại</text>
<text x="20" y="226" class="t-small">Lấy data key đã mã hoá cùng object → kms:Decrypt với đúng encryption context → plaintext key → giải mã cục bộ.</text>
<text x="20" y="247" class="t-small">Encryption context hiện trong CloudTrail dạng chữ rõ: dùng để audit, không bao giờ để secret vào đó.</text>
</svg>
</div>
<figcaption>Hình 2. Dữ liệu lớn không đi qua KMS; KMS chỉ thấy data key nhỏ và encryption context.</figcaption>
</figure>

**Encryption context** là cặp khoá-giá trị đi kèm lúc mã hoá và phải khớp y hệt lúc giải mã; nó đóng vai trò dữ liệu xác thực bổ sung (AAD). Ví dụ `Service=booking, Purpose=receipt` gắn ciphertext với đúng mục đích: lấy data key của hoá đơn đem đi giải mã với context của service khác sẽ bị từ chối. Context cũng xuất hiện trong CloudTrail nên đủ để trả lời "ai giải mã ảnh nào", và dùng được trong key policy (`kms:EncryptionContext:Service`). Script `envelope-demo.sh` trong lab chạy đúng vòng này và thử giải mã với context sai. Trong code thật, đừng tự ghép openssl: dùng **AWS Encryption SDK** (có sẵn cho .NET), nó lo định dạng, AES-GCM và cache data key.

## Rotation, multi-region key và alias

**Rotation tự động** của CMK đối xứng tạo material mới theo chu kỳ (mặc định 365 ngày, đặt được từ 90 đến 2.560 ngày). ID và ARN của key giữ nguyên, material cũ được giữ để giải mã dữ liệu đã mã hoá, nhưng dữ liệu cũ **không** bị mã hoá lại. Key dùng material tự import (`EXTERNAL`) không có rotation tự động; bạn import material mới rồi gọi rotation theo yêu cầu (`rotate-key-on-demand`, hỗ trợ cả key `EXTERNAL` từ 2025). Key bất đối xứng, HMAC và key trong custom key store chỉ xoay được bằng cách tạo key mới và đổi alias.

**Multi-region key** là một họ key có cùng ID (bắt đầu bằng `mrk-`) và cùng material ở nhiều region. Ciphertext mã hoá ở `ap-southeast-1` giải mã được bằng bản replica ở `ap-northeast-1` mà không cần gọi chéo region, hữu ích cho mã hoá phía client và dịch vụ hỗ trợ (secret DR của [bài 37](/articles/aws-backup-disaster-recovery-rpo-rto)). Mỗi replica có key policy, alias và grant riêng, nên phải chỉnh lại policy cho region đích. Với S3 replication, S3 tự mã hoá lại bằng key đích bạn chỉ định, nên multi-region key tiện nhưng không bắt buộc. Cờ `multi_region` đặt lúc tạo và không đổi sau đó.

**Alias** (`alias/toinoi-prod-trips`) là tên ổn định để code và policy tham chiếu; đổi key phía sau mà không sửa code. Policy IAM theo alias không thay cho quyền trên key policy.

## Bật mã hoá cho từng dịch vụ

| Dịch vụ | Cách bật | Lưu ý |
|---|---|---|
| S3 | Default encryption `aws:kms` với CMK, `bucket_key_enabled = true` | Bucket key giảm mạnh số request tới KMS (và chi phí); CloudTrail khi đó ghi context là ARN bucket thay vì object |
| RDS, Aurora | Chọn key **lúc tạo** instance hoặc cluster | Không bật tại chỗ cho DB đang chạy: snapshot, copy snapshot với key, rồi restore |
| EBS | Chọn key khi tạo volume, hoặc bật mặc định cả region (`aws ec2 enable-ebs-encryption-by-default`) | Volume có sẵn cũng phải qua snapshot copy |
| Secrets Manager | `--kms-key-id` khi tạo secret | Muốn chia sẻ chéo tài khoản phải dùng CMK, không dùng `aws/secretsmanager` |
| CloudWatch Logs | `kms_key_id` của log group | Key policy phải cho service `logs.<region>.amazonaws.com`, giới hạn bằng context ARN log group |

RDS và EBS dùng **grant**: khi bạn tạo volume mã hoá, EC2 tạo một grant tạm thời cho phép nó giải mã data key cho volume đó, và thu hồi khi volume xoá. Vì thế key policy phải cho `kms:CreateGrant` với điều kiện `kms:GrantIsForAWSResource`. Chia sẻ snapshot RDS hay EBS mã hoá sang tài khoản khác chỉ làm được khi nó dùng CMK (không phải key AWS managed); phải chia sẻ cả quyền dùng key cho tài khoản kia, tài khoản đích nên copy snapshot và mã hoá lại bằng key của chính nó.

## Cái giá phải trả

- **Phí và request**: mỗi key tính phí theo tháng, mỗi request tính theo số lượng, và request tới KMS dùng chung hạn mức mỗi region cho cả tài khoản (xem Service Quotas). Tracking nhận 6.250 toạ độ mỗi giây: gọi KMS một lần cho mỗi toạ độ sẽ chiếm phần lớn hạn mức bạn không kiểm soát. Lấy một data key cho cả lô, cache data key trong Encryption SDK, hoặc để S3 và Firehose lo.
- **Khoá nhầm thì không mở lại**: bỏ statement uỷ quyền IAM mà quên nêu admin khiến key khó quản trị.
- **Tách quyền chưa đủ**: nếu admin tự sửa được key policy, họ tự cấp quyền dùng cho mình. Giới hạn ai sửa policy qua SCP hoặc bài 51.
- **Xoá key = mất dữ liệu**: không có đường quay lại sau cửa sổ chờ. Tắt key trước, xoá sau.
- **Quên region**: key ở `ap-southeast-1` không dùng được ở region khác, trừ multi-region key đã replicate.

## Bài tập

**Bài tập:** bạn chạy `ScheduleKeyDeletion` với cửa sổ chờ 7 ngày cho key đang mã hoá instance RDS `trips`. Chuyện gì xảy ra ngay và sau 7 ngày?

<details><summary>Xem đáp án</summary><p>Ngay lúc đặt lịch, key chuyển sang trạng thái <b>Pending deletion</b> và không dùng được: mọi lần <code>Decrypt</code> và <code>GenerateDataKey</code> bị từ chối. RDS mất quyền dùng key: với instance có backup, khoảng hai giờ sau khi phát hiện nó chuyển sang <code>inaccessible-encryption-credentials-recoverable</code>; nếu không khôi phục (bật lại key rồi khởi động instance) trong bảy ngày, nó rơi vào trạng thái cuối <code>inaccessible-encryption-credentials</code> và chỉ còn đường restore từ snapshot, vốn cũng cần chính key đó. Trong 7 ngày bạn còn <code>CancelKeyDeletion</code> (key quay về trạng thái Disabled, phải bật lại). Hết 7 ngày key bị xoá vĩnh viễn: dữ liệu, kể cả backup, không đọc được nữa. Cách đúng là disable key, theo dõi CloudTrail xem còn ai gọi nó, rồi mới xoá.</p></details>

## Lab: key, S3 SSE-KMS và envelope encryption

Lab ở `examples/toi-noi-aws/p06-52-kms` có Terraform dựng một CMK đối xứng (rotation 365 ngày, multi-region), replica ở `ap-northeast-1` với key policy riêng ghi đúng region DR, alias, key policy tách admin và user, bucket SSE-KMS với bucket key kèm bucket policy chặn upload chọn thuật toán hoặc key khác, một log group mã hoá, và `envelope-demo.sh`. Bẫy đáng nhớ: Deny với điều kiện phủ định, kể cả `StringNotEqualsIfExists`, vẫn khớp khi header vắng mặt, nên mỗi statement phải kèm `Null = false` để upload không gửi header vẫn dùng mã hoá mặc định của bucket. Key tính phí theo tháng (primary và replica) và vào trạng thái chờ xoá sau `terraform destroy` (key đang chờ xoá không bị tính phí lưu trữ); README liệt kê từng khoản.

```bash
cd examples/toi-noi-aws/p06-52-kms
bash -n envelope-demo.sh
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
./envelope-demo.sh alias/toinoi-lab-trips toinoi-lab
```

Terraform đã qua `fmt` và `validate` bằng Docker; script đã qua `bash -n` và chạy thử với lệnh `aws` giả lập. **Chưa chạy `apply` hay gọi KMS trên tài khoản AWS thật.**

Bài trước: [bài 51](/articles/aws-iam-nang-cao-permission-boundary-abac-access-analyzer). Bài tiếp: [bài 53](/articles/aws-privatelink-transit-gateway-network-firewall).

## Tài liệu gốc

- [AWS Key Management Service Developer Guide](https://docs.aws.amazon.com/kms/latest/developerguide/overview.html).
- [Key policies in AWS KMS](https://docs.aws.amazon.com/kms/latest/developerguide/key-policies.html).
- [Rotating AWS KMS keys](https://docs.aws.amazon.com/kms/latest/developerguide/rotate-keys.html).
- [Multi-Region keys in AWS KMS](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html).
- [Reducing the cost of SSE-KMS with Amazon S3 Bucket Keys](https://docs.aws.amazon.com/AmazonS3/latest/userguide/bucket-key.html).
- [AWS Encryption SDK](https://docs.aws.amazon.com/encryption-sdk/latest/developer-guide/introduction.html).

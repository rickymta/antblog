---
slug: aws-iam-user-role-policy
title: "IAM: user, group, role và policy, quyền tối thiểu cho Tới Nơi"
summary: "Bài 03: đọc một IAM policy JSON (Effect, Action, Resource, Condition), phân biệt identity-based và resource-based policy, hiểu role và trust policy, rồi viết policy quyền tối thiểu cho service upload ảnh giao hàng lên S3."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, iam, policy, role, least-privilege, trust-policy]
language: vi
---

Khi tài xế giao xong một đơn, app chụp ảnh chứng từ và gửi lên service `delivery-photo` của [Tới Nơi](/articles/du-an-mau-toi-noi), service này ghi ảnh vào S3. Lần đầu chạy thử, nó bị `AccessDenied`. Một đồng nghiệp đề nghị: "Gắn `AdministratorAccess` cho nhanh, lúc nào ổn rồi thu hẹp sau." Cái "sau" đó hiếm khi đến, và một lỗi trong service nhỏ này có thể dẫn tới việc xoá database hay tắt cả hệ thống.

Đây là **bài 03** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần đọc [bài 02](/articles/aws-tai-khoan-iam-identity-center-cli) và có profile `toinoi-lab` chạy được. Bài này giải thích IAM từ đầu rồi viết một policy vừa đủ cho `delivery-photo`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một role có policy <code>Allow s3:*</code> trên mọi resource và một policy khác <code>Deny s3:DeleteObject</code>. Role đó xoá được object S3 không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Trong IAM, explicit Deny luôn thắng mọi Allow, bất kể thứ tự hay số lượng policy.</p></details>
</div>

## Ai gọi API: user, group và role

**IAM (Identity and Access Management)** trả lời hai câu hỏi cho mọi lời gọi API: *bạn là ai* (xác thực) và *bạn được làm gì* (phân quyền). "Bạn" ở đây gọi là **principal**.

| Khái niệm | Là gì | Credential | Dùng cho |
|---|---|---|---|
| IAM user | Danh tính cố định gắn với một người hoặc ứng dụng | Mật khẩu và access key dài hạn | Hạn chế dùng; người dùng nên đăng nhập qua Identity Center |
| IAM group | Tập hợp user để gán quyền chung | Không có | Gom user theo vai trò |
| IAM role | Danh tính **không có credential riêng**, ai được phép thì *assume* (đảm nhận) | Tạm thời, hết hạn | Workload (EC2, ECS, Lambda), người đăng nhập SSO, truy cập chéo tài khoản |

Quy tắc đơn giản cho Tới Nơi: **người dùng đăng nhập bằng SSO, workload dùng role, không phát access key dài hạn**. Role của `delivery-photo` sẽ được gắn vào máy chạy service, nên mã nguồn không chứa khoá nào.

## Đọc một policy

Quyền được viết trong **policy**, một tài liệu JSON. Đây là cấu trúc bạn sẽ đọc suốt 60 bài:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "ProofObjectsReadWrite",
      "Effect": "Allow",
      "Action": ["s3:PutObject", "s3:GetObject"],
      "Resource": "arn:aws:s3:::toinoi-lab-delivery-photos/proof/*"
    }
  ]
}
```

| Phần tử | Ý nghĩa |
|---|---|
| `Version` | Phiên bản ngôn ngữ policy; luôn dùng `2012-10-17` |
| `Statement` | Danh sách các câu lệnh quyền |
| `Sid` | Nhãn tuỳ chọn để đọc cho dễ |
| `Effect` | `Allow` hoặc `Deny` |
| `Action` | API được phép hoặc bị cấm, dạng `dịch-vụ:hành-động` |
| `Resource` | **ARN** của đối tượng bị tác động |
| `Principal` | Chỉ có ở resource-based policy: ai được hưởng quyền |
| `Condition` | Điều kiện phụ: IP nguồn, MFA, tag, prefix… |

**ARN (Amazon Resource Name)** là tên duy nhất của một tài nguyên. Ở trên, `arn:aws:s3:::toinoi-lab-delivery-photos/proof/*` nghĩa là mọi object có key bắt đầu bằng `proof/` trong bucket đó; S3 không có region hay account trong ARN nên hai phần giữa để trống. Ngược lại, một role có ARN `arn:aws:iam::123456789012:role/toinoi-lab-delivery-photo`.

Policy gắn vào hai nơi khác nhau:

- **Identity-based policy**: gắn vào user, group hoặc role, nói "danh tính này được làm gì".
- **Resource-based policy**: gắn vào tài nguyên (bucket policy của S3, queue policy của SQS, key policy của KMS), nói "ai được làm gì với tài nguyên này" nên có `Principal`.

Trong cùng một tài khoản, với đa số dịch vụ (như S3, SQS) chỉ cần một trong hai cho phép là đủ, miễn không bị Deny hay giới hạn bởi lớp khác. Có ngoại lệ quan trọng: trust policy của role và key policy của KMS phải tự cho phép, identity-based policy thôi chưa đủ. Khi truy cập chéo tài khoản, cần cả hai phía đồng ý.

## AWS quyết định cho hay chặn như thế nào

Mặc định mọi thứ bị **từ chối ngầm (implicit deny)**. Một request chỉ qua được khi có ít nhất một Allow phù hợp và không có Deny nào chặn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Sơ đồ quyết định rút gọn: nếu có explicit deny thì từ chối; nếu không, có allow phù hợp thì cho phép, không có thì từ chối ngầm.">
<defs><marker id="aws03-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="75" width="150" height="70" rx="10" class="f-surface s-line"/>
<text x="34" y="105" class="t-b">Request</text>
<text x="34" y="126" class="t-small">PutObject lên S3</text>
<rect x="250" y="75" width="170" height="70" rx="10" class="f-pk-soft s-pk"/>
<text x="264" y="105" class="t-b">Có Deny khớp?</text>
<text x="264" y="126" class="t-small">xét mọi loại policy</text>
<rect x="520" y="75" width="170" height="70" rx="10" class="f-pk-soft s-pk"/>
<text x="534" y="105" class="t-b">Có Allow khớp?</text>
<text x="534" y="126" class="t-small">và không bị chặn trần</text>
<rect x="740" y="20" width="120" height="60" rx="10" class="f-phys-soft s-phys"/>
<text x="754" y="56" class="t-b t-phys">CHO PHÉP</text>
<rect x="740" y="140" width="120" height="60" rx="10" class="f-warn-soft s-warn"/>
<text x="754" y="176" class="t-b t-warn">TỪ CHỐI</text>
<path d="M170 110 L248 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws03-1-a)"/>
<path d="M420 110 L518 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws03-1-a)"/>
<text x="436" y="100" class="t-small">không</text>
<path d="M690 100 L738 60" class="f-none s-phys" stroke-width="2" marker-end="url(#aws03-1-a)"/>
<text x="680" y="70" class="t-small">có</text>
<path d="M690 125 L738 160" class="f-none s-warn" stroke-width="2" marker-end="url(#aws03-1-a)"/>
<text x="690" y="165" class="t-small">không</text>
<path d="M335 145 L335 185 L738 185" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws03-1-a)"/>
<text x="350" y="178" class="t-small">có Deny: từ chối ngay, không xét Allow</text>
</svg>
</div>
<figcaption>Hình 1. Logic đánh giá rút gọn: Deny thắng, rồi tới Allow, còn lại là từ chối ngầm. "Bị chặn trần" nói tới permission boundary, SCP và session policy mà Phần 03 và Phần 06 sẽ đề cập; logic đầy đủ nằm trong tài liệu policy evaluation của AWS.</figcaption>
</figure>

Hệ quả thực tế: để cấm một việc chắc chắn, dùng Deny; để mở một việc, chỉ cần thêm đúng Allow. Đừng viết hàng loạt Allow rộng rồi cố vá bằng Deny.

## Role, trust policy và credential tạm thời

Một role có **hai policy khác nhau về vai trò**:

- **Trust policy** (một resource-based policy gắn vào role): trả lời *ai được phép assume role này*. Ví dụ "dịch vụ EC2".
- **Permission policy** (identity-based): trả lời *khi đã là role này thì được làm gì*.

Trust policy cho role của `delivery-photo`, cho phép EC2 assume:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "EC2AssumeRole",
      "Effect": "Allow",
      "Principal": { "Service": "ec2.amazonaws.com" },
      "Action": "sts:AssumeRole"
    }
  ]
}
```

Khi máy chạy, dịch vụ EC2 thay mặt service gọi **STS** (Security Token Service) `AssumeRole` và nhận một bộ credential gồm access key id, secret và session token, **tự hết hạn** (mặc định một giờ với AssumeRole thông thường; trên EC2 credential được làm mới tự động, bài 05 nói chi tiết). Mã nguồn dùng AWS SDK sẽ tự lấy credential này, không có file khoá nào để lộ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Máy EC2 chạy delivery-photo nhờ STS đảm nhận role sau khi trust policy cho phép, nhận credential tạm, rồi gọi S3 và permission policy quyết định quyền.">
<defs><marker id="aws03-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="70" width="170" height="80" rx="10" class="f-surface s-line"/>
<text x="34" y="102" class="t-b">Service</text>
<text x="34" y="124" class="t-small t-mono">delivery-photo</text>
<rect x="270" y="70" width="160" height="80" rx="10" class="f-pk-soft s-pk"/>
<text x="284" y="102" class="t-b">STS AssumeRole</text>
<text x="284" y="124" class="t-small">kiểm trust policy</text>
<rect x="510" y="70" width="160" height="80" rx="10" class="f-phys-soft s-phys"/>
<text x="524" y="102" class="t-b">Credential tạm</text>
<text x="524" y="124" class="t-small">hết hạn tự động</text>
<rect x="740" y="70" width="120" height="80" rx="10" class="f-phys-soft s-phys"/>
<text x="754" y="102" class="t-b">S3</text>
<text x="754" y="124" class="t-small">permission policy</text>
<path d="M190 110 L268 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws03-2-a)"/>
<path d="M430 110 L508 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws03-2-a)"/>
<path d="M670 110 L738 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws03-2-a)"/>
<text x="200" y="100" class="t-small">1</text>
<text x="440" y="100" class="t-small">2</text>
<text x="690" y="100" class="t-small">3</text>
<text x="20" y="190" class="t-small">1: xin đảm nhận role. 2: STS cấp credential nếu trust policy cho phép. 3: gọi S3; permission policy quyết định được làm gì.</text>
</svg>
</div>
<figcaption>Hình 2. Hai cửa kiểm tra của role: trust policy ở bước 2 (ai được vào) và permission policy ở bước 3 (vào rồi được làm gì).</figcaption>
</figure>

Trust policy cũng có thể dùng `Condition`. Ví dụ dưới đây cho phép principal trong tài khoản assume role chỉ khi đã đăng nhập bằng MFA:

```json
{
  "Effect": "Allow",
  "Principal": { "AWS": "arn:aws:iam::123456789012:root" },
  "Action": "sts:AssumeRole",
  "Condition": { "Bool": { "aws:MultiFactorAuthPresent": "true" } }
}
```

## Managed policy hay inline

| Loại | Ai viết | Dùng lại được | Ghi chú |
|---|---|---|---|
| AWS managed | AWS | Có | Tiện nhưng thường rộng; ví dụ `AdministratorAccess`, `AmazonS3FullAccess` |
| Customer managed | Bạn | Có, có phiên bản | Lựa chọn mặc định cho quyền của workload |
| Inline | Bạn | Không, gắn cứng vào một identity | Chỉ khi cần ràng buộc 1-1 |

**Không dùng `AdministratorAccess` cho workload.** Quyền của một service nên chỉ gồm những lời gọi API nó thật sự thực hiện, trên đúng tài nguyên nó cần. Đây là **least privilege**.

## Quyền tối thiểu cho `delivery-photo`

Policy ở phần đầu bài cho phép ghi và đọc đúng prefix `proof/`, không xoá, không động tới các prefix khác. Policy này chỉ thiếu quyền liệt kê object. Hành động `s3:ListBucket` áp dụng lên **bucket**, không phải object, nên cần statement riêng với `Resource` là ARN bucket và một `Condition` giới hạn prefix:

```json
{
  "Sid": "ListProofPrefixOnly",
  "Effect": "Allow",
  "Action": "s3:ListBucket",
  "Resource": "arn:aws:s3:::toinoi-lab-delivery-photos",
  "Condition": { "StringLike": { "s3:prefix": ["proof/*"] } }
}
```

Nhầm lẫn phổ biến nhất của người mới là đặt `ListBucket` vào statement có `Resource` kết thúc bằng `/*`; statement đó sẽ không bao giờ khớp.

**Debug `AccessDenied`**: đọc kỹ thông báo lỗi. Thông báo lỗi của AWS hiện nay thường nói rõ vì sao ("no identity-based policy allows…" hay "explicit deny in…"). Sau đó dùng ba công cụ:

1. **IAM Policy Simulator** (console, hoặc `aws iam simulate-principal-policy` và `simulate-custom-policy`) để thử một action trên một ARN mà không gọi thật.
2. **CloudTrail event history** giúp tìm sự kiện có `errorCode` là `AccessDenied`, để xem chính xác principal, action và resource.
3. **IAM Access Analyzer** kiểm tra policy và gợi ý thu hẹp quyền từ lịch sử truy cập.

Simulator mô phỏng theo policy bạn đưa vào, nên vẫn cần kiểm lại bằng một lời gọi thật ở tài khoản lab.

## Lỗi thường gặp

- **`Action: "*"` và `Resource: "*"` "cho tạm"**: tạm thời trở thành vĩnh viễn; hãy bắt đầu hẹp và nới dần.
- **Nhầm ARN bucket và ARN object**: `arn:aws:s3:::bucket` và `arn:aws:s3:::bucket/*` là hai thứ khác nhau.
- **Quên rằng Deny thắng**: một Deny cũ ở lớp khác khiến Allow mới không có tác dụng.
- **Gắn quyền cho user thay vì role**: kéo theo access key dài hạn.
- **Dán policy do đồng nghiệp chia sẻ mà không đọc**: kiểm tra từng `Action` và `Resource`.

## Bài tập

**Bài tập:** policy dưới đây được gắn cho service `delivery-photo`. Sửa nó thành quyền tối thiểu.

```json
{
  "Version": "2012-10-17",
  "Statement": [{ "Effect": "Allow", "Action": "s3:*", "Resource": "*" }]
}
```

<details><summary>Xem đáp án</summary><p>Thay <code>s3:*</code> bằng <code>s3:PutObject</code> và <code>s3:GetObject</code>; thay <code>Resource: "*"</code> bằng <code>arn:aws:s3:::toinoi-lab-delivery-photos/proof/*</code>. Nếu service cần liệt kê, thêm statement <code>s3:ListBucket</code> trên ARN bucket với <code>Condition</code> <code>s3:prefix</code> là <code>proof/*</code>, đúng như policy trong lab. Policy gốc cho phép xoá mọi bucket trong tài khoản, kể cả bucket chứa dữ liệu khác.</p></details>

## Lab: tạo role và kiểm bằng simulator

Lab ở `examples/toi-noi-aws/p01-03-iam-policies` có trust policy, policy quyền tối thiểu, policy quá rộng để so sánh, script `simulate.sh` và script tạo role. Script mô phỏng chỉ đọc (gọi `simulate-custom-policy`) và kiểm các kết quả mong đợi: ghi vào `proof/` được phép, ghi sang prefix khác hay xoá object bị từ chối ngầm, còn policy rộng cho phép tất cả. Script tạo role mặc định chỉ in lệnh; chỉ khi thêm `--apply` thì mới tạo, và `cleanup.sh` dọn lại. IAM role và policy không tính phí riêng.

```bash
cd examples/toi-noi-aws/p01-03-iam-policies
./simulate.sh toinoi-lab
./create-role.sh toinoi-lab
```

Script đã được kiểm tra cú pháp, chạy shellcheck, chạy thử với CLI giả lập, và tham số của các lệnh `aws iam` được đối chiếu với bộ mô hình dịch vụ của AWS CLI; **chưa chạy trên tài khoản AWS thật**, nên kết quả thực của simulator chưa được đối chiếu.

Bài trước: [bài 02](/articles/aws-tai-khoan-iam-identity-center-cli). Bài tiếp: [bài 04](/articles/aws-vpc-subnet-route-security-group).

## Tài liệu gốc

- [IAM User Guide](https://docs.aws.amazon.com/IAM/latest/UserGuide/introduction.html).
- [Policies and permissions in IAM](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html).
- [Policy evaluation logic](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html).
- [IAM roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles.html).
- [Testing IAM policies with the IAM policy simulator](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_testing-policies.html).

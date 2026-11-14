---
slug: aws-tai-khoan-iam-identity-center-cli
title: "Tài khoản AWS, IAM Identity Center và AWS CLI: thiết lập an toàn ngày đầu"
summary: "Bài 02: khoá root user bằng MFA, tạo người dùng qua IAM Identity Center, đặt Budgets để không bất ngờ khi nhận hoá đơn, cài AWS CLI v2 và đăng nhập bằng aws configure sso; kiểm tra bằng aws sts get-caller-identity."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, iam-identity-center, aws-cli, mfa, budgets, root-user]
language: vi
---

Linh vừa vào đội kỹ thuật của [Tới Nơi](/articles/du-an-mau-toi-noi) và cần chạy thử service báo giá trên AWS. Một đồng nghiệp nhắn: "Mình gửi access key của tài khoản chung qua chat nhé, dán vào máy là chạy." Đó là cách nhanh nhất để một khoá có quyền tuyệt đối nằm rải rác trong lịch sử chat, ổ đĩa và có thể cả Git. Bài này đi con đường khác: khoá cửa trước, rồi phát cho mỗi người một chìa tạm thời.

Đây là **bài 02** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần đọc [bài 01](/articles/aws-la-gi-region-az-shared-responsibility), có một địa chỉ email và một thẻ thanh toán để tạo tài khoản lab của riêng mình.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Để chạy AWS CLI trên laptop cho tiện, có nên tạo access key cho root user không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Access key của root có quyền vô hạn trên tài khoản, không thể bị giới hạn bằng IAM policy, và không hết hạn. Root chỉ dùng qua đăng nhập web cho vài việc đặc biệt; công việc hằng ngày dùng danh tính tạm thời qua IAM Identity Center.</p></details>
</div>

## Root user: khoá lại và để yên

Khi tạo tài khoản AWS, bạn đăng ký bằng một email và mật khẩu. Danh tính đó là **root user**: quyền vô hạn, kể cả đóng tài khoản. Không policy nào hạn chế được nó, nên mục tiêu là *ít dùng nhất có thể*.

Việc cần làm ngay sau khi tạo tài khoản:

1. **Bật MFA** (xác thực nhiều lớp) cho root, tốt nhất bằng passkey hoặc khoá FIDO2; ứng dụng authenticator là phương án thay thế. Từ 2024 AWS đã bắt đầu buộc root đăng ký MFA khi đăng nhập console (mở rộng dần theo loại tài khoản); dù tài khoản của bạn đã bị yêu cầu hay chưa, hãy bật ngay.
2. **Không tạo access key cho root.** Nếu đã lỡ tạo, xoá đi.
3. Dùng **email chung của nhóm** (ví dụ `aws-root@toinoi.example`, địa chỉ minh hoạ) thay vì email cá nhân, để khi một người nghỉ việc, quyền khôi phục tài khoản không mất theo.
4. Lưu mật khẩu root trong trình quản lý mật khẩu và đừng dùng nó ngoài các việc buộc phải dùng root.

Chỉ một nhóm nhỏ việc cần root: đổi email hoặc mật khẩu root, đóng tài khoản, bật quyền xem Billing cho IAM, sửa một bucket policy S3 lỡ chặn mọi principal, khôi phục khi tự khoá mình khỏi IAM… Danh sách đầy đủ có trong tài liệu "Tasks that require root user credentials" ở cuối bài.

## Budgets: cảnh báo trước khi có hoá đơn

Với tài khoản học, rủi ro lớn nhất là quên tắt tài nguyên. **AWS Budgets** cho bạn đặt hạn mức chi tiêu hằng tháng và gửi email khi chi phí thực tế hoặc dự báo vượt ngưỡng. Với tài khoản lab, đặt ngưỡng nhỏ và thêm cảnh báo ở 80% hạn mức.

Có hai giới hạn cần biết. Dữ liệu chi phí cập nhật chậm (vài giờ), nên Budgets là chuông báo chứ không phải phanh: nó không tự tắt tài nguyên trừ khi bạn cấu hình riêng budget action. Ngoài ra, ưu đãi cho tài khoản mới (free tier, credit) thay đổi theo thời gian; hãy đọc điều kiện hiện hành trên trang Free Tier thay vì dựa vào trí nhớ.

Bạn có thể tạo budget bằng console hoặc bằng script trong lab của bài (xem cuối bài). Nếu sau này user đăng nhập qua Identity Center đã có quyền mà vẫn không xem được trang Billing, hãy kiểm tra cài đặt "IAM user and role access to Billing information" trong Account; chỉ root đổi được cài đặt này.

## IAM Identity Center: danh tính tạm thời cho người

**IAM Identity Center** (tên cũ AWS Single Sign-On) là nơi tập trung quản lý *người* truy cập AWS. Ba khái niệm chính:

- **User và group**: tài khoản của từng người, gom thành nhóm (ví dụ `toinoi-lab-admins`, `toinoi-readonly`).
- **Permission set**: bản mẫu quyền, gồm một hoặc nhiều policy. Khi gán vào một tài khoản AWS, Identity Center tự tạo ở đó một IAM role tương ứng.
- **Assignment**: nối "nhóm nào" với "permission set nào" ở "tài khoản nào".

Người dùng đăng nhập vào một **AWS access portal** (địa chỉ dạng `https://toinoi-lab.awsapps.com/start`, minh hoạ), chọn tài khoản và role, và nhận credential **tạm thời**, hết hạn sau vài giờ.

| | IAM user + access key | IAM Identity Center |
|---|---|---|
| Credential | Dài hạn, nằm trong file trên máy | Ngắn hạn, tự hết hạn |
| Khi lộ khoá | Dùng được cho tới khi bạn phát hiện và xoá | Hết hạn sau vài giờ |
| Nhiều tài khoản AWS | Tạo user ở từng tài khoản | Một danh tính, gán vào nhiều tài khoản |
| Người nghỉ việc | Phải nhớ xoá user và khoá | Vô hiệu hoá một chỗ |
| MFA | Tự cấu hình từng user | Đặt chính sách một chỗ cho mọi người |

Các bước làm trên console, trong tài khoản lab của bạn:

1. Đăng nhập bằng root, chuyển console sang region `ap-southeast-1`, vào **IAM Identity Center** và chọn **Enable**. Chọn region cẩn thận vì Identity Center có một region chính và đổi region này về sau rất phiền. Chọn bật cùng với **AWS Organizations** (lựa chọn được khuyến nghị): nếu chưa có, AWS tạo một Organization với tài khoản của bạn làm management account (bài 28 quay lại). Kiểu còn lại, *account instance*, không cấp được quyền vào tài khoản AWS nên không dùng cho loạt bài này.
2. Tạo group `toinoi-lab-admins` và một user cho chính bạn, thêm vào group, bật MFA cho user.
3. Tạo permission set `ToiNoiLabAdmin` gắn managed policy `AdministratorAccess`, thời lượng phiên ngắn (ví dụ 4 giờ). Đây là quyền rộng, chỉ chấp nhận được trong tài khoản lab cá nhân; bài 03 sẽ nói vì sao workload thật không dùng nó.
4. Gán group `toinoi-lab-admins` với permission set đó vào tài khoản `123456789012`.

Từ giờ bạn không cần đăng nhập root nữa, trừ những việc đặc biệt nói ở trên.

## AWS CLI v2 và `aws configure sso`

**AWS CLI** là công cụ dòng lệnh gọi API của AWS. Dùng **phiên bản 2** (lệnh `aws`); cài theo hướng dẫn chính thức cho macOS, Linux hoặc Windows rồi kiểm tra:

```bash
aws --version
```

Kết quả phải bắt đầu bằng `aws-cli/2.`. Nếu hiện `aws-cli/1.`, bạn đang dùng bản cũ.

Sau đó cấu hình đăng nhập SSO. Lệnh này hỏi tuần tự về start URL, region của Identity Center, mở trình duyệt để bạn xác thực, rồi cho chọn tài khoản và role:

```bash
aws configure sso
```

Khi được hỏi, nhập:

- SSO session name: `toinoi`
- SSO start URL: địa chỉ access portal của bạn
- SSO region: `ap-southeast-1`
- Scope: giữ mặc định `sso:account:access`
- Default client Region: `ap-southeast-1`; output: `json`
- Profile name: `toinoi-lab`

Kết quả là hai khối trong tệp `~/.aws/config`. Nếu thích, bạn có thể tự viết thay vì chạy lệnh trên (tệp mẫu có trong lab):

```ini
[sso-session toinoi]
sso_start_url = https://toinoi-lab.awsapps.com/start
sso_region = ap-southeast-1
sso_registration_scopes = sso:account:access

[profile toinoi-lab]
sso_session = toinoi
sso_account_id = 123456789012
sso_role_name = ToiNoiLabAdmin
region = ap-southeast-1
output = json
```

Đăng nhập và kiểm tra ta là ai:

```bash
aws sso login --profile toinoi-lab
aws sts get-caller-identity --profile toinoi-lab
```

`aws sso login` mở trình duyệt, bạn xác thực (kèm MFA), và CLI lưu token đăng nhập vào bộ nhớ đệm trên máy. Lệnh `get-caller-identity` luôn dùng được với mọi danh tính và không cần quyền riêng; nó trả về `Account`, `UserId` và `Arn`. Với SSO, `Arn` có dạng `arn:aws:sts::123456789012:assumed-role/AWSReservedSSO_ToiNoiLabAdmin_<mã>/<tên-bạn>`: một role được assume, không phải một IAM user.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Luồng đăng nhập SSO: lập trình viên chạy aws sso login, trình duyệt xác thực với IAM Identity Center kèm MFA, CLI lưu token, mỗi lệnh đổi token lấy credential tạm của role rồi gọi API AWS.">
<defs><marker id="aws02-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="40" width="170" height="80" rx="10" class="f-surface s-line"/>
<text x="34" y="72" class="t-b">Máy của bạn</text>
<text x="34" y="94" class="t-small t-mono">aws sso login</text>
<rect x="250" y="40" width="190" height="80" rx="10" class="f-pk-soft s-pk"/>
<text x="264" y="72" class="t-b">IAM Identity Center</text>
<text x="264" y="94" class="t-small">xác thực + MFA</text>
<rect x="500" y="40" width="170" height="80" rx="10" class="f-phys-soft s-phys"/>
<text x="514" y="72" class="t-b">Credential tạm</text>
<text x="514" y="94" class="t-small">của role ToiNoiLabAdmin</text>
<rect x="720" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/>
<text x="734" y="72" class="t-b">API AWS</text>
<text x="734" y="94" class="t-small">ec2, s3, iam…</text>
<path d="M190 80 L248 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws02-1-a)"/>
<path d="M440 80 L498 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws02-1-a)"/>
<path d="M670 80 L718 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws02-1-a)"/>
<text x="200" y="68" class="t-small">1</text>
<text x="450" y="68" class="t-small">2</text>
<text x="685" y="68" class="t-small">3</text>
<rect x="20" y="160" width="840" height="56" rx="10" class="f-warn-soft s-warn dash"/>
<text x="34" y="184" class="t-b t-warn">Không có access key dài hạn nào nằm trên máy</text>
<text x="34" y="203" class="t-small">1: đăng nhập, token lưu cache. 2: mỗi lệnh CLI đổi token lấy credential ngắn hạn. 3: gọi API; credential tự hết hạn.</text>
</svg>
</div>
<figcaption>Hình 1. Luồng đăng nhập SSO của CLI. Token đăng nhập và credential của role đều hết hạn; khi hết, chạy lại <code>aws sso login</code>.</figcaption>
</figure>

## Dùng CLI hằng ngày: profile, output và `--query`

Mỗi lệnh nhận `--profile`, hoặc bạn đặt biến môi trường một lần cho cả phiên terminal:

```bash
export AWS_PROFILE=toinoi-lab
aws configure list
```

`aws configure list` cho biết CLI đang lấy credential và region từ đâu, rất hữu ích khi gặp "đang chạy nhầm tài khoản". Nếu thấy `AWS_ACCESS_KEY_ID` trong biến môi trường, nó sẽ **ghi đè** profile; hãy `unset` nó.

Output mặc định là `json`; thêm `--output table` để dễ đọc, hoặc `--output text` khi cần đưa vào script. Tham số `--query` dùng **JMESPath** để lọc kết quả ngay trên máy, chưa gọi thêm API nào:

```bash
aws ec2 describe-regions --query 'Regions[].RegionName' --output text
```

Lệnh này liệt kê các region đang được bật trong tài khoản; thêm `--all-regions` để thấy cả region opt-in chưa bật. Lệnh chỉ đọc nên không tính tiền.

## Quy ước của cả loạt bài

| Mục | Giá trị trong bài |
|---|---|
| Account ID | `123456789012` (giả; thay bằng của bạn khi chạy) |
| Region | `ap-southeast-1` |
| Profile | `toinoi-lab` |
| Tag mọi tài nguyên | `Project=ToiNoi`, `Env=lab` |

Không bao giờ dán access key, secret hay token vào bài viết, issue, commit hay chat. Nếu lỡ lộ, thu hồi ngay (deactivate rồi xoá khoá) trước khi làm việc khác.

## Lỗi thường gặp

- **Token hết hạn** (`The SSO session associated with this profile has expired`): chạy lại `aws sso login`.
- **Chạy nhầm tài khoản hoặc region**: chạy `aws sts get-caller-identity` và `aws configure list` để kiểm tra trước mọi lệnh tạo tài nguyên.
- **Giữ `AWS_ACCESS_KEY_ID` cũ trong shell**: biến này được ưu tiên hơn `AWS_PROFILE` (chỉ thua `--profile` ghi trên dòng lệnh); kết quả là danh tính không như bạn nghĩ.
- **Dùng `AdministratorAccess` ở mọi nơi**: tiện cho lab, nhưng là thói quen xấu; bài 03 viết quyền hẹp hơn.
- **Tin rằng Budgets sẽ chặn chi tiêu**: nó chỉ cảnh báo, chậm vài giờ.

## Bài tập

**Bài tập:** viết hai lệnh: một lệnh in ra danh tính hiện tại của profile `toinoi-lab`, một lệnh in tên các region đang bật trong tài khoản dưới dạng bảng một cột.

<details><summary>Xem đáp án</summary><p>Lệnh thứ nhất: <code>aws sts get-caller-identity --profile toinoi-lab</code>. Lệnh thứ hai: <code>aws ec2 describe-regions --profile toinoi-lab --query 'Regions[].[RegionName]' --output table</code>. Cặp ngoặc vuông trong JMESPath tạo danh sách lồng nhau, đó là điều kiện để <code>--output table</code> in thành bảng một cột.</p></details>

## Lab: kiểm tra thiết lập và đặt budget

Lab ở `examples/toi-noi-aws/p01-02-cli-setup` gồm tệp cấu hình mẫu, script `check-setup.sh` chỉ đọc (kiểm tra CLI v2, profile, danh tính, biến môi trường nguy hiểm, danh sách region) và script `create-budget.sh` tạo một budget hằng tháng. Script tạo budget **mặc định chỉ in lệnh**; phải thêm `--apply` mới tạo thật, và `delete-budget.sh` dọn lại. Phí của AWS Budgets (nếu có) xem trên trang giá hiện hành, đừng giả định là miễn phí.

```bash
cd examples/toi-noi-aws/p01-02-cli-setup
./check-setup.sh toinoi-lab
BUDGET_EMAIL=lab-owner@toinoi.example ./create-budget.sh toinoi-lab
```

Script đã được kiểm tra cú pháp bằng `bash -n` và shellcheck, chạy thử với CLI giả lập, và tham số của các lệnh `aws budgets` được đối chiếu với bộ mô hình dịch vụ của AWS CLI; **chưa được chạy trên tài khoản AWS thật**. Đọc kỹ phần in ra trước khi thêm `--apply`.

Bài trước: [bài 01](/articles/aws-la-gi-region-az-shared-responsibility). Bài tiếp: [bài 03](/articles/aws-iam-user-role-policy).

## Tài liệu gốc

- [Getting started with the AWS CLI v2](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html).
- [Configuring the AWS CLI to use IAM Identity Center](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html).
- [IAM Identity Center User Guide](https://docs.aws.amazon.com/singlesignon/latest/userguide/what-is.html).
- [Root user best practices for your AWS account](https://docs.aws.amazon.com/IAM/latest/UserGuide/root-user-best-practices.html).
- [Managing your costs with AWS Budgets](https://docs.aws.amazon.com/cost-management/latest/userguide/budgets-managing-costs.html).

---
slug: aws-iam-nang-cao-permission-boundary-abac-access-analyzer
title: "IAM nâng cao: permission boundary, condition key, ABAC, session policy và Access Analyzer"
summary: "Bài 51: logic đánh giá policy đầy đủ với SCP, RCP, permission boundary, session policy và resource policy; ABAC bằng tag cho team Tới Nơi; cho dev tự tạo role mà không vượt quyền; external id cho đối tác; Access Analyzer tìm quyền truy cập từ bên ngoài, quyền thừa và sinh policy từ CloudTrail."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, iam, abac, permission-boundary, access-analyzer, condition-key, security]
language: vi
---

Tới Nơi có ba team backend: `booking`, `dispatch` và `tracking`. Mỗi lần team dispatch cần một role cho Lambda mới, họ gửi yêu cầu cho platform team và đợi nửa ngày. Cho dev tự tạo role thì ai cũng có thể gắn `AdministratorAccess` cho role mình tạo, và cả bốn mươi kỹ sư thành admin trên thực tế. Bài này giải quyết mâu thuẫn đó: giao quyền mà vẫn giữ được trần.

Đây là **bài 51** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần xong [bài 03](/articles/aws-iam-user-role-policy) và nên đọc [bài 28](/articles/aws-organizations-scp-control-tower-landing-zone) về SCP và RCP; bài này không giải thích lại chúng mà đặt chúng vào bức tranh đầy đủ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một role có identity policy <code>Allow s3:*</code> trên mọi resource, nhưng permission boundary của nó chỉ cho <code>sqs:*</code>. Role đó ghi được object vào S3 không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Permission boundary là trần: quyền hiệu lực là phần giao của identity policy và boundary. <code>s3:*</code> nằm ngoài boundary nên bị từ chối ngầm, dù identity policy cho phép.</p></details>
</div>

## Một request đi qua những lớp nào

Bài 03 dừng ở "Deny thắng Allow, còn lại là từ chối ngầm". Khi có Organizations và nhiều team, thêm bốn lớp **trần** (ceiling): chúng không cấp quyền, chỉ giới hạn những gì lớp cấp quyền trao được.

| Lớp | Ai đặt | Giới hạn cái gì | Vai trò |
|---|---|---|---|
| SCP | Admin tổ chức | Mọi principal trong tài khoản hoặc OU | Trần |
| RCP | Admin tổ chức | Tài nguyên trong tài khoản, kể cả khi principal từ ngoài tổ chức gọi tới | Trần |
| Permission boundary | Platform team | Một user hoặc role cụ thể | Trần |
| Session policy | Người assume role | Một phiên làm việc | Trần |
| Identity policy | Chủ identity | Principal được gắn policy | Cấp quyền |
| Resource policy | Chủ tài nguyên | Ai được dùng tài nguyên này | Cấp quyền |

Quy tắc đọc: một Deny khớp ở **bất kỳ** lớp nào là dừng. Nếu không có Deny, request phải nằm trong **mọi** lớp trần đang áp dụng, và phải có ít nhất một Allow ở lớp cấp quyền. Trong cùng tài khoản, identity policy hoặc resource policy cho phép là đủ (trust policy của role và key policy của KMS phải tự cho phép). Boundary cũng không phải hàng rào cho mọi đường vào: trong cùng tài khoản, resource policy cấp quyền cho đích danh ARN của user (hoặc ARN của một phiên role) không bị implicit deny của boundary cản; nếu resource policy nêu ARN của role thì boundary và session policy vẫn giới hạn. Deny rõ ràng trong boundary thì luôn chặn. Truy cập chéo tài khoản cần cả hai bên cho phép: identity policy ở tài khoản người gọi và resource policy ở tài khoản sở hữu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 245" role="img" aria-label="Hình 1. Request đi qua Deny rõ ràng, ba lớp trần (SCP và RCP, permission boundary, session policy), rồi cần một Allow ở identity hoặc resource policy. Chặn ở bất kỳ lớp nào thì bị từ chối.">
<defs><marker id="aws51-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws51-1-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="120" height="70" rx="10" class="f-surface s-line"/><text x="32" y="60" class="t-b">Request</text><text x="32" y="82" class="t-small">tới một API</text>
<rect x="164" y="30" width="120" height="70" rx="10" class="f-warn-soft s-warn"/><text x="176" y="60" class="t-b t-warn">Deny rõ ràng?</text><text x="176" y="82" class="t-small">ở mọi policy</text>
<rect x="308" y="30" width="120" height="70" rx="10" class="f-pk-soft s-pk"/><text x="320" y="60" class="t-b">SCP và RCP</text><text x="320" y="82" class="t-small">trần tổ chức</text>
<rect x="452" y="30" width="120" height="70" rx="10" class="f-pk-soft s-pk"/><text x="464" y="60" class="t-b">Boundary</text><text x="464" y="82" class="t-small">trần của role</text>
<rect x="596" y="30" width="120" height="70" rx="10" class="f-pk-soft s-pk"/><text x="608" y="60" class="t-b">Session policy</text><text x="608" y="82" class="t-small">trần của phiên</text>
<rect x="740" y="30" width="120" height="70" rx="10" class="f-phys-soft s-phys"/><text x="752" y="60" class="t-b">Có Allow?</text><text x="752" y="82" class="t-small">identity / resource</text>
<path d="M140 65 L162 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-1-a)"/>
<path d="M284 65 L306 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-1-a)"/>
<path d="M428 65 L450 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-1-a)"/>
<path d="M572 65 L594 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-1-a)"/>
<path d="M716 65 L738 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-1-a)"/>
<path d="M224 100 L224 128 M368 100 L368 128 M512 100 L512 128 M656 100 L656 128 M656 128 L120 128 L120 152" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws51-1-b)"/>
<text x="330" y="146" class="t-small">chỉ cần một lớp chặn</text>
<rect x="20" y="154" width="200" height="46" rx="10" class="f-warn-soft s-warn"/><text x="34" y="182" class="t-b t-warn">TỪ CHỐI</text>
<rect x="740" y="154" width="120" height="46" rx="10" class="f-phys-soft s-phys"/><text x="754" y="182" class="t-b t-phys">CHO PHÉP</text>
<path d="M800 100 L800 152" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-1-a)"/>
<text x="20" y="226" class="t-small">Lớp trần chỉ giới hạn, không cấp quyền. Chéo tài khoản: cả hai tài khoản phải cho phép.</text>
</svg>
</div>
<figcaption>Hình 1. Ba lớp trần giao nhau, rồi cần một Allow thực sự; Deny hoặc một lớp trần chặn ở đâu thì request dừng ở đó.</figcaption>
</figure>

## ABAC: dùng tag thay cho nhiều role

**RBAC** gán quyền theo vai trò: mỗi team một bộ role và policy. **ABAC (attribute-based access control)** gán theo thuộc tính: một policy chung nói "principal có `Team=X` chỉ chạm tài nguyên có `Team=X`". Thêm team hay tài nguyên chỉ cần gắn tag.

| | RBAC | ABAC |
|---|---|---|
| Đơn vị quản lý | Role và policy theo từng team | Một policy dùng chung và tag |
| Thêm team mới | Tạo role và policy mới | Gắn tag `Team` cho role mới |
| Rủi ro chính | Số policy phình ra, lệch nhau | Ai sửa được tag là sửa được quyền |
| Phù hợp | Ít team, quyền khác biệt nhiều | Nhiều team, cấu trúc quyền giống nhau |

Policy cốt lõi nằm trong lab (rút gọn một statement):

```json
{
  "Sid": "SameTeamFunctionsOnly",
  "Effect": "Allow",
  "Action": ["lambda:InvokeFunction", "lambda:GetFunction", "lambda:UpdateFunctionCode"],
  "Resource": "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-*",
  "Condition": {
    "StringEquals": { "aws:ResourceTag/Team": "${aws:PrincipalTag/Team}" }
  }
}
```

`aws:PrincipalTag/Team` lấy từ tag của role (hoặc session tag khi assume role); `aws:ResourceTag/Team` từ tag của function bị gọi. Principal không có tag `Team` thì điều kiện không khớp: **không tag, không quyền**, đây là cách thất bại an toàn.

ABAC chỉ an toàn khi **tag được bảo vệ**. Lab có ba lớp: Allow `CreateFunction` chỉ khi `aws:RequestTag/Team` bằng tag của principal (dispatch không tạo được function mang nhãn booking); Allow `TagResource` chỉ trên function cùng team và chỉ với giá trị cùng team; Deny `UntagResource` với khoá `Team` trừ khi principal có `Role=platform-admin`. Ở cấp tổ chức, tag policy chuẩn hoá khoá và giá trị tag, còn SCP có thể cấm sửa tag của role. Mỗi action hỗ trợ condition key tag khác nhau; tra Service Authorization Reference trước khi viết policy ABAC cho dịch vụ mới.

## Permission boundary: cho dev tự tạo role mà không vượt quyền

Permission boundary là một managed policy gắn vào user hoặc role để đặt trần cho nó. Mẫu thiết kế gồm hai nửa:

1. **Boundary** `toinoi-lab-lambda-dev-boundary`: allow-list đúng thứ Lambda của Tới Nơi cần (ghi log, SQS `toinoi-*`, S3 `proof/*`, bảng DynamoDB `toinoi-*`, X-Ray) và Deny `iam:*`, `organizations:*`, `account:*`.
2. **Quyền của dev**: dev chỉ được tạo role khi role đó gắn đúng boundary này.

```json
{
  "Sid": "CreateOrChangeFnRolesOnlyWithBoundary",
  "Effect": "Allow",
  "Action": ["iam:CreateRole", "iam:PutRolePermissionsBoundary", "iam:AttachRolePolicy",
             "iam:DetachRolePolicy", "iam:PutRolePolicy", "iam:DeleteRolePolicy"],
  "Resource": "arn:aws:iam::123456789012:role/toinoi-lab-fn-*",
  "Condition": {
    "StringEquals": {
      "iam:PermissionsBoundary": "arn:aws:iam::123456789012:policy/toinoi-lab-lambda-dev-boundary"
    }
  }
}
```

Statement này cho tạo role `toinoi-lab-fn-*` kèm đúng boundary, và gắn hay sửa policy trên role đã có boundary đó. Đứng một mình, statement này thiếu hai thứ: **Deny** `iam:DeleteRolePermissionsBoundary` (nếu không dev tạo xong rồi gỡ boundary) và **Deny** sửa chính policy boundary (`CreatePolicyVersion`, `DeletePolicy`, `SetDefaultPolicyVersion`). Thêm `iam:PassRole` giới hạn `iam:PassedToService=lambda.amazonaws.com` để role chỉ gắn được vào Lambda.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Dev tạo role chỉ khi điều kiện boundary đúng; quyền hiệu lực của role là phần giao giữa identity policy và boundary.">
<defs><marker id="aws51-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="25" width="170" height="70" rx="10" class="f-surface s-line"/><text x="34" y="55" class="t-b">Dev team dispatch</text><text x="34" y="77" class="t-small">iam:CreateRole</text>
<rect x="250" y="25" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="264" y="55" class="t-b">Có đúng boundary?</text><text x="264" y="77" class="t-small t-mono">iam:PermissionsBoundary</text>
<rect x="500" y="25" width="170" height="70" rx="10" class="f-phys-soft s-phys"/><text x="514" y="55" class="t-b">Role fn-checker</text><text x="514" y="77" class="t-small">kèm boundary</text>
<rect x="730" y="25" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="744" y="55" class="t-b">Lambda</text><text x="744" y="77" class="t-small">assume role</text>
<path d="M190 60 L248 60" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-2-a)"/>
<path d="M440 60 L498 60" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-2-a)"/>
<path d="M670 60 L728 60" class="f-none s-phys" stroke-width="2" marker-end="url(#aws51-2-a)"/>
<rect x="20" y="140" width="250" height="75" rx="10" class="f-surface s-line"/><text x="34" y="168" class="t-b">Identity policy của role</text><text x="34" y="190" class="t-small">s3:*, sqs:*, ec2:* (dev viết rộng)</text>
<text x="290" y="184" class="t-b">∩</text>
<rect x="320" y="140" width="250" height="75" rx="10" class="f-pk-soft s-pk"/><text x="334" y="168" class="t-b">Permission boundary</text><text x="334" y="190" class="t-small">sqs toinoi-*, S3 proof/*, logs</text>
<text x="590" y="184" class="t-b">=</text>
<rect x="620" y="140" width="240" height="75" rx="10" class="f-phys-soft s-phys"/><text x="634" y="168" class="t-b t-phys">Quyền hiệu lực</text><text x="634" y="190" class="t-small">chỉ phần nằm trong cả hai</text>
</svg>
</div>
<figcaption>Hình 2. Điều kiện trên <code>iam:CreateRole</code> bảo đảm role nào do dev tạo cũng mang boundary; còn boundary bảo đảm identity policy dù dev viết rộng cũng không vượt trần.</figcaption>
</figure>

## Session policy, role chaining và đối tác bên ngoài

**Session policy** được truyền vào lúc assume role (`aws sts assume-role --policy ...`), thu hẹp quyền cho đúng phiên đó mà không đổi role. Dùng khi tool nội bộ nhận role rộng nhưng phiên này chỉ cần quyền đọc. Session policy chỉ giảm quyền, không bao giờ thêm. **Role chaining** (role A assume role B) giới hạn thời gian của phiên trong chuỗi tối đa một giờ.

Với đối tác thanh toán VíXanh cần đọc báo cáo đối soát, trust policy của role cho tài khoản của họ assume, kèm `sts:ExternalId`:

```json
{
  "Effect": "Allow",
  "Principal": { "AWS": "arn:aws:iam::123456789012:root" },
  "Action": "sts:AssumeRole",
  "Condition": { "StringEquals": { "sts:ExternalId": "<chuỗi ngẫu nhiên hai bên thoả thuận>" } }
}
```

Principal ở đây là account đối tác (lab dùng biến, mặc định là account minh hoạ). **Confused deputy** xảy ra khi một bên trung gian bị lừa dùng quyền của mình thay mặt kẻ khác: nếu nhà cung cấp phục vụ nhiều khách và chỉ kiểm tra account, khách A có thể xin nhà cung cấp truy cập role của khách B. External ID khác nhau theo từng khách chặn được kịch bản đó. Với dịch vụ AWS thay mặt bạn gọi tài nguyên, dùng `aws:SourceArn` và `aws:SourceAccount` thay cho external ID. Hai condition key khác đáng nhớ: `aws:MultiFactorAuthPresent` (chỉ có khi phiên xác thực bằng MFA, nên dùng với `BoolIfExists` để Deny khi vắng) và `aws:SourceVpce` (giới hạn request chỉ đi qua một VPC endpoint, bài 53 dùng nó trong bucket policy).

## Access Analyzer: quyền truy cập từ bên ngoài và quyền thừa

**IAM Access Analyzer** có ba việc khác nhau:

| Chức năng | Câu hỏi nó trả lời | Cách gọi |
|---|---|---|
| External access findings | Resource nào chia sẻ ra ngoài tài khoản hoặc tổ chức | Analyzer `ACCOUNT` hoặc `ORGANIZATION` |
| Unused access findings | Role, key, mật khẩu và quyền nào không dùng trong N ngày | Analyzer `ACCOUNT_UNUSED_ACCESS` |
| Policy validation và custom check | Policy này có lỗi cú pháp hay mở quá rộng không | `validate-policy`, `check-no-new-access` |

Analyzer unused access mặc định xét 90 ngày, tạo finding cho role, access key, mật khẩu và quyền không dùng. Loại này tính phí theo số role và user được phân tích (external access thì không), nên lab có biến tắt nó. Role khẩn cấp có thể "không dùng" đúng như thiết kế, nên finding là việc cần xem xét, không phải lệnh xoá.

```bash
aws accessanalyzer list-findings-v2 --analyzer-arn "$ARN" --profile toinoi-lab
aws accessanalyzer validate-policy --policy-type IDENTITY_POLICY \
  --policy-document file://policies/developer-create-role.json --profile toinoi-lab
```

Lệnh đầu liệt kê finding của analyzer; lệnh sau báo cảnh báo bảo mật và lỗi trong policy mà bạn sắp áp dụng. **Sinh policy từ CloudTrail** (`start-policy-generation`) đọc hoạt động thật của một role trong một khoảng thời gian rồi đề xuất policy tương ứng. Kết quả là điểm khởi đầu: hành động ở data event hoặc chưa có trong log sẽ thiếu, nên luôn rà bằng tay trước khi gắn. Để biết "ai làm gì", CloudTrail event history (90 ngày management event) trả lời nhanh bằng `aws cloudtrail lookup-events`; phần lưu lâu dài, data event và truy vấn log nằm ở [bài 54](/articles/aws-cloudtrail-config-security-hub-guardduty). **Credential report** (`aws iam generate-credential-report` rồi `get-credential-report`) liệt kê toàn bộ user, tuổi mật khẩu, MFA và access key trong một CSV, hữu ích để dọn user dài hạn còn sót.

## Lỗi thường gặp

- **Boundary thiếu Deny gỡ boundary**: dev tự gỡ boundary khỏi role mình tạo, trần biến mất.
- **ABAC mà ai cũng sửa được tag `Team`**: quyền trở thành thứ đổi được chỉ bằng một lệnh `tag-resource`. Bảo vệ khoá tag trước.
- **Nhầm boundary là phép hợp**: boundary giao với identity policy, không hợp với nó. Chạy simulator cho cả hai chiều.
- **Coi finding unused là lệnh xoá**: xác minh với chủ role trước khi gỡ quyền production.
- **Gắn nguyên policy sinh từ CloudTrail**: thiếu action hiếm gặp, hỏng đúng đêm cuối tháng.

## Bài tập

**Bài tập:** role `toinoi-prod-ops` có `AdministratorAccess` nhưng gọi `s3:PutObject` vào bucket của Tới Nơi vẫn bị `AccessDenied`. Liệt kê các lớp có thể là nguyên nhân và cách tìm.

<details><summary>Xem đáp án</summary><p>Admin chỉ là một identity policy, mọi lớp khác vẫn áp dụng: <b>SCP</b> chặn (region hay dịch vụ), <b>RCP</b> hoặc bucket policy có Deny (ví dụ chỉ cho <code>aws:SourceVpce</code>), <b>permission boundary</b> trên role, <b>session policy</b> của phiên, <b>VPC endpoint policy</b> nếu request đi qua endpoint, và với object SSE-KMS còn cần quyền trên key policy. Cách tìm: đọc thông báo lỗi (thường nói loại policy gây Deny), tra CloudTrail theo <code>errorCode</code> và <code>errorMessage</code>, rồi dùng simulator cô lập từng lớp. Đừng thêm quyền trước khi biết lớp nào chặn.</p></details>

## Lab: ABAC, boundary và Access Analyzer

Lab ở `examples/toi-noi-aws/p06-51-iam-advanced` gồm các policy JSON, Terraform dựng ba policy, hai role team, role đối tác với external ID và hai analyzer, cùng `simulate.sh` chạy 13 test case (ABAC cùng và khác team, tạo role có hay không có boundary, Deny gỡ boundary, trần boundary với identity `*`). Simulator chỉ đọc.

```bash
cd examples/toi-noi-aws/p06-51-iam-advanced
for f in policies/*.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "$f hợp lệ"; done
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
./simulate.sh toinoi-lab
```

IAM không tính phí riêng; analyzer unused access tính theo số role và user được phân tích (tắt bằng `-var enable_unused_access_analyzer=false`). Dọn bằng `terraform destroy`. Terraform đã qua `fmt` và `validate` bằng Docker, JSON đã parse được; **chưa chạy `apply` hay `simulate.sh` trên tài khoản AWS thật**. Simulator chỉ tự điền vài khoá principal và tổ chức (`aws:PrincipalAccount`, `aws:PrincipalOrgID`…); mọi khoá khác như `aws:PrincipalTag/Team` hay `aws:ResourceTag/Team` phải truyền qua `--context-entries`, và test case gỡ tag ra `explicitDeny` vì điều kiện phủ định `StringNotEquals` đúng khi khoá `aws:PrincipalTag/Role` vắng mặt. Simulator đánh giá được SCP đang áp lên tài khoản nhưng không hỗ trợ RCP và không có tham số session policy; thử session policy bằng một lần `assume-role` thật.

Bài trước: [bài 50](/articles/aws-eks-van-hanh-upgrade-addon-chi-phi). Bài tiếp: [bài 52](/articles/aws-kms-envelope-encryption-key-policy).

## Tài liệu gốc

- [Policy evaluation logic](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html).
- [Permissions boundaries for IAM entities](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_boundaries.html).
- [ABAC for AWS](https://docs.aws.amazon.com/IAM/latest/UserGuide/introduction_attribute-based-access-control.html).
- [The confused deputy problem](https://docs.aws.amazon.com/IAM/latest/UserGuide/confused-deputy.html).
- [What is IAM Access Analyzer](https://docs.aws.amazon.com/IAM/latest/UserGuide/what-is-access-analyzer.html).
- [Resource control policies (RCPs)](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_rcps.html).

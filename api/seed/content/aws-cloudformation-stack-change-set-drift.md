---
slug: aws-cloudformation-stack-change-set-drift
title: "CloudFormation: template, stack, change set, drift và vì sao hạ tầng phải là code"
summary: "Bài 21: viết template CloudFormation YAML cho VPC và S3 của Tới Nơi với Parameters, Mappings, Conditions, Outputs và intrinsic function; tạo stack, xem change set trước khi áp dụng, phát hiện drift, stack policy và deletion policy để không mất dữ liệu."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, cloudformation, iac, change-set, drift, stack]
language: vi
---

Ở Phần 01, bạn dựng VPC của Tới Nơi bằng cách bấm console: chọn CIDR, tạo bốn subnet, gắn route table, nhớ bật versioning cho bucket ảnh chứng từ. Tuần sau đội tạo môi trường staging và một người quên gắn tag, người khác chọn nhầm AZ. Sáng thứ Hai không ai nhớ ai đã bấm gì, vào lúc nào, vì sao.

Đây là **bài 21** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bạn cần xong Phần 01 (đặc biệt bài 04 về VPC và bài 06 về S3) và biết đọc YAML. Bài này giải thích CloudFormation, dịch vụ IaC gốc của AWS, và là nền để so sánh với Terraform ở bài 22 và CDK ở bài 24.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn đổi tên logical ID của một tài nguyên RDS trong template rồi cập nhật stack. CloudFormation đổi tên cái database đang chạy, đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p><b>Sai.</b> Với CloudFormation, logical ID là danh tính của tài nguyên trong stack. ID mới nghĩa là một tài nguyên mới cần tạo, còn ID cũ biến mất khỏi template nên bị xoá theo <code>DeletionPolicy</code> của nó. Nếu policy đó là <code>Delete</code>, dữ liệu mất. Phần "Vòng đời stack" và bài tập cuối bài đi sâu hơn.</p></details>
</div>

## Vì sao hạ tầng phải là code

Click console cho kết quả nhanh nhưng không để lại dấu vết có thể đọc lại. IaC đổi bốn thứ:

- **Lặp lại được:** cùng một template tạo `dev`, `staging` và `prod`, khác nhau ở tham số chứ không ở trí nhớ của người bấm.
- **Review được:** thay đổi hạ tầng đi qua pull request như code, có người đọc trước khi chạy.
- **Nhiều môi trường:** một thay đổi sửa một lần, áp dụng cho mọi môi trường theo thứ tự.
- **Audit được:** Git cho biết ai đổi gì và khi nào; CloudTrail cho biết ai gọi API nào.

CloudFormation là dịch vụ khai báo (declarative): bạn mô tả trạng thái mong muốn trong template, dịch vụ tự tính thứ tự tạo và gọi API. Một lần triển khai template gọi là **stack**. CloudFormation không tính phí riêng cho tài nguyên `AWS::`; bạn trả tiền cho tài nguyên nó tạo ra (extension bên thứ ba trong registry tính riêng).

## Giải phẫu một template

Template là YAML hoặc JSON với các mục có tên cố định:

| Mục | Vai trò | Ví dụ ở Tới Nơi |
|---|---|---|
| `Parameters` | Giá trị nhập lúc tạo hoặc cập nhật stack | `Env` chỉ nhận `dev`, `staging`, `prod` |
| `Mappings` | Bảng tra cứu cố định trong template | CIDR theo môi trường |
| `Conditions` | Điều kiện bật tắt tài nguyên hoặc thuộc tính | Chỉ tạo NAT Gateway khi là `prod` |
| `Resources` | Danh sách tài nguyên (bắt buộc) | VPC, subnet, bucket |
| `Outputs` | Giá trị xuất ra để người dùng hoặc stack khác đọc | `VpcId`, danh sách private subnet |

Các intrinsic function hay gặp: `!Ref` (tham chiếu tham số hoặc ID tài nguyên), `!GetAtt` (lấy thuộc tính như `Vpc.CidrBlock`), `!Sub` (nội suy chuỗi, ví dụ `toinoi-${Env}-vpc`), `!FindInMap`, `!Select`, `!GetAZs`, `!Cidr` và `!ImportValue` (đọc giá trị export của stack khác). Đoạn sau trích từ lab; `!Cidr` chia VPC thành các khối /24 và `!Select` lấy khối thứ ba cho private subnet.

```yaml
Parameters:
  Env:
    Type: String
    AllowedValues: [dev, staging, prod]
Mappings:
  EnvConfig:
    dev: { VpcCidr: 10.10.0.0/16 }
    prod: { VpcCidr: 10.30.0.0/16 }
Conditions:
  CreateNat: !Or
    - !Equals [!Ref Env, prod]
    - !Equals [!Ref EnableNat, "true"]
Resources:
  PrivateSubnetA:
    Type: AWS::EC2::Subnet
    Properties:
      VpcId: !Ref Vpc
      AvailabilityZone: !Select [0, !GetAZs ""]
      CidrBlock: !Select [2, !Cidr [!GetAtt Vpc.CidrBlock, 4, 8]]
```

Khi template phình to, có hai cách chia. **Nested stack** (`AWS::CloudFormation::Stack` với `TemplateURL` trỏ tới S3) biến template con thành một tài nguyên của stack cha; vòng đời đi cùng nhau. **Cross-stack reference** dùng `Export` ở stack mạng và `!ImportValue` ở stack service; stack nào còn import thì không thể sửa hay xoá giá trị export đó. Tên export phải duy nhất trong một region của một tài khoản, vì vậy lab đặt dạng `toinoi-${Env}-network-VpcId`.

## Vòng đời stack và change set

Stack có ba thao tác chính là create, update và delete. Quy tắc quan trọng nhất: **đừng update thẳng, hãy tạo change set trước**. Change set là bản xem trước: với mỗi tài nguyên, nó nói `Add`, `Modify` hay `Remove`, và với `Modify` nó nói `Replacement` là `True`, `False` hay `Conditional`. `Replacement: True` cho RDS hay bucket nghĩa là tài nguyên cũ sẽ bị thay bằng cái mới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Luồng thay đổi hạ tầng: template trong Git, change set để xem trước, người review, execute, stack; drift detection so stack với tài nguyên thật."><defs><marker id="aws21-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws21-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">Template</text><text x="34" y="92" class="t-small">vpc.yaml trong Git</text><path d="M170 75 L212 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws21-1-a)"/><rect x="214" y="40" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="228" y="70" class="t-b">Change set</text><text x="228" y="92" class="t-small">Add / Modify / Remove</text><path d="M364 75 L406 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws21-1-a)"/><rect x="408" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="422" y="70" class="t-b">Review</text><text x="422" y="92" class="t-small">Replacement: True?</text><path d="M558 75 L600 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws21-1-a)"/><rect x="602" y="40" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="616" y="70" class="t-b">Execute</text><text x="616" y="92" class="t-small">áp dụng</text><path d="M732 75 L774 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws21-1-a)"/><rect x="776" y="40" width="84" height="70" rx="10" class="f-phys-soft s-phys"/><text x="790" y="70" class="t-b">Stack</text><text x="790" y="92" class="t-small">COMPLETE</text><rect x="470" y="150" width="390" height="60" rx="10" class="f-warn-soft s-warn"/><text x="486" y="176" class="t-b t-warn">Sửa tay trong console</text><text x="486" y="196" class="t-small">tài nguyên thật lệch template = drift</text><path d="M818 150 L818 112" class="f-none s-warn dash" stroke-width="1.6" marker-end="url(#aws21-1-w)"/><text x="826" y="136" class="t-small">detect-stack-drift</text><text x="40" y="170" class="t-small">Rollback tự động khi lỗi:</text><text x="40" y="190" class="t-small t-mono">UPDATE_ROLLBACK_COMPLETE</text></svg>
</div>
<figcaption>Hình 1. Template vào Git, change set cho xem trước, người review chốt rồi mới execute. Drift là phần thực tế lệch khỏi template do sửa ngoài luồng.</figcaption>
</figure>

Khi lỗi giữa chừng, CloudFormation rollback về trạng thái trước. Với **create** thất bại, stack kết thúc ở `ROLLBACK_COMPLETE` và chỉ còn cách xoá rồi tạo lại (tuỳ chọn `--disable-rollback` giữ lại để điều tra). Với **update** thất bại, stack về `UPDATE_ROLLBACK_COMPLETE`. Trường hợp xấu là `UPDATE_ROLLBACK_FAILED`: chính bước rollback không hoàn tác được một tài nguyên, thường vì ai đó đã xoá tay tài nguyên đó hoặc thiếu quyền. Cách thoát là sửa nguyên nhân rồi chạy `continue-update-rollback`; nếu không sửa được, thêm `--resources-to-skip` để bỏ qua tài nguyên kẹt, chấp nhận stack và thực tế lệch nhau ở đó.

## Không để mất dữ liệu

Có ba lớp bảo vệ, mỗi lớp trả lời một câu hỏi khác nhau:

| Cơ chế | Chặn điều gì | Giới hạn |
|---|---|---|
| `DeletionPolicy: Retain` hoặc `Snapshot` | Tài nguyên bị xoá khi **xoá stack** hoặc bỏ khỏi template | `Snapshot` chỉ áp dụng cho một số loại (RDS, EBS, ElastiCache và vài loại khác) |
| `UpdateReplacePolicy` | Tài nguyên cũ bị xoá khi **update đổi thuộc tính buộc replacement** (ví dụ `Engine`) | Phải khai báo riêng, không tự theo `DeletionPolicy`; không áp dụng khi đổi logical ID |
| Stack policy | Cập nhật lên tài nguyên chỉ định (`Update:Replace`, `Update:Delete`) | Chỉ áp dụng cho update, không ngăn xoá cả stack; xoá cả stack cần termination protection |

Bucket ảnh chứng từ trong lab dùng cả `DeletionPolicy` lẫn `UpdateReplacePolicy` là `Retain`; một RDS PostgreSQL production nên dùng `Snapshot` cho cả hai (đoạn dưới rút gọn, thiếu các thuộc tính bắt buộc khác như class và subnet group):

```yaml
TripsDb:
  Type: AWS::RDS::DBInstance
  DeletionPolicy: Snapshot
  UpdateReplacePolicy: Snapshot
  Properties:
    Engine: postgres
    ManageMasterUserPassword: true
```

Mặc định không giống nhau giữa các loại: `DeletionPolicy` của `AWS::RDS::DBInstance` (không thuộc cluster) mặc định là `Snapshot`, còn `UpdateReplacePolicy` mặc định là `Delete` cho mọi loại. Đừng dựa vào mặc định: ghi tường minh để người review thấy, và bật thêm `DeletionProtection: true` trên chính DB để lệnh xoá bị từ chối. Stack policy nằm trong `stack-policy.json` của lab, cho phép mọi update nhưng từ chối replace và delete bucket. Khi thật sự cần đổi, bạn chỉ định một policy tạm thời cho lần update đó thay vì gỡ policy.

## Drift: khi thực tế lệch template

Drift là khi tài nguyên thật khác với những gì stack ghi nhận, thường vì có người vào console "sửa nhanh" lúc sự cố: mở thêm một rule security group, tắt versioning bucket. Lúc đó template không còn là sự thật. Lần deploy sau có thể ghi đè bản sửa tay (và gây lại sự cố), hoặc lỗi vì trạng thái không như dự kiến.

`detect-stack-drift` so thuộc tính khai báo trong template với tài nguyên thật và đánh dấu `IN_SYNC`, `MODIFIED` hay `DELETED`. Hai giới hạn: không phải loại tài nguyên nào cũng hỗ trợ drift (tra danh sách trước khi tin kết quả "không drift"), và drift detection chỉ báo cáo, không tự sửa; bạn sửa tài nguyên về như template hoặc cập nhật template cho khớp thực tế rồi deploy.

Để giảm drift, hạn chế quyền sửa tài nguyên ngoài role của pipeline (bài 25), và chạy kiểm drift định kỳ (bài 30). Khi cần triển khai cùng một baseline cho nhiều tài khoản hoặc region, dùng **StackSets** (với Organizations ở bài 28, có thể tự tạo stack ở tài khoản mới); bài này chỉ nhắc, chưa dựng lab.

## Dựng cho Tới Nơi: lab vpc.yaml

Template `vpc.yaml` trong lab tạo VPC hai AZ, bốn subnet, Internet Gateway, NAT Gateway tuỳ chọn và bucket `delivery-photos`. Trước khi chạy bất kỳ thứ gì, hãy lint:

```bash
pip install cfn-lint
cfn-lint vpc.yaml
```

`cfn-lint` bắt lỗi sai kiểu, thuộc tính không tồn tại và vi phạm best practice ngay trên máy, không cần tài khoản AWS. Sau đó tạo change set nhưng chưa execute:

```bash
aws cloudformation deploy \
  --template-file vpc.yaml \
  --stack-name toinoi-dev-network \
  --parameter-overrides Env=dev EnableNat=false \
  --tags Project=ToiNoi Env=dev Service=network Owner=platform \
  --no-execute-changeset
```

`deploy` tạo stack nếu chưa có hoặc tạo change set nếu đã có; `--no-execute-changeset` dừng ở bước xem trước. Bạn đọc change set trong console hoặc bằng `describe-change-set`, rồi chạy lại lệnh không có cờ đó để áp dụng. Đừng bật `EnableNat=true` nếu chỉ học: NAT Gateway tính tiền theo giờ ngay khi tồn tại.

Template đã qua `cfn-lint` nhưng **chưa được chạy trên tài khoản thật**; thứ tự tạo, thời gian và thông báo lỗi hãy tự xác nhận khi chạy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="Hình 2. Đổi logical ID của RDS làm CloudFormation tạo tài nguyên mới và xoá tài nguyên cũ, thay vì đổi tên."><defs><marker id="aws21-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="250" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="58" class="t-b">Template cũ</text><text x="34" y="80" class="t-small t-mono">TripsDb (đang chạy)</text><rect x="20" y="120" width="250" height="70" rx="10" class="f-pk-soft s-pk"/><text x="34" y="148" class="t-b">Template mới</text><text x="34" y="170" class="t-small t-mono">TripsDatabase (chưa có)</text><path d="M270 65 L360 65" class="f-none s-warn" stroke-width="2" marker-end="url(#aws21-2-a)"/><path d="M270 155 L360 155" class="f-none s-warn" stroke-width="2" marker-end="url(#aws21-2-a)"/><rect x="362" y="30" width="250" height="70" rx="10" class="f-warn-soft s-warn"/><text x="376" y="58" class="t-b t-warn">Remove TripsDb</text><text x="376" y="80" class="t-small">xoá, theo DeletionPolicy</text><rect x="362" y="120" width="250" height="70" rx="10" class="f-warn-soft s-warn"/><text x="376" y="148" class="t-b t-warn">Add TripsDatabase</text><text x="376" y="170" class="t-small">database mới, rỗng</text><rect x="650" y="75" width="210" height="60" rx="10" class="f-surface s-line dash"/><text x="664" y="102" class="t-b">Dữ liệu cũ</text><text x="664" y="122" class="t-small">còn hay mất tuỳ policy</text></svg>
</div>
<figcaption>Hình 2. Đổi logical ID không phải đổi tên: change set sẽ hiện một dòng Remove và một dòng Add. Đọc change set trước khi execute là lúc duy nhất bắt được điều này.</figcaption>
</figure>

## Lỗi thường gặp

- **Sửa tay rồi quên:** mở port trong console lúc sự cố và không cập nhật template; lần deploy sau xoá mất rule hoặc gây xung đột.
- **Đặt tên cố định cho tài nguyên:** thuộc tính như `BucketName` hay `DBInstanceIdentifier` làm replacement thất bại vì tên đã tồn tại. Để CloudFormation sinh tên, trừ khi tên là hợp đồng với hệ thống ngoài.
- **Export bị khoá:** stack mạng không đổi được giá trị export vì stack service còn import. Cân nhắc tham số hoặc SSM Parameter thay vì export khi hai stack đổi với nhịp khác nhau.
- **Bỏ qua change set:** `update-stack` thẳng vào production, không thấy `Replacement: True`.
- **Một stack khổng lồ:** template có giới hạn kích thước và số tài nguyên; chia theo vòng đời (mạng, dữ liệu, service).

**Bài tập:** trong template có `TripsDb` với `DeletionPolicy: Delete` và không có `UpdateReplacePolicy`. Bạn đổi logical ID thành `TripsDatabase` và chạy deploy. Chuyện gì xảy ra và bạn chặn thế nào?

<details><summary>Xem đáp án</summary><p>Change set có hai dòng: <code>Add TripsDatabase</code> và <code>Remove TripsDb</code>. Database mới được tạo rỗng; database cũ bị xoá ở bước cleanup. Vì tài nguyên cũ bị bỏ khỏi template, CloudFormation áp <code>DeletionPolicy</code> chứ không phải <code>UpdateReplacePolicy</code>; với <code>Delete</code> thì không có snapshot cuối và dữ liệu mất (nếu template đặt cứng <code>DBInstanceIdentifier</code>, bước tạo mới còn lỗi trùng tên). Cách chặn: (1) luôn đọc change set; (2) đặt <code>DeletionPolicy</code> là <code>Retain</code> hoặc <code>Snapshot</code> và bật <code>DeletionProtection</code> trước khi đổi; (3) gắn stack policy cấm <code>Update:Delete</code> cho tài nguyên đó; (4) nếu thật sự muốn đổi logical ID, dùng <b>stack refactoring</b> (ra mắt 2025, lệnh <code>create-stack-refactor</code> với <code>--resource-mappings</code>; tính năng này không chạy trên stack đang gắn stack policy), hoặc cách cũ: đặt <code>Retain</code>, gỡ tài nguyên khỏi stack rồi dùng <b>resource import</b> để nhận lại với logical ID mới.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p03-21-cloudformation` (`vpc.yaml`, `parameters.dev.json`, `stack-policy.json`); README ghi tài nguyên tính tiền và lệnh dọn. Bucket có `Retain` nên xoá stack không xoá bucket.

Bài trước: [bài 20](/articles/aws-cloudfront-waf-shield). Bài tiếp: [bài 22](/articles/aws-terraform-provider-state-s3-lock).

## Tài liệu gốc

- [AWS CloudFormation User Guide](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/Welcome.html)
- [Template anatomy và các section](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/template-anatomy.html)
- [Updating stacks using change sets](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/using-cfn-updating-stacks-changesets.html)
- [Detecting unmanaged configuration changes (drift)](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/using-cfn-stack-drift.html)
- [DeletionPolicy attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-attribute-deletionpolicy.html)
- [cfn-lint trên GitHub](https://github.com/aws-cloudformation/cfn-lint)

---
slug: aws-terraform-provider-state-s3-lock
title: "Terraform trên AWS: provider, resource, remote state trên S3 và lock"
summary: "Bài 22: cài Terraform, cấu hình provider AWS với profile SSO, viết resource VPC và S3 cho Tới Nơi, hiểu state và vì sao state không nằm trên máy cá nhân: backend S3 với use_lockfile, mã hoá, versioning; plan, apply, import và moved block."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, terraform, iac, remote-state, s3-backend, provider]
language: vi
---

Đội hạ tầng Tới Nơi có hai người cùng muốn sửa VPC staging vào chiều thứ Sáu: một người thêm subnet cho service `dispatch`, người kia đổi route cho `tracking`. Cả hai chạy `terraform apply` trên laptop của mình, mỗi máy có một tệp state riêng. Kết quả: hai bản state khác nhau cùng mô tả một VPC thật, và không ai còn biết bản nào đúng.

Đây là **bài 22** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bạn cần đọc [bài 21](/articles/aws-cloudformation-stack-change-set-drift) để có khái niệm IaC và change set, và dùng được AWS CLI với profile SSO (bài 02). Bài này giải thích Terraform đủ để hiểu vì sao state là phần nhạy cảm nhất, rồi dựng backend S3 có khoá.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn xoá nhầm tệp <code>terraform.tfstate</code> trên laptop. Các tài nguyên đã tạo trên AWS có biến mất theo không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Tài nguyên vẫn chạy trên AWS. Nhưng Terraform mất bản đồ giữa tên trong code và ID thật, nên lần <code>plan</code> sau nó thấy mọi thứ là "chưa có" và định tạo lại từ đầu, dẫn đến trùng tên hoặc tài nguyên mồ côi. Vì vậy state phải nằm ở nơi bền, lưu được các phiên bản cũ và có khoá.</p></details>
</div>

## HCL, provider và vòng lệnh cơ bản

Terraform đọc cấu hình viết bằng HCL, so với state rồi gọi API của nhà cung cấp qua **provider**. Với AWS đó là `hashicorp/aws`. Các khối thường gặp:

| Khối | Vai trò | Ví dụ |
|---|---|---|
| `terraform` | Phiên bản Terraform và provider, backend | `required_version = ">= 1.10"` |
| `provider` | Cách kết nối: region, profile, tag mặc định | `profile = "toinoi-lab-admin"` |
| `resource` | Tài nguyên Terraform quản lý | `aws_vpc.main` |
| `data` | Đọc thông tin có sẵn, không tạo mới | `aws_caller_identity`, `aws_availability_zones` |
| `variable` / `output` | Đầu vào và đầu ra | `var.env`, `output "vpc_id"` |
| `locals` | Giá trị tính sẵn trong module | `prefix = "toinoi-${var.env}"` |

Khối `terraform` và `provider` của lab ghim cả Terraform và provider, đồng thời khai báo tag chuẩn một lần bằng `default_tags` thay vì lặp ở từng resource:

```hcl
terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }
}

provider "aws" {
  region  = var.region
  profile = var.aws_profile

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Service = "platform"
      Owner   = var.owner
    }
  }
}
```

Provider đăng nhập qua chuỗi credential của AWS SDK, nên chỉ cần `aws sso login --profile toinoi-lab-admin` rồi trỏ `profile` tới đó; không có access key nào trong tệp `.tf`. Vòng lệnh chuẩn là `terraform init` (tải provider, khởi tạo backend), `plan` (so code với state và thực tế, cho ra danh sách thay đổi), `apply` (thực thi) và `destroy` (xoá mọi thứ trong state).

`init` còn ghi tệp **`.terraform.lock.hcl`** chứa phiên bản và hash provider đã chọn. Hãy commit tệp này: nó khiến mọi máy và CI dùng cùng một bản provider. `version = "~> 6.0"` cho phép 6.x nhưng chặn nhảy sang 7.0; muốn nâng, chạy `terraform init -upgrade` trong một pull request riêng và đọc kỹ plan.

## State: bản đồ giữa code và tài nguyên thật

State là tệp JSON ánh xạ địa chỉ trong code (`aws_vpc.main`) tới ID thật (`vpc-0abc...`) cùng toàn bộ thuộc tính Terraform biết. State cũng là bộ nhớ để Terraform tính phần khác biệt khi bạn đổi code.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Terraform so ba nguồn: code HCL, state và tài nguyên thật trên AWS; state nằm trong S3 có versioning và mã hoá."><defs><marker id="aws22-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="50" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="82" class="t-b">Code HCL</text><text x="36" y="106" class="t-small">aws_vpc.main (mong muốn)</text><rect x="325" y="50" width="230" height="80" rx="10" class="f-pk-soft s-pk"/><text x="341" y="82" class="t-b">State</text><text x="341" y="106" class="t-small">aws_vpc.main = vpc-0abc...</text><rect x="630" y="50" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="646" y="82" class="t-b">AWS thật</text><text x="646" y="106" class="t-small">VPC, subnet, bucket</text><path d="M250 90 L323 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws22-1-a)"/><path d="M630 90 L557 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws22-1-a)"/><text x="262" y="78" class="t-small t-mono">plan</text><text x="570" y="78" class="t-small t-mono">refresh</text><rect x="260" y="160" width="360" height="56" rx="10" class="f-surface s-line dash"/><text x="276" y="184" class="t-b">S3 bucket state</text><text x="276" y="204" class="t-small">versioning + mã hoá + .tflock</text><path d="M440 160 L440 132" class="f-none s-pk" stroke-width="1.6" marker-end="url(#aws22-1-a)"/></svg>
</div>
<figcaption>Hình 1. <code>plan</code> so code với state, <code>refresh</code> so state với thực tế. State lưu trong S3 để cả đội và CI cùng nhìn một bản đồ.</figcaption>
</figure>

Hai hệ quả thực tế. Thứ nhất, state có thể chứa dữ liệu nhạy cảm dưới dạng plaintext (mật khẩu database do Terraform sinh, giá trị `sensitive` vẫn nằm trong state); coi nó như secret: mã hoá, giới hạn người đọc, không đưa lên Git. Thứ hai, state tách theo môi trường và theo vòng đời (mạng, dữ liệu, service), để một lệnh sai không chạm tới mọi thứ.

## Remote state trên S3 với use_lockfile

Từ Terraform 1.10, backend S3 có thể tự khoá bằng `use_lockfile = true`: Terraform ghi một object `<key>.tflock` cạnh tệp state bằng cơ chế ghi có điều kiện (conditional write) của S3, và xoá nó khi xong. Tính năng ra mắt ở 1.10 dưới dạng thử nghiệm, thành GA ở 1.11; cũng từ 1.11, `dynamodb_table` và các tham số DynamoDB khác bị đánh dấu deprecated. Vì vậy Tới Nơi không dựng bảng DynamoDB, và khi vận hành thật nên dùng Terraform 1.11 trở lên.

```hcl
terraform {
  backend "s3" {
    bucket       = "toinoi-lab-tfstate-123456789012-ap-southeast-1"
    key          = "p03-22/lab/terraform.tfstate"
    region       = "ap-southeast-1"
    encrypt      = true
    use_lockfile = true
  }
}
```

Backend không nhận biến, nên giá trị được ghi cứng hoặc truyền bằng `-backend-config`. Trong repo, lab để cấu hình này ở `backend.tf.example` và **không bật**: bạn tạo bucket bằng `bootstrap/` rồi mới đổi tên tệp. Bucket state cần:

- **Versioning bật:** khôi phục được bản state cũ khi apply hỏng.
- **Mã hoá và chặn public access:** AES256 hoặc KMS; thêm policy từ chối kết nối không dùng TLS.
- **Chỉ role pipeline và vài người vận hành truy cập:** bucket policy hoặc IAM giới hạn principal. Cẩn thận: một `Deny` theo `aws:PrincipalArn` thiếu role của bạn sẽ khoá cả bạn, kể cả quyền sửa lại policy. Role của Identity Center có path, nên mẫu phải là `arn:aws:iam::123456789012:role/aws-reserved/sso.amazonaws.com/*/AWSReservedSSO_*`. Lối thoát khi lỡ khoá: root user của tài khoản vẫn xoá được bucket policy, hoặc tài khoản quản lý dùng centralized root access với tác vụ `S3UnlockBucketPolicy`.
- **Quyền tối thiểu:** `s3:ListBucket` trên bucket, `s3:GetObject` và `s3:PutObject` trên key state, và `s3:GetObject`, `s3:PutObject`, `s3:DeleteObject` trên `<key>.tflock`.
- **Key theo môi trường:** `toinoi/dev/network/terraform.tfstate`, `toinoi/prod/network/terraform.tfstate`.

Nếu apply của bạn bị kẹt do tiến trình chết giữa chừng, lần sau Terraform báo lỗi giữ khoá kèm thông tin người giữ; chỉ chạy `terraform force-unlock <ID>` khi chắc không còn tiến trình nào đang chạy.

## Đọc plan và đổi code an toàn

`plan` dùng các ký hiệu chính: `+` tạo, `-` xoá, `~` sửa tại chỗ, và `-/+` thay thế (xoá rồi tạo; với `create_before_destroy` thì hiện `+/-`, tạo trước rồi xoá). Dòng thật sự cần đọc kỹ là thay thế và `-` trên tài nguyên có dữ liệu (RDS, bucket): Terraform ghi rõ thuộc tính nào "forces replacement".

Có ba công cụ để đổi code mà không làm hỏng tài nguyên:

```hcl
# Bảo vệ: plan thất bại nếu có thay đổi định xoá tài nguyên này.
resource "aws_s3_bucket" "state" {
  bucket = "toinoi-lab-tfstate-123456789012-ap-southeast-1"
  lifecycle {
    prevent_destroy = true
  }
}

# Đổi tên trong code mà không xoá tài nguyên thật.
moved {
  from = aws_s3_bucket.photos
  to   = aws_s3_bucket.delivery_photos
}

# Nhận tài nguyên có sẵn (ví dụ bucket tạo tay ở Phần 01) vào state.
import {
  to = aws_s3_bucket.legacy_photos
  id = "toinoi-lab-delivery-photos"
}
```

`prevent_destroy` chỉ chặn khi khối còn trong code; xoá cả resource khỏi code vẫn xoá tài nguyên. `moved` ghi vào state phép đổi địa chỉ lúc apply, là cách đúng để refactor (khác với việc đổi logical ID trong CloudFormation ở bài 21). Khối `import` đưa tài nguyên có sẵn vào state; với `terraform plan -generate-config-out=generated.tf` Terraform có thể sinh cấu hình khởi đầu, bạn vẫn phải đọc và sửa.

## Chọn công cụ IaC cho Tới Nơi

| Tiêu chí | CloudFormation | Terraform | OpenTofu | AWS CDK |
|---|---|---|---|---|
| Ngôn ngữ | YAML/JSON | HCL | HCL | TypeScript, Python, C#... |
| State | AWS giữ (stack) | Bạn giữ (backend) | Bạn giữ (backend) | AWS giữ (qua CloudFormation) |
| Rollback tự động | Có | Không | Không | Có (của CloudFormation) |
| Đa nhà cung cấp | Chỉ AWS | Rất nhiều provider | Cùng hệ provider | Chủ yếu AWS |
| Giấy phép | Dịch vụ AWS | BUSL từ 2023 | Mã nguồn mở (Linux Foundation) | Apache 2.0 |

Tới Nơi chọn Terraform theo tài liệu kiến trúc (mục "Hạ tầng triển khai tham khảo": Terraform hoặc OpenTofu cho IaC), vì đội có thể cần quản lý cả những thứ ngoài AWS (ví dụ repository GitHub hoặc DNS của nhà cung cấp khác) bằng cùng một ngôn ngữ, và HCL dễ đọc khi review. Điều này không có nghĩa CloudFormation hay CDK sai: bài 24 cho thấy khi nào CDK hợp hơn. OpenTofu là fork nên đa phần cấu hình dùng được cho cả hai; OpenTofu hỗ trợ `use_lockfile` từ bản 1.10.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 2. Hai người cùng chạy apply: không có khoá thì bản ghi sau đè bản trước; có khoá tflock thì người thứ hai bị chặn."><defs><marker id="aws22-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws22-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="190" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="56" class="t-b">An: apply</text><text x="36" y="76" class="t-small">thêm subnet dispatch</text><rect x="20" y="140" width="190" height="60" rx="10" class="f-warn-soft s-warn"/><text x="36" y="166" class="t-b t-warn">Bình: apply</text><text x="36" y="186" class="t-small">đổi route tracking</text><rect x="330" y="70" width="260" height="90" rx="10" class="f-pk-soft s-pk"/><text x="346" y="98" class="t-b">S3: terraform.tfstate</text><text x="346" y="120" class="t-small t-mono">terraform.tfstate.tflock</text><text x="346" y="142" class="t-small">An giữ khoá</text><path d="M210 60 L328 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws22-2-a)"/><path d="M210 170 L328 140" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws22-2-w)"/><rect x="650" y="60" width="210" height="110" rx="10" class="f-surface s-line"/><text x="666" y="88" class="t-b">Không có lock</text><text x="666" y="110" class="t-small">hai bản state ghi đè nhau</text><text x="666" y="136" class="t-b">Có use_lockfile</text><text x="666" y="158" class="t-small">Bình thấy lỗi giữ khoá</text></svg>
</div>
<figcaption>Hình 2. Khoá <code>.tflock</code> nằm cạnh state trong S3. Người thứ hai phải chờ, thay vì ghi đè state của người thứ nhất.</figcaption>
</figure>

## Lab và kiểm chứng

Lab ở `examples/toi-noi-aws/p03-22-terraform-basics`: `main.tf` dựng VPC hai AZ (subnet tạo bằng `for_each`) và bucket ảnh chứng từ có versioning, mã hoá và chặn public; `bootstrap/` tạo bucket state; `backend.tf.example` là cấu hình backend, chưa bật. Kiểm tra trên máy bằng Docker, không cần tài khoản AWS:

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ bắt lỗi cú pháp và kiểu. Việc tạo tài nguyên thật, quyền của role và hành vi `use_lockfile` **chưa được kiểm chứng trên tài khoản thật** khi viết bài. Lab không có NAT Gateway nên không phát sinh phí theo giờ; README ghi từng lệnh dọn.

## Cái giá phải trả

Những điều bạn đánh đổi khi dùng Terraform:

- **State là single point of failure:** mất hoặc hỏng state là mất bản đồ. Versioning S3 và phân quyền chặt là bắt buộc, không phải tuỳ chọn.
- **Không rollback tự động:** apply lỗi giữa chừng để lại trạng thái nửa vời; bạn sửa code rồi apply tiếp hoặc dùng `terraform state` để chỉnh.
- **Drift không báo trước:** sửa tay trên console chỉ lộ ra ở lần `plan` sau. Kiểm tra định kỳ ở bài 30.
- **Provider và Terraform đều liên tục ra phiên bản mới:** ghim bằng lockfile và `required_version`, nâng trong pull request riêng.
- **Secret nằm trong state:** ưu tiên để dịch vụ quản lý secret (bài 23 dùng RDS-managed password).

**Bài tập:** hai kỹ sư cùng chạy `terraform apply` vào cùng một state trên S3 nhưng backend **không** bật `use_lockfile`. Chuyện gì có thể xảy ra, và S3 versioning giúp được gì?

<details><summary>Xem đáp án</summary><p>Cả hai đọc cùng một state, tính plan trên bản đó rồi cùng ghi. Bản ghi sau đè bản ghi trước, nên state mất dấu những tài nguyên người kia vừa tạo (thành tài nguyên mồ côi), hoặc hai bên cùng tạo một thứ và một bên lỗi giữa chừng. Trong trường hợp xấu, tệp state bị ghi dở. Versioning không ngăn xung đột nhưng cho phép khôi phục phiên bản state trước đó, rồi dùng <code>import</code> để nhận lại các tài nguyên bị bỏ sót. Với <code>use_lockfile = true</code>, người thứ hai nhận lỗi giữ khoá ngay lúc bắt đầu và phải chờ.</p></details>

Bài trước: [bài 21](/articles/aws-cloudformation-stack-change-set-drift). Bài tiếp: [bài 23](/articles/aws-terraform-module-vpc-ecs-rds).

## Tài liệu gốc

- [Terraform: S3 backend](https://developer.hashicorp.com/terraform/language/backend/s3)
- [Terraform: State](https://developer.hashicorp.com/terraform/language/state)
- [Terraform: moved block và refactoring](https://developer.hashicorp.com/terraform/language/modules/develop/refactoring)
- [Terraform: import block](https://developer.hashicorp.com/terraform/language/import)
- [Terraform AWS provider trên Registry](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Cấu hình AWS CLI với IAM Identity Center](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html)

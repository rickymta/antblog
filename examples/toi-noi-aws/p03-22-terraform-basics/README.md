# Lab bài 22 · Terraform trên AWS: provider, state trên S3 và lock

Mã đi kèm bài [`aws-terraform-provider-state-s3-lock`](../../../api/seed/content/aws-terraform-provider-state-s3-lock.md) của lộ trình **AWS qua Tới Nơi · Phần 03**.

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `versions.tf` | `required_version >= 1.10`, provider `hashicorp/aws ~> 6.0`, profile SSO và `default_tags`. |
| `variables.tf` | Biến có `validation` (env, số AZ). |
| `main.tf` | VPC hai AZ (public và private subnet, route table), bucket `delivery-photos` có versioning, mã hoá và chặn public access. |
| `outputs.tf` | ID VPC, subnet và tên bucket. |
| `backend.tf.example` | Ví dụ backend S3 với `use_lockfile = true`. **Chưa bật**: đổi tên thành `backend.tf` sau khi có bucket state. |
| `terraform.tfvars.example` | Mẫu giá trị biến; sao chép thành `terraform.tfvars` (đừng commit). |
| `bootstrap/` | Root module riêng tạo bucket state (versioning, mã hoá, chặn public, policy chỉ cho role được liệt kê). |

Lab này không có NAT Gateway: private subnet không có đường ra Internet, đủ để học state và plan mà không phát sinh phí theo giờ.

## Tài nguyên tạo ra và tính tiền thế nào

- **VPC, subnet, route table, Internet Gateway:** không tính tiền riêng.
- **S3 bucket (delivery-photos và state):** tính theo dung lượng lưu và số request; gần như không tốn khi rỗng. Versioning giữ cả phiên bản cũ nên dung lượng state tăng theo số lần apply.
- **Tệp `.tflock`:** là một object nhỏ trong bucket state, tồn tại trong lúc có lệnh đang chạy.

Terraform không tính phí. Ước tính chi phí thật bằng [Pricing Calculator](https://calculator.aws/).

## Kiểm trên máy (không cần tài khoản AWS)

Chạy trong thư mục này, rồi lặp lại trong `bootstrap/`:

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu; không chứng minh quyền, quota hay hành vi thật trên AWS. Xoá `.terraform/` và `.terraform.lock.hcl` nếu bạn không muốn giữ (chỉ commit lockfile khi bạn tự quản lý repo thật; xem bài).

## Chạy trong tài khoản lab của bạn

Các lệnh dưới đây **chưa được chạy trên tài khoản thật** khi viết bài.

```bash
# 1. Tạo bucket state (state của bước này ở local, vì chưa có nơi nào khác).
cd bootstrap
terraform init
terraform apply -var 'aws_profile=toinoi-lab-admin'

# (Tuỳ chọn) Siết bucket policy: chỉ các role trong danh sách được đọc/ghi state.
# Danh sách PHẢI có role bạn đang dùng; precondition sẽ chặn apply nếu thiếu.
terraform apply -var 'aws_profile=toinoi-lab-admin' \
  -var 'state_access_principal_arns=["arn:aws:iam::123456789012:role/aws-reserved/sso.amazonaws.com/*/AWSReservedSSO_AdministratorAccess_*","arn:aws:iam::123456789012:role/toinoi-github-terraform-*"]'

# 2. Quay lại thư mục lab, bật backend S3.
cd ..
cp backend.tf.example backend.tf   # sửa bucket cho đúng output ở bước 1
terraform init                     # state lưu trên S3, khoá bằng .tflock
terraform plan -var 'aws_profile=toinoi-lab-admin'
terraform apply -var 'aws_profile=toinoi-lab-admin'
```

Role chạy Terraform cần các quyền S3 tối thiểu: `s3:ListBucket` trên bucket state, `s3:GetObject` và `s3:PutObject` trên key state, và với `use_lockfile` còn cần `s3:GetObject`, `s3:PutObject`, `s3:DeleteObject` trên `<key>.tflock`. Nên dùng Terraform 1.11 trở lên: `use_lockfile` thử nghiệm ở 1.10 và GA ở 1.11.

### Nếu lỡ tự khoá mình khỏi bucket state

Statement `DenyAllExceptStateRoles` từ chối mọi principal không có trong danh sách, kể cả quyền `s3:PutBucketPolicy` để sửa lại. Precondition trong `bootstrap/main.tf` chặn trường hợp phổ biến nhất (role đang apply không có trong danh sách), nhưng vẫn có thể khoá người khác hoặc pipeline. Lối thoát:

- Đăng nhập root user của tài khoản và xoá bucket policy (root user của tài khoản sở hữu bucket luôn xoá được bucket policy).
- Nếu tài khoản thành viên đã tắt root credential (bài 28), từ tài khoản quản lý dùng centralized root access: `aws sts assume-root --target-principal <account-id> --task-policy-arn arn=arn:aws:iam::aws:policy/root-task/S3UnlockBucketPolicy`, rồi `aws s3api delete-bucket-policy --bucket <bucket>` bằng credential vừa nhận.

Sau đó apply lại `bootstrap/` với danh sách đúng.

## Dọn tài nguyên

```bash
# Lab chính (nếu bucket delivery-photos còn object, đặt force_destroy_photos_bucket=true và apply trước).
terraform destroy -var 'aws_profile=toinoi-lab-admin'

# Bucket state có prevent_destroy = true. Muốn xoá: bỏ khối lifecycle trong bootstrap/main.tf,
# xoá hết object và version trong bucket, rồi terraform destroy trong bootstrap/.
```

Nếu đã bật `backend.tf`, chạy destroy lab chính **trước** khi xoá bucket state; nếu không, bạn mất bản đồ tới tài nguyên đang chạy.

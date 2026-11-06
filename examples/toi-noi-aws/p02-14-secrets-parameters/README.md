# Lab p02-14-secrets-parameters — cấu hình và secret cho service đặt chuyến

Đi kèm bài 14: *Secrets Manager và Parameter Store: cấu hình, secret và rotation cho service Tới Nơi* (slug `aws-secrets-manager-parameter-store`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Parameter Store, loại Standard (4 parameter) | Không tính phí lưu trữ. Gọi API ở mức throughput mặc định không tính thêm. |
| Parameter Store, loại Advanced (không dùng trong lab) | Tính theo parameter mỗi tháng và theo số lần gọi API. |
| Secret `toinoi/lab/booking/db` | **Tính theo secret mỗi tháng** (tính theo tỉ lệ ngày nếu xoá sớm) và theo mỗi 10.000 lần gọi API. |
| KMS key `alias/toinoi-lab-config` | **Tính theo key mỗi tháng** và theo số lần gọi API. Xoá key có thời gian chờ tối thiểu 7 ngày, nên key vẫn tồn tại một thời gian sau `terraform destroy`. |
| IAM role và policy | Không tính tiền. |

Không ghi giá cụ thể vì giá đổi theo thời gian: dùng AWS Pricing Calculator.

## Điều kiện

- Terraform >= 1.10, AWS CLI v2 với profile SSO, `jq`.
- ARN của role SSO (hoặc principal khác) của bạn để cho phép assume role đọc cấu hình; xem `aws sts get-caller-identity`.

## Chạy

```bash
cd examples/toi-noi-aws/p02-14-secrets-parameters
terraform init
terraform apply -var reader_principal_arn=arn:aws:iam::123456789012:role/aws-reserved/sso.amazonaws.com/ap-southeast-1/AWSReservedSSO_Lab_0123456789abcdef
```

Chuỗi trên là giá trị giả: thay bằng ARN principal của bạn. Terraform chỉ tạo "vỏ" cho SecureString và secret với giá trị giữ chỗ; đặt giá trị thật bằng CLI để chúng không nằm trong Terraform state:

```bash
scripts/seed-values.sh
scripts/read-as-role.sh "$(terraform output -raw reader_role_arn)"
scripts/versions-demo.sh
```

Các lệnh CLI cơ bản, nếu muốn gõ tay:

```bash
aws ssm put-parameter --name /toinoi/lab/booking/quote-cache-ttl-seconds --type String --value 60 --overwrite
aws ssm get-parameters-by-path --path /toinoi/lab/booking --recursive --with-decryption
aws secretsmanager list-secret-version-ids --secret-id toinoi/lab/booking/db
```

Dùng secret trong ECS: lấy `terraform output -raw db_secret_arn` và gắn `reader_policy_arn` vào execution role. Lưu ý: secret ở đây là **JSON**, còn task definition của lab p02-12 inject nguyên giá trị secret vào `ConnectionStrings__Trips` (connection string Npgsql như bài 36 loạt .NET). Đừng trỏ thẳng `valueFrom` đó vào secret này; hoặc inject từng trường bằng hậu tố khoá JSON (`<arn>:password::`, `<arn>:host::`…) và để app tự ghép chuỗi kết nối, hoặc tạo secret riêng chứa connection string cho lab p02-12.

### Rotation (không nằm trong lab này)

Rotation bằng Lambda cần hàm rotation (từ Serverless Application Repository hoặc tự viết) và đường mạng từ hàm tới database; lab không tạo chúng. Với secret mật khẩu master do RDS quản lý (`manage_master_user_password`), RDS tự lo rotation. Cả hai cách **chưa được kiểm chứng trên tài khoản thật** trong lúc soạn bài. `scripts/versions-demo.sh` chỉ mô phỏng thủ công hai nhãn phiên bản `AWSCURRENT` và `AWSPREVIOUS`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
node -e 'JSON.parse(require("fs").readFileSync("reader-policy.json"))'
bash -n scripts/*.sh
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. **Chưa kiểm chứng trên tài khoản thật:** điều kiện `secretsmanager:ResourceTag/Project` và `kms:ViaService` trong policy đọc, việc ECS inject đúng khoá JSON, và hành vi rotation.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p02-14-secrets-parameters/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy -var reader_principal_arn=<giống lúc apply>
```

Secret được xoá ngay nhờ `recovery_window_in_days = 0` (chỉ dành cho lab; production nên giữ 7–30 ngày). KMS key chuyển sang trạng thái chờ xoá 7 ngày. Kiểm tra:

```bash
aws secretsmanager list-secrets --query 'SecretList[].Name'
aws ssm describe-parameters --parameter-filters Key=Name,Option=BeginsWith,Values=/toinoi/lab/booking
aws kms describe-key --key-id alias/toinoi-lab-config --query KeyMetadata.KeyState
```

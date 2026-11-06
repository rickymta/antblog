# Lab p02-12-ecs-fargate — service đặt chuyến trên ECS Fargate

Đi kèm bài 12: *ECS Fargate: cluster, task definition, service, execution role và task role* (slug `aws-ecs-fargate-task-definition-roles`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Cluster ECS | Không tính riêng khi dùng Fargate. |
| Task Fargate (`desired_count` = 1, 0,5 vCPU, 1 GB) | **Tính theo giây** theo vCPU và bộ nhớ trong lúc task chạy. Đây là khoản chính của lab. |
| CloudWatch Logs (`/ecs/toinoi-lab-booking`) | Theo dung lượng log nạp vào và lưu trữ; retention đặt 14 ngày. |
| IAM role, security group | Không tính tiền. |
| NAT gateway hoặc VPC endpoint (có từ bài 04 và bài 11) | Task trong private subnet cần một trong hai để pull image, đọc secret và ghi log; chúng tính theo giờ và theo GB. |
| ECS Exec | Không tính riêng; dùng Session Manager qua kênh `ssmmessages`. |

Không ghi giá cụ thể vì giá đổi theo thời gian: dùng AWS Pricing Calculator.

## Điều kiện

- Đã làm các bài 04 (VPC), 06 (bucket ảnh), 07 (RDS) và 11 (ECR có image). Terraform ở đây nhận ID/ARN của chúng qua biến.
- Một secret Secrets Manager chứa **nguyên connection string Npgsql** (như bài 36 loạt .NET), vì task definition inject cả giá trị vào `ConnectionStrings__Trips`. Secret JSON của lab p02-14 không dùng thẳng được; xem bài 14 về các cách dung hoà.
- Terraform >= 1.10, AWS CLI v2 với profile SSO, `envsubst` (gói `gettext`) cho script CLI, Session Manager plugin nếu muốn thử ECS Exec.
- Image phải có `curl` nếu giữ `healthCheck` của container. Image `aspnet` mặc định và image chiseled thường **không có curl**: hoặc thêm curl vào Dockerfile, hoặc xoá khối `healthCheck` và dựa vào health check của ALB (bài 13).

## Cách A: Terraform tạo tất cả

```bash
cd examples/toi-noi-aws/p02-12-ecs-fargate
terraform init
terraform apply \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var image=123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking@sha256:REPLACE_WITH_DIGEST \
  -var db_secret_arn=arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/booking/db-connection-string-AbCdEf
```

Các giá trị `vpc-…`, `subnet-…`, `REPLACE_WITH_DIGEST` và hậu tố `-AbCdEf` là giả: thay bằng giá trị của tài khoản lab.

## Cách B: Terraform tạo nền, AWS CLI tạo task definition và service

Cách này giúp bạn thấy API thô mà Terraform che đi.

```bash
terraform apply -var manage_workload=false \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var image=unused -var db_secret_arn=arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/booking/db-connection-string-AbCdEf

export EXECUTION_ROLE_ARN="$(terraform output -raw execution_role_arn)"
export TASK_ROLE_ARN="$(terraform output -raw task_role_arn)"
export SECURITY_GROUP="$(terraform output -raw task_security_group_id)"
export IMAGE_DIGEST=123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking@sha256:REPLACE_WITH_DIGEST
export DB_SECRET_ARN=arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/booking/db-connection-string-AbCdEf
export SUBNETS=subnet-0aaa1111,subnet-0bbb2222

scripts/register-and-create.sh
```

Quan sát:

```bash
aws ecs describe-services --cluster toinoi-lab --services toinoi-lab-booking \
  --query 'services[0].events[:5].message'
scripts/debug.sh
```

`task-definition.json` là bản đầy đủ dùng cho `register-task-definition`; bản `jsonencode` trong `main.tf` tương đương. Hai bản được viết tay riêng, nên khi sửa một bên hãy sửa cả bên kia.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
node -e 'JSON.parse(require("fs").readFileSync("task-definition.json"))'
bash -n scripts/*.sh
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. **Chưa kiểm chứng trên tài khoản thật:** quyền của hai role, việc task pull được image và đọc được secret trong mạng của bạn, hành vi của `healthCheck` với image cụ thể, ECS Exec.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p02-12-ecs-fargate/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

Cách A: `terraform destroy` với cùng các biến. Cách B: chạy `scripts/cleanup.sh` trước, rồi `terraform destroy -var manage_workload=false …`. Kiểm tra lại:

```bash
aws ecs list-clusters
aws ecs list-services --cluster toinoi-lab
```

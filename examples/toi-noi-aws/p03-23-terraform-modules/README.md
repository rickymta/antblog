# Lab bài 23 · Terraform module: VPC, ECS Fargate và RDS theo môi trường

Mã đi kèm bài [`aws-terraform-module-vpc-ecs-rds`](../../../api/seed/content/aws-terraform-module-vpc-ecs-rds.md) của lộ trình **AWS qua Tới Nơi · Phần 03**.

## Cấu trúc

```text
p03-23-terraform-modules/
├── modules/
│   ├── network/        VPC, public/private subnet theo AZ, route table, NAT Gateway tuỳ chọn
│   ├── rds-postgres/   DB subnet group, security group, aws_db_instance (mật khẩu do RDS quản lý)
│   └── ecs-service/    log group, execution role, task role, task definition, ECS service, security group
└── envs/
    └── dev/            ghép network -> rds -> service (for_each), tfvars và backend riêng
```

`envs/staging` và `envs/prod` không có trong lab để bài gọn: chúng là bản sao của `envs/dev` với `backend.tf`, `*.tfvars` và CIDR riêng (xem bài). Mỗi môi trường là một root module có state riêng; nên đặt ở tài khoản AWS riêng khi đã làm bài 28.

## Tài nguyên tạo ra và tính tiền thế nào

Nếu bạn tự `apply` `envs/dev` trong tài khoản lab (mặc định tạo ba service `quote`, `booking`, `dispatch`):

- **NAT Gateway và Elastic IP** (`enable_nat_gateway = true`): tính theo giờ tồn tại và theo dung lượng dữ liệu đi qua. Đây là khoản dễ bị quên nhất; tắt biến này thì task Fargate trong private subnet không kéo được image từ ECR (trừ khi có VPC endpoint, xem bài 53).
- **ECS Fargate:** tính theo vCPU và bộ nhớ cấp cho task trong thời gian task chạy. Có 2 + 2 + 2 task theo mặc định.
- **RDS PostgreSQL:** tính theo instance class mỗi giờ, dung lượng lưu, backup vượt quota; Multi-AZ gần như gấp đôi phần instance.
- **Secrets Manager:** secret do RDS quản lý tính theo secret mỗi tháng và số lần gọi API.
- **CloudWatch Logs:** theo dung lượng log ghi vào và lưu giữ.
- **VPC, subnet, route table, security group, IAM role:** không tính tiền riêng.

Ước tính bằng [Pricing Calculator](https://calculator.aws/). Cách chỉnh nhanh để rẻ hơn: `desired_count = 1`, `enable_nat_gateway = false` khi chỉ muốn `plan`, và nhớ dọn.

## Kiểm trên máy (không cần tài khoản AWS)

Chạy từ thư mục `p03-23-terraform-modules`. Vì `envs/dev` tham chiếu `../../modules`, cần mount cả thư mục lab và dùng `-chdir`:

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 -chdir=envs/dev init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 -chdir=envs/dev validate
```

`validate` kiểm cú pháp, kiểu, tham chiếu giữa module và vòng phụ thuộc, nhưng **không** chứng minh quyền IAM, quota, giá trị `engine_version`, hay việc `plan` thành công trên tài khoản thật. Chưa có `terraform plan` hay `apply` nào được chạy với mã này.

Xoá `.terraform/` và `.terraform.lock.hcl` sinh ra trong các thư mục sau khi kiểm nếu bạn không dùng tiếp.

## Chạy trong tài khoản lab của bạn

```bash
cd envs/dev
cp backend.tf.example backend.tf        # sau khi có bucket state (bài 22); hoặc bỏ qua để dùng state local khi học
terraform init
terraform plan  -var-file=dev.tfvars -var 'aws_profile=toinoi-lab-admin'
terraform apply -var-file=dev.tfvars -var 'aws_profile=toinoi-lab-admin'
```

Image `<account>.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/<service>:1.0.0` phải tồn tại trong ECR (bài 11); nếu chưa, ECS service vẫn tạo được nhưng task không khởi động; deployment circuit breaker đánh dấu deployment thất bại (lần deploy đầu không có bản cũ để rollback). Mật khẩu database do RDS sinh và nằm trong Secrets Manager (`terraform output db_secret_arn`); không có mật khẩu nào trong tfvars.

## Dọn tài nguyên

```bash
terraform destroy -var-file=dev.tfvars -var 'aws_profile=toinoi-lab-admin'
```

Với dev, `db_deletion_protection = false` và `db_skip_final_snapshot = true` để destroy chạy được. **Không** dùng hai giá trị này ở staging và prod. Nếu bật lại `deletion_protection`, phải apply tắt nó trước khi destroy.

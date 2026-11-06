# Lab p01-04-vpc

Lab của **bài 04 · VPC: subnet public và private, route table, NAT, security group và NACL** (`api/seed/content/aws-vpc-subnet-route-security-group.md`). Phần 01, cấp beginner.

Dựng VPC `10.20.0.0/16` của Tới Nơi: 6 subnet ở 2 AZ, Internet Gateway, route table và ba security group (`sg-alb` → `sg-app:8080` → `sg-db:5432`). Có hai cách làm tương đương: script AWS CLI (thư mục này) và Terraform (`terraform/`). **Dùng một trong hai, không dùng cả hai cho cùng một VPC.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tạo khi nào | Tính tiền |
|---|---|---|
| VPC, subnet, route table, security group, NACL | Luôn (NACL: `--with-nacl`) | Không tính phí riêng |
| Internet Gateway | Luôn | Không tính phí riêng (traffic ra Internet tính theo GB) |
| NAT Gateway + Elastic IP | Chỉ khi `--with-nat` (Terraform: `enable_nat_gateway = true`) | **Có**: theo giờ tồn tại và theo GB đi qua NAT; địa chỉ IPv4 công khai tính theo giờ. Xem trang giá Amazon VPC |
| VPC Flow Logs, Reachability Analyzer | Không tạo trong lab | Có nếu bạn tự bật: xem trang giá |

Mặc định **không tạo NAT**. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Service=network`, `Owner=platform`.

Các lab sau tìm tài nguyên của bài này **theo tag Name**, không cần chép ID: bài 05 (`--use-lab-vpc`) dùng `toinoi-lab-app-a` và sg `toinoi-lab-app`; bài 07 dùng `toinoi-lab-data-a/-b` và sg `toinoi-lab-db`; bài 08 dùng `toinoi-lab-public-a/-b`, `toinoi-lab-app-a/-b`, sg `toinoi-lab-alb` (thêm luật cổng 80) và `toinoi-lab-app`. Đừng đổi các tên này. Bài 05, 07, 08 cần đường ra Internet nên hãy dựng với NAT (`--with-nat` hoặc `enable_nat_gateway = true`).

## Tệp trong lab

| Tệp | Công dụng |
|---|---|
| `create-vpc.sh` | Dựng toàn bộ bằng AWS CLI. Dry-run mặc định; `--apply`, `--with-nat`, `--with-nacl` |
| `cleanup.sh` | Tìm VPC `toinoi-lab-vpc` theo tag và xoá theo thứ tự phụ thuộc. Dry-run mặc định |
| `terraform/` | Cấu hình Terraform tương đương (`main.tf`, `variables.tf`, `outputs.tf`) |

## Kiểm trên máy (không cần tài khoản AWS)

```bash
bash -n create-vpc.sh cleanup.sh
cd terraform
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
rm -rf .terraform .terraform.lock.hcl
```

Script đã qua shellcheck, chạy thử với lệnh `aws` giả lập, và tham số của 48 lệnh `aws ec2` được đối chiếu với bộ mô hình dịch vụ của AWS CLI (không gọi AWS thật); Terraform đã qua `fmt -check` và `validate`. **Chưa chạy trên tài khoản AWS thật.** Cụ thể chưa kiểm chứng: hành vi `cleanup.sh` khi còn ENI của ALB hoặc RDS trong subnet; NACL tuỳ chọn với RDS ở bài 07.

## Chạy bằng AWS CLI

```bash
aws sso login --profile toinoi-lab
./create-vpc.sh toinoi-lab                       # in lệnh, không tạo gì
./create-vpc.sh toinoi-lab --apply               # tạo VPC, không NAT
./create-vpc.sh toinoi-lab --apply --with-nat    # thêm NAT Gateway (tính tiền)
```

Kiểm tra sau khi tạo:

```bash
aws ec2 describe-subnets --profile toinoi-lab \
  --filters Name=tag:Project,Values=ToiNoi Name=tag:Env,Values=lab \
  --query 'Subnets[].[Tags[?Key==`Name`]|[0].Value,CidrBlock,AvailabilityZone]' --output table
```

## Chạy bằng Terraform

```bash
cd terraform
terraform init
terraform plan
terraform apply                                   # không NAT
terraform apply -var enable_nat_gateway=true      # thêm NAT (tính tiền)
```

Lab không có backend thật; state nằm cục bộ. Remote state trên S3 có ở bài 22.

## Dọn tài nguyên

Bản CLI:

```bash
./cleanup.sh toinoi-lab --apply
```

Bản Terraform: `terraform destroy` (với cùng `-var` đã dùng khi apply).

Nếu còn ALB, RDS hay EC2 của các bài sau trong VPC, hãy xoá chúng trước; nếu không `delete-subnet` sẽ lỗi vì còn ENI. Sau khi dọn, kiểm tra console VPC và mục NAT Gateways, Elastic IPs để chắc chắn không còn gì tính tiền.

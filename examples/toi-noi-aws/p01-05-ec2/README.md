# Lab p01-05-ec2

Lab của **bài 05 · EC2: chạy service đầu tiên của Tới Nơi bằng AMI, EBS, user data và Session Manager** (`api/seed/content/aws-ec2-ami-ebs-user-data.md`). Phần 01, cấp beginner.

Tạo một instance Amazon Linux 2023 tên `toinoi-lab-quote`, chạy container nginx **placeholder** nghe cổng 8080 (trả `200` ở `/health/ready` và một báo giá cố định ở `/v1/quote`), truy cập bằng Session Manager chứ không bằng SSH. Có hai cách tương đương: script AWS CLI (thư mục này) và Terraform (`terraform/`). **Dùng một trong hai.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Ghi chú | Tính tiền |
|---|---|---|
| EC2 instance (`t3.micro`, hoặc `t4g.micro` với `--arm`) | IMDSv2 bắt buộc, không SSH | Có: theo thời gian chạy. Xem trang giá EC2 và điều kiện free tier hiện hành |
| EBS root gp3 8 GiB, mã hoá | Xoá khi terminate | Có: theo GB-tháng, kể cả khi máy đang stop |
| IPv4 công khai (chế độ default VPC) | Subnet mặc định tự gán | Có: theo giờ |
| NAT Gateway (chỉ khi dùng VPC bài 04 với `--with-nat`) | Để SSM Agent và Docker ra Internet | Có: theo giờ và theo GB (xem bài 04) |
| IAM role + instance profile, security group | `toinoi-lab-quote-ec2`, `toinoi-lab-quote` | Không tính phí riêng |
| Snapshot (`snapshot.sh`, tuỳ chọn) | Chụp ổ root | Có: theo GB lưu |

Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Service=quote`, `Owner=platform`. Máy chỉ ra Internet để tải Docker và image nginx; không có cổng nào mở vào máy.

## Tệp trong lab

| Tệp | Công dụng |
|---|---|
| `user-data.sh` | Script cloud-init: cài Docker, ghi cấu hình nginx, chạy container `quote` cổng 8080 |
| `trust-policy-ec2.json` | Trust policy cho role của EC2 |
| `create-instance.sh` | Tạo role, instance profile, security group, instance. Dry-run mặc định; `--apply`, `--use-lab-vpc`, `--arm` |
| `check.sh` | Gọi `/health/ready` và `/v1/quote` trên máy qua Session Manager `send-command` |
| `snapshot.sh` | Chụp snapshot ổ root (tuỳ chọn, dry-run mặc định) |
| `cleanup.sh` | Terminate instance, xoá snapshot, security group, instance profile và role (dry-run mặc định) |
| `terraform/` | Cấu hình Terraform tương đương. `terraform/user-data.sh` là bản sao của `user-data.sh`, phải giữ giống nhau |

## Kiểm trên máy (không cần tài khoản AWS)

```bash
bash -n create-instance.sh check.sh cleanup.sh snapshot.sh user-data.sh
node -e 'JSON.parse(require("fs").readFileSync("trust-policy-ec2.json"))'
diff user-data.sh terraform/user-data.sh && echo "bản sao user data khớp"
cd terraform
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
rm -rf .terraform .terraform.lock.hcl
```

Script đã qua shellcheck và chạy thử với lệnh `aws` giả lập; tham số của các lệnh `aws` được kiểm bằng mô hình dịch vụ của AWS CLI (không gọi AWS thật). Terraform đã qua `fmt -check` và `validate`. **Chưa chạy trên tài khoản AWS thật.** Chưa kiểm chứng: thời gian SSM Agent báo Online; việc tải `nginx:stable-alpine` từ Docker Hub (có thể gặp giới hạn pull ẩn danh); hành vi `send-command` trong `check.sh`.

## Chạy bằng AWS CLI

Cần profile `toinoi-lab` đã đăng nhập (bài 02).

```bash
aws sso login --profile toinoi-lab
./create-instance.sh toinoi-lab                 # in lệnh
./create-instance.sh toinoi-lab --apply         # tạo thật trong default VPC
./check.sh toinoi-lab                           # kỳ vọng: "ready" rồi JSON báo giá 25500
```

Dùng VPC bài 04 (instance ở subnet private-app, không có IP công khai): chạy `../p01-04-vpc/create-vpc.sh toinoi-lab --apply --with-nat` trước, rồi `./create-instance.sh toinoi-lab --apply --use-lab-vpc`. Không có NAT, SSM Agent không ra được ngoài và máy sẽ không bao giờ Online.

Mở phiên tương tác (cần cài Session Manager plugin cho AWS CLI): `aws ssm start-session --target <instance-id> --profile toinoi-lab`.

Tuỳ chọn: `./snapshot.sh toinoi-lab --apply` chụp snapshot ổ root.

## Chạy bằng Terraform

```bash
cd terraform
terraform init
terraform apply                                                    # default VPC
terraform apply -var 'subnet_id=subnet-xxxxxxxx'                   # subnet private-app (cần NAT)
terraform apply -var architecture=arm64                            # Graviton
```

Lab không có backend thật; state nằm cục bộ (remote state ở bài 22).

## Dọn tài nguyên

Bản CLI: `./cleanup.sh toinoi-lab --apply`. Bản Terraform: `terraform destroy` (cùng `-var` đã dùng). Nếu đã dùng VPC bài 04, dọn tiếp bằng `../p01-04-vpc/cleanup.sh toinoi-lab --apply`, đặc biệt là NAT Gateway. Sau khi dọn, kiểm tra console EC2 (Instances, Volumes, Snapshots) và VPC (NAT Gateways, Elastic IPs).

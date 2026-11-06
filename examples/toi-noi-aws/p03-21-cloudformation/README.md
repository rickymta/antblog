# Lab bài 21 · CloudFormation: template, stack, change set, drift

Mã đi kèm bài [`aws-cloudformation-stack-change-set-drift`](../../../api/seed/content/aws-cloudformation-stack-change-set-drift.md) của lộ trình **AWS qua Tới Nơi · Phần 03**.

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `vpc.yaml` | Template: VPC hai AZ (2 public, 2 private subnet), Internet Gateway, NAT Gateway tuỳ chọn, bucket `delivery-photos` có versioning, mã hoá và chặn public access. |
| `parameters.dev.json` | Giá trị tham số cho môi trường `dev`. |
| `stack-policy.json` | Stack policy chặn `Update:Replace` và `Update:Delete` lên bucket `DeliveryPhotosBucket`. |

## Tài nguyên tạo ra và tính tiền thế nào

Không có lệnh nào trong repo tự tạo tài nguyên. Nếu bạn tự chạy trong tài khoản lab:

- **VPC, subnet, route table, Internet Gateway, stack CloudFormation:** không tính tiền riêng.
- **NAT Gateway và Elastic IP:** chỉ tạo khi `Env=prod` hoặc `EnableNat=true`. NAT Gateway tính tiền theo giờ tồn tại cộng theo dung lượng dữ liệu đi qua. Mặc định lab để `EnableNat=false` để không phát sinh phí này.
- **S3 bucket:** tính theo dung lượng lưu và số request; bucket rỗng gần như không tốn gì.

Xem [Pricing Calculator](https://calculator.aws/) để ước tính theo số liệu của bạn.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w python:3.12-slim sh -c 'pip install -q cfn-lint && cfn-lint vpc.yaml'
```

Lệnh trên chạy `cfn-lint` trong container tạm; không in gì nghĩa là template qua kiểm tra. Kiểm JSON:

```bash
node -e 'for (const f of ["stack-policy.json","parameters.dev.json"]) JSON.parse(require("fs").readFileSync(f)); console.log("json ok")'
```

## Chạy trong tài khoản lab của bạn

Dùng profile SSO (xem bài 02). Đây là các lệnh minh hoạ, chưa được chạy trên tài khoản thật khi viết bài.

```bash
export AWS_PROFILE=toinoi-lab-admin
export AWS_REGION=ap-southeast-1

aws cloudformation validate-template --template-body file://vpc.yaml

# Tạo change set nhưng CHƯA áp dụng; đọc kỹ rồi mới execute.
aws cloudformation deploy \
  --template-file vpc.yaml \
  --stack-name toinoi-dev-network \
  --parameter-overrides Env=dev EnableNat=false Owner=platform \
  --tags Project=ToiNoi Env=dev Service=network Owner=platform \
  --no-execute-changeset

# Xem change set vừa tạo (tên có tiền tố awscli-cloudformation-package-deploy-).
aws cloudformation list-change-sets --stack-name toinoi-dev-network

# Áp dụng thật.
aws cloudformation deploy \
  --template-file vpc.yaml \
  --stack-name toinoi-dev-network \
  --parameter-overrides Env=dev EnableNat=false Owner=platform \
  --tags Project=ToiNoi Env=dev Service=network Owner=platform

# Gắn stack policy bảo vệ bucket.
aws cloudformation set-stack-policy \
  --stack-name toinoi-dev-network \
  --stack-policy-body file://stack-policy.json

# Phát hiện drift.
aws cloudformation detect-stack-drift --stack-name toinoi-dev-network
```

`detect-stack-drift` trả về `StackDriftDetectionId`; dùng `describe-stack-drift-detection-status --stack-drift-detection-id <id>` rồi `describe-stack-resource-drifts --stack-name toinoi-dev-network --stack-resource-drift-status-filters MODIFIED DELETED` để xem lệch ở đâu.

## Dọn tài nguyên

Bucket có `DeletionPolicy: Retain`, nên xoá stack **không** xoá bucket. Làm theo thứ tự:

```bash
aws cloudformation delete-stack --stack-name toinoi-dev-network
aws cloudformation wait stack-delete-complete --stack-name toinoi-dev-network

# Bucket còn lại do policy Retain. Nếu là bucket lab rỗng, xoá thủ công (bucket có versioning cần xoá cả version).
aws s3 ls | grep toinoi-dev-delivery-photos
```

Stack policy không chặn `delete-stack`; muốn chặn xoá cả stack, bật termination protection bằng `aws cloudformation update-termination-protection --enable-termination-protection --stack-name toinoi-dev-network` (và tắt đi trước khi dọn).

# Lab p02-11-ecr — ECR cho service đặt chuyến

Đi kèm [bài 11 · ECR: đẩy image, quét lỗ hổng, tag bất biến và lifecycle policy](../../../api/seed/content/aws-ecr-image-scan-lifecycle.md).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Repository `toinoi/booking` | Không tính theo repository; tính theo dung lượng image lưu (GB-tháng). |
| Image đã push | Dung lượng nén lưu trong ECR. Lifecycle policy giúp giữ con số này nhỏ. |
| Pull-through cache rule | Bản thân rule không tính tiền; image được cache vào registry của bạn tính như image thường. |
| Quét basic | Không tính riêng. Lab bật `scan_on_push` ở repository cho dễ thấy; AWS khuyên đặt cấu hình quét ở cấp registry (`aws_ecr_registry_scanning_configuration`), và khi registry chuyển sang enhanced thì cờ ở repository bị bỏ qua. |
| Quét enhanced (Amazon Inspector) | **Không bật trong lab này.** Bật ở cấp registry sẽ kích hoạt Inspector và tính tiền theo image/lần quét; xem trang giá Inspector. |
| Data transfer | Pull cùng region không tính phí transfer ECR, nhưng đi qua NAT gateway thì NAT tính phí xử lý dữ liệu. |

Không ghi giá cụ thể vì giá đổi theo thời gian: dùng AWS Pricing Calculator.

## Điều kiện

- Terraform >= 1.10, AWS CLI v2 với profile SSO, Docker.
- Tài khoản lab. Không dùng account production.

## Chạy

```bash
cd examples/toi-noi-aws/p02-11-ecr
terraform init
terraform plan
terraform apply
```

Đẩy một image (mặc định build từ `examples/toi-noi-dotnet/31-38-trip-service`):

```bash
cd scripts
./push-image.sh 1.0.0
./check-scan.sh 1.0.0
```

Thử tính bất biến của tag: chạy lại `./push-image.sh 1.0.0` sau khi đổi code, bước `docker push` phải báo lỗi tag đã tồn tại (`ImageTagAlreadyExistsException`).

Xem trước lifecycle policy mà không xoá gì:

```bash
aws ecr start-lifecycle-policy-preview --repository-name toinoi/booking
aws ecr get-lifecycle-policy-preview --repository-name toinoi/booking
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
node -e 'for (const f of ["lifecycle-policy.json","repository-policy.json"]) JSON.parse(require("fs").readFileSync(f))'
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp, chưa kiểm quyền, quota hay hành vi trên tài khoản thật. Lifecycle policy và repository policy chưa được chạy trên tài khoản thật trong lúc soạn bài.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p02-11-ecr/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy
```

`force_delete = true` cho phép xoá repository dù còn image. Kiểm tra lại:

```bash
aws ecr describe-repositories --query 'repositories[].repositoryName'
aws ecr describe-pull-through-cache-rules
```

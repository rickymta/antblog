# Lab p04-36-s3-advanced: lifecycle, replication, Object Lock, event notification và S3 Tables

Lab của **bài 36** ([S3 nâng cao: storage class, replication, Object Lock, event notification và S3 Tables](../../../api/seed/content/aws-s3-nang-cao-storage-class-replication-object-lock.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 04**.

Terraform dựng hai nhóm tài nguyên: (1) bucket ảnh chứng từ `toinoi-lab-delivery-photos-<account-id>` (bài 06) với lifecycle theo prefix và kích thước, Intelligent-Tiering, event notification sang EventBridge rồi SQS; (2) bucket hoá đơn `toinoi-lab-invoices-<account-id>` có Object Lock, nhân bản sang bucket ở `ap-northeast-1`. Thêm một S3 table bucket và namespace cho Iceberg. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Bucket (rỗng) | Không tốn tiền khi chưa có object |
| Object, mọi version và bản sao ở region thứ hai | GB-tháng của từng storage class, ở **cả hai** region |
| Lifecycle transition, request PUT/GET | Số request; mỗi lần chuyển lớp là một request |
| Intelligent-Tiering | Phí giám sát và tự động hoá theo số object (không có phí lấy dữ liệu) |
| Replication | Request replication cộng **truyền dữ liệu giữa region** theo GB; RTC (`enable_rtc = true`) và replication metrics tính phí riêng |
| Object Lock | Không phí riêng, nhưng object bị khoá vẫn tính tiền lưu cho tới hết hạn |
| EventBridge rule, SQS queue | Theo số event và số request; với vài object test gần như bằng không |
| S3 table bucket (rỗng) | Chưa có bảng thì chưa có dung lượng; bảng Iceberg tính dung lượng, request và compaction |
| IAM role | Không tính tiền |

Giá cụ thể đổi theo thời gian và region: xem [S3 pricing](https://aws.amazon.com/s3/pricing/) và Pricing Calculator.

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Quyền tạo bucket, IAM role, EventBridge rule, SQS và S3 Tables ở cả `ap-southeast-1` và `ap-northeast-1`.
- Region có hỗ trợ S3 Tables: kiểm tra danh sách region trên trang tài liệu S3 Tables trước khi `apply`; nếu chưa có, đặt `-var create_table_bucket=false`.

## Chạy

```bash
cd examples/toi-noi-aws/p04-36-s3-advanced
terraform init
terraform plan
terraform apply
```

Biến chính: `object_lock_mode` (mặc định `GOVERNANCE`), `object_lock_days` (mặc định 1 để dọn được), `enable_rtc` (mặc định `false`), `create_table_bucket`. Để đổi sang hoá đơn thật phải giữ 10 năm, thay khối `default_retention` bằng `years = 10` và cân nhắc `COMPLIANCE`: **không thử COMPLIANCE ở tài khoản mà bạn muốn xoá bucket sớm**.

Sau khi `apply`:

```bash
scripts/object-lock-demo.sh                 # ghi object, thử xoá bị từ chối, legal hold, bypass governance
bash scripts/s3-tables-and-batch.sh run-tables   # tạo bảng Iceberg trips.trip_events
bash scripts/s3-tables-and-batch.sh access-point # in lệnh tạo access point VPC-only (chỉ in)
```

Thử event notification: `aws s3 cp ./anh.jpg s3://toinoi-lab-delivery-photos-<account-id>/proof/100123/delivered.jpg`, rồi `aws sqs receive-message --queue-url "$(terraform output -raw proof_queue_url)"` phải thấy event `Object Created`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
for f in scripts/*.sh; do bash -n "$f" && echo "ok $f"; done
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. Xoá object thử bằng script ở trên (script đã bypass governance), hoặc đợi hết hạn retention 1 ngày.
2. Bucket hoá đơn và bản sao có versioning và Object Lock: phải xoá **mọi version và delete marker** (kể cả ở bucket bản sao) trước khi `terraform destroy` xoá được bucket. Version còn nằm trong thời hạn retention sẽ chặn việc xoá; với GOVERNANCE dùng `--bypass-governance-retention` nếu có quyền.
3. `terraform destroy`.
4. Kiểm lại: `aws s3api head-bucket --bucket <tên>` phải báo 404 ở cả hai region.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider). Chưa có lần `apply` nào trên tài khoản AWS thật. Cần bạn xác nhận khi chạy: quyền tối thiểu của role replication khi bucket bật Object Lock, thứ tự tạo tài nguyên replication và versioning, hỗ trợ S3 Tables ở region của bạn, và định dạng event pattern của rule EventBridge với event `Object Created`.

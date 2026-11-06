# Lab p06-59-finops: budget, anomaly monitor và review chi phí hằng tháng

Lab của **bài 59** ([FinOps trên AWS: cost allocation, Savings Plans, Reserved, Spot, right-sizing và kiến trúc tiết kiệm](../../../api/seed/content/aws-finops-savings-plans-spot-right-sizing.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**.

Terraform dựng: budget tháng theo tag `Project=ToiNoi` (cảnh báo 80% thực tế và 100% dự báo), hai Cost Anomaly Detection monitor (theo dịch vụ và theo tag) với subscription email hằng ngày, một Cost Category chia dịch vụ thành lớp `compute`, `data`, `edge`, và hai budget Savings Plans (coverage, utilization) **tắt mặc định**. Kèm `queries/cost-per-trip.sql` (Athena trên CUR 2.0) và `scripts/finops-review.sh` (chỉ lệnh đọc). **Không có lệnh nào chạy khi bạn chỉ clone repo, và lab không mua Savings Plans hay Reserved.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| AWS Budgets (budget tháng, budget Savings Plans) | Budget loại chi phí và sử dụng có thể miễn phí ở mức cơ bản; xem trang giá Budgets cho giới hạn số lượng và budget action |
| Cost Anomaly Detection (monitor, subscription) | Không tính tiền riêng |
| Cost Category | Không tính tiền riêng |
| Lệnh Cost Explorer API trong `finops-review.sh` | Mỗi lần gọi API có thể tính phí nhỏ; script chỉ gọi một số ít lần |
| Data Exports, S3, Athena (nếu bạn tự bật cho SQL) | Lưu trữ S3 và dữ liệu Athena quét; ngoài phạm vi Terraform của lab |

Giá cụ thể đổi theo thời gian: xem [AWS Budgets](https://aws.amazon.com/aws-cost-management/aws-budgets/pricing/), [Cost Explorer](https://aws.amazon.com/aws-cost-management/aws-cost-explorer/pricing/) và Pricing Calculator.

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02), chạy ở tài khoản quản lý hoặc tài khoản đơn lẻ có quyền Billing.
- Tag `Project`, `Env`, `Service`, `Owner` đã **kích hoạt làm cost allocation tag** trong Billing (bài 10); tag chỉ có hiệu lực từ lúc kích hoạt.
- Đổi `alert_emails` sang hộp thư của bạn, chọn `monthly_limit` và `anomaly_threshold` phù hợp với tài khoản lab.

## Chạy

```bash
cd examples/toi-noi-aws/p06-59-finops
terraform init
terraform plan -var 'alert_emails=["ban@example.com"]' -var monthly_limit=50
terraform apply -var 'alert_emails=["ban@example.com"]' -var monthly_limit=50
```

Email xác nhận đăng ký có thể cần bấm. Chuẩn bị số liệu cho buổi review:

```bash
AWS_PROFILE=lab scripts/finops-review.sh
```

Sau khi đã mua Savings Plans (ngoài lab), bật các budget cam kết bằng `-var enable_commitment_budgets=true`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/finops-review.sh
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. `terraform destroy` (xoá budget, monitor, subscription và Cost Category).
2. Kiểm lại trong console Billing and Cost Management: không còn budget `toinoi-lab-*` và monitor `toinoi-lab-*`.
3. Nếu bạn tự bật Data Exports hoặc Athena để chạy SQL, xoá export, bucket và workgroup đó riêng.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider), script đã qua `bash -n`; SQL chưa chạy. **Chưa `apply` trên tài khoản AWS thật.** Budget lọc tag bằng `filter_expression` (kèm `metrics`), cùng dạng `FilterExpression` của bài 10, thay cho `cost_filter` kiểu cũ; SQL dùng `element_at(resource_tags, ...)` vì `resource_tags[...]` báo lỗi khi khoá vắng. Cần bạn xác nhận: khoá tag `Project` đã kích hoạt cost allocation, tên dịch vụ hiển thị trong Cost Category ở tài khoản của bạn, tên bảng, cột và khoá `resource_tags` (dạng `user_project`) của CUR 2.0 trước khi chạy SQL, và hành vi của hai budget Savings Plans (cần có Savings Plans đang hoạt động).

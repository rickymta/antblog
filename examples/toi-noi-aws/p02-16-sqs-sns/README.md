# Lab bài 16 — SQS và SNS: fan-out BookingCreated

Đi kèm [bài 16 · SQS và SNS: hàng đợi, dead-letter queue và fan-out thông báo khi có chuyến mới](../../../api/seed/content/aws-sqs-sns-dlq-fan-out.md) của lộ trình **AWS qua Tới Nơi · Phần 02**.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Số lượng | Tính tiền |
|---|---:|---|
| SNS topic standard `toinoi-lab-booking-events` | 1 | Theo số lần publish và số delivery; lab chỉ vài message |
| SQS queue chính `toinoi-lab-dispatch-push`, `toinoi-lab-stats` | 2 | Theo số request (mỗi lần send, receive, delete, kể cả receive rỗng) |
| SQS DLQ `...-dlq` tương ứng | 2 | Như trên |
| SNS subscription SQS (có filter policy) | 2 | Delivery tới SQS cùng region tính theo bảng giá SNS hiện hành |
| SNS subscription email | 0 hoặc 1 | Chỉ tạo khi đặt `alert_email`; cần xác nhận bằng link trong email |
| FIFO queue `toinoi-lab-trip-status.fifo` (do `fifo-demo.sh` tạo rồi xoá) | 1 tạm thời | Theo request, giá FIFO khác standard |

Không có tài nguyên nào tính tiền theo giờ chờ. Chi phí của lab đến từ số request; hãy xem trang Pricing của SQS và SNS rồi tính bằng AWS Pricing Calculator thay vì tin con số cố định trong tài liệu. Cấu hình đã bật `receive_wait_time_seconds = 20` để giảm số lần receive rỗng.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
for f in *.sh; do bash -n "$f"; done
node -e 'JSON.parse(require("fs").readFileSync("sample-booking-created.json"))'
```

`validate` chỉ chứng minh cú pháp và kiểu. Chưa kiểm chứng trên tài khoản thật: access policy của queue, hành vi filter policy, redrive. Xoá `.terraform` và `.terraform.lock.hcl` sau khi kiểm.

## Chạy trong tài khoản lab của bạn

```bash
export AWS_PROFILE=toinoi-lab-admin
terraform init
terraform apply                      # thêm -var alert_email=ban@example.com nếu muốn thử email
./publish-and-receive.sh             # publish 3 chuyến, nhận từ hai queue
./fifo-demo.sh                       # FIFO và deduplication id
./redrive.sh                         # sau khi có message trong DLQ
```

Kết quả mong đợi: queue `stats` nhận cả 3 message; queue `dispatch-push` chỉ nhận chuyến HCM/ride và HN/food vì filter policy loại chuyến DN/parcel; FIFO queue nhận 1 message dù gửi 2 lần cùng deduplication id. Một lần `receive-message` của standard queue có thể trả ít hơn số message đang có (SQS chỉ lấy mẫu một phần máy chủ); nếu thiếu, chạy lại lệnh receive.

Để thấy DLQ hoạt động, nhận message từ `dispatch-push` đủ `max_receive_count` lần (mặc định 5) mà không xoá, chờ hết visibility timeout giữa các lần, rồi kiểm queue `...-dispatch-push-dlq`.

## Dọn tài nguyên

```bash
terraform destroy
```

`fifo-demo.sh` tự xoá FIFO queue ở cuối; nếu script dừng giữa chừng hãy chạy `aws sqs list-queues --queue-name-prefix toinoi-lab-` và xoá queue còn sót bằng `aws sqs delete-queue`. Không commit tệp `terraform.tfstate`.

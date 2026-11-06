# Lab bài 18 — Lambda: webhook thanh toán VíXanh, SQS và S3

Đi kèm bài [Lambda: hàm xử lý webhook thanh toán, trigger từ S3 và SQS, cold start và quyền](../../../api/seed/content/aws-lambda-webhook-thanh-toan-cold-start.md) của lộ trình **AWS qua Tới Nơi · Phần 02**.

## Nội dung thư mục

| Tệp | Vai trò |
|---|---|
| `src/webhook/index.mjs`, `verify.mjs` | Hàm nhận webhook: xác minh chữ ký HMAC-SHA256, ghi SQS, trả 200 |
| `src/consumer/index.mjs` | Hàm đọc SQS theo batch, trả `batchItemFailures` |
| `src/photo/index.mjs` | Hàm kích hoạt bởi S3 khi có ảnh chứng từ |
| `test/verify.test.mjs` | Test chữ ký, chạy bằng `node --test` (không cần AWS) |
| `sign.mjs`, `sample-webhook.json` | Tạo header chữ ký để thử bằng curl |
| `trust-policy.json`, `webhook-permissions.json`, `consumer-permissions.json` | Trust policy và quyền tối thiểu cho hai role |
| `s3-notification.json` | Ví dụ cấu hình S3 notification tới hàm ảnh (không được `deploy.sh` áp dụng) |
| `deploy.sh`, `cleanup.sh` | Tạo và dọn tài nguyên bằng AWS CLI |

Hàm dùng AWS SDK for JavaScript v3 đã có sẵn trong runtime Node.js của Lambda, nên không cần `npm install` hay thư mục `node_modules` trong gói zip.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Số lượng | Tính tiền |
|---|---:|---|
| Lambda `toinoi-lab-vixanh-webhook`, `toinoi-lab-payment-consumer` | 2 | Theo số lần gọi và thời gian chạy theo GB-giây; có free tier, xem Pricing |
| Function URL | 1 | Không tính riêng; tính theo lần gọi hàm |
| SQS `toinoi-lab-payment-events` và DLQ | 2 | Theo số request |
| Secret `toinoi/lab/vixanh-webhook-secret` | 1 | Theo secret mỗi tháng và theo số lần gọi API |
| IAM role | 2 | Không tính phí |
| CloudWatch Logs | 2 log group | Theo dung lượng log nhận và lưu |

Chưa có provisioned concurrency hay VPC nên không có tài nguyên tính theo giờ chờ, ngoại trừ secret (tính theo tháng, bị xoá ngay khi `cleanup.sh` chạy). Tính chi phí bằng AWS Pricing Calculator.

## Kiểm trên máy

```bash
node --test test/
bash -n deploy.sh && bash -n cleanup.sh
for f in *.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f"; done
```

Các lệnh trên không chạm tới AWS. Chưa kiểm chứng trên tài khoản thật: `deploy.sh` từ đầu đến cuối, quyền `lambda:InvokeFunctionUrl` kèm `lambda:InvokeFunction` (cờ `--invoked-via-function-url`), hành vi event source mapping, tên runtime `nodejs22.x` có sẵn ở region của bạn.

## Chạy trong tài khoản lab của bạn

```bash
export AWS_PROFILE=toinoi-lab-admin
./deploy.sh
```

Cuối `deploy.sh` in ra Function URL và lệnh curl mẫu. Thử hai trường hợp: chữ ký đúng (kỳ vọng `200` và log của consumer) và chữ ký sai (kỳ vọng `401`). Để thấy DLQ, sửa `applyPaymentEvent` ném lỗi, deploy lại consumer, gửi webhook và theo dõi message sang queue `...-payment-events-dlq` sau 5 lần nhận.

Để thử trigger S3 (tuỳ chọn, tự làm): tạo hàm từ `src/photo`, cấp `aws lambda add-permission` cho `s3.amazonaws.com` với `--source-arn` của bucket và `--source-account`, rồi áp `s3-notification.json` (đổi ARN hàm cho khớp) bằng `aws s3api put-bucket-notification-configuration`.

## Dọn tài nguyên

```bash
./cleanup.sh
```

Script xoá event source mapping, Function URL, hàm, role, queue, secret (xoá ngay, không chờ khôi phục) và log group. Chạy lại được nếu bị gián đoạn. Nếu bạn tạo thêm hàm ảnh hoặc notification của S3, hãy xoá thủ công.

# Lab bài 17 — EventBridge: bus, rule, schedule, archive và replay

Đi kèm bài [EventBridge: event bus, rule, schedule và sự kiện chuyến của Tới Nơi](../../../api/seed/content/aws-eventbridge-event-bus-rule-schedule.md) của lộ trình **AWS qua Tới Nơi · Phần 02**.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Số lượng | Tính tiền |
|---|---:|---|
| Custom event bus `toinoi-lab-trips` | 1 | Theo số sự kiện custom đưa lên bus |
| Rule `trip-completed-hcm`, `trip-cancelled-by-driver` | 2 | Rule không tính phí riêng; tính theo sự kiện và delivery tới target |
| SQS queue `settlement`, `reconcile`, `trips-rule-dlq` | 3 | Theo số request SQS |
| Archive `toinoi-lab-trips-archive` | 1 | Theo dung lượng lưu và lượng sự kiện archive; replay tính theo sự kiện phát lại |
| Schema discoverer | 1 | Theo số sự kiện được khám phá, có phần miễn phí; xem Pricing |
| Schedule group, schedule `reconcile-0200` | 1 + 1 | Theo số lần gọi của Scheduler |
| IAM role cho Scheduler | 1 | Không tính phí |

Không có tài nguyên tính theo giờ chờ, nhưng archive và discoverer vẫn tiêu tiền theo lượng sự kiện. Lab chỉ gửi vài sự kiện. Tính chi phí thật bằng AWS Pricing Calculator, không dựa vào con số cố định.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n run-lab.sh
for f in *.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f"; done
```

`validate` chỉ chứng minh cú pháp và kiểu. Chưa kiểm chứng trên tài khoản thật: queue policy cho `events.amazonaws.com`, retry/DLQ của target, input transformer, replay. Xoá `.terraform` và `.terraform.lock.hcl` sau khi kiểm.

## Chạy trong tài khoản lab của bạn

```bash
export AWS_PROFILE=toinoi-lab-admin
terraform init
terraform apply
./run-lab.sh
```

`run-lab.sh` gồm bốn bước: `aws events test-event-pattern` cho sự kiện mẫu, `aws events put-events` với ba sự kiện, đọc queue `settlement` (kỳ vọng nhận TripCompleted HCM 52.500 và TripCancelled do tài xế; không nhận chuyến HN), rồi `aws events start-replay` một giờ gần nhất. Archive cần vài phút mới có dữ liệu; replay quá sớm sẽ không có gì để phát.

Để xem schedule 02:00: `aws scheduler get-schedule --name toinoi-lab-reconcile-0200 --group-name toinoi-lab`.

## Dọn tài nguyên

```bash
terraform destroy
```

Replay đang chạy phải kết thúc hoặc bị huỷ trước khi xoá archive. Không commit `terraform.tfstate`.

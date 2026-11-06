# Lab p06-57-step-functions: Saga đặt chuyến và đối soát đêm bằng Step Functions

Lab của **bài 57** ([Step Functions: Saga đặt chuyến, compensation, distributed map và các pattern serverless nâng cao](../../../api/seed/content/aws-step-functions-saga-dat-chuyen.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**.

Hai định nghĩa ASL viết bằng **JSONata** và biến (`Assign`):

- `asl/booking-saga.asl.json`: giữ tiền → tìm tài xế (callback `waitForTaskToken` qua SQS, có `HeartbeatSeconds`) → xác nhận chuyến (DynamoDB `updateItem` có điều kiện) → thu tiền → phát `TripConfirmed`. Mỗi lỗi chạy chuỗi bù trừ đúng thứ tự; bù trừ thất bại thì gửi vào queue xử lý thủ công.
- `asl/reconcile-nightly.asl.json`: distributed map đọc CSV chuyến trong ngày từ S3, chia lô 500 dòng, 100 lô song song, chấp nhận lỗi tới 1%, ghi kết quả về S3.

Terraform dựng role, log group, hai state machine và alias `live` cho saga. **Không có lệnh nào chạy khi bạn chỉ clone repo.** Lab không tạo Lambda, queue, bảng DynamoDB hay bucket: chúng là biến, lấy từ lab bài 16, 17, 18, 32 hoặc dùng stub của bạn.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| State machine Standard (saga, job cha của đối soát) | Số **state transition** của mỗi execution |
| Child execution Express của distributed map | Số lượt chạy và thời lượng, bộ nhớ (theo trang giá Express) |
| Log group, X-Ray | Dung lượng log nhận vào và lưu; số trace |
| Role, alias, phiên bản đã publish | Không tính tiền riêng |
| Các dịch vụ saga gọi (Lambda, SQS, DynamoDB, EventBridge) | Theo từng dịch vụ như bài 16, 17, 18, 32 |

Job đối soát với 2 triệu dòng, lô 500, sinh ra khoảng 4.000 child execution mỗi đêm; hãy ước tính bằng [Pricing Calculator](https://calculator.aws/) trước khi chạy ở quy mô đó. Giá cụ thể xem [AWS Step Functions pricing](https://aws.amazon.com/step-functions/pricing/).

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Lambda giữ tiền, thu tiền, hoàn giữ tiền và đối soát lô (có thể là stub trả JSON `{"holdId":"h-1"}`), queue `toinoi-lab-dispatch` và `toinoi-lab-saga-manual-review`, bảng `toinoi-lab-dispatch`, event bus `toinoi-lab-trips`, bucket đối soát có `manifests/<ngày>/trips.csv`. Truyền ARN/URL qua biến của Terraform.

## Chạy

```bash
cd examples/toi-noi-aws/p06-57-step-functions
terraform init
terraform plan -var recon_bucket_name=<bucket của bạn>
terraform apply -var recon_bucket_name=<bucket của bạn>
```

Chạy một saga (dùng alias để luôn đi qua phiên bản đã chọn):

```bash
aws stepfunctions start-execution \
  --state-machine-arn "$(terraform output -raw saga_alias_arn)" \
  --name "booking-b-20261106-0001" \
  --input file://input/start-booking.json
```

`--name` cố định theo `bookingId` cho một chuyến chỉ có một execution trong 90 ngày (tên execution Standard là duy nhất theo state machine). Khi saga dừng ở `FindDriver`, mô phỏng tài xế nhận bằng task token lấy từ message trong queue `toinoi-lab-dispatch`:

```bash
aws stepfunctions send-task-success --task-token "$TOKEN" --task-output '{"driverId":"d001"}'
aws stepfunctions send-task-failure --task-token "$TOKEN" --error NoDriverAvailable --cause "Hết tài xế trong 3 phút"
```

Execution Standard thất bại có thể chạy lại từ bước lỗi (trong 14 ngày, cùng execution ARN):

```bash
aws stepfunctions redrive-execution --execution-arn "$EXECUTION_ARN"
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
node scripts/check-asl.mjs asl/booking-saga.asl.json asl/reconcile-nightly.asl.json
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/validate-with-cli.sh
```

`check-asl.mjs` parse JSON, kiểm `StartAt`, mọi `Next` và `Catch`, state mồ côi, dấu `{% %}`. Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

Có credential thì kiểm bằng chính API của AWS (không tạo tài nguyên): `scripts/validate-with-cli.sh` gọi `aws stepfunctions validate-state-machine-definition` cho từng tệp.

## Dọn tài nguyên

1. Dừng execution còn chạy: `aws stepfunctions list-executions --state-machine-arn <arn> --status-filter RUNNING`, rồi `aws stepfunctions stop-execution --execution-arn <arn>`.
2. `terraform destroy` (xoá cả alias và các phiên bản đã publish).
3. Xoá các Lambda, queue, bảng, bucket stub mà bạn tự tạo ngoài lab.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate`, JSON parse được và qua `check-asl.mjs`, `templatefile` render ra JSON hợp lệ. Hai tệp ASL (sau khi thay placeholder như `validate-with-cli.sh`) qua `asl-validator` 4.0; tên trường JSONata của `ItemReader` (`Resource`, `ReaderConfig`, `Arguments`), `ItemBatcher.BatchInput`, `ResultWriter.Arguments` đối chiếu khớp tài liệu distributed map; `Catch` đọc `$states.errorOutput` và `Detail` dạng object của `events:putEvents` đúng mẫu tài liệu. **Chưa có execution nào chạy và chưa gọi `validate-state-machine-definition`** (API cần credential tài khoản thật). Cần bạn xác nhận khi chạy: quyền IAM thật khi distributed map khởi chạy child execution Express.

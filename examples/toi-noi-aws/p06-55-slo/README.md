# Lab p06-55-slo: log có cấu trúc, alarm theo burn rate, composite alarm và SLO

Lab của **bài 55** ([Quan sát nâng cao: X-Ray và ADOT, Logs Insights, anomaly detection, Application Signals và SLO](../../../api/seed/content/aws-observability-nang-cao-xray-adot-logs-insights-slo.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**. Cần đã xong [bài 10](../../../api/seed/content/aws-cloudwatch-budgets-tagging-co-ban.md) (CloudWatch cơ bản) và nên đọc [bài 37 loạt .NET](../../../api/seed/content/dotnet10-opentelemetry-traces-metrics-logs.md) (OpenTelemetry trong service).

Terraform dựng log group của service `booking` có data protection policy (che email và số điện thoại), hai metric filter, hai query Logs Insights đã lưu, bốn alarm (burn rate nhanh, latency p99, anomaly detection lưu lượng, log lỗi) gom vào một composite alarm gọi on-call qua SNS, luật lấy mẫu và group của X-Ray, và tuỳ chọn Synthetics canary cùng RUM. Hai tệp JSON trong `slo/` định nghĩa SLO của Application Signals, tạo bằng AWS CLI vì Terraform provider chưa có resource riêng cho SLO ở bản dùng khi soạn lab. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Log group, metric filter, query đã lưu | Log nạp và lưu tính theo GB; metric filter tạo metric tuỳ chỉnh tính theo số metric |
| Data protection policy | Tính theo GB log được quét; xem trang giá CloudWatch |
| Alarm (metric, anomaly detection, composite) | Theo số alarm; alarm anomaly detection và metric math tính cao hơn alarm thường |
| SNS | Theo số thông báo, không đáng kể ở lab |
| X-Ray sampling rule và group | Rule và group không tính riêng; trace tính theo lượng nhận và truy xuất |
| Application Signals (SLO) | Theo lượng span và metric được xử lý và số SLO; xem trang giá CloudWatch |
| Synthetics canary (`enable_canary`, mặc định tắt) | Theo số lần canary chạy, cộng S3 và log của canary |
| RUM (`enable_rum`, mặc định tắt) | Theo số event ghi nhận |

Giá cụ thể đổi theo thời gian và region: xem [CloudWatch pricing](https://aws.amazon.com/cloudwatch/pricing/) và Pricing Calculator. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Owner=platform`.

## Tệp trong lab

| Tệp | Công dụng |
| --- | --- |
| `logs.tf` | Log group, data protection, metric filter, query Logs Insights đã lưu |
| `alarms.tf` | SNS, alarm burn rate, p99, anomaly detection, log lỗi và composite alarm |
| `tracing.tf` | Sampling rule (thanh toán 100%, đặt chuyến 5%) và X-Ray group lọc trace chậm ở HCM |
| `synthetics_rum.tf` | Canary báo giá mỗi 5 phút và RUM, đều tuỳ chọn |
| `canary/nodejs/node_modules/booking-canary.js` | Mã canary: `POST /v1/quotes` và kiểm `price > 0` |
| `slo/*.json` | SLO availability (request-based) và latency p99 (period-based) cho Application Signals |
| `create-slos.sh` | In (hoặc chạy với `--apply`) lệnh tạo SLO |
| `slo-math.mjs` | Tính error budget và burn rate bằng Node |
| `queries/*.logsinsights` | Query Logs Insights |
| `samples/*.json` | Mẫu log JSON và Embedded Metric Format |

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), providers `hashicorp/aws ~> 6.0` và `hashicorp/archive ~> 2.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02), Node.
- Biến `alb_arn_suffix` (dạng `app/<tên>/<id>`) của ALB trước service booking; giá trị mặc định chỉ để minh hoạ nên alarm ALB sẽ không có dữ liệu.
- Với SLO: service `toinoi-booking` đã được instrument bằng ADOT hoặc CloudWatch agent bật Application Signals và đã xuất hiện trong trang Services. Tên `Service` và `Environment` trong `slo/*.json` phải khớp với những gì trang đó hiển thị. Trên ECS, Application Signals không tự phát hiện tên: đặt trong task definition biến `OTEL_RESOURCE_ATTRIBUTES=service.name=toinoi-booking,deployment.environment=ecs:toinoi-lab` (không đặt `deployment.environment` thì CloudWatch agent tự lấy theo tên cluster ECS, khó đoán trước).

## Chạy

```bash
cd examples/toi-noi-aws/p06-55-slo
terraform init
terraform plan
terraform apply -var alert_email=ban@example.com -var alb_arn_suffix=app/<tên-alb>/<id>
```

Tính budget trên máy, không cần AWS:

```bash
node slo-math.mjs 99.9 30
node slo-math.mjs 99.9 30 0.0144
```

Tạo SLO (mặc định chỉ in lệnh):

```bash
./create-slos.sh toinoi-lab
./create-slos.sh toinoi-lab --apply
```

Chạy một query Logs Insights từ CLI:

```bash
NOW="$(date +%s)"
aws logs start-query --log-group-name /toinoi/lab/booking \
  --start-time "$((NOW - 3600))" --end-time "$NOW" \
  --query-string file://queries/slow-by-city.logsinsights --profile toinoi-lab
aws logs get-query-results --query-id "<id từ lệnh trên>" --profile toinoi-lab
```

Diễn tập đường báo động (đặt trạng thái alarm con thủ công; trạng thái sẽ tự trở lại ở lần đánh giá kế tiếp):

```bash
aws cloudwatch set-alarm-state --alarm-name toinoi-lab-booking-burn-fast \
  --state-value ALARM --state-reason "diễn tập on-call" --profile toinoi-lab
```

Bật canary hoặc RUM: thêm `-var enable_canary=true` (kiểm `canary_runtime_version` bằng `aws synthetics describe-runtime-versions`) hoặc `-var enable_rum=true`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in slo/*.json samples/*.json; do
  node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "$f hợp lệ"
done
node --check canary/nodejs/node_modules/booking-canary.js
bash -n create-slos.sh
node slo-math.mjs 99.9 30
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Sau khi kiểm, xoá `.terraform`, `.terraform.lock.hcl` và thư mục `.build` (do `archive_file` sinh khi `plan`); không commit.

## Dọn tài nguyên

1. Xoá SLO nếu đã tạo: `aws application-signals delete-service-level-objective --id <tên-hoặc-arn> --profile toinoi-lab` cho từng SLO.
2. `terraform destroy`.
3. Kiểm lại: `aws cloudwatch describe-alarms --alarm-name-prefix toinoi-lab-`, `aws logs describe-log-groups --log-group-name-prefix /toinoi/lab/`, `aws synthetics describe-canaries` không còn tài nguyên của lab.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (cú pháp và schema provider); JSON đã parse được; mã canary đã qua `node --check`; `slo-math.mjs` đã chạy; `create-slos.sh` đã qua `bash -n` và chạy ở chế độ in lệnh. Hai tệp `slo/*.json` đã qua kiểm tra tham số với mô hình `CreateServiceLevelObjective` của botocore 1.42; provider Terraform 6.72 chưa có resource cho SLO của Application Signals nên SLO tạo bằng CLI. Canary dùng `syn-nodejs-puppeteer-18.0` (bản mới nhất tại thời điểm viết) với namespace `@aws/synthetics-puppeteer`, `@aws/synthetics-logger` (bắt buộc từ 13.1) và đặt tuỳ chọn ghi header/body ở cấu hình chung (từ 11.0 không đặt theo step). Data protection policy dùng đúng mẫu `CustomDataIdentifier` (Name, Regex) trong tài liệu CloudWatch Logs. `BurnRateConfigurations` chỉ sinh metric burn rate, không tự tạo alarm. **Chưa có lần `apply` hay lệnh AWS thật nào.** Cần bạn xác nhận khi chạy: SLO tạo thành công với `Environment` khớp trang Services, và độ lệch của xấp xỉ burn rate bằng metric math 12 chu kỳ 5 phút so với cửa sổ trượt chuẩn.

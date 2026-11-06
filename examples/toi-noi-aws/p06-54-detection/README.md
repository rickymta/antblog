# Lab p06-54-detection: CloudTrail, Config, GuardDuty, Security Hub, Inspector và Macie

Lab của **bài 54** ([CloudTrail, Config, Security Hub, GuardDuty và Inspector: audit và phát hiện cho dữ liệu khách](../../../api/seed/content/aws-cloudtrail-config-security-hub-guardduty.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**. Cần đã xong [bài 51](../../../api/seed/content/aws-iam-nang-cao-permission-boundary-abac-access-analyzer.md) và [bài 52](../../../api/seed/content/aws-kms-envelope-encryption-key-policy.md).

Terraform dựng lớp phát hiện cho **một tài khoản lab**: trail đa region có log file validation và data event S3, bốn cảnh báo từ CloudTrail (root, đổi IAM policy, xoá VPC Flow Logs, sửa trail), AWS Config với chín rule managed và một remediation chờ duyệt, GuardDuty kèm các protection plan, Security Hub (phiên bản mới và CSPM với chuẩn AWS Foundational Security Best Practices), Inspector, và tuỳ chọn Macie cùng CloudTrail Lake (chỉ cho tài khoản đã dùng Lake trước 31/05/2026). Ở hệ thống thật các dịch vụ này đặt ở tài khoản security và log archive của tổ chức (bài 28). **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Trail, bucket log, log group | Trail đầu tiên ghi management event miễn phí cho bản sao đầu; data event tính theo số event; lưu S3 và CloudWatch Logs tính theo GB |
| CloudTrail Lake (`enable_cloudtrail_lake`, mặc định tắt; đóng với khách mới từ 31/05/2026) | Theo dung lượng nạp và thời gian lưu |
| AWS Config | Theo configuration item ghi và theo số lần đánh giá rule |
| GuardDuty | Theo lượng log và sự kiện phân tích; mỗi protection plan có cách tính riêng. Runtime Monitoring và EBS malware scan không bật mặc định |
| Security Hub (mới) và CSPM | Theo số finding nạp và số lần kiểm control; xem trang giá |
| Inspector | Theo số tài nguyên được quét (instance, image, function) |
| Macie (`enable_macie`, mặc định tắt) | Theo bucket đánh giá và GB quét trong job |
| SNS, EventBridge rule, metric filter, alarm | Gần như không đáng kể ở quy mô lab |

Giá cụ thể đổi theo thời gian và region: xem trang giá của từng dịch vụ và Pricing Calculator. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Owner=security`.

## Tệp trong lab

| Tệp | Công dụng |
| --- | --- |
| `logging.tf` | Bucket log, CloudTrail, CloudWatch Logs, event data store tuỳ chọn |
| `alarms.tf` | SNS, metric filter và alarm từ CloudTrail |
| `config.tf` | Config recorder, rule managed, remediation SSM Automation (chờ duyệt) |
| `detection.tf` | GuardDuty, Security Hub, Inspector, Macie |
| `queries/*.logsinsights` | Truy vấn CloudWatch Logs Insights trên log group của trail: ai đọc secret, ai xoá Flow Logs |
| `queries/*.sql` | Cùng hai câu hỏi cho CloudTrail Lake, chỉ dùng được nếu tài khoản đã có Lake trước 31/05/2026 |
| `who-disabled-flow-logs.sh` | Tìm ai xoá VPC Flow Logs bằng event history (chỉ đọc) |
| `access-key-triage.sh` | Playbook access key lộ: khoanh vùng và (tuỳ chọn) vô hiệu hoá key |

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0` (cần bản có `aws_securityhub_account_v2`), AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02), `jq`.
- Tài khoản **chưa** có Config recorder hay trail trùng tên (mỗi region chỉ có một Config recorder), và chưa bật GuardDuty, Security Hub trong region này; nếu đã bật, tắt biến tương ứng thay vì tạo trùng.
- Quyền tạo các dịch vụ trên. Đã xem bảng giá ở trên.

## Chạy

```bash
cd examples/toi-noi-aws/p06-54-detection
terraform init
terraform plan
terraform apply -var alert_email=ban@example.com
```

Truy vấn "ai đọc secret, ai xoá Flow Logs" bằng CloudWatch Logs Insights trên log group của trail (trail đã đẩy sang `/toinoi/lab/cloudtrail`):

```bash
aws logs start-query --log-group-name /toinoi/lab/cloudtrail \
  --start-time "$(( $(date +%s) - 7*86400 ))" --end-time "$(date +%s)" \
  --query-string "$(cat queries/who-read-secret.logsinsights)" --profile toinoi-lab
aws logs get-query-results --query-id "<id từ lệnh trên>" --profile toinoi-lab
```

CloudTrail Lake đóng với khách mới từ 31/05/2026 (khách cũ vẫn dùng, Lake chỉ nhận bản vá lỗi nghiêm trọng; AWS khuyên chuyển sang CloudWatch). Nếu tài khoản đã có Lake từ trước: thêm `-var enable_cloudtrail_lake=true`, thay `<EVENT_DATA_STORE_ID>` trong `queries/*.sql` rồi chạy `aws cloudtrail start-query --query-statement file://queries/who-deleted-flow-logs.sql`.

Điều tra bằng event history (không cần Lake):

```bash
./who-disabled-flow-logs.sh toinoi-lab ap-southeast-1
./access-key-triage.sh <access-key-id> <iam-user> toinoi-lab
```

Trạng thái các dịch vụ:

```bash
aws cloudtrail get-trail-status --name "$(terraform output -raw trail_name)" --profile toinoi-lab
aws configservice describe-compliance-by-config-rule --profile toinoi-lab
aws guardduty list-detectors --profile toinoi-lab
aws securityhub describe-security-hub-v2 --profile toinoi-lab
```

Chạy thử một remediation: sau khi Config đánh dấu một bucket không tuân thủ, duyệt thủ công trong console (Config, Rules, Remediation) hoặc bằng `aws configservice start-remediation-execution`. Remediation để `automatic = false`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
bash -n who-disabled-flow-logs.sh access-key-triage.sh
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. `terraform destroy` (bucket log có `force_destroy = true`).
2. Kiểm lại ở region của lab: `aws configservice describe-configuration-recorders`, `aws guardduty list-detectors`, `aws securityhub describe-hub`, `aws cloudtrail describe-trails`. Dịch vụ tài khoản đã bật từ trước lab thì **không** destroy.
3. Nếu bật Macie hoặc CloudTrail Lake, kiểm console để chắc chắn job và event data store đã xoá (event data store có thể cần tắt termination protection trước khi xoá).

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (cú pháp và schema provider); script đã qua `bash -n` và chạy thử với lệnh `aws` giả lập (kiểm luồng jq, không gọi AWS thật); không có lệnh `aws` nào được chạy trên tài khoản thật. **Chưa có lần `apply` nào.** Đã đối chiếu tài liệu: runbook `AWS-DisableS3BucketPublicReadWrite` nhận `S3BucketName` và `AutomationAssumeRole`; `aws_securityhub_account_v2` (Security Hub mới) và `aws_securityhub_account` (CSPM) cùng tồn tại được trong một tài khoản; tên feature GuardDuty khớp danh sách của `aws_guardduty_detector_feature`; `requestParameters` trong CloudTrail Lake là map nên dùng `element_at`. Cần bạn xác nhận khi chạy: quyền tối thiểu cho role SSM, bucket policy log đủ cho trail và Config, và feature GuardDuty nào khả dụng ở region của bạn. Org trail, delegated admin, aggregation nhiều region và Detective không nằm trong lab.

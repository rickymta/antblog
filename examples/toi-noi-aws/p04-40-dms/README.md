# Lab p04-40-dms: DMS full load + CDC từ PostgreSQL tự vận hành sang Aurora

Lab của **bài 40** ([DMS và cắt chuyển gần như không downtime: từ PostgreSQL tự vận hành sang Aurora](../../../api/seed/content/aws-dms-migration-postgresql-zero-downtime.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 04**.

Terraform dựng các thành phần DMS: subnet group, endpoint nguồn (PostgreSQL tự vận hành) và đích (Aurora PostgreSQL), replication instance cùng replication task `full-load-and-cdc` (hoặc DMS Serverless với `-var use_serverless=true`). Cấu hình migration nằm ở các tệp JSON: `table-mappings.json` (bảng nào vào, bảng nào loại), `table-mappings-rehearsal-one-city.json` (ví dụ lọc một thành phố và đổi tên bảng để diễn tập) và `task-settings.json` (LOB, validation, xử lý lỗi). Thư mục `sql/` có chuẩn bị nguồn, đối soát và sửa sequence; `scripts/check-dms.sh` xem trạng thái task. **Không có lệnh nào chạy khi bạn chỉ clone repo, và `start_task` mặc định là `false`.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Replication instance | **Theo giờ** (lớp instance), cộng đĩa theo GB; Multi-AZ nhân đôi; tính cả khi task đang dừng |
| DMS Serverless | Theo DMS Capacity Unit (DCU) giờ đang dùng trong dải min và max; đặt max để chặn chi phí |
| Endpoint, subnet group, IAM role | Không tính tiền riêng |
| Truyền dữ liệu | Giữa AZ và (nếu có) giữa region theo biểu giá chung; nguồn ở on-premises còn tốn băng thông đường truyền |
| Aurora đích | Theo cluster ở bài 31 (instance theo giờ, dung lượng, I/O) |
| Logging CloudWatch | Theo dung lượng log ghi |

Giá cụ thể đổi theo thời gian và region: xem [DMS pricing](https://aws.amazon.com/dms/pricing/) và Pricing Calculator. **Replication instance chạy liên tục tính tiền; dọn ngay khi cutover xong.**

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Một PostgreSQL nguồn đã chuẩn bị theo `sql/source-prepare.sql` (`wal_level = logical`, user có quyền `REPLICATION`) và một cluster Aurora PostgreSQL đích (bài 31) đã có sẵn schema, tạo bằng `pg_dump --schema-only` để giữ index, ràng buộc và identity.
- Hai secret Secrets Manager (bài 14), mỗi cái có JSON gồm `username`, `password`, `engine`, `host`, `port`, `dbname`. Mật khẩu **không** nằm trong Terraform hay biến.
- VPC có private subnet ở ít nhất hai AZ và security group cho DMS ra được cổng 5432 tới nguồn và đích (bài 04); nguồn và đích cũng phải cho phép kết nối từ security group này.
- `create_service_roles = true` chỉ khi tài khoản chưa có role `dms-vpc-role` và `dms-cloudwatch-logs-role` (tạo trùng sẽ lỗi).

## Chạy

```bash
cd examples/toi-noi-aws/p04-40-dms
terraform init
terraform plan \
  -var 'subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var 'security_group_ids=["sg-0dms12345"]' \
  -var source_server_name=10.0.1.25 \
  -var source_secret_arn=arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi-lab-dms-source-AbCdEf \
  -var target_server_name=toinoi-lab-aurora.cluster-xxxx.ap-southeast-1.rds.amazonaws.com \
  -var target_secret_arn=arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi-lab-dms-target-GhIjKl
```

Các ID `subnet-…`, `sg-…`, địa chỉ IP, hostname và ARN secret là giả: thay bằng giá trị của tài khoản lab. Sau `terraform apply`, kiểm hai endpoint bằng **Test connection** trong console (hoặc `aws dms test-connection`), rồi khởi động task: `aws dms start-replication-task --replication-task-arn <arn> --start-replication-task-type start-replication`. Theo dõi:

```bash
export AWS_PROFILE=lab
scripts/check-dms.sh "$(terraform output -raw replication_task_arn)"
```

Khi full load xong và CDC đuổi kịp, đối soát bằng `sql/reconcile.sql` trên cả hai database. Chỉ ở bước cutover mới chạy `sql/fix-sequences.sql` trên đích.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
for f in *.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"))' "$f" && echo "ok $f"; done
bash -n scripts/check-dms.sh
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. Dừng task: `aws dms stop-replication-task --replication-task-arn <arn>`.
2. Trên nguồn, **xoá replication slot** nếu không còn dùng: `SELECT slot_name FROM pg_replication_slots;` rồi `SELECT pg_drop_replication_slot('<tên slot DMS đã tạo>');` (slot bị bỏ quên giữ WAL và làm đầy đĩa nguồn).
3. `terraform destroy` với cùng các `-var` đã dùng khi `apply`.
4. Kiểm lại: `aws dms describe-replication-instances` và `aws dms describe-endpoints` không còn tài nguyên `toinoi-lab-*`.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider), JSON đã parse được, script đã qua `bash -n`; chưa có lần `apply` hay migration nào trên tài khoản AWS thật. Cần bạn xác nhận khi chạy: các khoá trong `task-settings.json` được DMS chấp nhận khi khai báo một phần, giá trị `plugin_name` và `after_connect_script` với phiên bản DMS và PostgreSQL của bạn, các lớp instance và dải DCU còn được hỗ trợ ở region, và thời gian full load thật (phụ thuộc kích thước 730 triệu dòng và băng thông).

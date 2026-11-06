# Lab p04-31-aurora — Aurora PostgreSQL, RDS Proxy, failover và PITR

Đi kèm bài 31: *Aurora PostgreSQL và RDS Proxy: reader endpoint, failover, Serverless v2 và PITR* (slug `aws-aurora-postgresql-rds-proxy-failover`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Cluster `toinoi-lab-trips` (Aurora PostgreSQL) | Lưu trữ theo GB-tháng thực dùng. Với `storage_mode = aurora` (Standard) thêm tiền theo số request I/O; với `aurora-iopt1` (I/O-Optimized) không tính I/O nhưng đơn giá instance và lưu trữ cao hơn. |
| 3 instance `db.serverless` (writer, reader, reporting) | Theo ACU-giờ mà **mỗi instance** đang dùng, tính theo giây. `serverless_min_acu` > 0 nghĩa là mỗi instance luôn giữ tối thiểu số ACU đó kể cả khi rảnh; bằng 0 thì instance có thể tự tạm dừng, nhưng RDS Proxy và `rds.logical_replication = 1` của lab giữ writer và reader tier 0–1 luôn chạy (xem bài 31). |
| RDS Proxy | Theo vCPU của instance DB nằm sau proxy mỗi giờ (với Serverless v2 tính theo ACU); có mức tối thiểu. |
| Backup tự động (7 ngày) | Dung lượng backup vượt phần miễn phí tính theo GB-tháng. |
| Bản restore / clone khi diễn tập | Cluster mới: tính instance và lưu trữ như cluster thường. Clone `copy-on-write` chỉ tốn thêm cho các trang thay đổi. **Nhớ xoá sau khi thử.** |
| Secrets Manager (secret master do RDS tạo) | Theo secret mỗi tháng, theo số lần gọi API. |
| CloudWatch Logs (`postgresql`) và alarm | Theo log nạp vào/lưu trữ và theo số alarm. |
| Security group, subnet group, IAM | Không tính tiền. |

Không ghi giá cụ thể vì giá đổi theo thời gian và theo region: dùng AWS Pricing Calculator. Lab này tốn tiền ngay khi `apply` xong, nặng hơn các lab Phần 02.

## Điều kiện

- VPC với ít nhất hai private subnet ở hai AZ (bài 04) và một security group của nơi gọi database (task ECS hoặc EC2 thử nghiệm).
- Terraform >= 1.10, AWS CLI v2 với profile SSO.
- `psql` trên máy trong VPC nếu muốn chạy `connect-via-proxy.sh`. Cluster và proxy **không có endpoint công khai**.

## Chạy

```bash
cd examples/toi-noi-aws/p04-31-aurora
terraform init
terraform apply \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var client_security_group_id=sg-0client123
```

Các ID `vpc-…`, `subnet-…`, `sg-…` là giả: thay bằng giá trị của tài khoản lab. Bản engine mặc định là `17.11`; hãy kiểm tra bằng `aws rds describe-db-engine-versions --engine aurora-postgresql --query 'DBEngineVersions[].EngineVersion'` và đổi `engine_version` cùng `parameter_group_family` nếu bản đó không còn.

Tham số `rds.logical_replication` là static: sau `apply` đầu tiên, reboot từng instance để có hiệu lực (cần cho Blue/Green và CDC ở bài 34, 40).

Bảng, user và dữ liệu mẫu tạo bằng `psql` qua cluster endpoint (writer) trước. Tạo user `toinoi_app` với `GRANT rds_iam` để dùng IAM auth qua proxy, và một secret chứa username/password của user này (bài 14) rồi truyền `-var proxy_secret_arn=...`. Nếu để trống, lab dùng secret master cho nhanh, **không dùng cách đó ở production**.

Các script trong `scripts/` dùng biến `CLUSTER`, `SUBNET_GROUP`, `DB_SG` (xem `common.sh`):

```bash
scripts/failover.sh                 # failover chủ động, đo thời gian writer đổi chỗ
scripts/restore-drill.sh pitr       # PITR full-copy ra cluster mới, đo thời gian tới khi sẵn sàng
scripts/restore-drill.sh clone      # fast clone copy-on-write
scripts/restore-drill.sh cleanup    # xoá cluster diễn tập
scripts/connect-via-proxy.sh "$(terraform output -raw proxy_endpoint)"
scripts/blue-green.sh create 18.6 <cluster-parameter-group-đích>   # ví dụ 17 -> 18; kiểm phiên bản đích còn hỗ trợ
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/*.sh
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. Tên tham số của các lệnh `aws rds` trong `scripts/` đã được kiểm với bộ phân tích tham số của AWS CLI, chưa chạy thật. Blue/Green **không hỗ trợ** cluster dùng mật khẩu master do RDS quản lý (`manage_master_user_password = true`, như lab). Muốn thử `blue-green.sh`, trước hết cho proxy dùng secret riêng của `toinoi_app` (`proxy_secret_arn`), rồi tắt quản lý mật khẩu bằng `aws rds modify-db-cluster --no-manage-master-user-password --master-user-password ...` (secret do RDS tạo sẽ bị xoá). Cluster cũng không được đang là publisher của Debezium (bài 34).

**Chưa kiểm chứng trên tài khoản thật:** `apply` thành công với bản engine mặc định, RDS Proxy dùng secret master do RDS quản lý, điều kiện tiên quyết của Blue/Green, thời gian failover và restore.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p04-31-aurora/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
scripts/restore-drill.sh cleanup    # nếu đã diễn tập restore
terraform destroy                   # với cùng các biến
```

Kiểm tra lại, đặc biệt các cluster diễn tập và snapshot còn sót (snapshot thủ công không bị xoá cùng cluster):

```bash
aws rds describe-db-clusters --query 'DBClusters[].DBClusterIdentifier'
aws rds describe-db-proxies --query 'DBProxies[].DBProxyName'
aws rds describe-db-cluster-snapshots --snapshot-type manual --query 'DBClusterSnapshots[].DBClusterSnapshotIdentifier'
```

Log group `/aws/rds/cluster/toinoi-lab-trips/postgresql` không bị xoá cùng cluster; xoá thủ công nếu không cần.

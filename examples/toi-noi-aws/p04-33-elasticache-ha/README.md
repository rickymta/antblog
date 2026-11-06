# Lab p04-33-elasticache-ha — replication group cluster mode và MemoryDB

Đi kèm bài 33: *ElastiCache nâng cao và MemoryDB: HA, failover, cluster mode, eviction và pub/sub* (slug `aws-elasticache-nang-cao-memorydb-ha`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Replication group `toinoi-lab-ha` cluster mode (mặc định 2 shard, mỗi shard 1 primary và 1 replica, `cache.t4g.micro`) | **Theo giờ cho từng node**, tính cả khi không có traffic. Mặc định là 4 node. |
| Data transfer giữa AZ | Replica ở AZ khác primary làm phát sinh transfer cho replication; client khác AZ với node cũng vậy. |
| Log group slow log (CloudWatch Logs) | Theo dung lượng log nạp vào và lưu trữ (giữ 14 ngày). |
| Alarm CloudWatch (2 alarm mỗi node) | Theo số alarm. Mặc định 8 alarm. |
| MemoryDB (chỉ khi `create_memorydb = true`, mặc định 1 shard, 1 primary và 1 replica, `db.t4g.small`) | Theo giờ từng node, cộng **dung lượng dữ liệu ghi vào** transaction log theo GB, cộng lưu trữ snapshot vượt phần miễn phí. Không có chế độ tạm dừng. |
| Subnet group, parameter group, ACL, user, security group | Không tính tiền. |

Không ghi giá cụ thể vì giá đổi theo thời gian và theo engine (Valkey và Redis OSS khác nhau): dùng AWS Pricing Calculator. Lab này tốn tiền ngay khi `apply` xong.

## Điều kiện

- VPC với ít nhất hai private subnet ở hai AZ (bài 04) và một security group của nơi gọi cache (task ECS hoặc EC2 thử nghiệm bài 05).
- Terraform >= 1.10, AWS CLI v2 với profile SSO.
- Một máy trong VPC có `valkey-cli` hoặc `redis-cli` hỗ trợ TLS (`--tls`). Cache **không có endpoint công khai**; không thử từ laptop.

## Chạy

```bash
cd examples/toi-noi-aws/p04-33-elasticache-ha
terraform init
terraform apply \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var client_security_group_id=sg-0client123
```

Các ID `vpc-…`, `subnet-…`, `sg-…` là giả: thay bằng giá trị của tài khoản lab. Thêm `-var create_memorydb=true` để tạo MemoryDB. Kiểm tra bản engine hiện có trước (`aws elasticache describe-cache-engine-versions --engine valkey`, `aws memorydb describe-engine-versions --engine valkey`) và đổi `engine_version`, `parameter_group_family`, `memorydb_engine_version` nếu bản mặc định không còn.

Các script trong `scripts/` chạy trên máy trong VPC. Đặt `ENDPOINT` là `configuration_endpoint` (output của Terraform) và `RG` nếu tên khác:

```bash
export ENDPOINT="$(terraform output -raw configuration_endpoint)"
scripts/keyslot.sh                  # hash tag quyết định slot
scripts/pubsub-stream-demo.sh       # pub/sub không bền, Streams bền
scripts/probe.sh 180 &              # đo cửa sổ lỗi phía client trong 180 giây...
scripts/test-failover.sh 0001       # ...trong khi ép shard 0001 failover
scripts/reshard.sh 3                # tăng lên 3 shard (resharding online)
```

`test-failover.sh` và `reshard.sh` gọi API thật và chỉ nên chạy trong tài khoản lab. Chúng yêu cầu AWS CLI; các script còn lại chỉ cần `valkey-cli` hoặc `redis-cli`. Với Redis cục bộ không TLS, đặt `REDIS_TLS=0` (và `REDIS_CLUSTER=0` nếu không bật cluster mode).

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/*.sh
```

`keyslot.sh`, `pubsub-stream-demo.sh` và `probe.sh` đã chạy với một Redis 7 cục bộ bật cluster mode (`docker run --rm redis:7-alpine redis-server --cluster-enabled yes`, rồi `redis-cli CLUSTER ADDSLOTSRANGE 0 16383`). Tham số của lệnh `aws elasticache` và `aws memorydb` trong script đã được kiểm với bộ phân tích tham số của AWS CLI, chưa chạy thật.

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. **Chưa kiểm chứng trên tài khoản thật:** tạo replication group cluster mode với engine `valkey` và parameter group family `valkey8`, quy ước tên node `<id>-0001-001` dùng cho alarm, log delivery slow log, MemoryDB với engine Valkey và user xác thực bằng IAM, thời gian failover và resharding.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p04-33-elasticache-ha/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy   # với cùng các biến
```

Kiểm tra lại:

```bash
aws elasticache describe-replication-groups --query 'ReplicationGroups[].ReplicationGroupId'
aws memorydb describe-clusters --query 'Clusters[].Name'
aws elasticache describe-snapshots --query 'Snapshots[].SnapshotName'
```

Log group `/toinoi/lab/elasticache/toinoi-lab-ha/slow-log` bị Terraform xoá cùng stack; nếu bạn tạo thêm log group thủ công thì tự xoá.

# Lab p02-15-elasticache — cache báo giá, session và rate limit

Đi kèm bài 15: *ElastiCache: cache báo giá, session và rate limit bằng Redis hoặc Valkey* (slug `aws-elasticache-redis-cache-rate-limit`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Replication group theo node (mode `node`, mặc định 2 node `cache.t4g.micro`) | **Theo giờ cho từng node**, tính cả khi không có traffic. Hai node (primary và replica) nghĩa là hai lần giá node. |
| ElastiCache Serverless (mode `serverless`) | Theo dung lượng dữ liệu lưu (GB-giờ) cộng ECPU đã dùng; có mức tối thiểu tính tiền ngay cả khi ít dữ liệu. Trần trong lab: 1 GB và 5.000 ECPU/giây. |
| Snapshot | Lab đặt `snapshot_retention_limit = 0` nên không có snapshot. |
| Data transfer giữa AZ | Replica ở AZ khác với primary làm phát sinh transfer giữa AZ cho replication, và client ở AZ khác với node cũng vậy. |
| Subnet group, security group | Không tính tiền. |
| EC2 dùng để thử (bài 05) | Tính theo giờ như bài 05. |

Không ghi giá cụ thể vì giá đổi theo thời gian (kể cả chênh lệch giữa Valkey và Redis OSS): dùng AWS Pricing Calculator.

## Điều kiện

- VPC với private subnet ở ít nhất hai AZ (bài 04) và một security group của nơi gọi cache (task ECS ở bài 12, hoặc EC2 thử nghiệm ở bài 05).
- Terraform >= 1.10, AWS CLI v2 với profile SSO.
- Một máy trong VPC có `redis-cli` hoặc `valkey-cli` hỗ trợ TLS. Tên gói cài đặt khác nhau theo phiên bản Amazon Linux; xem phần kết nối với ElastiCache trong tài liệu chính thức. Cache **không có endpoint công khai**, không thử từ laptop trực tiếp.

## Chạy

```bash
cd examples/toi-noi-aws/p02-15-elasticache
terraform init
terraform apply \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var client_security_group_id=sg-0client123
```

Các ID `vpc-…`, `subnet-…`, `sg-…` là giả: thay bằng giá trị của tài khoản lab. Dùng ElastiCache Serverless: thêm `-var mode=serverless`. Dùng Redis OSS thay Valkey: thêm `-var engine=redis -var engine_version=7.1 -var parameter_group_name=default.redis7` (kiểm tra phiên bản được hỗ trợ hiện tại).

Tạo xong, chép thư mục `scripts/` lên máy thử rồi chạy:

```bash
scripts/smoke-test.sh "$(terraform output -raw endpoint)"
```

Script thử PING, cache báo giá với TTL ngẫu nhiên hoá, session tài xế, rate limit cửa sổ cố định và cửa sổ trượt bằng Lua, rồi in số `keyspace_hits` và `keyspace_misses`. Nếu đặt `auth_token`, xuất `REDISCLI_AUTH` thay vì truyền token qua tham số.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/smoke-test.sh
```

Hai script Lua có thể chạy thử với một Redis cục bộ, không cần AWS (đã chạy khi soạn bài với image `redis:7-alpine`):

```bash
docker run -d --rm --name redis-lab redis:7-alpine
docker run --rm --network container:redis-lab -v "$PWD/scripts:/s" redis:7-alpine \
  redis-cli --eval /s/rate-limit.lua rl:user-42:1 , 10 5
docker stop redis-lab
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. **Chưa kiểm chứng trên tài khoản thật:** việc tạo replication group với engine `valkey` và parameter group `default.valkey8`, ElastiCache Serverless với giới hạn trên, kết nối TLS từ client, và hành vi failover.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p02-15-elasticache/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy   # với cùng các biến
```

Kiểm tra lại và nhớ tắt EC2 thử nghiệm:

```bash
aws elasticache describe-replication-groups --query 'ReplicationGroups[].ReplicationGroupId'
aws elasticache describe-serverless-caches --query 'ServerlessCaches[].ServerlessCacheName'
```

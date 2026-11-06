# Lab p04-34-msk — Amazon MSK, IAM auth và MSK Connect với Debezium

Đi kèm bài 34: *Amazon MSK: Kafka managed, IAM auth, MSK Connect và outbox của Tới Nơi* (slug `aws-msk-kafka-iam-auth-msk-connect`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| MSK Provisioned (mặc định 3 broker `kafka.t3.small`, 10 GiB EBS mỗi broker) | **Theo giờ từng broker**, tính cả khi không có traffic, cộng EBS theo GB-tháng. `kafka.t3.small` chỉ dùng cho lab. Broker Express (`express.m7g.*`) tính theo giờ broker, theo dữ liệu ghi vào và lưu trữ dùng thực tế. |
| MSK Serverless (`mode = serverless`) | Theo giờ cluster, theo số partition mỗi giờ, theo dữ liệu vào/ra và lưu trữ. Không có chế độ tắt khi rảnh. |
| Data transfer giữa AZ | Replication giữa broker ở khác AZ và consumer khác AZ với broker leader. |
| Enhanced monitoring `PER_BROKER` | Một số mức giám sát nâng cao tính theo metric CloudWatch. |
| CloudWatch Logs (broker log, connector log) | Theo dung lượng log. |
| MSK Connect (chỉ khi `create_connector = true`) | Theo MCU-giờ của worker (mặc định 1 worker, 1 MCU), tính khi connector chạy. |
| S3 chứa plugin, Secrets Manager | Theo GB-tháng và theo secret mỗi tháng. |
| Security group, IAM policy, cấu hình broker | Không tính tiền. |

Không ghi giá cụ thể vì giá đổi theo thời gian và theo loại broker: dùng AWS Pricing Calculator. Đây là lab đắt nhất của Phần 04 khi để chạy: **dọn ngay sau khi thử**.

## Điều kiện

- VPC có ba private subnet ở ba AZ (bài 04) và một security group của nơi gọi Kafka.
- Terraform >= 1.10, AWS CLI v2 với profile SSO.
- Máy trong VPC để chạy công cụ Kafka: bản phân phối Kafka (đặt `KAFKA_HOME`) và jar `aws-msk-iam-auth` đặt trong thư mục `libs` của Kafka hoặc trong `CLASSPATH` (xem hướng dẫn IAM access control trong tài liệu MSK). MSK **không có endpoint công khai**.
- Nếu dùng connector: Aurora ở bài 31 đã bật `rds.logical_replication = 1` và reboot, tệp ZIP plugin Debezium PostgreSQL đã tải lên S3, và secret `toinoi-lab/debezium` có khoá `password`.

## Chạy

```bash
cd examples/toi-noi-aws/p04-34-msk
terraform init
terraform apply \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222","subnet-0ccc3333"]' \
  -var client_security_group_id=sg-0client123
```

Các ID `vpc-…`, `subnet-…`, `sg-…` là giả: thay bằng giá trị của tài khoản lab. Tạo cluster MSK mất khá lâu (hàng chục phút). Kiểm tra phiên bản Kafka hiện có bằng `aws kafka list-kafka-versions` và đổi `kafka_version` nếu `3.9.x` không còn.

Sau khi cluster `ACTIVE`, trên máy trong VPC:

```bash
export KAFKA_HOME=/opt/kafka
export CLUSTER_ARN="$(terraform output -raw cluster_arn)"
scripts/create-topics.sh
scripts/describe-topic.sh booking.events.v1
scripts/consumer-lag.sh dispatch-consumer
```

Tạo outbox và user Debezium trên Aurora (writer endpoint), rồi bật connector:

```bash
psql "host=<writer endpoint> dbname=tripsdb user=toinoi_admin sslmode=require" \
  -v dbz_password="$DBZ_PASSWORD" -f sql/outbox.sql
terraform apply ... -var create_connector=true \
  -var plugin_bucket_arn=arn:aws:s3:::toinoi-lab-plugins \
  -var db_host=<writer endpoint> -var db_security_group_id=sg-0db123
```

`scripts/key-partition-demo.sh` chạy cục bộ, không cần Kafka: so số partition được dùng khi khoá là `tripId` với khi khoá là `cityId`. Phép băm trong script chỉ để minh hoạ; Kafka dùng murmur2.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/*.sh
node -e 'JSON.parse(require("fs").readFileSync("debezium-outbox.json"))'
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. Tham số `aws kafka` trong script đã được kiểm với bộ phân tích tham số của AWS CLI; các lệnh `kafka-topics.sh` và `kafka-consumer-groups.sh` chưa chạy với broker thật. **Chưa kiểm chứng trên tài khoản thật:** `apply` với `kafka_version = "3.9.x"` và broker `kafka.t3.small`, broker Express (đổi `broker_instance_type`, bỏ đĩa và cấu hình tuỳ biến), MSK Serverless, IAM policy theo topic và group, và toàn bộ phần MSK Connect: config provider trong worker configuration (tên lớp khớp hướng dẫn của MSK Connect; plugin ZIP phải chứa jar `msk-config-providers`), quyền của service execution role cho topic nội bộ của Kafka Connect (đang để rộng, cần thu hẹp), `kafkaconnect_version = 3.7.x`, và cách Debezium EventRouter định tuyến topic.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p04-34-msk/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

Trên database, xoá replication slot trước nếu bỏ connector, nếu không slot tiếp tục giữ WAL và làm đầy đĩa:

```sql
SELECT pg_drop_replication_slot('toinoi_outbox');
```

```bash
terraform destroy   # với cùng các biến (kể cả create_connector)
```

Kiểm tra lại:

```bash
aws kafka list-clusters-v2 --query 'ClusterInfoList[].ClusterName'
aws kafkaconnect list-connectors --query 'connectors[].connectorName'
```

Log group `/toinoi/lab/msk/...` và `/toinoi/lab/msk-connect/outbox` do Terraform tạo và xoá cùng stack.

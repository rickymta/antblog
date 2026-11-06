# Lab p04-35-kinesis — Kinesis Data Streams và Amazon Data Firehose cho toạ độ GPS

Đi kèm bài 35: *Kinesis Data Streams và Data Firehose: nuốt 6.250 toạ độ GPS mỗi giây* (slug `aws-kinesis-data-streams-firehose-gps`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Kinesis stream `toinoi-lab-driver-location` chế độ `ON_DEMAND` (mặc định) | Theo giờ tồn tại của stream cộng theo dung lượng dữ liệu ghi vào và đọc ra. Có hạn mức dung lượng mặc định; kiểm tra quota hiện hành. |
| Chế độ `PROVISIONED` | Theo **shard-giờ** (tính cả khi rảnh) cộng theo đơn vị payload của mỗi lần PUT. |
| Retention trên 24 giờ | Tính thêm theo shard-giờ hoặc GB tuỳ chế độ. Lab đặt 24 giờ. |
| Consumer enhanced fan-out `lambda-dispatch` | Theo consumer-shard-giờ và theo dung lượng dữ liệu đọc qua enhanced fan-out. |
| Shard-level metrics (hai chỉ số) | Theo metric CloudWatch. |
| Firehose `toinoi-lab-driver-location-to-s3` | Theo dung lượng dữ liệu nhận (GB); chuyển đổi định dạng sang Parquet, dynamic partitioning (theo GB và theo số đối tượng S3 ghi ra) tính thêm. |
| S3 bucket lịch sử | Lưu trữ theo GB-tháng cộng request PUT cho mỗi đối tượng Firehose ghi. |
| Glue Data Catalog (database, table) | Có mức miễn phí theo số đối tượng; xem trang giá. |
| CloudWatch Logs và alarm | Theo log nạp vào/lưu trữ và theo số alarm. |
| Lambda và event source mapping (nếu truyền `consumer_function_arn`) | Theo Lambda như bài 18. |

Không ghi giá cụ thể vì giá đổi theo thời gian và theo region: dùng AWS Pricing Calculator. Stream tính tiền ngay khi tạo, kể cả khi không có dữ liệu.

## Điều kiện

- Terraform >= 1.10, AWS CLI v2 với profile SSO, `jq`.
- Không cần VPC: Kinesis, Firehose và S3 là dịch vụ khu vực, truy cập qua endpoint của AWS.

## Chạy

```bash
cd examples/toi-noi-aws/p04-35-kinesis
terraform init
terraform apply
```

Chế độ shard cố định: `-var stream_mode=PROVISIONED -var shard_count=8`. Muốn có Lambda đọc qua enhanced fan-out: `-var consumer_function_arn=<arn> -var consumer_failure_destination_arn=<arn SQS>`.

Tính trước số shard rồi thử ghi và đọc:

```bash
scripts/shard-math.sh 25000 4 200 30     # 6.250 bản ghi/giây; kích thước 200 byte và dự phòng 30% là giả định
scripts/shard-math.sh 50000 4 200 30     # nếu số tài xế tăng gấp đôi
scripts/put-sample.sh 50
scripts/read-sample.sh
```

Firehose chỉ ghi ra S3 khi buffer đầy hoặc hết thời gian (mặc định 128 MiB hoặc 300 giây): với vài chục bản ghi mẫu, chờ khoảng 5 phút rồi kiểm:

```bash
aws s3 ls "s3://$(terraform output -raw bucket_name)/" --recursive | head
```

Bản ghi mà Firehose không chuyển đổi được (ví dụ JSON thiếu `city`) nằm dưới tiền tố `errors/`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/*.sh
node -e 'JSON.parse(require("fs").readFileSync("sample-record.json"))'
```

`shard-math.sh` là phép tính thuần và đã chạy với hai bộ số của bài. `put-sample.sh` đã được kiểm phần dựng JSON `PutRecords` bằng một lệnh `aws` giả, không gửi dữ liệu thật; tham số các lệnh `aws kinesis` đã được kiểm với bộ phân tích tham số của AWS CLI.

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. **Chưa kiểm chứng trên tài khoản thật:** `apply` toàn bộ (đặc biệt Firehose với dynamic partitioning kết hợp chuyển đổi Parquet, `MetadataExtractionQuery` và JQ-1.6, giới hạn buffer tối thiểu), việc Firehose đọc stream cùng lúc với consumer enhanced fan-out, hành vi lỗi chuyển đổi vào tiền tố `errors/`, event source mapping với enhanced fan-out, và các hạn mức của chế độ on-demand.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p04-35-kinesis/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy
```

Bucket có `force_destroy = true` nên Terraform xoá cả đối tượng bên trong. Kiểm tra lại:

```bash
aws kinesis list-streams --query 'StreamNames'
aws firehose list-delivery-streams --query 'DeliveryStreamNames'
aws s3 ls | grep gps-history
```

Log group `/toinoi/lab/firehose/...` do Terraform tạo và xoá cùng stack.

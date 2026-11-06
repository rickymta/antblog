# Lab p04-32-dynamodb — dữ liệu nóng của dispatch trên DynamoDB

Đi kèm bài 32: *DynamoDB: single-table design cho vị trí tài xế và trạng thái chuyến, GSI, TTL và Streams* (slug `aws-dynamodb-single-table-gsi-ttl-streams`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Bảng `toinoi-lab-dispatch` (mặc định on-demand) | Theo số request ghi và đọc thực tế (WCU và RCU tương đương) cộng dung lượng lưu trữ theo GB-tháng. Trần `max_write_request_units` và `max_read_request_units` (mặc định 2.000) chặn hoá đơn phình khi có lỗi, đổi lại request vượt trần bị throttle. Chế độ `PROVISIONED` tính theo RCU/WCU khai báo mỗi giờ. |
| GSI `gsi1` | Mỗi ghi làm đổi khoá hoặc thuộc tính được chiếu của index tốn thêm đơn vị ghi cho index, cộng lưu trữ riêng. |
| Point-in-time recovery | Theo dung lượng bảng mỗi tháng. Cần để export ra S3. |
| DynamoDB Streams | Tính theo số lần đọc stream (Lambda qua event source mapping thường nằm trong phần miễn phí cho Lambda, kiểm tra trang giá). |
| TTL | Việc xoá không tính WCU. |
| Warm throughput (nếu đặt) | Làm nóng bảng có thể phát sinh phí; xem trang giá. |
| Alarm CloudWatch | Theo số alarm. |
| Event source mapping và Lambda (nếu truyền `stats_function_arn`) | Theo Lambda như bài 18; đích lỗi SQS/SNS tính riêng. |

Không ghi giá cụ thể vì giá đổi theo thời gian và theo region: dùng AWS Pricing Calculator.

## Điều kiện

- Terraform >= 1.10, AWS CLI v2 với profile SSO; `jq` cho `scripts/queries.sh`.
- Để thử không tốn tiền: chạy [DynamoDB Local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.html) bằng Docker (`docker run --rm -p 8000:8000 amazon/dynamodb-local`) và đặt `DDB_ENDPOINT=http://localhost:8000`, kèm thông tin xác thực giả (`AWS_ACCESS_KEY_ID=local AWS_SECRET_ACCESS_KEY=local`). DynamoDB Local không mô phỏng đủ hành vi của dịch vụ thật (đặc biệt TTL và giới hạn throughput).

## Chạy trên AWS thật

```bash
cd examples/toi-noi-aws/p04-32-dynamodb
terraform init
terraform apply
```

Truyền thêm `-var stats_function_arn=<arn Lambda> -var stats_failure_destination_arn=<arn SQS>` để tạo event source mapping cho Streams. Role của Lambda cần `dynamodb:DescribeStream`, `GetRecords`, `GetShardIterator`, `ListStreams` trên stream của bảng.

Chạy các access pattern (đặt `TABLE` nếu tên bảng khác):

```bash
scripts/seed.sh                              # 3 tài xế, 20 điểm lịch sử, 2 chuyến SEARCHING
scripts/queries.sh d001 w3gvk                # vị trí mới nhất, lịch sử 10 phút, tài xế rảnh trong ô
scripts/put-location.sh d001 10.7775 106.7015
scripts/claim-trip.sh t9001 d001 claim-t9001-d001   # thành công
scripts/claim-trip.sh t9001 d002 claim-t9001-d002   # bị từ chối: chuyến đã có người nhận
```

Với DynamoDB Local (không dùng Terraform), tạo bảng bằng `scripts/create-table.sh` trước `seed.sh`.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/*.sh
node -e 'JSON.parse(require("fs").readFileSync("table-definition.json"))'
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. Các script trong `scripts/` đã chạy trọn vòng (`create-table`, `seed`, `queries`, `put-location`, `claim-trip` hai lần) trên moto, một bộ mô phỏng DynamoDB cục bộ; đó không phải DynamoDB Local hay dịch vụ thật. **Chưa kiểm chứng trên tài khoản thật:** `apply` (đặc biệt `on_demand_throughput` cho bảng và GSI, `warm_throughput`), trần throughput, hành vi TTL thật, event source mapping với filter, và các con số WCU/RCU trong bài.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p04-32-dynamodb/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy
```

Kiểm tra lại. Bảng có PITR sẽ không còn backup sau khi xoá bảng, nhưng bản export hoặc backup thủ công (nếu bạn tạo) vẫn tồn tại và tính tiền:

```bash
aws dynamodb list-tables --query 'TableNames'
aws dynamodb list-backups --query 'BackupSummaries[].BackupName'
```

# Lab p04-39-opensearch: tìm quán theo tên và vị trí, phân tích log

Lab của **bài 39** ([OpenSearch Service: tìm quán và địa chỉ, geo query và phân tích log tập trung](../../../api/seed/content/aws-opensearch-tim-kiem-log-analytics.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 04**.

Terraform dựng một **OpenSearch Serverless collection** loại SEARCH (mặc định) hoặc một **domain managed** nhỏ trong VPC (`-var mode=domain`). Thư mục `requests/` có mapping index quán (analyzer bỏ dấu tiếng Việt, `geo_point`), truy vấn `geo_distance` 3 km, hoán đổi alias, reindex, index template và hai chính sách ISM cho log. `scripts/seed_and_search.py` nạp 6 quán mẫu ở Đà Nẵng rồi tìm "bun bo" trong 3 km. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Serverless collection (mode `serverless`) | **OCU-giờ** cho indexing và cho search, có mức tối thiểu tính tiền kể cả khi không có traffic, cộng dung lượng lưu. Lab đặt `standby_replicas = "DISABLED"` để giảm mức tối thiểu; môi trường production nên bật standby |
| Domain managed (mode `domain`) | **Theo giờ cho mỗi data node** (và master, UltraWarm nếu bật), cộng EBS theo GB; tính cả khi không có query |
| Chính sách mã hoá, mạng, quyền dữ liệu | Không tính tiền riêng |
| Truyền dữ liệu | Truyền ra Internet và giữa AZ theo biểu giá chung |

Giá cụ thể đổi theo thời gian và region: xem [OpenSearch Service pricing](https://aws.amazon.com/opensearch-service/pricing/) và Pricing Calculator. **Đây là lab tốn tiền theo giờ; hãy dọn ngay sau khi thử.**

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Python 3 với `pip install opensearch-py boto3` để chạy script nạp dữ liệu.
- Mode `domain`: private subnet và security group (bài 04), và một máy trong VPC (EC2 ở bài 05 hoặc ECS task) để gọi domain, vì domain trong VPC không có endpoint công khai. Lần đầu tạo domain trong VPC có thể cần service-linked role `es.amazonaws.com`.
- Biến bắt buộc `principal_arn`: ARN role hoặc user IAM được ghi dữ liệu. Dùng role SSO của bạn (xem `aws sts get-caller-identity`), không dùng access key dài hạn.

## Chạy

Serverless (mặc định):

```bash
cd examples/toi-noi-aws/p04-39-opensearch
terraform init
terraform apply -var principal_arn=arn:aws:iam::123456789012:role/<role-sso-cua-ban>
export AWS_PROFILE=lab
python3 scripts/seed_and_search.py --endpoint "$(terraform output -raw collection_endpoint)" --service aoss
```

Domain (chạy script từ máy trong VPC, `--service es`):

```bash
terraform apply -var mode=domain -var principal_arn=<arn> \
  -var 'vpc_subnet_ids=["subnet-0aaa1111"]' -var 'vpc_security_group_ids=["sg-0search123"]'
```

Các ID `subnet-…`, `sg-…` là giả: thay bằng giá trị của tài khoản lab. Các lệnh REST khác (alias, reindex, ISM) trong `requests/` gọi bằng một client ký SigV4 (ví dụ `awscurl`, hoặc `opensearch-py` như script). Ví dụ trên domain:

```bash
awscurl --service es --region ap-southeast-1 -X PUT "$ENDPOINT/_plugins/_ism/policies/toinoi-logs-hot-warm-delete" \
  -H 'Content-Type: application/json' -d @requests/ism-policy-hot-warm-delete.json
```

Trước khi hoán đổi alias, tạo alias `toinoi-merchants` trỏ vào `toinoi-merchants-v1` (và tạo `toinoi-merchants-v2` với mapping mới), rồi gọi `_reindex` và `_aliases` với hai tệp trong `requests/`. Serverless có `_aliases` nhưng **không có `_reindex`**: ở đó hãy nạp lại v2 từ nguồn (Aurora, hoặc chạy lại `seed_and_search.py` với tên index mới) rồi mới hoán đổi alias. ISM chỉ có trên domain; Serverless dùng data lifecycle policy cho collection time series.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
for f in requests/*.json data/*.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"))' "$f" && echo "ok $f"; done
python3 -m py_compile scripts/seed_and_search.py
```

Sau khi kiểm, xoá `.terraform`, `.terraform.lock.hcl` và `scripts/__pycache__` (không commit).

## Dọn tài nguyên

```bash
terraform destroy -var principal_arn=arn:aws:iam::123456789012:role/<role-sso-cua-ban>
```

(thêm các `-var` của mode `domain` nếu bạn đã dùng). Kiểm lại: `aws opensearchserverless list-collections` không còn `toinoi-lab-search`, hoặc `aws opensearch list-domain-names` không còn domain `toinoi-lab-search`.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider); JSON đã parse được, script Python đã qua `py_compile`. Chưa có lần `apply` hay truy vấn nào trên tài khoản AWS thật. Đã đối chiếu tài liệu: `OpenSearch_2.19` nằm trong danh sách phiên bản được hỗ trợ; Serverless có `_aliases` (quyền `aoss:CreateCollectionItems`) nhưng không có `_reindex`; bộ lọc `asciifolding` của Lucene đổi `đ` thành `d`. Cần bạn xác nhận khi chạy: quyền trong data access policy đủ cho các lệnh bạn dùng và kết quả tìm kiếm với dữ liệu tiếng Việt thật.

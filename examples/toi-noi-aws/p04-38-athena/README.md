# Lab p04-38-athena: Glue Data Catalog, Athena workgroup và bảng Parquet theo partition

Lab của **bài 38** ([Glue, Athena và Redshift Serverless: lakehouse phân tích 2 triệu chuyến mỗi ngày](../../../api/seed/content/aws-glue-athena-redshift-lakehouse.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 04**.

Terraform dựng bucket data lake, bucket kết quả Athena, Glue database `toinoi_lab_analytics`, bảng `trips_curated` (Parquet, partition `dt` và `city`, dùng partition projection), Athena workgroup có trần dữ liệu quét, một named query, và (tuỳ chọn) quyền Lake Formation che cột `customer_phone`. **Lab không tạo file Parquet nào**: thư mục `schema/` mô tả cấu trúc file, thư mục `sql/` có lệnh CTAS sinh Parquet từ JSON raw nếu bạn có dữ liệu thử. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Bucket lake và bucket kết quả (rỗng) | Không tốn tiền khi chưa có object; có dữ liệu thì theo GB-tháng; kết quả Athena tự xoá sau 7 ngày |
| Glue database và table | Lưu metadata trong Data Catalog có mức miễn phí theo số object và request; xem trang giá Glue |
| Glue crawler, Glue ETL | Lab **không tạo**; nếu bạn thêm, tính theo DPU-giờ |
| Athena query | **Theo lượng dữ liệu quét** (mỗi query có mức tính tối thiểu); partition projection và Parquet giúp quét ít hơn |
| Athena workgroup | Không tính tiền riêng; metric CloudWatch của workgroup tính theo giá CloudWatch |
| Lake Formation | Không phí riêng; chỉ tạo khi có `analyst_role_arn` |

Giá cụ thể đổi theo thời gian và region: xem [Athena pricing](https://aws.amazon.com/athena/pricing/) và Pricing Calculator.

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Để thử query thật, bạn cần ít nhất một file JSON lines hoặc Parquet trong `raw/trips/` hoặc `curated/trips/dt=.../city=.../` của bucket lake. Có thể xuất dữ liệu thử từ PostgreSQL của bài 07 hoặc tự tạo file nhỏ.
- Nếu dùng `analyst_role_arn`: tài khoản lab chưa dùng Lake Formation cho dữ liệu khác, vì thay đổi cách cấp quyền có thể ảnh hưởng nơi khác.

## Chạy

```bash
cd examples/toi-noi-aws/p04-38-athena
terraform init
terraform plan
terraform apply
```

Sau đó chạy lệnh trong thư mục `sql/` bằng console Athena (chọn workgroup `toinoi-lab-analytics`, database `toinoi_lab_analytics`), hoặc bằng script, ví dụ:

```bash
export AWS_PROFILE=lab
scripts/run-athena-query.sh "SELECT count(*) FROM trips_curated WHERE dt = '2026-03-02'"
```

Script in `QueryExecutionId`, số byte đã quét và 20 dòng đầu kết quả. Thử câu số 3 trong `sql/03-queries.sql` (không lọc `dt`) để thấy trần `bytes_scanned_cutoff_per_query` huỷ câu lệnh khi quét quá lớn.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/run-athena-query.sh
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. Xoá dữ liệu bạn đã đưa vào hai bucket (`aws s3 rm s3://<bucket> --recursive`).
2. `terraform destroy` (workgroup có `force_destroy = true` nên xoá được cả khi có lịch sử query).
3. Kiểm lại: `aws s3api head-bucket` cho hai bucket phải báo 404 và `aws glue get-databases` không còn `toinoi_lab_analytics`.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider), script đã qua `bash -n`; JSON/SQL chưa chạy trên Athena. Cần bạn xác nhận khi chạy: partition projection trên cột `dt` kiểu string với định dạng ngày, hành vi `INSERT INTO` hoặc CTAS ghi vào bảng có projection, cú pháp Iceberg và `OPTIMIZE` theo phiên bản engine, và cấu hình Lake Formation (đăng ký vị trí S3 và quyền cột) với role thật. Ước tính kích thước Parquet trong `schema/` là giả định.

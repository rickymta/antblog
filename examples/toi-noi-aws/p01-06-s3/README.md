# Lab p01-06-s3: bucket ảnh chứng từ giao hàng

Lab của **bài 06** ([S3: bucket, versioning, lifecycle và presigned URL](../../../api/seed/content/aws-s3-bucket-versioning-presigned-url.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 01**.

Script AWS CLI v2 (bash) tạo một bucket `toinoi-lab-delivery-photos-<account-id>` ở `ap-southeast-1` với Block Public Access, ACL tắt, mã hoá SSE-S3, versioning, lifecycle, CORS và bucket policy; sau đó thử presigned URL GET/PUT và quan sát delete marker. **Không có lệnh nào chạy khi bạn chỉ clone repo**: bạn tự chạy script trong tài khoản lab.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Bucket S3 (rỗng) | Không tốn tiền khi chưa có object |
| Object và mọi version của nó | Dung lượng lưu theo GB-tháng của từng storage class |
| Request PUT, GET, LIST, lifecycle transition | Số request |
| Lấy dữ liệu từ Standard-IA / Glacier Instant Retrieval | GB được lấy |
| Truyền dữ liệu ra Internet | GB ra khỏi AWS (phần ảnh test rất nhỏ) |

Giá cụ thể đổi theo thời gian và region: xem [S3 pricing](https://aws.amazon.com/s3/pricing/) và Pricing Calculator. Với vài object test, chi phí gần như bằng không, miễn là bạn chạy bước dọn.

## Chuẩn bị

- AWS CLI v2 đã đăng nhập vào **tài khoản lab** (bài 02). Script in danh tính và hỏi xác nhận trước khi tạo hoặc xoá.
- `curl`, `bash`; riêng `02b-presign-put.sh` cần `python3` và `boto3` (`python3 -m pip install --user boto3`).
- Sao chép `env.sh.example` thành `env.sh` và sửa `AWS_PROFILE`, tuỳ chọn `BUCKET`, `WEB_ORIGIN`, `APP_ROLE_ARN`. `env.sh` không chứa secret, nhưng đừng commit giá trị tài khoản thật.

## Chạy thử

```bash
cp env.sh.example env.sh
./01-create-bucket.sh        # tạo và cấu hình bucket
./02-presign.sh              # presigned GET: 403 khi không ký, 200 khi ký, 403 khi hết hạn
./02b-presign-put.sh         # presigned PUT: 200 đúng Content-Type, 403 sai Content-Type
./03-versioning-demo.sh      # delete marker, khôi phục, xoá vĩnh viễn bằng version id
```

Nếu có `APP_ROLE_ARN` (role ở bài 03), bucket policy cho phép role đó `GetObject` và `PutObject` trên `proof/*`. ARN không tồn tại sẽ làm S3 từ chối policy. Policy `delivery-photo-policy.json` của bài 03 viết cho tên `toinoi-lab-delivery-photos`; nếu muốn role đó thật sự upload được vào bucket của lab này, sửa ARN bucket trong policy thành tên có hậu tố account rồi cập nhật policy (`aws iam create-policy-version ... --set-as-default`).

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in *.sh; do bash -n "$f" && echo "ok $f"; done
for f in lifecycle.json *.json.tpl; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"))' "$f" && echo "ok $f"; done
docker run --rm -v "$PWD:/mnt" -w /mnt koalaman/shellcheck:stable -x -P /mnt *.sh
```

## Dọn tài nguyên

```bash
./99-cleanup.sh
```

Script xoá **mọi version và delete marker**, rồi xoá bucket. Nó từ chối chạy nếu tên bucket không bắt đầu bằng `toinoi-lab-`. Kiểm lại: `aws s3api head-bucket --bucket <tên>` phải báo lỗi 404.

## Chưa được kiểm chứng

Script đã qua `bash -n` và shellcheck, JSON đã parse được. Chưa có lần chạy nào trên tài khoản AWS thật: hành vi của lifecycle (chạy bất đồng bộ), lỗi 403 cụ thể của presigned URL và định dạng bucket policy với role thật cần bạn xác nhận khi chạy.

# Lab p01-07-rds: RDS PostgreSQL cho bảng trips

Lab của **bài 07** ([RDS PostgreSQL: tạo instance, Multi-AZ, backup và kết nối từ private subnet](../../../api/seed/content/aws-rds-postgresql-multi-az-backup.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 01**.

Script AWS CLI v2 (bash) tạo một RDS PostgreSQL `toinoi-lab-trips` trong private subnet, mật khẩu do RDS quản lý trong Secrets Manager, rồi thử Multi-AZ failover, snapshot và point-in-time restore. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Instance RDS `db.t4g.micro` | Giờ instance chạy, từ lúc tạo tới lúc xoá |
| Standby khi bật Multi-AZ (`03-...`) | Thêm một lượng instance và đĩa tương đương |
| Đĩa gp3 20 GiB (tự tăng tới 100 GiB nếu đầy) | GB-tháng đã cấp |
| Automated backup | Miễn phí tới bằng dung lượng đã cấp; phần vượt tính theo GB-tháng |
| Manual snapshot (`04-...`) | GB-tháng cho tới khi bạn xoá |
| Instance restore (`RESTORE=1`) | Như một instance mới, cho tới khi xoá |
| Secret do RDS quản lý | Theo secret mỗi tháng và số lần gọi API |
| Log group `/aws/rds/instance/.../postgresql` | Dung lượng log lưu; xoá bằng script dọn |
| DB subnet group, parameter group, security group | Không tốn tiền |

Giá cụ thể thay đổi theo region và thời gian: xem [RDS for PostgreSQL pricing](https://aws.amazon.com/rds/postgresql/pricing/) và Pricing Calculator. Đừng để lab chạy qua đêm.

## Điều kiện

- Lab bài 04 dựng **với NAT** (`create-vpc.sh --apply --with-nat`): VPC `toinoi-lab-vpc`, hai subnet private-data `toinoi-lab-data-a/-b`, security group `toinoi-lab-app` và `toinoi-lab-db` (sg-db, 5432 chỉ từ sg-app).
- Lab bài 05 chạy **trong VPC đó**: `create-instance.sh toinoi-lab --apply --use-lab-vpc`, để máy nằm ở subnet private-app với sg-app và có role `toinoi-lab-quote-ec2`. Máy ở default VPC không nối được tới DB.
- Sao chép `env.sh.example` thành `env.sh`. Để trống `VPC_ID`, `DB_SUBNET_IDS`, `SG_APP_ID` thì `common.sh` tự tra theo tag Name của bài 04; chỉ điền khi bạn dựng mạng theo cách khác. `PG_FAMILY` để trống thì script tự tra theo phiên bản engine.

## Chạy thử

```bash
cp env.sh.example env.sh
./01-create-db.sh              # subnet group, security group, parameter group, instance (vài phút)
./02-allow-ec2-read-secret.sh  # cho role EC2 đọc đúng secret của DB (nếu có EC2_ROLE_NAME)
```

Sau đó dùng Session Manager vào EC2, đưa `ec2-psql.sh` và `sql/trips.sql` lên máy (ví dụ dán nội dung), rồi chạy:

```bash
./ec2-psql.sh <endpoint> <secret-arn> trips.sql
```

`endpoint` và `secret-arn` được in ở cuối `01-create-db.sh`. Câu `SHOW` cuối của `trips.sql` phải cho `log_min_duration_statement = 500ms`. Tiếp tục thử (tuỳ chọn, tốn thêm tiền):

```bash
./03-multi-az-failover.sh      # bật Multi-AZ rồi ép failover
RESTORE=1 ./04-snapshot-restore.sh   # snapshot thủ công và restore ra instance mới
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in *.sh; do bash -n "$f" && echo "ok $f"; done
node -e 'JSON.parse(require("fs").readFileSync("policy-read-db-secret.json.tpl","utf8"))' && echo "ok policy"
docker run --rm -v "$PWD:/mnt" -w /mnt koalaman/shellcheck:stable -x -P /mnt *.sh
```

## Dọn tài nguyên

```bash
./99-cleanup.sh
```

Script xoá instance (kể cả bản restore, **không tạo final snapshot**), mọi manual snapshot tên bắt đầu bằng `toinoi-lab-`, log group của RDS, subnet group, parameter group, inline policy trên role EC2, và sg-db nếu chính lab này tạo (sg-db của bài 04 để `cleanup.sh` bài 04 xoá). Nó từ chối chạy nếu `DB_ID` không bắt đầu bằng `toinoi-lab-`. Cuối script liệt kê lại instance và snapshot còn sót; kết quả phải rỗng.

## Chưa được kiểm chứng

Script đã qua `bash -n` và shellcheck. Chưa chạy trên tài khoản AWS thật: tên family/phiên bản PostgreSQL, thời gian failover và khả năng cài `psql` bằng `dnf` trên Amazon Linux 2023 cần bạn xác nhận khi chạy.

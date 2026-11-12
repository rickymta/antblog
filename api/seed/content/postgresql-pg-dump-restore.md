---
slug: postgresql-pg-dump-restore
title: "Backup logic và khôi phục với pg_dump: kiểm cả dữ liệu lẫn quyền"
summary: "Tạo custom-format dump, restore sang database riêng và đối soát số dòng, tổng tiền, digest, owner cùng quyền đọc/ghi."
category: postgresql
level: intermediate
tags: [postgresql, pg-046, backup, restore, toi-noi]
language: vi
---

Có một file backup không đồng nghĩa có thể khôi phục. Với Tới Nơi, bản phục hồi phải đủ chuyến, đúng giá tiền, đúng chủ sở hữu và quyền truy cập. Bài này chạy trọn một vòng backup logic rồi restore vào database khác để kiểm những điều đó.

<div class="tkh-callout"><span class="tkh-callout-title">PG-046 · Phần 5/6 · Trung bình</span><p>Đọc trước: <a href="/articles/postgresql-wal-checkpoint-durability">WAL và checkpoint</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, PostgreSQL 18.1. Chạy <code>npm run p05</code>; source và đích phục hồi đều do runner tạo.</p></div>

## 1. Nêu phạm vi trước khi chạy

Lab dùng `pg_dump -Fc -n ops`: archive custom format của schema ops trong database nguồn. Đây là dump có chọn lọc, không phải backup toàn cluster. Roles thuộc cấp cluster, không tự được thay thế bởi dump database. [SQL dump và pg_dumpall](https://www.postgresql.org/docs/18/backup-dump.html).

Đích lab ở cùng cluster nên tn_owner, tn_app, tn_report đã tồn tại. Khi chuyển sang cluster mới, phải chuẩn bị roles/tablespaces hoặc mapping owner riêng. Không bỏ qua lỗi role không tồn tại rồi coi restore xong.

## 2. Ba bước thực thi

Dạng lệnh dưới đây minh hoạ tham số; PORT và DUMP_FILE cần thay bằng giá trị lab:

```text
pg_dump -h 127.0.0.1 -p <PORT> -U postgres -d postgres
        -Fc -n ops -f <DUMP_FILE>
createdb -h 127.0.0.1 -p <PORT> -U postgres p05_restored
pg_restore -h 127.0.0.1 -p <PORT> -U postgres
           -d p05_restored --exit-on-error <DUMP_FILE>
```

Runner dùng SQL CREATE DATABASE thay cho createdb, kết quả tương đương. Các argument host/port đều tường minh để không vô tình dùng server mặc định của máy. Database đích mới giúp tránh ghi đè dữ liệu đang phục vụ.

Exit-on-error khiến lỗi restore trở thành thất bại cần xử lý. Exit code 0 chỉ là cổng đầu tiên, sau đó mới đối soát dữ liệu và hành vi.

## 3. COUNT chưa đủ

```sql
SELECT count(*), sum(fare),
       md5(string_agg(id || ':' || tenant_id || ':' || fare || ':' || status,
                      ',' ORDER BY id)) AS data_digest
FROM ops.trips;
```

Fixture có **100 chuyến, 3.000.000 đ**; runner so digest nguồn và đích, thứ tự id được xác định tường minh. Digest này dùng cho dữ liệu nhỏ với cột không NULL và delimiter đã kiểm soát. Nó không phải checksum tổng quát cho dữ liệu bất kỳ hay bằng chứng chống giả mạo.

Với bảng lớn, cần chia chunk/khoảng khoá, định nghĩa serialize NULL/chuỗi/kiểu phức tạp và phối hợp tổng nghiệp vụ. Hai bảng cùng COUNT vẫn có thể khác một giá tiền hoặc khác khách hàng.

## 4. Khôi phục cả quyền

```sql
SELECT tableowner FROM pg_tables
WHERE schemaname = 'ops' AND tablename = 'trips';
SELECT has_table_privilege('tn_report', 'ops.trips', 'SELECT');
```

Runner yêu cầu owner vẫn là tn_owner, report có SELECT và UPDATE bằng report bị chặn với `42501`. Một bản dữ liệu đúng nhưng mọi người đều sửa được vẫn chưa đạt bàn giao.

Dump được tạo trước bước bật RLS để tách phép thử ACL khỏi policy. Phép thử RLS sau phục hồi vật lý nằm ở PG-048/049. Bài này chưa kiểm mọi extension, large object hoặc dependency ngoài ops; phạm vi đó cần được bổ sung khi chuyển từ schema lab sang database thực.

## 5. Bài tập restore có ràng buộc

Thêm một CHECK và một cột bắt buộc, tạo dump mới rồi restore. Ở đích, thử INSERT vi phạm và yêu cầu bị chặn. Nộp biên bản có phiên bản công cụ, phạm vi, thời điểm, kết quả restore, count/tổng/digest, owner/ACL và ca dữ liệu sai. Một ảnh file dump lớn hơn 0 byte chưa phải nghiệm thu khôi phục.

Đọc tiếp: [Pool và timeout](/articles/postgresql-connection-pool-timeout).

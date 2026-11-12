---
slug: postgresql-role-grant-least-privilege
title: "Role, GRANT và quyền tối thiểu: tách API, báo cáo và migration"
summary: "Tách owner, application, report và monitor của Tới Nơi; kiểm quyền được phép, quyền bị cấm và default privileges trên bảng tạo mới."
category: postgresql
level: beginner
tags: [postgresql, pg-041, roles, permissions, toi-noi]
language: vi
---

API đặt chuyến, job báo cáo và job migration cùng dùng tài khoản sở hữu bảng: một lỗi trong job báo cáo có thể xoá dữ liệu chuyến. Trước khi tối ưu vận hành, Tới Nơi cần trả lời **mỗi chương trình được làm gì, trên đối tượng nào?**

<div class="tkh-callout"><span class="tkh-callout-title">PG-041 · Phần 5/6 · Cơ bản</span><p>Đọc trước: <a href="/articles/postgresql-thuc-hanh-tranh-chap-nhan-chuyen">bài tổng hợp phần 4</a>. Lab phần này: <code>examples/toi-noi-postgresql/p05</code>, PostgreSQL 18.1 thật. Chạy <code>npm run p05</code> từ thư mục <code>examples/toi-noi-postgresql</code>; cần binary PostgreSQL 18 và OpenSSL. Runner tạo cluster tạm riêng rồi dọn khi kết thúc.</p></div>

## 1. Tách người sở hữu khỏi runtime

Lab dùng projection `ops.trips`: 100 chuyến hoàn thành, hai tenant, mỗi chuyến 30.000 đ. Nó độc lập với schema P01–P04 để các phép thử quản trị không làm thay đổi fixture cũ. `tn_owner` sở hữu bảng nhưng không đăng nhập; API chạy bằng `tn_app`; báo cáo dùng nhóm `tn_report` chỉ đọc.

| Role | Trách nhiệm | Quyền không cần |
|---|---|---|
| Owner / migration | Tạo và đổi đối tượng | Không chia sẻ credential với API |
| Application | Đọc, thêm, sửa theo use case | DDL và superuser |
| Report | Đọc dữ liệu được phép | UPDATE, DELETE |
| Monitor | Chẩn đoán hoạt động | Ghi dữ liệu nghiệp vụ |

Role có thể là danh tính đăng nhập hoặc nhóm quyền. Gán nhóm giúp quản lý nhiều tài khoản, nhưng cần xét cả quyền thừa kế và khả năng SET ROLE. [Tài liệu role membership](https://www.postgresql.org/docs/18/role-membership.html).

## 2. Cấp quyền theo lớp

Trích `p05/00-schema.sql`; DDL bảng và fixture nằm trong file đầy đủ:

```sql
CREATE ROLE tn_owner NOLOGIN;
CREATE ROLE tn_app LOGIN;
CREATE ROLE tn_report NOLOGIN;
CREATE SCHEMA ops AUTHORIZATION tn_owner;
SET ROLE tn_owner;
-- Tạo ops.trips tại đây.
GRANT USAGE ON SCHEMA ops TO tn_app, tn_report;
GRANT SELECT, INSERT, UPDATE ON ops.trips TO tn_app;
GRANT SELECT ON ops.trips TO tn_report;
ALTER DEFAULT PRIVILEGES IN SCHEMA ops
  GRANT SELECT ON TABLES TO tn_report;
RESET ROLE;
```

USAGE trên schema không tự cấp SELECT trên bảng. Default privileges được đặt khi role hiện tại là `tn_owner`, áp cho đối tượng tương lai do role đó tạo, không sửa quyền bảng cũ. Nếu migration vô tình tạo bảng bằng postgres, không nên mong quyền mặc định của owner tự chuyển sang.

Bảng lab dùng ID tường minh nên không có sequence. Với schema có sequence/identity, kiểm đường INSERT thật và quyền cần thiết thay vì copy nguyên ma trận này. Trong triển khai còn phải rà CONNECT, quyền PUBLIC, schema public và group membership đang tồn tại.

## 3. Kiểm cả cho phép và từ chối

```sql
SET ROLE tn_report;
SELECT count(*) FROM ops.trips; -- 100
DELETE FROM ops.trips;         -- phải bị từ chối
RESET ROLE;
SET ROLE tn_app;
ALTER TABLE ops.trips ADD COLUMN bad int; -- phải bị từ chối
```

Runner khẳng định hai lệnh nguy hiểm nhận SQLSTATE `42501`; kiểm owner vẫn là `tn_owner`; tạo `ops.next_report` bằng owner rồi xác nhận report có SELECT nhờ default privileges. Mỗi phép kiểm dùng tiến trình psql riêng để lỗi không làm sót role sang ca khác. Kết quả nằm trong `p05/OUTPUT.txt`.

Lab dùng SET ROLE từ quản trị để kiểm ACL. Kiểm xác thực bằng tài khoản ứng dụng thật là việc của PG-042. Đây là hai cổng khác nhau: một mật khẩu đúng không đảm bảo tài khoản có quyền đúng.

## 4. Khi gặp permission denied

Đừng sửa bằng GRANT ALL cho runtime. Ghi câu SQL, đối tượng và hành động thật sự cần; cấp quyền tối thiểu rồi chạy lại cả ca hợp lệ lẫn ca bị cấm. Nếu chỉ thử SELECT thành công, có thể bỏ sót việc tài khoản cũng được DELETE.

**Bài tập:** tạo job chỉ đọc giá chuyến, không đọc thông tin cá nhân. Thiết kế view với cột cho phép và kiểm cả truy cập qua view lẫn truy cập trực tiếp. Lời giải phải chứng minh đường trực tiếp bị chặn; nếu view dùng cách ly tenant, cần xét thêm ngữ cảnh quyền và RLS ở PG-049.

Đọc tiếp: [Xác thực và TLS](/articles/postgresql-ket-noi-auth-tls).

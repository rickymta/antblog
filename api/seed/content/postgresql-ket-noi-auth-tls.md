---
slug: postgresql-ket-noi-auth-tls
title: "Xác thực và bảo vệ kết nối PostgreSQL: SCRAM, HBA và TLS verify-full"
summary: "Kiểm TLS thật với chứng chỉ lab: đúng danh tính thì kết nối được; plaintext, sai mật khẩu và sai hostname đều bị từ chối."
category: postgresql
level: beginner
tags: [postgresql, pg-042, tls, scram, toi-noi]
language: vi
---

API kết nối được database chưa chứng minh nó đang nói chuyện với đúng server qua đường truyền được bảo vệ. Với Tới Nơi, ta phải kiểm riêng danh tính đăng nhập, mã hoá đường truyền và xác minh server.

<div class="tkh-callout"><span class="tkh-callout-title">PG-042 · Phần 5/6 · Cơ bản</span><p>Đọc trước: <a href="/articles/postgresql-role-grant-least-privilege">Role và GRANT</a>. Lab PostgreSQL 18.1: <code>p05/run.mjs</code>. Chạy <code>npm run p05</code> tại <code>examples/toi-noi-postgresql</code>. OpenSSL được dùng để tạo chứng chỉ tạm; cấu hình đường dẫn trong README.</p></div>

## 1. Ba lớp khác nhau

HBA chọn quy tắc xác thực theo loại kết nối, database, user và địa chỉ. SCRAM kiểm chứng mật khẩu. TLS bảo vệ đường truyền, còn cấu hình phía client quyết định có kiểm chứng chỉ và hostname hay không.

HBA chọn dòng khớp đầu tiên; lỗi xác thực ở đó không khiến server thử dòng dễ hơn bên dưới. `hostssl` chỉ khớp TLS. [Tài liệu pg_hba.conf](https://www.postgresql.org/docs/18/auth-pg-hba-conf.html).

## 2. Cấu hình lab

Runner tạo password ngẫu nhiên và chứng chỉ tự ký hai ngày, SAN `DNS:localhost`. Private key nằm trong thư mục cluster tạm, không đưa vào Git. Nó bật SSL, đặt password SCRAM cho tn_app và viết:

```text
host all postgres 127.0.0.1/32 trust
host all postgres ::1/128 trust
hostssl all tn_app 127.0.0.1/32 scram-sha-256
host all all 127.0.0.1/32 reject
host all all ::1/128 reject
host replication postgres 127.0.0.1/32 trust
```

Trust chỉ dành cho quản trị/replication của cluster lab nghe loopback. Đây không phải cấu hình production: tiến trình khác trên cùng máy có thể đóng vai postgres lúc lab hoạt động. Không chạy lab trust này trên máy nhiều người dùng không tin cậy. Trong triển khai, các danh tính quản trị và replication cũng cần xác thực.

`all` ở trường database không bao gồm physical replication, nên dòng replication cuối vẫn có tác dụng. Khi sửa HBA, kiểm `pg_hba_file_rules`, reload rồi thử kết nối mới; giữ phiên quản trị hợp lệ để sửa cấu hình.

## 3. Client xác minh server

Dạng connection string dưới đây dùng PORT và CERT do runner cấp:

```text
host=localhost hostaddr=127.0.0.1 port=<PORT>
user=tn_app dbname=postgres
sslmode=verify-full sslrootcert=<CERT>
```

`hostaddr` cố định địa chỉ đích, `host` vẫn được kiểm với tên chứng chỉ. Verify-full kiểm cả trust và hostname; chỉ yêu cầu mã hoá chưa diễn đạt đủ yêu cầu này. [SSL phía libpq](https://www.postgresql.org/docs/18/libpq-ssl.html).

```sql
SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid();
```

Kết quả t chứng minh phiên dùng TLS, nhưng tự nó không chứng minh client đã chọn verify-full. Ca sai hostname mới kiểm thêm phần đó.

## 4. Ma trận đã chạy

| Ca | Kết quả |
|---|---|
| Đúng password, trust anchor, localhost | TLS = true |
| sslmode=disable | HBA từ chối |
| Đúng TLS, sai password | Authentication thất bại |
| Đúng IP, host là wrong.invalid | Lỗi tên chứng chỉ |

Runner kiểm cả mã thoát và nội dung lỗi; `OUTPUT.txt` lưu bốn kết quả. Chứng chỉ tự ký được tin cậy tường minh trong lab. Production cần quản lý CA, hạn dùng, rotation và quyền private key, không copy key lab sang dùng tiếp. [Thiết lập SSL server](https://www.postgresql.org/docs/18/ssl-tcp.html).

## 5. Bài tập rotation

Thử trust anchor khác và yêu cầu kết nối phải thất bại. Sau đó thay server certificate bằng chứng chỉ cùng hostname do CA mới ký: ghi thứ tự cập nhật trust store, thay chứng chỉ và kiểm kết nối mới. Không hạ sslmode để làm bài thử thành công. Đầu ra là ma trận kết quả và lớp chịu trách nhiệm chặn từng ca.

Đọc tiếp: [Quan sát hoạt động PostgreSQL](/articles/postgresql-pg-stat-activity-quan-sat).

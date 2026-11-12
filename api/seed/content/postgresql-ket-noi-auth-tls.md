---
slug: postgresql-ket-noi-auth-tls
title: "Xác thực và bảo vệ kết nối PostgreSQL: pg_hba.conf, SCRAM và TLS verify-full"
summary: "API của Tới Nơi «kết nối được database» chưa có nghĩa là nó đang nói chuyện với đúng server qua đường truyền được bảo vệ. Đọc pg_hba.conf theo luật khớp dòng đầu, lưu mật khẩu dạng SCRAM, bật TLS với chứng chỉ lab, rồi chứng minh bằng lỗi thật: plaintext, sai mật khẩu và sai hostname đều bị từ chối — còn verify-ca thì không."
category: postgresql
level: beginner
tags: [postgresql, pg-042, authentication, pg-hba, scram, tls, sslmode, toi-noi]
language: vi
---

Đội hạ tầng của Tới Nơi chuyển database sang một máy mới. Chuỗi kết nối của API được sửa vội thành `host=10.20.0.15 sslmode=require`, API chạy bình thường, mọi người yên tâm vì «đã có SSL». Ba tuần sau, khi rà cấu hình, có người hỏi: nếu một máy khác trong mạng giả làm `10.20.0.15`, API có phát hiện không? Câu trả lời là **không** — `require` mã hoá đường truyền nhưng không kiểm server là ai (trừ một ngoại lệ ở mục 6). API sẽ gửi tên đăng nhập, rồi dữ liệu chuyến của khách, cho bất kỳ ai đứng giữa; server giả còn có thể đòi mật khẩu dạng thô (cleartext), và libpq mặc định chấp nhận phương thức server đòi — trừ khi client đặt `require_auth` (có từ PostgreSQL 16, lab không thử).

PG-041 trả lời «tài khoản được làm gì». Bài này trả lời ba câu độc lập:

1. **Ai được phép thử đăng nhập, từ đâu, bằng phương thức nào?** — `pg_hba.conf`.
2. **Làm sao chứng minh mình đúng là tài khoản đó?** — mật khẩu lưu và kiểm bằng SCRAM.
3. **Đường truyền có được bảo vệ, và client có đang nói chuyện với đúng server?** — TLS cùng tham số `sslmode` phía client.

Kết nối thành công không cho biết lớp nào đang làm việc, nên lab chứng minh bằng **các lần bị từ chối**.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-042 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-role-grant-least-privilege">PG-041 · Role, GRANT và quyền tối thiểu</a>. Lab: <code>examples/toi-noi-postgresql/p05</code> (PostgreSQL 18.1 thật; phần «PG-042» trong <code>run.mjs</code>, kết quả <code>OUTPUT.txt</code>). Cần OpenSSL hỗ trợ <code>req -addext</code> để tạo chứng chỉ tạm. Sau bài này, bạn kiểm được kết nối hợp lệ và các kết nối phải bị từ chối, đọc được dòng HBA đã khớp trong log server, và chọn đúng <code>sslmode</code>.</p>
</div>

## 1. Dữ liệu và cách chạy

Lab dùng lại cluster tạm và các role của PG-041 (`tn_owner`, `tn_app`, `tn_report`, `tn_monitor`; 100 chuyến, mỗi chuyến 30.000 đ). Chạy từ `examples/toi-noi-postgresql`:

```bash
npm run p05
```

Binary không ở vị trí mặc định thì đặt `PG_BIN` và `OPENSSL_BIN` như README của lab. Cluster chỉ nghe localhost và bị xoá khi runner kết thúc; mật khẩu, port và thư mục dữ liệu được che trong `OUTPUT.txt` bằng `<PASSWORD>`, `<PORT>`, `<DATA>`.

## 2. Thuật ngữ

- **Xác thực (authentication)**: chứng minh «tôi là `tn_app`». Khác với **phân quyền (authorization)** của PG-041: «`tn_app` được làm gì».
- **`pg_hba.conf`** (HBA — host-based authentication): tệp cấu hình phía server; mỗi dòng ghép loại kết nối, database, user, địa chỉ với một phương thức xác thực.
- **SCRAM-SHA-256**: xác thực mật khẩu kiểu thử thách–phản hồi (challenge–response): mật khẩu không đi qua mạng, server chỉ lưu một **verifier** (giá trị kiểm chứng) suy ra từ mật khẩu.
- **TLS** (Transport Layer Security; tên cũ là SSL, nên PostgreSQL vẫn dùng chữ `ssl` trong tham số): giao thức mã hoá đường truyền, cho phép một bên chứng minh danh tính bằng **chứng chỉ**.
- **Chứng chỉ (certificate)**: tài liệu gắn một khoá công khai với một danh tính (tên máy), được ký bởi một **CA** (certificate authority — nhà phát hành). Client tin chứng chỉ nếu chuỗi chữ ký dẫn về một **trust anchor** (gốc tin cậy) mà client đã có sẵn.
- **SAN** (Subject Alternative Name): trường trong chứng chỉ liệt kê các tên máy mà chứng chỉ hợp lệ cho.

## 3. Lớp thứ nhất: `pg_hba.conf` và luật khớp dòng đầu

Khi một kết nối tới, server đọc `pg_hba.conf` **từ trên xuống** và dùng **dòng đầu tiên** khớp với loại kết nối, database, user và địa chỉ client. Dòng đó quyết định phương thức xác thực. Nếu xác thực theo dòng đó thất bại, server **không** thử tiếp các dòng bên dưới — kết nối bị từ chối. Nếu không có dòng nào khớp, kết nối cũng bị từ chối. [Tài liệu pg_hba.conf](https://www.postgresql.org/docs/18/auth-pg-hba-conf.html).

Các giá trị cột `type` hay gặp:

| type | Khớp với |
|---|---|
| `local` | Kết nối qua Unix-domain socket |
| `host` | Kết nối TCP/IP, có hoặc không có TLS |
| `hostssl` | Chỉ kết nối TCP/IP **dùng TLS** |
| `hostnossl` | Chỉ kết nối TCP/IP **không** dùng TLS |

Runner ghi đè `pg_hba.conf` của cluster tạm, bật `ssl=on`, khởi động lại cluster rồi đọc view `pg_hba_file_rules`:

```text
--- 042 pg_hba_file_rules
   rule |  type   |   database    | user_name  |  address  |  auth_method  | error
  ------+---------+---------------+------------+-----------+---------------+-------
      1 | local   | {all}         | {postgres} | NULL      | trust         | NULL
      2 | host    | {all}         | {postgres} | 127.0.0.1 | trust         | NULL
      3 | host    | {all}         | {postgres} | ::1       | trust         | NULL
      4 | hostssl | {all}         | {tn_app}   | 127.0.0.1 | scram-sha-256 | NULL
      5 | host    | {all}         | {all}      | 127.0.0.1 | reject        | NULL
      6 | host    | {all}         | {all}      | ::1       | reject        | NULL
      7 | host    | {replication} | {postgres} | 127.0.0.1 | trust         | NULL
  (7 rows)
```

(Bảng chỉ có địa chỉ, không có cột netmask; trong tệp, dòng 4 viết là `127.0.0.1/32`.) Đọc theo thứ tự:

- **Dòng 1–3 và 7 — `trust` cho `postgres`**: không cần mật khẩu. Đây là **control plane của lab**: runner cần điều khiển cluster tạm mà không quản lý thêm bí mật. Cluster chỉ nghe loopback và chỉ sống trong lúc chạy lab, nhưng bất kỳ tiến trình nào trên cùng máy, kể cả của người dùng khác, đều có thể đăng nhập làm superuser trong khoảng đó. **Đây không phải cấu hình production**: đừng chạy lab trên máy có người dùng không tin cậy, đừng chép các dòng này đi đâu. Trong triển khai, tài khoản quản trị và replication cũng phải xác thực (SCRAM, chứng chỉ client, hoặc `peer` cho socket cục bộ).
- **Dòng 4 — `hostssl … tn_app … scram-sha-256`**: `tn_app` chỉ được thử đăng nhập qua TLS, từ 127.0.0.1, bằng SCRAM.
- **Dòng 5–6 — `reject`**: mọi kết nối TCP khác từ loopback bị từ chối **tường minh**. `tn_app` không dùng TLS thì trượt dòng 4 (`hostssl`), rơi xuống dòng 5 và bị chặn.
- **Dòng 7 — `replication`**: nằm sau dòng `reject` mà vẫn có tác dụng, vì từ khoá `all` ở cột database **không** khớp kết nối physical replication. Kết nối replication chỉ khớp dòng ghi `replication` (lab dùng nó cho PG-048).

Cột `error` báo dòng viết sai cú pháp. `pg_hba_file_rules` đọc **tệp hiện có trên đĩa**, không phải cấu hình đang nạp, nên dùng được để kiểm thay đổi **trước** khi reload. Thay đổi có hiệu lực sau reload (`SELECT pg_reload_conf();`) và chỉ áp cho kết nối **mới** — khi sửa HBA trên máy thật, giữ một phiên quản trị đang mở để còn sửa lại nếu tự khoá mình ngoài cửa.

Sai lầm kinh điển: dòng rộng đặt **trên** dòng hẹp. Nếu dòng 5 nằm trên dòng 4, `tn_app` bị từ chối dù dùng TLS và mật khẩu đúng. Lab không chạy cấu hình đảo này; đó là bài tập 2.

## 4. Lớp thứ hai: SCRAM và verifier

Runner đặt mật khẩu ngẫu nhiên 48 ký tự hex cho `tn_app`:

```sql
SET password_encryption = 'scram-sha-256';
ALTER ROLE tn_app PASSWORD '<PASSWORD>';
```

`password_encryption` quyết định mật khẩu được biến đổi thế nào **trước khi lưu**. Giá trị mặc định đã là `scram-sha-256` từ PostgreSQL 14; lab đặt tường minh để không phụ thuộc cấu hình. Rồi kiểm phần đầu của giá trị lưu trong `pg_authid`:

```sql
SELECT split_part(rolpassword, '$', 1) FROM pg_authid WHERE rolname = 'tn_app';
```

```text
PASS 042 stored verifier is SCRAM: SCRAM-SHA-256
```

Lab chỉ in phần tiền tố. Theo [tài liệu Password Authentication](https://www.postgresql.org/docs/18/auth-password.html), giá trị đầy đủ có dạng `SCRAM-SHA-256$<số vòng lặp>:<salt>$<StoredKey>:<ServerKey>`: salt ngẫu nhiên cho mỗi mật khẩu, số vòng lặp làm chậm việc dò mật khẩu (mặc định 4096, chỉnh bằng `scram_iterations` từ PG16). Server không giữ mật khẩu gốc. Khi đăng nhập, client và server trao đổi thông điệp để chứng minh client biết mật khẩu mà mật khẩu không đi qua mạng.

Vì sao không dùng `md5`? Giá trị md5 lưu trong catalog đủ để đăng nhập — ai lấy được nó (từ backup, từ dump `pg_authid`) thì đăng nhập được mà không cần biết mật khẩu. SCRAM khắc phục điều này. PostgreSQL 18 đã đánh dấu xác thực md5 là deprecated (lỗi thời, sẽ bị bỏ). Nếu `pg_hba.conf` ghi `md5` nhưng mật khẩu đã lưu dạng SCRAM, server tự dùng SCRAM — nên lộ trình chuyển đổi là đổi mật khẩu sang SCRAM trước, rồi mới sửa HBA thành `scram-sha-256`.

SCRAM bảo vệ **mật khẩu**; nó không bảo vệ **dữ liệu** đi sau đó. Đó là việc của TLS.

## 5. Lớp thứ ba: TLS phía server

Runner tạo một chứng chỉ **tự ký** (self-signed — chính nó đóng vai CA) sống hai ngày, tên `localhost`:

```bash
openssl req -x509 -newkey rsa:2048 -nodes -keyout server.key -out server.crt \
  -days 2 -subj /CN=localhost -addext subjectAltName=DNS:localhost
```

Hai tệp nằm trong thư mục dữ liệu của cluster tạm (`server.crt`, `server.key` là tên mặc định server tìm khi `ssl=on`); private key có quyền `0600` và bị xoá cùng cluster. **Không key nào vào Git.** Khi client cần tin chứng chỉ này, runner trao cho nó chính `server.crt` làm trust anchor (`sslrootcert`). Đó là cách hợp lệ trong lab vì ta kiểm soát cả hai đầu; production dùng CA của tổ chức hoặc của nhà cung cấp cloud, có quy trình xoay vòng (rotation) và theo dõi hạn dùng. [Secure TCP/IP Connections with SSL](https://www.postgresql.org/docs/18/ssl-tcp.html).

Server bật TLS không buộc mọi kết nối dùng TLS; chính dòng `hostssl` ở HBA mới **bắt buộc** `tn_app` dùng TLS.

## 6. Client quyết định có kiểm server hay không: `sslmode`

TLS chỉ chống được kẻ đứng giữa nếu **client kiểm chứng chỉ**. Việc đó do tham số `sslmode` của libpq (thư viện client C của PostgreSQL; psql và nhiều driver dựa trên nó) quyết định. Theo [tài liệu libpq SSL Support](https://www.postgresql.org/docs/18/libpq-ssl.html):

| sslmode | Mã hoá | Kiểm chuỗi chứng chỉ | Kiểm tên máy |
|---|---|---|---|
| `disable` | Không | — | — |
| `allow` | Chỉ khi server đòi | Không | Không |
| `prefer` (mặc định) | Nếu server hỗ trợ | Không | Không |
| `require` | Có | Không (\*) | Không |
| `verify-ca` | Có | Có | **Không** |
| `verify-full` | Có | Có | **Có** |

(\*) Ngoại lệ vì tương thích ngược: nếu có tệp root CA (`sslrootcert` hoặc mặc định `~/.postgresql/root.crt`), `require` hành xử như `verify-ca`. Không nên dựa vào ngoại lệ này — chuỗi kết nối sẽ đổi nghĩa tuỳ máy có tệp đó hay không. Ghi thẳng `verify-full`.

Bảng trên mô tả libpq. Driver không dựa trên libpq (JDBC, Npgsql, pgx…) có tham số và mặc định riêng; lab không kiểm chúng.

Chuỗi kết nối của lab:

```text
host=localhost hostaddr=127.0.0.1 port=<PORT> user=tn_app dbname=postgres
sslmode=verify-full sslrootcert=<CERT> connect_timeout=3
```

`hostaddr` là địa chỉ IP để kết nối (không tra DNS), còn `host` là **tên được so với chứng chỉ** khi `verify-full` — nhờ vậy lab thử được «đúng máy, sai tên» mà không sửa DNS. Mật khẩu đi qua biến môi trường `PGPASSWORD`, đủ cho lab ngắn hạn; ứng dụng thật đọc từ kho bí mật.

Kết nối hợp lệ, rồi hỏi server xem phiên này có dùng TLS:

```sql
SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid();
```

```text
PASS 042 TLS verify-full: t
--- 042 pg_stat_ssl of tn_app session
  ssl=true version=TLSv1.3 bits=256 user=tn_app
```

Phiên dùng TLS 1.3, thuật toán mã hoá với khoá 256 bit (cột `bits`), đăng nhập bằng `tn_app`. Nhưng `pg_stat_ssl` là góc nhìn của server: nó biết đường truyền được mã hoá, không biết client có kiểm chứng chỉ hay không — client `sslmode=require` cũng cho `ssl = t`. Muốn chứng minh client kiểm tên thì phải cho nó gặp tên sai.

## 7. Ma trận bị từ chối

Runner chạy bốn kết nối nữa, mỗi lần đổi đúng **một** yếu tố so với kết nối hợp lệ, và khẳng định cả mã thoát khác 0 lẫn nội dung lỗi:

| Ca | Đổi gì | Kết quả | Lớp chặn |
|---|---|---|---|
| A | `sslmode=disable` | Bị từ chối | HBA (dòng 5) |
| B | Sai mật khẩu, vẫn `verify-full` | Bị từ chối | SCRAM (dòng 4) |
| C | `host=wrong.invalid`, vẫn `verify-full` | Bị từ chối | Client kiểm tên máy |
| D | `host=wrong.invalid`, `sslmode=verify-ca` | **Kết nối được** | Không lớp nào |

**Ca A — plaintext:**

```text
PASS 042 rejects plaintext
--- 042 client error: plaintext
  psql: error: connection to server at "127.0.0.1", port <PORT> failed: FATAL:  pg_hba.conf rejects connection for host "127.0.0.1", user "tn_app", database "postgres", no encryption
```

`rejects connection` là thông báo của một dòng `reject` đã khớp; `no encryption` cho biết kết nối không dùng TLS nên dòng `hostssl` bị bỏ qua. Nếu không có dòng 5–6 và không dòng nào khớp, thông báo sẽ là `no pg_hba.conf entry for …` — hai thông báo khác nhau chỉ ra hai nguyên nhân khác nhau khi chẩn đoán.

**Ca B — sai mật khẩu:**

```text
PASS 042 rejects wrong password
--- 042 client error: wrong password
  psql: error: connection to server at "127.0.0.1", port <PORT> failed: FATAL:  password authentication failed for user "tn_app"
```

Kết nối qua TLS, khớp dòng 4; SCRAM thất bại thì dừng — server không xuống dòng 5 thử tiếp. Chi tiết nằm ở log server (mục 8).

**Ca C — sai tên máy với `verify-full`:**

```text
PASS 042 rejects wrong hostname
--- 042 client error: wrong hostname
  psql: error: connection to server at "127.0.0.1", port <PORT> failed: server certificate for "localhost" does not match host name "wrong.invalid"
```

Đúng IP, đúng chứng chỉ, đúng mật khẩu, nhưng không có `FATAL:` của server: chính **client** từ chối, vì chứng chỉ chỉ hợp lệ cho `localhost`. Đây là kịch bản kẻ đứng giữa cầm chứng chỉ hợp lệ của **một máy khác** cùng CA.

**Ca D — cùng tên sai, nhưng `verify-ca`:**

```text
PASS 042 verify-ca accepts wrong hostname: t
```

`t` là kết quả của `SELECT ssl FROM pg_stat_ssl …` trong phiên đã kết nối thành công. `verify-ca` chỉ kiểm chứng chỉ được ký bởi CA tin cậy, không kiểm tên. Nếu tổ chức dùng một CA nội bộ cấp chứng chỉ cho hàng trăm máy, `verify-ca` chấp nhận **bất kỳ** máy nào trong số đó. Đây là lý do bài khuyên `verify-full` cho kết nối từ ứng dụng; `verify-ca` chỉ hợp lý khi CA được dùng riêng cho đúng một nhóm server đáng tin như nhau.

## 8. Log server cho biết dòng HBA nào đã khớp

Client chỉ nhận thông báo tối thiểu. Runner lọc log server lấy các dòng `FATAL` và `DETAIL: Connection matched`:

```text
--- 042 server log: rejected attempts
  FATAL:  pg_hba.conf rejects connection for host "127.0.0.1", user "tn_app", database "postgres", no encryption
  FATAL:  password authentication failed for user "tn_app"
  DETAIL:  Connection matched file "<DATA>/pg_hba.conf" line 4: "hostssl all tn_app 127.0.0.1/32 scram-sha-256"
```

(Prefix thời gian và PID đã được runner bỏ.) Dòng `DETAIL` sau lần sai mật khẩu cho biết đúng **tệp, số dòng và nội dung dòng HBA** đã khớp — khi một tài khoản bị từ chối bất ngờ, đây là thông tin đầu tiên cần đọc: kết nối có rơi vào dòng bạn nghĩ không?

Hai điều cần đọc đúng trong excerpt này:

- Ca A (bị dòng `reject` chặn) chỉ có dòng `FATAL`, không kèm `DETAIL` số dòng trong output lab.
- Ca C không có dòng `FATAL` nào: lỗi phát sinh ở client trong lúc bắt tay TLS, trước khi tới bước xác thực. Bộ lọc của runner chỉ giữ `FATAL` và `DETAIL: Connection matched`, nên những dòng mức `LOG` khác mà server có thể ghi về lần bắt tay bị huỷ không có trong excerpt; bài không khẳng định nội dung của chúng. Ca D thành công nên không sinh lỗi.

Hệ quả vận hành: **lỗi xác minh server chỉ thấy ở phía client** — log PostgreSQL không bao giờ cho biết một API đang bị chuyển hướng tới server giả. Phải log lỗi kết nối ở ứng dụng.

## 9. Những gì lab chưa kiểm

- **Chứng chỉ client** (xác thực hai chiều: tuỳ chọn `clientcert=verify-full` trên dòng HBA, hoặc phương thức `cert`). Client trong lab chứng minh bằng mật khẩu.
- **Channel binding**: tham số libpq `channel_binding=require` buộc SCRAM gắn với phiên TLS cụ thể (biến thể `SCRAM-SHA-256-PLUS`), để kẻ đứng giữa không chuyển tiếp được phiên xác thực. Lab không bật.
- **Phiên bản TLS tối thiểu**: server có `ssl_min_protocol_version`, mặc định `TLSv1.2`. Lab chỉ quan sát TLS 1.3 tự thương lượng, không kiểm việc từ chối TLS cũ.
- **Xoay vòng chứng chỉ, thu hồi (CRL), hết hạn**: chứng chỉ lab sống hai ngày và không bao giờ được thay trong lúc chạy.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| «Có SSL rồi» nhưng không phát hiện server giả | `sslmode=require` hoặc `prefer`: mã hoá mà không kiểm danh tính | `verify-full` + `sslrootcert` trỏ tới CA đúng |
| Dùng `verify-ca`, tưởng đã kiểm tên | `verify-ca` chỉ kiểm chuỗi CA | `verify-full`; chứng chỉ có SAN đúng tên client dùng |
| Thêm dòng HBA mới mà không có tác dụng | Một dòng rộng hơn phía trên khớp trước | Đặt dòng hẹp lên trên; đọc `DETAIL: Connection matched` |
| `no pg_hba.conf entry for …` | Không dòng nào khớp (sai user, database, địa chỉ hoặc thiếu TLS cho `hostssl`) | Đối chiếu bốn trường với `pg_hba_file_rules` |
| `verify-full` lỗi tên khi kết nối bằng IP | Chứng chỉ chỉ có tên DNS, client dùng IP làm `host` | Dùng tên trong SAN làm `host`; IP vào `hostaddr` hoặc thêm IP vào SAN |
| Đổi `pg_hba.conf` sang `scram-sha-256` rồi tài khoản cũ không vào được | Mật khẩu vẫn lưu dạng md5 | Đặt lại mật khẩu với `password_encryption = scram-sha-256` trước |
| Để `trust` «tạm thời» | Ai tới được dòng đó là đăng nhập được | Chỉ cho lab cô lập |

## 11. Đánh đổi

- **`verify-full` và vận hành chứng chỉ.** Kiểm tên máy buộc chứng chỉ phải đúng với mọi tên client dùng (tên DNS, tên qua load balancer, tên qua pooler). Đổi kiến trúc mạng là đổi chứng chỉ. Cái giá này đáng trả: không kiểm tên thì TLS không chống được kẻ đứng giữa.
- **Mật khẩu hay chứng chỉ client.** Chứng chỉ client không có bí mật để gõ nhầm hay dán vào log, nhưng cần hạ tầng cấp và thu hồi. Mật khẩu SCRAM đơn giản hơn, đổi lại cần kho bí mật và xoay vòng mật khẩu.
- **HBA hẹp hay rộng.** Dòng ghi đúng user, database, dải địa chỉ thì an toàn hơn, nhưng tệp dài ra và thứ tự trở nên quan trọng; dòng `reject` tường minh ở cuối cho lỗi rõ ràng nhưng mọi dòng hợp lệ phải nằm trên nó.
- **Chi phí TLS.** Bắt tay TLS thêm độ trễ cho mỗi kết nối mới; connection pool (PG-047) trả chi phí đó một lần cho nhiều request. Lab không đo nó.

## 12. Bài tập

**Bài 1 — trust anchor sai.** Tạo một chứng chỉ tự ký thứ hai (cùng `CN=localhost`) nhưng **không** cài cho server. Trỏ `sslrootcert` của client vào chứng chỉ này, giữ `verify-full`.

<details><summary><b>Gợi ý và tiêu chí chấm</b></summary>
<p>Lab chưa chạy ca này. Kết nối phải thất bại ở client, trong lúc bắt tay TLS, vì chứng chỉ server không được ký bởi trust anchor client đang có — dù tên khớp. Lời giải đạt khi chứng minh được ba điều: mã thoát khác 0; thông báo lỗi là lỗi xác minh chứng chỉ (không phải lỗi mật khẩu hay HBA); log server không có dòng <code>password authentication failed</code> cho lần này, tức là mật khẩu chưa từng được dùng. Không được «sửa» bằng cách hạ <code>sslmode</code>.</p>
</details>

**Bài 2 — thứ tự HBA.** Trên cluster thử của bạn, đảo dòng 4 và dòng 5 của `pg_hba.conf` lab, reload, rồi chạy lại kết nối hợp lệ của mục 6.

<details><summary><b>Gợi ý và tiêu chí chấm</b></summary>
<p>Lab không chạy cấu hình đảo này. Theo luật khớp dòng đầu, kết nối TLS của <code>tn_app</code> từ 127.0.0.1 giờ khớp dòng <code>host all all 127.0.0.1/32 reject</code> trước. Lời giải phải nộp: output <code>pg_hba_file_rules</code> sau khi đảo, thông báo lỗi client và dòng log server; rồi đảo lại và chứng minh kết nối hợp lệ trở lại (<code>ssl = t</code>).</p>
</details>

**Bài 3 — xoay vòng chứng chỉ.** Tới Nơi chuyển sang một CA mới. Viết thứ tự các bước để thay chứng chỉ server mà không có lúc nào API phải hạ `sslmode`, và kiểm chứng sau mỗi bước.

<details><summary><b>Gợi ý và tiêu chí chấm</b></summary>
<p>Lab không chạy rotation. Một thứ tự hợp lý: (1) thêm CA mới vào trust store của client <b>bên cạnh</b> CA cũ (tệp <code>sslrootcert</code> có thể chứa nhiều chứng chỉ) và triển khai client; (2) thay <code>server.crt</code>/<code>server.key</code> bằng chứng chỉ CA mới ký, cùng SAN, rồi reload server (cấu hình SSL nạp lại được khi reload; chỉ kết nối mới dùng chứng chỉ mới); (3) xác minh kết nối mới <code>verify-full</code> thành công; (4) gỡ CA cũ khỏi client và chứng minh một server còn dùng chứng chỉ cũ bị từ chối. Lời giải phải nộp ma trận client (CA cũ / cả hai / CA mới) × server (chứng chỉ cũ / mới): kết quả kết nối và lớp chặn. Đảo bước 1 và 2 là lỗi: client chỉ biết CA cũ sẽ từ chối server mới.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · The pg_hba.conf File](https://www.postgresql.org/docs/18/auth-pg-hba-conf.html): cú pháp, thứ tự khớp, từ khoá `replication`.
- [PostgreSQL 18 · Password Authentication](https://www.postgresql.org/docs/18/auth-password.html): SCRAM, md5 và cách chuyển đổi.
- [PostgreSQL 18 · Secure TCP/IP Connections with SSL](https://www.postgresql.org/docs/18/ssl-tcp.html): cấu hình TLS phía server, chứng chỉ client.
- [PostgreSQL 18 · libpq SSL Support](https://www.postgresql.org/docs/18/libpq-ssl.html): bảng `sslmode`, các mối đe doạ mỗi mode chống được.
- [PostgreSQL 18 · Connection Strings](https://www.postgresql.org/docs/18/libpq-connect.html): `host`, `hostaddr`, `sslrootcert`, `channel_binding`.
- [PostgreSQL 18 · pg_hba_file_rules](https://www.postgresql.org/docs/18/view-pg-hba-file-rules.html) và [pg_stat_ssl](https://www.postgresql.org/docs/18/monitoring-stats.html).
- Lab: phần PG-042 trong `examples/toi-noi-postgresql/p05/run.mjs`, kết quả `p05/OUTPUT.txt`, giới hạn và cách chạy trong `p05/README.md`, xử lý «TLS không lên» trong `p05/RUNBOOK.md`.

Bài tiếp theo: [PG-043 · Quan sát phiên làm việc và truy vấn chậm](/articles/postgresql-pg-stat-activity-quan-sat) — `tn_app` đã đăng nhập đúng cách; giờ xem nó đang làm gì và đang chờ ai.

---
slug: postgresql-la-gi-va-vai-tro-trong-toi-noi
title: "PostgreSQL là gì và nằm ở đâu trong Tới Nơi?"
summary: "Khách gọi tổng đài hỏi chuyến 100004 có thật không và bị tính bao nhiêu tiền: câu trả lời phải đến từ đâu? Bài này phân biệt DBMS, database, schema, bảng và xác định PostgreSQL giữ trách nhiệm gì trong Tới Nơi."
category: postgresql
level: beginner
tags: [postgresql, pg-001, dbms, schema, client-server, toi-noi]
language: vi
---

Trưa 02/11, anh Võ Minh Đức đặt một suất mì Quảng qua Tới Nơi ở Đà Nẵng. Sáng hôm sau anh gọi tổng đài: "Hôm qua tôi bị trừ 55.000 đ, chuyến đó có thật không, ai giao?" Nhân viên chăm sóc khách hàng mở màn hình tra cứu. Ứng dụng trên điện thoại của anh Đức có lịch sử chuyến, dịch vụ đặt chuyến có log, có thể còn một bản sao trong cache để hiển thị nhanh. Nếu ba nơi nói ba điều khác nhau, nhân viên tin nơi nào?

Hệ thống chỉ trả lời được khi đã quyết định trước: **dữ liệu chuyến có đúng một nơi giữ bản gốc**, mọi bản sao đối chiếu về đó. Ở giai đoạn đầu của Tới Nơi, nơi ấy là PostgreSQL. Bài này giải thích PostgreSQL là gì, nó gồm những lớp nào (DBMS, database, schema, bảng) và vì sao dữ liệu chuyến không được để trong file hay chỉ trong bộ nhớ của ứng dụng.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-001 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>. Không cần biết SQL. Lab: <code>examples/toi-noi-postgresql/p01</code>. Sau bài này, bạn phân biệt được DBMS, database, schema, bảng và nói rõ trách nhiệm lưu dữ liệu nằm ở đâu trong Tới Nơi.</p>
</div>

## 1. Nguồn dữ liệu gốc của Tới Nơi

**Source of truth** (nguồn dữ liệu gốc, còn gọi là *system of record*) là nơi giữ bản gốc của một loại dữ liệu: khi các nơi khác lệch nhau, bản ở đây thắng. Một hệ thống có thể có nhiều nguồn dữ liệu gốc cho nhiều loại dữ liệu khác nhau, nhưng mỗi loại chỉ nên có một.

Với Tới Nơi, những dữ liệu sau không được phép mất hoặc sai, vì chúng là tiền và là lời hứa với khách, tài xế, quán:

| Dữ liệu | Ví dụ trong lab | Hậu quả nếu sai hoặc mất |
|---|---|---|
| Khách hàng | số điện thoại `0900000005` thuộc đúng một khách | khách đăng nhập nhầm tài khoản người khác |
| Tài xế | mã `8005`, thành phố, loại xe | điều phối sai người, trả thu nhập sai người |
| Chuyến | chuyến `100004`: đồ ăn, 55.000 đ, `completed` | tính sai tiền, không đối soát được với quán |

Hồ sơ [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) đã chốt: **giai đoạn đầu dùng PostgreSQL làm database chủ đạo**. Khi nói "chuyến 100004 có cước 55.000 đ", ta hiểu đó là giá trị trong bảng `trips` của PostgreSQL; ứng dụng, cache, báo cáo đọc hoặc ghi qua đó, không giữ bản gốc.

## 2. Vì sao không để dữ liệu chuyến trong file hay trong bộ nhớ ứng dụng?

Trả lời rõ câu này sẽ cho ta danh sách việc một hệ quản trị cơ sở dữ liệu phải làm. Theo hồ sơ, lúc cao điểm Tới Nơi tạo khoảng **70 chuyến/giây** và sửa bảng `trips` khoảng **560 lần/giây**. Thử đặt dữ liệu đó vào ba chỗ "dễ làm":

| Phương án | Chuyện xảy ra với Tới Nơi |
|---|---|
| Một file JSON/CSV trên máy chủ | Nhiều tiến trình cùng ghi một file thì ai chờ ai? Máy sập giữa lúc ghi thì file còn đọc được không? Tìm "chuyến của khách X tháng này" phải đọc cả file. |
| Biến trong bộ nhớ của dịch vụ | Mất sạch khi dịch vụ khởi động lại. Chạy hai bản dịch vụ thì mỗi bản thấy một tập dữ liệu khác nhau. |
| Chỉ một kho key-value trong bộ nhớ (ví dụ Redis) | Đọc ghi rất nhanh theo khoá, nhưng không có khái niệm khoá ngoại hay CHECK để chặn "chuyến của khách không tồn tại"; độ bền phụ thuộc cấu hình lưu xuống đĩa. Hợp làm cache, không hợp làm bản gốc của tiền. |

Từ bảng trên, ta rút ra bốn yêu cầu với nơi giữ bản gốc:

1. **Bền** (*durability*): đã báo ghi thành công thì sập máy không mất.
2. **Nhiều bên cùng dùng an toàn**: hàng trăm yêu cầu đồng thời không ghi đè nhau.
3. **Tự giữ luật dữ liệu**: số điện thoại không trùng, cước không âm, chuyến phải thuộc một khách có thật.
4. **Hỏi được nhiều kiểu**: theo khách, theo tài xế, theo thành phố và khoảng thời gian, mà không phải viết chương trình đọc file riêng cho mỗi câu hỏi.

Ghi log ra file hay đặt cache trước database vẫn là chuyện bình thường; điểm quan trọng là phân vai: log để điều tra, cache để đọc nhanh, **PostgreSQL giữ bản gốc**. Series lần lượt học cách PostgreSQL đáp ứng bốn yêu cầu: luật dữ liệu ở Phần 1, truy vấn ở Phần 2, lưu trữ và index ở Phần 3, xử lý đồng thời ở Phần 4, độ bền và phục hồi ở Phần 5.

## 3. DBMS, database, schema, bảng

Bốn từ này hay bị dùng lẫn. Trong PostgreSQL chúng là bốn lớp lồng nhau.

**DBMS** (*Database Management System*, hệ quản trị cơ sở dữ liệu) là phần mềm quản lý dữ liệu thay cho ứng dụng: lưu xuống đĩa, nhận câu lệnh, kiểm tra luật, điều phối nhiều người dùng. PostgreSQL là một DBMS. Tài liệu chính thức tự giới thiệu nó là một hệ quản trị cơ sở dữ liệu quan hệ–đối tượng (*object-relational*), phát triển từ dự án POSTGRES ở Đại học California, Berkeley, và phát hành theo giấy phép mã nguồn mở cho phép dùng miễn phí cả cho mục đích thương mại.

Một server PostgreSQL đang chạy quản lý một **database cluster** (cụm database). Chữ *cluster* ở đây không có nghĩa là nhiều máy: tài liệu định nghĩa nó là tập các database do **một** server quản lý, và trên đĩa là **một thư mục dữ liệu** (*data directory*). Bài [PG-002](/articles/postgresql-cai-dat-lab-psql-ket-noi) sẽ cho bạn nhìn tận mắt thư mục đó.

Bên trong cụm:

- **Database** (cơ sở dữ liệu): một tập dữ liệu có tên. Mỗi kết nối chỉ làm việc với **đúng một** database đã chọn lúc kết nối.
- **Schema** (lược đồ, ở đây hiểu là "thư mục tên"): nhóm đối tượng trong một database. Tài liệu viết gọn: "A database contains one or more named schemas, which in turn contain tables." Hai bảng trùng tên được phép nếu nằm ở hai schema khác nhau.
- **Table** (bảng): nơi chứa dữ liệu thật, gồm các **cột** (*column*) có kiểu xác định và các **dòng** (*row*). Một dòng của `trips` là một chuyến. Bảng, dòng, cột và khoá được học kỹ ở [PG-003](/articles/postgresql-bang-dong-cot-primary-foreign-key).

Chữ "schema" còn có nghĩa rộng hơn là **thiết kế cấu trúc dữ liệu** (bảng, cột, ràng buộc) — như trong "schema chuyến xe". Còn trong `toinoi.trips`, `toinoi` là schema theo nghĩa đối tượng chứa bảng.

Bốn lớp trong lab P01, sau khi chạy `00-reset`, `01-schema`, `02-fixture-valid` (số trong ngoặc là số dòng):

```text
Server postgres ── database cluster (một thư mục dữ liệu trên đĩa)
├── database postgres, template0, …      ← database khác cùng cụm
└── database đang kết nối (PGlite: template1 · Docker: toinoi)
    ├── schema pg_catalog, information_schema, public
    └── schema toinoi
        ├── cities (3)    customers (6)   drivers (6)
        └── merchants (3) trips (15)      trip_events (0)
```

`trip_events` chưa có dòng vì fixture chỉ nạp trạng thái cuối của 15 chuyến.

### Nhìn bốn lớp bằng chính lab

Các câu dưới đây chạy thật trên lab (sau `00`, `01`, `02`, runner PGlite — PostgreSQL 17.5). Chưa cần hiểu cú pháp, chỉ đọc kết quả; cách tự chạy ở PG-002.

```sql
SELECT current_database() AS database_hien_tai, current_user AS nguoi_dung, current_schema() AS schema_hien_tai;
SELECT datname FROM pg_database ORDER BY datname;
```

```text
 database_hien_tai | nguoi_dung | schema_hien_tai
-------------------+------------+-----------------
 template1         | postgres   | toinoi
(1 row)
  datname
-----------
 postgres
 template0
 template1
(3 rows)
```

Cụm có ba database; PGlite làm việc thẳng trong `template1`, còn server Docker của lab dùng database `toinoi` (PG-002). Lab chỉ phụ thuộc schema `toinoi`, không phụ thuộc tên database.

```sql
SELECT schema_name FROM information_schema.schemata ORDER BY schema_name;
SELECT table_name FROM information_schema.tables WHERE table_schema = 'toinoi' ORDER BY table_name;
```

```text
    schema_name
--------------------
 information_schema
 pg_catalog
 pg_toast
 public
 toinoi
(5 rows)
 table_name
-------------
 cities
 customers
 drivers
 merchants
 trip_events
 trips
(6 rows)
```

`pg_catalog` và `information_schema` là nơi PostgreSQL tự mô tả các đối tượng của nó — ta vừa đọc chúng để liệt kê schema và bảng. `pg_toast` là schema nội bộ (gặp lại ở Phần 3), `public` có sẵn trong mọi database mới, `toinoi` do `00-reset.sql` tạo.

## 4. Client–server: ai nói chuyện với PostgreSQL?

PostgreSQL làm việc theo mô hình **client–server** (máy khách – máy chủ). Tài liệu mô tả: chương trình server tên `postgres` quản lý các file dữ liệu, nhận kết nối từ client và thực hiện thao tác thay cho client. Client có thể nằm trên máy khác và nói chuyện qua mạng TCP/IP. Mỗi kết nối mới được server phục vụ bằng một tiến trình riêng, nên nhiều client làm việc cùng lúc được.

Ánh xạ vào Tới Nơi:

| Thành phần | Vai trò với PostgreSQL | Ghi chú |
|---|---|---|
| Dịch vụ đặt chuyến (C# theo hồ sơ) | client | mở kết nối, gửi câu lệnh ghi chuyến, nhận kết quả |
| `psql` trên máy người vận hành | client | công cụ dòng lệnh chính thức, dùng từ PG-002 |
| Tiến trình `postgres` | server | giữ dữ liệu, kiểm tra luật, trả kết quả |
| Ứng dụng trên điện thoại khách | **không** kết nối trực tiếp | đi qua API của dịch vụ |

Điện thoại khách không bao giờ nói chuyện thẳng với PostgreSQL. Hồ sơ Tới Nơi còn đặt ra luật: service sở hữu dữ liệu của mình, không để service khác ghi thẳng vào bảng nội bộ; cấp quyền cho từng client là chủ đề của Phần 5.

Ngoại lệ của lab P01: runner dùng **PGlite** — mã nguồn PostgreSQL 17.5 chạy ngay trong tiến trình Node, **một** kết nối, không qua mạng. Đủ để học luật dữ liệu và SQL, chưa đủ để mô phỏng nhiều client đồng thời; Phần 4 sẽ dùng server thật.

## 5. Schema `toinoi` trong lab

Mọi bảng của lab nằm trong schema riêng `toinoi`. File `00-reset.sql` làm đúng ba việc chính:

```sql
DROP SCHEMA IF EXISTS toinoi CASCADE;
CREATE SCHEMA toinoi;
SET search_path TO toinoi;
```

Kết quả lần chạy đầu (trích `OUTPUT.txt`):

```text
DROP SCHEMA IF EXISTS toinoi CASCADE;
NOTICE:  schema "toinoi" does not exist, skipping
DROP SCHEMA

CREATE SCHEMA toinoi;
CREATE SCHEMA
…
 schema_hien_tai |  mui_gio_phien   | phien_ban
-----------------+------------------+-----------
 toinoi          | Asia/Ho_Chi_Minh | 17.5
(1 row)
```

Vì sao dùng schema riêng thay vì đặt thẳng vào `public`?

- **Dọn sạch bằng một lệnh.** `DROP SCHEMA … CASCADE` xoá schema cùng mọi bảng bên trong; chạy lại `00` là về trạng thái trống. Lệnh này nguy hiểm đúng bằng sự tiện lợi của nó: chỉ chạy trên database lab.
- **Không đụng dữ liệu khác.** Nếu bạn chạy lab trên một server đã có dữ liệu riêng trong `public`, lab không chạm vào đó.
- **Tên đầy đủ rõ ràng.** `toinoi.trips` luôn chỉ đúng một bảng, bất kể đang đứng ở schema nào.

**`search_path`** là danh sách schema mà PostgreSQL lần lượt tìm khi bạn viết tên bảng không kèm schema. Thử đổi nó để thấy tác dụng:

```sql
SHOW search_path;
SELECT count(*) AS so_chuyen FROM trips;
SET search_path TO public;
SELECT count(*) FROM trips;
SELECT count(*) AS so_chuyen FROM toinoi.trips;
```

```text
 search_path
-------------
 toinoi
(1 row)
 so_chuyen
-----------
        15
(1 row)
SET
ERROR:  relation "trips" does not exist
 so_chuyen
-----------
        15
(1 row)
```

Bảng `trips` không biến mất; PostgreSQL chỉ không tìm thấy nó trong `public`. Tên đầy đủ `toinoi.trips` vẫn chạy. `search_path` chỉ sống trong phiên hiện tại, nên mỗi file của lab đều tự đặt lại `SET search_path TO toinoi;` ở đầu.

## 6. DBMS giữ luật, ứng dụng giữ quyết định

"PostgreSQL là nguồn dữ liệu gốc" không có nghĩa là dồn hết logic vào database. Hai bên chia việc:

| Việc | Ai chịu trách nhiệm | Ví dụ Tới Nơi |
|---|---|---|
| Lưu bền, không mất dữ liệu đã xác nhận | PostgreSQL | chuyến 100004 vẫn còn sau khi server khởi động lại |
| Luật bất biến của dữ liệu | PostgreSQL (ràng buộc) | số điện thoại không trùng, cước không âm, chuyến trỏ tới khách có thật |
| Nhiều yêu cầu đồng thời không phá nhau | PostgreSQL + cách ứng dụng viết giao dịch | hai tài xế bấm nhận cùng một chuyến (Phần 4) |
| Quyết định nghiệp vụ | ứng dụng | chọn tài xế nào, tính cước bao nhiêu, gửi thông báo |

Lab có sẵn bằng chứng cho dòng thứ hai. File `03-fixture-invalid.sql` cố thêm một khách mới dùng số điện thoại đã thuộc về khách Nguyễn Văn An. PostgreSQL từ chối, kể cả khi ứng dụng quên kiểm tra (trích `OUTPUT.txt`):

```text
ERROR:  duplicate key value violates unique constraint "uq_customers_phone"
DETAIL:  Key (phone)=(0900000001) already exists.
```

Cách khai báo `uq_customers_phone` được trình bày ở [PG-005](/articles/postgresql-create-table-constraints). Điều cần giữ lúc này: luật nằm trong database thì **mọi** client đều phải tuân theo, kể cả script sửa dữ liệu viết vội lúc nửa đêm.

## 7. PostgreSQL và các database khác trong blog

Blog dự kiến có series riêng cho từng engine, cùng nghiệp vụ Tới Nơi nhưng thiết kế theo cơ chế của engine đó. Các series dưới đây **mới ở mức dự kiến**:

| Engine | Mô hình dữ liệu chính | Series |
|---|---|---|
| MySQL, Microsoft SQL Server, Oracle Database | quan hệ, dùng SQL | dự kiến |
| MongoDB | document (tài liệu dạng JSON/BSON) | dự kiến |
| Apache Cassandra, ScyllaDB | wide-column phân tán, truy cập theo partition key | dự kiến |
| Redis | cấu trúc dữ liệu trong bộ nhớ (key-value và hơn thế) | dự kiến |

Theo hồ sơ Tới Nơi, MySQL và MongoDB có mặt để học so sánh và migration khi có số đo, không có quy luật "vượt một ngưỡng tải thì phải rời PostgreSQL". Series này chỉ bàn cơ chế của PostgreSQL; so sánh sâu để dành cho series của từng engine.

## 8. Những hiểu nhầm thường gặp

- **"Database" và "PostgreSQL" là một.** PostgreSQL là DBMS; một server quản lý nhiều database. Nghe "database chậm", hãy hỏi: server nào, database nào?
- **Không tìm thấy bảng là bảng bị mất.** Lỗi `relation "trips" does not exist` thường do `search_path` hoặc đang kết nối nhầm database. Kiểm tra `current_database()` và `SHOW search_path` trước khi hoảng.
- **Luật đã kiểm tra ở ứng dụng thì database không cần kiểm tra.** Ứng dụng có nhiều đường ghi (API, job, script vận hành); chỉ ràng buộc trong database chặn được tất cả.

## 9. Bài tập

**Bài 1.** Với mỗi dữ liệu sau, chọn nơi giữ bản gốc phù hợp (PostgreSQL / cache / file log) và giải thích ngắn: (a) cước của chuyến 100004; (b) lượt hiển thị menu quán trong 5 giây gần nhất để trang tải nhanh; (c) dòng ghi "dịch vụ đặt chuyến nhận request lúc 12:00:03" để điều tra lỗi.

<details><summary><b>Lời giải</b></summary><p>(a) PostgreSQL: là tiền, cần bền, có luật và phải đối soát. (b) Cache: cũ vài giây không sai tiền, bản gốc menu vẫn ở database. (c) Log: phục vụ điều tra; mất một dòng log không làm sai trạng thái chuyến. Đây là phân vai trong bài, chưa phải kiến trúc chốt của Tới Nơi.</p></details>

**Bài 2.** Sắp xếp từ lớn tới nhỏ và nêu tên cụ thể trong lab PGlite: dòng, schema, database cluster, bảng, database.

<details><summary><b>Lời giải</b></summary><p>Database cluster (do một server PostgreSQL 17.5 quản lý) → database (<code>template1</code>; trên Docker là <code>toinoi</code>) → schema (<code>toinoi</code>) → bảng (ví dụ <code>trips</code>) → dòng (ví dụ chuyến <code>100004</code>).</p></details>

**Bài 3.** Một đồng nghiệp chạy `SELECT count(*) FROM trips;` và nhận `ERROR:  relation "trips" does not exist`, trong khi bạn vừa đếm được 15 chuyến. Liệt kê ba khả năng và câu lệnh để kiểm tra từng khả năng.

<details><summary><b>Lời giải</b></summary><p>(1) <code>search_path</code> không chứa <code>toinoi</code>: chạy <code>SHOW search_path;</code> hoặc thử tên đầy đủ <code>toinoi.trips</code>. (2) Kết nối nhầm database: chạy <code>SELECT current_database();</code>. (3) Bảng chưa được tạo hoặc đã bị dọn (vừa chạy <code>00-reset</code> mà chưa chạy <code>01</code>): liệt kê bảng bằng câu <code>information_schema.tables</code> ở mục 3. Không kết luận "mất dữ liệu" trước khi loại trừ ba khả năng này.</p></details>

## 10. Đọc tiếp

- [PostgreSQL 17 — What Is PostgreSQL?](https://www.postgresql.org/docs/17/intro-whatis.html).
- [PostgreSQL 17 — Architectural Fundamentals](https://www.postgresql.org/docs/17/tutorial-arch.html): client–server.
- [PostgreSQL 17 — Creating a Database Cluster](https://www.postgresql.org/docs/17/creating-cluster.html): cụm và thư mục dữ liệu.
- [PostgreSQL 17 — Schemas](https://www.postgresql.org/docs/17/ddl-schemas.html): schema, `search_path`.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số liệu tải và các bảng chính.

Bài tiếp theo: [PG-002 · Dựng môi trường lab và kết nối bằng psql](/articles/postgresql-cai-dat-lab-psql-ket-noi).

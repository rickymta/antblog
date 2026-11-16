---
slug: mssql-la-gi-va-vai-tro-trong-toi-noi
title: "SQL Server 2022 là gì và nằm ở đâu trong Tới Nơi?"
summary: "Khách hỏi chuyến 100004 có thật không và bị tính bao nhiêu tiền: nếu dữ liệu chuyến nằm trên SQL Server thì câu trả lời đến từ đâu? Bài này phân biệt instance, database, schema, bảng, giới thiệu các edition của SQL Server 2022 và vị trí của nó trong Tới Nơi."
category: mssql
level: beginner
tags: [mssql, ms-001, sql-server-2022, instance, schema, client-server, toi-noi]
language: vi
---

Trưa 02/11, anh Võ Minh Đức đặt một suất mì Quảng qua Tới Nơi ở Đà Nẵng. Sáng hôm sau anh gọi tổng đài: "Hôm qua tôi bị trừ 55.000 đ, chuyến đó có thật không, ai giao?" Ứng dụng trên điện thoại có lịch sử chuyến, dịch vụ đặt chuyến có log, báo cáo của bộ phận tài chính có một bản xuất ra Excel. Nhân viên chăm sóc khách hàng tin nơi nào?

Câu trả lời chỉ có khi hệ thống đã quyết định trước: **dữ liệu chuyến có đúng một nơi giữ bản gốc**, mọi bản sao đối chiếu về đó. Series [PostgreSQL qua Tới Nơi](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) đặt bản gốc ấy trên PostgreSQL. Series này đặt cùng nghiệp vụ, cùng dữ liệu lên **Microsoft SQL Server 2022** — lựa chọn thường gặp ở ngân hàng, doanh nghiệp đã dùng hệ sinh thái Microsoft và các đội backend .NET. Bài đầu tiên trả lời: SQL Server là gì, một "SQL Server" gồm những lớp nào, và nó giữ trách nhiệm gì trong Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-001 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>. Không cần biết SQL. Lab: <code>examples/toi-noi-mssql/p01</code> (SQL Server 2022 chạy trong Docker, dựng ở MS-002). Sau bài này, bạn phân biệt được instance, database, schema, bảng; biết SQL Server 2022 có những edition nào và nói rõ trách nhiệm lưu dữ liệu nằm ở đâu trong Tới Nơi.</p>
</div>

## 1. Nơi giữ bản gốc phải làm được gì?

**Nguồn dữ liệu gốc** (*source of truth*) là nơi giữ bản dữ liệu có thẩm quyền: khi các bản sao lệch nhau, bản ở đây thắng. Với Tới Nơi, khách hàng, tài xế, quán và chuyến là những dữ liệu không được sai, vì chúng là tiền và là lời hứa với ba bên.

Bài [PG-001](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) đã phân tích vì sao không để dữ liệu chuyến trong file JSON, trong bộ nhớ ứng dụng hay chỉ trong một kho key-value. Kết luận là bốn yêu cầu, và chúng không phụ thuộc engine:

| Yêu cầu | Ví dụ ở Tới Nơi | SQL Server đáp ứng bằng | Học ở |
|---|---|---|---|
| **Bền**: đã báo thành công thì sập máy không mất | Chuyến 100004 đã thanh toán 55.000 đ | Transaction log ghi trước, recovery khi khởi động lại | Phần 5 |
| **Nhiều bên dùng cùng lúc an toàn** | Cao điểm ~70 chuyến/giây, hai tài xế bấm nhận cùng một chuyến | Transaction, khoá, row versioning | Phần 4 |
| **Tự giữ luật dữ liệu** | Số điện thoại không trùng, cước không âm | PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK | Phần 1 |
| **Hỏi được nhiều kiểu** | Chuyến của khách X tháng này, doanh thu theo thành phố | T-SQL, index, execution plan | Phần 2, 3 |

Log để điều tra, cache để đọc nhanh, file Excel để chia sẻ báo cáo; **SQL Server giữ bản gốc**. Phần còn lại của series là cách SQL Server đáp ứng từng yêu cầu — và chỗ nó làm *khác* PostgreSQL.

## 2. SQL Server 2022 là gì?

**Microsoft SQL Server** là một **hệ quản trị cơ sở dữ liệu quan hệ** (*RDBMS*): phần mềm lưu dữ liệu dưới dạng bảng có quan hệ với nhau, nhận câu lệnh SQL, kiểm luật, điều phối nhiều người dùng và ghi xuống đĩa an toàn. Ngôn ngữ của nó là **T-SQL** (*Transact-SQL*): SQL chuẩn cộng phần mở rộng riêng của Microsoft như biến `@x`, `TRY…CATCH`, `TOP`, mệnh đề `OUTPUT`.

**SQL Server 2022** là bản có số hiệu **16.x**. Microsoft không còn phát hành service pack; bản vá được gom thành **cumulative update** (CU) đánh số tăng dần. Lab của series ghim một bản cụ thể và in nó ra ngay khi dựng database:

```sql
SELECT CONVERT(nvarchar(20), SERVERPROPERTY('ProductVersion'))     AS phien_ban,
       CONVERT(nvarchar(20), SERVERPROPERTY('ProductUpdateLevel')) AS cu,
       CONVERT(nvarchar(60), SERVERPROPERTY('Edition'))            AS edition;
```

```text
phien_ban|cu|edition
---------|--|-------
16.0.4325.4|CU28|Developer Edition (64-bit)
```

Từ bản 2017, SQL Server chạy được trên **Linux**, và Microsoft phát hành ảnh container chính thức. Lab chạy đúng bản Linux đó trong Docker Desktop — trên Windows, macOS (Intel) hay Linux đều cùng một engine:

```sql
SELECT host_platform, host_distribution, host_release FROM sys.dm_os_host_info;
```

```text
host_platform|host_distribution|host_release
-------------|-----------------|------------
Linux|Ubuntu|22.04
```

Một số thành phần chỉ có trên Windows (ví dụ Reporting Services, xác thực Windows tích hợp đầy đủ); phần engine lưu trữ, T-SQL, index, transaction, backup mà series học thì giống nhau trên hai hệ điều hành.

### Edition: cùng engine, khác giới hạn và giấy phép

| Edition | Dùng khi | Giới hạn đáng nhớ |
|---|---|---|
| **Enterprise** | Production cần đầy đủ tính năng (online index rebuild, partition lớn, availability group nhiều replica…) | Trả phí theo core |
| **Standard** | Production cỡ vừa | Giới hạn số core và bộ nhớ buffer pool; thiếu một số tính năng Enterprise |
| **Express** | Ứng dụng nhỏ, miễn phí | Mỗi database tối đa 10 GB, ít CPU và RAM |
| **Developer** | Phát triển, kiểm thử, học — **miễn phí** | Đủ tính năng Enterprise nhưng **không được dùng cho production** |

Lab dùng **Developer Edition**. Khi một bài dùng tính năng không có ở Standard hay Express, bài sẽ ghi rõ — vì "chạy được trên lab" chưa có nghĩa là "chạy được trên bản công ty đang mua".

## 3. Instance, database, schema, bảng

Bốn từ này hay bị dùng lẫn. Trong SQL Server chúng là bốn lớp lồng nhau.

**Instance** là một bản SQL Server đang chạy: một tiến trình `sqlservr` với cấu hình, bộ nhớ, danh sách login riêng, lắng nghe một cổng mạng. Một máy có thể chạy nhiều instance; instance mặc định tên `MSSQLSERVER`, các instance khác gọi là *named instance* và kết nối bằng `máy\TÊN`. Lab có một instance mặc định trong container:

```sql
SELECT @@SERVERNAME AS server_name, @@SERVICENAME AS instance;
```

```text
server_name|instance
-----------|--------
9aab891f05ee|MSSQLSERVER
```

`server_name` là hostname của container — mỗi lần tạo lại container sẽ khác, nên đừng dựa vào nó trong script.

**Database** (cơ sở dữ liệu) là một tập dữ liệu có tên trong instance. Trên đĩa, mỗi database có ít nhất **một file dữ liệu** (`.mdf`) và **một file log** (`.ldf`). Đây là điểm khác lớn so với PostgreSQL, nơi cả cụm dùng chung một thư mục dữ liệu và một WAL: ở SQL Server, **mỗi database có transaction log riêng**, backup và restore theo từng database.

```sql
SELECT DB_NAME(database_id) AS db, type_desc, physical_name
FROM sys.master_files
WHERE database_id IN (1, DB_ID('ToiNoi'))
ORDER BY database_id, file_id;
```

```text
db|type_desc|physical_name
--|---------|-------------
master|ROWS|/var/opt/mssql/data/master.mdf
master|LOG|/var/opt/mssql/data/mastlog.ldf
ToiNoi|ROWS|/var/opt/mssql/data/ToiNoi.mdf
ToiNoi|LOG|/var/opt/mssql/data/ToiNoi_log.ldf
```

Mỗi instance có bốn **database hệ thống** mà bạn sẽ gặp suốt series:

| Database | Vai trò |
|---|---|
| `master` | Danh mục cấp instance: login, danh sách database, cấu hình. Mất `master` là instance không khởi động được |
| `model` | Khuôn mẫu: database mới được tạo bằng cách sao chép `model` |
| `msdb` | Lịch sử backup, job của SQL Server Agent |
| `tempdb` | Bảng tạm, kết quả trung gian, phiên bản dòng (Phần 4). **Tạo lại mỗi lần instance khởi động** — không bao giờ để dữ liệu thật ở đây |

```sql
SELECT database_id, name, collation_name, recovery_model_desc FROM sys.databases ORDER BY database_id;
```

```text
database_id|name|collation_name|recovery_model_desc
-----------|----|--------------|-------------------
1|master|SQL_Latin1_General_CP1_CI_AS|SIMPLE
2|tempdb|SQL_Latin1_General_CP1_CI_AS|SIMPLE
3|model|SQL_Latin1_General_CP1_CI_AS|FULL
4|msdb|SQL_Latin1_General_CP1_CI_AS|SIMPLE
5|ToiNoi|Vietnamese_100_CI_AS|FULL
```

Hai cột cuối sẽ quay lại nhiều lần. **Collation** là bộ quy tắc so sánh và sắp xếp chuỗi; database `ToiNoi` dùng collation tiếng Việt (MS-004 giải thích vì sao). **Recovery model** quyết định transaction log được giữ lại bao lâu, và vì thế bạn khôi phục được tới đâu (Phần 5). `ToiNoi` thừa hưởng `FULL` từ `model`.

**Schema** (lược đồ) là "thư mục tên" bên trong database, nhóm bảng, view, thủ tục; mỗi schema có một chủ sở hữu. Tên đầy đủ của một bảng có bốn phần `instance.database.schema.bảng`, thường viết ba phần `ToiNoi.toinoi.trips` hoặc hai phần `toinoi.trips`. Schema mặc định của hầu hết user là `dbo`; lab đặt bảng nghiệp vụ trong schema `toinoi` và thủ tục kiểm tra của lab trong schema `lab` để không lẫn với nhau.

Chữ "schema" còn được dùng theo nghĩa rộng là **thiết kế cấu trúc dữ liệu** (bảng, cột, ràng buộc) — như trong "schema chuyến xe" của MS-010. Trong `toinoi.trips`, `toinoi` là schema theo nghĩa đối tượng chứa bảng.

**Bảng** (*table*) chứa dữ liệu thật: các **cột** có kiểu xác định và các **dòng**. Một dòng của `toinoi.trips` là một chuyến. Bảng, khoá và cách SQL Server sắp xếp dòng trên đĩa là chủ đề của [MS-003](/articles/mssql-bang-khoa-chinh-clustered-index).

### Nhìn bốn lớp trong lab

Sau khi chạy lab P01 (00 → 07), instance trong container trông như sau:

```text
Instance MSSQLSERVER (container toinoi-mssql-lab-db-1, cổng 14330 trên máy bạn)
├── master, model, msdb, tempdb          ← database hệ thống
└── database ToiNoi  (ToiNoi.mdf + ToiNoi_log.ldf, collation Vietnamese_100_CI_AS)
    ├── schema dbo, sys, INFORMATION_SCHEMA, guest
    ├── schema lab      ← thủ tục kiểm tra của lab, không thuộc Tới Nơi
    └── schema toinoi
        ├── cities (3)     customers (6)    drivers (7)
        └── merchants (3)  trips (16)       trip_events (7)
```

Số trong ngoặc là số dòng, đọc từ catalog:

```sql
SELECT SCHEMA_NAME(t.schema_id) AS [schema], t.name AS bang, SUM(p.rows) AS so_dong
FROM sys.tables AS t
JOIN sys.partitions AS p ON p.object_id = t.object_id AND p.index_id IN (0, 1)
GROUP BY t.schema_id, t.name
ORDER BY 1, 2;
```

```text
schema|bang|so_dong
------|----|-------
toinoi|cities|3
toinoi|customers|6
toinoi|drivers|7
toinoi|merchants|3
toinoi|trip_events|7
toinoi|trips|16
```

Các view bắt đầu bằng `sys.` là **catalog view** — SQL Server tự mô tả chính nó bằng bảng. Bạn chưa cần nhớ cú pháp; chỉ cần biết mọi câu hỏi "có những gì trong database" đều trả lời được bằng truy vấn, và công cụ GUI như DBStudio cũng đọc từ đây để vẽ cây đối tượng.

## 4. Ai nói chuyện với SQL Server, và bằng cách nào?

SQL Server là **server**: nó chờ kết nối, còn ứng dụng, công cụ quản trị và script là **client**. Client và server nói chuyện qua giao thức **TDS** (*Tabular Data Stream*), mặc định trên cổng TCP **1433**.

| Client trong series | Dùng để |
|---|---|
| Dịch vụ backend của Tới Nơi (.NET qua `Microsoft.Data.SqlClient`, hoặc driver của ngôn ngữ khác) | Đọc, ghi dữ liệu nghiệp vụ |
| **DBStudio** | Công cụ GUI của series: cây đối tượng, query editor, execution plan, ERD, backup (MS-002) |
| `sqlcmd` trong container | Chạy file lab tái lập, không cần cài gì trên máy |

Mỗi kết nối đăng nhập bằng một **login** (cấp instance) và được ánh xạ thành một **user** trong database nó dùng. Lab dùng login quản trị `sa` cho đơn giản; tách tài khoản ứng dụng, migration và báo cáo theo quyền tối thiểu là bài MS-041.

```sql
SELECT DB_NAME() AS database_hien_tai, SUSER_SNAME() AS login, USER_NAME() AS user_trong_db, SCHEMA_NAME() AS schema_mac_dinh;
```

```text
database_hien_tai|login|user_trong_db|schema_mac_dinh
-----------------|-----|-------------|---------------
ToiNoi|sa|dbo|dbo
```

Login `sa` là quản trị instance nên trong mọi database nó được ánh xạ thành user `dbo`. Schema mặc định là `dbo`, vì vậy viết `trips` không kèm schema sẽ tìm `dbo.trips` — không có — và báo lỗi. Lab luôn viết đủ `toinoi.trips`.

### T-SQL, batch và `GO`

Script T-SQL được chia thành **batch**: nhóm câu lệnh gửi lên server một lần, biên dịch cùng nhau. Dòng `GO` là dấu ngăn batch do **client** hiểu (sqlcmd, DBStudio, SSMS), **không phải** câu lệnh T-SQL — gửi `GO` cho server sẽ bị báo lỗi cú pháp. Biến khai báo bằng `DECLARE` chỉ sống trong batch của nó. Trong DBStudio, `Ctrl+Enter` chạy batch đang đặt con trỏ, `Ctrl+Shift+Enter` chạy cả script (MS-002).

## 5. So với PostgreSQL

Người đã đọc series PostgreSQL sẽ gặp nhiều chữ quen nhưng nghĩa lệch nhau:

| Khái niệm | PostgreSQL | SQL Server |
|---|---|---|
| Đơn vị server | Database cluster (một thư mục dữ liệu) | Instance |
| File trên đĩa | Cả cụm chung thư mục và WAL | Mỗi database có file `.mdf`/`.ldf` riêng |
| Database hệ thống | `postgres`, `template0`, `template1` | `master`, `model`, `msdb`, `tempdb` |
| Schema mặc định | `public` (theo `search_path`) | `dbo` (theo user) |
| Một kết nối làm việc với | Đúng một database, đổi phải kết nối lại | Database hiện tại, đổi được bằng `USE` |
| Ngôn ngữ | SQL + PL/pgSQL | T-SQL (một ngôn ngữ cho cả truy vấn và thủ tục) |
| Truy vấn xuyên database | Cần extension (`postgres_fdw`) | Viết thẳng `ToiNoi.toinoi.trips` trong cùng instance |

Điểm cuối tiện nhưng nguy hiểm: một truy vấn có thể đọc chéo database, nên quyền và backup phải nghĩ theo đúng phạm vi đó.

## 6. Những hiểu nhầm thường gặp

- **"SQL Server" là một database.** SQL Server là phần mềm; một instance chứa nhiều database. Nghe "SQL Server chậm", hãy hỏi: instance nào, database nào, truy vấn nào?
- **Developer Edition dùng tạm cho production cũng được.** Không: giấy phép Developer cấm dùng cho production. Lab dùng Developer vì miễn phí và đủ tính năng để học.
- **`tempdb` là chỗ để dữ liệu tạm "lâu lâu".** `tempdb` bị xoá sạch mỗi lần instance khởi động lại.
- **Lỗi `Invalid object name 'trips'` nghĩa là mất bảng.** Thường là thiếu tên schema (đang tìm `dbo.trips`) hoặc tab đang ở database `master`. Kiểm `DB_NAME()` và viết đủ `toinoi.trips` trước khi kết luận.
- **Luật đã kiểm ở ứng dụng thì database không cần kiểm.** Tới Nơi có nhiều đường ghi: API, job, script vận hành, người sửa tay trong công cụ GUI. Chỉ ràng buộc trong database chặn được tất cả.

## 7. Bài tập

**Bài 1.** Với mỗi dữ liệu, chọn nơi giữ bản gốc (SQL Server / cache / file log) và giải thích ngắn: (a) cước của chuyến 100004; (b) menu quán được hiển thị trong 5 giây gần nhất để trang tải nhanh; (c) dòng ghi "dịch vụ đặt chuyến nhận request lúc 12:00:03".

<details><summary><b>Lời giải</b></summary><p>(a) SQL Server: là tiền, cần bền, có luật và phải đối soát. (b) Cache: cũ vài giây không làm sai tiền, bản gốc menu vẫn ở database. (c) Log: phục vụ điều tra; mất một dòng log không làm sai trạng thái chuyến.</p></details>

**Bài 2.** Sắp xếp từ lớn tới nhỏ và nêu tên cụ thể trong lab: bảng, instance, dòng, schema, database.

<details><summary><b>Lời giải</b></summary><p>Instance (<code>MSSQLSERVER</code> trong container) → database (<code>ToiNoi</code>) → schema (<code>toinoi</code>) → bảng (ví dụ <code>trips</code>) → dòng (ví dụ chuyến <code>100004</code>).</p></details>

**Bài 3.** Một đồng nghiệp mở tab mới, chạy `SELECT COUNT(*) FROM trips;` và nhận `Invalid object name 'trips'`, trong khi bạn vừa đếm được 16 chuyến. Liệt kê ba khả năng và câu lệnh kiểm tra từng khả năng.

<details><summary><b>Lời giải</b></summary><p>(1) Thiếu schema: <code>trips</code> được tìm trong <code>dbo</code>; thử <code>toinoi.trips</code>. (2) Tab đang ở database khác (thường là <code>master</code>): chạy <code>SELECT DB_NAME();</code> hoặc nhìn ô "Cơ sở dữ liệu đang dùng" trong DBStudio. (3) Bảng chưa được tạo hoặc vừa bị dựng lại (chạy <code>00-reset</code> mà chưa chạy <code>01</code>): liệt kê bảng bằng truy vấn <code>sys.tables</code> ở mục 3.</p></details>

**Bài 4.** Vì sao không nên lưu bảng đơn hàng tạm trong `tempdb` dù "chỉ cần giữ tới hết ngày"?

<details><summary><b>Lời giải</b></summary><p><code>tempdb</code> được tạo lại mỗi lần instance khởi động (cập nhật CU, container khởi động lại, failover). Dữ liệu cần giữ tới hết ngày vẫn là dữ liệu cần bền, nên phải nằm trong database của ứng dụng; dọn nó bằng job hoặc chính sách lưu giữ.</p></details>

## 8. Đọc tiếp

- [What is SQL Server?](https://learn.microsoft.com/sql/sql-server/what-is-sql-server) — tổng quan sản phẩm.
- [Editions and supported features of SQL Server 2022](https://learn.microsoft.com/sql/sql-server/editions-and-components-of-sql-server-2022) — bảng so sánh edition.
- [System databases](https://learn.microsoft.com/sql/relational-databases/databases/system-databases) — `master`, `model`, `msdb`, `tempdb`.
- [SQL Server on Linux](https://learn.microsoft.com/sql/linux/sql-server-linux-overview) — những gì có và không có trên Linux.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số liệu tải và các bảng chính.
- [PG-001](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi): cùng câu hỏi, trả lời bằng PostgreSQL.

Bài tiếp theo: [MS-002 · Dựng SQL Server 2022 bằng Docker và kết nối bằng DBStudio](/articles/mssql-cai-dat-docker-dbstudio).

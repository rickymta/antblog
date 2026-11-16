---
slug: mssql-cai-dat-docker-dbstudio
title: "Dựng SQL Server 2022 bằng Docker và kết nối bằng DBStudio"
summary: "Dựng một instance SQL Server 2022 riêng cho lab Tới Nơi bằng Docker Desktop, chạy file SQL tái lập bằng sqlcmd trong container, kết nối và chạy batch đầu tiên bằng DBStudio, ghi phiếu lab và dọn môi trường sạch sẽ."
category: mssql
level: beginner
tags: [mssql, ms-002, sql-server-2022, docker, dbstudio, sqlcmd, toi-noi]
language: vi
---

Tuần đầu tiên làm việc với SQL Server của Tới Nơi, Minh — backend developer mới vào nhóm — cài thử SQL Server lên máy cá nhân theo một bài hướng dẫn trên mạng. Hai ngày sau, đồng nghiệp gửi một script tạo bảng; trên máy Minh nó chạy được, trên máy đồng nghiệp lại báo lỗi ở một chữ tiếng Việt bị biến thành dấu `?`. Hoá ra hai máy khác bản vá, khác collation mặc định, và không ai nhớ mình đã bấm gì trong trình cài đặt.

Lab của series tránh chuyện đó bằng một nguyên tắc: **môi trường là code**. Một file `docker-compose.yml` mô tả đúng bản SQL Server, collation, múi giờ, cổng; một runner chạy các file SQL theo thứ tự và in kết quả để so. Bài này dựng lab đó, kết nối bằng công cụ GUI **DBStudio**, chạy batch đầu tiên và dọn môi trường.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-002 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/mssql-la-gi-va-vai-tro-trong-toi-noi">MS-001</a>. Cần Docker Desktop và Node.js 22. Lab: <code>examples/toi-noi-mssql</code>. Sau bài này, bạn có một instance SQL Server 2022 riêng, chạy được lab P01 với kết quả <code>80 PASS, 0 FAIL</code>, kết nối được bằng DBStudio và biết cách dọn sạch.</p>
</div>

## 1. Lab cần đạt bốn điều

| Yêu cầu | Lab làm thế nào |
|---|---|
| **Tái lập**: ai chạy cũng ra cùng kết quả | Ảnh SQL Server ghim theo digest; database tạo lại từ đầu bằng `00-reset.sql`; dữ liệu cố định; `OUTPUT.txt` là kết quả mẫu để so |
| **Cô lập**: không đụng tới SQL Server khác trên máy | Chạy trong container, cổng riêng `14330`, chỉ mở trên `127.0.0.1` |
| **Ghi được phiên bản** | Script đầu tiên in phiên bản, CU, edition và collation |
| **Dọn được** | Một lệnh xoá container và dữ liệu; không để lại service chạy nền |

Không cần cài SQL Server, sqlcmd hay driver ODBC trên máy: `sqlcmd` có sẵn **bên trong** ảnh container.

## 2. Chuẩn bị

- **Docker Desktop** đang chạy, engine Linux (trên Windows là WSL 2). Container SQL Server cần khoảng **2 GB RAM**; compose đặt `mem_limit: 2g`. Nếu Docker Desktop chỉ được cấp 2 GB tổng, hãy tăng lên ít nhất 4 GB trong phần Resources.
- **Node.js 22** để chạy runner `run.mjs` và test. Runner không có dependency npm, không cần `npm install`.
- **Máy Apple Silicon**: ảnh SQL Server chỉ có bản amd64, chạy qua giả lập — được, nhưng chậm hơn. Các phép đo hiệu năng trong series chạy trên máy x64.

Kiểm nhanh:

```sh
docker version --format '{{.Server.Version}}'
node --version
```

## 3. Dựng SQL Server 2022 bằng Docker Compose

Toàn bộ cấu hình nằm trong `examples/toi-noi-mssql/docker-compose.yml`. Đọc từng dòng một lần, vì mỗi dòng là một quyết định:

```yaml
services:
  db:
    image: mcr.microsoft.com/mssql/server:2022-latest@sha256:4525e3efac9f3e241cb003bb2ff2a82818b502f37e12049d788b1a9fa7e13b7f
    environment:
      ACCEPT_EULA: "Y"
      MSSQL_PID: "Developer"
      MSSQL_SA_PASSWORD: "ToiNoi_lab_2022!"
      MSSQL_COLLATION: "SQL_Latin1_General_CP1_CI_AS"
      TZ: "Asia/Ho_Chi_Minh"
    ports:
      - "127.0.0.1:14330:1433"
    volumes:
      - toinoi-mssql2022-data:/var/opt/mssql
      - ./p01:/lab/p01:ro
    healthcheck:
      test: ["CMD-SHELL", "/opt/mssql-tools18/bin/sqlcmd -C -S localhost -U sa -P \"$$MSSQL_SA_PASSWORD\" -Q \"SELECT 1\" -b -o /dev/null"]
    mem_limit: 2g
```

| Dòng | Ý nghĩa |
|---|---|
| `image: …2022-latest@sha256:…` | Tag `2022-latest` trôi theo CU mới; **digest** khoá đúng một bản. Bản đã kiểm chứng: **16.0.4325.4 (RTM-CU28)** |
| `ACCEPT_EULA` | Bắt buộc, xác nhận đồng ý điều khoản giấy phép |
| `MSSQL_PID: Developer` | Developer Edition: miễn phí, đủ tính năng, **không dùng cho production** (MS-001) |
| `MSSQL_SA_PASSWORD` | Mật khẩu login quản trị `sa`. Phải đạt chính sách mật khẩu (mục 9). Đây là giá trị giữ chỗ cho lab, không dùng ở đâu khác |
| `MSSQL_COLLATION` | Collation **của instance** (database hệ thống, `tempdb`). Database `ToiNoi` tự chọn collation tiếng Việt khi tạo (MS-004) |
| `TZ` | Múi giờ của hệ điều hành trong container; `SYSDATETIMEOFFSET()` trả giờ theo múi này |
| `127.0.0.1:14330:1433` | Cổng 1433 của container mở ra cổng **14330** trên máy bạn, chỉ cho chính máy bạn. Tránh đụng SQL Server khác đang dùng 1433 |
| `volumes` | Dữ liệu SQL Server nằm trong volume có tên, sống qua lần khởi động lại; thư mục `p01` được mount **chỉ đọc** vào `/lab/p01` để sqlcmd đọc file SQL |
| `healthcheck` | Docker coi container "healthy" khi đăng nhập được và chạy được `SELECT 1` — không phải chỉ khi tiến trình đã chạy |

Khởi động và chờ healthy:

```sh
cd examples/toi-noi-mssql
npm run db:up            # = docker compose up -d --wait
docker compose ps
```

```text
NAME                    STATUS                    PORTS
toinoi-mssql-lab-db-1   Up 17 minutes (healthy)   127.0.0.1:14330->1433/tcp
```

Lần đầu Docker tải ảnh khoảng 2,2 GB. Trong log của container, dòng cần tìm là:

```text
SQL Server is now ready for client connections. This is an informational message; no user action is required.
```

## 4. Chạy lab bằng runner

```sh
npm run p01
```

Runner `run.mjs` gọi, cho từng file `p01/NN-*.sql` theo thứ tự tên:

```sh
docker compose exec -T db /opt/mssql-tools18/bin/sqlcmd \
  -C -S localhost -U sa -P "…" -b -I -W -s "|" -f i:65001,o:65001 -i /lab/p01/00-reset.sql
```

| Tuỳ chọn sqlcmd | Vì sao |
|---|---|
| `-C` | Tin chứng chỉ TLS tự ký của container. Từ mssql-tools18, sqlcmd mặc định **mã hoá và xác minh chứng chỉ**; thiếu `-C` sẽ báo lỗi chứng chỉ |
| `-b` | Lỗi không được bắt ⇒ dừng file với mã thoát khác 0; runner báo `FAIL` |
| `-I` | Bật `QUOTED_IDENTIFIER`, giống driver ứng dụng và DBStudio |
| `-W -s "\|"` | Bỏ khoảng trắng thừa, ngăn cột bằng `\|` để kết quả gọn và so được |
| `-f i:65001,o:65001` | Đọc và in UTF-8 — không có nó chữ tiếng Việt trong file SQL hỏng ngay từ đầu vào |

Mỗi file là **một phiên kết nối riêng**: biến, `SET`, bảng tạm không sang file sau. Lỗi *mong đợi* được bắt trong SQL bằng thủ tục `lab.expect_error` và in thành dòng `PASS`/`FAIL`; runner đếm các dòng đó. Kết quả cuối phải là:

```text
-- [lab] File đã chạy: 8
-- [lab] PASS: 80
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

`npm test` chạy lại toàn bộ, so từng dòng với `p01/OUTPUT.txt`, thử script hoàn tác của MS-009 hai lần, rồi trả database về trạng thái đầy đủ.

### File đầu tiên in gì?

`00-reset.sql` xoá database `ToiNoi` cũ (nếu có), tạo lại với collation tiếng Việt, tạo schema `toinoi` và `lab`, rồi in vị trí của phiên:

```text
database_hien_tai|login|user_trong_db|schema_mac_dinh|collation_db
-----------------|-----|-------------|---------------|------------
ToiNoi|sa|dbo|dbo|Vietnamese_100_CI_AS
phien_ban|cu|edition
---------|--|-------
16.0.4325.4|CU28|Developer Edition (64-bit)
```

Để xoá được database đang có người dùng, script chạy `ALTER DATABASE ToiNoi SET SINGLE_USER WITH ROLLBACK IMMEDIATE` — **huỷ ngang** giao dịch của mọi phiên khác, kể cả tab DBStudio đang mở. Chấp nhận được trên lab; không bao giờ chạy câu này trên database thật.

## 5. Kết nối bằng DBStudio

**DBStudio** là công cụ quản trị nhiều loại database (PostgreSQL, MySQL, SQL Server, Oracle, MongoDB, Redis…) viết bằng Rust và Tauri, mã nguồn ở [github.com/rickymta/dbstudio](https://github.com/rickymta/dbstudio). Series dùng nó làm công cụ GUI vì một số tính năng hợp với việc học: chạy theo batch `GO`, transaction thủ công có trạng thái rõ ràng, sơ đồ execution plan, chặn câu lệnh phá huỷ trên kết nối production.

**Cài đặt.** Trang Releases có bộ cài cho Windows (`.msi`, `-setup.exe`), macOS (`.dmg`) và Linux (`.AppImage`, `.deb`, `.rpm`), kèm `SHA256SUMS.txt` để kiểm tra file tải về. Series dùng bản **0.1.7**. Bộ cài chưa được ký số, nên Windows SmartScreen sẽ hỏi lại trước khi chạy.

**Tạo kết nối.** Mở hộp thoại **Kết nối mới**:

1. Ô **URI kết nối**: dán chuỗi ADO.NET rồi bấm **Điền từ URI**:

   ```text
   Server=127.0.0.1,14330;Database=ToiNoi;User Id=sa;TrustServerCertificate=True;
   ```

   DBStudio hiểu cả dạng `sqlserver://…` và JDBC. Mật khẩu không được lấy từ chuỗi; bạn gõ lại ở ô **Mật khẩu**.
2. Kiểm lại phần **Chung**: **Loại cơ sở dữ liệu** là SQL Server, **Máy chủ** `127.0.0.1`, **Cổng** `14330`, **Cơ sở dữ liệu** `ToiNoi` (để trống thì đăng nhập vào `master`). Đặt **Tên** là `toinoi-lab`.
3. **Môi trường**: chọn **Phát triển**. Nhãn này không chỉ để tô màu: với kết nối **Sản xuất**, DBStudio bắt gõ lại tên database trước khi chạy `DROP`, `TRUNCATE` hay `DELETE`/`UPDATE` không có `WHERE`. Kết nối tới production thật sau này luôn chọn đúng nhãn.
4. Phần **Xác thực**: "Tên đăng nhập và mật khẩu", **Tên đăng nhập** `sa`. Tuỳ chọn **Lưu mật khẩu** cất mật khẩu vào keychain của hệ điều hành.
5. Phần **TLS**: `TrustServerCertificate=True` được điền thành chế độ **Bắt buộc** — mã hoá kết nối nhưng không xác minh chứng chỉ, phù hợp với chứng chỉ tự ký của container. Hai chế độ "Xác minh CA" và "Xác minh đầy đủ" dành cho server có chứng chỉ thật (MS-042).
6. Bấm **Kiểm tra kết nối**, rồi **Lưu**.

Ngoài ra còn công tắc **Chỉ đọc** ("Chặn mọi câu lệnh ghi dữ liệu") — hữu ích khi bạn chỉ cần xem dữ liệu production. Kết nối lab thì để tắt.

**Cây đối tượng.** Mở kết nối, bạn thấy danh sách database (bốn database hệ thống có nhãn riêng) và `ToiNoi`. Trong một database: schema → các nhóm *tables, views, functions, procedures, sequences, synonyms, types*; mở một bảng thấy *columns, indexes, triggers*. Đó chính là các catalog view `sys.*` ở MS-001, được vẽ thành cây.

## 6. Batch đầu tiên trong DBStudio

Mở một tab query cho kết nối `toinoi-lab`. Ô **Cơ sở dữ liệu đang dùng** trên tab cho biết batch sẽ chạy ở database nào — kiểm nó trước mọi câu lệnh; đổi ở đây tương đương chạy `USE`. Gõ:

```sql
SELECT CONVERT(nvarchar(20), SERVERPROPERTY('ProductVersion')) AS phien_ban,
       CONVERT(nvarchar(20), SERVERPROPERTY('ProductUpdateLevel')) AS cu;
PRINT N'Xin chào từ ToiNoi';
GO
SELECT CURRENT_TIMEZONE() AS tz,
       CAST(SYSDATETIMEOFFSET() AS varchar(34)) AS now_local,
       CAST(SYSUTCDATETIME() AS varchar(30))    AS now_utc;
```

Đặt con trỏ trong batch thứ nhất và bấm `Ctrl+Enter`: DBStudio chỉ gửi **batch đang đứng** (đoạn giữa hai dòng `GO`). Kết quả `SELECT` hiện ở lưới; dòng `PRINT` nằm ở tab **Thông điệp**. `Ctrl+Shift+Enter` chạy cả script: mỗi batch là một lần gửi lên server, mỗi result set thành một kết quả riêng. `Ctrl+.` huỷ câu đang chạy mà không mất kết nối.

Kết quả của batch thứ hai trên lab (giờ đã thay bằng `HH:MM:SS`):

```text
tz|now_local|now_utc
--|---------|-------
(UTC+07:00) Bangkok, Hanoi, Jakarta|2026-11-16 HH:MM:SS +07:00|2026-11-16 HH:MM:SS
```

### Múi giờ của server không phải múi giờ của bạn

`CURRENT_TIMEZONE()` ra giờ Việt Nam chỉ vì compose đặt `TZ: Asia/Ho_Chi_Minh`. Bỏ dòng đó, container chạy theo UTC và `SYSDATETIMEOFFSET()` trả `+00:00`. Server production có thể đặt bất kỳ múi giờ nào, nên lab **không bao giờ dựa vào giờ của server** để lưu dữ liệu: mọi mốc thời gian được ghi kèm độ lệch `+07:00` trong kiểu `datetimeoffset` (MS-004).

Mỗi tab query của DBStudio giữ **một kết nối riêng** suốt thời gian mở: `USE`, bảng tạm `#…`, biến phiên và transaction đang mở đều thuộc tab đó. Hai tab là hai phiên — đúng điều Phần 4 cần khi thử hai tài xế cùng nhận một chuyến.

## 7. Ghi phiếu lab

Mỗi lần chạy lab cho một bài viết hay một báo cáo, ghi lại:

| Mục | Giá trị trên máy viết bài |
|---|---|
| SQL Server | 16.0.4325.4, RTM-CU28, Developer Edition (64-bit), Linux Ubuntu 22.04 |
| Ảnh | `mcr.microsoft.com/mssql/server:2022-latest@sha256:4525e3ef…` |
| Docker | Docker Desktop, engine 29.9, Windows 11, WSL 2 |
| Collation | Instance `SQL_Latin1_General_CP1_CI_AS`, database `ToiNoi` `Vietnamese_100_CI_AS` |
| Múi giờ server | `(UTC+07:00) Bangkok, Hanoi, Jakarta` (từ `TZ` của container) |
| Kết quả | `npm run p01`: 8 file, 80 PASS, 0 FAIL |

Khi kết quả của bạn khác `OUTPUT.txt`, so phiếu lab trước tiên: khác CU, khác collation hay khác múi giờ giải thích phần lớn chênh lệch.

## 8. Dữ liệu giữa các lần chạy và dọn môi trường

| Lệnh | Container | Dữ liệu trong volume |
|---|---|---|
| `docker compose stop` / `start` | Dừng / chạy lại | Giữ nguyên |
| `docker compose down` | Xoá | **Giữ** (volume `toinoi-mssql2022-data` còn) |
| `npm run db:down` (= `docker compose down -v`) | Xoá | **Xoá** |

Với lab, mất dữ liệu không sao: `npm run p01` dựng lại trong vài giây. Ảnh Docker vẫn nằm trên máy sau khi dọn; xoá hẳn bằng `docker image rm mcr.microsoft.com/mssql/server:2022-latest` nếu không học tiếp.

## 9. Lỗi thường gặp

- **Container dừng ngay sau khi chạy.** Thường do mật khẩu `sa` không đạt chính sách: tối thiểu 8 ký tự và có ít nhất ba trong bốn nhóm (chữ hoa, chữ thường, chữ số, ký hiệu). Log ghi rõ:

  ```text
  ERROR: Unable to set system administrator password: Password validation failed. The password does not meet SQL Server password policy requirements because it is too short. The password must be at least 8 characters..
  ```

- **`Bind for 127.0.0.1:14330 failed: port is already allocated`.** Một tiến trình khác giữ cổng; đổi số bên trái trong `ports` và trong chuỗi kết nối.
- **sqlcmd báo lỗi chứng chỉ (`certificate verify failed`).** Thiếu `-C`. Trong ứng dụng .NET là thiếu `TrustServerCertificate=True` — chỉ chấp nhận trên lab.
- **`Login failed for user 'sa'`.** Sai mật khẩu, hoặc volume cũ được tạo với mật khẩu khác: `MSSQL_SA_PASSWORD` chỉ có tác dụng lúc khởi tạo lần đầu. Chạy `npm run db:down` rồi `npm run db:up`.
- **Chạy `docker compose exec … /opt/…` trong Git Bash báo không tìm thấy file `C:/Program Files/Git/opt/...`.** Git Bash tự đổi đường dẫn bắt đầu bằng `/`. Đặt `MSYS_NO_PATHCONV=1` trước lệnh; runner đã làm sẵn việc này.
- **Chữ tiếng Việt thành `?`.** Hoặc sqlcmd thiếu `-f 65001`, hoặc chuỗi trong SQL thiếu tiền tố `N'…'`. MS-004 chỉ rõ chữ nào mất và vì sao.

## 10. Bài tập

**Bài 1.** Chạy `npm run p01` hai lần liên tiếp. Dòng nào chỉ xuất hiện ở lần thứ hai, và vì sao test bỏ qua đúng dòng đó khi so với `OUTPUT.txt`?

<details><summary><b>Lời giải</b></summary><p>Dòng <code>Đã xoá database ToiNoi cũ</code>: lần đầu database chưa có nên khối <code>IF DB_ID(N'ToiNoi') IS NOT NULL</code> không chạy. Dòng này phụ thuộc trạng thái trước đó chứ không phụ thuộc kết quả của lab, nên test loại nó ra trước khi so.</p></details>

**Bài 2.** Bỏ dòng `TZ` khỏi compose, chạy `docker compose up -d --wait` (container được tạo lại), rồi chạy lại batch `CURRENT_TIMEZONE()` trong DBStudio. Kết quả `npm run p01` có đổi không?

<details><summary><b>Lời giải</b></summary><p><code>CURRENT_TIMEZONE()</code> chuyển sang UTC và <code>SYSDATETIMEOFFSET()</code> trả <code>+00:00</code>. Kết quả lab không đổi, vì mọi mốc thời gian trong lab là hằng số có độ lệch <code>+07:00</code>, không đọc giờ của server. Đó là lý do lab viết như vậy. Nhớ trả lại dòng <code>TZ</code> sau khi thử.</p></details>

**Bài 3.** Trong DBStudio, tạo thêm một kết nối tới cùng lab nhưng bật **Chỉ đọc**. Chạy `SELECT COUNT(*) FROM toinoi.trips;` rồi `DELETE FROM toinoi.trip_events WHERE 1 = 0;`. Điều gì xảy ra, và vì sao câu `DELETE` không xoá dòng nào vẫn bị chặn?

<details><summary><b>Lời giải</b></summary><p><code>SELECT</code> chạy bình thường (16 chuyến). <code>DELETE</code> bị DBStudio từ chối trước khi gửi lên server: chế độ chỉ đọc phân loại câu lệnh theo <em>loại</em> (ghi dữ liệu), không theo số dòng sẽ bị ảnh hưởng — vì số dòng chỉ biết được sau khi chạy. Lưu ý đây là lớp bảo vệ của công cụ; quyền thật nằm ở database (MS-041).</p></details>

**Bài 4.** Đồng nghiệp đề xuất bỏ digest, chỉ dùng `2022-latest` "cho luôn được bản vá mới". Nêu một lợi ích và một rủi ro, và cách giữ cả hai.

<details><summary><b>Lời giải</b></summary><p>Lợi ích: nhận bản vá bảo mật và sửa lỗi sớm. Rủi ro: hai người chạy cùng lab có thể ở hai CU khác nhau và ra kết quả khác nhau mà không biết. Cách giữ cả hai: ghim digest, và khi muốn lên CU mới thì đổi digest trong một commit riêng, chạy <code>npm test</code>, cập nhật <code>OUTPUT.txt</code> và phiếu lab.</p></details>

## 11. Đọc tiếp

- [Quickstart: Run SQL Server Linux container images with Docker](https://learn.microsoft.com/sql/linux/quickstart-install-connect-docker) — tài liệu chính thức về ảnh container.
- [Configure SQL Server settings with environment variables on Linux](https://learn.microsoft.com/sql/linux/sql-server-linux-configure-environment-variables) — `MSSQL_PID`, `MSSQL_COLLATION`, `MSSQL_SA_PASSWORD`…
- [sqlcmd utility](https://learn.microsoft.com/sql/tools/sqlcmd/sqlcmd-utility) — các tuỳ chọn `-C`, `-b`, `-f`, `-W`.
- [Password policy](https://learn.microsoft.com/sql/relational-databases/security/password-policy) — yêu cầu độ phức tạp của mật khẩu.
- [DBStudio](https://github.com/rickymta/dbstudio) — mã nguồn và bộ cài.
- [PG-002](/articles/postgresql-cai-dat-lab-psql-ket-noi): lab tương ứng của series PostgreSQL.

Bài tiếp theo: [MS-003 · Bảng, khoá chính và clustered index trong nghiệp vụ chuyến xe](/articles/mssql-bang-khoa-chinh-clustered-index).

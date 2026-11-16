# Lab SQL Server 2022 · Tới Nơi

Mã thực hành cho loạt bài **SQL Server 2022 qua dự án Tới Nơi**. Đọc cùng [khung chương trình](../../docs/mssql-master-curriculum.md) và [hồ sơ dự án mẫu Tới Nơi](../../api/seed/content/du-an-mau-toi-noi.md). Tiền là số nguyên đồng, thời điểm là `datetimeoffset` có độ lệch `+07:00`; mọi tên người, số điện thoại, biển số đều **hư cấu**.

| Phần | Chủ đề | Thư mục | Trạng thái |
|---|---|---|---|
| P01 | Nền tảng và mô hình dữ liệu | [`p01/`](p01/README.md) | 8 file SQL, `OUTPUT.txt`, thí nghiệm, test — **đã chạy trên SQL Server 2022 CU28 thật** (80 PASS, 0 FAIL) |
| P02 | T-SQL cho nghiệp vụ | [`p02/`](p02/README.md) | 14 file SQL trên lược đồ P01, dữ liệu 30 ngày giống hệt lab PostgreSQL, `OUTPUT.txt`, test — **đã chạy trên SQL Server 2022 CU28 thật** (331 PASS, 0 FAIL) |

## Phiên bản cố định

| Thành phần | Phiên bản | Ghi chú |
|---|---|---|
| Docker Desktop | đã kiểm 29.x (engine Linux) | SQL Server cần ~2 GB RAM cho container (`mem_limit: 2g`) |
| Ảnh SQL Server | `mcr.microsoft.com/mssql/server:2022-latest@sha256:4525e3ef…` | = **16.0.4325.4, RTM-CU28**, Developer Edition, Ubuntu 22.04. Ghim digest để mọi người chạy cùng bản; đổi CU thì chạy lại `npm test` và cập nhật `OUTPUT.txt` |
| sqlcmd | `/opt/mssql-tools18/bin/sqlcmd` có sẵn trong ảnh | Không cần cài trên máy |
| Node.js | ≥ 22 (đã kiểm 22.18.0) | Chỉ để chạy `run.mjs` và test; không có dependency npm |
| Công cụ GUI | DBStudio 0.1.7 | Tuỳ chọn, để xem và chạy truy vấn bằng giao diện (MS-002) |

## Chạy

```sh
cd examples/toi-noi-mssql
npm run db:up        # docker compose up -d --wait — chờ healthcheck "healthy"
npm run p01          # 00 → 07 trên database ToiNoi mới, in kết quả + bảng đếm
npm run p02          # lược đồ P01 + p02/00 → 20 (dữ liệu 30 ngày), ~1 phút
npm test             # chạy lại p01 và p02, so với OUTPUT.txt (tuần tự, ~1 phút)
npm run db:down      # docker compose down -v — dừng và XOÁ dữ liệu lab
```

Kết quả mong đợi của `npm run p01` kết thúc bằng:

```text
-- [lab] File đã chạy: 8
-- [lab] PASS: 80
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

**Runner.** `run.mjs` gọi `docker compose exec -T db sqlcmd … -i /lab/<file>` cho từng file (thư mục `p01`, `p02` được mount chỉ đọc vào `/lab/p01`, `/lab/p02`). Tên database truyền vào script qua biến sqlcmd `$(LAB_DB)`, mặc định `ToiNoi`; `LAB_DB=ToiNoi_thu node run.mjs …` chạy trên database riêng mà không đụng tới lab chính. Mỗi file là một phiên kết nối riêng. Tuỳ chọn sqlcmd: `-C` (tin chứng chỉ tự ký của container), `-b` (lỗi không bắt ⇒ dừng file, runner báo FAIL), `-I` (QUOTED_IDENTIFIER ON), `-W -s "|"` (cột gọn, ngăn bằng `|`), `-f i:65001,o:65001` (UTF-8 cho chữ tiếng Việt).

**Kiểm tra trong SQL.** `00-reset.sql` tạo schema `lab` với hai thủ tục: `lab.expect_error` (chạy một câu trong transaction rồi ROLLBACK, PASS khi lỗi đúng số hiệu và đúng tên ràng buộc) và `lab.expect_value`. Runner đếm các dòng bắt đầu bằng `PASS ` / `FAIL `.

## Kết nối từ máy của bạn

| Tham số | Giá trị |
|---|---|
| Máy chủ / cổng | `127.0.0.1` / `14330` (chỉ mở trên máy của bạn) |
| Đăng nhập | `sa` / `ToiNoi_lab_2022!` — mật khẩu giữ chỗ cho lab, không dùng ở đâu khác |
| TLS | Chứng chỉ tự ký: DBStudio chọn chế độ TLS "Ưu tiên" hoặc "Bắt buộc" (không xác minh chứng chỉ); sqlcmd thêm `-C`; ADO.NET thêm `TrustServerCertificate=True` |
| Database | `ToiNoi` — sau `npm test` ở trạng thái cuối của P02 |

Chuỗi dán vào ô "URI kết nối" của DBStudio:

```text
Server=127.0.0.1,14330;Database=ToiNoi;User Id=sa;TrustServerCertificate=True;
```

## Dọn môi trường

`npm run db:down` xoá container và volume `toinoi-mssql2022-data`. Ảnh Docker (~2,2 GB) vẫn còn; xoá bằng `docker image rm mcr.microsoft.com/mssql/server:2022-latest` nếu không dùng nữa.

# P04 · Thí nghiệm của bài (PG-031 … PG-040)

Nơi người viết bài đặt thí nghiệm, lời giải bài tập và phép thử phụ trích trong bài. Mỗi tệp chạy trên một **database mới chép từ nền** khai trong `manifest.json` (cùng nền với `p04/lab.json`: lược đồ P01 + P02 + `p04/00`–`02`), nên tệp được tự do sửa dữ liệu mà không ảnh hưởng tệp khác. Nền dựng ~1 s.

```sh
npm run p04:thu-nghiem                                                # mọi tệp trong manifest
node pg/run-pg.mjs p04/thu-nghiem/manifest.json --only pg034          # chỉ tệp có tên chứa "pg034"
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/moi.sql      # thử một tệp CHƯA khai trong manifest
```

`npm test` chạy mọi mục của manifest (test `test/p04.test.mjs`, skip khi máy không có PostgreSQL 18).

## Thêm một thí nghiệm

1. Tạo `pg0NN-<mo-ta>.sql` trong thư mục này. MỖI phiên cần `SET search_path TO toinoi, public;` ở bước đầu của nó (mỗi phiên là một kết nối mới).
2. Nhiều phiên: đặt `-- @A`, `-- @B`… ngay trước bước (bước không ghi phiên chạy trên phiên của bước trước). Runner gửi từng bước theo thứ tự trong tệp; bước đang chờ khoá được ghi "⏳" và runner đi tiếp; kết quả của nó in ở dòng "✔" sau bước làm nó được nhả.
3. Ghi chỉ thị kiểm cho mọi lỗi cố ý và kết quả then chốt — chỉ thị đặt ở bước của ĐÚNG phiên, kể cả khi bước đó chờ rồi mới lỗi:
   - `-- expect-blocked` — bước phải rơi vào chờ khoá; kết hợp `-- expect-error: 40001` / `-- expect-value: …` để chấm kết quả khi nó chạy xong.
   - `-- lab: cho-xong` — CHỜ bước xong ngay (không đi tiếp) — cho bước tự kết thúc bằng lỗi sau một khoảng: `lock_timeout` (`55P03`), `statement_timeout` (`57014`), deadlock (`40P01`). Kèm `expect-blocked` ⇒ kiểm nó đã từng chờ khoá.
   - `-- expect-mat-ket-noi` — bước làm phiên mất kết nối (phiên bị `pg_terminate_backend`, `idle_in_transaction_session_timeout`, `transaction_timeout`). Bước sau của phiên đó tự mở kết nối MỚI (nhớ `SET search_path` lại).
   - `-- lab: nhat-ky` — in các dòng mới của nhật ký server (PID đổi thành nhãn phiên, xid thành `<XID>`, thời gian thành `T`).
   - In xid: gọi `SELECT lab_dat_moc_xid();` đầu tệp rồi dùng `xid_td(xmin)`, `snap_td(pg_current_snapshot())`; khoá/chờ: `SELECT * FROM lab_khoa();`, `SELECT * FROM lab_cho();` (p04/00).
4. Thêm một dòng vào `"tep"` của `manifest.json`: `{ "file": "pg0NN-….sql", "bai": "PG-0NN", "nen": "p04" }`.
5. Chạy `node pg/run-pg.mjs p04/thu-nghiem/manifest.json --only pg0NN` tới khi ĐẠT, chạy HAI lần để chắc lịch chạy tất định.

## Các tệp hiện có

| Tệp | Bài | Phiên | Minh hoạ | Số đáng trích |
|---|---|---|---|---|
| `pg031-nhan-chuyen-nguyen-tu-now.sql` | PG-031 | A, B | Lời giải 3 bài tập: nhận chuyến UPDATE + INSERT, gõ sai `trip_id` ⇒ `23503`, COMMIT trả `ROLLBACK`; INSERT nhiều dòng nguyên tử (`23514`); `now()` / `statement_timestamp()` / `clock_timestamp()` | `requested \| t \| 1` → `accepted \| f \| 2 \| t` |
| `pg032-snapshot-dang-chay-rr-delete.sql` | PG-032 | A, B | Snapshot có danh sách đang chạy khác rỗng; Repeatable Read chụp snapshot ở câu đầu tiên, không ở BEGIN; DELETE chỉ đặt xmax, pgstattuple trước/sau VACUUM | `xmin=X+1 xmax=X+3 đang chạy={X+1}` |
| `pg033-go-roi-phien-giu-khoa.sql` | PG-033 | A, B, C, D | Chuỗi chờ A ← B ← C qua `pg_stat_activity` + `pg_blocking_pids`, tìm phiên gốc; `pg_cancel_backend` (57014) rồi `pg_terminate_backend`; advisory lock mức phiên giữ sau ROLLBACK | C xong với 38500 |
| `pg033-transaction-timeout.sql` | PG-033 | A, B | Bốn loại "hết giờ" (lock_timeout, statement_timeout, idle_in_transaction_session_timeout, **transaction_timeout — PG 17+**): `transaction_timeout = 300ms` cắt KẾT NỐI của giao dịch đang chạy `pg_sleep(1)` | nhật ký `[A] FATAL: terminating connection due to transaction timeout`; cước 400101 vẫn 31000 |
| `pg034-rc-update-khong-thay-dong-moi-khop.sql` | PG-034 | A, B | Read Committed: UPDATE gặp dòng bị khoá ⇒ chờ ⇒ đọc lại ĐÚNG dòng đó; dòng vừa bị A đổi để TRỞ NÊN khớp (400101 chuyển sang khu 113) bị BỎ SÓT | B cập nhật 400105, 400108 (2 dòng); 400101 ở khu 113 vẫn 31000 |
| `pg034-snapshot-cau-dau-va-bai-tap.sql` | PG-034 | A, B, C | READ UNCOMMITTED không đọc bẩn; RR chụp ở câu đầu; người ghi trước ROLLBACK ⇒ không 40001; lời giải S3 (`FOR NO KEY UPDATE`) và S4 (RR thử lại); `SET TRANSACTION` muộn ⇒ 25001 | 33000; 550000; 570000 |
| `pg035-version-rr-ham-luu-updated-at-xmin.sql` | PG-035 | A, B | Hai lần lưu chồng nhau ở RR ⇒ 40001; hàm `luu_cuoc` ba kết quả; `updated_at = now()` đi lùi; dùng `xmin` làm version | `da_luu` / `xung_dot` / `khong_ton_tai`; `38500 \| t` |
| `pg036-lease-fencing-lo-viec-idle-timeout.sql` | PG-036 | A, B, C | Thời hạn thuê kèm «vé» `attempts`; lấy theo lô `LIMIT 3`; `pgrowlocks` thấy khoá dòng; `idle_in_transaction_session_timeout` cắt worker treo (phụ thuộc thời gian 1 s) | `done \| 2 \| A,B`; A 2,3,4 · B 5,6,7 |
| `pg037-vong-ba-phien-lock-timeout.sql` | PG-037 | A, B, C | Vòng chờ ba phiên A→B→C→A (DETAIL 40P01 đủ ba process); sau 55P03 câu kế 25P02, COMMIT trả ROLLBACK; `pg_settings` ba tham số khoá; `LockRows` trên `Sort` | 148 bước |
| `pg038-siread-retry-sai-muc-tron.sql` | PG-038 | A, B | SIReadLock trong `pg_locks` (page index + tuple); hai SERIALIZABLE sửa cùng dòng ⇒ 40001 ở câu UPDATE; trộn mức cô lập ⇒ write skew lọt; retry sai chỉ gửi lại UPDATE | khu 111 còn 0 người |
| `pg039-idempotency-outbox-bai-tap.sql` | PG-039 | A, B, C | Băm `jsonb` không phụ thuộc thứ tự khoá; thiết kế hai giao dịch để lại khoá mồ côi; hai consumer cùng một tin | `lap_lai: true`; `P0001`; `bỏ qua (đã xử lý)` |
| `pg040-nhan-chuyen-bai-tap.sql` | PG-040 | A, B, C | Cách (a) ở RR ⇒ 40001; tài xế bấm «Nhận» hai lần với cách (b); thêm UNIQUE cho `trip_claims` rồi chạy lại lịch (a) | `23505 ux_trip_claims_trip` |

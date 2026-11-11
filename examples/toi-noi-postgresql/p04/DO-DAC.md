# P04 · Phép thử đồng thời (PG-036 … PG-040)

> Tệp này do `npm run p04:measure` (`node pg/run-dong-thoi.mjs p04/dong-thoi.mjs`) GHI RA — đừng sửa tay; chạy lại lệnh để đo lại.
> Số đếm dưới đây là kết quả THẬT của một lần chạy lệnh trên máy ở mục 1. Các phiên chạy đua thật nên số đếm (lượt thắng, số lần thử lại, số lỗi) DAO ĐỘNG giữa các lần chạy và giữa các máy; chỉ các BẤT BIẾN được kiểm cứng. Trích số đếm thì trích kèm khoảng min–max và cấu hình máy.

## 1. Máy và cấu hình

| Mục | Giá trị |
|---|---|
| Thời điểm chạy | 2026-11-11 12:54 UTC |
| CPU | Intel(R) Core(TM) i7-10700 CPU @ 2.90GHz · 16 luồng logic |
| RAM | 64 GB |
| Hệ điều hành | Windows 11 Pro (win32 10.0.26300, x64) |
| Ổ đĩa chứa cluster tạm | C: KINGSTON SA2000M81000G | SSD | NVMe | 932 GB |
| PostgreSQL | 18.1 (C:\Program Files\PostgreSQL\18\bin) |
| Node.js | v22.18.0 |
| Cluster tạm | listen_addresses=localhost, max_connections=40, shared_buffers=128MB, work_mem=4MB, hash_mem_multiplier=2, maintenance_work_mem=64MB, effective_cache_size=4GB, random_page_cost=4, seq_page_cost=1, effective_io_concurrency=16, jit=off, max_parallel_workers_per_gather=0, max_parallel_maintenance_workers=0, default_statistics_target=100, autovacuum=off, track_io_timing=off, max_wal_size=4GB, checkpoint_timeout=30min, log_checkpoints=off, log_temp_files=-1, shared_preload_libraries=pg_stat_statements, timezone=Asia/Ho_Chi_Minh |
| Cỡ | đầy đủ |
| Tổng thời gian lệnh | 7,4 phút |
| Tải CPU toàn máy (trung bình 3 s) | trước 6 % · sau 6 % |

**Cách chạy.** Mỗi kịch bản mở N phiên psql THẬT (N kết nối, N backend) tới cluster tạm và cho chúng chạy ĐUA:
Node điều khiển từng phiên qua ống (gửi một câu, chờ kết quả), không có khoảng nghỉ nhân tạo trừ chỗ ghi rõ.
Với K040 và K038, các phiên bắt đầu MỖI chuyến / MỖI vòng cùng lúc ("rào" — chờ nhau xong mới sang chuyến/vòng
sau) để tranh chấp xảy ra trên từng đơn vị. Mỗi LẦN chạy trên một database mới chép từ nền p04. Thử lại phía
client (`giaoDich()` trong `pg/lib/dong-thoi.mjs`): lỗi 40001/40P01 ⇒ ROLLBACK, chờ ngẫu nhiên
"full jitter" trong [0, min(trần, 2 ms · 2^lần)] rồi chạy lại TOÀN BỘ giao dịch, tối đa số lần ghi ở từng kịch bản.
Cột đếm `loi_<SQLSTATE>` đếm MỌI lần câu lệnh lỗi (kể cả lần sau đó thử lại thành công); `thu_lai` = số lần
chạy lại; `het_luot_thu` = giao dịch bỏ cuộc vì hết lượt. **Bất biến** kiểm bằng SQL sau mỗi lần (hoặc từ bộ
đếm phía client) — đó là phần được `npm test` kiểm cứng (bản nhỏ `--nho`).

## 2. Kết quả theo kịch bản

### K040-a · PG-040 · Nhận chuyến (a): đọc rồi ghi, không khoá, không điều kiện

Đúng code của 40-thuc-hanh-nhan-chuyen.sql cách (a): `SELECT status` ⇒ nếu 'requested' thì `UPDATE` không
điều kiện + ghi claim. CỐ Ý SAI — bất biến "đúng một người thắng" được đếm số vi phạm để minh hoạ.

Tải: chuyen = 1000 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | chuyen_nhieu_nguoi_thang | luot_thang | luot_thua |
|---:|---:|---:|---:|---:|
| 1 | 7,33 | 1.000 | 8.000 | 0 |
| 2 | 7,26 | 1.000 | 8.000 | 0 |
| 3 | 7,26 | 1.000 | 8.000 | 0 |
| min–max | 7,26–7,33 | 1.000–1.000 | 8.000–8.000 | 0–0 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| chuyến có > 1 người thắng | minh hoạ, đúng là = 0 | 1000 · 1000 · 1000 | VI PHẠM — minh hoạ cách làm sai |
| chuyến không ai thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| người được báo thắng ≠ trips.driver_id | minh hoạ, đúng là = 0 | 7000 · 7000 · 7000 | VI PHẠM — minh hoạ cách làm sai |
| tổng lượt "bạn đã nhận chuyến" | ghi nhận | 8000 · 8000 · 8000 | chỉ ghi nhận |

### K040-b · PG-040 · Nhận chuyến (b): UPDATE … WHERE status = 'requested'

Một câu `WITH nhan AS (UPDATE … WHERE code = … AND status = 'requested' RETURNING id) INSERT INTO trip_claims …`
(autocommit).

Tải: chuyen = 1000 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | luot_thang | luot_thua |
|---:|---:|---:|---:|
| 1 | 1,40 | 1.000 | 7.000 |
| 2 | 1,48 | 1.000 | 7.000 |
| 3 | 1,32 | 1.000 | 7.000 |
| min–max | 1,32–1,48 | 1.000–1.000 | 7.000–7.000 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| chuyến có > 1 người thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| chuyến không ai thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| người được báo thắng ≠ trips.driver_id | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| tổng lượt "bạn đã nhận chuyến" | ghi nhận | 1000 · 1000 · 1000 | chỉ ghi nhận |

### K040-c · PG-040 · Nhận chuyến (c): SELECT … FOR UPDATE rồi kiểm rồi ghi

Giao dịch Read Committed: `SELECT status … FOR UPDATE` ⇒ kiểm trong ứng dụng ⇒ UPDATE + claim.

Tải: chuyen = 1000 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | luot_thang | luot_thua |
|---:|---:|---:|---:|
| 1 | 4,00 | 1.000 | 7.000 |
| 2 | 3,93 | 1.000 | 7.000 |
| 3 | 3,92 | 1.000 | 7.000 |
| min–max | 3,92–4,00 | 1.000–1.000 | 7.000–7.000 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| chuyến có > 1 người thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| chuyến không ai thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| người được báo thắng ≠ trips.driver_id | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| tổng lượt "bạn đã nhận chuyến" | ghi nhận | 1000 · 1000 · 1000 | chỉ ghi nhận |

### K040-d · PG-040 · Nhận chuyến (d): khoá chính trip_assignments làm trọng tài

`INSERT INTO trip_assignments … ON CONFLICT (trip_id) DO NOTHING`; chèn được (1 dòng) mới UPDATE trips + claim.

Tải: chuyen = 1000 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | luot_thang | luot_thua |
|---:|---:|---:|---:|
| 1 | 3,34 | 1.000 | 7.000 |
| 2 | 3,45 | 1.000 | 7.000 |
| 3 | 3,86 | 1.000 | 7.000 |
| min–max | 3,34–3,86 | 1.000–1.000 | 7.000–7.000 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| chuyến có > 1 người thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| chuyến không ai thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| người được báo thắng ≠ trips.driver_id | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| tổng lượt "bạn đã nhận chuyến" | ghi nhận | 1000 · 1000 · 1000 | chỉ ghi nhận |

### K040-e · PG-040 · Nhận chuyến (e): code (a) nhưng REPEATABLE READ + thử lại

Đúng code SAI của (a) nhưng chạy ở REPEATABLE READ, thử lại khi 40001 (tối đa 10 lần): người ghi sau nhận 40001,
chạy lại thì đọc thấy 'accepted' ⇒ thôi. Cơ sở dữ liệu biến lỗi logic thành lỗi có thể thử lại.

Tải: chuyen = 1000 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | loi_40001 | luot_thang | luot_thua | thu_lai |
|---:|---:|---:|---:|---:|---:|
| 1 | 16,97 | 7.000 | 1.000 | 7.000 | 7.000 |
| 2 | 15,48 | 7.000 | 1.000 | 7.000 | 7.000 |
| 3 | 17,04 | 7.000 | 1.000 | 7.000 | 7.000 |
| min–max | 15,48–17,04 | 7.000–7.000 | 1.000–1.000 | 7.000–7.000 | 7.000–7.000 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| chuyến có > 1 người thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| chuyến không ai thắng | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| người được báo thắng ≠ trips.driver_id | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| tổng lượt "bạn đã nhận chuyến" | ghi nhận | 1000 · 1000 · 1000 | chỉ ghi nhận |

### K036-skip · PG-036 · Hàng đợi: FOR UPDATE SKIP LOCKED

Mỗi worker lặp: giao dịch [lấy việc kế tiếp `… FOR UPDATE SKIP LOCKED` ⇒ "xử lý" (pg_sleep) ⇒ ghi job_runs ⇒
đánh dấu done ⇒ COMMIT] tới khi hết việc. `rong_gia` = số lần SKIP LOCKED trả rỗng trong khi vẫn còn việc
(đang bị worker khác khoá) — worker phải hỏi lại.

Tải: viec = 600 · phien = 8 · xu_ly_ms = 2 · 3 lần.

| Lần | Thời gian (s) | it_nhat_moi_worker | nhieu_nhat_moi_worker | rong_gia | xu_ly | việc/giây |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1,32 | 74 | 76 | 2 | 600 | 456 |
| 2 | 1,21 | 74 | 76 | 7 | 600 | 496 |
| 3 | 1,21 | 74 | 76 | 6 | 600 | 496 |
| min–max | 1,21–1,32 | 74–74 | 76–76 | 2–7 | 600–600 |  |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| việc chưa xong | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| việc xử lý ≠ 1 lần | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |

### K036-cho · PG-036 · Hàng đợi: FOR UPDATE (chờ)

Như trên nhưng `FOR UPDATE` (không SKIP LOCKED): mọi worker cùng nhắm vào việc đầu hàng ⇒ xếp hàng chờ nhau.

Tải: viec = 600 · phien = 8 · xu_ly_ms = 2 · 3 lần.

| Lần | Thời gian (s) | it_nhat_moi_worker | nhieu_nhat_moi_worker | xu_ly | việc/giây |
|---:|---:|---:|---:|---:|---:|
| 1 | 9,36 | 0 | 115 | 600 | 64 |
| 2 | 9,42 | 0 | 132 | 600 | 64 |
| 3 | 9,37 | 2 | 159 | 600 | 64 |
| min–max | 9,36–9,42 | 0–2 | 115–159 | 600–600 |  |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| việc chưa xong | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| việc xử lý ≠ 1 lần | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |

### K036-nowait · PG-036 · Hàng đợi: FOR UPDATE NOWAIT + thử lại sau 1 ms

Như trên nhưng `FOR UPDATE NOWAIT`: bị khoá ⇒ lỗi 55P03 ngay ⇒ worker ROLLBACK, nghỉ 1 ms rồi hỏi lại.

Tải: viec = 600 · phien = 8 · xu_ly_ms = 2 · 3 lần.

| Lần | Thời gian (s) | it_nhat_moi_worker | loi_55P03 | nhieu_nhat_moi_worker | xu_ly | việc/giây |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 9,30 | 56 | 25.984 | 85 | 600 | 65 |
| 2 | 9,30 | 65 | 25.317 | 91 | 600 | 65 |
| 3 | 9,31 | 70 | 26.774 | 78 | 600 | 64 |
| min–max | 9,30–9,31 | 56–70 | 25.317–26.774 | 78–91 | 600–600 |  |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| việc chưa xong | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| việc xử lý ≠ 1 lần | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |

### K037-theo-chieu · PG-037 · Chuyển tiền: khoá theo chiều chuyển (trừ ví nguồn trước)

Mỗi giao dịch: trừ 1.000 đ ví nguồn, nghỉ 1 ms (`pg_sleep`, giả lập ứng dụng làm việc giữa hai câu), cộng ví
đích; cặp ví và chiều ngẫu nhiên trong 4 ví. `deadlock_timeout = 20ms` (để chạy nhanh); 40P01 ⇒ thử lại ≤ 20 lần.

Tải: lan_moi_phien = 150 · phien = 8 · vi = 4 · nghi_giua_ms = 1 · 3 lần.

| Lần | Thời gian (s) | chuyen_xong | loi_40P01 | thu_lai |
|---:|---:|---:|---:|---:|
| 1 | 32,86 | 1.200 | 1.184 | 1.184 |
| 2 | 30,77 | 1.200 | 1.122 | 1.122 |
| 3 | 32,59 | 1.200 | 1.184 | 1.184 |
| min–max | 30,77–32,86 | 1.200–1.200 | 1.122–1.184 | 1.122–1.184 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| tổng tiền 4 ví | phải = 800000 | 800000 · 800000 · 800000 | ĐẠT mọi lần |
| giao dịch chuyển thất bại sau 20 lần thử | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| số lần bị huỷ vì deadlock (40P01) | ghi nhận | 1184 · 1122 · 1184 | chỉ ghi nhận |

### K037-theo-id · PG-037 · Chuyển tiền: khoá cả hai ví theo thứ tự id trước

Như trên, nhưng mở đầu bằng `SELECT id FROM wallets WHERE id IN (a, b) ORDER BY id FOR NO KEY UPDATE`.

Tải: lan_moi_phien = 150 · phien = 8 · vi = 4 · nghi_giua_ms = 1 · 3 lần.

| Lần | Thời gian (s) | chuyen_xong |
|---:|---:|---:|
| 1 | 14,94 | 1.200 |
| 2 | 14,94 | 1.200 |
| 3 | 14,53 | 1.200 |
| min–max | 14,53–14,94 | 1.200–1.200 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| tổng tiền 4 ví | phải = 800000 | 800000 · 800000 · 800000 | ĐẠT mọi lần |
| giao dịch chuyển thất bại sau 20 lần thử | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| số lần bị huỷ vì deadlock (40P01) | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |

### K038-rr · PG-038 · Xin nghỉ trực: REPEATABLE READ (write skew lọt qua)

Mỗi vòng: đặt mọi người trực lại; mỗi phiên i (tài xế 9101+i, 2 người một khu) cùng lúc chạy giao dịch
"đếm người đang trực trong khu ⇒ nếu ≥ 2 thì tắt trực của mình". Sau vòng, đếm khu không còn ai trực.

Tải: vong = 200 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | duoc_nghi | khu_vi_pham |
|---:|---:|---:|---:|
| 1 | 0,89 | 1.600 | 800 |
| 2 | 0,87 | 1.600 | 800 |
| 3 | 0,90 | 1.600 | 800 |
| min–max | 0,87–0,90 | 1.600–1.600 | 800–800 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| khu không còn ai trực (cộng qua các vòng) | minh hoạ, đúng là = 0 | 800 · 800 · 800 | VI PHẠM — minh hoạ cách làm sai |
| giao dịch bỏ cuộc sau hết lượt thử | ghi nhận | 0 · 0 · 0 | chỉ ghi nhận |

### K038-ser · PG-038 · Xin nghỉ trực: SERIALIZABLE + thử lại (Seq Scan)

Như trên ở SERIALIZABLE, thử lại khi 40001 tối đa 10 lần (backoff full jitter, trần 50 ms). Bảng 24 dòng ⇒ planner
chọn Seq Scan ⇒ khoá vị từ cả bảng ⇒ phiên ở KHU KHÁC cũng xung đột (báo động giả) ⇒ nhiều lần thử lại.

Tải: vong = 200 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | duoc_nghi | khu_vi_pham | loi_40001 | thu_lai | tu_choi |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 11,13 | 800 | 0 | 2.539 | 2.539 | 800 |
| 2 | 10,34 | 800 | 0 | 2.488 | 2.488 | 800 |
| 3 | 9,61 | 800 | 0 | 2.498 | 2.498 | 800 |
| min–max | 9,61–11,13 | 800–800 | 0–0 | 2.488–2.539 | 2.488–2.539 | 800–800 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| khu không còn ai trực (cộng qua các vòng) | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| giao dịch bỏ cuộc sau hết lượt thử | ghi nhận | 0 · 0 · 0 | chỉ ghi nhận |

### K038-ser-index · PG-038 · Xin nghỉ trực: SERIALIZABLE + thử lại (ép dùng index)

Như K038-ser nhưng mỗi phiên `SET enable_seqscan = off` ⇒ đọc qua index ix_driver_duty_area ⇒ khoá vị từ
mịn (page index + dòng) ⇒ chỉ hai người CÙNG khu xung đột.

Tải: vong = 200 · phien = 8 · 3 lần.

| Lần | Thời gian (s) | duoc_nghi | khu_vi_pham | loi_40001 | thu_lai | tu_choi |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 3,18 | 800 | 0 | 800 | 800 | 800 |
| 2 | 3,48 | 800 | 0 | 800 | 800 | 800 |
| 3 | 3,53 | 800 | 0 | 800 | 800 | 800 |
| min–max | 3,18–3,53 | 800–800 | 0–0 | 800–800 | 800–800 | 800–800 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| khu không còn ai trực (cộng qua các vòng) | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| giao dịch bỏ cuộc sau hết lượt thử | ghi nhận | 0 · 0 · 0 | chỉ ghi nhận |

### K039 · PG-039 · Request lặp + request chết giữa chừng + publisher chết giữa chừng + 2 consumer

Ba pha. (1) **Request**: mỗi khoá idempotency được HAI phiên gửi đồng thời (bấm hai lần); với xác suất
`p_chet`, một request mở giao dịch, gọi `dat_chuyen()` rồi CHẾT trước COMMIT (giết tiến trình psql) —
client sau đó gửi lại cùng khoá. (2) **Publisher**: một nửa số phiên lấy outbox bằng `FOR UPDATE SKIP LOCKED`
(lô 5), "gửi" sang broker bằng kết nối RIÊNG (autocommit), rồi đánh dấu `published_at`; với xác suất `p_chet`
publisher chết sau khi gửi, trước khi đánh dấu ⇒ lô đó bị gửi lại. (3) **Consumer**: HAI consumer cùng nhận MỌI
tin của broker (như broker giao lại cho instance khác), chống trùng bằng `processed_messages`.

Tải: khoa = 400 · phien = 8 · p_chet = 0.1 · 3 lần.

| Lần | Thời gian (s) | consumer_bo_qua | consumer_xu_ly | publisher_chet | request_chet | tao_chuyen_moi | tin_gui_broker | tra_lai_ket_qua_cu |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 8,99 | 520 | 400 | 12 | 80 | 400 | 460 | 400 |
| 2 | 8,29 | 550 | 400 | 15 | 79 | 400 | 475 | 400 |
| 3 | 7,95 | 490 | 400 | 9 | 78 | 400 | 445 | 400 |
| min–max | 7,95–8,99 | 490–550 | 400–400 | 9–15 | 78–80 | 400–400 | 445–475 | 400–400 |

| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |
|---|---|---|---|
| khoá không có đúng 1 chuyến | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| chuyến tạo ra (phải = số khoá) | phải = 400 | 400 · 400 · 400 | ĐẠT mọi lần |
| chuyến thiếu/thừa sự kiện hoặc outbox | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| outbox chưa gửi | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| sự kiện không có đúng 1 thông báo | phải = 0 | 0 · 0 · 0 | ĐẠT mọi lần |
| tin TRÙNG ở broker (giao ít nhất một lần) | ghi nhận | 60 · 75 · 45 | chỉ ghi nhận |

## 3. Giới hạn của các số đếm

- Số đếm (lượt thắng, số lần thử lại, số deadlock, tin trùng, việc/giây) phụ thuộc lịch hệ điều hành, số lõi,
  tải máy và độ trễ ống psql ↔ Node; chạy lại sẽ khác. So SỰ CÓ MẶT/VẮNG MẶT của vi phạm và độ lớn tương đối,
  không so từng con số.
- "Không vi phạm trong N lần" của một cách làm KHÔNG chứng minh cách đó đúng — chỉ cách có lập luận (khoá, ràng
  buộc, SSI) mới bảo đảm; phép thử chỉ là bằng chứng bổ sung. Ngược lại, MỘT lần vi phạm là đủ bác bỏ.
- Mỗi phiên gửi từng câu qua psql (~0,3 ms/lượt trên máy lab): khoảng hở giữa "đọc" và "ghi" lớn hơn một ứng dụng
  gửi lô câu lệnh, nhỏ hơn ứng dụng gọi mạng giữa chừng. Tỉ lệ vi phạm thật ở production có thể cao hoặc thấp hơn.
- Thời gian/thông lượng đo trên một máy, có tiến trình khác chạy nền — chỉ dùng so sánh tương đối trong cùng bảng.
- K036 mô phỏng "xử lý" bằng `pg_sleep(0,002)` khi đang giữ khoá; trên Windows bước ngủ thực tế có thể dài hơn
  2 ms (độ phân giải bộ định thời).

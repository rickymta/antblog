# P06 · Mở rộng, tích hợp và vận hành đầu cuối (PG-051–060)

Lab chạy **PostgreSQL thật**: 18.1 cho mọi cluster chính và 17.7 làm nguồn cho bài nâng phiên bản (PG-059). PG-057 (PostGIS, pgvector) chạy riêng trên **PGlite**, vì bản PostgreSQL 18 cho Windows dùng ở các phần trước không kèm hai extension này. Mọi cluster do runner tạo đều tạm, chỉ nghe localhost và bị xoá khi kết thúc, kể cả khi assertion thất bại.

## Chạy

Tại `examples/toi-noi-postgresql`:

```powershell
# PostgreSQL 18 (mặc định C:\Program Files\PostgreSQL\18\bin, hoặc đặt PG_BIN)
# PostgreSQL 17: chỉ cần thư mục bin, không cần cài service. Ví dụ bản zip "binaries" của EDB, giải nén ra.
$env:PG17_BIN = 'D:\pg17\pgsql\bin'
npm run p06

# PG-057: cài dependency PGlite một lần (theo package-lock), rồi chạy
npm run p06:extensions:install
npm run p06:extensions
```

Cần Node 22+. Runner chính dùng initdb, pg_ctl, psql, pg_basebackup, pg_rewind, pg_dump, pg_restore và pg_upgrade của bản 18; initdb, pg_ctl, postgres của bản 17. Đường Linux/macOS chưa được chạy trong đợt viết này. Control plane của lab dùng `trust` trên loopback (như P03–P05): chỉ chạy trên máy cá nhân, không phải cấu hình production.

`run.mjs` lấy binary qua `pg/lib/bin.mjs`, tạo cluster bằng `pg/lib/cluster.mjs` (cấu hình chung LAB_SETTINGS), mở phiên đồng thời bằng `PsqlSession`. Override riêng của P06: primary dùng `wal_level=logical`, `autovacuum=on`, `wal_keep_size=512MB` (để pg_rewind còn đủ WAL quay về checkpoint chung); cluster 17 đặt `effective_io_concurrency=0` (Windows không có posix_fadvise) và tắt `shared_preload_libraries`.

## Dữ liệu

`fixture.sql` tạo:

- `gps`: 1.464 điểm GPS theo giờ của 20 tài xế, 01/10–30/11/2026 UTC, phân vùng theo tháng (`gps_oct` 744, `gps_nov` 720); `attrs jsonb` gồm `source` và `accuracy_m` (5–14 m).
- `gps_plain`: cùng dữ liệu trên bảng thường, cùng khoá chính `(driver_id, seen_at)`.
- `fares`: sổ cước rút gọn (VND), bắt đầu 2 chuyến / 150.000 đ; `fare_report`: materialized view tổng theo thành phố, có cột `refreshed_at`.

Sổ cước đi xuyên suốt runner, nên thứ tự chạy là một phần của phép thử:

| Bước | Thay đổi trên `fares` | Sổ sau bước |
|---|---|---|
| 056 | Thêm chuyến 3 (HCM, 50.000); chuyến tạm 99 (DN, 1.000) thêm trước REFRESH CONCURRENTLY rồi xoá | 3 chuyến / 200.000 |
| 055 | Sửa chuyến 3 thành 55.000; thêm cột `tip`, chuyến 1 tip 5.000 | 3 / 205.000 |
| 052 | Thêm chuyến 4 (HN, 30.000) | 4 / 235.000 |
| 058 | Chuyến 5 ghi lên primary cũ khi standby bị cắt → **mất** khi failover; chuyến 6 (HCM, 15.000) ghi lên primary mới; chuyến 7 ghi lên primary cũ khi split-brain → bị pg_rewind loại | 5 / 250.000 |

## Phép kiểm theo bài

| Bài | Phép kiểm trong `run.mjs` |
|---|---|
| 051 | Số dòng mỗi partition, điểm biên 00:00 UTC 1/11 vào `gps_nov`; EXPLAIN tài xế + tháng chỉ chạm `gps_nov`, so với bảng thường; lọc chỉ theo tài xế quét mọi partition; INSERT ngoài mọi partition → 23514; UNIQUE thiếu khoá phân vùng → 0A000; kích thước quan hệ |
| 052 | Standby dựng bằng `pg_basebackup -R`; `pg_stat_replication`; ghi trên standby → 25006; tạm dừng replay: standby đã nhận WAL nhưng chưa thấy chuyến 4; chờ replay tới LSN commit rồi mới đọc |
| 053 | `@>` đếm 147; gõ sai tên khoá trả 0 mà không lỗi; `->>`/`->`; CHECK `jsonb_typeof` → 23514; ép kiểu lỗi lúc truy vấn → 22P02; GIN `jsonb_path_ops` (planner mặc định vẫn seq scan trên dữ liệu nhỏ; plan dùng index chỉ hiện khi tắt seq scan để minh hoạ); cột generated STORED |
| 054 | Generic plan có `Subplans Removed: 1`; hàm trên khoá phân vùng làm mất pruning; partition tháng 12 tạo trước nhận dữ liệu; WAL của DELETE 744 dòng so với DETACH CONCURRENTLY và DROP |
| 055 | Publication/subscription, initial copy; slot `pgoutput`; subscription DISABLE giữ WAL; ENABLE bắt kịp; thêm cột chỉ ở publisher làm apply lỗi tới khi subscriber có cột; DROP SUBSCRIPTION xoá slot |
| 056 | Báo cáo cũ trước REFRESH; CONCURRENTLY thiếu unique index → 55000; REFRESH thường chặn người đọc (AccessExclusiveLock); CONCURRENTLY không chặn; `refreshed_at` tăng; đối soát báo cáo với bảng gốc |
| 057 | `extensions/run.mjs`: 5.000 tài xế, ST_DWithin 500 m trước/sau GiST, KNN; bẫy geometry SRID 4326 đo bằng độ; 2.000 vector 16 chiều; 20 vector truy vấn không nằm trong bảng; top-10 chính xác so với HNSW dựng 3 lần, ở ba mức `ef_search` (HNSW chọn tầng ngẫu nhiên nên recall đổi theo lần dựng) |
| 058 | Cắt standby khỏi primary; primary vẫn nhận ghi; crash primary; promote; chuyến đã commit bị mất; timeline 2; `target_session_attrs=read-write`; primary cũ khởi động lại → hai primary cùng nhận ghi; fencing + `pg_rewind` đưa node cũ thành standby của timeline 2 |
| 059 | Nguồn 17.7 có identity, jsonb, statistics. A: pg_dump bản 18 → pg_restore vào 18, digest, identity tiếp tục, nguồn không có ghi mới, statistics chưa có. B: `pg_upgrade --check` từ chối vì checksum khác nhau; initdb lại với `--no-data-checksums`; pg_upgrade copy mode, digest, statistics được mang sang, cluster 17 vẫn khởi động được |
| 060 | Báo cáo lệch sổ sau failover (200.000 so với 250.000) tới khi REFRESH; sổ cuối và báo cáo; node đã rewind theo kịp sổ |

`OUTPUT.txt` có hai loại dòng: `PASS`/`MEASURE` là cổng kiểm và số quan sát; khối `--- 0xx …` là output psql, log server hoặc output công cụ thật mà các bài dẫn lại. Port, PID, đường dẫn tạm và thời điểm được che; LSN, tên file WAL và digest là của lần chạy đã ghi, lần chạy khác có thể khác.

## Giới hạn

- Dữ liệu nhỏ, cache ấm, một máy. Số byte WAL ở 054/055/058 là quan sát của lần chạy, không phải benchmark; WAL cấp cluster có thể lẫn công việc nền (autovacuum đang bật). Không assert số millisecond hay số byte. Ở 054, `gps_oct` (DETACH/DROP) đã có thêm index GIN và cột generated từ 053, còn `gps_plain` (DELETE) thì không: phép so sánh WAL không cùng điều kiện.
- 051/053: plan trên dữ liệu nhỏ; ở dữ liệu lớn, planner có thể chọn khác. Plan dùng GIN khi `enable_seqscan=off` chỉ chứng minh index *dùng được*, không chứng minh nó nhanh hơn.
- 052/058: replication bất đồng bộ, không có synchronous standby, không có công cụ HA (Patroni, repmgr…), không có virtual IP hay DNS. "Network partition" được giả lập bằng cách xoá `primary_conninfo` trên standby; "crash" là `pg_ctl stop -m immediate`. Fencing trong lab là dừng tay node cũ.
- 055: một bảng, một subscription, cùng máy; không đo độ trễ apply, không thử conflict hay `disable_on_error`.
- 057: PGlite (PostgreSQL 18.6 trong WASM, đơn tiến trình), không phải server; recall đo trên 20 truy vấn, 3 lần dựng index, dữ liệu tổng hợp; lần chạy khác cho số khác, và không có ý nghĩa cho embedding thật.
- 059: một bảng nhỏ; không có extension, không đo downtime, không thử `--link`/`--clone`/`--swap`, không có replica hay logical replication khi cutover.

## Dọn

Runner xoá mọi cluster trong `finally`. Không commit dump, data directory hay binary PostgreSQL 17. Xem [HO-SO-BAN-GIAO.md](HO-SO-BAN-GIAO.md) cho cổng go/no-go và việc còn thiếu trước production.

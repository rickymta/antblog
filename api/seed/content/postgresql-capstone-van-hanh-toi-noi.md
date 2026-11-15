---
slug: postgresql-capstone-van-hanh-toi-noi
title: "Capstone: vận hành dữ liệu Tới Nơi từ schema tới sự cố — hồ sơ bàn giao và cổng go/no-go"
summary: "Bài cuối của series 60 bài: đi theo sổ cước Tới Nơi qua materialized view, CDC, standby, failover mất 184 byte WAL, split-brain và pg_rewind; báo cáo lệch 200.000 so với 250.000 tới khi REFRESH. Bàn giao D1–D9, cổng go/no-go trước khi mở lại ghi và khi nâng phiên bản, bằng chứng cần giữ, tiêu chí tự chấm, tổng kết P01–P06 và danh sách trung thực những gì lab chưa chứng minh."
category: postgresql
level: advanced
tags: [postgresql, pg-060, capstone, thuc-hanh, failover, split-brain, pg-rewind, go-no-go, runbook, toi-noi]
language: vi
---

02:10, thứ Sáu. Primary của Tới Nơi ở TP.HCM ngừng phản hồi. Người trực promote standby, và bốn phút sau kênh sự cố có dòng «database đã lên». Rồi những câu hỏi khác lần lượt tới. Kế toán: dashboard doanh thu ghi 200.000 đ, còn đối soát nhanh trên sổ ra 250.000 đ — có phải mất dữ liệu, có nên restore? Một tài xế ở Hà Nội: chuyến vừa hoàn thành lúc 02:09 không còn trong lịch sử. Đội hạ tầng: máy primary cũ vừa tự khởi động lại, ứng dụng có đang ghi vào nó không?

«Database đã lên» không trả lời câu nào. Muốn trả lời, người trực cần mọi thứ series này đã xây: ràng buộc để biết dữ liệu nào hợp lệ (Phần 1), đối soát để biết số nào đúng (Phần 2), kế hoạch thực thi để truy vấn kiểm tra không làm sập máy (Phần 3), bất biến giao dịch (Phần 4), runbook phục hồi đã chạy thật (Phần 5), và replication, failover, projection (Phần 6).

Đây là bài thực hành cuối Phần 6 và là bài cuối của series: đóng gói lab P06 thành **hồ sơ bàn giao** mà người khác chạy lại được, và đặt nó cạnh hồ sơ của năm phần trước.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-060 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ PG-051 … PG-059 (danh sách ở mục 12), đặc biệt <a href="/articles/postgresql-ha-failover-read-consistency">PG-058 · Failover và split-brain</a>, <a href="/articles/postgresql-materialized-view-reporting">PG-056 · Materialized view</a> và <a href="/articles/postgresql-thuc-hanh-runbook-phuc-hoi">PG-050 · Runbook phục hồi</a>. Lab: <code>examples/toi-noi-postgresql/p06</code> — <code>run.mjs</code>, <code>OUTPUT.txt</code> (kết quả thực trên PostgreSQL 18.1, cộng 17.7 cho PG-059), <code>README.md</code> (sổ cước, phép kiểm, giới hạn), <code>HO-SO-BAN-GIAO.md</code> (D1–D9 và cổng go/no-go). Chạy tại <code>examples/toi-noi-postgresql</code>: <code>npm run p06</code> (cần PostgreSQL 18 và biến <code>PG17_BIN</code> trỏ tới thư mục bin của PostgreSQL 17). Sau bài này, bạn bàn giao được hồ sơ vận hành có bằng chứng, cổng quyết định mở lại ghi, và danh sách những gì chưa được chứng minh.</p>
</div>

## 1. Đề bài

### Bối cảnh

Runner P06 dựng các cluster **PostgreSQL 18.1 tạm**, chỉ nghe localhost, xoá trong `finally` kể cả khi assertion thất bại: một primary (`wal_level=logical`), một subscriber cho logical replication, một standby, và các cluster 17.7/18 cho bài nâng phiên bản. PG-057 chạy riêng trên PGlite. Thứ tự trong `OUTPUT.txt`: 051 → 053 → 054 → 056 → 055 → 052 → 058 → **060** → 059. Thứ tự này là một phần của phép thử: một **sổ cước** (`fares`) đi xuyên qua các bước, và capstone kiểm sổ đó sau sự cố.

```bash
cd examples/toi-noi-postgresql
npm run p06                       # PostgreSQL 18 + PG17_BIN
npm run p06:extensions:install    # PG-057, một lần
npm run p06:extensions
```

Control plane của lab dùng `trust` trên loopback (README): chỉ chạy trên máy cá nhân.

### Sản phẩm bàn giao

`HO-SO-BAN-GIAO.md` định nghĩa chín sản phẩm:

| Mã | Nội dung | Bằng chứng |
|---|---|---|
| D1 | Schema và quyết định mô hình: partition GPS theo tháng, JSONB cho thuộc tính thiết bị, cột generated cho trường được lọc | 051, 053 |
| D2 | Truy vấn có kế hoạch kiểm chứng: pruning tĩnh và run-time, GIN | 051, 053, 054 |
| D3 | Vòng đời dữ liệu: partition tạo trước, DETACH CONCURRENTLY, đối soát trước DROP | 054 |
| D4 | Projection báo cáo: độ tươi, REFRESH CONCURRENTLY, đối soát với sổ gốc | 056, 060 |
| D5 | Tích hợp: logical replication, slot, quy trình đổi schema hai phía | 055 |
| D6 | Sẵn sàng cao: standby, đọc-sau-ghi theo LSN, failover, fencing, pg_rewind | 052, 058 |
| D7 | Nâng phiên bản: rehearsal hai đường, cổng go/no-go, giới hạn quay lại | 059 |
| D8 | Đánh giá extension theo workload, có điều kiện dừng | 057 |
| D9 | Runbook phục hồi từ P05 (PITR, quyền, RLS) | `p05/RUNBOOK.md` |

Nguyên tắc giữ từ PG-050: **không viết vào hồ sơ điều gì không có trong `OUTPUT.txt`**. Bước chưa chạy ghi «chưa chạy», không ghi «dự kiến đạt».

## 2. Đọc OUTPUT.txt như báo cáo nghiệm thu

`PASS`/`MEASURE` là cổng kiểm và số quan sát; khối `--- 0xx nhãn` là output psql, log server hoặc output công cụ thật. Port, PID, đường dẫn tạm và thời điểm bị che; LSN, tên file WAL và digest là của lần chạy đã ghi, lần khác có thể khác (README). Đầu và cuối tệp:

```text
P06 extension & integration lab · PostgreSQL 18.1 (+ 17.7 for PG-059)
Node v22.18.0 · win32/x64
Fixture: gps 1464 points (20 drivers, Oct–Nov 2026 UTC) partitioned by month + gps_plain copy; fares 2 rows / 150000 VND.
…
PASS P06: all assertions passed; temporary clusters cleaned up.
```

Dòng cuối chỉ xuất hiện khi mọi assertion đạt; runner không ghi đè `OUTPUT.txt` khi thất bại. Ba dòng đầu — phiên bản, nền tảng, fixture — là phần bắt buộc của mọi hồ sơ. Lab chưa chạy trên Linux/macOS.

## 3. Sổ cước: một bất biến đi qua mọi tính năng

Sổ bắt đầu với 2 chuyến / 150.000 đ (chuyến 1 HCM 85.000, chuyến 2 HN 65.000) và một materialized view `fare_report` tổng theo thành phố, có cột `refreshed_at`. Bảng của README:

| Bước | Thay đổi trên `fares` | Sổ sau bước |
|---|---|---|
| 056 | Thêm chuyến 3 (HCM, 50.000) | 3 chuyến / 200.000 |
| 055 | Sửa chuyến 3 thành 55.000; thêm cột `tip`, chuyến 1 tip 5.000 | 3 / 205.000 |
| 052 | Thêm chuyến 4 (HN, 30.000) | 4 / 235.000 |
| 058 | Chuyến 5 ghi lên primary cũ khi standby bị cắt → **mất** khi failover; chuyến 6 (HCM, 15.000) ghi lên primary mới; chuyến 7 ghi lên primary cũ khi split-brain → bị pg_rewind loại | 5 / 250.000 |

Cột «Sổ sau bước» là tổng `fare`: 205.000 = 85.000 + 65.000 + 55.000, tip 5.000 của chuyến 1 **không** được cộng. Phép kiểm của runner luôn là `count(*)||'|'||sum(fare)`, và `fare_report` cũng chỉ cộng `fare`. Đây là câu Phần 2 đã đặt ra, giờ quay lại trong hồ sơ: **«tổng tiền» là tổng của cột nào**. Bài này dùng định nghĩa của runner; nếu kế toán coi tip là doanh thu, cả báo cáo lẫn phép kiểm phải đổi cùng lúc.

Mỗi bước để lại bằng chứng trên chính sổ này:

```text
PASS 056 report before REFRESH (HCM): 1/85000
PASS 056 base table HCM: 2/135000
PASS 056 report after REFRESH (HCM): 2/135000
PASS 055 initial copy: 3|200000
PASS 055 disabled subscriber still sees: 50000
PASS 055 subscriber caught up after ENABLE: 55000
PASS 055 apply resumed after matching DDL on subscriber: tip 5000
PASS 052 standby received WAL but row not visible: 0
MEASURE 052 standby received-but-not-replayed bytes: 696
PASS 052 read-your-writes after waiting for commit LSN: 1
```

(Trích, bỏ các dòng xen giữa.) Projection (056) **cũ** so với sổ cho tới khi refresh. Bản sao logic (055) **có thể dừng**: subscriber tắt vẫn thấy 50.000 khi primary đã là 55.000, và đổi schema một phía làm apply lỗi. Standby vật lý (052) **đã nhận chưa chắc đã thấy**: 696 byte WAL tới nơi nhưng chưa replay; đọc-sau-ghi phải chờ replay tới LSN của commit. Ba góc của cùng một sự thật: bản sao nào cũng trễ, và hồ sơ phải nói độ trễ được kiểm thế nào.

## 4. Sự cố: mất primary, mất dữ liệu, hai writer

PG-058 giả lập mạng bị chia cắt bằng cách xoá `primary_conninfo` trên standby, và «crash» bằng `pg_ctl stop -m immediate` (README). Replication là bất đồng bộ.

```text
PASS 058 standby cut off from primary (network partition simulated)
PASS 058 old primary accepted write while standby cut off: 1
MEASURE 058 WAL not received by standby at failover: 184 bytes (primary flush 0/4000370, standby received 0/40002B8)
PASS 058 pg_promote: t
PASS 058 committed row lost by async failover: 0
PASS 058 new primary timeline: 00000002
PASS 058 target_session_attrs=read-write picks: standby (new primary)
```

Primary cũ nhận chuyến 5 (theo `run.mjs`: HN, 20.000 đ), commit thành công, rồi chết. **LSN** (*log sequence number*, vị trí trong dòng WAL) flush cuối của nó là `0/4000370`; standby chỉ nhận tới `0/40002B8`. Chênh 0x370 − 0x2B8 = 184 byte: WAL của chuyến 5. Sau `pg_promote`, chuyến 5 không tồn tại trên primary mới dù khách đã được báo thành công — tài xế Hà Nội ở đầu bài. Primary mới mở **timeline** 2, cùng cơ chế PG-050 đã thấy khi kết thúc PITR.

Rồi primary cũ khởi động lại mà không bị chặn:

```text
PASS 058 old primary restarted, not in recovery: f
--- 058 split-brain: two writable primaries
  old primary (timeline 00000001): 1,2,3,4,5,7
  new primary (timeline 00000002): 1,2,3,4,6
PASS 058 same connection string now picks: old primary
```

Đây là **split-brain**: hai node cùng nhận ghi, hai lịch sử khác nhau — chuyến 6 chỉ ở node mới, chuyến 7 chỉ ở node cũ. Dòng nguy hiểm nhất là dòng cuối: cùng connection string `host=old,new … target_session_attrs=read-write` trước đó chọn node mới, giờ chọn **node cũ**, vì nó đứng đầu danh sách và nhận ghi được. `target_session_attrs` chỉ hỏi «node này có nhận ghi không», không hỏi «node này có phải writer hợp pháp không» ([libpq · Connection Strings](https://www.postgresql.org/docs/18/libpq-connect.html#LIBPQ-CONNECT-TARGET-SESSION-ATTRS)). Ngăn split-brain là việc của **fencing** — chặn node cũ khỏi việc nhận ghi — không phải của driver.

Lab fence bằng cách dừng tay node cũ, rồi dùng **pg_rewind** — công cụ tua data directory của node cũ về điểm phân nhánh và chép phần khác biệt từ node mới ([PostgreSQL 18 · pg_rewind](https://www.postgresql.org/docs/18/app-pgrewind.html)):

```text
--- 058 pg_rewind
  pg_rewind: servers diverged at WAL location 0/40002B8 on timeline 1
  pg_rewind: rewinding from last common checkpoint at 0/3000080 on timeline 1
  pg_rewind: Done!
PASS 058 rewound old primary is standby: t
PASS 058 rows 5 and 7 discarded by rewind: 0
PASS 058 ledger on new primary: 5|250000
```

Điểm phân nhánh `0/40002B8` trùng LSN standby đã nhận lúc failover: hai số đo độc lập kể cùng một câu chuyện. Node cũ thành standby của timeline 2; chuyến 5 và 7 bị loại khỏi nó. Điều lab cho thấy nhưng không làm: khối split-brain là **lần cuối** hai chuyến đó còn đọc được. Trong sự cố thật, phải trích các dòng phân nhánh từ node cũ **trước** khi rewind để đối chiếu và nhập bù.

## 5. Báo cáo sau failover: projection cũ, không phải sổ sai

`fare_report` được refresh lần cuối ở bước 056, khi sổ là 200.000. Materialized view là dữ liệu vật lý, đi theo WAL sang standby y như bảng, nên primary mới thừa hưởng đúng bản cũ đó:

```text
PASS 060 report projection stale after failover: 200000 vs 250000
PASS 060 report reconciled after REFRESH: t
```

Đây là con số kế toán thấy. Chênh 50.000 = chuyến 3 (+5.000) + chuyến 4 (+30.000) + chuyến 6 (+15.000): toàn bộ là thay đổi **sau** lần refresh, không phải dữ liệu mất. Dữ liệu mất thật (chuyến 5) **không hiện** trong phép so này vì nó không có ở cả hai phía. Restore «để sửa báo cáo» không đem chuyến 5 về, mà còn có thể xoá chuyến 6.

Hành động đúng là `REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report` trên primary mới — dùng được vì unique index `fare_report_city` tạo ở 056 cũng đã theo WAL sang. Sổ và báo cáo cuối:

```text
--- 060 final ledger on new primary
   id | city | fare  | tip  
  ----+------+-------+------
    1 | HCM  | 85000 | 5000
    2 | HN   | 65000 |    0
    3 | HCM  | 55000 |    0
    4 | HN   | 30000 |    0
    6 | HCM  | 15000 |    0
  (5 rows)
--- 060 final report
   city | trips | total  
  ------+-------+--------
   HCM  |     3 | 155000
   HN   |     2 |  95000
  (2 rows)
PASS 060 replica follows new primary: 5|250000
```

Đối soát tay: HCM 85.000 + 55.000 + 15.000 = 155.000; HN 65.000 + 30.000 = 95.000; tổng 250.000. Id 5 thiếu trong dãy — người đọc sổ phải giải thích nó bằng bằng chứng mục 4, không bằng phỏng đoán. Assertion `reconciled after REFRESH` chỉ so **tổng**; khớp từng thành phố là đọc từ hai khối trên. Dòng cuối: node đã rewind theo kịp primary mới.

## 6. Báo cáo nghiệm thu D1–D8

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Mã</th><th>Khẳng định</th><th>Bằng chứng (trích từ OUTPUT.txt)</th></tr></thead>
<tbody>
<tr><td>D1</td><td>Partition theo tháng đặt đúng dòng; ràng buộc không bị partition làm yếu; JSONB có kiểm kiểu; cột generated khớp JSONB</td><td><code>rows per partition oct/nov: 744/720</code>; điểm 00:00 UTC 1/11 vào <code>gps_nov</code>; <code>23514: no partition of relation "gps" found for row</code>; <code>0A000: unique constraint on partitioned table must include all partitioning columns</code>; <code>CHECK jsonb_typeof</code> → 23514; <code>generated column agrees with JSONB: 147</code></td></tr>
<tr><td>D2</td><td>Lọc theo tài xế + tháng chỉ chạm một partition; thiếu khoá phân vùng thì quét mọi partition; pruning lúc chạy; GIN dùng được nhưng chưa được chọn</td><td>Kế hoạch chỉ <code>gps_nov</code>, 36 dòng như bảng thường; <code>Append</code> qua cả hai partition khi chỉ lọc tài xế; <code>Subplans Removed: 1</code>; hàm trên khoá phân vùng mất pruning; GIN chỉ hiện khi <code>enable_seqscan=off</code></td></tr>
<tr><td>D3</td><td>Tạo trước partition tháng 12; tách tháng cũ ít WAL hơn DELETE</td><td><code>December partition prepared ahead receives: 1</code>; <code>after DETACH: 721/744</code>; WAL DELETE 59736 · DETACH CONCURRENTLY 12296 · DROP 12680 byte (một lần chạy, không phải benchmark)</td></tr>
<tr><td>D4</td><td>Báo cáo có thể cũ; REFRESH thường chặn người đọc; CONCURRENTLY cần unique index và không chặn; đối soát với sổ</td><td><code>1/85000</code> so với <code>2/135000</code>; <code>55000: cannot refresh materialized view … concurrently</code>; reader chờ <code>AccessExclusiveLock</code>; <code>reader answered … (no wait)</code>; mục 5</td></tr>
<tr><td>D5</td><td>Initial copy khớp; slot giữ WAL khi subscriber dừng; DDL không được sao chép</td><td><code>initial copy: 3|200000</code>; slot không hoạt động giữ <code>6483 kB</code>; <code>missing replicated column: "tip"</code>; <code>DROP SUBSCRIPTION removed the slot: 0</code></td></tr>
<tr><td>D6</td><td>Standby chỉ đọc; đọc-sau-ghi theo LSN; failover bất đồng bộ mất dữ liệu đã commit; split-brain; rewind</td><td><code>25006</code>; 696 byte chưa replay; mục 4</td></tr>
<tr><td>D7</td><td>Hai đường nâng 17 → 18 giữ dữ liệu; cổng check bắt lỗi; còn đường quay lại</td><td>Mục 8</td></tr>
<tr><td>D8</td><td>Đánh giá extension tách hai thí nghiệm, có điều kiện dừng</td><td><code>p06/extensions/OUTPUT.txt</code>: 21 trước/sau GiST, bẫy 5000/5000, recall@10 theo <code>ef_search</code> (<a href="/articles/postgresql-extension-postgis-pgvector">PG-057</a>)</td></tr>
</tbody>
</table>
<p>D9 không chạy lại trong P06: nó là <code>p05/RUNBOOK.md</code> cùng bằng chứng ở <code>p05/OUTPUT.txt</code> (PG-050). Ô nào người nhận không tìm thấy bằng tìm chuỗi trong <code>OUTPUT.txt</code> là ô chưa được nghiệm thu.</p>
</details>

## 7. Cổng go/no-go trước khi mở lại ghi sau sự cố

**Go/no-go** là danh sách điều kiện phải cùng đạt trước một bước một chiều; một điều kiện trượt là «không đi». Sáu cổng của `HO-SO-BAN-GIAO.md`, kèm bằng chứng lab và phép kiểm bằng máy:

| # | Cổng | Kiểm bằng | Lab |
|---|---|---|---|
| G1 | **Một writer duy nhất**: node cũ đã bị fence trước khi promote | `SELECT pg_is_in_recovery()` trên **mọi** node trong connection string; đúng một node trả `f` | Split-brain có thật; `target_session_attrs` chọn node cũ khi nó đứng đầu. Fencing của lab là dừng tay |
| G2 | **Biết đã mất gì** | `pg_current_wal_flush_lsn()` cuối của node cũ (nếu đọc được), `pg_last_wal_receive_lsn()` của node promote, `pg_wal_lsn_diff` | 184 byte, chứa chuyến 5 đã commit |
| G3 | **Sổ gốc đúng** | Số chuyến, tổng tiền, digest so với nguồn độc lập (log ứng dụng, cổng thanh toán) | 5 chuyến / 250.000. Lab chỉ so count/sum, **không** tính digest cho `fares` và không có nguồn độc lập |
| G4 | **Projection đúng hoặc được đánh dấu cũ** | So `sum(total)` với `sum(fare)`; hiển thị `refreshed_at` | 200.000 so với 250.000 tới khi REFRESH |
| G5 | **Đường ghi thật** | Một thao tác ghi qua đúng connection string, pool, role ứng dụng | **Chưa chạy**: lab ghi bằng psql superuser |
| G6 | **Node cũ có số phận rõ ràng** | pg_rewind thành standby của timeline mới, hoặc dựng lại từ base backup | `rewound old primary is standby: t`; node đã rewind thấy 5 chuyến / 250.000 |

G1 đứng trước mọi thứ: đối soát trên cluster còn hai writer là vô nghĩa. G2 chạy **trước** khi rewind, vì sau đó dữ liệu phân nhánh biến mất. G4 không bao giờ là lý do restore đè sổ gốc. G3 và G5 là chỗ lab yếu nhất, hồ sơ phải nói thẳng. Như PG-050: người chạy `pg_promote` và `pg_rewind` — hai lệnh một chiều — không phải người chấp nhận mất chuyến 5.

## 8. Cổng go/no-go cho nâng phiên bản

PG-059 không chạm sổ cước; nó dùng bảng `tickets` riêng trên cluster 17.7. Hai đường:

```text
PASS 059 A dump/restore digest: 2/150000/fd0064873b687f567636614c521ab902
PASS 059 A identity continues after restore: 3
PASS 059 A source untouched (rollback target lacks new write): 2
PASS 059 A statistics after restore (before ANALYZE): 0
--- 059 B pg_upgrade --check against default 18 initdb
  Checking cluster versions                                     ok
  old cluster does not use data checksums but the new one does
  Failure, exiting
PASS 059 B digest after pg_upgrade: 2/150000/fd0064873b687f567636614c521ab902
PASS 059 B planner statistics carried over: true
PASS 059 B copy mode: old 17 cluster still starts with: 2 rows on 17.7
```

(Trích.) `initdb` bản 18 bật data checksums mặc định, cluster 17 không có, nên `--check` từ chối; lab initdb lại với `--no-data-checksums` rồi nâng ở copy mode. Cổng của hồ sơ, kèm phần production cần thêm:

| Cổng (kết quả lab 059) | Production cần thêm |
|---|---|
| `--check` sạch trên bản sao | Extension (PG-057), collation/ICU, tablespace, kiểu dữ liệu bị loại |
| Digest khớp nguồn (cả hai đường) | Đối soát mọi bảng nghiệp vụ |
| Identity tiếp tục (`RETURNING id` = 3) | Mọi sequence |
| Statistics: dump/restore 0 dòng `pg_stats` trước ANALYZE; pg_upgrade mang sang | `vacuumdb --analyze-in-stages` cho phần không được mang sang |
| Quay lại: cluster 17 (copy mode) vẫn khởi động | Thời hạn giữ cluster cũ; ghi mới sau cutover không tự quay về (nguồn 059 A vẫn 2 dòng) |

## 9. Bằng chứng cần giữ — của cả series

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Phần</th><th>Bằng chứng</th><th>Giữ vì</th></tr></thead>
<tbody>
<tr><td>P01</td><td><code>p01/OUTPUT.txt</code>: DDL, fixture hợp lệ/sai, kiểm tra ràng buộc</td><td>Định nghĩa dữ liệu hợp lệ — G3 dựa vào đây</td></tr>
<tr><td>P02</td><td><code>p02/OUTPUT.txt</code>: truy vấn đối soát có expected output</td><td>Định nghĩa «tổng tiền» và cách giải thích chênh lệch</td></tr>
<tr><td>P03</td><td><code>p03/DO-DAC.md</code>: cấu hình máy, số dòng, cache, trung vị và từng lần đo</td><td>Baseline hiệu năng; truy vấn kiểm tra trong sự cố không được là Seq Scan 1 triệu dòng</td></tr>
<tr><td>P04</td><td><code>p04/OUTPUT.txt</code>, <code>p04/DO-DAC.md</code> (8 phiên × 3 lần)</td><td>Bất biến «đúng một người thắng» phải còn đúng trên primary mới</td></tr>
<tr><td>P05</td><td><code>p05/OUTPUT.txt</code>, <code>p05/RUNBOOK.md</code>: backup_label, restore point, timeline, cổng G1–G5 của PITR</td><td>D9; đường lui khi failover không đủ</td></tr>
<tr><td>P06</td><td><code>p06/OUTPUT.txt</code>: LSN flush/receive, 184 byte, khối split-brain, log pg_rewind, sổ và báo cáo cuối; <code>HO-SO-BAN-GIAO.md</code></td><td>G1–G6, cổng nâng phiên bản</td></tr>
<tr><td>P06</td><td><code>p06/extensions/OUTPUT.txt</code>: phiên bản PGlite/postgis/vector, kế hoạch, recall</td><td>D8 và giới hạn của nó</td></tr>
<tr><td>—</td><td>Nhật ký quyết định: ai fence, ai promote, ai chấp nhận mất chuyến 5, lúc nào</td><td><b>Lab không có</b> — runner tự làm mọi bước</td></tr>
</tbody>
</table>
<p><b>Không</b> lưu vào repo: dump, data directory, WAL, binary PostgreSQL 17, chứng chỉ, mật khẩu (README P05, P06). Bằng chứng là output kiểm tra; dữ liệu được bảo vệ bằng quy trình riêng.</p>
</details>

## 10. Quyết định kiến trúc

Curriculum đặt sản phẩm cuối là «quyết định kiến trúc». Mỗi dòng dưới đây là một quyết định có điều kiện, kèm bằng chứng và điều kiện xem lại:

| Quyết định | Bằng chứng | Xem lại khi |
|---|---|---|
| Partition GPS theo tháng, khoá chính `(driver_id, seen_at)` | Pruning theo tháng; DETACH ít WAL hơn DELETE (051, 054) | Truy vấn chủ yếu không có điều kiện thời gian — partition thành chi phí |
| JSONB cho thuộc tính thiết bị; trường được lọc thành cột generated | Gõ sai khoá trả 0 không lỗi; cột generated khớp 147 (053) | Một khoá JSONB trở thành bắt buộc — chuyển thành cột có ràng buộc |
| Báo cáo đọc từ materialized view có `refreshed_at` | Lệch 200.000/250.000; CONCURRENTLY không chặn người đọc (056, 060) | Nghiệp vụ cần số tức thời — đổi kiến trúc projection |
| CDC qua logical replication, giám sát slot | Slot giữ WAL khi bên nhận dừng; DDL phải làm hai phía (055) | Slot giữ WAL vượt ngân sách đĩa — cần ngưỡng cảnh báo và quyền bỏ slot |
| Standby bất đồng bộ, fencing bắt buộc trước promote | Mất 184 byte; split-brain (058) | Nghiệp vụ không chấp nhận mất chuyến đã commit — cần synchronous replication, chưa đo |
| Nâng phiên bản bằng pg_upgrade copy mode sau rehearsal | Digest khớp, statistics mang sang, cluster cũ còn khởi động (059) | Không đủ đĩa cho bản sao — cân nhắc `--link`, chưa thử |
| Extension chỉ khi qua hai thí nghiệm độc lập | 057 | Nền tảng không hỗ trợ, hoặc điều kiện dừng của PG-057 xảy ra |

## 11. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm; từ 80 là đạt; tiêu chí 1, 3, 5 và 8 bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Bằng chứng thật | Mọi ô D1–D8 tìm được bằng tìm chuỗi trong `OUTPUT.txt` |
| 2 | Sổ là sợi chỉ | Giải thích từng thay đổi của sổ từ 150.000 tới 250.000 và định nghĩa «tổng tiền» (có tính tip không) |
| 3 | Một writer | Chỉ ra vì sao `target_session_attrs=read-write` không ngăn split-brain; fencing đặt trước promote |
| 4 | Đo mất mát | Tính được 184 byte từ hai LSN và nối với điểm phân nhánh của pg_rewind |
| 5 | Không restore vì báo cáo | Phân biệt 50.000 chênh do projection cũ với chuyến 5 mất thật |
| 6 | Dữ liệu phân nhánh | Trích chuyến 5 và 7 trước khi rewind; có kế hoạch nhập bù |
| 7 | Nâng phiên bản | Nêu cổng check, digest, identity, statistics và giới hạn quay lại |
| 8 | Trung thực | G3 và G5 được ghi là chưa chứng minh; không gọi số nào là RTO |
| 9 | Vai trò | Người thực thi khác người chấp nhận mất dữ liệu và người cho mở ghi |
| 10 | Giới hạn | Liệt kê đủ mục 14 |

## 12. Tổng kết Phần 6

| Bài | Đóng góp vào hồ sơ |
|---|---|
| [PG-051 · Partition cơ bản](/articles/postgresql-partition-co-ban) | 744/720 theo tháng; 23514 ngoài mọi partition; UNIQUE phải chứa khoá phân vùng |
| [PG-052 · Primary, standby và replication lag](/articles/postgresql-replication-primary-standby) | Standby chỉ đọc; đã nhận chưa chắc đã thấy; đọc-sau-ghi theo LSN |
| [PG-053 · JSONB và dữ liệu quan hệ](/articles/postgresql-jsonb-quan-he) | Khoá gõ sai im lặng; CHECK kiểu; cột generated |
| [PG-054 · Partition pruning và vòng đời](/articles/postgresql-partition-pruning-retention) | Pruning lúc chạy; partition tạo trước; DETACH CONCURRENTLY |
| [PG-055 · Logical replication và CDC](/articles/postgresql-logical-replication-cdc) | Slot giữ WAL; DDL hai phía; DROP SUBSCRIPTION xoá slot |
| [PG-056 · Materialized view](/articles/postgresql-materialized-view-reporting) | Độ tươi; khoá của REFRESH; đối soát với sổ |
| [PG-057 · PostGIS và pgvector](/articles/postgresql-extension-postgis-pgvector) | Hai thí nghiệm độc lập; bẫy độ/mét; recall theo `ef_search` |
| [PG-058 · Failover và split-brain](/articles/postgresql-ha-failover-read-consistency) | 184 byte mất; hai writer; fencing và pg_rewind |
| [PG-059 · Nâng phiên bản và cutover](/articles/postgresql-upgrade-migration-cutover) | Hai đường 17 → 18; checksum; giới hạn quay lại |

Sản phẩm đúng như khung đặt ra cho Phần 6: hồ sơ vận hành và diễn tập sự cố, kèm kế hoạch thay đổi.

## 13. Tổng kết cả series

| Phần | Sản phẩm cuối phần | Capstone |
|---|---|---|
| P01 · Nền tảng và mô hình dữ liệu | DDL, fixture hợp lệ/sai, truy vấn bất biến | [PG-010 · Schema chuyến xe có kiểm tra ràng buộc](/articles/postgresql-thuc-hanh-schema-chuyen-xe) |
| P02 · SQL cho nghiệp vụ | Sổ đối soát, 7 chuyến lệch, giải thích chênh +268.000 đ | [PG-020 · Truy vấn vận hành và đối soát thanh toán](/articles/postgresql-thuc-hanh-sql-doi-soat) |
| P03 · Lưu trữ, index và kế hoạch | Từ giả thuyết tới plan, đo lặp, cái giá phía ghi của index | [PG-030 · Tối ưu truy vấn dựa trên bằng chứng](/articles/postgresql-thuc-hanh-toi-uu-truy-van) |
| P04 · Giao dịch, MVCC và đồng thời | Bất biến «đúng một người thắng», lab và phép thử 8 phiên | [PG-040 · Hai tài xế cùng nhận một chuyến](/articles/postgresql-thuc-hanh-tranh-chap-nhan-chuyen) |
| P05 · Bảo mật, bảo trì và phục hồi | PITR có cổng, timeline 2, RPO/RTO tách mốc | [PG-050 · Bàn giao runbook vận hành và phục hồi](/articles/postgresql-thuc-hanh-runbook-phuc-hoi) |
| P06 · Mở rộng, tích hợp và vận hành | Sổ đi qua projection, CDC, failover; cổng go/no-go | PG-060 · bài này |

Đọc dọc cột giữa sẽ thấy cùng một phương pháp lặp lại sáu lần: **định nghĩa điều phải đúng trước, tạo tình huống làm nó sai, rồi giữ bằng chứng máy kiểm được**. Ràng buộc ở P01, expected output ở P02, kế hoạch và số đo ở P03, bất biến dưới tranh chấp ở P04, digest sau restore ở P05, sổ sau failover ở P06.

## 14. Những gì lab chưa chứng minh

Từ mục «Việc lab chưa chứng minh» của `HO-SO-BAN-GIAO.md` và mục Giới hạn của README:

- **RTO đầu cuối**: phát hiện sự cố, quyết định, fencing, chuyển DNS/VIP, pool kết nối lại. Lab không có công cụ HA (Patroni, repmgr…), không VIP hay DNS.
- **Synchronous replication và RPO = 0**; mất đồng thời hai node; replication slot trên standby được promote.
- **Logical replication khi cutover nâng phiên bản**; conflict khi apply; theo dõi độ trễ apply liên tục.
- **PostGIS/pgvector trên server thật** với dữ liệu và tải thật; chi phí ghi của HNSW.
- **Backup ngoài máy**, mã hoá, lưu giữ, diễn tập phục hồi định kỳ (P05).
- **Quy mô**: dữ liệu nhỏ, cache ấm, một máy. Số byte WAL là quan sát một lần chạy, không phải benchmark; autovacuum đang bật nên WAL cấp cluster có thể lẫn việc nền.
- Riêng capstone: G3 không có digest hay nguồn độc lập; G5 không chạy qua đường ứng dụng; nâng phiên bản chạy trên một bảng nhỏ, không extension, không đo downtime, không thử `--link`/`--clone`/`--swap`.

Câu kết của hồ sơ giữ nguyên: không cam kết RPO/RTO cho tới khi đo được.

## 15. Bẫy hay gặp khi tự làm

- **Tin «database đã lên»**, hoặc tin `target_session_attrs` chống được split-brain.
- **Rewind trước khi trích dữ liệu phân nhánh.**
- **Restore đè sổ vì báo cáo lệch.**
- **Đối soát bằng superuser rồi mở API**: quyền và pool cũng có thể hỏng sau failover.
- **Gọi con số nào của lab là RTO**, hoặc sửa `OUTPUT.txt` khi một cổng trượt.

## 16. Bài tập mở rộng

Chỉ chạy trên bản sao lab.

**Bài 1 — Fencing trước promote.** Sửa bản sao `run.mjs` để node cũ bị chặn trước khi được khởi động lại (ví dụ: khởi động lại nó trên một port không có trong connection string của ứng dụng), rồi chạy lại phần 058.

<details><summary><b>Một lời giải đúng phải chứng minh</b></summary>
<p>Output không còn khối «two writable primaries»; cùng connection string có <code>target_session_attrs=read-write</code> luôn chọn primary mới; trên mọi node đúng một <code>pg_is_in_recovery()</code> trả <code>f</code>. Lời giải giải thích vì sao cơ chế chặn đã chọn hoạt động cả khi máy cũ tự khởi động lại, và giữ nguyên các assertion khác.</p>
</details>

**Bài 2 — Trích dữ liệu phân nhánh.** Trước `pg_rewind`, trích từ node cũ mọi dòng `fares` không có trên primary mới.

<details><summary><b>Một lời giải đúng phải chứng minh</b></summary>
<p>Output liệt kê đúng chuyến 5 và 7 kèm nội dung, lấy bằng truy vấn trên node cũ khi nó đã bị chặn khỏi ứng dụng, không lấy từ mã nguồn runner. Lời giải nêu ai quyết định nhập bù chuyến nào (chuyến 5 khách đã được báo thành công; chuyến 7 ghi trong lúc split-brain) và chứng minh sổ sau nhập bù khớp với nguồn đối chiếu đã chọn.</p>
</details>

**Bài 3 — Hoàn thiện G3.** Thêm digest kiểu P05/PG-059 cho `fares` và một nguồn đối chiếu độc lập (ví dụ bảng log ứng dụng ghi song song).

<details><summary><b>Một lời giải đúng phải chứng minh</b></summary>
<p>Digest kỳ vọng được tính trước sự cố, không gõ tay sau khi thấy kết quả. Sau failover, digest khác kỳ vọng đúng ở chuyến 5, và nguồn đối chiếu chỉ ra chính chuyến đó. Lời giải nêu định nghĩa digest (cột nào, có tip không, sắp theo gì) và vì sao định nghĩa đó bắt được lỗi «đúng tổng, sai nội dung».</p>
</details>

## 17. Đọc tiếp

- [PostgreSQL 18 · High Availability, Load Balancing, and Replication](https://www.postgresql.org/docs/18/high-availability.html): standby, failover, timeline.
- [PostgreSQL 18 · pg_rewind](https://www.postgresql.org/docs/18/app-pgrewind.html) và [System Administration Functions](https://www.postgresql.org/docs/18/functions-admin.html): `pg_promote`, `pg_current_wal_flush_lsn`, `pg_last_wal_receive_lsn`, `pg_wal_lsn_diff`.
- [libpq · `target_session_attrs`](https://www.postgresql.org/docs/18/libpq-connect.html#LIBPQ-CONNECT-TARGET-SESSION-ATTRS), [REFRESH MATERIALIZED VIEW](https://www.postgresql.org/docs/18/sql-refreshmaterializedview.html), [pg_upgrade](https://www.postgresql.org/docs/18/pgupgrade.html).
- Lab: `examples/toi-noi-postgresql/p06/OUTPUT.txt`, `p06/run.mjs`, `p06/README.md`, `p06/HO-SO-BAN-GIAO.md`, `p05/RUNBOOK.md`.

Series «PostgreSQL qua dự án Tới Nơi» kết thúc ở đây. Sáu mươi bài bắt đầu từ một câu hỏi đơn giản — chuyến này có thật không, bị tính bao nhiêu tiền — và kết thúc ở chính câu hỏi đó, chỉ là hỏi lúc 02:10 sáng, sau khi một primary đã chết. Nếu muốn đi lại từ đầu với con mắt của người vừa làm xong capstone, hãy bắt đầu từ [PG-001 · PostgreSQL là gì và nằm ở đâu trong Tới Nơi?](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi), và giữ bên cạnh [hồ sơ dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) — nơi những con số 560 UPDATE/giây và 6.250 điểm GPS/giây vẫn đang chờ được đo trên hạ tầng thật.

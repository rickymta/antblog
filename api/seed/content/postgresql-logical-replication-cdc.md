---
slug: postgresql-logical-replication-cdc
title: "Logical replication và CDC: khi bên nhận ngừng đọc, primary giữ WAL tới bao giờ?"
summary: "Đội đối soát của Tới Nơi muốn một bản sao sổ cước cập nhật liên tục ở database riêng. Bài dựng publication, subscription và replication slot trên PostgreSQL 18.1 thật: initial copy, tắt bên nhận để thấy slot giữ WAL, bật lại để thấy nó bắt kịp, thêm cột chỉ ở publisher để thấy apply dừng — rồi bàn về slot bị bỏ quên, thứ tự đổi schema, REPLICA IDENTITY và khi nào nên chọn outbox thay vì CDC."
category: postgresql
level: intermediate
tags: [postgresql, pg-055, logical-replication, cdc, replication-slot, publication, subscription, toi-noi]
language: vi
---

Đội đối soát của Tới Nơi chạy báo cáo doanh thu trên một database riêng. Mỗi sáng họ dump bảng `fares` từ database chính rồi nạp lại. Giờ họ muốn số liệu tươi hơn: chuyến vừa được sửa giá lúc 10:15 thì 10:16 bên đối soát đã thấy, và họ không muốn chạy `SELECT * FROM fares` mỗi phút lên primary đang phục vụ đặt xe.

PostgreSQL làm được điều đó bằng **logical replication**: primary đọc lại nhật ký WAL của chính nó, dịch thành các thay đổi theo dòng («dòng 3 của `fares` đổi `fare` thành 55.000») và gửi sang một database khác để áp dụng. Kỹ thuật chung «bắt các thay đổi đã commit rồi chuyển đi nơi khác» gọi là **CDC** (change data capture).

Câu hỏi khiến người trực primary mất ngủ là: **nếu bên đối soát tắt máy cuối tuần để bảo trì thì sao?** Primary có chờ không? Thay đổi trong lúc đó đi đâu? Nếu họ quên bật lại? Bài này dựng logical replication trên hai cluster PostgreSQL 18.1 thật, cố tình làm bên nhận ngừng đọc, rồi cố tình làm lệch schema hai bên, để trả lời từng câu bằng output.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-055 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-wal-checkpoint-durability">PG-045 · WAL, checkpoint và độ bền dữ liệu</a>, <a href="/articles/postgresql-replication-primary-standby">PG-052 · Primary, standby và replication lag</a> và <a href="/articles/postgresql-idempotency-outbox-transaction">PG-039 · Idempotency và transactional outbox</a>. Lab: <code>examples/toi-noi-postgresql/p06</code>, PostgreSQL 18.1 thật (thêm PostgreSQL 17.7 cho PG-059); trong <code>examples/toi-noi-postgresql</code> đặt <code>PG17_BIN</code> trỏ tới thư mục bin của PostgreSQL 17 rồi chạy <code>npm run p06</code> (chi tiết trong README của lab). Runner tự dựng cluster tạm chỉ nghe localhost và tự xoá khi xong. Sau bài này, bạn dựng được một subscription, đọc được trạng thái replication slot, biết vì sao một slot không ai đọc là nguy cơ đầy đĩa, và đổi schema hai phía theo đúng thứ tự.</p>
</div>

## 1. Logical replication khác physical replication ở đâu

PG-052 đã dựng **physical replication**: standby nhận nguyên văn từng byte WAL và phát lại, nên nó là bản sao y hệt cả cluster — mọi database, mọi bảng, cùng phiên bản lớn, chỉ đọc. Bên đối soát không cần thế. Họ cần **một bảng**, trên một database mà họ còn tự tạo bảng báo cáo, index riêng, có thể ở phiên bản khác.

Logical replication đứng ở tầng cao hơn. Các thuật ngữ sẽ dùng suốt bài:

- **Logical decoding**: quá trình đọc WAL và dựng lại thay đổi ở mức dòng (INSERT, UPDATE, DELETE, TRUNCATE) theo đúng thứ tự commit. Chỉ chạy được khi primary đặt `wal_level = logical`, vì mức `replica` mặc định không ghi đủ thông tin để dựng lại dòng. Đổi `wal_level` cần khởi động lại server.
- **Publication**: tập hợp bảng mà phía nguồn (**publisher**) công bố, tạo bằng `CREATE PUBLICATION`.
- **Subscription**: phía nhận (**subscriber**) khai báo «kết nối tới publisher này, nhận publication kia», tạo bằng `CREATE SUBSCRIPTION`.
- **Replication slot**: một bản ghi bền trên publisher nhớ bên nhận đã xác nhận tới đâu trong dòng WAL. Slot là lý do không mất thay đổi khi bên nhận ngừng — và cũng là lý do primary có thể đầy đĩa.
- **pgoutput**: output plugin chuẩn mà logical replication tích hợp dùng để mã hoá thay đổi gửi đi.
- **Walsender**: tiến trình trên publisher đọc WAL, giải mã qua slot và đẩy dữ liệu. Phía subscriber có **apply worker** nhận và áp dụng từng giao dịch.

Subscriber là **database bình thường, ghi được**, không phải standby: đội đối soát thêm được bảng riêng, nhưng PostgreSQL cũng không ngăn ai sửa tay `fares` bên nhận và gây xung đột (mục 8).

## 2. Dựng publication và subscription

Primary của lab P06 chạy với `wal_level=logical`. Lúc tới bước 055, sổ cước có 3 chuyến, tổng 200.000 đ (bảng sổ trong README: bước 056 vừa thêm chuyến 3 ở HCM giá 50.000). Runner tạo thêm một cluster PostgreSQL 18.1 làm subscriber rồi chạy đúng các lệnh sau (cổng thật được thay bằng `<PORT>`):

```sql
-- Trên publisher (primary)
CREATE PUBLICATION fare_pub FOR TABLE fares;

-- Trên subscriber: bảng phải có sẵn, logical replication không chép schema
CREATE TABLE fares (id int PRIMARY KEY, city text NOT NULL, fare int NOT NULL CHECK (fare >= 0));
CREATE SUBSCRIPTION fare_sub
  CONNECTION 'host=127.0.0.1 port=<PORT> dbname=postgres user=postgres'
  PUBLICATION fare_pub;
```

Ba việc xảy ra khi `CREATE SUBSCRIPTION` chạy với tuỳ chọn mặc định:

1. Subscriber kết nối tới publisher và **tạo replication slot** cùng tên với subscription (`fare_sub`).
2. Với mỗi bảng trong publication, một **tablesync worker** chép dữ liệu hiện có sang — gọi là **initial copy** (`copy_data = true` mặc định).
3. Khi bảng đã đồng bộ, apply worker tiếp tục nhận các thay đổi commit sau thời điểm chép.

Tiến trình từng bảng nằm ở `pg_subscription_rel.srsubstate` trên subscriber. Theo tài liệu, trạng thái đi từ `i` (khởi tạo), `d` (đang chép dữ liệu), `f` (chép xong), `s` (đã đồng bộ) tới `r` (sẵn sàng, đang nhận luồng thay đổi bình thường). Runner đợi tới khi mọi bảng ở `r` rồi so sổ hai bên bằng cùng một câu `SELECT count(*)||'|'||sum(fare) FROM fares`:

```text
PASS 055 initial copy: 3|200000
```

Ba chuyến, 200.000 đ ở cả hai đầu. Ở dữ liệu thật, nên đối soát bằng digest theo khoá như PG-046, lúc không có ghi mới chen vào giữa hai lần đọc.

Theo tài liệu PostgreSQL 18, cột được ghép **theo tên**, không theo thứ tự; bảng đích được phép thừa cột; kiểu không bắt buộc trùng khít miễn là biểu diễn text chuyển được sang kiểu đích. Publisher gửi một cột mà subscriber không có thì là lỗi — mục 6 dựng lại đúng lỗi đó.

## 3. Nhìn vào slot và walsender

Ngay sau initial copy, runner đọc `pg_replication_slots` trên publisher:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-logical-replication-cdc/dbstudio-01.webp" alt="DBStudio chạy truy vấn của bước 055 trong p06/run.mjs và hiển thị kết quả 1 dòng" width="996" height="558" loading="lazy"></div><figcaption>DBStudio · bước 055 của <code>p06/run.mjs</code> · 1 dòng</figcaption></figure>

- **slot_type = logical, plugin = pgoutput**: slot này giải mã WAL thành thay đổi logic qua plugin chuẩn.
- **active = t**: đang có một walsender dùng slot.
- **wal_status = reserved**: theo tài liệu `pg_replication_slots`, WAL mà slot cần vẫn nằm trong giới hạn `max_wal_size`. Các giá trị khác là `extended` (vượt `max_wal_size` nhưng vẫn được giữ, do slot hoặc do `wal_keep_size`), `unreserved` (sắp bị xoá ở checkpoint tới) và `lost` (WAL cần thiết đã mất, slot không dùng được nữa).

Ba cột không có trong khối trên nhưng nên có trong truy vấn giám sát: **`restart_lsn`** — LSN cũ nhất slot còn cần, segment WAL từ đây trở đi không được xoá; **`confirmed_flush_lsn`** — vị trí bên nhận đã xác nhận; và **`catalog_xmin`** — slot logical giữ lại phiên bản cũ của bảng hệ thống (catalog) mà việc giải mã cần, nên VACUUM không dọn được chúng khi slot đứng yên.

Phía publisher, bên nhận hiện ra như một kết nối replication:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-logical-replication-cdc/dbstudio-02.webp" alt="DBStudio chạy truy vấn của bước 055 trong p06/run.mjs và hiển thị kết quả 1 dòng" width="996" height="536" loading="lazy"></div><figcaption>DBStudio · bước 055 của <code>p06/run.mjs</code> · 1 dòng</figcaption></figure>

`pg_stat_replication` cũng là view mà PG-052 dùng cho standby vật lý. Subscription xuất hiện ở đây với `application_name` bằng tên subscription. Trạng thái `streaming` nghĩa là initial copy đã xong và walsender đang đẩy thay đổi liên tục.

## 4. Bên nhận ngừng đọc: slot giữ chỗ

Đây là tình huống «đội đối soát tắt máy cuối tuần». Runner tắt subscription rồi sửa giá trên primary:

```sql
-- Trên subscriber
ALTER SUBSCRIPTION fare_sub DISABLE;
-- Runner đợi tới khi slot trên publisher chuyển active = f, rồi trên publisher:
UPDATE fares SET fare = 55000 WHERE id = 3;
SELECT pg_switch_wal(); CHECKPOINT;
```

`DISABLE` dừng apply worker; walsender bên publisher thoát theo. **Slot vẫn còn.** Primary không chờ ai: câu UPDATE commit ngay, như thể không có subscription nào. Logical replication mặc định là bất đồng bộ.

Bên nhận vẫn thấy giá cũ:

```text
PASS 055 disabled subscriber still sees: 50000
```

Còn trên publisher, slot đứng yên ở `restart_lsn` cũ trong khi WAL tiếp tục chạy:

```text
MEASURE 055 WAL bytes held back by inactive slot fare_sub: 6638440
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-logical-replication-cdc/dbstudio-03.webp" alt="DBStudio chạy truy vấn của bước 055 trong p06/run.mjs và hiển thị kết quả 1 dòng" width="996" height="558" loading="lazy"></div><figcaption>DBStudio · bước 055 của <code>p06/run.mjs</code> · 1 dòng</figcaption></figure>

Cột `retained` do runner tự đặt tên: nó là `pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn))`, tức khoảng cách từ chỗ slot còn cần tới vị trí WAL hiện tại. 6.638.440 byte hiển thị thành 6483 kB.

Đọc con số này cho đúng, vì nó dễ gây hiểu lầm:

- Câu UPDATE một dòng chỉ sinh vài trăm byte WAL. Phần lớn 6483 kB đến từ `pg_switch_wal()`: lệnh này đóng segment hiện tại, và vị trí WAL nhảy tới đầu segment kế tiếp, nên khoảng cách tính bằng LSN gồm cả phần cuối segment chưa dùng. Runner gọi nó (cùng `CHECKPOINT`) để tạo điều kiện cho việc xoá segment cũ, xem slot có giữ lại hay không.
- Primary của lab còn đặt `wal_keep_size = 512MB` (để pg_rewind ở PG-058 có đủ WAL). Với cấu hình đó, segment gần đây được giữ **kể cả khi không có slot**. Vì vậy lab chứng minh `restart_lsn` của slot không tiến khi bên nhận ngừng, nhưng **không** tách riêng được dung lượng đĩa do slot giữ so với do `wal_keep_size` giữ. README ghi rõ: số byte là quan sát của một lần chạy, có thể lẫn việc nền (autovacuum đang bật trên primary), không phải benchmark.

Điều cần nhớ là cơ chế, không phải con số: **khoảng cách này tăng theo lượng WAL mà toàn cluster sinh ra**, không chỉ theo thay đổi của bảng `fares`. Slot giữ WAL theo vị trí; WAL của bảng GPS, của VACUUM, của mọi database khác trên cùng cluster đều nằm trong khoảng đó.

Rồi bật lại:

```sql
ALTER SUBSCRIPTION fare_sub ENABLE;
```

```text
PASS 055 subscriber caught up after ENABLE: 55000
```

Runner đợi tới khi subscriber đọc đúng 55.000 cho chuyến 3. Không mất thay đổi nào: walsender mới bắt đầu giải mã từ vị trí slot đã nhớ, gửi lại giao dịch UPDATE, apply worker áp dụng nó. Đây là giá trị của slot — và là cái giá của nó.

## 5. Slot bị bỏ quên: cách một bảng báo cáo làm sập primary

Giả sử đội đối soát không bật lại vào thứ Hai. Hoặc họ xoá cluster đối soát cũ mà quên `DROP SUBSCRIPTION`. Slot `fare_sub` trên primary vẫn nằm đó, `active = f`, và theo thiết kế nó **giữ mọi segment WAL từ `restart_lsn`** cho tới khi có người đọc. Với mặc định của PostgreSQL 18, không có giới hạn nào cho việc này. Hậu quả nối tiếp nhau:

1. Thư mục `pg_wal` lớn dần theo tốc độ sinh WAL của toàn cluster.
2. Khi phân vùng chứa `pg_wal` đầy, PostgreSQL không ghi được WAL và dừng lại. Một bảng báo cáo phụ làm sập database đặt xe.
3. Trong lúc đó, `catalog_xmin` của slot giữ lại phiên bản cũ của bảng hệ thống, khiến catalog phình to.

Phòng ngừa có ba lớp.

**Giám sát.** Truy vấn sau dùng các cột có trong `pg_replication_slots` của PostgreSQL 18 (không chạy trong lab, không có output đi kèm):

```sql
-- không chạy trong lab
SELECT slot_name, slot_type, active, wal_status, safe_wal_size, inactive_since,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) AS behind
FROM pg_replication_slots
ORDER BY pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) DESC;
```

Cảnh báo khi một slot `active = f` quá một ngưỡng thời gian (`inactive_since`), khi khoảng cách vượt ngưỡng byte, hoặc khi `wal_status` rời `reserved`. Ngưỡng phải tính theo dung lượng đĩa trống và tốc độ sinh WAL đo được ở giờ cao điểm.

**Giới hạn cứng.** Tham số `max_slot_wal_keep_size` đặt mức WAL tối đa mà slot được giữ. Vượt mức, slot bị **vô hiệu hoá** (`wal_status = lost`) và WAL cũ được giải phóng. PostgreSQL 18 có thêm `idle_replication_slot_timeout` để vô hiệu hoá slot không hoạt động quá lâu. Cả hai **không chạy trong lab**:

```text
# postgresql.conf — không chạy trong lab
max_slot_wal_keep_size = 50GB
idle_replication_slot_timeout = 2d
```

Đây là đánh đổi chứ không phải nút «an toàn»: giới hạn bảo vệ primary, nhưng slot đã `lost` thì không nối lại được — subscriber đã lỡ một đoạn thay đổi, phải dựng lại subscription, chép lại dữ liệu và đối soát.

**Quy trình gỡ.** Ngừng dùng subscription thì `DROP SUBSCRIPTION` khi publisher còn kết nối được (mục 7). Không xoá tay file trong `pg_wal`; không `pg_drop_replication_slot` chỉ để dập cảnh báo khi chưa thống nhất với bên nhận cách đồng bộ lại.

## 6. DDL không đi theo: thêm cột ở sai phía trước

Tài liệu PostgreSQL 18 (mục Restrictions của logical replication) nói thẳng: **schema và lệnh DDL không được sao chép**. Publication chỉ mang dữ liệu. Đội sản phẩm thêm tiền tip vào chuyến; migration chạy trên primary trước, như mọi migration khác:

```sql
-- Trên publisher
ALTER TABLE fares ADD COLUMN tip int NOT NULL DEFAULT 0;
UPDATE fares SET tip = 5000 WHERE id = 1;
```

Câu UPDATE sinh ra một thay đổi có cột `tip`; apply worker bên đối soát không tìm thấy cột đó trong bảng đích. Runner đợi tới khi log subscriber có dòng lỗi, kiểm bảng đích vẫn chưa có cột `tip` (số `0` dưới đây là số cột tên `tip` trong `information_schema.columns`), rồi trích dòng log đầu tiên:

```text
PASS 055 subscriber unchanged while apply fails: 0
```

```text
ERROR:  logical replication target relation "public.fares" is missing replicated column: "tip"
```

Chuyện gì xảy ra sau lỗi này quan trọng hơn chính lỗi:

- Apply worker dừng ở giao dịch lỗi. Theo tài liệu, nó sẽ được khởi động lại và thử lại giao dịch đó — và lỗi lại — cho tới khi nguyên nhân được sửa. **Mọi giao dịch sau đó cũng bị chặn**, kể cả thay đổi của những chuyến không liên quan tới tip: logical replication áp dụng theo thứ tự commit, không nhảy qua.
- Slot không nhận được xác nhận mới, nên `confirmed_flush_lsn` và `restart_lsn` đứng yên. Một migration lệch phía biến thành đúng tình huống slot bị bỏ quên của mục 5, chỉ khác là `active` vẫn có thể là `t`.
- Không mất dữ liệu: thay đổi vẫn nằm trong WAL mà slot giữ.

Sửa bằng cách cho subscriber có cột còn thiếu:

```sql
-- Trên subscriber
ALTER TABLE fares ADD COLUMN tip int NOT NULL DEFAULT 0;
```

```text
PASS 055 apply resumed after matching DDL on subscriber: tip 5000
PASS 055 apply errors counted: t
```

Runner đợi tới khi chuyến 1 ở subscriber có `tip = 5000`: giao dịch bị kẹt đã được áp dụng lại từ đầu và luồng chạy tiếp. Dòng thứ hai kiểm `apply_error_count > 0` trong `pg_stat_subscription_stats` — view thống kê lỗi theo subscription, nơi nên gắn cảnh báo thay vì chờ người đọc log.

**Thứ tự đúng** cho từng loại thay đổi schema khi bảng đang được publish:

| Thay đổi | Làm trước ở | Lý do |
|---|---|---|
| Thêm cột | Subscriber, rồi publisher | Subscriber được phép thừa cột; publisher gửi cột mới khi bên nhận đã có chỗ chứa |
| Xoá cột | Publisher, rồi subscriber | Publisher ngừng gửi trước; xoá ở subscriber trước thì thay đổi tiếp theo lại thiếu cột |
| Đổi kiểu cột | Diễn tập riêng | Giá trị đi qua biểu diễn text; thu hẹp kiểu phải kiểm dữ liệu |
| Thêm bảng vào publication | Tạo bảng ở subscriber, `ALTER PUBLICATION … ADD TABLE`, rồi `ALTER SUBSCRIPTION … REFRESH PUBLICATION` | Subscription không tự biết bảng mới |

Lab chỉ chạy dòng đầu (theo thứ tự sai, để thấy lỗi); các dòng còn lại suy từ tài liệu. Cách nghĩ giống **expand–contract** ở <a href="/articles/postgresql-schema-migration-tuong-thich">PG-009</a>, chỉ khác là «bên cũ» giờ là một database do đội khác quản lý. Migration của primary phải có thêm mục «subscriber nào đang nhận bảng này, ai chạy DDL bên đó, theo thứ tự nào».

## 7. Gỡ subscription đúng cách

Cuối phần 055, runner gỡ subscription và kiểm publisher:

```sql
-- Trên subscriber
DROP SUBSCRIPTION fare_sub;
```

```text
PASS 055 DROP SUBSCRIPTION removed the slot: 0
```

`DROP SUBSCRIPTION` kết nối tới publisher và xoá luôn slot tương ứng; trên publisher không còn slot nào. Đường này chỉ thành công khi publisher còn kết nối được. Nếu cluster publisher đã mất, hoặc ngược lại cluster subscriber đã bị xoá mà slot vẫn nằm trên publisher, tài liệu `DROP SUBSCRIPTION` hướng dẫn tách slot khỏi subscription rồi xoá slot thủ công ở publisher. Dạng lệnh, **không chạy trong lab**:

```sql
-- không chạy trong lab
-- Trên subscriber, khi publisher không kết nối được:
ALTER SUBSCRIPTION fare_sub DISABLE;
ALTER SUBSCRIPTION fare_sub SET (slot_name = NONE);
DROP SUBSCRIPTION fare_sub;
-- Trên publisher, khi nó kết nối lại được (hoặc khi subscriber đã mất hẳn):
SELECT pg_drop_replication_slot('fare_sub');
```

## 8. REPLICA IDENTITY và những gì không đi theo

**REPLICA IDENTITY** là cách publisher cho subscriber biết «dòng nào» bị UPDATE hay DELETE. Bảng `fares` có khoá chính, và mặc định (`REPLICA IDENTITY DEFAULT`) dùng chính khoá đó: subscriber tìm dòng cần sửa theo khoá chính (nếu khoá bị đổi, WAL ghi thêm giá trị khoá cũ). Câu UPDATE chuyến 3 ở mục 4 đi qua đúng đường này.

Bảng không có khoá chính thì khác. Theo tài liệu, nếu bảng nằm trong một publication có publish UPDATE/DELETE mà không có replica identity, **câu UPDATE hoặc DELETE trên publisher sẽ bị từ chối** — ứng dụng đặt xe nhận lỗi, không chỉ bên đối soát. Lựa chọn, **không chạy trong lab**:

```sql
-- không chạy trong lab
ALTER TABLE gps_raw REPLICA IDENTITY USING INDEX gps_raw_device_seen_key; -- index unique, cột NOT NULL
ALTER TABLE gps_raw REPLICA IDENTITY FULL;                                 -- ghi cả dòng cũ vào WAL
```

`FULL` tốn WAL hơn và bắt subscriber tìm dòng khớp toàn bộ giá trị — chậm nếu không có index phù hợp. Tốt nhất là mọi bảng được publish đều có khoá chính.

Những thứ **không** đi theo, theo mục Restrictions của tài liệu PostgreSQL 18:

- **Schema và DDL** (mục 6).
- **Dữ liệu sequence**: identity ở subscriber không tiến theo publisher — vô hại với bản sao chỉ đọc, thành vấn đề khi dùng subscriber làm database chính (PG-059).
- **Large object**, view, materialized view: chỉ bảng được publish; `fare_report` của PG-056 phải dựng lại trên subscriber.

TRUNCATE thì **có** được sao chép. Với bảng phân vùng như `gps` (PG-051), `publish_via_partition_root` quyết định thay đổi mang tên partition hay tên bảng gốc.

Xung đột cũng nằm ngoài lab. Subscriber ghi được, nên nếu ai đó chèn tay dòng `id = 4` vào `fares` bên đối soát, lần publisher chèn chuyến 4 sẽ vi phạm khoá chính và apply dừng như mục 6. Tuỳ chọn `disable_on_error` cho subscription tự tắt thay vì thử lại mãi; lab không thử. Cách phòng đơn giản nhất là không cấp quyền ghi trên bảng được sao chép (<a href="/articles/postgresql-role-grant-least-privilege">PG-041</a>).

Vài tính năng khác, **không chạy trong lab**, để biết mà tra:

```sql
-- không chạy trong lab
-- Chỉ gửi một số cột (column list) và một số dòng (row filter):
CREATE PUBLICATION fare_hcm FOR TABLE fares (id, city, fare) WHERE (city = 'HCM')
  WITH (publish = 'insert');
-- Không gửi lại thay đổi vốn đến từ replication (tránh vòng lặp khi hai chiều):
CREATE SUBSCRIPTION ... WITH (origin = none);
-- Slot logical đi theo khi failover sang standby (PostgreSQL 17+):
CREATE SUBSCRIPTION ... WITH (failover = true);  -- kèm sync_replication_slots trên standby
```

Luật dễ vấp của row filter: nếu publication publish UPDATE hoặc DELETE, `WHERE` chỉ được dùng cột thuộc replica identity; `city` không thuộc khoá chính nên ví dụ trên giới hạn `publish = 'insert'`. Slot logical mặc định **không** đi theo khi failover (PG-058), vì thế mới có tuỳ chọn `failover` từ bản 17.

## 9. CDC ra ngoài PostgreSQL, và khi nào chọn outbox

Bên nhận trong lab là một PostgreSQL khác để kiểm hai đầu bằng SQL. Thực tế, CDC thường đẩy thay đổi vào Kafka qua connector như Debezium: connector tạo slot logical, đọc thay đổi, ghi vào topic rồi xác nhận LSN cho slot. Lab **không chạy** Debezium hay Kafka. Điều bài này chứng minh về slot vẫn áp dụng — slot của connector bị bỏ quên cũng giữ WAL — nhưng ngữ nghĩa giao nhận là của connector, phải kiểm riêng; phía nhận nên chịu được một thay đổi tới **hơn một lần** sau khi connector khởi động lại.

<a href="/articles/postgresql-idempotency-outbox-transaction">PG-039</a> đã giải bài toán «gửi sự kiện ra ngoài» bằng **transactional outbox**: ghi sự kiện vào bảng `outbox` cùng giao dịch với dữ liệu nghiệp vụ, một publisher đọc rồi gửi. Hai cách khác nhau ở chỗ ai định nghĩa thông điệp:

| Tiêu chí | CDC trên bảng nghiệp vụ | Transactional outbox |
|---|---|---|
| Thông điệp | Thay đổi dòng của bảng nội bộ (`fares` cột `fare` đổi từ 50000 thành 55000) | Sự kiện nghiệp vụ do ứng dụng định nghĩa («chuyến 3 được điều chỉnh giá») |
| Hợp đồng với bên nhận | Chính là schema bảng: đổi cột là vỡ bên nhận (mục 6) | Payload có phiên bản, độc lập với schema bảng |
| Bỏ sót thay đổi | Không: mọi thay đổi đã commit đều đi qua WAL, kể cả sửa tay bằng psql | Thay đổi không đi qua code ghi outbox thì không có sự kiện |
| Chi phí trên primary | Slot, WAL `logical`, nguy cơ giữ WAL | Thêm một dòng mỗi giao dịch, publisher polling hoặc… CDC |
| Giao nhận | Ít nhất một lần qua connector | Ít nhất một lần (PG-039 đã đo tin trùng) |

Hai cách không loại trừ nhau. Mẫu thường gặp là **dùng CDC để đọc chính bảng outbox**: ứng dụng vẫn định nghĩa sự kiện, còn slot logical thay publisher polling. Với Tới Nơi, bản sao `fares` cho đối soát — một database PostgreSQL khác, cùng đội vận hành, cần đúng từng dòng — hợp với logical replication. Sự kiện «chuyến hoàn tất» gửi cho dịch vụ khuyến mãi của đội khác thì nên đi qua outbox, để đội đặt xe đổi schema bảng mà không làm vỡ họ.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `pg_wal` trên primary phình to, đĩa sắp đầy | Slot không ai đọc: subscriber tắt, bị xoá mà quên DROP, hoặc apply bị kẹt | Cảnh báo theo `active`, `inactive_since`, khoảng cách tới `restart_lsn`; quyết định trước giữa giới hạn `max_slot_wal_keep_size` và trực sự cố |
| Subscriber ngừng cập nhật, log lặp một lỗi | Schema lệch (thiếu cột), vi phạm ràng buộc hoặc khoá ở bên nhận | Đọc log subscriber và `apply_error_count`; sửa nguyên nhân ở subscriber, không xoá slot |
| UPDATE/DELETE trên primary bị từ chối sau khi thêm bảng vào publication | Bảng không có khoá chính hay replica identity | Thêm khoá chính, hoặc `REPLICA IDENTITY USING INDEX`/`FULL` sau khi cân chi phí |
| Dùng subscriber làm database chính thì identity sinh mã trùng | Sequence không được sao chép | Đặt lại sequence ở bước cutover (PG-059) |
| Xoá cluster đối soát xong, primary vẫn giữ WAL | Slot mồ côi | Kiểm `pg_replication_slots` sau mỗi lần gỡ hệ thống tiêu thụ |

## 11. Đánh đổi

| Cách đưa sổ cước sang đối soát | Độ tươi | Chi phí trên primary | Rủi ro chính |
|---|---|---|---|
| Dump hằng đêm (PG-046) | Một ngày | Một lần đọc lớn mỗi đêm | Dữ liệu cũ; không có tác động liên tục |
| Truy vấn polling theo `updated_at` | Theo chu kỳ | Truy vấn lặp; cần cột và index | Bỏ sót DELETE và sửa không cập nhật `updated_at` |
| Physical standby (PG-052) | Gần thời gian thực | Thấp, nhưng chép cả cluster | Chỉ đọc, cùng phiên bản lớn, không thêm bảng riêng được |
| Logical replication | Gần thời gian thực, theo bảng | `wal_level=logical`, walsender, slot | Slot giữ WAL; DDL hai phía; sequence không đi theo |
| Outbox + broker | Gần thời gian thực, theo sự kiện | Thêm ghi mỗi giao dịch | Phải sửa ứng dụng; trùng tin |

Logical replication rẻ lúc dựng, đắt lúc vận hành: primary gánh thêm một phụ thuộc về dung lượng (không phải về commit) và một quy trình đổi schema hai phía. Chọn nó khi bên nhận cần từng dòng, gần thời gian thực, và có người chịu trách nhiệm giám sát slot.

## 12. Bài tập

**Bài 1.** Trên bản chép riêng của lab, đặt `max_slot_wal_keep_size` nhỏ (ví dụ vài chục MB) cho primary, bỏ `wal_keep_size = 512MB`, tắt subscription rồi sinh đủ WAL để vượt giới hạn.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Kết quả chạy thật của <code>pg_replication_slots</code> cho thấy <code>wal_status</code> đi qua các trạng thái và cuối cùng là <code>lost</code>, kèm cột <code>invalidation_reason</code>. Khi bật lại subscription, log subscriber báo lỗi (dán đúng dòng log của bạn, không chép từ bài). Bạn mô tả cách khôi phục bản sao — dựng lại subscription với initial copy hoặc làm trống bảng đích — và chứng minh bằng đối soát sổ hai bên. Kết luận phải nói rõ đây là đánh đổi: primary được bảo vệ, bản sao phải chép lại.</p>
</details>

**Bài 2.** Viết quy trình thêm cột `surge_multiplier numeric(3,2)` vào `fares` khi bảng đang được publish sang đối soát.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Thứ tự: subscriber trước, publisher sau; DDL nào chạy ở đâu, ai chạy. Một lần chạy thật cho thấy không có dòng <code>missing replicated column</code> trong log subscriber và <code>apply_error_count</code> không tăng. Một dòng có giá trị <code>surge_multiplier</code> ghi trên publisher xuất hiện ở subscriber. Nếu cột có <code>NOT NULL</code>, giải thích giá trị default ở hai phía và vì sao dòng cũ ở subscriber không vi phạm ràng buộc.</p>
</details>

**Bài 3.** Thêm bảng `trip_events` không có khoá chính vào `fare_pub`, rồi chạy một câu UPDATE trên publisher.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Thông báo lỗi thật mà ứng dụng (phía publisher, không phải subscriber) nhận được, kèm SQLSTATE. Hai cách sửa — thêm khoá chính, hoặc <code>REPLICA IDENTITY FULL</code> — và so sánh lượng WAL cho cùng một UPDATE nhiều dòng ở mỗi cách bằng <code>pg_current_wal_lsn()</code> trước/sau, ghi rõ đây là số đo một lần trên dữ liệu nhỏ. Kết luận chọn cách nào cho <code>trip_events</code> và vì sao.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Logical Replication](https://www.postgresql.org/docs/18/logical-replication.html): publication, subscription, initial copy, conflict, giám sát.
- [PostgreSQL 18 · Logical Replication Restrictions](https://www.postgresql.org/docs/18/logical-replication-restrictions.html): DDL, sequence, large object và những gì không được sao chép.
- [PostgreSQL 18 · CREATE SUBSCRIPTION](https://www.postgresql.org/docs/18/sql-createsubscription.html): `copy_data`, `disable_on_error`, `origin`, `failover`, cách ghép cột theo tên.
- [PostgreSQL 18 · pg_replication_slots](https://www.postgresql.org/docs/18/view-pg-replication-slots.html): `restart_lsn`, `confirmed_flush_lsn`, `wal_status`, `safe_wal_size`, `inactive_since`.
- [PostgreSQL 18 · Replication settings](https://www.postgresql.org/docs/18/runtime-config-replication.html): `max_slot_wal_keep_size`, `idle_replication_slot_timeout`.
- [PostgreSQL 18 · DROP SUBSCRIPTION](https://www.postgresql.org/docs/18/sql-dropsubscription.html): xoá slot từ xa và trường hợp publisher không kết nối được.
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-055), kết quả `p06/OUTPUT.txt`, giới hạn trong `p06/README.md`, sản phẩm D5 trong `p06/HO-SO-BAN-GIAO.md`.

Bài trước: [PG-054 · Partition pruning và vòng đời dữ liệu](/articles/postgresql-partition-pruning-retention). Bài tiếp theo: [PG-056 · Materialized view và dữ liệu báo cáo](/articles/postgresql-materialized-view-reporting).

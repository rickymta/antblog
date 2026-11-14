---
slug: rust-event-sourcing-co-chung-muc-projection-replay
title: "Event sourcing có chừng mực: bút toán là sự kiện, projection và replay"
summary: "Bài 49: dùng journal_entries làm event log, dựng projection số dư và thu nhập theo ngày có checkpoint, tái hiện lỗi bỏ sót giao dịch commit muộn trên PostgreSQL, rebuild, đổi phiên bản và biết khi nào không nên."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, postgresql, event-sourcing, projection]
language: vi
---

Finance hỏi thu nhập theo ngày, CS hỏi số dư ví của khách: hai câu hỏi, cùng một nguồn là các bút toán đã ghi. [Bài 48](/articles/rust-so-du-theo-thoi-diem-snapshot-dong-ky) tính lại từ bút toán mỗi khi có người hỏi. Bài này giữ sẵn kết quả trong bảng đọc, cập nhật khi có bút toán mới, và chạy thật chỗ dễ sai nhất của cách đó.

Đây là bài 49 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) và schema append-only của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key), trên PostgreSQL thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Projection đọc log bằng <code>WHERE id &gt; last_id ORDER BY id</code>, với <code>id</code> là cột identity tăng dần. Có thể bỏ sót vĩnh viễn một bút toán đã commit không?</p>
<details><summary>Xem đáp án</summary><p>Có. Identity cấp số lúc <code>INSERT</code>, không phải lúc <code>COMMIT</code>. Giao dịch nhận id nhỏ mà commit muộn sẽ hiện ra sau khi projection đã đi qua id lớn hơn, và truy vấn <code>id &gt; last_id</code> không bao giờ quay lại nhìn nó.</p></details>
</div>

## Sổ cái đã là event log, vậy event sourcing thêm gì?

Event sourcing là lưu chuỗi sự kiện bất biến làm nguồn sự thật, còn trạng thái hiện tại là kết quả gộp (fold) chuỗi đó. Sổ cái đã làm đúng điều ấy: `journal_entries` và `postings` chỉ thêm (trigger bài 24 chặn `UPDATE`, `DELETE`, `TRUNCATE`), số dư chỉ là tổng posting ([bài 13](/articles/rust-double-entry-journal-entry-posting)), sai thì sửa bằng bút toán đảo. Nên lab không tạo bảng sự kiện riêng: mỗi bút toán là một sự kiện.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Log bút toán bất biến đi qua một lô gồm đọc, gộp và ghi checkpoint trong cùng một transaction, rồi cập nhật hai bảng đọc: số dư và thu nhập theo ngày."><defs><marker id="rust-event-sourcing-co-chung-muc-projection-replay-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="55" width="250" height="120" rx="10" class="f-phys-soft s-phys"/><text x="36" y="90" class="t-b">Log bất biến</text><text x="36" y="115" class="t-small">journal_entries + postings</text><text x="36" y="138" class="t-small">chỉ thêm, có tx_id và id</text><rect x="320" y="55" width="240" height="120" rx="10" class="f-pk-soft s-pk"/><text x="336" y="90" class="t-b">Một lô, một transaction</text><text x="336" y="115" class="t-small">đọc sau checkpoint</text><text x="336" y="138" class="t-small">gộp vào bảng đọc</text><text x="336" y="161" class="t-small">ghi checkpoint mới</text><rect x="610" y="25" width="250" height="75" rx="10" class="f-phys-soft s-phys"/><text x="626" y="55" class="t-b">balances</text><text x="626" y="80" class="t-small">số dư từng tài khoản</text><rect x="610" y="130" width="250" height="75" rx="10" class="f-phys-soft s-phys"/><text x="626" y="160" class="t-b">income_by_day</text><text x="626" y="185" class="t-small">thu nhập theo ngày</text><path d="M270 115 L320 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-0-a)"/><path d="M560 100 L610 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-0-a)"/><path d="M560 130 L610 168" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-0-a)"/></svg>
</div>
<figcaption>Hình 1. Hai bảng đọc cùng sinh từ một log. Mất bảng nào thì dựng lại từ log, không mất dữ liệu gốc.</figcaption>
</figure>

## Projection có checkpoint: chạy lại không cộng trùng

Projection là hàm gộp từ sự kiện vào bảng. Lab có `Balances` (số dư từng tài khoản) và `IncomeByDay` (thu nhập theo ngày của `platform:revenue` và các ví `driver:*`). Thu nhập tính posting dương của bút toán thường và posting âm của bút toán đảo (trừ lại phần bị đảo); tiền chi ra khỏi ví tài xế không tính:

```rust
// src/projection.rs
pub fn income_delta(event: &LogEvent, account: &str, amount: i64) -> i64 {
    let income_account = account == "platform:revenue" || account.starts_with("driver:");
    let counts = if event.reversal_of.is_none() {
        amount > 0
    } else {
        amount < 0
    };
    if income_account && counts { amount } else { 0 }
}
```

Mỗi phiên bản projection có một dòng checkpoint, và mỗi projection chỉ có tối đa một phiên bản `active`:

```sql
-- sql/schema.sql
CREATE TABLE projection_checkpoints (
    name          text    NOT NULL,
    version       int     NOT NULL,
    last_tx_id    bigint  NOT NULL DEFAULT 0,
    last_entry_id bigint  NOT NULL DEFAULT 0,
    active        boolean NOT NULL DEFAULT false,
    PRIMARY KEY (name, version)
);
CREATE UNIQUE INDEX projection_checkpoints_one_active
    ON projection_checkpoints (name) WHERE active;
```

Hai cột `last_*` là vị trí đã xử lý; `last_tx_id` được giải thích ở mục sau. Chạy tám bút toán của ba chuyến (`TN-0002` được đảo cả hai bước như [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker)) theo lô 3:

```text
log: 8 bút toán, lag trước khi chạy: 8
lô số dư (limit 3): [3, 3, 2], checkpoint: entry 8
bằng số dư tính thẳng từ postings: true
chạy lại: áp dụng 0 sự kiện, bảng giữ nguyên: true
```

«Tính thẳng từ postings» là truy vấn `SUM` độc lập với projection, dùng làm chuẩn. Số dư cuối: `driver:DR-001` 20.400, `platform:revenue` 7.500, `customer:CU-002` về 0 sau hoàn tiền.

Phép gộp `balance + delta` không idempotent: áp dụng một lô hai lần là cộng đôi. An toàn khi chạy lại đến từ chỗ khác: **bảng và checkpoint đổi trong cùng một transaction**, nên hoặc cả hai đổi, hoặc không gì đổi.

```rust
// src/projection.rs, thân run_batch_with (rút gọn: bỏ kiểm tra vai `active`)
let mut tx = pool.begin().await?;
let (cursor, _active) = lock_checkpoint(&mut tx, p.name(), p.version()).await?;
let events = read_batch(&mut tx, mode, cursor, None, limit).await?;
let Some(last) = events.last().map(LogEvent::cursor) else {
    return Ok(0); // drop tx: không có gì để lưu
};
p.apply(&mut tx, &table, &events).await?;
save_checkpoint(&mut tx, p.name(), p.version(), last).await?;
tx.commit().await?;
```

`lock_checkpoint` dùng `SELECT ... FOR UPDATE`: hai worker cùng projection xếp hàng ở dòng checkpoint. Lab thử ba tình huống, mỗi tình huống một schema:

```text
chết trước COMMIT : lô 3 bút toán bị rollback, checkpoint = 0, bảng có 0 dòng
chạy lại          : bằng số dư chuẩn: true
tách hai transaction (SAI), sau khi chạy lại:
  customer:CU-001      chuẩn  -25.500 VND, projection  -51.000 VND
  driver:DR-001        chuẩn   20.400 VND, projection   40.800 VND
  platform:revenue     chuẩn    7.500 VND, projection   12.600 VND
  (rút gọn: hai tài khoản lệch khác được lược bớt)
4 worker cùng chạy: 8 sự kiện áp dụng, bằng chuẩn: true
```

Cách «SAI» (`apply_commit_without_checkpoint`) commit bảng rồi «chết» trước khi ghi checkpoint ở transaction thứ hai; chạy lại cộng lô đầu lần nữa, `CU-001` bị trừ 51.000 thay vì 25.500.

## id tăng dần có nghĩa là commit theo thứ tự id không?

Không, và đây là lỗi thật của cách đọc log đơn giản nhất. PostgreSQL cấp identity lúc `INSERT` và không thu hồi; tài liệu `nextval` chỉ bảo đảm các phiên nhận số khác nhau, không bảo đảm thứ tự commit. Cách đọc ngây thơ:

```sql
SELECT id, ... FROM journal_entries WHERE id > $1 ORDER BY id LIMIT $2
```

Lab dựng cuộc đua bằng hai transaction trong một hàm, không cần `sleep`. A ghi bút toán chia tiền `TN-0001` (id 1), chưa commit. B ghi chia tiền `TN-0003` (id 2) rồi commit. Projection chạy một lô, thấy id 2, đặt checkpoint là 2. Sau đó A commit.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Giao dịch A nhận id 1 nhưng commit sau B. Projection đọc sau khi B commit, thấy id 2 và đặt checkpoint là 2; sau khi A commit, truy vấn id lớn hơn 2 không bao giờ thấy id 1."><defs><marker id="rust-event-sourcing-co-chung-muc-projection-replay-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="16" class="t-small">thời gian chạy từ trái sang phải</text><rect x="20" y="28" width="130" height="60" rx="10" class="f-phys-soft s-phys"/><text x="34" y="54" class="t-b">A: INSERT</text><text x="34" y="74" class="t-small">nhận id 1</text><rect x="580" y="28" width="130" height="60" rx="10" class="f-warn-soft s-warn"/><text x="594" y="54" class="t-b">A: COMMIT</text><text x="594" y="74" class="t-small">chậm hơn B</text><rect x="160" y="108" width="130" height="60" rx="10" class="f-phys-soft s-phys"/><text x="174" y="134" class="t-b">B: INSERT</text><text x="174" y="154" class="t-small">nhận id 2</text><rect x="300" y="108" width="130" height="60" rx="10" class="f-phys-soft s-phys"/><text x="314" y="134" class="t-b">B: COMMIT</text><text x="314" y="154" class="t-small">hiện ra trước</text><rect x="20" y="215" width="410" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="242" class="t-b">Đọc lần 1: WHERE id &gt; 0</text><text x="36" y="266" class="t-small">thấy id 2, checkpoint = 2</text><rect x="450" y="215" width="410" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="466" y="242" class="t-b">Đọc lần 2: WHERE id &gt; 2</text><text x="466" y="266" class="t-small">id 1 đã commit mà không bao giờ hiện</text><path d="M365 168 L365 215" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-1-a)"/><path d="M645 88 L645 215" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-1-a)"/></svg>
</div>
<figcaption>Hình 2. Checkpoint theo id đã đi qua chỗ id 1 sẽ xuất hiện. Không có lỗi nào được báo, bảng chỉ thiếu một bút toán.</figcaption>
</figure>

Kết quả chạy, dòng đầu là cách ngây thơ:

```text
kịch bản A (INSERT trước, commit sau):
  id > checkpoint     khi giao dịch chậm còn mở: áp dụng 1, lag 0; cuối: MẤT DÒNG, DR-001 phải có 20.400 VND, projection có 0 VND
  xmin + con trỏ id   khi giao dịch chậm còn mở: áp dụng 0, lag 1; cuối: đủ, bằng số dư chuẩn
  xmin + (tx_id, id)  khi giao dịch chậm còn mở: áp dụng 0, lag 1; cuối: đủ, bằng số dư chuẩn
```

Tài xế `DR-001` mất 20.400 đồng trong bảng đọc, sổ vẫn cân, không có lỗi nào báo, và «lag 0» cũng nói dối: projection tin là đã đuổi kịp.

**Sửa bằng khoảng an toàn.** Tài liệu PostgreSQL định nghĩa `xmin` của snapshot là xid nhỏ nhất còn đang chạy: mọi xid nhỏ hơn đã commit hoặc rollback. Lab lưu xid của giao dịch ghi vào cột `tx_id` (mặc định `pg_current_xact_id()::text::bigint`) và chỉ đọc dòng có `tx_id` nhỏ hơn `pg_snapshot_xmin(pg_current_snapshot())`. Khi A còn mở, B có xid lớn hơn A nên bị giữ lại: lô áp dụng 0 sự kiện, lag là 1. A commit thì cả hai hiện đủ. Cái giá: projection chậm đúng bằng thời gian giao dịch chậm nhất còn mở.

Về cột `tx_id`: xid được cấp khi transaction ghi lần đầu, không phải lúc `BEGIN`, và trong savepoint hàm vẫn trả xid của transaction cha (test kiểm). Kiểu `xid8` 64 bit có cả epoch, theo tài liệu không quay vòng trong đời một cài đặt, khác `xid` 32 bit; nó không cast thẳng sang `bigint` nên phải qua `text`. Nhưng xid chỉ có nghĩa trong một cluster: restore sang cluster mới thì bộ đếm chạy lại, nhận qua logical replication thì `tx_id` mang số của nơi gửi, và con trỏ không còn so được với `xmin` tại chỗ. Lab không thử hai trường hợp này.

Khoảng an toàn chưa đủ nếu con trỏ vẫn là `id`. Kịch bản B: giao dịch cũ lấy xid sớm (cập nhật trạng thái thanh toán trước khi ghi bút toán), giao dịch mới ghi id 1 chưa commit, giao dịch cũ ghi id 2 rồi commit. Xid nhỏ hơn mà id lớn hơn: id 2 qua khoảng an toàn trước, checkpoint nhảy lên 2, id 1 bị bỏ sót như trước. Con trỏ phải là cặp `(tx_id, id)`, cùng chiều với khoảng an toàn:

```sql
-- src/log.rs, ReadMode::Safe
SELECT id, tx_id, idempotency_key, reversal_of, ... FROM journal_entries
WHERE (tx_id, id) > ($1, $2) AND (tx_id, id) <= ($3, $4)
  AND tx_id < pg_snapshot_xmin(pg_current_snapshot())::text::bigint
ORDER BY tx_id, id LIMIT $5
```

Điều kiện `<= ($3, $4)` chỉ dùng khi chuyển phiên bản (mục sau); bình thường nó là `i64::MAX`. Kết quả sáu tổ hợp:

| Cách đọc | Kịch bản A (commit muộn) | Kịch bản B (xid cấp sớm) |
| --- | --- | --- |
| `id > checkpoint` | mất dòng | mất dòng |
| `xmin` + con trỏ `id` | đủ | mất dòng |
| `xmin` + con trỏ `(tx_id, id)` | đủ | đủ |

Logical decoding hay một khoá chung để id trùng thứ tự commit là hai hướng khác, lab không thử nên không kết luận. Điểm yếu cần theo dõi: transaction đã có xid mà treo, kể cả `idle in transaction`, giữ `xmin` lại, mọi projection đứng yên và lag chỉ tăng. Transaction chỉ đọc chưa có xid thì không giữ (test kiểm). Tài liệu nói xid cấp từ một bộ đếm chung của cả cluster, nên giao dịch ghi ở database khác cũng có thể giữ; lab không thử. Vì phiên lạ giữ được, `drain` của lab chờ theo trạng thái («`xmin` đã vượt đầu log»), không theo khoảng thời gian cố định.

## Dựng lại từ đầu và đổi phiên bản mà không dừng sổ

`IncomeByDay` v1 có lỗi nghiệp vụ: chia ngày theo UTC. Thanh toán `TN-0002` lúc 17:30 UTC là 00:30 ngày hôm sau giờ Việt Nam, nên v1 cho doanh thu ngày 03/11 là 8.400 và ngày 04/11 là 4.200, trong khi đúng phải là 5.100 và 7.500. Sửa logic là phiên bản 2 và dựng lại từ log.

Sổ vẫn nhận bút toán suốt quy trình bốn bước:

1. `start_rebuild` tạo bảng riêng (`income_by_day_v2`) và checkpoint `active = false`, con trỏ 0.
2. Bản dựng lại chạy các lô như thường, với vai `Build`. Lab cho nó chạy dở 4 bút toán rồi ghi thêm chuyến `TN-0004`.
3. `cutover` trong một transaction: khoá hai checkpoint, cho bản mới bắt kịp tới đúng checkpoint bản cũ, so hai bảng bằng `EXCEPT` hai chiều, đổi tên bảng, lật `active`.
4. Bản cũ giữ lại dưới tên `income_by_day_v1` để lùi lại nếu cần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Log bất biến nuôi cả bảng đang chạy v1 và bảng dựng lại v2; transaction chuyển khoá checkpoint, cho v2 bắt kịp, đổi tên bảng rồi lật cờ active."><defs><marker id="rust-event-sourcing-co-chung-muc-projection-replay-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="190" rx="10" class="f-phys-soft s-phys"/><text x="36" y="70" class="t-b">Log bất biến</text><text x="36" y="95" class="t-small">đọc từ đầu bao nhiêu</text><text x="36" y="115" class="t-small">lần cũng ra cùng thứ tự</text><rect x="250" y="30" width="300" height="80" rx="10" class="f-phys-soft s-phys"/><text x="266" y="60" class="t-b">income_by_day (v1, active)</text><text x="266" y="85" class="t-small">vẫn phục vụ đọc và nhận lô mới</text><rect x="250" y="140" width="300" height="80" rx="10" class="f-pk-soft s-pk"/><text x="266" y="170" class="t-b">income_by_day_v2 (dựng lại)</text><text x="266" y="195" class="t-small">replay từ 0, active = false</text><rect x="590" y="30" width="270" height="190" rx="10" class="f-warn-soft s-warn dash"/><text x="606" y="64" class="t-b">Một transaction chuyển</text><text x="606" y="95" class="t-small">1. khoá hai checkpoint</text><text x="606" y="120" class="t-small">2. v2 bắt kịp tới v1</text><text x="606" y="145" class="t-small">3. đổi tên hai bảng</text><text x="606" y="170" class="t-small">4. lật cờ active</text><path d="M210 70 L250 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-2-a)"/><path d="M210 180 L250 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-2-a)"/><path d="M550 70 L590 90" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-2-a)"/><path d="M550 180 L590 160" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-event-sourcing-co-chung-muc-projection-replay-2-a)"/></svg>
</div>
<figcaption>Hình 3. Người đọc truy vấn bảng tên <code>income_by_day</code>; sau lần đổi tên, cùng tên đó trỏ vào dữ liệu v2.</figcaption>
</figure>

Phần cốt lõi của bước 3 (rút gọn: bỏ kiểm tra vai và chú thích):

```rust
// src/projection.rs, thân cutover
loop {
    let events = read_batch(&mut tx, ReadMode::Safe, new_cursor, Some(old_cursor), 500).await?;
    let Some(last) = events.last().map(LogEvent::cursor) else { break };
    new.apply(&mut tx, &new_table, &events).await?;
    new_cursor = last;
}
let diff = diff_rows(&mut tx, old.live_table(), &new_table).await?;
if require_same && diff != 0 {
    return Err(ProjectionError::Mismatch { diff_rows: diff });
}
// ALTER TABLE <live> RENAME TO <live>_v<old>; ALTER TABLE <live>_v<new> RENAME TO <live>;
// UPDATE projection_checkpoints SET active = false ...; SET active = true ...;
tx.commit().await?;
```

DDL của PostgreSQL chạy trong transaction, nên đổi tên và lật cờ là một bước: người đọc thấy bảng cũ hoặc bảng mới. `ALTER TABLE ... RENAME` lấy khoá `ACCESS EXCLUSIVE`: nó chờ mọi transaction đã đọc bảng kết thúc, truy vấn đến sau xếp hàng sau nó, nên một báo cáo chạy dài chặn được cả lượt; service thật cần `lock_timeout`. Ngoại lệ của «cũ hoặc mới»: transaction `REPEATABLE READ` mở trước khi chuyển, lần đầu đọc bảng sau khi chuyển, sẽ nhìn bảng mới qua snapshot cũ; test của lab đọc được 0 dòng thay vì 7. Bảng đọc nên truy vấn ở `READ COMMITTED`. Kết quả:

```text
số dư v2 (cùng logic): dựng dở 4 bút toán, rồi bắt kịp 6 trong transaction chuyển, lệch 0 dòng
thu nhập v1 -> v2, đòi giống hệt: bị chặn, lệch 8 dòng (rollback, chưa đổi gì)
thu nhập v1 -> v2, chấp nhận khác: bắt kịp 6, lệch 8 dòng
thu nhập trước (v1, ngày UTC, có chuyến TN-0004):
  2026-11-03 platform:revenue     8.400 VND
  2026-11-04 platform:revenue     4.200 VND
thu nhập sau (v2, ngày giờ Việt Nam):
  2026-11-03 platform:revenue     5.100 VND
  2026-11-04 platform:revenue     7.500 VND
bảng cũ income_by_day_v1 còn nguyên: true
bản v1 chạy lô sau khi bị thay: bị từ chối: true
```

(Rút gọn: chỉ trích các dòng `platform:revenue`.) Với cùng logic, `require_same` biến cutover thành phép kiểm: bảng dựng lại phải giống hệt bảng đang chạy, nếu không thì không chuyển. Khi logic đổi, hai bảng phải khác (8 dòng lệch), nên lab thử `require_same = true` để thấy nó bị chặn rồi mới cho phép khác. Cột `active` còn chặn bản cũ: tiến trình v1 chưa tắt mà chạy lô sẽ nhận `WrongRole`, không cộng chồng vào bảng của v2. Với log lớn, hãy cho bản dựng lại chạy sát đầu log trước khi chuyển.

## Lag đo được bao nhiêu?

Lag tính bằng số bút toán đã commit nằm sau checkpoint (`count(*) ... WHERE (tx_id, id) > checkpoint`): lặp lại được, không phụ thuộc đồng hồ. Lab nạp 2.500 chuyến (5.000 bút toán) bằng SQL rồi chạy lô 500:

```text
5000 bút toán trong log, lô 500 bút toán
lag sau từng lô: [5000, 4500, 4000, 3500, 3000, 2500, 2000, 1500, 1000, 500, 0]
dựng lại toàn bộ vào bảng mới: 257 ms (≈ 19450 bút toán/giây), lệch 0 dòng, 72 tài khoản
(số đo trên máy viết bài, không phải SLA)
```

Lag giảm đúng một lô mỗi bước và `--verify` kiểm điều đó. Con số thời gian thì không kiểm, nó là nhiễu: máy viết bài rất tải, các lần chạy khác từ 238 ms tới 2,5 giây, lệch cả chục lần. Chỉ nên rút ra cỡ độ lớn: vài nghìn bút toán dựng lại trong vài giây trở xuống, tăng theo kích thước log. Khi dựng lại tốn hàng giờ mới cần snapshot ở giữa như bài 48.

## So với CQRS ở loạt C#

[Bài CQRS và Mediator](/articles/dotnet10-cqrs-mediator-pipeline) của loạt C# cũng tách ghi và đọc, nhưng trả lời câu hỏi khác.

| Khía cạnh | CQRS (bài C#) | Event sourcing có chừng mực (bài này) |
| --- | --- | --- |
| Quyết định | Tách đường ghi và đường đọc | Chuỗi sự kiện là nguồn sự thật |
| Trạng thái hiện tại | Lưu trong bảng của aggregate | Kết quả gộp, dựng lại được |
| Đường đọc | Query chọn thẳng từ bảng đó, cùng PostgreSQL | Bảng đọc riêng, lệch thì xoá và replay từ log |

Chữ «projection» ở bài C# là phép chọn cột của query, không phải bảng đọc như ở đây. Hai thứ độc lập: CQRS không cần event sourcing (Trip ở C#), event sourcing không cần mediator. Giao sự kiện sang service khác là việc của outbox ở [bài 37](/articles/rust-outbox-inbox-idempotent-consumer).

## Khi nào không nên event sourcing toàn hệ thống?

Mặc định là không. Event sourcing trả giá bằng lag, replay, schema sự kiện phải tương thích mãi mãi và eventual consistency. Giá đó chỉ đáng khi lịch sử là chính dữ liệu. Với Tới Nơi:

| Thành phần | Event sourcing? | Lý do cụ thể |
| --- | --- | --- |
| Sổ cái | Có | Dữ liệu vốn là bút toán bất biến, cần kiểm toán và đối soát |
| Chuyến (8 trạng thái) | Không | Máy trạng thái nhỏ, truy vấn theo trạng thái hiện tại; cần lịch sử thì thêm bảng lịch sử |
| Vị trí tài xế | Không | Khối lượng lớn, giá trị hết sau vài giây, chỉ cần điểm mới nhất |
| Hồ sơ người dùng, PII | Không | Xoá hoặc ẩn dữ liệu cá nhân mâu thuẫn với log bất biến (bài 50 nói PII tối thiểu) |
| Khuyến mãi, ngân sách | Không | Cần chặn vượt ngân sách ngay lúc ghi, mà projection luôn trễ |
| Thông báo | Không | Phát một lần rồi thôi, Kafka đã giữ vai trò truyền |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Một PostgreSQL local, log tối đa 5.000 bút toán, không có tải ghi liên tục lúc đo. Chưa có giám sát lag, chưa đặt <code>idle_in_transaction_session_timeout</code> để chặn giao dịch treo, chưa dọn bảng <code>_v1</code>, chưa thử logical decoding, chưa kiểm <code>xmin</code> trên replica, với nhiều database, hay sau restore. <code>NaiveId</code>, <code>HorizonId</code> và <code>apply_commit_without_checkpoint</code> chỉ để minh hoạ, đừng sao chép vào service thật.</p></div>

## Chạy ví dụ và kiểm chứng

PostgreSQL chạy bằng Docker Compose trong `examples/toi-noi-rust/infra`; mỗi thí nghiệm có schema `lab_<uuid>` riêng và xoá khi xong. Thiếu `TOI_NOI_PG_URL` thì lab thoát mã 1. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/49-projection-replay/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/49-projection-replay/Cargo.toml
```

`--verify` in các kết quả trên rồi kiểm từng giá trị; đạt thì in `VERIFY OK`, thoát mã 0. `cargo test` chạy 12 test tích hợp. Cả hai không kiểm thời gian. Lab dùng `sqlx` 0.9.0, `tokio` 1.54.1.

**Bài tập:** `IncomeByDay` v3 loại bút toán `trip:TN-0003:*` khỏi thu nhập. Bạn đổi `income_delta`, dựng lại v3 và chạy `cutover` với `require_same = true`. Điều gì xảy ra, và vì sao đúng?

<details><summary>Xem lời giải</summary><p>Cutover trả <code>Mismatch</code> và rollback, bảng đang chạy không đổi: logic mới cố ý cho số khác nên «giống hệt» không còn là điều kiện đúng. Với thay đổi có chủ đích, dùng <code>require_same = false</code> sau khi xem bảng khác biệt, chắc rằng chỉ dòng của <code>TN-0003</code> đổi. <code>require_same = true</code> dành cho dựng lại cùng logic, nơi mọi khác biệt là bug.</p></details>

Bài trước: [bài 48](/articles/rust-so-du-theo-thoi-diem-snapshot-dong-ky). Bài tiếp: [bài 50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii).

## Tài liệu gốc

- [PostgreSQL: System Information Functions, `pg_current_snapshot`, `pg_snapshot_xmin` và `pg_current_xact_id`](https://www.postgresql.org/docs/current/functions-info.html).
- [PostgreSQL: Sequence Manipulation Functions, mục `nextval`](https://www.postgresql.org/docs/current/functions-sequence.html).
- [PostgreSQL: ALTER TABLE, mục RENAME và khoá](https://www.postgresql.org/docs/current/sql-altertable.html) và [Explicit Locking, `SELECT ... FOR UPDATE`](https://www.postgresql.org/docs/current/explicit-locking.html).
- [PostgreSQL: Logical Decoding](https://www.postgresql.org/docs/current/logicaldecoding.html).
- [docs.rs: sqlx 0.9.0, `Transaction`](https://docs.rs/sqlx/0.9.0/sqlx/struct.Transaction.html) và [`AssertSqlSafe`](https://docs.rs/sqlx/0.9.0/sqlx/struct.AssertSqlSafe.html).

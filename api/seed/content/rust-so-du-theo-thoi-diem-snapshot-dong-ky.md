---
slug: rust-so-du-theo-thoi-diem-snapshot-dong-ky
title: "Số dư theo thời điểm, snapshot và đóng kỳ"
summary: "Bài 48: tính số dư lúc 23:59 giờ Việt Nam bằng ranh giới nửa mở, so snapshot với tính lại trên 150.400 posting, đóng kỳ bằng trigger, sửa sai ở kỳ sau và dựng cuộc đua ghi/đóng kỳ bằng Barrier."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, postgresql, snapshot, period-close]
language: vi
---

Đây là bài 48 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao) đối soát theo từng dòng; báo cáo thì cần số dư vào đúng một thời điểm. Sáng 06/11, kế toán hỏi số dư sổ của `psp:vixanh:clearing` (không phải tiền thật ở VíXanh, như [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen) đã nói) lúc 23:59 ngày 05/11. Bốn truy vấn trông đều hợp lý cho ra bốn con số khác nhau, và chỉ một con số đúng.

Bài này chạy cả bốn trên PostgreSQL thật, rồi đi tiếp tới snapshot hằng ngày, đóng kỳ, và cuộc đua giữa lần ghi đang chạy với lần đóng kỳ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Truy vấn <code>WHERE posted_at::date &lt;= '2026-11-05'</code> chạy trên server đặt múi giờ UTC, để lấy số dư cuối ngày 05/11 giờ Việt Nam. Nó sai theo hướng thừa hay thiếu bút toán?</p>
<details><summary>Xem đáp án</summary><p>Thừa. Ngày UTC 05 kết thúc lúc 07:00 sáng 06/11 giờ Việt Nam, nên mọi bút toán từ 00:00 đến 07:00 ngày 06 bị kéo vào ngày 05.</p></details>
</div>

## «Số dư lúc 23:59» hỏi về ranh giới nào?

`posted_at` là `timestamptz`: PostgreSQL lưu một mốc tuyệt đối, múi giờ chỉ xuất hiện khi đọc ra. Câu hỏi thật của kế toán là «tổng mọi posting có `posted_at` trước 00:00 ngày 06/11 giờ Việt Nam». Chữ «23:59» chỉ là cách nói: viết `<= 23:59:00` thì mất bút toán lúc 23:59:30.

Cách gọn là ranh giới nửa mở `[from, to)`. Ngày 05 là `[05 00:00, 06 00:00)`, ngày 06 bắt đầu đúng chỗ ngày 05 dừng, hai ngày liền nhau không trùng và không hở. Lab tính mốc bằng `jiff`, đọc múi giờ theo tên chứ không cộng tay 7 tiếng:

```rust
// src/tz.rs (rút gọn: bỏ chú thích)
pub fn day_bounds(day: Date) -> Result<(Timestamp, Timestamp), Error> {
    let from = day.in_tz(VN_TZ)?.timestamp();
    let to = day.tomorrow()?.in_tz(VN_TZ)?.timestamp();
    Ok((from, to))
}
```

Lab so với `AT TIME ZONE 'Asia/Ho_Chi_Minh'` trong SQL:

```text
jiff day_bounds(2026-11-05) = [2026-11-04T17:00:00Z, 2026-11-05T17:00:00Z)
SQL AT TIME ZONE            = 2026-11-05T17:00:00Z
```

Bốn bút toán quanh nửa đêm, ghi theo thứ tự chúng tới sổ cái:

| Bút toán | `posted_at` (giờ Việt Nam) | `psp:vixanh:clearing` |
| --- | --- | --- |
| JE-000001 capture `TN-0001` | 05/11 23:58:00 | +25.500 |
| JE-000002 capture `TN-0002` | 06/11 00:00:00, đúng ranh giới | +12.000 |
| JE-000003 capture `TN-0003` | 05/11 23:59:59,5 | +12.000 |
| JE-000004 chia `TN-0001` | 06/11 00:10:00 | −25.500 |

Bốn điều kiện `WHERE` trên `e.posted_at` và kết quả chạy thật:

```sql
e.posted_at < '2026-11-06 00:00:00+07'                       -- nửa mở, giờ Việt Nam
e.posted_at::date <= date '2026-11-05'                       -- ngày theo múi giờ phiên (UTC)
e.posted_at between '2026-11-05 00:00:00+07' and '2026-11-05 23:59:59+07'
e.posted_at <= '2026-11-06 00:00:00+07'                      -- đóng ở đầu phải
```

```text
psp:vixanh:clearing cuối ngày 05/11:
  posted_at < 06/11 00:00+07 (nửa mở, đúng) =   37500
  posted_at::date <= '2026-11-05' (ngày UTC) =   24000
  BETWEEN ... 23:59:59 (đóng, mất 23:59:59,5) =   25500
  posted_at <= 06/11 00:00+07 (đóng ở đầu phải) =   49500
```

Chỉ dòng đầu đúng: 25.500 + 12.000 = 37.500 (dữ liệu này chưa có lịch sử trước đó). Ngày UTC kéo cả bốn bút toán vào ngày 05: 24.000. `BETWEEN` mất bút toán 23:59:59,5. `<=` ở đầu phải kéo bút toán đúng 00:00 của ngày sau vào: 49.500.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. Bốn bút toán quanh nửa đêm giờ Việt Nam. Ranh giới nửa mở đặt bút toán 00:00:00 vào ngày 06. Ngày UTC 05 kết thúc lúc 07:00 sáng giờ Việt Nam nên gom cả bốn bút toán."><rect x="20" y="30" width="520" height="120" rx="10" class="f-phys-soft s-phys"/><rect x="540" y="30" width="320" height="120" rx="10" class="f-pk-soft s-pk"/><text x="36" y="56" class="t-b">Ngày 05/11 giờ Việt Nam</text><text x="556" y="56" class="t-b">Ngày 06/11</text><path d="M40 110 L840 110" class="f-none s-phys" stroke-width="2"/><path d="M540 24 L540 160" class="f-none s-warn dash" stroke-width="2"/><circle cx="120" cy="110" r="7" class="f-phys-soft s-phys"/><circle cx="440" cy="110" r="7" class="f-phys-soft s-phys"/><circle cx="540" cy="110" r="7" class="f-warn-soft s-warn"/><circle cx="700" cy="110" r="7" class="f-phys-soft s-phys"/><text x="60" y="92" class="t-small">TN-0001 capture 23:58:00</text><text x="350" y="134" class="t-small">TN-0003 23:59:59,5</text><text x="556" y="92" class="t-small">TN-0002 00:00:00</text><text x="640" y="134" class="t-small">TN-0001 chia 00:10:00</text><text x="548" y="18" class="t-small">00:00 giờ Việt Nam = 17:00 UTC</text><rect x="20" y="180" width="840" height="64" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="206" class="t-b">Ngày UTC 05 gom cả bốn bút toán: 24.000 thay vì 37.500</text><text x="36" y="228" class="t-small">Ngày UTC 05 kết thúc lúc 07:00 sáng 06/11 giờ Việt Nam, không phải lúc nửa đêm</text></svg>
</div>
<figcaption>Hình 1. Ranh giới nửa mở (đường đứt) đặt bút toán 00:00:00 vào ngày 06; ngày UTC không cắt ở đó.</figcaption>
</figure>

## Thời điểm xảy ra và thời điểm ghi khác nhau thế nào?

JE-000003 xảy ra lúc 23:59:59,5 ngày 05 nhưng sự kiện tới sổ cái lúc 00:03 ngày 06. Cột `posted_at` nên mang giá trị nào?

- **DB cấp** (`default now()`): `posted_at` là lúc transaction ghi bắt đầu. Bút toán trễ rơi vào ngày 06, số dư ngày 05 phụ thuộc sự kiện tới nhanh hay chậm.
- **Nghiệp vụ truyền**: `posted_at` lấy từ sự kiện. Gửi lại cùng sự kiện cho cùng `posted_at`, số dư ngày 05 đúng dù bút toán tới trễ.

Lab chọn cách thứ hai và giữ lúc ghi ở một cột riêng:

```sql
-- sql/01_schema.sql (rút gọn: hai cột của journal_entries)
posted_at   timestamptz not null default now(),  -- thời điểm kế toán: quyết định kỳ
recorded_at timestamptz not null default now()   -- thời điểm ghi: để kiểm toán
```

Cùng dữ liệu, hai cột, hai câu hỏi:

```text
  theo posted_at  < 06/11 00:00+07 =   37500  (sự thật về ngày 05)
  theo recorded_at < 06/11 00:00+07 =   25500  (điều hệ thống biết lúc 00:00)
```

37.500 là số dư ngày 05 như ta biết hôm nay. 25.500 là con số báo cáo in lúc 00:00 đã thấy. Bên đầu cho báo cáo kế toán, bên sau trả lời «sao hôm qua báo khác?». Cái giá của `posted_at` do nghiệp vụ truyền là nó nhận giá trị từ bên ngoài: lab chỉ chặn khi nó rơi vào kỳ đã đóng hoặc ngoài mọi kỳ.

## Snapshot hằng ngày giúp được bao nhiêu?

Tính lại từ đầu đúng nhưng đắt khi lịch sử dài. `balance_snapshots(account_id, as_of, balance_dong)` lưu số dư ngay trước `as_of`: snapshot cuối ngày 29/10 có `as_of` là 00:00 ngày 30/10 giờ Việt Nam. Truy vấn lấy snapshot mới nhất không quá mốc hỏi, rồi cộng các posting trong `[as_of, mốc)`:

```sql
-- src/snapshot.rs: SNAPSHOT_DELTA_SQL
with snap as (
    select as_of, balance_dong from balance_snapshots
     where account_id = $1 and as_of <= $2::timestamptz
     order by as_of desc limit 1)
select (coalesce((select balance_dong from snap), 0) + coalesce(sum(p.amount_dong), 0))::bigint
  from postings p join journal_entries e on e.id = p.entry_id
 where p.account_id = $1
   and e.posted_at >= coalesce((select as_of from snap), '-infinity'::timestamptz)
   and e.posted_at <  $2::timestamptz
```

Để đo, lab sinh 32.000 chuyến từ 01/07 đến 31/10/2026: 60.800 bút toán, 150.400 posting, 402 tài khoản; riêng `psp:vixanh:clearing` có 60.800 posting. Snapshot 123 ngày là 49.446 dòng, dựng một lượt bằng window function trong chừng 1 giây (số đo trên máy viết bài, không phải SLA). Số dư lúc 23:59 ngày 30/10 của clearing ra 95.247.000 đồng ở cả hai cách. `EXPLAIN (ANALYZE, BUFFERS)`, chạy lần hai cho cache nóng, rút gọn:

```text
tính lại toàn bộ:
  Hash Join  (actual rows=60306.00)
    Seq Scan on postings p  (actual rows=60800.00)  Rows Removed by Filter: 89600  Buffers: shared hit=1254
    Seq Scan on journal_entries e  (actual rows=60306.00)  Rows Removed by Filter: 494  Buffers: shared hit=569
  Execution Time: 61.641 ms        Buffers: shared hit=1823

snapshot + chênh:
  Index Scan Backward using balance_snapshots_pkey  (actual rows=1.00)  Buffers: shared hit=4
  Nested Loop  (actual rows=494.00)
    Index Scan using journal_entries_posted_at_idx on journal_entries e  (actual rows=494.00)  Buffers: shared hit=13
    Index Scan using postings_account_entry_idx on postings p  (actual rows=1.00 loops=494)  Buffers: shared hit=1976
  Execution Time: 0.876 ms         Buffers: shared hit=1989

phần chênh của psp:vixanh:clearing: 494 posting nằm trên 12 trang heap
TIMING OFF: toàn bộ Execution Time: 36.328 ms | snapshot + chênh Execution Time: 1.401 ms
```

Thời gian giảm cỡ 70 lần mà buffer không giảm (1.823 và 1.989). Lạ, nếu coi buffer là thước đo công việc. Nhưng `Buffers` đếm số lần chạm trang, không phải số trang khác nhau. Bên tính lại chạm mỗi trang của hai bảng đúng một lần (1.254 + 569), và phải lọc 150.400 + 60.800 dòng, dựng hash 60.306 dòng, dò 60.800 lần. Bên snapshot có 1.976 hit ở một index scan lặp 494 vòng, mỗi vòng 4 hit (xuống cây index, đọc heap). 494 posting đó chỉ nằm trên 12 trang heap, nên phần lớn là chạm lại trang vừa chạm, và cả câu xử lý chưa tới 1.000 dòng. Khi cache nóng, một hit rất rẻ; thời gian đi theo số dòng xử lý, không theo số hit. Lab không đo cache lạnh.

Con số 70 cũng bị thổi phồng: `EXPLAIN ANALYZE` đọc đồng hồ ở từng dòng của từng node, nên câu nhiều dòng chịu thêm. Với `TIMING OFF`, bốn lần chạy cho khoảng 25–40 lần.

Với khách bình thường, `customer:CU-0042` có 107 posting: 431 so với 123 buffer, cả hai chừng 1 ms, chênh lệch nằm trong nhiễu. Kết luận đo được: snapshot đáng cho tài khoản nóng, và phần chênh vẫn tốn theo số bút toán từ snapshot tới mốc hỏi. Muốn giảm nữa thì snapshot dày hơn, hoặc chép `posted_at` vào `postings` để lọc theo `(account_id, posted_at)`; bài này chưa đo cách thứ hai. Thêm 400 mẫu (tài khoản × mốc, một nửa đúng ranh giới ngày): hai cách khớp cả 400.

## Khi nào snapshot sai, và dựng lại từ posting thế nào?

Snapshot chỉ đúng nếu sau khi nó được tạo, không có posting nào của tài khoản đó có `posted_at` trước `as_of` được ghi thêm. Thử: job tạo snapshot ngày 05 lúc 00:01, rồi capture `TN-0003` tới lúc 00:03 với `posted_at` 23:59:59,5.

```text
  không có trigger: snapshot + chênh = 25500, tính lại = 37500, snapshot còn lại của psp:vixanh:clearing = 1
  có trigger vô hiệu snapshot: snapshot + chênh = 37500, tính lại = 37500, snapshot còn lại của psp:vixanh:clearing = 0
```

Không có gì báo lỗi: truy vấn trả 25.500 một cách tự tin, lệch 12.000. Chữa bằng một trigger xoá snapshot bị posting ghi lùi vượt qua; truy vấn lùi về snapshot cũ hơn, hoặc tính từ đầu, và vẫn đúng:

```sql
-- sql/04_invalidate_snapshots.sql (rút gọn: bỏ chú thích)
create function invalidate_snapshots() returns trigger
language plpgsql as $$
begin
    delete from balance_snapshots s
     using journal_entries e
     where e.id = new.entry_id
       and s.account_id = new.account_id
       and s.as_of > e.posted_at;
    return null;
end;
$$;
create trigger postings_invalidate_snapshots after insert on postings
    for each row execute function invalidate_snapshots();
```

Dấu `>` là nửa mở: bút toán đúng 00:00 ngày 06 không làm hỏng snapshot cuối ngày 05. Proptest kiểm chỗ này: chuỗi 1–13 thao tác ghi bút toán hoặc chạy job snapshot, `posted_at` có cả đúng nửa đêm và trước nửa đêm một giây, rồi so hai cách ở mọi mốc:

```rust
// tests/snapshot_props.rs (rút gọn)
fn secs() -> impl Strategy<Value = i64> {
    prop_oneof![
        2 => 0..5 * 86_400i64,
        1 => (1..5i64).prop_map(|d| d * 86_400),
        1 => (1..5i64).prop_map(|d| d * 86_400 - 1),
    ]
}
```

16 ca chạy xanh. Tạm bỏ dòng `create trigger` thì test đỏ, và proptest rút gọn về ba thao tác: ghi một bút toán, chạy job snapshot, ghi lùi vào ngày đã chốt.

Snapshot cũng có thể sai vì ai đó sửa tay. Lab cộng 1 đồng vào snapshot cuối ngày 29/10 của clearing:

```text
snapshot sai +1 đồng: truy vấn lúc 12:00 ngày 30/10 = 94848001, đúng là 94848000; verify tìm thấy 1 dòng lệch: Some(Mismatch { account: "psp:vixanh:clearing", as_of: "2026-10-29T17:00:00Z", stored: 94467001, expected: 94467000 })
dựng lại 3 ngày từ posting (1206 dòng): còn 0 dòng lệch, truy vấn = 94848000
```

Posting là sự thật, snapshot là bản nhớ đệm: `verify_snapshots` so từng snapshot với tính lại từ đầu, `rebuild_snapshots` dựng lại một khoảng ngày chỉ từ posting.

## Đóng kỳ cấm gì, và sửa sai ở kỳ đã đóng bằng gì?

Báo cáo tháng cần lời hứa mạnh hơn snapshot: con số đã công bố không đổi nữa. Đó là việc của kỳ kế toán:

```sql
-- sql/01_schema.sql (rút gọn)
create table accounting_periods (
    id bigint generated always as identity primary key,
    label text not null unique,
    starts_at timestamptz not null,
    ends_at timestamptz not null,
    status text not null default 'OPEN' check (status in ('OPEN', 'CLOSED')),
    closed_at timestamptz,
    check (starts_at < ends_at),
    check ((status = 'CLOSED') = (closed_at is not null)),
    exclude using gist (tstzrange(starts_at, ends_at) with &&)
);
```

Kỳ cũng nửa mở, và `EXCLUDE` cấm hai kỳ chồng nhau. Trigger `BEFORE INSERT` trên `journal_entries` chỉ cho ghi khi `posted_at` rơi vào một kỳ đang mở; trigger thứ hai trên `postings` chặn thêm dòng vào bút toán cũ của kỳ đã đóng. Chỉ cần kiểm lúc `INSERT`, vì sổ đã append-only từ [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key); kỳ đã đóng cũng có trigger cấm mở lại.

```sql
-- sql/02_period_guard.sql (rút gọn: bỏ chú thích và lệnh tạo trigger)
create or replace function guard_period() returns trigger
language plpgsql as $$
declare
    p accounting_periods%rowtype;
begin
    select * into p from accounting_periods
     where new.posted_at >= starts_at and new.posted_at < ends_at
       for share;
    if not found then
        raise exception 'chưa có kỳ kế toán chứa posted_at = %', new.posted_at
            using errcode = 'restrict_violation';
    end if;
    if p.status = 'CLOSED' then
        raise exception 'kỳ % đã đóng: không ghi posted_at = % vào kỳ này', p.label, new.posted_at
            using errcode = 'restrict_violation';
    end if;
    return new;
end;
$$;
```

`FOR SHARE` sẽ cần ở phần sau. Đóng kỳ là một transaction: khoá dòng kỳ, chỉ cho đóng theo thứ tự, ghi snapshot cuối kỳ cho mọi tài khoản, rồi đổi trạng thái.

```rust
// src/period.rs: close_period (rút gọn: bỏ các nhánh lỗi)
let mut tx = conn.begin().await?;
// select ... from accounting_periods where label = $1 for update
// kiểm kỳ trước còn mở không
let cut = Timestamp::from_second(ends)?;
let rows = write_snapshot(&mut tx, cut).await?;
// update accounting_periods set status = 'CLOSED', closed_at = $2 where id = $1
tx.commit().await?;
```

Chạy thật, `now` do lab truyền vào:

```text
đóng 2026-10: 6 snapshot cuối kỳ
đóng 2026-12 trước 2026-11: không đóng kỳ 2026-12 được: kỳ trước 2026-11 còn mở
đóng 2026-11: 6 snapshot cuối kỳ
ghi posted_at = 2026-11-05T17:10:00Z vào kỳ đã đóng:
  SQLSTATE 23001: kỳ 2026-11 đã đóng: không ghi posted_at = 2026-11-05 17:10:00+00 vào kỳ này
```

Giờ tình huống thật: bút toán chia JE-000004 của `TN-0001` ghi phí 4.500 thay vì 5.100, phát hiện sau khi kỳ 11 đóng. Cách sửa không phải ghi lùi. Bút toán thứ nhất đảo JE-000004 (`JournalEntry::reversal_of` của `ledger-core`, cột `reversal_of` trỏ về nó, cách đảo như [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker)), bút toán thứ hai là bản đúng, cả hai có `posted_at` ở kỳ 12. Số JE-000005 thuộc về lần ghi vừa bị từ chối: identity không lùi lại.

```text
điều chỉnh ở kỳ 2026-12: JE-000006 đảo JE-000004, JE-000007 là bút toán đúng
platform:revenue cuối kỳ 11 (đã báo cáo) = 4500; đến hết 02/12 = 5100
```

Báo cáo kỳ 11 vẫn là 4.500, đúng như đã công bố, và vết sai sót nằm ở kỳ 12 với tham chiếu rõ. Cái giá: người đọc báo cáo cần biết số dư «đã công bố» khác số dư «đúng hôm nay».

## Ghi đang chạy lúc đóng kỳ thì ai thắng?

Trigger kiểm `CLOSED` chưa đủ. Một lần ghi với `posted_at` ngày 30/11 đã `INSERT` xong, chưa commit; trigger đã thấy kỳ 11 còn mở và cho qua. Lúc đó đóng kỳ: snapshot chỉ thấy dữ liệu đã commit nên không có bút toán kia, kỳ thành `CLOSED`. Rồi lần ghi dở commit, và kỳ đã đóng có thêm một bút toán mà snapshot không biết.

Lab dựng đúng chuỗi đó bằng `Barrier` và kênh, không `sleep`:

```rust
// src/race.rs: run_write_then_close (rút gọn)
let barrier = Arc::new(Barrier::new(2));
let (release_tx, release_rx) = oneshot::channel::<()>();
// writer:
post_on(&mut tx, &late, at, at, None).await?;   // chèn xong, chưa commit
barrier.wait().await;
release_rx.await?;
tx.commit().await?;
// closer (task khác), chỉ chạy sau barrier.wait() của main:
close_period(&mut conn, "2026-11", closed_at).await
```

Bất biến cần kiểm là «snapshot cuối kỳ bằng tính lại từ posting». Phần chờ cũng không dùng thời gian: main hỏi `pg_stat_activity` xem closer đã ở `wait_event_type = 'Lock'` chưa, hoặc đã xong chưa. Bản sai (trigger không có `FOR SHARE`, đóng kỳ không khoá) và bản đúng cho:

```text
trigger không khoá, đóng kỳ không khoá: RaceOutcome { closer_waited: false, writer_committed: true, mismatches: 1, snapshot_psp: 25500, full_psp: 37500 }
FOR SHARE ở trigger, FOR UPDATE ở đóng kỳ: RaceOutcome { closer_waited: true, writer_committed: true, mismatches: 0, snapshot_psp: 37500, full_psp: 37500 }
```

Bản đúng có hai khoá hàng trên chính dòng kỳ. Trigger giữ `FOR SHARE` tới hết transaction ghi; nhiều lần ghi không chặn nhau. `close_period` mở đầu bằng `FOR UPDATE`, xung khắc với `FOR SHARE`, nên phải đợi mọi lần ghi đang dở xong rồi mới tính snapshot. Ở READ COMMITTED, câu tính snapshot bắt đầu sau khi có khoá nên thấy bút toán vừa commit: 37.500 ở cả hai phía.

Thứ tự ngược lại: closer giữ khoá trước, lần ghi đến sau bị chặn ở trigger, và khi closer commit thì nó thấy `CLOSED`:

```text
đóng kỳ giữ khoá trước, lần ghi đến sau: CloseFirstOutcome { writer_blocked: true, writer_error: Some(("23001", "kỳ 2026-11 đã đóng: không ghi posted_at = 2026-11-30 16:59:00+00 vào kỳ này")) }
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 296" role="img" aria-label="Hình 2. Hai thứ tự khi ghi gặp đóng kỳ. Không khoá: closer tính snapshot trong lúc writer chưa commit, writer commit sau và snapshot thiếu bút toán. Có khoá: closer đợi FOR UPDATE tới khi writer commit, rồi mới tính snapshot và thấy bút toán."><path d="M440 20 L440 284" class="f-none s-line dash" stroke-width="2"/><text x="30" y="30" class="t-b">Không khoá</text><text x="30" y="52" class="t-small">Writer</text><text x="235" y="52" class="t-small">Closer</text><text x="470" y="30" class="t-b">FOR SHARE + FOR UPDATE</text><text x="470" y="52" class="t-small">Writer</text><text x="675" y="52" class="t-small">Closer</text><rect x="30" y="64" width="185" height="44" rx="8" class="f-phys-soft s-phys"/><text x="42" y="84" class="t-small">INSERT, kỳ còn OPEN</text><rect x="235" y="120" width="185" height="44" rx="8" class="f-phys-soft s-phys"/><text x="247" y="140" class="t-small">tính snapshot</text><text x="247" y="156" class="t-small">chưa thấy dòng mới</text><rect x="235" y="176" width="185" height="44" rx="8" class="f-phys-soft s-phys"/><text x="247" y="196" class="t-small">CLOSED, COMMIT</text><rect x="30" y="232" width="185" height="52" rx="8" class="f-warn-soft s-warn"/><text x="42" y="252" class="t-small">COMMIT</text><text x="42" y="270" class="t-small">snapshot thiếu 12.000</text><rect x="470" y="64" width="185" height="44" rx="8" class="f-phys-soft s-phys"/><text x="482" y="84" class="t-small">INSERT, giữ FOR SHARE</text><rect x="675" y="120" width="185" height="44" rx="8" class="f-warn-soft s-warn dash"/><text x="687" y="140" class="t-small">FOR UPDATE: phải đợi</text><rect x="470" y="176" width="185" height="44" rx="8" class="f-phys-soft s-phys"/><text x="482" y="196" class="t-small">COMMIT, nhả khoá</text><rect x="675" y="232" width="185" height="52" rx="8" class="f-pk-soft s-pk"/><text x="687" y="252" class="t-small">tính snapshot, CLOSED</text><text x="687" y="270" class="t-small">thấy bút toán: 37.500</text></svg>
</div>
<figcaption>Hình 2. Thời gian chảy từ trên xuống. Bên trái closer bỏ sót lần ghi dở; bên phải closer đợi writer commit.</figcaption>
</figure>

Khoá có giá. Đóng kỳ phải đợi lần ghi chậm nhất đang dở, nên transaction ghi phải ngắn. Lúc tính snapshot, các lần ghi mới vào kỳ đó bị chặn; job snapshot hằng ngày của lab cũng khoá `FOR UPDATE` mọi kỳ còn mở trước mốc, nên chặn ghi suốt lúc tính. Lab chưa đo đói khoá: khi `FOR UPDATE` đang đợi mà các lần ghi `FOR SHARE` mới vẫn tới, bên nào phải chờ và chờ bao lâu.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL lab theo `examples/toi-noi-rust/infra/README.md`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/48-snapshot-period-close/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/48-snapshot-period-close/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/48-snapshot-period-close/Cargo.toml
```

Lệnh đầu in mọi kết quả trích ở trên; thời gian mỗi lần mỗi khác. Lệnh thứ hai kiểm các số nghiệp vụ rồi in `VERIFY OK`, không kiểm thời gian. Lệnh thứ ba chạy 3 test múi giờ, 7 test PostgreSQL và 1 property test; thiếu `TOI_NOI_PG_URL` thì thất bại kèm cách bật compose. Schema `lab_<uuid>` bị xoá cuối ca kể cả khi lỗi.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Kỳ là tháng theo giờ Việt Nam, không có kỳ điều chỉnh cuối năm hay mở lại có phê duyệt. Lab chưa phân quyền ai được đóng kỳ (bài 50), chưa có job theo lịch, chưa chặn <code>posted_at</code> ở tương lai xa, chưa có cột «thời điểm xảy ra thật» cho bút toán điều chỉnh; <code>recorded_at</code> do lab truyền để kết quả tất định, production để database cấp. Dữ liệu đo do SQL sinh đều đặn, một node PostgreSQL ở READ COMMITTED; số đo của máy viết bài lúc tải cao, không phải SLA. Đây là mô hình học tập, không phải hướng dẫn kế toán hay quy định bắt buộc.</p>
</div>

**Bài tập:** Nếu `close_period` tính snapshot trước rồi mới `SELECT ... FOR UPDATE` dòng kỳ, kết quả giống bản đúng hay bản sai ở trên?

<details><summary>Xem lời giải</summary><p>Giống bản sai. Snapshot đã tính khi lần ghi dở chưa commit; <code>FOR UPDATE</code> đến sau chỉ đợi writer commit rồi đóng kỳ, nên kỳ đã đóng có bút toán mà snapshot không có. Khoá phải lấy trước khi đọc dữ liệu để tính.</p></details>

Bài trước: [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao). Bài tiếp: [bài 49](/articles/rust-event-sourcing-co-chung-muc-projection-replay).

## Tài liệu gốc

- [PostgreSQL: Explicit Locking, mục Row-Level Locks](https://www.postgresql.org/docs/current/explicit-locking.html).
- [PostgreSQL: Date/Time Functions, `AT TIME ZONE` và `now()`](https://www.postgresql.org/docs/current/functions-datetime.html).
- [PostgreSQL: Range Types, mục Constraints on Ranges (EXCLUDE)](https://www.postgresql.org/docs/current/rangetypes.html#RANGETYPES-CONSTRAINT).
- [docs.rs: `jiff`, múi giờ và `civil::Date::in_tz`](https://docs.rs/jiff/latest/jiff/).
- [docs.rs: `tokio::sync::Barrier`](https://docs.rs/tokio/latest/tokio/sync/struct.Barrier.html).

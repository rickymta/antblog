---
slug: rust-schema-so-cai-accounts-journal-postings-partition
title: "Schema sổ cái: accounts, journal_entries, postings, balance là projection"
summary: "Bài 25: dựng bốn bảng sổ cái trong PostgreSQL, để DB tự từ chối bút toán lệch bằng constraint trigger lúc commit, giữ account_balances làm projection, đọc EXPLAIN trên 120.000 posting và thử partition theo tháng."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, postgresql, schema, partition]
language: vi
---

Đây là bài 25 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) ghi một bút toán trong một transaction. Bài này hỏi transaction đó ghi vào bảng nào, và DB có tự giữ được bất biến «tổng bằng 0» của [bài 13](/articles/rust-double-entry-journal-entry-posting) hay chỉ trông vào code Rust phía trên.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>JournalEntry</code> của <code>ledger-core</code> đã từ chối bút toán lệch trước khi ghi. DB còn cần kiểm lại không? Nếu cần, một <code>CHECK</code> trên <code>postings</code> có đủ không?</p>
<details><summary>Xem đáp án</summary><p>Cần, vì script SQL chạy tay, service khác hay adapter có bug đều ghi thẳng được. <code>CHECK</code> không đủ: nó chỉ nhìn một dòng, còn tổng bằng 0 là tính chất của nhiều dòng cùng <code>entry_id</code>. Kiểm sau từng <code>INSERT</code> cũng sai vì giữa chừng bút toán chưa cân: phải kiểm lúc commit.</p></details>
</div>

## Bốn bảng, mỗi bảng giữ một loại sự thật

Schema chuẩn của loạt có bốn bảng, và bài 23 đến 27 dùng chung đúng các tên này:

| Bảng | Giữ gì | Có sửa được không |
| --- | --- | --- |
| `accounts` | danh sách tài khoản và loại (`asset`, `liability`, `revenue`, `expense`) | thêm, hiếm khi sửa |
| `journal_entries` | một dòng mỗi bút toán: `idempotency_key` duy nhất, `reversal_of`, `posted_at` | không, chỉ thêm |
| `postings` | các dòng của bút toán: tài khoản và số tiền có dấu | không, chỉ thêm |
| `account_balances` | số dư từng tài khoản, tính sẵn | có, nhưng chỉ qua đường ghi bút toán |

Ba bảng đầu là sự thật. Bảng thứ tư là **projection**: kết quả cộng các posting, giữ lại để đọc nhanh và luôn tính lại được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Bốn bảng của schema sổ cái. postings tham chiếu accounts qua account_id và journal_entries qua entry_id. account_balances tham chiếu accounts. Một constraint trigger kiểm tổng amount_dong của mỗi entry_id bằng 0 lúc commit."><defs><marker id="rust-schema-so-cai-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="230" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="56" class="t-b">accounts</text><text x="36" y="78" class="t-small">id text PK</text><text x="36" y="96" class="t-small">kind CHECK (4 giá trị)</text><text x="36" y="114" class="t-small">created_at</text><rect x="325" y="30" width="230" height="110" rx="10" class="f-phys-soft s-phys"/><text x="341" y="56" class="t-b">postings</text><text x="341" y="78" class="t-small">PK (entry_id, line_no)</text><text x="341" y="96" class="t-small">account_id → accounts</text><text x="341" y="114" class="t-small">amount_dong CHECK (&lt;&gt; 0)</text><rect x="630" y="30" width="230" height="110" rx="10" class="f-phys-soft s-phys"/><text x="646" y="56" class="t-b">journal_entries</text><text x="646" y="78" class="t-small">id identity PK</text><text x="646" y="96" class="t-small">idempotency_key UNIQUE</text><text x="646" y="114" class="t-small">reversal_of → id · posted_at</text><rect x="20" y="195" width="230" height="85" rx="10" class="f-pk-soft s-pk"/><text x="36" y="221" class="t-b">account_balances</text><text x="36" y="243" class="t-small">account_id PK → accounts</text><text x="36" y="261" class="t-small">balance_dong · version</text><rect x="325" y="195" width="535" height="85" rx="10" class="f-warn-soft s-warn dash"/><text x="341" y="221" class="t-b">Constraint trigger, chạy lúc COMMIT</text><text x="341" y="243" class="t-small">với mỗi entry_id: SUM(amount_dong) = 0 và ít nhất 2 posting</text><text x="341" y="261" class="t-small">sai thì RAISE 23514 và cả transaction bị rollback</text><path d="M325 85 L250 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-0-a)"/><text x="254" y="76" class="t-small">account_id</text><path d="M555 85 L630 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-0-a)"/><text x="574" y="76" class="t-small">entry_id</text><path d="M135 195 L135 140" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-schema-so-cai-0-a)"/><path d="M440 195 L440 140" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-schema-so-cai-0-a)"/></svg>
</div>
<figcaption>Hình 1. Ba bảng sự thật (xanh lục), một projection (xanh dương) và một ràng buộc nhiều dòng do trigger giữ (vàng).</figcaption>
</figure>

Phần cốt lõi của `sql/01_schema.sql` (rút gọn: bỏ `accounts` và `journal_entries`):

```sql
-- sql/01_schema.sql (rút gọn)
CREATE TABLE postings (
    entry_id    bigint NOT NULL REFERENCES journal_entries (id),
    line_no     int NOT NULL,
    account_id  text NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);
```

`postings` không có cột thời gian: `posted_at` nằm ở `journal_entries`, một thời điểm cho cả bút toán. `amount_dong` có dấu theo dòng tiền (dương là vào, âm là ra), không phải nợ/có. Mọi DDL chạy trong schema `lab_<uuid>` do pool đặt `search_path`, xoá cuối ca.

`CHECK` và khóa ngoại được kiểm ngay tại câu `INSERT`, từng dòng một. Ba vi phạm dưới đây cho SQLSTATE và thông điệp thật của PostgreSQL 18.6:

```text
accounts.kind = 'equity'
  SQLSTATE 23514: new row for relation "accounts" violates check constraint "accounts_kind_check"
postings.amount_dong = 0
  SQLSTATE 23514: new row for relation "postings" violates check constraint "postings_amount_dong_check"
postings.account_id = 'driver:DR-999' (chưa có tài khoản)
  SQLSTATE 23503: insert or update on table "postings" violates foreign key constraint "postings_account_id_fkey"
```

Nhưng `CHECK` chỉ thấy dòng đang ghi: posting -100 và +90 đều qua, phải nhìn cả hai mới thấy lệch -10.

## Làm sao để DB tự từ chối bút toán lệch?

Dùng **constraint trigger** khai báo `DEFERRABLE INITIALLY DEFERRED`: PostgreSQL xếp sự kiện vào hàng chờ và chạy lúc commit (sớm hơn chỉ khi transaction tự gọi `SET CONSTRAINTS ... IMMEDIATE`). Giữa transaction ta được ghi từng posting; chỉ trạng thái cuối phải hợp lệ. Hàm kiểm trong `sql/02_balance_check.sql` (rút gọn: bỏ nhánh chọn `entry_id` và vòng lặp):

```sql
-- sql/02_balance_check.sql (rút gọn)
SELECT count(*), COALESCE(sum(amount_dong), 0)
  INTO v_count, v_sum
  FROM postings
 WHERE entry_id = v_entry;

IF v_count < 2 THEN
    RAISE EXCEPTION 'bút toán % có % posting, cần ít nhất 2', v_entry, v_count
        USING ERRCODE = 'check_violation';
END IF;
IF v_sum <> 0 THEN
    RAISE EXCEPTION 'bút toán % lệch: tổng posting = %', v_entry, v_sum
        USING ERRCODE = 'check_violation';
END IF;

CREATE CONSTRAINT TRIGGER postings_balanced
    AFTER INSERT OR UPDATE OR DELETE ON postings
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_entry_balanced();
```

`sum(bigint)` trả `numeric` nên không tràn. Nhánh đếm không thừa: bút toán 0 dòng có tổng đúng bằng 0, chỉ `v_count` mới bắt được. Bút toán rỗng không kích hoạt trigger trên `postings`, nên cần trigger thứ hai, `entries_have_postings`, trên `journal_entries`. `UPDATE` chuyển posting sang bút toán khác thì hàm kiểm cả bút toán cũ lẫn mới. Lab mô phỏng code lỗi: ghi posting -100 và +90 rồi cập nhật số dư như thường. Từng câu chạy được, lỗi chỉ đến ở `COMMIT`:

```text
posting -100 và +90, rồi COMMIT
  SQLSTATE 23514: bút toán 4 lệch: tổng posting = -10
bút toán không có posting
  SQLSTATE 23514: bút toán 5 có 0 posting, cần ít nhất 2
bút toán một dòng
  SQLSTATE 23514: bút toán 6 có 1 posting, cần ít nhất 2
  sau ba lần thử: 1 bút toán, 3 posting
```

Số `4`, `5`, `6` là `id` của các bút toán bị huỷ: identity không hoàn lại giá trị khi rollback, nên mã `JE-` có thể có lỗ hổng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. Dòng thời gian một transaction ghi bút toán lệch. Bước 1 đến 5 là BEGIN, INSERT bút toán, hai INSERT posting và upsert số dư, tất cả chạy được. Bước 6 là COMMIT: constraint trigger chạy, thấy tổng bằng -10, ném lỗi 23514 và transaction bị rollback, mất cả posting, bút toán và số dư."><defs><marker id="rust-schema-so-cai-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="20" width="120" height="64" rx="10" class="f-phys-soft s-phys"/><text x="22" y="46" class="t-b">1 · BEGIN</text><text x="22" y="66" class="t-small">mở transaction</text><rect x="158" y="20" width="120" height="64" rx="10" class="f-phys-soft s-phys"/><text x="170" y="46" class="t-b">2 · INSERT</text><text x="170" y="66" class="t-small">bút toán rỗng</text><rect x="306" y="20" width="120" height="64" rx="10" class="f-phys-soft s-phys"/><text x="318" y="46" class="t-b">3 · INSERT</text><text x="318" y="66" class="t-small">posting -100</text><rect x="454" y="20" width="120" height="64" rx="10" class="f-phys-soft s-phys"/><text x="466" y="46" class="t-b">4 · INSERT</text><text x="466" y="66" class="t-small">posting +90</text><rect x="602" y="20" width="120" height="64" rx="10" class="f-phys-soft s-phys"/><text x="614" y="46" class="t-b">5 · UPSERT</text><text x="614" y="66" class="t-small">2 dòng số dư</text><rect x="750" y="20" width="120" height="64" rx="10" class="f-warn-soft s-warn"/><text x="762" y="46" class="t-b">6 · COMMIT</text><text x="762" y="66" class="t-small">chạy trigger</text><path d="M130 52 L158 52" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-1-a)"/><path d="M278 52 L306 52" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-1-a)"/><path d="M426 52 L454 52" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-1-a)"/><path d="M574 52 L602 52" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-1-a)"/><path d="M722 52 L750 52" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-schema-so-cai-1-a)"/><rect x="10" y="130" width="540" height="100" rx="10" class="f-none s-line dash"/><text x="26" y="156" class="t-b">Bước 1 đến 5: DB chưa kiểm tổng</text><text x="26" y="180" class="t-small">Trigger đã xếp một sự kiện chờ cho mỗi dòng vừa ghi (DEFERRED).</text><text x="26" y="198" class="t-small">Trạng thái dở dang là hợp lệ: bút toán được dựng từng dòng.</text><text x="26" y="216" class="t-small">Khóa ngoại và CHECK từng dòng thì đã chạy ngay ở mỗi INSERT.</text><rect x="590" y="130" width="280" height="100" rx="10" class="f-warn-soft s-warn"/><text x="606" y="156" class="t-b">Bước 6: SUM = -10 ≠ 0</text><text x="606" y="180" class="t-small">RAISE 23514, COMMIT thất bại</text><text x="606" y="198" class="t-small">rollback: mất cả posting, bút toán</text><text x="606" y="216" class="t-small">và hai lần cập nhật số dư</text><path d="M810 84 L810 130" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-schema-so-cai-1-a)"/></svg>
</div>
<figcaption>Hình 2. Kiểm tổng chạy ở bước cuối; lỗi xoá luôn phần số dư vì cùng transaction.</figcaption>
</figure>

Trigger không miễn phí: mỗi dòng ghi xếp một sự kiện, lúc commit mỗi sự kiện chạy một truy vấn cộng theo `entry_id`. Lab nạp 120.000 posting trong một transaction: ba lần chạy cho 3,28 đến 3,48 giây khi bật hai trigger, 1,35 đến 1,48 giây khi tắt bằng `ALTER TABLE ... DISABLE TRIGGER` (máy còn chạy các lab khác). Nạp lịch sử hàng triệu dòng thì phải tính. Chủ bảng tắt được trigger như lab vừa làm, nên role của ứng dụng không nên là chủ bảng. Ở phía C#, trigger này nằm trong migration dưới dạng `migrationBuilder.Sql(...)` (xem [bài EF Core và PostgreSQL](/articles/dotnet10-efcore-postgresql-mapping-migration)).

## account_balances: projection cập nhật cùng transaction

Đọc rồi ghi lại số dư từ ứng dụng sẽ mất cập nhật khi hai request đồng thời (bài 13 đã mô phỏng). Lab cộng thẳng trong DB:

```rust
// src/ledger.rs (rút gọn)
pub const UPSERT_BALANCE: &str = "\
INSERT INTO account_balances (account_id, balance_dong, version)
VALUES ($1, $2, 1)
ON CONFLICT (account_id) DO UPDATE
SET balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong,
    version      = account_balances.version + 1,
    updated_at   = now()";

// Gộp delta theo tài khoản; BTreeMap duyệt theo tên nên mọi bút toán khoá số dư theo cùng thứ tự.
for (account, delta) in deltas {
    sqlx::query(UPSERT_BALANCE).bind(account).bind(delta).execute(&mut **tx).await?;
}
```

`EXCLUDED` là dòng định chèn; phép cộng chạy trong DB, dưới khóa dòng, nên ở READ COMMITTED không mất cập nhật. `post_entry_in` nhận `&mut Transaction` nên bút toán, posting và số dư cùng commit hoặc cùng mất.

Deadlock có xảy ra không? Có, nếu sai thứ tự: mỗi upsert giữ khóa dòng số dư tới hết transaction. Test `opposite_lock_order_deadlocks` cho hai transaction khóa hai tài khoản ngược chiều; PostgreSQL huỷ đúng một bên với SQLSTATE `40P01`. Lab tránh bằng cách gộp delta rồi duyệt theo tên tài khoản. Transaction ghi nhiều bút toán, hay job khóa theo thứ tự khác, vẫn có thể kẹt, nên đường ghi production phải thử lại khi gặp `40P01`.

Chuyến `TN-0001` qua `ledger-core` (`capture_entry` với chính sách phí mặc định) ghi `JE-000001`:

```text
JE-000001 trip:TN-0001:capture
  customer:CU-001          -25500  (version 1)
  driver:DR-001             20400  (version 1)
  platform:revenue           5100  (version 1)
  tổng số dư: 0
```

Truy vấn đối chiếu so projection với `SUM(postings)` bằng `FULL JOIN`, nên bắt cả tài khoản có posting mà chưa có dòng số dư:

```sql
SELECT COALESCE(b.account_id, p.account_id) AS account_id,
       COALESCE(b.balance_dong, 0)           AS projected,
       COALESCE(p.total, 0)                  AS actual
FROM account_balances b
FULL JOIN (SELECT account_id, SUM(amount_dong)::bigint AS total
           FROM postings GROUP BY account_id) p USING (account_id)
WHERE COALESCE(b.balance_dong, 0) <> COALESCE(p.total, 0)
ORDER BY 1
```

Lab sửa tay số dư tài xế bớt 500, đối chiếu, dựng lại từ posting, rồi chạy 40 bút toán đồng thời:

```text
  LỆCH driver:DR-001: projection 19900, SUM(postings) 20400
  dựng lại 3 dòng số dư, còn lệch: 0
  customer:CU-001         -145500  (version 42)
  driver:DR-001            116400  (version 42)
  platform:revenue          29100  (version 42)
  tổng số dư: 0; tài khoản lệch: 0
```

`version` 42 là 1 (`TN-0001`) cộng 1 (dựng lại) cộng 40: không mất lần cập nhật nào. Kết quả đối chiếu khác rỗng là sự cố cần điều tra trước: dựng lại chỉ che nguyên nhân, và phải chặn ghi đồng thời vì `SUM` đọc theo snapshot, có thể đè mất bút toán vừa commit.

## Index nào cho truy vấn nào?

Khóa chính `(entry_id, line_no)` không tìm được theo tài khoản, mà đó là truy vấn của job đối chiếu và mọi sao kê. Lab sinh 60.000 bút toán hai dòng bằng `generate_series` (1.000 tài khoản, mỗi tài khoản khoảng 120 posting), chạy `EXPLAIN (ANALYZE, BUFFERS)` hai lần và lấy lần sau để cache ấm. Chưa có index:

```text
Aggregate  (cost=2383.60..2383.61 rows=1 width=40) (actual time=5.239..5.239 rows=1.00 loops=1)
  Buffers: shared hit=883
  ->  Seq Scan on postings  (cost=0.00..2383.00 rows=120 width=8) (actual time=0.016..5.220 rows=120.00 loops=1)
        Filter: (account_id = 'acct:0042'::text)
        Rows Removed by Filter: 119880
        Buffers: shared hit=883
Execution Time: 5.264 ms
```

Sau `CREATE INDEX postings_account_entry_idx ON postings (account_id, entry_id)` (hai kế hoạch trích bỏ dòng `Planning Time`):

```text
Aggregate  (cost=338.18..338.19 rows=1 width=40) (actual time=0.240..0.241 rows=1.00 loops=1)
  Buffers: shared hit=123
  ->  Bitmap Heap Scan on postings  (cost=5.35..337.58 rows=120 width=8) (actual time=0.040..0.220 rows=120.00 loops=1)
        Recheck Cond: (account_id = 'acct:0042'::text)
        Heap Blocks: exact=120
        Buffers: shared hit=123
        ->  Bitmap Index Scan on postings_account_entry_idx  (cost=0.00..5.32 rows=120 width=0) (actual time=0.025..0.025 rows=120.00 loops=1)
              Index Cond: (account_id = 'acct:0042'::text)
              Index Searches: 1
              Buffers: shared hit=3
Execution Time: 0.264 ms
```

Không index, PostgreSQL đọc cả bảng (`Seq Scan`), bỏ 119.880 dòng (`Rows Removed by Filter`), tốn 883 page. Có index, dò index mất 3 page, rồi nhảy tới 120 page của bảng (`Heap Blocks: exact=120`): tổng 123. Posting của một tài khoản nằm rải khắp bảng nên vẫn chạm 120 page: rẻ hơn 7 lần, không phải 1.000 lần.

Số đo trên máy viết bài (MacBook Pro M1 Max, PostgreSQL 18.6 trong Docker, cache ấm), không phải SLA. Số page ổn định, thời gian thì không: ba lần cho 5,1 đến 5,3 ms (không index) và 0,10 đến 0,26 ms (có index). Lab đặt `max_parallel_workers_per_gather = 0` để hai kế hoạch cùng chạy một tiến trình: 883 page còn dưới ngưỡng `min_parallel_table_scan_size` (8 MB) nên planner chưa chọn parallel, nhưng thử nhân đôi bảng là `Gather` xuất hiện và thời gian không còn so thẳng được.

Vì sao `(account_id, entry_id)` mà không phải `(account_id, posted_at)`? Vì `postings` không có `posted_at`, và một index không trải qua hai bảng. `entry_id` tăng theo thứ tự cấp số nên index này cho posting của một tài khoản gần đúng theo thời gian. Chỉ gần đúng: identity cấp số lúc `INSERT`, `now()` lấy giờ bắt đầu transaction, nên giao dịch đồng thời có thể lệch thứ tự. Sao kê theo khoảng thời gian thì join `journal_entries`. Bản partition buộc phải có `posted_at` trong bảng, và index ở đó là `(account_id, posted_at)`.

## Partition theo tháng đổi gì trong schema?

Lab dựng thêm `postings_p`, cùng nội dung `postings` nhưng `PARTITION BY RANGE (posted_at)` với sáu partition tháng 1 đến 6 năm 2026.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Partition pruning. Truy vấn lọc posted_at trong tháng 3 chỉ quét partition tháng 3 với 193 page; năm partition còn lại bị loại. Không lọc theo posted_at thì cả sáu partition bị quét, 1.124 page."><defs><marker id="rust-schema-so-cai-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="16" width="840" height="54" rx="10" class="f-pk-soft s-pk"/><text x="36" y="40" class="t-b">Truy vấn</text><text x="36" y="58" class="t-small">WHERE posted_at &gt;= '2026-03-01' AND posted_at &lt; '2026-04-01'</text><rect x="20" y="115" width="130" height="90" rx="10" class="f-none s-line dash"/><text x="36" y="143" class="t-b">2026_01</text><text x="36" y="163" class="t-small">20.582 dòng</text><text x="36" y="183" class="t-small">bị loại</text><rect x="162" y="115" width="130" height="90" rx="10" class="f-none s-line dash"/><text x="178" y="143" class="t-b">2026_02</text><text x="178" y="163" class="t-small">18.592 dòng</text><text x="178" y="183" class="t-small">bị loại</text><rect x="304" y="115" width="130" height="90" rx="10" class="f-pk-soft s-pk"/><text x="320" y="143" class="t-b">2026_03</text><text x="320" y="163" class="t-small">20.584 dòng</text><text x="320" y="183" class="t-small">quét: 193 page</text><rect x="446" y="115" width="130" height="90" rx="10" class="f-none s-line dash"/><text x="462" y="143" class="t-b">2026_04</text><text x="462" y="163" class="t-small">19.860 dòng</text><text x="462" y="183" class="t-small">bị loại</text><rect x="588" y="115" width="130" height="90" rx="10" class="f-none s-line dash"/><text x="604" y="143" class="t-b">2026_05</text><text x="604" y="163" class="t-small">20.522 dòng</text><text x="604" y="183" class="t-small">bị loại</text><rect x="730" y="115" width="130" height="90" rx="10" class="f-none s-line dash"/><text x="746" y="143" class="t-b">2026_06</text><text x="746" y="163" class="t-small">19.860 dòng</text><text x="746" y="183" class="t-small">bị loại</text><path d="M369 70 L369 115" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-schema-so-cai-2-a)"/><text x="385" y="98" class="t-small">Các partition của postings_p (tên đầy đủ: postings_p_2026_NN)</text><rect x="20" y="228" width="840" height="56" rx="10" class="f-warn-soft s-warn"/><text x="36" y="252" class="t-b">Không lọc theo posted_at: quét cả 6 partition, 1.124 page</text><text x="36" y="270" class="t-small">Lọc một tháng: 1 partition, 193 page. Pruning chỉ có khi điều kiện chứa partition key.</text></svg>
</div>
<figcaption>Hình 3. Partition pruning thấy trong <code>EXPLAIN</code>: kế hoạch một tháng chỉ nhắc tới <code>postings_p_2026_03</code>.</figcaption>
</figure>

Pruning là phần dễ chịu; phần phải trả nằm ở ràng buộc. Thứ nhất, giữ khóa chính cũ `(entry_id, line_no)` bị từ chối:

```text
PRIMARY KEY (entry_id, line_no) trên bảng PARTITION BY RANGE (posted_at):
  Some(("0A000", "unique constraint on partitioned table must include all partitioning columns"))
```

Khóa chính và unique của bảng partition phải chứa partition key, vì mỗi partition chỉ tự kiểm unique trong phạm vi nó; nên `postings_p` có khóa chính `(entry_id, line_no, posted_at)`. Thứ hai là lỗ hổng: DB không còn đảm bảo `(entry_id, line_no)` duy nhất toàn bảng.

```text
trùng (entry_id, line_no, posted_at) bị từ chối: true
trùng (entry_id, line_no) nhưng khác tháng được nhận: true
```

Thứ ba, khóa ngoại trỏ vào `postings_p` phải gồm đủ cột của một unique constraint, tức cả `posted_at`:

```text
FK (entry_id, line_no) -> postings_p:
  Some(("42830", "there is no unique constraint matching given keys for referenced table \"postings_p\""))
FK (entry_id, line_no, posted_at) -> postings_p: được chấp nhận
```

Chiều ngược lại êm: `postings_p` trỏ khóa ngoại tới `journal_entries(id)` và `accounts(id)` bình thường. Thứ tư, nặng nhất với sổ cái: partition luôn `journal_entries` theo `posted_at` thì sao?

```text
UNIQUE (idempotency_key) trên bảng PARTITION BY RANGE (posted_at):
  Some(("0A000", "unique constraint on partitioned table must include all partitioning columns"))
UNIQUE (idempotency_key, posted_at), cùng key khác posted_at: cả hai dòng được nhận
```

`idempotency_key` không thể unique toàn cục nữa. Thêm `posted_at` vào unique thì tạo được bảng, nhưng lần ghi lại `trip:TN-0001:capture` có `posted_at` mới là thành bút toán thứ hai. Vì vậy lab chỉ partition posting; nếu buộc phải partition `journal_entries`, khóa idempotency phải nằm ở một bảng riêng không partition. Thứ năm, dòng có `posted_at` ngoài mọi partition không có chỗ ở:

```text
posted_at = 2026-11-06 (ngoài mọi partition):
  Some(("23514", "no partition of relation \"postings_p\" found for row"))
```

Partition phải được tạo trước kỳ ghi, bằng job; hoặc có partition `DEFAULT`, nhưng nó hứng dữ liệu lạc và làm việc thêm partition mới chậm hơn.

## Khi nào CHƯA cần partition?

Với 120.000 posting, bảng và index chiếm khoảng 17 MB; truy vấn một tài khoản đọc 123 page, dưới 1 ms, với index thường. Bản partition cho một tài khoản trong một tháng đọc 24 page nhưng cũng chỉ trả 22 dòng thay vì 120: không phải lợi thế bù được các ràng buộc vừa liệt kê. Partition đáng giá ở chỗ khác: bỏ dữ liệu cũ bằng `DROP` hay `DETACH` thay vì `DELETE` hàng triệu dòng, `VACUUM` từng phần, chuyển tháng ít dùng sang lưu trữ rẻ hơn. Tài liệu PostgreSQL nêu quy tắc kinh nghiệm: bảng phải lớn hơn bộ nhớ máy chủ thì partition mới đáng.

Cách làm gợi ý: bắt đầu bằng bảng thường, đúng khóa chính, đúng index; đo trên dữ liệu cỡ thật; chuyển sang partition khi một con số cụ thể (thời gian `VACUUM`, kích thước bảng) chạm ngưỡng đã định.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL lab theo `examples/toi-noi-rust/infra/README.md`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/25-schema-so-cai/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/25-schema-so-cai/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/25-schema-so-cai/Cargo.toml
```

Lệnh đầu in mọi kết quả trích ở trên. Lệnh thứ hai in `VERIFY OK` (mã 0) hoặc `VERIFY FAIL: …` (mã 1); nó kiểm hình dạng kế hoạch và số page, không kiểm con số thời gian. Lệnh thứ ba chạy 9 test, 8 trong số đó dùng PostgreSQL thật. Thiếu `TOI_NOI_PG_URL` thì lab thất bại kèm cách bật compose.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Dữ liệu bench đều đặn (mỗi bút toán hai dòng, không ghi đồng thời lúc đo), nên số đo chỉ cho thấy hình dạng kế hoạch, không dự báo tải thật. Lab không có append-only (bài 24), không tách role ứng dụng khỏi chủ bảng, không có job đối chiếu định kỳ, không thử lại khi `40P01`, và chỉ tạo sáu partition cố định.</p>
</div>

**Bài tập:** một dev thêm cột `balance_dong` vào `postings` ("số dư sau dòng này") để khỏi cộng dồn. Hai request đồng thời cùng ghi posting cho `driver:DR-001`. Vì sao cột đó khó đúng, và với schema của bài này lấy «số dư tài khoản tại bút toán `JE-000007`» bằng cách nào mà không thêm cột?

<details><summary>Xem lời giải</summary><p>Cột đó buộc mỗi request đọc số dư dòng trước rồi cộng. Hai request đọc cùng giá trị cũ thì một số dư sai mà PostgreSQL không báo lỗi; muốn đúng phải khoá tuần tự mọi posting của tài khoản. Với schema này đó là một phép cộng: <code>SELECT COALESCE(SUM(amount_dong), 0) FROM postings WHERE account_id = $1 AND entry_id &lt;= 7</code>. Index <code>(account_id, entry_id)</code> giúp đọc đúng khoảng cần.</p></details>

Bài trước: [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key). Bài tiếp: [bài 26](/articles/rust-repository-unit-of-work-trait-transaction).

## Tài liệu gốc

- [PostgreSQL: CREATE TRIGGER, mục Constraint Triggers](https://www.postgresql.org/docs/current/sql-createtrigger.html).
- [PostgreSQL: Table Partitioning](https://www.postgresql.org/docs/current/ddl-partitioning.html).
- [PostgreSQL: Using EXPLAIN](https://www.postgresql.org/docs/current/using-explain.html).
- [PostgreSQL: INSERT, mục ON CONFLICT Clause](https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT).

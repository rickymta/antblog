---
slug: rust-feature-flag-config-migration-expand-contract
title: "Feature flag, cấu hình kiểu an toàn và migration không dừng"
summary: "Bài 56: thêm cột và bật quy tắc phí mới khi sổ đang nhận bút toán: expand/contract với hai phiên bản code, backfill có checkpoint, khóa thật qua pg_locks, cờ theo tripId, policy_version để replay, rollback."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, postgresql, migration, feature-flag]
language: vi
---

Finance muốn mỗi posting mang mã tiền tệ. Product muốn thử quy tắc phí mới: 15%, không dưới 3.000 đồng mỗi chuyến, để chuyến ngắn không còn lỗ. Cả hai phải đổi khi service đang nhận chuyến, chạy nhiều instance, không có giờ bảo trì, và bút toán đã ghi không được đổi một đồng nào.

Đây là bài 56 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab chạy trên PostgreSQL thật, với schema chuẩn của [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition), trigger append-only của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) và `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace); luồng ghi chuyến là phần nhỏ của [bài 54](/articles/rust-use-case-thanh-toan-chuyen-end-to-end).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cột <code>currency</code> đã thêm, backfill xong, mọi posting đã là <code>'VND'</code>. Chạy <code>ALTER TABLE postings ALTER COLUMN currency SET NOT NULL</code> lúc sổ đang nhận bút toán có an toàn không?</p>
<details><summary>Xem đáp án</summary><p>Dữ liệu thì đúng, khóa thì không. Không có ràng buộc nào chứng minh sẵn hết <code>NULL</code>, nên PostgreSQL quét cả bảng trong lúc giữ <code>ACCESS EXCLUSIVE</code>, và mọi bút toán mới đứng chờ. Cách an toàn là thêm <code>CHECK ... NOT VALID</code>, <code>VALIDATE</code> rồi mới <code>SET NOT NULL</code>.</p></details>
</div>

## Vì sao không đổi cột trong một lần triển khai?

Triển khai cuốn chiếu nghĩa là có lúc hai phiên bản code cùng đọc và ghi một database. Expand/contract tách nó thành ba pha, mỗi bước tương thích với cả hai phiên bản: **expand** (chỉ thêm: cột nullable, trigger), triển khai code mới rồi **backfill** theo lô, và **contract** (siết ràng buộc, bỏ cái cũ khi không còn ai dùng).

Lab có ba bản code: `AppV1` (không biết cột mới), `AppV2` ghi đúp (cả cột cũ lẫn mới) và `AppV2` chỉ ghi cột mới. Mỗi ô của Hình 1 là một lần ghi thật của bản code ở hàng đó, trên schema ở giai đoạn đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Ma trận ba phiên bản code qua bốn giai đoạn của migration. Code cũ ghi được ở ba giai đoạn đầu và lỗi 23502 sau contract B. Code mới ghi đúp lỗi 42703 trước expand và sau contract B. Code mới chỉ ghi cột mới lỗi 42703 trước expand, lỗi 23502 sau expand và ghi được từ contract A."><rect x="190" y="14" width="160" height="34" rx="8" class="f-pk-soft s-pk"/><text x="202" y="36" class="t-b">trước expand</text><rect x="360" y="14" width="160" height="34" rx="8" class="f-pk-soft s-pk"/><text x="372" y="36" class="t-b">sau expand</text><rect x="530" y="14" width="160" height="34" rx="8" class="f-pk-soft s-pk"/><text x="542" y="36" class="t-b">sau contract A</text><rect x="700" y="14" width="160" height="34" rx="8" class="f-pk-soft s-pk"/><text x="712" y="36" class="t-b">sau contract B</text><text x="20" y="94" class="t-b">code cũ</text><text x="20" y="162" class="t-b">mới, ghi đúp</text><text x="20" y="230" class="t-b">mới, chỉ cột mới</text><rect x="190" y="60" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="202" y="84" class="t-b">ghi được</text><text x="202" y="104" class="t-small">như trước</text><rect x="360" y="60" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="372" y="84" class="t-b">ghi được</text><text x="372" y="104" class="t-small">trigger điền hộ</text><rect x="530" y="60" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="542" y="84" class="t-b">ghi được</text><text x="542" y="104" class="t-small">trigger điền hộ</text><rect x="700" y="60" width="160" height="58" rx="8" class="f-warn-soft s-warn"/><text x="712" y="84" class="t-b">lỗi 23502</text><text x="712" y="104" class="t-small">currency bắt buộc</text><rect x="190" y="128" width="160" height="58" rx="8" class="f-warn-soft s-warn"/><text x="202" y="152" class="t-b">lỗi 42703</text><text x="202" y="172" class="t-small">thiếu cột mới</text><rect x="360" y="128" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="372" y="152" class="t-b">ghi được</text><text x="372" y="172" class="t-small">ghi cả hai cột</text><rect x="530" y="128" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="542" y="152" class="t-b">ghi được</text><text x="542" y="172" class="t-small">ghi cả hai cột</text><rect x="700" y="128" width="160" height="58" rx="8" class="f-warn-soft s-warn"/><text x="712" y="152" class="t-b">lỗi 42703</text><text x="712" y="172" class="t-small">fee_percent đã bị bỏ</text><rect x="190" y="196" width="160" height="58" rx="8" class="f-warn-soft s-warn"/><text x="202" y="220" class="t-b">lỗi 42703</text><text x="202" y="240" class="t-small">thiếu cột mới</text><rect x="360" y="196" width="160" height="58" rx="8" class="f-warn-soft s-warn"/><text x="372" y="220" class="t-b">lỗi 23502</text><text x="372" y="240" class="t-small">fee_percent bắt buộc</text><rect x="530" y="196" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="542" y="220" class="t-b">ghi được</text><text x="542" y="240" class="t-small">chỉ policy_version</text><rect x="700" y="196" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="712" y="220" class="t-b">ghi được</text><text x="712" y="240" class="t-small">chỉ policy_version</text><text x="20" y="282" class="t-small">Ô cảnh báo là lỗi thật của PostgreSQL, SQLSTATE ghi ngay trong ô.</text></svg>
</div>
<figcaption>Hình 1. Thứ tự an toàn đọc được từ ma trận: expand, code ghi đúp, backfill, contract A, code chỉ ghi cột mới, rồi mới contract B.</figcaption>
</figure>

Hàng cuối cho thấy vì sao thứ tự quan trọng: code «chỉ cột mới» lên trước contract A thì lỗi ngay. Đổi schema sự kiện ở [bài 55](/articles/rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version) cùng một ý: thêm trước, đổi người ghi, gỡ sau. Với loạt C#, [bài EF Core 10](/articles/dotnet10-efcore-postgresql-mapping-migration) coi migration là lịch sử có thể review; ở đây là thứ tự áp dụng nó.

## Expand và backfill: hai phiên bản cùng chạy, ghi đúp, không sửa số tiền

Expand thêm cột nullable, không giá trị mặc định. Code cũ không biết cột, nên trigger `BEFORE INSERT` ghi hộ `'VND'`; với `policy_version` thì ghi hộ `1`, vì mọi chuyến trước giờ dùng một quy tắc duy nhất. Phần của `postings` (rút gọn, trong code là các hằng `Step` của `src/migrate.rs`):

```sql
ALTER TABLE postings ADD COLUMN currency text;

CREATE FUNCTION postings_fill_currency() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN NEW.currency := COALESCE(NEW.currency, 'VND'); RETURN NEW; END; $$;
CREATE TRIGGER postings_fill_currency BEFORE INSERT ON postings
    FOR EACH ROW EXECUTE FUNCTION postings_fill_currency();
```

Backfill phải `UPDATE` posting, mà trigger append-only của bài 24 cấm mọi `UPDATE`. Expand nới đúng một kiểu: từ `NULL` sang `'VND'`, bốn cột gốc không đổi; ghi `'USD'`, sửa `amount_dong` hay `DELETE` vẫn nhận SQLSTATE `TN001` (có test).

```sql
IF TG_OP = 'UPDATE' AND OLD.currency IS NULL AND NEW.currency = 'VND'
   AND (OLD.entry_id, OLD.line_no, OLD.account_id, OLD.amount_dong)
       IS NOT DISTINCT FROM (NEW.entry_id, NEW.line_no, NEW.account_id, NEW.amount_dong)
THEN RETURN NEW; END IF;
RAISE EXCEPTION 'sổ cái append-only: % trên postings bị cấm', TG_OP USING ERRCODE = 'TN001';
```

Cái giá: từ expand tới contract A, câu «posting không bao giờ bị `UPDATE`» tạm không đúng. Số tiền vẫn được bảo vệ, nhưng phiên nào có quyền ghi cũng điền được `currency`. Mỗi `UPDATE` còn sinh bản dòng mới: thêm WAL, dead tuple, độ trễ replica. Lab nới bằng một nhánh trong trigger chứ không `DISABLE TRIGGER`, vì lệnh đó tắt kiểm tra cho mọi phiên; cửa sổ đóng lại ở contract A.

**Ghi đúp.** `fee_events` bản 1 chỉ có `fee_percent`. Quy tắc mới có sàn nên không biểu diễn được bằng một tỷ lệ; cột `policy_version` thay chỗ nó. Trong giai đoạn chuyển, code mới ghi cả hai (`WriteMode::Dual`, `src/app.rs`):

```rust
"INSERT INTO fee_events (trip_id, fare_dong, fee_percent, policy_version) \
 VALUES ($1, $2, $3, $4) ON CONFLICT (trip_id) DO NOTHING RETURNING policy_version"
```

Cái giá: `fee_percent` của chuyến theo quy tắc mới chỉ là tỷ lệ danh nghĩa (15), còn tỷ lệ thật của chuyến 12.000 đồng là 25%. Hai chuyến ghi giữa chừng:

```text
TN-0121 bởi code v1: currency ["VND"], fee_percent Some(20), policy_version Some(1)
TN-0122 bởi code v2: currency ["VND"], fee_percent Some(15), policy_version Some(2)
```

**Backfill theo lô.** Mỗi lô lấy `batch_size` khóa kế tiếp sau checkpoint, điền dòng còn `NULL` và dời checkpoint trong một transaction. `IS NULL` làm lô idempotent; `FOR UPDATE` trên dòng checkpoint để hai tiến trình cùng tên xếp hàng: test giữ dòng đó tới khi `pg_stat_activity` thấy cả hai đang chờ, rồi nhả, và không lô nào dùng chung checkpoint (`backfill_batch`, rút gọn):

```rust
// src/migrate.rs
let last: i64 = sqlx::query_scalar(
    "SELECT last_key FROM migration_checkpoints WHERE name = $1 FOR UPDATE").bind(spec.name)
    .fetch_one(&mut *tx).await?;
// upper = khóa lớn nhất trong batch_size khóa kế tiếp (SELECT max(k) FROM (SELECT DISTINCT ...))
// UPDATE postings SET currency = 'VND'
//  WHERE entry_id > $last AND entry_id <= $upper AND currency IS NULL
// UPDATE migration_checkpoints SET last_key = $upper ...; rồi tx.commit()
```

Lab chạy hai lô, ghi thêm chuyến bằng cả hai bản code giữa các lô, rồi «chết»; tiến trình mới chỉ đọc checkpoint và chạy tiếp:

```text
backfill postings.currency: 3 lô, chạy 2 lô rồi dừng; checkpoint đọc lại = 200
  lô 1: 250 dòng, checkpoint entry 100
  lô 2: 250 dòng, checkpoint entry 200
  lô 3: 100 dòng, checkpoint entry 248
còn NULL trước contract: 0 posting, 0 sự kiện
dấu vân tay posting cũ trước bb74c48290697ecfe6711f11f43883d6 / sau bb74c48290697ecfe6711f11f43883d6: giống nhau true
```

Mỗi dòng cũ được điền đúng một lần (250 + 250 + 100 = 600). Dấu vân tay là md5 của bốn cột gốc của mọi posting cũ: số tiền và tài khoản không đổi. Trigger cân bút toán của bài 25 khai báo cả `UPDATE`, nên mỗi dòng bị sửa kích hoạt lại kiểm «tổng bằng 0» lúc commit.

## Contract: NOT NULL có quét cả bảng không?

Lab đọc `pg_locks` trong transaction của từng câu DDL và đếm số lần quét bằng hiệu `pg_stat_get_xact_numscans` trước và sau câu lệnh:

```text
ADD COLUMN postings.currency (nullable)                    AccessExclusiveLock        quét 0
CREATE TRIGGER postings_fill_currency                      ShareRowExclusiveLock      quét 0
ADD CONSTRAINT postings_currency_nn CHECK (...) NOT VALID  AccessExclusiveLock        quét 0
VALIDATE CONSTRAINT postings_currency_nn                   ShareUpdateExclusiveLock   quét 1
SET NOT NULL postings.currency                             AccessExclusiveLock        quét 0
```

`ADD COLUMN` xin khóa mạnh nhất nhưng không quét và không viết lại bảng (`pg_relation_filenode` giữ nguyên). `VALIDATE` quét một lần nhưng chỉ lấy `ShareUpdateExclusiveLock`: giữ nó mở thì bút toán mới vẫn ghi được. `SET NOT NULL` sau khi có `CHECK` hợp lệ không quét (từ PostgreSQL 12), rồi `CHECK` tạm được bỏ. Còn `NULL` mà `VALIDATE` thì PostgreSQL trả `23514` và không đổi gì: hàng rào chặn contract sớm.

**Có khi không cần đường dài.** Giá trị là hằng thì từ PostgreSQL 11, `ADD COLUMN ... NOT NULL DEFAULT` chỉ ghi mặc định vào catalog: không viết lại bảng, không `UPDATE`, không phải nới trigger. Với riêng `currency`, đó là cách nên chọn ngoài đời; bài đi đường dài vì giá trị phải tính từ dữ liệu thì không có lối tắt này. `DEFAULT` volatile vẫn viết lại cả bảng. PostgreSQL 18 còn cho `ADD CONSTRAINT ... NOT NULL ... NOT VALID` rồi `VALIDATE`, không cần `CHECK` tạm:

```text
ADD COLUMN source NOT NULL DEFAULT hằng                    AccessExclusiveLock        quét 0
file dữ liệu giữ nguyên: true; dòng cũ đọc ra mặc định, không UPDATE: 150/150
DEFAULT clock_timestamp() viết lại bảng: true
ADD CONSTRAINT note_nn NOT NULL note NOT VALID             AccessExclusiveLock        quét 0
dòng mới NULL lúc chưa VALIDATE: SQLSTATE 23502
VALIDATE CONSTRAINT note_nn                                ShareUpdateExclusiveLock   quét 1
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Hai đường đổi cột sang NOT NULL. Đường thẳng giữ khóa ACCESS EXCLUSIVE trong lúc quét cả bảng và chặn ghi. Đường an toàn gồm ba bước: thêm CHECK NOT VALID không quét, VALIDATE quét một lần nhưng cho ghi, rồi SET NOT NULL không quét."><rect x="20" y="20" width="410" height="230" rx="10" class="f-warn-soft s-warn"/><text x="36" y="48" class="t-b">SET NOT NULL thẳng</text><text x="36" y="86" class="t-small">không ràng buộc nào chứng minh hết NULL</text><text x="36" y="116" class="t-small">AccessExclusiveLock, quét 1 lần cả bảng</text><text x="36" y="146" class="t-small">bút toán mới đứng chờ suốt lúc quét</text><text x="36" y="176" class="t-small">bảng càng lớn, hàng đợi càng dài</text><text x="36" y="226" class="t-small">lab: ghi bị chặn, thấy trong pg_stat_activity</text><rect x="450" y="20" width="410" height="230" rx="10" class="f-phys-soft s-phys"/><text x="466" y="48" class="t-b">Đường an toàn, ba bước</text><rect x="466" y="62" width="378" height="52" rx="8" class="f-surface s-phys"/><text x="478" y="83" class="t-b">1. ADD CONSTRAINT CHECK ... NOT VALID</text><text x="478" y="102" class="t-small">AccessExclusiveLock, quét 0: chỉ áp cho dòng mới</text><rect x="466" y="122" width="378" height="52" rx="8" class="f-surface s-pk"/><text x="478" y="143" class="t-b">2. VALIDATE CONSTRAINT</text><text x="478" y="162" class="t-small">ShareUpdateExclusiveLock, quét 1: ghi vẫn chạy</text><rect x="466" y="182" width="378" height="52" rx="8" class="f-surface s-phys"/><text x="478" y="203" class="t-b">3. SET NOT NULL (đã có CHECK hợp lệ)</text><text x="478" y="222" class="t-small">AccessExclusiveLock, quét 0: chỉ đổi catalog</text></svg>
</div>
<figcaption>Hình 2. Cả hai đường đều xin khóa mạnh nhất; khác nhau ở chỗ lúc giữ khóa đó có quét bảng hay không.</figcaption>
</figure>

Đường thẳng trong lab, sau khi backfill xong:

```text
khóa AccessExclusiveLock, quét 1 lần; bút toán mới bị chặn: true; ghi được sau khi commit: true
```

«Bị chặn» đọc từ `pg_stat_activity` (`pg_blocking_pids` chứa phiên DDL). Mọi số đo trên PostgreSQL 18.6, khớp tài liệu `ALTER TABLE`.

Contract tách hai giai đoạn. **A** (còn quay lại được): `SET NOT NULL` hai cột mới, nới `fee_percent`, trả trigger append-only về chặt (sửa `currency` của posting cũ lại nhận `TN001`). **B** (không quay lại được): bỏ trigger ghi hộ và cột `fee_percent`.

## Một ALTER đang chờ khóa có chặn cả truy vấn đọc không?

Có. Lab dựng ba phiên: A đọc dài, giữ `AccessShareLock` trên `postings`; B chạy `ADD COLUMN`; C là `SELECT count(*)` ngắn đến sau B.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 3. Hàng đợi khóa. Phiên A đọc dài giữ AccessShareLock. Phiên B chạy ALTER TABLE xin AccessExclusiveLock nên phải chờ A. Phiên C chạy SELECT ngắn xin AccessShareLock, tương thích với A nhưng vẫn phải xếp sau B."><defs><marker id="rust-feature-flag-config-migration-expand-contract-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">A · báo cáo đọc dài</text><text x="36" y="90" class="t-small">giữ AccessShareLock</text><text x="36" y="114" class="t-small">chưa commit</text><rect x="315" y="30" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="331" y="62" class="t-b">B · ALTER TABLE</text><text x="331" y="90" class="t-small">xin AccessExclusiveLock</text><text x="331" y="114" class="t-small">chờ A</text><rect x="610" y="30" width="250" height="110" rx="10" class="f-warn-soft s-warn dash"/><text x="626" y="62" class="t-b">C · SELECT ngắn</text><text x="626" y="90" class="t-small">xin AccessShareLock</text><text x="626" y="114" class="t-small">tương thích với A, xếp sau B</text><path d="M313 85 L272 85" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-feature-flag-config-migration-expand-contract-2-a)"/><path d="M608 85 L567 85" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-feature-flag-config-migration-expand-contract-2-a)"/><text x="20" y="178" class="t-small">pg_blocking_pids(C) chứa B, pg_blocking_pids(B) chứa A: lab đọc cả hai từ pg_stat_activity.</text></svg>
</div>
<figcaption>Hình 3. Mũi tên là «đang chờ». Ngay cả truy vấn không xung đột với ai cũng bị giữ lại sau một yêu cầu khóa mạnh đang xếp hàng.</figcaption>
</figure>

```text
ALTER bị chặn bởi phiên đọc dài: true; truy vấn đọc ngắn bị chặn bởi chính ALTER: true
  đang chờ: ALTER  AccessExclusiveLock
  đang chờ: đọc    AccessShareLock
nhả phiên đọc dài: ALTER xong (true), truy vấn đọc thấy 50 dòng
```

Cách chữa là `lock_timeout` ngắn: không lấy được khóa thì bỏ cuộc, trả hàng đợi, thử lại. Mẫu của lab: 200 ms, tối đa 5 lần, chỉ thử lại khi lỗi là `55P03` (`src/migrate.rs`):

```rust
match attempt(n).await {
    Ok(value) => return Ok((value, n)),
    Err(err)
        if n < max_attempts && sqlstate(&err).as_deref() == Some(LOCK_NOT_AVAILABLE) =>
    {
        n += 1;
    }
    Err(err) => return Err(err),
}
```

Phiên đọc dài còn giữ bảng thì `expand` hết hạn ở lần 1, hoàn tác sạch, người ghi không bị chặn; phiên đọc xong thì lần 2 thành công:

```text
lần 1: SQLSTATE 55P03; cột chưa được thêm: true; người ghi không bị chặn: true
phiên đọc dài kết thúc: thành công ở lần 2, cột có mặt: true
```

Lab thử lại ngay; production nên nghỉ có jitter giữa các lần. `expand` của lab gộp hai bảng trong một transaction để hoàn tác sạch, nên khóa `postings` có thể bị giữ thêm tối đa một `lock_timeout` lúc chờ `fee_events`; bảng lớn nên tách mỗi bảng một transaction.

## Feature flag: lưu ở đâu, đọc lúc nào, replay có ra đúng số cũ không?

**Lưu ở đâu.** Biến môi trường hay file cần triển khai lại. Redis nhanh nhưng không là nguồn sự thật ([bài 28](/articles/rust-redis-cache-so-du-rate-limit-lock-ttl)). Lab đặt cờ trong bảng `feature_flags` của database sổ cái: một nguồn cho mọi instance, đổi không cần deploy.

**Đọc lúc nào.** Đúng một lần cho mỗi chuyến, tại chỗ quyết định phí, và kết quả vào sự kiện `fee_events` cùng `policy_version`. Sau đó bút toán và replay đều đọc sự kiện, không hỏi lại cờ. Sự kiện đầu tiên thắng (`ON CONFLICT DO NOTHING`), nên hai instance thấy cờ khác nhau vẫn ra cùng một phiên bản.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 4. TripCompleted đi qua bước đọc cờ một lần, ghi policyVersion vào sự kiện fee_events, rồi sinh bút toán chia cước theo phiên bản đó. Replay đọc fee_events chứ không đọc cờ; cờ đổi sau đó không tác động tới chuyến đã có sự kiện."><defs><marker id="rust-feature-flag-config-migration-expand-contract-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">TripCompleted</text><text x="36" y="86" class="t-small">tripId, fareDong</text><rect x="225" y="30" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="241" y="62" class="t-b">Đọc cờ một lần</text><text x="241" y="86" class="t-small">bucket(tripId) &lt; tỷ lệ</text><rect x="450" y="30" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="466" y="62" class="t-b">fee_events</text><text x="466" y="86" class="t-small">policyVersion = 1 hoặc 2</text><rect x="675" y="30" width="185" height="80" rx="10" class="f-phys-soft s-phys"/><text x="691" y="62" class="t-b">Bút toán chia cước</text><text x="691" y="86" class="t-small">phí theo policyVersion</text><path d="M190 70 L223 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-feature-flag-config-migration-expand-contract-3-a)"/><path d="M415 70 L448 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-feature-flag-config-migration-expand-contract-3-a)"/><path d="M640 70 L673 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-feature-flag-config-migration-expand-contract-3-a)"/><rect x="20" y="150" width="395" height="80" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="182" class="t-b">Cờ đổi sau đó (bật, tắt, đổi tỷ lệ)</text><text x="36" y="206" class="t-small">không đụng tới chuyến đã có sự kiện</text><rect x="450" y="150" width="410" height="80" rx="10" class="f-phys-soft s-phys"/><text x="466" y="182" class="t-b">Replay đọc fee_events, không đọc cờ</text><text x="466" y="206" class="t-small">dựng lại phí từng chuyến, so với bút toán đã ghi</text><path d="M545 110 L545 148" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-feature-flag-config-migration-expand-contract-3-a)"/></svg>
</div>
<figcaption>Hình 4. Cờ chỉ có mặt ở một chỗ, trước sự kiện. Sau sự kiện, quy tắc phí là dữ liệu chứ không còn là cấu hình.</figcaption>
</figure>

Quy tắc phí là enum, `match` vét cạn: thêm `V3` mà quên cập nhật là lỗi biên dịch. Quy tắc mới có sàn, cài trên `FeePolicy` của [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron) (`src/policy.rs`):

```rust
fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
    let by_rate = self.rounding.round_ratio(fare, self.rate)?;
    Ok(by_rate.max(self.floor).min(fare))
}
```

**Bật theo tỷ lệ, tất định theo `tripId`.** Nhóm của chuyến là hàm thuần của tên cờ và mã chuyến. Hàm băm là FNV-1a tự viết, vì `DefaultHasher` không đặc tả thuật toán và có thể đổi giữa các bản Rust (`src/flags.rs`):

```rust
pub fn bucket(flag: FlagName, trip_id: &str) -> u8 {
    let bytes = flag.as_str().bytes()
        .chain(std::iter::once(b':'))
        .chain(trip_id.bytes());
    u8::try_from(finalize(fnv1a(bytes)) % 100).expect("số dư chia 100 nhỏ hơn 256")
}
// Flag::selects: self.enabled && bucket(self.name, trip_id) < self.rollout.percent()
```

```text
nhóm của TN-0001..TN-0004: [62, 15, 97, 9]
số chuyến được chọn trong TN-0001..TN-1000: [(0, 0), (10, 115), (30, 309), (50, 491), (100, 1000)]
đếm lại ra cùng số: true; tăng tỷ lệ chỉ thêm chuyến: true
bộ chọn ngẫu nhiên hỏi hai lần ở 50%: 513 trong 1000 chuyến đổi ý
```

Ở 10%, trong bốn chuyến đầu chỉ `TN-0004` được chọn. Dòng cuối là cách sai, chọn «ngẫu nhiên» mỗi lần hỏi: hơn nửa số chuyến đổi ý.

**Replay.** Cờ tắt cho `TN-0001`, `TN-0002`, bật 100% cho `TN-0003`, `TN-0004`. Bút toán cũ không đổi khi cờ bật; replay theo `policy_version` ra đúng từng đồng, còn áp quy tắc hiện hành thì lệch:

```text
TN-0001: cước 25500, quy tắc V1, phí 5100, tài xế 20400
TN-0002: cước 12000, quy tắc V1, phí 2400, tài xế 9600
TN-0003: cước 12000, quy tắc V2, phí 3000, tài xế 9000
TN-0004: cước 25500, quy tắc V2, phí 3825, tài xế 21675
  sự kiện {"schemaVersion":1,"tripId":"TN-0001","fareDong":25500,"policyVersion":1}
replay theo policy_version: 4 sự kiện, lệch 0, doanh thu replay 14325 / sổ 14325
replay bằng quy tắc hiện hành (SAI): lệch 2 chuyến, doanh thu replay 13650 / sổ 14325
  TN-0001 (sự kiện v1): phí replay 3825 / sổ 5100, tài xế replay 21675 / sổ 20400
```

(Rút gọn: bỏ ba dòng sự kiện và dòng lệch của `TN-0002`.) `TN-0001` giữ phí 5.100 và tài xế 20.400 dù quy tắc đã đổi. Tắt cờ rồi giao lại `TN-0004`: bản đóng băng trả đúng bút toán cũ, bản hỏi lại cờ sinh phí khác và nhận lỗi idempotency:

```text
tắt cờ, giao lại TN-0004: đóng băng -> trả bút toán cũ true (quy tắc V2); hỏi lại cờ -> khóa `trip:TN-0004:split` đã có một bút toán khác nội dung
hai worker cùng xử lý TN-0005 với hai phiên bản đề xuất khác nhau: cùng phiên bản true, 1 bên ghi mới, 1 bút toán chia cước
```

## Rollback: code lùi được, dữ liệu thì không

Rollback code không rollback dữ liệu: bút toán và sự kiện đã ghi vẫn nằm đó. Lab thử lùi từng bước:

| Đã làm | Lùi bằng | Lab quan sát |
| --- | --- | --- |
| Bật cờ | Tắt cờ, tức thì | Chuyến đã tính `V2` giữ `policy_version = 2`; bút toán không đổi, replay vẫn đúng |
| Expand, code ghi đúp | Triển khai lại code cũ | Code cũ ghi được, nhưng đọc `fee_percent` của `TN-0011` ra 15% khi tỷ lệ thật là 25% |
| Contract A | Code cũ vẫn chạy (trigger còn) | Hình 1: code cũ ghi được; nhưng dòng do code «chỉ cột mới» ghi có `fee_percent` rỗng, báo cáo của code cũ sẽ lỗi vì `NULL` |
| Contract B | Chỉ bằng dựng lại | Code cũ ghi lỗi `23502`, đọc báo cáo lỗi `42703` |

Sau contract B, «lùi» là một migration mới: thêm lại cột và trigger. Cột `fee_percent` trở lại rỗng ở 14 trên 15 dòng, và báo cáo của code cũ lỗi vì `NULL`. Lab suy lại được vì `policy_version` còn và tỷ lệ danh nghĩa của mỗi phiên bản biết trước. Cột bị bỏ mà không suy ra được thì chỉ còn bản sao lưu.

Thứ tự lùi vì thế là: tắt cờ trước, rồi lùi code, và chỉ contract B khi chắc không còn lùi.

## Cấu hình kiểu an toàn: sai thì không khởi động

Cấu hình là chuỗi thì gõ sai không báo gì: `legacy_fee_rule` chỉ nhận đúng `"v2"`, còn `"V2"` hay `"v3"` âm thầm thành `V1`. Với enum và `deny_unknown_fields`, lỗi hiện ra trước chuyến đầu tiên (`src/config.rs`, rút gọn):

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawConfig {
    default_fee_rule: PolicyVersion,
    migration: RawMigration,
}
// Config::from_json: serde_json::from_str rồi in_range("migration.batchSize", ..., 1, 10_000, ...)
```

```text
defaultFeeRule = "v3"
  -> không đọc được cấu hình: unknown variant `v3`, expected `v1` or `v2` at line 2 column 24
batchSise (gõ sai tên trường)
  -> không đọc được cấu hình: unknown field `batchSise`, expected one of `batchSize`, `lockTimeoutMs`, `maxAttempts` at line 3 column 28
batchSize = 0
  -> `migration.batchSize` = 0, cần 1 đến 10000
cách chuỗi, "v2" / "V2" / "v3" thành: [V2, V1, V1]
```

(Rút gọn: bỏ ca `lockTimeoutMs = 60000`.) Cờ trong DB cũng được kiểm lúc khởi động: `validate_all` từ chối tên ngoài enum `FlagName` và từ chối khi thiếu dòng của cờ đã biết.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Bảng nhỏ (vài trăm dòng), một PostgreSQL local, không có tải ghi liên tục: lab chứng minh <em>cơ chế</em> khóa và thứ tự, không đo thời gian giữ khóa trên bảng lớn. Cờ đọc từ DB mỗi chuyến, chưa có cache TTL, giao diện quản trị hay nhật ký ai đổi cờ. Backfill chưa điều tiết theo tải, chưa chờ replica. Guard chỉ có trên <code>postings</code>: không có trigger <code>TRUNCATE</code> và <code>journal_entries</code> của bài 24, còn <code>fee_events</code> chưa append-only. Chưa thử partition, khóa ngoại, đổi kiểu cột (viết lại bảng) hay khôi phục từ sao lưu.</p></div>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Mỗi thí nghiệm có schema `lab_<uuid>` riêng, dọn khi xong. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/56-feature-flag-migration/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/56-feature-flag-migration/Cargo.toml
```

`--verify` kiểm từng giá trị ở trên, đạt thì in `VERIFY OK`. `cargo test` chạy 17 test đơn vị và 15 test tích hợp. Không test nào assert thời gian: chờ khóa được quan sát qua `pg_stat_activity`. Lab dùng `sqlx` 0.9.0, `tokio` 1.54.1.

**Bài tập:** đổi tên `fee_events.fee_percent` thành `fee_rate_percent` mà code cũ (đọc `fee_percent`) vẫn chạy. Làm theo thứ tự nào, và khi nào được `DROP COLUMN fee_percent`?

<details><summary>Xem lời giải</summary><p>Không dùng <code>RENAME COLUMN</code>: code cũ lỗi <code>42703</code> ngay. Làm theo expand/contract: (1) thêm <code>fee_rate_percent</code> nullable cùng trigger ghi hộ; (2) code mới ghi cả hai cột; (3) backfill theo lô có checkpoint; (4) <code>CHECK ... NOT VALID</code>, <code>VALIDATE</code>, <code>SET NOT NULL</code>; (5) nới <code>NOT NULL</code> của <code>fee_percent</code>, đổi code mới sang chỉ ghi cột mới; (6) chỉ <code>DROP COLUMN fee_percent</code> khi không còn instance cũ và chấp nhận đây là bước không rollback được.</p></details>

Bài trước: [bài 55](/articles/rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version). Bài tiếp: [bài 57](/articles/rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao).

## Tài liệu gốc

- [PostgreSQL: ALTER TABLE, mục `ADD CONSTRAINT ... NOT VALID`, `VALIDATE CONSTRAINT` và `SET NOT NULL`](https://www.postgresql.org/docs/18/sql-altertable.html).
- [PostgreSQL: Explicit Locking, bảng chế độ khóa mức bảng](https://www.postgresql.org/docs/18/explicit-locking.html).
- [PostgreSQL: view `pg_locks`](https://www.postgresql.org/docs/18/view-pg-locks.html).
- [PostgreSQL: `lock_timeout` trong Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html).
- [Rust std: `DefaultHasher`](https://doc.rust-lang.org/std/hash/struct.DefaultHasher.html).
- [docs.rs: sqlx 0.9.0, `PgConnectOptions`](https://docs.rs/sqlx/0.9.0/sqlx/postgres/struct.PgConnectOptions.html).

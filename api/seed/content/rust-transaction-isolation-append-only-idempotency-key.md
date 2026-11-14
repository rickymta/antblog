---
slug: rust-transaction-isolation-append-only-idempotency-key
title: "Transaction, isolation và idempotency key: ghi bút toán an toàn"
summary: "Bài 24: ghi journal_entries và postings trong một transaction sqlx, chống ghi trùng bằng ON CONFLICT DO NOTHING, tái hiện write skew ở READ COMMITTED, retry 40001 ở SERIALIZABLE và cấm sửa sổ bằng trigger."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, sqlx, postgresql, transaction]
language: vi
---

Payment (C#) gửi `PaymentCaptured` cho chuyến `TN-0001`, mạng chậm nên nó gửi lại, và hai bản tới ledger gần như cùng lúc. Sổ cái trong bộ nhớ của bài 20 an toàn nhờ `&mut self`, còn PostgreSQL nhận nhiều kết nối song song.

Đây là bài 24 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 23](/articles/rust-sqlx-postgresql-pool-migration-query) đã dựng `PgPool`; bài này dùng nó để ghi `JournalEntry` của `ledger-core` ([bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace)) xuống PostgreSQL, và chạy thật bốn điều: ghi nguyên tử, ghi một lần dù bị gọi nhiều lần, giữ hạn mức khi hai giao dịch chồng nhau, và không ai sửa được sổ đã ghi.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong một transaction, <code>SELECT</code> kiểm tra xem <code>idempotency_key</code> đã có chưa, chưa có thì <code>INSERT</code>. Hai request cùng key chạy song song có thể tạo hai bút toán không?</p>
<details><summary>Xem đáp án</summary><p>Có, nếu chỉ dựa vào câu <code>SELECT</code> đó. Ở READ COMMITTED, cả hai <code>SELECT</code> có thể cùng thấy «chưa có» rồi cả hai <code>INSERT</code>. Người quyết định cuối cùng phải là ràng buộc <code>UNIQUE</code>.</p></details>
</div>

## Transaction trong sqlx: commit tường minh, drop là rollback

Một bút toán là một dòng `journal_entries` cộng nhiều dòng `postings`; ghi dở thì sổ có bút toán không cân. `pool.begin()` lấy một kết nối và gửi `BEGIN`, `tx.commit()` kết thúc và làm các thay đổi có hiệu lực. Nếu `Transaction` bị drop mà chưa commit (vì `?` trả lỗi sớm, hoặc quên), sqlx xếp sẵn một `ROLLBACK` và gửi nó khi kết nối được dùng lại hoặc về pool: mặc định an toàn là «không có gì xảy ra».

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
pub async fn insert_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
) -> Result<Option<i64>, sqlx::Error> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return Ok(None); // key đã có: không chèn gì
    };
    let id: i64 = row.get("id");
    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    Ok(Some(id))
}
```

Hàm nhận `&mut PgConnection` chứ không nhận pool, nên người gọi quyết định ranh giới transaction. Thí nghiệm ghi `TN-0001` (3 posting), rồi `TN-0002` với tài xế `driver:DR-999` không có trong `accounts`: hai posting đầu chèn xong, posting thứ ba vi phạm khoá ngoại. Cuối cùng nó ghi `TN-0003`, đếm bên trong rồi drop không commit:

```text
commit TN-0001      : Created { id: 1 } (JE-000001)
sau commit          : 1 bút toán, 3 posting
lỗi ở posting 3     : SQLSTATE 23503 ràng buộc postings_account_id_fkey
sau lỗi giữa chừng  : 1 bút toán, 3 posting
trong tx chưa commit: 2 bút toán, 6 posting
sau khi drop tx     : 1 bút toán, 3 posting
ghi lại key đã drop : Created { id: 4 } (id có khoảng trống)
```

Dòng cuối chứng minh rollback đã chạy: ghi lại đúng key của transaction đã drop mà không bị treo (nếu transaction cũ còn mở, `ON CONFLICT` sẽ chờ nó, xem mục sau). Và `id: 4` chứ không phải `2`: identity không hoàn lại khi rollback, id 2 và 3 đã bị dùng mất cho hai lần thất bại.

Một chi tiết của sqlx 0.9: `sqlx::query` nhận `impl SqlSafeStr`, nên literal `&'static str` thì được, còn `String` ghép bằng `format!` bị chặn lúc biên dịch (rút gọn):

```text
error[E0277]: dynamic SQL strings should be audited for possible injections
  = help: the trait `SqlSafeStr` is not implemented for `std::string::String`
  = note: prefer literal SQL strings with bind parameters or `QueryBuilder` to add dynamic data to a query.
          To bypass this error, manually audit for potential injection vulnerabilities and wrap with `AssertSqlSafe()`.
```

Lab chỉ dùng `AssertSqlSafe` cho tên schema `lab_<uuid>` do chính lab sinh (`src/lab.rs`) và mã SQLSTATE cố định trong một helper của test; các câu SQL còn lại là literal có tham số.

## Hai task cùng một idempotency_key thì ai thắng?

Payment gửi lại thì lần hai phải trả bút toán lần một, không ghi thêm. Cơ chế gồm hai phần: cột `idempotency_key` unique, và `INSERT ... ON CONFLICT (idempotency_key) DO NOTHING RETURNING id`. Key mới thì câu lệnh trả một dòng `id`; key đã có thì câu lệnh không báo lỗi mà trả **không dòng nào**. Không có dòng trả về nghĩa là đã có người ghi, lúc đó ta đọc lại bút toán cũ và so nội dung.

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
pub async fn post_entry(pool: &PgPool, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
    let mut tx = pool.begin().await?;
    match insert_entry(&mut tx, entry).await? {
        Some(id) => {
            tx.commit().await?;
            Ok(PostOutcome::Created { id })
        }
        None => {
            // `ON CONFLICT` đã đợi giao dịch kia commit; câu SELECT mới (READ COMMITTED) thấy bản đó.
            let existing = load_by_key(&mut tx, entry.idempotency_key()).await?;
            tx.rollback().await?;
            match existing {
                Some((id, old)) if &old == entry => Ok(PostOutcome::Replayed { id }),
                _ => Err(StoreError::IdempotencyConflict {
                    key: entry.idempotency_key().to_string(),
                }),
            }
        }
    }
}
```

Cơ chế chờ là lý do câu lệnh này đúng ở READ COMMITTED. Giả sử T1 đã `INSERT` key nhưng chưa commit, T2 chạy cùng câu. T2 chưa thể quyết định: T1 commit thì T2 phải nhận về 0 dòng, T1 rollback thì T2 được chèn. Nên T2 chờ T1 kết thúc. Khi T1 commit, T2 nhận không dòng nào, và `SELECT` tiếp theo của T2 (statement mới, snapshot mới) thấy bút toán của T1.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. T1 chèn key và các posting rồi commit; T2 chèn cùng key phải chờ T1 kết thúc, sau đó nhận không dòng nào, đọc lại bút toán của T1 và so nội dung."><defs><marker id="rust-transaction-isolation-append-only-idempotency-key-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="75" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">T1 · INSERT ... RETURNING</text><text x="34" y="80" class="t-small">key mới: nhận id, chưa commit</text><rect x="300" y="25" width="250" height="75" rx="10" class="f-phys-soft s-phys"/><text x="314" y="55" class="t-b">T1 · INSERT postings</text><text x="314" y="80" class="t-small">dòng 1, 2, 3 cùng transaction</text><rect x="580" y="25" width="270" height="75" rx="10" class="f-pk-soft s-pk"/><text x="594" y="55" class="t-b">T1 · COMMIT</text><text x="594" y="80" class="t-small">key hiện ra với mọi phiên</text><path d="M270 62 L300 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-0-a)"/><path d="M550 62 L580 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-0-a)"/><rect x="20" y="150" width="530" height="75" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="180" class="t-b">T2 · INSERT ... ON CONFLICT DO NOTHING</text><text x="34" y="205" class="t-small">đứng chờ T1 kết thúc (Lock / transactionid), chưa trả lời</text><rect x="580" y="150" width="270" height="75" rx="10" class="f-phys-soft s-phys"/><text x="594" y="180" class="t-b">T2 · nhận 0 dòng</text><text x="594" y="205" class="t-small">SELECT lại bản cũ, so nội dung</text><path d="M715 100 L715 150" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-0-a)"/><text x="20" y="252" class="t-small">Khớp: Replayed. Khác: IdempotencyConflict. T1 rollback thay vì commit: T2 chèn được và thành bút toán mới.</text></svg>
</div>
<figcaption>Hình 1. T2 không bị lỗi và không tạo bản sao: nó chờ T1 kết thúc rồi nhận kết quả tương ứng với cách T1 kết thúc.</figcaption>
</figure>

Lab kiểm chứng bằng cách quan sát: T1 chèn key và giữ transaction mở; T2 lấy `pg_backend_pid()` rồi chèn cùng key; kết nối thứ ba đọc `pg_stat_activity` của T2 cho tới khi thấy nó đang chờ; rồi T1 commit hoặc rollback:

```text
giao dịch đầu commit  : phiên hai chờ Lock/transactionid, bị chặn: true, kết quả: không chèn (đọc lại bút toán cũ)
giao dịch đầu rollback: phiên hai chờ Lock/transactionid, bị chặn: true, kết quả: chèn được, id 2
```

Nếu T1 rollback, T2 không replay mà thành người ghi thật. `DO NOTHING` cũng gọn hơn «INSERT rồi bắt lỗi 23505», vì ở PostgreSQL một câu lệnh gặp lỗi sẽ làm hỏng cả transaction đang mở. Không dùng `DO UPDATE`: sổ cái không sửa bản cũ, và trigger ở mục cuối sẽ chặn nó.

Câu trả lời «0 dòng» ấy chỉ có ở READ COMMITTED. `pool.begin()` dùng mức mặc định của server, và mặc định của PostgreSQL chính là mức này. Ở REPEATABLE READ và SERIALIZABLE, snapshot chốt từ câu đầu tiên, nên bản commit sau đó nằm ngoài tầm nhìn. PostgreSQL không trả 0 dòng rồi để ta đọc lại mà không thấy gì: chính câu `INSERT` báo 40001 (`could not serialize access due to concurrent update`). Lab chốt snapshot trước, cho bản khác commit, rồi mới chèn:

```text
snapshot cũ ReadCommitted : 0 dòng, đọc lại thấy id 1
snapshot cũ RepeatableRead: SQLSTATE 40001, thử giao dịch mới: 0 dòng, đọc lại thấy id 1
snapshot cũ Serializable  : SQLSTATE 40001, thử giao dịch mới: 0 dòng, đọc lại thấy id 1
```

Thử trong psql, T2 đang chờ T1 cũng cho kết quả như vậy: T1 commit thì T2 nhận 40001, T1 rollback thì T2 chèn được. Muốn ghi idempotent ở mức cao hơn thì phải bọc trong vòng retry của mục sau: giao dịch mới có snapshot mới, thấy bản cũ và nhận 0 dòng.

Rồi lab chạy đồng thời: `tokio::join!` hai lần `post_entry` cùng bút toán `TN-0001`, rồi tám task dùng `Barrier` để xuất phát cùng lúc với bút toán `TN-0002`. Task nào thắng có thể khác giữa các lần chạy, số bút toán được tạo thì không:

```text
tokio::join! 2 task : Created { id: 1 } / Replayed { id: 1 }
8 task cùng key     : 1 Created, 7 Replayed, cùng id: true
số dòng             : 2 bút toán, 6 posting
```

## Cùng key, nội dung khác thì sao?

Chỉ so key là chưa đủ. Nếu `trip:TN-0001:capture` đã ghi cước 25.500 mà có yêu cầu cùng key với cước 30.000, đó là lỗi của bên gọi, không phải replay. `load_by_key` dựng lại bút toán đã ghi thành `JournalEntry` của `ledger-core` (qua `try_new`, nên bất biến cân tổng vẫn được kiểm), rồi so `==` với bản mới; `PartialEq` do bài 20 derive nên thứ tự posting cũng được tính:

```text
cùng key, cước khác : idempotency_key trip:TN-0001:capture đã dùng cho bút toán khác nội dung
```

Thông điệp là `Display` của `StoreError::IdempotencyConflict`, dựng bằng `thiserror` như [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te).

`ON CONFLICT DO NOTHING` cũng dùng mất một giá trị identity. Thử trong psql: chèn key `a` được id 1, chèn lại `a` không trả dòng, chèn key `b` được id 3. Vậy mã `JE-000001` (`format!("JE-{:06}", id)`) tăng dần nhưng **không liên tục**.

## READ COMMITTED có chặn được write skew không?

Không. Giả sử tài xế `DR-002` có hạn mức rút 30.000 đồng và hai yêu cầu rút 20.400 đến cùng lúc. Mỗi yêu cầu tính tổng đã rút từ `postings`, nếu cộng thêm vẫn trong hạn mức thì ghi một bút toán rút (tiền ra `driver:DR-002`, vào `psp:vixanh:clearing`).

```rust
// src/scenarios.rs, thân hàm withdraw (rút gọn: bỏ chú thích, gộp dòng)
let mut tx = isolation.begin(pool).await?;
let used: i64 = sqlx::query(
    "SELECT COALESCE(SUM(-amount_dong), 0)::bigint FROM postings \
     WHERE account_id = $1 AND amount_dong < 0",
)
.bind(driver)
.fetch_one(&mut *tx)
.await?
.get(0);
let used = Money::from_dong(used);
if used.checked_add(amount).is_none_or(|total| total > WITHDRAW_LIMIT) {
    return Err(StoreError::LimitExceeded { used, requested: amount, limit: WITHDRAW_LIMIT });
}
if let Some(gate) = gate {
    gate.wait().await; // cả hai đã đọc `used`; giờ mới cùng ghi
}
insert_entry(&mut tx, &payout(key, driver, amount)?).await?;
tx.commit().await?;
```

`gate` là một `Barrier` hai bên, ép cả hai đọc `used` xong rồi mới cùng ghi, để lab tái hiện đúng thứ tự xen kẽ của race condition. Ở READ COMMITTED mỗi câu lệnh thấy dữ liệu đã commit lúc nó bắt đầu, cả hai `SELECT` chạy trước khi bên nào commit, và hai bên chèn **hai dòng khác nhau** nên không khoá chung dòng nào. PostgreSQL không có lý do để chặn ai. REPEATABLE READ cũng lọt: hai bên đọc trên snapshot riêng, không ghi đè dòng của nhau, nên không có xung đột nào để nó phát hiện.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 2. Hai giao dịch cùng đọc tổng đã rút bằng 0, cùng thấy còn trong hạn mức rồi cùng ghi 20.400. READ COMMITTED để cả hai thành công, tổng 40.800 vượt 30.000; SERIALIZABLE huỷ một bên bằng 40001 và lần thử lại bị từ chối."><defs><marker id="rust-transaction-isolation-append-only-idempotency-key-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="15" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="42" class="t-b">T1 · SELECT tổng đã rút → 0 đồng</text><rect x="460" y="15" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="474" y="42" class="t-b">T2 · SELECT tổng đã rút → 0 đồng</text><rect x="20" y="85" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="112" class="t-b">T1 · 0 + 20.400 ≤ 30.000: được phép</text><rect x="460" y="85" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="474" y="112" class="t-b">T2 · 0 + 20.400 ≤ 30.000: được phép</text><rect x="20" y="155" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="182" class="t-b">T1 · INSERT bút toán rút, COMMIT</text><rect x="460" y="155" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="474" y="182" class="t-b">T2 · INSERT bút toán rút, COMMIT</text><path d="M220 59 L220 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-1-a)"/><path d="M660 59 L660 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-1-a)"/><path d="M220 129 L220 155" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-1-a)"/><path d="M660 129 L660 155" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-transaction-isolation-append-only-idempotency-key-1-a)"/><rect x="20" y="225" width="400" height="70" rx="10" class="f-warn-soft s-warn"/><text x="34" y="252" class="t-b">READ COMMITTED</text><text x="34" y="276" class="t-small">cả hai thành công: tổng rút 40.800 &gt; 30.000</text><rect x="460" y="225" width="400" height="70" rx="10" class="f-pk-soft s-pk"/><text x="474" y="252" class="t-b">SERIALIZABLE</text><text x="474" y="276" class="t-small">một bên nhận 40001, lần thử lại bị từ chối</text></svg>
</div>
<figcaption>Hình 2. Write skew: mỗi giao dịch đúng khi nhìn riêng, sai khi cộng lại. Mức cô lập quyết định việc database có phát hiện được lỗi này hay không.</figcaption>
</figure>

Kết quả chạy ở hai mức đó:

```text
ReadCommitted : 2 ok, 0 bị từ chối, 0 lần 40001, số lần thử [1, 1], tổng đã rút 40800 đồng
RepeatableRead: 2 ok, 0 bị từ chối, 0 lần 40001, số lần thử [1, 1], tổng đã rút 40800 đồng
```

Cả hai yêu cầu thành công, tổng 40.800 vượt 30.000. Không có lỗi nào để bắt: sổ vẫn cân, chỉ có quy tắc nghiệp vụ nằm ngoài ràng buộc của bảng bị phá. Ngoài SERIALIZABLE, còn một hướng khác là khoá một dòng chung (ví dụ `SELECT ... FOR UPDATE`) trước khi tính tổng; lab không làm hướng này, một bài ở cấp nâng cao sẽ quay lại hướng này.

## SERIALIZABLE trả 40001: retry có giới hạn

Lab mở giao dịch bằng `pool.begin_with("BEGIN ISOLATION LEVEL SERIALIZABLE")`. Ở mức này, PostgreSQL bảo đảm kết quả như thể các giao dịch SERIALIZABLE chạy lần lượt. PostgreSQL không khoá chặn trước mà theo dõi ai đọc gì, ai ghi gì (SSI); khi thấy mẫu phụ thuộc có thể thành vòng không xếp được thứ tự (mỗi bên ghi vào vùng bên kia vừa đọc), nó huỷ một giao dịch với SQLSTATE `40001`. Lỗi có thể đến ở bất kỳ câu nào, kể cả `COMMIT`. Bảo đảm này chỉ có hiệu lực khi mọi giao dịch đụng tới dữ liệu đó cùng chạy SERIALIZABLE: một đường rút khác ở READ COMMITTED vẫn lọt.

Lấy SQLSTATE qua `sqlx::Error::Database`; lỗi không đến từ database (mất kết nối, timeout pool) trả `None`:

```rust
// src/store.rs
pub fn sqlstate(err: &sqlx::Error) -> Option<String> {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map(|code| code.into_owned()),
        _ => None,
    }
}
```

Thử lại cần ba điều kiện: chạy lại **cả transaction từ đầu** (kể cả bước kiểm tra hạn mức, vì kết quả đã đọc trước đó không còn hiệu lực); **có giới hạn** số lần; và chỉ thử lại với 40001, còn lỗi khác (khoá ngoại, `LimitExceeded`) trả ngay.

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
pub async fn retry_serializable<T, F, Fut>(
    max_attempts: u32,
    mut op: F,
) -> Result<(T, u32), StoreError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, StoreError>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Ok(value) => return Ok((value, attempt)),
            Err(err) if err.is_serialization_failure() => {
                if attempt >= max_attempts {
                    return Err(StoreError::RetriesExhausted { attempts: attempt });
                }
                tokio::time::sleep(Duration::from_millis(10 * u64::from(attempt))).await;
                attempt += 1;
            }
            Err(other) => return Err(other),
        }
    }
}
```

`op` nhận số lần thử và mở transaction mới mỗi lần; lab dùng số lần thử này để `Barrier` chỉ chặn ở lần đầu. Hai yêu cầu rút ở `SERIALIZABLE`, mỗi bên tối đa 3 lần:

```text
Serializable  : 1 ok, 1 bị từ chối, 1 lần 40001, số lần thử [1], tổng đã rút 20400 đồng
```

Một bên thành công ngay lần đầu. Bên kia bị 40001, thử lại, đọc thấy 20.400 đã rút nên bị `LimitExceeded`: hạn mức được giữ. Số lần 40001 phụ thuộc thứ tự chạy của các task; cái không đổi là tổng đã rút không vượt 30.000, và `--verify` kiểm điều đó. Vòng retry còn có test không cần dựng race condition: khối `DO` với `RAISE EXCEPTION ... USING ERRCODE = '40001'` phát một lỗi 40001 thật, để kiểm tra giới hạn số lần thử và việc «lỗi 23505 không bị thử lại».

SERIALIZABLE không miễn phí (phải theo dõi lần đọc, có thể huỷ nhầm), nên chỉ bật cho đường ghi có quy tắc liên quan đến nhiều dòng. Tài liệu PostgreSQL khuyên thử lại cả `40P01` (deadlock), nhưng `retry_serializable` **chưa xử lý** mã này: gặp mã này thì lỗi được trả thẳng cho người gọi. Lab không dựng deadlock nên chưa có test; code thật phải thêm.

## Làm sao cấm sửa sổ ngay trong database?

Append-only ở bài 20 là quy ước của kiểu (không có hàm sửa). Khi đưa xuống PostgreSQL, quy ước này phải thành cơ chế để không client nào sửa được dòng đã ghi. Trigger `BEFORE UPDATE OR DELETE` làm việc đó:

```sql
-- sql/schema.sql
CREATE FUNCTION forbid_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'sổ cái append-only: % trên % bị cấm', TG_OP, TG_TABLE_NAME
        USING ERRCODE = 'TN001';
END;
$$;

CREATE TRIGGER postings_append_only
    BEFORE UPDATE OR DELETE ON postings
    FOR EACH ROW EXECUTE FUNCTION forbid_mutation();

CREATE TRIGGER postings_no_truncate
    BEFORE TRUNCATE ON postings
    FOR EACH STATEMENT EXECUTE FUNCTION forbid_mutation();
```

`journal_entries` có hai trigger tương tự. Trigger riêng cho `TRUNCATE` là bắt buộc: `TRUNCATE` không kích hoạt trigger theo dòng, và chỉ gắn được trigger cấp statement. `TN001` là SQLSTATE tự đặt của loạt bài để Rust phân biệt với lỗi khác. Lab thử bốn câu lệnh phá sổ đang chứa `TN-0001`:

```text
UPDATE postings SET amount_dong = 1 WHERE entry_id = 1 -> SQLSTATE TN001: sổ cái append-only: UPDATE trên postings bị cấm
DELETE FROM postings WHERE entry_id = 1 -> SQLSTATE TN001: sổ cái append-only: DELETE trên postings bị cấm
UPDATE journal_entries SET idempotency_key = 'x' WHERE id = 1 -> SQLSTATE TN001: sổ cái append-only: UPDATE trên journal_entries bị cấm
TRUNCATE postings -> SQLSTATE TN001: sổ cái append-only: TRUNCATE trên postings bị cấm
sau khi thử phá: 1 bút toán, 3 posting, tổng 0 đồng
```

Nếu ghi sai thì sửa bằng bút toán đảo, như `reverse` của `ledger-core`.

## So với EF Core ở bài C#

Bài [Transaction, optimistic concurrency và idempotency](/articles/dotnet10-transaction-concurrency-idempotency) giải cùng bài toán bằng EF Core.

| Việc | EF Core (bài C#) | sqlx (bài này) |
|---|---|---|
| Transaction | `SaveChanges` bọc transaction ngầm | `pool.begin()`; drop là rollback |
| Chống ghi trùng | Bắt 23505 rồi đọc lại | `ON CONFLICT DO NOTHING RETURNING` |
| Ghi chồng | Token phiên bản (optimistic) | `SERIALIZABLE`/khoá dòng, retry 40001 |

Hai bên cùng giữ một nguyên tắc: ràng buộc unique là lớp quyết định cuối. sqlx đòi SQL tường minh ở từng bước; EF Core gọn hơn ở chỗ đó, đổi lại phải kiểm tra SQL do provider sinh ra ở các trường hợp biên.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chạy 2 đến 8 task trên một PostgreSQL local, không đo hiệu năng, không mô phỏng mất kết nối giữa chừng, chưa có <code>account_balances</code> (bài 25) và chưa dùng <code>reversal_of</code>. Chưa đặt <code>lock_timeout</code>: ở production, một transaction bị bỏ quên giữ key sẽ làm mọi lần ghi cùng key chờ vô hạn. Chủ sở hữu bảng vẫn chạy được <code>ALTER TABLE ... DISABLE TRIGGER</code>, nên role ứng dụng không nên là chủ bảng và cũng không nên có quyền <code>UPDATE</code>, <code>DELETE</code>, <code>TRUNCATE</code>.</p></div>

## Chạy ví dụ và kiểm chứng

PostgreSQL của lab chạy bằng Docker Compose trong `examples/toi-noi-rust/infra`; mỗi thí nghiệm tạo schema `lab_<uuid>` riêng và xoá khi xong. Thiếu `TOI_NOI_PG_URL` thì lab thoát mã 1 và nói cách bật compose. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/24-transaction-idempotency/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/24-transaction-idempotency/Cargo.toml
```

`--verify` in các dòng kết quả ở trên rồi kiểm từng giá trị; đạt thì in `VERIFY OK`, thoát mã 0. `cargo test` chạy 12 test tích hợp: chín thí nghiệm và ba test vòng retry. Lab dùng `sqlx` 0.9.0, không bật TLS vì chỉ nối localhost.

**Bài tập:** sau thí nghiệm 1, bút toán thành công kế tiếp có `id` và mã hiển thị là gì? Vì sao không nên dùng «id liên tục» để kết luận sổ không thiếu bút toán?

<details><summary>Xem lời giải</summary><p>Id 1 đến 4 đã được cấp (kể cả id 2 lỗi khoá ngoại và id 3 bị rollback), nên bút toán kế tiếp có id 5, mã <code>JE-000005</code>. Identity không hoàn lại khi rollback hay khi <code>ON CONFLICT DO NOTHING</code> không chèn gì, nên khoảng trống là bình thường. Muốn biết sổ có thiếu bút toán hay không thì phải đối soát với nguồn và kiểm tổng, không đếm id.</p></details>

Bài trước: [bài 23](/articles/rust-sqlx-postgresql-pool-migration-query). Bài tiếp: [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition).

## Tài liệu gốc

- [PostgreSQL: Transaction Isolation](https://www.postgresql.org/docs/current/transaction-iso.html) và [Serialization Failure Handling](https://www.postgresql.org/docs/current/mvcc-serialization-failure-handling.html).
- [PostgreSQL: INSERT, mục ON CONFLICT Clause](https://www.postgresql.org/docs/current/sql-insert.html).
- [PostgreSQL: Appendix A, bảng mã lỗi SQLSTATE](https://www.postgresql.org/docs/current/errcodes-appendix.html) và [CREATE TRIGGER](https://www.postgresql.org/docs/current/sql-createtrigger.html).
- [docs.rs: sqlx, `Transaction`](https://docs.rs/sqlx/0.9.0/sqlx/struct.Transaction.html), [`Error::Database`](https://docs.rs/sqlx/0.9.0/sqlx/enum.Error.html) và [`AssertSqlSafe`](https://docs.rs/sqlx/0.9.0/sqlx/struct.AssertSqlSafe.html).
- [docs.rs: tokio, `sync::Barrier`](https://docs.rs/tokio/latest/tokio/sync/struct.Barrier.html).

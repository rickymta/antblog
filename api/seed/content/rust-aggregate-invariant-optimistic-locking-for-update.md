---
slug: rust-aggregate-invariant-optimistic-locking-for-update
title: "Aggregate, invariant và concurrency: version, optimistic locking, FOR UPDATE"
summary: "Bài 53: aggregate Account giữ hạn mức rút trong domain; optimistic locking bằng version so với SELECT FOR UPDATE, retry có giới hạn và jitter, thứ tự khoá theo id tránh deadlock 40P01, kiểm bằng N task Barrier trên PostgreSQL."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, postgresql, concurrency, aggregate]
language: vi
---

Tài xế `DR-001` có 150.000 VND trong ví và hạn mức rút 100.000 mỗi kỳ. Hai thiết bị gần như cùng lúc gửi lệnh rút 20.400 (phần tiền chuyến `TN-0001`); mỗi lệnh đọc ví, kiểm hạn mức, rồi ghi. Chạy lần lượt thì đúng. Chồng nhau thì mỗi bút toán vẫn cân, nhưng ví đã sai.

Đây là bài 53 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 52 đặt [command handler](/articles/rust-command-handler-pipeline-domain-event-khong-framework) quanh một transaction, chưa thử lại khi xung đột. Bài 24 cho thấy [write skew phá hạn mức rút](/articles/rust-transaction-isolation-append-only-idempotency-key) và để ngỏ hướng khoá dòng; bài 42 dùng `FOR UPDATE` cho [hold](/articles/rust-hold-capture-void-so-du-kha-dung). Bài này đặt cạnh nhau: invariant sống ở đâu, version hay khoá, và hai ví thì khoá theo thứ tự nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tám task cùng rút 20.400 từ ví 150.000 có hạn mức 100.000. Mỗi task đọc ví, tính số dư mới trong bộ nhớ rồi ghi đè vào cột số dư; không khoá, không version. Tổng thực rút theo sổ là bao nhiêu, và cột số dư cuối ghi bao nhiêu?</p>
<details><summary>Xem đáp án</summary><p>Cả tám bút toán được ghi, tổng rút 163.200 VND, vượt hạn mức. Cột số dư chỉ ghi 129.600 VND (trừ đúng một lần 20.400) vì mỗi task ghi đè kết quả của bên kia.</p></details>
</div>

## Invariant của ví nằm ở domain hay ở SQL?

Aggregate là ranh giới nhất quán: nạp lên, quyết định, lưu lại. `Account` giữ hai invariant: số dư không âm sau mỗi lần chi ra, và tổng đã rút trong kỳ không vượt hạn mức. Quyết định nằm trong một hàm không biết SQL:

```rust
// src/account.rs (rút gọn: bỏ chú thích tài liệu)
fn spend(&mut self, amount: Money) -> Result<(), AccountError> {
    if amount <= Money::ZERO {
        return Err(AccountError::NotPositive(amount));
    }
    let balance = self.balance.checked_sub(amount)
        .filter(|left| *left >= Money::ZERO)
        .ok_or(AccountError::InsufficientBalance { balance: self.balance, requested: amount })?;
    let withdrawn = self.withdrawn.checked_add(amount)
        .filter(|total| *total <= self.withdraw_limit)
        .ok_or(AccountError::LimitExceeded {
            withdrawn: self.withdrawn, requested: amount, limit: self.withdraw_limit,
        })?;
    // Chỉ đổi trạng thái khi mọi kiểm tra đã qua.
    self.balance = balance;
    self.withdrawn = withdrawn;
    Ok(())
}
```

Trường của `Account` là private: code ngoài module chỉ đổi được qua `withdraw`, `transfer_to`, `receive`, còn `restore` kiểm lại invariant khi nạp từ database. Module không có `async` hay `sqlx`, nên unit test không cần database.

Hạn mức cần chỗ lưu, nên lab thêm bảng `account_limits`. Bảng này chỉ của bài 53, nằm ngoài bốn bảng chuẩn của bài 25. `CHECK (withdrawn_dong <= withdraw_limit_dong)` bắt được giá trị vô lý nhưng, như mục sau sẽ thấy, không bắt được cuộc đua.

`JournalEntry` cũng là aggregate, theo kiểu bất biến. Invariant của nó (ít nhất hai posting, tổng bằng 0) được kiểm lúc dựng ([bài 13](/articles/rust-double-entry-journal-entry-posting)) và bằng trigger lúc commit ([bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition)). Không ai sửa bút toán sau khi ghi, nên không có hai người sửa chồng nhau và `journal_entries` không có cột `version`. Lab thử sửa thì sổ tự từ chối:

```text
UPDATE journal_entries: SQLSTATE TN001; UPDATE postings: SQLSTATE TN001
```

Chống ghi trùng bút toán là việc của `idempotency_key` unique, không phải của version. Ở loạt C#, [aggregate Trip](/articles/dotnet10-ddd-bounded-context-aggregate) cũng bảo vệ invariant bằng hành vi; phần còn lại là chuyện qua được cuộc đua.

## Read-modify-write mất cập nhật như thế nào?

Lịch đầu là lịch cố định dựng bằng `Notify`, không phải may rủi: A nạp ví (150.000, version 1), B nạp thêm 50.000 và commit (200.000, version 2), rồi A mới ghi số nó tính từ bản cũ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. A nạp ví với số dư 150.000 và version 1. B nạp thêm 50.000 và commit, ví thành 200.000, version 2. A ghi đè số dư 129.600 tính từ bản cũ. Không khoá: số dư lưu 129.600, theo posting 179.600, mất 50.000. Optimistic: UPDATE với WHERE version = 1 trúng 0 dòng, A nạp lại version 2 và ghi 179.600."><defs><marker id="rust-aggregate-invariant-optimistic-locking-for-update-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="18" class="t-small">Thời gian chạy từ trên xuống</text><rect x="20" y="28" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">A · nạp: số dư 150.000, version 1</text><rect x="460" y="88" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="474" y="115" class="t-b">B · nạp 50.000, COMMIT: 200.000, version 2</text><rect x="20" y="148" width="400" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="175" class="t-b">A · ghi số dư 150.000 − 20.400 = 129.600</text><path d="M220 72 L220 148" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-0-a)"/><path d="M460 110 L420 164" class="f-none s-warn dash" stroke-width="2"/><rect x="20" y="214" width="400" height="72" rx="10" class="f-warn-soft s-warn"/><text x="34" y="240" class="t-b">Không khoá: ghi đè</text><text x="34" y="264" class="t-small">số dư lưu 129.600, theo posting 179.600: mất 50.000</text><rect x="460" y="214" width="400" height="72" rx="10" class="f-pk-soft s-pk"/><text x="474" y="240" class="t-b">Optimistic: WHERE version = 1</text><text x="474" y="264" class="t-small">0 dòng, nạp lại version 2, ghi 179.600</text></svg>
</div>
<figcaption>Hình 1. Lost update: câu UPDATE của A hợp lệ nên không có lỗi nào để bắt; chỉ điều kiện trên version cho A biết bản đã đọc đã cũ.</figcaption>
</figure>

Kết quả của bản không khoá, cạnh bản optimistic (mục sau):

```text
không khóa : 1 lần thử, rút được; số dư lưu 129.600 VND, số dư theo posting 179.600 VND; tổng số dư mọi tài khoản -50.000 VND, lệch projection ["driver:DR-001"]
optimistic : 2 lần thử, rút được; số dư lưu 179.600 VND, số dư theo posting 179.600 VND; tổng số dư mọi tài khoản 0 VND, lệch projection []
```

Sổ đóng kín nên tổng số dư mọi tài khoản phải bằng 0, và `account_balances` phải bằng tổng posting của từng tài khoản. Bản không khoá phá cả hai: 50.000 của B còn trong posting nhưng biến khỏi projection. Cùng lỗi ở quy mô tám task:

```text
không khóa : 8 ok, 0 bị từ chối, 0 hết lượt; số lần thử [1, 1, 1, 1, 1, 1, 1, 1]; số dư 129.600 VND, đã rút 20.400 VND, version 9
             tổng số dư mọi tài khoản 142.800 VND, lệch projection ["driver:DR-001"], chi quá hạn mức [("driver:DR-001", 163200, 100000)]
```

`account_limits` ghi `withdrawn = 20.400`, nằm trong hạn mức, nên `CHECK` im lặng: mỗi task ghi một con số riêng lẻ hợp lệ. Hạn mức bị vượt chỉ lộ ra khi cộng posting (163.200). `Barrier` đặt sau bước nạp bảo đảm cả tám task đọc cùng một bản cũ.

## Optimistic locking: 0 dòng là xung đột, rồi thử lại

Cách sửa thứ nhất không khoá khi đọc, chỉ thêm điều kiện lúc ghi: hàng phải còn ở đúng phiên bản đã đọc.

```rust
// src/store.rs, hàm save, nhánh Mode::Optimistic (rút gọn: bỏ chú thích)
let done = sqlx::query(
    "UPDATE account_balances \
     SET balance_dong = $2, version = version + 1, updated_at = now() \
     WHERE account_id = $1 AND version = $3",
)
.bind(id)
.bind(account.balance().dong())
.bind(account.version())
.execute(&mut *conn)
.await?;
if done.rows_affected() == 0 {
    return Err(StoreError::Conflict);
}
```

`rows_affected() == 0` là toàn bộ cơ chế phát hiện. Ở READ COMMITTED nó đúng vì: nếu bên kia đã sửa hàng nhưng chưa commit, `UPDATE` của ta chờ khoá hàng; khi bên kia commit, PostgreSQL đánh giá lại `WHERE` trên bản mới, thấy `version` không khớp và trả 0 dòng. Ở [loạt C#](/articles/dotnet10-transaction-concurrency-idempotency), EF đưa concurrency token vào `WHERE` thay ta; không ORM thì tự viết. Nguyên tắc giống nhau: mọi đường ghi tăng version, và sau xung đột phải đọc lại chứ không ghi đè cho hết lỗi.

Đường ghi chuẩn của bài 25 cộng dồn `account_balances` bằng upsert và tăng `version`, nên writer ngoài aggregate như lệnh nạp ví của B vẫn bị phát hiện. Writer nào quên tăng `version` thì cơ chế này mất tác dụng trong im lặng.

Xung đột là lỗi tạm thời, nên cần vòng thử lại. `retry_serializable` của bài 24 chỉ xử lý `40001` và ghi rõ chưa xử lý `40P01`; vòng dưới gộp cả ba loại:

```rust
// src/store.rs
pub fn is_retryable(&self) -> bool {
    match self {
        StoreError::Conflict => true,
        other => matches!(other.sqlstate().as_deref(), Some("40001" | "40P01")),
    }
}

// src/retry.rs, hàm with_retry (rút gọn: bỏ chú thích)
let mut attempt = 1;
loop {
    match op(attempt).await {
        Err(err) if err.is_retryable() => {
            if attempt >= policy.max_attempts {
                return Run { attempts: attempt, result: Err(StoreError::RetriesExhausted { attempts: attempt }) };
            }
            let wait = jitter.up_to(policy.ceiling_ms(attempt));
            tokio::time::sleep(Duration::from_millis(wait)).await;
            attempt += 1;
        }
        other => return Run { attempts: attempt, result: other },
    }
}
```

Mỗi lần thử mở transaction mới và nạp lại aggregate, vì domain phải quyết định lại: sau xung đột hạn mức có thể đã hết, và lỗi trả về là `LimitExceeded`. Số lần thử có giới hạn, hết thì trả `RetriesExhausted`. Thời gian chờ ngẫu nhiên trong `[0, min(trần, cơ sở × 2^(lần-1))]`, để các task vừa va nhau không thử lại cùng nhịp. Lỗi nghiệp vụ không bao giờ thử lại.

## Cần bao nhiêu lần thử khi nhiều task tranh một ví?

Có một cận chắc chắn. Mỗi lần T xung đột thì giữa lúc T nạp và lúc T ghi có một commit khác tăng `version`; các lần thử của T không chồng nhau, nên số xung đột của T không vượt số commit thành công. Ví chỉ cho bốn lần rút (4 × 20.400 = 81.600; lần năm sẽ là 102.000), nên mọi task xong sau tối đa 5 lần thử và `RetryPolicy::lab()` (6 lần) không bao giờ hết lượt. Một lần chạy, cạnh `FOR UPDATE`:

```text
optimistic : 4 ok, 4 bị từ chối, 0 hết lượt; số lần thử [1, 2, 3, 4, 5, 5, 5, 5]; số dư 68.400 VND, đã rút 81.600 VND, version 5
             tổng số dư mọi tài khoản 0 VND, lệch projection [], chi quá hạn mức []
FOR UPDATE : 4 ok, 4 bị từ chối, 0 hết lượt; số lần thử [1, 1, 1, 1, 1, 1, 1, 1]; số dư 68.400 VND, đã rút 81.600 VND, version 5
             tổng số dư mọi tài khoản 0 VND, lệch projection [], chi quá hạn mức []
```

Đúng bốn lần rút, sổ cân, projection khớp posting. Số lần thử đổi giữa các lần chạy (`--verify` chỉ kiểm đúng một task qua ngay lần đầu và không task nào quá 5 lần).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Số lần thử của tám task optimistic trên một ví: 1, 2, 3, 4, 5, 5, 5, 5. Đường nét đứt ở 5 lần là cận trên: bốn commit thành công nên mỗi task xung đột tối đa bốn lần."><rect x="110" y="14" width="1" height="224" class="f-muted"/><text x="20" y="34" class="t-small">task 1</text><rect x="110" y="20" width="100" height="20" rx="4" class="f-phys-soft s-phys"/><text x="220" y="34" class="t-small">1 lần</text><text x="20" y="62" class="t-small">task 2</text><rect x="110" y="48" width="200" height="20" rx="4" class="f-phys-soft s-phys"/><text x="320" y="62" class="t-small">2 lần</text><text x="20" y="90" class="t-small">task 3</text><rect x="110" y="76" width="300" height="20" rx="4" class="f-phys-soft s-phys"/><text x="420" y="90" class="t-small">3 lần</text><text x="20" y="118" class="t-small">task 4</text><rect x="110" y="104" width="400" height="20" rx="4" class="f-phys-soft s-phys"/><text x="520" y="118" class="t-small">4 lần</text><text x="20" y="146" class="t-small">task 5</text><rect x="110" y="132" width="500" height="20" rx="4" class="f-pk-soft s-pk"/><text x="620" y="146" class="t-small">5 lần</text><text x="20" y="174" class="t-small">task 6</text><rect x="110" y="160" width="500" height="20" rx="4" class="f-pk-soft s-pk"/><text x="620" y="174" class="t-small">5 lần</text><text x="20" y="202" class="t-small">task 7</text><rect x="110" y="188" width="500" height="20" rx="4" class="f-pk-soft s-pk"/><text x="620" y="202" class="t-small">5 lần</text><text x="20" y="230" class="t-small">task 8</text><rect x="110" y="216" width="500" height="20" rx="4" class="f-pk-soft s-pk"/><text x="620" y="230" class="t-small">5 lần</text><path d="M610 12 L610 244" class="f-none s-warn dash" stroke-width="2"/><text x="110" y="262" class="t-b">Nét đứt: cận 5 lần thử, vì chỉ có bốn commit thành công</text><text x="110" y="280" class="t-small">mỗi xung đột nghĩa là có một task khác vừa commit, nên số xung đột không vượt số commit thành công</text></svg>
</div>
<figcaption>Hình 2. Số lần thử của tám task tranh một ví, một lần chạy. Mốc nét đứt là cận do số commit thành công đặt ra.</figcaption>
</figure>

Phân bố phụ thuộc vào độ tranh chấp. `--measure` cho mười sáu task đều thành công (số tiền nhỏ, hạn mức lớn), mỗi dòng trên một schema mới (rút gọn: bỏ cột `ms` và năm dòng `FOR UPDATE` còn lại, dòng nào cũng toàn 1 lần thử):

```text
máy: 23:41  up 35 days, 15:25, 1 user, load averages: 139.75 154.24 127.61
chế độ       task   ví  số lần thử → số task                           tổng  max
optimistic     16   16  1→16                                             16    1
optimistic     16    4  1→4 2→4 3→4 4→4                                  40    4
optimistic     16    1  1→1 2→1 3→2 4→1 5→2 6→1 7→2 8→2 9→2 10→2         97   10
optimistic      2    1  1→1 2→1                                           3    2
optimistic      4    1  1→1 2→1 3→1 4→1                                  10    4
optimistic      8    1  1→1 2→1 3→1 4→1 5→1 6→2 7→1                      34    7
FOR UPDATE     16    1  1→16                                             16    1
```

Số đo trên máy viết bài (load average khoảng 100 đến 207, rất tải), không phải SLA. Bốn task mỗi ví: tổng 40 ở mọi lần đo, là quan sát chứ không phải bảo đảm. Mười sáu task trên một ví, qua sáu lần chạy: tổng từ 82 đến 129, tối đa từ 9 đến 16 lần thử; cận trên là 1+2+…+16 = 136, vì task commit thứ k xung đột tối đa k−1 lần. Chi phí của optimistic locking tăng cỡ bình phương số task trên một hàng nóng, và gần bằng không khi các task rơi vào hàng khác nhau.

## FOR UPDATE: xếp hàng thay vì thử lại

Cách thứ hai khoá hàng ngay lúc đọc. Task sau chờ task trước kết thúc nên không đọc bản cũ, và không có retry.

```rust
// src/store.rs, hàm load, nhánh lock = true (rút gọn: bỏ chú thích)
let head = sqlx::query(
    "SELECT balance_dong, version FROM account_balances WHERE account_id = $1 FOR UPDATE",
)
.bind(id)
.fetch_optional(&mut *conn)
.await?
.ok_or_else(|| StoreError::NoAccount(id.to_string()))?;
let limits = sqlx::query(
    "SELECT withdraw_limit_dong, withdrawn_dong FROM account_limits WHERE account_id = $1",
)
.bind(id)
.fetch_optional(&mut *conn)
.await?
.ok_or_else(|| StoreError::NoAccount(id.to_string()))?;
```

Hàm đọc hai câu thay vì một `JOIN`, và có lý do. Bản đầu của lab gộp thành `account_balances b JOIN account_limits l ... FOR UPDATE OF b` và tám task cho ra bảy lần rút thành công, tổng 142.800, vượt hạn mức; lần thứ tám chỉ bị chặn vì thiếu số dư (thí nghiệm riêng, chạy lại đều ra bảy, không nằm trong `--verify`). Lab giữ lại lỗi ở dạng một người chờ: A khoá ví và rút 20.400 nhưng chưa commit, B nạp bằng từng cách và bị chặn (lab đợi tới khi `pg_stat_activity` báo B đang chờ khoá, không đoán bằng thời gian), rồi A commit.

```text
đúng là: số dư 129.600 VND, đã rút 20.400 VND (A vừa commit)
JOIN, FOR UPDATE OF b     : người chờ thấy số dư 129.600 VND, đã rút 0 VND
JOIN, FOR UPDATE OF b, l  : người chờ thấy số dư 129.600 VND, đã rút 20.400 VND
hai câu (lock rồi đọc)    : người chờ thấy số dư 129.600 VND, đã rút 20.400 VND
```

[Tài liệu READ COMMITTED](https://www.postgresql.org/docs/current/transaction-iso.html#XACT-READ-COMMITTED) nói đúng chuyện này: sau khi chờ, câu lệnh đánh giá lại `WHERE`, khoá và trả bản mới của chính hàng đó, nhưng không thấy tác động của bên kia lên hàng khác. Mã nguồn PostgreSQL gọi bước này là EvalPlanQual: bảng không có trong `FOR UPDATE OF` giữ nguyên hàng đã đọc từ snapshot cũ. Nên số dư mới đi cùng số đã rút cũ, và hạn mức thủng. Khoá cả hai bảng thì hàng `l` cũng được đọc lại. Khoá trước rồi đọc ở câu riêng cũng đúng, vì ở READ COMMITTED câu thứ hai có snapshot mới; lab chọn cách này cho dễ nhìn. Bài 42 từng gặp đúng hiện tượng này với `SUM` đi cùng khoá.

Thí nghiệm `FOR UPDATE` không đặt được `Barrier` sau bước nạp, vì task đầu giữ khoá nên các task sau không bao giờ tới barrier; nó chỉ kiểm bất biến sau khi chạy.

Chọn cái nào tuỳ đặc điểm hàng:

| Tình huống | Optimistic (`version`) | `FOR UPDATE` |
|---|---|---|
| Ít tranh chấp (đa số ví không bị chạm đồng thời) | Không giữ khoá, hầu như không có retry | Khoá không tốn, nhưng giữ khoá suốt transaction |
| Một hàng nóng nhiều người cùng chạm | Retry tăng nhanh, có thể hết lượt | Xếp hàng, không retry, thông lượng một hàng tuần tự |
| Transaction gọi dịch vụ ngoài khi đang giữ hàng | Không treo ai, nhưng làm lại cả việc khi xung đột | Giữ khoá suốt lúc gọi, chặn mọi người sau |
| Hai ví trong một bút toán | Hai điều kiện version, retry cả hai; `UPDATE` vẫn giữ khoá hàng tới commit nên ghi ngược thứ tự vẫn có thể `40P01` | Phải khoá đúng thứ tự (mục sau) |

`NOWAIT` và `SKIP LOCKED` trả lời câu hỏi khác: khi hàng đang bị khoá thì làm gì. Lab giữ khoá `customer:CU-001` và thử trên kết nối khác:

```text
SELECT thường        : ["customer:CU-001", "customer:CU-002"]
FOR UPDATE SKIP LOCKED: ["customer:CU-002"]
FOR UPDATE NOWAIT    : SQLSTATE 55P03
```

`SELECT` thường không bị khoá chặn (MVCC). `NOWAIT` báo `55P03` ngay thay vì chờ. `SKIP LOCKED` lặng lẽ bỏ hàng đang bị khoá: đúng cho job nhặt việc như job hết hạn hold ở bài 42, vì dòng bị bỏ lại sẽ được lượt sau nhặt; sai cho lệnh rút, vì ví đang bận nghĩa là lệnh bị bỏ qua trong im lặng.

## Khoá hai ví: deadlock thật và thứ tự khoá

Chuyển ví sang ví đổi cả hai aggregate trong một bút toán, vì tổng bằng 0 là invariant của sổ chứ không của ví nào. Lời khuyên quen thuộc của DDD là mỗi transaction chỉ đổi một aggregate; ở đây nó nhường chỗ, và cái giá là phải khoá hai hàng. [Lab DDD của loạt C#](/articles/dotnet10-ddd-bounded-context-aggregate) không gặp cảnh này vì chỉ có một root và tham chiếu tài xế bằng ID. `transfer_once` khoá bằng `FOR UPDATE` theo `LockOrder`:

```rust
// src/store.rs, hàm transfer_once (rút gọn: bỏ chú thích và đoạn ghi)
let (first, second) = match order {
    LockOrder::AsGiven => (from, to),
    LockOrder::ById if from <= to => (from, to),
    LockOrder::ById => (to, from),
};
let a = load(conn, first, true).await?;
gate.reach().await; // lab: dừng sau khi khóa hàng đầu tiên
let b = load(conn, second, true).await?;
```

Hai lệnh ngược chiều (CU-001 sang CU-002 và ngược lại) với `AsGiven` và một `Barrier` sau khoá đầu tiên: mỗi bên giữ ví nguồn rồi xin ví của bên kia.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Bên trái: T1 giữ CU-001 và chờ CU-002, T2 giữ CU-002 và chờ CU-001, tạo vòng chờ nên PostgreSQL huỷ một bên bằng 40P01. Bên phải: khoá theo id, cả hai cùng xin CU-001 trước, bên đến sau xếp hàng nên không có vòng."><defs><marker id="rust-aggregate-invariant-optimistic-locking-for-update-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="22" class="t-b">Khoá theo thứ tự nhận được</text><rect x="20" y="50" width="130" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="77" class="t-b">T1</text><rect x="20" y="190" width="130" height="44" rx="10" class="f-phys-soft s-phys"/><text x="34" y="217" class="t-b">T2</text><rect x="290" y="50" width="130" height="44" rx="10" class="f-pk-soft s-pk"/><text x="304" y="77" class="t-b">CU-001</text><rect x="290" y="190" width="130" height="44" rx="10" class="f-pk-soft s-pk"/><text x="304" y="217" class="t-b">CU-002</text><path d="M150 72 L290 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-2-a)"/><text x="190" y="64" class="t-small">giữ</text><path d="M150 212 L290 212" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-2-a)"/><text x="190" y="228" class="t-small">giữ</text><path d="M120 94 L300 190" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-2-a)"/><path d="M120 190 L300 94" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-2-a)"/><text x="140" y="146" class="t-small">chờ</text><text x="240" y="146" class="t-small">chờ</text><rect x="20" y="256" width="400" height="34" rx="10" class="f-warn-soft s-warn"/><text x="34" y="278" class="t-small">vòng chờ: sau 1 giây PostgreSQL huỷ một bên bằng 40P01</text><text x="470" y="22" class="t-b">Khoá theo id nhỏ trước</text><rect x="470" y="50" width="390" height="44" rx="10" class="f-phys-soft s-phys"/><text x="484" y="77" class="t-small">T1: khoá CU-001, khoá CU-002, commit</text><rect x="470" y="110" width="390" height="44" rx="10" class="f-phys-soft s-phys"/><text x="484" y="137" class="t-small">T2: xin CU-001, xếp hàng sau T1</text><rect x="470" y="170" width="390" height="44" rx="10" class="f-phys-soft s-phys"/><text x="484" y="197" class="t-small">T2: được CU-001, khoá CU-002, commit</text><path d="M665 94 L665 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-2-a)"/><path d="M665 154 L665 170" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-aggregate-invariant-optimistic-locking-for-update-2-a)"/><rect x="470" y="256" width="390" height="34" rx="10" class="f-pk-soft s-pk"/><text x="484" y="278" class="t-small">thứ tự toàn phần thì không thể có vòng chờ</text></svg>
</div>
<figcaption>Hình 3. Deadlock cần một vòng chờ. Khi mọi giao dịch khoá cùng thứ tự, bên đến sau chỉ xếp hàng ở khoá đầu.</figcaption>
</figure>

```text
khóa theo thứ tự nhận được, không retry: 1 ok, 1 deadlock 40P01; số dư 45.000 VND / 55.000 VND
khóa theo thứ tự nhận được, retry 40P01: 2 ok, 0 deadlock; số lần thử [1, 2]; số dư 48.000 VND / 52.000 VND
khóa theo id, không retry:                2 ok, 0 deadlock; số lần thử [1, 1]; số dư 48.000 VND / 52.000 VND
12 lần chuyển ba ví hai chiều, khóa theo id: 12 ok, 0 deadlock; số lần thử tối đa 1; tổng ba ví 300.000 VND
nạn nhân chưa ROLLBACK: 1 bên vẫn khóa đủ hai hàng; nạn nhân nhận 40P01, lệnh kế của nó SQLSTATE 25P02
```

Đây là `40P01` thật. PostgreSQL chỉ kiểm vòng chờ sau khi một phiên đã chờ khoá đủ `deadlock_timeout` (1 giây, giá trị mặc định, trên server của lab), rồi huỷ một bên để bên kia đi tiếp. Tài liệu nói bên nào bị huỷ thì khó đoán: chạy lại lab, dòng đầu ra cả 45.000/55.000 lẫn 53.000/47.000, nên `--verify` chỉ kiểm «một ok, một deadlock». Dòng ba là cách sửa gốc: khoá id nhỏ trước, bất kể chiều chuyển.

Dòng năm tách hai việc hay bị gộp. Khoá của nạn nhân được nhả ngay lúc server huỷ transaction: bên kia khoá đủ hai hàng trong khi nạn nhân chưa gửi gì. Nhưng kết nối của nạn nhân nhận `25P02` cho tới khi có `ROLLBACK`, nên `in_tx` vẫn phải `ROLLBACK`. Vòng thử lại coi `40P01` là lỗi tạm thời và mở transaction mới.

Dòng bốn không có `Barrier`, không retry. «0 deadlock» chỉ nói lần chạy đó không gặp; cái chứng minh là lập luận ở Hình 3: thứ tự toàn phần không tạo được vòng chờ.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `examples/toi-noi-rust/infra` (lab thử trên PostgreSQL 18.6). Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/53-aggregate-concurrency/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/53-aggregate-concurrency/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/53-aggregate-concurrency/Cargo.toml -- --measure
```

`--verify` chạy các thí nghiệm ở trên và thêm `VERIFY OK` khi mọi kiểm tra qua; `cargo test` chạy 9 unit test và 13 test tích hợp. Mỗi thí nghiệm có schema `lab_<uuid>` riêng, dọn khi xong. Không kiểm tra nào assert thời gian. `--measure` tách riêng vì số liệu đổi mỗi lần chạy.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Hạn mức chưa có chu kỳ (đóng kỳ, đặt lại <code>withdrawn</code> chưa làm). <code>bank:payout:clearing</code> được cộng dồn ở cuối transaction nhưng vẫn là hàng nóng mà mọi lần rút phải tuần tự qua khi commit; production thường tách hoặc gộp theo lô. Key trùng là lỗi, không phải replay. <code>Barrier</code> chỉ là cách dàn lịch của lab, và pool không được nhỏ hơn số task đứng ở một barrier. <code>Account::restore</code> từ chối số dư âm nên aggregate này chỉ hợp với ví đã nạp, không hợp với <code>customer:&lt;id&gt;</code> đang làm tài khoản biên của bài 41. Chỉ thử READ COMMITTED.</p>
</div>

**Bài tập:** đổi mỗi lần rút thành 20.000 trong thí nghiệm tám task optimistic (hạn mức vẫn 100.000, ví vẫn 150.000). Có bao nhiêu task thành công, `RetryPolicy::lab()` (6 lần) có đủ không, và số dư cuối là bao nhiêu?

<details><summary>Xem lời giải</summary><p>Hạn mức bao gồm biên, nên 5 × 20.000 = 100.000 được rút: 5 task thành công, 3 bị <code>LimitExceeded</code>. Có năm commit nên tối đa 6 lần thử; 6 vừa đủ (chạy thử ra như <code>[1, 2, 3, 4, 5, 6, 6, 6]</code>, không task nào hết lượt). Số dư cuối 50.000 VND, đã rút 100.000 VND, version 6. <code>max_attempts</code> nhỏ hơn 6 thì task chậm chân có thể nhận <code>RetriesExhausted</code>.</p></details>

Bài trước: [bài 52](/articles/rust-command-handler-pipeline-domain-event-khong-framework). Bài tiếp: [bài 54](/articles/rust-use-case-thanh-toan-chuyen-end-to-end).

## Tài liệu gốc

- [PostgreSQL: Explicit Locking, Row-Level Locks](https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-ROWS) và [Deadlocks](https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-DEADLOCKS).
- [PostgreSQL: SELECT, The Locking Clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE) (`NOWAIT`, `SKIP LOCKED`).
- [PostgreSQL: Transaction Isolation, Read Committed](https://www.postgresql.org/docs/current/transaction-iso.html#XACT-READ-COMMITTED).
- [PostgreSQL: mã lỗi SQLSTATE](https://www.postgresql.org/docs/current/errcodes-appendix.html) (`40001`, `40P01`, `55P03`).
- [docs.rs: sqlx, `Transaction`](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html) và [docs.rs: tokio, `Barrier`](https://docs.rs/tokio/latest/tokio/sync/struct.Barrier.html).
- [std: `AsyncFnOnce`](https://doc.rust-lang.org/std/ops/trait.AsyncFnOnce.html), trait của closure async mà `in_tx` nhận.

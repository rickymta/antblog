---
slug: rust-hold-capture-void-so-du-kha-dung
title: "Hold, capture, void: giữ tiền trong ví và số dư khả dụng"
summary: "Bài 42: hold chỉ giảm số dư khả dụng, không ghi bút toán; capture toàn phần và một phần, void, hai hold đồng thời với SELECT FOR UPDATE, job hết hạn với SKIP LOCKED và capture lặp lại idempotent."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, postgresql, hold, concurrency]
language: vi
---

Khách `CU-001` có 40.000 VND trong ví. Khi chuyến `TN-0001` bắt đầu, app cần chắc rằng ví đủ trả cước ước tính 25.500 VND, nhưng chuyến chưa xong nên chưa được ghi thu tiền. Nếu khách mở thêm chuyến trong lúc đó, ví phải trả lời được «còn bao nhiêu có thể tiêu».

Đây là bài 42 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 14 đã dựng [máy trạng thái hold → capture → settle](/articles/rust-typestate-may-trang-thai-thanh-toan) trong bộ nhớ; bài 24 đã cho thấy [hai giao dịch chồng nhau phá hạn mức](/articles/rust-transaction-isolation-append-only-idempotency-key). Bài này đặt chúng lên PostgreSQL thật, chỉ áp dụng cho hold trong ví do sổ cái Tới Nơi quản lý.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ví có 40.000 VND và một hold ACTIVE 25.500 VND. Số dư trong sổ cái là bao nhiêu, khách còn tiêu được bao nhiêu, và hold đã thêm bao nhiêu bút toán?</p>
<details><summary>Xem đáp án</summary><p>Số dư sổ cái vẫn 40.000 VND, khả dụng 14.500 VND, hold thêm 0 bút toán. Hold là một lời hứa «đừng cho tiêu phần này», không phải tiền đã rời ví.</p></details>
</div>

## Hold có phải là một bút toán không?

Không. Bút toán ghi tiền đã chuyển. Hold chỉ giữ chỗ một phần số dư và có thể nhả mà không để dấu vết trong sổ. Ghi hold thành bút toán thì mỗi chuyến bị huỷ cần thêm một bút toán đảo để xoá thứ chưa từng xảy ra.

Vì vậy hold nằm ở một bảng riêng, và số dư khả dụng là con số **tính ra**, không lưu:

```text
available = balance − SUM(phần còn lại của hold ACTIVE)
phần còn lại = amount_dong − captured_dong
```

```sql
-- sql/schema.sql (rút gọn: bỏ chú thích và index)
CREATE TABLE holds (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id      text NOT NULL REFERENCES accounts (id),
    amount_dong     bigint NOT NULL CHECK (amount_dong > 0),
    captured_dong   bigint NOT NULL DEFAULT 0,
    status          text NOT NULL CHECK (status IN ('ACTIVE', 'CAPTURED', 'VOIDED', 'EXPIRED')),
    idempotency_key text NOT NULL UNIQUE,
    expires_at      timestamptz NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    CHECK (captured_dong >= 0 AND captured_dong <= amount_dong)
);
```

`balance` là cột `account_balances.balance_dong` của [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition), tức projection của `postings`. Hàm `funds` tính cả hai vế trong **một** câu lệnh, để hai con số đến từ cùng một snapshot:

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
let row = sqlx::query(
    "SELECT b.balance_dong, COALESCE((SELECT SUM(h.amount_dong - h.captured_dong) \
     FROM holds h WHERE h.account_id = b.account_id AND h.status = 'ACTIVE'), 0)::bigint AS held \
     FROM account_balances b WHERE b.account_id = $1",
)
.bind(account)
.fetch_optional(pool)
.await?
.ok_or_else(|| HoldError::NoBalanceRow(account.to_string()))?;
```

Thí nghiệm đầu nạp ví 40.000 VND bằng bút toán fixture `fixture:topup:customer:CU-001`: ví +40.000, `psp:vixanh:clearing` -40.000. Sổ dòng tiền đóng kín nên tiền từ ngoài vào hiện thành số âm ở tài khoản biên; clearing âm ở đây là đúng, theo cách đọc của [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen). Rồi tạo hold cho `TN-0001`:

```text
nạp ví (fixture)    : số dư 40.000 VND, giữ 0 VND, khả dụng 40.000 VND; 1 bút toán, 2 posting
clearing sau nạp ví : -40.000 VND
hold TN-0001 25.500 : Created { id: 1 }
sau hold            : số dư 40.000 VND, giữ 25.500 VND, khả dụng 14.500 VND; 1 bút toán, 2 posting
gửi lại cùng key    : Replayed { id: 1 }, giữ vẫn 25.500 VND
hold thêm 14.501    : Err(không đủ số dư khả dụng: còn 14.500 VND, cần 14.501 VND)
```

Số bút toán và posting không đổi. Gửi lại cùng `trip:TN-0001:hold` trả hold cũ thay vì giữ thêm một lần nữa, và ranh giới là 14.500: xin thêm 14.501 bị từ chối, đúng một đồng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Ba trạng thái của ví 40.000 VND. Khi đang giữ 25.500, khả dụng 14.500 và sổ cái chưa đổi. Sau capture một phần 12.000, 12.000 đã thu bằng bút toán, 13.500 phần dư được nhả nên khả dụng 28.000. Sau void, toàn bộ 40.000 khả dụng."><text x="40" y="28" class="t-b">Đang giữ cho TN-0001: sổ cái vẫn 40.000</text><rect x="40" y="38" width="510" height="44" rx="8" class="f-warn-soft s-warn"/><text x="54" y="66" class="t-small">Hold ACTIVE 25.500</text><rect x="550" y="38" width="290" height="44" rx="8" class="f-pk-soft s-pk"/><text x="564" y="66" class="t-small">Khả dụng 14.500</text><text x="40" y="118" class="t-b">Capture một phần 12.000: bút toán thật, hold đóng</text><rect x="40" y="128" width="240" height="44" rx="8" class="f-phys-soft s-phys"/><text x="54" y="156" class="t-small">Đã thu 12.000</text><rect x="280" y="128" width="560" height="44" rx="8" class="f-pk-soft s-pk"/><text x="294" y="156" class="t-small">Khả dụng 28.000 (gồm 13.500 vừa nhả)</text><text x="40" y="208" class="t-b">Void khi huỷ chuyến: không bút toán</text><rect x="40" y="218" width="800" height="44" rx="8" class="f-pk-soft s-pk"/><text x="54" y="246" class="t-small">Khả dụng 40.000</text></svg>
</div>
<figcaption>Hình 1. Chiều ngang là 40.000 VND. Chỉ capture làm sổ cái đổi; nhả phần dư và void chỉ đổi cách tính khả dụng.</figcaption>
</figure>

Hold khác authorization ở cổng thanh toán. `AUTHORIZED` ở bài 14 là PSP giữ hạn mức trên phương tiện thanh toán của khách, trong hệ thống và theo quy tắc của họ; Tới Nơi chỉ biết qua thông báo. Hold của bài này là phần ví mà **chính sổ cái** từ chối cho tiêu: ta tự quản lý con số `available`, tự quyết hạn và tự chịu trách nhiệm nếu để nó âm.

## Capture một phần và void: phần dư đi đâu?

Capture làm hai việc phải đi cùng nhau: ghi bút toán (`customer:CU-001` ra, `psp:vixanh:clearing` vào) và đóng hold. Bút toán commit mà hold vẫn ACTIVE thì phần tiền bị trừ hai lần trong `available`; hold đóng mà bút toán không ghi thì khách được tiêu lại số tiền chưa trả. Cả hai nằm trong một transaction, và hold bị khoá dòng trước:

```rust
// src/hold.rs (rút gọn: bỏ kiểm tra số tiền và chú thích)
pub async fn lock(conn: &mut PgConnection, id: i64) -> Result<Locked, HoldError> {
    let row = sqlx::query(
        "SELECT id, account_id, amount_dong, captured_dong, status, expires_at \
         FROM holds WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(HoldError::NotFound(id))?;
    let hold = HoldRow::from_row(&row)?;
    Ok(match hold.status {
        HoldStatus::Active => Locked::Active(ActiveHold { /* id, account, amount, expires_at */ }),
        _ => Locked::Closed(hold),
    })
}

impl ActiveHold {
    pub async fn capture(
        self,
        conn: &mut PgConnection,
        now: Timestamp,
        amount: Money,
        key: &str,
    ) -> Result<i64, HoldError> {
        // ... Expired nếu now >= expires_at; CaptureExceedsHold nếu amount > hold
        let entry = capture_entry(&self.account, amount, key)?;
        let Some(entry_id) = insert_entry(conn, &entry).await? else {
            return Err(HoldError::IdempotencyConflict { key: key.to_string() });
        };
        sqlx::query("UPDATE holds SET status = 'CAPTURED', captured_dong = $2 WHERE id = $1")
            .bind(self.id)
            .bind(amount.dong())
            .execute(&mut *conn)
            .await?;
        Ok(entry_id)
    }
}
```

`insert_entry` cũng cộng dồn `account_balances` trong cùng transaction, như bài 25. Kết quả của capture toàn phần (bỏ hai dòng):

```text
trước capture       : số dư 40.000 VND, giữ 25.500 VND, khả dụng 14.500 VND
capture 25.500      : Captured { entry_id: 2 }
sau capture         : số dư 14.500 VND, giữ 0 VND, khả dụng 14.500 VND; 2 bút toán, 4 posting
clearing sau capture: -14.500 VND
capture lặp, cùng key: Replayed { entry_id: 2 }; vẫn 2 bút toán
cùng key, số khác   : Err(idempotency_key trip:TN-0001:capture đã dùng cho nội dung khác)
void sau capture    : Err(hold 1 đang ở trạng thái CAPTURED)
```

Khả dụng trước và sau capture đều là 14.500: số dư giảm 25.500 đúng lúc hold thôi bị tính. Hai bước cùng một transaction nên không ai quan sát được trạng thái ở giữa. Gọi lại với cùng key và cùng số tiền là replay; cùng key khác số tiền là xung đột, giống bài 24.

Capture một phần dành cho chuyến dừng sớm. Hold 25.500, chuyến chỉ chạy 2 km nên cước còn 12.000:

```text
capture 12.000      : Captured { entry_id: 2 }
sau capture         : số dư 28.000 VND, giữ 0 VND, khả dụng 28.000 VND
hold                : CAPTURED, giữ 25.500 VND, captured 12.000 VND, nhả 13.500 VND
capture 25.501/25.500: Err(capture 25.501 VND vượt số đã giữ 25.500 VND)
```

Phần dư 13.500 không cần bút toán nào: hold không còn ACTIVE nên không còn trong `SUM`. Lab chỉ cho **một** lần capture mỗi hold; nhiều lần cần thêm quy tắc, và công thức `amount − captured` đã chừa chỗ. Void khi huỷ chuyến đơn giản hơn: hold sang `VOIDED`, không bút toán, khả dụng về lại 40.000. Void lần hai trả `AlreadyVoided`.

## Hai hold cùng lúc: vì sao thiếu khoá là sai?

`create_hold` đọc số dư, trừ các hold ACTIVE, rồi chèn nếu đủ: kiểm-rồi-ghi, cùng dạng với hạn mức rút ở bài 24. Hai chuyến `TN-0001` và `TN-0002` của cùng khách, mỗi chuyến 25.500, cùng đến lúc ví có 40.000: mỗi bên nhìn riêng đều đúng, cộng lại là 51.000.

Lab dựng race condition bằng `tokio::sync::Barrier`, không bằng may rủi. Với bản không khoá, cả hai task phải kiểm xong mới được cùng ghi:

```rust
// src/scenarios.rs (rút gọn: một task của race)
start.wait().await; // hai task xuất phát cùng lúc
let gate = match mode {
    LockMode::None => Gate::Barrier(&after_check), // dừng giữa «kiểm» và «ghi»
    LockMode::ForUpdate => Gate::Open,
};
create_hold(&pool, &*clock, &req(CU1, fare(), &key), mode, gate).await
```

Phần khoá nằm ở bước đầu của `create_hold`:

```rust
// src/store.rs (rút gọn: bỏ replay theo key và chú thích)
let balance_sql = match mode {
    LockMode::ForUpdate => {
        "SELECT balance_dong FROM account_balances WHERE account_id = $1 FOR UPDATE"
    }
    LockMode::None => "SELECT balance_dong FROM account_balances WHERE account_id = $1",
};
let balance: i64 = sqlx::query(balance_sql)
    .bind(req.account)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| HoldError::NoBalanceRow(req.account.to_string()))?
    .get(0);
let held: i64 = sqlx::query(
    "SELECT COALESCE(SUM(amount_dong - captured_dong), 0)::bigint FROM holds \
     WHERE account_id = $1 AND status = 'ACTIVE'",
)
.bind(req.account)
.fetch_one(&mut *tx)
.await?
.get(0);
let available = balance - held;
if req.amount.dong() > available {
    tx.rollback().await?;
    return Err(HoldError::InsufficientAvailable { available: Money::from_dong(available), requested: req.amount });
}
// INSERT INTO holds ... RETURNING id, rồi tx.commit()
```

Kết quả hai bản:

```text
không khóa : 2 tạo được, 0 bị từ chối; số dư 40.000 VND, giữ 51.000 VND, khả dụng -11.000 VND
FOR UPDATE : 1 tạo được, 1 bị từ chối; số dư 40.000 VND, giữ 25.500 VND, khả dụng 14.500 VND
```

Bản `FOR UPDATE` cho đúng một bên thắng; lab kiểm bất biến, không kiểm bên nào thắng.

Ba chi tiết khiến khoá này đúng. Một là khoá **hàng `account_balances`**, không khoá bảng `holds`: mọi thứ làm đổi số dư (capture, rút tiền) cũng cập nhật chính hàng đó. Hai là câu `SUM` chạy **sau** khi giữ khoá; ở READ COMMITTED mỗi câu lệnh có snapshot riêng nên nó thấy hold đã commit của bên đi trước. Tính `SUM` rồi mới khoá là quay lại bản sai, và gộp khoá với `SUM` vào một câu cũng sai: snapshot lấy lúc câu lệnh bắt đầu, chờ khoá không làm mới nó, và PostgreSQL chỉ xét lại chính hàng bị khoá. Lab cho B chạy câu gộp trong khi A giữ khoá, rồi A commit hold 25.500:

```text
một câu, sau khi chờ khóa: SUM thấy 0 VND
câu SUM riêng sau đó     : SUM thấy 25.500 VND
```

Ba là hàng phải tồn tại. `FOR UPDATE` trên hàng chưa có trả 0 dòng và không khoá gì, hai phiên cùng đi qua. Vì thế schema lab tạo hàng số dư 0 khi mở tài khoản (`upsert` của bài 25 thì chỉ tạo ở posting đầu), và `create_hold` gặp ví chưa có hàng trả `NoBalanceRow`, không đoán số dư là 0:

```text
hai phiên FOR UPDATE NOWAIT: 0 hàng và 0 hàng, không ai bị chặn
hold khi chưa có hàng: Err(ví customer:CU-004 chưa có hàng số dư)
hold sau nạp ví      : Created { id: 1 }
```

Để chắc bên thứ hai thật sự **đứng chờ** chứ không may mắn đến sau, lab cho bên A giữ khoá và dừng, rồi đọc `pg_stat_activity`:

```text
B đang chờ khóa của A (wait_event_type = Lock): true
A: Created { id: 1 }
B: Err(không đủ số dư khả dụng: còn 14.500 VND, cần 25.500 VND)
```

Giá phải trả: mọi hold của cùng một ví xếp hàng, nên ta không gọi mạng bên trong transaction này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Hai hold 25.500 trên ví 40.000. Bên trái, không khoá: cả hai phiên cùng tính khả dụng 40.000, cùng chèn hold, tổng giữ 51.000 vượt số dư. Bên phải, FOR UPDATE: phiên thứ nhất khoá hàng số dư, kiểm, chèn, commit; phiên thứ hai chờ khoá rồi thấy khả dụng 14.500 và bị từ chối."><text x="20" y="24" class="t-b">Không khoá</text><rect x="20" y="36" width="400" height="48" rx="10" class="f-phys-soft s-phys"/><text x="34" y="66" class="t-small">T1 · 40.000 − 0 = 40.000, đủ</text><rect x="20" y="106" width="400" height="48" rx="10" class="f-phys-soft s-phys"/><text x="34" y="136" class="t-small">T2 · 40.000 − 0 = 40.000, đủ</text><rect x="20" y="176" width="400" height="48" rx="10" class="f-phys-soft s-phys"/><text x="34" y="206" class="t-small">cả hai INSERT hold, cả hai COMMIT</text><rect x="20" y="246" width="400" height="64" rx="10" class="f-warn-soft s-warn"/><text x="34" y="274" class="t-b">Giữ 51.000 &gt; số dư 40.000</text><text x="34" y="296" class="t-small">khả dụng −11.000</text><path d="M220 84 L220 106" class="f-none s-phys" stroke-width="2"/><path d="M220 154 L220 176" class="f-none s-phys" stroke-width="2"/><path d="M220 224 L220 246" class="f-none s-warn" stroke-width="2"/><text x="460" y="24" class="t-b">SELECT ... FOR UPDATE trên account_balances</text><rect x="460" y="36" width="400" height="48" rx="10" class="f-pk-soft s-pk"/><text x="474" y="66" class="t-small">T1 · khoá hàng số dư, kiểm, INSERT, COMMIT</text><rect x="460" y="106" width="400" height="48" rx="10" class="f-warn-soft s-warn dash"/><text x="474" y="136" class="t-small">T2 · chờ khoá (wait_event_type = Lock)</text><rect x="460" y="176" width="400" height="48" rx="10" class="f-phys-soft s-phys"/><text x="474" y="206" class="t-small">T2 · khoá xong, SUM thấy hold của T1</text><rect x="460" y="246" width="400" height="64" rx="10" class="f-pk-soft s-pk"/><text x="474" y="274" class="t-b">Còn 14.500 &lt; 25.500: từ chối</text><text x="474" y="296" class="t-small">tổng giữ 25.500, khả dụng 14.500</text><path d="M660 84 L660 106" class="f-none s-pk" stroke-width="2"/><path d="M660 154 L660 176" class="f-none s-phys" stroke-width="2"/><path d="M660 224 L660 246" class="f-none s-pk" stroke-width="2"/></svg>
</div>
<figcaption>Hình 2. Cùng hai yêu cầu, khác nhau ở một mệnh đề <code>FOR UPDATE</code>.</figcaption>
</figure>

## Job hết hạn: clock truyền vào và SKIP LOCKED

Chuyến bị bỏ rơi thì hold phải tự nhả. Một job định kỳ chuyển hold ACTIVE đã quá `expires_at` sang `EXPIRED`. Job không gọi `now()` của máy hay của PostgreSQL mà nhận thời điểm từ `Clock` truyền vào, nên test dựng thời gian bằng `ManualClock::advance` thay vì chờ 15 phút:

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
pub async fn expire_due(pool: &PgPool, clock: &impl Clock, batch: i64) -> Result<Vec<i64>, HoldError> {
    let rows = sqlx::query(
        "WITH due AS (SELECT id FROM holds WHERE status = 'ACTIVE' AND expires_at <= $1 \
         ORDER BY expires_at, id LIMIT $2 FOR UPDATE SKIP LOCKED) \
         UPDATE holds h SET status = 'EXPIRED' FROM due WHERE h.id = due.id RETURNING h.id",
    )
    .bind(to_pg(clock.now())?)
    .bind(batch)
    .fetch_all(pool)
    .await?;
    let mut ids: Vec<i64> = rows.iter().map(|r| r.get("id")).collect();
    ids.sort_unstable();
    Ok(ids)
}
```

`SKIP LOCKED` giải quyết va chạm với capture đang chạy: hold 1 đã tới hạn theo đồng hồ của job, nhưng một capture bắt đầu trước hạn đang giữ khoá dòng của nó. Nếu job chờ khoá, cả lô đứng yên chỉ vì một dòng. `SKIP LOCKED` bỏ qua dòng đó và xử lý những dòng còn lại:

```text
3 hold đang ACTIVE  : id [1, 2, 3]
FOR UPDATE NOWAIT   : SQLSTATE 55P03 (dòng đang bị giữ)
job lần 1           : hết hạn id [2, 3]; hold 1 vẫn ACTIVE
job lần 2           : hết hạn id []
trạng thái cuối     : ["CAPTURED", "EXPIRED", "EXPIRED"] (capture ghi bút toán 4)
CU-002 sau hết hạn  : số dư 40.000 VND, giữ 0 VND, khả dụng 40.000 VND
```

Dòng `NOWAIT` là bằng chứng dòng 1 đang bị khoá (mã 55P03). Lần chạy đầu bỏ qua hold 1, capture hoàn tất rồi hold 1 thành `CAPTURED`, lần chạy sau không còn gì để làm. Nếu capture bị rollback, hold 1 vẫn ACTIVE và lần chạy sau nhặt nó. Cái giá: một lần chạy không thấy trọn tập hold, hợp lý với job chạy lặp lại, nhưng không phù hợp khi cần ảnh chụp đầy đủ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 3. Vòng đời của hold. Từ ACTIVE có ba lối ra: capture khi đồng hồ nhỏ hơn expires_at sang CAPTURED, void khi huỷ chuyến sang VOIDED, job khi expires_at nhỏ hơn hoặc bằng đồng hồ sang EXPIRED. Cả ba đều giữ khoá dòng và thấy status ACTIVE rồi mới đổi trạng thái nên chỉ một lối thắng."><defs><marker id="rust-hold-capture-void-so-du-kha-dung-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="40" y="75" width="200" height="64" rx="10" class="f-pk-soft s-pk"/><text x="56" y="102" class="t-b">ACTIVE</text><text x="56" y="124" class="t-small">đang trừ khả dụng</text><rect x="560" y="15" width="290" height="58" rx="10" class="f-phys-soft s-phys"/><text x="576" y="40" class="t-b">CAPTURED</text><text x="576" y="60" class="t-small">có bút toán, phần dư nhả</text><rect x="560" y="82" width="290" height="58" rx="10" class="f-phys-soft s-phys"/><text x="576" y="107" class="t-b">VOIDED</text><text x="576" y="127" class="t-small">không bút toán</text><rect x="560" y="149" width="290" height="58" rx="10" class="f-phys-soft s-phys"/><text x="576" y="174" class="t-b">EXPIRED</text><text x="576" y="194" class="t-small">không bút toán</text><path d="M240 92 L558 46" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hold-capture-void-so-du-kha-dung-0-a)"/><text x="400" y="56" text-anchor="middle" class="t-small">capture · now &lt; expires_at</text><path d="M240 107 L558 111" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hold-capture-void-so-du-kha-dung-0-a)"/><text x="400" y="101" text-anchor="middle" class="t-small">void · huỷ chuyến</text><path d="M240 122 L558 178" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hold-capture-void-so-du-kha-dung-0-a)"/><text x="400" y="172" text-anchor="middle" class="t-small">job · expires_at ≤ now</text><text x="40" y="250" class="t-small">Mỗi mũi tên là UPDATE dưới khoá dòng; sau mũi tên đầu tiên, hold không còn ACTIVE nên các mũi tên còn lại bị từ chối.</text></svg>
</div>
<figcaption>Hình 3. Ba lối ra, không lối quay lại. Ranh giới hạn dùng hai vế đối nhau, cùng so với một thời điểm.</figcaption>
</figure>

## Capture đến sau hạn thì xử lý thế nào?

Sự kiện báo chuyến xong có thể đến trễ vì hàng đợi hay retry, cùng kiểu trễ với webhook ở [bài 38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung), nên capture đến sau hạn là chuyện thường. Quy ước của lab: capture bị từ chối nếu `now >= expires_at`, kể cả khi job chưa chạy. Hai vế của ranh giới đối nhau nên không có khoảng hở và không có khoảng chồng:

```text
t0+14:59, hold 2    : Captured { entry_id: 3 }
t0+15:00, hold 1    : Err(hold 1 đã quá hạn lúc 2026-11-06T08:15:00Z)
job chưa chạy       : số dư 40.000 VND, giữ 25.500 VND, khả dụng 14.500 VND
job chạy            : hết hạn id [1]
sau job             : số dư 40.000 VND, giữ 0 VND, khả dụng 40.000 VND
capture sau job     : Err(hold 1 đang ở trạng thái EXPIRED)
bút toán            : 3 (2 nạp ví + 1 capture của hold 2)
```

Giữa «quá hạn» và «job chạy», hold vẫn trừ khả dụng. Đó là lựa chọn nghiêng về phía an toàn: ví có thể bị coi là thiếu tiền thêm một chu kỳ job, không bao giờ bị coi là thừa.

Từ chối chưa phải là xong: chuyến đã chạy mà chưa thu được tiền. Hướng xử lý là quyết định nghiệp vụ: hold mới nếu ví còn đủ, thời gian ân hạn, hoặc ghi nhận công nợ. Lab chỉ chứng minh phần kỹ thuật: capture trễ không ghi gì.

## Typestate đến đâu, enum ở đâu?

Bài 14 đặt trạng thái vào kiểu `Payment<Authorized>`. Với hold, cách đó đi được một đoạn ngắn. Trạng thái thật nằm trong cột `status`, nhiều tiến trình cùng đổi, và hai tiến trình có thể cùng cầm một `Hold<Active>` trong bộ nhớ riêng mà kiểu không biết bên kia đã void. Khoá dòng và trạng thái đọc dưới khoá quyết định bên nào thắng.

Lab dùng hai mức. Ở **ranh giới với DB** là `enum HoldStatus`, đọc từ chuỗi bằng `FromStr` và từ chối chuỗi lạ:

```rust
// src/hold.rs (rút gọn)
impl FromStr for HoldStatus {
    type Err = HoldError;

    fn from_str(raw: &str) -> Result<HoldStatus, HoldError> {
        match raw {
            "ACTIVE" => Ok(HoldStatus::Active),
            "CAPTURED" => Ok(HoldStatus::Captured),
            "VOIDED" => Ok(HoldStatus::Voided),
            "EXPIRED" => Ok(HoldStatus::Expired),
            other => Err(HoldError::UnknownStatus(other.to_string())),
        }
    }
}
```

**Trong một transaction**, typestate có ích ở dạng nhỏ: `ActiveHold` là bằng chứng «hold này ACTIVE và transaction hiện tại giữ khoá dòng». Chỉ `hold::lock` tạo được nó vì trường là private:

```text
error[E0451]: fields `id`, `account`, `amount` and `expires_at` of struct `ActiveHold` are private
```

`capture` và `void` nhận `self`, nên bằng chứng chỉ dùng được một lần. Dùng lại là lỗi biên dịch:

```text
error[E0382]: use of moved value: `active`
8 |     active.capture(&mut tx, now, Money::from_dong(25_500), "trip:TN-0001:capture").await?;
  |     ------ value moved here
9 |     active.void(&mut tx).await?; // dùng lại bằng chứng đã tiêu
  |     ^^^^^^ value used here after move
```

Hai lỗi này tạo thật bằng ví dụ riêng, đã gỡ khỏi lab. Kiểu này chặn được việc capture khi chưa kiểm hold ACTIVE, và việc capture hai lần trên cùng một bằng chứng. Nó không chặn được tiến trình khác đổi trạng thái giữa hai transaction; đó vẫn là việc của khoá và DB. Nó cũng không buộc bằng chứng vào đúng transaction đã khoá: truyền kết nối khác vẫn biên dịch, muốn chặt hơn thì `ActiveHold` phải mượn transaction. Job hết hạn cũng không dùng `ActiveHold`: nó là một câu `UPDATE` theo lô.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chạy trên một PostgreSQL local với vài phiên, chưa đo hiệu năng. Hàng số dư của clearing là điểm nóng: mọi capture đều cập nhật nó. Khoá hàng số dư chỉ đủ khi <strong>mọi</strong> thứ làm giảm số dư (rút tiền, payout, phí) cũng đi qua kiểm tra khả dụng: một đường rút bỏ qua <code>holds</code> phá bất biến mà không báo lỗi. Lab chưa gia hạn hold, chưa ân hạn sau hạn; số dư đầu là một bút toán fixture.</p></div>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `examples/toi-noi-rust/infra` (bài này không dùng Redis hay Kafka). Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/42-hold-capture-void/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/42-hold-capture-void/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/42-hold-capture-void/Cargo.toml
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng, xoá khi xong. Các kiểm tra luôn chạy; `--verify` thêm dòng `VERIFY OK`. Không kiểm tra nào assert thời gian chạy.

Bút toán capture là bước (1) của dòng tiền chuẩn: `customer:CU-001` -25.500, `psp:vixanh:clearing` +25.500. Clearing đi từ -40.000 lên -14.500, vẫn âm vì 14.500 còn nằm trong ví. Bước (2), chia cho nền tảng và tài xế, lab này chưa làm.

**Bài tập:** chuyến `TN-0003` của khách `CU-002` có hold 25.500 với hạn 15 phút. Capture đến lúc đúng `expires_at` và job cũng vừa chạy với cùng đồng hồ. Bên nào thắng, và vì sao hai bên không thể cùng thắng?

<details><summary>Xem lời giải</summary><p>Job thắng; capture bị từ chối vì <code>now &gt;= expires_at</code>. Hai điều kiện là hai vế đối nhau, nên với cùng một đồng hồ chỉ một vế đúng. Nếu hai đồng hồ lệch nhau và cả hai qua kiểm tra, khoá dòng quyết định: job khoá trước thì capture thấy <code>EXPIRED</code>; capture khoá trước thì job bỏ qua dòng đó.</p></details>

Bài trước: [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen). Bài tiếp: [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker).

## Tài liệu gốc

- [PostgreSQL: Explicit Locking, Row-Level Locks](https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-ROWS) và [SELECT, The Locking Clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE).
- [PostgreSQL: Transaction Isolation, Read Committed](https://www.postgresql.org/docs/current/transaction-iso.html#XACT-READ-COMMITTED).
- [PostgreSQL: pg_stat_activity](https://www.postgresql.org/docs/current/monitoring-stats.html) và [mã lỗi SQLSTATE](https://www.postgresql.org/docs/current/errcodes-appendix.html).
- [docs.rs: tokio, `Barrier`](https://docs.rs/tokio/latest/tokio/sync/struct.Barrier.html) và [`Notify`](https://docs.rs/tokio/latest/tokio/sync/struct.Notify.html).
- [docs.rs: jiff, `Timestamp`](https://docs.rs/jiff/latest/jiff/struct.Timestamp.html).
- [The Rust Reference: Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).

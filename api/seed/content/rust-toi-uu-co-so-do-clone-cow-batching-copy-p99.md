---
slug: rust-toi-uu-co-so-do-clone-cow-batching-copy-p99
title: "Tối ưu có số đo: tránh clone, Cow, batching và COPY"
summary: "Bài 61: đặt mục tiêu trước rồi đo. Cow giảm 10.000 cấp phát còn 500, mượn thay clone, gộp lô giảm 8.991 round trip còn 10, COPY và trigger, prepared statement, p99 với coordinated omission, khi nào dừng."
category: programming
level: advanced
tags: [rust, toi-noi, ledger, performance, cow, copy]
language: vi
---

Bài 60 dạy tìm chỗ chậm bằng profiler. Tìm ra rồi thì sửa gì, sửa tới đâu, bằng chứng nào cho thấy đã đủ? Bài này nhập lô bút toán vào PostgreSQL bằng bốn cách, giữ nguyên idempotency và `account_balances`, và ưu tiên con số không đổi theo tải máy.

Đây là bài 61 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dùng `ledger-core` ([bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace)), schema và trigger của [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition), cách đếm cấp phát của [bài 59](/articles/rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet), và đối chiếu với `unnest` của [bài 30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hàm <code>normalize</code> trả <code>Cow&lt;str&gt;</code> chạy trên 10.000 mã tài khoản; 95% đã đúng dạng chuẩn hoặc chỉ thừa khoảng trắng. Có bao nhiêu lần cấp phát?</p>
<details><summary>Xem đáp án</summary><p>500: chỉ 5% mã phải đổi chữ hoa. Mục 2 có số đo.</p></details>
</div>

## Đặt mục tiêu trước: đo cái gì, trên máy nào?

Mục tiêu viết trước khi sửa (giả định, không phải SLA): nhập lô tối thiểu 10.000 bút toán/giây, có idempotency và số dư; số cấp phát không tăng theo số dòng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 1. Thứ tự làm việc: đặt mục tiêu trước, đo bằng chỉ số không phụ thuộc tải, rồi mới đo thời gian kèm load average, cuối cùng quyết định giữ hay bỏ thay đổi."><defs><marker id="rust-toi-uu-co-so-do-clone-cow-batching-copy-p99-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">1. Mục tiêu trước</text><text x="34" y="88" class="t-small">viết ra số cần đạt</text><text x="34" y="108" class="t-small">và điều kiện đo</text><path d="M214 85 L243 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-toi-uu-co-so-do-clone-cow-batching-copy-p99-0-a)"/><rect x="249" y="30" width="190" height="110" rx="10" class="f-pk-soft s-pk"/><text x="263" y="62" class="t-b">2. Số không đổi theo tải</text><text x="263" y="88" class="t-small">cấp phát, round trip,</text><text x="263" y="108" class="t-small">message, byte, số dòng</text><path d="M443 85 L472 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-toi-uu-co-so-do-clone-cow-batching-copy-p99-0-a)"/><rect x="478" y="30" width="190" height="110" rx="10" class="f-warn-soft s-warn"/><text x="492" y="62" class="t-b">3. Thời gian + load</text><text x="492" y="88" class="t-small">ghi kèm load average,</text><text x="492" y="108" class="t-small">kết luận ở mức hướng</text><path d="M672 85 L701 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-toi-uu-co-so-do-clone-cow-batching-copy-p99-0-a)"/><rect x="707" y="30" width="153" height="110" rx="10" class="f-phys-soft s-phys"/><text x="721" y="62" class="t-b">4. Giữ hay bỏ</text><text x="721" y="88" class="t-small">đạt mục tiêu thì</text><text x="721" y="108" class="t-small">dừng</text><text x="20" y="178" class="t-small">Bước 2 chạy lại ra đúng số cũ trên mọi máy; bước 3 thì không.</text></svg>
</div>
<figcaption>Hình 1. Thứ tự làm việc, từ mục tiêu tới quyết định giữ hay bỏ.</figcaption>
</figure>

Máy là Apple M1 Max, 64 GB, rustc 1.98.1, PostgreSQL 18.6 trong Docker Desktop (Engine 29.7.2), dùng chung với việc khác: load average lúc đo từ 100 đến 440. Nên mỗi câu hỏi được trả lời trước bằng số đếm: cấp phát và byte (allocator bọc như bài 59, đếm theo thread), round trip và `Parse` (proxy TCP đọc message PostgreSQL), số lần trigger chạy (`pg_stat_user_functions`). Thông lượng, p50, p99 đến sau cùng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Số đo trên máy viết bài, không phải SLA</span>
<p>Số thời gian đo lúc máy tải cao, client và server cùng máy, không TLS: chỉ cho biết hướng. Số đếm chạy lại ra đúng số cũ và <code>--verify</code> kiểm chúng.</p>
</div>

## Cow và mượn: bớt cấp phát ở đâu?

Mã tài khoản từ bên ngoài đôi khi thừa khoảng trắng hoặc viết hoa loại tài khoản. `normalize_clone` luôn tạo `String`; `normalize` trả `Cow`, chỉ cấp phát khi phải đổi byte.

```rust
// src/account.rs
pub fn normalize(raw: &str) -> Option<Cow<'_, str>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return None; }
    let kind_end = trimmed.find(':').unwrap_or(trimmed.len());
    if trimmed.as_bytes()[..kind_end].iter().any(u8::is_ascii_uppercase) {
        let mut owned = trimmed.to_string();
        owned[..kind_end].make_ascii_lowercase();
        Some(Cow::Owned(owned))
    } else {
        Some(Cow::Borrowed(trimmed)) // đã chuẩn, hoặc chỉ thừa khoảng trắng
    }
}
```

Dữ liệu giả có 10.000 mã: 90% đã chuẩn, 5% chỉ thừa khoảng trắng, 5% thừa khoảng trắng và viết hoa:

| Cách | Cấp phát | Byte |
| --- | ---: | ---: |
| `normalize_clone` | 10.000 | 146.671 |
| `normalize` (`Cow`) | 500 | 7.335 |
| `AccountId::new` của `ledger-core` | 10.000 | 146.671 |

Dòng cuối là giới hạn của `Cow`: `AccountId` sở hữu một `String`, nên đi vào kiểu miền là cấp phát lại. `Cow` chỉ thắng ở đoạn chỉ đọc, như bind `&[&str]` lên PostgreSQL: 10.000 mã thô chỉ khớp 9.000 tài khoản, sau `normalize` khớp đủ.

Cùng ý đó cho các cột của `unnest`: bài 30 sao chép từng chuỗi, bản mượn chỉ giữ `&str` trỏ vào `JournalEntry`:

```rust
// src/columns.rs (rút gọn)
pub struct Columns<'a> {
    pub keys: Vec<&'a str>,
    pub key_of_line: Vec<&'a str>,
    pub line_nos: Vec<i32>,
    pub accounts: Vec<&'a str>,
    pub amounts: Vec<i64>,
}
```

Với 10.000 bút toán (30.000 dòng), mượn tốn 5 cấp phát, 1.480.000 byte; sao chép tốn 70.005 cấp phát cộng 64 lần cấp phát lại, 5.958.270 byte. Cái giá là vòng đời: bút toán phải sống lâu hơn các cột, và trình biên dịch buộc điều đó.

```text
error[E0505]: cannot move out of `entries` because it is borrowed
 --> examples/e0505.rs:8:10
7 |     let cols = Columns::borrow(&refs);
8 |     drop(entries);
  |          ^^^^^^^ move out of `entries` occurs here
9 |     println!("{}", cols.keys.len());
  |                    --------- borrow later used here
```

(Lỗi thật, rút gọn, đã bỏ khỏi lab.)

## Từng dòng, unnest hay COPY: bao nhiêu round trip?

Bốn cách ghi cùng dữ liệu, cùng bảo đảm: key trùng bị bỏ qua (`ON CONFLICT DO NOTHING`), `account_balances` cập nhật cùng transaction và khớp tổng posting. `PerRow` ghi mỗi bút toán một transaction, mỗi posting một `INSERT` và một upsert số dư: 9 câu lệnh. `Unnest` ghi mỗi lô 3 câu lệnh mảng, cộng `BEGIN` và `COMMIT`. `Copy` đổi riêng bước posting sang `copy_in_raw`. `Cte` gom cả lô vào một câu lệnh ba CTE.

```rust
// src/strategies.rs (rút gọn): Unnest và Copy cùng khung, khác bước posting
let mut tx = conn.begin().await?;                                  // 1. BEGIN
let fresh: Vec<(i64, String)> = sqlx::query_as(INSERT_ENTRIES_BATCH) // 2. bút toán mới
    .bind(&keys[..]).fetch_all(&mut *tx).await?;                   //    ... ON CONFLICT DO NOTHING RETURNING
let rows = NewRows::build(chunk, &mut ids);   // chỉ posting của bút toán mới, số dư đã gộp theo tài khoản
if use_copy {
    let mut copy = tx.copy_in_raw(COPY_POSTINGS).await?;           // 3. posting qua COPY
    copy.send(rows.copy_text()).await?;
    copy.finish().await?;
} else { /* insert ... select * from unnest($1::bigint[], ...) */ }
/* upsert account_balances từ unnest(tài khoản, độ biến thiên)  */   // 4. số dư
tx.commit().await?;                                                // 5. COMMIT
```

Round trip là một lần client dừng chờ server. Với sqlx đó là mỗi `ReadyForQuery`, vì nó chờ message này trước khi gửi lệnh kế. Riêng `COPY` có thêm một lần chờ không sinh `ReadyForQuery`: client gửi `Query`, chờ `CopyInResponse` rồi mới gửi `CopyData`. Proxy cộng cả hai. Số dưới đây trên 1.000 bút toán, lô 500, sau một bút toán làm ấm cache prepared statement:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 262" role="img" aria-label="Hình 2. Round trip để ghi 1.000 bút toán trên thang logarit: từng dòng 8.991, unnest 10, COPY 12, một câu CTE 2."><path d="M250 30 L250 206" class="f-none s-line dash"/><path d="M380 30 L380 206" class="f-none s-line dash"/><path d="M510 30 L510 206" class="f-none s-line dash"/><path d="M640 30 L640 206" class="f-none s-line dash"/><path d="M770 30 L770 206" class="f-none s-line dash"/><text x="246" y="226" class="t-small">1</text><text x="372" y="226" class="t-small">10</text><text x="500" y="226" class="t-small">100</text><text x="628" y="226" class="t-small">1.000</text><text x="754" y="226" class="t-small">10.000</text><text x="20" y="67" class="t-small">từng dòng, 1 tx/bút toán</text><rect x="250" y="50" width="514" height="26" rx="4" class="f-warn-soft s-warn"/><text x="258" y="67" class="t-small">8.991 round trip · 620 byte/bút toán</text><text x="20" y="109" class="t-small">lô 500, unnest 3 câu lệnh</text><rect x="250" y="92" width="130" height="26" rx="4" class="f-pk-soft s-pk"/><text x="388" y="109" class="t-small">10 round trip · 219 byte/bút toán</text><text x="20" y="151" class="t-small">lô 500, COPY posting</text><rect x="250" y="134" width="140" height="26" rx="4" class="f-pk-soft s-pk"/><text x="398" y="151" class="t-small">12 round trip · 149 byte/bút toán</text><text x="20" y="193" class="t-small">lô 500, 1 câu lệnh CTE</text><rect x="250" y="176" width="39" height="26" rx="4" class="f-pk-soft s-pk"/><text x="297" y="193" class="t-small">2 round trip · 212 byte/bút toán</text><text x="20" y="252" class="t-small">Đếm ReadyForQuery + CopyInResponse qua proxy, đã làm ấm cache prepared statement. Thang logarit: mỗi vạch gấp 10 lần.</text></svg>
</div>
<figcaption>Hình 2. Round trip ghi 1.000 bút toán và byte gửi lên mỗi bút toán.</figcaption>
</figure>

Kết quả khớp công thức thiết kế (9 mỗi bút toán; mỗi lô 5 với `Unnest`, 6 với `Copy`, 1 với `Cte`), `--verify` kiểm trên 199 bút toán. `COPY` không giảm round trip mà thêm một lượt mỗi lô. Nó gửi ít byte hơn, và proxy chỉ ra vì sao: riêng posting, `Bind` tốn 152.091 byte còn `CopyData` 82.613, trên 2.997 dòng. Mảng nhị phân tốn 4 byte độ dài cho mỗi phần tử, nên một dòng mất 36 byte cộng độ dài mã tài khoản; dòng text chỉ có bốn dấu phân cách và vài chữ số, khoảng 13 byte cộng độ dài mã. Đổi lại ta phải tự thoát `\`, tab, xuống dòng (`escape_copy_text`, cũng trả `Cow`).

Hai chi tiết nghiệp vụ. Cột `version` của `account_balances` đếm số lần hàng bị ghi: cùng 200 bút toán, lô gộp cho tổng version 2, từng dòng cho 199, số dư giống hệt; mã dựa vào `version` ([bài 53](/articles/rust-aggregate-invariant-optimistic-locking-for-update)) phải biết điều này. Và số dư ghi theo thứ tự mã tài khoản, để hai writer không khóa chéo nhau.

Thời gian, 10.000 bút toán, lô 500, `--release`, trung vị ba lượt (từng dòng chỉ 1.000 bút toán, một lượt), ở hai mức load. Dòng `Bare` là đúng hai câu lệnh của bài 30, chỉ có ở đợt sau:

| Cách ghi, bút toán/giây | Load 429–436 | Load 429–436, trigger | Load 101–133 | Load 101–133, trigger |
| --- | ---: | ---: | ---: | ---: |
| Từng dòng | 7 | 6 | 30 | 157 |
| `Bare` | | | 18.153 | 10.437 |
| `Unnest` | 2.312 | 2.344 | 14.323 | 8.815 |
| `Copy` | 2.576 | 3.131 | 12.777 | 8.854 |
| `Cte` | 5.940 | 2.836 | 15.215 | 9.108 |

Từng dòng chậm hơn gộp lô từ 50 lần trở lên ở cả hai đợt, cùng hướng với round trip. Giữa ba cách gộp lô, chênh lệch nằm trong nhiễu: `Copy` lúc load 430 thêm trigger lại nhanh hơn. Biến lớn nhất là máy: cùng code `Unnest`, load 430 chậm gấp bốn đến sáu lần load 100.

Vì sao khác 22.791 của bài 30 (load 14 đến 32)? `Bare` cùng đợt cho 18.153; thêm `ON CONFLICT`, `RETURNING` và `account_balances` còn 14.323; thêm trigger còn 8.815. Mỗi số từ một đợt nên chỉ đọc như hướng: idempotency và số dư tốn khoảng một phần năm, trigger hơn một phần ba, và phần lớn quãng từ 22.791 xuống 2.312 là load.

Mục tiêu đã viết chỉ nói idempotency và số dư: đợt load 100 vượt, đợt load 430 không cách nào chạm tới. Thêm trigger, thứ sổ cái thật phải có, thì còn 8.815 đến 9.108. Tôi không coi là đạt: mục tiêu viết thiếu trigger, và chưa đo trên máy rảnh.

## Prepared statement của sqlx: cache theo kết nối, đếm thế nào?

`sqlx` giữ cache prepared statement theo từng kết nối, mặc định 100 câu, tra theo nội dung SQL. `pg_stat_statements` không dùng được vì `shared_preload_libraries` của server lab rỗng, nên ta đếm message `Parse` ở tầng dây và đối chiếu `pg_prepared_statements` của phiên:

| Thí nghiệm | Parse | Round trip | `pg_prepared_statements` |
| --- | ---: | ---: | ---: |
| cùng câu SQL, 200 lần | 1 | 201 | 1 |
| cùng câu, `persistent(false)`, 200 lần | 200 | 400 | 0 |
| `IN ($1..$n)`, n = 1..50 | 50 | 100 | 50 |
| `= any($1)`, mảng dài 1..50 | 1 | 51 | 1 |
| cache 8 chỗ, 20 câu khác nhau | 20 | 52 | 8 |

Kết nối mới `Parse` lại câu quen vì cache thuộc về kết nối, không về pool. SQL động làm hỏng cache: mỗi số tham số là một câu mới, một `Parse` và thêm một round trip. Mảng với `= any($1)` hay `unnest` cho một câu với mọi kích thước. `persistent(false)` trên sqlx 0.9 tốn `Parse` và thêm một round trip mỗi lần, nên chỉ dùng cho câu hiếm. PgBouncer ở transaction pooling đổi kết nối server giữa các transaction và có `max_prepared_statements` cho việc này; lab chưa chạy qua PgBouncer, nên các số trên chỉ đúng cho kết nối thẳng.

## COPY và trigger cân bút toán tốn thế nào?

Bài 25 có hai constraint trigger `FOR EACH ROW` cùng gọi `check_entry_balanced`, hoãn tới lúc commit: `postings_balanced` trên `postings`, `entries_have_postings` trên `journal_entries`. `COPY FROM` kích hoạt trigger như `INSERT`: bút toán lệch nạp thẳng vào `postings` bằng `COPY` bị từ chối lúc commit với SQLSTATE 23514 (`--verify` thử đúng việc này). Trigger chạy theo dòng: 200 bút toán (600 posting) chạy 800 lần, 4 lần mỗi bút toán (1 cho dòng `journal_entries`, 3 cho posting), với cả `Unnest` lẫn `Copy`. Mỗi lần kiểm lại cả bút toán, nên ba trong bốn lần là kiểm thừa.

Ở lượt 10.000 bút toán, server báo đúng 40.000 lần gọi ở cả hai đợt. Thời gian trong hàm thì đổi: 537 đến 1.114 ms lúc load 430 (17% đến 32% thời gian ghi), 416 đến 425 ms lúc load 100 (khoảng 37%). Tắt trigger để kiểm theo lô thì cần quyền chủ bảng và ảnh hưởng mọi phiên, nên tôi giữ nó làm lưới an toàn cuối.

## p50, p99 và coordinated omission

Vấn đề nằm ở cách sinh tải. Vòng đóng gửi request kế khi request trước xong: server kẹt thì client ngừng gửi và không ghi những request lẽ ra đã phải gửi. Vòng mở gửi theo lịch, tính độ trễ từ giờ dự kiến.

```rust
// src/omission.rs: vòng mở, thời gian ảo
let arrival = i * sim.interval_us;       // giờ dự kiến, không đổi dù server kẹt
let start = arrival.max(server_free);
let done = start + sim.service(i);
server_free = done;
samples.record(done - arrival);          // gồm cả thời gian xếp hàng
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 3. Cùng một lần kẹt 1 giây: vòng đóng chỉ ghi một mẫu dài nên p99 vẫn 0,5 ms; vòng mở ghi độ trễ của mọi request đến trong lúc kẹt và lúc dồn hàng, p99 là 980 ms."><rect x="200" y="36" width="160" height="196" class="f-warn-soft s-warn dash"/><text x="206" y="30" class="t-small">server kẹt 1 giây</text><text x="20" y="78" class="t-b">Vòng đóng</text><text x="20" y="96" class="t-small">chờ xong mới gửi</text><rect x="120" y="66" width="640" height="24" rx="4" class="f-phys-soft s-phys"/><rect x="200" y="66" width="160" height="24" rx="4" class="f-warn-soft s-warn"/><text x="206" y="82" class="t-small">1 request, 1.000 ms</text><text x="120" y="114" class="t-small">3.999 mẫu 0,5 ms và 1 mẫu 1.000 ms: p99 = 0,5 ms, max = 1.000 ms</text><text x="20" y="168" class="t-b">Vòng mở</text><text x="20" y="186" class="t-small">gửi theo lịch</text><path d="M200 210 L200 146 L520 210 Z" class="f-warn-soft s-warn"/><path d="M120 210 L760 210" class="f-none s-muted"/><text x="528" y="170" class="t-small">độ trễ theo lúc request đến</text><text x="528" y="190" class="t-small">2.000 request bị ảnh hưởng; p99 = 980 ms</text><path d="M120 210 L120 218 M280 210 L280 218 M440 210 L440 218 M600 210 L600 218 M760 210 L760 218" class="f-none s-muted"/><text x="112" y="236" class="t-small">0 s</text><text x="272" y="236" class="t-small">1 s</text><text x="432" y="236" class="t-small">2 s</text><text x="592" y="236" class="t-small">3 s</text><text x="752" y="236" class="t-small">4 s</text><text x="120" y="258" class="t-small">Thời gian ảo: 4.000 request, 1.000 request/giây, mỗi request 0,5 ms, request thứ 500 kẹt.</text></svg>
</div>
<figcaption>Hình 3. Cùng một lần kẹt 1 giây, hai cách đo cho hai p99 khác nhau.</figcaption>
</figure>

Trong thời gian ảo, kết quả chạy lại ra đúng số cũ:

| Cách đo | p50 | p99 | max |
| --- | ---: | ---: | ---: |
| Vòng đóng | 0,5 ms | 0,5 ms | 1.000 ms |
| Vòng đóng + `record_corrected` | 0,5 ms | 951 ms | 1.000 ms |
| Vòng mở | 0,5 ms | 980 ms | 1.000 ms |

Vòng đóng báo p99 tốt hơn thực tế gần 2.000 lần. `record_corrected` (như `record_correct` của HdrHistogram) kéo p99 về gần vòng mở. Trên PostgreSQL thật, lab khóa bảng `probe` 500 ms (load 429): vòng đóng 2 luồng ghi 570 mẫu, p99 55 ms, max 512 ms; vòng mở 100 request/giây ghi đủ 400 mẫu, p99 476 ms, max 517 ms. Số mẫu dài của vòng đóng tối đa bằng số luồng.

So trước và sau cho một bút toán mỗi request, vòng mở 20 request/giây, 5 giây:

| Load | Lượt | Từng dòng (9 round trip), p50 / p99 | Một câu CTE (1 round trip), p50 / p99 |
| --- | ---: | ---: | ---: |
| 429–438 | 1 | 35 / 108 ms | 17 / 45 ms |
| 429–438 | 2 | 70 / 277 ms | 30 / 88 ms |
| 132–147 | 1 | 70 / 208 ms | 12 / 91 ms |
| 132–147 | 2 | 153 / 445 ms | 15 / 137 ms |

Hướng ổn định: đường một câu thấp hơn ở mọi lượt. Tỷ lệ thì không: p50 từ 2 đến 10 lần, p99 từ 2 đến 3 lần. Kết luận được phép: cùng điều kiện, đường ít round trip có đuôi ngắn hơn.

Vì sao giảm 9 lần round trip không cho 9 lần độ trễ? `--breakdown` dùng một kết nối, không tranh khóa, xen kẽ các trường hợp để cùng chịu nhiễu (200 vòng, hai lượt, load 115 đến 147). Tỷ lệ p50 giữa hai đường là 5,8 và 7,3 khi `synchronous_commit = on`, 8,9 và 9,3 khi `off`. Tắt flush, tỷ lệ về gần đúng 9: hai đường khác nhau gần như chỉ ở round trip. Bật flush, cả hai cùng cộng một phần không nhân theo round trip: `pg_stat_get_backend_io` báo mỗi commit tốn trung bình 2,1 đến 4,8 ms fsync WAL, và mỗi đường chỉ có một commit. Tức `9r + c` so với `r + c`. Tỷ lệ chỉ 2 lần lúc load 430 và p99 thì tôi chưa tách: không có số fsync của đợt đó, nên `c` lớn lên chỉ là giả thuyết. `off` chỉ để tách số; nó có thể mất transaction vừa commit, không dùng cho sổ cái.

## Khi nào dừng tối ưu?

Dừng khi đạt mục tiêu đã viết, hoặc khi thay đổi tiếp theo có lợi ích nhỏ hơn nhiễu mà làm code phức tạp hơn:

| Thay đổi | Chỉ số đổi | Quyết định |
| --- | --- | --- |
| Gộp lô `unnest` | round trip 8.991 xuống 10 (1.000 bút toán) | Giữ |
| Cột mượn `&str` | cấp phát 70.005 xuống 5 | Giữ |
| `Cow` khi chuẩn hóa | cấp phát 10.000 xuống 500 | Giữ ở biên đọc |
| `COPY` thay `unnest` | round trip 10 lên 12; byte 219 xuống 149 | Bỏ: thêm hàm thoát, thông lượng trong nhiễu |
| Một câu CTE cho cả lô | round trip 10 xuống 2 | Bỏ: phải khử key trùng trước, thông lượng trong nhiễu |
| Một câu CTE cho một bút toán | round trip 9 xuống 1, p50 và p99 thấp hơn ở mọi lượt | Giữ ở đường request đơn |

Bước kế tiếp không phải thêm kỹ thuật mà sửa mục tiêu cho có trigger và chạy lại `--pg-bench` khi máy rảnh.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Một writer, bảng bắt đầu rỗng, không đọc xen kẽ, không TLS, không PgBouncer. Bút toán cùng key nhưng khác nội dung không bị phát hiện ở đường lô ([bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) so nội dung). `track_functions` và `track_wal_io_timing` đặt theo phiên nhờ role lab là superuser. Chưa đo trên máy rảnh hay server thật.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ gốc repository, với PostgreSQL của lab đã bật:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --wire
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --pg-bench --n 10000
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --latency --rate 20
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --breakdown --n 200
```

Lệnh đầu không cần database. `--verify` in:

```text
[1/7] cấp phát: normalize_clone 10000, Cow 500, AccountId::new 10000; cột mượn 5, cột clone 70005
[2/7] coordinated omission (thời gian ảo): p99 vòng đóng 500 us, bù 951000 us, vòng mở 980000 us
[3/7] từng dòng, 1 transaction/bút toán     1791 round trip (cần 1791), Parse 0, số dư khớp posting
[3/7] lô 100, unnest 3 câu lệnh               10 round trip (cần 10), Parse 0, số dư khớp posting
[3/7] lô 100, COPY posting                    12 round trip (cần 12), Parse 0, số dư khớp posting
[3/7] lô 100, 1 câu lệnh CTE                   2 round trip (cần 2), Parse 0, số dư khớp posting
[4/7] idempotency: ghi lại và key trùng trong lô không thêm dòng, cả 4 cách
[5/7] Cow trong database: mã thô khớp tài khoản 9.000/10.000, sau normalize 10.000/10.000
[6/7] prepared: Parse cache bật 1 / persistent(false) 200 / IN động 50 / ANY mảng 1; cache 8 chỗ giữ 8 câu; pg_stat_statements nạp: không (đếm bằng proxy)
[7/7] tải thật: vòng mở ghi đủ 50 mẫu theo lịch, vòng đóng 14 mẫu, 0 lỗi (không assert thời gian)
[trigger] 200 bút toán (600 posting) chạy trigger 800 và 800 lần = 4 mỗi bút toán; COPY bút toán lệch bị chặn (SQLSTATE 23514)
VERIFY OK
```

`--verify` không assert thời gian. Số mẫu vòng đóng ở dòng `[7/7]` đổi theo tải (14 và 879 ở hai lần chạy liền nhau). `cargo test` chạy 23 ca (16 đơn vị, 7 tích hợp). Thiếu `TOI_NOI_PG_URL`, lab thoát mã 1, không bỏ qua.

**Bài tập:** nhập 100.000 bút toán bằng `Unnest` lô 500. Cần bao nhiêu round trip? Nếu mỗi round trip 0,5 ms, riêng chờ round trip mất bao lâu, và so với cách từng dòng?

<details><summary>Xem lời giải</summary><p>100.000 / 500 = 200 lô, mỗi lô 5 round trip: 1.000 round trip, 0,5 giây. Từng dòng: 9 × 100.000 = 900.000 round trip, 450 giây.</p></details>

Bài trước: [bài 60](/articles/rust-profiling-flamegraph-tokio-console-dhat). Bài tiếp: [bài 62](/articles/rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3).

## Tài liệu gốc

- [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) và [`std::alloc::GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html).
- [docs.rs: sqlx, `PgConnection::copy_in_raw`](https://docs.rs/sqlx/0.9.0/sqlx/postgres/struct.PgConnection.html) và [hdrhistogram](https://docs.rs/hdrhistogram).
- [PostgreSQL: COPY](https://www.postgresql.org/docs/current/sql-copy.html) và [Message Formats của giao thức](https://www.postgresql.org/docs/current/protocol-message-formats.html).
- [PostgreSQL: pg_stat_statements](https://www.postgresql.org/docs/current/pgstatstatements.html), [CREATE TRIGGER](https://www.postgresql.org/docs/current/sql-createtrigger.html) và [luồng COPY trong giao thức](https://www.postgresql.org/docs/current/protocol-flow.html#PROTOCOL-COPY).
- [PgBouncer: `max_prepared_statements`](https://www.pgbouncer.org/config.html).

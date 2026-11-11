---
slug: postgresql-isolation-read-committed-repeatable-read
title: "Read Committed và Repeatable Read: cùng một lịch chạy, hai kết quả"
summary: "Báo cáo của Tới Nơi đọc cước một chuyến hai lần ra hai số; hai nhân viên cùng nạp tiền vào một ví thì mất một khoản. Chạy cùng bốn lịch tranh chấp dưới Read Committed và Repeatable Read trên PostgreSQL 18.1 để so kết quả."
category: postgresql
level: intermediate
tags: [postgresql, pg-034, isolation, read-committed, repeatable-read, lost-update, serialization-failure, toi-noi]
language: vi
---

Báo cáo đối soát cuối ngày của Tới Nơi đọc cước chuyến 400107 ở đầu giao dịch được 37.000 đ, vài trăm mili-giây sau đọc lại trong **cùng giao dịch** ra 40.000 đ — vì tổng đài vừa sửa cước. Cùng tối đó, hai nhân viên chăm sóc khách cùng nạp tiền bồi thường vào ví 1001: một người +20.000 đ, người kia +50.000 đ. Ví lẽ ra lên 570.000 đ, nhưng chỉ còn 550.000 đ. Không câu lệnh nào báo lỗi.

Hai sự cố có chung một nguyên nhân: giao dịch của mình nhìn thấy (hoặc không nhìn thấy) thay đổi của giao dịch khác ra sao. Đó là **mức cô lập** (isolation level). Bài này chạy **cùng bốn lịch tranh chấp** dưới Read Committed — mặc định của PostgreSQL — rồi dưới Repeatable Read, đặt kết quả cạnh nhau, và chỉ ra chỗ mỗi mức bảo vệ được và không bảo vệ được.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-034 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-mvcc-snapshot">PG-032 · MVCC và snapshot</a>, <a href="/articles/postgresql-lock-row-table">PG-033 · Khoá dòng, khoá bảng và thời gian chờ</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>34-read-committed-repeatable-read.sql</code>; thí nghiệm <code>thu-nghiem/pg034-rc-update-khong-thay-dong-moi-khop.sql</code>, <code>thu-nghiem/pg034-snapshot-cau-dau-va-bai-tap.sql</code>). Sau bài này, bạn so sánh được cùng một lịch thực thi dưới hai mức cô lập và chọn mức (hoặc cách viết câu lệnh) phù hợp cho từng tình huống.</p>
</div>

## 1. Dữ liệu và cách chạy

PostgreSQL **18.1 thật**, nền nhỏ của Phần 4: chuyến 400107 cước 37.000 đ, khu 113 có 3 chuyến đang chờ (400102, 400105, 400108), ví khách 1001–1020 mỗi ví 500.000 đ.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 34
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg034-snapshot-cau-dau-va-bai-tap.sql
```

Ba phiên: A và B là hai giao dịch tranh chấp, C đặt lại dữ liệu trước mỗi lịch và đọc kết quả. Câu lẻ của B không có `BEGIN` thì tự commit ngay. Bước rơi vào chờ khoá được runner ghi `⏳`; kết quả của nó xuất hiện ở dòng `✔` sau khi bước kia nhả khoá — các bảng dưới đây xếp theo thứ tự xảy ra thật. Kết quả chép từ `p04/OUTPUT.txt`.

## 2. Hai mức, khác nhau ở thời điểm chụp snapshot

PG-032 đã nói mỗi câu lệnh đọc dữ liệu qua một **snapshot** — ảnh chụp «giao dịch nào đã commit». Hai mức trong bài khác nhau ở *lúc chụp*:

- **Read Committed** (RC, mặc định): **mỗi câu lệnh** chụp snapshot mới. Câu sau thấy mọi thứ đã commit trước khi nó bắt đầu.
- **Repeatable Read** (RR): chụp **một lần**, ở câu đầu tiên không phải lệnh điều khiển giao dịch (không phải ở `BEGIN`), rồi giữ tới hết giao dịch.

Chuẩn SQL định nghĩa mức cô lập bằng các hiện tượng bị cấm. Bảng của tài liệu PostgreSQL 18:

| Mức | Đọc bẩn | Đọc lại khác | Phantom | Bất thường tuần tự hoá |
|---|---|---|---|---|
| Read Uncommitted | chuẩn cho phép, **PG không** | có thể | có thể | có thể |
| Read Committed | không | có thể | có thể | có thể |
| Repeatable Read | không | không | chuẩn cho phép, **PG không** | có thể |
| Serializable | không | không | không | không |

Hai ô «PG không» là chỗ PostgreSQL chặt hơn chuẩn — chuẩn chỉ quy định hiện tượng nào *không được* xảy ra, chặt hơn vẫn hợp lệ. Read Uncommitted được nhận về cú pháp (lab: `SHOW transaction_isolation` → `read uncommitted`) nhưng chạy y như Read Committed. Cột cuối, Serializable, là chủ đề PG-038.

## 3. S1 · Đọc lại một dòng

| Bước | Phiên | Lệnh | READ COMMITTED | REPEATABLE READ |
|---|---|---|---|---|
| T1 | A | `BEGIN ISOLATION LEVEL …;` `SELECT fare FROM trips WHERE code = 400107;` | 37000 | 37000 |
| T2 | B | `UPDATE trips SET fare = 40000, version = version + 1 WHERE code = 400107;` (tự commit) | UPDATE 1 | UPDATE 1 |
| T3 | A | cùng câu `SELECT` | **40000** | **37000** |
| T4 | A | `COMMIT;` rồi `SELECT` lại (giao dịch mới) | — | 40000 |

Ở RC, T3 là câu mới, snapshot mới, nên thấy 40000: cùng giao dịch, cùng câu, khác kết quả — **đọc lại khác** (non-repeatable read). Ở RR, A đứng yên ở snapshot chụp lúc T1 cho tới khi kết thúc; giao dịch mới mới thấy 40000. Báo cáo đối soát nhiều câu cần các con số khớp nhau thì chạy ở RR.

## 4. S2 · Đếm lại theo điều kiện

| Bước | Phiên | Lệnh | READ COMMITTED | REPEATABLE READ |
|---|---|---|---|---|
| T1 | A | `BEGIN …;` `SELECT count(*) FROM trips WHERE pickup_area_id = 113 AND status = 'requested';` | 3 | 3 |
| T2 | B | `INSERT INTO trips (…, fare, pickup_area_id) VALUES (…, 44000, 113);` (tự commit) | INSERT 0 1 | INSERT 0 1 |
| T3 | A | cùng câu đếm | **4** | **3** |

Dòng «ma» (**phantom**) là dòng mới thoả điều kiện xuất hiện giữa hai lần đọc. Khác S1 ở chỗ không dòng nào A đã thấy bị sửa — có thêm dòng. Chuẩn SQL cho phép phantom ở Repeatable Read; PostgreSQL thì không, vì snapshot của cả giao dịch không chứa dòng B chèn sau đó.

## 5. S3 · Đọc, tính trong ứng dụng, ghi đè

Đây là sự cố ví 1001. Ứng dụng đọc số dư, cộng trong code, rồi ghi **con số đã tính**. Lab mô phỏng bằng `\gset` của psql (lưu kết quả vào biến `:so_du` của phiên):

```sql
SELECT balance AS so_du FROM wallets WHERE id = 1001 \gset
UPDATE wallets SET balance = :so_du + 20000 WHERE id = 1001;   -- B: + 50000
```

| Bước | Phiên | Lệnh | READ COMMITTED | REPEATABLE READ |
|---|---|---|---|---|
| T1 | A | `BEGIN …;` đọc số dư | 500000 | 500000 |
| T2 | B | `BEGIN …;` đọc số dư | 500000 | 500000 |
| T3 | A | `UPDATE … SET balance = 500000 + 20000` | UPDATE 1 | UPDATE 1 |
| T4 | B | `UPDATE … SET balance = 500000 + 50000` | ⏳ chờ A | ⏳ chờ A |
| T5 | A | `COMMIT;` | COMMIT | COMMIT |
| T6 | B | (T4 chạy tiếp) | ✔ **UPDATE 1** | ✔ **`ERROR: could not serialize access due to concurrent update`** (`40001`) |
| T7 | B | `COMMIT;` | COMMIT | thẻ **`ROLLBACK`** |
| T8 | C | `SELECT balance … WHERE id = 1001;` | **550000** | 520000 |
| T9 | B | thử lại cả giao dịch: đọc → `UPDATE … = :so_du + 50000` → `COMMIT` | — | đọc 520000, ghi ⇒ **570000** |

Ở RC, B chờ A rồi vẫn ghi 550000 — số nó tính từ 500000 đọc *trước* khi A commit. Khoản +20000 của A **mất** (lost update), không lỗi nào báo. Ở RR, B bị từ chối: tài liệu nói giao dịch RR không được sửa hay khoá dòng mà giao dịch khác đã sửa (và commit) sau khi nó bắt đầu. Lỗi rơi ở câu UPDATE, giao dịch hỏng, và `COMMIT` trả thẻ `ROLLBACK` chứ không báo lỗi (PG-031) — ứng dụng phải bắt `40001` ngay ở câu lệnh. B **thử lại từ đầu**, lần này đọc 520000 và ra đúng 570000.

## 6. S4 · Ghi tương đối

Cùng hai khoản nạp, nhưng để database tự cộng: `UPDATE wallets SET balance = balance + x`.

| Bước | Phiên | Lệnh | READ COMMITTED | REPEATABLE READ |
|---|---|---|---|---|
| T1 | A | `BEGIN …; UPDATE wallets SET balance = balance + 20000 WHERE id = 1001;` | UPDATE 1 | UPDATE 1 |
| T2 | B | `BEGIN …; UPDATE wallets SET balance = balance + 50000 WHERE id = 1001 …;` | ⏳ chờ A | ⏳ chờ A |
| T3 | A | `COMMIT;` | COMMIT | COMMIT |
| T4 | B | (T2 chạy tiếp) | ✔ `RETURNING balance` → **570000** | ✔ **`40001`** |
| T5 | B | `COMMIT;` | COMMIT | thẻ `ROLLBACK` |
| T6 | C | đọc số dư | **570000** | **520000** — B phải thử lại |

Đảo ngược S3: ở đây RC cho kết quả đúng mà không cần thử lại, còn RR từ chối B dù phép cộng hoàn toàn an toàn. RR không phân biệt «ghi đè giá trị cũ» với «cộng thêm»; nó chỉ thấy dòng đã bị giao dịch khác commit sau snapshot của B.

Bảng tóm tắt bốn lịch:

| Lịch | READ COMMITTED | REPEATABLE READ |
|---|---|---|
| S1 đọc lại một dòng | 37000 → 40000 | 37000 → 37000 |
| S2 đếm lại (phantom) | 3 → 4 | 3 → 3 |
| S3 đọc – tính – ghi đè | 550000: **mất +20000** | `40001` ⇒ thử lại ⇒ 570000 |
| S4 `balance = balance + x` | 570000, đúng | `40001` ⇒ phải thử lại |

## 7. Vì sao: RC đọc lại dòng, RR từ chối

Khi một câu UPDATE/DELETE/`SELECT … FOR UPDATE` ở **Read Committed** gặp dòng đang bị giao dịch khác sửa, nó **chờ** (PG-033). Nếu giao dịch kia ROLLBACK, nó sửa dòng nó đã tìm thấy. Nếu kia COMMIT, nó lấy **phiên bản mới** của dòng, kiểm lại điều kiện `WHERE` trên phiên bản đó, khớp thì áp phép sửa lên phiên bản mới (trong mã nguồn cơ chế này gọi là EvalPlanQual). Vì thế:

- S4 đúng: `balance + 50000` được tính trên 520000 vừa commit.
- S3 sai: biểu thức là hằng 550000 tính sẵn trong ứng dụng; đọc lại dòng không làm hằng đó đổi.
- `UPDATE … WHERE version = :v` hay `WHERE status = 'requested'` đúng khi chạy đồng thời: kiểm lại trên phiên bản mới ⇒ không khớp ⇒ 0 dòng. PG-035 dựng optimistic concurrency trên đúng tính chất này.

Nhưng câu lệnh chỉ đọc lại **những dòng nó đã tìm thấy**, không quét lại bảng. Tài liệu gọi đây là snapshot không nhất quán: câu ghi thấy thay đổi của giao dịch khác trên chính dòng nó sửa, nhưng không thấy thay đổi trên dòng khác. Thí nghiệm `pg034-rc-update-khong-thay-dong-moi-khop.sql` (có sẵn trong lab, khu 113 ban đầu có 400102, 400105, 400108; 400101 ở khu 112):

| Bước | Phiên | Lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN; UPDATE trips SET status = 'cancelled' … WHERE code = 400102;` | UPDATE 1 |
| T2 | A | `UPDATE trips SET pickup_area_id = 113 … WHERE code = 400101;` | UPDATE 1 — 400101 sắp **trở nên khớp** |
| T3 | B | `UPDATE trips SET fare = fare + 1000 WHERE pickup_area_id = 113 AND status = 'requested' RETURNING code;` | ⏳ chờ A (gặp 400102) |
| T4 | A | `COMMIT;` | COMMIT |
| T5 | B | (T3 chạy tiếp) | ✔ `400105`, `400108` |
| T6 | A | `SELECT code, fare … WHERE pickup_area_id = 113 AND status = 'requested'` | `400101 \| 31000` · `400105 \| 36000` · `400108 \| 39000` |

400102 bị huỷ ⇒ B kiểm lại, bỏ qua — đúng. Nhưng 400101 nằm ngoài lần quét ban đầu của B (lúc đó nó còn ở khu 112) nên bị **bỏ sót**, dù A commit trước khi B xong. Kết quả của B không ứng với *bất kỳ* snapshot nào: không phải trước A (thì 400102 được tăng), không phải sau A (thì 400101 được tăng). Tài liệu kết luận RC hợp với điều kiện đơn giản, không hợp với câu ghi có điều kiện tìm kiếm phức tạp.

**Repeatable Read** không đọc lại: dòng cần sửa đã bị giao dịch commit sau snapshot sửa trước ⇒ báo `40001` và để ứng dụng làm lại từ đầu với snapshot mới. Bất thường kiểu trên không xảy ra; đổi lại là lỗi phải xử lý, kể cả ở S4.

## 8. Snapshot chụp lúc nào, và khi nào RR không lỗi

Thí nghiệm `pg034-snapshot-cau-dau-va-bai-tap.sql` (không có trong lab chính) kiểm ba chi tiết hay bị hiểu sai:

| Thí nghiệm | Lịch | Kết quả |
|---|---|---|
| §1 Read Uncommitted | A `BEGIN; UPDATE … fare = 99000 WHERE code = 400103;` (chưa commit) · B `BEGIN ISOLATION LEVEL READ UNCOMMITTED; SELECT fare …` | B thấy **33000** — không đọc bẩn; A ROLLBACK |
| §2 RR chụp ở câu đầu | B `BEGIN ISOLATION LEVEL REPEATABLE READ;` (chưa chạy gì) · A đổi cước thành 34000, tự commit · B `SELECT` · A đổi 35000 · B `SELECT` | B thấy **34000** rồi **34000** |
| §3 RR, người ghi trước ROLLBACK | A `BEGIN RR; UPDATE wallets … + 20000 WHERE id = 1002;` · B `BEGIN RR; UPDATE … + 50000 … RETURNING balance` ⏳ · A `ROLLBACK` | B ✔ **550000**, không `40001` |

§2 nghĩa là `BEGIN ISOLATION LEVEL REPEATABLE READ` chưa «đóng băng» gì: thay đổi commit giữa `BEGIN` và câu đầu tiên vẫn được thấy. Cần mốc chính xác thì chạy câu đọc đầu tiên ngay. §3 khớp tài liệu: RR chỉ báo `40001` khi giao dịch sửa trước **commit**; nếu nó rollback, người chờ sửa tiếp bình thường.

## 9. Đặt mức cô lập và xử lý 40001

```sql
BEGIN ISOLATION LEVEL REPEATABLE READ;                 -- cho một giao dịch
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;       -- câu đầu của giao dịch đã BEGIN
SET default_transaction_isolation = 'repeatable read'; -- mặc định của phiên (hoặc role/database)
```

Lab: sau `SET default_transaction_isolation = 'repeatable read'`, một `BEGIN` trần cho `SHOW transaction_isolation` → `repeatable read`. `SET TRANSACTION` phải đứng trước câu truy vấn đầu tiên: sau `BEGIN; SELECT 1;` nó báo `25001 SET TRANSACTION ISOLATION LEVEL must be called before any query` (thí nghiệm §6). Đổi mặc định cho cả ứng dụng là quyết định lớn: mọi giao dịch ghi từ đó có thể nhận `40001`.

Ba quy tắc khi dùng RR:

1. **Thử lại toàn bộ giao dịch** từ câu đọc đầu tiên, không chỉ câu lỗi — dữ liệu đã đọc thuộc snapshot cũ (S3, T9).
2. **Giới hạn số lần thử** và nghỉ ngẫu nhiên giữa các lần; PG-038 đo chuyện này dưới tải.
3. Giao dịch **chỉ đọc** không bao giờ gặp lỗi tuần tự hoá ở RR — báo cáo dùng RR mà không cần vòng thử lại.

## 10. Chọn gì cho Tới Nơi

| Tình huống | Đề xuất | Vì sao |
|---|---|---|
| Báo cáo/đối soát nhiều câu cần số khớp nhau | RR, chỉ đọc | Một snapshot cho cả giao dịch; không có `40001` |
| Cộng/trừ ví, đếm lượt | RC + `balance = balance + x` | Đúng khi đồng thời, không cần thử lại (S4) |
| Đọc rồi quyết định trong code rồi ghi (S3) | RC + khoá dòng khi đọc (`FOR NO KEY UPDATE`), hoặc RR + thử lại | Bài tập 2; hoặc kiểm `version` — PG-035 |
| Đổi trạng thái chuyến | RC + `UPDATE … WHERE status = …` rồi kiểm số dòng | Kiểm lại điều kiện trên phiên bản mới |
| Câu ghi có điều kiện trên nhiều dòng mà dòng có thể «trở nên khớp» | RR + thử lại, hoặc Serializable | Tránh bỏ sót như mục 7 |
| Quy tắc trải trên nhiều dòng («mỗi khu ≥ 1 tài xế trực») | Serializable | RR vẫn để lọt — PG-038 |

**Đánh đổi.** RC không bao giờ trả `40001` cho bốn lịch trên, nhưng để lọt S3 và mục 7 một cách im lặng — lỗi chỉ lộ khi đối soát. RR biến các trường hợp đó thành lỗi rõ ràng, đổi lại mọi giao dịch ghi cần vòng thử lại và từ chối cả phép ghi an toàn như S4. Không mức nào miễn phí; chọn theo từng giao dịch, không phải một lần cho cả hệ thống.

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Ví thiếu một khoản, không lỗi | Đọc – tính – ghi đè ở RC (S3) | Ghi tương đối, khoá dòng khi đọc, hoặc RR + thử lại |
| Báo cáo hai câu lệch nhau | RC chụp snapshot mỗi câu | Chạy báo cáo trong một giao dịch RR |
| `40001` ở mức RR rồi dữ liệu không đổi | `COMMIT` sau lỗi trả `ROLLBACK`, ứng dụng tưởng đã lưu | Bắt `40001` ở câu lệnh, thử lại cả giao dịch |
| Thử lại chỉ câu UPDATE lỗi | Vẫn dùng số đọc từ snapshot cũ | Thử lại từ `BEGIN` |
| Đặt READ UNCOMMITTED để «nhanh hơn» | PostgreSQL chạy như RC | Không có gì để được |
| RR «không thấy» thay đổi đã commit trước `BEGIN`… mà lại thấy | Snapshot chụp ở câu đầu, không phải `BEGIN` | Chạy câu đọc đầu ngay sau `BEGIN` |

## 12. Bài tập

**Bài 1.** B chạy `BEGIN ISOLATION LEVEL REPEATABLE READ;` rồi chưa làm gì. A đổi cước 400103 thành 34.000 (tự commit). B `SELECT fare` ra bao nhiêu? A đổi tiếp thành 35.000; B `SELECT` lần nữa ra bao nhiêu?

<details><summary><b>Lời giải</b></summary>
<p><b>34000</b> rồi <b>34000</b>. Snapshot của RR chụp ở câu đầu tiên của B (sau khi A commit 34000), không phải lúc <code>BEGIN</code>; từ đó B đứng yên. Thí nghiệm <code>pg034-snapshot-cau-dau-va-bai-tap.sql</code> §2.</p>
</details>

**Bài 2.** Sửa lịch S3 cho đúng mà **giữ** Read Committed và giữ kiểu «đọc số dư vào ứng dụng, tính, rồi ghi con số đã tính» (ví 1003, A +20.000, B +50.000).

<details><summary><b>Lời giải</b></summary>
<p>Khoá dòng ngay khi đọc, để người thứ hai phải chờ trước khi đọc:</p>
<pre><code class="language-sql">BEGIN;
SELECT balance AS so_du FROM wallets WHERE id = 1003 FOR NO KEY UPDATE \gset
UPDATE wallets SET balance = :so_du + 20000 WHERE id = 1003;
COMMIT;</code></pre>
<p>B chạy cùng đoạn: câu đọc của B <b>⏳ chờ</b> A; sau khi A commit, B đọc được <b>520000</b>, ghi 570000; số dư cuối <b>570000</b> (thí nghiệm §4). <code>FOR NO KEY UPDATE</code> đủ vì không đổi khoá chính (PG-033). Cái giá: hai giao dịch chạy nối đuôi trên ví đó.</p>
</details>

**Bài 3.** Chạy S4 dưới Repeatable Read trên ví 1004 và viết phần thử lại của B sao cho số dư cuối đúng.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">-- B, lần 1: UPDATE … balance + 50000 ⏳ ⇒ sau khi A commit: ERROR 40001; COMMIT trả ROLLBACK
BEGIN ISOLATION LEVEL REPEATABLE READ;   -- lần 2: giao dịch mới, snapshot mới
UPDATE wallets SET balance = balance + 50000 WHERE id = 1004 RETURNING balance;  -- 570000
COMMIT;</code></pre>
<p>Số dư cuối 570000 (thí nghiệm §5). Ứng dụng thật bọc đoạn này trong vòng lặp có giới hạn số lần, chỉ lặp khi SQLSTATE là <code>40001</code>.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Transaction Isolation](https://www.postgresql.org/docs/18/transaction-iso.html) — bảng hiện tượng, hành vi UPDATE ở Read Committed, `40001` ở Repeatable Read.
- [PostgreSQL 18 · SET TRANSACTION](https://www.postgresql.org/docs/18/sql-set-transaction.html) và [Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html) — `default_transaction_isolation`.
- Lab: `p04/34-read-committed-repeatable-read.sql`, `p04/OUTPUT.txt`; thí nghiệm có sẵn `p04/thu-nghiem/pg034-rc-update-khong-thay-dong-moi-khop.sql` và thí nghiệm `p04/thu-nghiem/pg034-snapshot-cau-dau-va-bai-tap.sql` (không có trong lab chính).

Bài tiếp theo: [PG-035: Optimistic concurrency và cập nhật có điều kiện](/articles/postgresql-optimistic-version-conditional-update).

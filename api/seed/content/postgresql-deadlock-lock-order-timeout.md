---
slug: postgresql-deadlock-lock-order-timeout
title: "Deadlock, thứ tự lấy khoá và timeout: thanh toán và hoàn tiền chéo nhau trên ví VíXanh"
summary: "Thanh toán chuyến trừ ví khách rồi cộng ví tài xế, hoàn tiền đi chiều ngược lại — chạy cùng lúc trên cùng cặp ví là PostgreSQL báo deadlock. Tái hiện vòng chờ, đọc nhật ký server, thử lại, sửa bằng thứ tự khoá và lock_timeout."
category: postgresql
level: intermediate
tags: [postgresql, pg-037, deadlock, lock, lock-timeout, transaction, retry, toi-noi]
language: vi
---

Tối thứ Sáu, nhóm thanh toán của Tới Nơi thấy log API rải rác dòng `ERROR: deadlock detected`, kèm khiếu nại «hoàn tiền không thành công, bấm lại thì được». Hai nghiệp vụ dính vào nhau: **thanh toán chuyến** trừ ví VíXanh của khách rồi cộng ví tài xế; **hoàn tiền** do chăm sóc khách hàng bấm thì trừ ví tài xế rồi cộng ví khách. Mỗi câu lệnh đều đúng, mỗi giao dịch chạy một mình đều đúng. Chỉ khi hai giao dịch đụng **cùng một cặp ví theo hai chiều ngược nhau** thì chúng kẹt vào nhau.

PostgreSQL không treo mãi: nó phát hiện vòng chờ và huỷ một giao dịch với SQLSTATE `40P01`. Bài này tái hiện đúng tình huống đó trên hai phiên, đọc lỗi và nhật ký server, rồi đi qua ba lớp xử lý: thử lại khi bị huỷ, sửa tận gốc bằng **thứ tự lấy khoá**, và dùng `lock_timeout` làm lưới an toàn.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-037 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-lock-row-table">PG-033 · Khoá dòng, khoá bảng và thời gian chờ</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>37-deadlock-thu-tu-khoa.sql</code>, phép thử đồng thời K037 trong <code>DO-DAC.md</code>) và thí nghiệm <code>p04/thu-nghiem/pg037-vong-ba-phien-lock-timeout.sql</code>. Sau bài này, bạn tái hiện được một vòng chờ giữa hai giao dịch và sửa nó bằng cách cho mọi giao dịch truy cập dữ liệu theo cùng một thứ tự.</p>
</div>

## 1. Dữ liệu và cách chạy

Lab Phần 4 chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 dùng PGlite 17.5): runner dựng một cluster tạm trong `%TEMP%`, mỗi tệp bài chạy trên database chép từ nền, xong thì dọn. Mỗi phiên `A`, `B`, `C` là một tiến trình psql riêng — một kết nối riêng.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 37
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg037-vong-ba-phien-lock-timeout.sql
```

Bảng `wallets` có 44 ví: khách 1001–1020 mỗi ví 500.000 đ, tài xế 2001–2024 mỗi ví 200.000 đ, **tổng 14.800.000 đ**. Bất biến của bài: dù giao dịch nào bị huỷ, tổng tiền không đổi. Kết quả dưới đây chép từ `p04/OUTPUT.txt`; runner thay PID bằng nhãn phiên `[A]`, xid bằng `<XID>`, thời gian chờ bằng `T`.

| Phiên | Nghiệp vụ | Thứ tự đụng ví |
|---|---|---|
| A | Thanh toán chuyến 30.000 đ | trừ **1001** (khách) → cộng **2001** (tài xế) |
| B | Hoàn tiền 10.000 đ | trừ **2001** (tài xế) → cộng **1001** (khách) |
| C | Quan sát | `lab_cho()`, đọc số dư |

## 2. Vòng chờ là gì

PG-033 đã cho thấy: `UPDATE` một dòng thì giao dịch giữ **khoá dòng** tới khi COMMIT/ROLLBACK; giao dịch khác muốn sửa cùng dòng phải chờ — cụ thể là chờ `ShareLock` trên **mã giao dịch** (`transactionid`) của bên đang giữ. Chờ thì không sao, miễn bên giữ khoá rồi cũng kết thúc.

**Deadlock** (khoá chết, bế tắc) là khi các giao dịch chờ nhau thành **vòng**: A chờ B, B chờ A. Không bên nào kết thúc được vì bên nào cũng đang chờ. Tài liệu PostgreSQL nói rõ: chừng nào chưa phát hiện deadlock, giao dịch chờ khoá dòng hay khoá bảng sẽ chờ **vô thời hạn**. Vì vậy server phải tự phá vòng — bằng cách huỷ một giao dịch trong vòng.

## 3. Tái hiện: hai phiên, hai chiều

Lịch dưới đây là thứ tự **xảy ra thật**: bước T3 gửi trước nhưng chỉ chạy xong ở T6, sau khi B bị huỷ.

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN; UPDATE wallets … - 30000 WHERE id = 1001` | 470000 — A giữ 1001 |
| T2 | B | `BEGIN; UPDATE wallets … - 10000 WHERE id = 2001` | 190000 — B giữ 2001 |
| T3 | A | `UPDATE wallets … + 30000 WHERE id = 2001` | ⏳ chờ B |
| T4 | C | `SELECT * FROM lab_cho()` | A `active`, chờ `Lock: transactionid`, bị chặn bởi B |
| T5 | B | `UPDATE wallets … + 10000 WHERE id = 1001` | ⏳ chờ A ⇒ **vòng** ⇒ sau 200 ms: **`40P01`** |
| T6 | A | (bước T3 chạy tiếp) | ✔ **230000** |
| T7 | A | `COMMIT` | `COMMIT` |
| T8 | B | `COMMIT` | thẻ **`ROLLBACK`** |
| T9 | C | đọc 1001, 2001, tổng | 470000 · 230000 · 14800000 |

Ảnh chụp ở T4, ngay trước khi B khép vòng:

```text
 phien |     trang_thai      |         cho         | bi_chan_boi |                           cau_lenh
-------+---------------------+---------------------+-------------+--------------------------------------------------------------
 A     | active              | Lock: transactionid | B           | UPDATE wallets SET balance = balance + 30000, updated_at = n
 B     | idle in transaction | Client: ClientRead  |             | UPDATE wallets SET balance = balance - 10000, updated_at = n
(2 rows)
```

Đây mới là chờ bình thường: B đang rảnh, chỉ cần B commit là A chạy tiếp. Ở T5, B lại xin đúng dòng A đang giữ — vòng khép lại. Lỗi B nhận được:

```text
ERROR:  deadlock detected
DETAIL:  Process [B] waits for ShareLock on transaction <XID>; blocked by process [A].
Process [A] waits for ShareLock on transaction <XID>; blocked by process [B].
HINT:  See server log for query details.
CONTEXT:  while updating tuple (0,1) in relation "wallets"
-- [lab] SQLSTATE 40P01
```

B bị huỷ thì khoá của B được nhả ngay, và câu đang chờ của A chạy xong với **230000** = 200.000 + 30.000: khoản trừ 10.000 của B đã biến mất cùng giao dịch của B. A commit bình thường. `COMMIT` của B không báo lỗi mà trả thẻ `ROLLBACK` (PG-031) — ứng dụng phải kiểm. Kết quả cuối: chỉ thanh toán có hiệu lực, tổng vẫn 14.800.000 đ. Deadlock **không làm sai dữ liệu**; nó làm một nghiệp vụ thất bại.

## 4. Ai bị huỷ — và vì sao lab «chọn» được B

Kiểm vòng chờ tốn kém, nên PostgreSQL không kiểm mỗi lần một phiên phải chờ. Theo tài liệu `deadlock_timeout`, server lạc quan giả định deadlock hiếm, cứ chờ khoá một lúc, **đủ `deadlock_timeout`** mới chạy bộ dò. Mặc định là 1 giây; trong lab, phiên vượt ngưỡng đó trước sẽ chạy bộ dò, thấy vòng và chính nó nhận `40P01`.

Bài ép lịch chạy tất định bằng cách đặt `deadlock_timeout` của A là **10 s**, của B là **200 ms** — B luôn vượt ngưỡng trước. Đây là mẹo của lab, **không phải quy tắc** kiểu «phiên đến sau luôn chết». Tài liệu PostgreSQL nói thẳng rằng giao dịch nào bị huỷ là khó đoán và không nên dựa vào. Ứng dụng phải sẵn sàng cho **bất kỳ** bên nào nhận `40P01`.

Thí nghiệm cũng đọc `pg_settings` của cluster lab:

```text
       name       | setting | unit |  context
------------------+---------+------+-----------
 deadlock_timeout | 1000    | ms   | superuser
 lock_timeout     | 0       | ms   | user
 log_lock_waits   | off     | NULL | superuser
(3 rows)
```

`context = superuser`: chỉ superuser hoặc người được cấp quyền `SET` mới đổi được `deadlock_timeout` — lab làm được vì chạy bằng superuser. Tăng nó thì bớt lần kiểm thừa nhưng deadlock thật bị báo chậm hơn; tài liệu khuyên giá trị này nên lớn hơn thời gian giao dịch điển hình.

## 5. Lỗi trả về không đủ — đọc nhật ký server

Thông báo gửi cho client chỉ có số tiến trình và mã giao dịch: không biết câu lệnh nào. `HINT` chỉ đường: chi tiết nằm ở **nhật ký server**. Bản ghi tương ứng (B có bật `log_lock_waits`):

```text
[B] LOG:  process [B] detected deadlock while waiting for ShareLock on transaction <XID> after T ms
[B] DETAIL:  Process holding the lock: [A]. Wait queue: .
…
[B] ERROR:  deadlock detected
[B] DETAIL:  Process [B] waits for ShareLock on transaction <XID>; blocked by process [A].
	Process [A] waits for ShareLock on transaction <XID>; blocked by process [B].
	Process [B]: UPDATE wallets SET balance = balance + 10000, updated_at = now() WHERE id = 1001 RETURNING balance;
	Process [A]: UPDATE wallets SET balance = balance + 30000, updated_at = now() WHERE id = 2001 RETURNING balance;
```

Dòng `ERROR` trong nhật ký có **câu lệnh của cả hai phiên** — đủ để biết hai đường code nào đâm nhau. Thí nghiệm ba phiên (mục 9) chạy với `log_lock_waits` tắt vẫn có dòng `ERROR` kèm câu lệnh; thứ mất đi là dòng `LOG … detected deadlock … after T ms`. `log_lock_waits` còn ghi mọi lần chờ khoá lâu hơn `deadlock_timeout` dù không thành deadlock — hữu ích để tìm chỗ chờ khoá làm chậm hệ thống. Trên PostgreSQL 18 nó mặc định `off`.

## 6. Thử lại sau `40P01`

Giao dịch bị huỷ không để lại gì, nên cách xử lý tức thời là **chạy lại toàn bộ giao dịch** — không phải chỉ câu lệnh vừa lỗi, vì mọi thứ trước đó cũng đã bị huỷ. Lab cho B làm lại hoàn tiền sau khi A đã commit:

```sql
BEGIN;
UPDATE wallets SET balance = balance - 10000, updated_at = now() WHERE id = 2001;
UPDATE wallets SET balance = balance + 10000, updated_at = now() WHERE id = 1001;
COMMIT;
```

```text
  id  | balance
------+---------
 1001 |  480000
 2001 |  220000
(2 rows)
```

Tài liệu PostgreSQL (mục xử lý lỗi tuần tự hoá) khuyên có thể thử lại lỗi deadlock, và phải thử lại **cả giao dịch kèm logic quyết định câu lệnh nào chạy**. Phép thử đồng thời của lab dùng hàm `giaoDich()` (`pg/lib/dong-thoi.mjs`): gặp `40001`/`40P01` thì ROLLBACK, chờ ngẫu nhiên trong [0, min(trần, 2 ms · 2^lần)] rồi chạy lại toàn bộ hàm giao dịch, có **giới hạn số lần** (K037 cho tối đa 20). PG-038 đi chi tiết vào vòng thử lại này.

Thử lại chữa được triệu chứng, không chữa nguyên nhân: deadlock vẫn xảy ra, mỗi lần tốn ít nhất một `deadlock_timeout` chờ cộng một lần làm lại. Mục 8 cho thấy cái giá đó lớn cỡ nào.

## 7. Sửa tận gốc: mọi giao dịch khoá theo cùng một thứ tự

Vòng chờ cần ít nhất hai giao dịch lấy khoá **theo thứ tự ngược nhau**. Bỏ được điều kiện đó thì không thể có vòng. Tài liệu PostgreSQL gọi đây là cách phòng thủ tốt nhất: mọi ứng dụng lấy khoá trên nhiều đối tượng theo một thứ tự nhất quán.

Với ví, thứ tự tự nhiên là **id tăng dần**, bất kể chiều chuyển tiền. Mỗi giao dịch mở đầu bằng một câu khoá trước mọi ví sẽ đụng tới, rồi mới trừ/cộng:

```sql
SELECT id FROM wallets WHERE id IN (1001, 2001) ORDER BY id FOR NO KEY UPDATE;
```

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN;` câu khoá trên | 1001, 2001 — A giữ cả hai |
| T2 | B | `BEGIN;` cùng câu khoá (dù nghiệp vụ trừ 2001 trước) | ⏳ chờ A **ở 1001**, chưa giữ gì |
| T3 | A | trừ 1001, cộng 2001, `COMMIT` | `COMMIT` |
| T4 | B | (bước T2 chạy tiếp) | ✔ 1001, 2001 |
| T5 | B | trừ 2001, cộng 1001, `COMMIT` | `COMMIT` |
| T6 | C | đọc số dư | 1001 = **460000**, 2001 = **240000** |

B vẫn phải chờ — tranh chấp là thật — nhưng chờ ngay ở ví **đầu tiên**, khi tay chưa cầm khoá nào, nên không ai có thể chờ ngược lại B. Hai giao dịch xếp hàng thay vì kẹt nhau.

Ba chi tiết đáng giữ:

- **Vì sao câu này khoá đúng thứ tự.** Kế hoạch của nó (thí nghiệm §4) có nút `LockRows` nằm **trên** nút `Sort`: dòng được sắp theo `id` trước, rồi mới khoá lần lượt.

```text
 LockRows
   ->  Sort
         Sort Key: id
         ->  Seq Scan on wallets
               Filter: (id = ANY ('{1001,2001}'::integer[]))
```

- **`UPDATE … WHERE id IN (…)` một câu không đủ.** Nó khoá dòng theo thứ tự quét mà planner chọn — không có gì bảo đảm thứ tự đó giống nhau giữa các câu và các kế hoạch.
- **`FOR NO KEY UPDATE` thay vì `FOR UPDATE`.** Đủ mạnh để sửa số dư (cũng là khoá mà `UPDATE` thường tự lấy), nhưng không chặn các câu chỉ cần `FOR KEY SHARE` như kiểm khoá ngoại khi chèn dòng con (PG-033, PG-036). Tài liệu còn khuyên khoá **đầu tiên** lấy trên một đối tượng nên là khoá mạnh nhất sẽ cần — đọc bằng `FOR SHARE` rồi sau đó mới nâng lên để sửa là một cách tự tạo deadlock.

Khi giao dịch đụng nhiều bảng, «thứ tự» phải là thứ tự toàn cục: bảng nào trước (ví dụ luôn `trips` rồi mới `wallets`), trong mỗi bảng theo khoá chính. Viết nó thành một hàm/thư viện dùng chung — quy tắc chỉ có giá trị khi **mọi** đường ghi tuân theo.

## 8. Bằng chứng khi chạy đua thật: K037

Lịch tất định chứng minh cơ chế; phép thử K037 cho tám phiên chạy đua thật. Cấu hình (trích `p04/DO-DAC.md`): i7-10700 (16 luồng), 64 GB RAM, NVMe Kingston SA2000, Windows 11 Pro, Node 22.18, PostgreSQL 18.1, tải CPU nền ~6 %. Mỗi phiên làm 150 lần chuyển 1.000 đ giữa bốn ví tài xế 2001–2004 (cặp và chiều ngẫu nhiên), nghỉ `pg_sleep(0.001)` giữa hai câu `UPDATE`, `deadlock_timeout = 20ms` cho nhanh, thử lại tối đa 20 lần; mỗi kịch bản **3 lần**, mỗi lần một database mới.

| Kịch bản | Giây/lần (min–max) | Chuyển xong | Lần `40P01` | Tổng 4 ví |
|---|---|---|---|---|
| Khoá theo chiều chuyển | 30,77–32,86 | 1.200 | **1.122–1.184** | 800.000 (cả 3 lần) |
| Khoá theo id trước | 14,53–14,94 | 1.200 | **0** | 800.000 (cả 3 lần) |

Cả hai cách đều **đúng** (tổng tiền giữ nguyên, 0 giao dịch bỏ cuộc) — nhờ thử lại. Khác nhau ở cái giá: cách theo chiều gặp gần một deadlock cho mỗi lần chuyển, cách theo id không gặp lần nào. Giới hạn kết luận: trên Windows `pg_sleep(0.001)` ngủ thực ~15,6 ms (độ phân giải bộ định thời), nên thời gian bị chi phối bởi bước nghỉ giả lập; chỉ so tương đối hai cách trong cùng phép thử, không suy ra thông lượng thật. Bốn ví cho tám phiên cũng là mức tranh chấp cố ý cao.

## 9. Vòng dài hơn hai phiên

Vòng không nhất thiết chỉ có hai bên. Thí nghiệm §1 cho ba phiên, mỗi phiên trừ «ví của mình» rồi cộng ví kế tiếp: A 1001→1002, B 1002→1003, C 1003→1001; C đặt `deadlock_timeout` 200 ms, A và B 10 s.

| Bước | Phiên | Kết quả |
|---|---|---|
| T1–T3 | A, B, C | mỗi phiên trừ ví của mình: 499000 |
| T4 | A | cộng 1002 ⏳ chờ B |
| T5 | B | cộng 1003 ⏳ chờ C — chuỗi A → B → C, chưa vòng |
| T6 | C | cộng 1001 ⏳ chờ A ⇒ **vòng ba phiên** ⇒ `40P01` |
| T7 | B | ✔ 501000 (C bị huỷ, khoản trừ của C mất) |
| T8 | C, B | `COMMIT` → `ROLLBACK`; `COMMIT` |
| T9 | A | ✔ 500000, rồi `COMMIT` |

```text
DETAIL:  Process [C] waits for ShareLock on transaction <XID>; blocked by process [A].
Process [A] waits for ShareLock on transaction <XID>; blocked by process [B].
Process [B] waits for ShareLock on transaction <XID>; blocked by process [C].
```

Cuối cùng 1001 = 499000, 1002 = 500000, 1003 = 501000, tổng ba ví vẫn 1.500.000 đ. Vòng càng dài càng khó đoán khi đọc code; thứ tự khoá theo id phá được vòng ở mọi độ dài.

## 10. `lock_timeout`: lưới an toàn, không phải thuốc chữa

`deadlock_timeout` **không** phải thời hạn bỏ cuộc: nó chỉ là lúc bắt đầu kiểm vòng. Một giao dịch chờ một giao dịch khác đang «ngồi yên» (ví dụ giữ khoá trong lúc gọi API ngoài) sẽ chờ mãi mà không có deadlock nào. `lock_timeout` mới là giới hạn chờ: câu lệnh nào chờ **một** lần lấy khoá quá thời gian đó thì bị huỷ với `55P03`. Lab §3: A giữ ví 1002, B đặt `SET LOCAL lock_timeout = '300ms'` rồi sửa cùng ví.

```text
-- [lab] [B] ⏳ đã chờ khoá (Lock: transactionid) — bị chặn bởi [A] trước khi xong
ERROR:  canceling statement due to lock timeout
CONTEXT:  while updating tuple (0,2) in relation "wallets"
-- [lab] SQLSTATE 55P03
```

Điều dễ quên: `55P03` huỷ **câu lệnh**, nhưng giao dịch quanh nó đã hỏng. Thí nghiệm §2 cho B trừ ví 1004 trước rồi mới vấp `lock_timeout`; câu `SELECT` kế tiếp nhận `25P02 current transaction is aborted`, `COMMIT` trả `ROLLBACK`, và ví 1004 vẫn **500000** — khoản trừ đã mất theo. Ứng dụng phải ROLLBACK rồi quyết định: báo lỗi «hệ thống bận» hay thử lại cả giao dịch sau một khoảng chờ.

Theo tài liệu PostgreSQL 18: mặc định 0 (tắt); giới hạn áp riêng cho **từng lần** xin khoá, cả khoá tường minh (`LOCK TABLE`, `SELECT … FOR UPDATE`) lẫn khoá ngầm của `UPDATE`; không nên đặt trong `postgresql.conf` vì ảnh hưởng mọi phiên; và nếu `statement_timeout` khác 0 thì đặt `lock_timeout` bằng hoặc lớn hơn nó là vô nghĩa. Đặt theo phiên/giao dịch (`SET LOCAL`) hoặc theo role của ứng dụng.

`lock_timeout` không xoá deadlock: nếu nó nhỏ hơn `deadlock_timeout`, một vòng chờ sẽ hiện ra dưới dạng `55P03` thay vì `40P01` — sớm hơn, nhưng mất dấu hiệu «đây là deadlock» trong log.

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `40P01` lác đác giữa hai nghiệp vụ «ngược chiều» | Hai đường code khoá cùng tập dòng theo thứ tự khác nhau | Khoá trước theo thứ tự toàn cục (id tăng dần) |
| Thử lại chỉ câu lệnh lỗi, số dư sai | Phần đầu giao dịch đã bị huỷ cùng lúc | Thử lại toàn bộ giao dịch |
| Đọc lỗi không biết câu nào đâm câu nào | Client chỉ nhận PID và xid | Đọc nhật ký server; bật `log_lock_waits` nếu cần |
| Sửa bằng `UPDATE … WHERE id IN (a, b)` vẫn deadlock | Thứ tự khoá theo thứ tự quét | `SELECT … ORDER BY id FOR NO KEY UPDATE` trước |
| Sau `55P03` mọi câu báo `25P02` | Giao dịch đã hỏng | ROLLBACK rồi thử lại/báo lỗi |

## 12. Đánh đổi

- **Khoá trước toàn bộ hay khoá dần:** khoá trước mọi dòng sẽ đụng làm giao dịch giữ khoá lâu hơn một chút và phải biết trước tập dòng; đổi lại không có vòng chờ.
- **Thử lại hay sửa thứ tự:** thử lại là bắt buộc (deadlock có thể đến từ chỗ bạn chưa nghĩ tới), nhưng chỉ dựa vào thử lại thì trả giá bằng độ trễ — K037 cho thấy gần một lần làm lại cho mỗi giao dịch.
- **`lock_timeout` ngắn hay dài:** ngắn thì người dùng nhận lỗi sớm, nhưng giao dịch dài hợp lệ cũng bị cắt; dài thì kết nối đứng chờ, dễ cạn pool khi có sự cố.
- **Giao dịch ngắn:** giữ khoá trong lúc chờ người dùng hay gọi dịch vụ ngoài làm mọi kịch bản trên tệ hơn.

## 13. Bài tập

**Bài 1.** Viết giao dịch chuyển `:so_tien` từ ví `:nguon` sang ví `:dich` (hai số bất kỳ) sao cho hai lần chuyển ngược chiều chạy cùng lúc không thể deadlock.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">BEGIN;
SELECT id FROM wallets WHERE id IN (:nguon, :dich) ORDER BY id FOR NO KEY UPDATE;
UPDATE wallets SET balance = balance - :so_tien, updated_at = now() WHERE id = :nguon;
UPDATE wallets SET balance = balance + :so_tien, updated_at = now() WHERE id = :dich;
COMMIT;</code></pre>
<p>Câu khoá không phụ thuộc chiều chuyển: luôn khoá id nhỏ trước. Bên đến sau chờ ở ví đầu tiên khi chưa giữ gì, nên không có vòng. Lab §2 chạy đúng mẫu này (460000 / 240000), K037-theo-id cho 0 lần <code>40P01</code> qua 3 lần × 1.200 giao dịch. Vẫn bọc giao dịch trong vòng thử lại: <code>ck_wallets_balance_non_negative</code> hay lỗi khác vẫn có thể xảy ra, và deadlock có thể đến từ đường code khác.</p>
</details>

**Bài 2.** Một đồng nghiệp đặt trong `postgresql.conf`: `lock_timeout = 2s`, `deadlock_timeout = 5s`, «để bớt deadlock». Chuyện gì xảy ra với vòng chờ ở mục 3, và với một lệnh `ALTER TABLE` bảo trì ban đêm?

<details><summary><b>Lời giải</b></summary>
<p>Vòng chờ vẫn hình thành y như cũ. Sau 2 giây, phiên nào chờ đủ trước sẽ nhận <code>55P03</code> chứ không phải <code>40P01</code> — bộ dò (5 s) không bao giờ kịp chạy, nên log không còn dòng «deadlock detected» kèm câu lệnh của các bên, việc tìm hai đường code đâm nhau khó hơn. Số deadlock không giảm; chỉ đổi tên lỗi. Đặt trong <code>postgresql.conf</code> ảnh hưởng mọi phiên — kể cả <code>ALTER TABLE</code> bảo trì, lệnh này sẽ bỏ cuộc sau 2 giây chờ khoá dù người vận hành muốn nó chờ. Tài liệu PostgreSQL khuyên không đặt <code>lock_timeout</code> ở cấp server; đặt theo phiên, giao dịch (<code>SET LOCAL</code>) hoặc role.</p>
</details>

**Bài 3.** Ứng dụng bắt được `55P03` ở câu cộng tiền, ghi log rồi chạy tiếp câu kế tiếp và `COMMIT`. Dự đoán kết quả và số dư.

<details><summary><b>Lời giải</b></summary>
<p>Câu kế tiếp nhận <code>25P02</code> «current transaction is aborted, commands ignored until end of transaction block»; <code>COMMIT</code> trả thẻ <code>ROLLBACK</code>; mọi thứ giao dịch đã ghi trước đó cũng bị huỷ. Thí nghiệm §2 (<code>p04/thu-nghiem/pg037-vong-ba-phien-lock-timeout.sql</code>, không có trong lab chính): B trừ ví 1004 xuống 495000, vấp <code>lock_timeout</code> ở ví 1002, cuối cùng 1004 vẫn 500000. Không mất tiền, nhưng nếu ứng dụng chỉ nhìn «COMMIT không lỗi» thì sẽ báo thành công sai.</p>
</details>

## 14. Đọc tiếp

- [PostgreSQL 18 · Explicit Locking — Deadlocks](https://www.postgresql.org/docs/18/explicit-locking.html): ví dụ hai tài khoản, lời khuyên thứ tự khoá nhất quán.
- [PostgreSQL 18 · Lock Management](https://www.postgresql.org/docs/18/runtime-config-locks.html): `deadlock_timeout`.
- [PostgreSQL 18 · Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html): `lock_timeout`, `statement_timeout`.
- [PostgreSQL 18 · Serialization Failure Handling](https://www.postgresql.org/docs/18/mvcc-serialization-failure-handling.html): thử lại `40001`, `40P01`.
- [PostgreSQL 18 · Error Reporting and Logging](https://www.postgresql.org/docs/18/runtime-config-logging.html): `log_lock_waits`.
- Lab: `examples/toi-noi-postgresql/p04/37-deadlock-thu-tu-khoa.sql`, kết quả `p04/OUTPUT.txt`, phép thử K037 trong `p04/DO-DAC.md`; thí nghiệm `p04/thu-nghiem/pg037-vong-ba-phien-lock-timeout.sql`.

Bài tiếp theo: [PG-038 · Serializable và retry toàn giao dịch](/articles/postgresql-serializable-retry).

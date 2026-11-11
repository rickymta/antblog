---
slug: postgresql-serializable-retry
title: "Serializable và retry toàn giao dịch: giữ quy tắc «mỗi khu luôn còn một tài xế trực»"
summary: "Hai tài xế cùng khu bấm «Xin nghỉ» cùng lúc, mỗi người thấy còn 2 người trực nên được nghỉ — khu trống, không lỗi nào báo. Tái hiện write skew, để SERIALIZABLE bắt bằng 40001 ở COMMIT, viết retry có giới hạn, đo báo động giả."
category: postgresql
level: advanced
tags: [postgresql, pg-038, serializable, ssi, write-skew, retry, isolation, toi-noi]
language: vi
---

Điều phối Tới Nơi có một quy tắc cứng: **mỗi khu lúc nào cũng còn ít nhất một tài xế trực**, để khách đặt xe giờ thấp điểm vẫn có người nhận. Ứng dụng tài xế có nút «Xin nghỉ»: server đếm số người đang trực trong khu, còn từ 2 trở lên thì tắt trực của người xin. Khu SG-TT-01 (mã 111) có hai người trực, tài xế 9101 và 9102. Một trưa mưa, cả hai bấm «Xin nghỉ» cách nhau vài mili giây. Cả hai được duyệt. Khu 111 trống trơn — và không một dòng log lỗi nào.

Hai giao dịch không sửa cùng một dòng, nên không ai chờ ai, không có `lost update` như PG-034. Mỗi giao dịch, chạy một mình, đều đúng. Đây là **write skew** — loại bất thường mà Repeatable Read để lọt. Bài này tái hiện nó, cho mức **SERIALIZABLE** bắt nó, rồi đi vào phần khó thật sự: vòng **retry toàn giao dịch** có giới hạn, và cái giá của «báo động giả».

<div class="tkh-callout">
<span class="tkh-callout-title">PG-038 · Phần 4/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-isolation-read-committed-repeatable-read">PG-034 · Read Committed và Repeatable Read</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>38-serializable-retry.sql</code>, phép thử đồng thời K038 trong <code>DO-DAC.md</code>, hàm thử lại <code>giaoDich()</code> trong <code>pg/lib/dong-thoi.mjs</code>) và thí nghiệm <code>p04/thu-nghiem/pg038-siread-retry-sai-muc-tron.sql</code>. Sau bài này, bạn tái hiện được xung đột tuần tự hoá, viết và kiểm vòng retry có giới hạn, và chứng minh bất biến vẫn đúng khi nhiều phiên chạy đua.</p>
</div>

## 1. Dữ liệu và cách chạy

Lab chạy trên **PostgreSQL 18.1 thật**, cluster tạm tự dựng và tự dọn; mỗi phiên `A`, `B`, `C` là một kết nối psql riêng.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 38
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg038-siread-retry-sai-muc-tron.sql
```

Bảng `driver_duty(driver_id PK, area_id, on_duty, changed_at)` có 24 tài xế, **mỗi khu đúng 2 người trực**, index `ix_driver_duty_area (area_id)`. Khu 111: 9101 (phiên A) và 9102 (phiên B). Bất biến được đo bằng một hàm của tệp lab:

```sql
CREATE FUNCTION khu_vi_pham() RETURNS bigint LANGUAGE sql STABLE AS $$
  SELECT count(*) FROM (SELECT area_id FROM driver_duty GROUP BY area_id
                        HAVING count(*) FILTER (WHERE on_duty) = 0) AS k
$$;
```

Giao dịch «xin nghỉ» của mỗi tài xế: đếm `SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty`, nếu ≥ 2 thì `UPDATE driver_duty SET on_duty = false … WHERE driver_id = <mình>`.

## 2. Write skew: Repeatable Read để lọt

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN ISOLATION LEVEL REPEATABLE READ;` đếm khu 111 | **2** |
| T2 | B | `BEGIN ISOLATION LEVEL REPEATABLE READ;` đếm khu 111 | **2** |
| T3 | A | tắt trực 9101 | `UPDATE 1` |
| T4 | B | tắt trực 9102 — dòng khác, không chờ | `UPDATE 1` |
| T5 | A | `COMMIT` | `COMMIT` |
| T6 | B | `COMMIT` | `COMMIT` |
| T7 | C | đếm khu 111, `khu_vi_pham()` | **0 \| 1** |

PG-034 đã cho thấy Repeatable Read báo `40001` khi hai giao dịch **cùng sửa một dòng**. Ở đây chúng sửa hai dòng khác nhau; điều mỗi bên dựa vào để quyết định — «khu còn 2 người» — lại nằm ở **dòng của bên kia**. Snapshot của A không thấy B đổi 9102, snapshot của B không thấy A đổi 9101; không có xung đột ghi–ghi nào để phát hiện. Khoá dòng cũng không giúp gì: không ai khoá dòng của người kia.

**Write skew** (lệch ghi): hai giao dịch đọc cùng một tập dữ liệu, mỗi bên ghi một phần khác nhau dựa trên những gì đã đọc, và kết quả chung vi phạm một ràng buộc mà từng bên đều giữ đúng. Ràng buộc kiểu «ít nhất một dòng thoả điều kiện» không viết được bằng `CHECK` hay `UNIQUE`, nên database không có chỗ nào tự chặn.

## 3. SERIALIZABLE: cùng lịch, một bên bị huỷ ở COMMIT

Mức **SERIALIZABLE** của PostgreSQL bảo đảm: mọi tập giao dịch Serializable đã commit đồng thời đều có cùng hiệu ứng như khi chạy **lần lượt từng cái một**. Không có thứ tự lần lượt nào cho ra «khu 111 trống» (ai chạy sau cũng sẽ đếm được 1 và bị từ chối), nên một trong hai phải thất bại.

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN ISOLATION LEVEL SERIALIZABLE;` đếm | 2 |
| T2 | B | `BEGIN ISOLATION LEVEL SERIALIZABLE;` đếm | 2 |
| T3 | C | `SELECT * FROM lab_khoa() WHERE che_do = 'SIReadLock'` | A, B: `relation driver_duty SIReadLock` |
| T4 | A | tắt trực 9101 | `UPDATE 1` |
| T5 | B | tắt trực 9102 — **không chờ** | `UPDATE 1` |
| T6 | A | `COMMIT` | `COMMIT` |
| T7 | B | `COMMIT` | **`40001`** |
| T8 | C | đếm khu 111, `khu_vi_pham()` | **1 \| 0** |

```text
ERROR:  could not serialize access due to read/write dependencies among transactions
DETAIL:  Reason code: Canceled on identification as a pivot, during commit attempt.
HINT:  The transaction might succeed if retried.
-- [lab] SQLSTATE 40001
```

PostgreSQL cài mức này bằng **SSI** (Serializable Snapshot Isolation): đọc bằng snapshot y như Repeatable Read, cộng thêm việc **ghi nhớ mỗi giao dịch đã đọc những gì** bằng khoá vị từ `SIReadLock` (predicate lock — khoá trên «vùng dữ liệu đã đọc»). Khoá này **không chặn ai** — tài liệu nói rõ nó không gây chờ và không thể góp phần vào deadlock — nó chỉ để phát hiện khi một giao dịch ghi vào thứ mà giao dịch đồng thời khác đã đọc. Đó là một **phụ thuộc đọc–ghi** (rw-dependency): A đọc dòng 9102 mà B sau đó ghi, B đọc dòng 9101 mà A ghi. Khi thấy chuỗi phụ thuộc đọc–ghi có thể tạo vòng không tuần tự hoá được, PostgreSQL huỷ một giao dịch; `pivot` là giao dịch đứng giữa chuỗi đó.

Ba điều cần khắc vào code ứng dụng:

1. **Lỗi rơi ở `COMMIT`.** Câu `UPDATE` của B chạy xong ngay; chỉ khi B commit, sau A, mới bị từ chối. Code coi `COMMIT` là «chắc chắn thành công» sẽ bỏ sót lỗi này.
2. **Lỗi cũng có thể rơi ở câu lệnh.** Thí nghiệm §2: hai giao dịch Serializable **cùng sửa** dòng 9101 ⇒ B chờ A, A commit ⇒ câu `UPDATE` của B lỗi `40001 could not serialize access due to concurrent update` — giống Repeatable Read. Mọi câu trong giao dịch đều có thể ném `40001`.
3. **Mã lỗi luôn là `40001`** cho mọi lỗi tuần tự hoá, theo tài liệu — đó là thứ duy nhất ứng dụng cần so.

## 4. Retry: chạy lại TOÀN BỘ giao dịch

Lab cho B làm lại từ đầu, sau khi A đã commit:

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T9 | B | `BEGIN ISOLATION LEVEL SERIALIZABLE;` đếm | **1** |
| T10 | B | 1 < 2 ⇒ ứng dụng từ chối, `ROLLBACK` | không ghi gì |

Lần thử thứ hai cho **kết quả nghiệp vụ khác**: từ «được nghỉ» thành «từ chối». Đó là lý do phải chạy lại cả phần đọc và phần quyết định, không chỉ phát lại câu ghi. Tài liệu PostgreSQL viết: phải thử lại **toàn bộ** giao dịch, gồm cả logic quyết định chạy câu SQL nào với giá trị nào; PostgreSQL không có cơ chế tự thử lại vì nó không thể bảo đảm đúng.

Thí nghiệm §4 làm điều ngược lại — «retry» bằng cách mở giao dịch mới và chỉ gửi lại `UPDATE … 9102`: không có ai chạy song song nên `COMMIT` thành công, và khu 111 lại về **0** người trực. Retry sai biến một lỗi ồn ào (`40001`) thành một lỗi im lặng.

Phép thử đồng thời của lab dùng hàm `giaoDich()` trong `pg/lib/dong-thoi.mjs` (rút gọn, giữ nguyên logic):

```js
export async function giaoDich(w, fn, {
  muc = 'READ COMMITTED', toi_da = 1, thu_lai_khi = ['40001', '40P01'], co_so_ms = 2, tran_ms = 100,
  dem = null, ngau_nhien = Math.random,
} = {}) {
  for (let lan = 1; ; lan++) {
    try {
      await w.qx(`BEGIN ISOLATION LEVEL ${muc};`);
      const kq = await fn(w, lan);                       // đọc + quyết định + ghi, chạy lại mỗi lần
      if (kq?.huy) { await w.qx('ROLLBACK;'); return { ok: true, lan, kq }; }
      await w.qx('COMMIT;');                             // 40001 có thể ném ra ở đây
      return { ok: true, lan, kq };
    } catch (e) {
      if (!(e instanceof LoiSql)) throw e;               // lỗi lập trình: không thử lại
      if (!e.connLost) await w.q('ROLLBACK;');
      dem?.tang(`loi_${e.sqlstate}`);
      if (thu_lai_khi.includes(e.sqlstate) && lan < toi_da) {
        dem?.tang('thu_lai');
        await sleep(Math.floor(ngau_nhien() * Math.min(tran_ms, co_so_ms * 2 ** lan)));
        continue;
      }
      if (thu_lai_khi.includes(e.sqlstate)) dem?.tang('het_luot_thu');
      return { ok: false, lan, sqlstate: e.sqlstate };
    }
  }
}
```

Đọc từng quyết định thiết kế:

- **`fn` là cả giao dịch**: câu đếm, phép so ≥ 2 và câu `UPDATE` đều nằm trong `fn`, nên mỗi lần thử đọc lại dữ liệu mới. Nghiệp vụ từ chối thì `fn` trả `{ huy: true }` ⇒ `ROLLBACK` — kết quả hợp lệ, không phải lỗi.
- **`COMMIT` nằm trong `try`**: lỗi `40001` ở COMMIT đi đúng đường thử lại.
- **Chỉ thử lại `40001` và `40P01`**: mọi SQLSTATE khác (vi phạm ràng buộc, lỗi cú pháp) trả về ngay. Tài liệu cho phép cân nhắc thử lại cả `23505`/`23P01` trong một số trường hợp, nhưng cảnh báo chúng có thể là lỗi lâu dài.
- **Giới hạn số lần** (`toi_da`): hết lượt thì trả lỗi lên trên (đếm `het_luot_thu`) thay vì quay vô hạn. Tài liệu lưu ý một lần thử lại không bảo đảm thành công, tranh chấp cao có thể cần nhiều lần.
- **Backoff «full jitter»**: chờ ngẫu nhiên trong [0, min(trần, cơ_số · 2^lần)]. Phần luỹ thừa giãn dần các lần thử; phần ngẫu nhiên làm các phiên vừa đâm nhau không cùng quay lại đúng một thời điểm để đâm nhau lần nữa. K038 dùng cơ số 2 ms, trần 50 ms, tối đa 10 lần.
- **Không gì ngoài database trong `fn`**: gửi SMS hay gọi API trong giao dịch rồi thử lại là gửi hai lần. Tác dụng phụ ra ngoài thuộc về PG-039.

## 5. Báo động giả: Seq Scan khoá cả bảng

SSI bảo thủ: nó có thể huỷ giao dịch mà chạy tuần tự thật cũng không hề xung đột. Lab §3 cho A xin nghỉ ở khu **112** (9103), B ở khu **113** (9105) — hai khu không liên quan.

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `SERIALIZABLE`; đếm khu 112; tắt trực 9103 | 2 · `UPDATE 1` |
| T2 | B | `SERIALIZABLE`; đếm khu 113; tắt trực 9105 | 2 · `UPDATE 1` |
| T3 | A | `COMMIT` | `COMMIT` |
| T4 | B | `COMMIT` | **`40001`** — dù khác khu |

Nguyên nhân nằm ở mục 3, bước T3: khoá đọc là `relation driver_duty` — **cả bảng**. Bảng 24 dòng nên planner chọn Seq Scan, và theo tài liệu PostgreSQL, quét tuần tự **luôn** cần khoá vị từ mức bảng. Với SSI, «A đọc cả bảng» nghĩa là mọi dòng B ghi đều tính là thứ A đã đọc.

Cùng lịch, nhưng hai phiên `SET enable_seqscan = off` để truy vấn đi qua `ix_driver_duty_area`:

```text
 phien | loai  |     doi_tuong      |   che_do   | da_cap
-------+-------+--------------------+------------+--------
 A     | tuple | driver_duty (0,35) | SIReadLock | t
 B     | tuple | driver_duty (0,37) | SIReadLock | t
(2 rows)
```

Hết khoá mức bảng ⇒ **cả hai COMMIT**, khu 112 và 113 mỗi khu còn 1 người. `lab_khoa()` lọc bỏ index nên chỉ thấy dòng; thí nghiệm §1 đọc thẳng `pg_locks` (database mới, chưa ai sửa nên 9103/9104 nằm ở `(0,3)`/`(0,4)`):

```text
 phien | locktype |       relname       | page | tuple
-------+----------+---------------------+------+-------
 A     | page     | ix_driver_duty_area |    1 |  NULL
 A     | tuple    | driver_duty         |    0 |     3
 A     | tuple    | driver_duty         |    0 |     4
(3 rows)
```

Đó là «vùng đã đọc» khi đi index: page index chứa khoá `112` và hai dòng đã đọc. Sau khi A tự `UPDATE` dòng 9103, `pg_locks` không còn khoá đọc trên `(0,3)` mà thêm `page pk_driver_duty 1` (câu `UPDATE` tìm dòng qua khoá chính) — mỗi bên chỉ còn một dòng trong `lab_khoa()` vì lý do đó.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">enable_seqscan = off là công cụ lab, không phải cấu hình</span>
<p>Trên bảng lớn, planner tự chọn index cho truy vấn chọn lọc. Tài liệu PostgreSQL gợi ý khuyến khích index scan bằng cách giảm <code>random_page_cost</code> hoặc tăng <code>cpu_tuple_cost</code>, và tăng <code>max_pred_locks_per_transaction</code> / <code>max_pred_locks_per_relation</code> / <code>max_pred_locks_per_page</code> khi khoá vị từ bị gộp từ dòng lên page lên bảng — gộp lên bảng cũng làm tăng báo động giả. Bảng nhỏ «nóng» như <code>driver_duty</code> là chỗ báo động giả dễ xảy ra nhất.</p>
</div>

## 6. Bằng chứng khi chạy đua thật: K038

Phép thử K038 (trích `p04/DO-DAC.md`): i7-10700 (16 luồng), 64 GB RAM, NVMe Kingston SA2000, Windows 11 Pro, Node 22.18, PostgreSQL 18.1, tải CPU nền ~6 %. **8 phiên** = 8 tài xế 9101–9108 ở 4 khu (2 người mỗi khu), **200 vòng**; mỗi vòng đặt lại mọi người trực rồi cả 8 cùng xin nghỉ (có «rào»: chờ nhau xong mới sang vòng sau). Thử lại `40001` tối đa 10 lần, full jitter trần 50 ms. Mỗi kịch bản **3 lần**, mỗi lần một database mới.

| Kịch bản | Giây/lần (min–max) | Được nghỉ / từ chối | Khu-vòng vi phạm | Lần `40001` |
|---|---|---|---|---|
| REPEATABLE READ | 0,87–0,90 | 1.600 / 0 | **800** | — |
| SERIALIZABLE (Seq Scan) | 9,61–11,13 | 800 / 800 | **0** | 2.488–2.539 |
| SERIALIZABLE (ép index) | 3,18–3,53 | 800 / 800 | **0** | **800** (cả 3 lần) |

Đọc số:

- **Repeatable Read** vi phạm ở **mọi** khu, mọi vòng (4 × 200 = 800): cả 1.600 yêu cầu đều được duyệt. Con số tuyệt đối do «rào» ép mọi phiên cùng lúc — ngoài đời bấm không đồng thời đến vậy, đó cũng là lý do lỗi kiểu này lọt qua test tay.
- **Serializable + retry** đúng ở mọi lần: 0 vi phạm, 0 giao dịch bỏ cuộc, mỗi khu mỗi vòng đúng một người được nghỉ (800) và một người bị từ chối **sau khi thử lại** (800).
- **Ép index**: đúng 800 lần `40001` = một xung đột **thật** mỗi khu mỗi vòng. **Seq Scan**: ~2.500 lần — khoảng 1.700 lần là báo động giả giữa các khu, và mỗi vòng chậm hơn tương ứng.

Thời gian chỉ để so tương đối trong cùng bảng, trên một máy; không phải thông lượng thật.

## 7. Cách khác: vật chất hoá xung đột bằng khoá dòng

Không muốn retry cho tình huống này? Biến xung đột ngầm thành xung đột trên **một dòng cụ thể**: mọi giao dịch đổi người trực của một khu đều khoá dòng của khu trong `areas` trước khi đếm. Lab §4, ở Read Committed:

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN; SELECT id FROM areas WHERE id = 111 FOR NO KEY UPDATE;` đếm | 111 · 2 |
| T2 | B | `BEGIN;` cùng câu khoá | ⏳ chờ A |
| T3 | A | tắt trực 9101, `COMMIT` | `COMMIT` |
| T4 | B | (T2 chạy tiếp) rồi đếm | ✔ 111 · **1** ⇒ từ chối, `ROLLBACK` |

B đếm **sau** khi A commit (Read Committed: mỗi câu một snapshot mới) nên thấy 1. Cuối bài `khu_vi_pham() = 0`.

| | SERIALIZABLE + retry | Khoá dòng khu |
|---|---|---|
| Bắt được write skew ở đường code bạn **quên** | Có, nếu mọi giao dịch liên quan đều Serializable | Không — đường nào quên khoá là lọt |
| Cần vòng retry | Có (bắt buộc) | Không cho xung đột này (vẫn nên có cho `40P01`) |
| Chi phí khi tranh chấp | Làm lại giao dịch, báo động giả | Xếp hàng chờ trên dòng khu |
| Kiểu lỗi | `40001` ồn ào | Quên khoá ⇒ sai im lặng |

## 8. Bẫy: trộn mức cô lập

Bảo đảm của SSI nói về tập giao dịch **Serializable**. Thí nghiệm §3 cho A chạy Serializable, còn B — như một script vận hành hay job quên đặt mức — chạy Read Committed:

| Bước | Phiên | Kết quả |
|---|---|---|
| T1 | A (`SERIALIZABLE`) đếm khu 111 | 2 |
| T2 | B (`READ COMMITTED`) đếm, tắt trực 9102, `COMMIT` | 2 · `UPDATE 1` · `COMMIT` |
| T3 | A tắt trực 9101, `COMMIT` | `UPDATE 1` · **`COMMIT`** |
| T4 | C đếm khu 111 | **0** |

Không lỗi nào: B không để lại dấu vết đọc–ghi cho SSI kiểm. Muốn dựa vào SERIALIZABLE thì mọi đường ghi liên quan phải chạy ở mức đó — cách chắc nhất là đặt `default_transaction_isolation = 'serializable'` cho role hoặc database của ứng dụng (PG-034), và kiểm bằng `SHOW transaction_isolation` trong test.

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Bất biến «ít nhất một» bị phá, không lỗi nào | Write skew ở Read Committed/Repeatable Read | SERIALIZABLE + retry, hoặc khoá dòng đại diện |
| `40001` ở COMMIT bị coi là lỗi hệ thống | Code không ngờ COMMIT có thể thất bại | Đặt COMMIT trong vòng retry |
| Retry xong bất biến vẫn sai | Chỉ phát lại câu ghi, không đọc lại | Chạy lại cả hàm giao dịch |
| Quá nhiều `40001` giữa các bản ghi không liên quan | Seq Scan ⇒ khoá vị từ cả bảng; khoá bị gộp | Index phù hợp, `max_pred_locks_*`, giao dịch ngắn |
| SERIALIZABLE vẫn để lọt | Một bên chạy mức thấp hơn | `default_transaction_isolation` cho mọi role ghi |
| Retry gửi SMS hai lần | Tác dụng phụ ngoài DB nằm trong giao dịch | Outbox (PG-039) |

## 10. Đánh đổi

- **Đúng đắn rộng hay chi phí thấp:** SERIALIZABLE bảo vệ cả bất biến bạn chưa liệt kê, đổi lại mọi giao dịch ghi phải chịu được `40001` và chậm hơn khi tranh chấp. Khoá dòng đại diện rẻ hơn nhưng chỉ bảo vệ chỗ bạn nhớ khoá.
- **Giới hạn retry:** quá thấp thì người dùng nhận lỗi khi tranh chấp ngắn; quá cao thì một điểm nóng kéo dài độ trễ. Với trần 50 ms và 10 lần, tổng thời gian chờ tối đa là 310 ms (bài tập 2) — cộng thời gian chạy lại giao dịch.
- **Giao dịch nhỏ:** tài liệu PostgreSQL khuyên khai báo `READ ONLY` khi có thể, giới hạn số kết nối đang chạy (pool), không đưa vào giao dịch nhiều hơn mức cần cho toàn vẹn, không để kết nối «idle in transaction». Lab chưa chạy `SERIALIZABLE READ ONLY DEFERRABLE` (chờ tới khi snapshot an toàn để không bao giờ bị huỷ) — chưa có số đo.

## 11. Bài tập

**Bài 1.** Sau khi B nhận `40001` ở COMMIT (mục 3), lập trình viên «thử lại» bằng `BEGIN ISOLATION LEVEL SERIALIZABLE; UPDATE … 9102; COMMIT;`. Dự đoán COMMIT và số người trực khu 111.

<details><summary><b>Lời giải</b></summary>
<p>COMMIT thành công — không còn giao dịch nào chạy song song để SSI thấy xung đột — và khu 111 còn <b>0</b> người trực. Quyết định «được nghỉ» dựa trên lần đếm 2 của giao dịch đã bị huỷ; lần thử lại không đếm lại. Thí nghiệm §4 của <code>p04/thu-nghiem/pg038-siread-retry-sai-muc-tron.sql</code> (không có trong lab chính) chạy đúng lịch này. Bản đúng: chạy lại cả câu đếm, so ≥ 2 rồi mới ghi — lần đếm thứ hai ra 1, ứng dụng từ chối.</p>
</details>

**Bài 2.** Với `giaoDich()` cấu hình như K038 (`co_so_ms = 2`, `tran_ms = 50`, `toi_da = 10`), một giao dịch xui xẻo bị `40001` ở cả 10 lần. Nó chờ tối đa bao lâu cộng dồn, và kết quả trả về là gì?

<details><summary><b>Lời giải</b></summary>
<p>Chỉ chờ sau lần 1 tới lần 9 (lần 10 lỗi là hết lượt, không chờ). Trần mỗi lần là min(50, 2 · 2^lần): 4, 8, 16, 32, 50, 50, 50, 50, 50 ms ⇒ tối đa <b>310 ms</b>, thực tế trung bình khoảng một nửa vì chờ ngẫu nhiên trong [0, trần). Kết quả <code>{ ok: false, lan: 10, sqlstate: '40001' }</code> và bộ đếm <code>het_luot_thu</code> tăng 1 — ứng dụng phải báo lỗi cho tài xế, không được coi là «đã nghỉ». Trong K038 bộ đếm này bằng 0 ở cả 3 lần.</p>
</details>

**Bài 3.** Một job tối «cân bằng tài xế» chạy Read Committed và tắt trực tài xế ở khu đang có 2 người, cùng lúc ứng dụng (Serializable) duyệt «xin nghỉ» ở cùng khu. SERIALIZABLE có cứu được không? Sửa thế nào?

<details><summary><b>Lời giải</b></summary>
<p>Không. Thí nghiệm §3 cho đúng tình huống: giao dịch Read Committed không để lại dấu vết đọc cho SSI, hai bên cùng commit, khu còn 0 người. Sửa một trong hai cách: cho job chạy <code>SERIALIZABLE</code> (đặt <code>default_transaction_isolation</code> cho role của job) và bọc trong vòng retry; hoặc mọi đường đổi người trực — ứng dụng lẫn job — đều khoá dòng khu <code>SELECT … FROM areas WHERE id = :khu FOR NO KEY UPDATE</code> trước khi đếm (mục 7).</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Transaction Isolation — Serializable](https://www.postgresql.org/docs/18/transaction-iso.html): SSI, `SIReadLock`, độ mịn khoá vị từ, danh sách gợi ý hiệu năng.
- [PostgreSQL 18 · Serialization Failure Handling](https://www.postgresql.org/docs/18/mvcc-serialization-failure-handling.html): SQLSTATE nên thử lại, thử lại toàn bộ giao dịch.
- [PostgreSQL 18 · Explicit Locking](https://www.postgresql.org/docs/18/explicit-locking.html): khoá dòng dùng ở mục 7.
- [PG-037 · Deadlock, thứ tự lấy khoá và timeout](/articles/postgresql-deadlock-lock-order-timeout): `40P01` dùng chung vòng retry.
- Lab: `examples/toi-noi-postgresql/p04/38-serializable-retry.sql`, kết quả `p04/OUTPUT.txt`, K038 trong `p04/DO-DAC.md`, hàm `giaoDich()` ở `pg/lib/dong-thoi.mjs`; thí nghiệm `p04/thu-nghiem/pg038-siread-retry-sai-muc-tron.sql`.

Bài tiếp theo: [PG-039 · Idempotency và transactional outbox](/articles/postgresql-idempotency-outbox-transaction).

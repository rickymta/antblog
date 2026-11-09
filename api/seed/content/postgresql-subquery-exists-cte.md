---
slug: postgresql-subquery-exists-cte
title: "Subquery, EXISTS và CTE: tách truy vấn đối soát thành bước kiểm chứng được"
summary: "Câu đối soát tháng 10 của Tới Nơi báo 7 chuyến lệch tiền nhưng không ai chứng minh được nó đúng, còn câu «tài xế chưa chạy chuyến nào» trả 0 dòng dù có hai người như vậy. Học subquery vô hướng và tương quan, IN/EXISTS, bẫy NOT IN với NULL, LATERAL, và CTE có câu kiểm kê cho từng bước."
category: postgresql
level: intermediate
tags: [postgresql, pg-014, subquery, exists, not-in, cte, lateral, toi-noi]
language: vi
---

Cuối tháng 10, đội tài chính Tới Nơi chạy câu đối soát «chuyến nào có tiền thực thu khác tiền phải thu»: ba tầng subquery lồng nhau, trả về **7 chuyến**. Kế toán trưởng hỏi: phạm vi gồm bao nhiêu chuyến? Bước ghép với giao dịch có làm mất hay nhân đôi chuyến nào không? 7 dòng có giải thích hết chênh lệch của cả tháng không? Không ai trả lời được — câu lệnh chỉ có một đầu ra.

Cùng tuần, bộ phận tuyển dụng hỏi «tài xế nào đã đăng ký nhưng chưa chạy chuyến nào» để gọi hỗ trợ. Câu `NOT IN` của họ trả **0 dòng**; họ kết luận ai cũng đã chạy. Thật ra có hai tài xế như vậy.

Bài này đi qua các dạng subquery, phân biệt `IN` với `EXISTS`, giải thích vì sao `NOT IN` gặp NULL thì im lặng sai, rồi viết lại câu đối soát bằng **CTE** — mỗi bước có tên và có một con số để đối chiếu.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-014 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-null-case-coalesce">PG-011 · NULL, CASE và COALESCE</a>, <a href="/articles/postgresql-join-quan-he-cardinality">PG-013 · JOIN và số dòng sau khi nối bảng</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>14-subquery-exists-cte.sql</code>) và thí nghiệm thêm ở <code>p02/thu-nghiem/pg014-*.sql</code>. Sau bài này, bạn tách được một truy vấn đối soát thành các bước có câu kiểm cho từng bước.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2: ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**, 3.174 chuyến; kỳ tháng 10 có **3.168 chuyến đã kết thúc** (PG-012), 2.707 hoàn thành với cước 182.287.000 đ. `trips.driver_id` có 297 giá trị NULL (chuyến huỷ trước khi có người nhận, chuyến đang tìm tài xế). Thực thu một chuyến = Σ charge − Σ refund + Σ adjustment của giao dịch `succeeded`.

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/14-subquery-exists-cte.sql
```

Thí nghiệm `p02/thu-nghiem/pg014-kiem-ke-bat-loi.sql` và `p02/thu-nghiem/pg014-bai-tap.sql` chạy trên cùng nền: thay tệp cuối. Kết quả chép từ lần chạy thật trên PGlite 0.3.16 (PostgreSQL 17.5), bỏ dòng `-- [lab]`; chưa chạy trên server thật.

## 2. Subquery: một câu SELECT đặt trong câu khác

**Subquery** (truy vấn con) là một `SELECT` trong ngoặc, đặt trong câu lệnh khác. Đặt ở chỗ cần một giá trị (danh sách cột, `WHERE`, `FILTER`), nó là subquery **vô hướng** (scalar) và phải trả tối đa 1 dòng × 1 cột; sau `IN`/`EXISTS`, nó trả một tập dòng; trong `FROM`, nó là **bảng dẫn xuất** như bước gom giao dịch của PG-013.

Subquery vô hướng dùng như một hằng số: `SELECT round(avg(fare)) FROM trips WHERE status = 'completed'` ra **67339**. Khi subquery tham chiếu cột của câu bên ngoài, nó là **subquery tương quan** (correlated) — về nghĩa, được tính lại cho từng dòng bên ngoài. «Chuyến hoàn thành có cước cao hơn trung bình *của thành phố mình*»:

```sql
SELECT c.code AS tp,
       count(*) AS hoan_thanh,
       count(*) FILTER (WHERE t.fare > (SELECT avg(t2.fare) FROM trips AS t2
                                        WHERE t2.city_id = t.city_id AND t2.status = 'completed')) AS tren_tb_thanh_pho
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code COLLATE "C";
```

```text
 tp | hoan_thanh | tren_tb_thanh_pho
----+------------+-------------------
 DN |        389 |               190
 HN |        941 |               436
 SG |       1377 |               636
(3 rows)
```

`t2.city_id = t.city_id` là chỗ «tương quan»: `t` thuộc câu ngoài. Viết lại bằng cách gom trung bình theo thành phố trong `FROM` rồi nối (mẫu của PG-013) cho đúng ba con số này (câu 3 của `pg014-kiem-ke-bat-loi.sql`) — hai cách viết cùng nghĩa, kế hoạch thực thi để Phần 3.

Một subquery vô hướng trả **hơn một dòng** là lỗi lúc *chạy*, không phải lúc viết. Chuyến 100042 có hai giao dịch:

```sql
SELECT code, (SELECT p.amount FROM payments AS p WHERE p.trip_id = t.id) AS so_tien
FROM trips AS t
WHERE t.code = 100042;
```

```text
ERROR:  more than one row returned by a subquery used as an expression
```

SQLSTATE `21000`. Câu này chạy êm với mọi chuyến chỉ có một giao dịch — và vỡ vào ngày có chuyến đầu tiên trả bằng hai phương thức. Subquery vô hướng trả **0 dòng** thì không lỗi: giá trị là NULL. Muốn một giá trị từ phía «nhiều», nói rõ phép gộp (`sum`, `max`) hoặc dùng `LIMIT 1` kèm `ORDER BY` có nghĩa nghiệp vụ.

## 3. IN và EXISTS cho câu hỏi «có hay không»

«Bao nhiêu khách từng huỷ ít nhất một chuyến?» (câu 2 của lab):

```sql
SELECT (SELECT count(*) FROM customers AS c
         WHERE c.id IN (SELECT t.customer_id FROM trips AS t WHERE t.status = 'cancelled'))   AS dung_in,
       (SELECT count(*) FROM customers AS c
         WHERE EXISTS (SELECT 1 FROM trips AS t WHERE t.customer_id = c.id AND t.status = 'cancelled')) AS dung_exists;
```

```text
 dung_in | dung_exists
---------+-------------
     266 |         266
(1 row)
```

Nối `customers` với `trips` rồi đếm sẽ ra **461** — số *chuyến* huỷ, không phải số khách (`pg014-kiem-ke-bat-loi.sql`, câu 2). `IN` và `EXISTS` là **semi-join**: mỗi dòng bên ngoài được giữ hoặc bỏ, không bao giờ bị nhân bản, dù khớp bao nhiêu dòng bên trong.

- `x IN (subquery)` đúng khi `x` bằng ít nhất một giá trị subquery trả về.
- `EXISTS (subquery)` đúng khi subquery trả về ít nhất một dòng. Tài liệu PostgreSQL ghi rằng subquery thường chỉ chạy tới khi biết có ít nhất một dòng, không chạy hết; và vì chỉ «có dòng hay không» là quan trọng, quy ước hay dùng là `EXISTS (SELECT 1 …)`.

Với câu hỏi khẳng định như trên, hai cách cho cùng kết quả. Chúng khác nhau ở **phủ định**.

## 4. Bẫy NOT IN gặp NULL

Bộ phận tuyển dụng viết:

```sql
SELECT d.code FROM drivers AS d
WHERE d.id NOT IN (SELECT t.driver_id FROM trips AS t)
ORDER BY d.code;
```

```text
 code
------
(0 rows)
```

Không lỗi, không cảnh báo, không dòng nào. Nguyên nhân là logic ba giá trị của PG-011: `x NOT IN (a, b, NULL)` nghĩa là `x <> a AND x <> b AND x <> NULL`, mà `x <> NULL` ra NULL. Tài liệu PostgreSQL nói đúng điều đó: nếu không có giá trị nào bằng và có ít nhất một dòng bên phải là NULL, kết quả của `NOT IN` là **NULL, không phải true** — và `WHERE` loại dòng NULL. Câu 1 của tệp thí nghiệm kiểm trên hằng số: `1 IN (1, NULL)` ra `true`, `1 IN (2, NULL)` ra NULL, `1 NOT IN (1, NULL)` ra `false`, `1 NOT IN (2, NULL)` ra **NULL**, chỉ `1 NOT IN (2, 3)` ra `true`. Chỉ cần **một** trong 297 chuyến có `driver_id` NULL là mọi tài xế đều bị loại. `NOT EXISTS` không có vấn đề này, vì nó chỉ hỏi «có dòng nào khớp không»:

```sql
SELECT d.code, d.created_at FROM drivers AS d
WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.driver_id = d.id)
ORDER BY d.code;
```

```text
 code |       created_at
------+------------------------
 8219 | 2026-10-29 09:00:00+07
 8220 | 2026-10-29 09:00:00+07
(2 rows)
```

Hai tài xế Hà Nội đăng ký ngày 29/10 chưa nhận chuyến nào. `NOT IN` chỉ đúng khi subquery chắc chắn không trả NULL — thêm `WHERE t.driver_id IS NOT NULL` thì nó cũng ra 8219, 8220. Nhưng sự an toàn đó phụ thuộc vào việc người sửa câu sau này nhớ giữ dòng lọc. Quy tắc của Tới Nơi: **phủ định «không có» luôn viết bằng `NOT EXISTS`**.

## 5. LATERAL: N dòng đầu của mỗi nhóm

Subquery trong `FROM` bình thường được tính độc lập, không nhìn thấy bảng đứng trước nó. Từ khoá **`LATERAL`** cho phép nó tham chiếu cột của các mục `FROM` đứng trước — giống một subquery tương quan nhưng trả về *nhiều dòng, nhiều cột*. Dạng hay dùng là «top-N mỗi nhóm»: hai chuyến cước cao nhất mỗi thành phố (câu 4 của lab):

```sql
SELECT c.code AS tp, top.code, top.kind, top.fare
FROM cities AS c
CROSS JOIN LATERAL (
  SELECT t.code, t.kind, t.fare
  FROM trips AS t
  WHERE t.city_id = c.id AND t.status = 'completed'
  ORDER BY t.fare DESC, t.code
  LIMIT 2
) AS top
ORDER BY c.id, top.fare DESC, top.code;
```

```text
 tp |  code  | kind |  fare
----+--------+------+--------
 SG | 101601 | food | 150000
 SG | 101725 | food | 150000
 HN | 101699 | food | 149000
 HN | 101913 | food | 149000
 DN | 101355 | food | 150000
 DN | 101431 | food | 150000
(6 rows)
```

Ở cả ba thành phố, hai chuyến đầu cùng cước; `t.code` là tie-breaker (PG-007) nên «hai chuyến đầu» là xác định. Thành phố không có chuyến nào sẽ biến mất khỏi `CROSS JOIN LATERAL`; muốn giữ thì dùng `LEFT JOIN LATERAL (…) ON true`.

## 6. CTE: đặt tên cho từng bước đối soát

**CTE** (Common Table Expression) là các câu phụ khai trong `WITH` trước câu chính; tài liệu PostgreSQL mô tả chúng như những bảng tạm chỉ tồn tại trong một câu lệnh. Lợi ích ở đây không phải tốc độ mà là **mỗi bước có tên**, nên mỗi bước đếm được. Câu đối soát của tài chính viết lại thành bốn bước (câu 5 của lab):

```sql
WITH b1 AS (          -- phạm vi: chuyến đặt trong tháng 10 đã kết thúc
  SELECT t.id, t.code, t.status, t.fare,
         CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
  FROM trips AS t
  WHERE t.status IN ('completed', 'cancelled')
    AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
),
b2 AS (               -- thực thu mỗi chuyến, MỘT dòng mỗi chuyến
  SELECT p.trip_id,
         sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
  FROM payments AS p
  WHERE p.status = 'succeeded'
  GROUP BY p.trip_id
),
b3 AS (               -- ghép: phải giữ ĐÚNG số dòng của b1
  SELECT b1.*, coalesce(b2.thuc_thu, 0) AS thuc_thu
  FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id
),
b4 AS (               -- lệch: thực thu khác phải thu
  SELECT * FROM b3 WHERE thuc_thu <> phai_thu
)
SELECT (SELECT count(*) FROM b1)                 AS b1_pham_vi,
       (SELECT count(*) FROM b2)                 AS b2_chuyen_co_gd,
       (SELECT count(*) FROM b3)                 AS b3_sau_ghep,
       (SELECT count(DISTINCT id) FROM b3)       AS b3_chuyen_khac_nhau,
       (SELECT count(*) FROM b4)                 AS b4_lech;
```

```text
 b1_pham_vi | b2_chuyen_co_gd | b3_sau_ghep | b3_chuyen_khac_nhau | b4_lech
------------+-----------------+-------------+---------------------+---------
       3168 |            2688 |        3168 |                3168 |       7
(1 row)
```

Câu chính ở đây là một **câu kiểm kê**: nó không trả lời câu hỏi nghiệp vụ, nó in số dòng của từng bước. Mỗi số đối chiếu được với một điều đã biết:

- **b1 = 3.168** khớp số chuyến đã kết thúc trong kỳ của PG-012 — phạm vi đúng.
- **b3 = b1 = 3.168**, và số `id` khác nhau cũng 3.168 — bước ghép không mất, không nhân chuyến nào (b2 đã gom về một dòng mỗi chuyến, đúng bài học PG-013).
- **b2 = 2.688**: 3.168 − 2.688 = 480 chuyến không có giao dịch thành công; bài tập 3 tách con số này.

Giữ nguyên b1…b4 và thay câu chính bằng danh sách lệch cộng hai dòng tổng (câu tiếp theo của lab dùng `UNION ALL`):

```text
   ma    |  status   | phai_thu  | thuc_thu  | chenh
---------+-----------+-----------+-----------+--------
 100724  | completed |     76000 |         0 | -76000
 101084  | completed |     98000 |         0 | -98000
 101310  | completed |     85000 |    170000 |  85000
 101987  | cancelled |         0 |    -72000 | -72000
 102443  | completed |     54000 |     45000 |  -9000
 102642  | completed |     38000 |    380000 | 342000
 102739  | cancelled |         0 |     96000 |  96000
 TỔNG b4 |           |    351000 |    619000 | 268000
 TỔNG b3 |           | 182287000 | 182555000 | 268000
(9 rows)
```

Dòng kiểm cuối cùng trả lời câu hỏi thứ ba của kế toán: chênh của 7 dòng lệch (**+268.000 đ**) bằng đúng chênh của cả 3.168 chuyến — không còn khoản chênh nào nằm ngoài danh sách. Đọc từng dòng (thiếu thanh toán, thu trùng, hoàn không có thu, lệch số tiền, thu chuyến huỷ) là việc của PG-020.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Chuỗi CTE đối soát: b1 phạm vi 3.168 chuyến và b2 thực thu 2.688 chuyến ghép thành b3 3.168 dòng, rồi lọc thành b4 7 dòng lệch. Mỗi bước có một câu kiểm. Ba lỗi cố ý: quên GROUP BY ở b2 làm b3 thành 3.209 dòng, dùng JOIN thay LEFT JOIN làm b3 còn 2.688 dòng — cả hai bị câu kiểm bắt; quên lọc succeeded thì số dòng vẫn khớp nhưng b4 thành 102 dòng — câu kiểm số dòng không bắt được.">
<rect x="20" y="20" width="250" height="56" rx="8" class="f-pk-soft s-pk"/><text x="34" y="42" class="t-b t-pk">b1 · phạm vi · 3.168</text><text x="34" y="62" class="t-small">kiểm: = 3.168 của PG-012</text>
<rect x="20" y="96" width="250" height="56" rx="8" class="f-pk-soft s-pk"/><text x="34" y="118" class="t-b t-pk">b2 · thực thu · 2.688</text><text x="34" y="138" class="t-small">GROUP BY trip_id ⇒ 1 dòng/chuyến</text>
<rect x="340" y="56" width="250" height="56" rx="8" class="f-pk-soft s-pk"/><text x="354" y="78" class="t-b t-pk">b3 · b1 LEFT JOIN b2</text><text x="354" y="98" class="t-small">kiểm: 3.168 dòng = b1, 3.168 id</text>
<rect x="660" y="56" width="240" height="56" rx="8" class="f-phys-soft s-phys"/><text x="674" y="78" class="t-b t-phys">b4 · lệch · 7 dòng</text><text x="674" y="98" class="t-small">kiểm: Σ chênh = b3 = +268.000</text>
<line x1="270" y1="48" x2="340" y2="76" class="s-line" stroke-width="1.6"/>
<line x1="270" y1="124" x2="340" y2="96" class="s-line" stroke-width="1.6"/>
<line x1="590" y1="84" x2="660" y2="84" class="s-line" stroke-width="1.6"/>
<text x="20" y="190" class="t-b">Ba lỗi cố ý (p02/thu-nghiem/pg014-kiem-ke-bat-loi.sql)</text>
<rect x="20" y="204" width="280" height="76" rx="8" class="f-warn-soft s-warn"/><text x="34" y="226" class="t-b t-warn">A · b2 quên GROUP BY</text><text x="34" y="246" class="t-small">b3 = 3.209 ≠ 3.168</text><text x="34" y="266" class="t-small">câu kiểm bắt được</text>
<rect x="320" y="204" width="280" height="76" rx="8" class="f-warn-soft s-warn"/><text x="334" y="226" class="t-b t-warn">B · b3 dùng JOIN</text><text x="334" y="246" class="t-small">b3 = 2.688 ≠ 3.168, b4 còn 5 dòng</text><text x="334" y="266" class="t-small">câu kiểm bắt được</text>
<rect x="620" y="204" width="280" height="76" rx="8" class="f-surface s-dead dash"/><text x="634" y="226" class="t-b">C · b2 quên lọc succeeded</text><text x="634" y="246" class="t-small">số dòng vẫn khớp, b4 = 102</text><text x="634" y="266" class="t-small t-warn">kiểm số dòng KHÔNG bắt được</text>
</svg>
</div>
<figcaption><b>Hình 1 · Chuỗi CTE đối soát và câu kiểm của từng bước.</b> Câu kiểm số dòng bắt lỗi nở/mất dòng; lỗi trong định nghĩa nghiệp vụ cần một đối chiếu khác.</figcaption>
</figure>

## 7. Câu kiểm bắt được gì — và không bắt được gì

Tệp `pg014-kiem-ke-bat-loi.sql` cố ý làm hỏng chuỗi theo ba cách rồi chạy lại câu kiểm kê:

| Lỗi cố ý | b1 | b2 | b3 | id khác nhau ở b3 | b4 | Câu kiểm |
|---|---|---|---|---|---|---|
| A · b2 bỏ `GROUP BY` (mỗi giao dịch một dòng) | 3168 | 2729 | **3209** | 3168 | 86 dòng / 46 chuyến | b3 ≠ b1 ⇒ **bắt được** |
| B · b3 dùng `JOIN` thay `LEFT JOIN` | 3168 | 2688 | **2688** | 2688 | 5 (+442.000) | b3 ≠ b1 ⇒ **bắt được** |
| C · b2 bỏ `WHERE status = 'succeeded'` | 3168 | 2689 | 3168 | 3168 | **102** (+7.423.000) | mọi số dòng khớp ⇒ **không bắt được** |

A và B là lỗi *cấu trúc* (nở dòng, mất dòng — hai lỗi của PG-013), câu kiểm tóm được ngay. B nguy hiểm hơn vẻ ngoài: danh sách vẫn trông hợp lý (5 dòng) nhưng mất đúng hai chuyến thiếu thanh toán 100724 và 101084.

C là lỗi *định nghĩa*: cộng cả giao dịch `failed`/`pending` vào thực thu. Mọi số dòng vẫn khớp; b4 nhảy lên 102 — và 101084, chuyến thiếu tiền thật, lại *biến khỏi* danh sách vì lần thu thẻ `failed` được tính như đã thu. Bắt lỗi này cần đối chiếu nguồn độc lập (có **97** giao dịch không thành công: 96 `failed`, 1 `pending`). Câu kiểm chứng minh các bước **nhất quán với nhau**; định nghĩa có đúng nghiệp vụ không thì phải đối chiếu với đặc tả — việc của PG-020.

## 8. CTE có bị tính riêng không? MATERIALIZED và NOT MATERIALIZED

Theo release notes 12, trước bản này CTE luôn được tính riêng trước phần còn lại của câu; từ bản 12, CTE không đệ quy, không có tác dụng phụ và **chỉ được tham chiếu một lần** được tự động gộp vào câu ngoài. `MATERIALIZED` buộc tính riêng, `NOT MATERIALIZED` buộc gộp (có thể phải tính lặp nếu CTE được dùng nhiều lần). Câu 6 của lab chạy cùng truy vấn với hai lựa chọn:

```text
 materialized | not_materialized
--------------+------------------
            7 |                7
(1 row)
```

Kết quả giống nhau — đây là lựa chọn về **kế hoạch thực thi**, không về nghĩa. Lưu ý chuỗi b1…b4 ở mục 6: b1, b2, b3 được tham chiếu hơn một lần trong câu kiểm kê, nên theo quy tắc trên chúng mặc định không được gộp. Đo khác biệt bằng `EXPLAIN` là việc của Phần 3; ở Phần 2, đừng thêm `MATERIALIZED` khi chưa đo.

## 9. Chọn công cụ nào?

| Câu hỏi | Công cụ |
|---|---|
| «Có / không có» (khách từng huỷ, tài xế chưa chạy) | `EXISTS` / `NOT EXISTS` — không nhân dòng, an toàn với NULL |
| Một giá trị cho mỗi dòng | Subquery vô hướng, hoặc gom trước rồi nối (dễ kiểm số dòng) |
| Top-N mỗi nhóm | `LATERAL` (hoặc window function, PG-015) |
| Nhiều bước cần đối chiếu | CTE + câu kiểm kê |
| Cần cột của cả hai bảng, quan hệ đã kiểm | `JOIN` (PG-013) |

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `NOT IN` trả 0 dòng dù chắc chắn có kết quả | Subquery chứa NULL | `NOT EXISTS` |
| `21000 more than one row returned…` xuất hiện «tự nhiên» | Subquery vô hướng gặp dữ liệu một–nhiều | Gộp rõ (`sum`, `max`) hoặc `LIMIT 1` có `ORDER BY` |
| Đếm khách ra số chuyến | JOIN thay vì semi-join | `EXISTS` / `IN` |
| Danh sách lệch «hợp lý» nhưng thiếu ca nghiêm trọng | Bước ghép dùng `JOIN`, mất dòng | Câu kiểm b3 = b1 |

## 11. Đánh đổi

- **Subquery lồng hay CTE:** subquery lồng ngắn khi chỉ một tầng; từ hai tầng trở lên, CTE dài hơn nhưng đọc từ trên xuống và kiểm được từng bước.
- **Câu kiểm kê trong cùng truy vấn:** đếm đúng các bước đang chạy, đổi lại phải chép chuỗi `WITH` sang câu danh sách (PG-020 gom vào một VIEW).
- **`NOT IN` có lọc NULL hay `NOT EXISTS`:** cả hai đúng hôm nay; `NOT EXISTS` vẫn đúng khi ai đó xoá dòng lọc NULL.

## 12. Bài tập

**Bài 1.** Đếm khách chưa từng đặt chuyến nào bằng `NOT EXISTS`, rồi bằng `NOT IN`. Vì sao lần này `NOT IN` không bị bẫy? Có nên giữ nó không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT count(*) FROM customers AS c
WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.customer_id = c.id);</code></pre>
<p>Cả hai cách ra <b>25</b>. <code>NOT IN</code> đúng vì <code>trips.customer_id</code> là <code>NOT NULL</code> — tính đúng của câu dựa vào một ràng buộc ở bảng khác. <code>NOT EXISTS</code> đúng mà không cần dựa vào điều đó, nên vẫn là lựa chọn mặc định.</p>
</details>

**Bài 2.** Đếm chuyến hoàn thành chưa có đánh giá bằng `NOT EXISTS` và bằng `LEFT JOIN … IS NULL` (anti-join của PG-013). Hai cách có cùng kết quả không? Con số này khớp với số nào của PG-011?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT count(*) FROM trips AS t
WHERE t.status = 'completed'
  AND NOT EXISTS (SELECT 1 FROM trip_ratings AS r WHERE r.trip_id = t.id);</code></pre>
<p>Cả hai ra <b>816</b> — đúng nhóm «chưa đánh giá» của PG-011 (2.707 chuyến hoàn thành − 1.891 đánh giá). Với anti-join, kiểm <code>r.trip_id IS NULL</code>: <code>trip_id</code> là khoá chính nên chỉ NULL ở dòng do LEFT JOIN sinh ra.</p>
</details>

**Bài 3.** Thêm ba câu kiểm cho bước b2 của mục 6: (k1) `trip_id` trong b2 không trùng; (k2) mọi chuyến trong b2 thuộc phạm vi b1; (k3) số chuyến của b1 không có giao dịch thành công, tách theo trạng thái. Dự đoán k3 trước khi chạy.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">-- sau WITH b1 AS (…), b2 AS (…) như mục 6
SELECT (SELECT count(*) FROM (SELECT trip_id FROM b2 GROUP BY trip_id HAVING count(*) &gt; 1) AS x) AS k1_trung,
       (SELECT count(*) FROM b2 WHERE NOT EXISTS (SELECT 1 FROM b1 WHERE b1.id = b2.trip_id)) AS k2_ngoai_pham_vi,
       (SELECT count(*) FROM b1 WHERE NOT EXISTS (SELECT 1 FROM b2 WHERE b2.trip_id = b1.id)) AS k3_khong_gd;</code></pre>
<p>Kết quả: k1 = 0, k2 = 0, k3 = <b>480</b> = 447 chuyến huỷ không có giao dịch thành công + 33 chuyến hoàn thành (31 chuyến cước 0 đ, 100724, 101084). k2 = 0 nghĩa là b2 không có chuyến ngoài kỳ, nên 3.168 − 2.688 = 480 là đúng phép trừ. Lời giải đầy đủ, có cột tách theo trạng thái: <code>p02/thu-nghiem/pg014-bai-tap.sql</code>.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 17 · Subquery Expressions](https://www.postgresql.org/docs/17/functions-subquery.html): `EXISTS`, `IN`, `NOT IN` và kết quả khi gặp NULL.
- [PostgreSQL 17 · Scalar Subqueries](https://www.postgresql.org/docs/17/sql-expressions.html#SQL-SYNTAX-SCALAR-SUBQUERIES).
- [PostgreSQL 17 · LATERAL Subqueries](https://www.postgresql.org/docs/17/queries-table-expressions.html#QUERIES-LATERAL).
- [PostgreSQL 17 · WITH Queries (CTE)](https://www.postgresql.org/docs/17/queries-with.html) và [release notes 12.0](https://www.postgresql.org/docs/release/12.0/) (gộp CTE vào câu ngoài).
- Lab: `examples/toi-noi-postgresql/p02/14-subquery-exists-cte.sql`, kết quả đầy đủ ở `p02/OUTPUT.txt`; thí nghiệm ở `p02/thu-nghiem/pg014-*.sql`.

Bài tiếp theo: [PG-015 · Window function và cửa sổ tính toán](/articles/postgresql-window-function-frame).

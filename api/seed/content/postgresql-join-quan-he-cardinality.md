---
slug: postgresql-join-quan-he-cardinality
title: "JOIN và số dòng sau khi nối bảng: quan hệ một–nhiều không nhân đôi tổng tiền"
summary: "Dashboard mới của Tới Nơi báo doanh thu tháng 10 là 191.644.000 đ, trong khi bảng đối chiếu ghi 182.287.000 đ — chỉ vì thêm một dòng JOIN payments. Học INNER/LEFT JOIN, đếm số dòng trước và sau khi nối, nhận diện quan hệ một–nhiều và gom phía «nhiều» trước khi cộng tiền."
category: postgresql
level: beginner
tags: [postgresql, pg-013, join, left-join, cardinality, aggregate, toi-noi]
language: vi
---

Sáng 31/10, kế toán Tới Nơi mở dashboard mới «Doanh thu tháng 10 theo phương thức thanh toán». Ô tổng ghi **191.644.000 đ**; bảng đối chiếu của PG-012 — cùng database, cùng điều kiện «chuyến đã hoàn thành» — ghi **182.287.000 đ**. Người làm dashboard không đổi công thức doanh thu, chỉ thêm một dòng `JOIN payments` để lấy phương thức thanh toán. Không lỗi, không cảnh báo.

Vấn đề: một chuyến có thể có **nhiều** dòng thanh toán, và sau khi nối, mỗi dòng thanh toán kéo theo một bản sao của chuyến — và của cước. Bài này giải thích vì sao số dòng sau khi nối có thể lớn hơn hoặc nhỏ hơn số chuyến, và quy trình kiểm tra để tổng tiền sau JOIN không bị nhân đôi.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-013 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-bang-dong-cot-primary-foreign-key">PG-003 · Bảng, dòng, cột và khoá</a>, <a href="/articles/postgresql-aggregate-group-by-having">PG-012 · COUNT, SUM, GROUP BY và HAVING</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>13-join-cardinality.sql</code>) và thí nghiệm thêm ở <code>p02/thu-nghiem/pg013-*.sql</code>. Sau bài này, bạn biết cách kiểm tra quan hệ một–nhiều trước khi nối, để cộng tiền mà không bị nhân đôi.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2 là ảnh chụp lúc **01:00 ngày 31/10/2026 giờ Việt Nam**: 3.174 chuyến, trong đó **2.707 chuyến hoàn thành** với tổng cước **182.287.000 đ** (PG-012 đã đối chiếu).

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/13-join-cardinality.sql
```

Thí nghiệm `p02/thu-nghiem/pg013-bon-chuyen-no-dong.sql` và `p02/thu-nghiem/pg013-bai-tap.sql` chạy trên cùng nền, chỉ cần thay tệp cuối. Kết quả chép từ lần chạy thật trên PGlite 0.3.16 (PostgreSQL 17.5), bỏ dòng `-- [lab]`; chưa chạy trên server thật.

| Quan hệ | Mỗi chuyến có | Vì sao biết |
|---|---|---|
| `trips` → `payments` | 0, 1 hoặc 2 dòng (485 / 2.552 / 137 chuyến) | `payments.trip_id` **không** duy nhất |
| `trips` → `trip_events` | 1–7 dòng lịch sử (chuyến hoàn thành: 7) | `trip_events.trip_id` không duy nhất |
| `trips` → `trip_ratings` | 0 hoặc 1 dòng | `trip_id` là **khoá chính** của `trip_ratings` |
| `trips` → `cities`, `drivers`, `merchants` | 1 (hoặc 0 khi khoá ngoại NULL) | khoá ngoại trỏ tới khoá chính |

Đây là toàn bộ bí mật của bài: trước khi nối, phải biết phía bên kia có **tối đa mấy dòng** cho mỗi chuyến.

## 2. JOIN ghép dòng theo cặp

**JOIN** ghép dòng của hai bảng theo một điều kiện, thường là «khoá ngoại bằng khoá chính». Theo tài liệu PostgreSQL, với **INNER JOIN** (viết tắt `JOIN`), mỗi dòng R1 bên trái cho ra **một dòng cho mỗi dòng bên phải** thoả điều kiện với R1: không có dòng nào thì R1 biến mất, có hai dòng thì R1 xuất hiện hai lần. **LEFT JOIN** làm INNER JOIN rồi thêm một dòng có cột bên phải là NULL cho mỗi dòng bên trái không khớp — *ít nhất* một dòng cho mỗi dòng bên trái, nhưng vẫn có thể nhiều hơn.

Số dòng bên phải ứng với một dòng bên trái gọi là **cardinality** (bản số) của quan hệ: một–một, một–không-hoặc-một, một–nhiều, nhiều–một. Lấy bốn chuyến hoàn thành: 100041 trả thẻ một lần; 100042 bị thẻ từ chối rồi thu lại; 100048 trả một phần bằng ví, phần còn lại bằng thẻ; 100724 không có giao dịch nào.

```sql
SELECT t.code, t.fare, p.method, p.status, p.amount
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code, p.id;
```

```text
  code  |  fare  | method |  status   | amount
--------+--------+--------+-----------+--------
 100041 | 127000 | card   | succeeded | 127000
 100042 |  82000 | card   | failed    |  82000
 100042 |  82000 | card   | succeeded |  82000
 100048 | 119000 | wallet | succeeded | 104000
 100048 | 119000 | card   | succeeded |  15000
(5 rows)
```

Bốn chuyến nở thành năm dòng. Đổi sang `LEFT JOIN` thì 100724 quay lại với ba cột NULL — 6 dòng. Cộng `t.fare`: đúng là **404.000**, nhưng sau INNER JOIN thành **529.000** và sau LEFT JOIN thành **605.000**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Bốn chuyến nối với payments: 100041 khớp một dòng; 100042 và 100048 khớp hai dòng nên cước được cộng hai lần; 100724 không khớp dòng nào nên rơi khỏi INNER JOIN. Tổng cước đúng 404.000, sau INNER JOIN 529.000, sau LEFT JOIN 605.000.">
<text x="20" y="24" class="t-b">trips · 4 dòng</text>
<text x="300" y="24" class="t-b">payments khớp · 5 dòng = 5 dòng kết quả</text>
<rect x="20" y="40" width="220" height="28" rx="6" class="f-surface s-line"/><text x="32" y="59" class="t-mono">100041 · fare 127.000</text>
<rect x="20" y="80" width="220" height="60" rx="6" class="f-pk-soft s-pk"/><text x="32" y="114" class="t-mono t-pk">100042 · fare 82.000</text>
<rect x="20" y="150" width="220" height="60" rx="6" class="f-pk-soft s-pk"/><text x="32" y="184" class="t-mono t-pk">100048 · fare 119.000</text>
<rect x="20" y="220" width="220" height="28" rx="6" class="f-surface s-dead dash"/><text x="32" y="239" class="t-mono t-dead">100724 · fare 76.000</text>
<rect x="300" y="40" width="260" height="28" rx="6" class="f-surface s-line"/><text x="312" y="59" class="t-mono">card · succeeded · 127.000</text>
<rect x="300" y="80" width="260" height="28" rx="6" class="f-surface s-line"/><text x="312" y="99" class="t-mono">card · failed · 82.000</text>
<rect x="300" y="112" width="260" height="28" rx="6" class="f-warn-soft s-warn"/><text x="312" y="131" class="t-mono">card · succeeded · 82.000</text>
<rect x="300" y="150" width="260" height="28" rx="6" class="f-surface s-line"/><text x="312" y="169" class="t-mono">wallet · succeeded · 104.000</text>
<rect x="300" y="182" width="260" height="28" rx="6" class="f-warn-soft s-warn"/><text x="312" y="201" class="t-mono">card · succeeded · 15.000</text>
<line x1="240" y1="54" x2="300" y2="54" class="s-line" stroke-width="1.4"/>
<line x1="240" y1="110" x2="300" y2="94" class="s-pk" stroke-width="1.4"/><line x1="240" y1="110" x2="300" y2="126" class="s-pk" stroke-width="1.4"/>
<line x1="240" y1="180" x2="300" y2="164" class="s-pk" stroke-width="1.4"/><line x1="240" y1="180" x2="300" y2="196" class="s-pk" stroke-width="1.4"/>
<text x="580" y="130" class="t-warn">dòng 2 của 100042: fare 82.000 lặp lại</text>
<text x="580" y="200" class="t-warn">dòng 2 của 100048: fare 119.000 lặp lại</text>
<text x="300" y="239" class="t-small">(không có dòng) ⇒ INNER JOIN bỏ 100724; LEFT JOIN giữ với cột NULL</text>
<rect x="20" y="262" width="270" height="52" rx="8" class="f-phys-soft s-phys"/><text x="155" y="284" text-anchor="middle" class="t-b t-phys">Σ fare đúng: 404.000</text><text x="155" y="302" text-anchor="middle" class="t-small">mỗi chuyến cộng một lần</text>
<rect x="320" y="262" width="270" height="52" rx="8" class="f-warn-soft s-warn"/><text x="455" y="284" text-anchor="middle" class="t-b t-warn">sau INNER JOIN: 529.000</text><text x="455" y="302" text-anchor="middle" class="t-small">+82.000 +119.000 −76.000</text>
<rect x="620" y="262" width="280" height="52" rx="8" class="f-warn-soft s-warn"/><text x="760" y="284" text-anchor="middle" class="t-b t-warn">sau LEFT JOIN: 605.000</text><text x="760" y="302" text-anchor="middle" class="t-small">+82.000 +119.000, giữ 100724</text>
</svg>
</div>
<figcaption><b>Hình 1 · Nối một–nhiều nhân bản phía «một».</b> Mỗi dòng thanh toán khớp kéo theo một bản sao của chuyến, kể cả <code>fare</code>; INNER JOIN còn làm rơi chuyến không có dòng nào. Số liệu từ <code>p02/thu-nghiem/pg013-bon-chuyen-no-dong.sql</code>.</figcaption>
</figure>

JOIN không «sai» — nó trả đúng một dòng cho mỗi cặp khớp. Sai ở chỗ `sum(t.fare)` giả định mỗi chuyến một dòng, trong khi sau khi nối **một dòng là một cặp (chuyến, giao dịch)**.

## 3. Đếm trước, đếm sau

Thói quen đầu tiên: đếm phía bạn quan tâm, rồi đếm lại sau khi nối. Câu 1 của `13-join-cardinality.sql` đặt bốn phép đếm cạnh nhau:

```text
 chuyen_hoan_thanh | gd_cua_chung | dong_inner_join | dong_left_join
-------------------+--------------+-----------------+----------------
              2707 |         2800 |            2800 |           2832
(1 row)
```

2.707 chuyến hoàn thành có 2.800 dòng giao dịch. INNER JOIN cho **2.800** dòng — bằng số giao dịch, không phải số chuyến. LEFT JOIN cho **2.832**: thêm 32 chuyến không có giao dịch, mỗi chuyến một dòng NULL. Báo cáo «theo chuyến» mà số dòng sau nối khác số chuyến thì mọi `sum` trên cột của chuyến đều đáng ngờ.

## 4. Tổng tiền bị nhân như thế nào

Câu truy vấn của dashboard, rút gọn (câu 2 của lab):

```sql
SELECT count(*)                AS so_dong,
       count(DISTINCT t.id)    AS so_chuyen,
       sum(t.fare)             AS tong_cuoc_SAI,
       sum(p.amount)           AS tong_gd_moi_trang_thai
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed';
```

```text
 so_dong | so_chuyen | tong_cuoc_sai | tong_gd_moi_trang_thai
---------+-----------+---------------+------------------------
    2800 |      2675 |     191644000 |              189686000
(1 row)
```

Khoản chênh 9.357.000 đ gồm hai phần ngược chiều (câu 5 của tệp thí nghiệm): **+9.433.000 đ** là cước của 125 chuyến có 2 dòng giao dịch, bị cộng thêm một lần; **−76.000 đ** là 32 chuyến không có dòng nào rơi khỏi INNER JOIN (31 chuyến cước 0 đ và 100724). 182.287.000 + 9.433.000 − 76.000 = 191.644.000. Hai lỗi bù trừ một phần, nên «tổng gần đúng» không chứng minh câu lệnh đúng. Cột `tong_gd_moi_trang_thai` cũng không phải tiền đã thu: nó cộng cả `failed`, `pending` và cộng tiền hoàn như tiền thu.

Thêm `DISTINCT` không cứu được: `sum(DISTINCT t.fare)` loại trùng theo **giá trị cước**, ra **11.259.000 đ** — hai chuyến cùng 50.000 đ chỉ cộng một lần. Thứ bị trùng là *chuyến*, thứ bị loại lại là *số tiền*.

## 5. Câu hỏi phải đặt trước mỗi JOIN

> Với mỗi dòng bên trái, bên phải có **tối đa** mấy dòng?

**Đọc ràng buộc.** Cột nối bên phải là khoá chính hoặc `UNIQUE` thì câu trả lời là «tối đa 1», cho cả dữ liệu tương lai. `trip_ratings` có khoá chính `trip_id`, nên `trips LEFT JOIN trip_ratings` giữ nguyên 2.707 dòng. Chiều **nhiều–một** an toàn vì cùng lý do: câu 8 của lab nối `trips` với `cities`, `drivers`, `merchants`, trước và sau đều 2.707 dòng (chuyến hoàn thành luôn có tài xế nhờ CHECK của PG-005; chuyến chưa có tài xế sẽ rơi khỏi `JOIN drivers`).

**Đếm dữ liệu.** `payments.trip_id` không duy nhất — một chuyến *được phép* có nhiều giao dịch. Câu 3 của lab đếm phân bố trong chuyến hoàn thành: **2.550** chuyến có 1 dòng, **125** chuyến có 2 dòng. Đây chỉ là bằng chứng *tạm thời*: hôm nay tối đa 2, ngày mai một khách có thể trả bằng ba phương thức. Mẫu kiểm tra nhanh cho mọi khoá nối:

```sql
SELECT trip_id FROM payments GROUP BY trip_id HAVING count(*) > 1;
```

Trên toàn bảng, `payments.trip_id` có 137 giá trị lặp, `trip_events.trip_id` 3.173, `trip_ratings.trip_id` 0. Số 0 chỉ nói về dữ liệu hiện tại; muốn chắc mãi mãi thì cần ràng buộc.

## 6. Cách đúng: gom phía «nhiều» trước, rồi mới nối

Báo cáo ở mức **chuyến** thì đưa `payments` về **một dòng mỗi chuyến** trước khi nối (câu 4 của lab):

```sql
SELECT count(*)                       AS so_dong,
       count(DISTINCT t.id)           AS so_chuyen,
       sum(t.fare)                    AS tong_cuoc,
       sum(coalesce(g.thuc_thu, 0))   AS tong_thuc_thu
FROM trips AS t
LEFT JOIN (
  SELECT trip_id,
         sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
  FROM payments
  WHERE status = 'succeeded'
  GROUP BY trip_id
) AS g ON g.trip_id = t.id
WHERE t.status = 'completed';
```

```text
 so_dong | so_chuyen | tong_cuoc | tong_thuc_thu
---------+-----------+-----------+---------------
    2707 |      2707 | 182287000 |     182531000
(1 row)
```

Subquery trong `FROM` (bảng dẫn xuất `g`) có `GROUP BY trip_id` nên mỗi chuyến xuất hiện tối đa một lần; `LEFT JOIN` giữ 32 chuyến không có giao dịch, `coalesce(…, 0)` coi chúng là 0 đ (PG-011). Bằng chứng nằm ngay trong kết quả: `so_dong` = `so_chuyen` = 2.707 và `tong_cuoc` khớp PG-012.

Thực thu của Tới Nơi là Σ charge − Σ refund + Σ adjustment, chỉ tính `succeeded` (adjustment đã mang dấu). Con số này lớn hơn cước 244.000 đ — không còn là lỗi JOIN mà là **sai lệch thật** của vài chuyến; tìm từng chuyến là việc của PG-014 và PG-020.

## 7. Dòng bị rơi, và điều kiện ở ON hay WHERE

Lỗi đối xứng với «nở dòng» là **mất dòng**. Có khi dòng mất chính là thứ cần tìm: mẫu **anti-join** — `LEFT JOIN` rồi giữ dòng có khoá bên phải là NULL — liệt kê chuyến hoàn thành chưa có giao dịch thành công (câu 5 của lab, rút gọn):

```sql
SELECT count(*)
FROM trips AS t
LEFT JOIN payments AS p ON p.trip_id = t.id AND p.status = 'succeeded'
WHERE t.status = 'completed' AND p.id IS NULL;
```

Kết quả: **31** chuyến cước 0 đ (đúng nghiệp vụ) và **2** chuyến có cước mà chưa thu — 100724 và 101084 (chỉ có lần thu thẻ `failed`). `p.id` là khoá chính, nên NULL ở đó chỉ có thể do LEFT JOIN sinh ra.

Để ý `p.status = 'succeeded'` nằm trong **`ON`**. Theo tài liệu PostgreSQL, điều kiện ở `ON` được xử lý *trước* khi nối, ở `WHERE` *sau* khi nối — khác biệt chỉ lộ ra với outer join. Câu 6 của lab: đặt ở `WHERE` ra **2.674** chuyến, ở `ON` ra **2.707**. Ở `WHERE`, dòng NULL do LEFT JOIN sinh ra gặp `NULL = 'succeeded'` (ra NULL, không phải `true`) nên bị loại; LEFT JOIN biến thành INNER JOIN mà không báo lỗi. Quy tắc nhớ: **điều kiện chọn dòng của bảng bên phải đặt ở `ON`, của bảng bên trái đặt ở `WHERE`**.

## 8. Hai quan hệ một–nhiều trong cùng một câu

Nối `trips` với cả `payments` lẫn `trip_events` thì hai phía «nhiều» **nhân** với nhau: mỗi giao dịch × mỗi dòng lịch sử. Chuyến 100042 có 2 giao dịch và 7 sự kiện nên ra 2 × 7 = **14** dòng, và `sum(p.amount)` phình thành 82.000 × 2 × 7 = 1.148.000 đ (câu 7 của lab). Trên cả tháng, tổng thu thành công 183.640.000 đ (dòng `charge`, mọi chuyến) phình thành **1.283.272.000 đ** — gần gấp 7. Cách sửa vẫn là mục 6: gom **từng** phía nhiều trong subquery riêng, rồi mới nối với `trips`.

## 9. Câu hỏi «có hay không»: đừng JOIN rồi đếm

«Bao nhiêu chuyến hoàn thành từng dùng ví VíXanh?» là câu hỏi về **chuyến**. JOIN rồi `count(*)` ra **1.050** — số dòng giao dịch ví, gồm cả `failed` và chuyến có hai dòng ví. `count(DISTINCT t.id)` ra đúng **961** nhưng vẫn tạo 1.050 dòng rồi loại trùng. Cách nói thẳng ý định là `WHERE EXISTS (SELECT 1 FROM payments AS p WHERE p.trip_id = t.id AND p.method = 'wallet')` (câu 9 của lab, cũng ra 961): `EXISTS` chỉ hỏi «có ít nhất một dòng không», nên mỗi chuyến được tính một lần. PG-014 đi sâu vào `EXISTS`, `IN` và bẫy `NOT IN` với NULL.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn bước trước khi tin một tổng sau JOIN</span>
<p>(1) Viết ra <b>đơn vị một dòng</b> của câu trả lời: chuyến, giao dịch hay khách. (2) Với mỗi bảng được nối, trả lời «tối đa mấy dòng mỗi dòng bên trái» bằng ràng buộc (khoá chính, <code>UNIQUE</code>); không có ràng buộc thì gom về một dòng trước. (3) Đếm số dòng trước và sau khi nối; chênh lệch phải giải thích được bằng con số. (4) Đối chiếu tổng với một truy vấn độc lập không JOIN — ở đây là 182.287.000 đ cước tháng 10.</p>
</div>

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Tổng tăng sau khi «chỉ thêm một JOIN» | Cột tiền của phía «một» lặp theo phía «nhiều» | Gom phía nhiều về một dòng mỗi khoá rồi nối |
| Tổng giảm, vài chuyến biến mất | INNER JOIN bỏ chuyến không có dòng bên phải | `LEFT JOIN` và xử lý NULL có chủ đích |
| `LEFT JOIN` ra cùng số dòng với `JOIN` | Điều kiện của bảng phải nằm ở `WHERE` | Chuyển điều kiện đó vào `ON` |
| `sum(DISTINCT …)` nhỏ bất thường | Loại trùng theo giá trị, không theo dòng | Loại trùng theo khoá (gom trước) |
| Tổng phình gấp nhiều lần | Hai quan hệ một–nhiều nối cùng lúc | Gom từng phía trong subquery riêng |

## 11. Đánh đổi

- **Gom trước hay `count(DISTINCT …)` sau:** `count(DISTINCT khoá)` đủ để đếm nhưng không cứu được `sum` của phía «một»; gom trước dài hơn vài dòng và đúng cho mọi phép tổng hợp.
- **INNER hay LEFT:** INNER ngắn và đúng khi phía phải chắc chắn có dòng; LEFT an toàn hơn nhưng phải xử lý NULL ở mọi cột bên phải.
- **Một câu lớn hay nhiều bước:** một câu nhiều JOIN tiện cho dashboard nhưng khó kiểm tra; PG-014 tách nó thành các bước CTE có số dòng để đối chiếu.

## 12. Bài tập

**Bài 1.** Tính điểm sao trung bình theo thành phố của chuyến hoàn thành, kèm số chuyến và số chuyến có đánh giá. Trước khi chạy: câu lệnh có làm nở dòng không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp, count(*) AS chuyen_hoan_thanh,
       count(r.trip_id) AS co_danh_gia, round(avg(r.stars), 2) AS sao_tb
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code COLLATE "C";</code></pre>
<p>Kết quả: DN 389 / 266 / 4.30 · HN 941 / 670 / 4.17 · SG 1377 / 955 / 4.23. Không nở dòng: <code>cities</code> là nhiều–một qua khoá chính, <code>trip_ratings</code> tối đa một dòng mỗi chuyến; 389 + 941 + 1377 = 2.707.</p>
</details>

**Bài 2.** Kế toán muốn biết **tiền đã thu** qua từng phương thức (chuyến hoàn thành, giao dịch thành công). Đồng nghiệp viết `sum(t.fare) … GROUP BY p.method` sau khi nối `trips` với `payments`. Chỉ ra chỗ sai và viết lại.

<details><summary><b>Lời giải</b></summary>
<p>Câu đó gán <b>cả cước chuyến</b> cho mỗi dòng giao dịch: 100048 được tính 119.000 đ ở ví và 119.000 đ ở thẻ; tổng 184.489.000 đ, lớn hơn cả cước lẫn thực thu. Đơn vị một dòng ở đây là <b>giao dịch</b>: bỏ <code>t.fare</code>, cộng <code>sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END)</code> với cùng <code>JOIN</code>, <code>WHERE</code>, <code>GROUP BY p.method</code>.</p>
<p>Kết quả: card 686 / 46.932.000 · cash 1054 / 69.524.000 · wallet 963 / 66.075.000; tổng 182.531.000 đ, đúng thực thu ở mục 6. Chiều nối là nhiều–một nên không nở dòng.</p>
</details>

**Bài 3.** Dự đoán số dòng của `trips` (hoàn thành) `LEFT JOIN payments LEFT JOIN trip_ratings`, rồi tính `avg(r.stars)` trên kết quả đó. Vì sao lệch so với 4,22 của PG-011? Sửa để mỗi chuyến đúng một dòng.

<details><summary><b>Lời giải</b></summary>
<p>2.832 dòng (<code>payments</code> làm nở, <code>trip_ratings</code> thì không); điểm trung bình <b>4.24</b> vì chuyến có hai giao dịch được tính sao hai lần. Gom <code>payments</code> trước:</p>
<pre><code class="language-sql">SELECT count(*), round(avg(r.stars), 2), sum(g.so_gd)
FROM trips AS t
LEFT JOIN (SELECT trip_id, count(*) AS so_gd FROM payments GROUP BY trip_id) AS g
  ON g.trip_id = t.id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';</code></pre>
<p>Kết quả: 2707 | 4.22 | 2800. Lời giải đầy đủ: <code>p02/thu-nghiem/pg013-bai-tap.sql</code>.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 17 · Table Expressions — Joined Tables](https://www.postgresql.org/docs/17/queries-table-expressions.html#QUERIES-JOIN): INNER/OUTER JOIN và khác biệt `ON` với `WHERE`.
- [PostgreSQL 17 · Joins Between Tables](https://www.postgresql.org/docs/17/tutorial-join.html).
- [PostgreSQL 17 · Aggregate Functions](https://www.postgresql.org/docs/17/functions-aggregate.html).
- Lab: `examples/toi-noi-postgresql/p02/13-join-cardinality.sql`, kết quả đầy đủ ở `p02/OUTPUT.txt`; thí nghiệm ở `p02/thu-nghiem/pg013-*.sql`.

Bài tiếp theo: [PG-014 · Subquery, EXISTS và CTE](/articles/postgresql-subquery-exists-cte).

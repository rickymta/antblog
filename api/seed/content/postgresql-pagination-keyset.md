---
slug: postgresql-pagination-keyset
title: "Phân trang lịch sử chuyến bằng keyset: khoá trùng và điều kiện trang kế tiếp"
summary: "Kế toán của một khách doanh nghiệp Tới Nơi đối chiếu hoá đơn và thấy màn Lịch sử chuyến thiếu đúng hai chuyến đặt cùng giây. Dùng keyset với so sánh theo bộ giá trị (requested_at, code), rồi chứng minh ghép mọi trang lại thì không mất, không lặp dòng."
category: postgresql
level: advanced
tags: [postgresql, pg-018, pagination, keyset, row-comparison, order-by, toi-noi]
language: vi
---

Đầu tháng 11, kế toán của **Cửa hàng Phụ Kiện Lab** — khách doanh nghiệp đặt giao bưu kiện qua API của Tới Nơi — đối chiếu hoá đơn tháng 10 với màn «Lịch sử chuyến» trên web doanh nghiệp. Hoá đơn có 18 chuyến; lật hết các trang, chị chỉ đếm được 16. Hai chuyến biến mất, 101883 và 101882, cùng được đặt lúc 18/10 14:30:00 trong một lô API bốn chuyến. Đội web vừa thay phân trang `OFFSET` bằng nút «Xem thêm»: trang sau lấy các chuyến **cũ hơn giờ đặt của chuyến cuối** trang trước. Nghe hợp lý, và chạy đúng trên mọi dữ liệu thử — vì dữ liệu thử không có hai chuyến nào cùng giây.

[PG-007](/articles/postgresql-select-filter-order-limit) đã dùng `LIMIT … OFFSET` kèm khoá phụ để mỗi dòng có một vị trí duy nhất, và để dành cho bài này hai giới hạn còn lại của `OFFSET`: chi phí bỏ qua dòng và trang bị trôi khi dữ liệu thay đổi. Kỹ thuật thay thế — **keyset** — giải được cả hai, nhưng mang theo cái bẫy của câu chuyện trên. Bài này xử lý cái bẫy đó và **chứng minh** điều kiện trang kế tiếp: ghép mọi trang lại thì không mất, không lặp dòng nào.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-018 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-select-filter-order-limit">PG-007 · SELECT, lọc, sắp xếp và giới hạn</a>, <a href="/articles/postgresql-subquery-exists-cte">PG-014 · Subquery, EXISTS và CTE</a>, <a href="/articles/postgresql-window-function-frame">PG-015 · Window function</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>18-pagination-keyset.sql</code>). Sau bài này, bạn xử lý được khoá sắp xếp trùng và chứng minh được điều kiện trang kế tiếp — bằng lập luận và bằng truy vấn kiểm tra trên fixture.</p>
</div>

## 1. Dữ liệu: 18 chuyến, hai lô cùng giây

Khách doanh nghiệp `cb000000-…-000000000001` có 18 chuyến bưu kiện ở Sài Gòn. Màn lịch sử hiện **mới nhất trước**, 5 chuyến một trang. Đánh số vị trí bằng `row_number()` ([PG-015](/articles/postgresql-window-function-frame)) để có bảng đối chiếu — đây là «đáp án» mà mọi cách phân trang phải tái tạo được:

```sql
SELECT row_number() OVER w                AS vi_tri,
       (row_number() OVER w - 1) / 5 + 1  AS trang,
       code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
WINDOW w AS (ORDER BY requested_at DESC, code DESC)
ORDER BY requested_at DESC, code DESC;
```

```text
 vi_tri | trang |  code  |      requested_at
--------+-------+--------+------------------------
      1 |     1 | 102897 | 2026-10-28 10:30:00+07
      2 |     1 | 102611 | 2026-10-25 14:10:00+07
      3 |     1 | 102149 | 2026-10-21 08:50:00+07
      4 |     1 | 101885 | 2026-10-18 14:30:00+07
      5 |     1 | 101884 | 2026-10-18 14:30:00+07
      6 |     2 | 101883 | 2026-10-18 14:30:00+07
      7 |     2 | 101882 | 2026-10-18 14:30:00+07
      8 |     2 | 101555 | 2026-10-15 16:45:00+07
      9 |     2 | 101224 | 2026-10-12 11:05:00+07
     10 |     2 | 100976 | 2026-10-10 09:00:00+07
     11 |     3 | 100975 | 2026-10-10 09:00:00+07
     12 |     3 | 100974 | 2026-10-10 09:00:00+07
     13 |     3 | 100973 | 2026-10-10 09:00:00+07
     14 |     3 | 100972 | 2026-10-10 09:00:00+07
     15 |     3 | 100971 | 2026-10-10 09:00:00+07
     16 |     4 | 100669 | 2026-10-07 09:20:00+07
     17 |     4 | 100500 | 2026-10-05 15:40:00+07
     18 |     4 | 100130 | 2026-10-02 10:15:00+07
(18 rows)
```

Hai lô API đặt cùng giây: 10/10 09:00:00 có 6 chuyến (100971–100976), 18/10 14:30:00 có 4 chuyến (101882–101885). Cả hai **vắt qua ranh giới trang**. Đó không phải trường hợp hiếm: trong fixture, giờ đặt làm tròn tới phút và có 148 nhóm chuyến trùng giờ (305 chuyến). Ở hệ thống thật, `timestamptz` lưu tới micro giây nhưng lô gửi qua API, dữ liệu nhập từ file hay đồng hồ phía máy khách vẫn tạo ra giờ trùng — đừng dựa vào «hiếm khi trùng».

## 2. Keyset: «lấy tiếp sau dòng cuối tôi đã thấy»

**Phân trang keyset** (còn gọi *seek pagination*) không đếm vị trí. Trang sau bắt đầu **ngay sau dòng cuối** của trang trước, xác định bằng giá trị khoá sắp xếp của dòng đó — gọi là **con trỏ** (*cursor*). Ứng dụng giữ con trỏ (thường mã hoá thành một chuỗi trong URL hay phản hồi API) và gửi lại khi người dùng bấm «Xem thêm». Con trỏ ở đây là giá trị dữ liệu, không phải `DECLARE CURSOR` của PostgreSQL — thứ đó giữ trạng thái trong một phiên, không hợp với web nhiều phiên.

Ba điều kiện để keyset đúng:

1. **Thứ tự toàn phần:** `ORDER BY` trên một khoá **duy nhất** và **NOT NULL** — mỗi dòng có đúng một vị trí.
2. **Điều kiện «sau con trỏ»** dùng **đúng khoá đó, đúng chiều, so sánh chặt**.
3. Mọi trang dùng **cùng** `ORDER BY` và `LIMIT`.

`requested_at` một mình vi phạm điều kiện 1. Khoá đúng là cặp `(requested_at, code)`: `code` duy nhất, nên cặp cũng duy nhất. Trang 1 không có con trỏ:

```sql
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC
LIMIT 5;
```

Kết quả là 5 dòng đầu của bảng đối chiếu (102897 … 101884). Con trỏ ứng dụng giữ lại là khoá của dòng cuối: **(2026-10-18 14:30:00+07, 101884)**.

## 3. Con trỏ một cột: mất dòng hoặc lặp dòng

Đội web chỉ giữ giờ đặt và hỏi «các chuyến cũ hơn»:

```sql
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at < '2026-10-18 14:30:00+07'
ORDER BY requested_at DESC, code DESC
LIMIT 5;
```

```text
  code  |      requested_at
--------+------------------------
 101555 | 2026-10-15 16:45:00+07
 101224 | 2026-10-12 11:05:00+07
 100976 | 2026-10-10 09:00:00+07
 100975 | 2026-10-10 09:00:00+07
 100974 | 2026-10-10 09:00:00+07
(5 rows)
```

101883 và 101882 cùng giờ với con trỏ nên `<` loại chúng — và không trang nào sau đó còn cơ hội lấy lại. Đây chính là hai chuyến kế toán không tìm thấy. Không lỗi, không cảnh báo.

Sửa vội thành `<=` thì trang 2 bắt đầu bằng **101885, 101884** — lặp hai chuyến của trang 1, rồi 101883, 101882, 101555. Lặp dòng còn tệ hơn khi **một lô cùng giây dài hơn cỡ trang**. Thí nghiệm `pg018-con-tro-mot-cot-ket-trang.sql` (không có trong lab chính, nằm ở `p02/thu-nghiem/`) dùng cỡ trang 3 trên lô 10/10 sáu chuyến:

```text
-- requested_at <= '2026-10-10 09:00:00+07' LIMIT 3, lần 1 và lần 2:
 100976 | 2026-10-10 09:00:00+07
 100975 | 2026-10-10 09:00:00+07
 100974 | 2026-10-10 09:00:00+07
```

Dòng cuối vẫn mang giờ 09:00:00 nên con trỏ không nhích: «Xem thêm» trả mãi cùng ba chuyến. Đổi về `<` thì nhảy thẳng tới 100669, mất 100973, 100972, 100971. Với khoá không duy nhất, **không có** dấu so sánh nào đúng: `<` mất phần còn lại của nhóm trùng, `<=` lặp phần đã thấy.

## 4. So sánh bộ giá trị

PostgreSQL so sánh được hai **bộ giá trị** (*row constructor*, viết `(a, b)`). Với `<`, `<=`, `>`, `>=`, tài liệu PostgreSQL 17 (mục *Row Constructor Comparison*) mô tả: so từng cặp phần tử từ trái sang phải, dừng ở cặp **khác nhau** đầu tiên và lấy kết quả của cặp đó. Vậy

```text
(a, b) < (x, y)   ⇔   a < x   OR   (a = x AND b < y)
```

— đúng thứ tự từ điển, và cùng chiều với `ORDER BY a DESC, b DESC` khi ta đi về phía «nhỏ hơn». Trang 2:

```sql
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) < ('2026-10-18 14:30:00+07', 101884)
ORDER BY requested_at DESC, code DESC
LIMIT 5;
```

```text
  code  |      requested_at
--------+------------------------
 101883 | 2026-10-18 14:30:00+07
 101882 | 2026-10-18 14:30:00+07
 101555 | 2026-10-15 16:45:00+07
 101224 | 2026-10-12 11:05:00+07
 100976 | 2026-10-10 09:00:00+07
(5 rows)
```

Đúng vị trí 6–10. Trong nhóm 14:30:00, `code` quyết định: 101883 và 101882 nhỏ hơn 101884 nên được lấy; 101885 lớn hơn nên đã nằm ở trang 1. Trang 3 với con trỏ (2026-10-10 09:00:00+07, 100976) bắt đầu **giữa** lô 10/10: 100975, 100974, 100973, 100972, 100971 — vị trí 11–15.

Dạng viết tách `requested_at < $1 OR (requested_at = $1 AND code < $2)` cho cùng tập dòng; lab kiểm bằng `EXCEPT` hai chiều giữa hai dạng và đếm được `0` dòng lệch. Dạng tách dùng khi hệ quản trị cơ sở dữ liệu khác không hỗ trợ so sánh bộ, hoặc khi hai cột sắp ngược chiều (mục 6). Dạng bộ ngắn hơn, khó viết sai hơn, và chỉ mục nhiều cột phục vụ trực tiếp được dạng này (Phần 3).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Khoá sắp xếp có NULL là lỗ hổng không báo lỗi</span>
<p>Theo cùng mục tài liệu, gặp cặp có NULL trước khi gặp cặp khác nhau thì kết quả so sánh bộ là NULL — dòng bị <code>WHERE</code> loại, không báo gì. <code>requested_at</code> và <code>code</code> đều <code>NOT NULL</code> trong schema nên an toàn. Phân trang theo cột có thể NULL (ví dụ <code>completed_at</code>, NULL với chuyến huỷ) phải xử lý riêng: thay bằng biểu thức không NULL hoặc tách nhóm NULL — đừng để dòng bị mất mà không có dấu hiệu nào.</p>
</div>

## 5. Chứng minh điều kiện trang kế tiếp

Gọi `K(r) = (requested_at, code)` là khoá của dòng `r`, và sắp toàn bộ lịch sử theo `K` giảm dần thành danh sách `L` có vị trí 1…N. Vì `code` duy nhất nên `K` duy nhất: hai dòng khác nhau luôn so sánh được và không bao giờ bằng nhau — thứ tự là **toàn phần, chặt**.

**Mệnh đề.** Nếu trang 1 = n dòng đầu của `L`, và trang k+1 = n dòng đầu (theo cùng thứ tự) của tập `{ r : K(r) < c_k }`, với `c_k` là khoá dòng cuối trang k, thì trang k là đúng các vị trí `(k−1)·n + 1 … k·n` của `L`.

**Quy nạp.** Đúng với k = 1 theo định nghĩa. Giả sử trang k là vị trí `(k−1)n+1 … kn`; dòng cuối của nó ở vị trí kn, nên `c_k = K(L[kn])`. Vì thứ tự chặt và toàn phần, `K(r) < c_k` **khi và chỉ khi** `r` đứng sau vị trí kn trong `L` — không dòng nào đứng sau mà có khoá bằng `c_k`, không dòng nào đứng trước mà có khoá nhỏ hơn. Lấy n dòng đầu của tập đó thì được các vị trí `kn+1 … kn+n`. Khi tập rỗng, dừng.

Hệ quả: các trang là những đoạn **liên tiếp, rời nhau** của `L` và phủ hết `L` — không mất, không lặp. Chứng minh cũng chỉ ra chính xác chỗ nó gãy: `K` không duy nhất thì «khi và chỉ khi» sai — dòng cùng khoá với `c_k` ở sau vị trí kn bị `<` loại (mất), còn `<=` kéo lại dòng cùng khoá ở trước (lặp). Điều kiện so sánh khác chiều với `ORDER BY` cũng làm gãy cùng bước đó.

Lập luận cần được đối chiếu với dữ liệu thật. Lab đi hết mọi trang bằng truy vấn đệ quy: mỗi vòng lấy 5 dòng sau con trỏ của vòng trước, dừng khi không còn dòng (cú pháp `WITH RECURSIVE` giải thích ở [PG-019](/articles/postgresql-recursive-cte-cay-khu-vuc); `CROSS JOIN LATERAL` cho phép truy vấn con đọc con trỏ của vòng trước, xem [PG-014](/articles/postgresql-subquery-exists-cte)):

```sql
WITH RECURSIVE trang (so_trang, ma, con_tro_at, con_tro_code) AS (
  SELECT 1, p.ma, p.ats[cardinality(p.ats)], p.ma[cardinality(p.ma)]
  FROM (SELECT array_agg(code ORDER BY requested_at DESC, code DESC) AS ma,
               array_agg(requested_at ORDER BY requested_at DESC, code DESC) AS ats
        FROM (SELECT code, requested_at FROM trips
              WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
              ORDER BY requested_at DESC, code DESC LIMIT 5) AS dau) AS p
  UNION ALL
  SELECT tr.so_trang + 1, nx.ma, nx.ats[cardinality(nx.ats)], nx.ma[cardinality(nx.ma)]
  FROM trang AS tr
  CROSS JOIN LATERAL (
    SELECT array_agg(code ORDER BY requested_at DESC, code DESC) AS ma,
           array_agg(requested_at ORDER BY requested_at DESC, code DESC) AS ats
    FROM (SELECT code, requested_at FROM trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at, code) < (tr.con_tro_at, tr.con_tro_code)
          ORDER BY requested_at DESC, code DESC LIMIT 5) AS sau
  ) AS nx
  WHERE nx.ma IS NOT NULL          -- hết dòng ⇒ array_agg trả NULL ⇒ dừng
)
SELECT so_trang, ma FROM trang ORDER BY so_trang;
```

```text
 so_trang |                  ma
----------+--------------------------------------
        1 | {102897,102611,102149,101885,101884}
        2 | {101883,101882,101555,101224,100976}
        3 | {100975,100974,100973,100972,100971}
        4 | {100669,100500,100130}
(4 rows)
```

Câu kiểm tra thứ hai của lab đếm trên cùng truy vấn đệ quy:

```text
 so_trang | tong_ma_qua_cac_trang | ma_khac_nhau | so_chuyen
----------+-----------------------+--------------+-----------
        4 |                    18 |           18 |        18
(1 row)
```

18 mã qua các trang = 18 mã khác nhau (không lặp) = 18 chuyến của khách (không mất), và bốn trang khớp từng dòng với cột `trang` của bảng đối chiếu ở mục 1. Truy vấn kiểm tra này nên nằm trong bộ test của API phân trang, chạy trên fixture có khoá trùng cố ý — fixture không có khoá trùng thì cách sai ở mục 3 cũng qua test.

## 6. Hai cột sắp ngược chiều

Giả sử sản phẩm muốn: giờ đặt mới trước, nhưng trong cùng một lô thì **mã nhỏ trước** (thứ tự khách gửi lên). `ORDER BY requested_at DESC, code ASC` vẫn là thứ tự toàn phần, nhưng so sánh bộ `(a, b) < (x, y)` áp **một** chiều cho mọi cột nên không dùng được. Viết tách theo đúng định nghĩa thứ tự từ điển, mỗi cột một chiều:

```sql
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at < '2026-10-18 14:30:00+07'
       OR (requested_at = '2026-10-18 14:30:00+07' AND code > 101883))
ORDER BY requested_at DESC, code ASC
LIMIT 5;
```

```text
  code  |      requested_at
--------+------------------------
 101884 | 2026-10-18 14:30:00+07
 101885 | 2026-10-18 14:30:00+07
 101555 | 2026-10-15 16:45:00+07
 101224 | 2026-10-12 11:05:00+07
 100971 | 2026-10-10 09:00:00+07
(5 rows)
```

Con trỏ (18/10 14:30, 101883) là dòng cuối trang 1 theo thứ tự này (trang 1 kết thúc bằng 101882, 101883). Trong nhóm 14:30 điều kiện là `code > 101883`, còn nhóm 10/10 bắt đầu từ 100971 — mã nhỏ nhất. Với cột số còn một cách: đổi dấu để hai cột cùng chiều, `(requested_at, -code) < ('2026-10-18 14:30:00+07', -101883)` kèm `ORDER BY requested_at DESC, -code DESC` — thí nghiệm `pg018-nguoc-chieu-doi-dau.sql` ra đúng 5 dòng trên. Cách này không dùng được cho cột chữ, và chỉ mục phục vụ cách này phải được tạo trên biểu thức `-code` (Phần 3).

## 7. Trang trước và «còn trang sau?»

Keyset đi lùi bằng cách **đảo cả dấu so sánh lẫn chiều sắp**, rồi sắp lại cho đúng thứ tự hiển thị. Đang ở trang 3, dòng đầu trang là (2026-10-10 09:00:00+07, 100975):

```sql
SELECT code, requested_at
FROM (SELECT code, requested_at
      FROM trips
      WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
        AND (requested_at, code) > ('2026-10-10 09:00:00+07', 100975)
      ORDER BY requested_at ASC, code ASC
      LIMIT 5) AS truoc
ORDER BY requested_at DESC, code DESC;
```

Kết quả: 101883, 101882, 101555, 101224, 100976 — đúng trang 2. Quên đảo chiều sắp (giữ `DESC` bên trong) thì `LIMIT 5` lấy 5 dòng **mới nhất** trong mọi dòng phía trước — tức trang 1, không phải trang 2. Lỗi này cũng không báo gì; thí nghiệm `pg018-trang-truoc-con-trang-sau.sql` chạy cả hai.

Để biết có nên hiện nút «Xem thêm», lấy **n + 1** dòng, hiển thị n dòng: sau con trỏ của trang 2 (100976) truy vấn `LIMIT 6` trả 6 dòng ⇒ còn trang; sau con trỏ của trang 3 (100971) chỉ trả 3 ⇒ trang 4 là trang cuối, ẩn nút. Không cần `count(*)` cho việc đó.

## 8. Dữ liệu đổi giữa hai lần tải trang

Đây là chỗ `OFFSET` thua rõ nhất. Lab mô phỏng: khách đã xem trang 1, rồi đặt thêm một chuyến (trong `BEGIN … ROLLBACK` để không để lại dấu vết):

```text
INSERT … RETURNING code   →  103174
```

Trang 2 tải bằng `LIMIT 5 OFFSET 5`:

```text
  code  |      requested_at
--------+------------------------
 101884 | 2026-10-18 14:30:00+07
 101883 | 2026-10-18 14:30:00+07
 101882 | 2026-10-18 14:30:00+07
 101555 | 2026-10-15 16:45:00+07
 101224 | 2026-10-12 11:05:00+07
(5 rows)
```

Chuyến mới chen vào đầu danh sách, mọi dòng dịch xuống một bậc, nên 101884 — vừa thấy ở trang 1 — hiện lại, còn 100976 bị đẩy sang trang sau. Trang 2 bằng keyset với con trỏ (18/10 14:30, 101884) vẫn ra 101883, 101882, 101555, 101224, 100976 như trước: điều kiện gắn vào **giá trị** của dòng đã thấy, không vào số thứ tự. Xoá dòng cũng vậy: `OFFSET` làm mất một dòng ở ranh giới, keyset thì không. Sau `ROLLBACK`, khách vẫn 18 chuyến; sequence của `code` đã tiêu mất số 103174 ([PG-006](/articles/postgresql-insert-update-delete-returning)).

Keyset không biến danh sách thành ảnh chụp cố định. Chuyến mới hơn trang 1 không hiện cho tới khi người dùng tải lại từ đầu — với lịch sử «mới nhất trước», đó là hành vi mong muốn. Nhưng một dòng được chèn **với giờ cũ** (nhập bù, đồng bộ trễ) nằm sau con trỏ sẽ xuất hiện ở trang sau, còn nằm trước con trỏ thì người đang lật trang không thấy. Mỗi trang là một câu lệnh riêng nhìn dữ liệu ở thời điểm của nó; muốn mọi trang cùng nhìn một thời điểm là chuyện giao dịch và snapshot của Phần 4.

## 9. Vì sao keyset rẻ — chỉ nêu ý chính

Với `OFFSET k`, server vẫn phải tạo ra k dòng đầu theo thứ tự rồi bỏ đi; trang càng sâu càng đắt. Với keyset, nếu có chỉ mục theo đúng thứ tự `(customer_id, requested_at, code)`, PostgreSQL có thể bắt đầu đọc **ngay tại con trỏ** và dừng sau n dòng, nên trang thứ 1.000 tốn gần bằng trang 1. Schema của lab có `ix_trips_customer_requested ON trips (customer_id, requested_at DESC)` — chưa gồm `code`. Chỉ mục ấy đã đủ chưa, query plan trông ra sao và đo thế nào cho đúng là nội dung Phần 3; bài này không đo hiệu năng, và fixture 18 dòng cũng không đủ để đo.

## 10. Lỗi thường gặp và đánh đổi

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Thiếu vài dòng ở ranh giới trang, không lỗi | Con trỏ chỉ có cột không duy nhất, so `<` | Thêm khoá phụ duy nhất vào con trỏ và `ORDER BY` |
| Dòng của trang trước hiện lại; «Xem thêm» kẹt | So `<=` với khoá không duy nhất | Khoá duy nhất, so sánh chặt |
| Trang trước trả về trang đầu | Quên đảo chiều sắp khi đi lùi | `>` + `ASC` bên trong, sắp lại `DESC` bên ngoài |
| Mất dòng có cột sắp xếp NULL | So sánh bộ với NULL ra NULL | Khoá NOT NULL hoặc xử lý nhóm NULL riêng |
| Trộn chiều sắp mà vẫn so sánh bộ | `(a, b) <` áp một chiều cho mọi cột | Viết tách điều kiện, hoặc đổi dấu cột số |
| Lặp/mất dòng sau khi app «tối ưu» con trỏ | Con trỏ cắt giờ xuống mili giây/giây, hoặc mất múi giờ | Giữ nguyên độ chính xác của `timestamptz` và offset trong con trỏ |

Đánh đổi:

- **Keyset hay `OFFSET`:** keyset ổn định khi dữ liệu đổi và không đắt dần theo độ sâu, nhưng chỉ đi «trang sau/trang trước», không nhảy tới trang 7 và không tự cho tổng số trang. `OFFSET` có khoá phụ vẫn hợp với danh sách ngắn, ít đổi, cần nhảy trang (màn quản trị).
- **Con trỏ lộ giá trị hay mã hoá:** con trỏ dạng `2026-10-18T14:30:00+07:00|101884` dễ gỡ lỗi; mã hoá thành chuỗi mờ (*opaque token*) giữ được quyền đổi khoá sắp xếp sau này. Dù dạng nào, server phải kiểm tra rằng con trỏ thuộc đúng `ORDER BY` đang dùng.
- **Khoá phụ là `code` hay `id`:** cả hai duy nhất; `code` tăng theo thời điểm tạo nên con trỏ dễ đọc, `id` là uuid ngẫu nhiên — đúng về thứ tự nhưng vô nghĩa với người vận hành.

## 11. Bài tập

**Bài 1.** Kế toán muốn xem 18 chuyến theo **cước giảm dần**, 5 chuyến một trang. Nhiều chuyến trùng cước (hai chuyến 61.000 đ, hai chuyến 57.000 đ…). Viết trang 1, trang 2, và chỉ ra chuyến nào bị mất nếu con trỏ chỉ có cước.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">-- trang 1: ORDER BY fare DESC, code DESC LIMIT 5
-- trang 2:
SELECT code, fare FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (fare, code) &lt; (57000, 101555)
ORDER BY fare DESC, code DESC
LIMIT 5;</code></pre>
<p>Chạy thật: trang 1 là 100975 (78.000), 100500 (75.000), 102611 (61.000), 100976 (61.000), 101555 (57.000) ⇒ con trỏ (57000, 101555). Trang 2: 100669 (57.000), 101883, 100972 (53.000), 101885, 100974 (49.000). Nếu con trỏ chỉ có cước thì điều kiện <code>fare &lt; 57000</code> bắt đầu từ 101883 — <b>mất 100669</b>, chuyến cùng 57.000 đ với dòng cuối trang 1. Truy vấn đệ quy như mục 5 đổi khoá thành <code>(fare, code)</code> cho kết quả 4 trang, 18 mã, 18 mã khác nhau. Không có trong lab chính, nằm ở <code>p02/thu-nghiem/pg018-bai-tap-1-theo-cuoc.sql</code>.</p>
</details>

**Bài 2.** Người dùng đang ở trang 2 (dòng đầu là 101883) và bấm «Trang trước». Viết câu lệnh. Khi đang ở trang 1, câu «trang trước» trả gì, và ứng dụng dùng kết quả đó thế nào?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, requested_at
FROM (SELECT code, requested_at FROM trips
      WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
        AND (requested_at, code) &gt; ('2026-10-18 14:30:00+07', 101883)
      ORDER BY requested_at ASC, code ASC
      LIMIT 5) AS truoc
ORDER BY requested_at DESC, code DESC;</code></pre>
<p>Chạy thật: 102897, 102611, 102149, 101885, 101884 — đúng trang 1, gồm cả hai chuyến cùng giây 14:30 có mã lớn hơn 101883. Ở trang 1, con trỏ là (2026-10-28 10:30, 102897) và câu trả 0 dòng ⇒ ẩn nút «Trang trước» (cũng có thể xin 6 dòng như mục 7). Thí nghiệm ở <code>p02/thu-nghiem/pg018-trang-truoc-con-trang-sau.sql</code>.</p>
</details>

**Bài 3.** Một đồng nghiệp đề xuất bỏ `requested_at` khỏi con trỏ: «`code` tăng theo giờ đặt, vậy `WHERE code < $1 ORDER BY code DESC` là đủ.» Trên fixture, cách đó cho kết quả đúng. Có nên nhận đề xuất này không?

<details><summary><b>Lời giải</b></summary>
<p>Không, trừ khi sản phẩm đổi định nghĩa màn hình thành «sắp theo mã chuyến». Keyset đúng khi điều kiện và <code>ORDER BY</code> dùng <b>cùng</b> khoá với thứ tự hiển thị. Trên fixture, mã được sinh theo thứ tự giờ đặt nên hai thứ tự trùng nhau — đó là tính chất của dữ liệu thử, không phải cam kết của lược đồ. <code>code</code> do IDENTITY cấp lúc chèn: chuyến nhập bù, đồng bộ trễ hay giờ đặt lấy từ máy khách đều có thể có mã lớn mà giờ đặt cũ. Khi đó «sắp theo mã» và «sắp theo giờ đặt» là hai danh sách khác nhau, và người dùng thấy chuyến nằm sai chỗ. <code>code</code> có vai trò đúng của nó: khoá phụ để phá thế hoà trong <code>(requested_at, code)</code>.</p>
</details>

## 12. Đọc tiếp

- Trước đó: [PG-017 · UPSERT và xung đột khoá duy nhất](/articles/postgresql-upsert-on-conflict). `LIMIT`/`OFFSET` và khoá phụ: [PG-007](/articles/postgresql-select-filter-order-limit).
- Truy vấn đệ quy dùng ở mục 5: [PG-019 · Recursive CTE với cây khu vực](/articles/postgresql-recursive-cte-cay-khu-vuc).
- Tài liệu PostgreSQL 17: [Row Constructor Comparison](https://www.postgresql.org/docs/17/functions-comparisons.html#ROW-WISE-COMPARISON), [LIMIT and OFFSET](https://www.postgresql.org/docs/17/queries-limit.html), [Sorting Rows](https://www.postgresql.org/docs/17/queries-order.html).
- Lab: `examples/toi-noi-postgresql/p02/18-pagination-keyset.sql`, kết quả đầy đủ ở `p02/OUTPUT.txt`.

Bài tiếp theo: [PG-019 · Recursive CTE với cây khu vực](/articles/postgresql-recursive-cte-cay-khu-vuc).

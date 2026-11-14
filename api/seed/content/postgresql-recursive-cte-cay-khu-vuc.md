---
slug: postgresql-recursive-cte-cay-khu-vuc
title: "Recursive CTE với cây khu vực"
summary: "Báo cáo của Tới Nơi cộng dồn chuyến theo cây khu vực điều phối, và một bản cập nhật cây làm Hà Nội tụt từ 1.094 xuống 340 chuyến mà không có lỗi nào. Học WITH RECURSIVE: phần neo, vòng lặp, điều kiện dừng, SEARCH, CYCLE và truy vấn phát hiện chu trình."
category: postgresql
level: advanced
tags: [postgresql, pg-019, recursive-cte, with-recursive, cay, chu-trinh, toi-noi]
language: vi
---

Tới Nơi không điều phối theo quận huyện hành chính mà theo một **cây khu vực** riêng: thành phố chia thành vùng, vùng chia thành các khu đón khách. Bảng thưởng tài xế và báo cáo doanh thu vùng đều cần trả lời cùng một kiểu câu hỏi: "tổng của mọi thứ nằm **dưới** nút này". Cây hôm nay sâu ba tầng, nhưng vận hành đã báo trước sẽ tách khu đông thành "tiểu khu" — câu SQL không được giả định số tầng.

Một buổi sáng, báo cáo vùng Hà Nội chỉ còn 340 chuyến thay vì 1.094. Không truy vấn nào lỗi, không ràng buộc nào báo vi phạm: một bản cập nhật cây đã tạo ra **chu trình**. Bài này dùng `WITH RECURSIVE` để duyệt cây có điều kiện dừng, cộng dồn số liệu theo cây có đối chiếu, và phát hiện dữ liệu cây bị vòng trước khi tin vào con số.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-019 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-subquery-exists-cte">Subquery, EXISTS và CTE</a>, <a href="/articles/postgresql-join-quan-he-cardinality">JOIN và số dòng sau khi nối bảng</a>, <a href="/articles/postgresql-aggregate-group-by-having">COUNT, SUM, GROUP BY và HAVING</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>19-cay-khu-vuc.sql</code>, <code>19-chu-trinh.sql</code>). Sau bài này, bạn duyệt được cây xuống và lên với điều kiện dừng, cộng dồn số liệu theo cây có đối chiếu, và phát hiện chu trình bằng <code>CYCLE</code> hoặc mảng đường đi.</p>
</div>

## 1. Cây khu vực nằm trong một bảng

Bảng `areas` (thêm ở `p02/00-mo-rong-schema.sql`) lưu cây kiểu **danh sách kề** (adjacency list): mỗi nút một dòng, `parent_id` trỏ tới nút cha trong chính bảng đó qua FK tự tham chiếu `fk_areas_parent`. Gốc là thành phố, `parent_id` NULL. Cột `level` nhận `city` / `district` (vùng) / `zone` (khu); `id` là số nhỏ dễ đọc (2 = HN, 21 = HN-TT, 211 = HN-TT-01). Hai CHECK đáng chú ý: `ck_areas_root_no_parent` — nút là thành phố **khi và chỉ khi** không có cha; `ck_areas_not_own_parent` — `parent_id <> id`. Mục 5 cho thấy CHECK thứ hai chưa đủ.

Fixture có **34 nút**: 3 thành phố, 10 vùng, 21 khu; vùng `HN-NA` mới mở, chưa có khu. Chuyến gắn với khu đón qua `trips.pickup_area_id` (36 chuyến NULL vì mất GPS); K8 của `02-kiem-tra-fixture.sql` bảo đảm khu đón là nút `zone` đúng thành phố.

Với cây cố định ba tầng, hai lần JOIN lên cha là đủ (bài tập 2). Nhưng số JOIN bằng số tầng: thêm "tiểu khu" là phải sửa mọi báo cáo. **Truy vấn đệ quy** đi theo `parent_id` tới khi hết đường, không cần biết trước độ sâu.

## 2. WITH RECURSIVE chạy như thế nào?

**CTE đệ quy** (recursive common table expression) là CTE được tham chiếu chính nó, gồm hai phần nối bằng `UNION ALL` (hoặc `UNION`): **phần neo** chạy một lần cho các dòng xuất phát, **phần đệ quy** tham chiếu tên CTE và chạy lặp.

```sql
WITH RECURSIVE dem (n) AS (
  SELECT 1                       -- neo
  UNION ALL
  SELECT n + 1 FROM dem          -- đệ quy: chỉ thấy các dòng của vòng TRƯỚC
  WHERE n < 5                    -- điều kiện dừng
)
SELECT n FROM dem;               -- 5 dòng: 1 … 5
```

Chữ "đệ quy" dễ gây hiểu lầm. Theo [tài liệu PostgreSQL 17](https://www.postgresql.org/docs/17/queries-with.html), đây là một **vòng lặp** trên hai bảng tạm:

1. Chạy phần neo; các dòng vừa vào kết quả, vừa vào **bảng làm việc** (working table).
2. Chạy phần đệ quy, trong đó tên CTE **chỉ chứa bảng làm việc** — các dòng mới của vòng trước, không phải toàn bộ kết quả. Các dòng sinh ra được đưa vào **bảng trung gian** rồi nối vào kết quả.
3. Bảng trung gian thay cho bảng làm việc; quay lại bước 2.
4. **Dừng khi bảng làm việc rỗng** — một vòng không sinh dòng nào.

Với `UNION` (không `ALL`), PostgreSQL còn bỏ dòng trùng với dòng đã có (mục 5). Đánh số vòng cho truy vấn đi xuống từ `SG` rồi gom lại (thí nghiệm ngoài lab, nền `00 → 02` của P02):

| Vòng | Bảng làm việc | Kết quả cộng dồn |
|---|---|---|
| 0 · neo | `SG` | 1 dòng |
| 1 | 4 vùng `SG-DO`, `SG-NA`, `SG-TA`, `SG-TT` | 5 dòng |
| 2 | 10 khu `SG-DO-01` … `SG-TT-03` | 15 dòng |
| 3 | con của 10 khu: không có ⇒ **dừng** | 15 dòng |

- **Phải bảo đảm truy vấn tới được vòng rỗng.** Bỏ `WHERE n < 5` là chạy mãi. Trên cây, vòng rỗng đến khi gặp lá — chỉ đúng khi dữ liệu thật sự là cây (mục 5).
- **Thứ tự dòng không được hứa.** Tài liệu nói thuật toán ra kết quả theo chiều rộng nhưng đó là chi tiết cài đặt, không nên dựa vào. Muốn thứ tự nào, viết `ORDER BY`.
- **Phần đệ quy có giới hạn cú pháp**: không hàm gộp, không tham chiếu CTE hai lần, không đặt tham chiếu ở phía "có thể NULL" của outer join (mục 6). Hãy để truy vấn **ngoài** làm việc gộp và LEFT JOIN.

## 3. Đi xuống, đi lên

Truy vấn đầu tiên của `19-cay-khu-vuc.sql` lấy cây TP. Hồ Chí Minh kèm độ sâu, cha trước, con ngay dưới:

```sql
WITH RECURSIVE cay (id, code, name, do_sau) AS (
  SELECT id, code, name, 0 FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code, a.name, c.do_sau + 1
  FROM areas AS a
  JOIN cay AS c ON a.parent_id = c.id
) SEARCH DEPTH FIRST BY code SET thu_tu_duyet
SELECT do_sau, code, name
FROM cay
ORDER BY thu_tu_duyet;
```

```text
 do_sau |   code   |      name
--------+----------+-----------------
      0 | SG       | TP. Hồ Chí Minh
      1 | SG-DO    | Phía Đông
      2 | SG-DO-01 | Khu 01
      2 | SG-DO-02 | Khu 02
…
      2 | SG-TT-03 | Khu 03
(15 rows)
```

`a.parent_id = c.id` nghĩa là "tìm dòng có cha nằm trong bảng làm việc" — đi **xuống**. Mệnh đề `SEARCH` (có từ PostgreSQL 14) không đổi cách đánh giá; nó chỉ thêm một cột để `ORDER BY`. Trên 17.5, `DEPTH FIRST` sinh mảng đường đi như `{(SG),(SG-DO),(SG-DO-01)}` (hết nhánh này mới sang nhánh kia), còn `BREADTH FIRST` sinh bản ghi (độ sâu, khoá) như `(2,SG-DO-01)` (hết tầng này mới sang tầng sau). Cột trong `BY` quyết định thứ tự giữa anh em. Trước bản 14, phải tự mang theo mảng `ARRAY[code]`, nối `c.duong_di || a.code` mỗi vòng rồi sắp theo nó (lab có cách viết này). Lọc như `WHERE code LIKE 'SG-TT-%'` phải đặt ở truy vấn **ngoài**: lọc trong phần đệ quy là cắt luôn nhánh con.

Lật JOIN thành `a.id = t.parent_id` là đi **lên** — màn trực ca cần biết khu `HN-TA-02` thuộc vùng nào, thành phố nào:

```sql
WITH RECURSIVE to_tien (id, parent_id, code, level, bac) AS (
  SELECT id, parent_id, code, level, 0 FROM areas WHERE code = 'HN-TA-02'
  UNION ALL
  SELECT a.id, a.parent_id, a.code, a.level, t.bac + 1
  FROM areas AS a
  JOIN to_tien AS t ON a.id = t.parent_id
)
SELECT bac, code, level FROM to_tien ORDER BY bac;
-- 0 | HN-TA-02 | zone · 1 | HN-TA | district · 2 | HN | city
```

Đi lên dừng vì gốc có `parent_id` NULL: JOIN không khớp gì, vòng sau rỗng. Mỗi vòng sinh tối đa một dòng.

## 4. Cộng dồn theo cây: bảng closure

"Bao nhiêu chuyến và doanh thu nằm dưới mỗi vùng, mỗi thành phố" được lab giải hai bước: dựng **bảng closure** (bao đóng) — mọi cặp (tổ tiên, hậu duệ), kể cả cặp một nút với chính nó — rồi nối với chuyến theo hậu duệ, gom theo tổ tiên.

```sql
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas                     -- mỗi nút là hậu duệ của chính nó
  UNION ALL
  SELECT c.to_tien_id, a.id
  FROM closure AS c
  JOIN areas AS a ON a.parent_id = c.hau_due_id
)
SELECT n.code, n.level,
       count(t.id)                                              AS so_chuyen,
       coalesce(sum(t.fare) FILTER (WHERE t.status = 'completed'), 0) AS doanh_thu
FROM areas AS n
JOIN closure AS c ON c.to_tien_id = n.id
LEFT JOIN trips AS t ON t.pickup_area_id = c.hau_due_id
WHERE n.level <> 'zone'
GROUP BY n.id, n.code, n.level
ORDER BY n.code COLLATE "C";
```

```text
 code  |  level   | so_chuyen | doanh_thu
-------+----------+-----------+-----------
 DN    | city     |       456 |  26124000
…
 HN    | city     |      1094 |  62246000
 HN-BA | district |       340 |  19456000
 HN-NA | district |         0 |         0
 HN-TA | district |       295 |  16496000
 HN-TT | district |       459 |  26294000
 SG    | city     |      1588 |  92143000
…
(13 rows)
```

Ba chi tiết giữ số đúng, đều là bài học cũ của Phần 2: cặp (nút, chính nó) để chuyến ở chính một nút được tính cho nút đó; `LEFT JOIN` + `count(t.id)` để vùng `HN-NA` hiện 0 chứ không bị đếm thành 1 ([PG-011](/articles/postgresql-null-case-coalesce), [PG-013](/articles/postgresql-join-quan-he-cardinality)); doanh thu chỉ tính `completed` bằng `FILTER`, bọc `coalesce` vì `sum` trên tập rỗng là NULL ([PG-012](/articles/postgresql-aggregate-group-by-having)).

Không xảy ra hiện tượng nhân dòng như ở PG-013 vì mỗi khu chỉ có **một** đường lên gốc — một **giả định** về dữ liệu, nên lab đối chiếu ngay: cộng ba thành phố qua cây ra **3.138** chuyến, bằng đếm thẳng các chuyến có khu đón (3.138); cộng 36 chuyến mất GPS là đủ 3.174 chuyến của bảng. Đây là kiểu "tổng qua đường vòng = tổng đếm thẳng" của [PG-014](/articles/postgresql-subquery-exists-cte), và mục 5 cho thấy nó bắt được lỗi thật.

Cuối tệp là câu kiểm tra sức khoẻ của cây: đi xuống từ mọi gốc với **chốt an toàn** `WHERE c.do_sau < 10`, rồi so số nút đi tới được với tổng số nút. Kết quả `34 | 34 | 2 | 22`: 34 nút, cả 34 tới được từ gốc, sâu nhất 2, 22 lá (21 khu + `HN-NA`). Ngưỡng chốt phải rộng hơn độ sâu nghiệp vụ để không cắt nhầm cây hợp lệ, đủ nhỏ để dừng khi dữ liệu hỏng.

## 5. Khi dữ liệu không còn là cây

### 5.1 Một bản cập nhật qua mọi ràng buộc

File cập nhật cây Hà Nội bị **đảo cha** của hai vùng: `HN-TT` (21) nhận cha là khu `HN-TA-01` (221), `HN-TA` (22) nhận cha là khu `HN-TT-01` (211). `19-chu-trinh.sql` tái hiện trong một giao dịch, `ROLLBACK` ở cuối:

```sql
BEGIN;
UPDATE areas
SET parent_id = CASE id WHEN 21 THEN 221 WHEN 22 THEN 211 END
WHERE id IN (21, 22);      -- lab bọc trong WITH … RETURNING: (21, HN-TT, 221), (22, HN-TA, 211)
```

Câu lệnh thành công: cha tồn tại (FK), vùng có cha, không nút nào tự làm cha mình. Nhưng giờ có vòng **21 → 221 → 22 → 211 → 21** (đọc "con → cha"). CHECK chỉ nhìn **một dòng**: chặn được vòng một nút, không bao giờ thấy vòng bốn nút — đúng giới hạn [PG-010](/articles/postgresql-thuc-hanh-schema-chuyen-xe) đã ghi cho quy tắc nhiều dòng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 900 200" role="img" aria-label="Sau khi đảo cha, từ gốc HN chỉ còn HN-BA với hai khu và HN-NA. Bảy nút còn lại tách thành một đảo, trong đó bốn nút 21, 221, 22, 211 tạo vòng.">
<rect x="40" y="20" width="70" height="30" rx="6" class="f-pk-soft s-pk"/><text x="75" y="40" text-anchor="middle" class="t-mono">HN</text>
<path d="M75 50 L40 90 M75 50 L150 90" class="f-none s-line" stroke-width="1.4"/>
<rect x="10" y="90" width="70" height="26" rx="6" class="f-surface s-line"/><text x="45" y="108" text-anchor="middle" class="t-mono">HN-BA</text>
<rect x="115" y="90" width="70" height="26" rx="6" class="f-surface s-line"/><text x="150" y="108" text-anchor="middle" class="t-mono">HN-NA</text>
<text x="45" y="138" text-anchor="middle" class="t-small">231 · 232</text>
<text x="20" y="176" class="t-b">Dưới HN còn 5 nút, 340 chuyến</text><text x="20" y="192" class="t-small">(cây lành: 12 nút, 1.094 chuyến)</text>
<rect x="300" y="14" width="585" height="176" rx="10" class="f-warn-soft s-warn dash"/>
<text x="316" y="36" class="t-b t-warn">Đảo tách khỏi gốc: 7 nút, trong đó vòng 4 nút</text>
<rect x="340" y="60" width="110" height="30" rx="6" class="f-surface s-warn"/><text x="395" y="80" text-anchor="middle" class="t-mono">21 HN-TT</text>
<rect x="560" y="60" width="120" height="30" rx="6" class="f-surface s-warn"/><text x="620" y="80" text-anchor="middle" class="t-mono">221 HN-TA-01</text>
<rect x="560" y="130" width="120" height="30" rx="6" class="f-surface s-warn"/><text x="620" y="150" text-anchor="middle" class="t-mono">22 HN-TA</text>
<rect x="330" y="130" width="130" height="30" rx="6" class="f-surface s-warn"/><text x="395" y="150" text-anchor="middle" class="t-mono">211 HN-TT-01</text>
<path d="M450 75 H560 M620 90 V130 M560 145 H460 M395 130 V90" class="f-none s-warn" stroke-width="2"/>
<text x="505" y="68" text-anchor="middle" class="t-small t-warn">cha →</text><text x="630" y="114" class="t-small t-warn">cha ↓</text><text x="510" y="138" text-anchor="middle" class="t-small t-warn">← cha</text><text x="385" y="114" text-anchor="end" class="t-small t-warn">cha ↑</text>
<text x="710" y="80" class="t-small">+ 212, 213 (con của 21)</text><text x="710" y="150" class="t-small">+ 222 (con của 22)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Cây Hà Nội sau bản cập nhật đảo cha.</b> Vòng đi từ con tới cha: 21 → 221 → 22 → 211 → 21. Ba khu 212, 213, 222 không nằm trên vòng nhưng treo vào nó nên cũng mất đường về gốc. Số liệu: <code>19-chu-trinh.sql</code>.</figcaption>
</figure>

### 5.2 Hệ quả không báo lỗi, và vì sao đi xuống vẫn dừng

Chạy closure chỉ cho gốc Hà Nội (neo `WHERE code = 'HN'`): **5 nút, 340 chuyến** thay vì 12 nút, 1.094 chuyến. Không lỗi, không cảnh báo. Câu đối chiếu của mục 4 thì lệch ngay: qua cây 340, đếm thẳng các chuyến Hà Nội có khu đón vẫn 1.094 (thí nghiệm ngoài lab trong cùng giao dịch).

Vì sao đi xuống từ gốc **vẫn dừng** trên dữ liệu có vòng? Mỗi nút chỉ có một cha. Nếu đi xuống từ gốc chạm được một nút trên vòng, thì cha của nút đó — cũng trên vòng — phải được chạm trước, rồi cha của cha… lần ngược mãi trên vòng, không bao giờ về tới gốc. Vô lý. Vậy với cây lưu bằng một cột `parent_id`, **vòng không bao giờ tới được từ gốc**: đi xuống từ gốc luôn dừng, cái giá là nút trên vòng và hậu duệ của chúng biến mất khỏi báo cáo mà không có cảnh báo nào.

Ngược lại, hai kiểu truy vấn sau **chạy mãi**: đi **lên** từ một nút trên vòng hoặc treo vào vòng (như `HN-TT-01`), và bảng closure của mục 4 — phần neo là **mọi nút**, kể cả nút 21. Lab không chạy hai truy vấn "trần" đó. Mẹo `LIMIT` ở truy vấn ngoài mà tài liệu nhắc khi thử truy vấn có thể lặp thường vô tác dụng nếu truy vấn ngoài sắp xếp hay nối kết quả, và tài liệu khuyên không dùng ở production. Dùng một trong ba cách dưới.

### 5.3 Chốt độ sâu, CYCLE, hay mảng đường đi

**Chốt độ sâu** (`WHERE t.bac < 5`) làm truy vấn đi lên từ `HN-TT-01` dừng sau 6 dòng: `HN-TT-01, HN-TT, HN-TA-01, HN-TA, HN-TT-01, HN-TT`. Đó là dây an toàn, không phải chẩn đoán: kết quả chỉ "dài bất thường", và ngưỡng thấp quá thì cắt nhầm cây hợp lệ.

**Mệnh đề `CYCLE`** (có từ PostgreSQL 14): nêu cột nhận diện nút, PostgreSQL tự mang đường đi và đánh dấu dòng quay lại một nút đã có trên đường; dòng đó **vẫn được trả ra** nhưng không được đệ quy tiếp.

```sql
WITH RECURSIVE to_tien (id, parent_id, code, bac) AS (
  SELECT id, parent_id, code, 0 FROM areas WHERE code = 'HN-TT-01'
  UNION ALL
  SELECT a.id, a.parent_id, a.code, t.bac + 1
  FROM areas AS a JOIN to_tien AS t ON a.id = t.parent_id
) CYCLE id SET la_chu_trinh USING duong_di
SELECT bac, code, la_chu_trinh, duong_di FROM to_tien ORDER BY bac;
```

```text
 bac |   code   | la_chu_trinh |           duong_di
-----+----------+--------------+-------------------------------
   0 | HN-TT-01 | f            | {(211)}
   1 | HN-TT    | f            | {(211),(21)}
   2 | HN-TA-01 | f            | {(211),(21),(221)}
   3 | HN-TA    | f            | {(211),(21),(221),(22)}
   4 | HN-TT-01 | t            | {(211),(21),(221),(22),(211)}
(5 rows)
```

Theo [tài liệu `SELECT`](https://www.postgresql.org/docs/17/sql-select.html), dạng đầy đủ là `CYCLE cột SET cột_đánh_dấu TO giá_trị DEFAULT giá_trị_mặc_định USING cột_đường_đi`; bỏ `TO … DEFAULT …` thì cột đánh dấu là `boolean` `true`/`false`. Lọc `WHERE NOT la_chu_trinh` ở truy vấn ngoài nếu cần kết quả sạch. `CYCLE` cũng làm bảng closure dừng được (thí nghiệm ngoài lab), nhưng ra **98** cặp tổ tiên–hậu duệ thay vì 86 của cây lành (3×1 + 10×2 + 21×3) — vì 21 giờ vừa là tổ tiên vừa là hậu duệ của 22. Dừng được không có nghĩa là đúng: dữ liệu có vòng thì phép cộng dồn vô nghĩa; việc đúng là phát hiện rồi từ chối bản cập nhật.

**Tự viết bằng mảng** (trước bản 14): mang `ARRAY[id]`, tính `a.id = ANY (t.da_qua)` thành cột `lap`, chỉ đi tiếp `WHERE NOT t.lap`. Lab cho cùng kết quả: dòng bậc 4 có `{211,21,221,22,211}`, `lap = t`.

Còn đổi `UNION ALL` thành `UNION`? `UNION` chỉ bỏ dòng **trùng hoàn toàn**. Hai thí nghiệm ngoài lab, đi lên từ `HN-TT-01`:

| Cột mang theo | Kết quả | Vì sao dừng |
|---|---|---|
| Chỉ `id`, `UNION` | 4 dòng | Dòng `211` lần hai trùng hoàn toàn ⇒ bị loại, vòng sau rỗng |
| `id` + `bac`, `UNION`, chốt `bac < 12` | 13 dòng, 4 nút, `bac` lớn nhất 12 | Chỉ nhờ chốt; bỏ chốt là chạy mãi |

Tài liệu nói đúng điều này: một vòng thường không tạo dòng trùng hoàn toàn, nên khi truy vấn mang theo cột độ sâu thì `UNION` không cứu được.

### 5.4 Truy vấn kiểm tra cho cả bảng

Hai truy vấn của `19-chu-trinh.sql` nên chạy trước khi nhận bản cập nhật cây và định kỳ sau đó. Truy vấn (a) đi xuống từ gốc — luôn dừng — nên chạy **đầu tiên**:

```sql
-- a) Nút KHÔNG đi tới được từ gốc nào
WITH RECURSIVE tu_goc (id) AS (
  SELECT id FROM areas WHERE parent_id IS NULL
  UNION
  SELECT a.id FROM areas AS a JOIN tu_goc AS g ON a.parent_id = g.id
)
SELECT a.id, a.code
FROM areas AS a
WHERE NOT EXISTS (SELECT 1 FROM tu_goc AS g WHERE g.id = a.id)
ORDER BY a.id;
-- 7 dòng: 21 HN-TT, 22 HN-TA, 211 HN-TT-01, 212, 213, 221 HN-TA-01, 222
```

(b) **Nút nằm trên vòng**: neo là mọi nút, mang mảng đường đi, đi lên với điều kiện `WHERE NOT a.id = ANY (l.da_qua)` (dừng trước khi lặp), rồi giữ dòng mà `parent_id` bằng điểm xuất phát — bước kế tiếp sẽ quay về chính nó. Kết quả 4 dòng: 21, 22, 211, 221, mỗi dòng kèm vòng như `{21,221,22,211}`. Đủ để trả bản cập nhật kèm lý do; sau `ROLLBACK`, (a) trả 0 nút.

Với cây **cố định ba tầng** có cách rẻ hơn: K9 của `02-kiem-tra-fixture.sql` (cha của vùng phải là thành phố, cha của khu phải là vùng) trả đúng hai dòng `HN-TA`, `HN-TT` có cha cấp `zone` (thí nghiệm ngoài lab). Lý do: nếu đi lên luôn làm tầng **tăng nghiêm ngặt** thì không thể quay về chỗ cũ. Bài tập 3 biến điều đó thành ràng buộc schema. Khi có "tiểu khu" với độ sâu thay đổi, luật tầng cố định hết dùng được — lúc đó cần (a) và (b).

## 6. Lỗi thường gặp

Năm lỗi khi viết `WITH RECURSIVE`, đã chạy trên PostgreSQL 17.5 (thí nghiệm ngoài lab):

| Viết sai | SQLSTATE | Thông báo (rút gọn) |
|---|---|---|
| Quên chữ `RECURSIVE` | `42P01` | `relation "cay" does not exist` + gợi ý `Use WITH RECURSIVE` |
| Tham chiếu CTE ở phía `LEFT JOIN` | `42P19` | `… must not appear within an outer join` |
| Hàm gộp trong phần đệ quy | `42P19` | `aggregate functions are not allowed in a recursive query's recursive term` |
| Tham chiếu CTE hai lần | `42P19` | `… must not appear more than once` |
| Neo ép `varchar(40)`, phần đệ quy nối chuỗi | `42804` | `… character varying(40) in non-recursive term but type character varying overall` |

Phần neo quyết định kiểu cột: ép neo về đúng kiểu của cả cột (`code::text`) là chạy được. Những lỗi không có thông báo còn nguy hiểm hơn: neo "mọi nút" hoặc đi lên mà không chốt trên dữ liệu chưa kiểm; lọc trong phần đệ quy làm mất nhánh; tin thứ tự kết quả khi không `ORDER BY`; `count(*)` sau `LEFT JOIN`; cộng dồn theo cây mà không có câu đối chiếu "qua cây = đếm thẳng".

## 7. Đánh đổi

| Cách lưu và truy vấn cây | Ưu | Nhược |
|---|---|---|
| Danh sách kề + `WITH RECURSIVE` (bài này) | Sửa một nút = sửa một dòng; không giả định số tầng | Mỗi lần đọc phải duyệt; chu trình phải canh bằng truy vấn |
| Danh sách kề + JOIN theo số tầng (bài tập 2) | Đơn giản, cùng kết quả khi cây cố định | Thêm tầng là sửa mọi truy vấn |
| Bảng closure lưu sẵn | Đọc "mọi hậu duệ" chỉ là một JOIN | Đổi cây phải cập nhật nhiều dòng và giữ đồng bộ |
| Đường đi trong một cột (materialized path, hoặc kiểu `ltree` của extension cùng tên) | Tìm hậu duệ bằng so tiền tố | Chuyển nhánh là sửa cả nhánh; lab không dùng extension |

Hiệu năng (index `ix_areas_parent` cho JOIN đi xuống) để Phần 3 đo. Phần 4 sẽ gặp lại cây này: hai người sửa hai nút **cùng lúc**, từng thay đổi không tạo vòng nhưng ghép lại thì có — PGlite một kết nối không tái hiện được.

## 8. Bài tập

Lời giải không nằm trong luồng chính của lab; mỗi tệp đã chạy bằng `node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql <tệp>`.

**Bài 1.** Liệt kê các nút lá kèm đường đi đầy đủ từ gốc. Có bao nhiêu lá, và lá nào không phải khu?

<details><summary><b>Lời giải</b></summary>
<p>Đi xuống từ mọi gốc, mang <code>ARRAY[code]</code> làm đường đi, chốt <code>cardinality(c.duong_di) &lt; 10</code>; ở truy vấn ngoài giữ dòng không có con (<code>NOT EXISTS (SELECT 1 FROM areas AS con WHERE con.parent_id = c.id)</code>). Có 22 lá; lá duy nhất không phải khu là <code>HN-NA | district | HN &gt; HN-NA</code>. Đây là tín hiệu nghiệp vụ: vùng mới mở chưa chia khu thì chuyến ở đó không có khu đón hợp lệ theo K8 — phải chia khu trước khi nhận chuyến.</p>
</details>

**Bài 2.** Viết lại phép cộng dồn cấp vùng và thành phố của mục 4 **không dùng đệ quy**, dựa vào cây ba tầng. Đối chiếu, rồi nói cách này vỡ khi nào.

<details><summary><b>Lời giải</b></summary>
<p>CTE <code>chuyen_vung</code> nối mỗi chuyến có khu đón lên khu <code>z</code> rồi vùng <code>v = z.parent_id</code>, giữ <code>v.id AS vung_id</code> và <code>v.parent_id AS tp_id</code>; truy vấn ngoài lấy mọi nút không phải khu, <code>LEFT JOIN chuyen_vung AS cv ON cv.vung_id = n.id OR cv.tp_id = n.id</code>, rồi đếm và cộng doanh thu như mục 4. Ra đúng 13 dòng của mục 4 (HN 1.094 / 62.246.000, HN-NA 0 / 0…). Mỗi chuyến khớp đúng một vùng và một thành phố nên <code>OR</code> không làm nở dòng. Thêm tầng "tiểu khu" thì cách này vỡ mà không báo lỗi: <code>z.parent_id</code> khi đó là khu chứ không phải vùng. Cách này còn dựa vào K8 (khu đón luôn là nút <code>zone</code>), nên K8 phải chạy cùng báo cáo.</p>
</details>

**Bài 3.** Với cây cố định ba tầng, chặn chu trình ngay ở **schema**, rồi chứng minh bản cập nhật đảo cha bị từ chối.

<details><summary><b>Lời giải</b></summary>
<p>Lưu tầng của cha trên dòng con và bắt FK hai cột khớp tầng thật của cha, cộng CHECK thứ tự tầng — đi lên luôn tăng tầng nên không thể quay vòng:</p>
<pre><code class="language-sql">ALTER TABLE areas ADD COLUMN parent_level text;
UPDATE areas AS a SET parent_level = p.level FROM areas AS p WHERE p.id = a.parent_id;  -- UPDATE 31
ALTER TABLE areas ADD CONSTRAINT uq_areas_id_level UNIQUE (id, level);
ALTER TABLE areas ADD CONSTRAINT fk_areas_parent_level
  FOREIGN KEY (parent_id, parent_level) REFERENCES areas (id, level);
ALTER TABLE areas ADD CONSTRAINT ck_areas_level_order CHECK (
     (level = 'city'     AND parent_level IS NULL)
  OR (level = 'district' AND parent_level = 'city')
  OR (level = 'zone'     AND parent_level = 'district'));</code></pre>
<p><code>UPDATE areas SET parent_id = 221 WHERE id = 21</code> lỗi <code>23503 fk_areas_parent_level</code> (<code>Key (parent_id, parent_level)=(221, city) is not present</code>); sửa cả <code>parent_level = 'zone'</code> cho "khớp" thì lỗi <code>23514 ck_areas_level_order</code>. Chuyển một khu sang vùng khác cùng thành phố vẫn chạy (<code>UPDATE 1</code>). Đánh đổi: thêm một cột suy ra và một UNIQUE (như FK hai cột ở <a href="/articles/postgresql-mo-hinh-quan-he-chuan-hoa">PG-008</a>), và mọi thứ chết cùng giả định "số tầng cố định". Trên bảng thật, thêm FK theo <code>NOT VALID</code> → <code>VALIDATE</code> (<a href="/articles/postgresql-schema-migration-tuong-thich">PG-009</a>).</p>
</details>

## 9. Đọc tiếp

- [Tài liệu PostgreSQL 17 — WITH Queries](https://www.postgresql.org/docs/17/queries-with.html): thuật toán đánh giá, thứ tự tìm kiếm, phát hiện chu trình.
- [Tài liệu PostgreSQL 17 — SELECT, mệnh đề WITH](https://www.postgresql.org/docs/17/sql-select.html): cú pháp `SEARCH` và `CYCLE`.
- [Ghi chú phát hành PostgreSQL 14](https://www.postgresql.org/docs/release/14.0/): bản đưa vào `SEARCH` và `CYCLE`.
- Lab: `examples/toi-noi-postgresql/p02/19-cay-khu-vuc.sql`, `19-chu-trinh.sql`, kết quả thật trong `p02/OUTPUT.txt`.

Bài tiếp theo: [PG-020: Thực hành — truy vấn vận hành và đối soát thanh toán](/articles/postgresql-thuc-hanh-sql-doi-soat).

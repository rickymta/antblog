---
slug: postgresql-upsert-on-conflict
title: "UPSERT và xung đột khoá duy nhất: nhập thực đơn quán theo lô"
summary: "Quán Bún Bò Lab 101 gửi lại file thực đơn, cả lô bị từ chối vì một món lặp hai lần, còn lô cũ gửi muộn thì ghi đè giá mới. Dùng INSERT … ON CONFLICT, EXCLUDED và WHERE để nhập dữ liệu có quy tắc, chạy lại không đổi gì."
category: postgresql
level: intermediate
tags: [postgresql, pg-017, upsert, on-conflict, merge, unique, idempotency, toi-noi]
language: vi
---

Quán đối tác của Tới Nơi cập nhật thực đơn bằng cách tải lên một file: mỗi dòng một món, có mã món (`sku`), tên, giá, còn bán hay không. Tới Nơi nạp file vào bảng nhập rồi hợp nhất sang bảng thực đơn mà app khách đang đọc. Tuần trước, quán **Bún Bò Lab 101** gửi lô thứ hai — tăng giá, báo hết trà đá, thêm món — và **cả lô bị từ chối** vì món bún bò giò xuất hiện hai lần trong file. Sửa xong thì gặp lỗi khác: một dòng trùng tên món. Còn nếu hệ thống của quán gửi lại một file cũ, câu nhập «có rồi thì cập nhật» sẽ ghi giá cũ đè lên giá mới.

Không lỗi nào trong số đó là lỗi của PostgreSQL. Chúng là những quy tắc nhập dữ liệu mà câu lệnh chưa nói ra. Bài này viết các quy tắc đó thành SQL bằng **UPSERT** — `INSERT … ON CONFLICT` — và kiểm từng quy tắc bằng một lô dữ liệu cố ý trùng khoá.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-017 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-create-table-constraints">PG-005 · CREATE TABLE và ràng buộc</a>, <a href="/articles/postgresql-insert-update-delete-returning">PG-006 · INSERT, UPDATE, DELETE và RETURNING</a>, <a href="/articles/postgresql-subquery-exists-cte">PG-014 · Subquery, EXISTS và CTE</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>17-upsert-on-conflict.sql</code>). Sau bài này, bạn nhập được dữ liệu theo lô với quy tắc cập nhật rõ ràng, có fixture trùng khoá để chứng minh từng quy tắc.</p>
</div>

## 1. Bài toán: năm quy tắc nhập thực đơn

Trước khi viết SQL, viết quy tắc bằng lời. Lab dùng năm quy tắc:

| Mã | Quy tắc | Vì sao |
|---|---|---|
| R1 | Khoá món = (quán, `sku`): chưa có thì **thêm**, có rồi thì **cập nhật** | Hai quán có thể cùng dùng mã `BB-01` |
| R2 | Chỉ cập nhật khi `source_updated_at` (giờ sửa ở hệ thống quán) của dòng nhập **mới hơn** dòng đang có | Lô cũ gửi lại không đè dữ liệu mới |
| R3 | Dữ liệu không đổi thì **không ghi** | Tránh ghi thừa; mỗi UPDATE tạo một phiên bản dòng mới (Phần 4) |
| R4 | Trong cùng một lô, một món xuất hiện nhiều lần ⇒ lấy dòng mới nhất | File của quán có thể sửa một món hai lần |
| R5 | Dòng vi phạm ràng buộc khác (quán không tồn tại, trùng tên món khác) ⇒ **từ chối riêng dòng đó** | Một dòng hỏng không làm hỏng cả lô |

Bảng đích `menu_items` có khoá chính `pk_menu_items (merchant_id, sku)`, ràng buộc `uq_menu_items_name (merchant_id, name)` — trong một quán hai món không trùng tên —, FK tới `merchants` và `CHECK (price > 0)`; `updated_by_batch` ghi lô nào sửa dòng lần cuối. Bảng nhập `menu_item_imports` cố ý **không** có FK hay UNIQUE nghiệp vụ: nó nhận nguyên văn file, kể cả dòng sai, để còn đối chiếu và báo lại cho quán.

Lô 1 có 5 món ban đầu của quán (`e1000000-…-000000000101`), sửa lúc 20/10 08:00. Lô 2 có 8 dòng, mỗi dòng cài một tình huống:

| Dòng | sku | Nội dung | Tình huống |
|---|---|---|---|
| 1 | BB-01 | Bún bò đặc biệt 59.000 đ, 28/10 09:00 | Tăng giá từ 55.000 |
| 2 | BB-02 | Bún bò tái 45.000 đ, 28/10 09:00 | Y hệt, chỉ giờ mới hơn |
| 3 | BB-03 | Bún bò chả 42.000 đ, **15/10** 07:00 | Bản cũ gửi lại |
| 4 | TR-01 | Trà đá, `is_available = false` | Hết hàng |
| 5 | BB-04 | Bún bò giò 50.000 đ, 09:05 | Món mới… |
| 6 | BB-04 | Bún bò giò 52.000 đ, 09:30 | …sửa giá trong cùng lô |
| 7 | BB-05 | Nước cam 25.000 đ | `sku` mới nhưng trùng **tên** NC-01 |
| 8 | XX-01 | Món lạ, quán `…000000000999` | Quán không tồn tại |

## 2. UPSERT: INSERT … ON CONFLICT DO UPDATE

**UPSERT** (ghép từ *update* và *insert*) là thao tác «chưa có thì thêm, có rồi thì cập nhật». PostgreSQL viết nó thành một mệnh đề của `INSERT`:

```sql
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 1
ORDER BY line_no
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch;
```

```text
INSERT 0 5
```

Ba thành phần cần đọc đúng:

- **`(merchant_id, sku)`** là *conflict target* — các cột để PostgreSQL tìm ràng buộc UNIQUE/khoá chính làm **trọng tài** (*arbiter*). Dòng định chèn trùng khoá với trọng tài thì chuyển sang hành động thay thế; không thì chèn bình thường.
- **`EXCLUDED`** là bảng ảo chứa **dòng định chèn** (dòng bị xung đột). Dòng đang có trong bảng gọi bằng tên bảng hoặc bí danh — ở đây `m`. `SET price = EXCLUDED.price` nghĩa là «lấy giá trong file».
- **Thẻ lệnh** `INSERT 0 5`: theo tài liệu `INSERT`, con số cuối là số dòng **được chèn hoặc được cập nhật**. Lô 1 vào bảng trống nên cả 5 đều là chèn.

Vì sao không «SELECT xem có chưa, rồi INSERT hoặc UPDATE»? Giữa lúc đọc và lúc ghi, một phiên khác có thể chèn cùng khoá. Tài liệu PostgreSQL 17 cam kết `ON CONFLICT DO UPDATE` cho ra một trong hai kết cục — chèn hoặc cập nhật — một cách nguyên tử, kể cả khi nhiều phiên chạy đồng thời. Lab chỉ có một kết nối nên chưa chứng minh được ý này; để Phần 4.

## 3. Lỗi thứ nhất: một lô tự trùng khoá

Chạy đúng câu trên cho lô 2:

```text
ERROR:  ON CONFLICT DO UPDATE command cannot affect row a second time
HINT:  Ensure that no rows proposed for insertion within the same command have duplicate constrained values.
```

SQLSTATE `21000` (*cardinality violation* — vi phạm số lượng). Dòng 5 và 6 cùng khoá `(…101, BB-04)`: dòng 5 vừa được chèn thì dòng 6 xung đột với **chính dòng vừa chèn**. Tài liệu gọi câu lệnh này là «tất định»: không được tác động lên cùng một dòng hai lần, vì kết quả sẽ phụ thuộc thứ tự xử lý. PostgreSQL không đoán hộ dòng nào thắng — nó từ chối cả câu, kể cả 6 dòng không liên quan.

Quyết định «dòng nào thắng» là quy tắc R4, và ta phải tự viết nó. Cách gọn trong PostgreSQL là **`DISTINCT ON (biểu thức)`**: giữ **dòng đầu tiên** của mỗi nhóm cùng giá trị biểu thức, theo thứ tự của `ORDER BY`. `ORDER BY` phải bắt đầu bằng đúng các biểu thức trong `DISTINCT ON`, phần sau quyết định dòng nào đứng đầu:

```sql
SELECT DISTINCT ON (merchant_id, sku)
       merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2
ORDER BY merchant_id, sku, source_updated_at DESC, line_no DESC
```

Mỗi món còn một dòng — dòng có `source_updated_at` mới nhất; cùng giờ thì dòng nằm sau trong file thắng (`line_no DESC`). Thiếu khoá phụ này, hai dòng cùng giờ cho kết quả tuỳ kế hoạch thực thi — bẫy «thứ tự không xác định» của [PG-007](/articles/postgresql-select-filter-order-limit). `DISTINCT ON` là mở rộng riêng của PostgreSQL; ở hệ khác, cùng ý đó viết bằng `row_number()` ([PG-015](/articles/postgresql-window-function-frame)).

## 4. Lỗi thứ hai: ON CONFLICT chỉ bắt xung đột của trọng tài

Thay nguồn của câu UPSERT bằng truy vấn `DISTINCT ON` ở trên, chạy lại:

```text
ERROR:  duplicate key value violates unique constraint "uq_menu_items_name"
DETAIL:  Key (merchant_id, name)=(e1000000-0000-4000-8000-000000000101, Nước cam) already exists.
```

Dòng 7 (`BB-05`, «Nước cam») không đụng khoá `(merchant_id, sku)` — `BB-05` là mã mới — nhưng đụng ràng buộc **khác**: tên món trong quán. `ON CONFLICT (merchant_id, sku)` chỉ chuyển hướng xung đột của trọng tài được chỉ định; mọi ràng buộc còn lại (UNIQUE khác, FK, CHECK) vẫn sinh lỗi thường và huỷ cả câu.

Hai câu lỗi không để lại gì — lỗi thì huỷ cả câu, `SELECT count(*) FROM menu_items` vẫn ra `5`. R5 không thể giao cho `ON CONFLICT`: phải **lọc** dòng vi phạm ra trước bằng `EXISTS`/`NOT EXISTS` ([PG-014](/articles/postgresql-subquery-exists-cte)) và ghi chúng vào biên bản trả lời quán.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đừng «chữa» bằng ON CONFLICT DO NOTHING không có target</span>
<p>Bỏ target, <code>ON CONFLICT DO NOTHING</code> coi <b>mọi</b> UNIQUE và khoá chính là trọng tài: dòng 7 biến mất lặng lẽ với <code>INSERT 0 0</code>, quán không biết món không lên app. FK thì không bao giờ là trọng tài — dòng 8 vẫn gây <code>23503</code>. <code>DO UPDATE</code> thiếu target là lỗi <code>42601</code>. Thí nghiệm <code>pg017-do-nothing-khong-target.sql</code> (không có trong lab chính, nằm ở <code>p02/thu-nghiem/</code>) chạy cả ba.</p>
</div>

## 5. Xem trước: biên bản nhập từng dòng

Trước khi ghi, phân loại **từng dòng** của lô theo R1–R5. Đây vừa là bước kiểm tra (dry run), vừa là biên bản gửi lại cho quán:

```sql
SELECT i.line_no, i.sku,
       CASE
         WHEN NOT EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
           THEN 'TỪ CHỐI — quán không tồn tại (R5)'
         WHEN EXISTS (SELECT 1 FROM menu_items AS x
                      WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
           THEN 'TỪ CHỐI — trùng tên món khác (R5)'
         WHEN i.line_no <> first_value(i.line_no) OVER (PARTITION BY i.merchant_id, i.sku
                                                        ORDER BY i.source_updated_at DESC, i.line_no DESC)
           THEN 'BỎ QUA — có dòng mới hơn trong cùng lô (R4)'
         WHEN m.sku IS NULL
           THEN 'THÊM'
         WHEN i.source_updated_at <= m.source_updated_at
           THEN 'BỎ QUA — cũ hơn dữ liệu đang có (R2)'
         WHEN (i.name, i.price, i.is_available) IS NOT DISTINCT FROM (m.name, m.price, m.is_available)
           THEN 'BỎ QUA — không có gì thay đổi (R3)'
         ELSE 'CẬP NHẬT'
       END AS ket_qua
FROM menu_item_imports AS i
LEFT JOIN menu_items AS m ON m.merchant_id = i.merchant_id AND m.sku = i.sku
WHERE i.batch_id = 2
ORDER BY i.line_no;
```

```text
 line_no |  sku  |                   ket_qua
---------+-------+---------------------------------------------
       1 | BB-01 | CẬP NHẬT
       2 | BB-02 | BỎ QUA — không có gì thay đổi (R3)
       3 | BB-03 | BỎ QUA — cũ hơn dữ liệu đang có (R2)
       4 | TR-01 | CẬP NHẬT
       5 | BB-04 | BỎ QUA — có dòng mới hơn trong cùng lô (R4)
       6 | BB-04 | THÊM
       7 | BB-05 | TỪ CHỐI — trùng tên món khác (R5)
       8 | XX-01 | TỪ CHỐI — quán không tồn tại (R5)
(8 rows)
```

Thứ tự nhánh `CASE` là thứ tự ưu tiên: từ chối trước, rồi khử trùng lặp trong lô, rồi mới so với dữ liệu đang có. `first_value(...) OVER (...)` ([PG-015](/articles/postgresql-window-function-frame)) chọn dòng mới nhất mỗi nhóm (quán, sku) — cùng tiêu chí với `DISTINCT ON`. So ba cột bằng **`IS NOT DISTINCT FROM`** để NULL được coi là bằng nhau ([PG-011](/articles/postgresql-null-case-coalesce)).

Kỳ vọng đã rõ trước khi ghi: **1 thêm, 2 cập nhật, 3 bỏ qua, 2 từ chối**.

## 6. DO UPDATE … WHERE: không ghi đè dữ liệu mới, không ghi thừa

R2 và R3 là điều kiện **cho từng dòng xung đột**, nên chúng nằm trong mệnh đề `WHERE` của `DO UPDATE`. R4 và R5 nằm ở nguồn:

```sql
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT DISTINCT ON (i.merchant_id, i.sku)
       i.merchant_id, i.sku, i.name, i.price, i.is_available, i.source_updated_at, i.batch_id
FROM menu_item_imports AS i
WHERE i.batch_id = 2
  AND EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)                  -- R5
  AND NOT EXISTS (SELECT 1 FROM menu_items AS x
                  WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
ORDER BY i.merchant_id, i.sku, i.source_updated_at DESC, i.line_no DESC              -- R4
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch
WHERE EXCLUDED.source_updated_at > m.source_updated_at                                -- R2
  AND (m.name, m.price, m.is_available)
      IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.price, EXCLUDED.is_available)          -- R3
RETURNING m.sku, m.price, m.is_available, m.updated_by_batch;
```

```text
  sku  | price | is_available | updated_by_batch
-------+-------+--------------+------------------
 BB-01 | 59000 | t            |                2
 BB-04 | 52000 | t            |                2
 TR-01 |  5000 | f            |                2
(3 rows)

INSERT 0 3
```

Khớp biên bản: `INSERT 0 3` = 1 thêm (BB-04, giá 52.000 của dòng 6) + 2 cập nhật. BB-02 và BB-03 **xung đột nhưng không được cập nhật** vì `WHERE` trả false — theo tài liệu `INSERT`, chúng không được đếm trong thẻ lệnh và không có trong `RETURNING`. Chỉ đọc thẻ lệnh thì không biết dòng nào bị bỏ qua vì sao; biên bản mục 5 mới nói được.

Đọc lại `menu_items` (lab in đủ 6 món): BB-03 vẫn 45.000 đ — bản cũ 42.000 đ ngày 15/10 không đè được; BB-02 vẫn mang `updated_by_batch = 1` vì không có lần ghi nào; BB-01, BB-04, TR-01 mang số lô 2.

Một chi tiết của tài liệu cần biết trước Phần 4: dòng bị `WHERE` loại vẫn **bị khoá** trong lúc câu lệnh chạy, chỉ là không bị sửa. «Không ghi» không có nghĩa là «không chạm».

## 7. Idempotent: gửi lại lô không đổi gì

Một thao tác **idempotent** (luỹ đẳng) là thao tác chạy một lần hay nhiều lần cho cùng kết quả. Cổng đối tác thật sẽ gửi lại lô khi mạng chập chờn hoặc khi quán bấm «Tải lên» hai lần. Chạy lại đúng câu ở mục 6:

```text
INSERT 0 0
```

Lần hai, mọi dòng hoặc cũ hơn hoặc không đổi: R2 và R3 loại hết. R2 dùng `>` chứ không `>=`: với `>=`, dòng cùng giờ nguồn lọt qua R2 và chỉ còn R3 chặn. Bài tập 2 thử chiều ngược lại — bỏ R3.

## 8. DO NOTHING: chỉ thêm món mới

Có khi quy tắc đơn giản hơn: «chỉ thêm món chưa có, món đã có thì để nguyên». Lô 3 có BB-01 với giá 99.000 đ và một món mới CF-01:

```sql
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 3
ORDER BY line_no
ON CONFLICT (merchant_id, sku) DO NOTHING
RETURNING sku, price, updated_by_batch;
```

```text
  sku  | price | updated_by_batch
-------+-------+------------------
 CF-01 | 22000 |                3
(1 row)

INSERT 0 1
```

BB-01 bị bỏ qua lặng lẽ (giá vẫn `59000`); `RETURNING` chỉ trả dòng thật sự được chèn. `DO NOTHING` hợp với dữ liệu «ghi một lần» như sự kiện có mã duy nhất; với dữ liệu đổi theo thời gian như giá, nó làm mất cập nhật thật.

## 9. Conflict target phải khớp một ràng buộc

PostgreSQL tìm trọng tài bằng **suy luận chỉ mục duy nhất** (*unique index inference*): một ràng buộc UNIQUE/khoá chính chứa **đúng** các cột trong target, không hơn không kém (thứ tự không quan trọng). Không tìm được thì lỗi ngay khi lập kế hoạch:

```sql
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
VALUES ('e1000000-0000-4000-8000-000000000101', 'CF-02', 'Cà phê đen', 18000, true, '2026-10-30 08:00+07', 3)
ON CONFLICT (sku) DO NOTHING;
```

```text
ERROR:  there is no unique or exclusion constraint matching the ON CONFLICT specification
```

SQLSTATE `42P10`: `sku` chỉ duy nhất **trong một quán**. Lỗi này là tin tốt — nó buộc ta nói đúng khoá nghiệp vụ. Có thể gọi tên ràng buộc (`ON CONFLICT ON CONSTRAINT pk_menu_items DO NOTHING`, lab ra `INSERT 0 0`), nhưng tài liệu khuyên liệt kê cột: đổi tên ràng buộc thì câu vẫn chạy. Khoá là chỉ mục duy nhất có điều kiện (partial unique index, Phần 3) thì target thêm `WHERE <điều kiện>`.

## 10. MERGE: một câu, nhiều nhánh

PostgreSQL 15 có **`MERGE`** theo chuẩn SQL: nối nguồn với đích theo `ON`, rồi chọn hành động cho `WHEN MATCHED` / `WHEN NOT MATCHED`. Từ **PostgreSQL 17**, `MERGE` có `RETURNING`, hàm `merge_action()` (dòng đã `INSERT`, `UPDATE` hay `DELETE`) và đặt được trong `WITH`. Lab chạy nó trong `BEGIN … ROLLBACK` chỉ để xem (bản đầy đủ ở mục 4 tệp 17):

```sql
WITH ket_qua AS (
MERGE INTO menu_items AS m
USING ( /* nguồn đã lọc R5, khử trùng R4 như mục 6 */ ) AS s
ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED AND /* R2 và R3 */ THEN UPDATE SET price = s.price, /* … */ updated_by_batch = s.batch_id
WHEN NOT MATCHED THEN INSERT (/* … */) VALUES (/* … */)
RETURNING merge_action() AS hanh_dong, m.sku, m.price, m.is_available
)
SELECT * FROM ket_qua ORDER BY sku;
```

```text
 hanh_dong |  sku  | price | is_available
-----------+-------+-------+--------------
 UPDATE    | BB-01 | 59000 | t
 INSERT    | BB-04 | 52000 | t
 UPDATE    | TR-01 |  5000 | f
(3 rows)
```

Cùng 1 thêm, 2 cập nhật như UPSERT. `MERGE` diễn đạt được nhiều nhánh hơn (xoá món không còn trong file bằng `WHEN NOT MATCHED BY SOURCE`), nhưng hai câu **không thay thế cho nhau**: tài liệu `MERGE` nói nó theo luật cô lập giao dịch thông thường khi chạy đồng thời, và gợi ý `INSERT … ON CONFLICT` nếu cần chuyển sang UPDATE khi phiên khác vừa chèn cùng khoá (Phần 4). Mỗi dòng đích cũng chỉ được khớp một dòng nguồn, nên R4 vẫn cần.

## 11. Lỗi thường gặp và đánh đổi

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `21000 … cannot affect row a second time`, cả lô bị từ chối | Lô có hai dòng cùng khoá | Khử trùng lặp trước (`DISTINCT ON` với thứ tự xác định) |
| `23505` dù đã có `ON CONFLICT` | Vi phạm một UNIQUE khác trọng tài | Lọc dòng vi phạm ra, ghi vào biên bản |
| `23514`, một dòng giá 0 làm hỏng cả lô | CHECK không phải việc của `ON CONFLICT` | Đưa luật CHECK vào bộ lọc nguồn (bài tập 1) |
| Giá mới bị giá cũ đè | `DO UPDATE` không có điều kiện thời điểm | `WHERE EXCLUDED.<giờ nguồn> > <bảng>.<giờ nguồn>` |
| Chạy lại lô vẫn «cập nhật» hàng loạt | Thiếu R3 | So sánh `IS DISTINCT FROM` trước khi ghi |

Đánh đổi:

- **Chỉ UPSERT hay thêm biên bản:** UPSERT gọn và nguyên tử nhưng không nói dòng nào bị bỏ qua vì sao; lab chạy cả hai.
- **Lọc R5 trước hay để lỗi:** lọc trước giữ được lô nhưng phải chép luật ràng buộc vào truy vấn — đổi ràng buộc mà quên bộ lọc thì lô lại hỏng; ràng buộc trong bảng vẫn là chốt chặn cuối.
- **Giờ nguồn hay số lô làm tiêu chí «mới hơn»:** giờ nguồn đúng nghiệp vụ nhưng phụ thuộc đồng hồ của quán; số lô thì lô gửi lại mang số mới. Cần thống nhất với đối tác.

## 12. Bài tập

**Bài 1.** Sau toàn bộ tệp 17, quán gửi lô 4 gồm ba dòng: BB-02 hết hàng (giờ 30/10 10:00), BB-03 bản cũ 15/10 gửi lại lần nữa, và NC-01 với **giá 0 đ** do lỗi file (giờ 30/10 10:00). Chạy câu UPSERT của mục 6 với `batch_id = 4` thì ra gì? Sửa để chỉ dòng hỏng bị từ chối.

<details><summary><b>Lời giải</b></summary>
<p>Câu của mục 6 lỗi cả câu: <code>23514 new row for relation "menu_items" violates check constraint "ck_menu_items_price_positive"</code>, chi tiết chỉ ra dòng NC-01 giá 0. Không dòng nào được ghi, kể cả BB-02 hợp lệ. Sửa bằng cách thêm luật của CHECK vào bộ lọc nguồn:</p>
<pre><code class="language-sql">WHERE i.batch_id = 4
  AND i.price &gt; 0                    -- luật của ck_menu_items_price_positive
  AND EXISTS (...)                     -- R5 như mục 6
  AND NOT EXISTS (...)</code></pre>
<p>Kết quả chạy thật: <code>RETURNING</code> một dòng <code>BB-02 | 45000 | f | 4</code>, thẻ <code>INSERT 0 1</code>. BB-03 bị R2 loại, NC-01 bị lọc nên vẫn giá 25.000 đ — biên bản nhập phải thêm nhánh «TỪ CHỐI — giá không hợp lệ». Không có trong lab chính, nằm ở <code>p02/thu-nghiem/pg017-bai-tap-1-gia-0.sql</code>.</p>
</details>

**Bài 2.** Bỏ R3, chỉ giữ R2 trong `WHERE` của `DO UPDATE`. Từ trạng thái sau lô 1, lô 2 ghi những món nào? Gửi lại lô 2 lần nữa thì sao?

<details><summary><b>Lời giải</b></summary>
<p>Ghi 4 món: BB-01, <b>BB-02</b>, BB-04, TR-01 — tất cả mang <code>updated_by_batch = 2</code>. BB-02 không đổi tên, giá hay tình trạng, nhưng giờ nguồn 28/10 mới hơn 20/10 nên R2 cho qua: một lần ghi thừa, và cột <code>updated_by_batch</code> báo sai rằng lô 2 đã sửa món này. Gửi lại lần nữa: 0 dòng, vì R2 dùng <code>&gt;</code> và giờ nguồn giờ đã bằng nhau. Vậy R2 đủ cho tính idempotent, còn R3 cần cho «không ghi khi không đổi». Thí nghiệm chạy trong <code>BEGIN … ROLLBACK</code>, ở <code>p02/thu-nghiem/pg017-bai-tap-2-bo-r3.sql</code>.</p>
</details>

## 13. Đọc tiếp

- Trước đó: [PG-016 · Ngày giờ và múi giờ trong truy vấn](/articles/postgresql-date-time-timezone).
- `RETURNING` và thẻ lệnh: [PG-006](/articles/postgresql-insert-update-delete-returning). Ràng buộc UNIQUE, FK, CHECK: [PG-005](/articles/postgresql-create-table-constraints).
- Tài liệu PostgreSQL 17: [INSERT (mục ON CONFLICT)](https://www.postgresql.org/docs/17/sql-insert.html), [MERGE](https://www.postgresql.org/docs/17/sql-merge.html), [SELECT (DISTINCT ON)](https://www.postgresql.org/docs/17/sql-select.html), [Merge Support Functions](https://www.postgresql.org/docs/17/functions-merge-support.html).
- Lab: `examples/toi-noi-postgresql/p02/17-upsert-on-conflict.sql`, kết quả đầy đủ ở `p02/OUTPUT.txt`.

Bài tiếp theo: [PG-018 · Phân trang lịch sử chuyến bằng keyset](/articles/postgresql-pagination-keyset).

---
slug: mssql-upsert-merge
title: "Upsert: MERGE và các mẫu an toàn hơn khi nhập thực đơn quán theo lô"
summary: "Quán Bún Bò Lab 101 gửi lô thực đơn thứ hai và cả lô bị SQL Server từ chối với lỗi 2627, sửa xong lại gặp 2627 ở một ràng buộc khác, gửi lại thì gặp 8672. Bài dùng MERGE với WHEN MATCHED AND, NOT MATCHED BY TARGET/BY SOURCE, $action trong OUTPUT, giải thích vì sao MERGE cần HOLDLOCK khi chạy đồng thời và khi nào nên dùng mẫu UPDATE rồi INSERT … WHERE NOT EXISTS."
category: mssql
level: intermediate
tags: [mssql, ms-017, sql-server-2022, t-sql, merge, upsert, output, idempotency, toi-noi]
language: vi
---

Quán đối tác của Tới Nơi cập nhật thực đơn bằng cách tải lên một file: mỗi dòng một món, có mã món (`sku`), tên, giá, còn bán hay không. Tới Nơi nạp file vào bảng nhập rồi hợp nhất sang bảng thực đơn mà app khách đang đọc. Quán **Bún Bò Lab 101** gửi lô thứ hai: tăng giá, báo hết trà đá, thêm món. Câu `MERGE` "có rồi thì cập nhật, chưa có thì thêm" của đội backend từ chối **cả lô** với lỗi 2627 vì món bún bò giò xuất hiện hai lần trong file. Sửa xong thì gặp lại 2627, lần này ở ràng buộc tên món. Còn nếu hệ thống của quán gửi lại một file cũ, câu lệnh sẽ ghi giá cũ đè lên giá mới.

Không lỗi nào trong số đó là lỗi của SQL Server. Chúng là những quy tắc nhập dữ liệu mà câu lệnh chưa nói ra. Bài này viết các quy tắc đó thành T-SQL bằng **upsert** (thao tác "có thì cập nhật, chưa có thì thêm", ghép từ *update* và *insert*): trước hết bằng `MERGE`, rồi bằng mẫu hai câu mà nhiều đội SQL Server chọn thay cho `MERGE`.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-017 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-create-table-constraints">MS-005 · CREATE TABLE, ràng buộc và IDENTITY</a>, <a href="/articles/mssql-insert-update-delete-output">MS-006 · INSERT, UPDATE, DELETE và mệnh đề OUTPUT</a>, <a href="/articles/mssql-subquery-exists-cte-temp-table">MS-014 · Subquery, EXISTS, CTE và bảng tạm</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>17-upsert-merge.sql</code>). Sau bài này, bạn nhập được dữ liệu theo lô với quy tắc cập nhật rõ ràng, đọc được lỗi 2627/8672 của MERGE và biết vì sao MERGE cần <code>HOLDLOCK</code> khi chạy đồng thời.</p>
</div>

## 1. Bài toán: năm quy tắc nhập thực đơn

Trước khi viết SQL, viết quy tắc bằng lời:

| Mã | Quy tắc | Vì sao |
|---|---|---|
| R1 | Khoá món = (quán, `sku`): chưa có thì **thêm**, có rồi thì **cập nhật** | Hai quán có thể cùng dùng mã `BB-01` |
| R2 | Chỉ cập nhật khi `source_updated_at` (giờ sửa ở hệ thống quán) của dòng nhập **mới hơn** dòng đang có | Lô cũ gửi lại không đè dữ liệu mới |
| R3 | Dữ liệu không đổi thì **không ghi** | Một câu UPDATE không đổi giá trị vẫn khoá dòng, vẫn kích hoạt trigger nếu có, và đổi `updated_by_batch` thành một thông tin sai |
| R4 | Trong cùng một lô, một món xuất hiện nhiều lần ⇒ lấy dòng mới nhất | File của quán có thể sửa một món hai lần |
| R5 | Dòng vi phạm ràng buộc khác (quán không tồn tại, trùng tên món khác) ⇒ **từ chối riêng dòng đó** | Một dòng hỏng không làm hỏng cả lô |

Tệp lab tự tạo hai bảng trong schema `toinoi` (có `DROP TABLE IF EXISTS` trước nên chạy lại được). Bảng đích `toinoi.menu_items` có khoá chính `pk_menu_items (merchant_id, sku)`, ràng buộc `uq_menu_items_name (merchant_id, name)` (trong một quán, hai món không trùng tên), khoá ngoại tới `merchants` và `CHECK (price > 0)`; cột `updated_by_batch` ghi lô nào sửa dòng lần cuối. Bảng nhập `toinoi.menu_item_imports` cố ý **không** có khoá ngoại hay UNIQUE nghiệp vụ: nó nhận nguyên văn file, kể cả dòng sai, để còn đối chiếu và báo lại cho quán.

Lô 1 có 5 món ban đầu của quán (`e1000000-…-000000000101`), sửa lúc 20/10 08:00. Lô 2 có 8 dòng, mỗi dòng cài một tình huống:

| Dòng | sku | Nội dung | Tình huống |
|---|---|---|---|
| 1 | BB-01 | Bún bò đặc biệt 59.000 đ, 28/10 09:00 | Tăng giá từ 55.000 |
| 2 | BB-02 | Bún bò tái 45.000 đ, 28/10 09:00 | Y hệt, chỉ giờ mới hơn |
| 3 | BB-03 | Bún bò chả 42.000 đ, **15/10** 07:00 | Bản cũ gửi lại |
| 4 | TR-01 | Trà đá, `is_available = 0` | Hết hàng |
| 5 | BB-04 | Bún bò giò 50.000 đ, 09:05 | Món mới… |
| 6 | BB-04 | Bún bò giò 52.000 đ, 09:30 | …sửa giá trong cùng lô |
| 7 | BB-05 | Nước cam 25.000 đ | `sku` mới nhưng trùng **tên** NC-01 |
| 8 | XX-01 | Món lạ, quán `…000000000999` | Quán không tồn tại |

Lab chạy trên database riêng của P02 (`00` → `02` dựng lược đồ và fixture, xem `examples/toi-noi-mssql/p02`), rồi tới tệp 17. Mọi số trong bài lấy từ lần chạy trên SQL Server 2022 CU28 trong Docker; các dòng `PASS …` là kiểm tra của lab.

## 2. MERGE: một câu, nhiều nhánh

**`MERGE`** nối một **nguồn** (`USING`) với một **đích** theo điều kiện `ON`, rồi chọn hành động cho từng dòng theo nhánh `WHEN`:

| Nhánh | Dòng nào rơi vào | Hành động cho phép |
|---|---|---|
| `WHEN MATCHED [AND …]` | Dòng đích khớp một dòng nguồn | `UPDATE` hoặc `DELETE` |
| `WHEN NOT MATCHED [BY TARGET] [AND …]` | Dòng nguồn không khớp dòng đích nào | `INSERT` |
| `WHEN NOT MATCHED BY SOURCE [AND …]` | Dòng đích không khớp dòng nguồn nào | `UPDATE` hoặc `DELETE` |

Lô 1 đi vào bảng rỗng, nên mọi dòng rơi vào nhánh `NOT MATCHED BY TARGET`:

```sql
DECLARE @kq TABLE (hanh_dong nvarchar(10), sku varchar(20));

MERGE toinoi.menu_items AS m
USING (SELECT * FROM toinoi.menu_item_imports WHERE batch_id = 1) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED THEN
    UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
               source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku INTO @kq;          -- MERGE BẮT BUỘC kết thúc bằng ";"

SELECT hanh_dong, COUNT(*) AS so_dong FROM @kq GROUP BY hanh_dong ORDER BY hanh_dong;
```

```text
hanh_dong|so_dong
---------|-------
INSERT|5
PASS MERGE lô 1: @@ROWCOUNT = 5
```

Ba điểm của T-SQL:

- **`$action`** chỉ dùng được trong `OUTPUT` của `MERGE`, trả `INSERT`, `UPDATE` hoặc `DELETE` cho từng dòng. `inserted` và `deleted` vẫn là hai bảng ảo như ở [MS-006](/articles/mssql-insert-update-delete-output): dòng `INSERT` có `deleted.*` là NULL.
- **Thứ tự dòng của OUTPUT không được bảo đảm.** Lab ghi vào biến bảng `@kq` rồi mới `SELECT … ORDER BY`.
- **Dấu `;` cuối câu là bắt buộc** với `MERGE`. Thiếu nó, cả batch bị từ chối ngay khi biên dịch (đã chạy, mục 10 của tệp 17): `Msg 10713 … A MERGE statement must be terminated by a semi-colon (;).`

## 3. Lỗi thứ nhất: lô tự trùng khoá, 2627 trên khoá chính

Chạy đúng câu trên với `batch_id = 2`:

```text
Msg 2627: Violation of PRIMARY KEY constraint 'pk_menu_items'. Cannot insert duplicate key in object 'toinoi.menu_items'. The duplicate key value is (e1000000-0000-4000-8000-000000000101, BB-04).
PASS MERGE lô 2 ngây thơ: số lỗi = 2627
```

`MERGE` quyết định nhánh cho mọi dòng nguồn dựa trên đích **lúc câu lệnh bắt đầu**. BB-04 chưa có trong đích, nên cả dòng 5 lẫn dòng 6 đều rơi vào `NOT MATCHED BY TARGET`. Hai lệnh chèn cùng khoá, khoá chính từ chối dòng thứ hai, và **cả câu bị huỷ**: không dòng nào của lô 2 được ghi, kể cả BB-01 hợp lệ.

Khử trùng lặp trong lô (R4) bằng `ROW_NUMBER()` ([MS-015](/articles/mssql-window-function-window-clause)): đánh số các dòng cùng `(merchant_id, sku)` theo giờ sửa giảm dần, `line_no` làm tie-breaker, rồi giữ dòng số 1:

```sql
USING (SELECT *
       FROM (SELECT i.*, ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                            ORDER BY i.source_updated_at DESC, i.line_no DESC) AS rn
             FROM toinoi.menu_item_imports AS i
             WHERE i.batch_id = 2) AS x
       WHERE x.rn = 1) AS s
```

## 4. Lỗi thứ hai: MERGE chỉ so khớp theo ON

Đã khử trùng lặp, vẫn lỗi:

```text
Msg 2627: Violation of UNIQUE KEY constraint 'uq_menu_items_name'. Cannot insert duplicate key in object 'toinoi.menu_items'. The duplicate key value is (e1000000-0000-4000-8000-000000000101, Nước cam).
PASS MERGE lô 2 đã khử trùng lặp: số lỗi = 2627
PASS menu_items sau ba câu lỗi = 5
```

Điều kiện `ON` chỉ nói `(merchant_id, sku)`. BB-05 có `sku` mới nên được chèn, nhưng tên "Nước cam" trùng món NC-01 đang có, và `uq_menu_items_name` không nằm trong `ON`. `MERGE` không "bắt" xung đột của một ràng buộc mà nó không so khớp; đó là lỗi 2627 bình thường. Dòng 8 (quán không tồn tại) cũng làm câu lỗi: bỏ riêng BB-05 ra rồi chạy lại thì nhận `Msg 547: The MERGE statement conflicted with the FOREIGN KEY constraint "fk_menu_items_merchant" …`. Ba câu lỗi không để lại gì: bảng vẫn 5 món.

Kết luận của hai mục: **nguồn phải sạch trước khi vào MERGE**. R4 làm bằng `ROW_NUMBER()`, R5 làm bằng `EXISTS`/`NOT EXISTS`, còn ràng buộc trong bảng là chốt chặn cuối.

## 5. Xem trước: biên bản nhập từng dòng

Trước khi ghi, phân loại từng dòng của lô theo R1–R5. Kết quả này cũng là biên bản nhập gửi lại cho quán:

```sql
SELECT i.line_no, i.sku,
       CASE
         WHEN NOT EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
           THEN N'TỪ CHỐI — quán không tồn tại (R5)'
         WHEN EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                      WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
           THEN N'TỪ CHỐI — trùng tên món khác (R5)'
         WHEN ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                 ORDER BY i.source_updated_at DESC, i.line_no DESC) > 1
           THEN N'BỎ QUA — có dòng mới hơn trong cùng lô (R4)'
         WHEN m.sku IS NULL                          THEN N'THÊM'
         WHEN i.source_updated_at <= m.source_updated_at
           THEN N'BỎ QUA — cũ hơn dữ liệu đang có (R2)'
         WHEN i.name IS NOT DISTINCT FROM m.name AND i.price IS NOT DISTINCT FROM m.price
              AND i.is_available IS NOT DISTINCT FROM m.is_available
           THEN N'BỎ QUA — không có gì thay đổi (R3)'
         ELSE N'CẬP NHẬT'
       END AS ket_qua
FROM toinoi.menu_item_imports AS i
LEFT JOIN toinoi.menu_items AS m ON m.merchant_id = i.merchant_id AND m.sku = i.sku
WHERE i.batch_id = 2;
```

```text
line_no|sku|ket_qua
-------|---|-------
1|BB-01|CẬP NHẬT
2|BB-02|BỎ QUA — không có gì thay đổi (R3)
3|BB-03|BỎ QUA — cũ hơn dữ liệu đang có (R2)
4|TR-01|CẬP NHẬT
5|BB-04|BỎ QUA — có dòng mới hơn trong cùng lô (R4)
6|BB-04|THÊM
7|BB-05|TỪ CHỐI — trùng tên món khác (R5)
8|XX-01|TỪ CHỐI — quán không tồn tại (R5)
```

Lab ghi kết quả vào bảng tạm rồi in theo `line_no`. **`IS [NOT] DISTINCT FROM`** có từ SQL Server 2022: so sánh coi hai NULL là bằng nhau và không bao giờ trả UNKNOWN. Ba cột ở đây đều `NOT NULL` nên `=` cũng đúng, nhưng thói quen này an toàn khi bảng có cột cho phép NULL: `NULL <> 5` là UNKNOWN, và một điều kiện R3 viết bằng `<>` sẽ bỏ qua đúng những dòng vừa đổi từ NULL sang có giá trị.

Dự kiến: 1 thêm, 2 cập nhật, 3 bỏ qua, 2 từ chối.

## 6. MERGE đúng quy tắc

Nguồn đã lọc R5 và khử trùng lặp R4; R2 và R3 nằm ở điều kiện của nhánh `WHEN MATCHED AND …`. Lab chạy câu này trong `BEGIN TRANSACTION … ROLLBACK` để xem kết quả, rồi ghi thật bằng mẫu ở mục 8:

```sql
MERGE toinoi.menu_items WITH (HOLDLOCK) AS m
USING ( /* nguồn: R5 bằng EXISTS / NOT EXISTS, R4 bằng ROW_NUMBER() … WHERE rn = 1 */ ) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED AND s.source_updated_at > m.source_updated_at                        -- R2
             AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
                  OR s.is_available IS DISTINCT FROM m.is_available) THEN          -- R3
    UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
               source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku, deleted.price, inserted.price, inserted.is_available INTO @kq;
```

```text
hanh_dong|sku|gia_cu|gia_moi|con_ban
---------|---|------|-------|-------
UPDATE|BB-01|55000|59000|1
INSERT|BB-04|NULL|52000|1
UPDATE|TR-01|5000|5000|0
PASS MERGE đúng quy tắc: @@ROWCOUNT = 3
```

Đúng dự kiến của bảng xem trước. BB-02 và BB-03 khớp `ON` nhưng không thoả điều kiện `AND`, nên không có nhánh nào chạy cho chúng: không UPDATE, không có trong OUTPUT, không tính vào `@@ROWCOUNT`. Muốn biết dòng nào bị bỏ qua và vì sao, đọc biên bản ở mục 5, không phải OUTPUT. TR-01 giữ giá 5.000 đ nhưng vẫn là một lần cập nhật hợp lệ, vì `is_available` đổi từ 1 sang 0.

## 7. MERGE khi có nhiều phiên cùng chạy

Câu trên có `WITH (HOLDLOCK)` trên bảng đích. `HOLDLOCK` tương đương `SERIALIZABLE`: giữ khoá tới hết transaction, kể cả khoá trên **khoảng khoá chưa có dòng** (key-range lock). Lý do:

- `MERGE` đọc đích để quyết định nhánh, rồi mới ghi. Ở mức cô lập mặc định (Read Committed), khoá đọc được nhả sớm.
- Hai phiên cùng nhập một món **chưa có** (ví dụ hai lần gửi file cùng chứa BB-09) có thể cùng thấy "không khớp", cùng vào nhánh `INSERT`, và phiên chậm hơn nhận lỗi 2627.
- Tài liệu `MERGE` của Microsoft nêu rõ trường hợp này và khuyên dùng `HOLDLOCK` khi `MERGE` được dùng làm upsert.

`HOLDLOCK` đổi race condition lấy chờ khoá: phiên thứ hai đợi phiên thứ nhất commit, rồi thấy BB-09 đã có và đi vào nhánh `MATCHED`. Cái giá là khoá rộng hơn và nguy cơ deadlock cao hơn. Bài này chỉ giải thích; tái hiện bằng hai phiên, đọc khoá và xử lý deadlock nằm ở MS-036 và MS-038 (Phần 4).

## 8. Mẫu an toàn hơn: UPDATE rồi INSERT … WHERE NOT EXISTS

Nhiều đội SQL Server không dùng `MERGE` cho upsert mà viết hai câu trong một transaction. Mỗi câu làm đúng một việc, đọc và kiểm thử riêng được, và khoá được nêu rõ trên từng câu:

```sql
SET XACT_ABORT ON;
-- #nguon: 5 dòng của lô 2 sau R4 + R5 (lab dựng bằng SELECT … INTO)

BEGIN TRANSACTION;

UPDATE m WITH (UPDLOCK, SERIALIZABLE)
SET name = s.name, price = s.price, is_available = s.is_available,
    source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
OUTPUT N'UPDATE' AS hanh_dong, inserted.sku, deleted.price AS gia_cu, inserted.price AS gia_moi
FROM toinoi.menu_items AS m
JOIN #nguon AS s ON s.merchant_id = m.merchant_id AND s.sku = m.sku
WHERE s.source_updated_at > m.source_updated_at                                    -- R2
  AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
       OR s.is_available IS DISTINCT FROM m.is_available);                         -- R3

INSERT INTO toinoi.menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
OUTPUT N'INSERT' AS hanh_dong, inserted.sku, inserted.price AS gia_moi
SELECT s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id
FROM #nguon AS s
WHERE NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS m WITH (UPDLOCK, SERIALIZABLE)
                  WHERE m.merchant_id = s.merchant_id AND m.sku = s.sku);

COMMIT TRANSACTION;
```

```text
hanh_dong|sku|gia_cu|gia_moi
---------|---|------|-------
UPDATE|BB-01|55000|59000
UPDATE|TR-01|5000|5000
hanh_dong|sku|gia_moi
---------|---|-------
INSERT|BB-04|52000
PASS mẫu UPDATE rồi INSERT: số dòng cập nhật = 2
PASS mẫu UPDATE rồi INSERT: số dòng thêm = 1
```

Cùng 2 cập nhật, 1 thêm như `MERGE`. Thứ tự **UPDATE trước, INSERT sau** quan trọng: làm ngược lại thì món vừa chèn sẽ bị câu UPDATE xét thêm một lần. `UPDLOCK` lấy khoá cập nhật ngay khi đọc (hai phiên không cùng giữ được), `SERIALIZABLE` khoá cả khoảng khoá chưa có dòng, nên không phiên nào chèn được BB-04 chen vào giữa câu kiểm `NOT EXISTS` và câu `INSERT`. `SET XACT_ABORT ON` bảo đảm lỗi ở câu thứ hai huỷ luôn câu thứ nhất ([MS-006](/articles/mssql-insert-update-delete-output)).

Trạng thái sau lô 2:

```text
sku|name|price|is_available|source_updated_at|updated_by_batch
---|----|-----|------------|-----------------|----------------
BB-01|Bún bò đặc biệt|59000|1|2026-10-28 09:00:00.000 +07:00|2
BB-02|Bún bò tái|45000|1|2026-10-20 08:00:00.000 +07:00|1
BB-03|Bún bò chả|45000|1|2026-10-20 08:00:00.000 +07:00|1
BB-04|Bún bò giò|52000|1|2026-10-28 09:30:00.000 +07:00|2
NC-01|Nước cam|25000|1|2026-10-20 08:00:00.000 +07:00|1
TR-01|Trà đá|5000|0|2026-10-28 09:00:00.000 +07:00|2
```

BB-02 vẫn mang `updated_by_batch = 1`: lô 2 không sửa gì ở món này, và cột đó nói đúng như vậy. BB-03 giữ giá 45.000 đ, bản cũ 42.000 đ không đè lên được.

## 9. Idempotent: gửi lại lô không đổi gì

Đối tác gửi lại file là chuyện thường (mất mạng, retry). Câu nhập phải **idempotent**: chạy một lần hay nhiều lần cho cùng kết quả. Lab gửi lại lô 2 bằng cả hai cách:

```text
PASS gửi lại lô 2 bằng MERGE: @@ROWCOUNT = 0
PASS gửi lại lô 2 bằng MERGE: số dòng OUTPUT = 0
PASS gửi lại lô 2 bằng UPDATE + INSERT = 0 + 0
```

R2 đủ để lần gửi lại không đè dữ liệu (giờ nguồn giờ đã bằng nhau, `>` sai), R3 đủ để không có lần ghi nào khi dữ liệu y hệt. Bỏ cả hai thì lần gửi lại vẫn "thành công", nhưng ghi lại mọi dòng khớp và cập nhật `updated_by_batch` sai.

## 10. Lỗi 8672: một dòng đích khớp nhiều dòng nguồn

Bây giờ BB-04 đã có trong đích. Gửi lại lô 2 **không** khử trùng lặp, với nhánh `WHEN MATCHED THEN UPDATE` không điều kiện:

```text
Msg 8672: The MERGE statement attempted to UPDATE or DELETE the same row more than once. This happens when a target row matches more than one source row. A MERGE statement cannot UPDATE/DELETE the same row of the target table multiple times. Refine the ON clause to ensure a target row matches at most one source row, or use the GROUP BY clause to group the source rows.
PASS MERGE nguồn trùng khoá đã có: số lỗi = 8672
```

Cùng một lỗi dữ liệu (BB-04 hai lần trong lô) cho ra **hai triệu chứng khác nhau**: khi khoá chưa có là 2627 ở mục 3, khi khoá đã có là 8672. SQL Server không chọn hộ dòng nào thắng; quy tắc R4 phải do bạn viết. Bài tập 3 cho thấy một trường hợp lô trùng khoá mà vẫn không lỗi, và vì sao không nên dựa vào đó.

## 11. WHEN NOT MATCHED BY SOURCE: đồng bộ trọn thực đơn

Một số quán gửi **toàn bộ** thực đơn mỗi lần: món nào không có trong file thì coi như hết hàng. Nhánh `WHEN NOT MATCHED BY SOURCE` làm được việc đó, với một cái bẫy: nhánh này xét **mọi dòng của bảng đích**, không chỉ các dòng của quán đang nhập. Lab chạy trong `BEGIN TRANSACTION … ROLLBACK`, thêm tạm hai món của quán "Phở Lab 102", rồi đồng bộ quán 101 bằng file thiếu NC-01:

```sql
MERGE toinoi.menu_items AS m
USING #thuc_don_101 AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET is_available = 0
OUTPUT $action, inserted.merchant_id, inserted.sku INTO @kq;
```

```text
hanh_dong|quan|sku
---------|----|---
UPDATE|101|NC-01
UPDATE|102|PH-01
UPDATE|102|PH-02
PASS BY SOURCE không lọc quán: dòng bị tắt = 101/NC-01, 102/PH-01, 102/PH-02
```

Hai món phở của quán khác bị tắt theo. Nếu nhánh này là `DELETE`, chúng đã biến mất. Sửa bằng cách đặt điều kiện vào chính nhánh `WHEN`:

```sql
WHEN NOT MATCHED BY SOURCE AND m.merchant_id = 'e1000000-0000-4000-8000-000000000101' THEN
    UPDATE SET is_available = 0
```

```text
PASS BY SOURCE có điều kiện quán: dòng bị tắt = 101/NC-01
```

Đừng lọc đích bằng cách thêm `AND m.merchant_id = …` vào `ON`: tài liệu `MERGE` cảnh báo rằng lọc dòng đích trong `ON` có thể cho kết quả sai. Trong các cách tài liệu gợi ý có điều kiện ở nhánh `WHEN` (như trên) và `MERGE` vào một CTE đã lọc đích (bài tập 2).

## 12. Làm trong DBStudio

- Các mục có `MERGE` ngắn gọn nhất khi chạy bằng `Ctrl+Enter` trên từng batch `GO` của tệp 17: biến `@kq` chỉ sống trong batch khai báo nó. `Ctrl+Shift+Enter` chạy cả tệp; mỗi `SELECT` là một result set riêng, dòng `Msg …` và `PASS …` (in bằng `PRINT`) nằm ở tab **Thông điệp**.
- Thử `MERGE` có nhánh `BY SOURCE` an toàn: bấm "Bắt đầu" trên thanh công cụ transaction, chạy, đọc OUTPUT, rồi "Rollback".
- Trên kết nối gắn môi trường "Sản xuất", DBStudio bắt xác nhận câu phá huỷ như `UPDATE`/`DELETE` không có `WHERE`. `MERGE` có nhánh `BY SOURCE` sửa hoặc xoá dòng theo một cách khác hẳn `WHERE`, nên với `MERGE` hãy luôn chạy thử trong transaction và đọc OUTPUT trước khi commit.

## 13. So với PostgreSQL

Cùng dữ liệu, cùng quy tắc với [PG-017](/articles/postgresql-upsert-on-conflict):

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Upsert một câu | `INSERT … ON CONFLICT (khoá) DO UPDATE`, xử lý được phiên khác vừa chèn cùng khoá | Không có; `MERGE … WITH (HOLDLOCK)` hoặc UPDATE + INSERT trong transaction có `UPDLOCK, SERIALIZABLE` |
| Lô tự trùng khoá | `21000 … cannot affect row a second time` | 2627 nếu khoá chưa có, 8672 nếu khoá đã có |
| Vi phạm UNIQUE khác | `23505` (ON CONFLICT chỉ bắt khoá chỉ định) | 2627 (MERGE chỉ so khớp theo ON) |
| Biết dòng nào thêm, dòng nào sửa | `RETURNING merge_action()` (MERGE, từ 17) | `OUTPUT $action` |
| Điều kiện R3 | `IS DISTINCT FROM` theo bộ `(a, b, c)` | `IS DISTINCT FROM` từng cột (từ 2022), không có so sánh theo bộ |
| `NOT MATCHED BY SOURCE` | Có từ 17 | Có từ SQL Server 2008 |

## 14. Lỗi thường gặp và đánh đổi

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| 2627 trên khoá chính, cả lô bị từ chối | Lô có hai dòng cùng khoá chưa có trong đích | Khử trùng lặp nguồn bằng `ROW_NUMBER()` với thứ tự xác định |
| 8672 | Lô có hai dòng cùng khoá đã có trong đích | Như trên |
| 2627 ở một UNIQUE khác | Ràng buộc không nằm trong `ON` | Lọc dòng vi phạm ra, ghi vào biên bản |
| Giá mới bị giá cũ đè | `WHEN MATCHED` không có điều kiện thời điểm | `AND s.<giờ nguồn> > m.<giờ nguồn>` |
| Gửi lại lô vẫn "cập nhật" hàng loạt | Thiếu R3 | So sánh `IS DISTINCT FROM` trước khi ghi |
| Món của quán khác bị tắt/xoá | `NOT MATCHED BY SOURCE` không giới hạn đích | Điều kiện ở nhánh `WHEN` hoặc CTE lọc đích |
| Thỉnh thoảng 2627 khi tải cao | `MERGE` thiếu `HOLDLOCK` | `WITH (HOLDLOCK)` trên đích, hoặc mẫu UPDATE + INSERT có `UPDLOCK, SERIALIZABLE` |

Đánh đổi:

- **MERGE hay hai câu.** `MERGE` gọn, một lần quét, có `$action` và nhánh `BY SOURCE`. Hai câu dễ đọc, dễ đặt khoá và dễ kiểm thử riêng từng bước, nhưng phải giữ transaction và thứ tự đúng. Với upsert theo lô của Tới Nơi, lab giữ cả hai và kiểm chúng cho cùng kết quả.
- **Lọc R5 trước hay để lỗi.** Lọc trước giữ được lô nhưng phải chép luật ràng buộc vào truy vấn; đổi ràng buộc mà quên bộ lọc thì lô lại hỏng. Ràng buộc trong bảng vẫn là chốt chặn cuối.
- **Khoá rộng hay retry.** `HOLDLOCK`/`SERIALIZABLE` loại race condition nhưng làm phiên khác chờ lâu hơn; một số hệ chọn khoá hẹp và retry khi gặp 2627. Phần 4 so sánh hai cách bằng số liệu.

## 15. Bài tập

**Bài 1.** Lô 3 có hai dòng: BB-01 giá 99.000 đ (30/10 08:00) và món mới CF-01 Cà phê sữa 22.000 đ. Quán chỉ muốn "thêm món mới, món đã có thì để nguyên". Viết `MERGE` tương ứng.

<details><summary><b>Lời giải</b></summary>
<p>Chỉ cần nhánh <code>WHEN NOT MATCHED BY TARGET</code>. Dòng khớp không có nhánh nào nên bị bỏ qua, không lỗi:</p>
<pre><code class="language-sql">MERGE toinoi.menu_items WITH (HOLDLOCK) AS m
USING (SELECT * FROM toinoi.menu_item_imports WHERE batch_id = 3) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku, inserted.price INTO @kq;</code></pre>
<p>Kết quả trong lab: <code>PASS Bài 1: chỉ thêm món mới = INSERT CF-01 22000</code> và <code>PASS Bài 1: BB-01 giữ giá = 59000</code>. Lô 3 không tự trùng khoá; nếu có, vẫn phải khử trùng lặp, vì hai dòng cùng khoá mới sẽ cùng vào nhánh INSERT.</p>
</details>

**Bài 2.** Viết lại mục 11 mà không đặt điều kiện ở nhánh `WHEN`: giới hạn đích bằng một CTE chỉ chứa món của quán 101.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">WITH dich AS (
    SELECT * FROM toinoi.menu_items
    WHERE merchant_id = 'e1000000-0000-4000-8000-000000000101'
)
MERGE dich AS m
USING ( /* thực đơn trọn vẹn của quán 101 */ ) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET is_available = 0
OUTPUT $action, inserted.merchant_id, inserted.sku INTO @kq;</code></pre>
<p>Chạy sau bài 1 (quán 101 đã có CF-01), với file thiếu NC-01 và CF-01, trong <code>BEGIN TRANSACTION … ROLLBACK</code> có hai món phở của quán 102: <code>PASS Bài 2: MERGE vào CTE lọc quán = 101/CF-01, 101/NC-01</code>. Hai món phở không bị đụng tới vì chúng không nằm trong đích <code>dich</code>.</p>
</details>

**Bài 3.** Gửi lại lô 2 **không** khử trùng lặp, nhưng nhánh `WHEN MATCHED` giữ điều kiện R2 + R3 như mục 6. Có lỗi 8672 không? Vì sao không nên dựa vào kết quả đó?

<details><summary><b>Lời giải</b></summary>
<p>Không lỗi, <code>@@ROWCOUNT = 0</code> (lab: <code>PASS Bài 3: lỗi khi có điều kiện = 0</code>). Hai dòng BB-04 (09:05 và 09:30) đều không mới hơn dòng đích (09:30), nên điều kiện <code>AND</code> loại cả hai và không dòng đích nào thật sự bị cập nhật hai lần. Câu lệnh qua được chỉ vì dữ liệu lần này tình cờ cũ hơn. Lô sau có hai dòng BB-04 cùng mới hơn đích sẽ gặp lại 8672, và nếu SQL Server có chọn một dòng thì cũng không có gì bảo đảm đó là dòng mới nhất. R4 vẫn phải nằm trong nguồn.</p>
</details>

## 16. Đọc tiếp

- Trước đó: [MS-016 · Ngày giờ, AT TIME ZONE, DATETRUNC và DATE_BUCKET](/articles/mssql-ngay-gio-at-time-zone-date-bucket).
- `OUTPUT`, `inserted`/`deleted`, `XACT_ABORT`: [MS-006](/articles/mssql-insert-update-delete-output). Ràng buộc PK, UNIQUE, FK và lỗi 2627/547: [MS-005](/articles/mssql-create-table-constraints).
- Khoá, `SERIALIZABLE`, key-range lock và retry: MS-036 và MS-038 (Phần 4).
- Cùng bài toán trên PostgreSQL: [PG-017 · UPSERT và xung đột khoá duy nhất](/articles/postgresql-upsert-on-conflict).
- Microsoft Learn: [MERGE (Transact-SQL)](https://learn.microsoft.com/sql/t-sql/statements/merge-transact-sql), [OUTPUT clause](https://learn.microsoft.com/sql/t-sql/queries/output-clause-transact-sql), [Table hints](https://learn.microsoft.com/sql/t-sql/queries/hints-transact-sql-table), [IS [NOT] DISTINCT FROM](https://learn.microsoft.com/sql/t-sql/queries/is-distinct-from-transact-sql), [ROW_NUMBER](https://learn.microsoft.com/sql/t-sql/functions/row-number-transact-sql).

Bài tiếp theo: [MS-018 · Phân trang lịch sử chuyến: OFFSET/FETCH và keyset](/articles/mssql-phan-trang-keyset)

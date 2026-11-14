---
slug: postgresql-optimistic-version-conditional-update
title: "Optimistic concurrency và cập nhật có điều kiện: version, trạng thái và số dòng đã đổi"
summary: "Tổng đài viên Tới Nơi sửa cước chuyến 400108 trong một phút; khách huỷ chuyến giữa chừng, bấm Lưu xong thì chuyến «sống lại». Học cách ghi kèm điều kiện version/trạng thái và đọc số dòng đã đổi để bắt xung đột."
category: postgresql
level: intermediate
tags: [postgresql, pg-035, optimistic-concurrency, version, lost-update, read-committed, toi-noi]
language: vi
---

Một khách gọi tổng đài Tới Nơi xin đổi điểm đón của chuyến **400108**; tổng đài viên mở màn sửa chuyến, thấy cước 38.000 đ, trạng thái `requested`, rồi mất một phút trao đổi và gõ cước mới 45.000 đ. Trong phút đó, chính khách đã bấm **Huỷ** trên app. Tổng đài viên bấm **Lưu**: màn sửa gửi lại mọi trường của form, kể cả trạng thái `requested` đọc từ một phút trước. Chuyến đã huỷ «sống lại», hệ thống đi tìm tài xế cho một khách không còn cần xe. Hệ thống không báo lỗi nào.

Ở PG-034 ta đã thấy lost update (mất cập nhật) giữa hai giao dịch ngắn. Lần này khoảng hở giữa «đọc» và «ghi» là thời gian con người gõ phím — không thể mở giao dịch và giữ khoá dòng suốt một phút. Bài này dùng cách **lạc quan**: không khoá khi đọc, nhưng lúc ghi chỉ ghi nếu dòng *chưa đổi* kể từ khi đọc, và đọc **số dòng đã đổi** để biết mình thắng hay thua.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-035 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-isolation-read-committed-repeatable-read">PG-034 · Read Committed và Repeatable Read</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>35-optimistic-version.sql</code>, thí nghiệm <code>thu-nghiem/pg035-version-rr-ham-luu-updated-at-xmin.sql</code>). Sau bài này, bạn ngăn được ghi đè trạng thái bằng cột version và kiểm tra số dòng đã đổi, kể cả khi hai lần lưu chạy đồng thời thật.</p>
</div>

## 1. Lab dùng trong bài

Phần 4 chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 dùng PGlite 17.5). Runner tự dựng một cluster tạm trong `%TEMP%`, chép database nền cho từng tệp rồi dọn sạch:

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 35
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg035-version-rr-ham-luu-updated-at-xmin.sql
```

Nền của Phần 4 thêm cột `trips.version integer NOT NULL DEFAULT 1` (kèm `CHECK (version > 0)`) và 10 chuyến đang chờ nhận 400101–400110, đều `requested`, version 1; chuyến 400108 có cước 38.000 đ. Hai phiên psql thật `[A]` (tổng đài viên) và `[B]` (khách) chạy theo lịch tất định. Lệnh `SELECT … \gset` của psql lưu kết quả vào biến của *phiên* (`:v`, `:'trang_thai'`) — giả lập giá trị form mà ứng dụng giữ; `:ROW_COUNT` là số dòng mà câu trước đã đổi. Kết quả dưới đây chép từ `p04/OUTPUT.txt`, đã bỏ các dòng `-- [lab]`.

## 2. Lỗi gốc: form ghi lại mọi trường

Lịch §1 của lab, không kiểm tra gì cả:

| Bước | [A] tổng đài viên | [B] khách |
|---|---|---|
| T1 | Đọc `fare, status, version` → `38000 \| requested \| 1` | |
| T2 | | `UPDATE … SET status = 'cancelled' …` → `UPDATE 1` |
| T3 | `UPDATE trips SET fare = 45000, status = :'trang_thai' … WHERE code = 400108` → `UPDATE 1` | |
| T4 | | Đọc lại |

```text
 fare  |  status
-------+-----------
 45000 | requested
(1 row)
```

Cả hai câu UPDATE đều thành công, đều chạy dưới Read Committed mặc định, không ai chờ ai — chúng cách nhau cả phút. Database làm đúng điều được yêu cầu: ghi `status = 'requested'`. Lỗi nằm ở chỗ câu ghi của A **dựa trên một lần đọc đã cũ** mà không nói ra điều đó. Mức cô lập không cứu được: hai giao dịch không chồng nhau về thời gian, nên không mức nào coi đây là xung đột.

Hai cách sửa vội thường gặp đều không đủ. «Chỉ gửi trường đã đổi» (không gửi `status`) giữ được lệnh huỷ, nhưng cước 45.000 đ vẫn ghi vào một chuyến đã huỷ, và hai người cùng sửa cước thì người sau vẫn đè người trước. «Đọc lại ngay trước khi ghi rồi so trong ứng dụng» thu hẹp khoảng hở chứ không đóng nó: giữa câu đọc lại và câu ghi vẫn có thể có người chen vào (PG-040 đếm được điều này bằng phép thử đồng thời).

## 3. Optimistic concurrency: điều kiện nằm trong chính câu ghi

**Optimistic concurrency control** (kiểm soát đồng thời lạc quan) giả định xung đột hiếm: không khoá gì khi đọc, chỉ *kiểm tra* lúc ghi. Ngược lại, **pessimistic** (bi quan) khoá dòng ngay khi đọc (`SELECT … FOR UPDATE`, PG-036) và bắt người khác chờ.

Công cụ là một cột **version** (phiên bản dòng): mỗi lần sửa thì `version = version + 1`, và câu ghi kèm điều kiện `version = <giá trị đã đọc>`. Kiểm tra và ghi nằm trong **một** câu UPDATE, nên không có khoảng hở giữa chúng — đây là dạng compare-and-set (so rồi gán) ở mức dòng. Lịch §2:

```sql
-- [B] khách huỷ, cũng theo quy ước tăng version
UPDATE trips SET status = 'cancelled', updated_at = now(), version = version + 1
WHERE code = 400108 AND version = 1
RETURNING status, version;
```

```text
  status   | version
-----------+---------
 cancelled |       2
(1 row)
```

```sql
-- [A] lưu form với version 1 đã đọc
UPDATE trips SET fare = 45000, updated_at = now(), version = version + 1
WHERE code = 400108 AND version = :v;
SELECT :ROW_COUNT AS so_dong_doi;
```

```text
UPDATE 0
 so_dong_doi
-------------
           0
(1 row)
```

**0 dòng** là tín hiệu xung đột. Không có lỗi SQL nào — ứng dụng phải tự đọc số dòng đã đổi: psql có `:ROW_COUNT`, PL/pgSQL có `FOUND` và `GET DIAGNOSTICS … = ROW_COUNT`, thư viện client trả số này kèm thẻ lệnh `UPDATE 0` (JDBC `executeUpdate()` trả về số dòng). Khi gặp 0, ứng dụng báo «Chuyến đã được người khác sửa — tải lại» và đọc bản mới:

```text
 fare  |  status   | version
-------+-----------+---------
 38000 | cancelled |       2
(1 row)
```

Lệnh huỷ của khách còn nguyên. Tổng đài viên thấy chuyến đã huỷ và tự quyết định — đó là mấu chốt của cách lạc quan: database không chọn người thắng theo ý nghĩa nghiệp vụ, mà chỉ từ chối ghi đè mù.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Version chỉ có tác dụng khi MỌI đường ghi tuân thủ</span>
<p>Câu huỷ của khách cũng phải tăng version. Một đường ghi «quên» tăng version (job nền, script sửa dữ liệu) làm dòng đổi mà version giữ nguyên ⇒ người cầm version cũ vẫn ghi đè được. Mục 6 dùng trigger để chặn kiểu quên này.</p>
</div>

## 4. Hai lần lưu chồng nhau thật sự

Ở mục 3 hai câu ghi cách nhau xa. Nếu hai tổng đài viên cùng đọc version 2 rồi bấm Lưu *gần như cùng lúc* thì sao? Lịch §3 cho A mở giao dịch, ghi xong nhưng chưa commit, rồi B ghi:

| Bước | [A] | [B] |
|---|---|---|
| T1 | `BEGIN` | |
| T2 | `UPDATE … SET fare = 41000, version = version + 1 WHERE code = 400108 AND version = 2 RETURNING version` → **3** | |
| T3 | | `UPDATE … SET fare = 47000, version = version + 1 WHERE … AND version = 2` ⏳ *Lock: transactionid — bị chặn bởi [A]* |
| T4 | `COMMIT` | |
| T5 | | ✔ `UPDATE 0` |
| T6 | | Đọc: `41000 \| 3` |

Câu của B bắt đầu khi bản đã commit vẫn là version 2 — điều kiện khớp — nhưng dòng đang bị A khoá nên B chờ. Sau khi A commit, Read Committed không bỏ cuộc mà **đọc lại đúng dòng đó** ở phiên bản mới và đánh giá lại `WHERE`. Tài liệu PostgreSQL mô tả: «The search condition of the command (the WHERE clause) is re-evaluated» trên phiên bản đã cập nhật. Version giờ là 3 ⇒ điều kiện sai ⇒ B đổi 0 dòng. Kết quả cuối `41000 | 3`: không ai bị ghi đè mà không hay biết.

Cơ chế «đọc lại dòng» này (thường gọi là EvalPlanQual) là lý do UPDATE có điều kiện vẫn đúng khi chạy đồng thời thật ở mức mặc định. Giới hạn của cơ chế này (PG-034 có thí nghiệm riêng): câu UPDATE chỉ đọc lại những dòng nó đã gặp, không quét lại bảng — dòng *vừa trở nên* khớp điều kiện bị bỏ sót. Với UPDATE theo khoá một dòng như ở đây, điều đó không gây hại.

### Dưới Repeatable Read: 40001 thay vì 0 dòng

Thí nghiệm §1 chạy lại lịch trên, nhưng B mở `BEGIN ISOLATION LEVEL REPEATABLE READ` (không có trong lab chính, nằm ở `p04/thu-nghiem/pg035-version-rr-ham-luu-updated-at-xmin.sql`):

```text
-- [B] UPDATE … WHERE code = 400108 AND version = 1;   ⏳ bị chặn bởi [A]
-- [A] COMMIT;
-- [B] ✔
ERROR:  could not serialize access due to concurrent update
```

SQLSTATE **`40001`**. Repeatable Read không được phép sửa dòng mà giao dịch khác đã đổi sau khi snapshot được chụp, nên thay vì đọc lại, PostgreSQL huỷ cả giao dịch của B. B `ROLLBACK`, đọc lại thấy `41000 | 2`, và xử lý như xung đột.

Vậy ứng dụng phải coi **cả hai** là «có người sửa trước»: `UPDATE 0` (Read Committed) và lỗi `40001` (Repeatable Read/Serializable). Điểm khác: sau `40001` giao dịch đã hỏng, mọi việc khác trong giao dịch đó cũng mất — phải chạy lại từ đầu (PG-038 bàn kỹ về thử lại).

## 5. Điều kiện theo trạng thái: máy trạng thái trong WHERE

Không phải lúc nào cũng cần version. Khi câu hỏi là «chuyển trạng thái này có hợp lệ không», điều kiện chính là **trạng thái hiện tại**. Lịch §4 — tài xế 9101 [A] và 9102 [B] cùng bấm «Nhận» chuyến 400109:

```sql
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400109 AND status = 'requested'
RETURNING code, status;
```

A nhận `400109 | accepted`; câu y hệt của B (với driver 9102) trả `(0 rows)`, `UPDATE 0`, `:ROW_COUNT` = 0 ⇒ app báo B «chuyến đã có người nhận». Khi hai câu chạy đồng thời thật, cơ chế của mục 4 vẫn áp dụng: B chờ A, đọc lại thấy `accepted`, đổi 0 dòng.

Tập trạng thái hợp lệ đặt thẳng vào `WHERE`:

```sql
-- khách chỉ được huỷ khi chưa đón
UPDATE trips SET status = 'cancelled', updated_at = now(), version = version + 1
WHERE code = 400109 AND status IN ('requested', 'searching', 'accepted', 'arriving')
RETURNING status;                                    -- cancelled

-- tài xế bấm "Đã đón" sau khi khách huỷ
UPDATE trips SET status = 'picked_up', updated_at = now(), version = version + 1
WHERE code = 400109 AND status IN ('accepted', 'arriving');   -- UPDATE 0
```

Chuyến không thể đi từ `cancelled` sang `picked_up`, dù app tài xế gửi lệnh trễ. Điều kiện trạng thái hợp với *chuyển trạng thái* (thao tác có nghĩa nghiệp vụ rõ: nhận, huỷ, đón); version hợp với *sửa form* (nhiều trường, ai đổi trường nào cũng phải biết). Hai cách kết hợp được: «Lưu» của tổng đài viên có thể kèm cả `version = :v` lẫn `status IN (…)`.

## 6. 0 dòng có hai nghĩa — và ai tăng version?

`UPDATE 0` xảy ra cả khi version sai lẫn khi chuyến không tồn tại (mã gõ nhầm, đã bị xoá). Muốn báo đúng, ứng dụng phải đọc lại sau khi thua. Hàm dưới (thí nghiệm §2) gói cả hai bước:

```sql
CREATE FUNCTION luu_cuoc(p_code bigint, p_fare integer, p_version integer) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
  v_moi integer;
  v_hien_tai integer;
BEGIN
  UPDATE trips SET fare = p_fare, updated_at = now(), version = version + 1
  WHERE code = p_code AND version = p_version
  RETURNING version INTO v_moi;
  IF FOUND THEN
    RETURN 'da_luu version=' || v_moi;
  END IF;
  SELECT version INTO v_hien_tai FROM trips WHERE code = p_code;
  IF NOT FOUND THEN
    RETURN 'khong_ton_tai';
  END IF;
  RETURN 'xung_dot version_hien_tai=' || v_hien_tai;
END
$$;
```

A gọi `luu_cuoc(400105, 40000, 1)` → `da_luu version=2`; B vẫn cầm version 1 gọi `luu_cuoc(400105, 42000, 1)` → `xung_dot version_hien_tai=2`; `luu_cuoc(499999, …)` → `khong_ton_tai`. Việc trả version mới cho client rất quan trọng: lần lưu kế tiếp của chính A phải dùng version 2, không phải 1.

Bước đọc lại chỉ để *báo*, không để quyết định ghi — quyết định đã nằm trong câu UPDATE. Việc tự động đọc lại rồi «thử ghi lần nữa với version mới» chính là đè lên người khác bằng đường vòng.

### Trigger tự tăng version

§5 của lab gắn trigger `BEFORE UPDATE` để version tăng với **mọi** câu UPDATE:

```sql
CREATE FUNCTION tang_version() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  NEW.version := OLD.version + 1;
  RETURN NEW;
END
$$;

CREATE TRIGGER trg_trips_tang_version BEFORE UPDATE ON trips
FOR EACH ROW EXECUTE FUNCTION tang_version();
```

Chuyến 400110 đang ở version **1**. `UPDATE trips SET fare = fare + 1000 WHERE code = 400110 RETURNING version` (quên version) → **2**. Câu đúng quy ước `… version = version + 1 WHERE code = 400110 AND version = 2 RETURNING version` → **3**, không phải 4: trigger gán `NEW.version := OLD.version + 1` đè lên giá trị câu lệnh tự tính, nên không tăng đôi.

Trigger chữa được «quên tăng version», không chữa được «quên kiểm version» — câu `WHERE` vẫn là việc của mỗi đường ghi. Ngoài ra, trigger còn tăng version cả với những câu sửa không liên quan tới form (job nền cập nhật cột kỹ thuật), sinh ra xung đột «giả» làm tổng đài viên phải tải lại dù không ai đổi trường họ đang sửa. Chọn trigger là chọn sự an toàn hơn, đổi lại phải chấp nhận vài lần tải lại thừa.

## 7. Chọn gì làm «version»?

Thí nghiệm §3–§4 thử hai ứng viên khác ngoài cột `version`:

**`updated_at = now()`.** `now()` là giờ *bắt đầu giao dịch*, không phải giờ câu lệnh chạy. A mở giao dịch, B sửa chuyến 400107 và commit (now() của B muộn hơn), rồi A mới ghi `updated_at = now()` và commit:

```text
 fare  | di_lui
-------+--------
 38500 | t
(1 row)
```

`di_lui = t`: lần sửa sau ghi mốc *nhỏ hơn* lần sửa trước. Dùng mốc thời gian làm version thì có thể so «bằng» được, nhưng giá trị không tăng đơn điệu, phụ thuộc đồng hồ, và hai lần sửa trong cùng giao dịch mang cùng mốc. Giữ `updated_at` để hiển thị, đừng dùng làm điều kiện.

**Cột hệ thống `xmin`.** Mỗi dòng có sẵn `xmin` — theo tài liệu, là định danh của giao dịch đã chèn *phiên bản dòng* này; UPDATE tạo phiên bản mới nên `xmin` đổi. Thí nghiệm: khoá dòng bằng `FOR NO KEY UPDATE` không đổi `xmin` (`t`); câu `UPDATE trips SET fare = fare` — không đổi dữ liệu — vẫn đổi `xmin`, và câu lưu `… AND xmin = :'xm'::xid` của A ra **`UPDATE 0`** dù không có trường nghiệp vụ nào đổi.

| Ứng viên | Ưu | Nhược |
|---|---|---|
| Cột `version integer` | Rõ nghĩa, đơn điệu, nằm trong schema, client đọc/gửi dễ | Mọi đường ghi phải tăng (hoặc dùng trigger) |
| `updated_at` | Có sẵn ở nhiều bảng | Đi lùi được (thí nghiệm), không duy nhất |
| `xmin` | Không cần cột, tự đổi với mọi UPDATE | Kiểu `xid` 32 bit; tài liệu khuyên không dựa vào tính duy nhất của xid về lâu dài; đổi cả khi dữ liệu không đổi; gắn chặt vào nội bộ PostgreSQL |

Tới Nơi chọn cột `version` cho bảng `trips` — giá trị nằm trong hợp đồng API («gửi kèm version đã đọc»), không phụ thuộc chi tiết lưu trữ.

## 8. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Chuyến đã huỷ «sống lại» sau khi lưu form | Form gửi lại mọi trường, không điều kiện | `WHERE version = :v`, chỉ ghi trường đã sửa |
| Có cột version mà vẫn mất cập nhật | Một đường ghi không tăng version | Trigger `BEFORE UPDATE` hoặc rà mọi câu ghi |
| Bấm Lưu «thành công» nhưng không có gì đổi | Ứng dụng bỏ qua `UPDATE 0` | Đọc số dòng đã đổi; 0 ⇒ báo xung đột |
| Kiểm version bằng `SELECT` rồi mới `UPDATE` không điều kiện | Kiểm và ghi tách hai câu | Gộp điều kiện vào `WHERE` của câu ghi |
| Ở Repeatable Read, lưu bị lỗi thay vì 0 dòng | `40001` khi dòng bị sửa đồng thời | Xử lý như xung đột, rollback, tải lại |
| Tự động «thử lại với version mới» | Đè người khác bằng đường vòng | Báo xung đột để người dùng quyết định |

## 9. Đánh đổi

- **Lạc quan hay bi quan.** Lạc quan không giữ khoá trong lúc người gõ, hợp với form và API stateless; cái giá là người thua phải làm lại. Bi quan (`FOR UPDATE`, PG-036) hợp khi xung đột dày và làm lại đắt — nhưng chỉ trong giao dịch ngắn, không bao giờ kéo dài qua thời gian người dùng suy nghĩ.
- **Tỉ lệ xung đột.** Lạc quan rẻ khi xung đột hiếm (sửa form một chuyến). Bảng `trips` nhận khoảng 560 UPDATE/giây lúc cao điểm (hồ sơ Tới Nơi), nhưng đó là ≈ 8 lần sửa mỗi chuyến rải trên mọi chuyến đang chạy; xung đột chỉ dày khi nhiều người ghi cùng **một** dòng. Với dòng nóng như vậy, người dùng sẽ gặp «tải lại» liên tục — khi đó tách trường hay đổi thiết kế (ghi tương đối `balance = balance + x`, PG-034) tốt hơn.
- **Version hay điều kiện trạng thái.** Trạng thái diễn đạt đúng luật nghiệp vụ và không cần cột mới; version bắt được mọi thay đổi, kể cả trường bạn không nghĩ tới.

## 10. Bài tập

**Bài 1.** Lịch §3 (mục 4), nhưng B chạy `REPEATABLE READ`. Dự đoán B nhận gì, giao dịch của B còn dùng tiếp được không, và ứng dụng nên làm gì.

<details><summary><b>Lời giải</b></summary>
<p>B vẫn chờ A (⏳ bị chặn bởi [A]); sau khi A commit, B nhận <code>ERROR: could not serialize access due to concurrent update</code>, SQLSTATE <code>40001</code> — không phải <code>UPDATE 0</code>. Giao dịch của B đã hỏng: phải <code>ROLLBACK</code>, đọc lại (thí nghiệm ra <code>41000 | 2</code>) rồi báo xung đột như trường hợp 0 dòng. Đã chạy ở <code>p04/thu-nghiem/pg035-version-rr-ham-luu-updated-at-xmin.sql</code> §1.</p>
</details>

**Bài 2.** Viết hàm PL/pgSQL lưu cước kèm version, trả về một trong ba kết quả: đã lưu (kèm version mới), xung đột (kèm version hiện tại), chuyến không tồn tại.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">UPDATE trips SET fare = p_fare, updated_at = now(), version = version + 1
WHERE code = p_code AND version = p_version
RETURNING version INTO v_moi;
IF FOUND THEN RETURN 'da_luu version=' || v_moi; END IF;
SELECT version INTO v_hien_tai FROM trips WHERE code = p_code;
IF NOT FOUND THEN RETURN 'khong_ton_tai'; END IF;
RETURN 'xung_dot version_hien_tai=' || v_hien_tai;</code></pre>
<p>Thân hàm đầy đủ ở mục 6. Kết quả chạy thật: <code>da_luu version=2</code>, <code>xung_dot version_hien_tai=2</code>, <code>khong_ton_tai</code>. Quyết định ghi nằm trong câu UPDATE; câu SELECT sau đó chỉ để phân loại kết quả.</p>
</details>

**Bài 3.** Một thành viên đề nghị bỏ cột `version`, dùng `WHERE updated_at = :ts_da_doc` vì «bảng nào cũng có updated_at». Phản biện bằng một lịch hai phiên.

<details><summary><b>Lời giải</b></summary>
<p>A <code>BEGIN</code> (now() của A chốt tại đây); B sửa chuyến và commit với <code>updated_at = now()</code> muộn hơn; A ghi <code>updated_at = now()</code> rồi commit. Kết quả thí nghiệm §3: <code>38500 | t</code> — mốc mới <em>nhỏ hơn</em> mốc của B. Mốc thời gian không tăng đơn điệu, không duy nhất (hai lần sửa cùng giao dịch cùng mốc) và phụ thuộc đồng hồ máy chủ. So bằng vẫn có thể bắt được nhiều xung đột, nhưng cột <code>version</code> rõ nghĩa và không có các trường hợp biên này.</p>
</details>

## 11. Đọc tiếp

- [PostgreSQL 18 · Transaction Isolation](https://www.postgresql.org/docs/18/transaction-iso.html) — UPDATE ở Read Committed đánh giá lại `WHERE`; Repeatable Read báo lỗi tuần tự hoá.
- [PostgreSQL 18 · Basic Statements (PL/pgSQL) — Obtaining the Result Status](https://www.postgresql.org/docs/18/plpgsql-statements.html) — `FOUND`, `GET DIAGNOSTICS … ROW_COUNT`.
- [PostgreSQL 18 · System Columns](https://www.postgresql.org/docs/18/ddl-system-columns.html) — `xmin`, `xmax`, giới hạn của xid 32 bit.
- [PostgreSQL 18 · psql](https://www.postgresql.org/docs/18/app-psql.html) — `\gset`, biến `ROW_COUNT`.
- Lab: `examples/toi-noi-postgresql/p04/35-optimistic-version.sql`, kết quả ở `p04/OUTPUT.txt`; thí nghiệm `p04/thu-nghiem/pg035-version-rr-ham-luu-updated-at-xmin.sql`.

Bài tiếp theo: [PG-036 · FOR UPDATE, NOWAIT và SKIP LOCKED](/articles/postgresql-for-update-skip-locked).

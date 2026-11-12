---
slug: postgresql-rls-tenant-security
title: "Row-level security và cách ly tenant: kiểm cả đọc, ghi, ngữ cảnh và role đặc quyền"
summary: "Hai doanh nghiệp dùng chung bảng chuyến của Tới Nơi; một câu SELECT quên điều kiện tenant là lộ dữ liệu. Dựng policy USING/WITH CHECK trên PostgreSQL 18.1, kiểm ma trận đọc–ghi, vòng đời SET LOCAL, default-deny, owner, FORCE, superuser và chứng minh policy còn nguyên sau PITR."
category: postgresql
level: advanced
tags: [postgresql, pg-049, rls, row-level-security, tenant, policy, bao-mat, toi-noi]
language: vi
---

Tới Nơi bán gói doanh nghiệp: công ty đăng ký tài khoản, nhân viên đặt chuyến, kế toán công ty xem báo cáo chuyến hằng tháng trên cổng doanh nghiệp. Hai công ty — gọi là tenant 1 và tenant 2 (*tenant*: một khách hàng tổ chức dùng chung hệ thống nhưng phải thấy dữ liệu như thể hệ thống chỉ của riêng mình) — cùng nằm trong một bảng chuyến, phân biệt bằng cột `tenant_id`. Mọi câu truy vấn của cổng doanh nghiệp đều có `WHERE tenant_id = :tenant`. Mọi câu, trừ một: endpoint xuất CSV mới viết vội trong đợt chạy chiến dịch, quên điều kiện. Kế toán của tenant 1 tải về file có cả chuyến của tenant 2.

Chỉ cần một đường code lọt qua review là đủ. **Row-level security** (RLS — bảo mật mức dòng) đưa điều kiện tenant xuống database: PostgreSQL tự gắn điều kiện vào mọi câu đọc và ghi trên bảng, kể cả câu quên `WHERE`. Bài này dựng RLS cho bảng chuyến trong lab Phần 5, kiểm từng ô của ma trận «role × ngữ cảnh × thao tác», và chỉ ra chỗ RLS **không** bảo vệ bạn.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-049 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-role-grant-least-privilege">PG-041 · Role, GRANT và quyền tối thiểu</a>, <a href="/articles/postgresql-connection-pool-timeout">PG-047 · Connection pool và timeout</a>, <a href="/articles/postgresql-pitr-base-backup-wal">PG-048 · PITR với base backup và WAL</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, chạy <code>npm run p05</code> trên PostgreSQL 18.1 thật; mọi kết quả trong bài chép từ <code>p05/OUTPUT.txt</code>. Sau bài này, bạn kiểm được ma trận quyền theo tenant cho cả đọc lẫn ghi, quản lý vòng đời ngữ cảnh tenant khi dùng pool, và biết role nào đi vòng qua policy.</p>
</div>

## 1. Hai lớp: quyền trên bảng và quyền trên dòng

PG-041 đã tách role theo trách nhiệm: `tn_owner` sở hữu schema `ops` và bảng `ops.trips`; `tn_app` là role runtime của API, được `SELECT, INSERT, UPDATE`; `tn_report` chỉ được `SELECT`; `tn_monitor` nhận `pg_monitor`. ACL trong lab:

```text
   trips       | {tn_owner=arwdDxtm/tn_owner,tn_app=arw/tn_owner,tn_report=r/tn_owner}
```

`GRANT` trả lời câu hỏi «role này có được làm thao tác X trên **bảng** không». Nó không có cách nào nói «được đọc, nhưng chỉ những dòng có `tenant_id = 1`». RLS là lớp thứ hai, chạy **sau** khi GRANT đã cho phép: với mỗi câu lệnh, PostgreSQL lấy các **policy** (chính sách — một biểu thức boolean gắn với bảng, áp cho một số role và một số lệnh) phù hợp, rồi ghép biểu thức đó vào câu lệnh như một điều kiện bắt buộc.

Thiếu GRANT thì bạn nhận `42501 permission denied` trước khi RLS kịp xét. GRANT đủ mà policy loại dòng thì câu đọc **không lỗi gì cả** — chỉ trả ít dòng hơn. Phần lớn bài này xoay quanh hệ quả của chữ «không lỗi gì cả» đó.

Quy tắc của PostgreSQL 18 ([Row Security Policies](https://www.postgresql.org/docs/18/ddl-rowsecurity.html)) mà bài sẽ kiểm:

| Quy tắc | Hệ quả |
|---|---|
| Bật RLS mà không có policy nào áp cho role → **default-deny** | Không thấy, không sửa được dòng nào |
| Nhiều policy *permissive* (mặc định) ghép bằng `OR`; *restrictive* ghép bằng `AND` | Thêm policy permissive là **mở rộng** quyền |
| Superuser và role có `BYPASSRLS` luôn đi vòng qua RLS | Kiểm policy bằng `postgres` là kiểm sai |
| Owner của bảng đi vòng qua RLS, trừ khi bảng có `FORCE ROW LEVEL SECURITY` | Job chạy bằng owner không bị lọc |
| Thao tác cấp bảng (`TRUNCATE`, `REFERENCES`) và kiểm ràng buộc toàn vẹn không chịu RLS | Không cấp `TRUNCATE` cho runtime; có «kênh rò», mục 8 |

## 2. Policy của lab: USING và WITH CHECK

Fixture: `ops.trips` có 100 chuyến, `id` 1–50 thuộc tenant 1, 51–100 thuộc tenant 2, mỗi chuyến 30.000 đ, tổng 3.000.000 đ. Runner bật RLS và tạo một policy (`p05/run.mjs`):

```sql
ALTER TABLE ops.trips ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_scope ON ops.trips TO tn_app
USING (
  tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::int
)
WITH CHECK (
  tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::int
);
```

Catalog `pg_policies` ghi lại đúng như vậy:

```text
   policyname  |  roles   | cmd |                                          qual                                           |                                       with_check
  -------------+----------+-----+-----------------------------------------------------------------------------------------+-----------------------------------------------------------------------------------------
   tenant_scope | {tn_app} | ALL | (tenant_id = (NULLIF(current_setting('app.tenant_id'::text, true), ''::text))::integer) | (tenant_id = (NULLIF(current_setting('app.tenant_id'::text, true), ''::text))::integer)
  (1 row)
```

Đọc từng phần:

- **`TO tn_app`** — policy chỉ áp cho `tn_app` (và thành viên của nó). Role khác **không có policy nào** — giữ ý này cho mục 6.
- **`cmd = ALL`** — không ghi `FOR …` thì policy áp cho mọi lệnh.
- **`USING` (cột `qual`)** — điều kiện cho **dòng đang có**: dòng nào được thấy khi `SELECT`, được nhắm tới khi `UPDATE`/`DELETE`. Dòng không thoả bị loại **âm thầm**.
- **`WITH CHECK`** — điều kiện cho **dòng mới** của `INSERT` hoặc phiên bản mới sau `UPDATE`. Không thoả thì **câu lệnh lỗi**. Theo [CREATE POLICY](https://www.postgresql.org/docs/18/sql-createpolicy.html), bỏ trống `WITH CHECK` thì `USING` được dùng cho cả chiều ghi; lab ghi cả hai cho rõ.
- **`current_setting('app.tenant_id', true)`** — đọc tham số tuỳ biến (*custom setting*, tên có dấu chấm) mà ứng dụng đặt cho phiên. `missing_ok = true`: tham số chưa tồn tại thì trả `NULL` thay vì báo lỗi ([System Administration Functions](https://www.postgresql.org/docs/18/functions-admin.html)).
- **`NULLIF(…, '')`** — tham số tuỳ biến đã từng được đặt trong phiên (kể cả bằng `SET LOCAL` rồi transaction kết thúc) có thể còn tồn tại với giá trị rỗng; `''::int` sẽ làm câu lệnh lỗi thay vì trả 0 dòng. `NULLIF` đưa «rỗng» và «chưa đặt» về cùng một nghĩa.
- **`::int`** — giá trị như `'abc'` vẫn gây lỗi. Lỗi đáng có: ứng dụng truyền rác vào ngữ cảnh tenant thì nên vỡ to.

`tenant_id = NULL` cho `NULL`, không phải `true`, nên thiếu ngữ cảnh thì mọi dòng bị loại: thiết kế «đóng khi thiếu» (*fail closed*) — và là bẫy gỡ lỗi lớn nhất của RLS.

## 3. Chiều đọc: thiếu ngữ cảnh và đúng ngữ cảnh

Mỗi phép kiểm trong runner là một tiến trình `psql` mới, tức một **phiên mới**, nên ngữ cảnh của phép trước không rơi sang phép sau:

```sql
SET ROLE tn_app; SELECT count(*) FROM ops.trips;                              -- chưa đặt tenant
SET ROLE tn_app; SET app.tenant_id = '1'; SELECT count(*) FROM ops.trips;     -- tenant 1
```

```text
PASS 049 no tenant denies rows: 0
PASS 049 tenant 1: 50
```

Endpoint CSV quên `WHERE` giờ chỉ trả 50 chuyến của tenant 1 — đúng điều ta cần. Còn dòng đầu: **0 dòng, không lỗi**.

- **An toàn:** quên đặt ngữ cảnh không làm lộ dữ liệu. Policy kiểu «không có tenant thì cho xem hết» sẽ biến một middleware lỗi thành rò toàn bộ.
- **Bẫy:** «0 dòng» trông giống hệt «công ty này chưa có chuyến nào». Không SQLSTATE, không log server. Kỹ sư trực sẽ tìm lỗi ở dữ liệu, bộ lọc ngày, cache — trước khi nghĩ tới middleware không đặt `app.tenant_id`.

Cách giảm bẫy: ứng dụng tự kiểm «đã có tenant chưa» và trả lỗi rõ; log ghi tenant của mỗi transaction; theo dõi tỉ lệ request trả 0 dòng theo endpoint. Database cố tình im lặng; phần ồn ào phải nằm ở ứng dụng.

## 4. Chiều ghi: INSERT, UPDATE chuyển tenant, UPDATE dòng không thấy

Ba phép thử, đều với `tn_app` và `app.tenant_id = '1'`:

```sql
INSERT INTO ops.trips VALUES (101, 2, 30000, 'completed');   -- ghi chuyến cho tenant 2
UPDATE ops.trips SET tenant_id = 2 WHERE id = 1;            -- chuyển chuyến của mình sang tenant 2
WITH u AS (UPDATE ops.trips SET fare = fare WHERE id = 60 RETURNING 1)
SELECT count(*) FROM u;                                      -- sửa chuyến 60 của tenant 2
```

```text
--- 049 INSERT into other tenant
  ERROR:  42501: new row violates row-level security policy for table "trips"
--- 049 UPDATE moving row to other tenant
  ERROR:  42501: new row violates row-level security policy for table "trips"
PASS 049 UPDATE of other tenant row touches: 0
```

Ba kết quả, hai cơ chế:

1. **INSERT dòng tenant 2** — dòng mới không thoả `WITH CHECK` → `42501`, không dòng nào được ghi.
2. **UPDATE chuyến 1 sang tenant 2** — chuyến 1 thoả `USING` nên được nhắm tới, nhưng phiên bản mới không thoả `WITH CHECK` → lỗi. Đây là đường một tenant «đẩy» dữ liệu sang tenant khác, hoặc một bug làm chuyến biến mất khỏi công ty mình.
3. **UPDATE chuyến 60** — không thoả `USING` nên **không được nhìn thấy**; câu lệnh thành công và chạm 0 dòng.

Điểm thứ ba lặp lại bài học của [PG-035](/articles/postgresql-optimistic-version-conditional-update): **đọc số dòng bị ảnh hưởng**; nhận `UPDATE 0` mà báo «đã cập nhật» là lỗi ứng dụng. `42501` cũng là mã của «permission denied» do GRANT — phân biệt bằng nội dung thông báo, đừng chỉ bắt theo SQLSTATE.

Lab không chạy `DELETE` (`tn_app` không có quyền này, ACL `arw`) và `INSERT … ON CONFLICT`; tài liệu CREATE POLICY mô tả cách policy áp cho từng trường hợp, hãy tự kiểm nếu ứng dụng dùng.

## 5. Ngữ cảnh sống bao lâu: SET, SET LOCAL và connection pool

`SET app.tenant_id = '1'` đặt giá trị cho **cả phiên**, tức cả đời một kết nối. PG-047 đã cho thấy ứng dụng thật dùng pool: vài kết nối được trao lần lượt cho các request. Request của tenant 1 đặt ngữ cảnh bằng `SET` rồi trả kết nối; request kế tiếp của tenant 2 mượn đúng kết nối đó, middleware lỗi không đặt ngữ cảnh — và nó đọc dữ liệu tenant 1. RLS vẫn hoạt động hoàn hảo, chỉ là với sai ngữ cảnh.

`SET LOCAL` giới hạn giá trị trong **transaction hiện tại**. Lab kiểm đúng vòng đời này trong một phiên:

```sql
SET ROLE tn_app;
BEGIN;
SET LOCAL app.tenant_id = '2';
SELECT count(*) FROM ops.trips;   -- trong transaction
COMMIT;
SELECT count(*) FROM ops.trips;   -- sau COMMIT, cùng phiên
```

```text
PASS 049 SET LOCAL scope inside / after transaction: 50/0
```

Trong transaction thấy 50 chuyến của tenant 2; sau `COMMIT`, cùng kết nối thấy 0. Ngữ cảnh không sống quá transaction (`ROLLBACK` cũng vậy), nên kết nối về pool «sạch» về mặt tenant. Mẫu trong ứng dụng:

```sql
BEGIN;
SELECT set_config('app.tenant_id', $1, true);   -- true = chỉ trong transaction, như SET LOCAL
-- các câu truy vấn của request
COMMIT;
```

`set_config(name, value, is_local)` với `is_local = true` tương đương `SET LOCAL` nhưng nhận **tham số bind**. `SET` không nhận tham số bind nên dễ bị ghép chuỗi — mở đường injection ngay tại chỗ quan trọng nhất.

- **`SET LOCAL` ngoài transaction vô dụng.** Ở autocommit, `SET LOCAL` hết hiệu lực ngay khi câu của nó xong (PostgreSQL cảnh báo khi dùng ngoài khối transaction); `SELECT` sau đó không có ngữ cảnh.
- **Transaction pooling** (PgBouncer chế độ transaction) có thể đưa mỗi transaction tới một backend khác: `SET` cấp phiên vừa rò sang client khác vừa có thể mất với client hiện tại; dạng ổn định là `set_config(…, true)` trong transaction. Lab **không** chạy PgBouncer; đây là suy ra từ ngữ nghĩa chế độ pool, cần kiểm với cấu hình thật.
- **Lỗi giữa chừng phải `ROLLBACK`** trước khi trả kết nối; kiểm trong test tích hợp, đừng giả định driver làm hộ.

## 6. Default-deny: job báo cáo lặng lẽ hỏng

Trước khi bật RLS, PG-041 đã chứng minh `tn_report` đọc được 100 dòng (`PASS 041 report reads: 100`). Sau khi bật RLS với policy chỉ `TO tn_app`:

```text
PASS 049 report role without policy: 0
```

`tn_report` vẫn có `SELECT` trên bảng, nhưng **không có policy nào áp cho nó**, nên PostgreSQL dùng default-deny: 0 dòng. Không lỗi.

Hình dung ngày triển khai: API đúng, test tenant xanh. Sáng hôm sau, báo cáo doanh thu toàn hệ thống — job đêm chạy bằng `tn_report` — ra 0 đ. Không alert nào vì không có lỗi nào. Cách sửa phụ thuộc câu hỏi nghiệp vụ «báo cáo được thấy gì», và phải quyết định có chủ ý:

```sql
-- Không chạy trong lab: báo cáo nội bộ được đọc mọi tenant, chỉ đọc
CREATE POLICY report_read_all ON ops.trips FOR SELECT TO tn_report USING (true);
```

Báo cáo chạy cho **từng** doanh nghiệp thì nên dùng cùng cơ chế ngữ cảnh như `tn_app` thay vì `USING (true)`. Bài học chung: **bật RLS là đổi quyền của mọi role chạm bảng**. Trước khi bật, liệt kê mọi role có GRANT (`\dp ops.trips` hoặc `relacl`) và viết kỳ vọng số dòng cho từng role.

## 7. Ai đi vòng qua policy: owner, FORCE, superuser

Phần còn lại của ma trận kiểm ba role có đặc quyền:

```sql
SET ROLE tn_owner; SELECT count(*) FROM ops.trips;     -- owner, trước FORCE
ALTER TABLE ops.trips FORCE ROW LEVEL SECURITY;
SET ROLE tn_owner; SELECT count(*) FROM ops.trips;     -- owner, sau FORCE
SELECT count(*) FROM ops.trips;                        -- postgres (superuser), sau FORCE
```

```text
PASS 049 owner bypass: 100
PASS 049 forced owner without policy: 0
PASS 049 superuser still bypasses FORCE: 100
--- 049 table RLS flags
   relname | rls | forced
  ---------+-----+--------
   trips   | t   | t
  (1 row)
```

- **Owner thấy 100 trước FORCE.** API chạy bằng role sở hữu bảng thì RLS chỉ là trang trí — thêm một lý do để runtime không bao giờ là owner (PG-041).
- **Owner thấy 0 sau FORCE.** Không có policy `TO tn_owner` nên owner rơi vào default-deny, cùng bẫy với `tn_report`: migration dữ liệu chạy bằng owner sẽ «không thấy dòng nào để sửa». FORCE cần đi kèm policy cho owner hoặc một role bảo trì riêng có `BYPASSRLS`.
- **Superuser thấy 100 dù có FORCE.** Cờ `rls = t, forced = t` không đổi được điều đó.

Hệ quả cho kiểm thử: **phép kiểm policy chạy bằng `postgres` là vô nghĩa**. Runner luôn `SET ROLE` sang role thật trước khi đếm. Bảng role của PG-041 cho thấy chưa role ứng dụng nào có `BYPASSRLS`:

```text
    rolname   | login | super | bypassrls
  ------------+-------+-------+-----------
   tn_app     | t     | f     | f
   tn_monitor | f     | f     | f
   tn_owner   | f     | f     | f
   tn_report  | f     | f     | f
```

Đưa cột `bypassrls` vào kiểm định kỳ: role được cấp `BYPASSRLS` «tạm để chạy migration» rồi quên thu hồi là cửa sau không ai nhìn thấy.

## 8. Những đường RLS không chặn

Các trường hợp dưới đây **không chạy trong lab**; là SQL để bạn tự thử, theo tài liệu PostgreSQL 18.

**Kênh rò qua ràng buộc duy nhất.** Kiểm khoá chính bỏ qua RLS. `id` duy nhất toàn bảng, nên tenant 1 thử `INSERT … VALUES (60, 1, …)` sẽ nhận lỗi trùng khoá — tức biết chuyến 60 tồn tại dù không đọc được. Cách giảm: khoá gồm `(tenant_id, id)` hoặc định danh khó đoán.

**View và hàm `SECURITY DEFINER`.** Mặc định view truy cập bảng gốc bằng quyền của **chủ view**; view do `tn_owner` tạo trên bảng chưa FORCE sẽ đi vòng qua RLS. Từ PostgreSQL 15, `WITH (security_invoker = true)` buộc view xét quyền và policy theo người gọi ([CREATE VIEW](https://www.postgresql.org/docs/18/sql-createview.html)). Hàm `SECURITY DEFINER` chạy bằng quyền chủ hàm, cùng rủi ro.

**Hàm `LEAKPROOF`.** Điều kiện của policy được áp trước điều kiện người gọi viết, trừ hàm/toán tử được đánh dấu `LEAKPROOF` (cam kết không lộ đối số qua lỗi hay side effect; chỉ superuser đánh dấu được). Vì thế một số điều kiện trên bảng RLS không được planner dùng sớm như trên bảng thường.

**Policy restrictive.** Thêm «chỉ chuyến `completed`» cho báo cáo: `CREATE POLICY only_completed ON ops.trips AS RESTRICTIVE FOR SELECT TO tn_report USING (status = 'completed');`. Nó ghép bằng `AND`, nên chỉ có tác dụng khi đã có policy permissive cho phép; một mình nó vẫn ra default-deny.

**Sao lưu logic.** `pg_dump` mặc định đặt `row_security = off`: truy vấn lẽ ra bị policy lọc sẽ **báo lỗi** thay vì trả thiếu dòng ([row_security](https://www.postgresql.org/docs/18/runtime-config-client.html)), nên dump bằng role không vượt được RLS thất bại to thay vì thiếu dữ liệu âm thầm. Bản dump PG-046 của lab tạo **trước** khi bật RLS, không chứng minh gì về điều này.

## 9. Policy là dữ liệu catalog: còn nguyên sau PITR

Runner cố ý chạy phần 049 **trước** 048 để base backup chứa policy. Sau đó PG-048 thêm chuyến 101 (tenant 1, 45.000 đ), tạo restore point, xoá toàn bộ bảng, rồi phục hồi sang cluster khác tới restore point. Trên cluster đã phục hồi, đang pause:

```sql
SET ROLE tn_app; SET app.tenant_id = '1'; SELECT count(*) FROM ops.trips;
```

```text
PASS 048 recovered data including post-backup insert: 101/3045000/817b908d149b86ec12f316ce544dc676
PASS 048 RLS survives PITR: 51
```

Tenant 1 thấy 51: 50 chuyến gốc cộng chuyến 101 được ghi sau base backup và được replay từ WAL. Policy, cờ ENABLE/FORCE và role đều nằm trong catalog của cluster, nên phục hồi vật lý mang chúng theo nguyên vẹn. Điều này có hai mặt cho nghiệm thu phục hồi:

- Đếm 101 dòng bằng superuser chứng minh **dữ liệu** về; nó không chứng minh **mô hình quyền** về. Cổng nghiệm thu của PG-050 vì vậy có cả bước đếm bằng `tn_app` với ngữ cảnh tenant.
- Với phục hồi logic sang cluster khác, role là đối tượng toàn cluster (PG-046: `pg_dumpall --globals-only`), policy tham chiếu tên role. Thiếu role ở đích thì `CREATE POLICY … TO tn_app` không tạo được. Lab không thử kịch bản này.

## 10. Ranh giới tin cậy: RLS ở đây bảo vệ khỏi điều gì

`app.tenant_id` là tham số tuỳ biến: **phiên nào cũng đặt được, với giá trị nào cũng được**. `tn_app` đặt `'2'` thì thấy tenant 2. Database tin hoàn toàn vào ứng dụng ở bước gán ngữ cảnh.

Mô hình của lab bảo vệ khỏi: câu truy vấn quên điều kiện tenant (endpoint CSV ở đầu bài), INSERT/UPDATE vô tình ghi sai tenant, join hay subquery mới viết bỏ sót bộ lọc. Nó **không** bảo vệ khỏi: credential của `tn_app` bị lộ hay ứng dụng bị chiếm — kẻ tấn công chỉ cần `SET app.tenant_id`; SQL injection chạy được câu tuỳ ý — `set_config` là hàm như mọi hàm; ứng dụng lấy tenant từ tham số URL thay vì từ danh tính đã xác thực.

RLS kiểu này là **lưới an toàn cho bug truy vấn**, không phải ranh giới bảo mật giữa các tenant trước kẻ đã vào được role ứng dụng. Ứng dụng vẫn phải xác thực, suy ra tenant từ danh tính đã xác thực, và đặt ngữ cảnh ở một chỗ duy nhất. Khi cần ranh giới chặt hơn (chỉ nêu khái niệm, lab không dựng):

| Hướng | Ý tưởng | Giá phải trả |
|---|---|---|
| Role database theo tenant | Policy dựa trên `current_user`/membership; kết nối bằng role của tenant | Số role, số pool, credential tăng theo tenant |
| Ngữ cảnh được ký | Hàm `SECURITY DEFINER` kiểm token đã ký rồi mới ghi ngữ cảnh vào chỗ role runtime không sửa trực tiếp được | Tự viết, tự kiểm mã chữ ký; chi phí mỗi transaction |
| Schema/database theo tenant | Cách ly bằng đối tượng và GRANT | Migration nhân theo tenant; báo cáo chéo khó |
| Cluster riêng cho khách lớn | Cách ly cả tài nguyên | Vận hành, backup, giám sát nhân lên |

## 11. Lỗi thường gặp

- **Kiểm policy bằng superuser hoặc owner.** Luôn ra đủ dòng, không chứng minh gì.
- **Chiều ghi lỏng hơn chiều đọc.** `WITH CHECK (true)` viết «cho tiện», hay một policy permissive khác cho cùng role (ghép `OR`), sẽ mở chiều ghi. Đọc đúng không suy ra ghi đúng: luôn kiểm INSERT sai tenant và UPDATE chuyển tenant.
- **Coi 0 dòng là «không có dữ liệu».** Thiếu ngữ cảnh, role không có policy, owner sau FORCE đều trả 0 không lỗi.
- **Bật RLS mà quên role khác.** Job báo cáo im lặng ra 0.
- **`SET` cấp phiên với pool, hoặc ghép chuỗi tenant id.** Dùng `set_config(…, true)` với tham số bind, trong transaction.
- **API chạy bằng owner.** RLS bị bỏ qua hoàn toàn khi chưa FORCE.
- **Quên index.** Mọi truy vấn thêm `tenant_id = …`; bảng lớn không có index bắt đầu bằng `tenant_id` trả giá ở mọi câu. Lab 100 dòng chỉ có khoá chính, không đo chi phí này.

## 12. Đánh đổi

- **Phòng thủ chiều sâu vs minh bạch.** RLS chặn cả bug chưa nghĩ tới, đổi lại câu `SELECT` không còn tự mô tả hết những gì nó lọc.
- **Đóng khi thiếu vs dễ gỡ lỗi.** «0 dòng không lỗi» an toàn hơn mở khi thiếu, nhưng đẩy việc phát hiện sang giám sát ứng dụng.
- **FORCE vs vận hành.** Owner không còn là lối tắt; job bảo trì cần role và policy rõ ràng.
- **Chi phí truy vấn.** Thêm điều kiện và lời gọi `current_setting` mỗi câu; policy có subquery còn đắt hơn. Bài này không có số đo hiệu năng RLS — đo trên dữ liệu thật trước khi kết luận.

## 13. Bài tập

Không có lời giải chạy sẵn; mỗi bài nêu điều lời giải đúng phải chứng minh. Chạy trên bản sao lab hoặc cluster thử, không trên database có dữ liệu thật.

**Bài 1 — Tái sử dụng kết nối.** Trên **một** kết nối, ba «request» liên tiếp: (1) `BEGIN`, `set_config('app.tenant_id','1',true)`, đếm, `COMMIT`; (2) không đặt ngữ cảnh, đếm; (3) đặt tenant 2, đếm. Lặp lại với (1) lỗi giữa chừng và `ROLLBACK`. Đạt khi ra 50 / 0 / 50 ở cả hai lượt, **và** chứng minh đổi (1) sang `SET` cấp phiên làm (2) thấy 50 — tức là tái hiện được lỗi rò, không chỉ kết quả đúng.

**Bài 2 — Báo cáo sau khi bật RLS.** Viết policy cho `tn_report` (báo cáo nội bộ mọi tenant, hoặc báo cáo từng doanh nghiệp). Đạt khi có bảng kỳ vọng số dòng cho **mọi** role có GRANT trên `ops.trips` (`tn_app` có/không ngữ cảnh, `tn_report`, `tn_owner` trước/sau FORCE), kiểm bằng `SET ROLE`, và `tn_report` vẫn không ghi được.

**Bài 3 — Kênh rò khoá chính.** Với ngữ cảnh tenant 1, dùng một câu INSERT chứng minh chuyến 60 tồn tại, rồi đề xuất thay đổi schema đóng kênh rò. Đạt khi nêu SQLSTATE bạn thật sự quan sát được và giải thích vì sao thay đổi không phá ràng buộc nghiệp vụ.

**Bài 4 — View đi vòng và nghiệm thu sau phục hồi.** Tạo view bằng `tn_owner` trên bảng **chưa** FORCE, cấp `SELECT` cho `tn_app`, đếm qua view khi không có ngữ cảnh; tạo lại với `security_invoker = true`. Sau đó viết danh sách kiểm (SQL + kỳ vọng) cho cluster vừa phục hồi: `relrowsecurity`/`relforcerowsecurity`, `pg_policies`, `rolbypassrls`, đếm theo tenant bằng `tn_app`. Đạt khi giải thích được hai kết quả view theo «quyền của ai truy cập bảng gốc», và danh sách bắt được một bản phục hồi mất policy hoặc có role bất ngờ mang `BYPASSRLS`.

## 14. Đọc tiếp

- [PostgreSQL 18 · Row Security Policies](https://www.postgresql.org/docs/18/ddl-rowsecurity.html): default-deny, owner, FORCE, BYPASSRLS, kênh rò qua ràng buộc.
- [PostgreSQL 18 · CREATE POLICY](https://www.postgresql.org/docs/18/sql-createpolicy.html): `USING`/`WITH CHECK` theo từng lệnh, permissive và restrictive.
- [PostgreSQL 18 · ALTER TABLE](https://www.postgresql.org/docs/18/sql-altertable.html): `ENABLE`/`FORCE ROW LEVEL SECURITY`.
- [PostgreSQL 18 · System Administration Functions](https://www.postgresql.org/docs/18/functions-admin.html): `current_setting(…, missing_ok)`, `set_config(…, is_local)`.
- [PostgreSQL 18 · Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html): `row_security`.
- [PostgreSQL 18 · CREATE VIEW](https://www.postgresql.org/docs/18/sql-createview.html): `security_invoker`.
- Lab: `examples/toi-noi-postgresql/p05/run.mjs` (khối PG-049), `p05/OUTPUT.txt` (các dòng `049` và `048 RLS survives PITR`), `p05/00-schema.sql`, `p05/README.md`, `p05/RUNBOOK.md` (dòng «Tenant đọc sai»).

Bài tiếp theo — bài thực hành cuối Phần 5: [PG-050 · Bàn giao runbook vận hành và phục hồi](/articles/postgresql-thuc-hanh-runbook-phuc-hoi).

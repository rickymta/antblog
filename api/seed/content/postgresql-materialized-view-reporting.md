---
slug: postgresql-materialized-view-reporting
title: "Materialized view cho báo cáo doanh thu: độ tươi, REFRESH CONCURRENTLY và đối soát với sổ chuyến"
summary: "Màn hình doanh thu theo thành phố của Tới Nơi đọc từ một materialized view. Lab PostgreSQL 18.1 cho thấy báo cáo đứng ở 1 chuyến / 85.000 đ trong khi sổ đã 2 / 135.000, REFRESH thường giữ AccessExclusiveLock và chặn người đọc, CONCURRENTLY bị từ chối (55000) khi thiếu unique index rồi chạy được mà không chặn ai, và cách chốt hợp đồng độ tươi bằng refreshed_at và phép đối soát."
category: postgresql
level: intermediate
tags: [postgresql, pg-056, materialized-view, refresh-concurrently, reporting, locking, reconciliation, toi-noi]
language: vi
---

8:55 sáng, trưởng ca vận hành của Tới Nơi mở màn hình «doanh thu hôm nay theo thành phố» trước cuộc họp giao ban. Câu truy vấn gốc quét bảng chuyến, nhóm theo thành phố, cộng tiền — trên bảng hàng chục triệu dòng, mỗi lần mở màn hình là vài giây và một lần quét nặng chồng lên giờ cao điểm. Ai đó đề xuất: «tính sẵn kết quả, màn hình chỉ việc đọc».

Đề xuất đúng, và PostgreSQL có sẵn công cụ: **materialized view**. Nhưng ba tuần sau, kế toán hỏi vì sao màn hình ghi HCM thiếu một chuyến so với sổ; một tối khác, job tính lại báo cáo làm màn hình đứng mười mấy giây; và sau một lần chuyển primary, báo cáo lệch sổ 50.000 đ mà không ai hiểu vì sao. Ba sự cố ấy là ba câu hỏi mà mọi bảng tính sẵn phải trả lời: **dữ liệu cũ tới mức nào thì chấp nhận được, ai tính lại và tính lại có chặn người đọc không, làm sao biết con số khớp với sổ gốc.**

<div class="tkh-callout">
<span class="tkh-callout-title">PG-056 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-logical-replication-cdc">PG-055 · Logical replication và CDC</a>; nên ôn <a href="/articles/postgresql-aggregate-group-by-having">PG-012 · COUNT, SUM, GROUP BY</a>, <a href="/articles/postgresql-lock-row-table">PG-033 · Khoá dòng, khoá bảng và thời gian chờ</a> và <a href="/articles/postgresql-pg-stat-activity-quan-sat">PG-043 · Quan sát phiên</a>. Lab: <code>examples/toi-noi-postgresql/p06</code>, chạy <code>npm run p06</code> tại <code>examples/toi-noi-postgresql</code> với PostgreSQL 18 và biến <code>PG17_BIN</code> trỏ tới thư mục bin của PostgreSQL 17 (xem README của lab). Kết quả trong <code>p06/OUTPUT.txt</code>. Sau bài này, bạn định nghĩa được độ tươi của một báo cáo tính sẵn, chọn giữa REFRESH thường và CONCURRENTLY dựa trên khoá mà mỗi cách giữ, và viết phép đối soát báo cáo với sổ gốc.</p>
</div>

## 1. Dữ liệu và cách chạy

```powershell
cd examples/toi-noi-postgresql
$env:PG17_BIN = 'D:\pg17\pgsql\bin'   # chỉ cần cho PG-059, nhưng runner kiểm tra từ đầu
npm run p06
```

Runner `p06/run.mjs` dựng các cluster PostgreSQL 18.1 tạm, chỉ nghe localhost, chạy mười bài của Phần 6 theo thứ tự cố định rồi xoá sạch. `p06/fixture.sql` tạo một sổ cước rút gọn và báo cáo tính sẵn trên nó:

```sql
CREATE TABLE fares (
  id int PRIMARY KEY,
  city text NOT NULL,
  fare int NOT NULL CHECK (fare >= 0)  -- VND
);
INSERT INTO fares VALUES (1, 'HCM', 85000), (2, 'HN', 65000);
CREATE MATERIALIZED VIEW fare_report AS
  SELECT city, count(*) AS trips, sum(fare) AS total, now() AS refreshed_at
  FROM fares GROUP BY city;
```

Sổ bắt đầu với 2 chuyến / 150.000 đ. Bảng `fares` đi xuyên suốt Phần 6: sau bài này, PG-055 sửa và thêm cột, PG-052 và PG-058 thêm chuyến qua replication và failover, PG-060 đối soát lần cuối. Riêng khối PG-056 chỉ thêm một chuyến thật: chuyến 3 (HCM, 50.000 đ), đưa sổ lên 3 chuyến / 200.000 đ. Ngoài ra runner chèn tạm một chuyến 99 (DN, 1.000 đ) để tạo thay đổi cho lần refresh concurrently, rồi xoá nó trước bước đối soát.

Dữ liệu chỉ vài dòng, một máy. Lab chứng minh **hành vi** (ai chờ ai, khoá nào được giữ, số nào đúng), không đo thời gian refresh. Khối kết quả dưới đây chép từ `OUTPUT.txt`, đã bỏ hai khoảng trắng thụt lề runner thêm vào.

## 2. View, materialized view và bảng tổng hợp

Ba thứ hay bị gọi lẫn:

- **View** thường là một truy vấn có tên. Mỗi lần bạn `SELECT` từ view, PostgreSQL chạy lại truy vấn định nghĩa trên dữ liệu hiện tại. Luôn tươi, nhưng mỗi lần đọc trả đủ chi phí.
- **Materialized view** (view vật chất hoá — tôi sẽ viết tắt MV) cũng có truy vấn định nghĩa, nhưng **kết quả được lưu thành dữ liệu thật** trên đĩa, giống một bảng. Đọc MV là đọc dữ liệu đã lưu, nhanh như đọc bảng nhỏ. Dữ liệu chỉ thay đổi khi ai đó chạy `REFRESH MATERIALIZED VIEW`. Tài liệu PostgreSQL 18 (mục *Materialized Views*) nói rõ: MV không tự cập nhật khi bảng gốc đổi.
- **Bảng tổng hợp** (summary table) do ứng dụng hoặc trigger tự ghi. Linh hoạt nhất, nhưng logic cập nhật — và lỗi của nó — là của bạn.

MV nằm giữa: PostgreSQL lo phần tính lại, bạn lo phần **khi nào** tính lại. Bạn có thể đánh index trên MV như trên bảng. Bạn không thể `INSERT`/`UPDATE` trực tiếp vào nó.

`fare_report` có thêm cột `refreshed_at` bằng `now()`. Đây là cách đơn giản để mỗi dòng mang theo dấu thời gian của lần tính — nền tảng của «hợp đồng độ tươi» ở mục 4. Cột này có một tác dụng phụ với `CONCURRENTLY` mà mục 7 sẽ nói.

## 3. Thí nghiệm 1: báo cáo cũ một cách lặng lẽ

Runner thêm chuyến 3 rồi đọc cả hai phía:

```sql
INSERT INTO fares VALUES (3, 'HCM', 50000);
SELECT trips || '/' || total FROM fare_report WHERE city = 'HCM';
SELECT count(*) || '/' || sum(fare) FROM fares WHERE city = 'HCM';
```

```text
PASS 056 report before REFRESH (HCM): 1/85000
PASS 056 base table HCM: 2/135000
```

Báo cáo nói HCM có **1 chuyến / 85.000 đ**; sổ gốc có **2 chuyến / 135.000 đ**. Không lỗi, không cảnh báo: MV là kết quả của lần tính khi fixture tạo nó, lúc chuyến 3 chưa tồn tại — đúng sự cố «kế toán hỏi vì sao thiếu một chuyến». Báo cáo cũ là **bản chất** của MV; lỗi nằm ở chỗ không ai nói cho người đọc biết con số ấy tính lúc nào, và cũ tới đâu thì là quá cũ.

## 4. Hợp đồng độ tươi

**Độ tươi** (freshness) của báo cáo là khoảng cách giữa dữ liệu trong báo cáo và dữ liệu trong sổ. Một **hợp đồng độ tươi** là bản thoả thuận viết ra giấy giữa bên làm dữ liệu và bên dùng báo cáo, gồm ít nhất:

1. **Mức cũ tối đa** (SLA — cam kết mức dịch vụ): «màn hình doanh thu cũ không quá 15 phút trong giờ làm việc». Con số này là quyết định nghiệp vụ, không phải kỹ thuật.
2. **Dấu thời gian hiển thị**: màn hình luôn in «số liệu tính lúc 08:45». Người đọc phải thấy được độ cũ, không phải đoán.
3. **Ai refresh, theo lịch nào, và ai được báo khi refresh thất bại** (mục 10).
4. **Màn hình nào không được dùng MV**: nếu mỗi lần thanh toán phải thấy ngay trên màn hình, màn hình đó đọc bảng giao dịch, không đọc báo cáo tính sẵn.

Ý nghĩa chính xác của `refreshed_at = now()` cần được ghi vào hợp đồng. Trong PostgreSQL, `now()` trả **thời điểm bắt đầu giao dịch**, không phải thời điểm câu lệnh chạy hay commit (tài liệu *Date/Time Functions*). Ở mức cô lập mặc định Read Committed, câu `REFRESH` chụp snapshot khi câu bắt đầu — tức không sớm hơn `now()` của giao dịch ([PG-034](/articles/postgresql-isolation-read-committed-repeatable-read)). Vì vậy `refreshed_at` là **cận dưới**: mọi giao dịch đã commit trước thời điểm đó chắc chắn có trong báo cáo; vài giao dịch commit ngay sau đó cũng có thể có. Nếu job mở giao dịch rồi làm việc khác lâu trước khi `REFRESH`, cận dưới đó cách xa thực tế hơn.

Hai điểm tốt của cách đặt dấu thời gian **bên trong** MV:

- Nếu `REFRESH` thất bại, giao dịch rollback, `refreshed_at` giữ giá trị cũ. Màn hình không bao giờ hiển thị «vừa tính xong» cho một lần tính hỏng — khác với cách ghi dấu thời gian vào một bảng khác **trước** khi job chạy.
- Lab kiểm được nó tiến lên sau refresh:

```text
PASS 056 refreshed_at advanced: t
```

Một lưu ý nghiệp vụ: `refreshed_at` đo theo thời điểm **commit** vào sổ, không theo thời điểm nghiệp vụ của chuyến. Một chuyến kết thúc lúc 08:40 nhưng tới 08:50 mới được ghi (mạng chập chờn, hàng đợi chậm) sẽ không có trong báo cáo «tính lúc 08:45». Nếu báo cáo chốt theo ngày nghiệp vụ, hợp đồng cần thêm quy tắc cho dữ liệu đến muộn.

## 5. REFRESH thường: đúng, nhưng chặn người đọc

`REFRESH MATERIALIZED VIEW fare_report;` chạy lại truy vấn định nghĩa và **thay toàn bộ** nội dung MV. Theo tài liệu PostgreSQL 18 (mục *Explicit Locking*), lệnh này (không có `CONCURRENTLY`) lấy khoá **`ACCESS EXCLUSIVE`** trên MV — mức khoá bảng mạnh nhất, xung đột với mọi mức khác, kể cả `ACCESS SHARE` mà một câu `SELECT` cần.

Để quan sát được, runner mở hai phiên psql thật, giữ giao dịch refresh mở có chủ đích:

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | refresh | `BEGIN; REFRESH MATERIALIZED VIEW fare_report;` | refresh xong, giao dịch **chưa** commit |
| T2 | reader | `SELECT count(*) FROM fare_report;` | treo |
| T3 | runner | đọc `pg_stat_activity` và `pg_locks` | xem bên dưới |
| T4 | refresh | `COMMIT;` | reader nhận kết quả |

```text
PASS 056 plain REFRESH blocks readers: reader wait_event_type = Lock
```

`wait_event_type = Lock` nghĩa là phiên reader đang chờ một khoá cấp đối tượng ([PG-043](/articles/postgresql-pg-stat-activity-quan-sat)). Còn `pg_locks` trên `fare_report`, lọc theo hai phiên:

```text
--- 056 locks during plain REFRESH
    s    |        mode         | granted
---------+---------------------+---------
 refresh | AccessExclusiveLock | t
 refresh | ExclusiveLock       | t
 refresh | ShareLock           | t
 reader  | AccessShareLock     | f
(4 rows)
```

Đọc bảng này:

- Phiên refresh giữ **`AccessExclusiveLock`** (`granted = t`) — đúng như tài liệu.
- Phiên reader xin **`AccessShareLock`** và **chưa được cấp** (`granted = f`). Đó là toàn bộ lý do nó treo.
- Phiên refresh còn giữ `ExclusiveLock` và `ShareLock` trên cùng MV. Các khoá của **cùng một giao dịch** không xung đột với nhau, và cả hai đều yếu hơn `AccessExclusiveLock` đang giữ, nên chúng không thay đổi kết luận. Lab không truy nguyên bước nội bộ nào của REFRESH lấy hai khoá này; bài không suy đoán.

Trong lab, cửa sổ chặn kéo dài vì giao dịch được giữ mở có chủ đích. Ngoài đời, nó dài bằng thời gian chạy truy vấn định nghĩa cộng mọi thứ khác trong cùng giao dịch — «màn hình đứng mười mấy giây» ở đầu bài. Còn một hiệu ứng tệ hơn đã gặp ở PG-033: nếu `REFRESH` phải **chờ** một câu đọc dài đang giữ `ACCESS SHARE`, thì yêu cầu `ACCESS EXCLUSIVE` đang xếp hàng của nó khiến **mọi** câu đọc mới phía sau cũng phải xếp hàng. Một báo cáo chậm cộng một job refresh có thể làm treo tất cả người đọc MV.

REFRESH thường có ưu điểm của nó: tài liệu PostgreSQL 18 (*REFRESH MATERIALIZED VIEW*) nói khi refresh tác động lên nhiều dòng, cách này thường tốn ít tài nguyên hơn và xong nhanh hơn `CONCURRENTLY`. Nó cũng là cách duy nhất cho MV chưa từng có dữ liệu (mục 6).

## 6. CONCURRENTLY: điều kiện trước khi được dùng

`REFRESH MATERIALIZED VIEW CONCURRENTLY` tính lại mà **không chặn** người đọc. Runner thử ngay khi MV chưa có index nào:

```text
--- 056 CONCURRENTLY without unique index
ERROR:  55000: cannot refresh materialized view "public.fare_report" concurrently
HINT:  Create a unique index with no WHERE clause on one or more columns of the materialized view.
```

`55000` là `object_not_in_prerequisite_state` — đối tượng chưa ở trạng thái đủ điều kiện. Tài liệu PostgreSQL 18 nêu ba điều kiện của `CONCURRENTLY`:

1. MV phải có **ít nhất một unique index** chỉ dùng **tên cột** và **bao phủ mọi dòng** — tức không phải expression index và không có mệnh đề `WHERE` (partial index). HINT trong lỗi nói đúng điều đó.
2. MV phải **đã có dữ liệu**. Một MV tạo bằng `WITH NO DATA` (chưa populate, đọc vào sẽ lỗi) phải được `REFRESH` thường một lần trước.
3. Dù dùng `CONCURRENTLY`, **mỗi lúc chỉ một lệnh REFRESH** chạy trên một MV.

Vì sao cần unique index? Hiểu ở mức khái niệm theo tài liệu: thay vì thay cả MV, `CONCURRENTLY` tính kết quả mới vào một bảng tạm, **so sánh** (diff) với nội dung hiện tại, rồi chỉ áp các thay đổi bằng `DELETE` và `INSERT` trên MV. Để ghép dòng cũ với dòng mới, nó cần một khoá xác định duy nhất mỗi dòng — chính là unique index.

Với `fare_report`, khoá tự nhiên là `city`: truy vấn `GROUP BY city` nên mỗi thành phố đúng một dòng. Runner tạo:

```sql
CREATE UNIQUE INDEX fare_report_city ON fare_report (city);
```

Chọn khoá phải theo **định nghĩa truy vấn**, không theo dữ liệu hiện có. Nếu sau này báo cáo đổi sang `GROUP BY service_date, city, currency`, unique index phải đổi theo `(service_date, city, currency)`; nếu không, `CREATE UNIQUE INDEX` sẽ thất bại ngay khi dữ liệu có hai dòng cùng `city`, hoặc tệ hơn là lần REFRESH sau thất bại.

## 7. CONCURRENTLY: người đọc không phải chờ

Runner chèn tạm chuyến 99 (để refresh có việc làm), rồi lặp lại lịch chạy ở mục 5 với `CONCURRENTLY`:

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | refresh | `BEGIN; REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;` | refresh xong, giao dịch chưa commit |
| T2 | reader | `SELECT count(*) FROM fare_report;` | trả lời ngay |
| T3 | runner | đọc `pg_locks` của phiên refresh | xem bên dưới |
| T4 | refresh | `COMMIT;` | nội dung mới hiển thị cho mọi người |

```text
PASS 056 CONCURRENTLY: reader answered while refresh transaction open (no wait)
```

Runner đánh dấu «no wait» khi reader trả lời trong chưa tới 2 giây; đây là cổng hành vi, không phải số đo. Trong lúc giao dịch refresh còn mở, reader đọc thấy **nội dung cũ** — MVCC ([PG-032](/articles/postgresql-mvcc-snapshot)) giữ các phiên bản dòng cũ cho tới khi giao dịch refresh commit.

Khoá phiên refresh giữ trên `fare_report`:

```text
--- 056 locks during REFRESH CONCURRENTLY
    s    |       mode       | granted
---------+------------------+---------
 refresh | AccessShareLock  | t
 refresh | ExclusiveLock    | t
 refresh | RowExclusiveLock | t
(3 rows)
```

Mấu chốt là **`ExclusiveLock`** thay cho `AccessExclusiveLock`. Hai cái tên gần giống nhau nhưng khác hẳn. Theo bảng xung đột trong tài liệu *Explicit Locking*, `EXCLUSIVE` xung đột với mọi mức **trừ** `ACCESS SHARE` — tài liệu viết rằng nó chỉ cho phép các khoá `ACCESS SHARE` đồng thời, tức chỉ việc **đọc** chạy song song được, và nêu đích danh `REFRESH MATERIALIZED VIEW CONCURRENTLY` là lệnh lấy mức khoá này.

| Mức khoá trên MV | Ai lấy | `SELECT` (ACCESS SHARE) | REFRESH khác |
|---|---|---|---|
| `ACCESS EXCLUSIVE` | REFRESH thường | **chờ** | chờ |
| `EXCLUSIVE` | REFRESH CONCURRENTLY | chạy được | chờ |

Hai khoá còn lại khớp với cơ chế diff: `RowExclusiveLock` là mức mà `INSERT`/`UPDATE`/`DELETE` lấy, phù hợp với việc áp thay đổi bằng `DELETE`/`INSERT`; `AccessShareLock` là mức của việc đọc MV để so sánh. Cả hai đều không xung đột với `AccessShareLock` của reader.

Cột cuối của bảng trên cũng giải thích điều kiện 3 ở mục 6: `EXCLUSIVE` tự xung đột với chính nó, nên hai job cùng `REFRESH ... CONCURRENTLY` trên một MV sẽ xếp hàng. Lab không chạy hai refresh song song; bài tập 2 làm việc đó.

Một quan sát phụ: lab chạy `REFRESH ... CONCURRENTLY` **bên trong** `BEGIN … COMMIT` và lệnh thành công. Khác với `CREATE INDEX CONCURRENTLY`, lệnh này không bị cấm trong khối giao dịch — nhờ vậy job có thể refresh và ghi nhật ký refresh trong cùng một giao dịch.

**`CONCURRENTLY` không miễn phí.** Theo tài liệu và theo cơ chế diff ở trên:

- Nó phải tính kết quả mới **và** so sánh với kết quả cũ, nên thường nặng và chậm hơn REFRESH thường khi nhiều dòng thay đổi; tài liệu nói nó có thể nhanh hơn khi chỉ ít dòng thay đổi.
- Mỗi dòng thay đổi thành một `DELETE` + `INSERT` trên MV, tức sinh tuple chết như một bảng thường — MV cũng cần VACUUM ([PG-044](/articles/postgresql-vacuum-autovacuum-bloat)). REFRESH thường thay cả nội dung nên không để lại tuple chết theo cách đó.
- **Bẫy của chính `fare_report`**: cột `refreshed_at = now()` có giá trị mới ở **mọi** dòng sau mỗi lần refresh, nên theo cơ chế so sánh cả dòng, mọi dòng đều bị coi là «đã đổi» và bị xoá–chèn lại. Với hai dòng không sao; với MV hàng triệu dòng, lợi ích «chỉ áp phần thay đổi» biến mất. Lab không đo điều này (dữ liệu quá nhỏ để thấy). Với MV lớn, cân nhắc ghi dấu thời gian vào một bảng nhật ký refresh riêng, cập nhật trong **cùng giao dịch** với `REFRESH` để vẫn giữ tính chất «thất bại thì không tiến».

## 8. Đối soát với sổ gốc

Runner xoá chuyến tạm 99, refresh lần nữa, rồi kiểm:

```sql
DELETE FROM fares WHERE id = 99;
REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;
SELECT (SELECT sum(total) FROM fare_report) = (SELECT sum(fare) FROM fares);
```

```text
PASS 056 report after REFRESH (HCM): 2/135000
PASS 056 refreshed_at advanced: t
PASS 056 reconcile report vs base: t
--- 056 fare_report
 city | trips | total
------+-------+--------
 HCM  |     2 | 135000
 HN   |     1 |  65000
(2 rows)
```

HCM giờ là 2 chuyến / 135.000 đ, khớp sổ; tổng hai thành phố 200.000 đ đúng bằng `sum(fare)` của 3 chuyến; dòng DN của chuyến tạm đã biến mất cùng lần refresh (diff đã `DELETE` nó).

Phép đối soát của lab chỉ so **tổng tiền**. Trong vận hành, hãy so ít nhất **cả số chuyến lẫn tổng tiền, theo từng nhóm** — hai sai lệch ngược chiều có thể triệt tiêu nhau trong tổng chung (một chuyến bị đếm hai lần ở HCM, một chuyến bị mất ở HN có giá tương tự). Mẫu đối soát theo nhóm, **không chạy trong lab**:

```sql
SELECT coalesce(r.city, f.city) AS city,
       r.trips AS report_trips, f.trips AS ledger_trips,
       r.total AS report_total, f.total AS ledger_total
FROM fare_report r
FULL JOIN (SELECT city, count(*) AS trips, sum(fare) AS total
           FROM fares GROUP BY city) f USING (city)
WHERE r.trips IS DISTINCT FROM f.trips
   OR r.total IS DISTINCT FROM f.total;
-- Kỳ vọng ngay sau REFRESH, không có ghi mới: 0 dòng
```

`FULL JOIN` bắt được cả thành phố có trong sổ mà thiếu trong báo cáo và ngược lại; `IS DISTINCT FROM` xử lý đúng `NULL` ([PG-013](/articles/postgresql-join-quan-he-cardinality), [PG-012](/articles/postgresql-aggregate-group-by-having)). Trên hệ thống đang nhận ghi, hai vế phải đọc cùng một mốc: chạy đối soát trong một giao dịch Repeatable Read ngay sau refresh, hoặc so theo mốc nghiệp vụ (chuyến trước 00:00) thay vì «tất cả». Như [PG-020](/articles/postgresql-thuc-hanh-sql-doi-soat) đã nhấn mạnh, đối soát chỉ có giá trị khi chỉ ra được dòng nào lệch, không chỉ «lệch hay không».

## 9. Sau failover: báo cáo vẫn cũ như trước

Ở cuối runner, PG-060 kiểm báo cáo trên primary mới, sau khi PG-058 đã failover:

```text
PASS 060 report projection stale after failover: 200000 vs 250000
PASS 060 report reconciled after REFRESH: t
```

Báo cáo nói 200.000 đ, sổ nói 250.000 đ. Đọc kỹ để không kết luận sai: 200.000 chính là tổng của **lần refresh cuối ở bài này**. Sau đó PG-055, PG-052 và PG-058 đã sửa và thêm chuyến (sổ lên 5 chuyến / 250.000), nhưng không ai refresh. Replication vật lý ([PG-052](/articles/postgresql-replication-primary-standby)) sao chép nội dung MV y nguyên như mọi bảng; failover không tính lại gì cả. Primary mới thừa hưởng một báo cáo cũ đúng bằng mức cũ của nó trên primary cũ.

Bài học cho runbook sự cố ([PG-058](/articles/postgresql-ha-failover-read-consistency), [PG-060](/articles/postgresql-capstone-van-hanh-toi-noi)): sau khi đã xác nhận sổ gốc đúng, refresh các projection rồi đối soát lại; **không** restore đè sổ chỉ vì báo cáo lệch. Ngoài ra, `REFRESH` là thao tác ghi; trên standby (chỉ đọc) nó không chạy được, nên job refresh phải luôn trỏ vào primary hiện tại, và phải chạy lại được sau khi primary đổi.

## 10. Ai refresh, khi nào, và theo dõi thế nào

PostgreSQL lõi **không** có lịch refresh tự động. Các lựa chọn, **không lựa chọn nào chạy trong lab**:

| Cách | Ưu | Nhược |
|---|---|---|
| Scheduler của ứng dụng / cron hệ điều hành | Đơn giản, nằm trong hệ thống deploy sẵn có | Phải tự xử lý trùng lặp, primary đổi chỗ |
| Extension `pg_cron` | Lịch nằm trong database | Ngoài lõi, cần cài và cấp quyền riêng |
| Refresh ngay sau batch nhập dữ liệu | Báo cáo tươi đúng lúc có dữ liệu mới | Gắn chặt với pipeline nhập |

Ví dụ với `pg_cron`, **không chạy trong lab**:

```sql
SELECT cron.schedule('fare-report', '*/15 * * * *',
  $$SET lock_timeout = '5s'; REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report$$);
```

Dù chọn cách nào, job refresh nên:

- Đặt `lock_timeout` để không treo vô hạn sau một refresh khác hay một thao tác DDL, và `statement_timeout` theo thời lượng kỳ vọng.
- Chạy bằng role có quyền phù hợp. Từ PostgreSQL 17 có quyền `MAINTAIN` cho phép refresh mà không cần là chủ sở hữu MV; hãy kiểm tài liệu phiên bản bạn dùng trước khi cấp quyền.
- Báo động khi **thất bại** và khi **quá hạn** — hai thứ khác nhau: job có thể không chạy (scheduler chết) mà không có dòng log lỗi nào.

Câu theo dõi độ tươi, **không chạy trong lab**, đủ đơn giản để đưa vào dashboard:

```sql
SELECT max(refreshed_at) AS refreshed_at,
       now() - max(refreshed_at) AS age,
       now() - max(refreshed_at) > interval '15 minutes' AS breach_sla
FROM fare_report;
```

Theo dõi thêm thời lượng mỗi lần refresh: refresh 14 phút cho lịch 15 phút là sự cố đang tới.

## 11. Khi MV không phải câu trả lời

- **Cần tươi theo từng giao dịch**: đọc bảng gốc với index phù hợp ([PG-025](/articles/postgresql-index-nhieu-cot-partial-expression)).
- **Bảng tổng hợp do trigger duy trì**: cập nhật tức thì, nhưng mỗi lần ghi chuyến kéo theo cập nhật dòng tổng — ở tải cao, dòng tổng của HCM thành điểm tranh chấp khoá ([PG-033](/articles/postgresql-lock-row-table)). Lab không chạy cách này.
- **Bảo trì tăng dần** (incremental view maintenance — chỉ tính phần thay đổi khi bảng gốc đổi): PostgreSQL 18 lõi không có; có extension bên ngoài nhưng lab không thử, bài không đánh giá.
- **Báo cáo nặng chạy ở nơi khác**: đẩy dữ liệu sang hệ thống phân tích qua logical replication/CDC ([PG-055](/articles/postgresql-logical-replication-cdc)), giữ primary cho giao dịch.

## 12. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| Báo cáo thiếu chuyến so với sổ | MV chưa refresh (lab: 1/85.000 so với 2/135.000) | Hiển thị `refreshed_at`; SLA và cảnh báo quá hạn |
| Màn hình đứng khi job chạy | REFRESH thường giữ `AccessExclusiveLock`, reader chờ `AccessShareLock` | `CONCURRENTLY` với unique index; `lock_timeout` cho job |
| `55000 cannot refresh ... concurrently` | Thiếu unique index trên tên cột, không `WHERE` | Tạo unique index theo khoá của `GROUP BY` |
| `CONCURRENTLY` lỗi trên MV mới tạo | MV tạo `WITH NO DATA`, chưa populate | REFRESH thường lần đầu |
| Hai job refresh chồng nhau | `ExclusiveLock` tự xung đột | Một scheduler duy nhất; `lock_timeout`; khoảng lịch dài hơn thời lượng refresh |
| `refreshed_at` mới mà số vẫn sai | Dấu thời gian ghi ngoài giao dịch refresh, hoặc đọc nhầm node | Đặt dấu thời gian trong cùng giao dịch; đối soát theo nhóm |
| Báo cáo lệch sau failover | Replication sao chép MV cũ; không ai refresh (lab 060: 200.000 so với 250.000) | Runbook: xác nhận sổ, refresh, đối soát |
| MV phình dần | `CONCURRENTLY` xoá–chèn, cộng cột `now()` làm mọi dòng đổi | Theo dõi tuple chết; tách dấu thời gian khỏi MV lớn |

## 13. Đánh đổi

- **Tươi hay rẻ.** Refresh dày cho báo cáo tươi nhưng cộng tải đều đặn lên primary; refresh thưa nhẹ hơn nhưng báo cáo cũ hơn. SLA quyết định, không phải sở thích kỹ thuật.
- **REFRESH thường hay CONCURRENTLY.** Thường: ít tài nguyên hơn khi nhiều dòng đổi, không cần unique index, nhưng chặn người đọc suốt thời gian chạy. CONCURRENTLY: người đọc không bị chặn, nhưng cần unique index, nặng hơn khi thay đổi nhiều, và để lại tuple chết.
- **MV hay bảng tổng hợp.** MV: PostgreSQL lo tính lại, logic là một câu SQL dễ kiểm; bảng tổng hợp: tươi tức thì nhưng logic cập nhật và tranh chấp khoá là của bạn.

## 14. Bài tập

**Bài 1. Refresh thất bại thì `refreshed_at` có đứng yên không?** Tạo lại `fares` và `fare_report` như fixture. Ghi `max(refreshed_at)`, rồi làm một lần refresh thất bại (ví dụ: một phiên khác giữ `LOCK TABLE fare_report IN ACCESS SHARE MODE` trong giao dịch mở, phiên refresh đặt `lock_timeout = '1s'` rồi chạy REFRESH thường).

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Lịch chạy hai phiên có cột bước, phiên, câu lệnh, kết quả; SQLSTATE của lần refresh thất bại (dự kiến <code>55P03</code> <code>lock_not_available</code>, nhưng ghi đúng mã bạn nhận được); <code>max(refreshed_at)</code> trước, sau lần thất bại (phải bằng nhau) và sau một lần refresh thành công (phải lớn hơn). Kết luận «dấu thời gian trong MV không tiến khi refresh hỏng» chỉ hợp lệ khi có đủ ba giá trị.</p>
</details>

**Bài 2. Hai lần CONCURRENTLY chồng nhau.** Với unique index `fare_report_city`, mở phiên A: `BEGIN; REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;` (chưa commit), phiên B chạy cùng lệnh, phiên C đọc `fare_report`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Từ <code>pg_locks</code> và <code>pg_stat_activity</code> khi cả ba đang chạy: A giữ <code>ExclusiveLock</code> <code>granted = t</code>; B xin <code>ExclusiveLock</code> với <code>granted = f</code> và <code>wait_event_type = Lock</code>; C trả lời không chờ. Sau khi A commit, B hoàn tất. Phải dùng output của bạn để đối chiếu với bảng xung đột khoá trong tài liệu <em>Explicit Locking</em>, không chỉ trích lại bài.</p>
</details>

**Bài 3. Báo cáo theo ngày và chi phí của CONCURRENTLY.** Thêm cột `service_date date` cho `fares`, sinh khoảng 1 triệu chuyến trải trên 90 ngày, tạo MV `GROUP BY service_date, city` **không** có cột `now()`. Đo thời gian REFRESH thường và CONCURRENTLY khi (a) chỉ một ngày thay đổi, (b) mọi ngày thay đổi; sau đó thêm cột `now()` vào MV và lặp lại (a).

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Unique index đúng khoá <code>(service_date, city)</code>; bảng thời gian (mỗi ô ít nhất ba lần đo, ghi cấu hình máy, cache nóng/lạnh, phiên bản PostgreSQL); <code>n_tup_del</code>/<code>n_tup_ins</code> và <code>n_dead_tup</code> của MV trong <code>pg_stat_user_tables</code> trước và sau mỗi lần CONCURRENTLY. Kết luận «cột now() khiến CONCURRENTLY xoá–chèn mọi dòng» chỉ hợp lệ nếu số dòng bị xoá ở trường hợp (a) có cột <code>now()</code> bằng tổng số dòng MV, còn không có cột thì chỉ bằng số dòng của ngày đã đổi. Lab chưa chạy bài này; đừng chép số từ bất kỳ đâu.</p>
</details>

## 15. Đọc tiếp

- [PostgreSQL 18 · Materialized Views](https://www.postgresql.org/docs/18/rules-materializedviews.html): khái niệm và ví dụ dùng MV cho báo cáo.
- [PostgreSQL 18 · REFRESH MATERIALIZED VIEW](https://www.postgresql.org/docs/18/sql-refreshmaterializedview.html): `CONCURRENTLY`, điều kiện unique index, `WITH NO DATA`, mỗi lúc một lần refresh.
- [PostgreSQL 18 · Explicit Locking](https://www.postgresql.org/docs/18/explicit-locking.html): bảng xung đột khoá, `ACCESS EXCLUSIVE` và `EXCLUSIVE`.
- [PostgreSQL 18 · CREATE MATERIALIZED VIEW](https://www.postgresql.org/docs/18/sql-creatematerializedview.html) và [Date/Time Functions](https://www.postgresql.org/docs/18/functions-datetime.html) (ý nghĩa của `now()`).
- Trong chuỗi: [PG-033 · Khoá dòng, khoá bảng](/articles/postgresql-lock-row-table), [PG-043 · pg_stat_activity](/articles/postgresql-pg-stat-activity-quan-sat), [PG-060 · Capstone](/articles/postgresql-capstone-van-hanh-toi-noi).
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-056 và PG-060), `p06/fixture.sql`, kết quả `p06/OUTPUT.txt`, sổ cước theo bước và giới hạn trong `p06/README.md`, cổng go/no-go trong `p06/HO-SO-BAN-GIAO.md`.

Bài tiếp theo: [PG-057 · PostGIS tìm xe gần, pgvector tìm ghi chú tương tự](/articles/postgresql-extension-postgis-pgvector).

---
slug: postgresql-pg-stat-activity-quan-sat
title: "Quan sát phiên làm việc và truy vấn chậm: đọc pg_stat_activity khi API đứng"
summary: "Tối thứ Sáu, API hoàn tiền của Tới Nơi treo dù CPU database gần như rảnh. Đọc pg_stat_activity, sự kiện chờ, pg_blocking_pids và pg_locks trên PostgreSQL 18.1 thật; kiểm tra ai được xem câu SQL; giới hạn thời gian chờ bằng lock_timeout và idle_in_transaction_session_timeout; biết khi nào một ảnh chụp tức thời không đủ."
category: postgresql
level: beginner
tags: [postgresql, pg-043, monitoring, pg-stat-activity, wait-event, lock-timeout, observability, toi-noi]
language: vi
---

19:40 tối thứ Sáu, giờ cao điểm đặt xe. Kênh trực của Tới Nơi nhận cảnh báo: API hoàn tiền trả `504` sau 30 giây, màn chăm sóc khách hàng quay vòng mãi. Dashboard hạ tầng lại yên bình: CPU máy PostgreSQL khoảng 10 %, đĩa không bận, log ứng dụng không có `ERROR`. Có người đề xuất «nâng cấp máy database cho chắc».

Nâng cấp không giúp gì nếu các request **không làm việc mà đang chờ**. Câu hỏi đầu tiên của người trực là: *mỗi kết nối lúc này đang làm gì, nếu đang chờ thì chờ cái gì, chờ ai?* PostgreSQL trả lời qua view `pg_stat_activity`. Bài này đọc view ấy trên một tình huống tái hiện thật, rồi đi tiếp tới quyền xem câu SQL, các tham số giới hạn thời gian chờ, và những gì một ảnh chụp tức thời **không** nói được về truy vấn chậm.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-043 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-ket-noi-auth-tls">PG-042 · Xác thực và bảo vệ kết nối</a>; cơ chế khoá và cách gỡ hàng chờ ở <a href="/articles/postgresql-lock-row-table">PG-033 · Khoá dòng, khoá bảng và thời gian chờ</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, chạy <code>npm run p05</code> trên PostgreSQL 18.1 thật. Sau bài này, bạn đọc được trạng thái, sự kiện chờ và tuổi giao dịch của từng phiên, chỉ ra phiên chặn dựa trên bằng chứng, và biết cần thêm dữ liệu gì trước khi kết luận «query chậm».</p>
</div>

## 1. Dữ liệu và cách chạy

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Runner `p05/run.mjs` dựng một cluster PostgreSQL 18.1 tạm chỉ nghe localhost, nạp `p05/00-schema.sql`, xong thì dọn. Bảng `ops.trips` có 100 chuyến, mỗi chuyến 30.000 đ. Role liên quan: `tn_owner` sở hữu bảng, `tn_app` là ứng dụng (đọc, thêm, sửa), `tn_report` chỉ đọc bảng, `tn_monitor` được cấp role dựng sẵn `pg_monitor`.

Mỗi **phiên** (session) là một kết nối psql riêng, tức một tiến trình backend riêng trên server. Khối PG-043 mở bốn phiên: A giữ khoá, B bị chặn, C thử `lock_timeout`, và một phiên «ngồi yên trong giao dịch». Kết quả chép từ `p05/OUTPUT.txt`; runner thay PID bằng `<PID A>`, việc thay chuỗi làm lệch căn cột psql một chút.

## 2. `pg_stat_activity`: mỗi dòng là một tiến trình

Theo [tài liệu thống kê](https://www.postgresql.org/docs/18/monitoring-stats.html), view có **một dòng cho mỗi tiến trình server** — cả kết nối ứng dụng lẫn tiến trình nền như autovacuum launcher, checkpointer. Cột `backend_type` phân biệt chúng; khi trực sự cố, ta thường lọc `backend_type = 'client backend'`. Các cột hay dùng:

| Cột | Ý nghĩa |
|---|---|
| `pid` | Mã tiến trình; nối sang `pg_locks`, `pg_blocking_pids()` |
| `usename`, `application_name`, `client_addr` | Ai, service nào (tên client tự đặt), từ đâu |
| `xact_start` | Lúc giao dịch hiện tại bắt đầu; `NULL` nếu không trong giao dịch |
| `query_start`, `state_change` | Lúc câu lệnh (đang chạy hoặc câu cuối đã chạy) bắt đầu; lúc `state` đổi lần cuối |
| `state` | `active`, `idle`, `idle in transaction`, … |
| `wait_event_type`, `wait_event` | Nếu đang chờ: loại gì, sự kiện nào |
| `backend_xmin` | Xmin horizon của snapshot mà phiên đang giữ — quan trọng cho VACUUM ở PG-044 |
| `query` | Câu lệnh gần nhất |

Ba điểm dễ đọc sai:

- **`query` là câu gần nhất, không phải câu đang chạy.** Tài liệu nói rõ: khi `state = 'active'` đó là câu đang thực thi; ở mọi trạng thái khác, đó là câu **đã xong** cuối cùng. Phiên `idle in transaction` hiện `UPDATE …` nghĩa là UPDATE đã chạy xong và giao dịch còn mở.
- **Câu lệnh bị cắt** theo `track_activity_query_size` (mặc định 1024 byte). SQL do ORM sinh thường dài hơn.
- **Ảnh chụp, không phải lịch sử.** Ngoài ra, nếu đọc view thống kê trong một giao dịch thì thông tin hoạt động được chụp một lần và giữ tới hết giao dịch (tài liệu mô tả kèm hàm `pg_stat_clear_snapshot()`). Vòng lặp giám sát chạy trong một giao dịch dài sẽ thấy mãi một bức ảnh cũ.

Ba mốc thời gian, ba câu hỏi: `now() - xact_start` là **tuổi giao dịch** (đã giữ khoá và snapshot bao lâu), `now() - query_start` là tuổi câu lệnh, `now() - state_change` là thời gian ở trạng thái hiện tại — với `idle in transaction`, đó là thời gian **ngồi yên** trong giao dịch.

## 3. `state` và `wait_event`: hai câu hỏi khác nhau

`state` trả lời «backend có đang thực thi câu lệnh không»: `active` (đang chạy), `idle` (chờ lệnh mới, ngoài giao dịch), `idle in transaction` (trong giao dịch, không chạy gì), `idle in transaction (aborted)` (giao dịch đã có câu lỗi, chỉ còn ROLLBACK được); hiếm gặp hơn là `fastpath function call` và `disabled` (khi `track_activities` tắt).

`wait_event_type`/`wait_event` trả lời câu thứ hai: **nếu đang chờ, thì chờ gì**. PostgreSQL 18 nhóm sự kiện chờ thành: `Activity` (tiến trình nền chờ việc), `BufferPin`, `Client` (chờ client), `Extension`, `InjectionPoint`, `IO` (đọc/ghi), `IPC` (chờ tiến trình khác), `Lock` (khoá nặng: bảng, dòng qua mã giao dịch, advisory…), `LWLock` (khoá nhẹ bảo vệ bộ nhớ dùng chung), `Timeout`. Từ PostgreSQL 17 có view `pg_wait_events` liệt kê mọi sự kiện kèm mô tả — tra ở đó thay vì đoán.

Hai cột độc lập. Phiên `active` vẫn có thể chờ `Lock`: đã nhận câu lệnh nhưng không làm được gì. Phiên `idle in transaction` thường hiện `Client: ClientRead`: chờ ứng dụng gửi câu tiếp, trong khi vẫn **giữ mọi khoá** đã lấy. Phiên `active` có `wait_event` là `NULL` tức là không ở trong sự kiện chờ nào được theo dõi — thường là đang dùng CPU, nhưng đó chỉ có nghĩa là «không chờ thứ gì mà PostgreSQL đo được», chưa phải bằng chứng.

Với sự cố 19:40, «CPU 10 % mà API treo» thường có lời giải ngay ở đây: rất nhiều phiên `active` với `wait_event_type = 'Lock'`.

## 4. Lab: chụp lại người giữ khoá và người chờ

PG-033 đã giải thích vì sao UPDATE cùng dòng phải chờ; ở đây ta tập trung vào **quan sát**:

| Bước | Phiên A | Phiên B | Runner |
|---|---|---|---|
| 1 | `BEGIN; UPDATE ops.trips SET fare=fare WHERE id=1;` | — | A giữ khoá dòng id=1 |
| 2 | (không gửi gì thêm) | Cùng UPDATE, gửi bất đồng bộ | — |
| 3 | | ⏳ | lặp tới khi `pg_blocking_pids(B)` chứa PID A |
| 4 | | ⏳ | đọc `wait_event_type`, chụp `pg_stat_activity`, `pg_locks` |
| 5 | `ROLLBACK;` | ✔ hoàn thành | B không lỗi |

`SET fare=fare` không đổi giá trị nhưng vẫn lấy khoá dòng. Ở bước 3, runner **không** `sleep` cố định rồi đoán; nó hỏi catalog mỗi 100 ms (tối đa 15 giây) cho tới khi quan hệ chặn hiện ra, rồi mới kiểm tra:

```text
PASS 043 identifies lock wait: Lock
```

Ảnh chụp ở bước 4 (cột `s` là nhãn phiên runner gán từ PID):

```text
--- 043 pg_stat_activity
   s |        state        | wtype  |  wait_event   | blocked_by |                   query
  ---+---------------------+--------+---------------+------------+--------------------------------------------
   A | idle in transaction | Client | ClientRead    | {}         | UPDATE ops.trips SET fare=fare WHERE id=1;
   B | active              | Lock   | transactionid | {<PID A>}    | UPDATE ops.trips SET fare=fare WHERE id=1;
  (2 rows)
```

- **A** `idle in transaction`, chờ `Client: ClientRead` — không chạy gì, đợi ứng dụng. `query` là câu **cuối** đã xong. Không ai chặn A: A là **gốc**.
- **B** `active`, chờ `Lock: transactionid`, bị chặn bởi `<PID A>`. Cùng chuỗi `query`, nhưng ở B đó là câu **đang** chạy.

Chỉ nhìn cột `query`, hai dòng giống hệt. Thứ phân biệt người gây ra và người chịu ảnh hưởng là `state`, `wait_event_type` và `pg_blocking_pids()`.

Vì sao chờ `transactionid` mà không phải tên bảng? Khoá dòng được ghi trên chính dòng, không nằm trong bảng khoá dùng chung; phiên muốn sửa dòng chờ **giao dịch** đang giữ nó kết thúc:

```text
--- 043 pg_locks not granted
   s |   locktype    |   mode    | granted
  ---+---------------+-----------+---------
   B | transactionid | ShareLock | f
  (1 row)
```

B xin `ShareLock` trên mã giao dịch của A, chưa được cấp. A `ROLLBACK` thì B hoàn thành.

<div class="tkh-callout">
<span class="tkh-callout-title">Dùng pg_blocking_pids(), đừng tự nối pg_locks</span>
<p>Tự nối <code>pg_locks</code> đúng ở trường hợp đơn giản này nhưng bỏ sót thứ tự hàng chờ và «chặn mềm». <a href="https://www.postgresql.org/docs/18/functions-info.html">System Information Functions</a> khuyên dùng <code>pg_blocking_pids()</code>; hàm này cần truy cập độc quyền bảng khoá trong chốc lát, nên gọi khi chẩn đoán, không gọi mỗi giây cho mọi phiên. PG-033 có truy vấn tìm gốc của chuỗi chờ dài.</p>
</div>

## 5. Ai được xem câu SQL của người khác

`query` có thể chứa số điện thoại khách, địa chỉ đón, token dán nhầm. Theo tài liệu, superuser và role có quyền của `pg_read_all_stats` thấy đầy đủ mọi phiên; role khác chỉ thấy đầy đủ phiên **của chính mình**, còn với phiên khác thì `query` hiện `<insufficient privilege>`. `pg_monitor` gồm `pg_read_all_settings`, `pg_read_all_stats` và `pg_stat_scan_tables` ([Predefined Roles](https://www.postgresql.org/docs/18/predefined-roles.html)). Lab kiểm tra cả hai phía trên phiên B:

```text
PASS 043 tn_monitor sees query text: t
PASS 043 tn_report does not see query text: <insufficient privilege>
```

`tn_report` đọc được toàn bộ bảng `ops.trips` nhưng không đọc được câu của B. Quyền đọc **dữ liệu** và quyền đọc **hoạt động của người khác** là hai quyền tách biệt nhau. Với Tới Nơi: một role giám sát riêng có `pg_monitor`, không cấp cho `tn_app`; ứng dụng dùng bind parameter (`$1`, `$2`) để `query` chứa placeholder thay vì giá trị; không đưa nguyên văn `query` lên dashboard ai cũng xem.

## 6. Truy vấn trực ca

Lab chạy bản rút gọn chỉ cho hai PID. Lab **không** chạy nguyên văn bản mở rộng dưới đây; các cột đều có trong tài liệu PostgreSQL 18:

```sql
SELECT pid, usename, application_name, client_addr, state,
       wait_event_type || ':' || wait_event        AS waiting_on,
       pg_blocking_pids(pid)                      AS blocked_by,
       date_trunc('second', now() - xact_start)   AS xact_age,
       date_trunc('second', now() - state_change) AS in_state_for,
       left(query, 120)                           AS query_sample
FROM pg_stat_activity
WHERE backend_type = 'client backend'
  AND pid <> pg_backend_pid()
  AND state <> 'idle'
ORDER BY xact_start NULLS LAST;
```

`ORDER BY xact_start` đưa giao dịch già nhất lên đầu: phiên gốc thường là giao dịch mở lâu nhất, không phải câu chạy lâu nhất. Bỏ `idle` vì phiên rảnh ngoài giao dịch không giữ khoá — nhưng khi điều tra **cạn kết nối** (PG-047) thì lại phải đếm cả chúng. Bản tổng hợp nhanh:

```sql
SELECT state, wait_event_type, count(*)
FROM pg_stat_activity
WHERE backend_type = 'client backend'
GROUP BY 1, 2 ORDER BY 3 DESC;
```

40 phiên `active` mà 37 phiên `Lock` cho thấy một hàng chờ khoá, không phải server thiếu CPU.

`application_name` chỉ có ích khi ứng dụng đặt nó trong chuỗi kết nối (`application_name=refund-api`). Lab làm đúng điều này: mỗi phiên psql của runner mang tên `lab-<nhãn>`. Không có nó, `pid` chỉ là một con số và bạn phải dò ngược từ `client_addr` qua NAT, pod, container.

## 7. Đặt trần cho thời gian chờ

`pg_stat_activity` tìm ra phiên gây tắc; còn cấu hình dưới đây giúp **giới hạn thiệt hại** khi lần sau có người quên đóng giao dịch. PG-033 đã so sánh bốn loại timeout; lab P05 kiểm tra lại hai loại liên quan trực tiếp.

**`lock_timeout`** giới hạn **mỗi lần** xin khoá. Trong lúc A vẫn giữ dòng id=1, phiên C chạy:

```sql
SET lock_timeout = '200ms';
UPDATE ops.trips SET fare=fare WHERE id=1;
```

```text
PASS 043 lock_timeout SQLSTATE: 55P03
--- 043 lock_timeout error
  ERROR:  55P03: canceling statement due to lock timeout
```

C không treo theo B: sau 200 ms nhận `55P03` (`lock_not_available`), phiên vẫn sống, ứng dụng trả «hệ thống bận» thay vì giữ request 30 giây. Nếu câu nằm trong giao dịch, giao dịch đã hỏng và phải ROLLBACK (PG-037).

**`idle_in_transaction_session_timeout`** nhắm thẳng vào kiểu phiên A: ngồi yên **trong giao dịch** quá hạn thì server cắt kết nối, giao dịch bị huỷ, khoá được nhả. Runner mở phiên mới:

```sql
SET idle_in_transaction_session_timeout = '300ms';
BEGIN;
SELECT count(*) FROM ops.trips;
-- rồi không gửi gì nữa
```

Runner đợi tới khi PID này biến khỏi `pg_stat_activity`, rồi đọc nhật ký server (đã bỏ tiền tố thời gian/PID):

```text
PASS 043 idle_in_transaction_session_timeout ended backend: gone
--- 043 server log: idle timeout
  FATAL:  terminating connection due to idle-in-transaction timeout
```

`FATAL` nghĩa là **kết nối** kết thúc, không chỉ câu lệnh. Theo bảng mã lỗi, SQLSTATE là `25P03`; lab chỉ bắt dòng log, không in mã. Ứng dụng thấy lỗi ở lần gửi câu tiếp, và pool phải loại kết nối đã chết (PG-047).

Hai tham số còn lại ([Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html)): `statement_timeout` giới hạn tổng thời gian một câu, kể cả lúc chờ khoá (`57014`, kiểm ở PG-047); `transaction_timeout` (từ PostgreSQL 17) giới hạn tổng thời gian giao dịch kể cả khi đang bận, hết hạn thì cắt kết nối (thí nghiệm ở PG-033). Tất cả mặc định `0` — lab in ra điều này ở khối PG-047. Tài liệu khuyên không đặt `statement_timeout`, `lock_timeout`, `transaction_timeout` trong `postgresql.conf` vì ảnh hưởng mọi phiên. Đặt theo role là cách thông dụng (không chạy trong lab, con số chỉ minh hoạ cú pháp):

```sql
ALTER ROLE tn_app SET idle_in_transaction_session_timeout = '30s';
ALTER ROLE tn_app SET lock_timeout = '2s';
```

Thiết lập theo role áp dụng cho **kết nối mới**; kết nối đang nằm trong pool giữ giá trị cũ tới khi được mở lại.

## 8. Khi không phải chờ khoá: truy vấn chậm thật

Nếu ảnh chụp cho thấy nhiều phiên `active` với `wait_event` `NULL` hoặc loại `IO`, tuổi câu lệnh dài và **không** ai bị chặn, thì có thể câu lệnh chậm thật: thiếu index, query plan sai do thống kê cũ, sort tràn đĩa (Phần 3).

Khi đó `pg_stat_activity` chỉ cho **vài mẫu** ở một thời điểm. Năm request đang chạy lúc 19:41 không đại diện cho phân bố độ trễ cả buổi tối; câu 5 ms chạy 10.000 lần mỗi phút có thể tốn hơn câu 3 giây chạy một lần, mà câu nhanh hiếm khi lọt vào ảnh chụp. Cần nguồn có **lịch sử**. Các mục dưới là cấu hình để bạn tự thử; **lab P05 không bật và không chạy chúng**, nên bài không đưa kết quả:

- **`log_min_duration_statement`** ([Error Reporting and Logging](https://www.postgresql.org/docs/18/runtime-config-logging.html)): ghi log mọi câu chạy lâu hơn ngưỡng (mặc định `-1`, tắt).
- **`log_lock_waits`**: ghi log khi phiên chờ khoá lâu hơn `deadlock_timeout` — bằng chứng hàng chờ sau khi sự cố đã qua. Lab Phần 4 ghi nhận tham số này mặc định `off` trên 18.1 (PG-037).
- **`pg_stat_statements`** ([tài liệu](https://www.postgresql.org/docs/18/pgstatstatements.html)): thống kê tích luỹ theo dạng chuẩn hoá của câu lệnh — số lần gọi, tổng và trung bình thời gian, block đọc. Cần `shared_preload_libraries`, khởi động lại, rồi `CREATE EXTENSION`. Cột `query_id` của `pg_stat_activity` khớp `queryid` của nó khi `compute_query_id` bật (mặc định `auto` bật khi module được nạp).

```sql
-- Không chạy trong lab: cần pg_stat_statements đã nạp.
SELECT calls, round(total_exec_time) AS total_ms,
       round(mean_exec_time, 1) AS mean_ms, left(query, 80) AS query
FROM pg_stat_statements
ORDER BY total_exec_time DESC
LIMIT 10;
```

- **`auto_explain`** ([tài liệu](https://www.postgresql.org/docs/18/auto-explain.html)): tự ghi query plan của câu chạy quá `auto_explain.log_min_duration`; bật `log_analyze` có thời gian thật từng nút nhưng thêm overhead cho mọi câu.

Khi đã có câu nghi vấn thì quay về Phần 3: `EXPLAIN (ANALYZE, BUFFERS)` trên bản sao dữ liệu ([PG-021](/articles/postgresql-explain-ke-hoach-thuc-thi)), kiểm tra thống kê ([PG-027](/articles/postgresql-statistics-analyze-cardinality)).

## 9. Từ PID tới hành động

1. **Tổng hợp** `state` × `wait_event_type`. Nhiều `Lock` → hàng chờ khoá. Ứng dụng báo hết kết nối → PG-047. Nhiều `active` không chờ → mục 8.
2. **Tìm gốc** bằng `pg_blocking_pids()` (truy vấn ở PG-033). Ghi lại `pid`, `usename`, `application_name`, `xact_age`, `in_state_for`, `query_sample` **trước khi** làm gì — cắt phiên rồi thì bằng chứng mất.
3. **Nối PID với người chịu trách nhiệm**: `application_name` → service → nhóm. Giao dịch này đáng lẽ dài bao lâu? Migration đã báo trước khác hẳn request quên COMMIT.
4. **Gỡ có chủ đích**: `pg_cancel_backend(pid)` huỷ câu đang chạy, `pg_terminate_backend(pid)` cắt kết nối và huỷ giao dịch. Phiên `idle in transaction` không chạy câu nào để huỷ; chỉ terminate mới nhả khoá. PG-033 mục 7 cho thấy hậu quả của việc gỡ sai chỗ.
5. **Sửa lâu dài**: rút ngắn giao dịch (không gọi HTTP ra ngoài, không chờ người dùng khi đang giữ giao dịch), timeout theo role, cảnh báo khi `idle in transaction` quá N giây.

Đừng tự động kill mọi phiên có câu lệnh chạy lâu. Báo cáo đối soát chạy 5 phút có thể hợp lệ và không chặn ai; giao dịch ngồi yên 40 giây giữ khoá chuyến đang tranh chấp mới là vấn đề. Tiêu chí phải dựa trên **quan hệ chặn và trạng thái**, không chỉ thời gian.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| Hai phiên cùng câu UPDATE, kill nhầm phiên đang chờ | Đọc `query` mà bỏ qua `state` | `active` + `Lock` là nạn nhân; `idle in transaction` thường là gốc |
| «Phiên idle đang chạy câu DELETE này» | `query` của phiên không active là câu **đã xong** | Đọc `state` trước, `query` sau |
| Công cụ giám sát chỉ thấy `<insufficient privilege>` | Thiếu `pg_read_all_stats` | Cấp `pg_monitor` cho role giám sát, không cho ứng dụng |
| Vòng lặp giám sát thấy mãi một trạng thái | Đọc trong một giao dịch dài, ảnh chụp bị giữ | Đọc ở chế độ tự commit hoặc `pg_stat_clear_snapshot()` |
| Không biết PID thuộc service nào | Không đặt `application_name` | Đặt trong chuỗi kết nối mỗi service |
| `pg_cancel_backend` không gỡ được tắc | Gốc là `idle in transaction` | `pg_terminate_backend` có kiểm soát, rồi sửa ứng dụng |
| Kết luận «database chậm» từ 5 dòng `active` | Ảnh chụp không phải phân bố | Thêm log câu chậm, `pg_stat_statements`, metric theo thời gian |

## 11. Đánh đổi

- **Quyền giám sát rộng hay hẹp:** `pg_monitor` cho người trực thấy tất cả, kể cả dữ liệu nhạy cảm trong câu SQL; hẹp quá thì lúc sự cố không ai đọc được gì.
- **Timeout chặt hay lỏng:** `lock_timeout` 200 ms làm request thất bại nhanh nhưng cũng cắt migration hợp lệ cần chờ — đặt khác nhau cho role ứng dụng và role migration. `idle_in_transaction_session_timeout` quá ngắn cắt cả luồng hợp lệ có bước xử lý giữa hai câu.
- **Chụp thường xuyên hay thỉnh thoảng:** đọc `pg_stat_activity` rẻ; gọi `pg_blocking_pids()` cho mọi phiên mỗi giây thì không. Chỉ gọi khi đã thấy `Lock`.
- **Log nhiều hay ít:** ngưỡng `log_min_duration_statement` thấp và `auto_explain.log_analyze` cho nhiều dữ liệu, đổi lại dung lượng log và overhead.

## 12. Bài tập

**Bài 1.** Chạy lại lịch ở mục 4 bằng ba cửa sổ psql trên cluster của bạn (A, B và người trực). Chụp bằng truy vấn ở mục 6, chờ 30 giây, chụp lại, rồi cho A `COMMIT`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>(1) Ở cả hai ảnh chụp, A là <code>idle in transaction</code> với <code>Client:ClientRead</code>, B là <code>active</code> với <code>Lock:transactionid</code> và <code>blocked_by</code> chứa đúng PID của A — kết luận dựa trên quan hệ chặn, không dựa trên cột <code>query</code>. (2) Giữa hai ảnh, <code>xact_age</code> và <code>in_state_for</code> của A tăng khoảng 30 giây trong khi <code>query_start</code> không đổi. (3) Sau COMMIT, B hoàn thành và ảnh chụp thứ ba không còn ai bị chặn. Ghi PID và thời điểm mỗi ảnh.</p>
</details>

**Bài 2.** Tạo role `tn_oncall` đọc được câu SQL của mọi phiên nhưng **không** đọc được `ops.trips` và không cắt được phiên của người khác.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Một hướng: <code>CREATE ROLE tn_oncall LOGIN; GRANT pg_monitor TO tn_oncall;</code>, không cấp gì trên schema <code>ops</code>. Bằng chứng: dưới <code>tn_oncall</code>, <code>query</code> của một phiên khác là câu thật (đối chiếu <code>PASS 043 tn_monitor sees query text</code>); <code>SELECT count(*) FROM ops.trips</code> bị từ chối <code>42501</code>; <code>pg_terminate_backend</code> trên phiên của role khác bị từ chối. Quyền gửi tín hiệu tới phiên khác thuộc <code>pg_signal_backend</code>, không nằm trong <code>pg_monitor</code>.</p>
</details>

**Bài 3.** API hoàn tiền mở giao dịch, UPDATE chuyến, rồi gọi API ngân hàng (có khi 20 giây) trước khi COMMIT. Đề xuất thay đổi code và timeout theo role; chứng minh bằng thí nghiệm rằng phiên bị bỏ quên không giữ khoá quá hạn bạn chọn.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Sửa gốc: gọi ngân hàng <em>ngoài</em> giao dịch (ghi «đang hoàn tiền» và commit, gọi API, mở giao dịch thứ hai ghi kết quả — xem <a href="/articles/postgresql-idempotency-outbox-transaction">PG-039</a>). Lưới an toàn: <code>idle_in_transaction_session_timeout</code> cho role ứng dụng. Bằng chứng: phiên thứ hai UPDATE cùng dòng hoàn thành ngay sau thời hạn; nhật ký có dòng <code>terminating connection due to idle-in-transaction timeout</code>; PID phiên ngồi yên biến khỏi <code>pg_stat_activity</code>; ứng dụng coi kết nối bị cắt là lỗi, không phải thành công.</p>
</details>

**Bài 4.** Bật `pg_stat_statements` và `log_min_duration_statement = '100ms'` trên cluster của bạn, chạy `SELECT pg_sleep(0.2)` năm lần và một UPDATE bị chặn 1 giây. Mỗi câu hiện ở nguồn nào?

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Lab P05 không chạy bài này, không có kết quả mẫu. Bản nộp gồm: bằng chứng module đã nạp (<code>SHOW shared_preload_libraries</code>); dòng <code>pg_stat_statements</code> của <code>pg_sleep</code> với <code>calls</code> đúng số lần chạy; các dòng log <code>duration:</code> tương ứng; và nhận xét rằng UPDATE bị chặn cũng có thời gian dài ở cả hai nguồn dù không «chạy chậm» — chỉ <code>wait_event</code> hoặc <code>log_lock_waits</code> mới tách được thời gian chờ khoá.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · The Cumulative Statistics System](https://www.postgresql.org/docs/18/monitoring-stats.html) — `pg_stat_activity`, trạng thái, sự kiện chờ; [pg_wait_events](https://www.postgresql.org/docs/18/view-pg-wait-events.html); [pg_locks](https://www.postgresql.org/docs/18/view-pg-locks.html).
- [System Information Functions](https://www.postgresql.org/docs/18/functions-info.html) — `pg_blocking_pids()`; [Server Signaling Functions](https://www.postgresql.org/docs/18/functions-admin.html) — `pg_cancel_backend`, `pg_terminate_backend`.
- [Predefined Roles](https://www.postgresql.org/docs/18/predefined-roles.html) — `pg_monitor`, `pg_read_all_stats`, `pg_signal_backend`.
- [Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html) — bốn loại timeout; [Error Codes](https://www.postgresql.org/docs/18/errcodes-appendix.html) — `55P03`, `57014`, `25P03`.
- Trong series: [PG-033 · Khoá dòng, khoá bảng](/articles/postgresql-lock-row-table), [PG-037 · Deadlock và thứ tự khoá](/articles/postgresql-deadlock-lock-order-timeout), [PG-041 · Role và quyền tối thiểu](/articles/postgresql-role-grant-least-privilege).
- Lab: `p05/run.mjs` (khối PG-043), `p05/OUTPUT.txt`, `p05/RUNBOOK.md`.

Bài tiếp theo: [PG-044 · VACUUM, autovacuum và bloat](/articles/postgresql-vacuum-autovacuum-bloat) — cột `backend_xmin` ở mục 2 sẽ giải thích vì sao một giao dịch ngồi yên còn làm VACUUM không dọn được dòng chết.

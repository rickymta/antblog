---
slug: postgresql-connection-pool-timeout
title: "Connection pool, giới hạn kết nối và timeout: request đang chờ ở đâu?"
summary: "Tối khuyến mãi, API đặt xe của Tới Nơi chậm 8 giây trong khi câu SQL chỉ mất vài chục mili giây. Lập ngân sách max_connections, giới hạn theo role, đo hàng chờ của pool với 6 việc trên 2 kết nối thật, kiểm tra statement_timeout với mã 57014, và phân biệt application pool với transaction pooling."
category: postgresql
level: intermediate
tags: [postgresql, pg-047, connection-pool, max-connections, statement-timeout, pgbouncer, capacity, toi-noi]
language: vi
---

Tối thứ Sáu, Tới Nơi chạy khuyến mãi «giảm 30.000 đ cho chuyến sau 20:00». Lượng đặt xe tăng gấp ba trong mười phút. Autoscaler thêm pod cho API đặt xe, nhưng độ trễ không giảm mà tăng: p99 lên 8 giây, log bắt đầu có lỗi «timeout acquiring connection» từ thư viện pool, và một pod mới báo không kết nối được tới PostgreSQL. Dashboard database lại cho thấy câu SQL đặt xe vẫn chỉ mất trung bình vài chục mili giây.

Không câu lệnh nào chậm. Request chậm vì phải **xếp hàng chờ kết nối**, và mỗi pod mới mang theo một pool riêng, khiến tổng số kết nối chạm trần của server. Bài này tách thời gian chờ khỏi thời gian chạy SQL, lập ngân sách kết nối, đo một hàng chờ nhỏ trên kết nối PostgreSQL thật, và đặt timeout cho từng lớp để request thất bại nhanh, có kiểm soát, thay vì treo.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-047 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-pg-dump-restore">PG-046 · Backup logic và khôi phục với pg_dump</a> và <a href="/articles/postgresql-pg-stat-activity-quan-sat">PG-043 · Quan sát phiên làm việc</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, chạy <code>npm run p05</code> trên PostgreSQL 18.1 thật. Sau bài này, bạn lập được ngân sách kết nối cho nhiều service, tính được thời gian chờ của pool từ số kết nối và thời gian giữ kết nối, và chọn timeout cho từng lớp.</p>
</div>

## 1. Dữ liệu và cách chạy

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Runner `p05/run.mjs` dựng cluster PostgreSQL 18.1 tạm chỉ nghe localhost, có bảng `ops.trips` 100 chuyến (mỗi chuyến 30.000 đ) và role ứng dụng `tn_app` đăng nhập bằng mật khẩu qua TLS (PG-042). Khối PG-047: (1) in tham số kết nối và timeout của cluster; (2) mở **hai** phiên psql làm «pool», đẩy **sáu** việc `SELECT pg_sleep(0.05)` vào hàng, đo thời gian mỗi việc chờ kết nối; (3) kiểm `statement_timeout` và `CONNECTION LIMIT` theo role. Kết quả chép từ `p05/OUTPUT.txt` (số cổng thay bằng `<PORT>`). Đọc mục «Phương pháp đo» trong `p05/README.md` trước khi dùng con số ở mục 5 cho bất kỳ kết luận nào.

## 2. Một request đi qua mấy hàng chờ?

**Connection pool** (bể kết nối) là tập kết nối database mở sẵn và dùng lại: request **mượn** (acquire) một kết nối, chạy SQL, rồi **trả** (release). Khi mọi kết nối đang bị mượn, request mới **xếp hàng** trong pool. Một request đặt xe đi qua:

| Chặng | Ở đâu | Giới hạn bởi |
|---|---|---|
| 1. Chờ mượn kết nối | Pool trong tiến trình ứng dụng | Acquire timeout của thư viện pool |
| 2. Mở kết nối mới (nếu cần) | Mạng, TLS, xác thực, tạo backend | `connect_timeout` phía client |
| 3. Chờ khoá | Trong PostgreSQL | `lock_timeout` |
| 4. Thực thi | Trong PostgreSQL | `statement_timeout` (gồm cả chặng 3) |
| 5. Giữ kết nối giữa các câu | Code ứng dụng trong giao dịch | `idle_in_transaction_session_timeout`, `transaction_timeout` |

Điểm mấu chốt: **`statement_timeout` chỉ đếm thời gian phía server**, từ lúc câu lệnh tới server ([Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html)). Request chờ 7 giây ở chặng 1 rồi mới chạy SQL 40 ms thì không bao giờ chạm `statement_timeout = 5s`. Database thấy mọi thứ ổn; người dùng thấy hơn 7 giây. Khi ai đó báo «database timeout», câu hỏi đầu tiên là: ở chặng nào?

Chặng 3–5 nhìn được qua `pg_stat_activity` (PG-043); chặng 1 thì PostgreSQL **không thấy** — chỉ metric của thư viện pool (số request chờ, thời gian acquire) mới cho thấy chặng này.

## 3. Kết nối là tài nguyên có trần

PostgreSQL dùng **một tiến trình backend cho mỗi kết nối** ([Architectural Fundamentals](https://www.postgresql.org/docs/18/tutorial-arch.html)): mỗi kết nối có bộ nhớ riêng, và mở kết nối phải tạo tiến trình, bắt tay TLS, xác thực. Trần này do các tham số sau quy định ([Connections and Authentication](https://www.postgresql.org/docs/18/runtime-config-connection.html)):

- **`max_connections`**: tổng số kết nối đồng thời (mặc định 100; đổi cần khởi động lại server).
- **`superuser_reserved_connections`**: chỗ giữ riêng cho superuser (mặc định 3).
- **`reserved_connections`** (từ PostgreSQL 16): chỗ giữ cho role có quyền của `pg_use_reserved_connections` (mặc định 0) — để người trực không phải superuser vẫn vào được khi ứng dụng đã chiếm hết.

Cluster lab dùng `max_connections = 40` (cấu hình chung của lab, không phải khuyến nghị):

```text
--- 047 connection settings
                  name                 | setting
  -------------------------------------+---------
   idle_in_transaction_session_timeout | 0
   lock_timeout                        | 0
   max_connections                     | 40
   reserved_connections                | 0
   statement_timeout                   | 0
   superuser_reserved_connections      | 3
  (6 rows)
```

Role thường của lab dùng được tối đa 40 − 3 − 0 = **37** kết nối. Theo tài liệu, khi số chỗ còn lại chỉ bằng phần giữ riêng, kết nối mới của role thường bị từ chối; lỗi này có SQLSTATE `53300` (`too_many_connections`). Lab **không** thử lấp đầy 37 chỗ, nên bài không trích thông báo của trường hợp này. Bảng cũng xác nhận ba timeout đều **0 — tắt**: nếu bạn không đặt, không ai đặt hộ.

## 4. Ngân sách kết nối

Trần là hữu hạn nên phải chia như chia ngân sách. Ví dụ tự đặt với `max_connections = 100` (minh hoạ, **không** phải khuyến nghị):

| Hạng mục | Kết nối |
|---|---|
| `superuser_reserved_connections` | 3 |
| `reserved_connections` cho role trực ca | 3 |
| API đặt xe: 4 pod × pool 12 | 48 |
| Worker điều phối: 2 × 6 | 12 |
| Báo cáo, đối soát | 6 |
| Migration, giám sát, psql thủ công | 8 |
| **Dư** (deploy rolling: pod cũ và mới cùng sống) | **20** |

Bẫy của tối khuyến mãi: autoscaler đưa API lên 8 pod thì nhu cầu tăng thành 96 + 12 + 6 + 8 = 122, vượt 94 chỗ cho role thường. Pod mới không mở được kết nối; pod cũ vẫn xếp hàng vì vấn đề gốc (thời gian giữ kết nối, mục 6) còn nguyên. **Kích thước pool phải tính trên số bản sao tối đa**, không phải một pod.

`CONNECTION LIMIT` đặt trần theo role hoặc database ([CREATE ROLE](https://www.postgresql.org/docs/18/sql-createrole.html), [CREATE DATABASE](https://www.postgresql.org/docs/18/sql-createdatabase.html)), để một service bị tràn không chiếm hết chỗ của service khác. Lab đặt `CONNECTION LIMIT 1` cho `tn_app`, mở một kết nối giữ `pg_sleep(3)`, đợi `pg_stat_activity` thấy đúng một phiên `tn_app`, rồi mở kết nối thứ hai:

```text
PASS 047 role CONNECTION LIMIT 1 rejects second connection
--- 047 client error: role connection limit
  psql: error: connection to server at "127.0.0.1", port <PORT> failed: FATAL:  too many connections for role "tn_app"
```

Kết nối bị từ chối ngay lúc khởi tạo phiên với `FATAL` — ứng dụng nhận lỗi khi **mở** kết nối, không phải khi chạy SQL. Theo bảng mã lỗi, đây cũng là `53300`. Runner trả lại `CONNECTION LIMIT -1` sau phép thử. Tài liệu lưu ý giới hạn chỉ đếm kết nối thông thường (không đếm prepared transaction, background worker), và với database thì giới hạn chỉ được áp dụng **gần đúng**: hai phiên mở cùng lúc khi chỉ còn một chỗ có thể cùng thất bại.

## 5. Lab: sáu việc, hai kết nối

Mô hình tối giản của pool: hai phiên đã mở sẵn, sáu việc vào hàng **cùng lúc**, worker rảnh thì lấy việc kế tiếp. Mã trong `run.mjs` (định dạng lại):

```javascript
const workers = await Promise.all([session('pool-1'), session('pool-2')]);
const waits = []; let next = 0; const enqueued = performance.now();
await Promise.all(workers.map(async s => {
  while (next < 6) {
    const job = next++;
    waits[job] = +(performance.now() - enqueued).toFixed(1);
    await step(s, 'SELECT pg_sleep(0.05);');
  }
}));
```

`waits[job]` ghi **lúc worker nhận việc**, trước khi gửi SQL: đó là thời gian chờ trong hàng, không gồm thời gian chạy. Kết quả:

```text
MEASURE 047 6 jobs / 2 existing connections / 50ms service; queue waits ms: [0.1,0.1,53.4,53.5,104.7,104.9]
```

Tính tay: `N` = 6 việc, `c` = 2 kết nối, `S` = thời gian một việc giữ kết nối. Khi tất cả đến cùng lúc, việc thứ `k` (đếm từ 0) chờ `⌊k/c⌋` lượt phục vụ:

| Việc `k` | `⌊k/2⌋` | Dự đoán, S = 50 ms | Đo được (ms) |
|---|---|---|---|
| 0, 1 | 0 | 0 | 0,1 · 0,1 |
| 2, 3 | 1 | 50 | 53,4 · 53,5 |
| 4, 5 | 2 | 100 | 104,7 · 104,9 |

Hình dạng khớp: ba bậc, mỗi bậc hai việc. Mỗi bậc dài hơn 50 ms khoảng 2–3 ms vì một lượt thật không chỉ là `pg_sleep`: còn có độ trễ qua pipe của psql, lệnh đánh dấu `PsqlSession` gửi kèm để biết câu đã xong, và việc lập lịch của Node. Thời gian giữ kết nối thực tế S' khoảng 52–53 ms.

Trung bình chờ đo được: 316,7 / 6 ≈ **52,8 ms**. Công thức cho một đợt `N` việc đến cùng lúc (`N` chia hết cho `c`): trung bình = S' × (N/c − 1) / 2 = S' × 1. Với S' ≈ 52,5 ms, khớp. Việc cuối bắt đầu ở 104,9 ms nên xong khoảng 157 ms sau khi vào hàng — con số **suy ra**, lab không đo thời điểm hoàn thành.

Bài học: **thời gian chờ tăng tuyến tính theo vị trí trong hàng** và có thể lớn hơn thời gian chạy SQL nhiều lần — việc 5 chờ gấp đôi thời gian nó dùng database. Với 60 việc đến cùng lúc trên hai kết nối, việc cuối chờ ⌊59/2⌋ = 29 lượt ≈ 1,5 giây (tính tay, không đo), trong khi mỗi câu vẫn 50 ms và `statement_timeout` không hề biết.

Giới hạn (README nêu): đây không phải thư viện pool production — không huỷ việc hết hạn, không có trần độ dài hàng, không kiểm tra sức khoẻ hay thay kết nối hỏng, không tính chi phí mở kết nối. Một lần chạy, một máy Windows, không assert mili giây nào. Đừng dùng 52,8 ms làm số đo năng lực của bất kỳ hệ thống nào.

## 6. Từ mô hình tới hệ thống: định luật Little

Lab là một đợt việc dồn đến cùng lúc. Với request đến đều ở trạng thái ổn định, **định luật Little** nói: số việc trung bình trong hệ thống = tốc độ request đến × thời gian trung bình mỗi việc ở trong hệ thống. Áp vào pool:

> kết nối bận trung bình = request/giây cần database × thời gian giữ kết nối trung bình

Ví dụ minh hoạ (số tự đặt, không phải số đo):

- 200 request/giây, mỗi request giữ kết nối 20 ms → 200 × 0,02 = **4** kết nối bận. 48 kết nối của API là dư dả.
- Một bản sửa gọi API thanh toán (2 giây) **bên trong** giao dịch: giữ 2,02 s → cần 200 × 2,02 ≈ **404** kết nối bận. 48 kết nối chỉ phục vụ được 48 / 2,02 ≈ 24 request/giây. Tốc độ đến 200/giây vượt năng lực 24/giây: hàng chờ **tăng không giới hạn** chừng nào còn tải, acquire timeout lần lượt báo lỗi.

Với lab: hai kết nối, S' ≈ 52,5 ms, năng lực ≈ 2 / 0,0525 ≈ 38 việc/giây (tính từ số đo, lab không chạy tải đều).

Hệ quả, theo thứ tự nên làm:

1. **Giảm thời gian giữ kết nối** — đòn bẩy lớn nhất. Không gọi dịch vụ ngoài trong giao dịch; mượn muộn, trả sớm. PG-043 hướng dẫn cách bắt quả tang phiên `idle in transaction`.
2. **Đừng mặc định tăng pool.** Nếu các kết nối đang tranh CPU, đĩa hay cùng một khoá dòng, thêm kết nối chỉ dời hàng chờ từ pool vào trong PostgreSQL (thấy dưới dạng `wait_event_type` `Lock` hoặc `IO`), và tốn thêm bộ nhớ cho mỗi backend.
3. **Từ chối có kiểm soát** khi quá tải: trần độ dài hàng và acquire timeout, trả «hệ thống bận» nhanh thay vì để request treo.

## 7. Timeout theo từng lớp

Mỗi chặng ở mục 2 cần giới hạn riêng, và các giới hạn phải **lồng nhau**: lớp trong ngắn hơn lớp ngoài, để lỗi xảy ra ở chỗ biết rõ nguyên nhân nhất. Lab kiểm `statement_timeout` trên một kết nối của «pool»:

```sql
SET statement_timeout = '20ms';
SELECT pg_sleep(0.2);
```

```text
PASS 047 statement timeout SQLSTATE: 57014
--- 047 statement_timeout error
  ERROR:  57014: canceling statement due to statement timeout
```

`57014` (`query_canceled`) cũng là mã khi người vận hành gọi `pg_cancel_backend` (PG-033). Ứng dụng nên coi cả hai là «câu bị huỷ, giao dịch phải ROLLBACK». Theo tài liệu, khi nhiều câu nằm trong một chuỗi query đơn giản, giới hạn áp riêng cho từng câu; và không nên đặt `statement_timeout` trong `postgresql.conf` vì ảnh hưởng mọi phiên.

Lưu ý, câu lệnh `SET statement_timeout` trên là thiết lập **mức phiên**, nằm lại trên kết nối. Runner đóng phiên ngay sau đó nên không sao; trong pool thật, request kế tiếp mượn kết nối này sẽ thừa hưởng giới hạn 20 ms mà không biết. Dùng `SET LOCAL` trong giao dịch, hoặc `ALTER ROLE tn_app SET statement_timeout = '…'` (áp cho kết nối mới).

Khung tham khảo cho API đặt xe (giá trị minh hoạ, lab **không** chạy cấu hình này):

| Lớp | Ví dụ | Đặt ở đâu |
|---|---|---|
| Deadline request HTTP | 3 s | Gateway / framework |
| Acquire timeout | 500 ms | Thư viện pool |
| `statement_timeout` | 2 s | Role `tn_app` hoặc `SET LOCAL` |
| `lock_timeout` | 1 s | Nhỏ hơn `statement_timeout` mới có ý nghĩa (PG-037) |
| `idle_in_transaction_session_timeout` | 10 s | Role `tn_app` |
| `transaction_timeout` (PostgreSQL 17+) | 5 s | Role `tn_app`; role migration đặt riêng, dài hơn |

Hai bẫy hay gặp:

- **Pool chờ vô hạn theo mặc định.** Ví dụ node-postgres: `connectionTimeoutMillis` mặc định `0`, tức không giới hạn thời gian chờ. Hãy kiểm tra giá trị mặc định của thư viện bạn dùng; acquire timeout sẽ biến «treo 30 giây» thành «lỗi sau 500 ms».
- **Timeout phía client không dừng câu lệnh phía server.** Deadline HTTP hết thì ứng dụng bỏ request, nhưng câu SQL vẫn chạy và vẫn giữ kết nối, trừ khi driver gửi **yêu cầu huỷ** của giao thức PostgreSQL (như `PQcancel` của libpq). `statement_timeout` phía server là lưới an toàn cho trường hợp này.

## 8. Trả kết nối về pool cho sạch

- **Trả đúng một lần trên mọi nhánh**, kể cả exception và request bị huỷ; nhánh quên trả khiến pool cạn dần dù database rảnh. Dùng `try/finally`.
- **ROLLBACK trước khi trả** nếu giao dịch còn mở hoặc đã lỗi. Sau `57014` hay `55P03` trong giao dịch, mọi câu lệnh tiếp theo đều nhận `25P02` cho tới ROLLBACK (PG-033).
- **Xoá trạng thái phiên** nếu code dùng `SET` không `LOCAL`, bảng tạm, `LISTEN`, advisory lock mức phiên, prepared statement. `DISCARD ALL` ([tài liệu](https://www.postgresql.org/docs/18/sql-discard.html)) đặt lại phần lớn trạng thái này; nhiều pool tự chạy lệnh reset khi trả.
- **Loại kết nối đã chết.** Kết nối bị `idle_in_transaction_session_timeout` hay `pg_terminate_backend` cắt (PG-043) vẫn nằm trong pool như một đối tượng. Pool phải kiểm tra kết nối này và bỏ nó khi gặp lỗi mất kết nối.
- **Ngữ cảnh tenant theo giao dịch.** PG-049 đặt tenant bằng `SET LOCAL`; lab xác nhận thiết lập này chỉ có hiệu lực trong giao dịch (`PASS 049 SET LOCAL scope inside / after transaction: 50/0`). Đặt bằng `SET` mức phiên thì request sau, của tenant khác, có thể mượn đúng kết nối đó.

## 9. Application pool khác transaction pooling

«Pool» đến giờ là pool **trong ứng dụng**: mỗi kết nối gắn với một backend, request mượn trọn một phiên. Khi có quá nhiều tiến trình ứng dụng, người ta thêm một **connection pooler** như PgBouncer: ứng dụng mở nhiều kết nối tới pooler, pooler dồn vào ít kết nối thật tới PostgreSQL.

Lab P05 **không chạy PgBouncer**; phần này mô tả để bạn tự kiểm tra, không có số đo. Theo [tài liệu PgBouncer](https://www.pgbouncer.org/features.html), `pool_mode` có ba chế độ: **session** (client giữ kết nối server tới khi ngắt — như application pool), **transaction** (chỉ giữ trong một giao dịch; giao dịch sau có thể chạy trên backend khác), **statement** (trả sau từng câu, không cho giao dịch nhiều câu).

Ở chế độ transaction, mọi thứ dựa vào trạng thái **phiên** không còn đáng tin: `SET` không `LOCAL` (kể cả `SET statement_timeout`), advisory lock mức phiên, bảng tạm sống qua nhiều giao dịch, `LISTEN`, cursor `WITH HOLD`. Prepared statement cần đọc kỹ: PgBouncer bản gần đây hỗ trợ prepared statement mức giao thức trong chế độ transaction khi bật `max_prepared_statements`, còn `PREPARE` bằng SQL thì không — đừng kết luận «luôn được» hay «luôn hỏng», hãy đối chiếu phiên bản PgBouncer và driver. Timeout và tenant phải đặt bằng `SET LOCAL` hoặc thiết lập theo role/database.

Pooler đổi phép tính ngân sách: `max_connections` chỉ cần chứa pool phía server của pooler cộng các kết nối trực tiếp, còn số client có thể lớn hơn nhiều. Nhưng định luật Little vẫn đúng ở phía server: nếu mỗi giao dịch giữ backend 2 giây, không pooler nào giúp 20 backend phục vụ được 200 giao dịch mỗi giây. Hàng chờ chỉ dời từ ứng dụng sang pooler.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| p99 tăng vọt, SQL vẫn nhanh | Request xếp hàng trong pool | Đo riêng thời gian acquire và thời gian SQL |
| Pod mới không kết nối được sau autoscale | Pool × số pod vượt `max_connections` | Ngân sách theo số pod tối đa; `CONNECTION LIMIT` theo role |
| Request treo tới khi người dùng bỏ đi | Acquire timeout mặc định vô hạn | Đặt acquire timeout và trần độ dài hàng |
| Pool cạn dần dù tải bình thường | Nhánh lỗi không trả kết nối | `try/finally`; theo dõi số kết nối đang mượn khi rảnh |
| `25P02` ở request không liên quan | Kết nối trả về khi giao dịch đã hỏng | ROLLBACK trước khi trả |
| Tăng pool, database chậm hơn | Thêm kết nối tranh cùng tài nguyên | Giảm thời gian giữ trước; xem `wait_event` (PG-043) |
| Không vào được database lúc sự cố | Ứng dụng chiếm hết chỗ | `reserved_connections` cho role trực ca |

## 11. Đánh đổi

- **Pool nhỏ hay lớn:** nhỏ thì xếp hàng ở ứng dụng nhưng database ít tranh chấp; lớn thì hàng ứng dụng ngắn hơn nhưng có thể dời tranh chấp vào PostgreSQL và tốn ngân sách.
- **Acquire timeout ngắn hay dài:** ngắn thì từ chối sớm, nhưng đợt tăng ngắn hạn cũng gây lỗi; dài thì hấp thụ đợt tăng, nhưng khi quá tải thật request treo lâu và người dùng bấm lại khiến tải tệ hơn.
- **Retry:** thử lại sau lỗi acquire hay `57014` làm tăng tải đúng lúc đang quá tải. Retry phải có giới hạn, giãn cách, nằm trong deadline và chỉ cho thao tác idempotent ([PG-039](/articles/postgresql-idempotency-outbox-transaction)).
- **Application pool hay pooler:** pooler giảm số backend, đổi lại thêm một thành phần vận hành, một chặng mạng, và (chế độ transaction) mất tính năng mức phiên.

## 12. Bài tập

**Bài 1.** Sửa mô hình ở mục 5: thêm acquire timeout 80 ms — việc nào chờ quá 80 ms bị từ chối và **không bao giờ chạy**.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Theo số đo, việc 4 và 5 (chờ khoảng 105 ms) bị từ chối, việc 0–3 chạy. Bằng chứng: (1) danh sách việc bị từ chối và thời điểm từ chối; (2) số câu <code>pg_sleep</code> thực sự tới server bằng 4 (đếm phía worker hoặc qua <code>pg_stat_activity</code>) — việc hết hạn không bị chạy muộn; (3) số kết nối tới server luôn là 2; (4) báo cáo tách thời gian chờ và thời gian SQL từng việc, không gộp thành một latency trung bình. Nếu kiểm hạn lúc worker nhận việc thì việc 4 bị từ chối ở khoảng 105 ms chứ không phải 80 ms; hãy giải thích bạn chọn cách nào.</p>
</details>

**Bài 2.** `max_connections = 200`, `superuser_reserved_connections = 3`, `reserved_connections = 5`. API autoscale 4–12 pod; worker 3 pod × 5; báo cáo 10; migration và giám sát 10. Giữ lại 15 % chỗ của role thường làm phần dự phòng. Pool tối đa mỗi pod API là bao nhiêu?

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Role thường: 200 − 3 − 5 = 192. Dự phòng 15 % ≈ 28,8, làm tròn lên 29 → còn 163. Trừ 15 + 10 + 10 → 128 cho API; 128 / 12 pod ≈ 10,7 → tối đa <b>10</b> mỗi pod. Bài nộp phải ghi giả định và cách làm tròn, kiểm tổng ở số pod <em>tối đa</em>, đề xuất <code>CONNECTION LIMIT</code> 120 cho role API, rồi dùng định luật Little với tốc độ request và thời gian giữ kết nối bạn đo được để kiểm 120 kết nối có đủ không.</p>
</details>

**Bài 3.** Một kết nối trong pool chạy `SET statement_timeout = '20ms'` (như lab) rồi được trả về. Viết phép thử chứng minh request sau bị ảnh hưởng, rồi sửa.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Dùng pool một kết nối để request sau chắc chắn mượn đúng kết nối đó. Trước khi sửa: request thứ hai chạy <code>SELECT pg_sleep(0.05)</code> nhận <code>57014</code> dù không đặt timeout; <code>SHOW statement_timeout</code> trả <code>20ms</code>. Sau khi sửa (<code>BEGIN; SET LOCAL statement_timeout = '20ms'; …; COMMIT;</code> hoặc reset khi trả): <code>SHOW</code> trả mặc định và câu 50 ms chạy xong. Ghi cùng PID backend ở cả hai lần để chứng minh là một kết nối.</p>
</details>

## 13. Đọc tiếp

- [Connections and Authentication](https://www.postgresql.org/docs/18/runtime-config-connection.html) — `max_connections`, `superuser_reserved_connections`, `reserved_connections`; [Predefined Roles](https://www.postgresql.org/docs/18/predefined-roles.html) — `pg_use_reserved_connections`.
- [CREATE ROLE](https://www.postgresql.org/docs/18/sql-createrole.html), [CREATE DATABASE](https://www.postgresql.org/docs/18/sql-createdatabase.html) — `CONNECTION LIMIT`.
- [Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html) — `statement_timeout`, `lock_timeout`, `idle_in_transaction_session_timeout`, `transaction_timeout`; [Error Codes](https://www.postgresql.org/docs/18/errcodes-appendix.html) — `53300`, `57014`, `25P02`; [DISCARD](https://www.postgresql.org/docs/18/sql-discard.html).
- [PgBouncer · Features](https://www.pgbouncer.org/features.html) — chế độ pool và tính năng mức phiên.
- Trong series: [PG-043 · Quan sát phiên làm việc](/articles/postgresql-pg-stat-activity-quan-sat), [PG-033 · Khoá và thời gian chờ](/articles/postgresql-lock-row-table), [PG-049 · Row-level security](/articles/postgresql-rls-tenant-security).
- Lab: `p05/run.mjs` (khối PG-047), `p05/OUTPUT.txt`, giới hạn phép đo trong `p05/README.md`.

Bài tiếp theo: [PG-048 · Khôi phục theo thời điểm với base backup và WAL](/articles/postgresql-pitr-base-backup-wal).

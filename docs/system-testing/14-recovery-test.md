# SYS-14 — Recovery Test: phục hồi dịch vụ và dữ liệu sau sự cố

Một dịch vụ trả health check xanh sau sự cố vẫn có thể bỏ sót khoản thu, mất giao dịch mới nhất hoặc phát lại thao tác hai lần. Recovery Test kiểm tra hệ thống trở về trạng thái nghiệp vụ chấp nhận được sau lỗi, trong giới hạn thời gian và dữ liệu đã thống nhất.

Bài trước: [Data Integrity Test](13-data-integrity-test.md). Bài tiếp theo: UAT. [Mã thực hành](../../examples/toi-noi-testing/articles/system-testing-recovery-test/README.md) · [Báo cáo thực chạy](14-test-report.md).

## 1. Mục tiêu và chuẩn bị

- Phân biệt retry, restart, restore, failover và phục hồi nghiệp vụ.
- Định nghĩa RTO/RPO cùng cách đo, không suy từ thời gian chạy test.
- Dựng một chu trình bình thường → lỗi → gỡ lỗi → xác minh dữ liệu và công việc mới.
- Phát hiện tình huống đã thu tiền nhưng chưa xác nhận, tránh retry mù.
- Thực hành logical backup/restore trên PostgreSQL riêng, ghi nhận dữ liệu nằm ngoài snapshot.

Cần biết transaction, idempotency và SQL từ SYS-04/SYS-13. Phần API cần Node.js và dependency lab. Phần restore cần PowerShell trên Windows và các binary PostgreSQL trên PATH: initdb, pg_ctl, createdb, psql, pg_dump, pg_restore. Hai phần độc lập; lab API vẫn dùng PGlite, không kết nối cluster PostgreSQL của bài restore.

## 2. Khái niệm: phục hồi cái gì?

| Cơ chế | Mục tiêu | Điều chưa được chứng minh chỉ bằng cơ chế đó |
| --- | --- | --- |
| Retry | Thực hiện lại thao tác sau lỗi tạm thời | Không tạo tác dụng phụ trùng |
| Restart | Khởi động lại tiến trình | Dữ liệu bền vững và việc đang làm được tiếp tục |
| Restore | Dựng lại dữ liệu từ bản sao | Mọi giao dịch mới nhất đều có mặt |
| Failover | Chuyển phục vụ sang instance khác | Không split-brain, không mất dữ liệu |
| Reconciliation | Đối chiếu và xử lý trạng thái không khớp | Việc mới đã hoạt động bình thường |

Recovery Test là phép kiểm chủ động có failure model, điều kiện bắt đầu/kết thúc và bằng chứng. Backup job thành công chỉ chứng minh đã tạo được bản sao theo công cụ; phải khôi phục rồi đọc/ghi và đối soát mới có bằng chứng sử dụng được.

Resilience bao gồm khả năng chịu lỗi, suy giảm có kiểm soát và phục hồi. Disaster recovery xét thêm sự cố lớn như mất vùng hoặc mất hệ lưu trữ. Lab này kiểm một dependency giả và logical restore nhỏ, không diễn tập mất vùng.

## 3. RTO, RPO và cách đo

**RTO (Recovery Time Objective)** là mục tiêu thời gian phục hồi. Trước test phải chốt mốc bắt đầu: ví dụ lúc nghiệp vụ ngừng đáp ứng, không phải lúc operator bắt đầu restore. Mốc kết thúc cần là nghiệp vụ đã đạt tiêu chí, không chỉ tiến trình vừa mở cổng.

**RPO (Recovery Point Objective)** là mục tiêu giới hạn mất dữ liệu tính theo thời gian. Đo bằng các giao dịch đã được xác nhận trước sự cố và điểm dữ liệu thực sự khôi phục được; không chỉ nhìn giờ sửa file backup. Cần tìm cả lỗ hổng trong chuỗi giao dịch, không chỉ giao dịch có timestamp lớn nhất.

Ví dụ giả định cho bài tập: RTO 15 phút, RPO 5 phút, không thu trùng tiền. Sự cố 10:00, nghiệp vụ hoạt động đủ tiêu chí 10:12, dữ liệu liên tục khôi phục tới 09:57: thời gian phục hồi 12 phút, khoảng mất dữ liệu 3 phút. Nếu còn khoản thu không rõ trạng thái, phải ghi tồn đọng theo tiêu chí nghiệp vụ; không tự coi hệ thống đã phục hồi hoàn toàn. Đây không phải SLO đã có trong SPEC.

Ghi timeline: `t_failure`, `t_detected`, `t_recovery_started`, `t_db_ready`, `t_business_verified`. Dùng đồng hồ đơn điệu để đo duration trong tiến trình; dùng UTC và đồng bộ đồng hồ để ghép evidence nhiều máy. FixedClock trong API lab phục vụ tính tất định nghiệp vụ, không dùng đo thời gian sự cố thật.

## 4. Failure model và tiêu chí

| Sự cố | Phép kiểm bài này | Phạm vi |
| --- | --- | --- |
| Ví lỗi tạm thời một lần | error/timeout rồi thành công trong ngân sách retry | Đã tự động hóa |
| Mất phản hồi sau khi trừ tiền | Một lần mất rồi trả kết quả; hoặc mất cả ba lần | Đã tự động hóa, ví giả |
| Ví ngừng rồi trở lại | Chuyến mới thu được, chuyến cũ vẫn cần xử lý | Đã tự động hóa |
| Xoá nhầm dữ liệu | Restore snapshot vào DB mới, kiểm marker và ghi mới | Đã có script PostgreSQL riêng |
| Mất tiến trình giữa commit/gọi ví | Restart, đối soát và replay có kiểm soát | Bài tập mở rộng |
| Mất máy/vùng, thiếu WAL, failover | Cluster và storage riêng | Chưa chạy |

Test basis: R-PAY-03/04/06, R-NFR-05, R-STATE-06/07 và R-ACCEPT-05 trong [SPEC](../../examples/toi-noi-testing/SPEC.md). Script restore dùng fixture riêng, không tự gán requirement của Tới Nơi cho bảng probe.

## 5. Thực hành A — phục hồi dependency thanh toán

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-recovery-test
npm run typecheck
```

Suite dùng API thật của lab, repository PGlite, ví giả và timeout test 30 ms với tối đa hai retry. SPEC mặc định 3 giây; cấu hình rút ngắn chỉ giúp kiểm logic nhanh, không đo SLO production. Mỗi test reset dữ liệu, sau suite đóng DB.

| Case | Trình tự | Oracle |
| --- | --- | --- |
| RC-01 × 3 | Inject một error/timeout/lost_response rồi ví bình thường | Hai lần gọi cùng khóa, thu đúng 35.000, paid, một event completed, tài xế online |
| RC-02 | Timeout cả ba lần, gỡ lỗi, hoàn thành chuyến mới | Chuyến mới paid; chuyến cũ failed; gọi complete chuyến cũ bị 409, không gọi ví thêm |
| RC-03 | Lost response cả ba lần, sau đó ví bình thường | Chỉ trừ một lần nhưng DB failed; complete lại 409; không tự reconcile |

RC-01 với lost_response kiểm “đã làm nhưng chưa nhận được phản hồi”: cùng khóa `trip:<id>:fare` giúp ví giả không thu trùng. RC-02 kiểm khả năng tiếp nhận **việc mới sau outage**, vượt ra ngoài kiểm exception đơn lẻ.

RC-03 phơi bày giới hạn: HTTP 200/completed theo R-PAY-06 không đồng nghĩa đã xác nhận thanh toán. Không suy `failed` thành “chắc chắn chưa thu tiền”. Lab chưa có worker đối soát, query trạng thái nhà cung cấp hay ledger thật. PASS của case là ghi nhận đúng hành vi và khoảng trống, không chứng minh đã phục hồi đầy đủ nghiệp vụ tài chính.

Không sửa trực tiếp `payment_status='paid'` để làm test xanh. Cần evidence từ nhà cung cấp và quy trình cập nhật có kiểm toán; không cấp khóa mới để retry khoản cũ khi kết quả còn mơ hồ.

## 6. Thực hành B — backup/restore thật, dữ liệu tổng hợp

Từ root repository trên Windows:

```powershell
& ./examples/toi-noi-testing/articles/system-testing-recovery-test/restore-drill.ps1
# Nếu cổng đang bận, chọn cổng khác:
& ./examples/toi-noi-testing/articles/system-testing-recovery-test/restore-drill.ps1 -Port 55485
```

Script tạo thư mục GUID dưới TEMP, initdb cluster mới, chỉ listen 127.0.0.1 và mặc định cổng 55484. Nó không dùng cluster PostgreSQL đang có, không xoá database của bạn. Trust authentication chỉ dành cho fixture giả trên cluster local ngắn hạn; không dùng cấu hình này cho môi trường dùng chung. Cluster dừng trong finally; thư mục bằng chứng được giữ lại.

Trình tự thực nghiệm:

1. Tạo `source_drill`, bảng `recovery_probe`, commit marker 1 với amount 35.000.
2. Dùng `pg_dump -Fc -f` lấy logical backup.
3. Commit marker 2 với amount 42.000 sau backup; kiểm nguồn có hai bản ghi.
4. Mô phỏng xoá nhầm bằng DELETE chỉ trong database vừa tạo.
5. Tạo `restored_drill` từ template0; `pg_restore --exit-on-error --single-transaction` vào DB mới.
6. Kiểm chính xác kết quả `1:35000`: marker 1 còn nguyên, marker 2 không có vì ngoài snapshot.
7. Ghi marker 3, đọc lại count để chứng minh DB restore ghi được; lưu result.json và dừng cluster.

Script dừng khi exit code hoặc assertion sai. Không thêm `--clean` trỏ vào DB có sẵn. Nếu start lỗi vì cổng đã dùng, inspect log và chọn cổng khác; không dừng tiến trình chiếm cổng của người khác.

`restoreAndValidationMs` tính từ trước tạo DB đích đến sau kiểm ghi mới. Nó **không phải RTO ứng dụng**: không gồm phát hiện sự cố, cung cấp hạ tầng mới, đổi endpoint hay kiểm hành trình Tới Nơi. Script chỉ đo mất marker sau backup, chưa đo RPO thời gian; hai cờ trong report ghi false để tránh hiểu nhầm.

Fixture này không restore schema/dữ liệu Tới Nơi. Khi mở rộng cần thêm quote, trip, events, idempotency và dữ liệu liên quan; đối chiếu theo khóa, số tiền, snapshot như SYS-13, rồi chạy hành trình nghiệp vụ qua backend PostgreSQL tương ứng.

## 7. Từ logical restore tới PITR và failover

Logical dump là snapshot nhất quán của một database; roles/tablespaces cần kế hoạch riêng. Dùng custom format với pg_restore và kiểm lỗi; có dump không có nghĩa có đủ cấu hình, chứng thư, quyền và secret để phục vụ ứng dụng. [PostgreSQL — SQL Dump](https://www.postgresql.org/docs/18/backup-dump.html).

PITR dùng base backup cùng chuỗi WAL liên tục để phục hồi đến điểm chọn. Không thể lấy pg_dump rồi replay WAL lên đó. Cần kiểm đủ archive, đúng timeline và cấu hình phục hồi; mất một đoạn cần thiết có thể ngăn đạt điểm đích. Phần này là mở rộng chưa chạy. [PostgreSQL — Continuous Archiving and PITR](https://www.postgresql.org/docs/18/continuous-archiving.html).

Với xoá nhầm lúc 10:00, phục hồi trước lệnh sai cũng loại bỏ thay đổi hợp lệ sau điểm đó; phải có kế hoạch đối soát và đưa lại dữ liệu cần giữ. Với failover, kiểm fencing để primary cũ không tiếp tục ghi, replication lag, client reconnect và tính duy nhất của tác dụng phụ. Failback cũng cần diễn tập riêng.

## 8. Thiết kế recovery test nâng cao

**Điểm cắt:** inject lỗi trước commit, sau commit nhưng trước response, sau thu tiền nhưng trước lưu trạng thái, trong lúc worker giữ job. Mỗi điểm có oracle khác nhau. Retry cả giao dịch phải có giới hạn, backoff/jitter phù hợp và theo dõi retry amplification. Lab hiện không chứng minh các cơ chế này.

**Dữ liệu bền vững:** idempotency key và kết quả phải sống qua restart theo thiết kế. Ví giả lưu charge trong RAM nên không thể dùng việc tạo lại ví giả làm bằng chứng nhà cung cấp không thu trùng. Giữ evidence phía ngoài tiến trình bị lỗi.

**Backlog:** khôi phục dependency chưa đủ nếu hàng đợi tiếp tục phình. Kiểm tốc độ xử lý tồn, công việc cũ nhất, lỗi lặp và công bằng giữa việc mới/cũ. Đặt khoảng ổn định sau phục hồi thay vì chấp nhận một request thành công.

**Backup hỏng:** thử archive thiếu, sai checksum hoặc thiếu quyền giải mã trên bản sao dành riêng cho diễn tập. Kết quả cần fail rõ, không mở traffic với dữ liệu khôi phục một phần. Bài hiện chưa có thử nghiệm backup hỏng.

**Hạ tầng:** restore vào môi trường độc lập với nguồn để kiểm mất cả máy hoặc storage. Bài local đặt backup và data trên cùng máy nên không chứng minh chịu mất ổ đĩa. Theo dõi dung lượng tạm, tài nguyên restore và quyền truy cập evidence.

## 9. Runbook, evidence và quyết định

Runbook cần ghi người điều phối, người xác nhận dữ liệu, phạm vi inject, điều kiện dừng, backup chọn, cách ngăn tác dụng phụ tới hệ ngoài, timeline và tiêu chí mở traffic. Các vai trò này là mẫu tổ chức, không phải yêu cầu xin phép lại khi chạy fixture local đã được giao.

Mẫu evidence:

| Trường | Nội dung cần lưu |
| --- | --- |
| Build và dữ liệu | Commit, runtime, phiên bản DB, seed, backup ID/hash |
| Sự cố | Failure mode, thời điểm inject/gỡ lỗi, request ID |
| Kết quả | HTTP + DB + hệ thanh toán, không chỉ health check |
| Phục hồi | Timeline, marker/khóa thiếu, khoản thu mơ hồ, số job tồn |
| Quyết định | Đạt/chưa đạt mục tiêu, giới hạn và việc phải xử lý |

Tiêu chí bài tập: năm test API pass, typecheck pass, restore đúng marker và ghi mới được, cluster đã dừng. Nếu RC-03 còn failed thì ghi rõ cần reconciliation; không tự tuyên bố release đạt RTO/RPO. [Báo cáo](14-test-report.md) phân biệt các phép kiểm đã chạy với phần mở rộng.

## 10. Bài tập có hướng dẫn

1. Đo RPO thời gian cho restore drill. Gợi ý: lưu bên ngoài DB chuỗi marker đã ACK với thời gian commit, mốc sự cố và marker khôi phục; tìm lỗ hổng, không lấy giờ hoàn thành pg_dump làm điểm dữ liệu.
2. Thêm hai lỗi tạm thời rồi thành công. Gợi ý: failNext hai lần; kỳ vọng ba attempts, một khoản thu, một completed event.
3. Thiết kế worker giải quyết RC-03. Gợi ý: query giao dịch theo khóa cũ, đối chiếu amount/customer, lưu kết quả có kiểm toán; kiểm worker bị restart giữa các bước. Cần mở rộng port và SPEC trước khi coi đây là hành vi sẵn có.
4. Chạy restore với fixture chuyến đang in_transit. Gợi ý: đối chiếu driver busy, snapshot giá, lịch sử và idempotency; kiểm tiếp tục hành trình sau restore, tránh gọi cổng thật.
5. Thêm thử nghiệm archive không đọc được. Gợi ý: bản sao riêng của dump, exit nonzero, không phục vụ DB đích; giữ dump tốt làm đối chứng.

## 11. Tài liệu

Tra cứu ngày 13/11/2026: [SQL Dump](https://www.postgresql.org/docs/18/backup-dump.html), [Continuous Archiving/PITR](https://www.postgresql.org/docs/18/continuous-archiving.html). Nguồn nội bộ: [SPEC](../../examples/toi-noi-testing/SPEC.md), [retry thanh toán](../../examples/toi-noi-testing/src/app/payment.ts), [ví giả](../../examples/toi-noi-testing/src/infra/fake-payment-gateway.ts).

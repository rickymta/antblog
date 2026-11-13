# SYS-13 — Data Integrity Test: dữ liệu đúng từ đầu vào đến lưu trữ

Một API trả 201 chưa đủ chứng minh đặt chuyến thành công: có thể chuyến đã lưu nhưng nhật ký thiếu, báo giá bị dùng hai lần hoặc số tiền bị đổi khi chuyển kiểu dữ liệu. Data Integrity Test kiểm tra các bất biến dữ liệu tại những ranh giới đó, kể cả khi một thao tác thất bại giữa chừng.

Bài trước: [Usability Test](12-usability-test.md). Bài tiếp theo: Recovery Test. Mã chạy ở [lab SYS-13](../../examples/toi-noi-testing/articles/system-testing-data-integrity-test/README.md); kết quả thực chạy ở [báo cáo](13-test-report.md).

## 1. Mục tiêu và chuẩn bị

Sau bài này, bạn có thể:

- Phân loại toàn vẹn thực thể, tham chiếu, miền giá trị và nghiệp vụ.
- Chuyển yêu cầu thành bất biến và SQL oracle độc lập với response HTTP.
- Chèn lỗi giữa giao dịch, chứng minh không còn dữ liệu ghi dở.
- Đối soát dữ liệu liên bảng và kiểm tra chính bộ phát hiện sai lệch.
- Phân biệt bằng chứng local với kiểm thử concurrency, migration và phục hồi thật.

Cần đọc SYS-03, SYS-04 và biết SELECT/JOIN, khóa chính, transaction. Lab dùng Node.js, Vitest, Express và PGlite; không cần PostgreSQL server. Dữ liệu seed là dữ liệu giả. Chỉ chạy câu lệnh phá dữ liệu trên database cô lập của test.

## 2. Khái niệm và phạm vi

Data integrity là việc dữ liệu duy trì những quy tắc đúng đắn đã xác định qua tạo, cập nhật, truyền, lưu và xoá. “Không bị thay đổi” chưa đủ: một số tiền sai ngay từ đầu vẫn sai dù checksum không đổi.

| Loại | Bất biến ví dụ | Cơ chế bảo vệ và phép kiểm |
| --- | --- | --- |
| Thực thể | Mỗi chuyến có một định danh | PRIMARY KEY, thử chèn trùng |
| Tham chiếu | Nhật ký phải thuộc chuyến tồn tại | FOREIGN KEY, chèn orphan |
| Miền giá trị | Trạng thái thuộc tập cho phép | CHECK và NOT NULL, ghi giá trị sai |
| Nghiệp vụ | Một báo giá tối đa một chuyến | UNIQUE quote_id và kiểm tranh chấp |
| Nhất quán liên bảng | Giá, khách, phiên bản khớp báo giá | Đối soát JOIN với snapshot |
| Toàn vẹn biểu diễn | Tiền giảm lẻ không mất khi lưu | Round-trip API → numeric/JSON → repository |

Functional test hỏi chức năng có đúng quy tắc; integration test hỏi các thành phần phối hợp ra sao; integrity test tập trung vào trạng thái dữ liệu cuối và trạng thái sau lỗi. Chúng có thể dùng chung một kịch bản nhưng oracle khác nhau. Recovery Test ở bài 14 sẽ xét dữ liệu sau sự cố và khôi phục.

Data quality còn bao gồm tính đầy đủ, phù hợp, kịp thời của dữ liệu theo mục đích sử dụng. CHECK số điện thoại không rỗng không chứng minh người dùng sở hữu số đó; khóa ngoại không chứng minh chuyến được gán đúng tài xế theo nghiệp vụ.

## 3. ACID và ranh giới giao dịch

Atomicity: các thao tác trong một giao dịch cùng thành công hoặc cùng bị hoàn tác. Consistency: các bất biến được thiết kế phải giữ trước/sau giao dịch; database không tự biết mọi quy tắc nghiệp vụ. Isolation điều chỉnh tương tác giữa giao dịch đồng thời. Durability nói về dữ liệu đã commit qua sự cố theo bảo đảm và cấu hình của hệ lưu trữ.

Trong lab, `PgTripRepository.create()` bọc INSERT chuyến và INSERT các event trong `db.transaction()`. Vì vậy có thể kiểm lỗi event thứ hai khiến cả chuyến và event thứ nhất bị rollback. Điều này **không chứng minh toàn bộ use case đặt chuyến**, đánh dấu báo giá, idempotency và gọi ví nằm trong cùng một giao dịch.

PGlite trong bài chạy một kết nối, database tạm trong bộ nhớ. Kết quả không chứng minh isolation nhiều session hay durability sau mất điện. Rollback do lỗi câu SQL khác với crash-recovery.

## 4. Test basis và thiết kế oracle

Nguồn nghiệp vụ: [SPEC](../../examples/toi-noi-testing/SPEC.md). Những phép kiểm schema dùng schema hiện tại làm test basis; không tự gán mã yêu cầu mới vào SPEC.

| Case | Cơ sở | Kích thích | Kết quả bắt buộc |
| --- | --- | --- | --- |
| DI-01 | R-QUOTE-02/04, R-STATE-06 | Đặt xe 5 km lúc 09:00 | 35.000 đ, snapshot khớp, hai event đúng thứ tự/thời điểm |
| DI-02 | R-QUOTE-03, UNIQUE | Chèn chuyến thứ hai dùng cùng quote | SQLSTATE 23505, chỉ còn chuyến ban đầu |
| DI-03 | Transaction repository, R-STATE-06 | Event đầu hợp lệ, event sau tham chiếu chuyến không tồn tại | 23503, không có chuyến/event mới, dữ liệu cũ còn nguyên |
| DI-04 | CHECK/NOT NULL, R-STATE-01 | Ghi status lạ rồi fare NULL | 23514/23502, chuyến không đổi |
| DI-05 | R-GEN-01, R-FARE-03/10, R-QUOTE-04 | 0,5 km, cao điểm, GIAM20 | Discount 3.562,5 lưu chính xác, fare 15.000 |
| DI-06 | R-QUOTE-04 | Tăng trực tiếp fare thêm 1.000 | Đối soát từ 0 sai lệch thành đúng 1 chuyến |

Oracle DI-05 tính tay: `(12.000 + 4.500 × 0,5) × 1,25 = 17.812,5`; giảm 20% là `3.562,5`; còn `14.250`, nâng tối thiểu lên `15.000`, làm tròn nghìn vẫn `15.000`. Không gọi lại hàm tính cước của ứng dụng để tạo expected value.

DI-02 kiểm hàng rào unique khi ghi tuần tự, chưa kiểm hai request đồng thời. DI-01 chỉ kiểm nhật ký khởi tạo, chưa kiểm mọi chuyển trạng thái. Phạm vi đo phải nhỏ hơn hoặc bằng kết luận.

## 5. Chạy lab từ đầu

Tại thư mục repository:

```powershell
cd examples/toi-noi-testing
npm ci
npm test -- articles/system-testing-data-integrity-test
npm run typecheck
```

Mỗi test reset dữ liệu seed và đồng hồ; sau suite đóng database. Header `x-user` chỉ mô phỏng danh tính trong lab. Không dùng kết quả này để kết luận xác thực production an toàn.

Đọc test theo thứ tự DI-01 → DI-06. Với DI-03, tạo báo giá mới cho chuyến thử để tránh UNIQUE chặn trước điểm cần inject lỗi. Giữ một chuyến cũ làm mẫu đối chứng. Nếu chỉ kiểm exception mà không kiểm dữ liệu sau lỗi, test có thể bỏ lọt ghi dở.

## 6. Viết câu đối soát hữu ích

Câu sau được dùng trong suite cho phạm vi chuyến đã tạo:

```sql
SELECT t.id
FROM trips t LEFT JOIN quotes q ON q.id = t.quote_id
WHERE q.id IS NULL
   OR t.customer_id IS DISTINCT FROM q.customer_id
   OR t.pricing_version IS DISTINCT FROM q.pricing_version
   OR t.fare IS DISTINCT FROM (q.details->>'total')::numeric
   OR q.used_by_trip_id IS DISTINCT FROM t.id;
```

`IS DISTINCT FROM` xử lý NULL như một giá trị có thể so sánh, tránh điều kiện `<>` trả unknown khiến lọt bản ghi. LEFT JOIN giúp bộc lộ thiếu báo giá; schema hiện có FK sẽ chặn trường hợp đó trong đường ghi thông thường.

Query chưa phát hiện báo giá đánh dấu đã dùng nhưng không có chuyến nào: phải bổ sung phép đối soát chiều ngược, xuất phát từ quotes và LEFT JOIN trips. Nó cũng chưa so toàn bộ JSON, discount, trạng thái cuối với event cuối, hay tiền thực thu tại ví. Tách mỗi nhóm invariant thành query có mã lỗi để báo cáo dễ xử lý.

DI-06 chủ động sửa fare bằng SQL; schema hiện cho phép nên query phải bắt được. Test PASS ở đây nghĩa **bộ phát hiện hoạt động**, không nghĩa ứng dụng đã ngăn mọi đường gây sai lệch. Không sửa schema trong bài này.

## 7. Lỗi thường gặp khi thiết kế schema và test

- CHECK có thể chấp nhận biểu thức trả NULL; cần NOT NULL nếu trường bắt buộc.
- UNIQUE mặc định có ngữ nghĩa riêng với NULL; kiểm cả nullability của khóa nghiệp vụ. `trips.quote_id` ở lab là NOT NULL UNIQUE.
- FK chỉ kiểm sự tồn tại, không kiểm khách của chuyến có trùng khách của báo giá.
- Kiểu integer làm mất tiền lẻ trung gian nếu ép quá sớm. Dùng numeric ở DB và kiểm chuyển đổi khi đọc về ngôn ngữ ứng dụng.
- Chỉ kiểm tổng số hàng có thể bỏ lọt một bản ghi thiếu bù bởi một bản ghi thừa. Đối chiếu theo khóa và nội dung.
- Timestamp bằng nhau không xác định thứ tự event. Lab dùng ORDER BY id cho hai event tạo cùng thời điểm; hệ phân tán cần quy tắc sequence/version rõ ràng.
- Checksum bằng nhau không chứng minh giá đúng công thức; cần cả oracle nghiệp vụ.

## 8. Mở rộng nâng cao: concurrency, migration, đối soát tiền

**Concurrency:** dùng PostgreSQL server và hai kết nối độc lập. Đồng bộ bằng barrier để cả hai cùng đọc trạng thái trước khi ghi; cho hai transaction tranh cùng quote. Kiểm một chuyến tồn tại, event không trùng, response thua có nghĩa nghiệp vụ rõ ràng. Thử timeout, deadlock và retry toàn giao dịch; không dùng sleep ngẫu nhiên làm bằng chứng đã tạo race. Chọn isolation theo invariant, không coi SERIALIZABLE là thay thế thiết kế retry.

**Migration:** dựng dữ liệu phiên bản cũ có NULL, biên numeric và giá trị từng được cho phép. Chụp baseline theo khóa, chạy migration, đối chiếu số hàng, quan hệ, tổng tiền và snapshot. Thử đọc bằng phiên bản ứng dụng cũ/mới nếu rollout có giai đoạn cùng tồn tại. Một migration chạy xong không đồng nghĩa dữ liệu không bị biến nghĩa. Chạy lại để kiểm tính idempotent chỉ khi migration được thiết kế hỗ trợ việc đó.

**Đối soát thanh toán:** so trip, payment attempt và ledger nhà cung cấp theo khóa khoản thu, số tiền, trạng thái và thời gian. Timeout có thể xảy ra sau khi đã trừ tiền; retry phải dùng cùng khóa và có cơ chế truy vấn kết quả. Lab chỉ có ví giả, không có ledger/webhook thật (R-ARCH-02). SYS-04 đã kiểm một số retry/idempotency; bài này không chạy lại hoặc mở rộng phạm vi thanh toán đó.

**Hệ phân tán:** nếu dữ liệu có eventual consistency, định nghĩa thời gian hội tụ và watermark trước khi gắn nhãn sai lệch. Với outbox/inbox, inject lỗi trước/sau commit, kiểm duplicate, sai thứ tự và replay. Đây là thiết kế mở rộng, chưa có trong lab.

**Khối lượng lớn:** chia đối soát theo partition hoặc khoảng khóa, lưu watermark và đếm cả số hàng đã xét. Dùng snapshot phù hợp để tránh báo lỗi giả khi dữ liệu thay đổi giữa các query. Không tự sửa tài chính chỉ từ một query phát hiện; lưu evidence và áp dụng quy trình sửa có kiểm toán.

## 9. Báo cáo và tiêu chí hoàn thành

Ghi commit, runtime, backend, seed, case, SQLSTATE, invariant trước/sau và phạm vi chưa kiểm. Không đưa thông tin cá nhân vào evidence. Với lỗi thật, lưu ID có thể truy vết và expected/actual để tái hiện.

Điều kiện đạt bài tập: 6 case pass; DI-03 không để lại ghi dở; DI-06 phát hiện đúng bản ghi bị sửa; typecheck pass. Đây là tiêu chí bài học, không phải gate production. Kết quả hiện tại: [6/6 case đạt](13-test-report.md).

Khi thất bại: phân biệt setup sai, lỗi nghiệp vụ, lỗi bảo vệ schema, lỗi transaction và lỗi query đối soát. Nếu DI-03 trả 23505 thay vì 23503, kiểm dữ liệu báo giá mới trước khi kết luận rollback hỏng. Nếu DI-06 vẫn trả 0 hàng, dừng tin cậy báo cáo “không sai lệch” của query đó.

## 10. Bài tập và gợi ý đáp án

1. Thêm đối soát quote đã dùng nhưng thiếu trip. Gợi ý: LEFT JOIN từ quotes, lọc `used_by_trip_id IS NOT NULL`, so cả ID và quote_id; inject con trỏ không tồn tại vì cột này hiện chưa có FK.
2. Phát hiện thiếu event searching. Gợi ý: xoá một event trên DB test, kiểm chuỗi expected theo lifecycle, không chỉ tổng toàn bảng; xử lý tài xế huỷ có thể đưa chuyến về searching nhiều lần.
3. Mở rộng DI-05 sang nhiều giá trị phần lẻ. Gợi ý: lập bảng expected tính độc lập, gồm biên làm tròn; một mẫu 3.562,5 chưa đại diện mọi khả năng numeric.
4. Thiết kế hai transaction tranh quote. Gợi ý: mô tả barrier, kết quả cho phép và dọn dữ liệu trước khi chọn công cụ; không dùng hai Promise trên một kết nối để kết luận isolation production.
5. Chèn lỗi giữa đánh dấu báo giá và tạo chuyến. Gợi ý: khảo sát ranh giới use case, kiểm cả quote, trip, events, idempotency; ghi nhận khoảng trống nếu không có transaction bao trùm, không suy rộng từ DI-03.

## 11. Tài liệu tham khảo

Tra cứu ngày 13/11/2026:

- [PostgreSQL 18 — Constraints](https://www.postgresql.org/docs/18/ddl-constraints.html): CHECK, NOT NULL, UNIQUE, khóa chính/ngoại và giới hạn ràng buộc.
- [PostgreSQL 18 — Transactions](https://www.postgresql.org/docs/18/tutorial-transactions.html): giao dịch, COMMIT, ROLLBACK và savepoint.
- [SPEC Tới Nơi](../../examples/toi-noi-testing/SPEC.md), [schema SQL](../../examples/toi-noi-testing/src/infra/pglite/schema.sql), [trip repository](../../examples/toi-noi-testing/src/infra/pglite/pg-trip-repository.ts): cơ sở trực tiếp của lab và các giới hạn được nêu.

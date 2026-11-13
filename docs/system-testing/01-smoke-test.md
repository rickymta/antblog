# SYS-01 — Smoke Test: bản build có đủ tốt để bắt đầu kiểm sâu?

Trạng thái: bản nháp biên tập; có lab thực thi. Thuộc [lộ trình 16 bài](../system-testing-roadmap.md). Độc giả: tester mới biết HTTP và lập trình TypeScript cơ bản; phần nâng cao dành cho QA/SDET thiết kế release gate.

## 1. Bài toán và mục tiêu

Tới Nơi vừa cập nhật ứng dụng. Trang chủ vẫn mở, nhưng câu lệnh ghi chuyến không tương thích schema. Nếu bắt đầu hàng trăm ca voucher, nhóm sẽ mất thời gian ghi nhận cùng một lỗi nền. Ta cần vài phép kiểm đại diện để quyết định có thể tiếp tục vòng kiểm thử hay phải trả build về sửa.

Sau bài này, bạn có thể chọn smoke cases từ rủi ro, viết một hành trình đi xuyên API và kho dữ liệu, giải thích vì sao health xanh chưa đủ, phân loại lỗi test và lập quyết định tiếp tục kiểm thử. Bài dùng lab modular monolith; không giả định đã có microservices chạy thật.

## 2. Khái niệm và những điều dễ nhầm

Trong series này, **smoke test** là bộ kiểm tra nhỏ dành cho các khả năng thiết yếu của một build hoặc môi trường. Nó thường được chạy sau build/deploy và trước vòng kiểm thử sâu. Giá trị chính là phát hiện sớm lỗi khiến phần còn lại của vòng kiểm thử không có ý nghĩa.

Smoke không phải một test level độc lập: có thể chọn kiểm tra ở API, UI hoặc tích hợp. Một happy path có ích nhưng không bắt buộc chỉ có happy path; cơ chế chặn truy cập vô danh có thể quan trọng đủ để nằm trong smoke.

| Khái niệm | Câu hỏi trả lời | Ví dụ Tới Nơi |
|---|---|---|
| Health/liveness | Tiến trình có phản hồi không? | GET health trả 200 |
| Readiness | Instance có đủ điều kiện nhận traffic không? | Kiểm dependency thiết yếu theo thiết kế |
| Smoke | Các khả năng thiết yếu có dùng được không? | Đặt xe, nhận chuyến, hoàn tất, lưu và thu tiền |
| Sanity | Thay đổi cụ thể có hợp lý để kiểm tiếp không? | Sau sửa voucher, kiểm lỗi gốc và vùng liên quan |
| Confirmation | Lỗi đã sửa có còn tái hiện không? | Chạy lại ca từng báo defect |
| Regression | Thay đổi có làm hỏng phần đang hoạt động không? | Voucher sửa nhưng tổng thanh toán bị sai |

Smoke và sanity được các nhóm dùng tên khác nhau; cần ghi mục tiêu và phạm vi trong test plan thay vì tranh luận chỉ theo tên. Smoke tải thấp trong tài liệu k6 nhằm kiểm script và hệ thống trước bài đo tải, cần phân biệt với smoke build ở đây.

## 3. Thiết kế từ rủi ro và test basis

Đọc `examples/toi-noi-testing/SPEC.md` trước khi đọc implementation. Một oracle là căn cứ xác định đúng/sai; nếu lấy chính hàm đang kiểm để tính expected, hai bên có thể sai giống nhau. Trong lab, cước xe máy 5 km giờ thường tại SG, trời không mưa, không voucher là 35.000 đồng theo bảng cước. Test dùng con số đó, không gọi lại calculator để tạo expected.

Chọn case bằng ba câu hỏi: nếu hỏng có chặn hoạt động cốt lõi không, case có đi qua thành phần mới dễ hỏng không, và chi phí chạy/điều tra có phù hợp gate không? Không chọn chỉ vì case chạy nhanh hoặc đã có sẵn.

| Case | Basis/rủi ro | Tiền điều kiện và hành động | Expected |
|---|---|---|---|
| SM-01 | Hợp đồng `/api/health` của lab | App mới; gọi GET health | 200 và đúng `{status: 'ok'}` |
| SM-02 | R-API-03; mất chặn truy cập | Không gửi x-user; POST báo giá hợp lệ | 401 và UNAUTHENTICATED |
| SM-03 | R-STATE-02, R-PAY-03, bảng cước; đường tiền bị đứt | C001, D101 có trong seed; 09:00 VN 04/11/2026; báo giá 5 km, đặt bằng VíXanh, nhận và hoàn tất | 35.000; completed/paid khi đọc lại; DB lưu đúng; ví gọi một lần, thu đúng tiền |

Mọi case reset seed và dựng Express app mới; PGlite đóng cuối suite. Bộ này cố ý nhỏ: bỏ biên 100 km, tổ hợp voucher, retry và lỗi thanh toán khỏi smoke ban đầu để kiểm ở các suite chuyên sâu. Việc bỏ ca phải có nơi kiểm thay thế, không đồng nghĩa rủi ro được miễn.

Entry criteria: dependency đã cài, SPEC và seed đúng phiên bản, không dùng dữ liệu production. Exit criteria bài tập: cả ba case pass, không skip, exit code 0. Đây là điều kiện cho phép **kiểm tiếp**, không phải cho phép release production.

## 4. Thực hành có thể chạy lại

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-smoke-test/smoke.test.ts`. Dùng Node >=22.12; từ thư mục lab:

```sh
npm ci
npx vitest run articles/system-testing-smoke-test
npm run typecheck
```

Chỉ cần `npm ci` khi chuẩn bị dependency. PGlite chạy PostgreSQL dưới WASM nên không cần cài DB server. Supertest đưa HTTP request vào Express app trong tiến trình test. Fake clock cố định giờ, fake weather mặc định không mưa, fake VíXanh ghi lại các lần gọi và số tiền thu. Đó là các điều kiện kiểm soát, không phải các hệ thống bên ngoài đã được kiểm chứng.

Luồng chính gồm ba đoạn. Đầu tiên gọi báo giá và assert 35.000 để phát hiện cấu hình giá lệch. Sau đó dùng quoteId thực nhận để tạo chuyến, chuyển trạng thái bằng API tài xế. Cuối cùng đọc lại chuyến qua API và repository để kiểm trạng thái bền trong DB của phiên chạy, đồng thời kiểm sổ gọi ví. Không chỉ assert HTTP 200: server có thể trả thành công nhưng không lưu hoặc thu sai tiền.

```ts
expect(read.body.trip).toMatchObject({
  id, status: 'completed', fare: 35_000, paymentStatus: 'paid',
});
expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
expect(system.payments.calls).toHaveLength(1);
expect(system.payments.totalCharged()).toBe(35_000);
```

Khóa thu tiền được kiểm theo tripId tạo trong test, không hardcode ID do generator. Assert số lần gọi bằng 1 chỉ đúng cho happy path không inject lỗi; với retry, nhiều lần gọi cùng khóa có thể đúng. Không chuyển assertion này nguyên xi sang bài recovery.

Expected output là 3 test pass; thời gian chạy thay đổi theo máy. Kết quả đã chạy và các giới hạn được ghi trong [báo cáo](test-report.md).

## 5. Đọc thất bại và ra quyết định

| Dấu hiệu | Giả thuyết cần kiểm | Bằng chứng tiếp theo |
|---|---|---|
| PGlite không khởi tạo | Dependency, WASM, thiếu tài nguyên | Stack trace, phiên bản Node, lockfile |
| Health xanh, tạo chuyến lỗi | Mapping/schema hoặc quy tắc nghiệp vụ | Response code, bước thất bại, log DB |
| Giá khác 35.000 | Seed, giờ, mưa, đặc tả hoặc regression | Diff cấu hình và test basis; không tự sửa expected |
| Completed nhưng chưa paid | Đường thanh toán hỏng | Payment calls, khóa idempotency, trạng thái lưu |
| Chạy riêng pass, chạy chung fail | Trạng thái dùng chung hoặc cleanup thiếu | Seed, app lifecycle, thời điểm reset |

Lỗi môi trường cần báo blocked với bằng chứng; lỗi sản phẩm báo failed và tạo defect có request, expected/actual, phiên bản build, dữ liệu tái hiện. Không đổi mọi failure thành lỗi sản phẩm hoặc rerun cho tới khi xanh. Một lần pass sau retry không xóa lần lỗi trước.

Ví dụ quyết định: “SM-03 fail ở bước complete, paymentStatus khác paid. Dừng vòng kiểm chức năng phụ thuộc thanh toán; vẫn có thể review test design độc lập. Sau sửa, chạy confirmation rồi smoke lại.” Nếu cả ba pass, chỉ kết luận đường đi này hoạt động trong cấu hình lab đã nêu.

## 6. Mở rộng nâng cao

### Gate trước và sau triển khai

Suite hiện tại phù hợp gate kiểm mã với các thành phần được lắp ráp. Post-deploy smoke cần gọi base URL thật của môi trường cô lập để bao phủ routing, TLS, biến môi trường và artifact đang chạy. Khi đó phải ghi build ID và config version; kết quả của code cũ không phải bằng chứng cho build mới. Một pipeline hợp lý là build → suite này → deploy staging → post-deploy smoke → regression/risk suites → quyết định release.

### Dữ liệu và tác dụng phụ

Trong môi trường dùng chung, không reset toàn bộ DB như lab. Dùng tenant kiểm thử, dữ liệu gắn run ID, tài khoản dành riêng và cleanup giới hạn theo run ID. Các bước thu tiền phải dùng payment sandbox đã cấu hình. Nếu cleanup lỗi, lưu danh sách đối tượng để xử lý; không che lỗi bằng việc bỏ assertion.

### Hệ phân tán và nhất quán cuối cùng

Một hệ thống có broker có thể trả accepted trước khi trạng thái đọc được cập nhật. Khi SPEC cho phép eventual consistency, chờ có điều kiện đến deadline theo SLO, lưu lịch sử trạng thái và correlation ID. Không thêm sleep cố định tùy ý để làm test xanh. Deadline phải được thống nhất; lab này không có broker nên bài chưa chứng minh cơ chế đó.

### Chọn suite theo ngân sách thời gian

Ví dụ mục tiêu nội bộ là gate dưới hai phút: đo p95 thời gian suite qua nhiều lần chạy rồi loại các case trùng rủi ro hoặc chuyển sang suite sâu. Hai phút là giả định thiết kế, không phải chuẩn chung. Theo dõi tỷ lệ báo lỗi sai và số sự cố lọt qua để điều chỉnh suite; không dùng tổng số test làm thước đo chất lượng.

### Chứng minh test biết bắt lỗi

Trên nhánh bài tập, tạo bản adapter ví lỗi trong thư mục bài, làm cho nó báo thành công nhưng không ghi số tiền đã thu, rồi inject qua port. Expected: SM-03 đỏ ở oracle tiền, SM-01 vẫn xanh. Sau khi bỏ fault, suite xanh lại. Đây là thí nghiệm red/green đề xuất, chưa được tính vào báo cáo lần chạy hiện tại; không sửa `src/` dùng chung của lab để giữ một lỗi minh họa.

## 7. Anti-pattern và giới hạn

Chỉ kiểm health sẽ bỏ sót DB và thanh toán. Nhét toàn bộ regression vào smoke làm mất phản hồi nhanh. So sánh snapshot response quá rộng làm test nhạy với trường không liên quan. Dùng thời gian thực làm giá cao điểm thay đổi. Retry vô hạn che flakiness. Tắt assertion để vượt gate làm mất giá trị kiểm tra.

Suite dùng PGlite một kết nối; chưa kiểm concurrency PostgreSQL nhiều kết nối, restart durability, browser, mạng ngoài, JWT, payment provider thật hay khả năng chịu tải. Header x-user là cơ chế giả lập danh tính, không phải bảo mật production. Repository read-back trong cùng DB session chứng minh dữ liệu đã ghi/đọc được ở phiên đó, không chứng minh backup/restore hay tồn tại sau sự cố máy.

## 8. Bài tập và gợi ý đáp án

1. **Cơ bản:** bỏ header x-user trong case đặt xe. Expected: báo giá bị 401 ngay, không đi tới bước thanh toán. Xác định assertion nào phải bắt lỗi và vì sao.
2. **Trung bình:** thêm kiểm tra khách C002 không đọc được chuyến của C001. Expected theo R-API-05 là 404 để không lộ sự tồn tại. Quyết định có giữ trong smoke dựa trên mức rủi ro phân quyền của release.
3. **Nâng cao:** thiết kế post-deploy smoke cho staging có nhiều worker. Đáp án cần run ID, tenant riêng, timeout có căn cứ, correlation ID, payment sandbox và cleanup không đụng dữ liệu của lần chạy khác.
4. **Tự kiểm:** ba case pass có đủ release không? Không: các biên, phi chức năng, tích hợp thật và tiêu chí acceptance chưa được phủ. Health xanh nhưng SM-03 đỏ có mâu thuẫn không? Không, chúng trả lời hai câu hỏi khác nhau.

## 9. Tài liệu đọc tiếp

- [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf): test levels/types và confirmation/regression; phần pipeline, lựa chọn smoke ở đây là hướng dẫn thực tiễn.
- [Grafana k6 — Load test types](https://grafana.com/docs/k6/latest/testing-guides/test-types/): đối chiếu nghĩa smoke trong kiểm thử hiệu năng.
- SPEC và README của lab là test basis trực tiếp. Nguồn ngoài được tra cứu ngày 13/11/2026. Bài tiếp theo: Sanity Test, tập trung lựa chọn ca theo phạm vi thay đổi.

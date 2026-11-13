# SYS-05 — System Test: kiểm hành vi của toàn ứng dụng trong ranh giới đã chọn

Trạng thái: bản nháp hoàn chỉnh để review, có lab trình duyệt. Bài trước: [Integration Test](04-integration-test.md). [Lộ trình 16 bài](../system-testing-roadmap.md).

## 1. Tình huống và mục tiêu

API hoàn tất chuyến trả completed, DB lưu đúng, nhưng trang khách vẫn hiện “Đang tìm tài xế” vì polling không chạy. Kiểm integration của backend có thể xanh mà người dùng vẫn không biết chuyến đã kết thúc. Muốn phát hiện vấn đề này, cần đi qua cách người dùng thao tác và quan sát hệ thống đã lắp ráp.

Bài dành cho người biết HTTP, test case và Playwright cơ bản. Sau bài, bạn có thể xác định system under test, thiết kế hành trình nhiều vai trò, kiểm kết quả ở UI và API, xử lý chờ bất đồng bộ có điều kiện, và phân biệt một bài lab hệ thống với bằng chứng release production.

## 2. Khái niệm và các ranh giới dễ nhầm

System testing đánh giá hành vi và khả năng của hệ thống hoàn chỉnh trong phạm vi được xác định. Nó có thể bao gồm chức năng và phi chức năng; không bị giới hạn ở UI automation. “Hoàn chỉnh” cần hiểu theo đối tượng đang kiểm: bản MVP modular monolith của lab khác kiến trúc microservices mục tiêu của Tới Nơi.

| Khái niệm | Đối tượng chính | Ví dụ |
|---|---|---|
| Integration | Tương tác giữa thành phần/hệ thống | SQL mapping, retry payment port |
| System | Hệ thống được lắp ráp theo phạm vi đã chọn | Khách đặt, tài xế xử lý, khách thấy kết quả |
| E2E | Luồng từ điểm bắt đầu tới kết quả cuối | Đặt → nhận → hoàn tất trong hai UI |
| UAT | Sự đáp ứng nhu cầu và tiêu chí chấp nhận nghiệp vụ | Đại diện nghiệp vụ chấp nhận quy trình |

E2E là cách tổ chức hành trình, không đồng nghĩa mọi test hệ thống phải là chuỗi UI dài. Test lỗi đầu vào ở trang khách cũng kiểm hệ thống, dù không đi hết vòng đời chuyến. Một browser test chỉ mock mọi request có thể kiểm giao diện hữu ích nhưng không cung cấp bằng chứng đường browser–server–DB thật.

Trong bài, hệ thống gồm Chrome → HTML/JS khách và tài xế → HTTP server Express chạy riêng → use cases → repository PGlite. Clock, thời tiết, ví và danh tính x-user vẫn mô phỏng. Không có Kafka, GPS, mobile app, reverse proxy, nhà cung cấp thanh toán hay hệ thống đăng nhập production.

## 3. Chiến lược: ít hành trình nhưng oracle rõ

Chọn hành trình từ rủi ro người dùng: chuyến không hoàn thành được; khách không nhìn thấy tiến độ; huỷ nhưng phí sai; chỉnh dữ liệu vẫn đặt theo báo giá cũ. Không sao chép toàn bộ 24 ca giá của bài Functional lên browser: những tổ hợp tiền đã có kiểm nhanh, UI giữ các đường đại diện và lỗi ghép nối.

Test basis là SPEC: R-UI-01/02 cho hai trang và polling; R-UI-04 cho định dạng tiền; R-STATE-02 cho hành trình; R-PAY-03 cho kết quả thu; R-CANCEL-01/R-PAY-05 cho huỷ miễn phí; R-API-05 cho khách khác không đọc được chuyến. Case sửa dữ liệu chọn ô distance, không tuyên bố đã kiểm mọi field của R-UI-01.

Oracle tiền độc lập: xe máy 5 km, giờ thường, không mưa → 12.000 + 4.500 × 5 = 34.500, làm tròn 35.000. UI phải hiện `35.000 đ`. Không lấy số UI rồi dùng chính số đó làm expected cho API: cả hai có thể sai giống nhau.

## 4. Môi trường và quyền sở hữu dữ liệu kiểm thử

`system-testing.config.ts` dành riêng cho bài, mở server trên cổng 3115 với PGlite, không reuse server. Nếu cổng đang được dùng, runner dừng thay vì lấy một server không biết cấu hình và reset dữ liệu của nó. Đây là yêu cầu để kết quả có thể diễn giải được, không chỉ là tiện ích chạy test.

| Cấu hình | Giá trị và lý do |
|---|---|
| Browser | Chrome cài sẵn, headless; chưa có matrix nhiều engine |
| Dữ liệu | Seed C001, D101, SG, bike, 5 km; reset trước mỗi case |
| Giờ | Đặt về 09:00 VN 04/11/2026 trước mỗi case; clock tiếp tục chạy |
| DB | PGlite trong tiến trình server; không dùng DB dùng chung |
| Rate limit | 5 lượt/60 giây; case chỉ tạo một chuyến |
| Worker/retry | Một worker, zero retry; tránh reset chéo và che flakiness |
| Timeout | Test 30 giây, assertion 8 giây, server startup 60 giây |
| Evidence | Trace và screenshot khi fail, đặt dưới reports/system-testing |

API `/api/test/reset` và `/api/test/clock` chỉ dùng setup, không dùng để đưa chuyến thẳng tới completed. Các thao tác nghiệp vụ của hai vai trò đi qua UI. Không bật API test trên production. Server riêng được runner quản lý và dừng sau lượt chạy; browser context được đóng để dọn tài nguyên.

## 5. Thiết kế ba kịch bản

| ID | Tiền điều kiện, hành động | Expected | Cleanup |
|---|---|---|---|
| SY-01 | Seed sạch; khách báo giá/đặt bằng VíXanh; tài xế trong context riêng tải, nhận, arrive/pickup/start/complete | Cước 35.000; UI tài xế đi đúng nhãn; UI khách thấy D101 và Hoàn thành; nút huỷ disabled; API completed/paid; C002 đọc nhận 404 | Đóng context tài xế, context khách do runner quản lý; reset case sau |
| SY-02 | Seed sạch; khách đặt rồi huỷ ngay khi searching | UI Đã huỷ, phí 0 đ, nút huỷ disabled; API cancelled, cancellationFee=0, paymentStatus=none | Reset case sau |
| SY-03 | Seed sạch; báo giá 5 km rồi sửa thành 0,1 km và báo giá lại | Giá cũ bị ẩn, nút đặt disabled; API 422 DISTANCE_OUT_OF_RANGE; UI có thông báo lỗi | Context tự đóng; không tạo trip |

SY-01 dùng ID tạo thực tế, không hardcode UUID hoặc chọn dòng đầu danh sách. UI tài xế được kiểm current-trip-id để chắc đang xử lý đúng chuyến. Khách C002 được kiểm qua API như oracle bổ trợ; đây chưa phải hành trình UI đầy đủ của người thứ ba hoặc bài pentest.

## 6. Thực hành và giải thích code

Mã: `examples/toi-noi-testing/articles/system-testing-system-test/e2e/system.spec.ts`. Từ thư mục lab, Node >=22.12 và Chrome có sẵn:

```sh
npm ci
npx playwright test --config system-testing.config.ts
npm run typecheck
```

Chỉ cài dependency khi cần. Expected: ba test pass; browser/server khởi động tự động. Không cần mở `npm start` trước. Config E2E mặc định có thể dùng memory hoặc reuse server nên không tương đương cấu hình bài này. Kết quả thực đo nằm trong [báo cáo SYS-05](05-test-report.md).

### Bước 1 — Setup có kiểm kết quả

Reset và clock đều assert HTTP 200. Nếu test control bị tắt, test dừng ở setup thay vì chạy trên dữ liệu cũ rồi sinh triệu chứng khó hiểu. Reset DB không tự đặt lại clock nên cần hai lệnh riêng. Clock mốc 09:00 cách xa biên cao điểm để thời gian thao tác không làm đổi oracle tiền.

### Bước 2 — Đi qua UI và tách vai trò

Khách fill distance, chọn VíXanh, click báo giá rồi đặt. Tài xế dùng browser context riêng để không chia sẻ cookie/storage nếu ứng dụng có bổ sung chúng sau này. Nhưng danh tính lab vẫn là mã người dùng trong form, không phải kiểm đăng nhập thật. URL tài xế được suy từ origin trang khách để không khóa file test vào một cổng khác với server được chọn.

### Bước 3 — Chờ điều kiện thay vì sleep

Trang khách polling mỗi hai giây. `toHaveText('Hoàn thành')` tự chờ tới khi điều kiện đạt hoặc hết tám giây; không sleep cố định rồi đọc DOM một lần. Tám giây là budget test local, không phải SLA cập nhật của sản phẩm. Zero retry toàn test không mâu thuẫn với assertion retry: assertion đang chờ trạng thái trong cùng một lần thực hiện, không chạy lại nghiệp vụ sau failure.

SY-03 đăng ký `waitForResponse` trước click để không bỏ lỡ response nhanh. Predicate lọc method POST và đường quotes. Sau đó kiểm status/code và thông báo UI; chỉ thấy text lỗi không chứng minh server đã nhận dữ liệu mới.

### Bước 4 — Kiểm hai mặt của kết quả

Trong SY-01, UI khách và tài xế đều hoàn thành, còn API xác nhận paymentStatus paid, fare và driverId. Nếu chỉ kiểm API, lỗi polling hoặc render có thể lọt. Nếu chỉ kiểm label UI, bug hiển thị sớm trước khi server hoàn thành có thể lọt. Bài integration trước đã kiểm ledger ví; System Test này không có endpoint ledger và không tự nhận đã đối soát tiền thực tại provider.

## 7. Điều tra failure và tiêu chí kết thúc

| Failure | Phân loại ban đầu | Việc kiểm tiếp |
|---|---|---|
| Cổng bận/Chrome thiếu | Môi trường blocked | Xem process/cấu hình hoặc cài browser đúng yêu cầu; không tự dùng server lạ |
| Báo giá không hiện | UI/network/API | Trace request, response và DOM; phân biệt click không xảy ra với server lỗi |
| API completed nhưng UI khách cũ | Polling/render hoặc lỗi mạng | Request GET lặp, response và thời điểm DOM đổi |
| Tài xế không thấy chuyến | Seed/filter/trạng thái | ID khách vừa tạo, city/vehicle và response danh sách |
| Test riêng pass, cả suite fail | Reset/data isolation | Worker count, context chưa đóng, clock và seed |

Entry criteria: môi trường riêng, Chrome và dependency có sẵn, test API được bật đúng ở lab, SPEC xác định. Exit bài tập: ba case pass không skip/retry, typecheck đạt và giới hạn bằng chứng được ghi. Nếu có lỗi intermittent, giữ trace và điều tra; không tăng timeout hay bật retry rồi gọi là ổn định.

Một report tốt ghi commit, cấu hình DB/browser, dữ liệu, bước đầu tiên lệch, expected/actual và artifacts. Trace có thể chứa payload; khi dùng dữ liệu ngoài lab cần xử lý quyền truy cập artifacts theo dữ liệu thật. Không cần ghi lại mọi click trong báo cáo nếu trace đã cung cấp bằng chứng đó.

## 8. Mở rộng nâng cao

### 8.1. Hành trình thất bại và nhất quán cuối cùng

Happy path không bao phủ mất mạng, ví decline hoặc hết hạn báo giá. Có thể inject fault qua test control trước hành động để kiểm UI không báo thanh toán thành công sai. Nếu hệ thống thật dùng broker, cần deadline theo SLO và correlation ID để theo dõi từ command tới event và read model. Không tăng sleep tùy ý để che độ trễ không kiểm soát.

### 8.2. Chạy song song mà không xóa dữ liệu nhau

Một worker phù hợp lab nhỏ nhưng trở thành nút thắt khi suite lớn. Thiết kế mỗi worker có tenant/database riêng hoặc dữ liệu gắn run ID, account riêng và cleanup theo phạm vi. Reset toàn DB chỉ an toàn khi worker sở hữu độc quyền DB. Browser context tách biệt không làm DB tách biệt; hai bài test có thể vẫn tranh cùng tài xế.

### 8.3. System testing phi chức năng

Cùng môi trường có thể kiểm accessibility, compatibility, resilience hay performance nhưng cần mục tiêu và oracle riêng. Ba browser tests hiện tại không đo p95/p99, không chứng minh usability hay WCAG. Một người thật hoàn thành nhiệm vụ còn phụ thuộc khả năng hiểu UI, không chỉ locator click được nút. Các phần đó có bài riêng trong lộ trình.

### 8.4. Production parity và triển khai

Server từ TypeScript source không chứng minh container/build artifact production, migration hay reverse proxy đúng. Bước tiếp theo trên staging là deploy artifact được định danh, chạy luồng với payment sandbox và cấu hình tương đương, giữ dữ liệu tổng hợp. Tránh gọi test staging “giống production” nếu bỏ rate limit, auth hoặc cache mà không ghi khác biệt. Bài Deployment Test sẽ tập trung khoảng trống này.

### 8.5. Chọn hành trình theo rủi ro và khả năng chẩn đoán

Giữ vài hành trình kiếm tiền/huỷ/khôi phục quan trọng ở browser; đẩy tổ hợp rộng xuống API/component. Khi một kịch bản dài có nhiều điểm lỗi độc lập, tách tại ranh giới mục tiêu để failure dễ giải thích. Tuy nhiên đừng dùng API setup để bỏ qua chính bước UI mà mục tiêu đang cần kiểm. Với bài này, tạo trip qua API sẽ làm mất bằng chứng nút Đặt chuyến hoạt động.

## 9. Anti-pattern và giới hạn

Không dùng nth(0) để nhận bất kỳ chuyến nào vì có thể kiểm nhầm dữ liệu. Không mock toàn bộ API rồi gọi là đã chạy toàn hệ thống backend. Không lấy trạng thái từ biến JS nội bộ thay vì quan sát giao diện công khai. Không coi nút disabled là đủ bảo vệ server; API vẫn cần kiểm quyền và transition riêng.

Chỉ Chrome local được chạy. Chưa kiểm Safari/Firefox/mobile, GPS, đăng nhập JWT, provider thật, mạng chậm, server restart, concurrency hoặc mọi field invalidation. Clock là adjustable chạy tiếp, không đóng băng tuyệt đối. PGlite trong phiên không chứng minh dữ liệu bền sau crash. Ba case pass phản ánh ba hành trình trong ranh giới đã mô tả, không thay hồ sơ nghiệm thu.

## 10. Bài tập và đáp án định hướng

1. **Cơ bản:** sau báo giá, sửa voucher. Expected quote-result ẩn và book-button disabled theo R-UI-01. Giữ test kiểm input event thực sự, không gọi invalidateQuote trực tiếp.
2. **Trung bình:** nhập mã khách không tồn tại rồi báo giá. Expected API 401 và thông báo lỗi, không hiện quote hợp lệ. Đây là danh tính mô phỏng, chưa kiểm credential thật.
3. **Trung bình:** cho tài xế huỷ nhận rồi tài xế khác nhận lại. Đáp án cần trạng thái searching sau huỷ, tài xế cũ online, khách thấy tài xế mới; phân biệt driver cancel với customer cancel.
4. **Nâng cao:** chuyển suite sang hai worker. Đáp án cần hai server/DB hoặc chiến lược namespace rõ; chỉ đổi workers=2 trong config hiện tại là chưa đủ vì beforeEach reset toàn DB.
5. **Tự kiểm:** API paid có chứng minh người dùng thấy hoàn thành? Không. UI “Hoàn thành” có chứng minh provider thu tiền đúng? Không. Cần kết hợp bằng chứng và chỉ kết luận trong phạm vi quan sát.

## 11. Nguồn tham khảo

- [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf), mục 2.2.1: phạm vi các test level; ví dụ Tới Nơi là thiết kế riêng của series.
- [Playwright Web server](https://playwright.dev/docs/test-webserver): quản lý server và reuseExistingServer.
- [Playwright Assertions](https://playwright.dev/docs/test-assertions): assertion chờ điều kiện và khác biệt với assertion giá trị tức thời.
- SPEC/README lab, `src/server.ts`, `src/web/customer.js`, `src/web/driver.js`: basis, cấu hình và hành vi thực tế để truy vết.

Nguồn ngoài đối chiếu ngày 13/11/2026. Bài kế tiếp: Regression Test — lựa chọn và duy trì bằng chứng sau mỗi thay đổi.

# SYS-06 — Regression Test: giữ hành vi đúng sau mỗi thay đổi

Trạng thái: bản nháp hoàn chỉnh để review, có lab và thí nghiệm lỗi giả lập. Bài trước: [System Test](05-system-test.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Tình huống và mục tiêu

Nhóm muốn gom cách xử lý phụ phí trong công thức cước. Một bản refactor giả định đưa phí mưa vào cước cơ bản cùng phí khối lượng bưu kiện. Với 5 km giờ thường, giá vẫn đúng 43.000 khi mưa. Đến cao điểm, phí mưa bị nhân 1,25 và cước thành 54.000 thay vì 52.000. Chỉ kiểm ca từng dùng để review refactor sẽ bỏ sót tương tác này.

Đây là tình huống học tập, không phải lỗi vừa được phát hiện trong `src/`. Lab giữ nguyên sản phẩm và đặt candidate sai riêng trong file test. Sau bài, bạn có thể lập impact map, chọn regression theo rủi ro, phân biệt expected change với regression, kiểm sức phát hiện lỗi của suite và duy trì gate đáng tin cậy.

Điều kiện tiên quyết: đã đọc bài Functional/Integration, biết test oracle, assertions và cách chạy Vitest. Phần nâng cao bàn về selection, flakiness, mutation và baseline; không yêu cầu đã vận hành CI lớn.

## 2. Khái niệm và phân biệt

Regression testing kiểm xem thay đổi có gây hậu quả bất lợi cho hành vi trước đó hay không. Tác nhân có thể là code, cấu hình, schema, dependency, môi trường hoặc dữ liệu. Những test chức năng đã có trở thành regression khi được dùng để đánh giá ảnh hưởng sau thay đổi; không cần viết lại thành một loại cú pháp khác.

| Hoạt động | Mục tiêu | Ví dụ |
|---|---|---|
| Confirmation | Kiểm lỗi đã báo được sửa | Mã có khoảng trắng dùng được sau sửa trim |
| Sanity | Kiểm tập trung thay đổi và vùng gần | Trim, mã rỗng và consumer của mã chuẩn |
| Regression | Phát hiện hành vi khác bị hỏng | Refactor cước làm sai phí giao đồ ăn |
| Smoke | Xác định build/môi trường đủ hoạt động để kiểm tiếp | Có thể đặt và hoàn tất chuyến |

Confirmation pass không chứng minh vùng khác an toàn. Regression cũng không nhất thiết chạy mọi case: có thể chọn theo impact và risk, nhưng phải công bố những gì bỏ lại. “Full regression” chỉ có nghĩa đầy đủ **suite được định nghĩa**, không phải kiểm mọi hành vi có thể xảy ra của sản phẩm.

Nếu yêu cầu đổi giá mở cửa, giá mới khác baseline chưa chắc là defect. Cần đối chiếu thay đổi được chấp nhận: hành vi nào được phép đổi, hành vi nào phải giữ. Refactor trong bài không đổi SPEC, nên mọi expected nghiệp vụ giữ nguyên.

## 3. Impact analysis trước khi chọn test

Đọc diff giả định và lần theo callers: `calculateFare` dùng trực tiếp cho ride/parcel và được `priceFood` dùng để tính deliveryFee. Giá còn được snapshot vào quote/trip rồi chuyển tới thanh toán. Ảnh hưởng không dừng ở file vừa sửa.

| Vùng ảnh hưởng | Rủi ro | Suite nên chọn và lý do |
|---|---|---|
| Công thức base/peak/rain | Sai thứ tự cộng/nhân | RG-01…04 phân biệt các điều kiện |
| Voucher | Giảm trên sai số tiền | RG-06 và FN-02/04/05 của SYS-03 |
| Bưu kiện | Nhầm phí khối lượng với phí mưa | RG-07 có cả hai khoản |
| Đồ ăn | Áp voucher nhầm phí giao | RG-08 kiểm riêng tiền món/phí giao |
| Snapshot giá chuyến | Giá đặt khác quote | FN-06 của SYS-03 |
| Lưu SQL và thu ví | Mapping/amount sai khi đi qua adapter | SYS-04 và smoke SYS-01 |
| Hiển thị/luồng người dùng | UI không hiển thị đúng kết quả cuối | SYS-05, chạy riêng qua Playwright |

Đây là mapping thiết kế thủ công, không phải hệ thống tự phân tích dependency đã triển khai. Mở rộng suite nếu diff chạm schema, auth hoặc dùng chung tiền tệ. Nếu không xác định impact chắc chắn, chạy rộng hơn; “không có test được chọn” không được tự diễn giải là pass.

## 4. Baseline và oracle không phụ thuộc implementation

Test basis là SPEC: R-FARE-03/05 quy định mưa cộng sau cao điểm; R-PARCEL-04 đưa phí khối lượng vào base; R-FOOD-04/05 tính voucher trên tiền món, không giảm phí giao. Dùng số tính tay và mã yêu cầu, không gọi phiên bản cũ để định nghĩa mọi expected: baseline cũ có thể chứa bug.

Với bike 5 km, base 34.500. Cao điểm không mưa: 43.125 → làm tròn 44.000. Cao điểm có mưa: 43.125 + 8.000 = 51.125 → 52.000. Candidate sai tính (34.500 + 8.000) × 1,25 = 53.125 → 54.000. Ở giờ thường, cả hai cùng ra 43.000, cho thấy một case đúng chưa đủ phân biệt.

Với bưu kiện 6 kg, phụ phí khối lượng 2.000: (34.500 + 2.000) × 1,25 + 8.000 = 53.625 → 54.000. Đây lại là giá **đúng** cho bưu kiện; không được lấy expected của ride áp cho parcel chỉ vì cùng 5 km.

Với food, tiền món 60.000, FOOD15 giảm 9.000; phí giao cao điểm/mưa 52.000 không áp voucher; tổng 103.000. Assert các thành phần để phát hiện lỗi bị tổng tiền che lấp.

## 5. Ma trận test và fault fixture

| Case | Input/hành động | Expected | Vai trò |
|---|---|---|---|
| RG-01 | Bike 5 km, thường, khô | 35.000 | Baseline không phụ phí |
| RG-02 | Bike 5 km, thường, mưa | 43.000 | Phụ phí mưa độc lập |
| RG-03 | Bike 5 km, cao điểm, khô | 44.000 | Hệ số độc lập |
| RG-04 | Bike 5 km, cao điểm, mưa | 52.000 | Tương tác bắt lỗi refactor |
| RG-05 | Chạy fixture sai với thường/mưa và cao điểm/mưa | 43.000 ở ca đầu; 54.000 khác oracle 52.000 ở ca sau; hàm thật 52.000 | Thí nghiệm khả năng phân biệt |
| RG-06 | API ride 7,4 km, cao điểm/mưa/GIAM20 | Trước giảm 64.625; giảm 12.925; tổng 52.000 | R-FARE-09 oracle chuẩn |
| RG-07 | API parcel 6 kg, bike 5 km, cao điểm/mưa | Phụ phí 2.000, afterPeak 45.625, total 54.000 | Consumer parcel |
| RG-08 | API food 60.000, 5 km, cao điểm/mưa/FOOD15 | Giảm món 9.000, phí giao 52.000, tổng 103.000; fare.discount=0 | Consumer food |

Pure tests không có state bên ngoài. Mỗi API execution có system/app memory mới, clock 11:30 VN 04/11/2026, SG đang mưa, C001 và voucher seed chưa dùng. Không có DB dùng chung cần cleanup; instance không chia sẻ giữa cases. Fixture nằm riêng trong test, không thay `src/`.

RG-05 xanh vì assertion yêu cầu nhận ra output sai của fixture. Không báo rằng “đã chạy build lỗi đỏ rồi sửa sản phẩm xanh”, và không gọi đây là điểm mutation score. Thí nghiệm chỉ minh họa một fault cụ thể bị oracle phân biệt; chưa chứng minh mọi lỗi được phát hiện.

## 6. Thực hành và lựa chọn phạm vi chạy

Mã: `examples/toi-noi-testing/articles/system-testing-regression-test/regression.test.ts`. Từ thư mục lab, Node >=22.12:

```sh
npm ci
npm test -- articles/system-testing-regression-test
npm run typecheck
```

Expected: tám tests pass. Chỉ cần `npm ci` khi chuẩn bị dependency. Để kiểm rộng hơn các bài API đã viết:

```sh
npm test -- articles/system-testing-smoke-test articles/system-testing-sanity-test articles/system-testing-functional-test articles/system-testing-integration-test articles/system-testing-regression-test
```

Expected 50 tests: 3 + 8 + 24 + 7 + 8. Đây là nhóm SYS-01…04/06, không phải toàn bộ regression repository. Browser SYS-05 dùng `npx playwright test --config system-testing.config.ts` và cần Chrome; Vitest cố ý không chạy browser specs.

Khi xem RG-05, hãy đọc assertion đầu và cuối: test nhỏ thường/mưa chấp nhận cả candidate sai lẫn hàm hiện tại, còn ca tương tác tách chúng. Bài học là chọn input có khả năng làm lộ fault, không chỉ tăng số case gần giống nhau. API cases bổ sung consumer khác để tránh suite chỉ xoay quanh ride.

## 7. Gate, báo cáo và xử lý failure

Entry: yêu cầu thay đổi và hành vi được giữ rõ ràng, test basis xác định, môi trường tái hiện được. Exit của bài: tám test mới và suite được chọn pass, typecheck đạt, phạm vi chưa chạy ghi trong [báo cáo](06-test-report.md). Một suite xanh không tự là quyết định release.

| Failure | Phân biệt | Hành động |
|---|---|---|
| Expected khác sau đổi yêu cầu | Thay đổi có chủ đích hay regression? | Đối chiếu yêu cầu phê duyệt, cập nhật oracle có lý do |
| Chỉ consumer food fail | Shared function hoặc cách truyền voucher | So itemsDiscount/deliveryFee, không chỉ total |
| Chạy lại mới pass | Flaky test, môi trường hay race sản phẩm? | Giữ failure đầu, trace/log/seed và điều tra |
| Nhiều case fail cùng lỗi setup | Môi trường hoặc fixture chung | Phân loại blocked; không tạo hàng loạt defect trùng |
| Fault fixture không bị phân biệt | Oracle yếu hoặc input không kích hoạt fault | Chọn ca tương tác, bổ sung assertion đúng basis |

Defect nên ghi commit tốt/xấu nếu biết, case ID, input, expected/actual, cấu hình và bước tái hiện tối thiểu. Nếu dùng git bisect để tìm commit gây lỗi, mỗi lượt phải có môi trường và dữ liệu nhất quán; test flaky sẽ khiến kết quả bisect không đáng tin.

## 8. Quản lý suite lâu dài

### 8.1. Các tầng chạy trong CI

Một thiết kế thực tiễn: mỗi PR chạy component/API theo impact cộng smoke nền; trước merge/release chạy nhóm integration quan trọng; định kỳ và trước release chạy regression rộng/browser theo rủi ro. Đây là đề xuất pipeline, chưa thêm workflow mới vào repo. Tần suất phụ thuộc chi phí và hậu quả bỏ sót, không có quy tắc “nightly là đủ” cho mọi dự án.

### 8.2. Flaky quarantine có trách nhiệm

Quarantine tách test bất ổn khỏi gate chính để tránh chặn sai, nhưng đồng thời tạo lỗ hổng coverage. Mỗi test cần owner, issue, thời hạn xử lý, lý do và phương án kiểm bù. Không đưa test payment quan trọng ra khỏi gate rồi báo coverage đầy đủ. Retry giúp thu evidence nhưng không biến failure đầu thành chưa từng tồn tại.

### 8.3. Bảo trì và xóa test

Gộp case trùng chỉ khi chúng bảo vệ cùng rủi ro và giữ oracle quan trọng. Không xóa test cũ đơn giản vì chưa fail lâu: có thể nó đang bảo vệ hành vi đã ổn định. Khi chức năng bị loại bỏ theo yêu cầu, cập nhật traceability và ghi lý do xóa. Tránh snapshot lớn không chọn field: chúng làm tăng review noise và dễ dẫn tới chấp nhận baseline mới hàng loạt.

## 9. Mở rộng nâng cao

### 9.1. Selection, prioritization và minimization

Selection chọn test cần chạy; prioritization đổi thứ tự để nhận tín hiệu quan trọng sớm; minimization giảm suite bằng loại trùng. Chúng là ba quyết định khác nhau. Coverage-based selection có thể bỏ tác động qua cấu hình, schema hoặc dependency động; kết hợp mapping yêu cầu, lịch sử lỗi và suite nền. Không dùng dữ liệu thời gian chạy đơn lẻ để loại một case chậm nhưng bảo vệ rủi ro tiền cao.

### 9.2. Mutation testing

Mutation tạo thay đổi nhỏ để đánh giá test có phát hiện không. Stryker phân biệt killed, survived, no coverage, timeout và lỗi compile/runtime. Phân tích mutant sống sót trước khi thêm test: có thể thiếu oracle, input không kích hoạt hoặc mutation tương đương. Fault fixture của bài chỉ là ví dụ thủ công, không phải một lượt Stryker. Không báo mutation score khi chưa có tập mutant và kết quả công cụ thực đo.

### 9.3. Differential testing khi thay thuật toán

Chạy bản cũ và mới trên cùng bộ dữ liệu có thể tìm khác biệt, nhưng không quyết định bên nào đúng. Phân loại expected change theo SPEC, lỗi cũ cần sửa, hay regression mới. Đối với tiền, dùng dữ liệu biên, tổ hợp và oracle tính độc lập; tránh mặc định mọi output cũ đều là chuẩn vàng.

### 9.4. Regression phi chức năng

Sau đổi thuật toán, chức năng vẫn đúng nhưng latency hoặc memory có thể xấu đi. Regression hiệu năng cần workload, cấu hình máy, warm-up, mẫu đo và ngưỡng biến động chấp nhận. Không so 127 ms với 160 ms của hai lần chạy unit suite rồi kết luận sản phẩm chậm. Security/accessibility cũng cần baseline và tiêu chí riêng, được triển khai ở các bài tương ứng.

### 9.5. Metrics không đánh lừa

Theo dõi escaped defects, thời gian phát hiện, tỷ lệ flake, thời gian điều tra và coverage yêu cầu có giá trị hơn chỉ đếm test. Tỷ lệ pass phải có mẫu số và số skip/quarantine. Một gate 100% pass trên mười ca được chọn không tương đương 100% pass trên bộ release gồm cả nghìn ca. Báo cáo bài này chỉ ghi nhóm đã chạy.

## 10. Anti-pattern và giới hạn

Chỉ chạy test cạnh file sửa bỏ qua shared consumers. Cập nhật expected theo output mới để xanh làm mất oracle. Copy implementation cũ làm expected đóng băng cả bug cũ. Gọi mọi failure là regression sản phẩm che lỗi môi trường. Dùng số test lớn thay phân tích rủi ro tạo cảm giác an toàn không có căn cứ.

Lab chưa sửa sản phẩm, chưa kiểm mọi consumer, chưa chạy mutation engine, performance regression hoặc lựa chọn tự động theo diff. API cases dùng memory và fake; bằng chứng SQL/browser nằm ở suite riêng. Không có tuyên bố rằng candidate sai đã được deploy hoặc bug thật đã được sửa.

## 11. Bài tập và đáp án định hướng

1. **Cơ bản:** tại sao RG-02 không bắt lỗi fixture? Vì không cao điểm, chuyển vị trí cộng mưa chưa đổi tổng; cần RG-04 để kích hoạt tương tác.
2. **Trung bình:** refactor ảnh hưởng xe car, chọn thêm case gì? Car 5 km cao điểm/mưa: base 80.000, cao điểm 100.000, cộng mưa 108.000. Kiểm thêm voucher và sàn theo risk, không chỉ thay vehicle mà giữ oracle bike.
3. **Trung bình:** một test food flaky, có nên xóa khỏi gate? Đáp án cần phân loại nguyên nhân, owner/thời hạn và kiểm bù nếu quarantine; không chấp nhận âm thầm bỏ coverage.
4. **Nâng cao:** thiết kế mapping cho thay đổi `money.ts`. Phải mở rộng tới ride, parcel, food, voucher và phí huỷ; chỉ chạy RG-01…04 chưa đủ. Đề xuất thứ tự chạy theo hậu quả sai tiền và khả năng phát hiện sớm.
5. **Tự kiểm:** tám tests xanh có nghĩa mutation score 100% không? Không. Một candidate sai bị phát hiện không đại diện mọi mutation. Output khác baseline có luôn là defect? Không, cần đối chiếu yêu cầu thay đổi.

## 12. Nguồn tham khảo

- [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf), mục 2.2.3: confirmation và regression; chiến lược suite ở đây là thực hành riêng của series.
- [Stryker — Mutant states and metrics](https://stryker-mutator.io/docs/mutation-testing-elements/mutant-states-and-metrics/): diễn giải trạng thái mutation và mẫu số của metric.
- SPEC lab cùng `src/domain/fare.ts`, `food.ts`, `parcel.ts`: yêu cầu và các consumer được dùng trong impact map.

Nguồn ngoài đối chiếu ngày 13/11/2026. Bài kế tiếp: Performance Test — đặt mục tiêu, mô hình workload và đo kết quả có căn cứ.

# SYS-03 — Functional Test: biến yêu cầu thành bằng chứng hành vi

Trạng thái: bản nháp hoàn chỉnh để review, có lab tự động. Bài trước: [Sanity Test](02-sanity-test.md). [Lộ trình 16 bài](../system-testing-roadmap.md).

## 1. Bài toán và mục tiêu học tập

Một khách đặt xe máy 5 km thấy cước đúng vào buổi sáng. Đến giờ cao điểm có mưa, hệ thống cộng tiền mưa trước khi nhân hệ số. Trang vẫn hoạt động, request trả 200, nhưng khách trả thừa. Smoke có thể xanh, sanity cho một thay đổi voucher cũng có thể xanh; cần thiết kế phép kiểm từ quy tắc chức năng để phát hiện tổ hợp sai này.

Bài dành cho người đã biết HTTP, JSON và assertion cơ bản. Sau bài, bạn có thể tách yêu cầu thành điều kiện kiểm thử, chọn phân vùng và dữ liệu biên, lập bảng quyết định có oracle độc lập, kiểm chuyển trạng thái và giải thích phần phạm vi chưa được phủ. Lab tập trung cước xe máy và một phần vòng đời chuyến Tới Nơi, không tuyên bố phủ toàn bộ chức năng sản phẩm.

## 2. Functional testing là gì?

Kiểm thử chức năng đánh giá hệ thống thực hiện chức năng theo yêu cầu hay không: nhận dữ liệu nào, áp dụng quy tắc nào, trả kết quả gì và thay đổi trạng thái ra sao. “Tạo chuyến thành công” chỉ là một phần; từ chối dữ liệu sai, bảo toàn giá đã báo và không chuyển trạng thái trái phép cũng là hành vi chức năng.

Functional là hướng kiểm tra, không phải một cấp kiểm thử riêng. Có thể kiểm công thức tại component, hợp đồng dữ liệu tại integration hoặc hành trình toàn hệ thống. API testing chỉ nói về giao diện được tác động; một API test có thể kiểm chức năng, bảo mật hoặc hiệu năng tùy mục tiêu.

| Khái niệm | Câu hỏi chính | Ví dụ |
|---|---|---|
| Functional | Hành vi có đúng yêu cầu? | Cước có cộng mưa sau cao điểm? |
| Performance | Hành vi đáp ứng nhanh và ổn định dưới tải nào? | p95 báo giá ở workload xác định |
| Usability | Người dùng có thực hiện nhiệm vụ thuận lợi? | Có hiểu vì sao voucher bị từ chối? |
| Regression | Sau thay đổi, hành vi đã đúng có bị ảnh hưởng? | Sửa voucher có làm sai cước tối thiểu? |
| UAT | Giải pháp có đáp ứng nhu cầu và tiêu chí nghiệm thu? | Đại diện nghiệp vụ chấp nhận quy trình đặt xe |

Cùng một ca tính cước có thể được chạy lần đầu để kiểm chức năng, sau đó được chọn vào regression. Các nhãn không loại trừ nhau. Functional testing cũng không đồng nghĩa chỉ happy path hay chỉ kiểm qua UI.

## 3. Từ test basis tới điều kiện kiểm thử

Nguồn trực tiếp là `examples/toi-noi-testing/SPEC.md`. Trước khi viết test, trích điều kiện có thể kiểm và điều gì sẽ sai nếu vi phạm:

| Basis | Điều kiện | Rủi ro |
|---|---|---|
| R-FARE-02 | Làm tròn 0,1 km half-up trước kiểm khoảng [0,5;100] | Từ chối nhầm quãng đường hoặc nhận quá giới hạn |
| R-FARE-03/06/08 | Cao điểm → mưa → voucher → tối thiểu → làm tròn tiền | Thu thừa/thiếu tiền |
| R-FARE-07 | Tổng không dưới cước tối thiểu | Giá bất hợp lệ khi giảm sâu |
| R-VOUCHER-06 | Tiền giảm không vượt trần | Vượt ngân sách khuyến mãi |
| R-VALID-01 | Báo lỗi đầu tiên theo thứ tự quy định | Client nhận lỗi không nhất quán |
| R-QUOTE-04 | Chuyến giữ snapshot báo giá | Giá thay đổi sau khi khách đã đồng ý |
| R-STATE-02/03 | Chỉ chuyển trạng thái hợp lệ; lỗi giữ nguyên chuyến | Hoàn thành chuyến khi chưa chở khách |

Điều kiện kiểm thử chưa phải test case. “Có cước tối thiểu” cần thêm dữ liệu làm số sau giảm thực sự thấp hơn sàn. Test 5 km với GIAM20 không đi vào nhánh đó; dùng 0,5 km mới tạo được tình huống cần kiểm.

Oracle là căn cứ xác định đúng/sai. Bài dùng giá tính tay từ SPEC, trạng thái quy định và mã lỗi cụ thể. Không lấy chính `calculateFare` để tạo expected cho API vì cùng lỗi công thức có thể khiến cả hai phía đồng ý sai.

## 4. Phân vùng và biên: kiểm thứ tự chuẩn hóa

Với quãng đường, các phân vùng nghiệp vụ sau chuẩn hóa là dưới 0,5; từ 0,5 đến 100; trên 100. Sai kiểu như chuỗi `"5"` thuộc vấn đề định dạng ở API, không cùng nhóm lỗi ngoài khoảng. Vì SPEC làm tròn trước validation, cần kiểm input thô quanh điểm chuyển do làm tròn.

| Input thô km | Sau chuẩn hóa | Expected |
|---:|---:|---|
| 0,44 | 0,4 | 422 DISTANCE_OUT_OF_RANGE |
| 0,45 | 0,5 | 200; distanceKm=0,5 |
| 0,5 | 0,5 | 200 |
| 5 | 5 | 200; điểm đại diện bên trong |
| 7,35 | 7,4 | 200; kiểm half-up |
| 100 | 100 | 200 |
| 100,04 | 100 | 200 |
| 100,05 | 100,1 | 422 DISTANCE_OUT_OF_RANGE |

Nếu kiểm khoảng trước làm tròn, 0,45 và 100,04 có thể bị từ chối sai. Tám dataset này là lựa chọn biên và điểm đại diện theo rủi ro, không phải tuyên bố bao phủ mọi số thực hoặc toàn bộ một tiêu chuẩn BVA. Muốn tính coverage biên phải định nghĩa rõ miền độ chính xác đầu vào và tập boundary items.

Một ca sai nhiều field khó chỉ ra field gây lỗi. Trước tiên dùng ca một lỗi; sau đó thêm ca nhiều lỗi để kiểm **thứ tự** trả lỗi. Với cityCode=XX và distance=0,1, SPEC yêu cầu CITY_NOT_SUPPORTED. Với distance=0,1 và mã voucher không tồn tại, expected là DISTANCE_OUT_OF_RANGE.

## 5. Bảng quyết định: tương tác cao điểm, mưa và voucher

Cố định xe máy 5 km tại SG, voucher GIAM20 hợp lệ, chưa tiêu lượt. Cước cơ bản 12.000 + 4.500 × 5 = 34.500. Có ba điều kiện Boolean, mọi tổ hợp đều khả thi; tám dòng dưới là toàn bộ tổ hợp của **mô hình nhỏ này**.

| Cao điểm | Mưa | Voucher | Trước giảm | Giảm | Tổng cuối |
|---|---|---|---:|---:|---:|
| Không | Không | Không | 34.500 | 0 | 35.000 |
| Không | Không | Có | 34.500 | 6.900 | 28.000 |
| Không | Có | Không | 42.500 | 0 | 43.000 |
| Không | Có | Có | 42.500 | 8.500 | 34.000 |
| Có | Không | Không | 43.125 | 0 | 44.000 |
| Có | Không | Có | 43.125 | 8.625 | 35.000 |
| Có | Có | Không | 51.125 | 0 | 52.000 |
| Có | Có | Có | 51.125 | 10.225 | 41.000 |

Dòng cuối: 34.500 × 1,25 + 8.000 = 51.125; giảm 20% là 10.225, còn 40.900, làm tròn lên 41.000. Nếu nhân cả mưa, trước giảm thành 53.125. Kiểm `beforeDiscount` giúp thấy sai ở đâu, thay vì chỉ biết tổng cuối lệch.

Tám dòng không phủ trường hợp voucher hết hạn, vượt lượt, trần giảm hoặc sàn cước. Lab bổ sung hai ca: 0,5 km + GIAM20 cho afterDiscount=11.400 nhưng total=15.000; 100 km + GIAM20 cho cước 462.000, giảm bị chặn ở 30.000, tổng 432.000. Những quyết định khác như car, parcel, food cần mô hình riêng.

## 6. Trạng thái và tác dụng phụ

Chức năng có trạng thái phải kiểm cả kết quả hiện tại lẫn ảnh hưởng sau thao tác. Ví dụ chuyến đã accepted không được complete ngay. Case hợp lệ nhận chuyến là bước chuẩn bị; sau lệnh complete sai phải có 409 INVALID_TRANSITION, đọc lại vẫn accepted và không phát sinh gọi ví.

Nếu chỉ assert 409, một implementation có thể đã sửa trạng thái rồi mới trả lỗi. Đọc lại giúp phát hiện lỗi “thất bại nhưng vẫn ghi dữ liệu”. Case này dùng đúng tài xế đã nhận; nếu dùng tài xế khác, lỗi phân quyền có thể chặn trước và không thực sự kiểm transition.

Với báo giá, tạo quote lúc không mưa rồi đổi thời tiết thành mưa trước khi book. Chuyến phải giữ 35.000 và raining=false, trong khi quote mới phải là 43.000 và raining=true. Assertion thứ hai xác nhận thay đổi thời tiết đã có tác dụng; nếu fake weather không đổi, assertion snapshot một mình có thể xanh vô nghĩa.

## 7. Lab và ma trận truy vết

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-functional-test/functional.test.ts`. Mỗi execution tạo system, app và clock mới. Dữ liệu mặc định: C001, tài xế D101, SG, bike, 09:00 VN ngày 04/11/2026, không mưa. Dòng cao điểm đổi clock sang 11:30; dòng mưa bật SG. Không có dữ liệu bền ngoài tiến trình cần cleanup.

| Nhóm | Basis | Hành động | Expected / số execution |
|---|---|---|---|
| FN-01 | R-FARE-02 | POST báo giá với tám distance | Chuẩn hóa hoặc lỗi theo bảng biên / 8 |
| FN-02 | R-FARE-03/06/08 | POST báo giá theo tám tổ hợp | beforeDiscount, discount, total theo bảng / 8 |
| FN-03 | R-VALID-01 | Sai kiểu, sai city, hai tình huống nhiều lỗi | 400/422 và code cụ thể / 4 |
| FN-04 | R-FARE-07 | 0,5 km + GIAM20 | 15.000 sau áp sàn / 1 |
| FN-05 | R-VOUCHER-06 | 100 km + GIAM20 | Giảm 30.000, tổng 432.000 / 1 |
| FN-06 | R-QUOTE-04 | Quote → đổi mưa → book → quote mới | Giá snapshot và giá mới khác đúng / 1 |
| FN-07 | R-STATE-02/03 | Book → accept → complete sai → đọc lại | 409, accepted, chưa gọi ví / 1 |

Từ thư mục lab, dùng Node >=22.12:

```sh
npm ci
npm test -- articles/system-testing-functional-test
npm run typecheck
```

Chỉ cài bằng `npm ci` khi cần chuẩn bị dependency theo lockfile. Expected: **24 test executions pass**, không skip. Báo cáo thực đo nằm ở [SYS-03 Test Report](03-test-report.md).

Test gọi Express qua supertest, dùng kho memory và ví/thời tiết giả. Đây là kiểm hành vi qua API trên ứng dụng được lắp ráp, chưa phải một triển khai production được truy cập qua mạng ngoài. Các bảng expected được viết tường minh trong test; đọc được ngay dữ liệu gây lỗi khi một dòng parameterized fail.

## 8. Đọc kết quả và đánh giá coverage

Entry criteria: SPEC được xác định, dữ liệu và clock kiểm soát được, dependency lab cài đúng. Exit của bài tập: 24 executions pass, typecheck đạt, các expected đã đối chiếu bảng; mọi khoảng trống phải được liệt kê. Exit này chỉ đánh giá lab của bài, không thay tiêu chí release.

| Kết quả bất thường | Hướng điều tra |
|---|---|
| 0,45 bị từ chối | Kiểm validation trước hay sau normalize |
| Chỉ dòng vừa mưa vừa cao điểm sai | Kiểm vị trí cộng phụ phí trong công thức |
| Tổng đúng nhưng discount sai | Kiểm làm tròn giữa chừng hoặc sai số bị tổng che lấp |
| Giá chuyến thay đổi khi mưa đổi | Kiểm snapshot và luồng tạo chuyến có tính lại giá |
| Complete trả 409 nhưng trạng thái đổi | Kiểm thứ tự validation, mutation và ranh giới giao dịch |

Coverage mô hình bảng quyết định là 8/8 dòng khả thi đã định nghĩa. Đây không phải 100% chức năng pricing: chưa xét các loại xe/dịch vụ khác, biên giờ, minAmount, hết hạn, lượt dùng và lỗi payment. Requirement có ít nhất một case không đồng nghĩa mọi điều kiện trong requirement đã được kiểm. Báo cáo cần ghi cả số case và khoảng trống, tránh chỉ báo tỷ lệ pass.

## 9. Mở rộng nâng cao

### 9.1. Property và metamorphic testing

Với cùng xe, giờ, thời tiết và không voucher, tăng quãng đường trong miền hợp lệ không nên làm tổng giảm. Đây là quan hệ giữa hai lần chạy, hữu ích khi có nhiều input. Nhưng một hàm luôn trả 15.000 cũng thỏa tính không giảm; phải giữ các oracle ví dụ cụ thể. Với voucher, điều kiện áp dụng/minAmount có thể làm quan hệ thay đổi, cần xác định miền trước khi sinh dữ liệu.

Một invariant khác là tổng tiền nguyên và chia hết cho 1.000 đối với cước ride trong SPEC hiện tại. Không áp dụng tùy tiện cho mọi tiền trung gian hoặc tổng đồ ăn: quy tắc làm tròn có phạm vi riêng. Có thể dùng fast-check trong lab để sinh distance, lưu seed và counterexample để tái hiện; phần này là bài mở rộng chưa chạy trong suite hiện tại.

### 9.2. Model-based testing và oracle độc lập

Từ bảng trạng thái trong SPEC có thể lập mô hình riêng rồi sinh chuỗi accept/arrive/pickup/start/complete/cancel. Sau mỗi lệnh so trạng thái quan sát được với mô hình. Không import bảng TRANSITIONS của implementation làm oracle: nếu bảng sản phẩm sai, model cũng sai theo. Cần bao phủ transition hợp lệ và không hợp lệ; state coverage một mình chưa chứng minh các cạnh đúng.

### 9.3. Tổ hợp lớn và pairwise

Khi thêm city, vehicle, kind, voucher eligibility, phương thức thanh toán và thời gian, số tổ hợp tăng nhanh. Có thể dùng pairwise để chọn các tương tác hai yếu tố, nhưng lỗi ở ba yếu tố như cao điểm × mưa × voucher vẫn cần ca chủ đích. Giữ tổ hợp rủi ro tiền cao, loại tổ hợp bất khả thi bằng constraint và công bố mô hình đã giảm; không gọi pairwise là bao phủ toàn bộ.

### 9.4. Mutation testing để đánh giá assertion

Các mutation có ý nghĩa gồm đổi cộng mưa sang trước hệ số, bỏ Math.max sàn giá, hoặc thay giới hạn trần bằng giảm phần trăm không chặn. FN-02, FN-04 và FN-05 lần lượt phải phát hiện chúng. Chạy mutation trên bản sao hoặc cấu hình công cụ phù hợp, không lưu bug vào `src/` dùng chung. Mutation sống sót cần phân tích tương đương hay thiếu oracle; tỷ lệ mutation không thay coverage yêu cầu. Bài này chưa chạy mutation.

### 9.5. Thời gian và concurrency

Clock cố định tạo kết quả lặp lại nhưng chưa kiểm biên 10:59:59/11:00/12:59:59/13:00 hoặc ngày voucher theo UTC+7. Thêm chúng theo R-GEN-02 và R-FARE-04, không phụ thuộc timezone máy. Hai tài xế nhận đồng thời hoặc hai request dùng lượt voucher cần DB/transaction thật; memory suite tuần tự không chứng minh chống race condition.

## 10. Anti-pattern và giới hạn

Kiểm mỗi status 200 bỏ sót sai tiền. Copy công thức implementation để tính expected nhân đôi lỗi. Sinh mọi expected lúc chạy từ response trước đó biến so sánh thành vòng tròn. Snapshot JSON toàn bộ làm thay đổi field không liên quan gây nhiễu. Một test dài phủ quá nhiều chức năng khiến failure đầu tiên che các hành vi sau; bảng dữ liệu độc lập giữ chẩn đoán rõ hơn.

Không gom lỗi môi trường thành lỗi nghiệp vụ và không đổi expected chỉ để xanh. Khi yêu cầu thay đổi, ghi lý do cùng phiên bản test basis rồi cập nhật bảng, test và bài đọc đồng bộ. Suite này chưa kiểm persistence PostgreSQL, UI, external payment, security, performance hoặc toàn bộ functional requirements.

## 11. Bài tập và đáp án định hướng

1. **Cơ bản:** thêm 13:00 VN, không mưa, 5 km. Expected 35.000 vì khoảng cao điểm kết thúc loại trừ 13:00. Thêm 12:59:59 để phân biệt biên; dùng API của clock/date đúng độ chính xác.
2. **Trung bình:** thiết kế ca trần giảm GIAM20 bắt đầu có tác dụng. Trần 30.000 tương ứng A=150.000; với giá xe máy giờ thường, distance sau chuẩn hóa 30,6 km cho A=149.700, giảm 29.940; 30,7 km cho A=150.150, giảm 30.000. Không giả định quãng đường tùy ý có thể tạo chính xác A=150.000 sau bước 0,1 km.
3. **Trung bình:** tại sao FN-06 còn tạo quote mới? Để xác nhận điều kiện môi trường đã đổi, tránh ca snapshot pass do setup không có tác dụng.
4. **Nâng cao:** tạo model trạng thái độc lập từ SPEC và thiết kế chuỗi có cancel. Đáp án cần trạng thái trước/sau, actor hợp lệ, tác dụng phụ tài xế và oracle lỗi; không chỉ gọi nextStatus rồi so với chính TRANSITIONS.
5. **Tự kiểm:** 24/24 pass có phải toàn bộ functional testing xong? Không. Hãy liệt kê ít nhất ba vùng chưa phủ và lý do ưu tiên: food/parcel, biên ngày voucher và tính lượt đồng thời là những ví dụ hợp lệ.

## 12. Tài liệu tham khảo

- [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf), mục 2.2.2 và 4.2: thuật ngữ test types, phân vùng, biên, bảng quyết định và chuyển trạng thái. Ví dụ và chiến lược lab trong bài là thiết kế riêng cho Tới Nơi.
- `examples/toi-noi-testing/SPEC.md`: test basis trực tiếp cho mã R-… và các con số expected.
- `examples/toi-noi-testing/README.md`: cách khởi tạo lab và giới hạn mô phỏng.

Nguồn ngoài đối chiếu ngày 13/11/2026. Bài kế tiếp: Integration Test — kiểm hợp đồng và hành vi khi API, repository và cổng thanh toán phối hợp.

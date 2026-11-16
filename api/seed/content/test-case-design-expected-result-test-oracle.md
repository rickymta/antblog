---
slug: test-case-design-expected-result-test-oracle
title: "Kết quả mong đợi và test oracle: viết expected result kiểm chứng được cho cước và huỷ chuyến"
summary: "Kết quả mong đợi phải bao phủ mọi đầu ra quan sát được: phản hồi HTTP, bảng chi tiết cước, trạng thái lưu, trip_events, trạng thái tài xế, lời gọi ví và khoá idempotency. Các nguồn test oracle, oracle problem, oracle cước độc lập bằng phân số, và hai anti-pattern: chép kết quả từ hệ thống, assert quá lỏng hoặc quá chặt. Lab vitest 38 ca."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-04, expected-result, test-oracle, oracle-problem, idempotency, vitest, toi-noi]
language: vi
---

«Kết quả mong đợi: huỷ thành công, phí đúng.» Dòng này xuất hiện trong rất nhiều bộ test case và gần như vô dụng. «Thành công» là HTTP 200 hay là chuyến đã về `cancelled` trong database? «Phí đúng» là bao nhiêu, đúng theo cái gì? Còn những thứ không nằm trong phản hồi — tài xế đã được trả về `online` chưa, ví VíXanh có bị gọi hai lần không, khoá idempotency gửi sang ví có đúng không — thì dòng đó không nhắc tới. Một test case như vậy đạt cả khi hệ thống trừ tiền khách hai lần. Bài này dạy viết **kết quả mong đợi kiểm chứng được** (verifiable expected result): liệt kê mọi đầu ra quan sát được, chọn **test oracle** phù hợp cho từng đầu ra, và tránh hai cái bẫy phổ biến nhất — chép kết quả từ hệ thống vào expected, và assert quá lỏng hoặc quá chặt.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-04</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System (qua API)</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không phải một kỹ thuật riêng — áp cho kết quả mong đợi của mọi test case</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-giai-phau-test-case">TCD-02 · Giải phẫu test case</a> · <a href="/articles/test-case-design-logic-va-cu-the">TCD-03 · Test case logic và cụ thể</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): oracle cước độc lập bằng phân số BigInt so từng bước của bảng chi tiết (R-FARE-10) với hệ thống trên 16 dòng và 500 mẫu ngẫu nhiên; test huỷ chuyến kiểm phí, <code>trip_events</code>, tài xế, lời gọi ví giả và khoá idempotency — <code>examples/toi-noi-testing/articles/test-case-design-expected-result-test-oracle/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.1, 1.4.3 · FL-1.4.1 (K2), FL-1.4.3 (K2); thuật ngữ <i>test oracle</i>, <i>expected result</i> theo ISTQB Glossary</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một test case có bốn phần cốt lõi: tiền điều kiện, đầu vào, hành động và kết quả mong đợi. Ba phần đầu quyết định *chạy cái gì*; phần thứ tư quyết định *biết được gì* sau khi chạy. Test case có đầu vào hoàn hảo nhưng kết quả mong đợi nghèo nàn thì chạy xong vẫn không biết hệ thống đúng hay sai — chỉ biết nó «không sập».

Vấn đề có hai tầng. Tầng thứ nhất là **phạm vi**: kết quả mong đợi phải phủ những đầu ra nào. Với một lệnh huỷ chuyến của Tới Nơi, đầu ra không chỉ là thân JSON trả về; còn có trạng thái chuyến được lưu, một dòng nhật ký `trip_events`, trạng thái tài xế, và — quan trọng nhất về tiền — lời gọi sang ví VíXanh với số tiền và khoá idempotency. Tầng thứ hai là **nguồn**: con số mong đợi lấy từ đâu. SPEC có ví dụ tính tay cho đúng một trường hợp cước (R-FARE-09); mọi trường hợp khác phải tính lại. Tính bằng gì, ai đảm bảo phép tính đó đúng, và nếu không có cách nào biết con số đúng thì sao? Đó là câu hỏi về **test oracle**.

Bài này tiếp nối [TCD-03](/articles/test-case-design-logic-va-cu-the) (nơi kết quả mong đợi được tính từ SPEC cho bảng đơn đồ ăn) và tổng quát hoá những gì [TEST-041 Random Testing](/articles/random-testing) và [TEST-042 Metamorphic Testing](/articles/metamorphic-testing) đã làm cho riêng thuật toán cước.

## 2. Lý thuyết theo ISTQB

### 2.1. Kết quả mong đợi trong các hoạt động kiểm thử (CTFL v4.0.1 mục 1.4.1, 1.4.3)

CTFL không có một mục riêng cho «cách viết kết quả mong đợi», nhưng kết quả mong đợi xuất hiện xuyên suốt các hoạt động của mục 1.4.1:

- **Test design** triển khai test condition thành test case — mà test case, theo định nghĩa, gồm kết quả mong đợi. Đây là lúc quyết định *đầu ra nào* được kiểm.
- **Test implementation** tạo testware cần để chạy; với kết quả mong đợi, đó có thể là bảng giá trị tính sẵn, hoặc một oracle chạy được (như hàm tính cước độc lập trong lab).
- **Test execution** chạy test và **so sánh kết quả thực tế với kết quả mong đợi**; mọi bất thường được phân tích để tìm nguyên nhân và báo cáo.

Mục 1.4.3 xếp test case (kèm kết quả mong đợi) vào testware của test design, còn test script và test data vào testware của test implementation. Hệ quả thực tế: một bảng kết quả mong đợi là **testware**, phải được review, có phiên bản, truy vết về test basis — như mọi testware khác ([TEST-010](/articles/testware-san-pham-kiem-thu)).

ISTQB Glossary định nghĩa (diễn đạt lại):

- **Expected result** (kết quả mong đợi): hành vi quan sát được mà test item được dự đoán sẽ có trong điều kiện xác định, dựa trên test basis.
- **Test oracle** (còn gọi *oracle*): nguồn dùng để xác định kết quả mong đợi, để so với kết quả thực tế của hệ thống đang kiểm.

Hai chữ cần nhấn: «**quan sát được**» — kết quả mong đợi phải là thứ đo được khi chạy; «**nguồn**» — oracle không nhất thiết là một con người hay một tài liệu, nó có thể là một chương trình, một bản cũ của hệ thống, hay một tính chất.

### 2.2. Đầu ra quan sát được

Một test case kiểm hành vi có trạng thái (stateful) nên liệt kê đầu ra theo năm nhóm:

| Nhóm | Ví dụ ở Tới Nơi | Cách quan sát trong lab |
|---|---|---|
| Phản hồi trực tiếp | mã HTTP, thân JSON, header (`Idempotent-Replayed`) | supertest |
| Giá trị dẫn xuất | bảng chi tiết cước 11 bước (R-FARE-10), `cancellationFee`, `feeReason` | thân phản hồi |
| Trạng thái lưu | chuyến `cancelled`, `cancelledAt`, `paymentStatus`; tài xế `online` | đọc lại qua `GET /api/trips/:id`, repository |
| Nhật ký và sự kiện | dòng `trip_events` (R-STATE-06); trong hệ thống đích: sự kiện Kafka (chưa có trong lab) | `listEvents` của repository |
| Lời gọi ra ngoài | gọi ví VíXanh: số lần, số tiền, khoá idempotency, khách | `FakePaymentGateway.calls`, `totalCharged()` |

Ba nhóm cuối là nơi defect nghiêm trọng hay ẩn: phản hồi nói `cancelled` nhưng database chưa ghi; chuyến huỷ nhưng tài xế kẹt `busy` không nhận được chuyến mới; ví bị gọi lại với khoá mới mỗi lần thử nên khách bị trừ tiền ba lần. Không nhóm nào trong số đó lộ ra nếu kết quả mong đợi chỉ là «200 OK».

Ngược lại, «mọi đầu ra» cũng gồm **những thứ không được xảy ra**: ca từ chối không được ghi thêm dòng nhật ký; phí 0 thì không được gọi ví; gửi lại cùng khoá thì không được gọi ví lần hai. Kết quả mong đợi dạng phủ định («đúng 0 lời gọi») là một phần đầy đủ của test case, không phải chi tiết thừa.

### 2.3. Các nguồn test oracle

| Nguồn | Mô tả | Ở Tới Nơi | Điểm yếu |
|---|---|---|---|
| **Đặc tả** | đọc kết quả thẳng từ test basis | R-FARE-09 = 52.000 đ; R-CANCEL-03 = 10.000/20.000 đ; mã lỗi trong R-API-06 | chỉ có cho vài ví dụ; đặc tả có thể sai hoặc mơ hồ |
| **Tính lại độc lập** | một chương trình (hoặc người) tính lại từ đặc tả bằng cách khác | oracle cước bằng phân số BigInt trong lab | tốn công; nếu «độc lập» nhưng chép code sản phẩm thì sai giống hệt |
| **Bất biến / oracle bộ phận** | tính chất luôn phải đúng, không cần biết giá trị chính xác | cước ≥ tối thiểu, là bội số 1.000; mỗi khoá idempotency chỉ trừ tiền một lần | chỉ bắt được một phần defect (mục 4.5) |
| **Hệ thống so sánh / bản trước** | so với bản cũ, hệ thống khác cùng chức năng, hoặc kết quả đã được duyệt | bảng cước của `pricingVersion` cũ khi refactor không đổi luật | lỗi có sẵn trong bản cũ thành «đúng» |
| **Quan hệ giữa các lần chạy** | metamorphic relation | mưa cộng đúng 8.000 đ vào số tiền trước voucher ([TEST-042](/articles/metamorphic-testing)) | không kiểm từng kết quả riêng lẻ |
| **Phán đoán của con người** | chuyên gia domain nhìn và đánh giá | thông điệp lỗi tiếng Việt có dễ hiểu không; màn hình giải thích phí huỷ | chậm, khó lặp lại, không tự động được |

Không có nguồn nào tốt nhất cho mọi đầu ra. Một test case tốt thường **phối hợp**: mã lỗi lấy từ đặc tả, số tiền lấy từ oracle tính lại, «không trừ hai lần» là bất biến, câu chữ thông báo để người review.

### 2.4. Oracle problem

**Oracle problem** là tình huống không có cách khả thi để biết kết quả đúng — hoặc biết được nhưng quá đắt. Ba dạng gặp ở Tới Nơi:

1. **Không có giá trị đúng.** Kiến trúc đích dự định hệ số giá theo khu vực và thời điểm; nếu một mô hình đề xuất hệ số, không ai biết «đúng» là 1,18 hay 1,21. Lời giải: bất biến và quan hệ (MT).
2. **Có giá trị đúng nhưng tính đắt.** Cước của «ô tô 37,8 km, cao điểm, mưa, voucher 13% trần 21.000 đ» tính tay được, nhưng không ai tính tay 500 trường hợp. Lời giải: oracle tính lại độc lập, tự động.
3. **Đầu ra không quan sát được trọn vẹn.** Khi ví trừ tiền nhưng phản hồi bị mất (`lost_response`), hệ thống ghi `paymentStatus = failed` trong khi tiền đã thật sự bị trừ. Test ở mức API thấy `failed`; chỉ test có quyền nhìn vào phía ví mới thấy tiền. Lời giải: mở rộng điểm quan sát (bản giả có `totalCharged()`), và kiểm bất biến ở phía ví.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Cách dùng trong bài | Ghi chú |
|---|---|---|
| expected result | kết quả mong đợi | không viết «kết quả kỳ vọng» |
| actual result | kết quả thực tế | |
| test oracle | test oracle | ISTQB Glossary |
| oracle problem | oracle problem | không có / quá đắt để có oracle |
| partial oracle | oracle bộ phận | bất biến, kiểm một phần |
| invariant | bất biến | tính chất luôn đúng |
| observable output | đầu ra quan sát được | |
| side effect | tác dụng phụ | thay đổi ngoài phản hồi trực tiếp |
| golden master / approved snapshot | kết quả đã duyệt | khác «kết quả chép từ hệ thống» ở bước duyệt |
| false positive / false negative | báo lỗi nhầm / bỏ lọt lỗi | |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Hai tính năng có rủi ro tiền cao nhất của Tới Nơi là tính cước và huỷ chuyến có phí. Quy tắc dùng trong bài:

| Mã | Quy tắc (tóm tắt) |
|---|---|
| R-GEN-01 | Mọi số tiền tính bằng đồng; bước trung gian chính xác, không làm tròn; chỉ làm tròn ở bước cuối |
| R-FARE-03 | Thứ tự: cơ bản (+ phụ phí bưu kiện) → × 1,25 cao điểm → + 8.000 đ mưa → − voucher → nâng lên tối thiểu → làm tròn lên 1.000 đ |
| R-FARE-09 | Ví dụ: xe máy 7,4 km, cao điểm, mưa, giảm 20% ⇒ 45.300 → 56.625 → 64.625 → 51.700 → **52.000 đ** |
| R-FARE-10 | Báo giá trả bảng chi tiết từng bước; giá trị trung gian có thể lẻ dưới 1 đồng |
| R-CANCEL-01…07 | Phí huỷ theo trạng thái và mốc 2 phút; 10.000 đ xe máy, 20.000 đ ô tô; miễn khi tài xế trễ; lý do `searching`/`grace_period`/`driver_late`/`charged`; phí > 0 thu theo R-PAY-05 và lưu `cancellationFee` |
| R-STATE-06, -07 | Mỗi lần đổi trạng thái ghi một dòng `trip_events`; `cancelledAt` ghi khi huỷ |
| R-ACCEPT-05 | Khách huỷ chuyến đã có tài xế ⇒ tài xế về `online` |
| R-PAY-03 | Chuyến `vixanh` hoàn thành ⇒ thu đúng `fare`, khoá `trip:<id>:fare` |
| R-PAY-04 | Hết giờ 3 giây; hết giờ/lỗi tạm thời ⇒ thử lại tối đa 2 lần **cùng khoá**; bị từ chối ⇒ không thử lại |
| R-PAY-05 | Phí huỷ > 0: `vixanh` ⇒ thu qua ví, khoá `trip:<id>:cancel-fee`; `cash` ⇒ ghi nhận, `paymentStatus = cash`; phí 0 ⇒ `none` |
| R-PAY-06 | Thu tiền thất bại không chặn huỷ: vẫn `cancelled`, `paymentStatus = failed`, HTTP 200 |
| R-IDEM-02 | Gửi lại cùng khoá, cùng nội dung ⇒ cùng mã và thân, header `Idempotent-Replayed: true`, không xử lý lại |

### 3.2. System context

Kiến trúc đích: lệnh huỷ đi qua API Gateway tới **Booking**, **Trip Lifecycle** kiểm chuyển trạng thái, **Payment & Wallet** gọi VíXanh, **Driver Availability** trả tài xế về `online`, và Booking phát sự kiện qua Kafka (chưa có trong lab). Mỗi mũi tên đó là một đầu ra có thể sai độc lập. Trong lab, tất cả gói trong `cancelByCustomer` (`src/app/trips.ts`); ví là `FakePaymentGateway` ghi lại mọi lời gọi; nhật ký đọc qua `listEvents` của repository. Bản giả có chế độ lỗi `decline`, `error`, `timeout`, `lost_response` (R-TEST-03) — chính các chế độ này tạo ra những đầu ra mà test thủ công trên môi trường thật gần như không dựng được.

### 3.3. Rủi ro

| Rủi ro | Hậu quả | Đầu ra phải kiểm |
|---|---|---|
| Thử lại với khoá idempotency mới | trừ tiền khách 2–3 lần | khoá của từng lời gọi; tổng đã trừ |
| Phản hồi nói `cancelled`, database chưa ghi | khách thấy huỷ, tài xế vẫn chạy tới | đọc lại trạng thái đã lưu |
| Tài xế không về `online` | tài xế mất thu nhập, chuyến mới không có người nhận | trạng thái tài xế |
| Cước lệch vài nghìn đồng do sai số số thực | khiếu nại, đối soát lệch | từng bước của bảng chi tiết, so chính xác tuyệt đối |
| Gửi lại lệnh huỷ thu phí lần hai | khiếu nại, hoàn tiền | số lời gọi ví sau khi gửi lại |
| Expected «chép từ hệ thống» | lỗi hiện có thành chuẩn, test xanh mãi | nguồn của kết quả mong đợi |

## 4. Phân tích kỹ thuật

### 4.1. Từ «phí đúng» tới kết quả mong đợi kiểm chứng được

Bốn tiêu chí để một kết quả mong đợi kiểm chứng được:

1. **Cụ thể**: có giá trị hoặc quy tắc tính ra giá trị, không có «hợp lý», «đúng», «thành công».
2. **Có nguồn**: ghi rõ lấy từ mã yêu cầu nào hoặc oracle nào.
3. **Đủ phạm vi**: phủ các nhóm đầu ra ở mục 2.2 mà test condition liên quan, cả khẳng định lẫn phủ định.
4. **Vừa đủ chặt**: so đúng những gì đặc tả quy định, không so những gì đặc tả không quy định (mục 4.6).

Áp vào test case HUY-01 «khách huỷ chuyến xe máy thanh toán VíXanh, 5 phút sau khi tài xế nhận»:

| Bản yếu | Bản kiểm chứng được |
|---|---|
| Huỷ thành công, phí đúng, tiền được thu | (1) HTTP 200; `cancellationFee` = 10.000, `feeReason` = `charged` (R-CANCEL-01, -03, -06). (2) `trip.status` = `cancelled`, `cancelledBy` = `customer`, `cancelledAt` = thời điểm huỷ theo đồng hồ được inject, `paymentStatus` = `paid`, `fare` không đổi = 35.000 (R-STATE-07, R-PAY-05, R-QUOTE-04). (3) `GET /api/trips/:id` trả đúng chuyến như phản hồi huỷ. (4) `trip_events` = `requested`, `searching`, `accepted`, `cancelled`; dòng cuối có thời điểm = thời điểm huỷ (R-STATE-06). (5) Tài xế `D101` = `online` (R-ACCEPT-05). (6) Ví được gọi **đúng 1 lần**: số tiền 10.000, khoá `trip:<id>:cancel-fee`, khách `C001`; tổng đã trừ = 10.000 (R-PAY-05). |

Bản phải dài gấp mười, nhưng mỗi dòng trong đó ứng với một rủi ro ở mục 3.3. Thiếu dòng (6) thì defect «thử lại với khoá mới» không bị bắt; thiếu dòng (3) thì defect «chưa ghi database» không bị bắt.

Một chi tiết có chủ ý: trường `description` của lời gọi ví («Phí huỷ chuyến») **không** được kiểm. SPEC không quy định nội dung đó; kiểm nó là assert quá chặt — đổi câu chữ thì test đỏ mà không có gì sai.

### 4.2. Ma trận đầu ra × nguồn oracle cho lệnh huỷ

| Đầu ra | Nguồn oracle | Kiểu assert |
|---|---|---|
| HTTP 200 / 409 | đặc tả (R-API-06, R-STATE-03) | bằng |
| `cancellationFee`, `feeReason` | đặc tả (R-CANCEL), hoặc bảng quyết định của [TEST-033](/articles/decision-table-testing) | bằng |
| `cancelledAt` | đồng hồ được inject (`FixedClock`) | bằng với thời điểm biết trước |
| trạng thái lưu | đọc lại, so với phản hồi | bằng toàn bộ đối tượng |
| `trip_events` | đặc tả (R-STATE-06) + đồng hồ | bằng, **theo thứ tự** |
| trạng thái tài xế | đặc tả (R-ACCEPT-05) | bằng |
| lời gọi ví | đặc tả (R-PAY-04, -05) | số lần, khoá, số tiền, khách — không so mô tả |
| tổng tiền đã trừ | bất biến «mỗi khoá trừ một lần» (R-PAY-04, R-NFR-05) | bằng |

### 4.3. Oracle cước độc lập

Với cước, đặc tả chỉ cho một ví dụ. Muốn có kết quả mong đợi cho bảng 16 dòng và cho 500 mẫu ngẫu nhiên, lab viết một **oracle tính lại độc lập**. «Độc lập» có nghĩa cụ thể, kiểm được bằng mắt khi review:

- **Không import gì từ `src/`.** Bảng giá, hệ số 1,25, phụ phí 8.000 đ, tham số voucher chép từ SPEC (R-FARE-01, -04, -05, Phụ lục A). Nếu oracle gọi chính `calculateFare` để tính expected thì nó chỉ kiểm được rằng hàm bằng chính nó.
- **Khác cách biểu diễn số.** Sản phẩm tính bằng số nguyên «milli-đồng» (`src/domain/money.ts`). Oracle dùng **phân số BigInt** (tử số, mẫu số tối giản). Hai cách khác nhau mà cho cùng kết quả là bằng chứng mạnh hơn hai bản sao của cùng một cách.
- **Đọc số thập phân như con người đọc.** Quãng đường 7,35 km được đọc từ chuỗi `"7.35"` thành phân số 735/100 rồi làm tròn half-up bằng số nguyên. Không có bước `× 10` bằng số thực.
- **Trả đủ các bước của R-FARE-10**, để so **từng bước** chứ không chỉ tổng.

```ts
// oracle-cuoc.ts (trích)
const base = openingFee.cong(distanceCharge).cong(parcelSurcharge);   // bước 1
const afterPeak = v.caoDiem ? base.nhan(HE_SO_CAO_DIEM) : base;       // bước 2 — 125/100
const beforeDiscount = afterPeak.cong(rainSurcharge);                 // bước 3
discount = beforeDiscount.nhan(PhanSo.of(vc.percent, 100)).nhoNhat(PhanSo.of(vc.tran)); // bước 4
const atLeast = afterDiscount.lonNhat(minimumFare);                   // bước 5
const total = PhanSo.of(atLeast.nhan(PhanSo.of(1, 1000)).tran() * 1000n); // bước 6
```

So sánh với hệ thống cũng phải chính xác: số trong JSON (ví dụ `33562.5`) được đọc qua dạng chuỗi thành phân số rồi so **bằng tuyệt đối**. Không có `toBeCloseTo`, không có dung sai — với tiền, dung sai một đồng là defect.

**Trước khi dùng oracle để phán xử hệ thống, phải kiểm oracle.** Lab kiểm oracle bằng chính những ví dụ có sẵn trong SPEC: R-FARE-09 (bảng 45.300 → 56.625 → 64.625 → 12.925 → 51.700 → 52.000), ví dụ làm tròn quãng đường của R-FARE-02 (0,45 ⇒ 0,5; 0,44 ⇒ 0,4; 7,35 ⇒ 7,4; 100,05 ⇒ 100,1), ví dụ phụ phí của R-PARCEL-03 và ví dụ làm tròn của R-FARE-08. Chỉ khi oracle qua hết các ví dụ đó, nó mới được tin.

Bảng test case giá trị OR-01…OR-16 (đồng hồ 09:00 giờ VN, hoặc 11:30 với cao điểm; TP. Hồ Chí Minh trừ OR-13, 14 ở Hà Nội). Cột cuối do người viết tính tay, và lab kiểm nó khớp oracle trước khi so với hệ thống:

| Dòng | Đầu vào | Truy vết | Tổng mong đợi |
|---|---|---|---:|
| OR-01 | xe máy 7,4 km, cao điểm, mưa, `GIAM20` | R-FARE-09 | 52.000 |
| OR-02 | xe máy 5 km | R-FARE-01 | 35.000 |
| OR-03 | ô tô 18,6 km, cao điểm | R-FARE-04, R-GEN-01 | 287.000 |
| OR-04 | xe máy 0,45 km | R-FARE-02, -07 | 15.000 |
| OR-05 | xe máy 0,44 km | R-FARE-02 | 422 `DISTANCE_OUT_OF_RANGE` |
| OR-06 | ô tô 100,04 km | R-FARE-02 | 1.125.000 |
| OR-07 | xe máy 3,3 km, cao điểm | R-FARE-04, -10 | 34.000 |
| OR-08 | xe máy 0,5 km, `GIAM20` | R-FARE-07 | 15.000 |
| OR-09 | ô tô 100 km, `GIAM20` | R-VOUCHER-06 | 1.095.000 |
| OR-10 | bưu kiện xe máy 3 km, 6,0 kg | R-PARCEL-03 | 28.000 |
| OR-11 | bưu kiện xe máy 3 km, 6,1 kg, cao điểm | R-PARCEL-04 | 37.000 |
| OR-12 | bưu kiện ô tô 3 km, 30 kg | R-PARCEL-03 | 108.000 |
| OR-13 | Hà Nội, xe máy 5 km, `HN10` | R-VOUCHER-06 | 422 `VOUCHER_MIN_AMOUNT` |
| OR-14 | Hà Nội, ô tô 5 km, `HN10` | R-VOUCHER-06 | 72.000 |
| OR-15 | xe máy 7,35 km, mưa | R-FARE-02, -05 | 54.000 |
| OR-16 | ô tô 18,6 km, cao điểm, `GIAM20` | R-FARE-06, R-VOUCHER-06 | 257.000 |

Kiểm tay ba dòng:

- **OR-07.** 12.000 + 4.500 × 3,3 = 26.850 ⇒ × 1,25 = **33.562,5** (bảng chi tiết phải trả đúng số lẻ này, R-FARE-10) ⇒ làm tròn lên **34.000 đ**.
- **OR-08.** 12.000 + 4.500 × 0,5 = 14.250 ⇒ giảm 20% = 2.850 ⇒ 11.400 < 15.000 ⇒ nâng lên tối thiểu **15.000 đ**. Voucher không kéo cước xuống dưới tối thiểu (R-FARE-07).
- **OR-14.** 25.000 + 11.000 × 5 = 80.000 ≥ 40.000 (tối thiểu của `HN10`) ⇒ giảm 10% = 8.000 ≤ trần 15.000 ⇒ **72.000 đ**. Còn OR-13: xe máy 5 km = 34.500 < 40.000 ⇒ `VOUCHER_MIN_AMOUNT`.

### 4.4. Bẫy số thực: vì sao oracle phải «decimal-safe»

OR-03 có mặt trong bảng vì một lý do: nó là nơi một oracle viết vội bằng số thực sai. Trong JavaScript, `25000 + 11000 * 18.6` cho `229600.00000000003` thay vì 229.600. Nhân 1,25 ra 287.000,00000000006; làm tròn lên 1.000 đ ra **288.000 đ**. Đáp án đúng là 229.600 × 1,25 = **287.000 đ** — đã là bội số 1.000, giữ nguyên. Hệ thống (dùng milli-đồng nguyên) trả 287.000; oracle số thực sẽ báo hệ thống **sai** — một false positive mà người điều tra rất dễ kết luận nhầm là defect của sản phẩm.

Lab quét toàn bộ 996 quãng đường hợp lệ (0,5…100 km, bước 0,1) × 2 loại xe × có/không cao điểm = **3.984 tổ hợp**: oracle số thực sai ở **12** tổ hợp, đều là ô tô giờ cao điểm (18,6; 32,2; 36,2; 40,2; 44,2; 67,4; 71,4; 75,4; 79,4; 83,4; 87,4; 91,4 km). 12/3.984 là 0,3% — đủ hiếm để lọt qua mọi bộ test «vài ví dụ», đủ thường để một bộ test ngẫu nhiên báo đỏ thất thường. Bài học: oracle cho tiền phải dùng số học chính xác (số nguyên đơn vị nhỏ, phân số, hoặc kiểu decimal), và R-GEN-01 viết đúng điều đó cho sản phẩm thì cũng áp cho oracle.

### 4.5. Bất biến là oracle bộ phận

Khi không có (hoặc chưa có) oracle đầy đủ, **bất biến** vẫn bắt được một lớp defect. Ví dụ cho cước: tổng > 0; tổng ≥ cước tối thiểu của loại xe; tổng là bội số 1.000 đ. Cho thanh toán: mỗi khoá idempotency trừ tiền tối đa một lần.

Bất biến có hai giới hạn. Nó **không đủ**: bản lỗi trong lab (`buggy-fare.ts`, phụ phí trời mưa bị nhân 1,25 khi cao điểm) trả OR-01 = 54.000 đ — sai 2.000 đ — mà vẫn thoả cả ba bất biến cước. Nhưng nó **rất đáng giá ở chỗ oracle đầy đủ không với tới**: ở HUY-04 (ví trừ tiền nhưng mất phản hồi cả ba lần), không có giá trị «đúng» nào cho `paymentStatus` được SPEC nói rõ ngoài `failed` theo R-PAY-06; nhưng bất biến «tổng đã trừ = 10.000 đ, không phải 30.000 đ» kiểm được, và đó là đúng điều khách hàng quan tâm.

Oracle bộ phận cũng là cặp đôi tự nhiên của random testing ([TEST-041](/articles/random-testing)): sinh 10.000 đầu vào, kiểm bất biến trên tất cả, kiểm giá trị chính xác bằng oracle độc lập trên một mẫu.

### 4.6. Hai anti-pattern

**Anti-pattern 1 — «chạy hệ thống rồi chép kết quả vào expected».** Cách làm: gọi API, thấy 54.000, dán 54.000 vào cột mong đợi. Test đó không kiểm hệ thống đúng; nó kiểm hệ thống **không đổi**. Lab dựng đúng tình huống: lấy kết quả của bản lỗi cho 14 dòng giá trị của bảng làm «expected» ⇒ bản lỗi đạt **14/14** dòng của chính nó. Dùng expected từ oracle độc lập ⇒ bản lỗi trượt đúng **OR-01** (54.000 ≠ 52.000). Trên 500 mẫu ngẫu nhiên, so từng bước của bảng chi tiết bắt bản lỗi ở **đúng 116 mẫu** — chính là 116 mẫu có cả cao điểm lẫn mưa — và không báo nhầm mẫu nào khác.

Kết quả chép từ hệ thống *có* chỗ dùng: làm **baseline cho regression** khi refactor mà không đổi luật (nguồn «bản trước» ở mục 2.3). Điều kiện là baseline đó phải được **duyệt** — đối chiếu với đặc tả hoặc oracle độc lập ít nhất một lần, ghi lại ai duyệt, theo phiên bản nào (`pricingVersion`). Không có bước duyệt thì baseline chỉ là một bản chụp của lỗi.

**Anti-pattern 2 — assert quá lỏng hoặc quá chặt.**

- *Quá lỏng*: `expect(res.status).toBe(200)`; `expect(total).toBeGreaterThan(0)`; `expect(calls.length).toBeGreaterThan(0)`. Đạt với gần như mọi bản lỗi.
- *Quá chặt*: so nguyên thân phản hồi với một bản chụp, gồm cả `quoteId`, `createdAt`, `expiresAt`. Lab gọi hai lần cùng một báo giá cách nhau 1 giây: thân giống hệt nhau trừ đúng ba trường đó. Test so nguyên sẽ đỏ mỗi khi bộ sinh id đổi, đồng hồ đổi, hay thứ tự test đổi — đỏ mà không có defect, và sau vài lần như vậy đội sẽ cập nhật bản chụp mà không đọc (quay về anti-pattern 1).
- *Vừa đủ chặt*: so **chính xác** mọi giá trị nghiệp vụ; với giá trị sinh ra (id, thời điểm) thì so theo **quan hệ** hoặc theo **nguồn đã biết**: `createdAt` bằng thời điểm của đồng hồ được inject; `expiresAt − createdAt` = 5 phút (R-QUOTE-01); `quoteId` khác báo giá trước.

Thứ tự của danh sách là một trường hợp riêng. `trip_events` có thứ tự xác định (theo thời gian, R-STATE-06), danh sách chuyến chờ của tài xế có thứ tự quy định (R-ACCEPT-06) ⇒ so **theo thứ tự**. Một danh sách mà đặc tả không quy định thứ tự ⇒ so như **tập hợp** (`expect.arrayContaining`, hoặc sắp xếp trước khi so). Assert thứ tự cho một danh sách không có thứ tự là quá chặt; bỏ qua thứ tự cho một danh sách có thứ tự là quá lỏng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Một test case với nhiều đầu ra: phản hồi HTTP, bảng chi tiết cước, trạng thái lưu, trip_events, trạng thái tài xế, lời gọi ví. Mỗi đầu ra có nguồn oracle riêng: đặc tả, oracle tính lại độc lập bằng phân số, đồng hồ được inject, bất biến. Bộ so sánh cho kết quả đạt hoặc trượt. Kết quả chép từ hệ thống bị gạch bỏ vì không phải nguồn oracle.">
<defs>
<marker id="tcd04-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="130" y="24" text-anchor="middle" class="t-small t-b">NGUỒN ORACLE</text>
<text x="460" y="24" text-anchor="middle" class="t-small t-b">SO SÁNH</text>
<text x="790" y="24" text-anchor="middle" class="t-small t-b">ĐẦU RA QUAN SÁT ĐƯỢC</text>
<rect x="20" y="40" width="220" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="130" y="60" text-anchor="middle" class="t-b t-pk">Đặc tả (SPEC)</text>
<text x="130" y="76" text-anchor="middle" class="t-small">mã lỗi · 10.000/20.000 đ · khoá</text>
<rect x="20" y="94" width="220" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="130" y="114" text-anchor="middle" class="t-b t-pk">Oracle tính lại độc lập</text>
<text x="130" y="130" text-anchor="middle" class="t-small">phân số BigInt · 11 bước</text>
<rect x="20" y="148" width="220" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="130" y="168" text-anchor="middle" class="t-b t-pk">Đồng hồ được inject</text>
<text x="130" y="184" text-anchor="middle" class="t-small">cancelledAt · trip_events</text>
<rect x="20" y="202" width="220" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="130" y="222" text-anchor="middle" class="t-b t-pk">Bất biến (oracle bộ phận)</text>
<text x="130" y="238" text-anchor="middle" class="t-small">một khoá ⇒ trừ một lần</text>
<rect x="20" y="256" width="220" height="44" rx="8" class="f-warn-soft s-warn"/>
<text x="130" y="276" text-anchor="middle" class="t-b t-warn">Chép từ hệ thống</text>
<text x="130" y="292" text-anchor="middle" class="t-small">không phải oracle</text>
<line x1="30" y1="278" x2="230" y2="278" class="s-warn" stroke-width="2"/>
<rect x="370" y="110" width="180" height="110" rx="10" class="f-phys-soft s-phys"/>
<text x="460" y="140" text-anchor="middle" class="t-b t-phys">Bộ so sánh</text>
<text x="460" y="162" text-anchor="middle" class="t-small">bằng tuyệt đối cho tiền</text>
<text x="460" y="180" text-anchor="middle" class="t-small">quan hệ cho id, thời điểm</text>
<text x="460" y="198" text-anchor="middle" class="t-small">thứ tự khi SPEC quy định</text>
<line x1="240" y1="62" x2="368" y2="135" class="s-muted" stroke-width="1.4" marker-end="url(#tcd04-a)"/>
<line x1="240" y1="116" x2="368" y2="150" class="s-muted" stroke-width="1.4" marker-end="url(#tcd04-a)"/>
<line x1="240" y1="170" x2="368" y2="175" class="s-muted" stroke-width="1.4" marker-end="url(#tcd04-a)"/>
<line x1="240" y1="224" x2="368" y2="200" class="s-muted" stroke-width="1.4" marker-end="url(#tcd04-a)"/>
<rect x="680" y="40" width="220" height="34" rx="6" class="f-surface s-line"/><text x="790" y="62" text-anchor="middle" class="t-small">HTTP · thân · header replay</text>
<rect x="680" y="82" width="220" height="34" rx="6" class="f-surface s-line"/><text x="790" y="104" text-anchor="middle" class="t-small">bảng chi tiết cước R-FARE-10</text>
<rect x="680" y="124" width="220" height="34" rx="6" class="f-surface s-line"/><text x="790" y="146" text-anchor="middle" class="t-small">trạng thái lưu (đọc lại)</text>
<rect x="680" y="166" width="220" height="34" rx="6" class="f-surface s-line"/><text x="790" y="188" text-anchor="middle" class="t-small t-mono">trip_events</text>
<rect x="680" y="208" width="220" height="34" rx="6" class="f-surface s-line"/><text x="790" y="230" text-anchor="middle" class="t-small">tài xế online/busy</text>
<rect x="680" y="250" width="220" height="34" rx="6" class="f-surface s-line"/><text x="790" y="272" text-anchor="middle" class="t-small">lời gọi ví: số lần · tiền · khoá</text>
<line x1="678" y1="57" x2="552" y2="135" class="s-muted" stroke-width="1.2" marker-end="url(#tcd04-a)"/>
<line x1="678" y1="99" x2="552" y2="148" class="s-muted" stroke-width="1.2" marker-end="url(#tcd04-a)"/>
<line x1="678" y1="141" x2="552" y2="162" class="s-muted" stroke-width="1.2" marker-end="url(#tcd04-a)"/>
<line x1="678" y1="183" x2="552" y2="178" class="s-muted" stroke-width="1.2" marker-end="url(#tcd04-a)"/>
<line x1="678" y1="225" x2="552" y2="192" class="s-muted" stroke-width="1.2" marker-end="url(#tcd04-a)"/>
<line x1="678" y1="267" x2="552" y2="205" class="s-muted" stroke-width="1.2" marker-end="url(#tcd04-a)"/>
<text x="460" y="252" text-anchor="middle" class="t-b">đạt / trượt</text>
<line x1="460" y1="220" x2="460" y2="238" class="s-muted" stroke-width="1.4" marker-end="url(#tcd04-a)"/>
<text x="460" y="318" text-anchor="middle" class="t-small">Lab: 16 dòng bảng + 500 mẫu cước, 8 ca huỷ + 1 ca hoàn thành — examples/toi-noi-testing/articles/test-case-design-expected-result-test-oracle</text>
</svg>
</div>
<figcaption><b>Hình 1 · Mỗi đầu ra một nguồn oracle.</b> Một test case huỷ chuyến có sáu nhóm đầu ra; mỗi nhóm lấy kết quả mong đợi từ nguồn phù hợp nhất. «Chép từ hệ thống» không có mũi tên nào vào bộ so sánh.</figcaption>
</figure>

## 5. Thực hành

### 5.1. Tổ chức mã trong lab

Mọi tệp ở [`articles/test-case-design-expected-result-test-oracle`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/test-case-design-expected-result-test-oracle); không sửa `src/`.

| Tệp | Vai trò |
|---|---|
| `phan-so.ts` | phân số BigInt: đọc số thập phân chính xác, cộng/trừ/nhân/so sánh, làm tròn trần và half-up |
| `oracle-cuoc.ts` | `oracleCuoc()` trả 12 trường (quãng đường + 11 bước của R-FARE-10); `soChiTiet()` liệt kê trường lệch |
| `bang-cuoc.ts` | 16 dòng OR-01…OR-16 kèm tổng tính tay |
| `buggy-fare.ts` | bản chép `calculateFare` có một defect: khi cao điểm, phụ phí trời mưa bị nhân 1,25 |
| `oracle-cuoc.test.ts` | 29 ca: kiểm oracle, so với hệ thống, mẫu ngẫu nhiên, số thực, anti-pattern |
| `huy-chuyen.test.ts` | 9 ca: HUY-01…08 (huỷ) và HT-01 (hoàn thành, R-PAY-03) |

So một dòng của bảng với hệ thống qua API — mọi trường, chính xác tuyệt đối:

```ts
// oracle-cuoc.test.ts (trích)
const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);
expect(res.status).toBe(200);
expect(soChiTiet(res.body.quote.fare, oracleCuoc(d))).toEqual([]);   // 12 trường, không dung sai
expect(res.body.quote).toMatchObject({ total: d.tongTinhTay, peak: d.caoDiem, raining: d.mua });
```

Test huỷ kiểm sáu nhóm đầu ra trong cùng một test case:

```ts
// huy-chuyen.test.ts (trích HUY-01)
expect(res.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });           // (1) phản hồi
const doc = await request(c.app).get(`/api/trips/${c.id}`).set('x-user', KHACH).expect(200);
expect(doc.body.trip).toEqual(res.body.trip);                                                // (2) trạng thái lưu
expect(await trangThaiNhatKy(c.system, c.id)).toEqual([
  ['requested', T0.toISOString()], ['searching', T0.toISOString()],
  ['accepted', T0.toISOString()], ['cancelled', now],
]);                                                                                          // (3) trip_events
expect((await c.system.deps.drivers.findById('D101'))?.status).toBe('online');               // (4) tài xế
expect(c.system.payments.calls).toHaveLength(1);                                             // (5) lời gọi ví
expect(c.system.payments.calls[0]).toMatchObject({ idempotencyKey: c.khoa, amount: 10_000, customerId: 'C001' });
expect(c.system.payments.totalCharged()).toBe(10_000);                                       // (6) bất biến
```

Lab đặt thời gian chờ của ví giả là 50 ms thay cho 3 giây của R-PAY-04 để bộ test chạy nhanh; **số lần thử lại giữ đúng 2** như SPEC. Đây là quyết định về môi trường test cần ghi rõ: kết quả mong đợi về số lời gọi không phụ thuộc thời gian chờ, nên thu nhỏ thời gian chờ không làm yếu test.

### 5.2. Các test case huỷ và hoàn thành

| Test case | Tiền điều kiện | Kết quả mong đợi chính |
|---|---|---|
| HUY-01 | `vixanh`, xe máy, huỷ 5 phút sau khi nhận | 6 nhóm đầu ra như mục 4.1 |
| HUY-02 | như HUY-01, gửi lại cùng `Idempotency-Key` sau 1 phút | cùng mã và thân; `Idempotent-Replayed: true`; ví vẫn 1 lời gọi; `trip_events` vẫn 4 dòng (R-IDEM-02) |
| HUY-03 | ví hết giờ 2 lần rồi trả lời | 3 lời gọi, **cùng một khoá**; `paid`; tổng đã trừ 10.000 (R-PAY-04) |
| HUY-04 | ví trừ tiền nhưng mất phản hồi cả 3 lần | 200, `cancelled`, `failed`; 3 lời gọi; tổng đã trừ **10.000, một lần** |
| HUY-05 | ví từ chối | 1 lời gọi, không thử lại; `failed`; vẫn `cancelled`, 200 (R-PAY-06) |
| HUY-06 | tiền mặt, huỷ sau 5 phút | phí 10.000; `paymentStatus = cash`; **0** lời gọi ví (R-PAY-05) |
| HUY-07 | `vixanh`, huỷ sau 1 phút | phí 0, `grace_period`, `none`; **0** lời gọi; tài xế `online` |
| HUY-08 | `vixanh`, ô tô (D103) | 1 lời gọi 20.000 đ; D103 `online` (R-CANCEL-03) |
| HT-01 | `vixanh`, chạy tới `complete` | 1 lời gọi khoá `trip:<id>:fare`, số tiền = 35.000 lấy từ **oracle cước**; `completed`, `paid`, `completedAt` = đồng hồ (R-PAY-03) |

HT-01 cho thấy các oracle **nối với nhau**: số tiền ví phải thu không lấy từ báo giá của hệ thống mà từ oracle cước độc lập. Nếu hệ thống tính sai cước, test thu tiền cũng đỏ — đúng như mong muốn, vì khách thực sự bị thu sai.

### 5.3. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-expected-result-test-oracle --reporter=verbose
```

Kết quả thật (rút gọn):

```text
✓ oracle-cuoc.test.ts > TCD-04 · 1. kiểm chính oracle … > R-FARE-09: 45.300 → 56.625 → 64.625 → −12.925 → 51.700 → 52.000 đ
✓ oracle-cuoc.test.ts > TCD-04 · 2. oracle so với hệ thống qua POST /api/quotes — mọi bước của R-FARE-10 > OR-03
✓ oracle-cuoc.test.ts > TCD-04 · 3. … > quét 0,5…100 km × 2 loại xe × cao điểm có/không (3.984 tổ hợp): oracle số thực sai 12 tổ hợp
✓ oracle-cuoc.test.ts > TCD-04 · 4. … > expected từ oracle độc lập: bản lỗi trượt đúng dòng có cao điểm + mưa (OR-01: 54.000 ≠ 52.000)
✓ huy-chuyen.test.ts > TCD-04 · 6. … > HUY-04 · ví trừ tiền nhưng mất phản hồi cả 3 lần: failed nhưng tiền đã bị trừ đúng MỘT lần (oracle bộ phận)

 Test Files  2 passed (2)
      Tests  38 passed (38)
```

38 ca = 29 ca ở `oracle-cuoc.test.ts` (5 ca kiểm chính oracle + 16 ca so với hệ thống + 3 ca mẫu ngẫu nhiên và số thực + 3 ca chép kết quả + 2 ca assert lỏng/chặt) + 9 ca ở `huy-chuyen.test.ts`. Bản `src/` hiện tại đạt toàn bộ, kể cả 500 mẫu ngẫu nhiên khớp oracle ở mọi trường. Những ca «chứng minh anti-pattern» được viết thành test đạt: chúng khẳng định rằng bản lỗi *đạt* bộ expected chép từ chính nó, và *trượt* bộ expected từ oracle.

### 5.4. Một quan sát từ HUY-04

HUY-04 đạt — hệ thống làm đúng SPEC: `paymentStatus = failed` theo R-PAY-06, và ví giả bảo đảm một khoá chỉ trừ một lần. Nhưng đọc kết quả mong đợi theo góc nhìn khách hàng: hệ thống ghi «thu thất bại» trong khi tiền **đã bị trừ**. SPEC không có quy tắc đối soát cho trường hợp này (webhook và ledger của thanh toán nằm ngoài phạm vi lab, R-ARCH-02). Đó là một khoảng trống của test basis, phát hiện được nhờ viết kết quả mong đợi cho **cả hai phía** của lời gọi ra ngoài. Ghi vào defect report về test basis và gửi BA; test case thì giữ nguyên, kèm ghi chú.

## 6. Góc nhìn nâng cao

**Oracle độc lập nhưng ai viết?** Oracle do chính developer viết song song với sản phẩm dễ mang cùng cách hiểu sai đặc tả. Cách giảm: người viết oracle khác người viết sản phẩm, oracle viết từ SPEC chứ không từ code, và oracle phải qua các ví dụ có sẵn trong SPEC. Nếu SPEC sai, cả hai cùng sai — lúc đó chỉ review với BA và phán đoán của con người cứu được.

**Khi oracle và hệ thống lệch nhau, chưa biết ai sai.** Test đỏ là một *bất thường*; phân tích có thể chỉ ra defect ở sản phẩm, ở oracle, ở test data, hoặc ở đặc tả. Vì vậy `soChiTiet()` trả về **trường nào lệch, giá trị nào**, không chỉ «khác nhau» — để người điều tra thấy ngay lệch ở bước nào của R-FARE-03.

**Kết quả mong đợi phụ thuộc thời gian.** Mọi thời điểm trong kết quả mong đợi của lab đều suy ra từ `FixedClock`. Test dùng đồng hồ thật thì chỉ assert được kiểu «gần đây» — tức quá lỏng. Đồng hồ được inject (R-ARCH-03) là điều kiện để viết kết quả mong đợi chính xác cho thời điểm; [TCD-05](/articles/test-case-design-test-data) bàn kỹ dữ liệu thời gian và múi giờ.

**Snapshot test có đáng dùng?** Snapshot (golden master) hợp với đầu ra lớn, ổn định, khó viết tay (ví dụ HTML email, báo cáo đối soát) — với điều kiện: loại bỏ trường sinh ra (id, thời điểm) trước khi chụp, bản chụp đầu tiên được duyệt với test basis, và mọi lần cập nhật bản chụp đều qua review như sửa code. Không có ba điều kiện đó, snapshot là anti-pattern 1 có công cụ hỗ trợ.

**Oracle cho đầu ra phi chức năng.** Thời gian phản hồi, bộ nhớ… cũng cần kết quả mong đợi kiểm chứng được («p95 dưới 500 ms» của R-NFR-02, kèm điều kiện đo), không phải «nhanh». Bài này giới hạn ở đầu ra chức năng.

**Những điểm SPEC chưa rõ, phát hiện khi viết kết quả mong đợi.** (1) R-FARE-10 liệt kê «cước tối thiểu» trong bảng chi tiết nhưng không nói đó là hằng số của loại xe hay giá trị sau bước 5; lab trả hằng số, test chấp nhận theo cách hiểu đó — cần BA xác nhận. (2) R-API-11 không nói phản hồi huỷ có phản ánh `paymentStatus` *sau* khi thu phí hay không (thu đồng bộ hay bất đồng bộ); lab thu đồng bộ nên trả `paid`. (3) Trường hợp «đã trừ tiền nhưng ghi failed» ở mục 5.4. (4) `trip_events` (R-STATE-06) không có endpoint đọc; test phải đọc qua repository — trong hệ thống đích cần một điểm quan sát (bảng, API nội bộ, hoặc sự kiện) để test ở mức hệ thống kiểm được yêu cầu này.

| Sai lầm thường gặp | Cách tránh |
|---|---|
| Expected chỉ có mã HTTP | liệt kê 5 nhóm đầu ra; kiểm cả phủ định |
| Expected chép từ lần chạy | oracle từ SPEC hoặc tính lại độc lập; baseline phải được duyệt |
| Oracle gọi lại code sản phẩm | không import `src/`; khác cách biểu diễn số |
| Oracle tiền bằng số thực | số nguyên đơn vị nhỏ, phân số, decimal |
| So thời điểm bằng đồng hồ thật | inject đồng hồ; so bằng hoặc theo quan hệ |
| So nguyên bản chụp gồm id, thời điểm | loại trường sinh ra, so giá trị nghiệp vụ |
| Bỏ qua lời gọi ra ngoài | bản giả ghi lời gọi; kiểm số lần, khoá, số tiền |
| Kiểm cả nội dung không quy định (mô tả, câu chữ) | chỉ so cái SPEC quy định |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K2).** Viết kết quả mong đợi kiểm chứng được cho test case «tài xế D101 huỷ chuyến `accepted` 3 phút sau khi nhận, khách thanh toán `vixanh`». Nêu đủ các nhóm đầu ra và mã yêu cầu.

<details><summary><b>Lời giải gợi ý</b></summary>
<p>(1) HTTP 200, thân <code>{ trip }</code> (R-API-11), không có <code>cancellationFee</code>/<code>feeReason</code>. (2) <code>trip.status</code> = <code>searching</code>, <code>driverId</code> = null, <code>acceptedAt</code> = null, <code>etaMinutes</code> = null, <code>searchingSince</code> = thời điểm huỷ theo đồng hồ được inject, <code>cancellationFee</code> = 0, <code>paymentStatus</code> giữ <code>pending</code> (R-STATE-04, R-CANCEL-05). (3) Đọc lại qua <code>GET /api/trips/:id</code> bằng khách: như (2). (4) <code>trip_events</code> thêm đúng một dòng <code>searching</code> với thời điểm huỷ (R-STATE-06). (5) D101 = <code>online</code> (R-ACCEPT-05). (6) Ví: <b>0</b> lời gọi (R-CANCEL-05). Phần (6) và việc <code>paymentStatus</code> không đổi là kết quả mong đợi dạng phủ định — thiếu chúng thì defect «thu phí khách khi tài xế huỷ» lọt.</p>
</details>

**Bài tập 2 (K2).** Đội viết oracle cước như sau: `const expected = calculateFare(input).total; expect(apiTotal).toBe(expected)`. Nhận xét và sửa.

<details><summary><b>Lời giải gợi ý</b></summary>
<p>Oracle gọi chính hàm của sản phẩm nên chỉ kiểm được rằng tầng API trả đúng giá trị hàm miền tính ra (một kiểm tra tích hợp hữu ích, nhưng không phải kiểm luật cước). Defect trong <code>calculateFare</code> sẽ có mặt ở cả hai vế. Sửa: oracle không import <code>src/</code>, tham số chép từ SPEC, dùng số học chính xác; kiểm oracle bằng R-FARE-09 và các ví dụ của R-FARE-02/08; so từng bước của bảng chi tiết, không chỉ tổng. Có thể giữ test cũ dưới tên đúng của nó: «API trả kết quả của hàm miền».</p>
</details>

**Bài tập 3 (K3).** Tính kết quả mong đợi cho báo giá ô tô 18,6 km lúc 17:30 giờ VN, TP. Hồ Chí Minh đang mưa, voucher `GIAM20`. Ghi các bước của R-FARE-10.

<details><summary><b>Lời giải gợi ý</b></summary>
<p>Cơ bản: 25.000 + 11.000 × 18,6 = 229.600. 17:30 thuộc [17:00; 19:00) ⇒ × 1,25 = 287.000. Mưa: + 8.000 = 295.000 (trước giảm). <code>GIAM20</code>: 20% × 295.000 = 59.000 &gt; trần 30.000 ⇒ giảm 30.000 ⇒ 265.000 ≥ 30.000 tối thiểu ⇒ làm tròn lên: <b>265.000 đ</b>. Một oracle số thực tính 25.000 + 11.000 × 18,6 ra 229.600,00000000003 và cho 266.000 đ — cùng họ lỗi với OR-03.</p>
</details>

**Câu 1 (K1).** Theo ISTQB Glossary, test oracle là gì?

a) Công cụ tự động chạy test
b) Nguồn dùng để xác định kết quả mong đợi để so với kết quả thực tế
c) Người chịu trách nhiệm phê duyệt kết quả test
d) Tập test case có độ phủ cao nhất

<details><summary><b>Đáp án: b</b></summary>
<p>a nhầm với công cụ thực thi. c: con người có thể là một nguồn oracle nhưng định nghĩa không giới hạn ở người phê duyệt. d không liên quan.</p>
</details>

**Câu 2 (K2).** Theo CTFL v4.0.1 mục 1.4.1, việc so sánh kết quả thực tế với kết quả mong đợi thuộc hoạt động nào?

a) Test analysis
b) Test design
c) Test execution
d) Test completion

<details><summary><b>Đáp án: c</b></summary>
<p>Test execution chạy test, so sánh kết quả thực tế với mong đợi và phân tích bất thường. Test design là lúc <i>quyết định</i> kết quả mong đợi gồm gì, không phải lúc so sánh.</p>
</details>

**Câu 3 (K2).** Kết quả mong đợi nào kiểm chứng được tốt nhất cho «khách huỷ chuyến `cash` 5 phút sau khi tài xế xe máy nhận»?

a) Huỷ thành công
b) HTTP 200 và phí huỷ hợp lý
c) HTTP 200; `cancellationFee` 10.000, `feeReason` `charged`; chuyến `cancelled`, `paymentStatus` `cash`; không có lời gọi ví; tài xế `online`
d) Phản hồi giống bản chụp đã lưu từ lần chạy trước

<details><summary><b>Đáp án: c</b></summary>
<p>a, b mơ hồ. d là anti-pattern chép kết quả (và quá chặt nếu bản chụp có id, thời điểm). c cụ thể, có nguồn (R-CANCEL-03, R-PAY-05, R-ACCEPT-05), phủ cả đầu ra phủ định.</p>
</details>

**Câu 4 (K2).** Bất biến «cước luôn là bội số 1.000 đ và không nhỏ hơn cước tối thiểu» là loại oracle gì, và giới hạn của nó?

a) Oracle đầy đủ; phát hiện mọi defect tính cước
b) Oracle bộ phận; không phát hiện được cước sai nhưng vẫn thoả tính chất
c) Oracle từ hệ thống so sánh; phụ thuộc bản cũ
d) Không phải oracle vì không có giá trị cụ thể

<details><summary><b>Đáp án: b</b></summary>
<p>Bất biến là oracle bộ phận: bản lỗi trong lab trả 54.000 đ thay vì 52.000 đ vẫn thoả cả hai tính chất. d sai: oracle không bắt buộc cho giá trị cụ thể.</p>
</details>

**Câu 5 (K2).** Test so nguyên thân báo giá với một bản chụp, đỏ mỗi khi đổi thứ tự chạy test dù giá không đổi. Nguyên nhân và cách sửa phù hợp nhất?

a) Hệ thống có defect; báo developer
b) Assert quá chặt vì gồm trường sinh ra như `quoteId`, `createdAt`; so giá trị nghiệp vụ chính xác và so id/thời điểm theo quan hệ hoặc theo đồng hồ được inject
c) Bỏ test vì không ổn định
d) Cập nhật bản chụp mỗi lần đỏ

<details><summary><b>Đáp án: b</b></summary>
<p>a: không có defect vì giá không đổi. c bỏ luôn giá trị của test. d dẫn tới anti-pattern chép kết quả, vì bản chụp được cập nhật mà không duyệt.</p>
</details>

**Câu 6 (K2).** Chuyến `vixanh`, ví hết giờ hai lần rồi trả lời thành công. Kết quả mong đợi nào về lời gọi ví đúng với R-PAY-04?

a) 1 lời gọi
b) 3 lời gọi với 3 khoá idempotency khác nhau
c) 3 lời gọi với cùng một khoá idempotency; tiền chỉ bị trừ một lần
d) 2 lời gọi, vì lần thứ ba không cần

<details><summary><b>Đáp án: c</b></summary>
<p>Hai lần hết giờ ⇒ hai lần thử lại ⇒ tổng 3 lời gọi, dùng lại đúng khoá. b là đúng defect mà kết quả mong đợi phải bắt được: khoá mới mỗi lần thử có thể trừ tiền nhiều lần.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB — Certified Tester Foundation Level Syllabus v4.0.1, mục 1.4.1 Test Activities and Tasks, 1.4.3 Testware; LO FL-1.4.1 (K2), FL-1.4.3 (K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra *test oracle*, *expected result*, *actual result*: https://glossary.istqb.org/
- ISTQB — Certified Tester Advanced Level Test Analyst Syllabus v4.0 (bối cảnh: test oracle problem và kỹ thuật dựa trên quy tắc): https://www.istqb.org/
- ISO/IEC/IEEE 29119-3 — Test documentation (trường expected result trong test case specification).
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-GEN-01, R-FARE-01…10, R-PARCEL, R-CANCEL, R-STATE-06/07, R-ACCEPT-05, R-PAY-03…06, R-IDEM-02; mã bài tại `articles/test-case-design-expected-result-test-oracle/`.
- Bài liên quan: [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-041 · Random Testing](/articles/random-testing) · [TEST-042 · Metamorphic Testing](/articles/metamorphic-testing) · [Test double: dummy, stub, fake, mock, spy](/articles/test-doubles-dummy-stub-fake-mock-spy) · [TCD-05 · Test data](/articles/test-case-design-test-data).

Bài tiếp theo: [TCD-05 · Test data cho từng test case](/articles/test-case-design-test-data).

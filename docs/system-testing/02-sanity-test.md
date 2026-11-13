# SYS-02 — Sanity Test: kiểm đúng vùng thay đổi trước khi mở rộng

Trạng thái: bản nháp hoàn chỉnh để review; lab đi kèm. Bài trước: [Smoke Test](01-smoke-test.md). Thuộc [lộ trình kiểm thử hệ thống](../system-testing-roadmap.md).

## 1. Tình huống, điều kiện tiên quyết và mục tiêu

Khách Tới Nơi dán mã ` giam20 ` từ tin nhắn vào ô voucher. Mã tồn tại nhưng ứng dụng báo không tìm thấy vì đường xử lý giả định chỉ viết hoa, chưa bỏ khoảng trắng hai đầu. Nhóm vừa bổ sung `trim`. Có cần chạy mọi ca đặt xe, giao hàng, thanh toán và báo cáo ngay để đánh giá thay đổi này không?

Ta cần một vòng kiểm tập trung: lỗi gốc hết chưa, mã đã đúng trước đây còn dùng được không, mã rỗng và mã sai có bị hiểu nhầm không, dữ liệu sau chuẩn hóa có đi tiếp đúng tới đặt chuyến và giới hạn lượt dùng không? Đó là mục tiêu của sanity suite trong bài này.

Đây là **tình huống lỗi giả lập phục vụ học tập**. Hàm trong lab hiện tại đã chuẩn hóa đúng; bài không khẳng định đã phát hiện hay sửa một defect thật trong repository. Fault fixture nằm trong thư mục bài, không sửa mã sản phẩm dùng chung.

Bạn cần biết HTTP status, cấu trúc test Arrange–Act–Assert và cách chạy Vitest. Sau bài, bạn có thể phân biệt sanity với confirmation/regression, dựng impact map, chọn các ca có lý do, kiểm chứng fixture lỗi và viết một kết luận đúng phạm vi.

## 2. Nền tảng: sanity là lựa chọn phạm vi theo thay đổi

Trong series này, **sanity testing** là vòng kiểm ngắn, tập trung vào thay đổi cụ thể và những hành vi liên quan trực tiếp, nhằm đánh giá thay đổi có đủ hợp lý để tiếp tục kiểm sâu hay không. Đây là quy ước vận hành của nhóm; không phải một test level độc lập hoặc một tên suite bắt buộc của mọi tổ chức.

Sanity thường xuất hiện sau bản sửa nhỏ hoặc một build có thay đổi rõ ràng. Nó có thể dùng manual testing, component tests hoặc API tests. Một ca đã tồn tại có thể được chọn vào sanity của release này, nhưng không cần chạy trong sanity của release khác.

| Hoạt động | Căn cứ chọn | Kết luận hợp lệ |
|---|---|---|
| Smoke | Khả năng thiết yếu của build/môi trường | Có thể bắt đầu vòng kiểm tiếp theo |
| Confirmation | Điều kiện từng gây ra lỗi đã biết | Lỗi đó không còn tái hiện với ca đã chạy |
| Sanity | Thay đổi và vùng ảnh hưởng gần | Thay đổi hoạt động hợp lý trong phạm vi chọn |
| Regression | Rủi ro ảnh hưởng bất lợi do thay đổi | Các hành vi được chọn vẫn đáp ứng yêu cầu |

Một sanity suite có thể gồm ca confirmation cộng một tập regression có chủ đích. Nó không thay thế regression đầy đủ. “Smoke rộng, sanity hẹp” là mẹo nhớ có giới hạn: phạm vi thực tế phải xuất phát từ mục tiêu, rủi ro và kiến trúc, không từ số lượng case.

CTFL v4.0.1 phân biệt confirmation và regression; việc đặt tên sanity suite ở đây là thực tiễn của series, không gán thêm mã learning objective. Đọc phần kiểm thử liên quan đến thay đổi trong [syllabus chính thức](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf).

### Khi nào vòng sanity nhỏ không đủ?

Một dòng sửa trong hàm chuẩn hóa dùng chung có thể ảnh hưởng nhiều dịch vụ. Thay đổi schema, cơ chế xác thực, kiểu dữ liệu tiền hoặc thư viện nền có thể cần regression rộng và kiểm môi trường riêng dù diff rất nhỏ. Nếu chưa biết callers và dữ liệu tiêu thụ đầu ra, cần điều tra impact trước khi gọi suite nhỏ là đủ.

## 3. Test basis, lỗi giả lập và oracle độc lập

Theo `SPEC.md`:

- R-VOUCHER-02: bỏ khoảng trắng hai đầu, viết hoa, chuỗi rỗng sau trim tương đương không dùng voucher.
- R-VOUCHER-03: mã không tồn tại trả `VOUCHER_NOT_FOUND`.
- R-VOUCHER-06: tiền giảm là phần trăm của số tiền áp dụng, giới hạn bởi trần giảm.
- R-VOUCHER-07: báo giá không tiêu lượt; tạo chuyến tiêu lượt. Voucher mẫu GIAM20 có một lượt mỗi khách mỗi ngày.
- R-QUOTE-04: chuyến giữ giá của báo giá, không tự tính lại theo điều kiện mới.

Fixture lỗi là `raw.toUpperCase()`. Với ` giam20 `, nó trả ` GIAM20 `, khác expected `GIAM20`. Hàm thật `normalizeVoucherCode` trả đúng mã. Không dùng kết quả hàm thật làm expected cho fixture: expected phải đến từ yêu cầu.

Oracle tiền được tính tay: xe máy 5 km, SG, giờ thường, không mưa → 12.000 + 4.500 × 5 = 34.500 đồng. GIAM20 giảm 6.900, còn 27.600; sau cước tối thiểu và làm tròn lên nghìn là **28.000**. Không voucher: 34.500 làm tròn thành **35.000**. Giảm 20% trên 35.000 là tính sai thứ tự, dù đôi khi tổng làm tròn vẫn trùng. Vì vậy test kiểm cả `discount` lẫn `total`.

## 4. Impact map và lựa chọn test

Chuỗi phụ thuộc cần xem là input API → chuẩn hóa mã → tra cứu voucher → tính giá → lưu quote → tạo chuyến → thống kê lượt dùng. Ca confirmation chỉ kiểm bước chuẩn hóa chưa đủ chứng minh toàn chuỗi dùng dữ liệu đúng.

| ID | Basis/rủi ro | Input và hành động | Expected | Vai trò |
|---|---|---|---|---|
| SN-01 | R-VOUCHER-02; thiếu trim | Gọi fixture và hàm thật với ` giam20 ` | Fixture khác GIAM20; hàm thật bằng GIAM20 | Minh họa phát hiện lỗi gốc |
| SN-02 | Chuẩn hóa không nối vào API | POST quote với ` giam20 ` | 200, GIAM20, giảm 6.900, tổng 28.000 | Confirmation ở API |
| SN-03 | Làm hỏng mã chuẩn | POST quote với GIAM20 | Như SN-02 | Regression gần |
| SN-04 | Nhầm rỗng với mã sai | Ba input: chuỗi rỗng, toàn khoảng trắng, không gửi field | 200, voucherCode null, discount 0, total 35.000 | Partition đầu vào |
| SN-05 | Nuốt lỗi mã không tồn tại | POST quote với ` khongco ` | 422, VOUCHER_NOT_FOUND | Negative path |
| SN-06 | Canonical code bị mất khi tạo chuyến | Báo giá bằng mã có khoảng trắng; tạo chuyến; báo giá lại bằng mã chuẩn | Chuyến giữ GIAM20 và 28.000; lần sau VOUCHER_USAGE_LIMIT | Hợp đồng downstream |

Mỗi case có system/app mới, khách C001 và clock 09:00 giờ VN ngày 04/11/2026; không mưa. SN-06 tạo dữ liệu chỉ trong system của chính nó. Cleanup bằng việc không giữ/chia sẻ instance giữa các case; không có dữ liệu ngoài tiến trình để xóa.

Những phạm vi chưa chọn: mọi tổ hợp thành phố/dịch vụ, ngày hết hạn, biên trần giảm, hủy hoàn lượt, concurrency, thanh toán và browser. Chúng cần functional/regression hoặc suite chuyên biệt. Nếu diff chạm `voucherDiscountMilli`, eligibility hoặc repository, phải bổ sung phạm vi đó ngay; bảng này chỉ phù hợp giả định sửa normalization.

## 5. Lab: từ assertion nhỏ tới API có trạng thái

Mã: `examples/toi-noi-testing/articles/system-testing-sanity-test/sanity.test.ts`. Từ thư mục lab, dùng Node >=22.12:

```sh
npm ci
npm test -- articles/system-testing-sanity-test
npm run typecheck
```

`npm ci` chỉ cần khi chuẩn bị/cập nhật dependency theo lockfile. Expected: **8 tests pass** vì SN-04 chạy ba dataset. Sáu ID case không có nghĩa runner chỉ báo sáu executions.

### Bước 1 — Kiểm khả năng phân biệt đúng và sai

```ts
const beforeFix = (raw: string) => raw.toUpperCase();
expect(beforeFix(' giam20 ')).not.toBe('GIAM20');
expect(normalizeVoucherCode(' giam20 ')).toBe('GIAM20');
```

Suite xanh có chủ đích: assertion đầu yêu cầu fixture khác oracle. Nó chứng minh fixture biểu diễn lỗi thiếu trim và hàm hiện tại đáp ứng ca đó. Đây không phải log một build sản phẩm lỗi đã chạy đỏ rồi được sửa xanh. Nếu muốn thực hành red/green, trong bản sao riêng của test thay `.not.toBe` bằng `.toBe`, chạy riêng SN-01 để quan sát failure, rồi hoàn nguyên; không commit assertion cố ý fail.

### Bước 2 — Kiểm đường API thực sự gọi hành vi đúng

Gửi `voucherCode` nguyên trạng qua supertest. Không trim trong helper của test: nếu helper tự sửa input, ca sẽ xanh dù server còn lỗi. Kiểm response status, canonical code, discount và total để tránh chỉ kiểm “không bị exception”.

### Bước 3 — Kiểm dữ liệu sau thay đổi được tiêu thụ đúng

SN-06 dùng quoteId thực trả về để tạo chuyến với Idempotency-Key riêng. Sau đó báo giá lại bằng GIAM20. Vì chuyến chưa bị hủy đã tiêu lượt, response phải là 422 `VOUCHER_USAGE_LIMIT`. Nếu code được lưu có khoảng trắng, phần đếm theo mã chuẩn có thể bỏ sót và cho dùng lần nữa. Đây là lý do chọn một ca có trạng thái dù thay đổi ban đầu chỉ là chuỗi.

Kho memory phù hợp kiểm điều phối và logic nhanh trong bài này; chưa chứng minh query, collation hoặc transaction của PostgreSQL. Kết quả chạy thật và giới hạn nằm trong [báo cáo SYS-02](02-test-report.md).

## 6. Entry, exit và cách điều tra lỗi

Entry: có yêu cầu chuẩn hóa rõ ràng, biết diff giả định, seed/clock xác định, môi trường chạy được. Nếu smoke nền đã fail ở phần cần dùng, sửa môi trường hoặc phân loại block trước khi giải thích sanity.

Exit: tám executions pass, không skip, assertion không bị giảm độ chặt; các vùng chưa kiểm được ghi lại. Khi có failure không giải thích được, chưa đạt gate sanity. Khi pass, chỉ cho phép tiếp tục vòng kiểm sâu; không tự động chấp nhận release.

| Thất bại | Cần phân biệt | Bước điều tra |
|---|---|---|
| Unit normalize pass, API fail | Wiring, schema, lookup hay rule khác | Xem error code và input thực gửi; lần theo priceQuote |
| SN-04 fail với chuỗi trắng | Quy ước null/empty hoặc validation | Đối chiếu R-VOUCHER-02 trước khi sửa expected |
| Total đúng, discount sai | Sai thứ tự tính bị làm tròn che mất | So từng bước với oracle 34.500 → 6.900 → 27.600 |
| SN-06 không bị giới hạn | Không lưu code chuẩn hoặc không đếm lượt | Kiểm trip và scope khách/ngày, rồi repository |
| Chạy riêng pass, cùng suite fail | Dùng chung clock, seed hay state | Kiểm beforeEach và tham chiếu instance |

Defect report nên gồm build/commit, case ID, input nguyên văn có khoảng trắng được thể hiện rõ, clock, seed, expected/actual và bước đầu tiên lệch. Không chỉ chụp ảnh chữ “test failed”.

## 7. Nâng cao: mở rộng theo rủi ro thật

### 7.1. Thay đổi nhỏ ở hàm dùng chung

Với ride, parcel và food cùng nhận voucher, lập caller map từ code và hợp đồng API. Nếu normalization thay đổi cách giải thích mã cho mọi dịch vụ, thêm ít nhất một đường đại diện cho từng cấu trúc input khác nhau, đặc biệt food áp giảm lên tiền món thay vì phí giao. Chọn theo khác biệt xử lý, không nhân bản mọi case vô điều kiện.

### 7.2. Unicode và canonicalization

`trim` và `toUpperCase` không tự giải quyết mọi ký tự vô hình, khoảng trắng bên trong hoặc Unicode normalization. Không tự quyết định xóa tất cả whitespace: mã `GIAM 20` có thể cần báo không tồn tại thay vì biến thành GIAM20. Với mã chỉ cho phép ASCII, cần hợp đồng validation rõ và test ký tự ngoài tập. Nếu muốn nhận Unicode, cần quyết định chuẩn lưu, đối chiếu và nguy cơ nhiều biểu diễn trùng mã trước khi viết expected.

### 7.3. Giới hạn lượt và concurrency

SN-06 chạy tuần tự, chứng minh lượt sau nhìn thấy lượt trước trong memory. Hai request đồng thời có thể cùng đọc usedToday=0 rồi cùng tạo chuyến. Để kiểm yêu cầu “không vượt một lượt”, cần môi trường transaction thực và invariant tổng số chuyến hợp lệ ≤1; không suy ra điều đó từ ca tuần tự. Đây là phần giao giữa sanity theo impact và integration/data-integrity testing.

### 7.4. Tự động chọn test và rủi ro bỏ sót

Có thể gắn mapping `normalizeVoucherCode → SN-01…SN-06` để chạy nhanh theo diff. Nhưng file-level mapping không phát hiện mọi ảnh hưởng từ cấu hình, dependency hoặc schema. Luôn có suite nền và lịch regression rộng, theo dõi lỗi lọt qua để sửa mapping. Nếu không xác định được impact, mở rộng suite thay vì coi “không có test được chọn” là pass.

### 7.5. Làm mạnh bằng property testing

Một property hữu ích là idempotence của normalization: N(N(x)) = N(x), có xử lý null đúng kiểu. Tuy nhiên một hàm luôn trả null cũng thỏa property ấy. Vì thế property bổ trợ các ví dụ GIAM20, chuỗi rỗng và mã sai; không thay thế oracle nghiệp vụ. Với sinh chuỗi Unicode, giới hạn miền dữ liệu theo API và ghi seed để tái hiện khi fail.

## 8. Anti-pattern và giới hạn bằng chứng

Chỉ chạy lại đúng ca lỗi gốc là confirmation, chưa đánh giá tác động lân cận. Tự trim input trong test che mất lỗi server. Chạy mọi test rồi gọi là sanity khiến mục tiêu lựa chọn phạm vi không rõ. Gán sanity “luôn manual” hoặc “luôn automated” làm mất tính linh hoạt. Dùng độ dài diff để suy ra mức rủi ro bỏ qua shared code.

Lab không thay mã production và không chứng minh lịch sử bug. API dùng danh tính qua x-user giả lập; không kiểm auth token. Không có browser, DB thật nhiều kết nối hoặc mạng dịch vụ ngoài trong lượt này. Phần Unicode, concurrency và property testing là hướng mở rộng chưa được thực thi ở suite kèm bài.

## 9. Bài tập, đáp án định hướng và tự kiểm

1. **Cơ bản:** thêm input `gIaM20`. Expected giống GIAM20, 28.000 và giảm 6.900. Giải thích nó phủ quy tắc viết hoa chứ không chỉ trim.
2. **Trung bình:** thêm `GIAM 20`. Với SPEC hiện tại chỉ bỏ khoảng trắng hai đầu, mã này không có trong seed nên expected VOUCHER_NOT_FOUND. Không thay test helper để xóa khoảng trắng bên trong.
3. **Trung bình:** nếu đổi phần trăm GIAM20 từ 20 thành 25, tám test hiện tại còn là sanity phù hợp không? Không thể dùng nguyên oracle. Phải làm rõ thay đổi seed/requirement và thêm ca trần giảm, cước tối thiểu, snapshot giá; thay đổi phạm vi từ normalization sang pricing.
4. **Nâng cao:** thiết kế hai request đồng thời cho một lượt voucher. Đáp án cần barrier khởi chạy, transaction thật, kết quả chấp nhận/từ chối, SQL kiểm invariant và cleanup theo run ID. Chạy Promise.all trên memory không chứng minh production an toàn.
5. **Tự kiểm:** tất cả test xanh có chứng minh pentest, performance hay UAT đạt không? Không. Unit normalization pass nhưng SN-02 fail có được đóng lỗi không? Chưa: hành vi người dùng qua API chưa đáp ứng yêu cầu.

## 10. Tài liệu tham khảo

- [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf): phân biệt confirmation/regression và phân tích tác động của thay đổi.
- [Vitest Expect](https://vitest.dev/api/expect.html): các assertion dùng để so giá trị chính xác và kiểm cấu trúc kết quả.
- `examples/toi-noi-testing/SPEC.md`, `src/domain/voucher.ts`, `src/app/quote.ts`: yêu cầu và implementation để truy vết lab, không dùng implementation thay yêu cầu.

Nguồn ngoài đối chiếu ngày 13/11/2026. Bài tiếp theo: Functional Test — mở rộng từ một thay đổi cụ thể sang bao phủ chức năng theo yêu cầu, phân vùng, biên và bảng quyết định.

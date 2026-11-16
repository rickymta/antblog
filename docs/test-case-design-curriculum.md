# Thiết kế Test Case (Test Case Design) — nhánh TCD của chuyên mục Kiểm thử

Ngày lập: 16/11/2026. Nhánh bổ sung cho curriculum TEST-001…162 (`testing-master-curriculum.md`), dùng mã riêng **TCD-01…TCD-10** để không đánh lại mã cũ — giống nhánh SYS ([system-testing-roadmap.md](system-testing-roadmap.md)).

## Vì sao cần nhánh này

S04/S05 dạy **từng kỹ thuật** (EP, BVA, decision table, state transition…), TEST-010 giới thiệu testware ở mức nhận biết. Còn thiếu phần nối giữa hai đầu: từ một đặc tả thật, làm sao ra **một bộ test case viết tốt, đủ, không thừa, truy vết được, có thứ tự chạy và bảo trì được**. Nhánh TCD lấp chỗ đó, theo hoạt động test analysis → test design → test implementation của CTFL v4.0.1 mục 1.4.1.

## Ranh giới với bài sẵn có

| Bài sẵn có | Đã dạy | TCD không lặp lại, chỉ dùng kết quả |
|---|---|---|
| TEST-007 Quy trình kiểm thử | Tên 7 nhóm hoạt động | TCD-01 đi sâu test analysis; TCD-03 đi sâu test implementation |
| TEST-010 Testware | Một test case mẫu, một bảng truy vết nhỏ (R-PARCEL) | TCD-02 dạy cách viết từng trường và lỗi hay gặp; TCD-08 dựng và dùng RTM nhiều-nhiều, phân tích ảnh hưởng |
| TEST-031…039 Kỹ thuật hộp đen | Cách áp từng kỹ thuật | TCD-06 chọn và phối hợp kỹ thuật cho một tính năng, loại trùng, single-fault cho ca âm |
| TEST-041/042 Random, Metamorphic | Oracle cho thuật toán | TCD-04 tổng quát các nguồn oracle và cách viết expected result |
| TEST-148 (kế hoạch), TEST-156 (kế hoạch) | — | TCD-09 dạy ưu tiên test case mức K3 của CTFL 5.1.5; TCD-05 dạy test data **cho từng test case**, không phải quản lý test data cấp tổ chức |
| SYS-03 Functional Test | Ca chức năng cho cước | TCD-10 là capstone thiết kế trọn quy trình, khác domain (đặt chuyến + idempotency) |

## Danh sách bài

Mọi bài: `category: testing`, `level: intermediate`, Knowledge Level L2 (TCD-06, 09, 10: L3). Lab: `examples/toi-noi-testing/articles/<slug>/` (README + `*.test.ts`), **không sửa `src/`**.

| Mã | Slug | Chủ đề | Domain SPEC | Bài cần đọc trước | Tham chiếu ISTQB |
|---|---|---|---|---|---|
| TCD-01 | `test-case-design-test-analysis-test-condition` | Test analysis: đọc test basis, nhận diện và đặt tên test condition, mức chi tiết, defect phát hiện ngay khi phân tích, ưu tiên sơ bộ | R-VOUCHER | TEST-007, TEST-010 | CTFL 1.4.1, 1.4.3, 1.4.4 |
| TCD-02 | `test-case-design-giai-phau-test-case` | Giải phẫu test case: ID, tiêu đề, mục tiêu, tiền điều kiện, dữ liệu, bước, kết quả mong đợi, hậu điều kiện, ưu tiên, truy vết; các lỗi viết hay gặp và cách sửa | R-ACCEPT | TCD-01 | CTFL 1.4.3; ISO/IEC/IEEE 29119-3 (tham khảo) |
| TCD-03 | `test-case-design-logic-va-cu-the` | Test case logic (high-level) và test case cụ thể (low-level); test procedure, test suite, test execution schedule, phụ thuộc giữa test case | R-FOOD | TCD-02 | CTFL 1.4.1 (test design, test implementation), 1.4.3 |
| TCD-04 | `test-case-design-expected-result-test-oracle` | Kết quả mong đợi kiểm chứng được: mọi đầu ra quan sát được (response, trạng thái lưu, nhật ký, lời gọi bên ngoài); nguồn oracle; oracle problem; chống chép kết quả từ hệ thống | R-FARE, R-CANCEL, R-PAY | TCD-02 | CTFL 1.4.1, 1.4.3; thuật ngữ test oracle (ISTQB Glossary) |
| TCD-05 | `test-case-design-test-data` | Test data cho từng test case: dữ liệu tiền điều kiện, đầu vào, mong đợi; độc lập giữa ca; builder/fixture; đồng hồ và múi giờ; dữ liệu giả lập, không dùng dữ liệu thật | R-VOUCHER-07, R-GEN-02, R-QUOTE-01 | TCD-03 | CTFL 1.4.1, 1.4.3; CTAL-TA v4.0 (bối cảnh) |
| TCD-06 | `test-case-design-phoi-hop-ky-thuat` | Chọn và phối hợp kỹ thuật cho một tính năng: ca dương/ca âm, EP → BVA → decision table → state; gộp ca dương, tách ca âm (single fault) theo thứ tự kiểm R-VALID-01; đo phủ theo từng kỹ thuật, loại ca thừa | R-VALID-01, R-PARCEL, R-FARE-02 | TCD-03, TEST-031…034 | CTFL 4.1, 4.2; CTAL-TA v4.0 (bối cảnh) |
| TCD-07 | `test-case-design-user-story-acceptance-criteria` | Test case từ user story và acceptance criteria: dạng scenario (Given/When/Then) và dạng rule; ATDD; phát hiện mơ hồ; example mapping (ngoài ISTQB) | R-CANCEL, R-UI-01 | TCD-02, TEST-016 | CTFL 4.5.1, 4.5.2, 4.5.3 |
| TCD-08 | `test-case-design-traceability-matrix` | Ma trận truy vết (RTM): xuôi/ngược, nhiều-nhiều, độ phủ yêu cầu và độ phủ test condition, lỗ hổng và ca mồ côi, phân tích ảnh hưởng khi SPEC đổi, sinh RTM từ mã test | R-STATE, R-CANCEL | TCD-01, TCD-02 | CTFL 1.4.4, 1.4.3 |
| TCD-09 | `test-case-design-uu-tien-review-bao-tri` | Ưu tiên test case (risk-based, coverage-based kiểu additional, requirements-based, có phụ thuộc); review bộ test case bằng checklist; anti-pattern; bảo trì khi yêu cầu đổi | R-PAY, R-IDEM | TCD-08 | CTFL 5.1.5 (FL-5.1.5 K3), 3.2 (review) |
| TCD-10 | `test-case-design-thuc-hanh-dat-chuyen` | Capstone: thiết kế trọn bộ test case cho «đặt chuyến từ báo giá» — phân tích, condition, kỹ thuật, logic → cụ thể, data, oracle, RTM, ưu tiên, review, chạy qua API bằng supertest, báo cáo | R-QUOTE-03, R-IDEM, R-VALID-02, R-API-08, R-API-13 | TCD-01…09 | Tổng hợp các mục trên |

Thứ tự đọc: 01 → 02 → 03 → (04, 05) → 06 → 07 → 08 → 09 → 10.

## Quy ước chung

- Thông tin bài: callout `tkh-callout` như các bài TEST, dòng Series ghi `TCD · Thiết kế Test Case`.
- Mã trong bài: test condition `TCOND-<NHÓM>-nn`, test case `TC-<NHÓM>-nnn`, dòng dữ liệu `TD-nn`; trích SPEC bằng mã `R-…` đã có. Tiêu chí mới không có trong SPEC phải ghi rõ là giả định bài tập.
- Cấu trúc 8 phần theo §6 của `toi-noi-testing-antblog.md`; bài tập có lời giải trong `<details>`.
- Chỉ ghi mã LO khi chắc chắn (danh sách đã kiểm ở `testing-master-curriculum.md` §2.2); CTAL-TA v4.0 chỉ ghi «bối cảnh», không ghi mã LO.

### Thuật ngữ thống nhất (chuẩn bị cho từ điển keyword)

Dùng đúng dạng chữ dưới đây trong văn xuôi, để bộ dò keyword khớp ổn định. Lần đầu xuất hiện trong bài có thể kèm tiếng Việt hoặc tiếng Anh trong ngoặc.

| Dạng dùng trong bài | Ghi chú |
|---|---|
| test basis | không dịch |
| test condition | không dịch; số nhiều vẫn viết «test condition» |
| test case logic (logical test case) · test case cụ thể (concrete test case) | không viết «test case cấp cao/cấp thấp» lẫn lộn; có thể nhắc «high-level/low-level» một lần |
| test procedure · test suite · test execution schedule | không dịch |
| tiền điều kiện (precondition) · hậu điều kiện (postcondition) | |
| kết quả mong đợi (expected result) | không viết «kết quả kỳ vọng» |
| test oracle | mục có sẵn trong từ điển |
| test data | không viết «dữ liệu test» lẫn lộn; «dữ liệu kiểm thử» chấp nhận được |
| ma trận truy vết (traceability matrix) | viết tắt RTM sau lần đầu |
| ca dương / ca âm (positive / negative test case) | |
| acceptance criteria · user story · Given/When/Then · ATDD | |
| ưu tiên test case (test case prioritization) | |

Danh sách ứng viên cho từ điển: [`content-dictionary/glossary-candidates/testing-test-case-design.json`](content-dictionary/glossary-candidates/testing-test-case-design.json).

## Đưa lên site

- Learning path `testing-tcd-thiet-ke-test-case`, priority 55 (sau S04/S05 = 50, trước S06 = 60), `intermediate`.
- Ảnh bìa: brief trong `apps/web/src/lib/article-covers.json`, sinh bằng `apps/web/scripts/generate-covers.mjs`; ảnh path qua `path-covers.json`.

## Trạng thái

| Mã | Trạng thái |
|---|---|
| TCD-01…TCD-10 | Đã viết (17/11/2026). Lab: 24 tệp test, 367 passed + 1 todo (`npx vitest run articles/test-case-design`). Learning path và ảnh bìa đã thêm |

Câu hỏi về SPEC phát hiện khi viết (chưa sửa SPEC, cần chủ blog/BA quyết):

- R-FARE-07/R-FARE-10: khi cước tối thiểu kéo giá lên, `discount` vẫn hiện số giảm theo R-VOUCHER-06 (xe máy 1 km GIAM20 hiện giảm 3.300 đ nhưng khách chỉ tiết kiệm 2.000 đ). R-FOOD-01: tổng đơn đồ ăn có thể lẻ nghìn (FOOD15 trên 50.000 đ ⇒ 69.500 đ).
- R-VOUCHER-02 (khoảng trắng giữa mã), R-VOUCHER-07 (huỷ có phí vẫn trả lượt? «cùng ngày» theo giờ báo giá hay giờ tạo chuyến).
- R-CANCEL: không có API xem trước phí («biết trước» trong user story TCD-07 chưa có cơ sở); «now» là giờ máy chủ của lần xử lý đầu; thứ tự `grace_period`/`driver_late` của R-CANCEL-06 không kiểm được vì ETA ≥ 1.
- R-PAY-06 + `lost_response`: tiền đã trừ nhưng `paymentStatus = failed`, chưa có quy tắc đối soát. R-PAY-04 và R-NFR-05 cùng ghi «2 lần thử lại» — sửa một chỗ sẽ mâu thuẫn.
- R-API-01 vs R-VALID-02: chưa đặt 400 `INVALID_JSON`/413 trong thứ tự kiểm; lab trả `INVALID_JSON` cả khi đã vượt rate limit (DR-TCD10-01, cùng gốc DR-050-02). R-API-13: yêu cầu đúng +60 giây có còn trong cửa sổ không. R-IDEM-02: thêm trường lạ khi retry ⇒ 422.
- R-API-06: dòng 401 lặp hai lần.

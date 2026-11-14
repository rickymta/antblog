# Khảo sát ngôn ngữ — nhóm bài kiểm thử (api/seed/content)

Chỉ đọc, không sửa file nội dung nào. Danh sách 79 file: `review-survey/group-files.txt`. Thống kê cụm từ: `review-survey/term-counts.txt`.

## 1. Phạm vi và phương pháp

- Số file trong nhóm: **79**.
- Quét `title:` và `summary:` của cả 79 file.
- Đọc kỹ phần văn xuôi (bỏ khối code, ~300 dòng đầu) của **8 bài**: `system-testing-smoke-test.md`, `unit-testing-fundamentals.md`, `system-testing-load-test.md`, `verification-va-validation.md`, `error-defect-failure-root-cause.md`, `system-testing-release-dossier.md`, `python-shap-explainability-audit-human-review.md`, `walkthrough.md`.
- Đếm cụm từ bằng grep cố định (`grep -F`) trên toàn thư mục content.
- Mức độ: cao = sai nghĩa/khó hiểu; trung bình = gượng; thấp = có thể tốt hơn.
- Loại: A = thuật ngữ dịch sai/gượng; B = diễn đạt không thuần Việt.

## 2. Lỗi trong 8 bài đọc kỹ

### 2.1 system-testing-smoke-test.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 3 (title) | «bản build có đủ tốt để bắt đầu kiểm sâu?» | A | «…để bắt đầu kiểm thử chuyên sâu?» | cao |
| 4 (summary) | «đội kiểm sâu tiếp không» | A | «đội kiểm thử chuyên sâu tiếp không» | cao |
| 21 (bảng) | «…kiểm thu tiền (R-API-15, R-API-03, R-STATE-02, R-PAY-03)» | B | «…kiểm tra số tiền đã thu» | trung bình |
| 35 | «Kiểm sâu ngay: nếu build hỏng…» | A | «Kiểm thử chuyên sâu ngay» | cao (cùng cụm) |
| 36 | «Trả build về ngay» | B | «Trả lại bản build ngay» | thấp |
| 107 | «Mapping DB, cấu hình bảng giá» | A | «ánh xạ (mapping) DB» | thấp |

Ghi chú: cụm «kiểm sâu» xuất hiện ở ít nhất 6 chỗ (cả title, summary, body, figcaption, svg); đây là lỗi lặp, sửa một lần theo từ điển là đủ. Bài dùng lẫn «case» và «ca» (ví dụ «ba case», «3 smoke case SM-01…03», «Các case của nó»), nên chọn một.

### 2.2 unit-testing-fundamentals.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 3 (title) | «Unit Testing Fundamentals: năm test đầu tiên cho cước Tới Nơi» | B | «…năm test case đầu tiên cho chức năng tính cước của Tới Nơi» | trung bình |
| 11 | «Một lần sửa hệ số cao điểm có thể khiến phụ phí mưa cũng bị nhân lên» | B | «…khiến phụ phí trời mưa cũng bị nhân lên» | thấp |
| 53 | «…FIRST được dùng theo nghĩa… ngoài phần LO nêu trên» | B | «ngoài mục tiêu học tập (LO) nêu trên» hoặc bỏ viết tắt | thấp |
| 95 | «…đáng kiểm tra trước» | B | «nên kiểm tra trước» | thấp |
| 154 | «Giữ bản thử nghiệm cạnh bài giúp các lab khác tiếp tục dùng quy tắc chuẩn» | B | Viết lại: «Giữ bản luyện tập ngay cạnh bài để các lab khác vẫn dùng quy tắc chuẩn» | trung bình |
| 198 | «…phải kiểm thêm thành phần thời gian và chỗ lắp ráp» | B | «…và cách các thành phần ghép nối với nhau» | trung bình |

### 2.3 system-testing-load-test.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 4 (summary) | «…kế toán planned = sent + dropped…» | A | «…đối soát planned = sent + dropped…» | trung bình |
| 4 (summary) | «…gate đặt trước phép đo…» | A | «…ngưỡng đạt/không đạt đặt trước khi đo…» | trung bình |
| 4 (summary) | «…mà không suy ra capacity production» | A | «…mà không suy ra công suất môi trường production» | trung bình |
| 50 | «Kiểm bất biến kế toán planned = sent + dropped và đặt gate gồm cả drop…» | A | «Kiểm bất biến đối soát… và đặt ngưỡng gồm cả drop…» | trung bình |
| 106 | «…sẽ làm đổi mix lỗi giữa chừng» | A/B | «…sẽ làm thay đổi cơ cấu lỗi giữa chừng» | trung bình |
| 161 | «…không bắn bù thành burst» | A/B | «…không gửi bù thành đợt dồn» | thấp |
| 167 | «…tạo ra một spike nhân tạo» | A | «…tạo ra một đỉnh tải nhân tạo» | thấp |
| 212 | «Exit của bài tập: kế toán khớp, chạy đủ profile, gate đạt…» | A | «đối soát khớp, chạy đủ profile, đạt ngưỡng…» | trung bình |

Ghi chú: «kế toán» ở đây là ẩn dụ (không phải nghiệp vụ kế toán), chỉ xuất hiện 3 lần trong bài này. Các lần «kế toán» còn lại trong nhóm (ví dụ `system-testing-data-integrity-test.md`, `maintenance-testing.md`) dùng đúng nghĩa.

### 2.4 verification-va-validation.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 11 | «…mọi test của công thức cước đều xanh, ví dụ kiểm trong đặc tả ra đúng 52.000 đ» | B | «…ví dụ trong đặc tả cho ra đúng 52.000 đ» | trung bình |
| 148 | «…giá của chuyến là ảnh chụp giá của báo giá (R-QUOTE-04)» | A | «…giá của chuyến là bản chụp giá của báo giá» (hoặc «bản sao giá») | trung bình |
| 253 | «Lệnh và kết quả chạy thật (…; bỏ 6 dòng ✓ V1…Q2):» | B | «…(đã lược bỏ 6 dòng ✓ V1…Q2):» | thấp |
| 44 | «Chuyến R-FARE-09 phải ra 52.000 đ» | B | «phải cho ra 52.000 đ» | thấp |

### 2.5 error-defect-failure-root-cause.md

Bài sạch hơn các bài khác. Chỉ có hai điểm nhỏ:

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 192 | «Code đúng; máy tính phí chạy nhanh 5 giây so với máy ghi acceptedAt» | B | «Code đúng; đồng hồ của máy tính phí nhanh hơn máy ghi acceptedAt 5 giây» | thấp–trung bình |
| 11 | «…không nhận được đồng phí bù nào» | B | «…không nhận được khoản bù phí nào» | thấp |

### 2.6 system-testing-release-dossier.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 4 (summary) | «gắn mỗi bằng chứng với commit, môi trường và đường dẫn nó phụ thuộc» | A/B | «…với commit, môi trường và danh sách tệp mà nó phụ thuộc» (watchPaths) | cao |
| 4 (summary) | «phát hiện bằng chứng đã cũ so với ứng viên» | A | «…so với bản ứng viên phát hành» | trung bình |
| 3 (title) | «gom mười sáu phép kiểm thành một quyết định release…» | B | «gom mười sáu hoạt động kiểm thử…» | thấp |
| 4 (summary) | «Dossier thật của lab Tới Nơi hôm nay» | A | «Hồ sơ phát hành thật…» (hoặc giữ «dossier» và định nghĩa một lần) | thấp–trung bình |
| 34 | «Đọc cùng nhau theo một chính sách» | B | «Đọc tất cả theo cùng một chính sách» | trung bình |
| 35 | «Một bảng toàn chữ "pass" không phát hành sản phẩm» | B | «…không đủ để phát hành sản phẩm» | thấp |
| 61 | «…kiểm độ mới theo commit… là thiết kế thực tiễn của series» | A/B | «…kiểm tra độ mới theo commit… là thiết kế thực hành của series» | trung bình |
| 249 | «sửa tài liệu không làm ứng viên mơ hồ» | B | «…không làm trạng thái ứng viên mơ hồ» | thấp–trung bình |
| 11 | «Người làm UAT: "Bộ kịch bản đã sẵn, chưa có buổi nào."» | B | «…chưa có phiên nào» | thấp |

Ghi chú: «kiểm độ mới» có trong bảng/sơ đồ (dòng 127, 136) và câu mô tả (dòng 61); đề xuất đổi đồng bộ. «ứng viên» đúng nghĩa candidate ở nhiều bài khác trong nhóm; chỉ sửa ở đây khi nói về bản phát hành.

### 2.7 python-shap-explainability-audit-human-review.md

Bài này có mật độ lỗi cao nhất trong 8 bài đọc.

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 4 (summary) | «…và đo công suất, SLA, dải điểm đáng duyệt» | A/B | «…và đo công suất, SLA, các dải điểm cần người duyệt» | cao |
| 4 (summary) | «ghi nhật ký quyết định khó sửa» | A/B | «ghi nhật ký quyết định chỉ ghi thêm, không sửa được» (đúng với nội dung bài) | trung bình |
| 4 (summary) | «chỉ RV-001 được chốt» | B | «chỉ RV-001 được quyền chốt» | thấp |
| 11 | «…một nhật ký mà service chấm điểm không viết lại được» | B | «…một nhật ký mà dịch vụ chấm điểm không viết lại được» (bài dùng «service» 3 lần, «dịch vụ» 3 lần) | thấp |
| 13 | «…nên lab kiểm đường ống và ràng buộc» | A | «…nên lab kiểm luồng xử lý và ràng buộc» | trung bình |
| 18 (callout) | «nhân một đóng góp SHAP với 0,943 được đóng góp ở thang log-odds của p» | B | «nhân một đóng góp SHAP với 0,943 thì thu được đóng góp ở thang log-odds của p» | trung bình |
| 47 | «Lab chạy trên ngày 50 đến 60» | B | «Lab dùng dữ liệu ngày 50 đến 60» | thấp–trung bình |
| 58 | «log-odds của p là hàm affine của điểm thô» | A | «…là hàm tuyến tính (affine) của điểm thô» | thấp |
| 67 | «…một trong hai mang một tạo tác của dữ liệu» | A | «…một trong hai mang dấu vết của bộ sinh dữ liệu» | thấp |
| 71, 123 | «…một họ tín hiệu»; «Họ tín hiệu có đóng góp lớn nhất» | A | «nhóm tín hiệu» | trung bình |
| 94 | «…dịch thành tập đặc trưng mong đợi» | B | «…chuyển thành tập đặc trưng kỳ vọng» | thấp–trung bình |
| 131 | «…khách gian lận không dùng mã khuyến mãi nào ngoài NEWUSER50, nên chuyến sạch hay có lịch sử khuyến mãi còn chuyến gian lận thì hầu như không» | B | «…nên chuyến sạch thường có lịch sử khuyến mãi, còn chuyến gian lận thì hầu như không» (câu hiện thiếu vị ngữ) | cao |
| 142 | «Hàng đợi nằm ở schema lab61 của PostgreSQL hạ tầng chung» | B | «…của PostgreSQL dùng chung trên hạ tầng» | trung bình |
| 142 | «Ưu tiên là chi phí kỳ vọng của bỏ sót» | B | «…chi phí kỳ vọng của việc bỏ sót» | thấp |
| 167 | «Việc ghi phải idempotent» | A | Giữ hoặc chú thích một lần «(ghi lặp không làm đổi kết quả)» | thấp |
| 189 | «…một câu UPDATE viết vội phá được» | B | «…một câu UPDATE viết vội có thể phá được» | thấp |
| 216 | «pii.py cất đi ngày giờ và tiền có dấu chấm ngăn nghìn» | B | «pii.py trước hết loại khỏi diện che các ngày giờ và số tiền có dấu chấm ngăn nghìn» | trung bình |
| 216 | «…quét… và che dãy từ 9 chữ số trở lên… (còn che thêm được gì thì là rò)» | B | «…(nếu hàm còn che thêm được gì thì đó là rò rỉ)» | cao |
| 224 | «Ở dữ liệu thật hãy dùng mã định danh giả (HMAC-SHA-256 có khóa…)» | A | «mã định danh băm có khóa (pseudonym)»: «giả» dễ hiểu là fake | trung bình |
| 228 | «Tỉ lệ sai cho phép» rơi ra từ chi phí… | B | «…được suy ra từ chi phí…» | thấp |
| 228 | «Lab… đếm lãi ròng bằng nhãn cuối» | B | «…tính lãi ròng theo nhãn cuối» | thấp |
| 236 | «điểm hòa vốn p* = phí duyệt / (tiền chuyến + xử lý)» | A | Ghi rõ «+ 20.000 đ chi phí bỏ sót»: «xử lý» mơ hồ | trung bình |
| 239 | «…mà tổng vẫn chưa lỗ» | B | «…mà tổng vẫn có lãi ròng dương» | thấp–trung bình |
| 239 | «…«chờ p95» chỉ tính đề xuất đã được duyệt nên bị cắt khi còn tồn đọng» | B | «…nên bị thiên lệch khi còn tồn đọng» | trung bình |
| 259 | «…từ 65 trở lên chỉ còn 5» | B | «…từ 65 trở lên tồn đọng chỉ còn 5 đề xuất» | thấp–trung bình |
| 259 | «Một seed chưa đủ cho kết luận thứ bậc» | A | «…kết luận về thứ tự xếp hạng» | thấp |
| 4 / 13 | «service», «audit», «gain» không giải thích hoặc dùng lẫn tiếng Việt | A | Thống nhất, chú thích lần đầu | thấp |

### 2.8 walkthrough.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 120 (callout) | «…điểm dừng có câu hỏi soạn sẵn do người khác tác giả (thường là QA) chuẩn bị» | B | «…do người không phải tác giả (thường là QA) chuẩn bị» | trung bình |
| 177 | «…gắn vào đúng bước dễ hiểu sai» | B | «…gắn vào những bước dễ bị hiểu sai» | trung bình–thấp |
| 263 (bảng Q5) | «…có cao điểm, mưa (R-FOOD-04)» | B | «…có tính phụ phí cao điểm và phụ phí mưa (R-FOOD-04)»: đơn mẫu 09:00 và không mưa nên cụm hiện tại dễ đọc nhầm | cao |
| 282 | «…ở hoạt động communication and analysis» | A | «…ở bước giao tiếp và phân tích» (bảng 2.3 cũng để nguyên tiếng Anh) | trung bình |
| 100–103 (bảng 2.3) | Cột «Hoạt động (3.2.2)»: Planning, Review initiation, Individual review, Communication and analysis, Fixing and reporting | A | Dùng tên tiếng Việt của syllabus | thấp–trung bình |
| 87 (hình 1) | «…tìm tối đa anomaly, thu metric» | A/B | «…tìm tối đa điểm bất thường, thu thập số liệu» | trung bình |
| 120 | «time-box 45–60 phút» | A | «giới hạn thời gian 45–60 phút» | thấp |
| 165, 254 | «domain» | A | «nghiệp vụ» | thấp |
| 44 | «moderator» | A | «người điều phối» (nhất quán với bảng vai trò) | thấp |
| 105 | «người vừa trình bày vừa ghi chép sẽ bỏ sót» | B | «…dễ bỏ sót» | thấp |

Ghi chú: «work product» và «anomaly» là thuật ngữ ISTQB dùng dày đặc trong nhóm (xem mục 3). Có thể giữ, nhưng bài `error-defect-failure-root-cause.md` đã dùng «sản phẩm công việc» và «điều bất thường», nên nên thống nhất.

## 3. Cụm từ lặp (đếm trên toàn thư mục content, grep -F)

| Cụm hiện tại | Đề xuất | Số lần (toàn thư mục) | Số file | Nhận xét |
|---|---|---|---|---|
| kiểm sâu | kiểm thử chuyên sâu / kiểm sâu hơn | 12 | 6 | Lỗi A rõ ràng, sửa đồng loạt |
| work product | sản phẩm công việc | 113 | 15 | Nhất quán: bài error-defect đã dùng tiếng Việt. Không sai, nhưng lệch nhau |
| anomaly | điểm bất thường (giữ + chú thích một lần) | 128 | 16 | Thuật ngữ ISTQB, có thể giữ. Nên thống nhất |
| ứng viên (nghĩa bản phát hành) | bản ứng viên phát hành | 290 (toàn thư mục, gồm cả nghĩa «ứng viên» thông thường) | 46 | Phần lớn dùng đúng. Chỉ sửa trong ngữ cảnh release |
| kế toán (ẩn dụ đối soát) | đối soát | 3 | 1 (load-test) | Các lần khác là nghĩa đúng |
| họ tín hiệu | nhóm tín hiệu | 2 | 1 (shap) | Lỗi A nhỏ |
| kiểm độ mới | kiểm tra độ mới | 4 | 1 (release) | Gượng |
| đường ống (pipeline) | luồng xử lý | 32 | 16 (toàn thư mục) | Trong nhóm chỉ 1 chỗ (shap dòng 13) |
| ảnh chụp (nghĩa snapshot) | bản chụp / bản sao | 114 | 64 (toàn thư mục) | Chưa đối chiếu hết nghĩa. Chỉ xác nhận 1 chỗ sai (verification dòng 148) |
| gate | cổng kiểm / ngưỡng | 765 | 173 (toàn thư mục) | Dùng «gate» rất phổ biến trong VN QA; chỉ nên dịch lần đầu. Không xếp là lỗi |
| capacity | công suất | 91 | 38 | Nhóm dùng lẫn «capacity» và «công suất». Nên chuẩn hoá |
| cước sàn / mức sàn / cước tối thiểu | cước tối thiểu | — | — | Lẫn nhau trong `unit-test-design-pricing-engine.md` và `unit-testing-fundamentals.md`. Nên chọn một |

Cụm không phải lỗi, không nên sửa: domain (nghĩa kỹ thuật, 651 lần toàn thư mục), bỏ sót, phép kiểm (nghĩa «test»), dossier (tên riêng của bài).

## 4. Các điểm đáng chú ý trong 79 file (chỉ từ title/summary)

Đây là các chỗ tôi thấy khi quét title và summary (dòng 3 và 4) của các bài không đọc kỹ. Ký hiệu «summary, dòng 4».

- `informal-review.md` (summary, dòng 4): «không quy trình xác định, không bắt buộc đầu ra ghi chép» là câu cụt, thiếu động từ. Đề xuất: «không có quy trình xác định, không bắt buộc ghi lại kết quả». Mức trung bình.
- `combinatorial-testing.md` (summary, dòng 4): «giá trị lỗi che tổ hợp» là dịch từng chữ của «error masking». Đề xuất: «giá trị gây lỗi che khuất tổ hợp khác». Mức trung bình.
- `bay-nguyen-ly-kiem-thu.md` (summary, dòng 4): «đếm thử không gian đầu vào» dễ đọc thành «đếm thử». Đề xuất: «thử đếm số tổ hợp đầu vào…». Mức thấp.
- `random-testing.md` (summary, dòng 4): «thu nhỏ một lỗi cước tối thiểu» là dịch «shrinking». Đề xuất: «rút gọn về phản ví dụ tối thiểu». Mức trung bình–thấp.
- `unit-test-design-pricing-engine.md` (summary, dòng 4): «chuẩn hoá quãng đường… cước sàn» mơ hồ «chuẩn hoá». Đề xuất: «xử lý quãng đường làm tròn…; cước tối thiểu». Mức thấp–trung bình.
- `system-testing-recovery-test.md` (summary, dòng 4): «việc mới sau outage» là câu cụt. Đề xuất: «việc mới phát sinh sau sự cố». Mức thấp.
- `ba-thuc-hanh-review-goi-yeu-cau.md` (summary, dòng 4): «kết luận phần nào đủ để phát triển» nên là «phần nào đã đủ điều kiện để phát triển». Mức thấp–trung bình.
- `api-testing-tu-goc-nhin-code.md` (summary, dòng 4): «kể cả nhánh khó gây ra» → «khó kích hoạt». Mức thấp.
- `mc-dc-testing.md` (summary, dòng 4): «điều kiện ghép» mơ hồ. Mức thấp.
- `system-testing-usability-test.md` (summary, dòng 4): «formative», «outcome» để nguyên tiếng Anh; cân nhắc «đánh giá hình thành», «kết quả». Mức thấp.
- `system-testing-deployment-test.md` (SYS-01 chạy smoke trên ứng dụng «lắp ráp trong tiến trình test», dòng 67): «lắp ráp» là dịch từng chữ, nên là «được dựng sẵn». Mức thấp.

## 5. Mật độ và nhận xét

- Trong 8 bài đọc kỹ: khoảng 55 mục ghi nhận, trong đó ~6 mục mức cao, ~25 mục trung bình, còn lại thấp. Trung bình khoảng 7 mục/bài. Bài nhiều nhất là `python-shap-explainability-audit-human-review.md` (~24 mục). Bài ít nhất là `error-defect-failure-root-cause.md` (2 mục).
- Ước lượng toàn nhóm: khoảng 4–7 mục/bài. Con số này chỉ dựa trên 8 bài đọc kỹ và quét title/summary, không đáng tin cho toàn bộ 79 bài.
- Kiểu lỗi phổ biến nhất:
  1. Câu dịch máy có trật tự kiểu Anh hoặc thiếu vị ngữ/danh từ (ví dụ: «Chuyến R-FARE-09 phải ra…», «…chỉ còn 5», «…PostgreSQL hạ tầng chung», «…chuyến sạch hay có lịch sử khuyến mãi còn chuyến gian lận thì hầu như không»).
  2. Ẩn dụ hoặc calque không rõ nghĩa («kế toán» đối soát, «ảnh chụp giá», «đường ống», «họ tín hiệu», «cất đi», «dải điểm đáng duyệt»).
  3. Thuật ngữ tiếng Anh giữ lẫn tiếng Việt, không nhất quán giữa các bài (work product, anomaly, capacity/công suất, service/dịch vụ, gate).
- Lỗi phổ biến nhất về nội dung (cao): cụm «kiểm sâu» và một số câu lược mất danh từ/vị ngữ trong các bài có nhiều công thức (SHAP, walkthrough).
- Lỗi A lặp lại thực sự ít; phần lớn là lỗi B một lần, rải rác. Việc sửa theo từ điển (kiểm sâu, work product, anomaly, capacity) sẽ bao phủ được phần đáng kể các lỗi lặp; các câu cụt cần sửa tay.

## 6. Ba ví dụ tiêu biểu

1. `python-shap-explainability-audit-human-review.md`, dòng 216: «(còn che thêm được gì thì là rò)» → «(nếu hàm còn che thêm được gì thì đó là rò rỉ)». Cao.
2. `walkthrough.md`, dòng 263: «có cao điểm, mưa (R-FOOD-04)» → «có tính phụ phí cao điểm và phụ phí mưa (R-FOOD-04)». Cao. Đơn mẫu 09:00 không mưa nên cụm cũ dễ bị đọc nhầm.
3. `system-testing-smoke-test.md`, dòng 3: «…bắt đầu kiểm sâu?» → «…bắt đầu kiểm thử chuyên sâu?». Cao, lặp ở dòng 4 và 35.

Ngoài ra: `system-testing-release-dossier.md`, dòng 4: «đường dẫn nó phụ thuộc» → «danh sách tệp mà nó phụ thuộc». Cao.

## 7. Ghi chú về phạm vi

- Không sửa file nội dung nào. Chỉ tạo báo cáo này và `group-files.txt`, `term-counts.txt` trong scratchpad.
- Các mức độ và đề xuất là gợi ý của người khảo sát. Biên tập viên có thể quyết định khác về văn phong.
- Một số cụm đếm trên toàn thư mục bao gồm cả nghĩa không thuộc nhóm kiểm thử (ví dụ «ứng viên», «kế toán», «ảnh chụp»). Các con số ở mục 3 cần đọc cùng ngữ cảnh.

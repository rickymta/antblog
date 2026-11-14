# Khảo sát chất lượng tiếng Việt: nhóm python-*.md

Thư mục: H:\Work\tech-blog\api\seed\content
Chỉ đọc, không sửa file nội dung nào.

## Phạm vi và phương pháp

- Số file trong nhóm: 66 (python-*.md), tổng khoảng 20.444 dòng.
- Đã rà title và summary của cả 66 bài (grep dòng `title:` và `summary:`).
- Đã đọc kỹ phần văn xuôi của 8 bài (có bỏ qua phần lớn khối code): 01 va-uv, 02 bien-kieu, 23 concurrency, 39 outbox-inbox, 45 danh-gia-ocr, 59 real-time-scoring, 61 shap-explainability, 70 use-case-ho-so. Mỗi bài đọc tối đa 300 dòng đầu.
- Đếm mật độ các cặp thuật ngữ trên toàn bộ thư mục content (mọi file .md, không chỉ nhóm python).

Ký hiệu: A = thuật ngữ bị dịch sai hoặc gượng; B = diễn đạt không thuần Việt (câu cụt, lược danh từ, calque, từ ghép tự chế). Mức độ: cao = sai nghĩa hoặc khó hiểu; trung = gượng; thấp = có thể tốt hơn.

## Danh sách lỗi

### Title và summary (dòng 3-4 của frontmatter)

| # | File | Dòng | Gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|---|---|
| 1 | python-design-pattern-strategy-state-adapter.md | 4 | "Adapter bọc SDK bản đồ trả mét" | B | "Adapter bọc SDK bản đồ, đổi khoảng cách trả về theo mét" | cao |
| 2 | python-real-time-scoring-feature-store-redis-latency.md | 4 | "cầu dao" | A | "ngắt mạch (circuit breaker)" | cao |
| 3 | python-real-time-scoring-feature-store-redis-latency.md | 4 | "từ chối bớt tải" | A | "giảm tải chủ động (load shedding)" | trung |
| 4 | python-real-time-scoring-feature-store-redis-latency.md | 4 | "độ trễ tiêm vào" | B | "độ trễ được chèn vào (fault injection)" | thấp |
| 5 | python-shap-explainability-audit-human-review.md | 4 | "ghi nhật ký quyết định khó sửa" | B | "ghi nhật ký quyết định không thể sửa, chỉ ghi thêm" | cao |
| 6 | python-shap-explainability-audit-human-review.md | 4 | "dải điểm đáng duyệt" | B | "dải điểm cần người duyệt" | trung |
| 7 | python-use-case-ho-so-tai-xe-end-to-end.md | 4 | "Máy luôn dừng ở needs_review" | B | "Hệ thống luôn dừng ở needs_review" | trung |
| 8 | python-outbox-inbox-idempotent-consumer.md | 4 | "dispatcher thuê dòng bằng SKIP LOCKED" | A | "dispatcher giữ tạm dòng bằng lease và SKIP LOCKED" | trung |
| 9 | python-outbox-inbox-idempotent-consumer.md | 4 | "consumer khử trùng bằng inbox" | A | "consumer loại trùng bằng inbox" | trung |
| 10 | python-ocr-service-fastapi-worker-timeout.md | 4 | "tải thử p50/p95" | A | "kiểm thử tải, đo p50/p95" | trung |
| 11 | python-ocr-service-fastapi-worker-timeout.md | 4 | "tiến trình con có thể kill" | A | "tiến trình con có thể bị dừng cưỡng bức" | thấp |
| 12 | python-face-match-grpc-service-batching-model-loading.md | 4 | "deadline hai đầu" | B | "hạn chờ ở cả hai phía" | trung |
| 13 | python-danh-gia-ocr-cer-wer-threshold.md | 4 | "chọn ngưỡng đưa người duyệt từ số liệu" | B | "chọn ngưỡng để chuyển hồ sơ cho người duyệt dựa trên số liệu" | trung |
| 14 | python-danh-gia-ocr-cer-wer-threshold.md | 4 | "phân tích thay/xóa/chèn theo trường" | B | "phân tích lỗi thay thế, xóa, chèn theo trường" | thấp |
| 15 | python-rule-engine-scoring-strategy-explainability.md | 4 | "chỉ ra khi nào rule hết đủ" | B | "chỉ ra khi nào bộ rule không còn đủ" | trung |
| 16 | python-rule-engine-scoring-strategy-explainability.md | 4 | "cấu hình YAML kiểm khi nạp" | B | "cấu hình YAML được kiểm tra khi nạp" | thấp |
| 17 | python-hau-xu-ly-ocr-regex-parse-truong-checksum.md | 4 | "confidence là min các từ" | B | "độ tin cậy là giá trị nhỏ nhất trong các từ" | trung |
| 18 | python-luu-ket-qua-ocr-mongodb-pii-retention.md | 4 | "TTL index cho bản gốc", "audit ai xem" | B | "chỉ mục TTL để tự xóa bản gốc sau thời hạn", "nhật ký kiểm toán ai đã xem" | trung |
| 19 | python-feature-engineering-tu-event-kafka.md | 4 | "tính online khớp tính theo lô" | B | "tính trực tuyến khớp với tính theo lô" | thấp |
| 20 | python-redis-cache-rate-limit-geo.md | 4 | "Redis không phải nguồn sự thật" | A | "Redis không phải nguồn dữ liệu chuẩn" | trung |
| 21 | python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric.md | 4 | "chi phí hai chiều" | B | "chi phí của hai loại sai sót" | trung |
| 22 | python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric.md | 4 | "nhãn trễ" | B | "nhãn đến muộn" | thấp |
| 23 | python-face-embedding-cosine-threshold-far-frr.md | 4 | "báo riêng ảnh không lấy được mặt" | B | "báo riêng các ảnh không tách được khuôn mặt" | thấp |
| 24 | python-face-embedding-cosine-threshold-far-frr.md | 4 | "kiểm trên nhiều cách chia người" | B | "kiểm trên nhiều cách chia theo người" | thấp |
| 25 | python-mongodb-pymongo-motor-document-ocr.md | 4 | "Motor đã deprecated" | A | "Motor đã bị đánh dấu lỗi thời (deprecated)" | thấp |
| 26 | python-decorator-context-manager.md | 4 | "Kiểm thử bằng FakeTimer, không ngủ thật" | B | "không dùng sleep thật" | thấp |
| 27 | python-frozen-dataclass-enum-immutability.md | 4 | "chỉ ra frozen là bản nông" | B | "frozen chỉ bất biến ở mức nông" | thấp |
| 28 | python-match-pattern-matching-none.md | 4 | "tách None, chuỗi rỗng và khóa vắng" | B | "tách None, chuỗi rỗng và khóa không có trong dict" | trung |
| 29 | python-functools-itertools-pipeline.md | 4 | "groupby phải sort trước" | B | "groupby cần sắp xếp dữ liệu trước" | trung |
| 30 | python-class-dataclass-property-encapsulation.md | 4 | "advance() từ chối chuyển sai" | B | "advance() từ chối chuyển trạng thái không hợp lệ" | trung |
| 31 | python-dieu-kien-vong-lap-ham.md | 4 | "từ chối ngoài miền bằng ValueError" | B | "từ chối giá trị ngoài miền hợp lệ bằng ValueError" | thấp |
| 32 | python-fastapi-jwt-oauth2-authorization.md | 4 | "có chín ca token xấu" | A | "có chín trường hợp token xấu" | thấp |
| 33 | python-integration-test-testcontainers-fixture.md | 4 | "schema riêng mỗi ca" | A | "schema riêng cho mỗi ca kiểm thử" | thấp |
| 34 | python-dong-goi-ruff-pre-commit-cau-truc-du-an.md | 4 | "sẵn sàng cho service" | B | "sẵn sàng để đưa thành service" | thấp |
| 35 | python-list-dict-set-tuple.md | 4 | "hiểu mutability, alias, hashability" | A | "tính khả biến, bí danh, khả băm" | thấp |
| 36 | python-face-detection-landmarks-mediapipe-opencv.md | 4 | "gom lý do chuyển người duyệt" | B | "gom lý do chuyển cho người duyệt" | thấp |

### Thân bài đã đọc (8 bài)

| # | File | Dòng | Gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|---|---|
| 37 | python-concurrency-lock-queue-semaphore.md | 11 | "Hai tài xế cùng chạy tới đón một khách, hệ thống chỉ còn tên người ghi sau." | B | "...hệ thống chỉ ghi nhận tên của người ghi sau cùng" | trung |
| 38 | python-concurrency-lock-queue-semaphore.md | 200 | "Ta đổi thời gian lấy việc phía sau không bị quá tải." | B | "Ta đánh đổi thời gian để service phía sau không bị quá tải" | trung |
| 39 | python-concurrency-lock-queue-semaphore.md | 238 | "mở khóa join() dù việc chưa làm" | B | "khiến join() trả về dù việc chưa xong" | trung |
| 40 | python-concurrency-lock-queue-semaphore.md | 77 | "Không tái nhập." | B | "Lock không hỗ trợ tái nhập (non-reentrant)" | thấp |
| 41 | python-concurrency-lock-queue-semaphore.md | 99 | "cửa sổ race rộng" | A | "khoảng tranh chấp rộng hơn" | thấp |
| 42 | python-outbox-inbox-idempotent-consumer.md | 114 | "SKIP LOCKED cho một góc nhìn không nhất quán về dữ liệu" | B | "SKIP LOCKED cho kết quả không nhất quán (bỏ qua dòng đang bị khóa)" | cao |
| 43 | python-outbox-inbox-idempotent-consumer.md | 236 | "chưa có chỗ cất nó (ví dụ một topic dead-letter riêng)" | B | "cho tới khi có nơi chuyển nó đi, ví dụ một topic dead-letter" | trung |
| 44 | python-outbox-inbox-idempotent-consumer.md | 224 | "in tên ca" | A | "in tên trường hợp kiểm tra" | thấp |
| 45 | python-danh-gia-ocr-cer-wer-threshold.md | 206 | "Riêng môi trường được kiểm tách bằng check_environment" | B | "được kiểm riêng bằng check_environment" | trung |
| 46 | python-danh-gia-ocr-cer-wer-threshold.md | 213 | "thử phá 1 (thu ảnh còn một nửa trước OCR)" | B | "thu nhỏ ảnh còn một nửa trước OCR" | trung |
| 47 | python-danh-gia-ocr-cer-wer-threshold.md | 179 | "một ngày hợp lệ nên không kiểm tra nào bắt được" | B | "ngày bị đọc sai vẫn là ngày hợp lệ nên không kiểm tra nào bắt được" | trung |
| 48 | python-danh-gia-ocr-cer-wer-threshold.md | 107 | "ảnh tối tệ nhất" | B | "ảnh tối là tệ nhất" | thấp |
| 49 | python-danh-gia-ocr-cer-wer-threshold.md | 248 | "bộ test vàng thật (không regression...)" | B | "(không có hồi quy...)" | thấp |
| 50 | python-danh-gia-ocr-cer-wer-threshold.md | 70 | "CER toàn bộ 0,0943 là số trộn" | B | "số hỗn hợp" | thấp |
| 51 | python-real-time-scoring-feature-store-redis-latency.md | 131 | "đây là các số của một máy ồn, không phải máy rảnh" | B | "đây là các số của một máy đang bận, không phải máy rảnh" | trung |
| 52 | python-real-time-scoring-feature-store-redis-latency.md | 149 | "Thứ ít phụ thuộc tải hơn thời gian trôi qua là CPU tiêu thụ." | B | "Đại lượng ít phụ thuộc tải hơn thời gian trôi qua là CPU tiêu thụ" | trung |
| 53 | python-real-time-scoring-feature-store-redis-latency.md | 159 | "trong 50 ms một tiến trình chỉ phục vụ nổi cỡ 15 chuyến chồng nhau" | B | "...chỉ xử lý kịp khoảng 15 chuyến chồng nhau" | trung |
| 54 | python-real-time-scoring-feature-store-redis-latency.md | 202 | "đồng hồ hỏng mà không ai biết" | B | "ngân sách thời gian bị vượt mà không ai biết" | trung |
| 55 | python-real-time-scoring-feature-store-redis-latency.md | 246 | "cust_promo_share_30d > 0 gần như tố «sạch»" | B | "gần như lộ ra là «sạch»" | cao |
| 56 | python-real-time-scoring-feature-store-redis-latency.md | 246 | "loại key khi đầy bộ nhớ" | A | "chính sách loại bỏ key khi đầy bộ nhớ (eviction)" | thấp |
| 57 | python-real-time-scoring-feature-store-redis-latency.md | 177 | "thêm bản service" | B | "thêm bản sao của service" | thấp |
| 58 | python-real-time-scoring-feature-store-redis-latency.md | 215 | "phân biệt ban đêm vắng khách với updater tắc" | B | "updater bị nghẽn" | thấp |
| 59 | python-shap-explainability-audit-human-review.md | 131 | "chuyến sạch hay có lịch sử khuyến mãi còn chuyến gian lận thì hầu như không" | B | "chuyến sạch thường có lịch sử khuyến mãi, còn chuyến gian lận thì hầu như không" | cao |
| 60 | python-shap-explainability-audit-human-review.md | 18 | "nhân một đóng góp SHAP với 0,943 được đóng góp ở thang log-odds" | B | "khi nhân đóng góp SHAP với 0,943, kết quả được quy về thang log-odds" | trung |
| 61 | python-shap-explainability-audit-human-review.md | 71, 124 | "họ tín hiệu" | A | "nhóm tín hiệu" | thấp |
| 62 | python-shap-explainability-audit-human-review.md | 189 | "Hai mươi ca thử, mỗi ca phải bị chặn" | A | "Hai mươi trường hợp thử, mỗi trường hợp phải bị chặn" | thấp |
| 63 | python-shap-explainability-audit-human-review.md | 216 | "pii.py cất đi ngày giờ và tiền" | B | "pii.py bỏ qua ngày giờ và tiền khi tìm số giấy tờ" | trung |
| 64 | python-shap-explainability-audit-human-review.md | 216 | "(còn che thêm được gì thì là rò)" | B | "(nếu còn che thêm được thì đó là chỗ rò)" | thấp |
| 65 | python-shap-explainability-audit-human-review.md | 240 | "«chờ p95» ... nên bị cắt khi còn tồn đọng" | B | "nên bị thiếu, chệch khi còn tồn đọng" | trung |
| 66 | python-shap-explainability-audit-human-review.md | 228 | "«Tỉ lệ sai cho phép» rơi ra từ chi phí" | B | "được suy ra từ chi phí" | thấp |
| 67 | python-use-case-ho-so-tai-xe-end-to-end.md | 82 | "không dịch thì lỗi đó rollback cả hồ sơ" | B | "nếu không chuyển đổi lỗi thì lỗi đó rollback cả hồ sơ" | trung |
| 68 | python-use-case-ho-so-tai-xe-end-to-end.md | 80 | "Không có điểm «giả định» nào được điền vào chỗ trống." | B | "Không có giá trị giả định nào được điền vào chỗ trống." | trung |
| 69 | python-use-case-ho-so-tai-xe-end-to-end.md | 168 | "đừng thừa kế cấu hình Tesseract từ bài khác" | B | "đừng dùng lại cấu hình Tesseract từ bài khác" | trung |
| 70 | python-use-case-ho-so-tai-xe-end-to-end.md | 168 | "OcrResult.confidence là một số theo giấy tờ" | B | "tính riêng cho từng giấy tờ" | thấp |
| 71 | python-use-case-ho-so-tai-xe-end-to-end.md | 17 | "OCR đọc đúng chữ trên thẻ giả vẫn đọc đúng" | B | "OCR đọc đúng chữ trên thẻ giả" (lặp) | thấp |
| 72 | python-use-case-ho-so-tai-xe-end-to-end.md | 164 | "FakeRiskScore kẹp cosine âm về 0" | B | "FakeRiskScore chặn cosine âm về 0" | thấp |
| 73 | python-use-case-ho-so-tai-xe-end-to-end.md | 84 | "có ca kiểm số lần gọi bằng 0" | A | "có một ca kiểm thử xác nhận số lần gọi bằng 0" | thấp |
| 74 | python-use-case-ho-so-tai-xe-end-to-end.md | 94 | "hồ sơ lạ là 404" | B | "hồ sơ không tồn tại thì trả 404" | thấp |
| 75 | python-use-case-ho-so-tai-xe-end-to-end.md | 146 | "Ca biên của ngưỡng" | A | "Trường hợp biên của ngưỡng" | thấp |
| 76 | python-va-uv-tu-source-code-den-chuong-trinh.md | 223 | "không tự động là quyết định xác minh hay kết luận gian lận" | B | "không phải là quyết định tự động xác minh hay kết luận gian lận" | thấp |
| 77 | python-va-uv-tu-source-code-den-chuong-trinh.md | 91 | "nếu thư mục chưa nằm trong repo nào" | B | "nếu thư mục chưa nằm trong repo Git nào" | thấp |
| 78 | python-bien-kieu-du-lieu-decimal.md | 251 | "Hãy tìm đầu vào bộc lộ sai lệch." | B | "Hãy tìm một đầu vào cho thấy sai lệch." | thấp |
| 79 | python-bien-kieu-du-lieu-decimal.md | 245 | "cấu hình có hiệu lực theo thời điểm" | B | "cấu hình có hiệu lực theo thời gian" | thấp |

Tổng kê danh sách trên: 79 mục (A: 18, B: 61). Mức cao: 6; trung: 33; thấp: 40.

## Cặp thuật ngữ lặp lại (đếm trên toàn thư mục content)

Các số đếm bằng grep trên mọi file .md, không phân biệt hoa thường. Số có dấu ~ là ước lượng vì regex thô, có nhiễu.

| Từ đang dùng | Đề xuất | Số lần / số file | Ghi chú |
|---|---|---|---|
| khử trùng | loại trùng / khử trùng lặp (dedupe) | 49 / 23 | Nghĩa "khử trùng" gốc là diệt khuẩn, dễ hiểu sai |
| nguồn sự thật | nguồn dữ liệu chuẩn (source of truth) | 45 / 26 | Calque từ "source of truth" |
| output thật | kết quả chạy thật / kết quả thực tế | 43 / 27 | Có cả "số đo thật" (23 / 20) và "có số đo thật" (44 / 34), mức thấp |
| Máy làm chủ ngữ | hệ thống | ~99 / 61 | Calque "machine" của "system"; đếm gồm cả nghĩa máy thật |
| ca (trường hợp kiểm thử) | trường hợp / ca kiểm thử | ~190 / ~100 | Dùng "ca" như "case", ví dụ "có chín ca token xấu" |
| cầu dao | ngắt mạch (circuit breaker) | 7 / 2 | Cầu dao là công tắc điện; bài aws-cloudwatch dùng với nghĩa khác |
| thuê (lease) | giữ tạm (lease) | 11 / 5 | Ví dụ "thuê dòng", "thuê một lô dòng" |
| tải thử | kiểm thử tải | 3 / 1 | Dễ hiểu là "thử tải về" |
| thoái lui | hồi quy (regression) | 4 / 2 | Ít dùng, nên giữ "regression" hoặc "hồi quy" |

## Nhận xét về kiểu lỗi

1. Lỗi thuật ngữ lặp lại nhiều nhất là calque: "nguồn sự thật", "output thật", "khử trùng", "Máy" làm chủ ngữ, và "ca" thay cho "trường hợp". Các cụm này lặp trên nhiều bài, nên sửa một lần bằng tìm và thay là hiệu quả.
2. Summary (dòng 4) là chỗ dày đặc lỗi nhất. Các câu viết kiểu gạch đầu dòng hoặc dùng dấu hai chấm liệt kê, nên dễ lược danh từ và trở nên cụt.
3. Thân bài có lỗi mang tính logic như "SKIP LOCKED cho một góc nhìn không nhất quán", "gần như tố «sạch»", "chuyến sạch ... còn chuyến gian lận thì hầu như không" (thiếu vị ngữ). Đây là loại cần sửa trước vì người đọc có thể hiểu sai kết luận.
4. Hai bài có văn phong tốt nhất trong mẫu là 01 va-uv và 02 bien-kieu, với khoảng 2 mục mỗi bài. Các bài nặng về thống kê và hàng đợi (23, 45, 59, 61) dày đặc nhất, khoảng 9 đến 14 mục mỗi bài.
5. Mật độ ước tính: khoảng 5 lỗi mỗi bài trong 8 bài đọc kỹ (43 mục thân bài / 8 bài, chưa tính summary). Các bài chưa đọc có thể khác; không nên suy rộng tuyệt đối.

## Giới hạn

- Chỉ đọc kỹ 8 trên 66 bài. Mật độ cho 58 bài còn lại là suy đoán, chưa đo.
- Không đánh giá phần code, khối ```, inline code, tên riêng hay thuật ngữ được giữ nguyên hợp lý.
- Không sửa file nào.

# Khảo sát chất lượng tiếng Việt: nhóm ba-*.md (Business Analysis)

Thư mục: H:\Work\tech-blog\api\seed\content
Chế độ: chỉ đọc, không sửa file nội dung nào.

## 1. Phạm vi và phương pháp

- Số file trong nhóm: 64 (tổng 4.997 dòng, khoảng 68.000 từ).
- Đã rà title và summary của cả 64 file (dòng 3–4 trong front matter).
- Đã đọc kỹ phần văn xuôi của 8 bài, bỏ khối code: ba-la-gi-tu-tinh-nang-den-bai-toan, ba-use-case-xac-nhan-diem-don, ba-decision-table-phi-huy, ba-idempotency-retry-giao-dich-loi, ba-sql-kiem-chung-gia-thuyet, ba-moscow-uu-tien-theo-muc-tieu, ba-thuc-hanh-thay-doi-chinh-sach-huy, ba-capstone-bao-ve-ho-so-phan-tich.
- Đếm cặp thuật ngữ: Grep/grep trên toàn bộ thư mục content (606 file .md) và riêng nhóm ba-*.md.
- Các bài còn lại (56 bài) chỉ rà qua title/summary, chưa đọc thân bài.

Mức độ: cao = sai nghĩa hoặc khó hiểu; trung bình = gượng, lược từ, thuật ngữ chưa Việt hóa gây vướng; thấp = có thể tốt hơn.

## 2. Mật độ ước tính

- Trong 8 bài đã đọc: 62 mục (cao 7, trung bình 29, thấp 26), tức khoảng 7,75 mục/bài. Riêng cao + trung bình: khoảng 4,5 mục/bài.
- Ngoại suy cho 64 bài: khoảng 250–500 mục, phần lớn là trung bình/thấp. Đây là ước lượng từ mẫu 8 bài, không phải đếm đầy đủ.
- Lỗi lặp lại nhiều nhất là kiểu câu rối/lược động từ hoặc danh từ (loại B), và thuật ngữ tiếng Anh giữ nguyên trong câu tiếng Việt không có giải thích (loại A). Lỗi kiểu "dịch từng chữ làm sai nghĩa" hiếm hơn, nhưng có vài chỗ rõ.

## 3. Lỗi chi tiết trong 8 bài đã đọc

### ba-la-gi-tu-tinh-nang-den-bai-toan.md

| Dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 4 (summary) | "Vận hành đề nghị thêm chat để khách và tài xế tìm nhau dễ hơn" | B | "Xem xét đề nghị thêm chat để khách và tài xế tìm nhau dễ hơn" | cao |
| 61 | "kiểm tra khả năng đo việc hiển thị/đọc ghi chú" | B | "kiểm tra xem có đo được việc khách có nhìn thấy và đọc ghi chú không" | trung bình |
| 89 | "Phải biết thông tin có đủ đúng và tài xế có thể xem an toàn" | B | "Cần biết thông tin đã đủ chính xác chưa và tài xế có thể xem an toàn không" | trung bình |
| 99 | "Đổi lời kể thành sự thật" | B | "Coi lời kể là sự thật" | trung bình |
| 54 | "lý do người dùng chọn có thể thiếu chính xác" | B | "lý do người dùng chọn có thể không chính xác" | thấp |
| 92 | "chỉ khiến bảng trông chắc chắn hơn thực tế" | B | "chỉ làm bảng trông chắc chắn hơn thực tế" | thấp |
| 98 | "Ra mắt chat đo việc giao sản phẩm" | B | "Ra mắt chat chỉ mô tả việc đưa sản phẩm ra" | thấp |

### ba-use-case-xac-nhan-diem-don.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 43 | "Chi tiết giao diện ở wireframe, cùng liên kết bước use case để tránh hai nơi mô tả khác nhau." | B | "Chi tiết giao diện nằm ở wireframe; giữ liên kết với bước use case tương ứng để tránh hai nơi mô tả khác nhau." | cao |
| 27 | "ở lúc mở đang trong phạm vi thao tác" | B | "lúc mở màn hình, lượt vẫn còn trong phạm vi thao tác" | trung bình |
| 32 | "Một yêu cầu trực tiếp hoặc phiên hết hạn vẫn cần xử lý theo hợp đồng." | B | "Ngay cả khi gọi API trực tiếp hoặc phiên đã hết hạn, yêu cầu vẫn phải được xử lý theo hợp đồng." | trung bình |
| 60 | "nếu transition nhận chuyến trước" | A | "nếu bước chuyển trạng thái nhận chuyến xảy ra trước" | thấp |
| 66 | "khi đổi scope" | A | "khi đổi phạm vi" | thấp |
| 52 | "kiểm lại bằng định danh" | B | "kiểm lại bằng mã định danh" | thấp |

### ba-decision-table-phi-huy.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 17 | "Các biến G/L chỉ được đánh giá khi dữ liệu cần thiết có nghĩa và đầy đủ." | B | "Các biến G/L chỉ được xét khi dữ liệu cần thiết tồn tại và đầy đủ." | trung bình |
| 37 | "Ngoài ba trạng thái được phép đi D0." | B | "Trạng thái nằm ngoài ba trạng thái trên đi theo D0." | trung bình |
| 78 | "để kiểm 2:59 và 3:00" | B | "để kiểm ca 2 phút 59 giây và 3 phút" (đơn vị đang dùng là phút:giây, dễ đọc nhầm thành giờ) | trung bình |
| 52 | "với tiền mặt, lab ghi nhận phí để thu sau" | A | Giải thích "lab" lần đầu, hoặc đổi thành "hệ thống thực hành" | thấp |
| 33 | "phải giữ hit policy này" | A | "phải giữ chính sách khớp dòng này" | thấp |
| 13 (callout) | "khách chủ chuyến" | B | Thống nhất với bài use-case: "khách chủ lượt" (xem mục 5) | thấp |

### ba-idempotency-retry-giao-dich-loi.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 11 | "liệu hai nhân viên có hoàn vượt tiền đã thu hay không" | B | "liệu hai nhân viên có thể hoàn quá số tiền đã thu hay không" | trung bình |
| 17 | "Khách hoặc nhân viên gửi hoàn 30.000 cho payment P" | B | "Khách hoặc nhân viên gửi yêu cầu hoàn 30.000 cho giao dịch P" | trung bình |
| 19 | "lưu qua reload thế nào" | B | "lưu được sau khi tải lại trang" | trung bình |
| 35 | "không đủ để suy mọi lần retry trả cùng body hoặc tự chữa mọi lỗi" | B/A | "không đủ để suy ra mọi lần thử lại đều trả cùng nội dung, hoặc tự khắc phục mọi lỗi" | trung bình |
| 41 | "Hết retry budget không chứng minh tiền chưa chuyển." | A/B | "Hết số lần thử lại cho phép không chứng minh tiền chưa được chuyển." | trung bình |
| 47 | "tổng đã hoàn thành cộng tiền đang giữ cho refund processing không vượt capturedAmount" | B | "tổng số tiền đã hoàn xong cộng số tiền đang giữ cho yêu cầu hoàn đang xử lý không vượt số tiền đã thu" | trung bình |
| 51 | "Chỉ giải phóng theo transition có bằng chứng và rule được duyệt." | B | "Chỉ giải phóng khi có bằng chứng về chuyển trạng thái và quy tắc đã được duyệt." | trung bình |
| 57 | "unknown không bị biến thành failed" | A | "kết quả chưa rõ không bị ghi thành thất bại" | trung bình |
| 39 | "owner ngoại lệ" | A | "người chịu trách nhiệm cho ngoại lệ" | thấp |

### ba-sql-kiem-chung-gia-thuyet.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 61 | "COUNT(s.id) bỏ null nhưng vẫn đếm ticket, không tự trở thành số chuyến có ticket." | B | "COUNT(s.id) bỏ qua giá trị null nhưng vẫn đếm từng ticket, nên không cho ra số chuyến có ticket." | cao |
| 11 | "Ba chuyến join với ba ticket tạo bốn dòng" | B | "Khi nối ba chuyến với ba ticket, kết quả có bốn dòng vì một chuyến có hai ticket." | trung bình |
| 21 | "Đây là oracle nhỏ để kiểm truy vấn." | A | "Đây là đáp án đối chiếu nhỏ để kiểm truy vấn." | trung bình |
| 51 | "kết quả null nghĩa không có tỷ lệ để báo, không phải 0% tốt đẹp" | B | "kết quả null nghĩa là không có tỷ lệ để báo cáo, không được hiểu là 0%" | trung bình |
| 59 | "Viết phiên bản pre-aggregate ticket rồi join" | A | "Viết phiên bản gộp ticket trước rồi mới join" | thấp |
| 49 | "Truy vấn nhỏ đã giả định cohort và outcome hoàn tất" | A/B | "...giả định nhóm đối tượng và kết quả đã đầy đủ" | thấp |

### ba-moscow-uu-tien-theo-muc-tieu.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 42 | "'CS sẽ xử lý thủ công'" | B | "CSKH" (đã dùng ở các bài khác; "CS" chưa được định nghĩa) | trung bình |
| 36 (bảng) | "Chưa có bằng chứng và vượt lát cắt" | B | "Chưa có bằng chứng và nằm ngoài lát cắt của đợt này" | thấp |
| 17 | "không phải ra mắt mọi kênh liên lạc" | B | "không nhằm ra mắt mọi kênh liên lạc" | thấp |
| 48 | "Không lặng lẽ đổi nhãn để bảng cân đối." | B | "Không lặng lẽ đổi nhãn để bảng trông cân đối." | thấp |

### ba-thuc-hanh-thay-doi-chinh-sach-huy.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 13 | "Bài thực hành khép phần 3 yêu cầu lập hồ sơ tác động cho CR-BA-01." | B | "Bài này khép lại phần 3 bằng việc lập hồ sơ tác động cho CR-BA-01." | cao |
| 54 | "phải nói rõ tài xế rời rồi người khác nhận có chọn version mới không." | B | "phải nói rõ: khi tài xế rời chuyến và người khác nhận chuyến, người nhận có dùng phiên bản mới không?" | cao |
| 21 | "Không lấy target giảm huỷ 9% của P01 làm bằng chứng chính sách này hiệu quả." | A/B | "Không dùng mục tiêu giảm huỷ 9% ở P01 làm bằng chứng rằng chính sách này hiệu quả." | trung bình |
| 34 | "Đơn vị giây ở đây để đọc case" | B | "Ở đây dùng đơn vị giây để tiện đọc ca" | trung bình |
| 48 | "Có nơi dùng diễn giải 'huỷ sớm'... không chỉ ký tự." | B | "Có chỗ diễn đạt khác, như 'huỷ sớm'... không chỉ khớp ký tự." | trung bình |
| 73 | "chỉ ghi số tiền chưa đủ xác minh toàn bộ hành vi" | B | "chỉ ghi số tiền thì chưa đủ để xác minh toàn bộ hành vi" | trung bình |
| 27 | "elapsed < 120 giây" | A | "thời gian đã trôi qua < 120 giây" | thấp |
| 52 | "gắn version tại một mốc đã xác định" | A | "gắn phiên bản chính sách tại một mốc đã xác định" | thấp |
| 77–79 | "process có điểm chọn version/replay, transition/effect được giữ" | A | "quy trình có điểm chọn phiên bản/replay, chuyển trạng thái và hiệu ứng được giữ" | thấp |
| 81 | "kiểm case thay đổi và regression" | A | "kiểm thử các ca thay đổi và hồi quy" | thấp |
| 87 | "coverage case biên và ngoại lệ" | A | "độ phủ các ca biên và ngoại lệ" | thấp |
| 11 | "tài xế đã chuyển arriving có được hưởng không" | B | "tài xế đã chuyển sang arriving có được hưởng không" | thấp |

### ba-capstone-bao-ve-ho-so-phan-tich.md

| Dòng | Câu gốc | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 62 | "Nêu giả định từng tin, bằng chứng khiến bạn đổi ý" | B | "Nêu giả định đứng sau từng kết luận và bằng chứng khiến bạn đổi ý" | cao |
| 56 | "Nếu biến số giả thành dữ liệu thật, tự ghi approval hoặc khẳng định nhân quả không có căn cứ" | B/A | "Nếu coi dữ liệu giả lập là dữ liệu thật, tự ghi nhận phê duyệt hoặc khẳng định nhân quả không có căn cứ" | cao |
| 42 | "7,5% so 13,16% có nghĩa thành công chưa?" / "Giải thích tự chọn và thiết kế đánh giá còn thiếu." | B | "7,5% so với 13,16%..." / "Giải thích về thiên lệch tự chọn và thiết kế đánh giá còn thiếu." | trung bình |
| 48 | "Giả sử stakeholder muốn cho sửa điểm sau accepted. Trong mười phút, chỉ ra yêu cầu, state/rule, quyền..." | B | "Giả sử stakeholder muốn cho phép sửa điểm đón sau khi chuyến đã accepted. Trong mười phút, chỉ ra những yêu cầu, rule, quyền... nào bị ảnh hưởng." | trung bình |
| 60 | "đọc xong series chưa thay việc thực hành" | B | "đọc xong chuỗi bài không thể thay thế việc thực hành" | trung bình |
| 66 | "so cách đặt câu hỏi của mình trước/sau" | B | "so sánh cách đặt câu hỏi của mình trước và sau khi học" | trung bình |
| 66 | "hoàn thành tài liệu là một mốc học tập, không phải bằng chứng mọi giả thuyết đã đúng" | B | "hoàn thành bộ tài liệu là một mốc học tập, không chứng minh mọi giả thuyết đã đúng" | trung bình |
| 19 | "Không trình bày số liệu giả lập như thành tích production." | A | "...như thành tích vận hành thật." | trung bình |
| 11 | "Người review cần lần được từ vấn đề tới bằng chứng" | B | "Người review cần lần theo được từ vấn đề đến bằng chứng" | thấp |
| 36 | "thiết kế ca" | A | "ca kiểm thử" | thấp |
| 44 | "Khả năng giữ ranh giới hiểu biết" | B | "Khả năng phân định rõ điều đã biết và chưa biết" | thấp |
| 50 | "telemetry tài xế đọc bị thiếu trên một app version" | A | "dữ liệu theo dõi việc tài xế đọc bị thiếu trên một phiên bản app" | thấp |
| 54 | "so phương án" | B | "so sánh phương án" | thấp |
| 17 / 32 | "P08 multi-stop"; "artifact" lặp nhiều lần | A | "giao nhiều điểm"; "sản phẩm/tài liệu" | thấp |

## 4. Lỗi ở title và summary (rà toàn bộ 64 bài)

| File:dòng | Nội dung | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| ba-la-gi-tu-tinh-nang-den-bai-toan.md:4 | "Vận hành đề nghị thêm chat" | B | "Xem xét đề nghị thêm chat" | cao |
| ba-data-dictionary-don-vi-null.md:4 | "tránh suy nghĩa từ tên trường" | B | "tránh suy ra ý nghĩa từ tên trường" | trung bình |
| ba-acceptance-criteria-given-when-then.md:4 | "revision cũ, trạng thái đổi" | B | "phiên bản cũ, trạng thái thay đổi" | trung bình |
| ba-giai-quyet-yeu-cau-xung-dot.md:4 | "Dùng đề nghị xem lịch sử trao đổi của CSKH" | B | "Dựa vào đề nghị xem lịch sử..." (làm rõ đề nghị là của ai, dùng để làm gì) | trung bình |
| ba-as-is-to-be-quy-trinh.md:4 | "tách đặc tả lab khỏi hành vi quan sát" | A | Làm rõ "lab" hoặc đổi cách gọi | trung bình |
| ba-phong-van-nghiep-vu-khong-dan-dat.md:4 | "ghi chép phân biệt phát ngôn, diễn giải..." | B | "ghi chép tách riêng phát ngôn, diễn giải..." | thấp |
| ba-khao-sat-lay-mau-thien-lech.md:4 | "giới hạn suy rộng" | A | "giới hạn khi khái quát kết quả" | thấp |
| ba-facilitation-ke-chuyen-bang-chung.md:3 | "kể câu chuyện bằng bằng chứng" | B | "kể câu chuyện từ bằng chứng" | thấp |
| ba-dashboard-do-ket-qua-sau-phat-hanh.md:4 | "phân biệt exposure, adoption và outcome" | A | Giữ tên nếu có chú thích; nếu không, "tiếp cận, mức sử dụng và kết quả" | thấp |
| ba-thuc-hanh-uat-dat-chua-co-gia-tri.md:4 | "Chẩn đoán adoption" | A | "Chẩn đoán mức sử dụng" | thấp |
| ba-user-story-vai-tro-nhu-cau-gia-tri.md:4 | "tách task kỹ thuật khỏi kết quả" | A | "tách công việc kỹ thuật khỏi kết quả" | thấp |
| ba-change-request-impact-analysis.md:4 | "Lập hồ sơ thay đổi có delta" | A | "có phần chênh lệch so với baseline" | thấp |
| ba-kiem-tra-nhat-quan-mo-hinh.md:4 | "Walkthrough cùng một case" | A | "Rà soát từng bước một ca" | thấp |
| ba-nghiem-thu-co-dieu-kien-go-no-go.md:4 | "tỷ lệ test pass" | A | "tỷ lệ test đạt" | thấp |
| ba-bpmn-task-event-gateway-lane.md:4 | "giữa participant" | A | "giữa các bên tham gia" | thấp |
| ba-domain-glossary-mo-hinh-du-lieu.md:4 | "Thống nhất booking, trip, payment và refund" | A | Giữ tên nếu bảng thuật ngữ có; cân nhắc thêm nghĩa tiếng Việt | thấp |
| ba-scope-assumption-constraint-dependency.md:4 | "lập sổ theo dõi có owner" | A | "có người chịu trách nhiệm" | thấp |
| ba-risk-assumption-log.md:4 | "ghi tín hiệu, owner, phản ứng" | A | "ghi tín hiệu, người phụ trách, phản ứng" | thấp |
| ba-brd-srs-tai-lieu-theo-rui-ro.md:4 | "một nguồn chuẩn cho rule, version và approval" | A | "...quy tắc, phiên bản và phê duyệt" | thấp |
| ba-thuc-hanh-tong-hop-discovery.md:4 | "bằng evidence matrix và discovery memo" | A | Giữ tên, thêm nghĩa lần đầu | thấp |
| ba-po-pm-ux-dev-tester-phoi-hop.md:4 | "bảng trách nhiệm và decision log" | A | "bảng phân công trách nhiệm và nhật ký quyết định" | thấp |
| ba-state-model-trang-thai-chuyen.md:4 | "state, event, actor, guard và effect" | A | Giữ tên, đây là ký hiệu chuẩn; chỉ cần chú thích lần đầu | thấp |
| ba-quan-sat-quy-trinh-doc-tai-lieu.md:4 | "Lập observation log" | A | "nhật ký quan sát" | thấp |

Nhận xét thêm: title và summary khá tốt. Lỗi chủ yếu là tên tiếng Anh chưa có chú thích và một vài chỗ gượng.

## 5. Cặp thuật ngữ đếm được

Đếm trên toàn bộ thư mục content (606 file .md) và riêng nhóm ba-*.md (64 file). Với "lab", "case", "version", "status" và các từ tiếng Anh khác, số toàn thư mục bị tính thêm các file không thuộc BA.

| Đang dùng | Đề xuất | Số lần (BA) | Số lần (toàn bộ) | Ghi chú |
|---|---|---|---|---|
| Vận hành đề nghị | Xem xét đề nghị | 3 | 3 | Lỗi A/B, mức cao, nên sửa hết |
| version | phiên bản | 48 | 933 | BA dùng lẫn "version" và "phiên bản" (46 lần). Nên chọn một cách và giữ nhất quán |
| transition | chuyển trạng thái | 23 | 111 | "transition" đứng sau động từ ở nhiều chỗ. Cụm tiếng Việt chỉ xuất hiện 5 lần trong BA |
| case | ca / trường hợp | 104 | 948 | Dùng nhiều, nhưng có chỗ dùng "ca" và chỗ "case" không nhất quán |
| lab | (giải thích lần đầu) | 50 | 5.816 | Nhiều bài dùng "lab" như tên của bộ bài thực hành, nhưng không giải thích trong nhóm BA |
| huỷ | hủy | 215 (huỷ) / 0 (hủy) | 1.534 / 365 | BA đã thống nhất "huỷ", nhưng site-wide đang lẫn hai cách viết |
| delta | phần thay đổi / chênh lệch | 9 | 34 | Tiếng Anh giữ nguyên, nên có chú thích |
| walkthrough | rà soát từng bước | 9 | 110 | |
| retry | thử lại | 32 | 571 | Có thể giữ, nhưng "hết retry budget" cần sửa |
| artifact | sản phẩm / tài liệu | 4 | 107 | |
| oracle | đáp án đối chiếu | 1 | 412 (phần lớn là Oracle DB) | Chỉ 1 chỗ trong BA |
| production | vận hành thật | 4 | 557 | |
| suy nghĩa | suy ra ý nghĩa | 1 | 1 | Lỗi A (chỉ có 1 chỗ) |
| tự chữa | tự khắc phục | 1 | 1 | |
| chủ chuyến / chủ lượt | Thống nhất một cách | 11 / 7 | 17 / 7 | Hai cách gọi cho cùng một vai trò |
| toạ độ / tọa độ | Thống nhất một cách | 2 / 0 | 72 / 95 | Lẫn hai cách viết chính tả |

Các cặp dùng nhiều nhưng không đề xuất sửa: retry (có thể giữ), status, state, timeout, callback (thuật ngữ kỹ thuật được chấp nhận rộng rãi).

## 6. Ví dụ tiêu biểu (gốc → đề xuất)

1. "Vận hành đề nghị thêm chat để khách và tài xế tìm nhau dễ hơn" → "Xem xét đề nghị thêm chat để khách và tài xế tìm nhau dễ hơn" (ba-la-gi-tu-tinh-nang-den-bai-toan.md:4).
2. "Chi tiết giao diện ở wireframe, cùng liên kết bước use case để tránh hai nơi mô tả khác nhau." → "Chi tiết giao diện nằm ở wireframe; giữ liên kết với bước use case tương ứng để tránh hai nơi mô tả khác nhau." (ba-use-case-xac-nhan-diem-don.md:43).
3. "COUNT(s.id) bỏ null nhưng vẫn đếm ticket, không tự trở thành số chuyến có ticket." → "COUNT(s.id) bỏ qua giá trị null nhưng vẫn đếm từng ticket, nên không cho ra số chuyến có ticket." (ba-sql-kiem-chung-gia-thuyet.md:61).

## 7. Nhận xét về kiểu lỗi phổ biến

- Lược động từ hoặc danh từ khiến câu khó đọc: "cùng liên kết...", "Bài thực hành khép phần 3 yêu cầu...", "Nêu giả định từng tin...", "so 13,16%".
- Thuật ngữ tiếng Anh giữ nguyên trong câu thuần Việt, không có chú thích lần đầu hoặc không có từ Việt tương đương: "transition", "case", "lab", "delta", "unknown", "owner", "approval", "walkthrough", "artifact".
- Trật tự từ kiểu tiếng Anh: "Ba chuyến join với ba ticket", "Viết phiên bản pre-aggregate ticket rồi join", "Hết retry budget không chứng minh...".
- Một số từ dùng sai nghĩa hoặc gượng: "Vận hành đề nghị", "suy nghĩa", "tự chữa", "lưu qua reload", "Đổi lời kể thành sự thật", "trạng thái đổi".
- Nhất quán thuật ngữ giữa các bài còn yếu: version/phiên bản, chủ chuyến/chủ lượt, huỷ/hủy, toạ/tọa.
- Ví dụ giả lập được đánh dấu rõ, nên không phải lỗi. Văn phong dễ đọc nói chung, lỗi chủ yếu nằm ở câu cụt và một số thuật ngữ.

## 8. Hạn chế

- 56 bài chưa đọc thân bài; số liệu mật độ và số lỗi toàn nhóm là ngoại suy từ mẫu 8 bài.
- Phân loại mức độ là đánh giá chủ quan của người rà soát; một số mục "trung bình" có thể do biên tập viên quyết định giữ.
- Không sửa file nào.

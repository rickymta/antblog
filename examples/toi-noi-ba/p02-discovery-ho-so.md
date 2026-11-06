# P02 · Hồ sơ discovery giả lập

Dùng cho BA-009–BA-016. Đây là dữ liệu sáng tác để luyện phân tích, không phải nghiên cứu thực tế. P02 giả định **một tuần discovery đã được chấp thuận trong bài tập**; D-01 của P01 vẫn là đề xuất trong hồ sơ gốc. Không có quyết định duyệt chat, pilot hoặc mục tiêu 9%.

## 1. Phạm vi và nguồn

Discovery tại khu vực A, tập trung ca đón khách đặt xe. Giữ nguyên baseline P01: 1.000 lượt được nhận, 120 lượt huỷ trước đón, 50 lượt huỷ có nhãn “không tìm thấy nhau”, 80 cuộc gọi CSKH. Không cộng mẫu định tính dưới đây vào baseline hoặc dùng để ước lượng tỷ lệ toàn khu vực.

Nhãn nguồn: INT = lời kể phỏng vấn; OBS = quan sát mô phỏng; DOC = tài liệu giả lập; DATA = thông tin về dữ liệu; WS = đề nghị trong workshop. ID người và case chỉ dùng trong bài tập. Case C-01, C-02, C-03 là các ca khác nhau.

| ID | Nội dung giả lập | Bối cảnh / giới hạn |
|---|---|---|
| INT-01 | Khách K-01: “Tôi ghi cổng Đông rồi nhưng anh ấy vẫn tới cổng Tây.” | Kể lại C-01 ở địa điểm nhiều cổng; chưa xem được màn hình của tài xế hoặc bản ghi lúc đó |
| INT-02 | Tài xế D-01: “Tôi thường nhìn ghim trước. Khi dừng xe mới mở phần chi tiết.” | Lời kể thói quen của một người; không phải tài xế của C-01; chưa đo tần suất |
| INT-03 | Nhân viên CSKH S-01: “Một ca khó có thể gọi lại vài lần; đôi lúc tôi tìm chuyến bằng số điện thoại.” | Mô tả cách làm; chưa kiểm tra quyền và quy trình chính thức |
| OBS-01 | C-02: khi xe đã dừng, D-02 mở chi tiết, kéo xuống rồi đọc được “Cổng số 2”. | Quan sát mô phỏng một ca có người nghiên cứu; chứng minh có thể thấy sau thao tác, không chứng minh dễ thấy ở mọi ca |
| OBS-02 | C-03: K-02 tìm tên cổng trên biển và nói không biết cổng này được gọi là gì trong app. | Quan sát tại địa điểm nhiều lối vào; chưa khẳng định nguyên nhân của các lượt huỷ trong baseline |
| DOC-01 | Hướng dẫn hỗ trợ v0.8 yêu cầu ghi ID lượt đặt xe vào ticket liên quan điểm đón. | Tài liệu giả lập; chưa có owner xác nhận còn hiệu lực hoặc mức tuân thủ |
| DATA-01 | Mẫu kiểm tra chất lượng gồm 20 ticket điểm đón, 8 ticket thiếu ID lượt. | Mẫu thuận tiện do trưởng ca chọn, độc lập với tập 80 cuộc gọi; không suy ra 40% của tất cả ticket hoặc cuộc gọi thiếu ID |
| DOC-02 | Bản thiết kế giả lập cho thấy vùng ghi chú trên màn hình chi tiết. | Chưa kiểm phiên bản đã triển khai; thiết kế không chứng minh thông tin đã tới người dùng |
| WS-01 | Vận hành muốn xem toàn bộ lịch sử trao đổi để giải quyết khiếu nại. | Đề nghị giải pháp, chưa duyệt phạm vi dữ liệu/quyền truy cập |
| WS-02 | PO đề nghị xem kết quả discovery vào cuối tuần trước khi chọn phương án. | Mốc review giả lập; không phải cam kết phát hành cuối tuần |

## 2. Câu hỏi nghiên cứu khởi tạo

- RQ-01: Hai bên dùng thông tin nào để xác định cùng điểm đón?
- RQ-02: Thông tin có ích được tạo, truyền và nhận biết ở bước nào?
- RQ-03: Các nhóm địa điểm/người dùng có khó khăn khác nhau ra sao?
- RQ-04: Dữ liệu hỗ trợ có đủ liên kết để kiểm tra giả thuyết và đo lường không?
- RQ-05: CSKH cần tối thiểu thông tin nào để xử lý từng loại ca?

## 3. Mẫu evidence matrix

| Claim ID | Nhận định hẹp | Nguồn | Loại và phạm vi bằng chứng | Nguồn phản biện / giới hạn | Mức hỗ trợ, không phải xác suất | Bước xác minh / owner đề xuất |
|---|---|---|---|---|---|---|
| CL-01 | | | | | | |

Phân biệt: **được hỗ trợ trong phạm vi nguồn**, **còn tranh luận**, **chưa đủ hỗ trợ**, **bị bác bỏ trong phạm vi đã kiểm**. Nhiều nguồn lặp lại cùng một lời kể không phải nhiều xác nhận độc lập.

## 4. Mẫu xác nhận kết quả

| ID | Nội dung cần xác nhận | Người xác nhận phù hợp | Phản hồi / hạn đã thoả thuận | Trạng thái | Ảnh hưởng nếu chưa xác nhận |
|---|---|---|---|---|---|
| CF-01 | | | | Chờ xác nhận | |

Xác nhận người tham gia được ghi đúng ý không phải phê duyệt yêu cầu hoặc xác nhận suy luận nhân quả.

## 5. Mẫu xử lý xung đột

| ID | Vị thế / nhu cầu từng bên | Mục tiêu chung | Ràng buộc đã xác nhận | Các phương án | Người có quyền chốt | Quyết định, điều kiện, ý kiến còn khác |
|---|---|---|---|---|---|---|
| CFLC-01 | | | | | | Chưa chốt |

## 6. Sản phẩm cuối phần

Nộp stakeholder map, interview guide, observation/document log, biên bản workshop, đánh giá khảo sát, journey/blueprint, conflict log và discovery memo 1–2 trang. Có thể gộp các bảng trong một tệp; giữ ID nguồn để truy vết. Không cần phần mềm khảo sát hoặc dữ liệu cá nhân thật.

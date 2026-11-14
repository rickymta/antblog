---
slug: ba-use-case-xac-nhan-diem-don
title: "Use case: actor, precondition, main flow và alternative flow"
summary: "Đặc tả use case xác nhận thông tin điểm đón với ranh giới hệ thống, luồng chính, ngoại lệ và postcondition rõ; tránh dùng precondition để bỏ qua tình huống thay đổi đồng thời."
category: business-analysis
level: beginner
tags: [business-analysis, ba-027, use-case, requirements, toi-noi]
language: vi
---

Story nói khách muốn xác nhận thông tin. AC nói revision khác thì cần xem lại. Nhưng giữa lúc mở màn hình và bấm xác nhận có các tương tác nào? Use case nối chúng thành một cuộc trao đổi có mục tiêu, để người review lần theo cả đường thuận lẫn đường thay thế.

<div class="tkh-callout"><span class="tkh-callout-title">BA-027 · Phần 4/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/ba-acceptance-criteria-given-when-then">Acceptance criteria</a>. Đầu ra: UC-P04-01. Đây là use case đề xuất, không khẳng định chức năng điểm đón đã có trong lab.</p></div>

## 1. Chốt actor và ranh giới

Actor chính là khách chủ lượt. Hệ thống trong use case gồm giao diện và xử lý chuẩn của Tới Nơi; tài xế là stakeholder liên quan nhưng không trực tiếp thực hiện hành động xác nhận trong use case này.

Mục tiêu: khách biết nội dung nào được ghi nhận là đã xác nhận. Không có mục tiêu “tài xế đã đọc” hoặc “khách đã lên xe”. Nếu gộp các mục tiêu đó, postcondition sẽ vô tình đổi vòng đời chuyến.

## 2. Header của UC-P04-01

| Trường | Nội dung đề xuất |
|---|---|
| Trigger | Khách chọn xem/xác nhận thông tin điểm đón |
| Actor chính | Khách đã xác thực, có quyền với lượt |
| Precondition | Lượt tồn tại và khách có quyền truy cập; lúc mở màn hình, lượt vẫn còn trong phạm vi thao tác |
| Thành công | Xác nhận gắn với revision chuẩn đã xem |
| Bảo đảm tối thiểu | Không ghi xác nhận sai revision/quyền; không tự chuyển state chuyến |
| Ngoài scope | Sửa sau nhận chuyến, chat, ảnh, xác nhận của tài xế |

Precondition là điều dùng để mô tả bối cảnh vào luồng; không có nghĩa hệ thống được bỏ kiểm tra quyền khi thực thi. Một yêu cầu gọi trực tiếp hoặc phiên đã hết hạn vẫn cần xử lý theo hợp đồng.

## 3. Main flow đủ chi tiết để theo dõi dữ liệu

1. Khách yêu cầu xem thông tin điểm đón của lượt.
2. Hệ thống trả thông tin chuẩn, revision và khả năng thao tác hiện tại.
3. Khách đọc nội dung, chọn xác nhận bản đang xem.
4. Hệ thống kiểm tra quyền, state và revision còn nhất quán tại thời điểm quyết định ghi.
5. Hệ thống ghi xác nhận cho revision đó, trả kết quả.
6. Giao diện hiển thị nội dung/revision đã xác nhận theo phản hồi chuẩn.

Không chèn “click nút xanh ở góc phải” vào logic nếu đó chưa là quyết định cần ràng buộc. Chi tiết giao diện nằm ở wireframe; giữ liên kết với bước use case tương ứng để tránh hai nơi mô tả khác nhau.

## 4. Alternative flow phải có điểm rẽ và điểm kết

| Nhánh | Rẽ tại | Xử lý đề xuất | Kết thúc/quay lại |
|---|---|---|---|
| A1: revision đổi | Bước 4 | Không ghi bản cũ, tải nội dung mới | Về bước 2 để khách xem lại |
| A2: state không còn searching | Bước 4 | Không ghi, hiển thị trạng thái hiện tại | Kết thúc không thành công cho thao tác ghi |
| A3: sai quyền/phiên không hợp lệ | Bước đọc hoặc ghi | Không trả dữ liệu/không ghi ngoài quyền | Theo luồng xác thực/truy cập đã chốt |
| A4: không biết kết quả do timeout | Bước 5–6 | Ghi nhận trạng thái chưa biết, kiểm tra lại bằng mã định danh | Thành công hoặc lỗi xác định sau đối soát |

Không dùng nhánh “lỗi khác: làm lại” cho mọi trường hợp. Mỗi loại lỗi có thể đòi quyết định khác; việc lặp không giới hạn có thể làm khách chờ mãi và gây tải.

## 5. State thay đổi giữa chừng không phá use case

Lượt đang searching khi mở có thể accepted trước bước 4. Vì vậy không viết precondition “luôn searching” rồi bỏ nhánh A2. Use case cần mô tả dữ liệu được kiểm lại lúc ghi.

Nếu ghi thắng trước transition nhận chuyến, xác nhận có thể hợp lệ; nếu transition thắng trước, từ chối ghi. Đây là kết quả nghiệp vụ đề xuất, nhóm kỹ thuật sẽ chọn cơ chế bảo đảm; không dựa vào thứ tự người dùng nhìn thấy trên hai màn hình.

## 6. Nối use case với các sản phẩm khác

US-P04-01 giải thích mục tiêu; RULE-P04-01/02 ràng buộc quyền/state/revision; AC-P04-01–05 đưa ví dụ kiểm thử; wireframe minh hoạ trạng thái hiển thị. Use case không sao chép toàn bộ rule hoặc NFR vào từng bước, nhưng cần dẫn ID đúng phiên bản.

Một bảng truy vết ngắn giúp tìm ảnh hưởng khi đổi scope: cho phép ghi sau accepted sẽ tác động bước 4, nhánh A2, rule quyền/state, AC và nội dung cho tài xế. Không chỉ sửa một dòng trong use case.

## 7. Bài tập và câu hỏi ôn tập

Viết postcondition cho A1 và A4. Với A1 phải rõ chưa xác nhận bản mới; với A4 chưa biết kết quả không đồng nghĩa chưa lưu. Nêu câu hỏi còn mở về retry, nội dung rỗng và dữ liệu phiên bản.

**Câu hỏi:** Use case nhiều bước hơn có luôn tốt hơn không?

<details><summary>Gợi ý trả lời</summary><p>Không. Chi tiết cần đủ để giải thích tương tác và quyết định. Lặp mọi thao tác giao diện hoặc chi tiết lưu trữ khiến luồng khó đọc mà không nhất thiết giảm rủi ro hiểu sai.</p></details>

## 8. Tham khảo và bài tiếp theo

- Hồ sơ thiết kế và ID: `examples/toi-noi-ba/p04-goi-yeu-cau.md`.
- [State model Tới Nơi](/articles/ba-state-model-trang-thai-chuyen): phân biệt state nghiệp vụ với trạng thái của thao tác xác nhận.

Bài tiếp theo: [Story mapping và chia lát cắt có giá trị](/articles/ba-story-mapping-lat-cat-gia-tri).

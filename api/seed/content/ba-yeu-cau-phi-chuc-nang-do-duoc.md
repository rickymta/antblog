---
slug: ba-yeu-cau-phi-chuc-nang-do-duoc
title: "Yêu cầu phi chức năng: từ “nhanh, an toàn” đến tiêu chí đo"
summary: "Đặc tả chất lượng với điều kiện, thước đo và cách kiểm: phân biệt p95, tỷ lệ lỗi, tải, trải nghiệm đầu cuối và quyền truy cập trong gói yêu cầu điểm đón."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-030, non-functional, performance, toi-noi]
language: vi
---

“Màn hình phải nhanh và an toàn” thể hiện mong muốn nhưng chưa cho phép nhóm nghiệm thu. Nhanh đo từ thao tác nào tới kết quả nào, trên thiết bị và mạng nào? An toàn chống truy cập sai bằng cách nào, với actor nào?

<div class="tkh-callout"><span class="tkh-callout-title">BA-030 · Phần 4/8 · Trung bình</span><p>Đọc trước: <a href="/articles/ba-wireframe-prototype-kiem-tra-hieu-biet">Wireframe/prototype</a>. Đầu ra: quality scenario và kế hoạch đo. Các ngưỡng là giả định bài tập, không thay SLO hiện có của Tới Nơi.</p></div>

## 1. Viết chất lượng trong một bối cảnh

NFR-P04-01 đề xuất: thời gian từ thao tác mở tới nội dung điểm đón dùng được có p95 không quá hai giây trên lần mở thành công; tỷ lệ lỗi không quá 1% trên tất cả lần mở trong phép đo. Mỗi lần mở tối đa năm giây rồi phân loại lỗi nếu chưa có nội dung dùng được.

Định nghĩa điểm cuối rất quan trọng. API trả xong nhưng UI chưa hiển thị không phải cùng phép đo với trải nghiệm đầu cuối. “Dùng được” cần diễn đạt dữ liệu cần thấy và khả năng thao tác.

## 2. Gắn workload và môi trường

| Thành phần | Đề xuất / phần cần chốt |
|---|---|
| Workload | 100 phiên đồng thời, mỗi phiên mở một lượt mỗi 10 giây |
| Thời gian đo | 15 phút, cần quy định warm-up và cửa sổ tính |
| Thiết bị/mạng | Cần chốt cấu hình và điều kiện tham chiếu |
| Dữ liệu/cache | Cần chốt kích thước, phân bố và trạng thái cache |
| Tín hiệu | Bắt đầu thao tác, nội dung dùng được, timeout/lỗi |
| Owner | Nghiệp vụ xác nhận mục tiêu; kỹ thuật/test xác nhận phép đo khả thi |

100 phiên không có nghĩa 100 request mỗi giây. Với nhịp trên, tốc độ khởi tạo trung bình lý tưởng là khoảng 10 lần mở mỗi giây, nhưng burst và request phụ thuộc cần được mô tả. Không dùng một con số concurrency thay mọi đặc điểm tải.

## 3. Hiểu p95 và lỗi cùng nhau

p95 là một phân vị của phân bố thời gian theo cách tính thống nhất, không phải trung bình. Đo nhiều lần nhanh không bù cho một nhóm đáng kể trải nghiệm rất chậm nếu mục tiêu là phân vị.

Nếu chỉ lấy lần thành công, p95 có thể đẹp trong khi lỗi nhiều. Vì vậy báo cả số lần khởi tạo, thành công, lỗi và phân bố latency; nêu cách tính percentile và xử lý mẫu thiếu. Không loại timeout khỏi mẫu số tỷ lệ lỗi.

## 4. Chuyển “an toàn” thành các hành vi kiểm được

P04 đề xuất khách chỉ đọc/ghi đúng quyền, tài xế chỉ đọc lượt được gán và không thể thay ID để lấy dữ liệu người khác. Đây là hành vi chức năng phục vụ bảo mật; phân loại NFR không được làm mất rule cụ thể.

Các câu hỏi chất lượng khác gồm log có lộ nội dung không, dữ liệu giữ bao lâu, ai được tra cứu. Chưa có chính sách thì ghi cần quyết định, không điền một thời hạn tuỳ ý rồi tuyên bố đã tuân thủ.

## 5. Không gom mọi chất lượng vào một tiêu chí

Khả năng truy cập, khả năng sử dụng, khả năng phục hồi và hiệu năng có cách kiểm khác nhau. Chọn tình huống liên quan phạm vi: người dùng nhận biết trạng thái mà không chỉ dựa màu; timeout có đường kiểm lại; nội dung cũ không được hiển thị như dữ liệu vừa xác nhận.

Nếu cần áp dụng một chuẩn bên ngoài, phải chỉ rõ phiên bản và phạm vi kiểm khi đặc tả. Bài này không gán chứng nhận chất lượng cho một wireframe hoặc vài scenario.

## 6. Review đánh đổi trước khi chốt ngưỡng

Ngưỡng càng chặt có thể cần thay đổi kiến trúc, chi phí hoặc scope. BA giải thích vì sao người dùng cần mức đó, nhóm kỹ thuật đưa bằng chứng/ước lượng, người có quyền chốt đánh đổi. Không dùng “ngành ai cũng thế” làm lý do cho hai giây.

## 7. Bài tập và câu hỏi ôn tập

Viết lại “lưu ổn định” thành một scenario mất phản hồi, kết quả người dùng thấy, giới hạn retry và cách đối soát cần chốt. Liệt kê dữ kiện còn thiếu để nghiệm thu NFR-P04-01.

**Câu hỏi:** p95 đạt trong máy của developer có đủ nghiệm thu không?

<details><summary>Gợi ý trả lời</summary><p>Chưa nếu điều kiện đó không khớp hợp đồng đo. Cần cùng workload, thiết bị/mạng, dữ liệu, cửa sổ đo và định nghĩa kết quả đã thống nhất.</p></details>

## 8. Tham khảo và bài tiếp theo

- Hồ sơ và ngưỡng giả định: `examples/toi-noi-ba/p04-goi-yeu-cau.md`.
- [Metric card và mẫu số](/articles/ba-problem-statement-muc-tieu-chi-so).

Bài tiếp theo: [BRD, SRS hay tài liệu gọn](/articles/ba-brd-srs-tai-lieu-theo-rui-ro).

---
slug: ba-uu-tien-gia-tri-chi-phi-do-tin-cay
title: "Ưu tiên theo giá trị, chi phí và độ tin cậy của bằng chứng"
summary: "Dùng bảng điểm minh bạch và phân tích độ nhạy để so sánh phương án, tránh biến ước lượng thành sự thật hoặc điểm số thành quyết định tự động."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-046, prioritization, confidence, toi-noi]
language: vi
---

Hai ý tưởng đều được mô tả là “tác động lớn”. Một ý tưởng dựa trên quan sát hành vi, ý tưởng kia dựa trên một lời đề nghị. Nếu bỏ độ tin cậy, bảng ưu tiên có thể làm hai loại bằng chứng trông ngang nhau.

<div class="tkh-callout"><span class="tkh-callout-title">BA-046 · Phần 6/8 · Trung bình</span><p>Đầu ra: bảng so sánh và kiểm tra độ nhạy. Mọi con số trong bài là giả định riêng để học kỹ thuật, không phải ước lượng hoặc kết quả thực của Tới Nơi.</p></div>

## 1. So các phương án cùng mục tiêu

Ví dụ A là cải thiện hướng dẫn điểm đón, B là xây chat; cùng được xem xét cho mục tiêu giảm khó khăn khi gặp nhau. Không đem chính sách miễn phí huỷ vào cùng bảng nếu chưa thống nhất outcome và cách đo lợi ích của nó.

Phương án giữ nguyên kèm thu thập bằng chứng cũng phải xuất hiện. Phương án này có chi phí cơ hội, nhưng có thể hợp lý khi điều chưa biết quá lớn.

## 2. Công thức học tập

Dùng điểm minh hoạ S = reach × impact × confidence / effort. Reach là số chuyến đủ điều kiện trong một tuần giả định; impact là hệ số do nhóm định nghĩa; confidence từ 0 đến 1; effort là tuần công ước lượng cùng phạm vi. Đây là công cụ để gợi mở thảo luận, không phải phép tính lợi nhuận.

Thang impact phải có diễn giải dùng chung. Nếu người này chấm 3 vì thích UI còn người kia chấm 3 vì giảm cuộc gọi, công thức không sửa được sự thiếu nhất quán.

## 3. Tính thử và đọc đúng

| Phương án | Reach | Impact | Confidence | Effort | S |
|---|---:|---:|---:|---:|---:|
| A: hướng dẫn | 1.000 | 1 | 0,6 | 2 | 300 |
| B: chat | 1.000 | 2 | 0,3 | 5 | 120 |

A cao hơn theo giả định này. Không được viết “A mang lại 300 đơn vị giá trị thực”. S chỉ giúp làm lộ cơ sở và điểm chưa chắc; effort cũng chưa phải cam kết của đội thực hiện.

## 4. Độ nhạy có đổi thứ tự không?

Nếu confidence của A chỉ là 0,2, điểm A còn 100 và B lên trước. Điểm hoà của A nằm ở confidence 0,24 khi các yếu tố khác giữ nguyên. Như vậy quyết định nhạy với bằng chứng về A; một hoạt động kiểm chứng nhỏ có thể hữu ích hơn tranh luận thêm về số thập phân.

Tương tự, nếu B phải thêm công cụ vận hành và effort tăng, thứ tự càng khác. Đừng bỏ chi phí hỗ trợ để làm phương án ưa thích trông rẻ hơn.

## 5. Điều kiện bắt buộc đứng ngoài bảng điểm

Quyền truy cập, tính đúng của tiền và giới hạn vận hành không được bù bằng reach lớn. Loại phương án không đáp ứng điều kiện bắt buộc hoặc ghi việc cần làm để đáp ứng trước khi so. Đây là bước xác định tính khả thi, không phải một trọng số nhỏ trong tổng điểm.

## 6. Ghi khuyến nghị có điều kiện

“Ưu tiên kiểm chứng A vì điểm tốt hơn dưới giả định hiện có; nếu confidence dưới 0,24 hoặc effort vượt ngưỡng mới, xem xét lại.” Khuyến nghị này cho biết tín hiệu nào khiến nhóm đổi quyết định, thay vì bảo vệ thứ hạng bất kể dữ liệu mới.

## 7. Thực hành

Giữ confidence A = 0,6, tính effort của A để hoà với B (S = 120). Sau đó viết một câu hỏi giúp xác minh chi phí còn thiếu.

<details><summary>Đáp án</summary><p>Effort hoà là 5 tuần công: 1.000 × 1 × 0,6 / 5 = 120. Hỏi xem ước lượng đã gồm nội dung hướng dẫn, kiểm tra accessibility, đo lường và đào tạo CSKH chưa; các phần cần thiết phải nằm trong phạm vi so sánh.</p></details>

## 8. Đọc tiếp

Bảng và công thức ở đây là ví dụ thực hành của series, không áp dụng một chuẩn chứng chỉ nào. Tiếp: [phụ thuộc xuyên đội](/articles/ba-phu-thuoc-yeu-cau-xuyen-doi).

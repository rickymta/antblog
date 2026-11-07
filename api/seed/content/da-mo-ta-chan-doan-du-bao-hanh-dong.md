---
slug: da-mo-ta-chan-doan-du-bao-hanh-dong
title: "Phân tích mô tả, chẩn đoán, dự báo và đề xuất hành động"
summary: "Cùng một bảng huỷ chuyến có thể trả lời điều gì? Chọn loại phân tích theo quyết định, phân biệt dự báo có điều kiện với tác động của can thiệp và viết khuyến nghị đúng mức bằng chứng."
category: data-analysis
level: intermediate
tags: [data-analysis, da-006, descriptive, predictive, prescriptive, toi-noi]
language: vi
---

Từ bảng cao điểm W1/W2, một người viết “tỷ lệ huỷ tăng”, người khác viết “do thiếu tài xế”, người thứ ba dự đoán “tuần sau sẽ có 216 lượt huỷ”, rồi đề xuất tăng thưởng. Bốn câu này không cùng loại và không thể dùng cùng một phép chia để bảo vệ.

DA cần biết mình đang trả lời “đã xảy ra gì”, “vì sao”, “có thể xảy ra gì” hay “nên làm gì”. Việc đặt đúng tên giúp xác định bằng chứng còn thiếu.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-006 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/da-kpi-metric-chi-so-bao-ve">DA-005: định nghĩa chỉ số</a>. Đầu ra: bảng chọn phương pháp và một khuyến nghị có điều kiện. Không yêu cầu học máy; mục tiêu là biết câu hỏi nào cần dữ liệu và cách kiểm chứng nào.</p>
</div>

## 1. Bốn loại phân tích phục vụ bốn câu hỏi

[Microsoft Learn](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/2-data-analysis) phân biệt các loại phân tích theo câu hỏi chúng trả lời. Ta dùng bốn nhóm sau cho ca Tới Nơi:

| Loại | Câu hỏi | Ví dụ đầu ra | Bằng chứng tối thiểu cần xem xét |
|---|---|---|---|
| Mô tả — descriptive | Điều gì đã xảy ra? | Số/tỷ lệ huỷ theo cohort, khung giờ | Dữ liệu đúng phạm vi, công thức và đối soát |
| Chẩn đoán — diagnostic | Những yếu tố nào có thể giải thích? | Các giả thuyết, phân nhóm và bằng chứng đối chiếu | Nguồn bổ sung, thứ tự thời gian, yếu tố gây nhiễu |
| Dự báo — predictive | Điều gì có thể xảy ra trong điều kiện đã nêu? | Dự báo số lượt và khoảng bất định | Lịch sử phù hợp, baseline và đánh giá ngoài dữ liệu dùng xây mô hình |
| Đề xuất — prescriptive | Nên chọn hành động nào theo mục tiêu/ràng buộc? | So sánh lựa chọn, chi phí và điều kiện áp dụng | Dự kiến tác động, ràng buộc, rủi ro và bằng chứng kiểm chứng |

Các nhóm không phải bậc nghề nghiệp. Một phân tích mô tả chính xác, kịp thời có thể hữu ích hơn mô hình dự báo phức tạp cho câu hỏi chỉ cần đối soát số hôm qua.

## 2. Mô tả: điều ta làm được với P01

Baseline giả lập có W1 120/1.000 và W2 180/1.500, cùng 12%. Cao điểm từ 16% lên 18%, còn lại từ 8% xuống 6%. Đây là những phép mô tả kiểm tra được trong phạm vi bảng đã cấp.

Một biểu đồ cột nhóm có thể thể hiện diễn biến ngược chiều. Bảng đi kèm phải giữ số lượt để người đọc biết quy mô. Nhận định “cao điểm cần xác minh thêm” là hợp lý như đề xuất điều tra; câu “thiếu tài xế đã làm huỷ tăng” đã vượt dữ liệu.

Mô tả không cần dùng mọi cột có sẵn. Nếu câu hỏi là theo khung giờ, hãy bắt đầu bằng phân nhóm có ý nghĩa đó. Việc thử hàng trăm phân nhóm rồi chỉ báo cáo nhóm biến động lớn cần được ghi là khám phá, không phải kiểm chứng giả thuyết đặt trước.

## 3. Chẩn đoán: tìm lời giải thích cạnh tranh

Cùng một kết quả có thể phù hợp nhiều giả thuyết: thời tiết xấu; thiếu tài xế; thay đổi loại khách; điểm đón khó tiếp cận; thay đổi ghi nhận huỷ; pipeline W1 bỏ sót sự kiện. Cần thiết kế phép đối chiếu giúp phân biệt chúng.

Ví dụ, trước giả thuyết thiếu tài xế, hãy hỏi có số tài xế sẵn sàng phục vụ đúng khu vực/thời điểm không. Tổng số tài xế online toàn thành phố có thể gồm người đang bận hoặc ở xa; nó chưa đo đúng nguồn cung liên quan.

Nếu thấy lúc nguồn cung thấp thì huỷ cao, vẫn phải xét yếu tố như mưa có thể đồng thời làm giảm cung và tăng huỷ. Phân tích chẩn đoán có thể thu hẹp giả thuyết và phát hiện sai cách đo, nhưng tên “chẩn đoán” không tự bảo đảm đã tìm ra nguyên nhân nhân quả.

## 4. Dự báo: phân biệt phép tính kịch bản với mô hình đã đánh giá

Giả sử chỉ để luyện lập kế hoạch, W3 dự kiến có 1.200 lượt được nhận cao điểm. Nếu tỷ lệ giữ ở 18%, số huỷ kỳ vọng theo kịch bản là 1.200 × 18% = **216**. Điều kiện “giữ ở 18%” quyết định toàn bộ kết quả; hai tuần P01 chưa đủ để xác nhận điều kiện ấy.

Ta có thể trình bày độ nhạy:

| Tỷ lệ giả định | Lượt nhận giả định | Số huỷ theo kịch bản |
|---|---:|---:|
| 16% | 1.200 | 192 |
| 18% | 1.200 | 216 |
| 20% | 1.200 | 240 |

Đây là **ba giả định để tính kịch bản**, không phải khoảng tin cậy hay khoảng dự báo có mức xác suất. Lượng lượt nhận 1.200 cũng là giả định, có thể sai cùng lúc với tỷ lệ.

Để xây dự báo thực tế, cần đủ lịch sử có tính đại diện, so sánh với baseline đơn giản và đánh giá trên các kỳ tương lai so với tập dùng xây mô hình. Không trộn ngẫu nhiên các bản ghi theo thời gian nếu làm vậy khiến dữ liệu tương lai lọt vào quá trình dự báo quá khứ. Các phần sau mới triển khai kỹ thuật này.

## 5. Dự báo rủi ro không cho biết tác động can thiệp

Một mô hình có thể dự đoán nhóm khách nào hay huỷ. Điều đó chưa trả lời nhóm nào sẽ đổi hành vi nếu được thêm chat hoặc giảm giá. Người có rủi ro cao có thể huỷ vì lý do chat không giải quyết được.

Tương tự, nếu mô hình dùng nhãn lý do huỷ đã ghi sau kết quả để dự đoán trước chuyến, nó đang sử dụng thông tin không có ở thời điểm quyết định. Hiệu quả đánh giá có thể trông rất tốt nhưng không sử dụng được. Luôn ghi **thời điểm dự đoán** và những trường thực sự có tại thời điểm đó.

Muốn chọn can thiệp, nhóm cần bằng chứng về tác động của chính can thiệp cùng chi phí và giới hạn triển khai, thay vì chỉ dự báo kết quả nếu mọi thứ tiếp tục như cũ.

## 6. Đề xuất hành động khi chưa biết chắc nguyên nhân

Khuyến nghị vẫn có thể hữu ích khi chưa đủ bằng chứng cho rollout. Với P01, ta so sánh các bước tiếp chứ không giả vờ đã tính được ROI của chat.

| Lựa chọn | Có thể học được gì? | Chi phí/rủi ro cần xác minh | Điều kiện trước khi làm |
|---|---|---|---|
| Kiểm tra tracking và dữ liệu trễ | Biến động có do cách đo không | Thời gian DE/backend | Có nguồn và lịch thay đổi |
| Review một tập ca cao điểm và ca đối chiếu | Khó khăn thực tế và chất lượng nhãn | Công sức, thiên lệch mẫu | Kế hoạch lấy mẫu và quyền truy cập |
| Quan sát điểm đón | Bối cảnh không có trong log | Độ phủ thời điểm/địa điểm | Phạm vi quan sát được thống nhất |
| Pilot phương án được chọn sau discovery | Bằng chứng tác động | Chi phí, ảnh hưởng người dùng, đo lường | Giả thuyết và thiết kế đánh giá phù hợp |

Một khuyến nghị hiện tại có thể là “ưu tiên kiểm tra cách đo, rồi xác minh ca cao điểm”. Không nên chấm điểm chính xác hay gán số tiền tiết kiệm khi chưa có ước lượng được cung cấp. Có thể ghi tiêu chí định tính và thông tin cần thu thập để so sánh tiếp.

## 7. Chọn phương pháp theo quyết định, không theo công cụ

Nếu cuộc họp cần biết dữ liệu có đủ tin để điều tra không, bảng đối soát và quality log là đầu ra phù hợp. Nếu cần bố trí nguồn lực tháng tới, dự báo và phân tích độ nhạy mới trở nên cần thiết. Nếu muốn chọn thay đổi sản phẩm, phải thiết kế đánh giá hiệu quả và chỉ số bảo vệ.

SQL, bảng tính, Python hay dashboard chỉ hỗ trợ các hoạt động đó. Chọn một công cụ học máy không biến câu hỏi mô tả thành dự báo hợp lệ; đặt một nút “recommendation” không biến tương quan thành bằng chứng về hành động.

## 8. Bài tập và lời giải

Phân loại bốn câu, chỉ ra điều kiện còn thiếu:

1. “W2 có 180 lượt huỷ trước đón trong 1.500 lượt được nhận.”
2. “Thiếu tài xế gây ra toàn bộ phần tăng ở cao điểm.”
3. “Nếu W3 có 1.200 lượt và tỷ lệ 18%, số huỷ theo kịch bản là 216.”
4. “Nên kiểm tra tracking trước khi chi ngân sách pilot.”

<details><summary>Gợi ý trả lời</summary><p>Câu 1 là mô tả trong phạm vi baseline. Câu 2 là kết luận nhân quả chưa được dữ liệu hiện tại hỗ trợ; cần kiểm tra giả thuyết và yếu tố cạnh tranh. Câu 3 là tính kịch bản có điều kiện, chưa phải dự báo đã được đánh giá. Câu 4 là đề xuất bước kiểm chứng; cần owner, phạm vi và chi phí được xác nhận, không được ghi là quyết định đã duyệt.</p></details>

**Bài viết:** Soạn khuyến nghị tối đa 120 từ, nêu điều biết, điều chưa biết và bước tiếp. Không dùng “chắc chắn”, “nguyên nhân chính” hay ước lượng hiệu quả khi không có bằng chứng.

<details><summary>Tiêu chí tự chấm</summary><p>Khuyến nghị phải nối với câu hỏi vận hành, giữ số và tỷ lệ đúng, tách giả thuyết khỏi kết luận, chọn bước có thể thực hiện và nêu điều kiện cần xác nhận. Không gọi ba kịch bản 16/18/20% là khoảng tin cậy.</p></details>

## 9. Tham khảo và bước tiếp

- [Microsoft Learn — Types of data analytics](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/2-data-analysis).
- [DA-005: hợp đồng chỉ số](/articles/da-kpi-metric-chi-so-bao-ve).

Lưu lựa chọn phương pháp vào `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`. Bài tiếp: [DA-007: nguồn dữ liệu Tới Nơi](/articles/da-nguon-du-lieu-chuyen-su-kien-thanh-toan-cskh).

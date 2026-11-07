---
slug: da-chua-du-du-lieu-bat-dinh-dieu-kien-dung
title: "Khi nào chưa đủ dữ liệu để kết luận?"
summary: "Phân biệt thiếu dữ liệu, biến động ngẫu nhiên và thiếu bằng chứng nhân quả. Thực hành khoảng kịch bản, assumption log và điều kiện dừng để đưa ra khuyến nghị hữu ích mà không tạo cảm giác chắc chắn giả."
category: data-analysis
level: advanced
tags: [data-analysis, da-009, uncertainty, evidence, sensitivity, toi-noi]
language: vi
---

“Có 2.500 lượt trong hai tuần rồi, sao vẫn bảo chưa đủ dữ liệu?” Số dòng nhiều không tự cho biết ta được phép kết luận điều gì. Bộ P01 đủ để kiểm tra các tỷ lệ mô tả trong đề bài, nhưng không có thông tin can thiệp để kết luận chat sẽ giảm huỷ.

DA cần nói chính xác **thiếu gì cho câu hỏi nào**, tác động của thiếu hụt tới quyết định và bước nào có thể giảm bất định đó. Câu “cần thêm dữ liệu” đứng một mình chưa giúp nhóm hành động.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-009 · Phần 1/8 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/da-yeu-to-anh-huong-ban-do-gia-thuyet">DA-008: bản đồ giả thuyết</a>. Đầu ra: assumption log, bảng độ nhạy và điều kiện tiếp tục/dừng. Bài không đặt ngưỡng cỡ mẫu chung cho mọi bài toán và không suy luận ý nghĩa thống kê từ bảng tổng hợp P01.</p>
</div>

## 1. “Đủ” phải gắn với phát biểu muốn đưa ra

| Phát biểu | P01 hỗ trợ đến đâu? | Còn thiếu |
|---|---|---|
| Bảng W2 có 180/1.500 = 12% | Tính đúng theo dữ kiện giả lập | Không phải kết quả kiểm toán hệ thống thật |
| Cao điểm W2 cao hơn W1 2 điểm phần trăm trong bảng | Mô tả được | Chưa chứng minh tính ổn định ở kỳ khác |
| Khách toàn Tới Nơi có trải nghiệm xấu hơn | Chưa đủ | Độ phủ dịch vụ/vùng và các mặt trải nghiệm |
| Thiếu tài xế gây ra mức tăng | Chưa đủ | Phép đo cung, yếu tố gây nhiễu, thiết kế so sánh |
| Chat sẽ giảm huỷ | Chưa đủ | Dữ liệu/thiết kế đánh giá tác động can thiệp |

Một phân tích có thể hoàn thành phần mô tả và vẫn để phần nhân quả mở. Cần nói rõ phạm vi đó trong kết luận và tiêu chí đóng công việc.

## 2. Bốn loại bất định không nên gộp chung

**Bất định về định nghĩa:** “huỷ” có cùng nghĩa giữa hai nhóm không; cohort có cùng biên kỳ không? Cách giảm là xác nhận nghiệp vụ, không phải thu thêm triệu dòng với định nghĩa mơ hồ.

**Bất định về đo lường và độ phủ:** thiếu sự kiện, trễ pipeline, nhóm khách không được ghi nhận. Cách giảm là đối soát và bổ sung nguồn; có trường hợp cần thu thập lại.

**Biến động ngẫu nhiên và khả năng suy rộng:** tỷ lệ của một giai đoạn có thể khác giai đoạn khác. Nếu quan sát đủ mọi lượt của W1/W2 thì tỷ lệ mô tả chính hai cohort ấy không có sai số lấy mẫu, nhưng kết luận về hành vi ổn định hoặc tương lai vẫn cần mô hình và giả định phù hợp.

**Bất định nhân quả/mô hình:** dữ liệu quan sát có thể phù hợp nhiều cơ chế. Dự báo chính xác cũng không tự cho biết tác động của hành động. Cần thiết kế đánh giá đáp ứng câu hỏi.

Phân biệt các loại này giúp tránh dùng khoảng tin cậy để che một định nghĩa sai hoặc dùng cỡ mẫu lớn để thay cho nhóm so sánh phù hợp.

## 3. Cỡ mẫu không có một ngưỡng chung

Muốn biết cần bao nhiêu đơn vị quan sát, phải biết đang ước lượng gì, cần độ chính xác nào, mức thay đổi nào có ý nghĩa thực tế và cách các quan sát liên quan với nhau. Một tài xế có nhiều chuyến không tạo ra từng ấy tài xế độc lập; một khách có nhiều lần đặt cũng vậy.

Nếu quan sát chỉ lấy từ giờ dễ thu thập hoặc người chủ động phản hồi, mẫu lớn vẫn có thể lệch. Nếu phân tích hàng trăm nhóm rồi chọn kết quả nổi bật nhất, cần tính đến cách tìm kiếm đó khi xác nhận phát hiện.

[Tài liệu về khoảng tin cậy của Penn State](https://online.stat.psu.edu/stat100/Lesson09) giới thiệu mối liên hệ giữa cỡ mẫu, độ chính xác và mức tin cậy. Khi áp dụng, phải xem giả định lấy mẫu và phương pháp phù hợp. P01 chưa cung cấp dữ liệu cấp khách/tài xế hoặc thiết kế mẫu để tự gán một khoảng tin cậy cho tác động.

Không thấy khác biệt “có ý nghĩa thống kê” cũng chưa chứng minh hai phương án tương đương. Có thể phép đo chưa đủ nhạy với mức thay đổi đáng quan tâm; câu hỏi tương đương cần tiêu chí và thiết kế riêng.

## 4. Khoảng kịch bản khi còn kết quả chưa biết

**Biến thể MISS-01 độc lập:** W2 vẫn có 1.500 lượt thuộc cohort, trong đó cao điểm 750. Đã xác nhận 135 huỷ cao điểm và 45 huỷ ở nhóm còn lại, nhưng còn 30 lượt cao điểm chưa rõ kết quả. Biến thể này thay giả định “đã đủ kết quả” của baseline chỉ cho bài tập này; 30 lượt đã nằm trong mẫu số, không thêm vào tổng.

Nếu không có thêm huỷ trong 30 lượt, tổng huỷ là 180; nếu cả 30 đều huỷ trước đón, tổng là 210. Vì vậy:

```text
Toàn W2: từ 180/1.500 = 12% tới 210/1.500 = 14%
Cao điểm: từ 135/750 = 18% tới 165/750 = 22%
```

Đây là **giới hạn theo những kết quả chưa biết trong giả định bài tập**, không phải khoảng tin cậy 95%. Nó chưa tính các lỗi khác như lượt đã gắn nhãn sai hoặc mẫu số thiếu. Nếu những giả định về 1.500 lượt và 180 huỷ xác nhận không đáng tin, giới hạn cũng không còn bảo đảm.

Khoảng này giúp nói rõ: kết luận “W2 đúng bằng W1 ở 12%” chưa được chốt; tỷ lệ cuối của W2 có thể cao hơn. Nó vẫn chưa giải thích nguyên nhân hay dự báo kết quả can thiệp.

## 5. Kiểm tra độ nhạy của quyết định

Giả sử **riêng bài tập MISS-01**, vận hành đang thảo luận ngưỡng rà soát khi tỷ lệ cuối **lớn hơn 13%**. Đây chỉ là quy tắc luyện tính, không phải ngưỡng vận hành đã duyệt.

Với 1.500 lượt, 13% tương ứng 195 huỷ. Nếu 15 trong 30 lượt trở thành huỷ, tổng là 195 và chỉ bằng 13%; cần ít nhất **16** lượt để vượt ngưỡng: 196/1.500 ≈ **13,07%**.

Do phần chưa biết có thể nằm ở hai phía ngưỡng, quyết định theo quy tắc này nhạy với dữ liệu thiếu. Báo cáo phải nêu điều kiện đó, đề nghị xác minh hoặc dùng một hành động tạm thời phù hợp mức rủi ro do người có thẩm quyền lựa chọn. Không tự thay “lớn hơn” bằng “lớn hơn hoặc bằng”.

Một hành động điều tra nhỏ có thể được cân nhắc khi chưa chốt hết số, nhưng quyết định đầu tư diện rộng cần bằng chứng khác. Mức chắc chắn cần thiết phụ thuộc hậu quả và khả năng đảo ngược của hành động, không chỉ một ngưỡng số học.

## 6. Assumption log: ghi giả định cùng cách kiểm tra

| ID | Giả định | Nếu sai thì ảnh hưởng gì? | Cách kiểm tra / người phối hợp | Trạng thái |
|---|---|---|---|---|
| AS-01 | Hai kỳ dùng cùng định nghĩa nhận/huỷ | So sánh không cùng đại lượng | Chủ metric và backend review | Chưa xác nhận trên nguồn thật |
| AS-02 | Mỗi lượt có đúng một khoá đếm | Số nhận/huỷ có thể bị nhân | DA kiểm tra grain và khoá | Chỉ được giả định trong bảng P01 |
| AS-03 | Kết quả cohort đã đủ lúc chốt | Tỷ lệ cuối có thể tăng/sửa | DE và vận hành đối soát | Có trong đề gốc, bị thay ở MISS-01 |
| AS-04 | Dữ liệu A phản ánh câu hỏi muốn quyết định | Suy rộng sai vùng/dịch vụ | Người dùng báo cáo xác nhận phạm vi | Không dùng cho toàn hệ thống |

Mỗi giả định cần trạng thái riêng. “Tạm chấp nhận để tính ví dụ” khác “đã xác minh bằng nguồn”. Đừng đổi trạng thái chỉ vì một giả định xuất hiện ở nhiều bài.

## 7. Đặt điều kiện dừng, tiếp tục và chuyển câu hỏi

**Dừng phát hành kết luận cuối** nếu chưa rõ mẫu số, nguồn có lệch không giải thích được hoặc dữ liệu thiếu có thể đổi quyết định. Vẫn có thể bàn giao báo cáo giới hạn dữ liệu và kế hoạch xử lý, với nhãn phù hợp.

**Tiếp tục mô tả có điều kiện** nếu phép tính có thể kiểm tra và phạm vi được ghi rõ, nhưng chưa đủ để suy rộng. Ví dụ P01 gốc cho phép trình bày các tỷ lệ của bảng; chưa cho phép gọi đó là hiệu quả một can thiệp.

**Chuyển sang thiết kế đánh giá khác** nếu câu hỏi thực sự là nhân quả. Thu thêm nhiều tuần có thể giúp hiểu mùa vụ nhưng không tự giải quyết mọi yếu tố gây nhiễu. Các lựa chọn như thử nghiệm hoặc thiết kế quan sát cần giả định và năng lực tương ứng.

Mỗi điều kiện phải có người sở hữu và mốc review. Không để “đang chờ dữ liệu” vô thời hạn mà không biết ai làm gì tiếp theo.

## 8. Viết câu chưa biết sao cho hữu ích

Thay “chưa đủ dữ liệu để kết luận” bằng:

> Trong MISS-01, còn 30/750 lượt cao điểm chưa rõ kết quả. Theo các giả định đã nêu, tỷ lệ toàn W2 nằm từ 12% tới 14%; phần chưa biết có thể làm kết quả vượt ngưỡng rà soát giả lập 13%. Cần chủ nguồn xác minh các lượt này trước khi chốt nhận định theo ngưỡng. Hiện chưa có bằng chứng về tác động của chat.

Đoạn này nêu thiếu gì, phạm vi tác động và việc tiếp theo. Nó không giả vờ biết xác suất mỗi kịch bản hoặc hứa chắc lấy thêm dữ liệu sẽ cho một kết luận thuận ý.

## 9. Bài tập và câu hỏi ôn tập

Trong MISS-01, nếu xác minh thêm 10 lượt là huỷ, 10 lượt không huỷ và 10 lượt vẫn chưa biết, tính giới hạn mới của tỷ lệ toàn W2. Quy tắc “lớn hơn 13%” đã quyết định được chưa?

<details><summary>Gợi ý lời giải</summary><p>Huỷ đã biết là 190; tối đa 200. Giới hạn là 190/1.500 ≈ 12,67% tới 200/1.500 ≈ 13,33%. Vẫn đi qua ngưỡng; cần ít nhất 6 trong 10 lượt còn lại là huỷ để tổng vượt 195. Đây là khoảng kịch bản, không phải khoảng tin cậy.</p></details>

**Câu hỏi:** Nếu thu được thêm một triệu lượt nhưng không biết chúng thuộc vùng nào, có giải quyết câu hỏi riêng cho A không?

<details><summary>Gợi ý trả lời</summary><p>Không tự động. Thiếu mapping phạm vi là vấn đề định nghĩa/độ phủ. Cần xác định được các lượt phù hợp hoặc đổi phạm vi với người dùng kết quả; số lượng lớn không thay thế điều kiện này.</p></details>

Checklist: phát biểu muốn kết luận; loại bất định; giả định chưa kiểm; độ nhạy; hành động phù hợp; điều kiện dừng; owner và mốc kiểm tra lại.

## 10. Tham khảo và bước tiếp

- [Penn State — Confidence Intervals](https://online.stat.psu.edu/stat100/Lesson09): nền tảng về độ chính xác và mức tin cậy.
- [DA-008: yếu tố gây nhiễu và cơ cấu nhóm](/articles/da-yeu-to-anh-huong-ban-do-gia-thuyet).

Mẫu assumption log và các biến thể: `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`. Bài tiếp: [DA-010: thực hành kế hoạch phân tích huỷ chuyến](/articles/da-thuc-hanh-ke-hoach-phan-tich-huy-chuyen).

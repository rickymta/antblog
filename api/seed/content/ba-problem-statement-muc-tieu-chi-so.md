---
slug: ba-problem-statement-muc-tieu-chi-so
title: "Problem statement, mục tiêu và chỉ số thành công"
summary: "Viết problem statement có bằng chứng và metric có tử số, mẫu số, cohort, thời gian đo. Phân biệt giảm 3 điểm phần trăm với giảm 25%, đặt guardrail và tránh kết luận quá sớm từ pilot điểm đón."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-005, metrics, problem-statement, baseline, toi-noi]
language: vi
---

“Sau khi có giao diện mới, tỷ lệ huỷ chỉ còn 9%.” Nghe có vẻ đạt mục tiêu. Nhưng nếu nhóm đo mới chỉ tính các lượt đã hoàn thành xử lý, còn nhóm cũ tính tất cả lượt được nhận, hai con số chưa thể so sánh. Nếu tuần sau ít mưa hơn, kết quả cũng chưa đủ để khẳng định giao diện tạo ra toàn bộ chênh lệch.

BA cần làm rõ câu hỏi đo lường trước khi dashboard được dựng. Một chỉ số có tên đẹp nhưng định nghĩa thiếu có thể dẫn nhóm tới quyết định sai.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-005 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/ba-baccm-sau-khai-niem-cot-loi">BACCM</a>. Đầu ra: problem statement và metric card. Sau bài này, bạn có thể viết mục tiêu có phạm vi, kiểm tra mẫu số và nêu giới hạn của kết luận từ dữ liệu.</p>
</div>

## 1. Problem statement mô tả vấn đề trước khi chọn cách giải

Với dữ kiện giả lập P01, một bản nháp phù hợp là:

> Trong tuần quan sát tại khu vực A, 120 trong 1.000 lượt đặt xe đã được tài xế nhận bị huỷ trước đón. Trong đó, 50 lượt được gắn nhãn “không tìm thấy nhau”. Hiện tượng cho thấy cần tìm hiểu khó khăn khi xác định điểm đón và tác động với khách, tài xế, CSKH. Nguyên nhân và tính đại diện của tuần quan sát chưa được xác minh.

Đoạn này có nhóm sự kiện, phạm vi, bằng chứng và điều chưa biết. Nó không khẳng định “vì thiếu chat” hoặc biến 80 cuộc gọi CSKH thành 80 chuyến khác nhau.

Phân tích hiện trạng giúp thiết lập bối cảnh và baseline cho thay đổi, cũng là điểm được nhấn mạnh trong [task card Analyze Current State của IIBA](https://www.iiba.org/contentassets/51018f337f654c39a1c2bb87bdfd5210/task-card-analyze-current-state.pdf). Những công thức và quy ước đo bên dưới là thiết kế thực hành riêng cho Tới Nơi.

## 2. Từ vấn đề đến mục tiêu và thước đo

| Thành phần | Câu hỏi | Bản nháp P01 |
|---|---|---|
| Kết quả mong muốn | Điều gì tốt hơn đối với nghiệp vụ? | Ít lượt đón thất bại trước khi khách lên xe |
| Chỉ số chính | Theo dõi kết quả bằng gì? | Tỷ lệ huỷ trước đón trên lượt đã được nhận |
| Baseline | Hiện trạng được đo ra sao? | 120/1.000 = 12% trong một tuần giả lập tại A |
| Target | Mức mong muốn là bao nhiêu? | Không quá 9% trong pilot bốn tuần, đang đề xuất |
| Guardrail | Điều gì không được xấu đi? | Khiếu nại trên 1.000 lượt được nhận không tăng |
| Điều kiện ra quyết định | Cần gì trước khi đánh giá? | Baseline đại diện, metric thống nhất, dữ liệu đủ và người duyệt mục tiêu |

9% là mục tiêu học tập, chưa được chứng minh khả thi. Guardrail chưa có baseline nên chưa thể tuyên bố pilot đạt toàn bộ mục tiêu chỉ vì chỉ số chính đạt 9%.

## 3. Metric card: định nghĩa đủ để hai người tính ra cùng kết quả

| Trường | Đề xuất định nghĩa M-01 |
|---|---|
| Tên | Tỷ lệ lượt huỷ trước đón |
| Đơn vị đếm | Một lượt đặt xe duy nhất theo ID; nhiều lần đổi tài xế không tự tạo lượt mới |
| Cohort | Lượt thuộc khu vực A theo quy tắc khu vực đã chốt, có lần đầu được tài xế nhận trong cửa sổ đo |
| Tử số | Số ID trong cohort kết thúc bằng huỷ trước sự kiện đón khách |
| Mẫu số | Toàn bộ ID hợp lệ trong cohort, kể cả lượt đang chờ kết quả |
| Công thức | Tử số / mẫu số × 100%; mẫu số bằng 0 thì báo không có dữ liệu |
| Thời gian | Baseline theo tuần; pilot bốn tuần; dùng cùng múi giờ và mốc biên đầu bao gồm, cuối không bao gồm |
| Dữ liệu cần có | ID, khu vực, thời điểm lần đầu được nhận, sự kiện đón, kết quả cuối; cần mapping sang nguồn thật |
| Chất lượng | Kiểm tra ID trùng, thiếu sự kiện, thời gian sai thứ tự và độ trễ nhập dữ liệu |
| Người xác nhận | BA phối hợp người phụ trách dữ liệu và vận hành; PO/đại diện nghiệp vụ duyệt cách dùng |

Đây là **đề xuất hợp đồng đo**, không khẳng định database hiện có các trường với đúng tên trên. Cần đối chiếu schema và sự kiện thực tế, không thêm trạng thái mới vào hệ thống chỉ để tiện viết bài.

Các lượt chưa có kết quả vẫn ở mẫu số nhưng báo cáo phải gắn nhãn **tạm thời**, kèm số lượng chưa xác định. Không dùng tỷ lệ tạm thời để nghiệm thu. Nhóm cần chốt thời gian chờ kết quả dựa vào nghiệp vụ và độ trễ dữ liệu; ca quá hạn phải được đối soát hoặc công bố là chưa đủ dữ liệu. Không tự loại khỏi mẫu số để làm đẹp chỉ số.

## 4. Ba tỷ lệ cùng dữ liệu, ba câu hỏi khác nhau

Từ 1.000 lượt được nhận, 120 lượt huỷ trước đón và 50 lượt huỷ có nhãn không tìm thấy nhau:

- **120/1.000 = 12%:** bao nhiêu phần trăm lượt được nhận bị huỷ trước đón?
- **50/1.000 = 5%:** bao nhiêu phần trăm lượt được nhận có kết quả huỷ với nhãn này?
- **50/120 ≈ 41,7%:** trong các lượt huỷ trước đón, nhãn này chiếm bao nhiêu?

Giả sử một bộ kết quả minh hoạ độc lập có 90 lượt huỷ trên 1.000 lượt cùng định nghĩa, tỷ lệ là 9%. Chênh lệch tuyệt đối so với 12% là **3 điểm phần trăm**; mức giảm tương đối là **(12 − 9)/12 = 25%**. Đây là phép tính minh hoạ, không phải dữ liệu pilot đã diễn ra.

Với nhiều tuần, tính tỷ lệ chung bằng **tổng tử số / tổng mẫu số**. Không lấy trung bình đơn giản tỷ lệ từng tuần khi mẫu số khác nhau. Ví dụ tuần X có 10/100 = 10%, tuần Y có 90/1.000 = 9%; tỷ lệ gộp là 100/1.100 ≈ 9,09%, không phải 9,5%.

## 5. Guardrail và chỉ số chẩn đoán có vai trò khác nhau

Guardrail giúp nhận biết giải pháp có gây đánh đổi ngoài ý muốn. Nếu làm nút huỷ khó tìm khiến số lượt huỷ giảm nhưng khiếu nại tăng, nhóm chưa đạt mục tiêu trải nghiệm mong muốn.

Để đo khiếu nại, cần định nghĩa một case khiếu nại hợp lệ, cách gộp case trùng, cách nối case với lượt và thời gian tiếp nhận sau lượt đó. **80 cuộc gọi CSKH không phải baseline khiếu nại.** Một cuộc gọi hỏi đường chưa chắc là khiếu nại; một khiếu nại có thể sinh nhiều cuộc gọi.

Các chỉ số như tỷ lệ có ghi chú, tỷ lệ ghi chú được hiển thị hoặc số người hoàn thành thao tác prototype là chỉ số chẩn đoán. Chúng giúp tìm mắt xích bị đứt nhưng không thay thế kết quả nghiệp vụ. “Nhiều người dùng tính năng” chưa có nghĩa “ít ca đón thất bại”.

## 6. Đạt ngưỡng chưa đủ để kết luận nguyên nhân

Trước khi khuyến nghị mở rộng, BA cần cùng nhóm xem:

1. Baseline và pilot có cùng định nghĩa, chất lượng dữ liệu và cửa sổ ghi nhận kết quả không?
2. Cơ cấu địa điểm, thời điểm, nhóm người dùng hoặc điều kiện bên ngoài có khác đáng kể không?
3. Có thay đổi đồng thời như chương trình khuyến mãi hay chính sách huỷ không?
4. Mức biến động và lượng dữ liệu có đủ hỗ trợ quyết định không?

Bốn tuần không tự bảo đảm đủ mẫu hoặc quan hệ nhân quả. Khi cần ước lượng tác động, phải thiết kế so sánh phù hợp, thống nhất cỡ mẫu và cách phân tích trước khi chạy; phần P07 sẽ đi sâu vào pilot và thử nghiệm. Ở đây, kết luận đúng có thể chỉ là “đã quan sát tỷ lệ thấp hơn, chưa tách được tác động của giải pháp”.

## 7. Bài tập và tự kiểm tra

Một báo cáo giả lập có 2.000 lượt được nhận, 150 lượt huỷ trước đón và 300 lượt chưa có kết quả. Người viết báo cáo lấy 150/1.700 ≈ 8,82% rồi kết luận đạt mục tiêu 9%. Hãy chỉ ra sai khác với M-01 và viết lại kết luận.

<details><summary>Gợi ý lời giải</summary><p>M-01 không loại lượt chưa có kết quả khỏi mẫu số. Tỷ lệ hiện ghi nhận là 150/2.000 = 7,5%, nhưng còn 300 lượt chưa xác định nên chưa dùng nghiệm thu. Trong giới hạn dữ kiện bài tập, tỷ lệ cuối có thể nằm từ 7,5% đến 22,5% nếu lần lượt không có hoặc cả 300 lượt đều huỷ trước đón. Cần đối soát kết quả và kiểm tra guardrail trước khi kết luận.</p></details>

Viết thêm metric card cho guardrail khiếu nại. Bài làm đạt yêu cầu khi phân biệt cuộc gọi với case, có quy tắc gộp trùng, cohort, thời gian chờ, nguồn cần kiểm tra và owner xác nhận.

**Câu hỏi ôn tập:** Tại sao không lấy 41,7% làm baseline cho mục tiêu giảm huỷ trước đón từ 12% xuống 9%?

<details><summary>Gợi ý trả lời</summary><p>41,7% dùng mẫu số là lượt đã huỷ, còn mục tiêu dùng tất cả lượt đã được nhận. Hai chỉ số đo hai hiện tượng khác nhau.</p></details>

## 8. Tham khảo và bài tiếp theo

- [IIBA — Analyze Current State](https://www.iiba.org/contentassets/51018f337f654c39a1c2bb87bdfd5210/task-card-analyze-current-state.pdf): vai trò của phân tích hiện trạng và baseline.
- [BA là gì?](/articles/ba-la-gi-tu-tinh-nang-den-bai-toan): nguồn số liệu giả lập ban đầu.

Bài tiếp theo: [Scope, assumption, constraint và dependency](/articles/ba-scope-assumption-constraint-dependency).

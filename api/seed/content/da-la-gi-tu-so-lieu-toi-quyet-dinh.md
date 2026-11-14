---
slug: da-la-gi-tu-so-lieu-toi-quyet-dinh
title: "Phân tích dữ liệu là gì? Từ số liệu chuyến xe tới quyết định"
summary: "Số lượt huỷ của Tới Nơi tăng từ 120 lên 180 có nghĩa dịch vụ đang tệ đi? Bắt đầu học DA bằng cách đọc đúng mẫu số, phân biệt dữ kiện với giả thuyết và biến một bảng số liệu thành bước kiểm chứng tiếp theo."
category: data-analysis
level: beginner
tags: [data-analysis, da-001, metric, decision-making, toi-noi]
language: vi
---

“Tuần này có 180 lượt huỷ, tuần trước chỉ có 120. Chắc điểm đón đang có vấn đề. Làm dashboard rồi đề xuất cách giảm huỷ giúp anh nhé.”

Nếu chỉ vẽ hai cột 120 và 180, ta đã có một biểu đồ đúng số. Nhưng để quyết định có nên thay đổi điểm đón, nhóm Tới Nơi còn cần biết: có bao nhiêu lượt được nhận ở mỗi tuần, những lượt nào được tính là huỷ và dữ liệu đã đầy đủ chưa? Công việc phân tích bắt đầu ở những câu hỏi đó.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-001 · Phần 1/8 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>. Không cần biết SQL. Đầu ra: bản nhận định một trang. Sau bài này, bạn phân biệt được dữ liệu, thông tin, insight và hành động; tính đúng tỷ lệ và viết kết luận có giới hạn.</p>
</div>

## 1. Phân tích dữ liệu là gì?

Trong series này, **Data Analysis** là quá trình làm rõ câu hỏi, kiểm tra và xử lý dữ liệu, tìm những đặc điểm có ý nghĩa rồi diễn giải chúng để hỗ trợ quyết định. **Data Analyst** là người làm công việc đó; developer, BA hay người vận hành cũng có thể thực hiện một phần hoạt động phân tích.

Một phép tính đúng chưa bảo đảm trả lời đúng câu hỏi. Tính tổng số lượt huỷ có thể giúp dự tính khối lượng cần xử lý. Tính tỷ lệ huỷ trên số lượt được nhận giúp mô tả kết quả đón khách trong một cohort — nhóm lượt cùng thoả điều kiện và nằm trong khoảng thời gian đã định. Hai cách đo có thể dẫn tới hai nhận xét khác nhau mà không mâu thuẫn.

Theo [Microsoft Learn về vai trò DA](https://learn.microsoft.com/en-us/training/career-paths/data-analyst/), công việc này bao gồm phối hợp với các bên liên quan, khảo sát và làm sạch dữ liệu, biến đổi, mô hình hoá và trình bày kết quả. Vì vậy, dashboard chỉ là một dạng đầu ra. Một phân tích có ích cũng có thể kết thúc bằng thông báo “chưa đủ bằng chứng, cần sửa cách ghi nhận trước”.

## 2. Cùng đọc một bộ số liệu của Tới Nơi

**Toàn bộ số dưới đây là giả lập riêng cho DA-P01, không lấy từ baseline BA và không phải dữ liệu production.** Chỉ xét dịch vụ đặt xe tại khu vực A. W1 và W2 là hai tuần liên tiếp; lượt được xếp vào tuần theo thời điểm tài xế nhận lần đầu, dùng múi giờ Asia/Ho_Chi_Minh. Mỗi lượt chỉ xuất hiện một lần trong bảng tổng hợp.

Trong bài tập, đã chờ đủ kết quả của các lượt ở cả hai tuần và xác nhận không thiếu hoặc trùng bản ghi. Đây là điều kiện bài tập cho sẵn để luyện đọc số; khi làm thật, DA phải kiểm tra điều kiện ấy.

| Kỳ | Lượt được nhận | Huỷ trước đón | Tỷ lệ huỷ trước đón |
|---|---:|---:|---:|
| W1 | 1.000 | 120 | 12% |
| W2 | 1.500 | 180 | 12% |

Công thức đang dùng là:

```text
Tỷ lệ huỷ trước đón của cohort
= số lượt trong cohort bị huỷ trước đón / tổng lượt trong cohort

W1 = 120 / 1.000 = 12%
W2 = 180 / 1.500 = 12%
```

Số lượt huỷ tăng **60 lượt**, tương đương **50%** so với W1. Tổng lượt được nhận cũng tăng 50%. Tỷ lệ huỷ quan sát được vì thế không đổi: chênh lệch **0 điểm phần trăm**.

Không thể lấy 180 chia 1.000 vì tử số và mẫu số thuộc hai cohort khác nhau. Cũng không thể gọi đây là tỷ lệ huỷ trên mọi yêu cầu đặt xe: những yêu cầu chưa được tài xế nhận nằm ngoài mẫu số của bài.

### Tỷ lệ chung chưa kể hết câu chuyện

Giả sử cùng bộ dữ liệu đã đối soát được chia thành giờ cao điểm và các giờ còn lại. Hai nhóm không giao nhau và gộp lại đủ toàn bộ cohort; lịch phân loại giống nhau ở hai tuần.

| Nhóm | W1: được nhận | W1: huỷ | W1: tỷ lệ | W2: được nhận | W2: huỷ | W2: tỷ lệ |
|---|---:|---:|---:|---:|---:|---:|
| Cao điểm | 500 | 80 | 16% | 750 | 135 | 18% |
| Còn lại | 500 | 40 | 8% | 750 | 45 | 6% |
| Tổng | 1.000 | 120 | 12% | 1.500 | 180 | 12% |

Trong khi tỷ lệ chung đứng yên, tỷ lệ quan sát ở cao điểm tăng **2 điểm phần trăm**, còn nhóm giờ khác giảm 2 điểm phần trăm. Đây là phát hiện đáng điều tra. Phát hiện này chưa chứng minh cao điểm gây huỷ, chưa xác định nguyên nhân và chưa chứng minh chênh lệch sẽ lặp lại ở tuần sau.

Khi đọc bảng, hãy cộng lại các nhóm trước: 135 + 45 = 180 và 750 + 750 = 1.500. Bước đơn giản này giúp phát hiện lỗi chia nhóm hoặc lấy nhầm tập dữ liệu trước khi kể một câu chuyện hấp dẫn.

## 3. Từ dữ liệu tới hành động: bốn lớp khác nhau

| Lớp | Ví dụ trong bài | Điều cần giữ rõ |
|---|---|---|
| Dữ liệu | Các số đếm theo tuần và khung giờ | Nguồn, phạm vi, đơn vị và độ đầy đủ |
| Thông tin | Tỷ lệ chung cùng là 12%; cao điểm từ 16% lên 18% | Phép tính phải khớp định nghĩa |
| Insight phục vụ quyết định | Chỉ nhìn tỷ lệ chung sẽ bỏ qua diễn biến ngược chiều giữa các khung giờ | Phát hiện có ích nhưng chưa phải nguyên nhân |
| Hành động đề xuất | Ưu tiên kiểm tra các ca huỷ cao điểm, dữ liệu thời gian đón và thay đổi ghi nhận | Cần người phụ trách, phạm vi và cách kiểm chứng |

Một insight tốt phải làm rõ điều người ra quyết định cần chú ý. Câu “W2 có 180 lượt huỷ” chỉ nhắc lại số. Câu “cần tách khung giờ trước khi quyết định can thiệp toàn khu vực” nối số liệu với lựa chọn tiếp theo.

Hành động đầu tiên không nhất thiết là ra mắt tính năng. Nhóm có thể cần xác minh dữ liệu, quan sát điểm đón hoặc thử một thay đổi nhỏ. Mức độ hành động phải phù hợp mức bằng chứng đang có.

## 4. Những câu hỏi dữ liệu này chưa trả lời

Ta chưa biết liệu cao điểm có mưa nhiều hơn, thiếu tài xế, có chiến dịch khuyến mãi, đổi chính sách phí hay ứng dụng ghi nhận trạng thái khác trước hay không. Ta cũng chưa có phân bố thời gian đón, dữ liệu lý do huỷ đã xác minh hoặc chi phí hỗ trợ.

Ví dụ, có người nói “huỷ tăng vì tài xế đến chậm”. Đó là **giả thuyết**, chưa phải kết luận. Cần kiểm tra nguồn thời gian, nhóm so sánh và những yếu tố cùng tác động. Ngay cả khi hai đại lượng biến động cùng nhau, quan hệ đó vẫn chưa đủ để kết luận nhân quả.

[NIST mô tả EDA](https://www.itl.nist.gov/div898/handbook/eda/section1/eda11.htm) như cách khám phá cấu trúc, ngoại lệ và giả định của dữ liệu. Tinh thần phù hợp với bài này là xem dữ liệu đang nói gì trước khi ép nó chứng minh câu chuyện ban đầu.

## 5. Viết bản nhận định một trang

Một bản nháp dùng được cho cuộc trao đổi với vận hành:

> Trong cohort lượt đặt xe được nhận ở A, số huỷ trước đón tăng từ 120 lên 180 khi tổng lượt được nhận tăng từ 1.000 lên 1.500. Tỷ lệ chung quan sát được không đổi ở 12%. Tuy nhiên, nhóm cao điểm tăng từ 16% lên 18%, còn nhóm giờ khác giảm từ 8% xuống 6%. Đề xuất ưu tiên kiểm tra dữ liệu và các ca cao điểm trước khi chọn phương án can thiệp. Chưa đủ bằng chứng quy nguyên nhân cho điểm đón hoặc dự báo hiệu quả của chat.

Kèm theo đoạn nhận định là bảng nguồn, công thức và ba việc cần làm: vận hành xác nhận định nghĩa khung giờ; DA kiểm tra thêm các tuần và phân nhóm; người phụ trách dữ liệu xác nhận có thay đổi cách ghi nhận hay không. Đây là đề xuất phân công trong bài tập, chưa phải công việc đã được duyệt.

Không nên viết “dịch vụ ổn vì tỷ lệ không đổi”. Tỷ lệ huỷ chưa đại diện cho toàn bộ chất lượng: thời gian đón, khiếu nại hay thu nhập tài xế có thể thay đổi. Số ca huỷ tuyệt đối tăng cũng có thể tạo thêm nhu cầu xử lý; cần dữ liệu riêng trước khi quy đổi thành số nhân viên hỗ trợ.

## 6. Những lỗi dễ gặp

- **Đọc số đếm như tỷ lệ:** luôn hỏi mẫu số có thay đổi không.
- **Chỉ nhìn trung bình chung:** phân nhóm theo câu hỏi có cơ sở, rồi đối soát lại tổng.
- **Nhầm phần trăm với điểm phần trăm:** từ 16% lên 18% là tăng 2 điểm phần trăm, hay tăng 12,5% tương đối.
- **Đổi phát hiện thành nguyên nhân:** dùng nhãn giả thuyết khi chưa có bằng chứng phù hợp.
- **Chọn công cụ trước câu hỏi:** bảng tính đủ để kiểm tra phép tính này; chưa có căn cứ phải mua kho dữ liệu (data warehouse).

## 7. Bài tập và tự kiểm tra

Viết lại nhận định trên trong tối đa 150 từ, gồm hai dữ kiện, một giới hạn và một bước tiếp có người phụ trách. Đưa cả số tuyệt đối lẫn tỷ lệ vào nhận định.

**Bài tính độc lập:** một cohort khác có 2.000 lượt được nhận và 220 lượt huỷ trước đón. Tính tỷ lệ, rồi so với W2. Không gộp cohort bài tập này vào baseline P01.

<details><summary>Gợi ý lời giải</summary><p>Tỷ lệ là 11%. So với W2, số huỷ tăng 40 lượt nhưng tỷ lệ giảm 1 điểm phần trăm, tương đương giảm khoảng 8,33% tương đối. Chỉ so sánh nghiệp vụ sau khi xác nhận cohort, định nghĩa và độ đầy đủ tương đương; phép tính chưa chứng minh một giải pháp có tác dụng.</p></details>

**Câu hỏi ôn tập:** Vì sao tỷ lệ chung 12% ở cả hai tuần chưa đủ để dừng điều tra?

<details><summary>Gợi ý trả lời</summary><p>Các nhóm diễn biến ngược chiều có thể bù nhau trong số chung. Hơn nữa, tỷ lệ này chỉ mô tả một khía cạnh của trải nghiệm. Cần xem nhóm liên quan tới quyết định và các chỉ số khác, đồng thời tránh diễn giải biến động mẫu thành kết luận chắc chắn.</p></details>

Trước khi bàn giao, kiểm tra: đã ghi cohort và thời gian; đã tính đúng mẫu số; đã đối soát tổng; đã tách dữ kiện với giả thuyết; đã ghi giới hạn và bước tiếp. Nếu thiếu một mục, người đọc có thể dùng đúng số cho sai quyết định.

## 8. Đọc thêm và hồ sơ thực hành

- [Microsoft Learn — Training for Data Analysts](https://learn.microsoft.com/en-us/training/career-paths/data-analyst/): phạm vi công việc DA.
- [NIST — What is EDA?](https://www.itl.nist.gov/div898/handbook/eda/section1/eda11.htm): cách tiếp cận khám phá dữ liệu.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).

Hồ sơ và số đối soát trong repository: `examples/toi-noi-da/p01-nen-tang.md`. Khung chương trình: `docs/da-master-curriculum.md`.

Bài tiếp theo: [DA-002: DA phối hợp với BA, BI, DE và DS](/articles/da-vai-tro-phoi-hop-ba-bi-de-ds).

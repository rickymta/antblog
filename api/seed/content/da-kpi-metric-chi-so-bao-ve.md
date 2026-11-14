---
slug: da-kpi-metric-chi-so-bao-ve
title: "KPI, metric và chỉ số bảo vệ: đo đúng điều cần biết"
summary: "Thiết kế hợp đồng chỉ số huỷ trước đón cho Tới Nơi: grain, cohort, mẫu số, kỳ đo, dữ liệu trễ và chỉ số bảo vệ để tránh cải thiện một con số nhưng làm trải nghiệm xấu hơn."
category: data-analysis
level: intermediate
tags: [data-analysis, da-005, kpi, metric-contract, guardrail, toi-noi]
language: vi
---

Hai dashboard cùng ghi “tỷ lệ huỷ” nhưng một cái chia cho mọi yêu cầu, cái kia chia cho lượt đã được nhận. Cả hai phép chia có thể đúng; tên gọi quá ngắn khiến người xem tưởng chúng là một chỉ số.

Trước khi thống nhất màu biểu đồ, nhóm Tới Nơi cần thống nhất **đang đếm cái gì, trên tập nào và tại thời điểm nào**. Đó là công việc thiết kế chỉ số.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-005 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/da-analysis-brief-cau-hoi-kiem-chung">DA-004: analysis brief</a>. Đầu ra: metric contract và danh sách chỉ số bảo vệ dự kiến. Sau bài này, bạn định nghĩa được chỉ số đủ rõ để hai người tính độc lập và đối chiếu kết quả.</p>
</div>

## 1. Metric, KPI, target và guardrail

**Metric** là đại lượng có định nghĩa đo. **KPI** là chỉ số được chọn làm trọng tâm theo dõi một mục tiêu cụ thể. Không phải cứ xuất hiện trên dashboard thì một metric trở thành KPI.

**Target** là mức đích mong muốn trong một thời gian xác định; **baseline** là mức tham chiếu được chọn theo điều kiện rõ ràng. **Guardrail**, hay chỉ số bảo vệ, theo dõi mặt mà nhóm không muốn làm xấu đi khi cải thiện mục tiêu chính. Microsoft phân biệt nhóm chỉ số thành công, bảo vệ và chất lượng trong [hướng dẫn chuẩn bị thử nghiệm](https://www.microsoft.com/en-us/research/articles/patterns-of-trustworthy-experimentation-pre-experiment-stage/).

Trong P01, tỷ lệ huỷ trước đón là metric chính của câu hỏi. Chưa có target giảm huỷ được duyệt, chưa có baseline đủ dài cho một thử nghiệm. Hai tuần minh hoạ giúp học định nghĩa và so sánh; không tự biến 12% thành chuẩn chung của Tới Nơi.

## 2. Bắt đầu bằng grain và cohort

**Grain** là ý nghĩa của một đơn vị quan sát: một lượt, một khách, một sự kiện hay một giao dịch. P01 dùng một lượt đặt xe đã được nhận. Một lượt có nhiều sự kiện nhưng chỉ được đếm một lần trong mẫu số.

**Cohort** là tập lượt thoả điều kiện đã chốt. Với P01, ta xếp lượt vào W1/W2 theo lần nhận đầu tiên và giữ cohort ấy khi chờ kết quả. Nếu một lượt được nhận cuối W1 nhưng huỷ đầu W2, nó vẫn thuộc tử số của cohort W1 theo định nghĩa này.

Đếm “mọi huỷ xảy ra trong W2” trả lời một câu hỏi theo sự kiện, có thể hữu ích cho vận hành. Nhưng chia số đó cho “lượt nhận trong W2” sẽ trộn hai tập nếu có lượt đi qua biên tuần.

## 3. Hợp đồng M-01 cho tỷ lệ huỷ trước đón

Đây là bản **v0.1 đề xuất trong bài tập**. Các chỗ chưa chốt cho dữ liệu thực tế được giữ rõ.

| Trường | Định nghĩa |
|---|---|
| Tên / ID | M-01 — Tỷ lệ huỷ trước đón trong cohort đã được nhận |
| Mục đích | Mô tả kết quả giai đoạn sau nhận, trước đón |
| Grain | Một lượt đặt xe duy nhất |
| Phạm vi | Dịch vụ ride, khu vực A; mapping A cần xác nhận |
| Mốc cohort | Lần tài xế nhận đầu tiên trong khoảng [đầu kỳ, đầu kỳ kế tiếp) |
| Múi giờ | Asia/Ho_Chi_Minh khi xác định ranh giới kỳ báo cáo |
| Mẫu số | Số lượt duy nhất thuộc cohort |
| Tử số | Số lượt thuộc cohort được xác nhận huỷ trước khi đón |
| Công thức | Tử số / mẫu số × 100% |
| Kết quả chưa biết | Giữ trong cohort, báo riêng; chưa công bố tỷ lệ cuối kỳ khi chưa đủ điều kiện chốt |
| Loại trừ | Dữ liệu test/trùng chỉ loại theo quy tắc có bằng chứng và lưu log |
| Phân nhóm | Cao điểm/còn lại theo thời điểm nhận; lịch cụ thể cần xác nhận |
| Owner | Chủ chỉ số phía vận hành, DA soạn; chưa gán người thật |
| Version / cập nhật | Lưu phiên bản nguồn, thời điểm chốt và lịch sử sửa số |

Với mẫu số bằng 0, tỷ lệ **không xác định**, không hiển thị 0% như một kết quả tốt. Nếu chưa đủ kết quả, có thể hiển thị số huỷ đã xác nhận trên cohort kèm nhãn “tạm tính” và số chưa biết; không được làm người đọc hiểu đó là tỷ lệ cuối cùng.

## 4. Biên thời gian và dữ liệu trễ

Dùng khoảng đầu kỳ có tính, đầu kỳ kế tiếp không tính giúp một thời điểm biên chỉ thuộc một kỳ. Không dùng “tới 23:59:59” như cách bao hết ngày nếu nguồn có phần giây nhỏ hơn.

Độ trễ có hai mặt: hành trình chưa kết thúc và kết quả đã xảy ra nhưng chưa tới kho báo cáo. Mỗi mặt cần cách xác minh. Có thể mọi lượt nguồn đã đóng nhưng pipeline vẫn thiếu sự kiện; ngược lại pipeline cập nhật đầy đủ nhưng vẫn còn lượt chưa có kết quả.

P01 giả định bảng luyện tính đã đủ kết quả. Khi triển khai thật, thời gian chờ và điều kiện chốt phải do nhóm xác định từ dữ liệu và nhu cầu sử dụng; bài không áp một con số giờ tuỳ ý. Nếu sửa kỳ cũ, phải cập nhật cả kết luận dùng số đó và thông báo người nhận theo quy trình DA-003.

## 5. Đọc đúng mức thay đổi và tổng hợp

Trong baseline P01, cao điểm từ 80/500 = 16% lên 135/750 = 18%. Chênh lệch là **2 điểm phần trăm**. Mức tăng tương đối là (18% − 16%)/16% = **12,5%**. Nên ghi cả số đếm để người đọc hiểu quy mô.

Không lấy trung bình đơn giản các tỷ lệ khi mẫu số khác nhau. Biến thể độc lập: nhóm X có 10/100 huỷ, nhóm Y có 180/900. Tỷ lệ chung là 190/1.000 = **19%**, không phải (10% + 20%)/2 = 15%. Công thức an toàn là tổng tử số chia tổng mẫu số của các nhóm không trùng và bao phủ đúng tập.

Nếu nhóm chồng lấn, ngay cả cộng tử số/mẫu số cũng sai. Một khách có lượt ở cả hai khu vực không thể được cộng thành hai khách duy nhất cho toàn hệ thống. Grain phải đi cùng mỗi chỉ số.

## 6. Chọn chỉ số bảo vệ theo cách có thể bị đánh đổi

Giả sử một thay đổi làm nút huỷ khó tìm hơn. Tỷ lệ huỷ có thể giảm trong khi khách chờ lâu hoặc phải gọi hỗ trợ. Vì vậy, cần dự kiến mặt trái của hành động và cách phát hiện nó.

| Chỉ số dự kiến | Điều muốn bảo vệ | Khoảng trống hiện tại |
|---|---|---|
| Tỷ lệ lượt có khiếu nại trong cửa sổ theo dõi | Trải nghiệm khi gặp vấn đề | Cần mapping ticket–lượt, định nghĩa khiếu nại và cửa sổ |
| Thời gian chờ và tỷ lệ chưa giải quyết | Tránh “giảm huỷ” bằng cách giữ lượt treo | Cần lịch sử, cách xử lý lượt chưa đón/đã huỷ |
| Tỷ lệ hoàn thành trên toàn bộ yêu cầu | Tránh cải thiện cohort đã nhận bằng cách loại khách khó phục vụ trước nhận | Cần dữ liệu mọi yêu cầu và kết quả |
| Tỷ lệ lỗi ứng dụng ở luồng liên quan | Tránh đổi trải nghiệm gây lỗi | Cần định nghĩa phiên/lượt và nguồn ghi nhận |

Đây là danh sách đề xuất, **chưa có baseline, ngưỡng hoặc kết quả đo**. Mỗi chỉ số muốn sử dụng đều cần hợp đồng riêng. Không dùng “không tăng” mà quên cửa sổ, sai số và mức tác động được coi là đáng quan tâm.

Chỉ số chất lượng dữ liệu có vai trò khác: ví dụ tỷ lệ lượt thiếu thời điểm nhận cho biết phép đo có đáng tin không. Nó không thay thế chỉ số trải nghiệm, nhưng có thể khiến nhóm phải hoãn dùng kết quả.

## 7. Bài tập: tìm lỗi trong định nghĩa

Review câu sau: “Tỷ lệ huỷ tuần = số event cancelled tuần đó / số khách đã đặt xe; lấy dữ liệu hiện tại, tuần sau chạy lại cũng được.” Hãy chỉ ra ít nhất bốn điểm cần sửa và viết hợp đồng ngắn cho M-01.

<details><summary>Gợi ý lời giải</summary><p>Tử số event và mẫu số khách khác grain; event trong kỳ chưa chắc thuộc cohort cùng kỳ; chưa phân biệt huỷ trước/sau nhận và trước đón; thiếu ranh giới kỳ báo cáo/timezone; không có chính sách dữ liệu trễ và version. Với M-01, đếm lượt duy nhất ở cả tử và mẫu, cố định cohort theo lần nhận đầu, kiểm chứng lịch sử trước đón, lưu thời điểm chốt và kết quả chưa biết.</p></details>

**Bài tính:** Nhóm X/Y trong ví dụ có tỷ lệ chung bao nhiêu? Nếu nhóm Y thiếu 20 kết quả cuối, có được tiếp tục ghi “tỷ lệ cuối kỳ 19%” không?

<details><summary>Gợi ý trả lời</summary><p>Khi đủ kết quả như đề gốc, 190/1.000 = 19%. Nếu đổi điều kiện sang thiếu 20 kết quả, phải nêu đó là số đã xác nhận/tạm tính và đánh giá khả năng sửa tỷ lệ; không giữ nhãn kết quả cuối. Không loại 20 lượt khỏi mẫu số chỉ để hết giá trị thiếu.</p></details>

Checklist: grain; cohort; tử/mẫu cùng tập; biên thời gian; mẫu số 0; trạng thái chưa biết; loại trừ có log; owner/version; guardrail có định nghĩa riêng.

## 8. Tham khảo và bước tiếp

- [Microsoft Research — Pre-Experiment Stage](https://www.microsoft.com/en-us/research/articles/patterns-of-trustworthy-experimentation-pre-experiment-stage/): chọn chỉ số và chuẩn bị đánh giá tác động.
- [DA-003: xử lý số liệu sửa lại](/articles/da-vong-doi-bai-toan-phan-tich).

Mẫu hợp đồng: `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`. Bài tiếp: [DA-006: chọn loại phân tích](/articles/da-mo-ta-chan-doan-du-bao-hanh-dong).

---
slug: ba-bpmn-handoff-timer-ngoai-le
title: "Handoff, timer và luồng ngoại lệ trong BPMN"
summary: "Mô tả bàn giao có xác nhận, chờ phản hồi và hết giờ trong luồng thu phí huỷ. Tách timeout khỏi từ chối, retry khỏi xử lý mới và trạng thái chuyến khỏi kết quả thanh toán."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-022, bpmn, timeout, handoff, toi-noi]
language: vi
---

Chuyến đã huỷ, hệ thống gửi yêu cầu thu phí sang VíXanh nhưng chưa thấy phản hồi. Nếu quy trình chỉ vẽ “thu tiền → kết thúc”, BA chưa chỉ ra người dùng thấy gì, có thử lại không hoặc ai xử lý khi hết lượt thử. Nếu vẽ “lỗi ví → quay lại accepted”, mô hình lại trái baseline.

Ngoại lệ cần được mô tả như một phần của quy trình. Chúng quyết định trách nhiệm ở chỗ bàn giao, nhất là khi hai participant không cùng biết kết quả tại cùng thời điểm.

<div class="tkh-callout"><span class="tkh-callout-title">BA-022 · Phần 3/8 · Trung bình</span><p>Đọc trước: <a href="/articles/ba-state-model-trang-thai-chuyen">State model</a>. Đầu ra: mô tả luồng tương tác, timeout và bàn giao xử lý sau. Baseline lấy R-PAY/R-IDEM; không giả định lab đã có màn hình xử lý thủ công.</p></div>

## 1. Handoff cần cả nội dung và cách biết đã được nhận

Với phí huỷ qua ví, bên gửi cần nêu số tiền, định danh chuyến và khoá chống xử lý lặp. Bên nhận trả thành công, declined hoặc lỗi/không phản hồi theo hợp đồng. Mũi tên “gửi sang ví” chưa đủ diễn đạt trách nhiệm khi phản hồi bị mất.

| Handoff | Điều cần xác định |
|---|---|
| Tới Nơi → VíXanh | Số tiền, khoá `trip:<id>:cancel-fee`, điều kiện gửi |
| VíXanh → Tới Nơi | Kết quả, ý nghĩa lỗi, cách nối với yêu cầu |
| Tới Nơi → xử lý sau | Trường hợp failed, thông tin cần giữ và owner cần chốt |

Phần xử lý sau là điểm cần thiết kế thêm nếu mở rộng lab; không ghi “CSKH tự động nhận ticket” như thể đã được SPEC định nghĩa.

## 2. Chọn timer theo phạm vi muốn mô tả

Theo [BPMN 2.0.2](https://www.omg.org/spec/BPMN/2.0.2/PDF), boundary timer gắn vào activity và có thể ngắt hoặc không ngắt activity tuỳ loại. Timer không phải chính sách huỷ chuyến mặc định; nhánh sau timer phải nói rõ công việc nào tiếp diễn.

Ở mô hình phân tích của bài, activity “chờ kết quả lần gọi ví” có timer ngắt sau ba giây để rẽ sang đánh giá retry. Việc ngừng chờ phía Tới Nơi không chứng minh VíXanh chưa xử lý hoặc đã huỷ yêu cầu của nó. Vì vậy retry vẫn dùng cùng khoá nghiệp vụ để tránh thu hai lần.

Một timer nhắc việc không ngắt có ý nghĩa khác: nó tạo việc theo dõi trong khi activity gốc còn chạy. Không dùng hai loại thay nhau chỉ vì đều có hình đồng hồ.

## 3. Luồng phân tích thu phí huỷ

Đầu vào: khách đã huỷ hợp lệ, có phí lớn hơn 0 và phương thức vixanh. Tách miễn phí/tiền mặt trước bước gọi ví. Chuỗi dưới là mô tả logic, không phải mã BPMN thực thi:

```text
Gửi lần 1 bằng khoá K
  thành công               → paid
  declined                 → failed, không retry
  timeout/lỗi tạm thời     → nếu còn lượt, gửi lại bằng K

Mỗi lần chờ tối đa 3 giây; tối đa 3 lần gọi tổng cộng.
Hết lượt mà chưa thành công → failed, cần xử lý sau.
Trạng thái chuyến vẫn cancelled.
```

Không đổi K theo số lần gọi. Khoá mới khiến cổng ví có thể coi đây là giao dịch mới dù phía người dùng chỉ bấm huỷ một lần. Quy tắc bảo đảm không trừ hai lần cùng khoá là giả định hợp đồng đã nêu trong SPEC lab, không phải bảo đảm chung của mọi cổng thanh toán.

## 4. Tách các loại kết quả lỗi

| Tình huống | Ý nghĩa trong baseline | Hành động |
|---|---|---|
| Thành công | Có kết quả thu thành công | paid |
| declined | Bị từ chối | Không retry; failed |
| Timeout hoặc lỗi tạm thời | Chưa có kết quả dùng được của lần gọi | Retry nếu còn lượt, giữ K |
| Hết lượt thử | Chưa thành công sau giới hạn | failed; chuyến vẫn cancelled |

Timeout là trạng thái hiểu biết của bên chờ, không tự là bằng chứng không có giao dịch. Khi thiết kế vận hành thật cần làm rõ đối soát, phản hồi muộn và phục hồi, nhưng không tự bổ sung cách xử lý chưa có vào baseline lab.

## 5. Bàn giao sau lỗi cần owner và dữ liệu

Đề xuất bản ghi xử lý sau có thể gồm mã chuyến, số tiền, khoá nghiệp vụ, kết quả từng lần và nguồn đối soát. Quyền xem và thời hạn lưu cần được chốt theo bối cảnh; bài chỉ dùng dữ liệu giả.

| Câu hỏi mở | Vì sao quan trọng |
|---|---|
| Ai xem failed và khi nào? | Tránh hàng đợi không ai nhận |
| Căn cứ nào xác nhận đã thu/chưa thu? | Tránh thu lại khi chỉ mất phản hồi |
| Quyết định thu lại/miễn/đối soát thuộc ai? | BA không tự cấp quyền tài chính |
| Nếu người phụ trách vắng mặt? | Cần phương án chuyển cấp |

Một end event tại “ghi failed” chỉ kết thúc phạm vi xử lý tự động đang vẽ. Nó không khẳng định nghĩa vụ đối soát hoặc hỗ trợ đã hoàn tất.

## 6. Timer nghiệp vụ và lịch thực thi khác nhau

Hết năm phút tìm tài xế là điều kiện nghiệp vụ R-STATE-05; vòng quét 15 giây là lịch xử lý của lab. Timeout ba giây là giới hạn chờ mỗi lần gọi ví. Hai con số phục vụ hai tình huống khác nhau, không dùng timer ví để huỷ tìm tài xế.

Nếu mô hình có nhánh timer cạnh tranh với một thông điệp, cần review điều gì xảy ra khi chúng đến gần nhau và dữ liệu được cập nhật lúc nào. BPMN trên giấy không tự giải quyết transaction hoặc race; tài liệu phải nêu kết quả nghiệp vụ được chấp nhận để kỹ thuật triển khai.

## 7. Bài tập và câu hỏi ôn tập

Lập bảng ba lần gọi cho ca: lần 1 timeout, lần 2 lỗi tạm thời, lần 3 thành công. Ghi cùng K, kết quả cuối và trạng thái chuyến. Thay lần 2 bằng declined rồi xác định còn gọi lần 3 không.

<details><summary>Gợi ý trả lời</summary><p>Ca đầu kết thúc cancelled/paid nếu lần 3 thành công. Ca thay đổi dừng ở lần 2, cancelled/failed vì declined không retry. Không quay chuyến về accepted và không đổi khoá.</p></details>

## 8. Tham khảo và bài tiếp theo

- [OMG — BPMN 2.0.2, Events](https://www.omg.org/spec/BPMN/2.0.2/PDF), tham chiếu ký pháp timer; luồng thanh toán là ví dụ riêng từ SPEC lab.
- `examples/toi-noi-testing/SPEC.md`, R-PAY-04–06, R-IDEM và R-STATE-05.

Bài tiếp theo: [Kiểm tra nhất quán giữa quy trình, trạng thái và business rule](/articles/ba-kiem-tra-nhat-quan-mo-hinh).

---
slug: ba-kiem-tra-nhat-quan-mo-hinh
title: "Kiểm tra nhất quán giữa quy trình, trạng thái và business rule"
summary: "Walkthrough cùng một case qua BPMN, transition table, decision table và kết quả thanh toán. Phát hiện chỗ tài liệu mâu thuẫn, thiếu điều kiện hoặc vượt nguồn trước khi đưa vào phát triển."
category: business-analysis
level: advanced
tags: [business-analysis, ba-023, model-review, traceability, toi-noi]
language: vi
---

Sơ đồ quy trình nói tài xế huỷ sẽ kết thúc chuyến. Transition table nói chuyến quay về searching. Decision table lại áp phí khách vì đã qua hai phút. Mỗi tài liệu có thể được một người review riêng và trông hợp lý, nhưng khi cùng áp dụng vào một sự kiện, chúng cho ba kết quả không tương thích.

Review liên mô hình dùng một case duy nhất để lần theo mọi quyết định và tác động. Mục tiêu là tìm chỗ không nối được, không phải chỉ xác nhận từng hình đã đúng ký pháp.

<div class="tkh-callout"><span class="tkh-callout-title">BA-023 · Phần 3/8 · Nâng cao</span><p>Đọc trước: <a href="/articles/ba-bpmn-handoff-timer-ngoai-le">Handoff và ngoại lệ</a>. Đầu ra: traceability matrix và defect log của mô hình. Các lỗi minh hoạ là lỗi cài trong bài tập, không phải kết luận code lab đang có lỗi.</p></div>

## 1. Mỗi mô hình trả lời một phần câu hỏi

| Mô hình | Câu hỏi chính | Không tự trả lời |
|---|---|---|
| Process/BPMN | Công việc nào diễn ra, theo đường nào, ai tham gia? | Toàn bộ công thức phí |
| State model | Đối tượng có chuyển trạng thái hợp lệ không? | Lợi ích kinh doanh của chính sách |
| Decision table | Với dữ kiện này, kết quả quyết định là gì? | Cách xử lý cập nhật cạnh tranh |
| Dữ liệu/hợp đồng | Dữ kiện ở đâu, có nghĩa gì, version nào? | Quyền ưu tiên và quyết định đầu tư |

Các mô hình liên kết bằng ID, đầu vào và outcome. Nếu bảng phí cần acceptedAt nhưng quy trình không xác định lấy mốc nào sau lần nhận lại, đó là khoảng trống dù từng ô riêng chưa có lỗi cú pháp.

## 2. Chuẩn bị case đủ dữ kiện

Case review: khách chủ chuyến huỷ xe máy ở accepted, acceptedAt 10:00:00, ETA 5 phút, xử lý lúc 10:02:00, phương thức vixanh, ví declined; không có replay hoặc tranh chấp đồng thời.

Kết quả kỳ vọng theo baseline: chuyển cancelled, phí 10.000 đ, lý do charged; ví declined không retry, paymentStatus failed. Trạng thái thu tiền không làm đảo trạng thái chuyến. Không gọi đây là “huỷ thất bại” chỉ vì thu tiền thất bại.

| Bước walkthrough | Mô hình/nguồn | Kết quả cần thống nhất |
|---|---|---|
| Kiểm actor và state | R-STATE | Cho phép customer_cancel |
| Chọn transition | State table | accepted → cancelled |
| Tính phí | D4 / R-CANCEL | 10.000 đ, charged |
| Xử lý ví | Process / R-PAY | declined, không retry |
| Ghi outcome | Dữ liệu | cancelled/failed; giữ phí và lý do |

## 3. Cài lỗi có chủ đích để luyện review

Đưa vào bản nháp ba lỗi: G dùng `<= 2 phút`; gateway chỉ cho phép huỷ nếu ví đã thu thành công; driver_cancel đi cancelled. Chạy case trên và case driver_cancel qua cả ba mô hình.

| Lỗi nháp | Vì sao sai | Sửa theo nguồn |
|---|---|---|
| Miễn ở đúng hai phút | Trái R-CANCEL-02 | Dùng `< 2 phút` |
| Thu tiền thành công mới huỷ | Trái R-PAY-06 | Tách outcome chuyến và thanh toán |
| driver_cancel → cancelled | Trái R-STATE-04 | Quay searching và thực hiện các effect liên quan |

Đừng sửa riêng chữ trong hình rồi để bảng case cũ. Mỗi finding cần liệt kê mọi sản phẩm bị ảnh hưởng và cách kiểm lại để không còn hai phiên bản khác nhau.

## 4. Các invariant giúp bắt lỗi xuyên mô hình

- Chỉ transition hợp lệ và actor đúng mới được thay trạng thái.
- Tài xế rời chuyến không thu phí khách theo baseline.
- Khách huỷ sau picked_up bị từ chối, không phải huỷ miễn phí.
- Retry/replay không tạo lần thu mới cho cùng yêu cầu đã được xử lý.
- Không có chuyện thanh toán thất bại tự phục hồi chuyến cancelled thành accepted.

Invariant là điều phải giữ qua nhiều đường đi. Nó bổ sung cho case cụ thể vì một case chỉ kiểm một tổ hợp. Dù vậy, danh sách invariant trên chỉ bao phủ phạm vi bài, không chứng minh toàn hệ thống đúng.

## 5. Defect log cho tài liệu

| Trường | Ví dụ finding M-02 |
|---|---|
| Trigger | Khách huỷ hợp lệ, ví declined |
| Mâu thuẫn | Process chặn huỷ nhưng R-PAY-06 cho phép hoàn tất huỷ |
| Ảnh hưởng | UI, API, CSKH hiểu khác kết quả chuyến |
| Nguồn chuẩn | SPEC lab, R-PAY-06 và R-STATE |
| Sửa đề xuất | Tách ghi nhận huỷ khỏi thành công thanh toán |
| Cách xác nhận | Walkthrough declined, timeout hết lượt và thành công |
| Owner/status | Người sở hữu mô hình xác nhận; chưa đóng khi chỉ sửa một nơi |

Nếu nguồn cũng mâu thuẫn, không để BA tự chọn văn bản mới nhất theo thời gian file. Ghi câu hỏi nghiệp vụ và chuyển đúng người có quyền xác nhận baseline, giữ trạng thái còn mở.

## 6. Review thứ tự, dữ liệu và version

Một quy trình có đủ task vẫn có thể sai thứ tự: tính lại phí trước khi xử lý replay có thể dùng chính sách mới cho yêu cầu cũ; đọc acceptedAt trước khi tài xế rời rồi sử dụng sau khi người khác nhận có thể không còn phù hợp.

BA cần nêu dữ kiện phải nhất quán tại quyết định và kết quả khi có cạnh tranh. Không buộc một giải pháp khoá cụ thể, nhưng cũng không để từ “atomic” thay thế mô tả hành vi cần bảo đảm. Review phải có developer và tester để nối kỳ vọng với khả năng thực thi.

## 7. Bài tập và câu hỏi ôn tập

Chọn C02, C06, C10, C12, C13 của hồ sơ P03. Với mỗi ca ghi actor, state trước/sau, rule chọn, phí/lý do, outcome thanh toán và nguồn. Tìm một điểm cần thêm quyết định, như acceptedAt thiếu hoặc hiệu lực version mới.

**Câu hỏi:** Các mô hình cùng cho một kết quả có đủ chứng minh đúng không?

<details><summary>Gợi ý trả lời</summary><p>Không. Chúng có thể cùng dựa vào một giả định sai. Cần đối chiếu nguồn được chấp nhận và mục tiêu nghiệp vụ, ngoài kiểm tra nhất quán nội bộ.</p></details>

## 8. Tham khảo và bài tiếp theo

- `examples/toi-noi-testing/SPEC.md`, R-STATE/R-CANCEL/R-PAY/R-IDEM.
- [Decision table](/articles/ba-decision-table-phi-huy), [State model](/articles/ba-state-model-trang-thai-chuyen).

Bài tiếp theo: [Thực hành: thay đổi chính sách huỷ trên nhiều luồng](/articles/ba-thuc-hanh-thay-doi-chinh-sach-huy).

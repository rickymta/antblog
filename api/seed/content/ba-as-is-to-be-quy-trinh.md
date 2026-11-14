---
slug: ba-as-is-to-be-quy-trinh
title: "As-is và to-be: mô tả hiện tại trước khi cải tiến"
summary: "Dựng quy trình hiện tại có nguồn, tách đặc tả lab khỏi hành vi quan sát và đề xuất luồng tương lai cho Tới Nơi. Xác định trigger, outcome, ranh giới và khoảng trống trước khi vẽ chi tiết."
category: business-analysis
level: beginner
tags: [business-analysis, ba-017, process-model, as-is, to-be, toi-noi]
language: vi
---

Sau discovery, nhóm muốn sửa ngay luồng điểm đón. Một người vẽ “khách nhập cổng → tài xế đọc → gặp nhau”, rồi gọi đó là quy trình hiện tại. Nhưng P02 mới có lời kể và vài quan sát ở các tình huống khác nhau; chưa ai xác nhận tài xế luôn đọc trước khi tới.

Mô hình as-is giúp nhìn cách công việc đang diễn ra hoặc được quy định. Giá trị của mô hình phụ thuộc vào việc nói rõ nguồn và giới hạn, không phụ thuộc số ô vẽ đẹp trên trang.

<div class="tkh-callout"><span class="tkh-callout-title">BA-017 · Phần 3/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/ba-thuc-hanh-tong-hop-discovery">Tổng hợp discovery</a>. Đầu ra: một as-is có nguồn, một to-be có mục tiêu và gap list. P03 dùng SPEC lab làm baseline đặc tả; không coi đó là bằng chứng vận hành thật.</p></div>

## 1. Chọn loại hiện trạng đang mô tả

Có ít nhất ba lớp: quy trình được tài liệu quy định, hành vi đã quan sát và cách phần mềm thực thi. Chúng cần được đối chiếu, không tự đồng nhất. Với điểm đón, P02 chưa đủ nguồn để dựng một hành trình thực tế hoàn chỉnh; với huỷ chuyến, SPEC lab cho phép mô hình hoá hành vi được đặc tả.

| Mô hình | Nguồn | Cách ghi nhãn |
|---|---|---|
| Đường đi thông tin điểm đón | INT/OBS của P02 | Bản nháp tổng hợp, có ô chưa xác minh |
| Khách huỷ chuyến | R-STATE, R-CANCEL, R-PAY trong SPEC | Baseline đặc tả lab |
| Cải thiện cách nhận biết điểm đón | Giả thuyết và phương án | To-be đề xuất, chưa duyệt |

## 2. Chốt trigger, outcome và phạm vi

Với bài toán huỷ, trigger là khách gửi yêu cầu huỷ, không phải “hệ thống tự nhiên đang ở accepted”. Outcome có thể là yêu cầu bị từ chối hoặc chuyến bị huỷ với phí/trạng thái thanh toán được xác định. Việc đã huỷ không đồng nghĩa đã thu tiền thành công.

Phạm vi bản đầu chỉ gồm trường hợp khách chủ chuyến huỷ. Tài xế rời chuyến và hệ thống hết thời gian tìm tài xế là luồng khác, sẽ nối ở mô hình đầy đủ. Ghi rõ để tránh một nhãn “huỷ” che ba hành vi khác nhau.

## 3. Vẽ đường chính và các điểm phải quyết định

Sơ đồ dưới là flow phân tích, chưa phải BPMN. Sơ đồ mô tả khách huỷ theo baseline ở mức khái quát; quyền, replay và cập nhật nhất quán cần được làm rõ ở mô hình chi tiết.

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 860 200" role="img" aria-label="Luồng khách yêu cầu huỷ: kiểm tra điều kiện, quyết định phí và ghi nhận kết quả; điều kiện không hợp lệ thì từ chối."><defs><marker id="ba17-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0L10 5L0 10z" class="f-pk"/></marker></defs><g class="f-pk-soft s-pk"><rect x="20" y="30" width="170" height="52" rx="8"/><rect x="230" y="30" width="170" height="52" rx="8"/><rect x="440" y="30" width="170" height="52" rx="8"/><rect x="650" y="30" width="190" height="52" rx="8"/></g><g class="t-small" text-anchor="middle"><text x="105" y="61">Yêu cầu huỷ</text><text x="315" y="61">Kiểm tra điều kiện</text><text x="525" y="61">Quyết định phí</text><text x="745" y="61">Ghi nhận kết quả</text></g><g class="s-pk" stroke-width="2" marker-end="url(#ba17-a)"><path d="M190 56H228"/><path d="M400 56H438"/><path d="M610 56H648"/><path d="M315 82V126"/></g><rect x="230" y="130" width="170" height="45" rx="8" class="f-warn-soft s-warn"/><text x="315" y="158" text-anchor="middle" class="t-small">Từ chối nếu sai</text><text x="440" y="153" class="t-small">Huỷ thành công ≠ thu tiền thành công</text></svg></div><figcaption>As-is theo đặc tả lab, rút gọn để thấy quyết định nghiệp vụ.</figcaption></figure>

Gắn mỗi bước với nguồn: quyền và trạng thái từ R-STATE; phí từ R-CANCEL; thu ví và thất bại thanh toán từ R-PAY. Khi một bước chưa có nguồn, thêm câu hỏi thay vì lấp bằng hành vi tưởng hợp lý.

## 4. Đừng làm mất ngoại lệ chỉ để sơ đồ gọn

Baseline cho phép khách huỷ ở searching, accepted, arriving. Ở trạng thái khác, từ chối chứ không tính phí 0 rồi huỷ. Nếu ví thất bại, chuyến vẫn cancelled và paymentStatus có thể failed. Cả hai nhánh cần xuất hiện trong tài liệu đi kèm, dù sơ đồ tổng quan chưa thể hiện hết.

Một điểm quan trọng khác là việc tài xế rời chuyến từ accepted/arriving đưa chuyến về searching, không phải cancelled. Nếu đặt mọi hành động có chữ “huỷ” vào cùng một đường kết thúc, mô hình sẽ sai nghiệp vụ.

## 5. To-be cần giải thích khoảng cách và giá trị

Giả sử nhóm đề xuất làm rõ ghi chú điểm đón. To-be nên nêu khác biệt: khách nhận hướng dẫn mô tả, tài xế nhận cùng thông tin đúng lúc, CSKH có dữ liệu liên kết để hỗ trợ. Đây là phương án, chưa chứng minh giảm tỷ lệ huỷ.

| Khoảng cách cần xử lý | Thay đổi đề xuất | Bằng chứng cần bổ sung |
|---|---|---|
| Khó gọi tên cổng ở C-03 | Hướng dẫn theo địa điểm | Người dùng có hiểu, ai duy trì dữ liệu? |
| Thời điểm nhận biết ghi chú còn mơ hồ | Thử cách hiển thị | Quan sát có/không dẫn dắt, điều kiện sử dụng an toàn |
| Ticket thiếu liên kết trong mẫu | Làm rõ cách ghi ID | Quy trình, chất lượng dữ liệu, ngoại lệ |

To-be không cần thêm bước ở mọi vị trí. Thay đổi có thể chỉ là bỏ thao tác dư hoặc làm rõ trách nhiệm. Mỗi thay đổi nên nối với nhu cầu, người phụ trách và cách đánh giá.

## 6. Review hai mô hình cạnh nhau

Đi qua một tình huống thuận lợi và một tình huống lỗi với người nghiệp vụ. Hỏi bước nào thay đổi, dữ liệu nào được tạo, ai nhận bàn giao và outcome nào khác trước. Nếu không giải thích được khác biệt, bản to-be có thể chỉ là bản vẽ lại cho đẹp.

Lưu phiên bản riêng cho as-is và to-be. Khi đề xuất được duyệt, ghi quyết định và ngày hiệu lực; đừng xoá bản cũ vì còn cần giải thích dữ liệu hoặc sự cố của giai đoạn trước.

## 7. Bài tập và tự kiểm tra

Vẽ hai luồng: khách huỷ accepted và tài xế rời accepted. Nêu trigger, actor, outcome, nguồn SPEC. Sau đó đề xuất một thay đổi hỗ trợ điểm đón và ghi ba giả định cần xác minh.

**Câu hỏi:** As-is theo SPEC có chứng minh code chạy đúng không?

<details><summary>Gợi ý trả lời</summary><p>Không. Đó là mô hình của đặc tả. Muốn biết thực thi, cần kiểm thử hoặc quan sát hệ thống và đối chiếu cùng phiên bản.</p></details>

## 8. Tham khảo và bài tiếp theo

- [Dự án Tới Nơi](/articles/du-an-mau-toi-noi); SPEC lab trong repository: `examples/toi-noi-testing/SPEC.md`, R-STATE/R-CANCEL/R-PAY.
- Hồ sơ P03: `examples/toi-noi-ba/p03-mo-hinh-quy-trinh.md`.

Bài tiếp theo: [BPMN cơ bản: task, event, gateway và lane](/articles/ba-bpmn-task-event-gateway-lane).

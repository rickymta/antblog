---
slug: ba-wireframe-prototype-kiem-tra-hieu-biet
title: "Wireframe và prototype để kiểm tra hiểu biết"
summary: "Dùng wireframe để làm rõ ý nghĩa xác nhận và prototype để thử giả thuyết sử dụng. Ghi nhận hành vi, câu hỏi mở và giới hạn trước khi coi thiết kế đã được xác thực."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-029, wireframe, prototype, toi-noi]
language: vi
---

Khách nhìn nhãn “Đã xác nhận” và nghĩ tài xế đã biết mình đứng ở đâu. Nhóm thiết kế chỉ muốn nói hệ thống ghi nhận khách xác nhận thông tin. Một nhãn ngắn có thể tạo kỳ vọng vượt xa hành vi thật; prototype giúp phát hiện cách hiểu này trước khi xây đầy đủ.

<div class="tkh-callout"><span class="tkh-callout-title">BA-029 · Phần 4/8 · Trung bình</span><p>Đọc trước: <a href="/articles/ba-story-mapping-lat-cat-gia-tri">Story mapping</a>. Đầu ra: wireframe trạng thái và kế hoạch thử giả thuyết. Không dùng prototype như bằng chứng API hoặc đồng bộ đã hoạt động.</p></div>

## 1. Chọn điều muốn học

Giả thuyết P04: khách phân biệt “tôi đã xác nhận nội dung” với “tài xế đã đọc”. Câu hỏi thử: sau thao tác, người tham gia nghĩ ai biết thông tin gì? Đừng hỏi dẫn dắt “Bạn hiểu tài xế chưa đọc đúng không?”.

[GOV.UK mô tả nhiều cách làm prototype](https://www.gov.uk/service-manual/design/making-prototypes). Độ chi tiết nên phục vụ câu hỏi: bản giấy có thể đủ kiểm bố cục/câu chữ; tương tác thực tế hơn cần khi kiểm luồng và phản hồi.

## 2. Wireframe trạng thái thường

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 720 260" role="img" aria-label="Wireframe đề xuất: thông tin cổng số 2, nút xác nhận thông tin, chú thích không đồng nghĩa tài xế đã đọc."><rect x="120" y="15" width="480" height="225" rx="12" class="f-surface s-line"/><text x="145" y="48" class="t-b">Điểm đón của bạn</text><rect x="145" y="65" width="430" height="55" rx="6" class="f-pk-soft s-pk"/><text x="165" y="99" class="t-small">Cổng số 2 · Thông tin hiện tại</text><rect x="145" y="135" width="260" height="43" rx="7" class="f-pk-soft s-pk"/><text x="165" y="162" class="t-small">Xác nhận thông tin này</text><text x="145" y="204" class="t-small">Bạn xác nhận nội dung mình cung cấp.</text><text x="145" y="224" class="t-small">Điều này không có nghĩa tài xế đã đọc.</text></svg></div><figcaption>Wireframe khái niệm, không phải UI đã duyệt. Revision là cơ chế thiết kế; chỉ hiển thị cho khách nếu giúp họ ra quyết định.</figcaption></figure>

## 3. Thiết kế cả trạng thái khó

| Trạng thái | Cần truyền đạt | Điều không được hứa |
|---|---|---|
| Đang gửi | Đang xử lý thao tác | Chưa báo đã lưu |
| Revision đổi | Cần xem lại nội dung mới | Không giữ dấu xác nhận cho bản khác |
| accepted thắng trước | Không còn ghi trong phạm vi hiện tại | Không giả vờ lưu rồi bỏ qua |
| Timeout | Chưa biết kết quả, đang/có thể kiểm lại | Không khẳng định thất bại hoặc thành công khi chưa biết |

Không chỉ vẽ màn hình đẹp nhất. Phản hồi lỗi là nơi người dùng quyết định thao tác tiếp và có thể tạo lặp nếu hiểu sai.

## 4. Thử bằng nhiệm vụ, không bằng lời khen

Giao nhiệm vụ: “Bạn đang chờ xe ở cổng số 2. Hãy kiểm tra app đang dùng thông tin nào và làm điều bạn cho là cần.” Sau đó hỏi người tham gia giải thích trạng thái họ thấy. Ghi thao tác, câu nói, lời nhắc và chỗ họ dừng.

Không cho biết trước nút nào cần bấm. Nếu phải trợ giúp, ghi mức trợ giúp; hoàn thành nhờ hướng dẫn không giống tự hoàn thành. Dùng dữ liệu giả và nói rõ prototype có giới hạn.

## 5. Ghi kết quả theo giả thuyết

Một phiên có thể cho thấy nhãn bị hiểu là tài xế đã đọc. Ghi nguồn, ngữ cảnh, câu diễn giải và đề xuất sửa; không kết luận “mọi khách hiểu sai” từ một người. Sau khi sửa, thử lại câu hỏi trọng tâm thay vì chỉ hỏi người làm thiết kế thích phiên bản nào.

Prototype dùng được không chứng minh giảm huỷ, đáp ứng hiệu năng hoặc an toàn truy cập. Các kết luận đó cần nguồn và phương pháp khác.

## 6. Liên kết quyết định thiết kế

Mỗi trạng thái wireframe nối tới AC/use case. Nếu thiết kế thêm nút sửa sau accepted, nó đã mở rộng scope và phải review rule, quyền, đồng bộ, dữ liệu. Không để một thao tác thêm trên prototype âm thầm trở thành yêu cầu đã duyệt.

## 7. Bài tập và tự kiểm tra

Vẽ trạng thái revision đã đổi và timeout. Viết một nhiệm vụ thử, hai câu hỏi không dẫn dắt và bảng ghi kết quả. Đánh dấu đâu là quan sát, đâu là giả thuyết thiết kế.

**Câu hỏi:** Có nên hiển thị tên trường pickupRevision cho khách để họ hiểu hệ thống?

<details><summary>Gợi ý trả lời</summary><p>Chỉ nếu thật sự giúp quyết định. Thường cần giải thích thông tin đã đổi và yêu cầu xem lại; tên trường kỹ thuật có thể không giúp người dùng.</p></details>

## 8. Tham khảo và bài tiếp theo

- [GOV.UK — Making prototypes](https://www.gov.uk/service-manual/design/making-prototypes).

Bài tiếp theo: [Yêu cầu phi chức năng: từ “nhanh, an toàn” đến tiêu chí đo](/articles/ba-yeu-cau-phi-chuc-nang-do-duoc).

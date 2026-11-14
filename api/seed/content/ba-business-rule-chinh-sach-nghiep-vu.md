---
slug: ba-business-rule-chinh-sach-nghiep-vu
title: "Business rule: tách chính sách khỏi màn hình"
summary: "Viết quy tắc nghiệp vụ có nguồn, phạm vi, điều kiện biên và phiên bản. Dùng phí huỷ Tới Nơi để phân biệt chính sách, quy trình và cách thể hiện trên giao diện."
category: business-analysis
level: beginner
tags: [business-analysis, ba-019, business-rule, cancellation, toi-noi]
language: vi
---

Thiết kế ghi “hiện cảnh báo phí sau hai phút”. Developer hỏi: hai phút tính từ lúc đặt hay lúc tài xế nhận? Tester hỏi: đúng hai phút có phí chưa? CSKH hỏi: tài xế đến trễ thì sao? Một câu về màn hình chưa đủ để cả nhóm thực hiện cùng một chính sách.

Business rule cần diễn đạt quyết định nghiệp vụ độc lập với vị trí nút hoặc màu thông báo. Giao diện trình bày và hỗ trợ người dùng hiểu rule; giao diện không phải nguồn duy nhất định nghĩa rule.

<div class="tkh-callout"><span class="tkh-callout-title">BA-019 · Phần 3/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/ba-bpmn-task-event-gateway-lane">BPMN cơ bản</a>. Đầu ra: rule catalog có nguồn và ví dụ biên. Bài dùng R-CANCEL của SPEC lab làm baseline, không đề xuất đổi mức phí hiện hành.</p></div>

## 1. Tách ba lớp đang bị trộn

| Lớp | Câu hỏi | Ví dụ |
|---|---|---|
| Chính sách/quy tắc | Trong điều kiện nào có phí? | Khách huỷ accepted từ hai phút trở lên có phí cơ sở, xét miễn do trễ |
| Quy trình | Khi nào kiểm tra và áp dụng? | Sau khi xác nhận yêu cầu huỷ hợp lệ |
| Giao diện | Người dùng thấy và thao tác gì? | Hiển thị phí cùng lý do trước/xung quanh thao tác theo thiết kế được duyệt |

Rule phải vẫn có ý nghĩa khi yêu cầu tới từ API hoặc kênh hỗ trợ khác. Chỉ ẩn nút huỷ trên app không thay thế kiểm tra quyền và trạng thái ở nơi thực thi nghiệp vụ.

## 2. Viết rule đủ để phân biệt trường hợp

Thay vì “huỷ sớm miễn phí”, viết rõ phạm vi và điều kiện:

> Với yêu cầu huỷ hợp lệ của khách chủ chuyến khi trạng thái là accepted, nếu thời điểm xử lý trừ acceptedAt nhỏ hơn hai phút thì phí bằng 0; đúng hai phút không thuộc khoảng miễn này.

Đây là cách diễn giải R-CANCEL-01/02. Điều kiện hợp lệ rất quan trọng: chuyến picked_up không được khách huỷ theo baseline; không thể dùng điều kiện thời gian để vượt qua giới hạn trạng thái.

Một rule đơn lẻ cũng chưa phải toàn bộ quyết định. Ở accepted từ hai phút hoặc arriving, phải xét R-CANCEL-04: khi `now > acceptedAt + etaMinutes + 10 phút` thì miễn do trễ. Đúng bằng mốc trễ chưa đủ điều kiện miễn này.

## 3. Rule catalog cho ca huỷ

| ID tham chiếu | Nội dung baseline | Dữ liệu cần | Câu hỏi review |
|---|---|---|---|
| R-STATE-02/03 | Khách huỷ chỉ ở searching, accepted, arriving | Actor, chủ chuyến, trạng thái hiện tại | Yêu cầu tới sau pickup thì xử lý gì? |
| R-CANCEL-01/02 | searching miễn; accepted dưới hai phút miễn | State, now, acceptedAt | Đúng hai phút có nằm trong miễn không? |
| R-CANCEL-03 | Xe máy 10.000 đ, ô tô 20.000 đ | Loại xe hợp lệ | Đơn đồ ăn dùng mức xe máy theo SPEC |
| R-CANCEL-04 | Miễn khi vượt hẳn mốc trễ | acceptedAt, ETA, now | Dấu so sánh là lớn hơn, không phải lớn hơn hoặc bằng |
| R-CANCEL-06 | Ưu tiên lý do miễn searching → grace_period → driver_late | Kết quả điều kiện | Phí giống nhau nhưng lý do có đúng không? |

Catalog đầy đủ nên có người sở hữu, phiên bản nguồn, hiệu lực và trạng thái phê duyệt. Các ID trên là ID có sẵn của lab; không tạo rule khác cùng mã. Với đề xuất riêng của BA, dùng mã riêng và liên kết rule bị tác động.

## 4. Dùng ví dụ để bộc lộ cách hiểu khác nhau

Với xe máy, acceptedAt = 10:00:00, ETA = 5 phút, dữ liệu hợp lệ:

| Tình huống độc lập | Kết quả baseline |
|---|---|
| accepted, khách huỷ 10:01:59 | 0 đ, grace_period |
| accepted, khách huỷ 10:02:00 | 10.000 đ, charged |
| arriving, khách huỷ 10:15:00 | 10.000 đ, charged |
| arriving, khách huỷ 10:15:01 | 0 đ, driver_late |

Ví dụ không thay thế định nghĩa. Nếu chỉ cho bốn dòng ví dụ, người đọc vẫn cần biết quy tắc cho mọi thời điểm ở giữa và các loại xe khác. Kết hợp rule ngắn với case biên giúp review hiệu quả hơn.

## 5. Thiếu dữ liệu không phải một điều kiện miễn phí

Nếu accepted nhưng acceptedAt bị thiếu, nhóm không biết đã qua hai phút hay chưa. Không được tự coi là 0 đ hoặc lấy thời điểm tạo chuyến thay vào mà không có quyết định nghiệp vụ.

Ghi lại rằng đây là dữ liệu bất thường, kèm câu hỏi về cách xử lý: từ chối, đối soát, chuyển hỗ trợ hay cơ chế khác? Mỗi phương án có ảnh hưởng khác. Bài không bổ sung hành vi mới cho lab; mục tiêu là phát hiện lỗ hổng cần được quyết định thay vì để từng nơi tự đoán.

Tương tự, một ETA được cập nhật trên giao diện không tự là ETA dùng cho rule. Cần định nghĩa nguồn và thời điểm chốt dữ liệu nếu hệ thống phát triển vượt baseline.

## 6. Quản lý thay đổi bằng phiên bản

CR-BA-01 trong hồ sơ P03 đề xuất đổi khoảng miễn ở accepted từ dưới hai phút sang dưới ba phút. Giữ rule hiện tại làm baseline, ghi đề xuất riêng. Điều này chưa đổi arriving, ngưỡng trễ hay quyền huỷ.

Trước khi duyệt, cần xác định chuyến đang chạy theo phiên bản nào, khi chuyến được tài xế khác nhận lại thì có đổi phiên bản không, nội dung thông báo và cách xử lý yêu cầu gửi lại. Sửa con số trong một tài liệu mà không nối các nơi phụ thuộc sẽ tạo ra hai chính sách cùng tồn tại ngoài ý muốn.

## 7. Bài tập và câu hỏi ôn tập

Viết rule đề xuất ba phút theo mẫu: ID riêng, nguồn yêu cầu, phạm vi, điều kiện, kết quả, ngoại lệ, trạng thái và ba ví dụ biên. Ghi ít nhất hai quyết định còn mở trước khi áp dụng.

**Câu hỏi:** Một yêu cầu khách huỷ không hợp lệ có nên trả phí 0 không?

<details><summary>Gợi ý trả lời</summary><p>Không nên dùng 0 để biểu diễn từ chối. Phí 0 là kết quả của một quyết định phí trong phạm vi phù hợp; từ chối nghĩa là không thực hiện huỷ. Hai kết quả này dẫn tới hành vi khác nhau.</p></details>

## 8. Tham khảo và bài tiếp theo

- Baseline trong repository: `examples/toi-noi-testing/SPEC.md`, R-STATE và R-CANCEL-01–07.
- Hồ sơ đề xuất: `examples/toi-noi-ba/p03-mo-hinh-quy-trinh.md`.

Bài tiếp theo: [Decision table: đặc tả phí huỷ không bỏ sót tổ hợp](/articles/ba-decision-table-phi-huy).

---
slug: ba-thiet-ke-uat-scenario
title: "Thiết kế UAT scenario từ hành trình nghiệp vụ"
summary: "Chuẩn bị bối cảnh, dữ liệu, kết quả mong đợi và bằng chứng cho UAT; bao gồm ngoại lệ có ý nghĩa thay vì chỉ kiểm từng màn hình."
category: business-analysis
level: beginner
tags: [business-analysis, ba-050, uat, scenario, toi-noi]
language: vi
---

“Mở trang và bấm xác nhận” là thao tác, chưa phải một scenario nghiệp vụ đủ rõ. Người nghiệm thu cần biết ai đang xác nhận điều gì, trong trạng thái nào và thế nào được coi là đúng.

<div class="tkh-callout"><span class="tkh-callout-title">BA-050 · Phần 7/8 · Cơ bản</span><p>Đầu ra: năm scenario P07 nối với yêu cầu P04. Các ca là thiết kế đề xuất, chưa có execution result thật.</p></div>

## 1. Từ hành trình đến ca kiểm tra

Hành trình rút gọn: khách xem điểm đón, xác nhận, tài xế được phân công đọc, CS giải thích khi có vướng mắc. Chọn các rủi ro khiến hành trình sai: xác nhận nhầm revision, sai quyền, chuyến đã accepted, hoặc mất response.

Không cần biến mọi field thành một UAT scenario độc lập. Chi tiết validation vẫn có thể được QA kiểm ở lớp phù hợp, rồi UAT tập trung vào outcome nghiệp vụ.

## 2. Mẫu một scenario đầy đủ

UAT-P07-01: khách A là chủ chuyến đang searching, pickupRevision 3 đã được hiển thị. A xác nhận revision 3 trước khi có thay đổi khác. Mong đợi xác nhận được ghi cho đúng chuyến và revision; tải lại vẫn thể hiện kết quả. Bằng chứng cần trạng thái sau thao tác và mã tham chiếu dữ liệu giả.

Không dùng dòng “thành công” làm expected duy nhất. Nó không giúp phát hiện trường hợp UI báo thành công nhưng dữ liệu chưa lưu.

## 3. Bộ ca nối traceability

| ID | Bối cảnh/hành động | Expected nghiệp vụ |
|---|---|---|
| P07-01 | Chủ chuyến, revision hiện tại | Ghi nhận đúng revision |
| P07-02 | Người khác xác nhận | Từ chối, không đổi dữ liệu |
| P07-03 | Revision 3 gửi khi hiện tại là 4 | Yêu cầu đọc lại; không ghi xác nhận cũ |
| P07-04 | Tài xế được phân công đọc | Thấy đúng thông tin được phép xem |
| P07-05 | Timeout sau thao tác | Hiển thị chưa rõ và tra cứu theo thiết kế |

Các ID đầy đủ có tiền tố UAT-. Thêm ca accepted thắng cuộc đua trước xác nhận để bảo vệ RULE-P04-01; chỉ stale revision chưa kiểm được rule trạng thái.

## 4. Chuẩn bị dữ liệu có chủ đích

Tạo hai khách và hai tài xế giả để kiểm đúng/sai quyền. Giữ được trạng thái searching đủ lâu cho thao tác; có cách tạo revision mới hoặc đổi state có kiểm soát. Không dùng dữ liệu đã bị ca trước sửa mà không biết, vì kết quả sẽ khó lặp lại.

## 5. Ghi actual outcome độc lập

Actual cần mô tả điều quan sát, không chép expected. Ví dụ “UI báo đã xác nhận; reload mất nhãn; dữ liệu vẫn chưa confirmed” giúp điều tra hơn một chữ fail. Gắn build và thời điểm để tránh so nhầm sau khi môi trường cập nhật.

## 6. Test lại sau sửa

Sau khi sửa stale revision, chạy lại ca lỗi và xem phạm vi regression liên quan: đúng revision, accepted trước xác nhận, tài xế đọc. Không tự coi tất cả UAT cũ vẫn có hiệu lực với build mới; dùng impact analysis để xác định bằng chứng cần cập nhật.

## 7. Thực hành

Viết scenario cho cuộc đua giữa accepted và xác nhận điểm đón. Nêu thứ tự sự kiện có kiểm soát và outcome ở hai thứ tự khác nhau.

<details><summary>Gợi ý</summary><p>Nếu accepted được ghi trước, xác nhận không được chấp nhận theo đề xuất P04. Nếu xác nhận hợp lệ được ghi trước, kết quả đó tồn tại khi chuyến chuyển accepted. Cần kiểm nhất quán, không chỉ hai thao tác tuần tự không có cạnh tranh.</p></details>

## 8. Đọc tiếp

Đối chiếu [acceptance criteria P04](/articles/ba-acceptance-criteria-given-when-then). Tiếp: [phân loại phản hồi](/articles/ba-defect-change-cau-hoi-mo).

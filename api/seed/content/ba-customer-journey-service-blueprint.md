---
slug: ba-customer-journey-service-blueprint
title: "Customer journey và service blueprint"
summary: "Nối trải nghiệm khách tại điểm đón với hoạt động của tài xế, CSKH và hệ thống. Phân biệt journey với blueprint, gắn từng nhận định với nguồn và tránh vẽ hành trình giả như sự kiện đã quan sát."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-014, customer-journey, service-blueprint, toi-noi]
language: vi
---

Khách thấy “đã gửi ghi chú”. Tài xế chỉ nhìn ghim trên bản đồ. CSKH tìm lượt đặt xe bằng một thông tin khác. Mỗi màn hình riêng có thể trông hợp lý, trong khi toàn bộ hành trình vẫn có chỗ đứt giữa các bên.

Journey giúp nhìn từ mục tiêu và trải nghiệm của khách. Blueprint mở rộng xuống hoạt động cung cấp dịch vụ phía sau các điểm tiếp xúc. Khi dùng cả hai, BA có thể chỉ ra nơi cần xác minh hoặc thay đổi thay vì chỉ đề nghị sửa một màn hình.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-014 · Phần 2/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/ba-khao-sat-lay-mau-thien-lech">Khảo sát và lấy mẫu</a>. Đầu ra: journey và blueprint nháp có nguồn. Mục tiêu: nối trải nghiệm với hoạt động hỗ trợ, giữ rõ dữ kiện và giả định trong mô hình.</p>
</div>

## 1. Chọn hành trình và góc nhìn trước khi vẽ

Phạm vi bài là khách đặt xe tại A, từ lúc xác định điểm chờ tới lúc gặp tài xế hoặc tìm hỗ trợ khi không gặp được. Đây là góc nhìn khách; hành trình làm việc của tài xế là mô hình liên quan nhưng không được trộn thành một người dùng có mọi hành động.

[GOV.UK mô tả experience map như cách thể hiện hành động, suy nghĩ và cảm nhận qua thời gian](https://www.gov.uk/service-manual/user-research/creating-an-experience-map/). Với hồ sơ P02 còn ít nguồn, ta xây **bản nháp tổng hợp để tìm khoảng trống**, không tuyên bố đó là hành trình đại diện đã được xác thực.

Không ghép lời khách C-01, thao tác tài xế C-02 và quan sát khách C-03 thành một chuyến hoàn chỉnh. Có thể đặt chúng trên cùng bản đồ tổng hợp nếu mỗi ô ghi nguồn/case và người đọc biết chúng không cùng sự kiện.

## 2. Journey nháp: mỗi ô có mức chắc chắn

| Chặng | Mục tiêu/hành động cần hiểu | Bằng chứng P02 | Điều còn thiếu |
|---|---|---|---|
| Xác định nơi chờ | Khách gọi tên được vị trí | OBS-02: K-02 không biết tên cổng tương ứng trong app | Cách gọi tên ở địa điểm khác |
| Gửi thông tin điểm đón | Khách cung cấp thông tin để tài xế tìm | INT-01: K-01 kể đã ghi cổng Đông | Nội dung thực tế và thời điểm gửi trong C-01 |
| Chờ tài xế | Khách biết hai bên có cùng điểm đón không | INT-01 nói tài xế tới cổng Tây | Hai phía nhìn thấy gì, thay đổi nào xảy ra |
| Tìm hỗ trợ | Hai bên có cách xử lý khi không gặp | INT-03: CSKH có thể nhận nhiều cuộc gọi cho một ca | Dòng thời gian hỗ trợ của chính C-01 |
| Kết quả | Đón được hoặc kết thúc theo luồng nghiệp vụ | Baseline P01 có lượt huỷ ở mức tổng hợp | Kết quả cụ thể của các case và nguyên nhân |

Cột suy nghĩ/cảm xúc chỉ nên thêm khi có lời kể hoặc nguồn phù hợp. Chưa có dữ kiện thì ghi “chưa biết”, không vẽ đường cảm xúc giảm mạnh chỉ vì câu chuyện nghe có vẻ bực bội. Trải nghiệm có thể khác nhau giữa khách quen và khách mới, cần tách nhánh nếu khác biệt đó ảnh hưởng quyết định.

## 3. Blueprint thêm hoạt động cung cấp dịch vụ

[Nielsen Norman Group định nghĩa service blueprint qua quan hệ giữa các thành phần dịch vụ gắn với touchpoint của một hành trình](https://www.nngroup.com/articles/service-blueprints-definition/). Ta dùng các lớp: hành động khách, tương tác khách nhìn thấy, hoạt động hậu trường và hệ thống/quy trình hỗ trợ.

| Lớp | Gửi điểm đón | Chuẩn bị gặp nhau | Xử lý khó khăn |
|---|---|---|---|
| Khách | Nhập/chọn thông tin, cần xác minh chi tiết | Theo dõi và xác định nơi chờ | Liên hệ theo kênh đang có |
| Tương tác nhìn thấy | Thông tin phản hồi trên app, chưa xác minh C-01 | Thông tin tài xế/điểm đón khách thấy, cần kiểm | Trao đổi với CSKH, chi tiết chưa đủ nguồn |
| Hậu trường | Tiếp nhận/lưu thông tin: giả thuyết cần nhóm kỹ thuật xác nhận | Cung cấp thông tin cho tài xế; OBS-01 chỉ xác nhận C-02 đọc được sau thao tác | CSKH tìm lượt liên quan; INT-03 |
| Hỗ trợ | Hợp đồng dữ liệu và phiên bản, chưa kiểm | Điều kiện hiển thị, DOC-02 mới là thiết kế | Ticket và hướng dẫn DOC-01; chất lượng DATA-01 |

Ranh giới giữa tương tác nhìn thấy và hậu trường thường được gọi là line of visibility. Dù ứng dụng có giao diện, không phải mọi hoạt động của nó đều nằm phía trước: khách thấy kết quả, nhưng không thấy toàn bộ xử lý bên trong.

Blueprint này không khẳng định kiến trúc backend hiện có. Mọi hoạt động chưa có nguồn được gắn nhãn để nhóm kỹ thuật xác nhận; không suy ra service mới chỉ từ một ô trên bảng.

## 4. Tìm handoff và điểm đứt có thể kiểm tra

| Handoff | Rủi ro giả thuyết | Cách xác minh |
|---|---|---|
| Khách → thông tin lưu | Tên cổng khách dùng khác cách hệ thống nhận diện | Quan sát nhập liệu, đối chiếu nội dung được ghi nhận |
| Thông tin lưu → tài xế | Dữ liệu có nhưng không được nhận biết đúng lúc | Kiểm cùng case và phiên bản; quan sát không dẫn dắt |
| Cuộc gọi → ticket/lượt | Không nối được liên hệ với lượt cần xử lý | Review mapping và mẫu có quy tắc chọn rõ |

Điểm đứt trên bản đồ là vị trí cần tìm hiểu, chưa phải root cause. OBS-01 cho thấy thao tác kéo xuống ở một ca; muốn kết luận chỗ đặt ghi chú gây khó phải có thêm bằng chứng về bối cảnh, hành vi và tác động.

## 5. Từ as-is sang to-be mà vẫn giữ lịch sử

Để thử hướng dẫn cổng, tạo bản to-be riêng: khách nhận chỉ dẫn theo địa điểm; bộ phận vận hành có người cập nhật; tài xế nhận cùng tham chiếu. Ghi rõ đó là phương án thiết kế và chưa có bằng chứng tác động.

Mỗi thay đổi phía trước cần nhìn xuống hậu trường: ai tạo dữ liệu cổng, ai xác nhận còn đúng, nếu cổng đóng thì xử lý ra sao? Đây là cách blueprint giúp phát hiện chi phí vận hành trước khi nhóm chỉ ước lượng giao diện.

Không sửa bản as-is cho đẹp theo phương án mong muốn. Lưu phiên bản và câu hỏi để khi pilot có kết quả, nhóm biết điều đã thay đổi so với giả định ban đầu.

## 6. Những lỗi thường gặp

- Coi journey chỉ là danh sách màn hình, bỏ sót hoạt động ngoài app như tìm biển cổng.
- Vẽ một luồng thuận duy nhất rồi coi không có ngoại lệ.
- Điền cảm xúc và backend bằng suy đoán nhưng không gắn nhãn.
- Gộp nhiều case thành một “câu chuyện thật”.
- Blueprint kết thúc ở ô hệ thống mà không có người phụ trách việc hỗ trợ hoặc cập nhật dữ liệu.

Giải pháp không phải thêm thật nhiều lớp; chỉ thêm lớp giúp giải thích vấn đề hoặc quyết định đang cần làm rõ.

## 7. Bài tập và câu hỏi ôn tập

Vẽ journey năm chặng và blueprint bốn lớp cho hồ sơ P02 bằng bảng hoặc giấy. Mỗi nhận định thực tế có ID nguồn; ô chưa có dữ kiện ghi chưa biết. Chọn hai handoff và viết hoạt động xác minh.

**Câu hỏi:** Có thể viết “khách bực bội” vì họ gọi hỗ trợ nhiều lần không?

<details><summary>Gợi ý trả lời</summary><p>Chưa đủ. Nhiều cuộc gọi là hành vi/tín hiệu cần hiểu; cảm xúc cần lời kể hoặc bằng chứng phù hợp. Hơn nữa INT-03 chỉ nói một ca có thể gọi nhiều lần, chưa xác nhận một khách cụ thể đã làm vậy.</p></details>

## 8. Tham khảo và bài tiếp theo

- [GOV.UK — Creating an experience map](https://www.gov.uk/service-manual/user-research/creating-an-experience-map/).
- [NN/g — Service Blueprints: Definition](https://www.nngroup.com/articles/service-blueprints-definition/).

Bài tiếp theo: [Giải quyết yêu cầu xung đột giữa khách, tài xế và vận hành](/articles/ba-giai-quyet-yeu-cau-xung-dot).

---
slug: ba-trade-off-giao-hang-nhieu-diem
title: "Trade-off khi mở rộng Tới Nơi sang giao hàng nhiều điểm"
summary: "Phân tích giá trị, trạng thái giao từng điểm, giá và vận hành để so phương án mở rộng; tránh chỉ tăng số địa chỉ trên giao diện."
category: business-analysis
level: advanced
tags: [business-analysis, ba-063, trade-offs, multi-stop, toi-noi]
language: vi
---

Một cửa hàng muốn gửi ba đơn trong một chuyến. Thêm danh sách địa chỉ là phần dễ nhìn thấy; những câu hỏi khó hơn là ai trả tiền khi điểm thứ hai thất bại và điểm thứ ba có còn được giao không.

<div class="tkh-callout"><span class="tkh-callout-title">BA-063 · Phần 8/8 · Nâng cao</span><p>Đầu ra: option paper cho giao nhiều điểm. Đây là sáng kiến giả lập mới, chưa nằm trong baseline lab hoặc tự kế thừa rule một chuyến thành rule từng điểm.</p></div>

## 1. Chốt vấn đề cần giải quyết

Giả thuyết cửa hàng muốn giảm công điều phối và chi phí tổng; tài xế muốn hành trình rõ và được trả phù hợp; người nhận muốn biết thời gian đến. Ba lợi ích có thể xung đột. Cần phỏng vấn và dữ liệu về số điểm, khoảng cách, thời gian chờ và tỷ lệ không liên lạc được trước khi chọn giải pháp.

Không coi “nhiều điểm” là một phân khúc đồng nhất: ba điểm gần nhau khác nhiều điểm xa có cửa sổ nhận hàng hẹp.

## 2. So ba phương án

| Phương án | Giá trị tiềm năng | Đánh đổi |
|---|---|---|
| Giữ chuyến riêng, cải thiện thao tác tạo | Ít thay đổi trạng thái và đối soát | Có thể không giảm chi phí vận chuyển |
| Gộp tối đa ba điểm, thứ tự cố định | Phạm vi dễ giải thích hơn | Khó xử lý đổi thứ tự/chờ lâu |
| Tối ưu nhiều điểm động | Linh hoạt và có thể hiệu quả hơn | Phức tạp giá, ETA, điều phối và hỗ trợ |

Số ba chỉ là giới hạn đề xuất cho bài tập, không phải tối ưu đã được chứng minh. Lựa chọn phải dựa trên nhu cầu và năng lực chứ không chỉ thuật toán hấp dẫn.

## 3. Tách shipment, stop và trip

Trip là hành trình tài xế, stop là điểm dừng, shipment là đơn cần giao. Quan hệ giữa chúng cần được chốt: một stop có thể chứa nhiều shipment không, một shipment có thể chuyển chuyến không? Nếu gộp tất cả vào trip.status, hệ thống khó diễn tả giao thành công một phần.

State model mới phải mô tả cả cấp đơn và hành trình. Không tự thêm “partially_completed” vào SPEC rồi coi là baseline đã thống nhất.

## 4. Chạy tình huống khó trước

Điểm A giao thành công; B không liên lạc được; C đang chờ. Có tiếp tục C không, B được thử lại hay hoàn về cửa hàng, ai chấp thuận thay đổi? Cần trạng thái hàng, bằng chứng giao, giới hạn chờ, quyền quyết định và thông báo cho từng bên.

Nếu khách yêu cầu bỏ C sau khi A đã giao, không thể áp máy móc phí huỷ toàn chuyến cũ. Phải định nghĩa phần dịch vụ đã thực hiện và cách tính điều chỉnh mới.

## 5. Giá và tích hợp

Chốt báo giá cho cả tuyến hay từng đơn, hiệu lực khi đổi điểm, cách phân bổ phí và hoàn tiền khi giao một phần. P05 cung cấp cách phân tích idempotency/đối soát, không cung cấp sẵn chính sách hoàn cho multi-stop. Callback từng đơn cũng cần khoá liên kết và chống tác động lặp.

## 6. Phương án thử có điều kiện

Có thể nghiên cứu pilot thứ tự cố định, giới hạn khu vực, không thu hộ và có owner xử lý giao thất bại. Đây là phương án để review, chưa phải khuyến nghị chắc chắn. Điều kiện trước pilot gồm rule ngoại lệ, năng lực hỗ trợ, cách đo đúng hẹn từng stop và chi phí mỗi đơn hoàn tất.

Guardrail cần cả khách, tài xế và hàng hoá; tối ưu quãng đường nhưng làm tăng thời gian chờ không chắc tăng giá trị tổng.

## 7. Thực hành

Nộp option paper hai trang: nhu cầu, ba phương án, mô hình khái niệm, ca A/B/C, câu hỏi giá và điều kiện thử. Nêu bằng chứng nào khiến bạn chọn giữ chuyến riêng.

<details><summary>Gợi ý</summary><p>Nếu nhu cầu chủ yếu là nhập đơn nhanh, số đơn gộp thấp hoặc chi phí hỗ trợ giao một phần quá lớn, cải thiện tạo chuyến riêng có thể phù hợp hơn. Cần dữ liệu trước khi chấp nhận độ phức tạp mới.</p></details>

## 8. Đọc tiếp

Ôn [glossary](/articles/ba-domain-glossary-mo-hinh-du-lieu) và [hợp đồng hoàn tiền](/articles/ba-thuc-hanh-hop-dong-hoan-tien). Tiếp: [capstone hồ sơ BA](/articles/ba-capstone-bao-ve-ho-so-phan-tich).

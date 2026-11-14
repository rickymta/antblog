---
slug: ba-mapping-du-lieu-doi-soat
title: "Mapping dữ liệu và đối soát: nối đúng giao dịch, tìm đúng chênh lệch"
summary: "Thiết kế bảng ánh xạ và quy trình đối soát hoàn tiền, phân biệt lệch thời điểm với sai số tiền và xác định người xử lý ngoại lệ."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-037, mapping, reconciliation, toi-noi]
language: vi
---

Hai hệ thống cùng ghi 30.000 chưa chứng minh chúng đang nói về cùng một khoản hoàn tiền. BA cần chốt ý nghĩa, khoá nối và thời điểm trước khi so tổng.

<div class="tkh-callout"><span class="tkh-callout-title">BA-037 · Phần 5/8 · Trung bình</span><p>Đầu ra: mapping và bảng ngoại lệ cho refund đề xuất. Dữ liệu minh hoạ độc lập, không phải giao dịch thật hoặc API hiện có của Tới Nơi.</p></div>

## 1. Bắt đầu từ đơn vị bản ghi

Một dòng nội bộ là một refund, trong khi file đối tác có thể là một lần cập nhật refund. Nếu cộng tất cả các dòng trạng thái, một giao dịch processing rồi succeeded bị cộng hai lần. Hãy ghi grain — mỗi dòng đại diện cho gì — ở đầu tài liệu.

## 2. Mapping cần cả quy tắc

| Nội bộ | Đối tác giả lập | Quy tắc đề xuất |
|---|---|---|
| refundId | merchant_ref | Giữ nguyên chuỗi, không chuyển số |
| providerRef | transaction_ref | Khoá đối tác, không thay refundId |
| amountVnd | amount_minor | Chỉ nối khi đã xác nhận currency và đơn vị |
| succeeded | SUCCESS | Mapping có phiên bản và người phụ trách |
| occurredAt | event_time | Lưu múi giờ/offset; phân biệt receivedAt |

Không mặc định mọi đồng tiền đều có hai chữ số thập phân. Với bài tập chỉ dùng VND nguyên; hợp đồng đối tác phải xác nhận biểu diễn cụ thể. Trường chưa biết cách mapping được đưa vào ngoại lệ, không tự coi là failed.

## 3. Định nghĩa tập so sánh

Giả sử đối soát các refund tạo trong ngày D theo Asia/Ho_Chi_Minh, lấy trạng thái mới nhất đến D+1 lúc 09:00. Đây là cửa sổ đề xuất, cần thống nhất với đối tác và bộ phận vận hành. Lưu snapshot nguồn để lần chạy lại có thể giải thích vì sao kết quả khác.

Không trộn tổng refund tạo ngày D với tổng refund hoàn tất ngày D. Cả hai phép đo có ích nhưng trả lời hai câu hỏi khác nhau.

## 4. Phân loại chênh lệch

| Refund | Nội bộ | Đối tác | Phân loại và hành động |
|---|---|---|---|
| RF-A | processing 30.000 | succeeded 30.000 | Tra cứu, xác nhận rồi cập nhật theo quy tắc |
| RF-B | succeeded 20.000 | succeeded 25.000 | Lệch tiền, chuyển người phụ trách điều tra |
| RF-C | processing 10.000 | Không tìm thấy | Chưa đủ kết luận; kiểm tra khoá/cửa sổ |
| Không có | Không có | RF-D succeeded 5.000 | Giao dịch không khớp, không tạo bù tự động |

Đừng gộp mọi trường hợp thành “lỗi thanh toán”. Mỗi loại cần người chịu trách nhiệm, bằng chứng và đường phục hồi riêng.

## 5. Kiểm tổng và kiểm từng dòng

Tổng khớp vẫn có thể che một khoản dư 5.000 và một khoản thiếu 5.000. Kiểm tra số lượng, tổng theo currency, khoá trùng và chênh lệch của từng refund. Khi xử lý thủ công, giữ giá trị trước/sau, người xử lý và lý do; không sửa file nguồn để báo cáo đẹp hơn.

## 6. Thiết kế việc đóng ngoại lệ

Đề xuất bộ phận vận hành nhận RF-C, bộ phận kỹ thuật hỗ trợ truy vết, người có thẩm quyền tài chính xác nhận điều chỉnh tiền. SLA cụ thể còn mở. Một ticket chỉ đóng khi có bằng chứng outcome và trạng thái hai bên phù hợp, không vì đã gửi email hỏi đối tác.

## 7. Thực hành

Thêm hai dòng đối tác cho RF-A: processing lúc 10:00 và succeeded lúc 10:05. Viết quy tắc lấy bản ghi và cách xử lý nếu hai sự kiện cùng thời điểm nhưng mâu thuẫn.

<details><summary>Gợi ý review</summary><p>Dùng phiên bản hoặc thứ tự sự kiện do hợp đồng bảo đảm nếu có. Nếu chỉ có timestamp không đủ phân xử, tra cứu trạng thái có thẩm quyền và giữ ngoại lệ; không chọn ngẫu nhiên dòng cuối file.</p></details>

## 8. Nguồn và bài tiếp

Hồ sơ thực hành P05 trong repository là nguồn dữ kiện giả lập. Ôn [data dictionary](/articles/ba-data-dictionary-don-vi-null), rồi đọc [event và trạng thái chờ](/articles/ba-event-dong-bo-trang-thai-cho).

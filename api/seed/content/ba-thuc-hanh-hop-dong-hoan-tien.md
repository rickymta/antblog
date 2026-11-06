---
slug: ba-thuc-hanh-hop-dong-hoan-tien
title: "Thực hành BA: review hợp đồng hoàn tiền trước khi tích hợp"
summary: "Tổng hợp glossary, dictionary, state model, mapping và kịch bản cạnh tranh thành một gói review refund có quyết định mở và tiêu chí nghiệm thu."
category: business-analysis
level: advanced
tags: [business-analysis, ba-040, case-study, refund, toi-noi]
language: vi
---

Một response mẫu đẹp có thể che hợp đồng chưa xác định ai xử lý refund chờ ba ngày. Bài cuối phần 5 đặt các tài liệu cạnh nhau để tìm những khoảng trống đó.

<div class="tkh-callout"><span class="tkh-callout-title">BA-040 · Phần 5/8 · Nâng cao</span><p>Đầu ra: gói review đề xuất hoàn tiền. Hồ sơ P05 chứa dữ kiện và RF-01–RF-08; chưa có tính năng refund trong baseline lab được dùng cho series.</p></div>

## 1. Giới hạn tình huống

Chỉ xét payment đã xác nhận thu tiền, VND nguyên dương và hoàn một phần. Không suy rằng huỷ chuyến luôn có tiền để hoàn: lab cho phép huỷ thành công trong khi thu ví thất bại. Trước refund phải xác minh nguồn tiền đã thu, không dựa riêng vào trip.status.

Hoàn qua nhiều nhà cung cấp, chênh lệch tỷ giá và tranh chấp chưa nằm trong bài tập. Việc loại khỏi phạm vi phải được thể hiện trong hợp đồng và giao diện.

## 2. Chuẩn bị gói tối thiểu

| Thành phần | Câu hỏi cần giải đáp |
|---|---|
| Glossary/dictionary | Captured, reserved, refunded khác nhau thế nào? |
| Request/response | Ai được yêu cầu, input nào bắt buộc? |
| State model | Bằng chứng nào cho phép chuyển trạng thái? |
| Mapping/đối soát | Khóa nào nối giao dịch và ai xử lý lệch? |
| Ma trận lỗi/retry | Khi nào gửi lại, khi nào tra cứu? |
| Kịch bản nghiệm thu | Kết quả và tiền khả dụng sau mỗi bước? |

Mỗi quy tắc có ID ổn định và nguồn. Một ví dụ JSON không thay được điều kiện chuyển trạng thái.

## 3. Review đường đi bình thường

RF-01: payment P đã thu 100.000, chưa hoàn/giữ; yêu cầu 30.000 hợp lệ. Sau khi giữ, khả dụng là 70.000. Khi succeeded, 30.000 chuyển từ giữ sang đã hoàn; khả dụng vẫn 70.000, không trừ thêm lần nữa.

RF-02 gửi lại cùng key/payload: nhận kết quả theo hợp đồng, không thêm khoản hoàn. RF-03 dùng key đó với 40.000: bị từ chối theo rule đề xuất, không thay yêu cầu gốc.

## 4. Review đường đi không chắc chắn

RF-04 timeout rồi callback thành công: hiển thị chờ trước khi có bằng chứng; không tạo refund mới. RF-05 callback lặp: không ghi thêm tác động. RF-07 xác thực callback sai: không cập nhật tiền, giữ dấu điều tra phù hợp.

RF-08 processing cũ đến sau succeeded: không lùi trạng thái. Nếu sự kiện mâu thuẫn thay vì chỉ cũ, chuyển đối soát; đó là tình huống khác với bỏ qua bản lặp.

## 5. Review cạnh tranh

RF-06 gửi 60.000 và 50.000 đồng thời từ 100.000 khả dụng. Không bắt buộc yêu cầu nào thắng, nhưng không được cùng giữ thành công. Reviewer cần bằng chứng kiểm cạnh tranh từ triển khai sau này; chạy hai request tuần tự không chứng minh điều kiện này.

Nếu refund đầu bị rejected có bằng chứng, kiểm rule giải phóng tiền giữ. Khi chưa rõ kết quả, không coi số đó khả dụng chỉ vì client hết thời gian đợi.

## 6. Decision log trước khi sẵn sàng triển khai

Scope/retention key, mã lỗi, quyền duyệt refund, nguồn trạng thái có thẩm quyền, timeout/retry budget và SLA ngoại lệ còn mở. Gán owner và hạn cho từng mục. Những mục ảnh hưởng tiền và quyền cần giải quyết trước khi coi gói đủ rõ để triển khai.

Kết luận review có thể là “tiếp tục phân tích”. Hoàn thành tám bài học không đồng nghĩa hợp đồng được phê duyệt.

## 7. Bài nộp và tự đánh giá

Nộp một bộ liên kết gồm dictionary, bảng transition, mapping, tám kịch bản RF và decision log. Nhờ người khác tìm đường gây hoàn lặp hoặc vượt capturedAmount; sửa mọi tài liệu bị ảnh hưởng, không chỉ ghi chú ở cuối bài.

<details><summary>Điểm cần sửa trong câu “timeout thì hoàn lại lần nữa”</summary><p>Timeout chỉ nói người gọi chưa nhận kết quả. Cần giữ danh tính ý định, tra cứu hoặc retry theo hợp đồng và theo dõi ngoại lệ; tạo ý định mới có nguy cơ hoàn hai lần.</p></details>

## 8. Tổng kết phần 5

Ôn [glossary và mô hình dữ liệu](/articles/ba-domain-glossary-mo-hinh-du-lieu) khi các tên gọi chưa nhất quán. Phần tiếp theo chuyển sang quản lý backlog, phiên bản và tác động thay đổi. Hồ sơ P05 là đề xuất học tập, không phải bằng chứng tính năng đã triển khai.

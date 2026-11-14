---
slug: ba-nghiem-thu-co-dieu-kien-go-no-go
title: "Nghiệm thu có điều kiện và quyết định go/no-go"
summary: "Ghi rõ phạm vi chấp nhận, rủi ro tồn đọng, điều kiện trước phát hành và quyền dừng; tránh coi tỷ lệ test pass là quyết định tự động."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-052, acceptance, release-readiness, toi-noi]
language: vi
---

Một báo cáo có 99 ca pass và một ca fail không đủ để quyết định phát hành. Nếu ca fail cho phép người khác sửa điểm đón, tác động không thể được che bằng tỷ lệ 99%.

<div class="tkh-callout"><span class="tkh-callout-title">BA-052 · Phần 7/8 · Trung bình</span><p>Đầu ra: decision record cho bản triển khai giả lập. Không có go/no-go thật được ký trong bài; các ngưỡng và quyền phải do dự án cụ thể xác định.</p></div>

## 1. Tách hai quyết định

Nghiệm thu xác nhận phạm vi yêu cầu có được chấp nhận theo bằng chứng hay không. Quyết định phát hành còn xét vận hành, triển khai, hỗ trợ, đo lường và rủi ro. Một gói UAT được chấp nhận chưa đồng nghĩa hệ thống đã đủ điều kiện chạy cho mọi người dùng.

## 2. Đọc báo cáo theo rủi ro

| Mục giả lập | Tình trạng | Ý nghĩa |
|---|---|---|
| Happy path | Pass theo build B | Bằng chứng trong điều kiện đã chạy |
| Sai actor | Fail | Không đáp ứng rule quyền, chặn theo tiêu chí đề xuất |
| Câu chữ hướng dẫn | Có lỗi nhỏ | Xem tác động hiểu sai, không chỉ hình thức |
| Runbook CS | Chưa diễn tập | Readiness chưa có bằng chứng |

Không cộng blocked vào passed. Với ca chưa chạy, ghi rõ giới hạn mức tự tin và kế hoạch bổ sung.

## 3. Điều kiện phải hoàn thành trước phát hành

Ví dụ: sửa lỗi quyền, chạy lại ca liên quan trên build xác định, diễn tập hỗ trợ và xác nhận dữ liệu đo. Gán owner, bằng chứng hoàn thành và người kiểm. Khi điều kiện chưa đạt, “chấp nhận có điều kiện” không phải giấy phép bỏ qua điều kiện.

Nếu build thay đổi sau review, phải đánh giá lại phạm vi bằng chứng bị ảnh hưởng.

## 4. Tồn đọng được chấp nhận sau phát hành

Một lỗi câu chữ không làm hiểu sai có thể được người có thẩm quyền chấp nhận tạm, với hạn sửa và owner. Ghi tác động, workaround, phạm vi người dùng và trigger xem lại. Không xếp lỗi quyền hoặc tính đúng tiền vào loại này chỉ vì lịch gấp.

Hai danh sách trước/sau phát hành cần tách rõ; nếu dùng chung nhãn “follow-up”, nhóm dễ hiểu nhầm việc nào được phép để lại.

## 5. Điều kiện dừng và phương án phục hồi

Đề xuất pilot cần xác định ai theo dõi, tín hiệu nào dừng và cách đưa người dùng về luồng phù hợp. Tắt tính năng không xoá dữ liệu đã ghi hoặc thông báo đã gửi. Cần cách xử lý giao dịch đang dở, giữ dấu vết và hỗ trợ khách.

## 6. Decision record mẫu

Ghi build/phạm vi, tóm tắt bằng chứng, rủi ro còn lại, quyết định, điều kiện, người có thẩm quyền, thời điểm và mốc review tiếp. Trong tình huống còn lỗi sai actor, khuyến nghị no-go cho phạm vi bị ảnh hưởng. Nếu tách phạm vi khác, phải chứng minh nó độc lập và đáp ứng điều kiện riêng.

## 7. Thực hành

Viết lại câu “UAT pass, cho chạy rồi sửa sau” thành một quyết định có thể kiểm tra. Nếu không đủ dữ liệu, liệt kê bằng chứng còn thiếu thay vì tự điền người ký.

<details><summary>Điểm tự kiểm</summary><p>Người khác phải biết chính xác bản nào, ai được dùng, điều gì chưa đạt, ai chấp nhận rủi ro, hạn nào và trường hợp nào phải dừng. Thiếu một điều trọng yếu thì quyết định chưa đủ rõ.</p></details>

## 8. Đọc tiếp

Liên hệ [phụ thuộc xuyên đội](/articles/ba-phu-thuoc-yeu-cau-xuyen-doi). Tiếp: [yêu cầu chuyển đổi](/articles/ba-transition-migration-dao-tao-van-hanh).

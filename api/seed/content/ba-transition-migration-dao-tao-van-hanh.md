---
slug: ba-transition-migration-dao-tao-van-hanh
title: "Transition requirement: dữ liệu, đào tạo và vận hành khi chuyển đổi"
summary: "Lập yêu cầu chuyển đổi tạm thời để đưa giải pháp vào sử dụng, với kiểm tra dữ liệu cũ, diễn tập hỗ trợ và điều kiện kết thúc workaround."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-053, transition, migration, toi-noi]
language: vi
---

Tính năng đã xong nhưng CS không biết nhãn “đã xác nhận” có nghĩa gì, còn chuyến cũ không có revision. Đó là khoảng trống chuyển đổi, không nhất thiết là thêm một màn hình mới.

<div class="tkh-callout"><span class="tkh-callout-title">BA-053 · Phần 7/8 · Trung bình</span><p>Đầu ra: transition checklist cho đề xuất P04. Các trường và phương án migration trong bài cần thiết kế; chưa có migration thật được chạy.</p></div>

## 1. Yêu cầu tạm thời cũng cần người phụ trách

Transition requirement mô tả điều cần để đi từ hiện trạng sang trạng thái đích: dữ liệu, đào tạo, hỗ trợ hoặc vận hành song song. Một số việc sẽ kết thúc sau chuyển đổi, nhưng vẫn phải có tiêu chí hoàn thành và người chịu trách nhiệm.

Phân biệt với yêu cầu lâu dài như kiểm tra quyền ở mọi lần xác nhận. Đào tạo đợt đầu là hoạt động chuyển đổi; quy trình đào tạo người mới có thể là năng lực vận hành lâu dài.

## 2. Không tự tạo lịch sử xác nhận

Chuyến cũ không có bằng chứng khách xác nhận thì không được backfill confirmed = true chỉ để giao diện đẹp. Đề xuất giữ trạng thái chưa biết/không áp dụng cho dữ liệu legacy và hiển thị phù hợp. Ý nghĩa null phải nằm trong dictionary, không để từng đội suy diễn.

Cũng cần chốt chuyến đang hoạt động lúc bật tính năng có được tham gia hay chỉ dành cho chuyến mới. Quyết định này ảnh hưởng cả UI, đo lường và hỗ trợ.

## 3. Checklist có bằng chứng

| Việc | Owner đề xuất | Bằng chứng hoàn thành |
|---|---|---|
| Chọn cohort chuyển đổi | Nghiệp vụ + kỹ thuật | Rule trước/sau mốc rõ |
| Xử lý dữ liệu cũ | Đội dữ liệu | Mapping, số lượng và mẫu đối chiếu |
| Đào tạo CS | Vận hành | Diễn tập giải thích ca stale/timeout |
| Thiết lập giám sát | Kỹ thuật + analytics | Sự kiện mẫu đi hết pipeline |
| Chuẩn bị phục hồi | Release owner | Diễn tập và người trực xác nhận |

Đây là phân công giả lập, cần thay bằng người phụ trách có thẩm quyền trong dự án thật.

## 4. Kiểm tra migration theo ý nghĩa

Số dòng trước/sau bằng nhau chưa chứng minh dữ liệu đúng. Kiểm tra khoá, quan hệ, đơn vị, trạng thái và mẫu ngoại lệ. Nếu có script sửa dữ liệu, cần phương án chạy lại không tạo lặp và dấu vết các bản ghi được tác động. BA xác định quy tắc và bằng chứng nghiệp vụ; kỹ thuật thiết kế cách thực hiện.

## 5. Đào tạo bằng tình huống

CS nhận một ca “app báo chờ nhưng khách đã bấm xác nhận”. Họ phải tra đúng chuyến, phân biệt đã lưu với chưa rõ và giải thích bước tiếp. Một buổi trình chiếu có danh sách điểm danh chưa chứng minh nhân viên làm được việc.

## 6. Kết thúc workaround

Nếu phải tra cứu thủ công trong pilot, hãy ghi giới hạn công suất, quyền truy cập, thời hạn và điều kiện chuyển sang công cụ chính thức. Khi hết thời hạn mà công cụ chưa sẵn sàng, phải review lại; không mặc định kéo dài vô hạn. Việc tắt tính năng cũng cần giữ hỗ trợ cho giao dịch đã phát sinh.

## 7. Thực hành

Viết ba yêu cầu chuyển đổi cho chuyến legacy chưa có pickupRevision. Với mỗi yêu cầu, ghi tiêu chí, người phụ trách và cách kiểm tra trước/sau.

<details><summary>Gợi ý</summary><p>Không suy ra xác nhận quá khứ; phân biệt legacy trong UI/dữ liệu đo; kiểm một mẫu chuyến cũ và mới qua hành trình đọc. Cách biểu diễn cụ thể cần được chốt cùng dictionary và contract.</p></details>

## 8. Đọc tiếp

Ôn [data dictionary](/articles/ba-data-dictionary-don-vi-null). Tiếp: [dashboard và metric](/articles/ba-dashboard-do-ket-qua-sau-phat-hanh).

---
slug: ba-domain-glossary-mo-hinh-du-lieu
title: "Domain glossary và mô hình dữ liệu khái niệm"
summary: "Thống nhất booking, trip, payment và refund trước khi bàn bảng dữ liệu; xác định quan hệ, nguồn chuẩn và khác biệt giữa lab với mô hình đích."
category: business-analysis
level: beginner
tags: [business-analysis, ba-033, domain, data, toi-noi]
language: vi
---

CSKH nói “đơn đã thanh toán”, developer thấy trip completed còn payment failed. Hai người có thể đang dùng một từ cho hai trạng thái khác nhau. Glossary là nơi nhóm chốt nghĩa theo bối cảnh, không chỉ dịch từ tiếng Anh.

<div class="tkh-callout"><span class="tkh-callout-title">BA-033 · Phần 5/8 · Cơ bản</span><p>Đọc trước phần 4. Đầu ra: glossary và mô hình quan hệ khái niệm. Refund trong P05 là đề xuất mới, không phải tính năng lab.</p></div>

## 1. Ghi nghĩa và ranh giới

| Thuật ngữ | Nghĩa cần phân biệt |
|---|---|
| Booking | Ý định/ghi nhận đặt dịch vụ trong mô hình nghiệp vụ đích |
| Trip | Vòng đời thực hiện chuyến; lab dùng trips làm đối tượng chính |
| Payment | Việc thu tiền và kết quả riêng với trạng thái chuyến |
| Refund | Ý định hoàn tiền từ một giao dịch đã thu, đề xuất ở P05 |

Không bắt lab phải có bốn bảng chỉ vì glossary có bốn khái niệm. Mô hình khái niệm chưa quyết định thiết kế lưu trữ.

## 2. Định danh khác với nhãn hiển thị

Trip ID dùng nối dữ liệu; mã hiển thị có thể phục vụ tra cứu. Tên khách hoặc số điện thoại không tự là khoá nghiệp vụ ổn định. Ghi trường nào định danh đối tượng, ai cấp và có thay đổi không.

## 3. Quan hệ phải có cardinality

Trong thiết kế refund, một payment có thể có nhiều lần hoàn từng phần; mỗi refund thuộc một payment. Quan hệ này đòi invariant tổng hoàn/giữ chỗ không vượt số đã thu. “Trip có payment” chưa đủ nếu hệ thống có nhiều loại khoản thu như cước và phí huỷ.

Hỏi trước về miền nghiệp vụ, rồi mới vẽ 1–n. Đừng suy quan hệ từ một mẫu có đúng một record.

## 4. Chọn source of truth theo thuộc tính

Trạng thái chuyến do chủ thể nghiệp vụ chuyến quản lý; kết quả giao dịch ngoài cần nguồn thanh toán/đối soát phù hợp. Một dashboard bản sao không tự có quyền sửa kết quả chuẩn.

Ghi owner, cách lấy dữ liệu và độ trễ chấp nhận được. “Nguồn chuẩn” không có nghĩa nguồn đó không bao giờ lỗi; khi mâu thuẫn cần cơ chế xác minh.

## 5. Đồng nghĩa và đa nghĩa

Ghi “huỷ” có thể là khách kết thúc chuyến hoặc tài xế rời để tìm lại. Dùng actor + event + outcome để phân biệt. Với refund, “đã yêu cầu” khác “đã hoàn thành”; glossary cần ví dụ và phản ví dụ.

## 6. Liên kết với yêu cầu

Mỗi rule nhắc payment/refund nên dùng ID glossary. Khi nghĩa đổi, tìm API, báo cáo, AC và hướng dẫn hỗ trợ dùng từ đó. Không sửa nghĩa trong glossary mà giữ báo cáo cũ như không ảnh hưởng.

## 7. Thực hành

Viết glossary cho “phí”, “đã thu” và “đã hoàn”; thêm nguồn, owner cần xác nhận, ví dụ trái nghĩa. Vẽ payment 1–n refund và giải thích invariant.

<details><summary>Vì sao trip completed không chứng minh paid?</summary><p>Baseline lab cho phép hoàn thành chuyến dù thu ví thất bại. Hai vòng đời phản ánh hai kết quả khác nhau.</p></details>

## 8. Nguồn và bài tiếp

Nguồn: SPEC lab R-PAY và `examples/toi-noi-ba/p05-tich-hop.md`. Tiếp theo: [Data dictionary](/articles/ba-data-dictionary-don-vi-null).

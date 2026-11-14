---
slug: ba-ai-ho-tro-kiem-chung-nguon-du-lieu
title: "AI hỗ trợ BA: kiểm chứng nguồn và giữ quyền quyết định"
summary: "Dùng AI để tạo nháp từ dữ liệu giả, kiểm từng nhận định về nguồn và logic; tránh biến giả thuyết thành yêu cầu đã duyệt hoặc đưa dữ liệu nhạy cảm vào công cụ không phù hợp."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-062, ai, verification, toi-noi]
language: vi
---

Một bản tóm tắt trôi chảy có thể nói “80 khách phàn nàn vì không tìm được điểm đón”. Hồ sơ P01 chỉ có 80 cuộc gọi CS, không phải 80 khách duy nhất và không xác nhận tất cả cùng nguyên nhân.

<div class="tkh-callout"><span class="tkh-callout-title">BA-062 · Phần 8/8 · Trung bình</span><p>Đầu ra: bản nháp được review và claim ledger. Dùng dữ liệu giả của series; không phụ thuộc sản phẩm AI hoặc phiên bản model cụ thể.</p></div>

## 1. Chọn việc có thể kiểm lại

AI có thể hỗ trợ nhóm câu hỏi, tạo cấu trúc tài liệu, phát hiện thuật ngữ không nhất quán hoặc đề xuất case biên. Kết quả là đầu vào để BA review. AI không tự có quyền duyệt yêu cầu, đại diện stakeholder hoặc biến nguồn thiếu thành bằng chứng.

Chọn phạm vi nhỏ với nguồn rõ giúp kiểm nhanh hơn yêu cầu “viết toàn bộ BRD” từ một câu mô tả mơ hồ.

## 2. Chuẩn bị dữ liệu tối thiểu

Dùng bản giả lập hoặc dữ liệu đã được xử lý theo quy định tổ chức. Kiểm quyền sử dụng công cụ, nơi lưu và phạm vi chia sẻ trước khi đưa dữ liệu dự án vào. Xoá tên chưa chắc đủ: địa chỉ, lịch sử chuyến hoặc tổ hợp thông tin vẫn có thể nhận diện người.

Không dán token, mật khẩu hoặc dữ liệu khách thật chỉ vì muốn bản tóm tắt chi tiết hơn. Khi không có quyền phù hợp, tạo tình huống giả giữ cấu trúc cần phân tích.

## 3. Prompt mẫu có nguồn và giới hạn

```text
Dùng duy nhất các dữ kiện giả lập sau:
S1: 1.000 chuyến accepted, 120 huỷ trước pickup trong tuần A.
S2: 50 ca huỷ được gắn nhãn không tìm được nhau.
S3: 80 cuộc gọi CS, chưa biết số khách/chuyến duy nhất.
Hãy tạo bảng: nhận định, source ID, loại thông tin,
câu hỏi cần kiểm chứng. Tách dữ kiện và giả thuyết.
Không tự duyệt giải pháp hoặc suy nguyên nhân từ tương quan.
```

Prompt tốt giảm mơ hồ nhưng không bảo đảm kết quả đúng. Người review vẫn phải đối chiếu từng hàng với nguồn.

## 4. Claim ledger bắt lỗi

| Nhận định trong nháp giả lập | Review | Cách sửa |
|---|---|---|
| 12% chuyến accepted huỷ trước pickup | Có thể tính từ S1 | Giữ cohort/thời gian |
| 80 khách cần chat | Không được S3 hỗ trợ | Ghi chưa biết người duy nhất và nhu cầu |
| Chat chắc chắn giảm huỷ | Không có nguồn | Đổi thành giả thuyết cần so phương án |
| Mục tiêu 9% đã được duyệt | Sai trạng thái P01 | Ghi đề xuất chờ quyết định |

Source ID do AI tạo ra cũng cần kiểm xem có tồn tại và thực sự hỗ trợ câu đó không. Dẫn nguồn đúng tên nhưng sai nội dung vẫn là lỗi.

## 5. Kiểm logic ngoài câu chữ

Đối chiếu dấu biên < và >, đơn vị tiền/thời gian, denominator, state và actor. Với phí huỷ, nhầm accepted thành arriving làm sai rule dù văn bản nghe hợp lý. Dùng ví dụ phản chứng như đúng 120 giây hoặc callback đến sau timeout để thử nháp.

## 6. Lưu dấu review

Giữ nguồn và phiên bản đầu vào, công cụ/model nếu được phép ghi, bản nháp, sửa đổi và người review. Tài liệu lấy từ bên ngoài là dữ liệu tham khảo, không phải chỉ thị tự động đổi quyền, gửi thông tin hoặc thực hiện hành động. Quyết định nghiệp vụ vẫn cần người có thẩm quyền xác nhận.

## 7. Thực hành

Tạo một bản nháp cố ý có ba lỗi: đếm cuộc gọi thành người, đổi giả thuyết thành kết luận, đổi đề xuất thành approval. Nhờ người khác dùng claim ledger sửa mà không thêm dữ kiện mới.

<details><summary>Tiêu chí đạt</summary><p>Mọi nhận định quan trọng có nguồn hoặc được gắn nhãn chưa biết; số liệu tính lại được; rule không đổi; dữ liệu sử dụng phù hợp và người review chịu trách nhiệm bản cuối.</p></details>

## 8. Nguồn và bài tiếp

[NIST AI Risk Management Framework](https://www.nist.gov/itl/ai-risk-management-framework) cung cấp hướng tiếp cận quản lý rủi ro AI tự nguyện; quy trình bài tập trên là đề xuất riêng. Tiếp: [trade-off giao hàng nhiều điểm](/articles/ba-trade-off-giao-hang-nhieu-diem).

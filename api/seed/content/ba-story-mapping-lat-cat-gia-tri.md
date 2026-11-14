---
slug: ba-story-mapping-lat-cat-gia-tri
title: "Story mapping và chia lát cắt có giá trị"
summary: "Sắp story theo hành trình điểm đón và chia đợt có kết quả người dùng kiểm chứng được, thay vì phát hành từng tầng database, API rồi giao diện."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-028, story-mapping, mvp, toi-noi]
language: vi
---

Kế hoạch ghi tuần đầu database, tuần sau API, cuối cùng giao diện. Đó có thể là thứ tự công việc kỹ thuật, nhưng sau tuần đầu khách chưa thể xác nhận thông tin nào. Story map giúp nhóm nhìn thấy phần giá trị được hoàn thành xuyên suốt hành trình.

<div class="tkh-callout"><span class="tkh-callout-title">BA-028 · Phần 4/8 · Trung bình</span><p>Đọc trước: <a href="/articles/ba-use-case-xac-nhan-diem-don">Use case</a>. Đầu ra: story map và phạm vi lát cắt đầu. Toàn bộ chức năng P04 là đề xuất, chưa triển khai.</p></div>

## 1. Chọn backbone theo hoạt động người dùng

Backbone của tình huống: xem thông tin → nhận biết sai khác → xác nhận → hiểu kết quả. Không đặt database, API và frontend làm các chặng người dùng. Chúng là phần thực hiện bên dưới mỗi lát cắt.

Story map không thay roadmap hoặc ước lượng. Story map giúp đặt câu hỏi liệu phạm vi dự định có kết thúc ở một kết quả người dùng có thể kiểm chứng không.

## 2. Xếp công việc dưới hành trình

| Đợt đề xuất | Xem | Xử lý sai khác | Xác nhận | Nhận kết quả |
|---|---|---|---|---|
| Lát cắt 1 | Đọc bản chuẩn của lượt mình | Nhận biết revision/state đã đổi | Xác nhận r hiện hành khi searching | Thành công đúng r hoặc trạng thái chưa biết/lỗi rõ |
| Lát cắt sau | Hướng dẫn điểm đón phong phú hơn | Sửa theo phạm vi được duyệt | Quy trình mở rộng nếu cần | Hỗ trợ/tra cứu lịch sử theo quyền |

Lát cắt 1 chưa xử lý mọi nhu cầu sản phẩm, nhưng phải có quyền truy cập, điều kiện ghi và xử lý lỗi cần thiết cho phạm vi đó. “MVP” không phải lý do bỏ kiểm quyền hoặc báo thành công giả khi mất mạng.

## 3. Chia theo khác biệt có ý nghĩa

Có thể thu hẹp nhóm người dùng, phạm vi trạng thái, loại thông tin hoặc kênh thử. Với P04, chỉ cho phép ghi khi chuyến đang searching là một giới hạn rõ. Giới hạn này giảm vấn đề phối hợp sau khi nhận chuyến nhưng phải được giải thích cho khách khi state đổi.

Không chia “lưu thành công trước, kiểm revision sau” nếu việc thiếu kiểm revision khiến khách xác nhận dữ liệu cũ. Chia phải giữ hành vi cốt lõi nhất quán, không tạo một đợt biết trước là sai nghiệp vụ.

## 4. Giữ phụ thuộc kỹ thuật trong kế hoạch

Mỗi lát cắt có task dữ liệu, hợp đồng, UI và kiểm thử. Một task chuẩn bị môi trường có thể chưa tạo giá trị trực tiếp nhưng vẫn cần; ghi nó là enabler với lý do, đầu ra và phụ thuộc.

Lab chưa có điểm đón nên không thể ước lượng như thể chỉ thêm một nút. Nhóm cần xem dữ liệu chuẩn, quyền, phiên bản và hợp đồng tương tác. BA làm rõ phạm vi; developer ước lượng dựa trên thông tin đó.

## 5. Đặt tiêu chí hoàn thành cho một lát cắt

Demo một khách chủ lượt đọc r4, xác nhận thành công; demo r4 đã cũ; demo accepted thắng trước thao tác ghi; demo người sai quyền và mất phản hồi. Nếu chỉ demo luồng thuận, phạm vi được tuyên bố hoàn tất có thể rộng hơn phần kiểm chứng được.

Các tiêu chí này chưa chứng minh giá trị kinh doanh. Sau triển khai thích hợp còn cần đo khả năng hiểu, mức độ sử dụng và kết quả theo mục tiêu; không lấy số story hoàn thành thay cho chỉ số sản phẩm.

## 6. Tránh ưu tiên bằng vị trí thẻ

Thẻ ở hàng trên không mặc nhiên là Must. Ghi lý do ưu tiên: rủi ro nào, nhu cầu nào, điều gì học được và phụ thuộc gì. Nếu bản đồ quá lớn, chốt một hành trình và một quyết định phát hành trước rồi mở rộng sau.

## 7. Bài tập và tự kiểm tra

Chia gói P04 thành hai lát cắt; mỗi lát có mục tiêu, phần nằm ngoài phạm vi, phụ thuộc và ví dụ demo. Giải thích vì sao kiểm quyền/revision phải đi cùng hành vi ghi.

**Câu hỏi:** Khi API xong, có được coi story của khách là hoàn thành không?

<details><summary>Gợi ý trả lời</summary><p>Chỉ khi kết quả đã thoả phạm vi story và kênh sử dụng được xác định. Với story khách sử dụng app, riêng API thường mới là một phần công việc.</p></details>

## 8. Tham khảo và bài tiếp theo

- Hồ sơ P04: `examples/toi-noi-ba/p04-goi-yeu-cau.md`; story map là đề xuất phân chia riêng cho tình huống.
- [User story](/articles/ba-user-story-vai-tro-nhu-cau-gia-tri).

Bài tiếp theo: [Wireframe và prototype để kiểm tra hiểu biết](/articles/ba-wireframe-prototype-kiem-tra-hieu-biet).

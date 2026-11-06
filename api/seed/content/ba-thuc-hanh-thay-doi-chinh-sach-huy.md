---
slug: ba-thuc-hanh-thay-doi-chinh-sach-huy
title: "Thực hành: thay đổi chính sách huỷ trên nhiều luồng"
summary: "Phân tích đề xuất mở rộng khoảng miễn phí huỷ từ hai lên ba phút: tác động tới rule, quy trình, trạng thái, thông báo, version và replay. Tạo hồ sơ thay đổi có bằng chứng và tiêu chí review."
category: business-analysis
level: advanced
tags: [business-analysis, ba-024, change-impact, case-study, toi-noi]
language: vi
---

“Cho khách thêm một phút miễn phí, chắc chỉ sửa một con số.” Đề xuất nghe nhỏ nhưng lập tức nảy sinh câu hỏi: áp dụng cho chuyến đang chạy hay chỉ lượt nhận mới, tài xế đã chuyển arriving có được hưởng không, gửi lại yêu cầu huỷ cũ có tính theo chính sách mới không?

Bài thực hành khép phần 3 yêu cầu lập hồ sơ tác động cho CR-BA-01. Bạn không sửa lab hoặc tự duyệt chính sách; bạn làm cho thay đổi đủ rõ để người có thẩm quyền xem xét và nhóm kỹ thuật hiểu phần cần thực hiện.

<div class="tkh-callout"><span class="tkh-callout-title">BA-024 · Phần 3/8 · Nâng cao · Thực hành tổng hợp</span><p>Đọc BA-017–BA-023. Đầu ra: impact map, mô hình/bảng thay đổi, bộ case và decision log. Dành khoảng 90 phút làm bài. Nguồn và ca mẫu ở <code>examples/toi-noi-ba/p03-mo-hinh-quy-trinh.md</code>.</p></div>

## 1. Đề bài và baseline

CR-BA-01 đề xuất đổi khoảng miễn phí khi khách huỷ ở **accepted** từ dưới hai phút sang dưới ba phút kể từ acceptedAt. Chưa có quyết định phê duyệt. Giữ nguyên trạng thái được phép huỷ, arriving, ngưỡng miễn do trễ, mức tiền, tài xế rời chuyến và xử lý thanh toán.

Không lấy target giảm huỷ 9% của P01 làm bằng chứng chính sách này hiệu quả. Miễn phí lâu hơn có thể thay đổi hành vi khách/tài xế theo nhiều hướng; cần giả thuyết giá trị và cách đánh giá riêng trước khi triển khai.

## 2. Khoanh delta và phần phải giữ nguyên

| Hạng mục | Baseline | Đề xuất | Kiểm tra |
|---|---|---|---|
| Khoảng miễn ở accepted | elapsed < 120 giây | elapsed < 180 giây | 119/120/179/180 giây |
| arriving | Có phí cơ sở, xét miễn do trễ | Không đổi | arriving dù elapsed < 180 |
| Mốc trễ | Lớn hơn acceptedAt + ETA + 10 phút | Không đổi | Đúng mốc và vượt mốc |
| Tài xế rời | Về searching, không thu khách | Không đổi | Reset mốc nhận/tìm theo SPEC |
| Replay | Trả kết quả đã xử lý | Không đổi | Gửi lại sau khi version thay đổi |
| Ví declined | cancelled/failed, không retry | Không đổi | Không phục hồi chuyến |

Đơn vị giây ở đây để đọc case; khi triển khai phải giữ độ chính xác của phép so sánh và thống nhất thời điểm dùng để quyết định. Không làm tròn 179,9 thành 180 trước khi so nếu chưa có chính sách đó.

## 3. Lập impact map theo nơi dùng chính sách

| Nơi bị tác động | Cần xem lại | Câu hỏi phải chốt |
|---|---|---|
| Rule catalog | G và version nguồn | Ai duyệt, hiệu lực lúc nào? |
| Decision table | D2; các case 120–179 giây | Giữ ưu tiên lý do miễn thế nào? |
| Process/API | Chọn policy, replay và điểm tính phí | Có tính lại kết quả cũ không? |
| Dữ liệu | Dấu vết version và dữ kiện quyết định | Có lưu đủ để giải thích khiếu nại? |
| Giao diện/nội dung | Thông báo khoảng miễn và arriving | Cách tránh hứa miễn khi không còn đủ điều kiện? |
| CSKH | Hướng dẫn giải thích theo case/version | Người hỗ trợ tra nguồn nào? |
| Kiểm thử/đo lường | Biên mới và regression | Có phân biệt cohort/version khi đọc kết quả? |

Không chỉ tìm chuỗi số “2” trong code/tài liệu. Có nơi dùng diễn giải “huỷ sớm”, hoặc 120.000 mili giây; tác động cần lần theo ý nghĩa và nguồn, không chỉ ký tự.

## 4. Phân tích hiệu lực và chuyến đang chạy

Hai phương án minh hoạ: dùng chính sách hiện hành tại lúc huỷ; hoặc gắn version tại một mốc đã xác định, chẳng hạn lần nhận chuyến. Mỗi phương án có đánh đổi về trải nghiệm, dữ liệu và khả năng giải thích.

Nếu chọn gắn version theo lần nhận, phải nói rõ tài xế rời rồi người khác nhận có chọn version mới không. Vì baseline xoá acceptedAt/ETA lúc rời chuyến, không thể dùng mốc nhận cũ để tính khoảng miễn cho lần nhận mới. Việc thêm trường policyVersion là thiết kế đề xuất, chưa có sẵn trong SPEC.

Hồ sơ có thể khuyến nghị một phương án nhưng giữ trạng thái chờ phê duyệt. Không tự chọn giờ hiệu lực hoặc tuyên bố điều khoản đã được khách chấp nhận; đó là quyết định cần người phụ trách phù hợp xem xét.

## 5. Bộ case review và đáp án mẫu

Các ca độc lập dùng xe máy, acceptedAt 10:00:00, ETA 5 phút, khách đúng quyền:

| Ca | State / thời điểm | v1 | v2 đề xuất |
|---|---|---|---|
| A | accepted / 10:01:59 | Miễn | Miễn |
| B | accepted / 10:02:00 | 10.000 đ | Miễn |
| C | accepted / 10:02:59 | 10.000 đ | Miễn |
| D | accepted / 10:03:00 | 10.000 đ | 10.000 đ |
| E | arriving / 10:01:30 | 10.000 đ | 10.000 đ |
| F | arriving / 10:15:00 | 10.000 đ | 10.000 đ |
| G | arriving / 10:15:01 | Miễn do trễ | Miễn do trễ |
| H | picked_up / 10:03:00 | Từ chối | Từ chối |

Thêm ca searching, ô tô, driver_cancel, ví failed và replay từ hồ sơ P03. Với mỗi case phải nêu phí **và** lý do, transition và payment outcome nếu liên quan; chỉ ghi số tiền chưa đủ xác minh toàn bộ hành vi.

Nếu yêu cầu đã xử lý dưới v1 có phí, gửi lại đúng khoá và đúng nội dung sau khi v2 có hiệu lực vẫn trả kết quả cũ theo R-IDEM. Đây không phải case “v2 tính sai”, mà là giữ hợp đồng replay. Muốn xử lý điều chỉnh phí sau đó cần nghiệp vụ riêng, không lén đổi kết quả idempotency.

## 6. Gói đề xuất và điều kiện được phép triển khai

Nộp: baseline, mô tả delta, bảng quyết định v2, process có điểm chọn version/replay, transition/effect được giữ, impact log và bộ case. Ghi quyết định còn mở về hiệu lực, nhận lại, dữ liệu version, nội dung và quyền phê duyệt.

Điều kiện đề xuất trước triển khai: rule được chốt; dữ liệu đủ giải thích version; UI/API/hỗ trợ cùng hiểu; kiểm case thay đổi và regression; có owner theo dõi. Đây là tiêu chí review của bài tập, không phải lời xác nhận hệ thống đang đạt.

Rollback cũng cần phân tích: dừng áp dụng cho lượt mới không đồng nghĩa sửa lại lịch sử hoặc tính lại mọi lượt đã huỷ. Nếu giữ version cho lượt đang chạy, cách rút thay đổi phải tôn trọng quyết định hiệu lực đã duyệt hoặc có quyết định thay thế rõ ràng.

## 7. Rubric và phản biện

Chấm 0–2 cho mỗi tiêu chí: delta chính xác; nhất quán liên mô hình; hiệu lực/version/replay; coverage case biên và ngoại lệ; thẩm quyền/điều kiện triển khai. Mốc tự học: ít nhất 8/10, không mục nào 0.

Tự trả lời ba câu trước review: arriving có thay không; khách huỷ và driver_cancel khác gì; replay cũ có dùng v2 không? Nếu câu trả lời không nhất quán giữa sơ đồ và bảng, sửa trước khi trình.

**Câu hỏi:** Vì sao chưa thể kết luận thay đổi miễn phí sẽ giảm huỷ?

<details><summary>Gợi ý trả lời</summary><p>Phân tích tác động cho biết hệ thống phải thay đổi thế nào để thực thi chính sách. Nó chưa chứng minh phản ứng người dùng hoặc giá trị kinh doanh; cần giả thuyết, đo lường và đánh giá thích hợp.</p></details>

## 8. Tham khảo và bước tiếp theo

- `examples/toi-noi-testing/SPEC.md`, R-CANCEL/R-STATE/R-PAY/R-IDEM.
- Hồ sơ thực hành: `examples/toi-noi-ba/p03-mo-hinh-quy-trinh.md`.
- [Review liên mô hình](/articles/ba-kiem-tra-nhat-quan-mo-hinh), [Decision table](/articles/ba-decision-table-phi-huy).

Phần 3 hoàn thành ở đây. Tiếp tục với [User story: vai trò, nhu cầu và giá trị](/articles/ba-user-story-vai-tro-nhu-cau-gia-tri).

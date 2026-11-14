---
slug: ba-bpmn-task-event-gateway-lane
title: "BPMN cơ bản: task, event, gateway và lane"
summary: "Đọc và dựng một luồng BPMN phân tích yêu cầu huỷ chuyến của Tới Nơi. Phân biệt công việc, sự kiện, quyết định, trách nhiệm và giao tiếp giữa các participant."
category: business-analysis
level: beginner
tags: [business-analysis, ba-018, bpmn, process-model, toi-noi]
language: vi
---

Hai người vẽ cùng quy trình, nhưng một người dùng hình thoi cho “tính phí”, người kia dùng nó cho “được huỷ không?”. Một người kéo mũi tên từ khách sang hệ thống như thể cùng một tiến trình. Sơ đồ có vẻ quen mắt nhưng cách hiểu về việc gì chạy và ai kiểm soát đã khác nhau.

BPMN cung cấp ký pháp chung để trao đổi quy trình. Ở bài này, ta dùng một tập nhỏ đủ biểu diễn luồng phân tích, chưa xây quy trình thực thi bằng engine.

<div class="tkh-callout"><span class="tkh-callout-title">BA-018 · Phần 3/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/ba-as-is-to-be-quy-trinh">As-is và to-be</a>. Đầu ra: BPMN một pool, hai lane và nhánh từ chối. Mục tiêu: chọn đúng loại phần tử và giải thích được đường đi qua sơ đồ.</p></div>

## 1. Học ký pháp theo câu hỏi cần diễn đạt

Theo [đặc tả BPMN 2.0.2 của OMG](https://www.omg.org/spec/BPMN/2.0.2/), event, activity, gateway và các loại flow có ý nghĩa riêng. Bảng dưới là diễn giải ngắn để đọc ví dụ.

| Phần tử | Dùng trong ví dụ |
|---|---|
| Start/end event | Bắt đầu và kết thúc phạm vi quy trình |
| Task | Một công việc, ví dụ kiểm điều kiện hoặc quyết định phí |
| Exclusive gateway | Chọn nhánh theo điều kiện đã có |
| Pool | Participant đang được mô hình hoá |
| Lane | Chia trách nhiệm bên trong pool |
| Sequence flow | Thứ tự xử lý trong cùng pool |
| Message flow | Giao tiếp giữa các participant khác nhau |

Gateway không tự tính phí. Một task chuẩn bị kết quả, rồi gateway định tuyến theo kết quả đó. Không dùng sequence flow vượt biên pool; mũi tên giữa hai lane cùng pool vẫn có thể là sequence flow.

## 2. Chọn participant và mức chi tiết

Sơ đồ ví dụ chỉ mô tả bên trong participant “Tới Nơi”. Khách là người gửi yêu cầu bên ngoài; yêu cầu đã được tiếp nhận tại điểm bắt đầu của phạm vi rút gọn. Hai lane là phân công phân tích “Tiếp nhận” và “Xử lý nghiệp vụ”, không khẳng định hai service hoặc hai bảng database thực tế.

Nếu mở rộng để mô tả khách, tạo pool riêng và nối trao đổi bằng message flow. Không nhét khách vào lane nội bộ chỉ để tiết kiệm diện tích khi họ thực sự là participant độc lập trong mô hình này.

## 3. Sơ đồ rút gọn cho yêu cầu huỷ

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 900 320" role="img" aria-label="BPMN rút gọn trong pool Tới Nơi: tiếp nhận yêu cầu, kiểm điều kiện, gateway cho phép hoặc từ chối; nhánh cho phép quyết định phí rồi kết thúc phạm vi minh hoạ."><defs><marker id="ba18-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0L10 5L0 10z" class="f-pk"/></marker></defs><rect x="10" y="15" width="880" height="290" class="f-surface s-line"/><path d="M95 15V305M95 155H890" class="s-line"/><text x="25" y="145" class="t-small" transform="rotate(-90 25 145)">TỚI NƠI</text><text x="45" y="85" class="t-small">Nhận</text><text x="40" y="235" class="t-small">Xử lý</text><circle cx="145" cy="85" r="18" class="f-surface s-pk" stroke-width="2"/><rect x="205" y="55" width="160" height="60" rx="10" class="f-pk-soft s-pk"/><text x="285" y="90" text-anchor="middle" class="t-small">Tiếp nhận yêu cầu</text><rect x="205" y="200" width="160" height="60" rx="10" class="f-pk-soft s-pk"/><text x="285" y="235" text-anchor="middle" class="t-small">Kiểm điều kiện</text><path d="M465 200L495 230L465 260L435 230Z" class="f-surface s-pk"/><text x="465" y="237" text-anchor="middle" class="t-b">×</text><rect x="565" y="200" width="160" height="60" rx="10" class="f-pk-soft s-pk"/><text x="645" y="235" text-anchor="middle" class="t-small">Quyết định phí</text><rect x="565" y="55" width="160" height="60" rx="10" class="f-warn-soft s-warn"/><text x="645" y="90" text-anchor="middle" class="t-small">Trả lý do từ chối</text><circle cx="810" cy="85" r="18" class="f-surface s-pk" stroke-width="4"/><circle cx="810" cy="230" r="18" class="f-surface s-pk" stroke-width="4"/><g class="f-none s-pk" stroke-width="2" marker-end="url(#ba18-a)"><path d="M163 85H203"/><path d="M285 115V198"/><path d="M365 230H433"/><path d="M495 230H563"/><path d="M465 200V85H563"/><path d="M725 85H790"/><path d="M725 230H790"/></g><text x="500" y="216" class="t-small">Có</text><text x="475" y="140" class="t-small">Không</text><text x="735" y="285" class="t-small">Sang xử lý tiếp*</text></svg></div><figcaption>*End phía dưới kết thúc phạm vi minh hoạ quyết định, chưa có nghĩa chuyến đã huỷ hoặc tiền đã thu. Luồng đầy đủ phải nối cập nhật chuyến và xử lý thanh toán.</figcaption></figure>

Đọc từ start, qua tiếp nhận và kiểm điều kiện. Gateway chọn một nhánh có/không. Đường từ chối kết thúc; đường được phép đi tới quyết định phí. Hai đường không cần chờ nhau vì đây không phải tách song song.

## 4. Viết điều kiện sao cho đọc được

“Được phép?” phải có định nghĩa ở bảng đi kèm: khách đúng quyền và trạng thái thuộc searching, accepted, arriving; quy trình thực tế còn bước xác thực và replay, các bước này được tách khỏi sơ đồ nhập môn. Người review phải biết phần nào đã giản lược.

Nhãn “Có” và “Không” gắn với cùng một câu hỏi. Nếu thêm nhánh “Không biết”, xác định dữ liệu nào khiến chưa thể kết luận và người phụ trách xử lý. Không để thiếu dữ liệu rơi vào nhánh miễn phí mặc định.

## 5. Tên task diễn đạt công việc, không diễn đạt trạng thái

“Đã huỷ” phù hợp làm kết quả, còn “Ghi nhận huỷ” là công việc. “10.000 đồng” là dữ liệu/kết quả tính, không phải task. “accepted” là trạng thái chuyến, không nên thay cho thao tác “Kiểm tra trạng thái hiện tại”.

Một sơ đồ đọc được vẫn cần rule và dữ liệu đi kèm. BPMN cho thấy khi nào quyết định phí xảy ra; decision table sau này sẽ xác định mức phí. Tách hai việc giúp đổi chính sách mà không phải nhét toàn bộ công thức vào hình thoi.

## 6. Walkthrough để tìm lỗi mô hình

Đi qua một case khách huỷ searching: đường cho phép, kết quả phí 0. Đi qua một case picked_up: đường từ chối. Hỏi người review tại sao token đi theo mỗi nhánh; câu trả lời phải dựa vào điều kiện và dữ kiện case.

Kiểm các lỗi phổ biến: task không có đầu ra, gateway hỏi về điều chưa được tính hoặc chưa được đọc, message flow trong cùng pool, sequence flow qua pool, nhánh song song ghép sai hoặc end bị hiểu nhầm thành completed của chuyến.

## 7. Bài tập và câu hỏi ôn tập

Vẽ lại sơ đồ với pool khách riêng, mô tả gửi yêu cầu và nhận phản hồi. Giữ xử lý nội bộ trong pool Tới Nơi. Đánh dấu các message, không kéo sequence flow từ khách sang task của hệ thống.

**Câu hỏi:** End event có luôn tương ứng cancelled/completed không?

<details><summary>Gợi ý trả lời</summary><p>Không. Nó kết thúc đường xử lý trong phạm vi quy trình đang mô hình hoá. Trạng thái nghiệp vụ phải được ghi rõ bằng kết quả hoặc tác động của task.</p></details>

## 8. Tham khảo và bài tiếp theo

- [OMG — BPMN 2.0.2](https://www.omg.org/spec/BPMN/2.0.2/), định nghĩa chính thức của ký pháp. Sơ đồ bài là ví dụ phân tích rút gọn, không phải file thực thi.
- Baseline: `examples/toi-noi-testing/SPEC.md`, R-STATE/R-CANCEL.

Bài tiếp theo: [Business rule: tách chính sách khỏi màn hình](/articles/ba-business-rule-chinh-sach-nghiep-vu).

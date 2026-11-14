---
slug: ba-decision-table-phi-huy
title: "Decision table: đặc tả phí huỷ không bỏ sót tổ hợp"
summary: "Chuyển rule phí huỷ thành bảng quyết định có thứ tự ưu tiên, kiểm tra độ phủ và điều kiện biên. Phân biệt từ chối huỷ, miễn phí và thu phí trong Tới Nơi."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-020, decision-table, business-rule, toi-noi]
language: vi
---

Đoạn văn “huỷ dưới hai phút miễn phí, tài xế trễ cũng miễn phí” khá dễ đọc. Nhưng khi nhiều điều kiện cùng đúng, API cần trả một lý do; khi một trường dữ liệu thiếu, nhóm cần biết có được đưa vào bảng không. Bảng quyết định làm các trường hợp đó hiện ra rõ hơn.

<div class="tkh-callout"><span class="tkh-callout-title">BA-020 · Phần 3/8 · Trung bình</span><p>Đọc trước: <a href="/articles/ba-business-rule-chinh-sach-nghiep-vu">Business rule</a>. Đầu ra: bảng quyết định và bộ test case chứng minh độ phủ. Phạm vi: yêu cầu huỷ của khách chủ chuyến, dữ liệu hợp lệ; xác thực và replay được xử lý ở lớp quy trình tương ứng.</p></div>

## 1. Xác định miền đầu vào trước khi lập bảng

Đặt `S` là trạng thái; `G` là `now - acceptedAt < 2 phút`; `L` là `now > acceptedAt + etaMinutes + 10 phút`; `B` là phí cơ sở theo loại xe. Các biến G/L chỉ được xét khi dữ liệu cần thiết tồn tại và đầy đủ.

Không tính G ở searching khi acceptedAt chưa tồn tại. Không đưa actor tài xế vào bảng phí khách. Phân chia miền đầu vào như vậy giúp tránh tạo tổ hợp vô nghĩa rồi gán kết quả tuỳ ý.

## 2. Bảng quyết định theo thứ tự dòng

Bảng dưới dùng quy tắc **dòng phù hợp đầu tiên**. Dấu “—” nghĩa là không cần xét ở dòng đó, không phải dữ liệu bị thiếu. Thứ tự dòng là một phần của đặc tả.

| Dòng | S | G | L | Kết quả | Lý do |
|---|---|---|---|---|---|
| D0 | Ngoài searching/accepted/arriving | — | — | Từ chối huỷ | INVALID_TRANSITION |
| D1 | searching | — | — | Huỷ, phí 0 | searching |
| D2 | accepted | Đúng | — | Huỷ, phí 0 | grace_period |
| D3 | accepted hoặc arriving | — | Đúng | Huỷ, phí 0 | driver_late |
| D4 | accepted hoặc arriving | — | — | Huỷ, phí B | charged |

D4 không có nghĩa mọi accepted đều có phí; dòng này chỉ được xét sau các dòng trên. D2 trước D3 giữ ưu tiên lý do theo R-CANCEL-06. Nếu xuất bảng sang công cụ hoặc đổi cách sắp xếp, phải giữ hit policy này hoặc viết lại điều kiện thành các miền loại trừ nhau.

## 3. Chứng minh độ phủ và xem chồng lấn

Chia trước theo state. Trạng thái nằm ngoài ba trạng thái được phép thì đi vào D0. Searching đi D1. Accepted đi D2 nếu G đúng; nếu không, xét L ở D3 rồi D4. Arriving bỏ qua G, xét L rồi mức phí cơ sở. Như vậy mọi đầu vào **trong miền hợp lệ đã định nghĩa** có một kết quả được chọn.

Điều kiện D3 và D4 có thể cùng đúng về mặt biểu thức, nhưng first-match giải quyết thứ tự. Nếu nhóm yêu cầu mỗi đầu vào chỉ khớp đúng một dòng, cần viết D4 với L sai và điều kiện G sai cho accepted, tách arriving riêng. Không trộn hai cách hiểu rồi tuyên bố bảng “không chồng lấn”.

Trường hợp dữ liệu bất thường như accepted thiếu acceptedAt nằm ngoài miền đánh giá đã thống nhất, phải có luồng xử lý riêng được duyệt. Độ phủ bảng không chứng minh hệ thống đã xử lý mọi dữ liệu hỏng.

## 4. Tách mức tiền khỏi điều kiện thu

| Loại xe theo baseline | B |
|---|---:|
| Xe máy, gồm đơn đồ ăn | 10.000 đ |
| Ô tô | 20.000 đ |

Việc tách bảng giúp tránh lặp lại toàn bộ điều kiện miễn cho mỗi loại xe. Tuy nhiên, phải chỉ rõ nguồn loại xe và miền giá trị được hỗ trợ. Loại xe không xác định không được tự rơi về mức rẻ nhất.

Đây là quyết định phí, chưa phải kết quả thu tiền. Với ví, phí B có thể đã được xác định nhưng thanh toán thất bại; với tiền mặt, lab ghi nhận phí để thu sau. Không dùng “charged” trong R-CANCEL-06 để kết luận chắc chắn tiền đã bị trừ.

## 5. Chọn test case để kiểm bảng

Giả sử acceptedAt 10:00:00, ETA 5 phút, xe máy; các test case độc lập:

| Case | Dữ kiện | Dòng chọn / kết quả |
|---|---|---|
| T1 | searching | D1 / 0, searching |
| T2 | accepted, 10:01:59 | D2 / 0, grace_period |
| T3 | accepted, 10:02:00 | D4 / 10.000, charged |
| T4 | arriving, 10:01:30 | D4 / 10.000, charged |
| T5 | arriving, 10:15:00 | D4 / 10.000, charged |
| T6 | arriving, 10:15:01 | D3 / 0, driver_late |
| T7 | picked_up | D0 / từ chối |

Thêm test case ô tô có phí để kiểm B; thêm actor sai để kiểm luồng quyền bên ngoài bảng. Với thời gian, các giá trị trên chỉ minh hoạ ranh giới theo giây; triển khai phải thống nhất đơn vị/độ chính xác, không làm tròn thời gian trước phép so sánh nếu chưa có quy tắc cho phép.

## 6. Khi chính sách thay đổi

CR-BA-01 chỉ đổi G từ `< 2 phút` sang `< 3 phút` ở accepted. T3 đổi kết quả, nhưng T4 không đổi vì arriving không dùng G. T5/T6 vẫn kiểm biên trễ, không bị thay theo khoảng miễn.

Review phần chênh lệch theo từng nhóm: dòng nào thay, test case nào thay, test case nào phải giữ nguyên. Một thay đổi tưởng nhỏ có thể làm sai toàn bộ arriving nếu developer tái sử dụng G cho cả hai trạng thái mà tài liệu không chỉ rõ phạm vi.

## 7. Bài tập và câu hỏi ôn tập

Viết lại bảng thành các dòng loại trừ nhau, rồi chứng minh mỗi test case T1–T7 chỉ khớp một dòng. Thêm C03/C04 trong hồ sơ P03 để kiểm mốc 2 phút 59 giây và 3 phút cho v2 đề xuất.

**Câu hỏi:** Vì sao không sắp xếp bảng theo mức phí tăng dần cho dễ nhìn?

<details><summary>Gợi ý trả lời</summary><p>Vì bảng hiện dùng first-match; đổi thứ tự có thể đổi dòng được chọn và lý do kết quả. Muốn sắp lại, phải giữ ngữ nghĩa bằng điều kiện loại trừ hoặc cơ chế ưu tiên rõ ràng.</p></details>

## 8. Tham khảo và bài tiếp theo

- SPEC lab: `examples/toi-noi-testing/SPEC.md`, R-CANCEL-01–07 và R-PAY-05/06. Bảng D0–D4 là cách biểu diễn riêng, không thay ID nguồn.

Bài tiếp theo: [State model: trạng thái chuyến và chuyển trạng thái hợp lệ](/articles/ba-state-model-trang-thai-chuyen).

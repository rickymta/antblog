---
slug: ba-quan-sat-quy-trinh-doc-tai-lieu
title: "Quan sát quy trình và đọc tài liệu hiện có"
summary: "Đối chiếu điều người dùng nói, điều quan sát được và điều tài liệu quy định. Lập observation log, kiểm tra nguồn tài liệu và xử lý chênh lệch trong discovery điểm đón của Tới Nơi."
category: business-analysis
level: beginner
tags: [business-analysis, ba-011, observation, document-analysis, toi-noi]
language: vi
---

Tài liệu hỗ trợ ghi “ticket phải có ID lượt đặt xe”. Một nhân viên nói đôi khi tìm chuyến bằng số điện thoại. Trong mẫu 20 ticket do trưởng ca chọn, 8 ticket thiếu ID. BA nên kết luận nhân viên làm sai, tài liệu lỗi thời hay phần mềm thiếu kiểm tra?

Chưa đủ bằng chứng để chọn một nguyên nhân. Quan sát và đọc tài liệu giúp nhìn rõ chênh lệch, rồi đặt câu hỏi có mục tiêu thay vì quy lỗi quá sớm.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-011 · Phần 2/8 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/ba-phong-van-nghiep-vu-khong-dan-dat">Phỏng vấn nghiệp vụ</a>. Đầu ra: observation log và document register. Mục tiêu: ghi hành vi có bối cảnh, kiểm tra hiệu lực tài liệu và phân biệt quy trình được quy định với cách làm quan sát được.</p>
</div>

## 1. Ba loại nguồn trả lời các câu hỏi khác nhau

| Nguồn | Giúp hiểu | Không tự chứng minh |
|---|---|---|
| Phỏng vấn | Cách người tham gia nhớ, giải thích và đánh giá sự việc | Hành vi chính xác trong mọi lần sử dụng |
| Quan sát | Hành động và điều kiện nhìn thấy trong phiên | Mức độ phổ biến trong toàn bộ người dùng hoặc ý nghĩ bên trong |
| Tài liệu | Quy tắc, thiết kế, hướng dẫn đã được ghi lại | Phiên bản đang chạy hoặc mức tuân thủ thực tế |

[Hướng dẫn contextual research của GOV.UK](https://www.gov.uk/service-manual/user-research/contextual-research-and-observation) nhấn mạnh việc hiểu cách người dùng làm việc trong bối cảnh thực tế. Trong bài này, các phiên quan sát đều được mô phỏng; thực hành nhằm học cách ghi bằng chứng và giới hạn của nó.

## 2. Lập kế hoạch quan sát quanh câu hỏi

RQ-02 hỏi thông tin điểm đón được nhận biết lúc nào. Với tài xế, có thể quan sát thao tác khi xe đã dừng; không yêu cầu họ thao tác điện thoại khi lái. Xác định trước điểm bắt đầu, kết thúc và điều cần ghi: màn hình đang mở, thao tác chuyển màn hình, nội dung xuất hiện, hành động tiếp theo.

Phân biệt quan sát hoạt động tự nhiên với giao một nhiệm vụ thử nghiệm. Nếu BA nói “hãy tìm ghi chú”, phiên đó cho thấy khả năng tìm khi được yêu cầu, chưa cho biết tài xế có tự nhận ra cần tìm hay không. Ghi mọi lời nhắc vì chúng có thể làm thay đổi hành vi.

Thông báo cách ghi nhận, người có thể xem và quyền dừng phiên. Chỉ thu thông tin cần cho câu hỏi; dùng dữ liệu giả cho bài tập. Ghi lại các hạn chế như môi trường mô phỏng hoặc sự hiện diện của người quan sát.

## 3. Observation log: mô tả trước, diễn giải sau

Nguồn OBS-01 trong hồ sơ P02: ở C-02, khi xe đã dừng, D-02 mở chi tiết, kéo xuống rồi đọc được “Cổng số 2”.

| Trường | Ghi chép phù hợp |
|---|---|
| Nguồn / case | OBS-01 / C-02 |
| Bối cảnh | Phiên mô phỏng, xe đã dừng, có người nghiên cứu |
| Hành động quan sát | Mở chi tiết → kéo xuống → đọc ghi chú |
| Nội dung thấy được | “Cổng số 2” xuất hiện trong phần chi tiết |
| Diễn giải có thể | Ghi chú cần thêm thao tác để nhận biết trong phiên này |
| Điều chưa biết | Có lời nhắc không; phiên bản; hành vi lúc không bị quan sát; dữ liệu C-01 |
| Câu hỏi tiếp | Ghi chú có được nhận biết đúng lúc trong các trường hợp khác? |

Không ghi “tài xế lười đọc” vào cột quan sát. Đó là phán xét, không phải hành động nhìn thấy. Nếu chưa ghi thời lượng, không bổ sung một con số ước đoán rồi dùng làm chỉ số hiệu năng.

## 4. Đọc tài liệu cùng nguồn gốc và phiên bản

Document register cho hồ sơ P02 có thể bắt đầu như sau:

| ID | Tài liệu | Thông tin có thể dùng | Điều cần xác nhận |
|---|---|---|---|
| DOC-01 | Hướng dẫn hỗ trợ v0.8 | Có yêu cầu ghi ID lượt vào ticket | Người phụ trách, ngày hiệu lực, đối tượng áp dụng, ngoại lệ |
| DOC-02 | Bản thiết kế màn hình chi tiết | Có vùng ghi chú trong thiết kế | Phiên bản triển khai, điều kiện hiển thị, mapping dữ liệu |

Đọc tài liệu để tìm từ ngữ nghiệp vụ, luồng, ngoại lệ và yêu cầu liên quan; ghi tham chiếu mục/đoạn để người khác kiểm tra. Thiếu người phụ trách hoặc ngày hiệu lực thì đánh dấu chưa xác nhận, không tự coi tài liệu là baseline.

Trong dự án thật, tài liệu còn có thể là mẫu biểu, ticket, biên bản và hợp đồng API. Chúng không có cùng thẩm quyền. Một comment trên ticket là manh mối khác với quyết định chính sách được người có quyền chốt.

## 5. Xử lý chênh lệch mà không chọn nguồn “thắng” quá sớm

DOC-01 và DATA-01 cho thấy khoảng cách giữa nội dung hướng dẫn và mẫu record. Có thể có ticket cũ, quy trình ngoại lệ, khó lấy ID hoặc thiếu tuân thủ. Chưa có nguồn phân biệt các khả năng này.

| Chênh lệch | Câu hỏi kiểm tra | Người phối hợp đề xuất |
|---|---|---|
| Hướng dẫn đòi ID, mẫu có record thiếu | Ticket tạo lúc nào, thuộc loại nào, hướng dẫn có hiệu lực khi đó không? | CSKH và người phụ trách tài liệu |
| Thiết kế có ghi chú, khách nói tài xế tới khác cổng | Dữ liệu và phiên bản trong cùng trường hợp có thể kiểm tra không? | Developer, UX, phụ trách dữ liệu |
| Nhân viên nói tìm bằng số điện thoại | Khi nào cần cách này, quyền và luồng được duyệt là gì? | CSKH và người quản lý truy cập |

8/20 = 40% **trong mẫu đã chọn**, không phải 40% của tất cả ticket. Mẫu này cũng độc lập với 80 cuộc gọi của P01, nên không dùng phép nhân 80 × 40% để suy ra 32 cuộc gọi thiếu ID.

## 6. Biến log thành phát hiện có giới hạn

Một phát hiện hợp lý: “Trong mẫu 20 ticket được chọn thuận tiện, 8 record thiếu ID lượt; việc dùng dữ liệu hỗ trợ để đo theo lượt cần kiểm tra thêm.” Câu này đủ để đề xuất kiểm tra chất lượng dữ liệu, nhưng không kết tội quy trình hay ước lượng mức lỗi toàn bộ hệ thống.

Phát hiện nên kèm nguồn, phạm vi, tác động tới quyết định và bước tiếp. Nếu chỉ đưa ảnh màn hình cùng lời nhận xét “UX chưa tốt”, người review khó biết phải làm gì hoặc đánh giá lập luận ra sao.

## 7. Bài tập và câu hỏi ôn tập

Từ OBS-02, viết một observation log: khách K-02 nhìn biển và nói không biết tên cổng tương ứng trong app. Tách phần nhìn thấy, lời nói và giả thuyết. Thêm hai câu hỏi xác minh mà không tự kết luận bản đồ sai.

Sau đó lập ba mục cần xác nhận cho DOC-01. Bài làm đạt yêu cầu khi nêu hiệu lực, phạm vi/ngoại lệ và người phụ trách, đồng thời giữ nguyên hạn chế chọn mẫu của DATA-01.

**Câu hỏi:** OBS-01 có bác bỏ INT-01 không?

<details><summary>Gợi ý trả lời</summary><p>Không. Chúng là hai trường hợp khác nhau. Một phiên đọc được ghi chú không chứng minh khách C-01 gửi đúng hoặc tài xế C-01 đã nhận và hiểu thông tin đó.</p></details>

## 8. Tham khảo và bài tiếp theo

- [GOV.UK — Contextual research and observation](https://www.gov.uk/service-manual/user-research/contextual-research-and-observation).
- Hồ sơ nguồn: `examples/toi-noi-ba/p02-discovery-ho-so.md`.

Bài tiếp theo: [Workshop: chuẩn bị, điều phối và xác nhận kết quả](/articles/ba-workshop-dieu-phoi-xac-nhan).

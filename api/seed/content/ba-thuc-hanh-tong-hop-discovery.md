---
slug: ba-thuc-hanh-tong-hop-discovery
title: "Thực hành: tổng hợp discovery từ dữ liệu mâu thuẫn"
summary: "Tổng hợp phần 2 bằng evidence matrix và discovery memo: nhận định nào được hỗ trợ, điều gì còn mâu thuẫn, cần xác minh gì và đã đủ căn cứ chọn giải pháp hay chưa."
category: business-analysis
level: advanced
tags: [business-analysis, ba-016, discovery, evidence, case-study, toi-noi]
language: vi
---

Cuối tuần discovery, BA có lời kể khách đã nhập cổng Đông, quan sát một tài xế đọc được ghi chú, tài liệu yêu cầu ticket có ID và một mẫu ticket lại thiếu ID. Nếu chỉ gom các câu thành bản tóm tắt, nhóm vẫn không biết nên tin điều gì hoặc làm bước nào tiếp.

Bài cuối phần 2 yêu cầu biến các nguồn thành nhận định có phạm vi và khuyến nghị. Những điều chưa biết phải hiện rõ trong kết luận, không bị làm mờ để hồ sơ trông hoàn chỉnh.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-016 · Phần 2/8 · Nâng cao · Thực hành tổng hợp</span>
<p>Đọc BA-009–BA-015 trước khi làm. Đầu ra: evidence matrix, confirmation log và discovery memo 1–2 trang, kèm các bảng hỗ trợ. Dành khoảng 90 phút thực hành. Hồ sơ nguồn và mẫu có tại <code>examples/toi-noi-ba/p02-discovery-ho-so.md</code>; mọi dữ liệu đều giả lập.</p>
</div>

## 1. Đề bài và giới hạn

Trong bài tập, discovery một tuần đã được chấp thuận. PO muốn xem kết quả cuối tuần theo WS-02, chưa cam kết phát hành hoặc chọn chat. Bạn cần trả lời:

1. Hiện biết gì về việc hai bên xác định điểm đón?
2. Bằng chứng nào mâu thuẫn thật, bằng chứng nào chỉ khác case hoặc phạm vi?
3. Dữ liệu có đủ hỗ trợ việc chọn giải pháp và chuẩn bị pilot không?
4. Khuyến nghị tiếp theo là gì, ai cần quyết định, điều kiện nào còn thiếu?

Giữ baseline P01 riêng với mẫu P02. Không cộng 20 ticket vào 80 cuộc gọi; không tính tỷ lệ người gặp khó từ hai phiên quan sát. Không tự bổ sung phỏng vấn hoặc phản hồi xác nhận chưa có trong đề bài.

## 2. Đọc gói nguồn trước khi viết nhận định

| Nguồn | Nội dung ngắn | Giới hạn quan trọng |
|---|---|---|
| INT-01 | Khách K-01 kể đã nhập cổng Đông nhưng tài xế tới Tây | C-01; chưa kiểm thông tin phía tài xế |
| INT-02 | D-01 thường nhìn ghim trước, mở chi tiết khi dừng | Người khác, không phải tài xế C-01 |
| INT-03 | CSKH nói một ca có thể gọi lại nhiều lần | Lời kể về cách làm, chưa là phép đếm |
| OBS-01 | D-02 mở chi tiết, kéo xuống và đọc cổng số 2 | C-02; một phiên mô phỏng |
| OBS-02 | K-02 chưa biết tên cổng tương ứng trong app | C-03; khác C-01 và C-02 |
| DOC-01 | Hướng dẫn v0.8 yêu cầu ghi ID lượt vào ticket | Chưa xác nhận hiệu lực |
| DATA-01 | 8/20 ticket trong mẫu thiếu ID lượt | Mẫu thuận tiện, độc lập tập cuộc gọi P01 |
| DOC-02 | Thiết kế có vùng ghi chú | Chưa xác nhận phiên bản triển khai |
| WS-01 | Vận hành muốn xem lịch sử trao đổi | Đề nghị chưa duyệt |
| WS-02 | PO muốn review cuối tuần | Mốc quyết định, không phải ngày phát hành |

Tách **nguồn** khỏi **claim — nhận định**. Một nguồn có thể hỗ trợ nhiều nhận định, và một nhận định cần nhiều nguồn. Tóm tắt lời INT-01 trong biên bản workshop không tạo thêm một xác nhận độc lập cho INT-01.

## 3. Lập evidence matrix và đánh giá mức hỗ trợ

Không cần gán xác suất 80% khi chưa có mô hình và dữ liệu để tính. Dùng mức hỗ trợ định tính, kèm lý do: được hỗ trợ trong phạm vi nguồn, còn tranh luận, chưa đủ hỗ trợ hoặc bị bác bỏ trong phạm vi đã kiểm.

| Claim | Nguồn | Đánh giá hợp lý | Bước tiếp |
|---|---|---|---|
| CL-01: có khó khăn gọi tên cổng trong C-03 | OBS-02 | Được hỗ trợ trong phiên mô phỏng | Quan sát các kiểu địa điểm/nhóm khác |
| CL-02: mọi tài xế đều không đọc ghi chú | INT-02, OBS-01 | Không được hỗ trợ; OBS-01 có đọc ghi chú | Thu hẹp nhận định về thời điểm nhận biết và kiểm thêm |
| CL-03: ghi chú có thể chưa được nhận biết đúng lúc | INT-02, OBS-01 | Giả thuyết đáng kiểm, chưa chứng minh tác động huỷ | Quan sát không dẫn dắt và đối chiếu cùng case |
| CL-04: không thể mặc định mọi ticket đều nối được lượt | DATA-01 | Có dấu hiệu trong mẫu cần xử lý | Kiểm schema, quy tắc chọn mẫu, thời điểm và ngoại lệ |
| CL-05: chat sẽ làm tỷ lệ huỷ còn 9% | Không có nguồn hiệu quả | Chưa đủ hỗ trợ | So sánh phương án, thiết kế đánh giá; giữ 9% là đề xuất |

Mức hỗ trợ phải gắn với **câu cụ thể**. DATA-01 hỗ trợ “có record thiếu trong mẫu”, không hỗ trợ “40% tất cả ticket thiếu”. Chỉ cần đổi vài chữ, mức chắc chắn hợp lý đã khác.

## 4. Phân loại mâu thuẫn và chọn cách giải

**INT-01 với OBS-01:** chưa phải mâu thuẫn trực tiếp vì khác case. Việc C-02 đọc được ghi chú không giải thích dữ liệu của C-01. Cần kiểm cùng lượt, phiên bản và thời điểm nếu muốn xác minh đường đi thông tin của C-01.

**DOC-01 với DATA-01:** có chênh lệch giữa quy định được ghi và mẫu thực tế. Chưa biết tài liệu còn hiệu lực, record thuộc thời kỳ nào hoặc có ngoại lệ gì. Cần owner tài liệu và dữ liệu xác nhận trước khi kết luận vi phạm quy trình hoặc lỗi hệ thống.

**WS-01 với nhu cầu giới hạn dữ liệu:** đây là xung đột phương án/đánh đổi cần làm rõ, không phải hai dữ kiện đối nghịch. Hãy phân loại case hỗ trợ, dữ liệu cần có và người có quyền chốt.

**80 cuộc gọi với 120 lượt huỷ:** không thể so trực tiếp để nói bao nhiêu lượt huỷ đã gọi hỗ trợ. Đơn vị đếm khác nhau và chưa có mapping. Đây là khoảng trống dữ liệu, không phải mâu thuẫn.

## 5. Confirmation log và kế hoạch xác minh

| ID | Điều cần xác nhận | Người phù hợp đề xuất | Nếu chưa có phản hồi |
|---|---|---|---|
| CF-01 | INT-01 được ghi đúng lời kể và giới hạn hiểu biết | Người đóng vai K-01 trong bài tập | Giữ nguồn ở trạng thái chưa xác nhận bản tóm tắt |
| CF-02 | DOC-01 còn hiệu lực, áp dụng cho loại ticket nào | Owner quy trình CSKH | Chưa dùng làm baseline quy trình |
| CF-03 | Cách chọn 20 ticket, định nghĩa thiếu ID và khả năng nối khác | Người cung cấp DATA-01 và phụ trách dữ liệu | Không suy rộng hay dùng để tính tỷ lệ toàn hệ thống |
| CF-04 | Dữ liệu tối thiểu cần cho các loại ca hỗ trợ | CSKH và owner dữ liệu/truy cập | Giữ WS-01 là đề nghị chưa duyệt |

Ghi hạn đã thoả thuận khi có người nhận việc. Không tự ghi người dùng hoặc đồng nghiệp đã xác nhận chỉ để khép bài tập. Nếu mô phỏng thêm phản hồi để luyện tập, đánh dấu dữ kiện bổ sung và không trộn vào gói nguồn gốc.

Ưu tiên xác minh theo ảnh hưởng quyết định. Kiểm liên kết ticket/lượt và điều kiện hiển thị ghi chú giúp định hướng phương án; tranh luận màu nút chat chưa hỗ trợ câu hỏi cần chốt cuối tuần.

## 6. Gợi ý discovery memo

> **Vấn đề:** có tín hiệu khó khăn xác định điểm đón, nhưng nguồn hiện có chưa giải thích được nguyên nhân hoặc tỷ lệ toàn khu vực. Baseline 12% và target đề xuất 9% của P01 không phải kết quả discovery mới.
>
> **Phát hiện có giới hạn:** C-03 cho thấy khó gọi tên cổng trong một phiên; C-02 cho thấy ghi chú được đọc sau thao tác mở chi tiết/kéo xuống. Mẫu ticket có record thiếu ID; cần kiểm khả năng nối dữ liệu trước khi dùng cho đo theo lượt.
>
> **Khuyến nghị:** chưa cam kết chat hoặc pilot. Chuẩn bị thử cách mô tả/nhận biết điểm đón với dữ liệu giả để phân biệt giả thuyết; đồng thời xác nhận quy trình và chất lượng liên kết dữ liệu. Kế hoạch tiếp cần owner, nguồn lực và thời hạn được PO/người có thẩm quyền duyệt, không tự kéo dài discovery.
>
> **Quyết định cần xin:** chọn hoạt động xác minh nào trước và có cấp nguồn lực hay không. Nếu không thể tiếp cận bằng chứng quan trọng, thu hẹp kết luận, đổi ưu tiên hoặc dừng hướng hiện tại với lý do được ghi lại.

Đây là một lời giải có thể chấp nhận. Bạn có thể khuyến nghị khác nếu vẫn giữ giới hạn nguồn và làm rõ đánh đổi. “Chưa làm chat” không có nghĩa đã chứng minh chat vô ích; hồ sơ hiện chưa đủ căn cứ chọn nó.

## 7. Rubric, bài nộp và tự phản biện

Nộp tám sản phẩm đã xây trong phần 2: stakeholder map, interview guide, observation/document log, workshop record, đánh giá khảo sát, journey/blueprint, conflict log và memo kèm evidence matrix. Có thể dùng bảng ngắn, không cần tài liệu dài cho mỗi kỹ thuật.

Chấm mỗi tiêu chí 0–2; mốc tự học đề xuất từ 8/10 và không mục nào bằng 0:

| Tiêu chí | Điều cần thấy để đạt 2 |
|---|---|
| Truy vết | Claim có ID nguồn và phân biệt nguồn độc lập với bản sao |
| Phạm vi kết luận | Không suy rộng mẫu thuận tiện, không ghép case sai |
| Xử lý khác biệt | Phân biệt mâu thuẫn dữ kiện, thiếu dữ liệu và xung đột mục tiêu |
| Kế hoạch xác minh | Câu hỏi, owner đề xuất, cách kiểm và tác động quyết định rõ |
| Khuyến nghị | Có bước tiếp, quyền chốt, điều kiện và giới hạn |

Ba câu phản biện nên tự trả lời trước khi nộp:

1. Nguồn nào sẽ làm bạn đổi khuyến nghị?
2. Nhóm stakeholder nào đang thiếu, nên kết luận nào phải thu hẹp?
3. Nếu chỉ được thực hiện một hoạt động tiếp theo, bạn chọn gì và vì sao?

<details><summary>Gợi ý cho câu thứ ba</summary><p>Không có đáp án duy nhất. Nếu chọn kiểm dữ liệu, phải giải thích quyết định nào phụ thuộc chất lượng liên kết. Nếu chọn quan sát thông tin điểm đón, phải nêu giả thuyết cần phân biệt và giới hạn của mẫu. Chọn vì “dễ làm nhất” mà không nối với quyết định là chưa đủ.</p></details>

## 8. Tham khảo và bước tiếp theo

- Hồ sơ/mẫu: `examples/toi-noi-ba/p02-discovery-ho-so.md`.
- [Phỏng vấn](/articles/ba-phong-van-nghiep-vu-khong-dan-dat) · [Quan sát/tài liệu](/articles/ba-quan-sat-quy-trinh-doc-tai-lieu) · [Khảo sát](/articles/ba-khao-sat-lay-mau-thien-lech) · [Xử lý xung đột](/articles/ba-giai-quyet-yeu-cau-xung-dot).

Phần 2 hoàn thành ở đây. Tiếp tục với [As-is và to-be: mô tả hiện tại trước khi cải tiến](/articles/ba-as-is-to-be-quy-trinh). Không coi các giả thuyết còn mở của P02 là quy trình đã duyệt khi bắt đầu mô hình hoá.

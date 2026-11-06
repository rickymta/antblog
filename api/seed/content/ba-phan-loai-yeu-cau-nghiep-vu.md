---
slug: ba-phan-loai-yeu-cau-nghiep-vu
title: "Business requirement, stakeholder requirement và solution requirement"
summary: "Phân biệt mục tiêu kinh doanh, nhu cầu stakeholder, yêu cầu chức năng, phi chức năng và chuyển đổi. Dùng bài toán điểm đón của Tới Nơi để nối yêu cầu tới giá trị và cách kiểm chứng."
category: business-analysis
level: beginner
tags: [business-analysis, ba-003, requirements, traceability, acceptance-criteria, toi-noi]
language: vi
---

Trong một trang ghi chép, BA nhận được bốn câu: “Giảm huỷ chuyến”, “Tài xế cần biết khách đứng cổng nào”, “Hiển thị ghi chú điểm đón” và “Màn hình phải nhanh”. Nếu gom cả bốn vào danh sách tính năng, nhóm sẽ khó biết câu nào giải thích mục tiêu, câu nào mô tả nhu cầu và câu nào đủ rõ để triển khai.

Phân loại yêu cầu giúp nhóm nối những câu này thành một lập luận có thể kiểm tra. Nó cũng cho thấy khi nào một tính năng không còn phục vụ mục tiêu, hoặc một mục tiêu chưa có yêu cầu nào hỗ trợ.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-003 · Phần 1/8 · Cơ bản</span>
<p>Bài trước: <a href="/articles/ba-po-pm-ux-dev-tester-phoi-hop">Phối hợp giữa BA và các vai trò</a>. Đầu ra: danh sách yêu cầu có phân loại và bảng truy vết nhỏ. Mọi yêu cầu minh hoạ dưới đây là đề xuất của bài tập BA, chưa phải SPEC đã duyệt của Tới Nơi.</p>
</div>

## 1. Bắt đầu bằng câu hỏi mà yêu cầu trả lời

Các nhóm dưới đây giúp diễn đạt yêu cầu ở những mức khác nhau. Không cần ép mọi câu ghi chú vào đúng một ô ngay; có câu còn là giả thuyết hoặc ý tưởng thiết kế và cần được làm rõ trước.

| Nhóm | Câu hỏi | Ví dụ đề xuất |
|---|---|---|
| Business requirement — yêu cầu kinh doanh | Tổ chức muốn đạt kết quả gì, vì sao? | Giảm tỷ lệ huỷ trước đón tại A trong pilot |
| Stakeholder requirement — yêu cầu của bên liên quan | Bên liên quan cần khả năng/thông tin gì để đạt kết quả? | Tài xế cần nhận biết cổng khách đang chờ trước khi tới điểm đón |
| Solution requirement — yêu cầu giải pháp | Giải pháp cần làm được gì và có chất lượng nào? | Hiển thị ghi chú của lượt được nhận; đáp ứng ngưỡng thời gian phản hồi |
| Transition requirement — yêu cầu chuyển đổi | Cần gì để chuyển từ hiện tại sang trạng thái mới? | Hướng dẫn CSKH xử lý ca dùng luồng điểm đón mới trước pilot |

Solution requirement thường được tách thành **functional** — hành vi/chức năng, và **non-functional** — chất lượng hoặc điều kiện mà giải pháp cần đáp ứng. Đây không phải cách chia “quan trọng” với “phụ”. Một ngưỡng chất lượng hoặc quyền truy cập bị bỏ sót có thể làm giải pháp không sử dụng được.

## 2. Business requirement cần kết quả đo được

“Giảm huỷ chuyến” cho biết hướng đi nhưng còn thiếu phạm vi và thước đo. Bản nháp tốt hơn cho tình huống học tập là:

> **BR-01 · Đề xuất:** Trong pilot bốn tuần ở khu vực A, tỷ lệ lượt huỷ trước đón trên các lượt đặt xe đã được tài xế nhận không quá 9%; đồng thời tỷ lệ khiếu nại trên 1.000 lượt được nhận không tăng so với baseline tương đương.

Baseline minh hoạ từ bài trước là 120/1.000 = 12% trong một tuần. Trước khi duyệt BR-01, nhóm phải kiểm tra tính đại diện của baseline, đo baseline khiếu nại và thống nhất cách chờ kết quả của cohort pilot. Con số 9% là mục tiêu để thảo luận, không phải bằng chứng cho tác động của một tính năng.

Yêu cầu này sẽ được đánh giá bằng dữ liệu vận hành sau pilot. Không thể chạy một unit test để kết luận tổ chức đã đạt BR-01. Ngược lại, việc chỉ số đạt mục tiêu cũng chưa tự chứng minh tính năng là nguyên nhân: khu vực, thời tiết hoặc cơ cấu lượt đặt xe có thể đã thay đổi.

## 3. Stakeholder requirement giữ nhu cầu trước khi chốt giao diện

Từ discovery, nhóm có thể đề xuất:

| ID | Yêu cầu | Nguồn cần xác nhận |
|---|---|---|
| SHR-01 | Tài xế cần xác định được cổng/điểm khách đang chờ trước khi đến đón | Phỏng vấn và quan sát tài xế |
| SHR-02 | Khách cần biết thông tin điểm đón nào đã được gửi cho tài xế | Quan sát khách đặt xe |
| SHR-03 | CSKH cần xác định thông tin điểm đón liên quan khi xử lý phản ánh | Workshop quy trình hỗ trợ |

Những câu này chưa bắt buộc dùng chat, banner hay một màn hình mới. Nhiều phương án có thể đáp ứng cùng một nhu cầu. Đây là lý do nên ghi lại nhu cầu ngay cả khi nhóm đã nghiêng về một thiết kế cụ thể.

“Stakeholder requirement” cũng không có nghĩa mọi lời stakeholder nói đều được duyệt. BA còn phải làm rõ xung đột, tính khả thi, giá trị và thẩm quyền quyết định.

## 4. Solution requirement cần đủ rõ để triển khai và kiểm tra

Giả sử nhóm đang đánh giá phương án hiển thị ghi chú điểm đón. Để học cách viết, ta đặt các yêu cầu đề xuất sau; chúng chưa thay đổi hành vi của lab hiện có.

**FR-01 — chức năng:** Khi tài xế mở chi tiết lượt đặt xe đã nhận và được phân công cho mình, hệ thống hiển thị ghi chú điểm đón hiện hành của lượt đó. Nếu không có ghi chú, hiển thị “Khách chưa thêm ghi chú điểm đón”.

**FR-02 — kiểm soát truy cập:** Hệ thống từ chối cung cấp ghi chú điểm đón cho tài xế không được phân công lượt đó, kể cả khi truy cập trực tiếp bằng mã lượt đặt xe.

**NFR-01 — hiệu năng, ngưỡng minh hoạ:** Trong môi trường kiểm thử tham chiếu đã thống nhất, với 100 phiên tài xế đồng thời đọc chi tiết lượt của mình trong 15 phút, ít nhất 95% lần mở thành công hiển thị nội dung trong tối đa 2 giây tính từ thao tác mở; tỷ lệ lần mở lỗi không quá 1% trên tất cả lần mở.

NFR-01 vẫn phải được nhóm kỹ thuật và nghiệp vụ thống nhất thiết bị, điều kiện mạng, dữ liệu và cách ghi nhận kết quả trước khi coi là sẵn sàng kiểm thử. Các con số là giả định bài tập, không lấy từ tải hoặc SLO chính thức của Tới Nơi. Việc đo riêng lỗi giúp tránh loại hết các lần lỗi rồi báo “95% nhanh”.

Một yêu cầu tốt cũng làm lộ câu hỏi còn mở: nếu khách sửa ghi chú sau khi tài xế đã đọc thì có thông báo không; ai được sửa; lưu phiên bản nào; thông tin được giữ bao lâu? Chưa có quyết định thì ghi thành câu hỏi, không tự lấp bằng phỏng đoán.

### Acceptance criteria cụ thể hoá hành vi

Cho FR-01, một ví dụ kiểm tra là:

```text
Given tài xế D được phân công lượt B
And ghi chú hiện hành của B là "Cổng số 2, cạnh nhà thuốc"
When D mở chi tiết lượt B thành công
Then phần điểm đón hiển thị "Cổng số 2, cạnh nhà thuốc"
```

Cần thêm ví dụ không có ghi chú và trường hợp truy cập trái phân công cho FR-02. Tiêu chí chấp nhận không thay thế toàn bộ yêu cầu; một ví dụ luồng thuận chưa bao phủ ngoại lệ hoặc chất lượng cần đáp ứng.

## 5. Transition requirement không nên biến mất khỏi kế hoạch

**TR-01 · Đề xuất:** Trước khi pilot bắt đầu, người phụ trách CSKH xác nhận nhóm trực pilot đã thực hành xử lý ba tình huống: không có ghi chú, thông tin hai bên không thống nhất và tài xế không truy cập được thông tin. Tài liệu hướng dẫn có phiên bản và đầu mối hỗ trợ.

TR-01 phục vụ việc chuyển sang cách làm mới. Nếu hệ thống cần một chức năng đào tạo thường xuyên, chức năng đó lại là yêu cầu giải pháp riêng. Phân biệt này giúp nhóm không bỏ quên công việc chuẩn bị chỉ vì nó không tạo ra màn hình hoặc API.

Migration dữ liệu cũng có thể tạo yêu cầu chuyển đổi, nhưng không phải thay đổi nào cũng cần migration. Với ví dụ ghi chú hiện có, phải kiểm tra dữ liệu trước khi quyết định có công việc chuyển đổi dữ liệu hay không.

## 6. Nối yêu cầu thành bảng truy vết nhỏ

Traceability — khả năng truy vết — cho phép đi từ mục tiêu xuống yêu cầu và từ yêu cầu quay lại lý do tồn tại. Quan hệ không nhất thiết một-một.

| Yêu cầu | Liên kết về trước | Bằng chứng kiểm tra dự kiến | Trạng thái |
|---|---|---|---|
| BR-01 | Problem brief điểm đón | Báo cáo pilot với định nghĩa metric và guardrail | Đề xuất |
| SHR-01 | BR-01 | Xác nhận nhu cầu qua quan sát/phỏng vấn | Cần xác minh |
| FR-01 | SHR-01, SHR-02 | Review ví dụ, kiểm thử chức năng, UAT hành trình | Đề xuất |
| FR-02 | FR-01; nhu cầu bảo vệ dữ liệu cần xác nhận | Kiểm thử truy cập không được phân công | Đề xuất |
| NFR-01 | FR-01, SHR-01 | Báo cáo đo trên môi trường và điều kiện đã chốt | Cần chốt điều kiện |
| TR-01 | SHR-03; rollout của FR-01 | Biên bản thực hành và xác nhận sẵn sàng hỗ trợ | Đề xuất |

Liên kết FR-01 → SHR-01 → BR-01 là lập luận về đóng góp kỳ vọng, **không chứng minh quan hệ nhân quả**. Nếu pilot không cải thiện, nhóm cần kiểm tra lại giả thuyết này.

Truy vết cũng làm rõ khoảng trống: FR-02 có lý do hợp lý nhưng chính sách chi tiết chưa được xác nhận. Ghi khoảng trống giúp người review đặt đúng câu hỏi trước khi triển khai.

## 7. Bài tập, lỗi thường gặp và tự kiểm tra

Hãy phân loại năm câu sau và sửa chúng để dùng được trong buổi refinement:

1. “App phải dễ dùng.”
2. “Thêm nút chat màu xanh.”
3. “Khách cần biết tài xế đang tới cổng nào.”
4. “Giảm cuộc gọi hỗ trợ về điểm đón.”
5. “CSKH phải được hướng dẫn trước khi pilot.”

Gợi ý: câu 1 nghiêng về chất lượng sử dụng nhưng cần nhiệm vụ, nhóm người dùng và thước đo; câu 2 là đề xuất thiết kế còn thiếu lý do; câu 3 là nhu cầu stakeholder cần xác nhận; câu 4 là mục tiêu kinh doanh còn thiếu mẫu số, baseline và thời gian; câu 5 là chuyển đổi còn thiếu cách xác nhận sẵn sàng.

Đừng biến câu 1 thành “dễ dùng 100%” hoặc câu 4 thành một tỷ lệ tự đặt rồi coi đã được phê duyệt. Bản sửa đạt yêu cầu phải ghi phần nào được đề xuất và phần nào còn cần thống nhất.

**Câu 1.** Bảo mật luôn là yêu cầu phi chức năng?

<details><summary>Gợi ý trả lời</summary><p>Không nên phân loại chỉ bằng từ khoá. Một quy tắc từ chối truy cập như FR-02 mô tả hành vi chức năng phục vụ bảo mật; các thuộc tính và điều kiện bảo mật khác có thể thuộc yêu cầu chất lượng. Quan trọng là nội dung rõ và kiểm tra được.</p></details>

**Câu 2.** Các test FR-01 đều đạt thì BR-01 đã đạt chưa?

<details><summary>Gợi ý trả lời</summary><p>Chưa. Test cho biết hành vi đáp ứng kỳ vọng trong điều kiện kiểm tra. Kết quả kinh doanh cần đo sau triển khai và xem xét các yếu tố ảnh hưởng.</p></details>

**Câu 3.** Có thể bỏ ID yêu cầu vì danh sách còn ngắn không?

<details><summary>Gợi ý trả lời</summary><p>Có thể chọn cách quản lý nhẹ, nhưng cần một cách tham chiếu ổn định khi review, kiểm thử và thay đổi. ID ngắn giúp tránh nhầm khi tiêu đề hoặc thứ tự thay đổi.</p></details>

## 8. Tham khảo và hướng học tiếp

- [IIBA — Needs and Solutions, Requirements and Designs](https://www.iiba.org/professional-development/knowledge-centre/articles/needs-and-solutions/): bối cảnh phân biệt nhu cầu, giải pháp, yêu cầu và thiết kế.
- [IIBA — Glossary](https://production.iiba.org/career-resources/a-business-analysis-professionals-foundation-for-success/babok/glossary/): tra các nhóm requirement; các câu minh hoạ trong bài là nội dung riêng của Tới Nơi.
- Bài liên quan: [BA là gì?](/articles/ba-la-gi-tu-tinh-nang-den-bai-toan), [Phối hợp giữa BA và các vai trò](/articles/ba-po-pm-ux-dev-tester-phoi-hop), [Verification và Validation](/articles/verification-va-validation).

Bài tiếp theo: [BACCM: nhìn bài toán qua sáu khái niệm cốt lõi](/articles/ba-baccm-sau-khai-niem-cot-loi).

---
slug: da-nguon-du-lieu-chuyen-su-kien-thanh-toan-cskh
title: "Nguồn dữ liệu Tới Nơi: chuyến, sự kiện, thanh toán và CSKH"
summary: "Lập source inventory cho phân tích huỷ chuyến: đọc đúng schema Tới Nơi, kiểm tra grain và lịch sử sự kiện, tránh nhân số khi nối thanh toán với CSKH và ghi rõ dữ liệu nào chưa có."
category: data-analysis
level: intermediate
tags: [data-analysis, da-007, source-inventory, grain, lineage, toi-noi]
language: vi
---

“Có bảng chuyến rồi, cứ lấy status cancelled là biết tất cả lượt huỷ trước đón.” Câu này bỏ qua hai điều: một trạng thái hiện tại chưa kể lại toàn bộ hành trình, và bộ dữ liệu minh hoạ có thể không ghi đầy đủ lịch sử giống hệ thống vận hành.

DA cần lập **source inventory**, danh mục nguồn dữ liệu: thông tin nào ở đâu, một dòng nghĩa là gì, ai hiểu nguồn, có thể dùng tới đâu và thiếu gì để trả lời câu hỏi.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-007 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/da-mo-ta-chan-doan-du-bao-hanh-dong">DA-006: chọn loại phân tích</a> và <a href="/articles/du-an-mau-toi-noi">hồ sơ Tới Nơi</a>. Đầu ra: danh mục nguồn và kế hoạch đối soát. Bài này phân tích schema và dữ liệu mẫu trong mã nguồn, không tuyên bố đã kết nối database đang chạy.</p>
</div>

## 1. Phân biệt ba mức “có dữ liệu”

Một bảng được mô tả trong thiết kế, một bảng có dữ liệu mẫu và một nguồn vận hành đã kiểm tra là ba trạng thái khác nhau. Nếu không ghi rõ, người đọc dễ hiểu “có schema” thành “đủ dữ liệu để kết luận”.

Trong P01, bằng chứng dùng để tính là bảng tổng hợp giả lập. Hồ sơ chung Tới Nơi cung cấp schema SQL và script mẫu để học database. Hai bộ này có mục đích khác nhau, nên không ghép số của chúng thành cùng một baseline.

| Mã nguồn học tập | Nội dung | Trạng thái dùng trong P01 |
|---|---|---|
| SRC-01 | Bảng tổng hợp W1/W2 của DA | Đã cung cấp để luyện tính, không có từng chuyến |
| SRC-02 | Schema và script SQL trong hồ sơ chung | Đã đọc mã nguồn; chưa kiểm tra một instance đang chạy |
| SRC-03 | Bản xuất dữ liệu thanh toán cho phân tích này | Chưa được cung cấp hoặc đối soát |
| SRC-04 | Bản xuất dữ liệu CSKH có liên kết với lượt | Chưa được cung cấp hoặc đối soát |
| SRC-05 | Nhật ký pipeline/tracking và lịch thay đổi | Chưa được cung cấp |

“Chưa cung cấp” không có nghĩa mọi phần khác của repository đều không có thanh toán hoặc hỗ trợ. Nhãn này chỉ xác định bằng chứng hiện đang có cho bài phân tích này.

## 2. Đọc bảng chuyến: những gì schema cho biết

Trong [hồ sơ chung](/articles/du-an-mau-toi-noi), `trips` có `id`, `kind`, `city_id`, `customer_id`, `driver_id`, `status`, `fare`, `requested_at`, `accepted_at`, `completed_at`, `updated_at` cùng một số trường khác. Grain là một chuyến; `kind` phân biệt `ride`, `food`, `parcel`.

M-01 cần lượt ride đã được nhận ở A. `city_id` giúp phân loại thành phố, nhưng **A chưa được ánh xạ tới thành phố hoặc vùng cụ thể** trong hồ sơ P01. Không tự cho rằng `city_id = 1` là A. Tương tự, phải xác nhận `accepted_at` có đáp ứng nghĩa “nhận lần đầu” trong mọi luồng cần phân tích không.

Schema này không có cột `picked_up_at` hoặc `cancelled_at`. Muốn xác định thứ tự đón và huỷ cần kiểm tra lịch sử phù hợp, không dùng `updated_at` để thay cho mọi mốc nghiệp vụ: bản ghi có thể được sửa vì lý do khác.

`fare` là giá chuyến trong schema mẫu, không tự chứng minh đã thu tiền hoặc doanh thu đã ghi nhận. Một lượt huỷ có thể liên quan phí/hoàn tiền, nhưng phải có định nghĩa nghiệp vụ và nguồn riêng trước khi tính.

## 3. Đọc sự kiện: có bảng chưa chắc có đủ lịch sử

`trip_events` trong hồ sơ chung có `id`, `trip_id`, `status`, `occurred_at`. Mỗi dòng là một sự kiện trạng thái. Quan hệ từ chuyến tới sự kiện là một–nhiều.

Phần script sinh 200.000 chuyến của hồ sơ chung chèn vào `trips`; đoạn đó không sinh đầy đủ chuỗi `trip_events` cho từng chuyến. Không thể chạy script mẫu rồi mặc định đã có lịch sử đủ để kiểm toán M-01.

Ngay cả khi đọc được sự kiện `cancelled`, cần biết lịch sử có đầy đủ và đúng thứ tự không. Không thấy `picked_up` có thể do chưa đón, cũng có thể do thiếu sự kiện. Nếu thời điểm bằng nhau, `id` tự tăng không tự bảo đảm thứ tự nghiệp vụ thực tế; cần hợp đồng về cách ghi nhận và thứ tự từ chủ nguồn.

Danh mục nguồn nên ghi các câu hỏi: có nạp dữ liệu (ingest) trễ không, có thử lại (retry) tạo ra bản sao không, có sửa sự kiện không, có cách phân biệt thời gian xảy ra và thời gian nạp không? Schema mẫu không chứa mọi metadata đó; chỉ ghi thành yêu cầu khảo sát hoặc đề xuất mở rộng.

## 4. Thanh toán và CSKH có grain riêng

Một chuyến có thể có nhiều lần thử thanh toán hoặc hoàn tiền. Một lượt liên hệ hỗ trợ có thể sinh nhiều message; một ticket có thể liên quan nhiều lượt hoặc không liên kết được lượt nào. Không chọn khoá nối chỉ vì các cột có tên gần giống.

Ví dụ độc lập, chưa phải dữ liệu P01: chuyến T-A có hai bản ghi thanh toán và ba ticket liên quan. Nối thẳng cả hai tập với chuyến có thể tạo **2 × 3 = 6 dòng**. Nếu đếm dòng thành số chuyến, một chuyến thành sáu; nếu cộng tiền sau join, mỗi khoản có thể bị lặp ba lần.

Cách tiếp cận ở mức thiết kế: xác định grain đích; tổng hợp từng nguồn về grain thích hợp trước khi nối, hoặc dùng phép kiểm tra tồn tại nếu chỉ cần biết có ticket hay không. Khi quan hệ ticket–chuyến là nhiều–nhiều, cần quy tắc phân bổ hoặc chỉ số phù hợp; không tự ép thành một–một.

Ở Phần 2 ta sẽ viết SQL cho tình huống này. Hiện tại, điều cần làm là ghi được cardinality — số quan hệ giữa các thực thể — và các phép đối soát cần chạy.

## 5. Một hàng inventory cần nhiều hơn tên bảng

| Trường | Ví dụ cho nguồn sự kiện dự kiến |
|---|---|
| Mục đích | Xác nhận lịch sử nhận, đón và huỷ của cohort |
| Chủ nguồn | Backend/chủ dịch vụ; cần chỉ định người cụ thể |
| Grain và khoá | Một sự kiện; ID sự kiện và liên kết trip_id |
| Thời gian | Ý nghĩa occurred_at; múi giờ; cơ chế xử lý trễ cần xác minh |
| Độ phủ | Kỳ nào, loại chuyến nào, phiên bản nào được ghi đủ? |
| Chất lượng | Thiếu/trùng; thứ tự; đối soát trạng thái cuối |
| Quyền và giới hạn | Ai được xem; có cần thông tin định danh thật không? |
| Phiên bản | Schema, tập trích, thời điểm lấy và người tạo |
| Bằng chứng hiện tại | Schema đã có; dữ liệu lịch sử đầy đủ chưa được cung cấp |

Theo [Government Data Quality Framework](https://www.gov.uk/government/publications/the-government-data-quality-framework/the-government-data-quality-framework), metadata và cách truyền đạt chất lượng giúp người dùng hiểu dữ liệu có phù hợp nhu cầu không. Với Tới Nơi, inventory là nơi lưu các điều kiện sử dụng đó, thay vì để chúng nằm trong tin nhắn riêng.

## 6. Kế hoạch đối soát trước khi dùng nguồn

Đầu tiên, kiểm tra số lượng và độ phủ: tổng số lượt thuộc cohort ở nguồn và ở bản trích có khớp không? Có thành phố, khung giờ hay phiên bản ứng dụng bị thiếu riêng không?

Tiếp theo, kiểm tra khoá và grain: trip ID có duy nhất trong bảng một lượt không; event ID có trùng không; có sự kiện không tìm được chuyến tương ứng không? Sau khi join, đối chiếu số lượt duy nhất, số dòng và các tổng bị sai lệch khi dòng bị nhân bản.

Cuối cùng, chọn các ca nghiệp vụ để cùng rà soát với chủ nguồn: lượt chưa nhận, lượt nhận cuối kỳ, lượt huỷ trước đón, lượt có sự kiện đến trễ và lượt bị sửa kết quả. Kiểm tra vài ca không chứng minh toàn bộ nguồn đúng, nhưng giúp phát hiện hiểu sai nghĩa cột trước khi viết truy vấn lớn.

Không lấy trung bình kết quả hai nguồn mâu thuẫn để tạo một số “dung hoà”. Phải làm rõ định nghĩa, thời điểm chốt và người chịu trách nhiệm về thông tin; chỉ định nguồn tham chiếu cho từng đại lượng dựa trên bằng chứng.

## 7. Lấy đủ dữ liệu cho câu hỏi

Để đếm lượt, thường cần ID ổn định chứ không cần tên và số điện thoại khách. Để phân tích vùng, có thể dùng nhóm khu vực đã được xác nhận thay vì toàn bộ lịch sử vị trí chính xác. Để review nhãn hỗ trợ, có thể cần một mẫu nội dung được xử lý phù hợp với phạm vi truy cập.

Đây là thiết kế dữ liệu phục vụ bài học, chưa thay thế quy định truy cập của tổ chức. Nhóm cần thống nhất ai có quyền cấp dữ liệu, ai được đọc và giữ bao lâu trước khi trích xuất. Hash một ID không tự biến mọi dữ liệu liên quan thành vô danh.

## 8. Bài tập và tự kiểm tra

Lập năm hàng inventory cho SRC-01–SRC-05. Với mỗi hàng, ghi trạng thái bằng chứng, grain, chủ nguồn dự kiến, câu hỏi trả lời được và ít nhất một giới hạn.

**Bài tình huống:** Có 100 lượt, mỗi lượt có hai lần thử thanh toán và ba ticket. Nếu mọi liên kết đều đầy đủ, phép join thẳng tạo bao nhiêu dòng? Có thể suy ra số khách duy nhất không?

<details><summary>Gợi ý lời giải</summary><p>Có 100 × 2 × 3 = 600 dòng. Số lượt duy nhất vẫn là 100 nếu khoá và liên kết đúng. Không suy ra số khách duy nhất nếu chưa có mapping khách; một khách có thể có nhiều lượt. Cần kiểm tra grain đích và xử lý từng nguồn trước khi cộng tiền hoặc đếm ticket.</p></details>

**Câu hỏi:** Trong P01, có thể dùng việc không thấy picked_up để kết luận huỷ trước đón không?

<details><summary>Gợi ý trả lời</summary><p>Chưa thể với một nguồn lịch sử chưa được kiểm tra độ đầy đủ. Cần xác nhận hợp đồng trạng thái, độ phủ sự kiện và các trường hợp thiếu/trễ. Bảng tổng hợp P01 cho phép luyện tính theo giả định đề bài, không chứng minh hệ thống nguồn đã được kiểm toán.</p></details>

## 9. Tham khảo và bước tiếp

- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): schema và script là nguồn đối chiếu trực tiếp của bài.
- [Government Data Quality Framework](https://www.gov.uk/government/publications/the-government-data-quality-framework/the-government-data-quality-framework): quản lý chất lượng và metadata.

Mẫu inventory: `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`. Bài tiếp: [DA-008: các yếu tố ảnh hưởng](/articles/da-yeu-to-anh-huong-ban-do-gia-thuyet).

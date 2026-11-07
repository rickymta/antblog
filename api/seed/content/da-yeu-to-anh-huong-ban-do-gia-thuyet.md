---
slug: da-yeu-to-anh-huong-ban-do-gia-thuyet
title: "Các yếu tố ảnh hưởng: nghiệp vụ, con người, dữ liệu và hệ thống"
summary: "Lập bản đồ giả thuyết cho huỷ chuyến Tới Nơi, tách thay đổi thực tế khỏi thay đổi cách đo, nhận diện yếu tố gây nhiễu và kiểm tra tác động của cơ cấu nhóm trước khi quy kết nguyên nhân."
category: data-analysis
level: advanced
tags: [data-analysis, da-008, confounding, hypothesis, segmentation, toi-noi]
language: vi
---

Vận hành cho rằng huỷ cao điểm tăng vì tài xế thiếu. Sản phẩm cho rằng khách không tìm được điểm đón. Kỹ thuật nhớ tuần vừa rồi có đổi tracking. Mỗi bên có một lời giải thích hợp lý, nhưng tính hợp lý chưa cho biết lời giải thích nào đúng.

DA cần biến các lời kể thành giả thuyết có thể kiểm tra, tìm bằng chứng phân biệt chúng và chủ động tìm điều có thể bác bỏ cách giải thích mình đang nghiêng về.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-008 · Phần 1/8 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/da-nguon-du-lieu-chuyen-su-kien-thanh-toan-cskh">DA-007: nguồn dữ liệu</a>. Đầu ra: hypothesis register và kế hoạch xác minh. Sau bài này, bạn nhận diện được yếu tố gây nhiễu, thay đổi cơ cấu nhóm và rủi ro diễn giải tương quan thành tác động.</p>
</div>

## 1. Tách hiện tượng thực tế khỏi phép đo

Số huỷ trên báo cáo có thể đổi vì khách/tài xế hành xử khác, hoặc vì cách ghi nhận/lọc/tổng hợp khác. Hai nhóm nguyên nhân này có thể xảy ra cùng lúc.

Một phiên bản ứng dụng mới có thể làm khó luồng tìm điểm đón, đồng thời sửa việc gửi event huỷ. Khi thấy số huỷ tăng sau phát hành, ta không được chọn một trong hai lời giải thích chỉ bằng thứ tự thời gian. Cần đối soát nguồn, độ phủ và hành vi thực tế.

Mẫu tư duy đơn giản:

```text
Bối cảnh, sản phẩm, vận hành → hành vi/kết quả thực tế
Hành vi/kết quả → ghi nhận nguồn → pipeline → định nghĩa báo cáo
```

Mỗi mắt xích có thể làm số cuối thay đổi. Sơ đồ này là khung điều tra, không phải mô hình nhân quả đã xác nhận cho Tới Nơi.

## 2. Nhóm yếu tố để tránh bỏ sót

| Nhóm | Giả thuyết cho Tới Nơi | Bằng chứng cần tìm |
|---|---|---|
| Nghiệp vụ | Chính sách phí hoặc điều kiện nhận chuyến thay đổi | Phiên bản chính sách, thời điểm áp dụng, nhóm bị ảnh hưởng |
| Con người | Khách mới hoặc tài xế mới có khó khăn khác trước | Định nghĩa nhóm, cohort, hành trình và dữ liệu đủ thời gian |
| Môi trường | Mưa, sự kiện địa phương, đường bị hạn chế | Dữ liệu đúng địa điểm/thời gian và độ phủ |
| Cung cầu | Nhu cầu tăng nhanh hơn tài xế sẵn sàng | Cung có thể phục vụ, trạng thái bận/rảnh, khoảng cách |
| Sản phẩm | Ghim điểm đón, hướng dẫn hoặc màn hình thay đổi | Lịch rollout, mức tiếp xúc, lỗi ứng dụng |
| Dữ liệu | Nhãn huỷ, timezone, định nghĩa cohort thay đổi | Schema/metric version, đối soát và kiểm tra dữ liệu |
| Hệ thống | Pipeline trễ hoặc lỗi ghi sự kiện | Log, thời điểm nhận dữ liệu, độ thiếu theo nhóm |

Tất cả đang là giả thuyết. P01 không có ngày thực tế, nên chưa được gắn W2 với một trận mưa hay ngày lễ cụ thể. Việc có sẵn công cụ tra thời tiết không giải quyết được thiếu mapping ngày/vùng của bộ minh hoạ.

## 3. Yếu tố gây nhiễu: một biến cùng liên quan nhiều phía

Giả sử ta muốn hiểu mối liên hệ giữa thời gian chờ và huỷ. Mưa có thể khiến tài xế di chuyển chậm hơn và khách dễ đổi kế hoạch hơn. Khi ấy, quan hệ quan sát giữa chờ lâu và huỷ có thể phản ánh một phần tác động của mưa.

**Confounder — yếu tố gây nhiễu** là khái niệm cần xem xét khi một yếu tố có vai trò trong cả phía phơi nhiễm và kết quả của câu hỏi nhân quả. Không phải mọi cột tương quan đều là biến nên “kiểm soát”. Phải hiểu thứ tự và cơ chế trước khi chọn cách phân tích.

Ví dụ, nếu đang đánh giá thay đổi hướng dẫn điểm đón, thời gian tìm nhau có thể là một mắt xích mà hướng dẫn tác động qua đó. Giữ thời gian này cố định có thể làm mất một phần tác động muốn đo. Đưa tất cả cột vào mô hình không tự loại được sai lệch.

Ngay ở mức P01, hãy ghi: yếu tố xảy ra trước hay sau kết quả; có thể ảnh hưởng cả hai phía không; có phải chỉ được quan sát ở một nhóm đã chọn không? Chưa rõ thì giữ câu hỏi mở, chưa dùng mô hình như một máy xác nhận nguyên nhân.

## 4. Thay đổi cơ cấu nhóm có thể đảo nhận xét chung

**Biến thể MIX-01 độc lập, không thay bảng W1/W2 của P01:** hai nhóm H/L có độ khó phục vụ khác nhau. Tất cả lượt thuộc đúng một nhóm và đã đủ kết quả.

| Kỳ | H: nhận / huỷ | H: tỷ lệ | L: nhận / huỷ | L: tỷ lệ | Toàn bộ |
|---|---|---:|---|---:|---|
| V1 | 100 / 20 | 20% | 900 / 90 | 10% | 110/1.000 = 11% |
| V2 | 900 / 162 | 18% | 100 / 8 | 8% | 170/1.000 = 17% |

Trong mỗi nhóm, tỷ lệ giảm 2 điểm phần trăm, nhưng toàn bộ tăng từ 11% lên 17%. Nguyên nhân toán học là tỷ trọng nhóm H có tỷ lệ cao tăng từ 10% lên 90%.

Nếu dùng tỷ trọng V1 làm một phép chuẩn hoá mô tả, tỷ lệ V2 theo cơ cấu ấy là 10% × 18% + 90% × 8% = **9%**. Con số 9% không thay tỷ lệ thực tế V2 là 17%; nó trả lời câu hỏi khác: nếu cân các tỷ lệ nhóm V2 theo cơ cấu tham chiếu V1 thì kết quả là gì?

Phép tính không chứng minh dịch vụ đã được một can thiệp làm tốt hơn. Nhóm có thể còn khác nhau ở nhiều yếu tố, và cách chia nhóm có thể chưa thích hợp cho câu hỏi nhân quả. Giá trị của ví dụ là buộc DA kiểm tra cơ cấu trước khi kết luận từ số chung.

Trong baseline P01 gốc, cao điểm và còn lại đều chiếm 50% mỗi tuần. Không dùng lời giải thích đổi tỷ trọng của MIX-01 cho bảng P01 khi tỷ trọng ở đó không đổi.

## 5. Biến giả thuyết thành kế hoạch có thể bác bỏ

| ID | Giả thuyết | Dấu hiệu dự kiến nếu đúng | Bằng chứng làm yếu giả thuyết | Bước xác minh |
|---|---|---|---|---|
| H-01 | Thiếu event ở W1 làm tỷ lệ cao điểm trông thấp | Nguồn và kho lệch riêng W1/cao điểm | Đối soát đầy đủ, lịch ghi nhận ổn định | DE/backend kiểm tra SRC-05 và lịch sử |
| H-02 | Điểm đón nhiều cổng gây khó tìm nhau | Ca đã xác minh tập trung khó khăn đó | Ca đối chiếu cùng địa điểm không có dấu hiệu, hoặc nhãn sai | Lập mẫu ca huỷ và ca đón thành công để review |
| H-03 | Cung có thể phục vụ giảm ở cao điểm | Thời gian chờ và mức thiếu cung đồng biến trong phạm vi phù hợp | Cung ổn định, biến động chỉ ở bản app mới | Xác định phép đo cung và phân tích theo thời gian/vùng |
| H-04 | Nhãn lý do huỷ bị đổi cách dùng | Phân bố nhãn đổi quanh phiên bản/form mới | Nhãn và quy trình ổn định qua đối soát | Review phiên bản và mẫu nhãn với CSKH |

“Bằng chứng làm yếu” không phải phép bác bỏ tuyệt đối. Chẳng hạn, cung ổn định toàn thành phố chưa bác được thiếu cung ở một cụm nhỏ. Mọi phép kiểm tra phải khớp phạm vi của giả thuyết.

Với H-02, chỉ hỏi người đã huỷ sẽ thiếu nhóm đối chiếu. Cần cân nhắc cả người được đón thành công ở điều kiện tương tự, đồng thời không coi mẫu thuận tiện là đại diện cho toàn bộ khách.

## 6. Ưu tiên xác minh theo khả năng đổi quyết định

Kiểm tra cách đo thường nên làm sớm vì nếu sai, các phân tích tiếp theo có thể phải tính lại. Nhưng thứ tự cụ thể phụ thuộc chi phí, độ sẵn sàng và tác động của từng thiếu hụt.

Một bảng ưu tiên tốt nêu lý do: H-01 cần xác minh trước khi so sánh; H-02 có thể giúp thiết kế phương án điểm đón nhưng cần kế hoạch mẫu; H-03 cần nguồn mới nên chưa hứa kết quả trong cùng hạn. Không gán xác suất “80% do tài xế” nếu chỉ có cảm nhận.

Sau mỗi phép kiểm tra, cập nhật trạng thái: chưa kiểm tra, có dấu hiệu, bị yếu đi, chưa phân biệt được, hoặc cần thiết kế đánh giá tác động. Không dùng “đã xác nhận” chỉ vì tìm được một ca phù hợp câu chuyện.

## 7. Những sai lầm cần tránh

- Dùng thời điểm release làm bằng chứng đủ cho nguyên nhân; nhiều thay đổi có thể trùng ngày.
- Chỉ lấy nhóm có kết quả đầy đủ, bỏ qua lượt huỷ/treo rồi kết luận trải nghiệm chung tốt hơn.
- Chọn nhóm sau khi xem dữ liệu nhưng trình bày như giả thuyết đặt trước.
- Gộp dịch vụ hoặc vùng có cơ cấu thay đổi mà không kiểm tra tỷ trọng.
- Coi lý do người dùng chọn là nguyên nhân đã xác minh; nhãn có thể phản ánh lựa chọn gần nhất hoặc mặc định.

Tinh thần khám phá và kiểm tra giả định được trình bày trong [NIST — What is EDA?](https://www.itl.nist.gov/div898/handbook/eda/section1/eda11.htm). Bản đồ giả thuyết giúp áp dụng tinh thần đó vào bài toán cụ thể thay vì chỉ vẽ thêm biểu đồ.

## 8. Bài tập và tự đánh giá

Từ bốn giả thuyết H-01–H-04, chọn hai việc nên xác minh trước. Nêu bằng chứng cần có, người phối hợp, kết quả nào khiến bạn đổi ý và điều còn chưa biết dù kiểm tra thành công.

**Bài tính MIX-01:** Giải thích vì sao tỷ lệ chung xấu hơn dù từng nhóm tốt hơn. Có được thay số dashboard V2 bằng 9% không?

<details><summary>Gợi ý lời giải</summary><p>Tỷ trọng nhóm H tăng mạnh, kéo tỷ lệ chung lên. Tỷ lệ 9% dùng cơ cấu tham chiếu V1, hữu ích để so sánh mô tả nhưng không thay tỷ lệ thực tế 17%. Nếu công bố cả hai, phải ghi nhãn, công thức và mục đích; không dùng chuẩn hoá như bằng chứng nhân quả.</p></details>

**Câu hỏi:** Xem thêm nhiều dòng dữ liệu có tự giải quyết yếu tố gây nhiễu không?

<details><summary>Gợi ý trả lời</summary><p>Không. Thêm dữ liệu cùng cơ chế chọn mẫu hoặc cùng thiếu hụt đo lường có thể làm con số ổn định hơn mà vẫn lệch. Cần thiết kế so sánh và nguồn phù hợp với giả thuyết, không chỉ tăng số dòng.</p></details>

## 9. Tham khảo và bước tiếp

- [NIST — What is EDA?](https://www.itl.nist.gov/div898/handbook/eda/section1/eda11.htm).
- [DA-007: nguồn và độ phủ dữ liệu](/articles/da-nguon-du-lieu-chuyen-su-kien-thanh-toan-cskh).

Mẫu hypothesis register: `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`. Bài tiếp: [DA-009: khi chưa đủ dữ liệu để kết luận](/articles/da-chua-du-du-lieu-bat-dinh-dieu-kien-dung).

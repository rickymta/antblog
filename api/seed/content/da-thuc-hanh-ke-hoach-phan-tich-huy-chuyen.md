---
slug: da-thuc-hanh-ke-hoach-phan-tich-huy-chuyen
title: "Thực hành: lập kế hoạch phân tích huỷ chuyến tại Tới Nơi"
summary: "Bài tổng hợp Phần 1: bảo vệ analysis brief, hợp đồng chỉ số, danh mục nguồn và kế hoạch xác minh huỷ chuyến. Có hồ sơ đầu vào, rubric, gợi ý đáp án và tình huống dữ liệu trễ để tự review."
category: data-analysis
level: advanced
tags: [data-analysis, da-010, capstone, analysis-plan, toi-noi]
language: vi
---

Bạn là DA trong một nhóm Tới Nơi giả lập. Đại diện vận hành muốn biết có nên ưu tiên xử lý nhóm cao điểm hay không; phía sản phẩm đề nghị cân nhắc chat. Cuộc họp cần một kế hoạch có thể thực hiện và một bản nhận định trung thực về dữ liệu hiện có.

Nhiệm vụ của bạn là **bảo vệ kế hoạch phân tích**, chưa phải chứng minh chat hiệu quả, dựng kho dữ liệu hay viết mô hình dự báo. Đây là bài tổng hợp mười bài đầu của series.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-010 · Phần 1/8 · Nâng cao</span>
<p>Đọc trước DA-001–DA-009, đặc biệt <a href="/articles/da-chua-du-du-lieu-bat-dinh-dieu-kien-dung">điều kiện chưa được kết luận</a>. Đầu ra: gói kế hoạch 4–6 trang hoặc tài liệu tương đương gồm brief, metric contract, inventory, hypothesis/assumption log và decision log. Không yêu cầu SQL hoặc dataset từng chuyến.</p>
</div>

## 1. Đề bài và phạm vi được giao

Lời đề nghị giả lập: “Số huỷ từ 120 lên 180, cao điểm có vẻ tệ hơn. Tuần tới cần biết có nên dành người điều tra điểm đón và cân nhắc chat không.”

Bạn được phép dùng bảng P01 và hồ sơ chung của dự án. Chưa có export từng chuyến, lịch sử đầy đủ, thanh toán, CSKH, tracking hay dữ liệu thời tiết theo kỳ. Không tự bổ sung ngày, khung giờ cao điểm hoặc kết quả pilot rồi coi là dữ kiện.

| Kỳ | Cao điểm: nhận / huỷ | Còn lại: nhận / huỷ | Tổng: nhận / huỷ |
|---|---|---|---|
| W1 | 500 / 80 | 500 / 40 | 1.000 / 120 |
| W2 | 750 / 135 | 750 / 45 | 1.500 / 180 |

Đề gốc giả định các lượt đã đủ kết quả, không thiếu/trùng trong bảng, nhóm giờ loại trừ nhau và bao phủ cohort. Phạm vi là ride tại A, cohort theo nhận lần đầu, múi giờ Asia/Ho_Chi_Minh. Đây là giả định cho bảng luyện tính, không phải kết quả kiểm toán dữ liệu thực tế.

Hồ sơ trong repository:

- `examples/toi-noi-da/p01-nen-tang.md`: baseline và sản phẩm DA-001–DA-003.
- `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`: mẫu điền, nguồn, biến thể và checklist cho DA-004–DA-010.

## 2. Sản phẩm A — Analysis brief một trang

Nêu người sử dụng kết quả, quyết định cần hỗ trợ và câu hỏi chính. Phân biệt việc trả lời ngay bằng bảng tổng hợp với việc chỉ có thể lập kế hoạch xác minh.

Giữ phạm vi ride/A/W1–W2 và ghi rõ những gì chưa thuộc nhiệm vụ: đánh giá toàn bộ dịch vụ, tính doanh thu, quyết định mua warehouse hoặc chứng minh tác động chat. Đề xuất hạn trả lời gắn với điều kiện nguồn sẵn sàng; dùng mốc tương đối T nếu chưa có ngày.

Tiêu chí hoàn thành cần kiểm tra được: số đối soát, định nghĩa rõ, giả thuyết có trạng thái, dữ liệu thiếu có người phối hợp và khuyến nghị không vượt bằng chứng. Không dùng “phải tìm được nguyên nhân” làm điều kiện bắt buộc khi chưa biết nguồn có đáp ứng không.

## 3. Sản phẩm B — Metric contract M-01

Hợp đồng phải có grain, cohort, tử số/mẫu số, biên kỳ, timezone, quy tắc loại trừ, dữ liệu chưa có kết quả, version và owner. Nêu rằng A, lịch cao điểm và điều kiện chốt thật còn cần xác nhận.

Thêm ít nhất hai guardrail đề xuất, giải thích chúng phát hiện đánh đổi nào và còn thiếu dữ liệu gì. Chẳng hạn, khiếu nại theo lượt và tỷ lệ hoàn thành trên mọi yêu cầu giúp phát hiện những mặt khác nhau; cả hai đều cần hợp đồng riêng và hiện chưa có số để chốt baseline.

Đừng mượn mục tiêu pilot hoặc số CSKH của series BA làm số DA. Các tình huống cùng dùng Tới Nơi nhưng không cùng dataset nếu chưa có mapping và đối soát.

## 4. Sản phẩm C — Source inventory và kế hoạch kiểm tra

Lập danh mục SRC-01–SRC-05: bảng DA, schema/script chung, nguồn thanh toán, nguồn CSKH và nhật ký tracking/pipeline. Mỗi hàng có trạng thái bằng chứng, grain, khoá liên kết, thời gian, chủ nguồn, độ phủ và giới hạn.

Với schema chung, chỉ ra được `trips` có trạng thái hiện tại và `accepted_at`, còn `trip_events` mô tả sự kiện; script sinh chuyến không bảo đảm có lịch sử đủ để xác minh M-01. Thanh toán và CSKH chưa được cung cấp cho ca này.

Kế hoạch kiểm tra cần bao gồm khoá duy nhất, trùng/thiếu, biên kỳ, độ đầy đủ lịch sử và số dòng sau join. Nếu một chuyến có nhiều payment attempt và nhiều ticket, phải nêu nguy cơ nhân số trước khi cộng tiền hoặc đếm lượt.

## 5. Sản phẩm D — Bản nhận định và kế hoạch giả thuyết

Tính số và tỷ lệ chung, tỷ lệ từng nhóm, chênh lệch điểm phần trăm và mức tăng số huỷ. Nêu điều gì trong dữ liệu gợi ý ưu tiên cao điểm, đồng thời ghi giới hạn của phát hiện.

Chọn ít nhất ba giả thuyết khác loại: một về cách đo, một về bối cảnh/vận hành và một về sản phẩm/con người. Mỗi giả thuyết cần dấu hiệu mong đợi, bằng chứng có thể làm yếu, nguồn cần có, người phối hợp và bước tiếp.

Thêm assumption log và điều kiện dừng: chưa thống nhất mẫu số, chưa có mapping vùng/giờ, dữ liệu thiếu có thể đổi kết luận hoặc nguồn mâu thuẫn chưa giải thích được. Không yêu cầu mọi giả thuyết phải có kết quả trong cùng một hạn.

## 6. Sản phẩm E — Decision log và trình tự thực hiện

Đề xuất chuỗi công việc: xác nhận brief/metric → kiểm tra nguồn → đối soát → phân tích → review → bàn giao. Với mỗi bước, ghi ai làm, ai xác nhận và điều kiện qua bước.

Decision log cần ít nhất ba quyết định:

| ID | Quyết định cần xem xét | Người có thẩm quyền dự kiến | Trạng thái ban đầu |
|---|---|---|---|
| D-DA-01 | Chấp nhận phạm vi và định nghĩa phân tích? | Chủ quyết định vận hành, với chủ metric | Đề xuất |
| D-DA-02 | Dành nguồn lực xác minh dữ liệu/ca cao điểm? | Người quản lý nguồn lực liên quan | Chờ xem kế hoạch |
| D-DA-03 | Có thiết kế pilot phương án điểm đón/chat? | Sản phẩm/vận hành theo thẩm quyền | Chưa đủ căn cứ để chốt |

Tên vai trò là phân công trong bài tập; khi triển khai phải gán người và phạm vi quyền cụ thể. Không ghi “đã duyệt” vì một tài liệu đề xuất trông hoàn chỉnh.

## 7. Gợi ý đáp án: phần nào có thể kết luận ngay?

Thử tự làm trước khi mở gợi ý.

<details><summary>Đối soát và nhận định mẫu</summary>
<p>W1 có 120/1.000 = 12%, W2 có 180/1.500 = 12%. Số huỷ tăng 60 lượt, tương đương 50%; số lượt được nhận cũng tăng 50%. Cao điểm từ 16% lên 18%, tăng 2 điểm phần trăm hay 12,5% tương đối. Còn lại từ 8% xuống 6%, giảm 2 điểm phần trăm. Tỷ trọng hai nhóm đều 50% ở cả hai tuần.</p>
<p>Nhận định: tỷ lệ chung trong bảng không đổi nhưng diễn biến nhóm khác nhau. Ưu tiên xác minh cách đo và bối cảnh cao điểm là một đề xuất điều tra có cơ sở mô tả. Chưa biết nguyên nhân, tính ổn định theo thời gian hoặc hiệu quả chat; không áp số này cho toàn bộ Tới Nơi.</p>
</details>

<details><summary>Brief và bước tiếp mẫu</summary>
<p>Hỗ trợ quyết định có dành nguồn lực điều tra cao điểm. Bàn giao bảng mô tả, metric contract v0.1, inventory và kế hoạch xác minh; chưa hứa đánh giá tác động. Đề xuất T+3 sau khi xác nhận phạm vi/khả năng cung cấp nguồn, review thiếu hụt cuối T+1. Ưu tiên xác nhận mapping A/khung giờ, định nghĩa lịch sử nhận–đón–huỷ và thay đổi tracking. Nếu chưa có nguồn, giao hiện trạng và điều kiện cần để tiếp tục.</p>
</details>

<details><summary>Một giả thuyết đạt yêu cầu</summary>
<p>H-01: nguồn W1 thiếu event cao điểm có thể làm mức W1 trông thấp. Cần đối soát nguồn với bản trích và kiểm tra lịch thay đổi/độ trễ; DE và backend phối hợp. Đối soát đầy đủ, cùng định nghĩa và tracking ổn định sẽ làm giả thuyết yếu đi. Nếu phát hiện thiếu, phải tính lại mô tả trước khi giải thích nghiệp vụ. Hiện chưa có SRC-05 nên trạng thái vẫn là chưa kiểm tra.</p>
</details>

Đáp án không cần giống từng câu. Nó phải giữ đúng ranh giới giữa dữ kiện, giả định, giả thuyết, đề xuất và quyết định.

## 8. Tình huống phát sinh: chọn đúng phiên bản dữ liệu

Sau khi bàn giao, người review đưa **MISS-01 của DA-009**: W2 có 30 lượt cao điểm chưa rõ kết quả, đã nằm trong mẫu số 750/1.500; số huỷ xác nhận vẫn là 135 cao điểm và 45 còn lại. Đây là biến thể thay giả định đầy đủ, không cộng thêm vào biến thể 15 lượt trễ của DA-003.

Hãy sửa nhận định, trạng thái số liệu và kế hoạch tiếp. Nếu dùng quy tắc luyện tính “rà soát khi tỷ lệ cuối lớn hơn 13%”, kết luận theo ngưỡng đã được chốt chưa?

<details><summary>Gợi ý xử lý</summary><p>Tỷ lệ cuối toàn W2 trong giả định MISS-01 nằm từ 12% tới 14%; cao điểm từ 18% tới 22%. Chưa chốt được theo ngưỡng, vì cần ít nhất 16 trong 30 lượt thành huỷ để vượt 13%. Ghi nhãn tạm tính, yêu cầu xác minh, lưu phiên bản và thông báo người đã nhận kết luận cũ. Đây là giới hạn kịch bản, không phải khoảng tin cậy hoặc tác động chat.</p></details>

Nếu chọn làm thêm MIX-01 của DA-008, nộp thành phụ lục độc lập. Không dùng tổng 17% của MIX-01, 13% của DA-003 hay khoảng 12–14% của MISS-01 để sửa baseline gốc trừ khi đề bài đang xét đúng biến thể đó.

## 9. Rubric tự chấm

Đây là thang điểm học tập, không phải tiêu chuẩn chứng chỉ hay quy tắc duyệt dự án.

| Tiêu chí | Điểm tối đa | Dấu hiệu đạt |
|---|---:|---|
| Brief và phạm vi | 15 | Quyết định rõ, in/out và hạn có điều kiện |
| Metric contract | 20 | Grain/cohort/tử-mẫu/thời gian/chốt/version đủ rõ |
| Nguồn và chất lượng | 15 | Phân biệt nguồn có thật trong hồ sơ với nguồn chưa cung cấp; có kiểm tra grain/join |
| Phép tính và diễn giải | 15 | Đúng số, tỷ lệ, điểm phần trăm; không quy nhân quả |
| Giả thuyết và bất định | 15 | Có phương án giải thích cạnh tranh, khả năng bác bỏ và điều kiện dừng |
| Kế hoạch, bàn giao, quyết định | 10 | Có owner, trạng thái và bước tiếp khả thi |
| Xử lý MISS-01 | 10 | Đúng khoảng, ngưỡng và version; không trộn biến thể |

Mục tiêu luyện tập là từ 80/100 và không có lỗi trọng yếu. Lỗi trọng yếu gồm: tạo bằng chứng chưa có; đổi mẫu số âm thầm; khẳng định chat có tác dụng từ bảng mô tả; trộn biến thể; ghi quyết định đã duyệt khi chưa có xác nhận. Nếu mắc lỗi này, sửa và review lại dù tổng điểm các mục khác cao.

## 10. Chuẩn bị buổi bảo vệ kế hoạch

Hãy chuẩn bị trả lời: vì sao bắt đầu ở cao điểm; vì sao chưa chọn database; nguồn nào xác nhận huỷ trước đón; làm gì nếu chỉ có trạng thái cuối; điều gì khiến bạn đổi khuyến nghị; ai quyết định bước tiếp?

Câu trả lời tốt về database là cần xác định workload và nguồn trước, rồi đánh giá lựa chọn ở P04. Câu trả lời tốt về tác động chat là hiện chưa có thiết kế/bằng chứng để ước lượng; kế hoạch có thể dẫn tới pilot sau khi xác minh vấn đề và phương án. Cả hai câu đều thể hiện biết giới hạn công việc, không phải thiếu chủ động.

## 11. Hoàn thành Phần 1 và chuẩn bị Phần 2

Phần 1 hoàn thành khi bạn có một gói kế hoạch có thể review, tính lại các số đã cho, giải thích giới hạn và bảo vệ thứ tự xác minh. Chưa cần một dashboard chạy thật để chứng minh đã hiểu công việc DA.

Phần 2 sẽ đi vào SQL và đọc dữ liệu nghiệp vụ. Trước khi chuyển, giữ metric contract và inventory làm đầu vào; truy vấn sẽ phải phục vụ định nghĩa đã chốt, đồng thời giúp phát hiện chỗ định nghĩa hoặc nguồn chưa đủ.

Tài liệu nền: [DA-004: analysis brief](/articles/da-analysis-brief-cau-hoi-kiem-chung), [DA-005: metric contract](/articles/da-kpi-metric-chi-so-bao-ve), [DA-007: nguồn dữ liệu](/articles/da-nguon-du-lieu-chuyen-su-kien-thanh-toan-cskh), [DA-009: bất định](/articles/da-chua-du-du-lieu-bat-dinh-dieu-kien-dung). Tham khảo thêm [Microsoft Learn — Core analytics tasks](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/4-tasks) về các hoạt động nền tảng; rubric và hồ sơ thực hành ở đây là thiết kế riêng của antblog.

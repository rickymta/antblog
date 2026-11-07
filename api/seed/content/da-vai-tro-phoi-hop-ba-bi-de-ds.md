---
slug: da-vai-tro-phoi-hop-ba-bi-de-ds
title: "Một DA làm gì và phối hợp với BA, BI, DE, DS ra sao?"
summary: "Ai xác nhận tỷ lệ huỷ, ai sửa pipeline và ai quyết định thử thay đổi điểm đón? Phân chia công việc DA trong Tới Nơi bằng đầu ra, trách nhiệm và điều kiện bàn giao thay vì chỉ nhìn chức danh."
category: data-analysis
level: beginner
tags: [data-analysis, da-002, collaboration, responsibility, toi-noi]
language: vi
---

Sau khi xem báo cáo huỷ chuyến, vận hành nhắn DA: “Em kiểm tra số, sửa luôn dữ liệu rồi chọn database nào chạy dashboard nhanh nhé.” Cùng lúc, BA muốn biết có nên thêm chat, còn backend nói số trên màn hình của họ không giống báo cáo.

Một DA có thể tham gia cả ba cuộc trao đổi. Nhưng nếu chưa thống nhất ai xác nhận nghiệp vụ, ai quản lý dữ liệu và ai có quyền chốt thay đổi, nhóm rất dễ bàn giao một dashboard đẹp mà không ai chịu trách nhiệm về số bên trong.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-002 · Phần 1/8 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/da-la-gi-tu-so-lieu-toi-quyet-dinh">DA-001: từ số liệu tới quyết định</a>. Đầu ra: bảng trách nhiệm và phiếu bàn giao phân tích huỷ chuyến. Sau bài này, bạn xác định được việc DA cần làm, người cần phối hợp và người chốt từng loại quyết định.</p>
</div>

## 1. Bắt đầu từ công việc cần có

Với bộ số liệu DA-P01, nhóm đã biết tỷ lệ huỷ chung của W1 và W2 cùng là 12%, nhưng nhóm cao điểm lần lượt là 16% và 18%. Bước tiếp cần xác minh sự khác biệt và tìm bằng chứng giải thích, chưa phải triển khai chat.

Để làm được, nhóm phải có người xác định câu hỏi, giải thích trạng thái chuyến, cung cấp dữ liệu, kiểm tra phép tính và quyết định dành nguồn lực cho bước tìm hiểu tiếp theo. Các trách nhiệm đó vẫn tồn tại dù tổ chức có đủ chức danh hay một người kiêm nhiều việc.

[Microsoft Learn về các vai trò phân tích](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/3-roles) phân biệt công việc xây luồng dữ liệu, tạo mô hình dùng lại, phân tích, khoa học dữ liệu và phân tích nghiệp vụ. Bảng dưới là **cách phân công giả lập cho Tới Nơi**, không phải chuẩn chức danh áp dụng cho mọi công ty.

| Vai trò / năng lực | Trọng tâm trong ca Tới Nơi | Đầu ra để người khác dùng |
|---|---|---|
| DA — Data Analyst | Kiểm tra số liệu, phân nhóm và giải thích giới hạn | Truy vấn, bảng đối soát, phát hiện và khuyến nghị |
| BA — Business Analyst | Làm rõ nhu cầu, quy trình, phương án và yêu cầu thay đổi | Problem brief, mô hình nghiệp vụ và câu hỏi cần xác nhận |
| BI — Business Intelligence | Tổ chức báo cáo dùng lặp lại và trải nghiệm khai thác | Dashboard, bộ lọc, quyền xem và lịch cập nhật |
| DE — Data Engineer | Đưa dữ liệu từ nguồn tới nơi phân tích một cách tin cậy | Pipeline, cơ chế chạy lại và thông tin độ mới |
| AE — Analytics Engineer | Chuyển dữ liệu thành mô hình và chỉ số dùng chung | Data mart, định nghĩa metric có phiên bản và kiểm tra |
| DS — Data Scientist | Xây và đánh giá mô hình hoặc phương pháp phân tích chuyên sâu khi cần | Thiết kế thử nghiệm, dự báo hoặc mô hình được đánh giá |
| Backend / chủ nguồn | Giải thích và duy trì dữ liệu phát sinh từ ứng dụng | Hợp đồng sự kiện, ý nghĩa trạng thái và lịch sử thay đổi |
| Vận hành / chủ quyết định | Xác nhận ngữ cảnh sử dụng và chọn hành động trong thẩm quyền | Định nghĩa được xác nhận, quyết định và người thực hiện |

BI có thể là năng lực của đội, tên sản phẩm hoặc tên vị trí; không mặc định luôn có một người “BI” đứng riêng với DA. Tương tự, DA có thể làm mô hình dữ liệu hay thống kê, và DE có thể viết truy vấn phân tích. Điều cần chốt là **trách nhiệm cụ thể trong ca này**.

Chưa cần DS chỉ để chia 180 cho 1.500. Nếu câu hỏi chuyển thành dự báo nhu cầu hoặc thiết kế thử nghiệm có ảnh hưởng chéo giữa khách và tài xế, nhóm mới đánh giá năng lực chuyên sâu cần bổ sung.

## 2. Công việc thường ngày của DA trong ca huỷ chuyến

DA đầu tiên làm rõ người hỏi muốn quyết định điều gì: cần bố trí người xử lý nhiều ca hơn, hay cần thay đổi trải nghiệm vì tỷ lệ huỷ tăng? Hai mục tiêu dẫn tới cách phân tích khác nhau.

Tiếp đó, DA đọc định nghĩa nguồn cùng backend và vận hành. “Huỷ” trên màn hình có thể tính cả lượt chưa được nhận, còn DA-P01 chỉ tính huỷ trước đón trong cohort đã được nhận. Chênh số chưa chắc là lỗi phần mềm; phải đặt hai định nghĩa cạnh nhau trước khi yêu cầu sửa.

Sau khi lấy dữ liệu, DA kiểm tra độ đầy đủ, trùng lặp và thời điểm chốt. Khi có bảng chi tiết, DA thử các phép đối soát: số dòng sau join có tăng bất thường không, tổng nhóm có bằng tổng chung không, lượt cuối tuần đã đủ kết quả chưa?

Cuối cùng, DA trình bày phát hiện, giới hạn và lựa chọn tiếp theo. Nếu chưa có dữ liệu lý do huỷ đáng tin, DA ghi “chưa biết” và đề xuất cách thu thập. DA cũng lưu phiên bản truy vấn và nguồn để người khác kiểm tra lại, rồi theo dõi xem khuyến nghị được dùng thế nào.

## 3. Bảng trách nhiệm: phân biệt làm, kiểm tra và chốt

Chỉ ghi “DA phụ trách dữ liệu” quá rộng để xử lý tranh chấp. Ta cần tách người thực hiện, người kiểm tra và người quyết định đầu ra đã đủ dùng hay chưa.

| Công việc | Người thực hiện chính | Người kiểm tra / phối hợp | Người chốt trong bài tập |
|---|---|---|---|
| Chọn câu hỏi và phạm vi A | DA cùng BA | Vận hành | Chủ quyết định vận hành |
| Định nghĩa huỷ trước đón, cohort và khung giờ | DA soạn | BA, backend, vận hành | Chủ chỉ số do vận hành chỉ định |
| Cấp tập dữ liệu và mô tả độ mới | DE | Backend, DA | Chủ nguồn dữ liệu |
| Kiểm tra truy vấn và đối soát | DA | Một người có năng lực review độc lập | Người phụ trách phân tích |
| Công bố dashboard cho đúng nhóm người dùng | Người phụ trách BI | DA, quản trị quyền | Chủ báo cáo |
| Duyệt pilot hoặc thay đổi chính sách | BA và DA cung cấp bằng chứng | Sản phẩm, vận hành, kỹ thuật | Người có thẩm quyền theo phạm vi thay đổi |

Các vai trò “chủ chỉ số”, “chủ nguồn” và “chủ báo cáo” cần được gán cho người cụ thể khi làm thật. Một người có thể giữ nhiều vai trò, nhưng việc chấp nhận một định nghĩa không tự động trao quyền đọc toàn bộ dữ liệu hoặc sửa ứng dụng.

Ví dụ, DA đề xuất loại bản ghi trùng trong lớp phân tích. Chủ nguồn phải xác nhận đó là trùng do ingest hay là hai sự kiện hợp lệ. DA không tự xoá sự kiện nguồn để số đẹp hơn; cách sửa phải giữ được dấu vết và khả năng giải thích báo cáo cũ.

## 4. Bàn giao bằng bằng chứng thay vì một file đính kèm

Một phiếu bàn giao dữ liệu cho DA có thể gồm:

| Trường | Nội dung cần có cho ca này |
|---|---|
| Câu hỏi | So sánh huỷ trước đón trong cohort W1/W2 ở A |
| Tập dữ liệu | Phiên bản, người tạo, thời điểm trích và phạm vi |
| Đơn vị một dòng | Một lượt đặt xe, hay một sự kiện của lượt? |
| Định nghĩa thời gian | Thời điểm nhận đầu tiên, múi giờ và điều kiện chốt kết quả |
| Chất lượng | Các kiểm tra đã chạy, kết quả và phần chưa xác minh |
| Hạn chế sử dụng | Nhóm chưa có dữ liệu, trường chưa đáng tin và quyền truy cập |
| Người tiếp nhận | Người review, câu hỏi còn mở và điều kiện chấp nhận |

Trong hồ sơ P01 hiện tại, ta chỉ có bảng tổng hợp giả lập. Phiếu phải nói đúng điều đó; không ghi “đã kiểm tra từng chuyến trong production”. Một bảng minh hoạ hợp lệ không thay thế bằng chứng truy xuất từ hệ thống.

## 5. Ba tình huống dễ vướng

**“Dashboard chậm, DA tự chọn database mới được không?”** DA có thể ghi lại truy vấn, thời gian trả lời, lượng dữ liệu và nhu cầu độ mới. DE/DBA cùng người sở hữu nền tảng kiểm tra phương án vận hành; chủ ngân sách chốt đầu tư. P04 sẽ học cách lập hồ sơ lựa chọn, nhưng thẩm quyền không tự sinh ra từ việc biết tên một database.

**“Hai báo cáo khác số, đội nào sai?”** So sánh định nghĩa, filter, thời điểm cập nhật, timezone và quyền lọc dữ liệu trước. Nếu một báo cáo đếm sự kiện còn một báo cáo đếm lượt duy nhất, cả hai có thể tính đúng nhưng chỉ một cái phù hợp câu hỏi. Ghi rõ phiên bản định nghĩa đã thống nhất rồi đối soát lại.

**“DA tìm thấy cao điểm xấu hơn, có được kết luận cần chat?”** DA được khuyến nghị điều tra hoặc thử phương án khi có cơ sở. Việc chat xử lý được nguyên nhân nào cần BA, vận hành và kỹ thuật cùng xem xét; việc dành nguồn lực phải có người chốt. Phát hiện mô tả chưa là bằng chứng về hiệu quả giải pháp.

## 6. Khi nhóm chỉ có hai hoặc ba người

Một developer có thể kiêm DE và người sở hữu báo cáo; một BA có thể kiêm DA. Hãy giữ danh sách trách nhiệm và các lần xác nhận dù chưa cần thêm nhân sự.

Ví dụ, người viết truy vấn có thể tự kiểm tra bằng một phép tính khác, rồi nhờ người hiểu nghiệp vụ xem lại mẫu số. Đây chưa thay thế đầy đủ review kỹ thuật độc lập, nên phải ghi giới hạn đó. Khi báo cáo được dùng cho quyết định quan trọng hơn, nhóm cân nhắc bổ sung người review phù hợp.

Không nên chia vai trò bằng công cụ: “biết Python là DS”, “dùng Power BI là BI”, “viết SQL là DE”. Cùng một công cụ có thể phục vụ nhiều trách nhiệm. Cách bàn giao và mức bằng chứng mới cho biết công việc đã hoàn thành tới đâu.

## 7. Bài tập và tự đánh giá

Giả sử nhóm Tới Nơi của bài tập có một DA, một backend và một quản lý vận hành; chưa có DE/BI/DS riêng. Phân công sáu việc: chốt mẫu số, lấy dữ liệu, đối soát, dựng báo cáo, xử lý refresh lỗi và duyệt điều tra cao điểm.

Với mỗi việc, ghi người làm, người kiểm tra, người chốt và đầu ra bàn giao. Nếu không có người đủ năng lực kiểm tra, ghi đó là khoảng trống cần giải quyết, không tự đánh dấu “đã review”.

<details><summary>Gợi ý phân công</summary><p>DA soạn metric và quản lý vận hành xác nhận nghĩa nghiệp vụ; backend giải thích trạng thái nguồn và hỗ trợ trích dữ liệu. DA tính, đối soát và dựng báo cáo. Backend có thể kiểm tra logic dữ liệu nếu đủ năng lực; vận hành kiểm tra ví dụ nghiệp vụ. Phải chốt riêng ai theo dõi refresh, ai thay thế khi vắng mặt. Quản lý vận hành duyệt bước điều tra trong thẩm quyền, không mặc định được duyệt mọi thay đổi ngân sách hay hệ thống.</p></details>

**Câu hỏi:** Vì sao DA không nên tự nhận trách nhiệm “bảo đảm mọi số liệu đều đúng”?

<details><summary>Gợi ý trả lời</summary><p>Độ đúng phụ thuộc định nghĩa, ghi nhận ở nguồn, pipeline và phép tính. DA chịu trách nhiệm kiểm tra và diễn giải trong phạm vi đã thống nhất, đồng thời phải làm rõ phụ thuộc và vấn đề chưa xác minh. Một lời cam kết chung không thay thế trách nhiệm cụ thể của từng khâu.</p></details>

Checklist bàn giao: từng việc có người sở hữu; có người xác nhận nghiệp vụ; có review phù hợp; có giới hạn dữ liệu; có người duy trì báo cáo; quyết định đầu tư hoặc thay đổi có đúng người chốt.

## 8. Đọc thêm và thực hành tiếp

- [Microsoft Learn — Roles in data analytics](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/3-roles): tham khảo phạm vi các vai trò; phân công Tới Nơi trong bài là thiết kế học tập riêng.
- [BA, PO, PM, UX, Developer và Tester phối hợp thế nào?](/articles/ba-po-pm-ux-dev-tester-phoi-hop): góc nhìn phối hợp nghiệp vụ.
- [DA-001: từ số liệu tới quyết định](/articles/da-la-gi-tu-so-lieu-toi-quyet-dinh).

Mẫu thực hành trong repository: `examples/toi-noi-da/p01-nen-tang.md`.

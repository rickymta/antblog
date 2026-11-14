---
slug: da-vong-doi-bai-toan-phan-tich
title: "Vòng đời một bài toán phân tích dữ liệu"
summary: "Nhận yêu cầu làm báo cáo huỷ chuyến rồi bắt đầu từ đâu? Đi qua tám bước phân tích trong Tới Nơi, từ chốt câu hỏi và kiểm tra dữ liệu tới bàn giao, ra quyết định và đo lại; mỗi bước có đầu ra và điều kiện hoàn thành."
category: data-analysis
level: beginner
tags: [data-analysis, da-003, workflow, data-quality, toi-noi]
language: vi
---

Bạn đã biết tỷ lệ huỷ cần có mẫu số đúng và đã xác định được người phối hợp. Vận hành Tới Nơi muốn có kết quả trước cuộc họp tuần tới. Việc còn lại có phải chỉ là lấy dữ liệu rồi vẽ biểu đồ?

Trong một bài phân tích thật, ta có thể phát hiện dữ liệu đến trễ ngay khi tính xong, hoặc nhận ra hai bên hiểu “huỷ” khác nhau ở buổi review. Một quy trình tốt giúp nhóm nhận diện những điểm ấy sớm, biết quay lại bước nào và biết điều gì đã đủ bằng chứng để bàn giao.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-003 · Phần 1/8 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/da-vai-tro-phoi-hop-ba-bi-de-ds">DA-002: vai trò và phối hợp</a>. Đầu ra: checklist vòng đời phân tích và một phiếu bàn giao. Sau bài này, bạn sắp xếp được công việc, xác định đầu ra từng bước và xử lý tình huống phải quay lại kiểm tra dữ liệu.</p>
</div>

## 1. Quy trình cần dẫn tới một quyết định rõ ràng

[Microsoft Learn về các công việc phân tích](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/4-tasks) trình bày các hoạt động chuẩn bị, mô hình hoá, trực quan hoá, phân tích và quản lý tài sản dữ liệu. Trong bài này, ta tổ chức thành tám bước riêng cho ca Tới Nơi, bổ sung việc chốt câu hỏi đầu vào và theo dõi quyết định đầu ra.

```text
Câu hỏi → Định nghĩa → Nguồn dữ liệu → Kiểm tra và chuẩn bị
        → Phân tích → Review → Bàn giao quyết định → Theo dõi
```

Đây là thứ tự để tổ chức công việc, không phải đường đi một chiều. Phân tích có thể làm lộ câu hỏi mới; review có thể phát hiện dữ liệu sai. Khi quay lại, ghi rõ điều đã đổi và phần kết quả phải tính lại. Không cần bắt đầu từ đầu nếu lỗi chỉ ảnh hưởng một lát cắt đã xác định.

## 2. Bước 1 — Chốt câu hỏi và cách dùng câu trả lời

Yêu cầu ban đầu là “huỷ tăng, làm báo cáo giúp anh”. DA cần hỏi: ai dùng báo cáo, để chọn hành động gì và cần kết quả khi nào?

Trong bài tập, câu hỏi được thu hẹp thành: **“Trong cohort lượt đặt xe được nhận ở A, W2 khác W1 về huỷ trước đón thế nào; nhóm nào cần kiểm tra thêm trước khi chọn can thiệp?”**

Đầu ra là một analysis brief ngắn, ghi người nhận, phạm vi, câu hỏi và quyết định được hỗ trợ. Chưa đặt nhiệm vụ “chứng minh chat giúp giảm huỷ”, vì câu đó đã gài sẵn kết luận mong muốn.

Điều kiện qua bước: DA và người ra quyết định hiểu giống nhau về câu hỏi. Nếu chỉ cần biết khối lượng công việc hỗ trợ, hãy bổ sung dữ liệu lượt liên hệ CSKH thay vì lấy số huỷ để thay cho số cuộc gọi.

## 3. Bước 2 — Chốt định nghĩa trước khi so sánh

Ta dùng quy ước DA-P01: mỗi lượt chỉ thuộc một cohort theo thời điểm nhận đầu tiên; tử số là lượt huỷ trước đón trong cohort; mẫu số là toàn bộ lượt đã được nhận trong cohort. Múi giờ là Asia/Ho_Chi_Minh.

W1/W2 hiện là nhãn tuần giả lập, không có ngày lịch cụ thể. “Cao điểm” cũng mới là nhãn của nhóm không giao với nhóm “còn lại”. Để tạo bộ dữ liệu có timestamp thật, vận hành phải xác nhận ngày và khung giờ cụ thể trước; DA không tự điền rồi trình bày như một quy tắc đã có.

Đầu ra là metric contract — bản thoả thuận định nghĩa chỉ số. Ngoài công thức, cần có điều kiện loại trừ, thời điểm chốt kết quả, cách sửa số khi dữ liệu đến trễ và người xác nhận.

Điều kiện qua bước: các bên có thể phân loại cùng một vài ví dụ mà không bất đồng. Nếu backend hiểu “nhận” là mỗi lần gửi offer còn vận hành hiểu là tài xế đồng ý, phải giải quyết khác biệt đó trước khi tính.

## 4. Bước 3 — Xác định nguồn và phạm vi lấy dữ liệu

Với mỗi thông tin cần có, ghi nguồn dự kiến, người phụ trách nguồn và bằng chứng cho thấy nguồn đáp ứng định nghĩa. Chưa có quyền hoặc trường cần thiết thì ghi thiếu, không giả lập trường đó trong kết quả chính thức.

| Thông tin cần | Nguồn cần kiểm tra khi triển khai | Câu hỏi xác minh |
|---|---|---|
| Một lượt duy nhất | Dữ liệu nghiệp vụ chuyến | ID nào xác định lượt, có tái sử dụng không? |
| Thời điểm nhận và kết quả trước đón | Lịch sử trạng thái hoặc sự kiện phù hợp | Nguồn có đủ lịch sử hay chỉ trạng thái cuối? |
| Khu vực và dịch vụ | Thuộc tính nghiệp vụ tại thời điểm phát sinh | Có thay đổi định nghĩa hoặc mapping không? |
| Độ mới của tập dữ liệu | Metadata trích xuất/pipeline | Nguồn và bản sao đã cập nhật tới đâu? |

Bảng này là danh sách yêu cầu cần xác minh, không khẳng định những bảng hoặc cột ấy đã tồn tại trong Tới Nơi. Chỉ trạng thái cuối có thể không đủ để chứng minh một lượt huỷ trước hay sau thời điểm đón.

Ở P01, nguồn thực tế của bài học là **bảng tổng hợp giả lập đã cung cấp**. Ta phân tích được các phép tính trong bảng, nhưng không truy ngược được từng chuyến. Giữ rõ giới hạn đó khi bàn giao.

## 5. Bước 4 — Kiểm tra và chuẩn bị dữ liệu

Khi có dữ liệu chi tiết, kiểm tra ít nhất: thiếu trường bắt buộc, trùng ID theo grain đã chọn, thời gian bất hợp lý, cohort chưa đủ kết quả và phép join làm số dòng bị nhân lên. Lưu quy tắc biến đổi cùng số dòng bị ảnh hưởng.

Đối với bảng tổng hợp P01, ta thực hiện được các kiểm tra nhỏ hơn:

- Mỗi kỳ có đúng hai nhóm không giao nhau theo giả định bài tập.
- Số huỷ không âm và không lớn hơn số được nhận.
- W1: 500 + 500 = 1.000; 80 + 40 = 120.
- W2: 750 + 750 = 1.500; 135 + 45 = 180.
- Tỷ lệ từng ô và tỷ lệ tổng dùng đúng tử số/mẫu số.

Đầu ra là quality log: đã kiểm tra gì, kết quả, điểm chưa xác minh và ảnh hưởng tới kết luận. Không ghi “dữ liệu sạch 100%” chỉ vì phép cộng khớp; bảng tổng hợp vẫn không cho biết hệ thống nguồn đã bỏ sót sự kiện hay chưa.

Nếu phải loại dữ liệu, lưu cả kết quả trước và sau cùng lý do. Đừng lặng lẽ bỏ các ca có thời gian chờ cao vì nghĩ “chắc lỗi”; đó có thể chính là nhóm người dùng đang gặp vấn đề.

## 6. Bước 5 — Phân tích theo câu hỏi, giữ lại cách tính

Trên bộ P01, kết quả cơ bản là số huỷ tăng 50% khi số lượt được nhận cũng tăng 50%, tỷ lệ chung cùng là 12%. Phân nhóm cho thấy tỷ lệ cao điểm tăng từ 16% lên 18%, nhóm còn lại giảm từ 8% xuống 6%.

Lưu lại bảng kết quả, công thức, phiên bản nguồn và các nhóm đã xem. Nếu thử rất nhiều cách chia nhóm rồi chỉ giữ nhóm có biến động lớn nhất, người đọc dễ hiểu nhầm một phát hiện tình cờ là bằng chứng mạnh. Ghi nhóm nào được chọn từ câu hỏi ban đầu, nhóm nào mới nảy ra khi khám phá.

Không tự biến mối liên hệ thành nguyên nhân. Đầu ra phù hợp hiện tại là danh sách nhóm cần điều tra cùng các giả thuyết về dữ liệu, vận hành và sản phẩm. Chưa có cơ sở ước tính chat sẽ giảm bao nhiêu lượt huỷ.

## 7. Bước 6 — Review số và cách diễn giải

Review gồm hai việc khác nhau: kiểm tra phép tính và kiểm tra kết luận có vượt bằng chứng không.

| Câu review | Ví dụ cần phát hiện |
|---|---|
| Tử số và mẫu số có cùng cohort? | Lấy số huỷ xảy ra trong tuần chia cho số lượt nhận trong tuần, dù hai số này gồm các lượt khác nhau |
| Tổng nhóm có khớp số chung? | Một lượt bị đưa vào hai vùng |
| Hai kỳ có cùng điều kiện chốt? | W2 còn nhiều lượt chưa có kết quả |
| Biểu đồ và câu chữ có cùng ý nghĩa? | Ghi “tỷ lệ tăng 50%” trong khi số đếm tăng 50% |
| Khuyến nghị có đòi hỏi bằng chứng mà ta chưa có không? | Đề xuất triển khai chat toàn khu vực từ một bảng phân nhóm |

Một người đọc lại nhận định cùng bảng số là bước hữu ích. Nếu họ không tìm được mẫu số hoặc hiểu “cao điểm gây huỷ” từ câu chữ, cần sửa trước khi bàn giao. Lưu người review, ngày, phạm vi đã xem và vấn đề chưa đóng.

## 8. Bước 7 — Bàn giao để người nhận chọn bước tiếp

Gói bàn giao tối thiểu gồm brief, định nghĩa chỉ số, bảng kết quả, quality log và bản nhận định ngắn. Mở đầu bằng câu trả lời cho câu hỏi, rồi mới trình bày chi tiết kỹ thuật cần để kiểm tra.

Khuyến nghị dự kiến cho ca này: xác minh lịch cao điểm và thay đổi cách ghi nhận, sau đó thu thập thêm bằng chứng về nhóm cao điểm. Đây là khuyến nghị điều tra, chưa phải quyết định duyệt pilot.

Decision log — sổ quyết định — cần ghi người có thẩm quyền, lựa chọn, lý do, trạng thái và bước tiếp. Nếu chưa có người phản hồi, trạng thái vẫn là “đề xuất/chờ quyết định”. Không đổi thành “đã chốt” chỉ vì gửi xong báo cáo.

## 9. Bước 8 — Theo dõi và biết khi nào đóng việc

Nếu báo cáo dùng lặp lại, cần người phụ trách lịch làm mới dữ liệu (refresh), kiểm tra độ mới và xử lý số liệu sửa lại. Nếu chỉ là phân tích một lần, cần lưu gói có thể kiểm tra lại và ghi hạn sử dụng theo giả định.

Phân biệt hai mốc hoàn thành: **hoàn thành phân tích đã giao** và **hoàn thành đánh giá tác động của hành động**. Có thể bàn giao phân tích mô tả khi chưa chạy pilot; không vì thế tuyên bố vấn đề huỷ đã được giải quyết.

Nếu sau đó triển khai can thiệp, nhóm cần thống nhất cách đánh giá riêng: cohort, thời gian, chỉ số chính, chỉ số bảo vệ và thiết kế so sánh. Thấy tỷ lệ giảm sau ngày ra mắt thì chưa đủ để quy toàn bộ mức giảm đó cho can thiệp.

## 10. Khi dữ liệu đến trễ: quay lại bước nào?

**Biến thể độc lập để luyện xử lý sự cố:** sau khi đã gửi báo cáo W2, chủ nguồn thông báo 15 lượt trong nhóm cao điểm trước đây được ghi là chưa có kết quả, nay xác nhận huỷ trước đón. Tất cả 15 lượt đã nằm trong mẫu số 750 của nhóm cao điểm và 1.500 của W2. Trạng thái nhóm còn lại không đổi.

Biến thể này **thay thế giả định đã đủ kết quả của W2 trong bài tập sự cố**, không tự sửa baseline P01. Tử số W2 tăng từ 180 lên 195, mẫu số giữ 1.500; tỷ lệ sửa lại là **13%**. Nhóm cao điểm có 150/750 = **20%**.

DA quay lại kiểm tra dữ liệu và quy tắc chốt, tính lại các đầu ra bị ảnh hưởng, nhờ review rồi gửi bản đính chính cho người đã nhận bản cũ. Quality log phải nêu vì sao lần trước chưa phát hiện; decision log ghi quyết định nào cần xem lại. Giữ cả phiên bản cũ và mới với nhãn rõ ràng để tránh người dùng trộn số.

Không chỉ sửa một ô trên dashboard: câu “tỷ lệ chung không đổi” cũng phải sửa, vì W2 đã cao hơn W1 1 điểm phần trăm trong biến thể này.

## 11. Bài tập: lập checklist có điều kiện hoàn thành

Tạo bảng tám hàng tương ứng tám bước, với các cột: người làm, đầu vào, đầu ra, bằng chứng hoàn thành và việc còn mở. Điền bằng hồ sơ P01; đánh dấu rõ bước nào mới ở mức giả lập.

Sau đó xử lý biến thể 15 lượt trễ. Viết thông báo đính chính trong năm câu, nêu phiên bản bị ảnh hưởng, số cũ/mới, nguyên nhân sửa, kết luận phải thay và bước kiểm tra tiếp.

<details><summary>Gợi ý tự chấm</summary><p>Đạt khi đủ tám bước; có người xác nhận định nghĩa và review; tách bảng minh hoạ khỏi dữ liệu trích xuất thật; tính đúng W2 195/1.500 = 13% và cao điểm 150/750 = 20%; giữ dấu vết phiên bản; nêu cần thông báo lại và xem xét quyết định đã dùng số cũ. Không được thêm 15 vào mẫu số vì các lượt đã ở trong cohort.</p></details>

**Câu hỏi:** Có cần chờ chứng minh được nguyên nhân huỷ mới được bàn giao phân tích không?

<details><summary>Gợi ý trả lời</summary><p>Không, nếu nhiệm vụ đã thống nhất là phân tích mô tả và chọn nhóm cần điều tra. Phải bàn giao đúng phạm vi, nêu điều chưa biết và không trình bày kết quả mô tả như kết luận nhân quả. Nếu nhiệm vụ cần đánh giá tác động, bằng chứng hiện tại chưa đủ để đóng nhiệm vụ đó.</p></details>

Checklist cuối: câu hỏi có người dùng; metric có định nghĩa; nguồn có giới hạn; kiểm tra có bằng chứng; kết quả có thể tính lại; đã review; quyết định có trạng thái; theo dõi có người phụ trách.

## 12. Đọc thêm và hồ sơ thực hành

- [Microsoft Learn — Core analytics tasks](https://learn.microsoft.com/en-us/training/modules/data-analytics-microsoft/4-tasks): nhóm hoạt động nền tảng; quy trình tám bước trong bài là cách tổ chức riêng cho Tới Nơi.
- [DA-001: từ số liệu tới quyết định](/articles/da-la-gi-tu-so-lieu-toi-quyet-dinh).
- [DA-002: vai trò và phối hợp](/articles/da-vai-tro-phoi-hop-ba-bi-de-ds).

Checklist và phiếu bàn giao nằm trong repository: `examples/toi-noi-da/p01-nen-tang.md`. Bài tiếp theo: [DA-004: từ “huỷ chuyến tăng” tới câu hỏi kiểm chứng được](/articles/da-analysis-brief-cau-hoi-kiem-chung).

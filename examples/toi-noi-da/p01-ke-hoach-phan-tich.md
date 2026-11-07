# DA-P01 · Gói kế hoạch phân tích huỷ chuyến

Dùng cho DA-004–DA-010, đọc cùng [baseline P01](p01-nen-tang.md) và [khung series](../../docs/da-master-curriculum.md). Mọi tình huống là giả lập. Đây là mẫu làm bài và dữ kiện kiểm tra; chưa có export nguồn, pilot hoặc quyết định triển khai.

## 1. Quy tắc phiên bản tình huống

| Mã | Dữ kiện | Phạm vi sử dụng |
|---|---|---|
| BASE-P01 | W1: 120/1.000; W2: 180/1.500; đủ kết quả theo giả định đề | Phân tích chính; không đổi số từ các biến thể |
| LATE-15 | Thêm 15 huỷ cao điểm W2, đã trong mẫu số; tổng 195/1.500 | Bài sự cố DA-003, độc lập với MISS-01 |
| MIX-01 | Cơ cấu nhóm H/L đổi, tổng V1 11%, V2 17% | Bài cơ cấu DA-008; không phải W1/W2 |
| MISS-01 | W2 còn 30 kết quả cao điểm chưa biết, trong mẫu số 750/1.500 | Bài bất định DA-009 và tình huống phát sinh DA-010 |

BASE-P01: A là khu vực giả lập chưa mapping; W1/W2 chưa có ngày cụ thể; lịch cao điểm chưa được chốt. Các bảng luyện tính đã cho biết nhóm để tính, nhưng không đủ để sinh timestamp hoặc tra thời tiết thật. Không nhập số baseline/mục tiêu của BA vào gói này.

## 2. Analysis brief · mẫu DA-004

| Trường | Điền nội dung / gợi ý |
|---|---|
| Mã / phiên bản / trạng thái | Ví dụ BRIEF-DA-P01 v0.1, đề xuất |
| Người dùng / quyết định | Có ưu tiên xác minh cao điểm trước khi chọn can thiệp? |
| Câu hỏi chính | So sánh số và tỷ lệ huỷ trước đón cùng cohort |
| Phạm vi | ride, A, W1/W2; ghi điều chưa chốt |
| Ngoài phạm vi | Hiệu quả chat, mọi dịch vụ, doanh thu, thay database |
| Đầu vào | BASE-P01, schema/script chung, câu hỏi mở |
| Đầu ra | Đối soát, nhận định, metric contract, kế hoạch nguồn/giả thuyết |
| Hạn / phụ thuộc | Đề xuất T+3 ngày làm việc sau xác nhận phạm vi và khả năng cấp nguồn; review thiếu hụt cuối T+1 |
| Nghiệm thu | Đúng số, giới hạn rõ, kiểm tra được, mỗi thiếu hụt có bước tiếp |
| Người xác nhận / thời điểm | Để chưa xác nhận nếu không có thông tin |

T là mốc tương đối của bài tập, không phải lịch triển khai đã được duyệt.

## 3. Metric contract M-01 · mẫu DA-005

- ID / version / trạng thái: M-01 v0.1 đề xuất.
- Tên: tỷ lệ huỷ trước đón trong cohort lượt đã được nhận.
- Grain: một lượt đặt xe duy nhất; xác minh khoá và nghĩa nhận lần đầu.
- Cohort: ride tại A, nhận lần đầu trong [đầu kỳ, đầu kỳ kế tiếp), Asia/Ho_Chi_Minh.
- Mẫu số: mọi lượt duy nhất thuộc cohort; không bỏ lượt chưa có kết quả để làm đẹp tỷ lệ.
- Tử số: lượt trong cohort được xác nhận huỷ trước đón.
- Mẫu số 0: không xác định, không biểu diễn thành 0%.
- Loại trừ: chỉ theo quy tắc có bằng chứng, lưu log và số ảnh hưởng.
- Chốt kỳ: BASE-P01 cho giả định đủ kết quả; nguồn thật cần xác nhận điều kiện chốt, lịch sử và độ trễ.
- Phân nhóm: thời điểm nhận theo lịch cao điểm cần chốt; A cần mapping trước khi trích nguồn thật.
- Owner: chủ metric phía vận hành; DA soạn; chưa gán người thật.
- Sửa số: lưu phiên bản nguồn, thời điểm chốt và thông báo kết luận bị ảnh hưởng.

Guardrail đề xuất: lượt có khiếu nại trong cửa sổ đã định; thời gian chờ/kết quả chưa giải quyết; tỷ lệ hoàn thành trên mọi yêu cầu. Chưa có baseline, ngưỡng hay hợp đồng hoàn chỉnh cho chúng. Cần ghi dữ liệu và người xác nhận riêng.

Biến thể tỷ lệ DA-005: X 10/100, Y 180/900 → tổng 190/1.000 = 19%, không phải trung bình không trọng số 15%.

## 4. Lựa chọn phân tích · DA-006

| Câu hỏi | Loại / đầu ra | Giới hạn |
|---|---|---|
| W1/W2 khác nhau thế nào? | Mô tả, bảng và phép tính | Chỉ phạm vi BASE-P01 |
| Vì sao cao điểm khác? | Kế hoạch chẩn đoán, giả thuyết cạnh tranh | Chưa có bằng chứng nguyên nhân |
| Nếu 1.200 lượt và tỷ lệ 16/18/20%? | Kịch bản 192/216/240 huỷ | Không phải khoảng dự báo/tin cậy |
| Nên làm gì tiếp? | So sánh bước xác minh có điều kiện | Chưa duyệt đầu tư hoặc pilot |

## 5. Source inventory · mẫu DA-007

| ID | Nguồn / grain | Bằng chứng hiện có | Người phối hợp dự kiến | Cần xác minh |
|---|---|---|---|---|
| SRC-01 | Bảng tổng hợp kỳ–nhóm | Có trong hồ sơ DA, giả định đủ kết quả | DA | Không có từng chuyến; chỉ kiểm phép tính/đối soát tổng |
| SRC-02 | trips một chuyến; trip_events một sự kiện | Schema/script trong hồ sơ chung, chưa kiểm instance | Backend/chủ dịch vụ | Nghĩa accepted_at; lịch sử đủ; mapping A; sự kiện trễ |
| SRC-03 | Thanh toán: grain chưa chốt | Chưa có export cho ca DA | Chủ thanh toán | Attempt, thu/hoàn, khoá liên kết, thời điểm, quyền |
| SRC-04 | CSKH: ticket/message/lượt cần phân biệt | Chưa có export cho ca DA | Chủ CSKH | Mapping nhiều–nhiều, nhãn, cửa sổ và dữ liệu cần thiết |
| SRC-05 | Tracking/pipeline/change log | Chưa cung cấp | DE/backend | Độ mới, độ phủ, thay đổi schema/quy tắc |

Bổ sung mỗi nguồn: phiên bản, thời điểm trích, nơi lưu, quyền sử dụng, giới hạn lưu giữ và bằng chứng kiểm tra. Không tự điền thông tin không có. Script mẫu sinh trips không chứng minh trip_events có lịch sử đầy đủ; không suy picked_up vắng mặt đồng nghĩa chưa từng đón khi nguồn chưa kiểm độ phủ.

Checklist đối soát: grain/khoá; số lượt nguồn–bản trích; thiếu/trùng; biên kỳ; dữ liệu trễ; lịch sử trạng thái; số dòng sau join; tổng đại lượng trước/sau join. Bài nhân bản: 100 lượt × 2 attempt × 3 ticket = 600 dòng nếu join thẳng đầy đủ; không phải 600 lượt.

## 6. Hypothesis register · mẫu DA-008

| ID | Giả thuyết | Nguồn cần | Điều làm yếu giả thuyết | Owner / trạng thái |
|---|---|---|---|---|
| H-01 | W1 thiếu event cao điểm | SRC-02/05 | Đối soát đủ, cùng định nghĩa/độ phủ | DE/backend; chưa kiểm |
| H-02 | Điểm đón nhiều cổng gây khó | Mẫu ca và quan sát có đối chiếu | Ca đã xác minh không khớp cơ chế giả định | DA/vận hành; chưa kiểm |
| H-03 | Cung có thể phục vụ giảm | Nguồn cung theo vùng/thời gian cần khảo sát | Cung phù hợp ổn định nhưng biến động theo yếu tố khác | Vận hành/DA; chưa kiểm |
| H-04 | Nhãn huỷ đổi cách sử dụng | Form/version và mẫu CSKH | Nhãn/quy trình ổn định qua kiểm tra | Sản phẩm/CSKH; chưa kiểm |

Người học điền thêm: dấu hiệu dự kiến, yếu tố cạnh tranh, phạm vi phép kiểm, chi phí/khả năng tiếp cận, thứ tự ưu tiên và mốc review. Bằng chứng làm yếu không tự bác bỏ tuyệt đối nếu độ phủ không đủ.

### MIX-01 · chỉ dùng cho bài cơ cấu

| Kỳ | H: nhận | H: huỷ | L: nhận | L: huỷ |
|---|---:|---:|---:|---:|
| V1 | 100 | 20 | 900 | 90 |
| V2 | 900 | 162 | 100 | 8 |

H: 20% → 18%; L: 10% → 8%; toàn bộ: 11% → 17%. Chuẩn hoá tỷ lệ V2 theo cơ cấu V1: 0,1 × 18% + 0,9 × 8% = 9%. Đây là mô tả có trọng số tham chiếu, không thay tỷ lệ thực tế 17% và không chứng minh tác động.

## 7. Assumption log và điều kiện dừng · DA-009

| ID | Giả định | Cách kiểm tra | Trạng thái trong hồ sơ |
|---|---|---|---|
| AS-01 | Hai kỳ cùng định nghĩa | Chủ metric + backend review | Chưa xác nhận trên nguồn thật |
| AS-02 | Một lượt chỉ đếm một lần | Kiểm khoá/grain và join | Được giả định ở bảng BASE-P01 |
| AS-03 | Kết quả đã đủ lúc chốt | Đối soát nguồn và pipeline | Giả định BASE-P01, bị thay trong MISS-01 |
| AS-04 | Phạm vi phù hợp quyết định | Review brief với người dùng | Chỉ A/ride; không suy rộng toàn hệ thống |

Người học bổ sung hậu quả nếu sai, owner và thời điểm kiểm tra. Dừng chốt kết luận nếu mẫu số chưa rõ, nguồn mâu thuẫn hoặc phần thiếu có thể đổi quyết định; bàn giao báo cáo giới hạn thay vì giả vờ đã giải quyết. Quyết định rollout/hiệu quả can thiệp cần thiết kế và bằng chứng khác với mô tả.

### MISS-01 · chỉ dùng cho bất định và capstone phát sinh

30 lượt cao điểm W2 chưa rõ kết quả, đã trong mẫu số 750 cao điểm và 1.500 tổng. Huỷ đã xác nhận: 135 cao điểm + 45 còn lại = 180. Các lượt khác coi như đã xác minh theo giả định biến thể.

- Toàn W2: 180–210 huỷ, tỷ lệ 12–14%.
- Cao điểm: 135–165 huỷ, tỷ lệ 18–22%.
- Không phải khoảng tin cậy; chỉ giới hạn kịch bản theo giả định.
- Ngưỡng luyện tính **lớn hơn 13%**: 195/1.500 = 13%, cần thêm ít nhất 16 huỷ để vượt (196/1.500 ≈ 13,07%). Ngưỡng chưa được duyệt cho vận hành.
- Nếu biết thêm 10 huỷ, 10 không huỷ, còn 10 chưa biết: tổng 190–200, tỷ lệ khoảng 12,67–13,33%; cần ít nhất 6 huỷ trong 10 lượt còn lại để vượt ngưỡng.
- Không cộng thêm LATE-15 vào MISS-01.

## 8. Decision log · mẫu DA-010

| ID | Quyết định | Người chốt dự kiến | Trạng thái | Bước tiếp |
|---|---|---|---|---|
| D-DA-01 | Phạm vi và metric có phù hợp? | Chủ quyết định/metric | Đề xuất | Xác nhận câu hỏi, A, lịch giờ và chốt kỳ |
| D-DA-02 | Có dành nguồn lực xác minh cao điểm? | Người quản lý nguồn lực | Chờ xem kế hoạch | Xem nguồn, người làm, phụ thuộc và hạn |
| D-DA-03 | Có thiết kế pilot chat/điểm đón? | Sản phẩm/vận hành theo thẩm quyền | Chưa đủ căn cứ để chốt | Xác minh vấn đề, phương án, đo lường và guardrail |

Gói nộp DA-010: 4–6 trang hoặc tương đương; brief; M-01; inventory; bảng số; hypothesis/assumption log; trình tự làm; decision log; phụ lục xử lý MISS-01. Giữ trạng thái đề xuất nếu chưa có xác nhận.

Rubric 100 điểm: brief 15; metric 20; nguồn 15; tính/diễn giải 15; giả thuyết/bất định 15; bàn giao 10; MISS-01 10. Mục tiêu ≥80 và không lỗi trọng yếu. Lỗi trọng yếu: bịa bằng chứng, đổi mẫu số, quy hiệu quả chat từ mô tả, trộn biến thể, ghi phê duyệt chưa có. Phải sửa trước khi coi hoàn thành.

## 9. Đáp án số để đối soát

BASE-P01: W1 12%, W2 12%; huỷ tăng 60 (50%); cao điểm 16% → 18% (+2 điểm phần trăm, +12,5% tương đối); còn lại 8% → 6% (−2 điểm phần trăm); tỷ trọng nhóm 50% ở cả hai kỳ. Nhận định hợp lệ: ưu tiên xác minh cao điểm và cách đo; chưa có nguyên nhân hoặc hiệu quả chat được chứng minh.

Sau khi hoàn thành P01, viết dataset SQL nhỏ và kiểm tra có thể chạy cho P02; không coi bảng tổng hợp này là lịch sử chuyến đã tồn tại.

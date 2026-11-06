# P03 · Hồ sơ mô hình quy trình và quy tắc

Hồ sơ học tập cho BA-017–BA-024. Không sửa SPEC, code, phí hay trạng thái hiện hành của lab. P02 vẫn có giả thuyết chưa xác nhận; không biến chúng thành sự thật khi vẽ as-is.

## 1. Nguồn và phạm vi

- Hồ sơ chung: `api/seed/content/du-an-mau-toi-noi.md`, vòng đời 8 trạng thái.
- Nguồn chi tiết: `examples/toi-noi-testing/SPEC.md`, R-STATE-01–07, R-CANCEL-01–07, R-PAY-02/04/05/06, R-IDEM-01–05. Đây là **baseline đặc tả lab được đọc tại thời điểm soạn P03**, không phải bằng chứng đã quan sát production hoặc chứng nhận code thực thi đúng.
- Nguồn discovery: `p02-discovery-ho-so.md`. Luồng hành vi người dùng vẫn phải gắn nhãn quan sát/lời kể/giả thuyết.
- Phạm vi mô hình chính: khách chủ chuyến yêu cầu huỷ; tách khỏi tài xế rời chuyến và hệ thống hết thời gian tìm tài xế.

## 2. Baseline cần giữ nguyên

Trạng thái: `requested`, `searching`, `accepted`, `arriving`, `picked_up`, `in_transit`, `completed`, `cancelled`.

- Khách huỷ được ở `searching`, `accepted`, `arriving`; chuyển sang `cancelled`.
- Tài xế được gán rời chuyến ở `accepted`/`arriving`: về `searching`, không thu phí khách; xoá gán tài xế, `acceptedAt`, ETA, đặt lại `searchingSince`.
- Hệ thống huỷ ở `searching` khi chờ tìm tài xế từ 5 phút; lab quét định kỳ 15 giây, nên không cam kết huỷ chính xác tại mili giây thứ 300.000.
- Khách huỷ `searching`: 0 đ. Ở `accepted`, miễn phí nếu `now - acceptedAt < 2 phút`, từ 2 phút có phí. `arriving` có phí cơ sở.
- Miễn do tài xế trễ: `now > acceptedAt + etaMinutes + 10 phút`; đúng bằng vẫn chưa được miễn theo điều kiện này.
- Phí cơ sở: xe máy (gồm đơn đồ ăn) 10.000 đ; ô tô 20.000 đ.
- Phí ví thu thất bại không đảo trạng thái chuyến: vẫn `cancelled`, `paymentStatus = failed`. Tiền mặt ghi phí để thu sau, ngoài phạm vi thu tiền của lab. Phí 0: `paymentStatus = none`.

## 3. CR-BA-01 — biến thể đề xuất, chưa được duyệt

Mở rộng cửa sổ miễn phí ở **accepted** từ dưới 2 phút thành dưới 3 phút. Không đổi `arriving`, ngưỡng trễ, mức tiền, quyền huỷ hoặc hành vi tài xế rời chuyến.

Đề xuất version cho bài tập: `cancel-v1` (2 phút), `cancel-v2` (3 phút). Cách gắn version theo lần nhận chuyến là **phương án để phân tích**, không phải trường đã tồn tại hoặc quyết định triển khai. Cần chốt hiệu lực, trường hợp nhận lại, thông báo, replay idempotency và dữ liệu còn thiếu trước khi duyệt.

## 4. Bộ ca walkthrough

Các ca khách huỷ dưới đây mặc định chủ chuyến đúng quyền, xe máy, acceptedAt = 10:00:00, ETA = 5 phút, dữ liệu hợp lệ, không có replay; mỗi ca độc lập. Mốc trễ là 10:15:00. Bảng chỉ so phí dự kiến nếu áp dụng từng version; không khẳng định v2 đã dùng thật.

| Case | Trạng thái / thời điểm | v1 | v2 đề xuất |
|---|---|---:|---:|
| C01 | accepted / 10:01:59 | 0 | 0 |
| C02 | accepted / 10:02:00 | 10.000 | 0 |
| C03 | accepted / 10:02:59 | 10.000 | 0 |
| C04 | accepted / 10:03:00 | 10.000 | 10.000 |
| C05 | arriving / 10:01:30 | 10.000 | 10.000 |
| C06 | arriving / 10:15:00 | 10.000 | 10.000 |
| C07 | arriving / 10:15:01 | 0 | 0 |
| C08 | picked_up / 10:03:00 | Từ chối | Từ chối |
| C09 | searching, không có acceptedAt | 0 | 0 |
| C10 | driver_cancel từ accepted | Về searching, không thu khách | Không đổi |

Ca mở rộng: C11 ô tô accepted 10:03:00 → 20.000 đ; C12 khách huỷ có phí qua ví nhưng ví declined → cancelled/failed, không thử lại declined; C13 gửi lại cùng khoá và cùng nội dung sau khi đã xử lý → trả lại kết quả cũ, không tính phí theo version mới; C14 accepted nhưng thiếu acceptedAt → dữ liệu bất thường cần quyết định xử lý, không mặc định miễn phí.

## 5. Mẫu truy vết và impact log

| ID mô hình/rule | Nguồn | Trạng thái baseline/đề xuất | Dữ liệu đầu vào | Case kiểm | Owner cần xác nhận |
|---|---|---|---|---|---|
| | | | | | |

| Hạng mục bị tác động | Trước/sau CR-BA-01 | Rủi ro | Quyết định còn mở | Người chốt / hạn |
|---|---|---|---|---|
| | | | | |

## 6. Điều kiện tự kiểm

Mỗi đường kết thúc có outcome; mỗi gateway có nhánh rõ; mỗi rule có nguồn/version; mỗi transition có actor/guard/effect; decision table không lẫn từ chối với phí 0. Sơ đồ chỉ là mô hình phân tích, không tự chứng minh tính đúng của engine/code. Đối chiếu đồng thời bài viết, bảng và sơ đồ khi thay đổi.

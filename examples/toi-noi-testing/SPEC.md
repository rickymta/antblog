# Đặc tả Tới Nơi — bản lab kiểm thử (SPEC v1.1)

Tài liệu này là **test basis** của phòng thí nghiệm `toi-noi-testing`: mọi ca kiểm thử của loạt bài «Kiểm thử phần mềm theo ISTQB qua dự án Tới Nơi» suy ra từ đây. Mỗi yêu cầu có một mã cố định (`R-…`); bài viết và ca kiểm thử trích dẫn mã này. Hồ sơ nghiệp vụ đầy đủ của Tới Nơi: bài «Dự án mẫu Tới Nơi».

Phạm vi bản lab: báo giá có thời hạn và đặt chuyến cho ba dịch vụ (đặt xe, giao bưu kiện, giao đồ ăn), tài xế nhận và thực hiện chuyến, huỷ chuyến và phí huỷ, thu tiền qua ví «VíXanh», API HTTP có idempotency và một giao diện web tối giản. Ngoài phạm vi: bản đồ thật, điều phối chủ động (gửi offer cho tài xế), theo dõi vị trí, đánh giá, khuyến mãi cho tài xế.

Quy ước trình bày: số tiền viết theo kiểu Việt Nam (`12.000 đ`), số thập phân dùng dấu phẩy (`7,4 km`). Trong JSON, số tiền là số nguyên đồng, quãng đường/khối lượng là số thực với dấu chấm (`7.4`).

## 0. Vị trí của bản lab trong kiến trúc Tới Nơi (R-ARCH)

Kiến trúc đích của Tới Nơi (tài liệu «Đề xuất kiến trúc tổng thể») là các microservice nối bằng REST/gRPC và Kafka. Bản lab là **một modular monolith kiểu giai đoạn MVP** (mục 18 của tài liệu kiến trúc): một tiến trình, nhưng giữ ranh giới module và hợp đồng rõ ràng.

- **R-ARCH-01 — Module.**

  | Module của lab | Ứng với service đích | Mã nguồn |
  |---|---|---|
  | Pricing & Quote | Pricing & Quote Service (§4.3) | `src/domain/fare.ts`, `parcel.ts`, `food.ts`, `pricing.ts`, `src/app/quote.ts` |
  | Promotion | Promotion Service (§4.10) | `src/domain/voucher.ts` |
  | Booking | Booking / Order Service (§4.4) | `src/app/trips.ts` (`createTrip`, `cancelByCustomer`), `src/api/idempotency.ts` |
  | Dispatch | Matching & Dispatch + Driver Availability (§4.5, §4.6) | `src/app/trips.ts` (`acceptTrip`, `listAvailableTrips`) |
  | Trip lifecycle | Trip / Delivery Lifecycle Service (§4.7) | `src/domain/trip-state.ts`, `src/domain/cancel.ts`, `driverStep` |
  | Payment adapter | Payment & Wallet Service (§4.9) | `src/app/payment.ts`, `src/infra/fake-payment-gateway.ts` |

- **R-ARCH-02 — Ngoài phạm vi của lab** (có trong kiến trúc đích, KHÔNG có ở đây; bài viết nhắc tới thì ghi rõ «chưa có trong lab»): Kafka và sự kiện (`BookingCreated`, `DriverAssigned`…), transactional outbox/inbox, xử lý sự kiện trùng/sai thứ tự, Redis (lease, cache ứng viên), realtime gateway (khách xem trạng thái bằng cách hỏi lại mỗi 2 giây), offer có TTL, API Gateway/BFF, webhook và ledger của thanh toán, xác thực bằng token thật.
- **R-ARCH-03 — Móc kiểm thử** (theo §17.2 của tài liệu kiến trúc): đồng hồ tiêm vào (`Clock`), thời tiết giả, ví VíXanh giả có chế độ lỗi, bản đồ giả lập bằng quãng đường do client gửi (R-GEN-06), điều phối tất định (R-ACCEPT-06), API dữ liệu kiểm thử chỉ bật ở môi trường test (R-TEST).

---

## 1. Quy định chung (R-GEN)

- **R-GEN-01 — Đơn vị tiền.** Mọi số tiền tính bằng đồng (VND). Các bước trung gian của công thức cước được tính **chính xác, không làm tròn**; chỉ bước cuối cùng của R-FARE-03 làm tròn (R-FARE-08). Số tiền khách phải trả luôn là số nguyên đồng.
- **R-GEN-02 — Giờ nghiệp vụ.** Mọi quy tắc theo giờ hoặc theo ngày (giờ cao điểm, ngày hiệu lực voucher, lượt dùng voucher trong ngày) dùng **giờ Việt Nam (UTC+7, không có giờ mùa hè)**, bất kể múi giờ của máy chủ. «Thời điểm đặt» là thời điểm hệ thống nhận yêu cầu báo giá hoặc tạo chuyến (theo đồng hồ của hệ thống, khách không tự chọn).
- **R-GEN-03 — Thành phố.** Hệ thống phục vụ 6 thành phố, mã: `SG` (TP. Hồ Chí Minh), `HN` (Hà Nội), `DN` (Đà Nẵng), `HP` (Hải Phòng), `CT` (Cần Thơ), `NT` (Nha Trang). Mã khác ⇒ lỗi `CITY_NOT_SUPPORTED`.
- **R-GEN-04 — Dịch vụ và loại xe.** Dịch vụ (`kind`): `ride` (đặt xe), `parcel` (giao bưu kiện), `food` (giao đồ ăn). Loại xe (`vehicle`): `bike` (xe máy), `car` (ô tô). Đơn đồ ăn luôn do xe máy giao.
- **R-GEN-05 — Thời tiết.** Trạng thái «đang mưa» của một thành phố do dịch vụ thời tiết cung cấp tại thời điểm đặt; khách không tự khai.
- **R-GEN-06 — Quãng đường.** Trong hệ thống đích, Pricing tính quãng đường từ điểm đón/điểm đến qua bản đồ MapViet. **Giản lược của lab:** client gửi thẳng quãng đường (`distanceKm`) như thể đã lấy từ MapViet; lab không có điểm đón/điểm đến.

## 2. Cước đặt xe và bưu kiện (R-FARE)

- **R-FARE-01 — Bảng giá theo loại xe.**

  | Loại xe | Phí mở cửa | Giá mỗi km | Cước tối thiểu |
  |---|---|---|---|
  | `bike` | 12.000 đ | 4.500 đ | 15.000 đ |
  | `car` | 25.000 đ | 11.000 đ | 30.000 đ |

  Cước cơ bản = phí mở cửa + giá mỗi km × quãng đường (km, sau khi làm tròn theo R-FARE-02).
- **R-FARE-02 — Quãng đường.** Quãng đường do khách gửi được làm tròn tới **0,1 km theo kiểu half-up** (0,45 ⇒ 0,5; 0,44 ⇒ 0,4; 7,35 ⇒ 7,4) **trước** mọi bước khác. Quãng đường sau khi làm tròn phải nằm trong **[0,5; 100] km, tính cả hai đầu**; ngoài khoảng ⇒ lỗi `DISTANCE_OUT_OF_RANGE`. Ví dụ: 0,45 hợp lệ (thành 0,5); 100,04 hợp lệ (thành 100,0); 100,05 không hợp lệ (thành 100,1).
- **R-FARE-03 — Thứ tự tính (cố định).**
  1. Cước cơ bản (R-FARE-01), cộng phụ phí khối lượng nếu là bưu kiện (R-PARCEL-03).
  2. Nhân hệ số cao điểm nếu thuộc giờ cao điểm (R-FARE-04).
  3. Cộng phụ phí mưa nếu đang mưa (R-FARE-05).
  4. Trừ tiền giảm của voucher nếu có (R-FARE-06, R-VOUCHER).
  5. Nâng lên cước tối thiểu của loại xe nếu thấp hơn (R-FARE-07).
  6. Làm tròn lên bội số 1.000 đ (R-FARE-08).
- **R-FARE-04 — Giờ cao điểm.** Thời điểm đặt (giờ Việt Nam) thuộc **[11:00; 13:00)** hoặc **[17:00; 19:00)** ⇒ nhân **1,25**. 11:00:00 là cao điểm; 12:59:59 là cao điểm; 13:00:00 không phải cao điểm.
- **R-FARE-05 — Phụ phí mưa.** Thành phố đang mưa tại thời điểm đặt ⇒ cộng **8.000 đ** (cộng sau hệ số cao điểm, nên không bị nhân 1,25).
- **R-FARE-06 — Bước voucher.** Tiền giảm tính trên số tiền **sau bước 3** (đã gồm cao điểm và mưa) theo R-VOUCHER-06.
- **R-FARE-07 — Cước tối thiểu.** Áp sau voucher: nếu số tiền sau bước 4 nhỏ hơn cước tối thiểu của loại xe (R-FARE-01) thì lấy cước tối thiểu. Voucher không bao giờ kéo cước xuống dưới mức tối thiểu.
- **R-FARE-08 — Làm tròn.** Kết quả bước 5 làm tròn **lên** bội số gần nhất của 1.000 đ (51.700 ⇒ 52.000; 52.000 giữ nguyên; 52.001 ⇒ 53.000).
- **R-FARE-09 — Ví dụ kiểm (oracle).** Xe máy, 7,4 km, giờ cao điểm, đang mưa, voucher giảm 20% (trần đủ lớn):
  12.000 + 4.500 × 7,4 = **45.300** ⇒ × 1,25 = **56.625** ⇒ + 8.000 = **64.625** ⇒ − 12.925 (20%) = **51.700** ⇒ ≥ 15.000 nên giữ ⇒ làm tròn lên **52.000 đ**.
- **R-FARE-10 — Bảng chi tiết cước.** Kết quả báo giá trả kèm từng bước: phí mở cửa, tiền theo km, phụ phí bưu kiện, cước cơ bản, sau cao điểm, phụ phí mưa, trước giảm, tiền giảm, sau giảm, cước tối thiểu, tổng. Các giá trị trung gian có thể có phần lẻ dưới 1 đồng (ví dụ 17.812,5) vì R-GEN-01 không làm tròn giữa chừng.

## 3. Bưu kiện (R-PARCEL)

- **R-PARCEL-01 — Khối lượng.** Khối lượng do khách gửi được làm tròn tới **0,1 kg half-up** rồi phải thoả **0 < w ≤ 30 kg**; ngược lại ⇒ lỗi `WEIGHT_OUT_OF_RANGE`. Ví dụ: 0,04 ⇒ 0,0 (không hợp lệ); 0,05 ⇒ 0,1 (hợp lệ); 30,04 ⇒ 30,0 (hợp lệ); 30,05 ⇒ 30,1 (không hợp lệ). Đơn bưu kiện bắt buộc có khối lượng.
- **R-PARCEL-02 — Loại xe theo khối lượng.** w > 20 kg bắt buộc dùng ô tô; chọn xe máy ⇒ lỗi `VEHICLE_TOO_SMALL`. w = 20,0 kg vẫn đi xe máy được.
- **R-PARCEL-03 — Phụ phí khối lượng.** Phần vượt 5 kg tính **2.000 đ cho mỗi kg bắt đầu** (kg chưa trọn tính là một kg): w ≤ 5 ⇒ 0 đ; 5,1 ⇒ 2.000 đ; 6,0 ⇒ 2.000 đ; 6,1 ⇒ 4.000 đ; 30 ⇒ 50.000 đ. Công thức: `2.000 × ⌈w − 5⌉` khi w > 5.
- **R-PARCEL-04 — Vị trí trong công thức.** Phụ phí khối lượng cộng vào cước cơ bản (bước 1 của R-FARE-03), nên **bị nhân** hệ số cao điểm.

## 4. Giao đồ ăn (R-FOOD)

- **R-FOOD-01 — Tổng đơn.** Khách trả = (tiền món − tiền giảm voucher) + phí giao.
- **R-FOOD-02 — Món.** Đơn có 1–20 dòng món; mỗi dòng có tên (1–100 ký tự), đơn giá là số nguyên **bội số của 1.000 đ** trong [1.000; 5.000.000], số lượng nguyên trong [1; 50]. Sai ⇒ lỗi kiểm tra đầu vào `VALIDATION_ERROR`. Tiền món = Σ đơn giá × số lượng.
- **R-FOOD-03 — Tiền món tối thiểu.** Tiền món < **30.000 đ** ⇒ lỗi `SUBTOTAL_TOO_LOW`. Đúng 30.000 đ là hợp lệ.
- **R-FOOD-04 — Phí giao.** Phí giao tính theo **công thức xe máy** của R-FARE trên quãng đường quán → khách (R-FARE-02 áp dụng), có cao điểm và mưa, có cước tối thiểu 15.000 đ và làm tròn lên 1.000 đ; **voucher không trừ vào phí giao**.
- **R-FOOD-05 — Voucher cho đơn đồ ăn.** Tiền giảm tính trên **tiền món** (R-VOUCHER-06); điều kiện tối thiểu của voucher so với tiền món. Tiền giảm luôn là số nguyên đồng (vì đơn giá là bội số 1.000 đ).
- **R-FOOD-06 — Loại xe.** Đơn đồ ăn chỉ nhận `vehicle` bỏ trống hoặc `bike`; gửi `car` ⇒ `VALIDATION_ERROR`.

## 5. Voucher (R-VOUCHER)

- **R-VOUCHER-01 — Thuộc tính.** `code`; `percent` (số nguyên 1–50); `maxDiscount` (trần tiền giảm, đồng); `minAmount` (đồng, số tiền tối thiểu để được dùng); `kinds` (danh sách dịch vụ áp dụng); `cityCodes` (danh sách thành phố áp dụng); `validFrom`, `validTo` (ngày theo giờ Việt Nam, **tính cả hai đầu**); `perCustomerPerDay` (số lần mỗi khách được dùng trong một ngày, mặc định 1). `kinds` hoặc `cityCodes` rỗng nghĩa là áp dụng cho mọi dịch vụ / mọi thành phố.
- **R-VOUCHER-02 — Chuẩn hoá mã.** Mã khách nhập được bỏ khoảng trắng hai đầu và đổi sang chữ hoa trước khi tra (` giam20 ` ≡ `GIAM20`). Chuỗi rỗng sau khi bỏ khoảng trắng coi như không nhập voucher.
- **R-VOUCHER-03 — Tồn tại.** Không có voucher với mã đó ⇒ `VOUCHER_NOT_FOUND`.
- **R-VOUCHER-04 — Thời hạn.** Ngày đặt (giờ Việt Nam) nằm ngoài [validFrom; validTo] (chưa tới hoặc đã qua) ⇒ `VOUCHER_EXPIRED`.
- **R-VOUCHER-05 — Phạm vi.** Dịch vụ không thuộc `kinds` hoặc thành phố không thuộc `cityCodes` ⇒ `VOUCHER_NOT_APPLICABLE`.
- **R-VOUCHER-06 — Tiền giảm.** Gọi A là số tiền áp voucher: với đặt xe/bưu kiện là số tiền sau bước 3 của R-FARE-03; với đồ ăn là tiền món. Nếu A < `minAmount` ⇒ `VOUCHER_MIN_AMOUNT` (A = `minAmount` là hợp lệ). Tiền giảm = **min(percent% × A, maxDiscount)**.
- **R-VOUCHER-07 — Lượt dùng.** Số chuyến **chưa bị huỷ** mà khách đã tạo với mã đó trong cùng ngày (giờ Việt Nam) phải nhỏ hơn `perCustomerPerDay`; ngược lại ⇒ `VOUCHER_USAGE_LIMIT`. Báo giá không tiêu lượt; tạo chuyến mới tiêu lượt. Chuyến bị huỷ (bởi khách hoặc hệ thống) trả lại lượt. Lượt dùng được kiểm **hai lần**: khi báo giá và khi tạo chuyến từ báo giá (ngày tính theo thời điểm của từng lần kiểm). Thời hạn và phạm vi voucher chỉ kiểm khi báo giá.
- **R-VOUCHER-08 — Thứ tự kiểm tra.** Khi có nhiều lỗi voucher, chỉ báo lỗi đầu tiên theo thứ tự: `VOUCHER_NOT_FOUND` → `VOUCHER_EXPIRED` → `VOUCHER_NOT_APPLICABLE` → `VOUCHER_MIN_AMOUNT` → `VOUCHER_USAGE_LIMIT`.

## 6. Báo giá (R-QUOTE)

- **R-QUOTE-01 — Báo giá có thời hạn.** Mỗi lần báo giá thành công tạo một báo giá được lưu, có `quoteId`, `createdAt`, `expiresAt = createdAt + 5 phút`, `pricingVersion` và bảng chi tiết giá. Báo giá còn hiệu lực khi **thời điểm hiện tại < `expiresAt`**; đúng bằng `expiresAt` là đã hết hạn.
- **R-QUOTE-02 — Phiên bản giá.** `pricingVersion` là tên bộ quy tắc giá đã dùng (bản hiện tại: `fare-2026.1`) và được lưu cùng báo giá và chuyến.
- **R-QUOTE-03 — Đặt chuyến từ báo giá.** Tạo chuyến phải gửi `quoteId` của **chính khách đó**. Kiểm theo thứ tự: không có báo giá hoặc báo giá của khách khác ⇒ `QUOTE_NOT_FOUND` (422) → hết hạn ⇒ `QUOTE_EXPIRED` (422) → đã dùng cho một chuyến khác ⇒ `QUOTE_ALREADY_USED` (409) → hết lượt voucher (R-VOUCHER-07) ⇒ `VOUCHER_USAGE_LIMIT` (422). Mỗi báo giá đặt được **tối đa một chuyến**, kể cả khi hai yêu cầu tới cùng lúc.
- **R-QUOTE-04 — Không đổi giá hồi tố.** Chuyến lưu ảnh chụp giá của báo giá (`fare`, `fareBreakdown`, `discount`, `peak`, `raining`, `pricingVersion`). Giờ cao điểm, thời tiết hay voucher thay đổi sau khi báo giá **không** làm đổi giá của chuyến.

## 7. Thứ tự kiểm tra yêu cầu (R-VALID)

- **R-VALID-01 — Báo giá.** Khi một yêu cầu báo giá vi phạm nhiều quy tắc, chỉ báo lỗi đầu tiên theo thứ tự: xác thực (401) → vai trò (403) → định dạng đầu vào `VALIDATION_ERROR` (400) → `CITY_NOT_SUPPORTED` → `DISTANCE_OUT_OF_RANGE` → `WEIGHT_OUT_OF_RANGE` → `VEHICLE_TOO_SMALL` → `SUBTOTAL_TOO_LOW` → lỗi voucher theo R-VOUCHER-08.
- **R-VALID-02 — Tạo chuyến.** Xác thực (401) → vai trò (403) → giới hạn tần suất (429, R-API-13) → idempotency (R-IDEM: thiếu khoá 400, khoá dùng lại khác nội dung 422, đang xử lý 409, hoặc trả lại kết quả cũ) → định dạng đầu vào (400) → lỗi báo giá theo R-QUOTE-03.

## 8. Vòng đời chuyến (R-STATE)

- **R-STATE-01 — Trạng thái.** `requested`, `searching`, `accepted`, `arriving`, `picked_up`, `in_transit`, `completed`, `cancelled`. `completed` và `cancelled` là trạng thái kết thúc.
- **R-STATE-02 — Chuyển hợp lệ.**

  | Từ | Sự kiện | Tới | Ai gây ra |
  |---|---|---|---|
  | `requested` | `search` | `searching` | hệ thống, ngay khi tạo chuyến |
  | `searching` | `accept` | `accepted` | tài xế (R-ACCEPT) |
  | `accepted` | `arrive` | `arriving` | tài xế được gán |
  | `arriving` | `pickup` | `picked_up` | tài xế được gán |
  | `picked_up` | `start` | `in_transit` | tài xế được gán |
  | `in_transit` | `complete` | `completed` | tài xế được gán |
  | `searching`, `accepted`, `arriving` | `customer_cancel` | `cancelled` | khách chủ chuyến |
  | `accepted`, `arriving` | `driver_cancel` | `searching` | tài xế được gán |
  | `searching` | `system_cancel` | `cancelled` | hệ thống (R-STATE-05) |

- **R-STATE-03 — Chuyển không hợp lệ.** Mọi cặp (trạng thái, sự kiện) không có trong bảng R-STATE-02 bị từ chối với `INVALID_TRANSITION` (HTTP 409); chuyến giữ nguyên.
- **R-STATE-04 — Tài xế huỷ.** Chuyến về `searching`, bỏ gán tài xế, xoá `acceptedAt` và ETA; tài xế trở lại `online`; mốc bắt đầu tìm tài xế (`searchingSince`) đặt lại bằng thời điểm huỷ. Khách không mất phí (R-CANCEL-05).
- **R-STATE-05 — Hết thời gian tìm tài xế.** Chuyến ở `searching` mà **thời điểm hiện tại − `searchingSince` ≥ 5 phút** bị hệ thống huỷ với lý do `no_driver`, phí huỷ 0 đ. Hệ thống quét các chuyến như vậy định kỳ (máy chủ lab: mỗi 15 giây).
- **R-STATE-06 — Nhật ký trạng thái.** Mỗi lần chuyến đổi trạng thái ghi một dòng `trip_events` (trạng thái mới, thời điểm). Tạo chuyến ghi hai dòng: `requested` rồi `searching`, cùng thời điểm.
- **R-STATE-07 — Mốc thời gian.** `acceptedAt` ghi khi nhận; `completedAt` khi hoàn thành; `cancelledAt` khi huỷ (khách hoặc hệ thống).
- **R-STATE-08 — Đối chiếu với trạng thái theo domain của kiến trúc đích (§9.1).** Lab giữ **một** vòng đời 8 trạng thái (như hồ sơ dự án); hệ thống đích tách trạng thái theo từng service. Bảng đối chiếu (tham khảo, không phải yêu cầu kiểm thử):

  | Trạng thái lab | Booking | Dispatch | Trip | Payment (chuyến `vixanh`) |
  |---|---|---|---|---|
  | `requested` | `CREATED` | — | — | — |
  | `searching` | `PENDING_DISPATCH` | `SEARCHING` | — | — |
  | `accepted` | `ASSIGNED` | `ASSIGNED` | `DRIVER_EN_ROUTE` | — |
  | `arriving` | `ASSIGNED` | `ASSIGNED` | `DRIVER_EN_ROUTE` → `DRIVER_ARRIVED` | — |
  | `picked_up`, `in_transit` | `ASSIGNED` | `ASSIGNED` | `IN_PROGRESS` | — |
  | `completed` | `COMPLETED` | — | `COMPLETED` | `INITIATED` → `CAPTURED` / `FAILED` (`paymentStatus` = `paid` / `failed`) |
  | `cancelled` (lý do `no_driver`) | `CANCELLED` | `EXPIRED` | — | — |
  | `cancelled` (khách huỷ) | `CANCELLED` | — | `CANCELLED` nếu đã có tài xế | phí huỷ nếu có: `CAPTURED` / `FAILED` |

  Lab không có trạng thái `OFFERED` (không gửi offer — R-ACCEPT-06) và không có hoàn tiền (`REFUND_*`).

## 9. Tài xế nhận chuyến (R-ACCEPT)

- **R-ACCEPT-01 — Điều kiện.** Tài xế nhận được chuyến khi đồng thời: đang `online` (không phải `offline`, không phải `busy`); cùng thành phố với chuyến; đúng loại xe của chuyến; chuyến đang `searching`.
- **R-ACCEPT-02 — Thứ tự kiểm tra và mã lỗi.** Kiểm tra theo thứ tự, báo lỗi đầu tiên: chuyến không tồn tại ⇒ `TRIP_NOT_FOUND` (404; riêng lệnh nhận chỉ trả 404 khi chuyến không tồn tại, không áp quy tắc xem của R-API-05) → tài xế `offline` ⇒ `DRIVER_OFFLINE` (409) → tài xế `busy` ⇒ `DRIVER_BUSY` (409) → khác thành phố ⇒ `CITY_MISMATCH` (422) → khác loại xe ⇒ `VEHICLE_MISMATCH` (422) → chuyến đã `cancelled` ⇒ `INVALID_TRANSITION` (409) → chuyến không còn `searching` vì đã có tài xế ⇒ `TRIP_ALREADY_TAKEN` (409).
- **R-ACCEPT-03 — Một người thắng.** Khi nhiều tài xế cùng nhận một chuyến (đồng thời), **đúng một** người thành công; những người còn lại nhận `TRIP_ALREADY_TAKEN` và trạng thái của họ giữ `online`. Một tài xế cũng không thể giữ hai chuyến đang chạy: hai lệnh nhận đồng thời của cùng một tài xế cho hai chuyến khác nhau ⇒ đúng một thành công, lệnh kia `DRIVER_BUSY`.
- **R-ACCEPT-04 — ETA.** Lệnh nhận gửi `etaMinutes` (số nguyên 1–60, thời gian dự kiến tới điểm đón), lưu vào chuyến cùng `acceptedAt`.
- **R-ACCEPT-05 — Trạng thái tài xế.** Nhận thành công ⇒ tài xế `busy`. Chuyến `completed`, hoặc khách huỷ chuyến đã có tài xế, hoặc tài xế huỷ ⇒ tài xế về `online`. Tài xế `busy` khi và chỉ khi đang có đúng một chuyến ở `accepted`/`arriving`/`picked_up`/`in_transit`.
- **R-ACCEPT-06 — Điều phối tất định (kéo).** Lab không chủ động gửi offer: tài xế tự xem danh sách chuyến `searching` cùng thành phố và đúng loại xe của mình rồi chọn nhận. Thứ tự danh sách cố định: `requestedAt` tăng dần, cùng thời điểm thì theo `id` tăng dần.
- **R-ACCEPT-07 — Nhận lại là idempotent.** Tài xế gửi lại lệnh nhận cho chuyến **mình đang giữ** (chuyến ở `accepted`/`arriving`/`picked_up`/`in_transit` với chính tài xế đó) ⇒ 200 với chuyến hiện tại, không đổi gì (giữ `acceptedAt` và `etaMinutes` cũ). Kiểm tra này đứng ngay sau bước «chuyến không tồn tại» của R-ACCEPT-02.

## 10. Phí huỷ của khách (R-CANCEL)

- **R-CANCEL-01 — Phí theo trạng thái.**

  | Trạng thái lúc khách huỷ | Phí |
  |---|---|
  | `searching` | 0 đ |
  | `accepted`, chưa đủ 2 phút kể từ `acceptedAt` | 0 đ |
  | `accepted`, từ 2 phút trở lên kể từ `acceptedAt` | phí huỷ (R-CANCEL-03) |
  | `arriving` | phí huỷ (R-CANCEL-03) |
  | trạng thái khác | không được huỷ — `INVALID_TRANSITION` |

- **R-CANCEL-02 — Mốc 2 phút.** «Chưa đủ 2 phút» nghĩa là `now − acceptedAt < 2 phút`. Đúng 2 phút 0 giây tính là «từ 2 phút trở lên» ⇒ có phí.
- **R-CANCEL-03 — Mức phí.** 10.000 đ với chuyến xe máy (gồm mọi đơn đồ ăn), 20.000 đ với chuyến ô tô.
- **R-CANCEL-04 — Miễn phí khi tài xế trễ.** Nếu `now > acceptedAt + etaMinutes + 10 phút` (lớn hơn hẳn) thì phí là 0 đ, kể cả khi R-CANCEL-01 có phí. Đúng bằng mốc đó vẫn thu phí.
- **R-CANCEL-05 — Tài xế huỷ.** Tài xế huỷ (R-STATE-04) không bao giờ thu phí khách. Hệ thống huỷ (R-STATE-05) cũng không thu phí.
- **R-CANCEL-06 — Lý do phí.** Kết quả huỷ trả kèm lý do: `searching` (huỷ khi đang tìm), `grace_period` (chưa đủ 2 phút), `driver_late` (miễn do tài xế trễ), `charged` (có phí). Khi nhiều lý do miễn cùng đúng, thứ tự ưu tiên: `searching` → `grace_period` → `driver_late`.
- **R-CANCEL-07 — Thu phí.** Phí > 0 được thu theo R-PAY-05 và lưu vào chuyến (`cancellationFee`).

## 11. Thanh toán (R-PAY)

- **R-PAY-01 — Phương thức.** `cash` (tiền mặt, mặc định) hoặc `vixanh` (ví điện tử VíXanh).
- **R-PAY-02 — Trạng thái thanh toán của chuyến** (`paymentStatus`): `cash` (thu tiền mặt, hệ thống không gọi ví), `pending` (chờ thu qua ví), `paid` (đã thu qua ví), `failed` (thu qua ví thất bại, cần xử lý sau), `none` (không có gì phải thu). Lúc tạo chuyến: `cash` hoặc `pending` theo phương thức.
- **R-PAY-03 — Thu khi hoàn thành.** Chuyến `vixanh` hoàn thành ⇒ gọi VíXanh thu đúng số tiền `fare` với khoá idempotency `trip:<id>:fare`.
- **R-PAY-04 — Hết giờ và thử lại.** Mỗi lần gọi cổng thanh toán **hết giờ sau 3 giây**. Hết giờ hoặc lỗi tạm thời (cổng không trả lời được) ⇒ thử lại **tối đa 2 lần** (tổng tối đa 3 lần gọi), **dùng lại đúng khoá idempotency**. Bị từ chối (`declined`) ⇒ không thử lại. Cổng thanh toán đảm bảo cùng một khoá idempotency chỉ trừ tiền một lần.
- **R-PAY-05 — Phí huỷ.** Khách huỷ có phí > 0: chuyến `vixanh` ⇒ thu qua VíXanh với khoá `trip:<id>:cancel-fee` theo cùng quy tắc R-PAY-04; chuyến `cash` ⇒ ghi nhận phí, `paymentStatus = cash` (thu ở chuyến sau, ngoài phạm vi lab). Phí 0 ⇒ `paymentStatus = none`.
- **R-PAY-06 — Thanh toán không chặn nghiệp vụ.** Thu tiền thất bại (từ chối hoặc hết lượt thử) **không** chặn việc hoàn thành hay huỷ chuyến: chuyến vẫn sang `completed`/`cancelled`, `paymentStatus = failed`, API trả 200.

## 12. Idempotency của API (R-IDEM)

- **R-IDEM-01 — Bắt buộc khoá.** `POST /api/trips` và `POST /api/trips/:id/cancel` bắt buộc header `Idempotency-Key`: 8–128 ký tự trong `A–Z a–z 0–9 _ . : -` (ví dụ một UUID). Thiếu hoặc sai định dạng ⇒ 400 `IDEMPOTENCY_KEY_REQUIRED`.
- **R-IDEM-02 — Gửi lại cùng yêu cầu.** Phạm vi của khoá là (người gọi, endpoint — kể cả `:id` của chuyến). Cùng phạm vi + cùng khoá + **cùng nội dung thân** ⇒ trả lại **đúng mã HTTP và thân của lần xử lý đầu**, kèm header `Idempotent-Replayed: true`; không tạo chuyến mới, không huỷ lại, không thu tiền lại. «Cùng nội dung» nghĩa là cùng JSON sau khi sắp xếp khoá (thứ tự khoá và khoảng trắng không quan trọng; thân rỗng tương đương `{}`).
- **R-IDEM-03 — Dùng lại khoá cho nội dung khác.** Cùng phạm vi + cùng khoá + khác nội dung ⇒ 422 `IDEMPOTENCY_KEY_REUSED`, không xử lý.
- **R-IDEM-04 — Đang xử lý.** Yêu cầu thứ hai tới khi yêu cầu đầu cùng khoá chưa xong ⇒ 409 `IDEMPOTENCY_IN_PROGRESS`.
- **R-IDEM-05 — Kết quả nào được lưu.** Mọi phản hồi 2xx và 4xx do bước xử lý tạo ra được lưu và trả lại (kể cả lỗi như `QUOTE_EXPIRED`). Phản hồi 5xx **không** được lưu: client được thử lại với cùng khoá. Các lỗi xảy ra **trước** bước idempotency (401, 403, 429) không gắn với khoá. Bản lab giữ khoá suốt vòng đời tiến trình (hệ thống đích: tối thiểu 24 giờ).
- **R-IDEM-06 — Lệnh nhận chuyến** không cần header; tính idempotent của nó theo R-ACCEPT-07.

## 13. API HTTP (R-API)

- **R-API-01 — Định dạng.** JSON UTF-8, thân yêu cầu tối đa 10 KB (vượt ⇒ 413 `PAYLOAD_TOO_LARGE`; JSON hỏng ⇒ 400 `INVALID_JSON`). Thời điểm trả về dạng ISO 8601 UTC.
- **R-API-02 — Lỗi.** Mọi lỗi trả thân `{ "error": { "code": "<MÃ>", "message": "<thông điệp tiếng Việt>" } }`. Lỗi kiểm tra đầu vào có thêm `error.details` (danh sách trường sai).
- **R-API-03 — Xác thực (giản lược của lab).** Header `x-user: customer:<id>` hoặc `x-user: driver:<id>`, `<id>` phải tồn tại. Thiếu, sai cú pháp hoặc id không tồn tại ⇒ 401 `UNAUTHENTICATED`. *Đây là giản lược cho lab: hệ thống thật dùng token ký số (ví dụ OAuth2/JWT qua API Gateway), không tin header do client tự đặt.*
- **R-API-04 — Vai trò.** Endpoint dành cho khách mà tài xế gọi (hoặc ngược lại) ⇒ 403 `FORBIDDEN`.
- **R-API-05 — Quyền xem chuyến (chống IDOR).** Khách chỉ xem/huỷ được chuyến của chính mình. Tài xế xem được chuyến mình đang được gán, hoặc chuyến `searching` mà mình đủ điều kiện nhận về thành phố và loại xe. Chuyến không tồn tại **và** chuyến tồn tại nhưng không được xem đều trả **404 `TRIP_NOT_FOUND`** với cùng thông điệp, để không lộ việc chuyến có tồn tại. Báo giá của khách khác được xử lý tương tự (`QUOTE_NOT_FOUND`).
- **R-API-06 — Bảng mã lỗi → HTTP.**

  | HTTP | Mã |
  |---|---|
  | 400 | `VALIDATION_ERROR`, `INVALID_JSON`, `IDEMPOTENCY_KEY_REQUIRED` |
  | 401 | `UNAUTHENTICATED` |
  | 403 | `FORBIDDEN` |
  | 404 | `TRIP_NOT_FOUND`, `NOT_FOUND` (đường dẫn không tồn tại) |
  | 409 | `INVALID_TRANSITION`, `TRIP_ALREADY_TAKEN`, `DRIVER_OFFLINE`, `DRIVER_BUSY`, `QUOTE_ALREADY_USED`, `IDEMPOTENCY_IN_PROGRESS` |
  | 413 | `PAYLOAD_TOO_LARGE` |
  | 422 | `CITY_NOT_SUPPORTED`, `DISTANCE_OUT_OF_RANGE`, `WEIGHT_OUT_OF_RANGE`, `VEHICLE_TOO_SMALL`, `SUBTOTAL_TOO_LOW`, `CITY_MISMATCH`, `VEHICLE_MISMATCH`, `VOUCHER_NOT_FOUND`, `VOUCHER_EXPIRED`, `VOUCHER_NOT_APPLICABLE`, `VOUCHER_MIN_AMOUNT`, `VOUCHER_USAGE_LIMIT`, `QUOTE_NOT_FOUND`, `QUOTE_EXPIRED`, `IDEMPOTENCY_KEY_REUSED` |
  | 429 | `RATE_LIMITED` |
  | 500 | `INTERNAL_ERROR` (không lộ chi tiết lỗi nội bộ) |

- **R-API-07 — `POST /api/quotes`** (khách). Thân yêu cầu theo dịch vụ:
  - `ride`: `{ "kind": "ride", "cityCode", "vehicle", "distanceKm", "voucherCode"? }`
  - `parcel`: `{ "kind": "parcel", "cityCode", "vehicle", "distanceKm", "parcelWeightKg", "voucherCode"? }`
  - `food`: `{ "kind": "food", "cityCode", "vehicle"?: "bike", "distanceKm", "items": [{ "name", "unitPrice", "quantity" }], "voucherCode"? }`

  Trả 200 `{ "quote": { quoteId, pricingVersion, createdAt, expiresAt, kind, cityCode, vehicle, distanceKm, parcelWeightKg, items, peak, raining, voucherCode, fare: <bảng R-FARE-10>, itemsSubtotal, itemsDiscount, deliveryFee, discount, total } }`. Với `ride`/`parcel`: `total = fare.total`, `discount = fare.discount`, ba trường đồ ăn và `items` là `null`. Với `food`: `fare` là bảng chi tiết của phí giao, `deliveryFee = fare.total`, `discount = itemsDiscount`, `total = itemsSubtotal − itemsDiscount + deliveryFee`. Báo giá được lưu (R-QUOTE-01) nhưng không tiêu lượt voucher. Endpoint này không bị giới hạn tần suất.
- **R-API-08 — `POST /api/trips`** (khách, cần `Idempotency-Key`). Thân `{ "quoteId", "paymentMethod"?: "cash" | "vixanh" }` (mặc định `cash`). Giá lấy từ báo giá (R-QUOTE-04) — client không gửi giá. Tạo chuyến ở `searching`, trả **201** `{ "trip": … }`.
- **R-API-09 — `GET /api/trips/:id`** (khách hoặc tài xế, theo R-API-05) ⇒ 200 `{ "trip": … }`.
- **R-API-10 — Lệnh của tài xế.** `POST /api/trips/:id/accept` (thân `{ "etaMinutes" }`), `/arrive`, `/pickup`, `/start`, `/complete` ⇒ 200 `{ "trip": … }`. Chỉ tài xế. Lệnh nhận kiểm tra theo R-ACCEPT-02 và R-ACCEPT-07. Với `/arrive`, `/pickup`, `/start`, `/complete`: tài xế không phải tài xế được gán của chuyến ⇒ 404 `TRIP_NOT_FOUND`; tài xế được gán nhưng trạng thái không cho phép ⇒ 409 `INVALID_TRANSITION`.
- **R-API-11 — `POST /api/trips/:id/cancel`** (khách chủ chuyến hoặc tài xế được gán, cần `Idempotency-Key`; thân tuỳ chọn `{ "reason" }`, tối đa 200 ký tự). Khách ⇒ 200 `{ "trip", "cancellationFee", "feeReason" }`. Tài xế ⇒ 200 `{ "trip" }` với chuyến đã về `searching`.
- **R-API-12 — `GET /api/driver/available-trips`** (tài xế) ⇒ 200 `{ "trips": [...] }` theo R-ACCEPT-06.
- **R-API-13 — Giới hạn tần suất tạo chuyến.** Mỗi khách tối đa **5 yêu cầu `POST /api/trips` trong 60 giây trượt** (cấu hình được; 0 = tắt). Yêu cầu vượt ⇒ 429 `RATE_LIMITED` kèm header `Retry-After` (số giây tới khi yêu cầu cũ nhất trong cửa sổ ra khỏi cửa sổ, làm tròn lên, tối thiểu 1). Mọi yêu cầu đã qua xác thực và vai trò đều được đếm — kể cả yêu cầu bị từ chối vì dữ liệu sai và yêu cầu gửi lại cùng `Idempotency-Key` — trừ chính các yêu cầu bị 429. Bộ đếm nằm trong bộ nhớ của một tiến trình.
- **R-API-14 — Đầu vào không tin cậy.** Trường lạ trong thân yêu cầu bị bỏ qua; chuỗi được giới hạn độ dài; mọi truy vấn database dùng tham số (không ghép chuỗi SQL).
- **R-API-15 — `GET /api/health`** (không cần xác thực) ⇒ 200 `{ "status": "ok" }` — dùng cho giám sát và để công cụ kiểm thử chờ máy chủ sẵn sàng.

## 14. Móc kiểm thử (R-TEST)

- **R-TEST-01 — Chỉ bật ở môi trường test.** Các endpoint `/api/test/*` chỉ tồn tại khi máy chủ chạy với `NODE_ENV=test` hoặc `TOINOI_TEST_API=1`. Mặc định (kể cả `npm start`) chúng **không tồn tại** (404 `NOT_FOUND`). Không có backdoor kiểm thử nào khác trong mã chạy thật.
- **R-TEST-02 — Danh sách.** Đều là `POST`, không cần xác thực:

  | Endpoint | Thân | Tác dụng |
  |---|---|---|
  | `/api/test/reset` | — | Xoá chuyến, báo giá, khoá idempotency, bộ đếm tần suất; nạp lại dữ liệu mẫu (Phụ lục A); thời tiết và ví về mặc định |
  | `/api/test/clock` | `{ "now": "<ISO 8601 có múi giờ>" }` | Dời đồng hồ hệ thống tới thời điểm đó (đồng hồ tiếp tục trôi) |
  | `/api/test/weather` | `{ "raining": ["SG", …] }` | Đặt danh sách thành phố đang mưa |
  | `/api/test/payment` | `{ "mode": "none" \| "decline" \| "error" \| "timeout" \| "lost_response" }` | Chế độ lỗi của ví VíXanh giả (R-TEST-03) |
  | `/api/test/expire-searches` | — | Chạy ngay bước quét R-STATE-05, trả `{ "expired": n }` |

- **R-TEST-03 — Ví VíXanh giả.** `none`: chạy bình thường; `decline`: từ chối; `error`: lỗi tạm thời ngay lập tức; `timeout`: không trả lời và không trừ tiền; `lost_response`: đã trừ tiền nhưng không trả lời. Cùng khoá idempotency chỉ trừ tiền một lần.

## 15. Giao diện web (R-UI)

- **R-UI-01 — Trang khách `/`.** Nhập mã khách, dịch vụ, thành phố, loại xe, quãng đường, khối lượng (bưu kiện), tiền món (đồ ăn — một dòng món), voucher, phương thức thanh toán; nút «Báo giá» hiện tổng tiền, cờ cao điểm/mưa, mã và hạn của báo giá; nút «Đặt chuyến» tạo chuyến từ báo giá (bấm lại không tạo chuyến trùng — cùng `Idempotency-Key` cho một báo giá) và hiện trạng thái (tự làm mới mỗi 2 giây); nút «Huỷ chuyến» hiện phí huỷ. Sửa bất kỳ ô nào sau khi báo giá ⇒ phải báo giá lại.
- **R-UI-02 — Trang tài xế `/driver.html`.** Nhập mã tài xế, tải danh sách chuyến đang chờ, nhận chuyến với ETA, bấm lần lượt các bước tới hoàn thành, hoặc huỷ.
- **R-UI-03 — Khả năng tiếp cận.** Mọi ô nhập có nhãn; thông báo lỗi dùng `role="alert"`; kết quả cập nhật trong vùng `aria-live`. Phần tử quan trọng có `data-testid` cố định.
- **R-UI-04 — Hiển thị tiền.** Số tiền hiện theo định dạng Việt Nam kèm « đ» (ví dụ `52.000 đ`).

## 16. Yêu cầu phi chức năng (R-NFR) — mục tiêu khởi tạo

Các con số dưới đây lấy từ «SLO gợi ý ban đầu» (§14.1) của tài liệu kiến trúc: là **mục tiêu khởi tạo để thảo luận, chưa phải cam kết** trước khi có số đo workload thật; cần định nghĩa rõ SLI, cửa sổ đo, phần loại trừ phụ thuộc bên ngoài và error budget. Bản lab chạy một tiến trình trên một máy nên chỉ dùng để minh hoạ phương pháp đo, không dùng để kết luận hệ thống đạt hay không.

- **R-NFR-01 — Sẵn sàng.** Public API sẵn sàng **99,9% theo tháng** (≈ 43 phút gián đoạn tối đa mỗi tháng 30 ngày).
- **R-NFR-02 — Độ trễ Booking API.** p95 của `POST /api/trips` **dưới 500 ms**, **không tính thời gian tìm tài xế** (API trả ngay khi chuyến ở `searching`).
- **R-NFR-03 — Độ trễ Quote API.** p95 của `POST /api/quotes` **dưới 500 ms** (trong hệ thống đích phụ thuộc bản đồ MapViet).
- **R-NFR-04 — Lan truyền trạng thái.** Thay đổi trạng thái chuyến tới được ứng dụng khách trong p95 **dưới 2–3 giây**. Lab không có realtime gateway: trang khách hỏi lại mỗi 2 giây.
- **R-NFR-05 — Gọi cổng thanh toán.** Hết giờ 3 giây, tối đa 2 lần thử lại cùng khoá idempotency (R-PAY-04); không bao giờ trừ tiền hai lần cho cùng một khoản.
- **R-NFR-06 — Bảo mật.** Không lộ dữ liệu chuyến/báo giá của khách khác (R-API-05); lỗi 500 không lộ stack trace; giới hạn tần suất (R-API-13); kích thước thân yêu cầu giới hạn (R-API-01); API kiểm thử tắt mặc định (R-TEST-01).
- **R-NFR-07 — Mức tải tham chiếu** (không phải mục tiêu, để dựng mô hình tải): theo hồ sơ dự án, lúc cao điểm ≈ **70 chuyến tạo mới/giây** và ≈ **30.000 truy vấn đọc/giây** toàn hệ thống.

---

## Phụ lục A — Dữ liệu mẫu của máy chủ lab

| Loại | Mã | Mô tả |
|---|---|---|
| Khách | `C001`, `C002`, `C003` | Nguyễn Văn An, Trần Thị Bình, Lê Minh Châu |
| Tài xế | `D101`, `D102` | SG, xe máy, `online` |
| Tài xế | `D103` | SG, ô tô, `online` |
| Tài xế | `D104` | SG, xe máy, `offline` |
| Tài xế | `D201` / `D202` | HN, xe máy / ô tô, `online` |
| Voucher | `GIAM20` | 20%, trần 30.000 đ, tối thiểu 0 đ, mọi dịch vụ, mọi thành phố, 01/01/2026–31/12/2030, 1 lần/ngày |
| Voucher | `FOOD15` | 15%, trần 25.000 đ, tối thiểu 50.000 đ, chỉ `food`, mọi thành phố, 01/01/2026–31/12/2030, 2 lần/ngày |
| Voucher | `HN10` | 10%, trần 15.000 đ, tối thiểu 40.000 đ, chỉ `ride`, chỉ `HN`, 01/01/2026–31/12/2030, 1 lần/ngày |
| Voucher | `XMAS50` | 50%, trần 50.000 đ, tối thiểu 0 đ, mọi dịch vụ, mọi thành phố, 24/12/2025–26/12/2025 (đã hết hạn), 1 lần/ngày |

## Phụ lục B — Bảng truy vết yêu cầu → mã nguồn

| Nhóm | Mã nguồn chính |
|---|---|
| R-GEN-02 | `src/domain/time.ts` |
| R-FARE | `src/domain/fare.ts`, `src/domain/money.ts`, `src/domain/pricing.ts` |
| R-PARCEL | `src/domain/parcel.ts` |
| R-FOOD | `src/domain/food.ts` |
| R-VOUCHER | `src/domain/voucher.ts`, `src/app/quote.ts` |
| R-QUOTE | `src/app/quote.ts`, `src/app/trips.ts` (`createTrip`) |
| R-STATE | `src/domain/trip-state.ts`, `src/app/trips.ts` |
| R-ACCEPT | `src/app/trips.ts`, `src/infra/pglite/pg-trip-repository.ts` |
| R-CANCEL | `src/domain/cancel.ts` |
| R-PAY | `src/app/payment.ts`, `src/infra/fake-payment-gateway.ts` |
| R-IDEM | `src/api/idempotency.ts`, `src/infra/*/…IdempotencyStore` |
| R-API | `src/api/app.ts`, `src/api/schemas.ts`, `src/api/http-errors.ts`, `src/api/rate-limit.ts` |
| R-TEST | `src/api/test-routes.ts`, `src/server.ts` |
| R-UI | `src/web/` |

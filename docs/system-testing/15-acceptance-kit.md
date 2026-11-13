# SYS-15 — Hồ sơ thực hành UAT

**Trạng thái: mẫu đã chuẩn bị; chưa chạy phiên người dùng, chưa có sign-off.** Không điền PASS từ automation vào kết quả người dùng. [Bài học](15-uat.md) · [Báo cáo hỗ trợ](15-test-report.md).

## A. Phiếu chuẩn bị

| Trường | Giá trị cần điền trước phiên |
| --- | --- |
| Build/commit, môi trường/backend | Chưa điền |
| Ngày, browser/version, URL | Chưa điền |
| Người quyết định và phạm vi thẩm quyền | Chưa chỉ định |
| Đại diện khách/tài xế/vận hành, QA | Chưa chỉ định |
| Phạm vi | Mẫu: ride, SG, bike, tiền mặt; review rủi ro ví |
| Ngoài phạm vi | Food/parcel, provider thật, auth thật, toàn bộ thiết bị/thành phố |
| Dữ liệu | C001, D101, 5 km, không mưa, 09:00 VN; reset giữa scenario |
| Entry | Build cố định, smoke đạt, môi trường sẵn, tiêu chí đã được review |
| Exit dự kiến | Evidence đủ, lỗi chặn xử lý, ngoại lệ có owner, người quyết định xác nhận |

Entry/exit là mẫu cần thống nhất, chưa được ai phê duyệt. Móc test điều chỉnh clock do QA chuẩn bị theo README/SPEC; không bật móc test trên production.

## B. Kịch bản phiên người dùng

### UA-01 — Đặt và hoàn thành chuyến tiền mặt

Vai trò: khách + tài xế. Basis: R-QUOTE-04, R-IDEM-02, R-STATE, R-PAY-01.

1. Khách lấy báo giá bike/SG/5 km, xác nhận thấy 35.000 đ và chọn cash.
2. Đặt chuyến, bấm lại theo hành vi UI; lưu trip ID. Tài xế mở danh sách, nhận với ETA 5 phút.
3. Tài xế lần lượt tới đón, đón khách, bắt đầu, hoàn thành. Khách quan sát kết quả.
4. Đối chiếu một chuyến, giá giữ nguyên, completed; QA bổ sung evidence không gọi ví và tài xế online.

Hỏi khách giải thích trạng thái cuối và số tiền. Không coi cash là bằng chứng trao tiền thật. Kết quả phiên: **NOT RUN**. API hỗ trợ: đã pass, xem report.

### UA-02 — Huỷ trước và đúng mốc tính phí

Vai trò: khách + tài xế. Basis: R-CANCEL-01/02/03/06/07. Dùng hai chuyến độc lập, reset giữa lượt; QA đặt clock 1 phút hoặc đúng 2 phút sau nhận.

Khách huỷ, quan sát phí/lý do: lượt 1 miễn phí `grace_period`, lượt 2 phí 10.000 `charged`. Kiểm trạng thái cancelled, tài xế tiếp tục nhận việc. Ghi khách có hiểu phí và lý do không; UI hiển thị khác tên mã API là bình thường nếu ý nghĩa đúng. Không yêu cầu màn preview chưa có trong SPEC. Kết quả phiên: **NOT RUN**.

### UA-03 — Báo giá hết hạn

Vai trò: khách. Basis: R-QUOTE-01/03. Lấy báo giá, QA đưa clock đúng hạn 5 phút, khách đặt. Kỳ vọng lỗi hết hạn, không có chuyến; người dùng lấy báo giá mới và thử đặt lại. Phần tự động chỉ kiểm từ chối quote cũ; bước UI báo giá lại phải được quan sát riêng. Kết quả phiên: **NOT RUN**.

### UA-04 — Khoản thu chưa xác nhận

Vai trò: nghiệp vụ thanh toán/vận hành. Basis: R-PAY-04/06 và SYS-14 RC-03. Xem evidence mô phỏng ví đã thu nhưng DB failed; mô tả cách tra cứu, đối soát, tránh thu trùng và cập nhật có kiểm toán.

Kết quả: **BLOCKED cho phiên xử lý end-to-end**, vì lab chưa có console, ledger thật hay worker reconciliation. Có thể review evidence kỹ thuật, nhưng không đánh dấu workflow xử lý tiền đã đạt. Người chịu trách nhiệm và phương án chấp nhận phạm vi: **chưa chỉ định**.

### UA-05 — Hiểu trạng thái và tự tiếp tục công việc

Vai trò: khách + tài xế. Basis: R-UI-01/02/03/04. Quan sát song song UA-01/02/03; yêu cầu người dùng giải thích giá, trạng thái, lỗi và bước tiếp theo. Ghi chỗ cần hỗ trợ. Đây là evidence UAT về tác vụ; không thay nghiên cứu usability SYS-12. Kết quả: **NOT RUN**.

### UA-06 — Bàn giao xử lý tồn đọng

Vai trò: vận hành + business owner. Tiêu chí đề xuất mở rộng: có owner, kênh tiếp nhận, mức ưu tiên, evidence và cách đóng incident. Review một trường hợp failed và đối chiếu giới hạn recovery. Lab không có support console; thực hiện bằng tabletop nếu được thống nhất, ghi rõ hình thức. Kết quả: **NOT RUN**, chưa có owner/runbook nghiệp vụ được duyệt.

## C. Worksheet cho mỗi lượt

```text
Run ID / Scenario ID / build:
Người thực hiện / vai trò / thời điểm:
Dữ liệu và trip/quote ID:
Expected theo tiêu chí:
Actual quan sát được:
Evidence (ảnh/log/test report; dữ liệu giả):
Mức trợ giúp từ QA:
Kết quả: PASS | FAIL | BLOCKED | NOT RUN
Defect hoặc yêu cầu thay đổi:
Người xử lý / hạn / retest build:
```

PASS chỉ khi đã kiểm đủ tiêu chí scenario. BLOCKED nêu nguyên nhân không thể kiểm; NOT RUN là chưa thực hiện. Lưu actual cụ thể, không chỉ “như mong đợi”.

## D. Danh sách rủi ro cần review

| ID | Evidence hiện có | Cần bổ sung | Owner/quyết định |
| --- | --- | --- | --- |
| UAR-01 | SYS-14: đã thu nhưng failed | Quy trình/worker đối soát và kiểm tránh thu trùng | Chưa chỉ định |
| UAR-02 | SYS-12 chưa có user study | Quan sát người dùng thật theo phạm vi | Chưa chỉ định |
| UAR-03 | SYS-14 restore probe, chưa RTO/RPO ứng dụng | Diễn tập dữ liệu/hành trình thật trong staging | Chưa chỉ định |
| UAR-04 | Lab x-user, ví giả | Kiểm môi trường tích hợp/auth/provider phù hợp trước production | Chưa chỉ định |

## E. Biên bản quyết định — chưa ký

```text
Build và phạm vi nghiệm thu:
Các tiêu chí bắt buộc / evidence đã review:
Số scenario PASS / FAIL / BLOCKED / NOT RUN:
Quyết định: CHƯA CÓ QUYẾT ĐỊNH
Các lựa chọn khi review: chấp nhận | chấp nhận có điều kiện | chưa chấp nhận
Ngoại lệ, ảnh hưởng, workaround:
Owner / hạn xử lý / điều kiện cần retest:
Điều kiện làm quyết định hết hiệu lực:
Người có thẩm quyền / vai trò / ngày xác nhận: CHƯA ĐIỀN
```

Hiện trạng chuẩn bị: năm scenario NOT RUN, một scenario BLOCKED; bốn automated checks pass là evidence riêng. Không có biên bản nghiệm thu được ký và chưa có quyết định release.

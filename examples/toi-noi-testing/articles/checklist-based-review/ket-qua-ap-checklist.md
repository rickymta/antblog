# Kết quả áp checklist CL-PAY v1.0 lên SPEC R-PAY và kiến trúc §4.9, §8.3 (LỜI GIẢI)

> Lời giải bài thực hành TEST-028. Tự áp `checklist-thanh-toan.md` lên `SPEC.md` mục 11 (R-PAY, kèm R-IDEM, R-NFR-05,
> R-ARCH-02, R-TEST-03) trước khi đọc. Mức: Major = có thể sai tiền hoặc sai trạng thái nghiệp vụ; Minor = câu chữ
> hoặc thiếu sót không đổi hành vi đã cài đặt (cùng thang với TEST-027).

## 1. Từng mục

| Mục | Kết quả | Bằng chứng / phát hiện |
|---|---|---|
| PAY-01 | Phát hiện (đã biết) | Trùng **A-01** của TEST-027: R-PAY-03 «thu đúng số tiền `fare`», nhưng R-API-07 dùng `quote.fare` cho bảng chi tiết (đồ ăn: chỉ phí giao); SPEC chưa nói `fare` của chuyến = `quote.total`. Mã lab làm đúng (`fare: q.total`, `src/app/trips.ts`). Major |
| PAY-02 | Đạt | Cước ≥ cước tối thiểu (R-FARE-07), tiền món ≥ 30.000 đ (R-FOOD-03); phí 0 ⇒ không gọi ví (R-PAY-05); không có hoàn tiền |
| PAY-03 | Đạt | R-GEN-01: số nguyên đồng |
| PAY-04 | Đạt | `trip:<id>:fare`, `trip:<id>:cancel-fee`; thử lại dùng lại đúng khoá (R-PAY-04) |
| PAY-05 | Đạt | R-IDEM-02 cho tạo/huỷ chuyến; `/complete` lần hai ⇒ `INVALID_TRANSITION` (R-STATE-03), và cổng chống trùng theo khoá |
| PAY-06 | Đạt | Hai khoá khác hậu tố |
| PAY-07 | Câu hỏi (đã biết) | 3 giây mỗi lần có; thời gian tối đa của `/complete` chưa nêu — trùng **A-04** của TEST-027 |
| PAY-08 | Đạt | R-PAY-04: `declined` không thử lại, tối đa 2 lần thử lại |
| PAY-09 | **Phát hiện C-1 — Major** | Hết lượt thử ⇒ `failed` (R-PAY-06). Nhưng R-TEST-03 thừa nhận cổng có thể đã trừ tiền mà mất phản hồi (`lost_response`): khách bị trừ tiền, hệ thống ghi «thất bại». SPEC gộp «không rõ» vào «thất bại». Kiến trúc §8.3 thì giữ `PENDING` tới khi có kết quả chắc chắn. Xác nhận bằng `pay-09-lost-response.test.ts` |
| PAY-10 | **Phát hiện C-2 — Minor** | R-PAY-02 liệt kê 5 giá trị nhưng không có bảng chuyển: `failed` có thành `paid` được không? `pending` tồn tại bao lâu? |
| PAY-11 | Phát hiện (đã biết, hoãn) | «`failed` cần xử lý sau» — trùng **A-07** của TEST-027 (hoãn sang hệ đích, đối soát §8.3) |
| PAY-12 | Đạt | R-PAY-06: không chặn hoàn thành/huỷ, API vẫn 200 |
| PAY-13 | Đạt (ngầm định) | Không endpoint nào (R-API-07…15) cho client đặt `paymentStatus`. Đề xuất ghi thành quy tắc tường minh |
| PAY-14 | Không áp dụng ở lab | Webhook ngoài phạm vi lab (R-ARCH-02). Kiến trúc §4.9, §14: đạt |
| PAY-15 | **Phát hiện C-3 — Minor (lab) / bắt buộc (đích)** | SPEC không yêu cầu lưu mã giao dịch phía cổng; mã lab chỉ lưu `paymentStatus`, bỏ `chargeId`. Kiến trúc §4.9 yêu cầu lưu external reference ⇒ thiếu dữ liệu cho đối soát của C-1 |

## 2. Ngoài checklist

| Mã | Phát hiện | Mức | Hệ quả cho checklist |
|---|---|---|---|
| O-1 | Khoá `trip:<id>:fare` duy nhất chỉ khi id chuyến duy nhất. Lab dùng id tuần tự (`id-0001`…) bắt đầu lại mỗi lần khởi động máy chủ. Ví giả cũng mất dữ liệu nên lab không lỗi; nhưng nếu nhiều lần chạy cùng gọi một sandbox cổng thật, chuyến mới mang khoá cũ sẽ được trả kết quả cũ — **không bị thu tiền** | Minor ở lab; Major nếu dùng sandbox chung | Thêm PAY-16 vào v1.1 |

## 3. Áp mục kiến trúc lên §4.9 và §8.3 (góc nhìn TTA 5.2.1)

| Mục | Kiến trúc đích | Ghi chú |
|---|---|---|
| PAY-04, PAY-05 | Đạt — lưu idempotency key, unique constraint (§4.9, §14) | |
| PAY-09 | Đạt — `PENDING` tới khi chắc chắn (§8.3) | Lab lệch có chủ đích? ⇒ ghi rõ trong R-PAY |
| PAY-11 | Đạt — đối soát định kỳ (§8.3) | Chưa nêu chu kỳ, người chịu trách nhiệm |
| PAY-14 | Đạt — verify chữ ký, chống replay, xử lý trùng/trễ (§4.9) | |
| PAY-15 | Đạt — không lưu PAN/CVV, lưu external reference (§4.9) | |
| — | **Câu hỏi K-1**: §8.3 vẽ «Confirm booking / complete trip» rồi tạo payment intent; §9.2 chỉ nói «khi đến đúng thời điểm nghiệp vụ». Authorize lúc đặt hay chỉ capture lúc hoàn thành? Ảnh hưởng phí huỷ và tiền bị giữ | Hỏi kiến trúc sư |

## 4. Tổng hợp

- 15 mục: 8 Đạt · 5 Phát hiện (3 mới: C-1, C-2, C-3; 2 đã biết từ TEST-027: PAY-01 = A-01, PAY-11 = A-07) · 1 Câu hỏi (PAY-07 = A-04) · 1 Không áp dụng (PAY-14).
- Phát hiện **mới** so với lần review bằng checklist 8 câu của TEST-027: C-1 (Major), C-2, C-3, O-1, K-1.
- Checklist v1.1: thêm PAY-16. Không xoá mục nào: PAY-02 «Đạt» lần này không có nghĩa mục vô dụng.

# Đặc tả phí huỷ của khách — BẢN NHÁP v0.3

> Work product đưa vào inspection của bài TEST-026. Đây là **bản nháp có lỗi gài sẵn** để luyện tập,
> KHÔNG phải đặc tả của lab. Bản đã duyệt nằm ở `SPEC.md` mục 10 (R-CANCEL-01…07).

- Tác giả: BA nhóm Booking
- Ngày: 03/11/2026
- Mục đích: thay mục «Phí huỷ» của SPEC khi được duyệt
- Tài liệu liên quan: SPEC v1.1 — R-GEN, R-STATE, R-PAY, R-IDEM
- Kích thước: 7 mục yêu cầu, ≈ 1 trang A4

---

**N-CANCEL-01 — Phí theo trạng thái.** Khi khách huỷ chuyến, phí được tính theo trạng thái của chuyến lúc huỷ:

| Trạng thái | Phí |
|---|---|
| `searching` | 0 đ |
| `accepted`, trong vòng 2 phút sau khi tài xế nhận | 0 đ (lý do `grace_period`) |
| `accepted`, sau 2 phút | phí huỷ (N-CANCEL-03) |
| `arriving` | phí huỷ (N-CANCEL-03) |
| `picked_up` | gấp đôi phí huỷ |

**N-CANCEL-02 — Mốc thời gian.** Thời gian chờ tính từ lúc tài xế bấm «Nhận» trên ứng dụng tài xế, theo đồng hồ
của điện thoại tài xế (gửi kèm lệnh nhận).

**N-CANCEL-03 — Mức phí.** 10.000 đ với xe máy, 20.000 đ với ô tô. Phí có thể khác nhau giữa các thành phố,
xem bảng phí theo thành phố.

**N-CANCEL-04 — Tài xế trễ.** Nếu tài xế đến trễ đáng kể so với thời gian dự kiến đã báo, khách được miễn phí huỷ.

**N-CANCEL-05 — Tài xế huỷ.** Tài xế huỷ chuyến thì khách không mất phí.

**N-CANCEL-06 — Lý do.** Kết quả huỷ trả kèm một lý do: `searching`, `grace`, `late` hoặc `charged`.

**N-CANCEL-07 — Thu phí.** Phí được trừ vào ví VíXanh của khách ngay khi huỷ. Mỗi yêu cầu huỷ tạo một khoản phí
và được ghi vào chuyến.

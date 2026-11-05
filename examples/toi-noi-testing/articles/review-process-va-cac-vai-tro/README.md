# TEST-027 · Review Process và các vai trò

Bài: `/articles/review-process-va-cac-vai-tro` (Series 03 · Static Testing & Review Techniques).

Tài liệu review đặc tả thanh toán VíXanh (`SPEC.md` mục 11, R-PAY-01…06) — không có code, không cần chạy lệnh nào.

| Tệp | Nội dung |
|---|---|
| `ke-hoach-review-r-pay.md` | Kế hoạch review đã điền (planning): phạm vi, phân vai, tiêu chí vào/ra, lịch 5 hoạt động, checklist 8 mục, 3 kịch bản |
| `nhat-ky-review-r-pay.md` | Mẫu nhật ký phát hiện (trống) + nhật ký sau họp (7 mục) + báo cáo review |

Cách dùng khi tự luyện: đọc `SPEC.md` mục 11 với checklist và 3 kịch bản trong kế hoạch, tự ghi vào mẫu nhật ký trống, rồi mới so với nhật ký mẫu.

Ghi chú: mục A-01 (R-PAY-03 nói «thu đúng số tiền `fare`» trong khi R-API-07 định nghĩa `quote.fare` là bảng chi tiết — với đơn `food` thì `fare.total` chỉ là phí giao) là phát hiện thật trên SPEC v1.1. Mã nguồn đã làm đúng ý định (`src/app/trips.ts` gán `fare: q.total` cho chuyến); chỉ chữ của đặc tả chưa nói rõ.

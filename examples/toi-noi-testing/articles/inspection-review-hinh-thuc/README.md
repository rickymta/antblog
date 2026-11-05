# Inspection: review hình thức nhất (TEST-026)

Thư mục của bài [Inspection: review hình thức nhất](https://antblog.xyz/articles/inspection-review-hinh-thuc).
Bài thực hành là **static testing** — không có code chạy; chỉ có tài liệu review dạng Markdown:

| Tệp | Vai trò trong inspection |
|---|---|
| `dac-ta-phi-huy-ban-nhap-v0.3.md` | Work product được inspect: bản nháp đặc tả phí huỷ **có lỗi gài sẵn** (không phải đặc tả của lab — bản đúng là `SPEC.md` mục R-CANCEL) |
| `checklist-inspection.md` | Checklist CL-SPEC v1.0 cho sẵn, dùng ở bước individual review |
| `nhat-ky-inspection.md` | Lời giải: kế hoạch, vai trò, tiêu chí vào/ra, nhật ký cá nhân, biên bản họp, 12 defect, số đo, đề xuất cải tiến |

## Cách làm

1. Đọc bản nháp v0.3 và checklist; bấm giờ thời gian chuẩn bị.
2. Ghi mỗi anomaly: mã mục, vị trí, mô tả, loại (mơ hồ / thiếu / mâu thuẫn / thuật ngữ), mức đề xuất.
3. Đối chiếu với `SPEC.md` (R-GEN, R-STATE, R-PAY, R-IDEM) — inspection không chỉ đọc một tài liệu.
4. Tính: số defect theo loại, thời gian chuẩn bị, mật độ defect/trang (bản nháp ≈ 1 trang).
5. So với `nhat-ky-inspection.md`. Lời giải tìm 12 defect (2 critical, 8 major, 2 minor), 1 câu hỏi, 1 đề xuất.

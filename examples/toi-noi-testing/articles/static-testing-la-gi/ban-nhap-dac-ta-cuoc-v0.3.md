# Đặc tả cước đặt xe và bưu kiện — BẢN NHÁP v0.3 (chưa review)

> Tài liệu dùng cho bài TEST-021 «Static Testing là gì?». Đây là bản nháp **cố tình gài defect** để luyện static testing.
> Đừng dùng làm đặc tả thật — đặc tả đã chốt của lab là `SPEC.md` (R-FARE, R-PARCEL, R-GEN, R-NFR).
> Người soạn (hư cấu): nhóm Pricing & Quote. Nguồn cấp trên để đối chiếu: biên bản chốt nghiệp vụ = `SPEC.md` v1.1.

| Dòng | Nội dung |
|---|---|
| D-01 | Cước gồm phí mở cửa và tiền theo km. Bảng giá: xe máy — mở cửa 12.000 đ, 4.500 đ/km, tối thiểu 15.000 đ; ô tô — mở cửa 25.000 đ, 11.000 đ/km, tối thiểu 30.000 đ. |
| D-02 | Quãng đường do khách gửi được làm tròn tới 0,1 km. |
| D-03 | Quãng đường hợp lệ từ 0,5 km đến dưới 100 km; ngoài khoảng báo lỗi `DISTANCE_OUT_OF_RANGE`. |
| D-04 | Giờ cao điểm 11h–13h và 17h–19h: cước nhân 1,25. |
| D-05 | Khi trời mưa cộng phụ phí 8.000 đ. |
| D-06 | Voucher giảm theo phần trăm của cước, không vượt trần của voucher. |
| D-07 | Cước tối thiểu được áp trước khi trừ voucher. |
| D-08 | Tiền được làm tròn tới 1.000 đ. |
| D-09 | Bưu kiện trên 5 kg cộng 2.000 đ cho mỗi kg vượt. |
| D-10 | Ô tô có cước tối thiểu 35.000 đ. |
| D-11 | Hệ thống phải trả báo giá nhanh. |
| D-12 | Ví dụ: xe máy 7,4 km, cao điểm, trời mưa, voucher 20%: 12.000 + 4.500 × 7,4 = 45.300 → × 1,25 = 56.625 → + 8.000 = 64.625 → − 20% = 51.900 → 52.000 đ. |
| D-13 | Một chuyến dài tối đa 100 km. |

## Việc của người review

1. Đọc từng dòng, **không chạy gì cả**. Ghi mỗi phát hiện: dòng, loại (mơ hồ, thiếu, mâu thuẫn, sai, trùng lặp, không kiểm thử được), mô tả, đối chiếu `SPEC.md`.
2. Tìm ít nhất 5 defect. Lời giải: `nhat-ky-review-mau.md` (đọc sau khi tự làm).

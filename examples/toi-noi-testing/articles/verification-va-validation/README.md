# TEST-005 · Verification và Validation

Bài: `/articles/verification-va-validation` — Series 01, Testing Knowledge Hub «Tới Nơi».

```bash
npx vitest run articles/verification-va-validation --reporter=verbose
```

| Nhóm | Ca | Kiểm gì |
|---|---|---|
| Verification | V1–V4 | Công thức cước đúng đặc tả: ví dụ kiểm R-FARE-09, thứ tự cao điểm → mưa (R-FARE-03/05), làm tròn lên (R-FARE-08), cước tối thiểu sau voucher (R-FARE-07) |
| Câu hỏi validation | Q1–Q2 | Hành vi ĐÚNG đặc tả nhưng đặt ra câu hỏi cho người dùng: chuyến 0,5 km «giảm 2.850 đ» mà vẫn trả 15.000 đ; lượt GIAM20 trong ngày bị tiêu cho chuyến không được lợi (R-FARE-07 + R-VOUCHER-07) |

Kết quả mong đợi: 6/6 ca xanh. Q1, Q2 in ra hai dòng `[Q1]`, `[Q2]` — đó là dữ liệu để mang đi hỏi người dùng/Product Owner, không phải bằng chứng hệ thống «sai đặc tả».
Không sửa `src/`.

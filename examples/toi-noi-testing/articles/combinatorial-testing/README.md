# TEST-038 — Combinatorial Testing

Bài: `/articles/combinatorial-testing`.

Mô hình tổ hợp 11 tham số của điều kiện voucher (SPEC R-VOUCHER-01…07): dịch vụ, `kinds`, `cityCodes`, ngày trong hạn, giờ đặt, `minAmount`, A so với min, trần giảm, `perCustomerPerDay`, lịch sử lượt dùng, cách nhập mã — 10.368 tổ hợp, còn 5.616 sau ba ràng buộc C1–C3.

| Tệp | Nội dung |
|---|---|
| `t-wise.ts` | Bộ sinh t-wise tham lam có ràng buộc (độ phủ tính trên bộ-t khả thi) |
| `voucher-model.ts` | Mô hình hợp lệ + ràng buộc, mô hình trộn giá trị lỗi, biến hàng thành test case + oracle, 7 test âm base choice |
| `voucher-flow.ts` | Bản sao luồng kiểm voucher của `priceQuote` + 4 bản build có lỗi gài (V1 bậc 1, V2/V3 bậc 2, V4 bậc 3) — không sửa `src/` |
| `combinatorial.test.ts` | 11 test: mã thật xanh trên mọi tổ hợp, bản sao trùng `priceQuote`, kích thước/độ phủ bộ t = 1…3, sức bắt lỗi, hiện tượng che lỗi |
| `do-suc-bat-loi.ts` | In bảng chi phí t = 1…4, số test đỏ theo bản build, so với bộ ngẫu nhiên, mô hình trộn |
| `voucher.pict.txt` | Cùng mô hình theo cú pháp PICT — tham khảo, chưa chạy |

```bash
npx vitest run articles/combinatorial-testing              # 11 passed (~5 s)
npx tsx articles/combinatorial-testing/do-suc-bat-loi.ts   # ~12 s
```

Kết quả chính: t = 1 → 3 test, t = 2 → 14, t = 3 → 40, t = 4 → 110, đầy đủ 5.616. t = 2 chắc chắn bắt V1–V3; t = 3 chắc chắn bắt V4. Trộn giá trị lỗi vào mô hình: 3-wise cần 70 test, 50 test dừng ở lỗi, chỉ 83,3% bộ-3 hợp lệ thực sự được chạy.

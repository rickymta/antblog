# TEST-037 — Pairwise Testing

Bài: `/articles/pairwise-testing`.

Mô hình cấu hình ứng dụng khách Tới Nơi: nền tảng (Android/iOS/Web) × phiên bản (cũ nhất/phổ biến/mới nhất — đã qua EP) × thanh toán (`cash`/`vixanh`, SPEC R-PAY-01) × ngôn ngữ (vi/en) × mạng (Wi-Fi/4G/mạng yếu) = 108 tổ hợp.

| Tệp | Nội dung |
|---|---|
| `pairwise.ts` | Bộ sinh t-wise tham lam tối giản (liệt kê không gian hợp lệ rồi chọn hàng phủ nhiều bộ-t chưa phủ nhất), đo độ phủ, chọn ngẫu nhiên tất định |
| `app-config-model.ts` | Mô hình 5 tham số, bộ 9 hàng dựng từ mảng trực giao L9, 5 lỗi tương tác **giả lập** F1…F5 (hư cấu, không phải lỗi của `src/`) |
| `pairwise.test.ts` | 9 test: số tổ hợp, độ phủ each-choice/pairwise, bộ L9, lỗi nào bộ nào chạm tới |
| `in-bo-test.ts` | In hai bộ pairwise, bảng so sánh và thí nghiệm 1.000 bộ ngẫu nhiên |
| `toi-noi-app.pict.txt` | Cùng mô hình viết theo cú pháp PICT — tham khảo, chưa chạy |

```bash
npx vitest run articles/pairwise-testing          # 9 passed
npx tsx articles/pairwise-testing/in-bo-test.ts   # in bộ test + bảng so sánh
```

Kết quả chính: each-choice 3 hàng (29/67 cặp); pairwise tham lam 12 hàng, L9 9 hàng (67/67 cặp); 3-wise 28 hàng; đầy đủ 108. Bộ phủ đủ cặp luôn chạm F1–F3; F4 (3 tham số) và F5 (4 tham số) chỉ chạm nhờ may.

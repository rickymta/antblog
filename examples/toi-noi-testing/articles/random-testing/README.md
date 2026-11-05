# TEST-041 — Random Testing

Bài: `/articles/random-testing`.

Sinh 10.000 yêu cầu báo giá ngẫu nhiên bằng fast-check, chạy qua lõi định giá `src/domain/pricing.ts` (`normalizePriceRequest` + `priceRequest`) và kiểm bằng oracle tự động — không tính lại cước, chỉ kiểm tính chất SPEC bắt buộc:

| Mã | Tính chất | Nguồn |
|---|---|---|
| O1 | không NaN/Infinity, tổng là số nguyên đồng | R-GEN-01 |
| O2 | tổng cước (phí giao với đồ ăn) chia hết 1.000 đ | R-FARE-08 |
| O3 | tổng cước ≥ cước tối thiểu của loại xe | R-FARE-07 |
| O4 | tổng − max(sau giảm, tối thiểu) ∈ [0; 1.000) | R-FARE-08 |
| O5 | 0 ≤ tiền giảm ≤ trần voucher | R-VOUCHER-06 |
| O6 | đồ ăn: tổng = tiền món − giảm + phí giao | R-FOOD-01 |
| — | mã lỗi đầu tiên đúng thứ tự R-VALID-01 (tính độc lập bằng số nguyên mét/gam) | R-VALID-01, R-FARE-02, R-PARCEL-01/02, R-FOOD-03 |

| Tệp | Nội dung |
|---|---|
| `generators.ts` | `unguidedCase` (phân phối cố định 0–120 km, kể cả vùng không hợp lệ), `uniformCase(maxMeters)`, `profileCase` (operational profile: tỉ lệ dịch vụ và cao điểm từ hồ sơ dự án, phần còn lại giả định) |
| `oracle.ts` | `judge(priceFn, case)` trả danh sách vi phạm; `srcPrice` = lõi định giá thật |
| `buggy-pricing.ts` | hai bản có lỗi: `minimum-before-voucher` (đảo bước 4/5 của R-FARE-03), `rain-times-peak` (mưa bị nhân 1,25) |
| `random-pricing.test.ts` | RT-01 phân phối cố định · RT-02 operational profile · RT-03 tái lập bằng seed |
| `buggy-pricing.test.ts` | BG-01 tìm + thu nhỏ phản ví dụ · BG-02 xác suất phát hiện theo số lần chạy · BG-04 đổi phân phối · BG-03 oracle quá yếu |

```bash
npx vitest run articles/random-testing                      # 7 passed, 1 skipped
SHOW_RED=1 npx vitest run articles/random-testing -t SHOW_RED   # báo cáo gốc của fast-check (1 failed — cố ý)
```

PowerShell: `$env:SHOW_RED='1'; npx vitest run articles/random-testing -t SHOW_RED` (xong thì `Remove-Item Env:SHOW_RED`).

Kết quả mong đợi (seed 41): RT-01 không vi phạm, 29,1% ca rơi vào `DISTANCE_OUT_OF_RANGE`, 331 đơn đồ ăn có tổng không chia hết 1.000 đ (đúng SPEC); BG-01 thất bại ở lần chạy 557, thu nhỏ còn xe máy 0,45 km, voucher 8% trần 1.000 đ ⇒ 14.000 đ < 15.000 đ; BG-02 numRuns 100 ⇒ 0/20 seed, 1.000 ⇒ 3/20; BG-04 (0–3 km) 20/20 seed, trung vị 170 lần; BG-03 10.000 ca xanh dù bản lỗi ra 54.000 đ ở ca R-FARE-09.

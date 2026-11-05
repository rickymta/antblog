# Boundary Value Analysis (TEST-032)

Thư mục của bài [Boundary Value Analysis](https://antblog.xyz/articles/boundary-value-analysis). BVA 2-value và 3-value cho quãng đường (R-FARE-02), khối lượng (R-PARCEL-01), ngưỡng xe máy (R-PARCEL-02) và giờ cao điểm (R-FARE-04) — chú ý biên **trước** và **sau** khi làm tròn half-up.

| Tệp | Nội dung |
|---|---|
| `bva-model.ts` | Miền có thứ tự (giá trị nguyên theo bước độ chính xác), hàm sinh coverage item `twoValueItems` / `threeValueItems`, 6 miền của bài |
| `bva-items.test.ts` | Đếm coverage item 2-value vs 3-value cho từng miền |
| `bva-component.test.ts` | 52 ca 3-value trên `normalizeDistanceKm`, `normalizeWeightKg`, `assertVehicleFits`, `isPeakTime` |
| `bva-api.test.ts` | 14 ca biên qua `POST /api/quotes` (quãng đường, khối lượng, ngưỡng xe máy, giờ cao điểm) |
| `bien-loi.ts` | Ba bản cài đặt CÓ LỖI CỐ Ý: kiểm trước khi làm tròn · `toFixed(1)` · `kg === 0` |
| `bva-so-sanh.test.ts` | Ma trận: bốn bộ ca (thô/sau làm tròn × 2-value/3-value) bắt được lỗi nào |

## Lệnh chạy

```bash
npx vitest run articles/boundary-value-analysis
```

Kết quả mong đợi: 4 file, **76 test đạt**.

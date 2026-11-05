# Equivalence Partitioning (TEST-031)

Thư mục của bài [Equivalence Partitioning](https://antblog.xyz/articles/equivalence-partitioning). Phân vùng quãng đường (R-FARE-02), mã thành phố (R-GEN-03) và loại xe (R-GEN-04) của `POST /api/quotes`, viết test theo phân vùng ở hai test level và đo EP coverage.

| Tệp | Nội dung |
|---|---|
| `ep-model.ts` | 10 phân vùng (D1–D4, C1–C3, V1–V3), bảng 8 ca Each Choice EP-01…08, hàm `epCoverage` |
| `ep-component.test.ts` | EP trên `calculateFare`: phân vùng đầu vào (gồm NaN, Infinity) và phân vùng đầu ra «bị nâng lên cước tối thiểu» (rỗng với ô tô) |
| `ep-api.test.ts` | 8 ca Each Choice qua API, 9 đại diện khác của cùng phân vùng, 1 ca minh hoạ defect masking |
| `ep-coverage.test.ts` | EP coverage: bộ Each Choice 100%, bộ «thường gặp» 50% |
| `quang-duong-loi.ts` | Hai bản `normalizeDistanceKm` CÓ LỖI CỐ Ý (quên cận trên; kiểm trước khi làm tròn) |
| `ep-gioi-han.test.ts` | Cùng bảng EP trên bản đúng và hai bản lỗi: EP bắt lỗi A, bỏ lọt lỗi B (lỗi biên — xem TEST-032) |

## Lệnh chạy

```bash
npx vitest run articles/equivalence-partitioning
```

Kết quả mong đợi: 4 file, **34 test đạt**.

# TEST-020 — Maintenance Testing

Bài: `/articles/maintenance-testing`.

Tình huống: Tới Nơi đang vận hành và có ba thay đổi — tăng giá mỗi km xe máy 4.500 ⇒ 5.000 đ (con số giả định, R-FARE-01), ngừng phục vụ Nha Trang (R-GEN-03), chuyển kho dữ liệu từ bộ nhớ sang PostgreSQL. Mỗi thay đổi ứng với một nhóm trigger của CTFL v4.0.1 mục 2.3.

| Tệp | Trigger | Nội dung |
|---|---|---|
| `pricing-2026-2.ts` | Modification | Bản thay đổi đề xuất `fare-2026.2` (bảng giá mới, bỏ NT) — không sửa `src/` |
| `impact.test.ts` | Modification + retirement | 15 test regression có sẵn: dự đoán «đổi/giữ» kết quả mong đợi, kiểm cả bản cũ (src) lẫn bản mới |
| `retire-city.ts` | Retirement | Script lưu trữ chuyến của một thành phố, khôi phục vào kho lưu trữ, truy xuất |
| `retirement.test.ts` | Retirement | MT-R1…R5 trên PGlite: chỉ lưu chuyến đã kết thúc, chặn chuyến đang chạy, tổng kiểm, khôi phục, truy xuất |
| `migration-env.test.ts` | Upgrade/migration | MT-E1…E4: cùng bộ regression chạy trên kho bộ nhớ và PGlite |

```bash
npx vitest run articles/maintenance-testing        # 29 passed
```

Kết quả mong đợi: `impact.test.ts` 16/16 (9 ca kết quả mong đợi đổi: M-01, 02, 06, 08, 09, 10, 11, 12, 15; 6 ca giữ: M-03, 04, 05, 07, 13, 14), `retirement.test.ts` 5/5, `migration-env.test.ts` 8/8 (4 ca × 2 môi trường).

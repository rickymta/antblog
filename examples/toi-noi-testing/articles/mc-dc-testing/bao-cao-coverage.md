# Báo cáo coverage — bài thực hành nâng cao Series 05 (TEST-047)

- Đối tượng: `src/domain/voucher.ts`, `src/app/quote.ts` (lab `toi-noi-testing`, không sửa).
- Công cụ: vitest 3.2.7, `@vitest/coverage-v8` (đo theo khối của V8).
- Ngày chạy: 05/11/2026, Windows 11, Node 22.

## 1. Statement và branch (công cụ đo)

| Bộ test | Số ca | voucher.ts stmt / branch | quote.ts stmt / branch | Dòng chưa phủ |
|---|---:|---|---|---|
| Chỉ 10 ca MC/DC (M1–M10) | 10 | 100% / 87,5% | 96,77% / 76,92% | voucher.ts 24–25; quote.ts 49–50 |
| MC/DC + bổ sung S1–S6, Đ1–Đ2 | 18 | **100% / 100%** | **100% / 100%** | — |

Nhánh còn thiếu sau 10 ca MC/DC (đọc `coverage-final.json`): `?? ''` và `? null` của `normalizeVoucherCode`,
`return null` khi không có mã, `voucher?.code ?? null` khi không có voucher, nhánh ném của `requireRole`.

## 2. MC/DC (kiểm bằng `mcdc.ts` — V8 không đo MC/DC)

| Điều kiện | Cặp độc lập |
|---|---|
| C1 tìm thấy mã | M1 ↔ M2 |
| C2 ngày ≥ validFrom | M1 ↔ M3 |
| C3 ngày ≤ validTo | M1 ↔ M4 |
| C4 kinds rỗng | M5 ↔ M6 |
| C5 kinds chứa dịch vụ | M1 ↔ M5 |
| C6 cityCodes rỗng | M7 ↔ M8 |
| C7 cityCodes chứa thành phố | M1 ↔ M7 |
| C8 A ≥ minAmount | M1 ↔ M9 |
| C9 còn lượt | M1 ↔ M10 |

MC/DC của V: 9/9 = 100% với 10 ca = N + 1. Từng quyết định trong code (voucher.ts:34, 40, 41, 42, 52, 63): 100%.

## 3. Lỗi tìm được / không tìm được

- Bản lỗi `!kindOk && !cityOk`: bộ 3 ca đạt 100% stmt / 100% branch trên bản lỗi mà không lộ lỗi; M5 và M7 lộ lỗi.
- Không phát hiện defect mới trong `src/`.

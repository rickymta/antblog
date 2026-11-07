# TEST-042 — Metamorphic Testing

Bài: `/articles/metamorphic-testing`.

Kiểm thuật toán cước của lab (`src/domain/pricing.ts`) bằng metamorphic relation (MR) — không cần biết tổng tiền đúng của từng ca. Source test case sinh ngẫu nhiên bằng fast-check (kết hợp MT với random testing như CTAL-TA 3.3.2 gợi ý); mỗi MR biến source thành follow-up rồi đánh giá chung cặp kết quả.

| MR | Biến đổi source ⇒ follow-up | Quan hệ phải thoả | Quy tắc |
|---|---|---|---|
| MR1 | quãng đường + 0,1 km | tổng không giảm | R-FARE-01…03 |
| MR2 | tắt mưa ⇒ bật mưa | «trước giảm» tăng đúng 8.000 đ | R-FARE-05 |
| MR3 | giờ thường ⇒ cao điểm | tổng không giảm | R-FARE-04 |
| MR4 | giờ thường ⇒ cao điểm (không mưa, voucher không trần) | tiền giảm × 1,25 | R-FARE-06 |
| MR-sai | tắt ⇒ bật mưa | TỔNG tăng đúng 8.000 đ — quan hệ sai | — |
| MR-yếu | tắt ⇒ bật mưa | tổng không giảm — quan hệ quá yếu | — |

| Tệp | Nội dung |
|---|---|
| `variants.ts` | `srcPrice` (lõi thật) + bốn bản lỗi B1 mưa bị nhân cao điểm · B2 tối thiểu trước voucher · B3 làm tròn gần nhất · B4 voucher tính trên cước cơ bản |
| `relations.ts` | bộ sinh source `sourceArb`, các MR, `mrProperty(price, mr)` |
| `metamorphic.test.ts` | MT-01 ví dụ tay · MT-02 bản thật · MT-03 ma trận bản lỗi × MR · MT-04 chẩn đoán · MT-05 MR sai/yếu · MT-06 cỡ biến đổi follow-up · MT-07 oracle tính chất của TEST-041 (import `../random-testing/`) trên cùng bốn bản lỗi |

```bash
npx vitest run articles/metamorphic-testing                        # 7 passed, 1 skipped
SHOW_RED=1 npx vitest run articles/metamorphic-testing -t SHOW_RED # báo cáo gốc fast-check: MR1 trên B2 (1 failed — cố ý)
```

PowerShell: `$env:SHOW_RED='1'; npx vitest run articles/metamorphic-testing -t SHOW_RED` (xong thì `Remove-Item Env:SHOW_RED`).

Kết quả mong đợi (seed 42): MR1–MR4 thoả trên bản thật; ma trận MT-03 (10.000 cặp mỗi ô) — B1 chỉ MR2 đỏ, B2 chỉ MR1 đỏ, B3 không MR nào đỏ, B4 chỉ MR4 đỏ; MR-sai đỏ trên bản thật (chênh 7.000 đ); MR-yếu xanh trên B1; MT-06: MR1 bước 0,1 km phát hiện B2 ở 31/50 seed, bước 0,1–5 km chỉ 9/50; MT-07: oracle TEST-041 bắt B2, B3, không bắt B1, B4.

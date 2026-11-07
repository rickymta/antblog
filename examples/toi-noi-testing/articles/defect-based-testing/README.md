# TEST-054 · Defect-based Testing

Bài: `/articles/defect-based-testing`.

- `lich-su-defect.csv` — 30 defect lịch sử **giả định** của Tới Nơi, phân loại theo nhiều trục độc lập: `module`, `cum` (mô hình lỗi), `odc` (loại defect kiểu ODC), `ctfl` (nhóm của CTFL 4.4.1), `muc_do`, `dua_vao` (pha đưa vào), `phat_hien` (nơi phát hiện), `tai_dien_cua` (lặp lại defect nào). Mở được bằng bảng tính (dấu phân cách `;`).
- `phan-loai.ts` + `phan-loai.test.ts` — đếm theo từng trục, có điểm trọng số mức độ (critical 10, major 5, minor 2, trivial 1 — giả định). In bốn bảng: theo module, theo mô hình lỗi, theo nhóm CTFL, theo loại ODC.
- `mo-hinh-loi.test.ts` — test thiết kế từ ba cụm hay tái diễn: **FM-R** làm tròn (996 giá trị x,x5 + 31.872 tổ hợp cước so với oracle BigInt độc lập), **FM-T** giờ Việt Nam (48 mốc cao điểm ±1 giây, 24 mốc ngày voucher quanh 17:00 UTC), **FM-S** trạng thái (đủ 72 cặp trạng thái × sự kiện; dọn dữ liệu khi rời `accepted`). Mỗi bộ chạy với `src/` (phải 0 ca sai) và với bản tái tạo defect cũ.
- `ban-loi.ts` — bản tái tạo defect lịch sử: D-01 (toFixed), D-04 (làm tròn gần nhất), D-06/D-09 (làm tròn tiền giảm), D-02 (giờ máy chủ UTC), D-24 (đầu mút `<=`), D-05 (ngày theo `toISOString`), D-08, D-18 (chuyển trạng thái sai); và một biến thể **tương đương** (làm tròn về đồng sau ×1,25) mà bộ test đúng ra không thể bắt.

```bash
npx vitest run articles/defect-based-testing --reporter=verbose
```

Kết quả mong đợi: `17 passed`. Số ca sai in ra: src 0 ở cả ba mô hình; D-01 398; D-04 13.366; D-06/D-09 1.078; biến thể tương đương 0; D-02 20; D-24 8; D-05 12; D-08 2; D-18 1.

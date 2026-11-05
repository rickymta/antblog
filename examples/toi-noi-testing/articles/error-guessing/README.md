# Error Guessing (TEST-051)

Thư mục của bài [Error Guessing](https://antblog.xyz/articles/error-guessing): danh sách 12 fault attack cho thanh toán
VíXanh và huỷ chuyến (SPEC R-PAY, R-CANCEL, R-IDEM, R-STATE) và lần thử chúng trên API lab.

| Tệp | Nội dung |
|---|---|
| `fault-attacks.test.ts` | 12 fault attack FA-01…FA-12 (supertest trên `createApp` + kho trong bộ nhớ, đồng hồ `FixedClock` 09:00 giờ VN 04/11/2026, ví VíXanh giả). FA-12 chen lệnh «Đang tới đón» của tài xế vào giữa lúc use case huỷ đọc và ghi chuyến (bọc `TripRepository` bằng `Proxy`). Ca `FA-03R` đo hết giờ THẬT 3 giây — chỉ chạy khi đặt `FA_REAL_TIMEOUT=1`. |

## Lệnh chạy

```bash
npx vitest run articles/error-guessing
FA_REAL_TIMEOUT=1 npx vitest run articles/error-guessing -t "FA-03R"   # ≈ 9 giây
```

Kết quả mong đợi: `12 passed | 1 skipped` (lệnh đầu), mỗi ca in một dòng `[FA-xx] …` với kết quả thực tế.
Lệnh thứ hai in `[FA-03R] … tài xế chờ phản hồi «Hoàn thành» ≈ 9000 ms`.

Các ca đều XANH vì chúng khẳng định **hành vi hiện tại** của lab. Ba ca ghi lại anomaly (bài viết mục 5.3):
FA-03R (tài xế chờ ≈ 9 giây), FA-10 (cách đếm «200 ký tự») và FA-12 (thông báo `INVALID_TRANSITION` nói sai luật).

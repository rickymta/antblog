# TEST-046 · Multiple Condition Testing

Bài: `/articles/multiple-condition-testing` (Series 05).

Quyết định «tài xế nhận được chuyến» (SPEC R-ACCEPT-01) tách thành 5 atomic condition như code
`src/app/trips.ts` kiểm (không offline, không busy, cùng thành phố, đúng loại xe, chuyến đang searching)
⇒ 2^5 = 32 tổ hợp, 8 bất khả thi (tài xế vừa offline vừa busy), 24 khả thi được chạy thật trên `acceptTrip`.

| Tệp | Nội dung |
|---|---|
| `bang-chan-tri.ts` | Sinh 2^N tổ hợp, đánh dấu bất khả thi, oracle theo thứ tự R-ACCEPT-02 |
| `dung-tinh-huong.ts` | Dựng hệ thống trong bộ nhớ cho một dòng (tài xế DX, chuyến phụ, tài xế DK) rồi gọi lệnh nhận |
| `nhan-chuyen-sai-thu-tu.ts` | Bản SAI: kiểm loại xe trước thành phố |
| `multiple-condition.test.ts` | 29 test: đếm tổ hợp, 24 dòng khả thi, so bộ «lật từng điều kiện» với bộ đủ tổ hợp |
| `in-bang.ts` | In bảng 32 dòng: mong đợi, code thật, bản sai |

```bash
npx vitest run articles/multiple-condition-testing          # 29 passed
npx tsx articles/multiple-condition-testing/in-bang.ts      # bảng ở mục 5 của bài
```

Không sửa `src/`.

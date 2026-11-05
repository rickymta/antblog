# TEST-033 · Decision Table Testing

Bài: `/articles/decision-table-testing`.

Bảng quyết định cho việc huỷ chuyến — người huỷ × trạng thái × đủ 2 phút × tài xế trễ × loại xe (SPEC R-CANCEL-01…06, R-STATE-02/04/05). Không sửa `src/`.

| Tệp | Nội dung |
|---|---|
| `bang-quyet-dinh.ts` | Bảng viết thành dữ liệu: 5 điều kiện, 13 quy tắc đã rút gọn (DT-01…DT-13) kèm bộ dữ liệu đại diện, bảng đầy đủ 96 cột, lý do không khả thi, checksum |
| `chay-to-hop.ts` | Dựng chuyến đúng tổ hợp (kho bộ nhớ + `FixedClock`, ETA 5 phút) rồi huỷ qua `cancelByCustomer` / `cancelByDriver` / `expireStaleSearches` |
| `buggy-cancel-fee.ts` | Bản chép `customerCancellationFee` có một defect cố ý: miễn phí 2 phút áp cả cho `arriving` |
| `decision-table.test.ts` | 64 ca vitest |

```bash
npx vitest run articles/decision-table-testing --reporter=verbose
```

Kết quả mong đợi: `64 passed` —
4 ca kiểm chính bảng (96 = 40 khả thi + 56 không khả thi; mỗi cột khả thi khớp đúng một quy tắc; checksum 48 = 40 + 8),
13 ca DT-01…DT-13 (100% coverage bảng rút gọn), 40 ca bảng đầy đủ, 2 ca biên (kết hợp BVA),
5 ca về defect: bộ 13 ca bắt được nhờ DT-06; đổi đại diện «–» của DT-06 hoặc dùng bộ «mỗi trạng thái một ca» thì lọt.

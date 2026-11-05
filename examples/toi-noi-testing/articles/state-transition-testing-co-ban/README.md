# TEST-034 · State Transition Testing (cơ bản)

Bài: `/articles/state-transition-testing-co-ban`.

Máy trạng thái 8 trạng thái của chuyến (SPEC R-STATE-01…06). Không sửa `src/`.

| Tệp | Nội dung |
|---|---|
| `bang-trang-thai.ts` | Mô hình chép từ SPEC (không import bảng của `src/`): 8 trạng thái, 9 sự kiện, 12 chuyển hợp lệ T01…T12, 5 test case TC-ST-01…05 dạng chuỗi sự kiện, hàm đo all states / valid transitions coverage |
| `buggy-trip-state.ts` | Bản chép bảng chuyển có một defect cố ý: khách huỷ được ở `picked_up` |
| `state-transition.test.ts` | 96 ca vitest |

```bash
npx vitest run articles/state-transition-testing-co-ban --reporter=verbose
```

Kết quả mong đợi: `96 passed` —
1 ca mô hình (72 ô = 12 hợp lệ + 60 trống), 7 ca domain cho 5 test case và coverage (8/8 trạng thái, 12/12 chuyển hợp lệ),
72 ca all transitions coverage trên `nextStatus`, 2 ca defect (bộ 5 test case vẫn xanh trên bản lỗi; thử 60 ô trống lệch đúng một ô),
5 ca use case thật (trạng thái cuối + nhật ký `trip_events`), 6 ca chuyển không hợp lệ qua use case (INV-01…06: `INVALID_TRANSITION`, chuyến/nhật ký/tiền đã thu không đổi),
2 ca ô trống mà API trả mã khác (`TRIP_NOT_FOUND`, `TRIP_ALREADY_TAKEN` / nhận lại idempotent), 1 ca mô hình tất định.

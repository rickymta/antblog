# TEST-035 · Use Case Testing và Scenario-based Testing

Bài: `/articles/use-case-testing-scenario-based-testing`.

- `kich-ban.ts` — mô hình kịch bản của use case UC-01 «Khách đặt xe máy»: 7 kịch bản đã xác định (1 luồng chính, 4 extension, 2 exception), hàm `scenarioCoverage` (số kịch bản đã chạy / số đã xác định — CTAL-TA v4.0 mục 3.2.3) và 4 coverage item của simple loop coverage cho vòng «tài xế huỷ ⇒ tìm lại».
- `uc01-dat-xe-may.test.ts` — 11 test case chạy qua API HTTP của lab (supertest, kho trong bộ nhớ, `FixedClock`, ví VíXanh giả, `/api/test/expire-searches`). Mỗi test case đi trọn một kịch bản; KB-05 và KB-06 có thêm test case biên (4:59/5:00, 1:59/2:00); TC-08, TC-09 ghép vòng lặp với ngoại lệ. Không sửa `src/`.

```bash
npx vitest run articles/use-case-testing-scenario-based-testing --reporter=verbose
```

Kết quả mong đợi: `11 passed`, kèm dòng in ra
`Scenario coverage UC-01: 6/7 = 85.7% · chưa chạy: KB-07` (KB-07 thuộc giao diện, R-UI-01) và
`Simple loop coverage (tài xế huỷ ⇒ tìm lại): 3/4 · chưa phủ: tối đa` (SPEC không quy định số lần tối đa — xem bài).

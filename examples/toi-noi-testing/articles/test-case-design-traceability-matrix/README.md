# TCD-08 · Ma trận truy vết (RTM) sinh từ mã test

Bài: `/articles/test-case-design-traceability-matrix`.

RTM cho R-STATE và R-CANCEL (SPEC mục 8 và 10): yêu cầu ↔ test condition ↔ test case ↔ kết quả ↔ defect, nhiều-nhiều. RTM không viết tay mà **sinh ra** từ SPEC.md, một registry test condition và tag trong tiêu đề test. Không sửa `src/`, không thêm thư viện.

| Tệp | Nội dung |
|---|---|
| `dieu-kien.ts` | Registry phía test analysis: 20 test condition (`TCOND-STATE-01…09`, `TCOND-CANCEL-01…11`) và yêu cầu chúng truy về; R-STATE-08 loại khỏi phạm vi có lý do; 1 defect **giả lập** `DEF-SIM-001` |
| `trang-thai.test.ts` | 7 test thật cho R-STATE (6 có tag, 1 cố ý không tag) |
| `phi-huy.test.ts` | 10 test thật cho R-CANCEL (9 có tag đúng, 1 trỏ vào mã `R-CANCEL-08` không tồn tại) |
| `dung-chuyen.ts` | Dựng chuyến tới trạng thái cần bằng use case thật, `FixedClock` |
| `rtm.ts` | Đọc SPEC, đọc tiêu đề test (`TC-… @R-… @TCOND-…`), dựng RTM; tính độ phủ yêu cầu, độ phủ test condition, lỗ hổng, mồ côi, tag lệch; truy vấn ảnh hưởng và truy vết ngược |
| `rtm.test.ts` | 13 test kiểm chính RTM |
| `rtm-reporter.ts` | Reporter vitest: ghép kết quả chạy thật vào RTM, in ra và ghi `reports/rtm-tcd08.md` |
| `in-rtm.ts` | In RTM tĩnh, `--impact R-…` (phân tích ảnh hưởng), `--back <TC hoặc DEF>` (truy vết ngược) |

```bash
npx vitest run articles/test-case-design-traceability-matrix
# kèm cột kết quả:
npx vitest run articles/test-case-design-traceability-matrix --reporter=default --reporter=./articles/test-case-design-traceability-matrix/rtm-reporter.ts
# truy vấn:
npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts --impact R-CANCEL-02
npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts --back DEF-SIM-001
```

Kết quả mong đợi: `30 passed (30)` — 13 (RTM) + 7 (R-STATE) + 10 (R-CANCEL). Trạng thái RTM **có chủ đích** để bài phân tích:

- Phạm vi: 15 yêu cầu đọc từ SPEC, loại R-STATE-08 ⇒ 14.
- Độ phủ yêu cầu 13/14 = 92,9%; độ phủ test condition 16/20 = 80%.
- Lỗ hổng: R-STATE-05; test condition chưa phủ: TCOND-STATE-04, TCOND-STATE-07, TCOND-CANCEL-08, TCOND-CANCEL-09.
- Mồ côi: test «khách huỷ lần thứ hai…» (không tag — dù nó kiểm đúng TCOND-STATE-04) và TC-CANCEL-090 (`@R-CANCEL-08` không có trong SPEC).
- Tag lệch: TC-CANCEL-006 gắn `@R-CANCEL-02` nhưng TCOND-CANCEL-05 chỉ truy về R-CANCEL-01.
- Ảnh hưởng khi đổi R-CANCEL-02 (mốc 2 phút): TCOND-CANCEL-02, 03; TC-CANCEL-002, 003, 004, 006.
- Ảnh hưởng khi thêm trạng thái (R-STATE-01/02/03, R-CANCEL-01): 9 test condition, 10 test case; test không tag không lọt vào danh sách.

Trong CI thật nên đổi các khẳng định thành «không mồ côi, không tag lệch, độ phủ yêu cầu ≥ ngưỡng đã thống nhất».

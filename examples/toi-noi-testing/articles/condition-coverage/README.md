# TEST-045 · Condition Coverage

Bài: `/articles/condition-coverage` (Series 05).

Quyết định «voucher áp được» = `trongHan && dungDichVu && dungThanhPho && duTien` (SPEC R-VOUCHER-04…06),
đo condition coverage và decision coverage của ba bộ test theo hai cách đếm: bảng chân trị (mọi điều kiện
có giá trị) và code thật (chỉ điều kiện `src/domain/voucher.ts` đã tính tới trước khi ném lỗi).

| Tệp | Nội dung |
|---|---|
| `dieu-kien-voucher.ts` | Oracle theo SPEC (`giaTriTheoSpec`) và lời gọi code thật (`chayThat`) |
| `do-phu.ts` | Đếm coverage item điều kiện (2N) và quyết định (2) |
| `bo-test.ts` | Voucher thử HN11 và Bộ A (CC không DC), Bộ B (CC + DC, 2 test), Bộ C (N + 1 = 5 test) |
| `dot-bien.ts` | Ba bản cài đặt sai M1–M3 của quyết định |
| `condition-coverage.test.ts` | 16 test kiểm các khẳng định của bài |
| `bang-do-phu.ts` | In bảng giá trị điều kiện + coverage |

```bash
npx vitest run articles/condition-coverage          # 16 passed
npx tsx articles/condition-coverage/bang-do-phu.ts  # bảng ở mục 5.2 của bài
# v8 chỉ với hai test «lật chéo» của if (!kindOk || !cityOk):
npx vitest run articles/condition-coverage -t "lật chéo" --coverage --coverage.include=src/domain/voucher.ts --coverage.reporter=text
```

Không sửa `src/`. Không cần dữ liệu ngoài.

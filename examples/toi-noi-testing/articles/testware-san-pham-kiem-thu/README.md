# Testware và các sản phẩm kiểm thử (TEST-010)

Thư mục của bài [Testware và các sản phẩm kiểm thử](https://antblog.xyz/articles/testware-san-pham-kiem-thu). Một bộ testware tối thiểu cho phụ phí khối lượng bưu kiện (SPEC R-PARCEL-01…04):

| Tệp | Loại testware (CTFL v4.0.1 mục 1.4.3) |
|---|---|
| `test-data/td-parcel-01.json` | test data — bộ TD-PARCEL-01; mỗi dòng ghi test case và mã yêu cầu nó phủ |
| `parcel-surcharge.test.ts` | automated test script cho TC-PARCEL-001…005 (mức miền) + TC-PARCEL-002 qua API |
| `traceability.test.ts` | kiểm ma trận truy vết hai chiều SPEC.md ↔ test data |
| `buggy-parcel.ts` | bản cài đặt CÓ LỖI, chỉ để tái hiện defect report DEF-0042 trong bài — không phải mã của lab |

## Lệnh chạy

```bash
npx vitest run articles/testware-san-pham-kiem-thu
```

Kết quả mong đợi: 2 file, **13 test đạt** (10 dòng dữ liệu + 1 test qua API + 2 test truy vết).

Chạy cùng bộ test trên bản lỗi để xem test log làm bằng chứng cho DEF-0042:

```bash
TOINOI_PARCEL_IMPL=buggy npx vitest run articles/testware-san-pham-kiem-thu/parcel-surcharge.test.ts   # bash
$env:TOINOI_PARCEL_IMPL='buggy'; npx vitest run articles/testware-san-pham-kiem-thu/parcel-surcharge.test.ts   # PowerShell
```

Kết quả mong đợi với bản lỗi: **3 test đỏ** (TD-04 6,0 kg; TD-06 20,0 kg; TD-08 30,0 kg — đúng các khối lượng nguyên trên 5 kg), 7 test đạt, 1 test bỏ qua (test qua API luôn dùng mã thật nên bị tắt ở chế độ này).

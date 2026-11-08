# p01/thu-nghiem — thí nghiệm SQL trong các bài PG-001…PG-010

Những câu SQL bài viết dùng để minh hoạ nhưng không thuộc luồng chính `00`…`07` của lab: ví dụ lỗi thường gặp, câu đối chiếu, lời giải bài tập. Mỗi tệp chạy **sau** một nền dữ liệu nhất định — chạy sai nền thì kết quả (mã chuyến, số dòng) sẽ khác trong bài.

```bash
# ví dụ: thí nghiệm của PG-004 cần nền 00 → 03 (03 tiêu số của sequence)
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/03-fixture-invalid.sql p01/thu-nghiem/pg004-kieu-du-lieu.sql
```

| Bài | Tệp | Chạy sau |
|---|---|---|
| PG-001 | `pg001-moi-truong.sql` | `00` → `02` |
| PG-002 | `pg002-phien-ban-mui-gio.sql` | không cần nền |
| PG-002 | `pg002-collation.sql` | không cần nền |
| PG-003 | `pg003-khoa-va-quan-he.sql` | `00` → `02` |
| PG-003 | `pg003-di-theo-khoa.sql` | `00` → `02` |
| PG-003 | `pg003-bai-tap-2.sql` | `00` → `02` |
| PG-004 | `pg004-kieu-du-lieu.sql` | `00` → `03` |
| PG-004 | `pg004-bai-tap-2.sql` | không cần nền |
| PG-005 | `pg005-ten-rang-buoc.sql` | `00` → `02` |
| PG-005 | `pg005-bai-tap.sql` | `00` → `02` |
| PG-006 | `pg006-update-0-lost-update.sql` | `00` → `04` |
| PG-006 | `pg006-bai-tap.sql` | `00` → `04` |
| PG-006 | `pg006-them.sql` | `00` → `04` |
| PG-007 | `pg007-loi-thuong-gap.sql` | `00` → `02` |
| PG-008 | `pg008-bang-phang-bat-thuong.sql` | `00` → `02` |
| PG-008 | `pg008-rang-buoc-lien-bang.sql` | `00` → `02` |
| PG-008 | `pg008-bai-tap-3.sql` | `00` → `02` |
| PG-009 | `pg009-bai-tap-1.sql` | `00` → `02` |
| PG-009 | `pg009-bai-tap-3.sql` | `00` → `04`, `06`, `06-migration-rollback` |
| PG-010 | `pg010-mo-rong-fk-hai-cot.sql` | `00` → `02` |

`npm test` chạy mọi tệp trên đúng nền ghi trong `manifest.json` (`test/thu-nghiem.test.mjs`). Các tệp này **không** nằm trong `npm run p01` và không ảnh hưởng `OUTPUT.txt`. Lỗi cố ý được đánh dấu `-- expect-error: <SQLSTATE> [tên]` ngay trước câu lệnh; chỉ thị phải đứng một mình trên dòng (runner không nhận chữ thừa phía sau).

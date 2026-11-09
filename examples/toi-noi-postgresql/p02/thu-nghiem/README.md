# p02/thu-nghiem — thí nghiệm SQL trong các bài PG-011…PG-020

Những câu SQL bài viết dùng để minh hoạ nhưng không thuộc luồng chính `00`…`20` của lab: lỗi thường gặp, câu đối chiếu, lời giải bài tập. Nền của mọi tệp là `p01/00-reset.sql p01/01-schema.sql` rồi `p02/00` → `p02/02` (schema mở rộng, fixture, kiểm tra fixture); thí nghiệm của PG-017 cần thêm `p02/17-upsert-on-conflict.sql` (bảng thực đơn và lô nhập). Chạy sai nền thì số liệu sẽ khác trong bài.

```bash
# ví dụ: thí nghiệm của PG-013
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/thu-nghiem/pg013-bon-chuyen-no-dong.sql
```

| Bài | Tệp | Chạy sau |
|---|---|---|
| PG-011 | `pg011-bai-tap.sql` | `00` → `02` |
| PG-011 | `pg011-logic-ba-gia-tri.sql` | `00` → `02` |
| PG-012 | `pg012-bai-tap.sql` | `00` → `02` |
| PG-012 | `pg012-thu-nghiem.sql` | `00` → `02` |
| PG-013 | `pg013-bai-tap.sql` | `00` → `02` |
| PG-013 | `pg013-bon-chuyen-no-dong.sql` | `00` → `02` |
| PG-014 | `pg014-bai-tap.sql` | `00` → `02` |
| PG-014 | `pg014-kiem-ke-bat-loi.sql` | `00` → `02` |
| PG-015 | `pg015-bai-tap.sql` | `00` → `02` |
| PG-015 | `pg015-khung-cua-so.sql` | `00` → `02` |
| PG-016 | `pg016-bai-tap.sql` | `00` → `02` |
| PG-016 | `pg016-mui-gio.sql` | `00` → `02` |
| PG-017 | `pg017-bai-tap-1-gia-0.sql` | `00` → `02`, `17` |
| PG-017 | `pg017-bai-tap-2-bo-r3.sql` | `00` → `02`, `17` |
| PG-017 | `pg017-do-nothing-khong-target.sql` | `00` → `02`, `17` |
| PG-018 | `pg018-bai-tap-1-theo-cuoc.sql` | `00` → `02` |
| PG-018 | `pg018-con-tro-mot-cot-ket-trang.sql` | `00` → `02` |
| PG-018 | `pg018-nguoc-chieu-doi-dau.sql` | `00` → `02` |
| PG-018 | `pg018-trang-truoc-con-trang-sau.sql` | `00` → `02` |
| PG-019 | `pg019-bai-tap-1.sql` | `00` → `02` |
| PG-019 | `pg019-bai-tap-2.sql` | `00` → `02` |
| PG-019 | `pg019-bai-tap-3.sql` | `00` → `02` |
| PG-019 | `pg019-chu-trinh-them.sql` | `00` → `02` |
| PG-019 | `pg019-vong-lap-va-loi.sql` | `00` → `02` |
| PG-020 | `pg020-bai-tap.sql` | `00` → `02` |
| PG-020 | `pg020-cau-noi.sql` | `00` → `02` |

`npm test` chạy mọi tệp trên đúng nền ghi trong `manifest.json` (`test/p02-thu-nghiem.test.mjs`). Các tệp này **không** nằm trong `npm run p02` và không ảnh hưởng `OUTPUT.txt`. Chỉ thị `-- expect-error`, `-- expect-rows`, `-- expect-value`, `-- expect-line`, `-- expect-md5` phải đứng một mình trên dòng.

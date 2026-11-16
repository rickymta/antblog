# P01 · Nền tảng và mô hình document

Lab cho mười bài MG-001 … MG-010 trong [khung chương trình](../../../docs/mongodb-master-curriculum.md). Sản phẩm của phần: **collection cho thành phố, khách, tài xế, quán (menu nhúng) và chuyến (lịch sử nhúng); validator `$jsonSchema` + quy tắc liên trường; index unique; fixture hợp lệ/sai; kiểm tra bất biến liên collection** — trên MongoDB 8.0 thật. Cách cài và chạy chung ở [README của lab](../README.md).

```sh
npm run db:up && npm run p01     # 8 file, 115 PASS, kết quả như OUTPUT.txt
npm test                         # so khớp OUTPUT.txt, thử rollback, thí nghiệm MG-004
```

Trạng thái kiểm chứng: **MongoDB 8.0.38 Community, replica set một node**, Docker Desktop 29.9, Node 22.18.0, Windows 11 — 115 PASS, 0 FAIL.

## Các file

| File | Minh hoạ | Bài dùng | Kết quả mong đợi |
|---|---|---|---|
| `00-reset.js` | Xoá database `toinoi`, đọc phiên bản, replica set, FCV; database chỉ xuất hiện khi có collection | MG-001, MG-002 | `8.0.38\|rs0\|PRIMARY\|toinoi\|8.0` |
| `01-schema.js` | 6 collection, validator `$jsonSchema` (kiểu BSON, `required`, `enum`, `additionalProperties: false`) + 5 quy tắc liên trường trong `$and` của `trips`; 5 index ngoài `_id` | MG-003, MG-005 | 10 index; `trips` strict/error |
| `02-fixture-valid.js` | 3 thành phố, 6 khách, 6 tài xế, 3 quán có menu nhúng, 15 chuyến (trường không áp dụng thì vắng, không `null`), bộ đếm mã | MG-003, MG-006, MG-007, MG-010 | 7 PASS; document chuyến 100004; bảng 8 trạng thái × 3 loại |
| `03-fixture-invalid.js` | 37 ca sai, mỗi ca vi phạm đúng một quy tắc (121 và 11000; ca liên trường kiểm đúng điều kiện #1…#5), đọc `errInfo.details`; update cũng bị kiểm; 4 bẫy lọt qua (không khoá ngoại, `city_id` không tồn tại, collection gõ sai tên tự tạo, `bypassDocumentValidation`) | MG-004, MG-005, MG-010 | 43 PASS; dữ liệu vẫn 3/6/6/3/15 |
| `04-lifecycle.js` | Tài xế 8007; mã chuyến từ bộ đếm (`findOneAndUpdate` + `$inc`); chuyến 100015 qua 7 trạng thái bằng update có điều kiện + `$push`; hai tài xế tranh nhận (người sau khớp 0); `findOneAndUpdate` trả bản sau; `updateMany`, upsert, `replaceOne` mất trường; xoá khách không bị chặn → chuyến mồ côi → khôi phục từ `findOneAndDelete` | MG-006 | Mã **100015** (không có lỗ hổng), hoàn thành sau 31 phút, 7 sự kiện |
| `05-find.js` | Khoảng nửa mở theo giờ VN, ngày UTC vs ngày VN, so Date/UUID với chuỗi ra 0, khoá phụ khi sắp xếp, skip/limit, projection và `$slice`, `$elemMatch`, `null` vs vắng, `$ne` khớp cả document vắng trường, collation `vi`/`en` strength 1 | MG-007 | 8 vs 9 chuyến ngày 02/11; `vi` strength 1 không coi "van" = "Văn" |
| `06-migration.js` | Thêm `vehicle_plate` + `schema_version` cho tài xế: expand (`collMod`), backfill `bulkWrite` `ordered:false` có dòng bẩn (`MongoBulkWriteError`), idempotent, kiểm `$nor`, `validationAction: "warn"` và log 20294, contract, `validate()` | MG-009 | 12 PASS; 8 tài xế có biển số |
| `06-migration-rollback.js` | Hoàn tác: nới validator → sao lưu biển số → `$unset` → validator v1; chạy lặp được | MG-009 | Không chạy trong `npm run p01`; test chạy 2 lần |
| `07-checks.js` | 12 kiểm tra mong đợi 0 vi phạm: tham chiếu bằng `$lookup`, thành phố quán/tài xế, tài xế busy ⇔ một chuyến chạy, sự kiện cuối = trạng thái, thứ tự thời gian, bộ đếm, `validate()` mọi collection, index unique; đối soát doanh thu; kiểm kê | MG-008, MG-010 | 15 PASS; doanh thu 397.000 đ; 3/6/8/3/16 |
| `OUTPUT.txt` | Kết quả thật của `npm run p01` | Mọi bài | Test so khớp (bỏ dòng "Đã xoá database toinoi cũ") |
| `thu-nghiem/mg004-kieu-bson.js` | Số trong mongosh (int/double, `2**31` thành double), 0.1+0.2 double vs Decimal128, `$sum` tự lên long, Date không múi giờ, `$dateTrunc` theo giờ VN, UTF-8 và NFC/NFD, ObjectId chứa thời điểm, UUID 16 byte, so khác kiểu | MG-004 | `npm run p01:thu-nghiem`, 13 PASS |

## Khác biệt có chủ đích so với lab PostgreSQL / SQL Server

Cùng nghiệp vụ, cùng dữ liệu, nhưng thiết kế theo MongoDB:

| Chủ đề | PostgreSQL / SQL Server | MongoDB (lab này) |
|---|---|---|
| Đơn vị lưu | Dòng trong bảng, cột cố định | Document BSON trong collection, hình dạng do validator giới hạn |
| Lịch sử trạng thái | Bảng `trip_events` riêng, ghi cùng transaction | Mảng `events` nhúng trong chuyến; một `updateOne` đổi trạng thái và `$push` là nguyên tử |
| Menu quán | Bảng riêng (chưa có ở P01) | Mảng `menu` nhúng trong quán |
| Định danh | `uuid` / `uniqueidentifier` | `UUID()` (binary subtype 4); `ObjectId` là mặc định khi không chỉ định |
| Mã chuyến tăng dần | `IDENTITY` / sequence | Collection `counters` + `findOneAndUpdate` `$inc` |
| Ràng buộc | PK, FK, UNIQUE, CHECK | `_id` + index unique; validator thay CHECK; **không có khoá ngoại** — 07 kiểm bằng `$lookup` |
| Trường không áp dụng | `NULL` | Trường vắng mặt; validator cấm `null` |
| Lấy lại document vừa ghi | `RETURNING` / `OUTPUT` | `findOneAndUpdate` / `findOneAndDelete` với `returnDocument` |
| Ràng buộc chưa kiểm dữ liệu cũ | `NOT VALID` / `WITH NOCHECK` | `validationLevel: "moderate"` hoặc `validationAction: "warn"`; kiểm bằng `$nor` và `validate()` |
| Ghi nhầm tên bảng | Lỗi "không tồn tại" | Collection mới được tự tạo, không báo gì |

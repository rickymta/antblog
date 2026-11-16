# Lab MongoDB 8.0 · Tới Nơi

Mã thực hành cho loạt bài **MongoDB 8.0 qua dự án Tới Nơi**. Đọc cùng [khung chương trình](../../docs/mongodb-master-curriculum.md) và [hồ sơ dự án mẫu Tới Nơi](../../api/seed/content/du-an-mau-toi-noi.md). Tiền là số nguyên đồng (BSON `int`/`long`), thời điểm là BSON `Date` (UTC, hiển thị theo giờ VN); mọi tên người, số điện thoại, biển số đều **hư cấu**. Dữ liệu trùng với lab PostgreSQL và SQL Server để đối chiếu.

| Phần | Chủ đề | Thư mục | Trạng thái |
|---|---|---|---|
| P01 | Nền tảng và mô hình document | [`p01/`](p01/README.md) | 8 script mongosh, `OUTPUT.txt`, thí nghiệm, test — **đã chạy trên MongoDB 8.0.38 thật** (115 PASS, 0 FAIL) |

## Phiên bản cố định

| Thành phần | Phiên bản | Ghi chú |
|---|---|---|
| Docker Desktop | đã kiểm 29.x (engine Linux) | Container giới hạn 1 GB RAM (`mem_limit: 1g`) |
| Ảnh MongoDB | `mongo:8.0@sha256:d97fea17…` | = **MongoDB 8.0.38** Community, FCV 8.0. Ghim digest để mọi người chạy cùng bản; đổi bản thì chạy lại `npm test` và cập nhật `OUTPUT.txt` |
| mongosh | có sẵn trong ảnh | Không cần cài trên máy |
| Node.js | ≥ 22 (đã kiểm 22.18.0) | Chỉ để chạy `run.mjs` và test; không có dependency npm |
| Công cụ GUI | DBStudio 0.1.7 | Tuỳ chọn; mọi ảnh minh hoạ trong series chụp từ bản này trên Windows 11 (MG-002) |

## Chạy

```sh
cd examples/toi-noi-mongodb
npm run db:up        # docker compose up -d --wait — tự khởi tạo replica set rs0, chờ "healthy"
npm run p01          # 00 → 07 trên database toinoi mới, in kết quả + bảng đếm
npm test             # chạy lại p01, so với OUTPUT.txt, thử rollback và thí nghiệm
npm run db:down      # docker compose down -v — dừng và XOÁ dữ liệu lab
```

Kết quả mong đợi của `npm run p01` kết thúc bằng:

```text
// [lab] File đã chạy: 8
// [lab] PASS: 115
// [lab] FAIL: 0
// [lab] KẾT QUẢ: ĐẠT
```

**Replica set một node.** `mongod` chạy với `--replSet rs0`; healthcheck gọi `rs.initiate()` ở lần đầu rồi chỉ báo "healthy" khi node đã là PRIMARY. Transaction nhiều document, change stream và oplog (Phần 4, 6) cần replica set, nên lab dựng sẵn từ đầu.

**Runner.** `run.mjs` gọi `docker compose exec -T db mongosh --quiet --norc <URI> /lab/lib/check.js /lab/<file>` cho từng file (`lib/` và `p01/` được mount chỉ đọc vào `/lab`). Mỗi file là một kết nối riêng. `lib/check.js` định nghĩa `table()` (in document thành bảng `cột|cột`, Date theo giờ VN), `expectValue()`, `expectError()` (PASS khi lỗi đúng mã: 121 validation, 11000 trùng khoá; tham số số = chỉ số điều kiện `$and` trong validator bị vi phạm) và `vn()` (giờ VN → Date). Runner đếm các dòng bắt đầu bằng `PASS ` / `FAIL `; lỗi không được bắt làm mongosh thoát mã khác 0 và tính là FAIL.

## Kết nối từ máy của bạn

| Tham số | Giá trị |
|---|---|
| Máy chủ / cổng | `127.0.0.1` / `27018` (chỉ mở trên máy của bạn; tránh đụng MongoDB có sẵn ở 27017) |
| Xác thực | **Không** — chỉ cho lab cục bộ. Phần 5 bật xác thực. Không bao giờ mở MongoDB không xác thực ra mạng |
| Replica set | `rs0`, một node. Từ máy ngoài container phải thêm `directConnection=true` (cấu hình replica set ghi tên `localhost:27017`, là địa chỉ bên trong container) |
| Database | `toinoi` (sau khi chạy `p01/00-reset.js` … `02`) |

Chuỗi dán vào ô "URI kết nối" của DBStudio:

```text
mongodb://127.0.0.1:27018/toinoi?directConnection=true
```

## Dọn môi trường

`npm run db:down` xoá container và volume `toinoi-mongo8-data`. Ảnh Docker (~0,9 GB) vẫn còn; xoá bằng `docker image rm mongo:8.0` nếu không dùng nữa.

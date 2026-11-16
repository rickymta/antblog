# Lab Apache Cassandra 5.0 · Tới Nơi

Mã thực hành cho loạt bài **Apache Cassandra 5.0 qua dự án Tới Nơi**. Đọc cùng [khung chương trình](../../docs/cassandra-master-curriculum.md) và [hồ sơ dự án mẫu Tới Nơi](../../api/seed/content/du-an-mau-toi-noi.md). Tiền là số nguyên đồng (`bigint`), thời điểm là `timestamp` ghi kèm độ lệch `+0700`; mọi định danh là UUID cố định, mọi địa điểm là hư cấu hoặc địa danh công cộng.

| Phần | Chủ đề | Thư mục | Trạng thái |
|---|---|---|---|
| P01 | Nền tảng và mô hình hoá theo truy vấn | [`p01/`](p01) | 9 file CQL + 1 script, `OUTPUT.txt`, test — **đã chạy trên Cassandra 5.0.9 thật** (20 PASS, 0 FAIL) |

## Phiên bản cố định

| Thành phần | Phiên bản | Ghi chú |
|---|---|---|
| Docker Desktop | đã kiểm 29.9 (engine Linux, WSL 2) | Mỗi node giới hạn `mem_limit: 1600m`, heap 768 MB |
| Ảnh Cassandra | `cassandra:5.0.9@sha256:acd754f0…` | Ghim digest; đổi bản thì chạy lại `npm test` và cập nhật `OUTPUT.txt` |
| cqlsh, nodetool | có sẵn trong ảnh | Không cần cài Java/Python trên máy |
| Node.js | ≥ 22 (đã kiểm 22.18.0) | Chỉ để chạy `run.mjs` và test; không có dependency npm |
| Công cụ GUI | DBStudio 0.1.7 | Tuỳ chọn, để xem và chạy truy vấn bằng giao diện (CS-002) |

## Chạy

```sh
cd examples/toi-noi-cassandra
npm run db:up        # docker compose up -d --wait — chờ node1 "healthy" (lần đầu ~1 phút)
npm run p01          # 00 → 09 trên keyspace toinoi mới, in kết quả + bảng đếm
npm test             # chạy lại p01 và so từng dòng với p01/OUTPUT.txt
npm run db:down      # dừng và XOÁ dữ liệu lab (cả 3 node nếu đã bật cụm)
```

Kết quả mong đợi của `npm run p01` kết thúc bằng:

```text
-- [lab] File đã chạy: 10
-- [lab] PASS: 20
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

**Runner.** `run.mjs` chạy từng file `p01/NN-*.cql` bằng `cqlsh -f` **bên trong** container `node1` (một phiên cqlsh cho mỗi file) và `p01/NN-*.sh` bằng `sh` trong container (nodetool, COPY). Trong file CQL:

- `-- ## Tiêu đề` được in vào kết quả để biết đoạn kết quả thuộc câu nào (runner chèn một dòng mốc rồi thay bằng tiêu đề, vì cqlsh không in lại câu lệnh).
- `-- EXPECT: <chuỗi>` đứng ngay trước một câu lệnh nghĩa là câu đó **phải** lỗi và thông điệp chứa `<chuỗi>` → `PASS`; không lỗi hoặc lỗi khác → `FAIL`. Mọi lỗi không khai báo đều là `FAIL`.

`npm test` bỏ qua những gì phụ thuộc máy và thời điểm chạy: thời gian của `COPY FROM`, độ trễ và ước lượng kích thước trong `nodetool tablestats/tablehistograms`.

## Cổng và kết nối từ máy của bạn

| Node | Cổng CQL trên máy | Bật bằng |
|---|---|---|
| `node1` | `127.0.0.1:19042` | `npm run db:up` |
| `node2`, `node3` | `19043`, `19044` | `docker compose --profile cluster up -d --wait` (Phần 3) |

Mỗi node đặt `native_transport_port` bằng cổng mở ra máy (`docker/entry.sh`) và quảng bá `broadcast_rpc_address = 127.0.0.1`. Lý do: driver Cassandra (DBStudio, ứng dụng) hỏi node danh sách node trong cụm rồi kết nối tới **địa chỉ node tự khai**. Nếu node khai IP nội bộ của mạng Docker (172.x), driver trên máy bạn không với tới và báo `Connect timeout`. Lab không bật xác thực (PasswordAuthenticator bật ở CS-041).

Chuỗi dán vào ô "URI kết nối" của DBStudio, rồi chọn **Xác thực: Không cần xác thực**, **TLS: Tắt**:

```text
cassandra://127.0.0.1:19042/toinoi
```

## Ảnh minh hoạ

Ảnh DBStudio của các bài nằm ở `apps/web/public/images/articles/cassandra/` và được chụp tự động bằng [`shots/`](shots/README.md). Chụp lại sau `npm run p01` (ảnh đọc đúng trạng thái dữ liệu đó).

## Dọn môi trường

`npm run db:down` xoá container và volume `node*-data`. Ảnh Docker (~360 MB) vẫn còn; xoá bằng `docker image rm cassandra:5.0.9` nếu không dùng nữa.

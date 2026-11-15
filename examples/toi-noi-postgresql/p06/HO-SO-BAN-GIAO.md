# Hồ sơ bàn giao capstone P06 · Tới Nơi

Phạm vi: diễn tập local bằng `npm run p06` và `npm run p06:extensions`. Mọi thao tác phá dữ liệu (crash, promote, rewind, nâng phiên bản) chỉ chạm cluster tạm do runner tạo. Production cần người phụ trách, quyền thực thi và mục tiêu được xác nhận riêng.

## Sản phẩm bàn giao

| Mã | Nội dung | Bằng chứng trong lab |
|---|---|---|
| D1 | Schema và quyết định mô hình: partition GPS theo tháng, JSONB cho thuộc tính thiết bị, cột generated cho trường được lọc | 051, 053 |
| D2 | Truy vấn có kế hoạch kiểm chứng: pruning tĩnh và run-time, index GIN | 051, 053, 054 |
| D3 | Vòng đời dữ liệu: partition tạo trước, DETACH CONCURRENTLY, đối soát trước DROP | 054 |
| D4 | Projection báo cáo: định nghĩa độ tươi, REFRESH CONCURRENTLY, đối soát với sổ gốc | 056, 060 |
| D5 | Tích hợp: logical replication, slot, quy trình đổi schema hai phía | 055 |
| D6 | Sẵn sàng cao: standby, đọc-sau-ghi theo LSN, failover, fencing, pg_rewind | 052, 058 |
| D7 | Nâng phiên bản: rehearsal hai đường, cổng go/no-go, giới hạn quay lại | 059 |
| D8 | Đánh giá extension theo workload, có điều kiện dừng | 057 |
| D9 | Runbook phục hồi từ P05 (PITR, quyền, RLS) | `../p05/RUNBOOK.md` |

## Cổng go/no-go trước khi mở lại ghi sau sự cố

1. **Một writer duy nhất.** Node cũ đã bị fence (dừng, chặn mạng hoặc gỡ khỏi routing) trước khi promote. Lab 058 cho thấy node cũ khởi động lại vẫn nhận ghi và `target_session_attrs=read-write` sẽ chọn nó nếu đứng đầu danh sách host.
2. **Biết đã mất gì.** Ghi `pg_current_wal_flush_lsn()` cuối của node cũ (nếu đọc được) và `pg_last_wal_receive_lsn()` của node được promote; phần chênh là WAL chưa tới standby. Lab: 184 byte, gồm chuyến 5 đã commit.
3. **Sổ gốc đúng.** Đối soát số chuyến, tổng tiền và digest với nguồn đối chiếu độc lập (log ứng dụng, cổng thanh toán) trước khi mở ghi. Lab chỉ kiểm số chuyến và tổng tiền (5 chuyến / 250.000 đ) và không có nguồn độc lập; digest theo cách của P05 là việc cần thêm.
4. **Projection đúng hoặc được đánh dấu cũ.** Báo cáo sau failover lệch sổ (200.000 so với 250.000) cho tới khi REFRESH. Không restore đè sổ gốc chỉ vì báo cáo cũ.
5. **Đường ghi thật.** Thử một thao tác ghi qua đúng đường ứng dụng sẽ mở lại (connection string, pool, quyền). Lab chỉ ghi bằng psql với superuser, nên cổng này chưa được chứng minh.
6. **Node cũ có số phận rõ ràng.** pg_rewind thành standby của timeline mới, hoặc dựng lại từ base backup; không để nó chạy độc lập với dữ liệu phân nhánh.

## Cổng go/no-go cho nâng phiên bản

| Cổng | Lab 059 | Production cần thêm |
|---|---|---|
| `pg_upgrade --check` sạch trên bản sao | Phát hiện checksum khác nhau, sửa bằng initdb `--no-data-checksums` | Extension, collation/ICU, tablespace, kiểu dữ liệu bị loại bỏ |
| Digest sau nâng khớp nguồn | Khớp cả hai đường | Đối soát theo bảng nghiệp vụ, không chỉ một bảng |
| Sequence/identity tiếp tục đúng | `RETURNING id` = 3 | Mọi sequence, kể cả sequence dùng chung |
| Statistics sẵn sàng | dump/restore: 0 dòng `pg_stats` trước ANALYZE; pg_upgrade 18: có | `vacuumdb --analyze-in-stages` cho phần statistics không được mang sang |
| Còn đường quay lại | Copy mode: cluster 17 vẫn khởi động | Thời hạn giữ cluster cũ; ghi mới sau cutover không tự quay về (059 A: nguồn vẫn 2 dòng) |

## Việc lab chưa chứng minh

- RTO đầu cuối: phát hiện sự cố, quyết định, fencing, chuyển DNS/VIP, pool kết nối lại.
- Synchronous replication và RPO = 0; mất đồng thời hai node; replication slot trên standby được promote.
- Logical replication khi cutover nâng phiên bản; conflict khi apply; theo dõi độ trễ apply liên tục.
- PostGIS/pgvector trên server thật với dữ liệu và tải thật; chi phí ghi của index HNSW.
- Backup ngoài máy, mã hoá, lưu giữ và diễn tập phục hồi định kỳ (xem P05).

Các mục này ghi là phép thử còn thiếu; không cam kết RPO/RTO cho tới khi đo được.

# Schema Parquet lịch sử GPS, phiên bản 1

Do `go/cmd/history-writer` ghi. **Hợp đồng với phía Python (LOC-51)**: không đổi tên cột, không đổi kiểu, không đổi ý nghĩa. Muốn thêm cột thì thêm vào cuối và tăng `toinoi.schema_version`.

## Bố cục thư mục (kiểu Hive)

```text
<root>/date=2026-11-06/city=HCM/part-<unix_ms>-<n>.parquet
```

- `date`: ngày theo **giờ Việt Nam (UTC+7)** của `event_time`, không phải UTC. Một điểm lúc 18:00 UTC ngày 06 nằm ở `date=2026-11-07`.
- `city`: mã thành phố (`HCM`, `HN`, `DN`, `CT`, `HP`, `NT`, hoặc `UNK` nếu thiếu).
- `date` và `city` **chỉ nằm trong đường dẫn**, không có trong file. Polars: `scan_parquet(".../**/*.parquet", hive_partitioning=True)`. PyArrow: `pyarrow.dataset.dataset(root, partitioning="hive")`.
- Mỗi lần flush ghi một file mới. File được ghi vào tên tạm bắt đầu bằng dấu chấm rồi đổi tên, nên người đọc (bỏ qua file bắt đầu bằng `.`) không bao giờ thấy file dở. Có thể có nhiều file mỗi phân vùng.
- Giao hàng *ít nhất một lần* (commit offset sau khi ghi file): sau sự cố có thể có dòng trùng. Khử trùng theo `(driver_ref, event_time, seq)`.

## Cột

| Cột | Kiểu Parquet | Null? | Ý nghĩa |
|---|---|---|---|
| `event_time` | INT64 TIMESTAMP(MILLIS, UTC) | không | giờ lấy mẫu, đã quy về đồng hồ server |
| `received_time` | INT64 TIMESTAMP(MILLIS, UTC) | không | giờ `location-ingest` nhận lô |
| `driver_ref` | STRING (dictionary) | có | theo `id_level`: 0 = `driver_id`, 1 = HMAC theo tháng (16 ký tự hex), 2 = NULL |
| `id_level` | INT32 | không | 0 raw, 1 pseudonym, 2 coarse. Hằng số trong một thư mục đầu ra |
| `trip_id` | STRING (dictionary) | có | NULL khi chưa có chuyến, và luôn NULL ở mức 2 |
| `lat_e7`, `lng_e7` | INT32 | không | độ × 10^7. Mức 2: làm tròn về bội của 10^4 (≈ 0,001° ≈ 111 m) |
| `accuracy_dm` | INT32 | có | 0,1 m; NULL = app không báo |
| `speed_dmps` | INT32 | có | 0,1 m/s; NULL = không báo, **0 = đứng yên** |
| `heading_ddeg` | INT32 | có | 0,1 độ, 0..3599 |
| `seq` | INT64 | không | số thứ tự điểm của app |
| `flags` | INT32 | không | bit mask `PointFlag` (xem `location.proto`) |
| `sample_reason` | STRING (dictionary) | không | vì sao điểm được giữ: `first`, `move`, `turn`, `heartbeat`, `flagged` (hoặc `all` khi không giảm mẫu) |

Metadata mỗi file (key-value): `toinoi.schema_version = 1`, `toinoi.id_level = 0|1|2`. Nén ZSTD. Dòng sắp theo `(driver_ref, event_time)` trong mỗi file.

## Giảm mẫu

Điểm GPS đến mỗi 4 giây; lịch sử chỉ giữ điểm khi: là điểm đầu của tài xế, **hoặc** có cờ, **hoặc** đã di chuyển ≥ 60 m từ điểm giữ gần nhất, **hoặc** đổi hướng ≥ 30° sau ≥ 4 s, **hoặc** đã 60 s chưa giữ điểm nào (đứng yên vẫn có nhịp tim). Vì vậy **khoảng cách thời gian giữa hai dòng không đều**; đừng giả định lấy mẫu 4 giây khi dựng đặc trưng. Cột `sample_reason` cho biết lý do giữ.

## Mức định danh

| Mức | Thư mục gợi ý | `driver_ref` | Toạ độ | Dùng cho | Retention gợi ý (chính sách, không phải luật) |
|---|---|---|---|---|---|
| 0 raw | `history-raw/` | `driver_id` thật | nguyên | điều tra, tranh chấp, bằng chứng | ngắn nhất, truy cập hạn chế |
| 1 pseudonym | `history-pseudo/` | HMAC(khoá tháng, `driver_id`) | nguyên | pipeline ETA, đặc trưng chuyến (LOC-51) | dài hơn |
| 2 coarse | `history-coarse/` | NULL | ≈ 111 m | nhu cầu theo ô, huấn luyện dự báo | dài nhất |

Mức 1 **vẫn là dữ liệu cá nhân**: quỹ đạo đủ để nhận ra người. HMAC chỉ che mã tài xế và không nối được qua tháng (khoá đổi theo tháng, tức cùng một tài xế có hai `driver_ref` khác nhau ở hai tháng). Con số retention trên là điểm khởi đầu để chốt với pháp chế, cần kiểm lại văn bản pháp luật hiện hành khi viết bài.

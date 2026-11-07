# Lab 55 · Feature engineering từ event Kafka: velocity, thiết bị, GPS và leakage

Đi kèm bài `python-feature-engineering-tu-event-kafka`. Lab biến log event của bộ dữ liệu gian lận tổng hợp (bài 54) thành đặc trưng, đo chúng, tái hiện leakage bằng số thật, rồi tính lại cùng đặc trưng theo kiểu online từ envelope Kafka.

- `synthetic_fraud.py`: copy NGUYÊN từ lab 54 (`make_events(seed=42, n_trips=200_000)`, schema ghi trong docstring đầu file). Không sửa.
- `features.py`: `trip_table` nối ba loại event theo `trip_id`; `add_features` (từ chối bảng chưa sắp theo `paid_at`, vì `rolling().over()` sai âm thầm) thêm 30 đặc trưng bằng polars (`Expr.rolling(..., closed="left")` theo thiết bị, khách, tài xế, cặp khách-tài xế; `haversine_expr`; nhảy vị trí so với chuyến trước của cùng tài xế). Cuối file là các đặc trưng RÒ RỈ (cửa sổ nhìn tương lai, gom theo giờ bằng `group_by_dynamic`, lịch sử gian lận tính ngay từ nhãn cuối) chỉ để tái hiện lỗi.
- `online.py`: `OnlineFeatures` nhận từng envelope, giữ danh sách (thời điểm, giá trị) theo khóa bằng `bisect`, trả đặc trưng khi chuyến đủ ba event. Phủ đủ 30 đặc trưng, cùng định nghĩa với bản polars.
- `stream.py`: gửi envelope qua Kafka thật (aiokafka, ba topic `lab55.*`, mỗi topic một partition) rồi đọc lại bằng một consumer không dùng consumer group.
- `main.py`: sáu bước và `--verify`. Ghi `out/features.parquet` (gitignore).

## Hạ tầng cần

Kafka trong hạ tầng dùng chung (`localhost:9094`, ghi đè bằng `TOINOI_KAFKA_BOOTSTRAP`):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

Lab tạo ba topic `lab55.*` (xóa leftover từ lần chạy hỏng) và xóa chúng khi chạy xong; không tạo consumer group. Khi chưa có Kafka: `FAIL: không nối được Kafka tại <url> (docker compose ...)` và exit code 1.

## Chạy

Cần uv và Python 3.14; lần đầu cần mạng để tải numpy, polars, scikit-learn, aiokafka. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/55-feature-engineering-event examples/toi-noi-python/55-feature-engineering-event/main.py
uv run --project examples/toi-noi-python/55-feature-engineering-event examples/toi-noi-python/55-feature-engineering-event/main.py --verify
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 33/33` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/33` với k là số ca đạt.
- Thời gian chạy phụ thuộc tải máy: chưa có số đo trên máy rảnh (các lần chạy ghi trong bài đều ở load average hàng chục đến hàng trăm trên 10 lõi, mỗi lần từ vài phút đến hơn mười phút). Huấn luyện chín mô hình HistGradientBoosting, hai hồi quy logistic và đọc/ghi Kafka là phần lâu nhất. `--verify` chỉ kiểm bất biến và thứ bậc lớn, không kiểm số tuyệt đối hay tốc độ, và đạt ở seed 1 và 2 của bộ sinh.
- Số đo trong bài (PR-AUC, số chuyến lệch, số chuyến đảo thứ tự) phụ thuộc phiên bản numpy/polars/scikit-learn và thứ tự giao bản ghi của broker; số chuyến đảo thứ tự giữa các topic có thể khác giữa các lần chạy, `--verify` chỉ đòi nó lớn hơn 0.
- Mọi dữ liệu là tổng hợp; số đo không đại diện cho dữ liệu thật. Điểm gian lận là đầu vào cho người duyệt, không tự khóa tài khoản.

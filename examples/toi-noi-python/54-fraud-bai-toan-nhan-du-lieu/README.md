# Lab 54 · Gian lận trong Tới Nơi: loại hình, nhãn, dữ liệu và metric

Đi kèm bài `python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric`, bài mở đầu phần 07 (phát hiện gian lận). Lab dựng bộ dữ liệu gian lận TỔNG HỢP mà các bài 55 đến 61 dùng lại, rồi đo bốn điều trên chính bộ đó: tín hiệu của từng loại gian lận, độ trễ của nhãn, cách chia train/valid theo thời gian, và metric trên dữ liệu mất cân bằng (precision, recall, PR-AUC, ROC-AUC, chi phí hai chiều).

- `synthetic_fraud.py`: `make_events(seed=42, n_trips=200_000)` trả về `FraudData(events, labels)`. `events` là log `BookingCreated`/`TripCompleted`/`PaymentCaptured` (600.000 dòng, sắp theo thời gian, KHÔNG có cột nhãn); `labels` là bảng riêng một dòng mỗi chuyến (`fraud_type`, `is_fraud`, `label_known_at`). Docstring đầu file ghi đủ tên cột và kiểu. `iter_envelopes(events)` đổi mỗi dòng thành envelope Kafka. Các bài 55 đến 61 copy nguyên file này, không sửa.
- `dataset.py`: nối ba loại event thành bảng chuyến, đặc trưng theo từng chuyến (chưa có cửa sổ thời gian), nhãn biết đến một ngày nào đó (`known_label`), chia theo thời gian, bộ chấm điểm nền (hồi quy logistic).
- `metrics.py`: `average_precision`, `roc_auc`, ma trận nhầm lẫn, chi phí (phí duyệt cho mọi chuyến gắn cờ cộng thiệt hại của chuyến bỏ sót) và ngưỡng chọn theo chi phí, viết tay bằng numpy; verify đối chiếu với scikit-learn.
- `main.py`: tám bước và `--verify`. Ghi `out/events.parquet`, `out/labels.parquet` (gitignore).

Bốn loại gian lận trong bộ dữ liệu, tổng cộng 2,0 % số chuyến: `ghost_trip` (chuyến ảo, 0,7 %), `promo_abuse` (lạm dụng khuyến mãi, 0,6 %), `gps_spoof` (GPS giả, 0,4 %), `collusion` (cấu kết tài xế và khách, 0,3 %). Đây là dữ liệu tổng hợp: số liệu đo trên nó không đại diện cho dữ liệu thật của bất kỳ nền tảng nào, và điểm rủi ro chỉ là đầu vào cho người duyệt, không tự khóa tài khoản.

## Chạy

Cần uv và Python 3.14; lần đầu cần mạng để tải numpy, polars, scikit-learn. Không cần Docker. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu/main.py
uv run --project examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu/main.py --verify
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 43/43` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL n/43` với n là số ca đạt. Chưa có số đo thời gian chạy trên máy rảnh (các lần chạy ghi trong bài đều ở máy rất bận).
- Dữ liệu lặp lại được với cùng `seed` và `n_trips` trên cùng phiên bản numpy; bộ sinh số ngẫu nhiên của numpy không cam kết giữ nguyên dòng số giữa các phiên bản, nên các con số trong bài có thể lệch chút ít ở máy khác. `--verify` chỉ kiểm bất biến và thứ bậc lớn (ROC-AUC cao hơn hẳn PR-AUC, nhãn chưa chín làm PR-AUC đo được thấp hơn hẳn, chia ngẫu nhiên làm danh sách đen thiết bị trông tốt hơn chia theo thời gian...), không kiểm số tuyệt đối.
- Chi phí trong bước 6 (`REVIEW_COST_VND`, `HANDLING_COST_VND`) là giả định minh họa, không phải số liệu vận hành.
- Đã chạy với Python 3.14.7, numpy 2.5.4, polars 2.1.0, scikit-learn 1.9.1.

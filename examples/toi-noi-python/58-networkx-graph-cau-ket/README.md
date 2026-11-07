# Lab 58 · Đồ thị networkx: gắn cờ nhóm khách - tài xế - thiết bị cho người duyệt

Đi kèm bài `python-networkx-graph-cau-ket-tai-xe-khach`. Lab dựng đồ thị khách - tài xế - thiết bị theo cửa sổ thời gian từ bộ dữ liệu gian lận tổng hợp (bài 54), tìm nhóm nghi cấu kết (`collusion`), rồi đo đặc trưng đồ thị có cứu được điểm yếu của đặc trưng từng chuyến hay không. Nhóm bị gắn cờ là đầu vào cho người duyệt, không phải kết luận cấu kết.

- `synthetic_fraud.py`, `dataset.py`: copy NGUYÊN từ lab 54 (`make_events(seed=42, n_trips=200_000)`, hằng `TRAIN_END_DAY=30`, `VALID_START_DAY=40`). `features.py`: copy NGUYÊN từ lab 55 (30 đặc trưng cửa sổ trong `FEATURES`). Không sửa.
- `graph.py`: không đọc nhãn. `snapshot(trips, day)` dựng đồ thị đầu ngày `day` từ các chuyến đã thu tiền trong 28 ngày trước đó (polars lọc cặp từ 2 chuyến và thiết bị từ 2 tài khoản, networkx giữ đồ thị), tách thành phần lớn bằng Louvain, tính thống kê nhóm (mật độ, số chu trình độc lập, betweenness, core), luật `RULE` gắn cờ nhóm, và `add_graph_features` gắn 14 đặc trưng vào từng chuyến.
- `main.py`: tám bước và `--verify`. Nhãn chỉ dùng để đánh giá và để huấn luyện ở bước 5. Không ghi file ra đĩa. `main.py --digest` chỉ in dấu vân tay của ảnh chụp ngày 50; `--verify` gọi nó ở hai tiến trình mới với `PYTHONHASHSEED` khác nhau để chắc ảnh chụp không phụ thuộc thứ tự băm của tập hợp.

## Chạy

Cần uv và Python 3.14; lần đầu cần mạng để tải numpy, polars, scikit-learn, networkx. Không cần Docker. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/58-networkx-graph-cau-ket examples/toi-noi-python/58-networkx-graph-cau-ket/main.py
uv run --project examples/toi-noi-python/58-networkx-graph-cau-ket examples/toi-noi-python/58-networkx-graph-cau-ket/main.py --verify
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 53/53` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/53` với k là số ca đạt.
- Trên máy tương đối rảnh (load average dưới 60 trên 10 lõi) lab chạy khoảng 6 phút ở load average 60 đến 125; khi máy rất bận (load average 280 đến 385) đã đo tới 43 đến 58 phút. Phần lâu nhất là các ảnh chụp đồ thị và 10 lần huấn luyện HistGradientBoosting (bước 9 chạy lại chuỗi đo ở seed 7 và 123); lần `--verify` cuối mất 58 phút khi load average 280 đến 385, nên đừng suy ra thời gian trên máy rảnh. `--verify` chỉ kiểm bất biến và thứ bậc lớn (đồ thị thô có thành phần khổng lồ, g_edge_trips khớp phép đếm lại độc lập, kết luận đặc trưng đồ thị nâng collusion ở ba seed 42/7/123, lọc cạnh giữ dưới 10 %, ảnh chụp bất biến khi cắt event, bỏ cột nhãn và đổi `PYTHONHASHSEED`, đồ thị dựng trên cả 60 ngày đẹp offline nhưng tụt khi phục vụ, đặc trưng đồ thị nâng collusion), không kiểm tốc độ.
- Số đo trong bài (PR-AUC theo loại, số nhóm, byte mỗi cạnh) phụ thuộc phiên bản numpy/polars/scikit-learn/networkx; Louvain dùng `seed=0`. Mọi dữ liệu là tổng hợp; số đo không đại diện cho dữ liệu thật.
- Đã chạy với Python 3.14.7, numpy 2.5.4, polars 2.1.0, scikit-learn 1.9.1, networkx 3.7.

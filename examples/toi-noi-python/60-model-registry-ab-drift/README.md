# Lab 60 · Model registry, shadow mode, A/B, drift và rollback cho bộ chấm điểm gian lận

Đi kèm bài `python-model-registry-ab-drift-retraining`. Lab huấn luyện ba phiên bản bộ chấm điểm (công thức bài 57), đăng ký vào một registry dạng thư mục, rồi chạy shadow mode, A/B theo hash khách, đo drift bằng PSI, quyết định huấn luyện lại và rollback. Điểm gian lận là đầu vào cho người duyệt, không tự khóa tài khoản.

- `registry.py`: `Registry` (thư mục mỗi phiên bản bất biến, `manifest.json`, con trỏ `current.json` thay nguyên tử, `history.jsonl` chỉ ghi thêm, khóa tệp `fcntl.flock`, chỉ có trên Unix; con trỏ ghi qua file tạm có `fsync` rồi `os.replace`; con trỏ lệch lịch sử thì promote và rollback từ chối); `promote` (kiểm sha256, danh sách đặc trưng, phiên bản thư viện, cổng holdout, `force=True` kèm lý do), `rollback` (về bản đứng dưới trong ngăn xếp promote, không huấn luyện lại). Kiểm soát ai được ghi vào thư mục registry là việc của hạ tầng, không nằm trong lab.
- `routing.py`: `assign_arm` (sha256 «tên thí nghiệm:mã khách» chia lấy dư 10.000), `ShadowRouter` (quyết định chỉ theo champion; mô hình thử được chấm và ghi log, lỗi của nó bị bắt).
- `drift.py`: `psi` hai phía (khoảng đóng bên phải, biên là phân vị cộng một biên ngay dưới mỗi phân vị; cùng logic với `model.psi` của lab 57), `RetrainPolicy` và `retrain_decision` (tín hiệu thành một hành động: none, monitor, investigate, retrain, rollback). Ngưỡng của chính sách là giả định của lab; tác động của đặc trưng đo bằng mean|SHAP| (không dùng gain).
- `train.py`: ba phiên bản v1 (đến ngày 30, nhãn cuối), v2 (đến ngày 40, nhãn cuối), v3 (đến ngày 50, nhãn biết tại ngày 50, không chờ nhãn chín). `costs.py`: chi phí từng chuyến, phí duyệt tính cho mọi cờ.
- `main.py`: chín bước và `--verify`. Ghi `out/registry/`, `out/shadow_log.jsonl` (gitignore). `mlflow_demo.py`: cùng ý tưởng bằng MLflow, tùy chọn.
- `tests/`: pytest (registry, PSI, chia nhóm A/B, chính sách).
- `synthetic_fraud.py`, `dataset.py`, `metrics.py` copy NGUYÊN từ lab 54; `features.py` từ lab 55; `model.py`, `model_io.py`, `prepare.py` từ lab 57.

## Hạ tầng cần

Không cần Docker. Trên macOS, LightGBM cần OpenMP: `brew install libomp`. `mlflow_demo.py` cần thêm nhóm phụ thuộc `mlflow` (xem dưới); kiểm tra ngày 07/11/2026: `mlflow` 3.18.0 cài và chạy được trên Python 3.14.7 (arm64 macOS), nên `requires-python = ">=3.14"`.

## Chạy

Cần uv và Python 3.14; lần đầu cần mạng để tải lightgbm, scikit-learn, polars. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/60-model-registry-ab-drift examples/toi-noi-python/60-model-registry-ab-drift/main.py
uv run --project examples/toi-noi-python/60-model-registry-ab-drift examples/toi-noi-python/60-model-registry-ab-drift/main.py --verify
uv run --project examples/toi-noi-python/60-model-registry-ab-drift pytest examples/toi-noi-python/60-model-registry-ab-drift -q
uv run --group mlflow --project examples/toi-noi-python/60-model-registry-ab-drift examples/toi-noi-python/60-model-registry-ab-drift/mlflow_demo.py --verify
```

`mlflow_demo.py` đọc `out/registry/` nên chạy sau `main.py`.

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `Kiểm chứng: OK n/n` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/n` với k là số ca đạt. Nó kiểm bất biến, biên và quan hệ lớn (v3 gắn cờ ít hơn 0,75 lần v1 ở cả ba seed, PSI của bộ đếm lịch sử khách vượt 0,25), không ghim số tuyệt đối hay tốc độ.
- Lần đầu lab dựng đặc trưng từ event cho seed 42 (lưu `out/table.parquet`), sinh `out/customers.parquet`, rồi sinh lại dữ liệu cho seed 7 và 123 (`out/table-seed*.parquet`). Thời gian đo được ghi trong bài, kèm load average.
- Số trong bài phụ thuộc phiên bản numpy/polars/scikit-learn/lightgbm. Mọi dữ liệu là tổng hợp, kèm hai tạo tác của bộ sinh (tỉ lệ NEWUSER50 của chuyến sạch giảm dần, khách gian lận không dùng mã nào ngoài NEWUSER50) nên số đo không đại diện cho dữ liệu thật; hai kịch bản (nguồn GPS đổi, promote nhầm) do lab đặt vào.
- CẢNH BÁO: joblib dùng pickle; chỉ nạp file do chính pipeline của mình tạo.

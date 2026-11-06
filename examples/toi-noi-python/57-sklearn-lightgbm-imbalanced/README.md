# Lab 57 · scikit-learn và LightGBM trên dữ liệu gian lận mất cân bằng

Đi kèm bài `python-scikit-learn-lightgbm-imbalanced-cross-validation`. Lab huấn luyện bộ chấm điểm gian lận trên 30 đặc trưng của bài 55: Pipeline với `ColumnTransformer`, cross-validation theo thời gian so với ngẫu nhiên, trọng số lớp (`scale_pos_weight`), calibration, chọn ngưỡng theo chi phí, rồi lưu mô hình bằng `joblib` kèm metadata để các bài 59 đến 61 nạp lại.

- `model.py`: `make_pipeline` (chọn đặc trưng theo tên), `make_lgbm` (mặc định 100 vòng đã chọn bằng CV), `fit_calibrated` (gốc ngày < 22 + hiệu chỉnh ngày 22 đến 30), `time_folds`, `total_cost` và `best_cost_threshold` (chi phí = 8.000 x chuyến gắn cờ + (tiền + 20.000) x gian lận bỏ sót), `expected_cost_flag` (gắn cờ khi p x (tiền + 20.000) > 8.000), `reference_distribution` và `psi` cho bài 60.
- `model_io.py`: `save_model`, `load_model` (mọi lỗi nạp là `ModelLoadError`; kiểm sha256, meta, danh sách đặc trưng, scikit-learn/lightgbm cùng minor; lệch patch chỉ cảnh báo), `ModelInputError` cho None/NaN/thiếu khóa, `ScoringModel.predict_proba(df)`, `score_one(dict)`, `threshold`, `flag(p, amount)`, `booster`, `preprocess`, `contributions(df)` (đóng góp theo điểm THÔ trước hiệu chỉnh, cho SHAP). API cho bài 59, 60, 61.
- `main.py`: tám bước và `--verify`. Ghi `out/model.joblib` và `out/model_meta.json` (gitignore).
- `tests/test_model_io.py`: pytest, 25 ca (thêm ca PSI cho cờ hiếm) (lưu/nạp, bit lật, khác danh sách đặc trưng, khác phiên bản thư viện, `time_folds`, `expected_cost_flag`).
- `synthetic_fraud.py`, `dataset.py`, `metrics.py` copy NGUYÊN từ lab 54; `features.py` copy NGUYÊN từ lab 55; `prepare.py` dựng bảng 30 đặc trưng + nhãn, lưu `out/table.parquet`; seed khác lưu `out/table-seed<N>.parquet`.

## Hạ tầng cần

Không cần Docker. Trên macOS, LightGBM cần OpenMP: `brew install libomp` (không có thì `import lightgbm` lỗi `Library not loaded: @rpath/libomp.dylib`). Trên Linux chưa kiểm. Kiểm tra ngày 06/11/2026: `lightgbm` 4.7.0 có wheel cho Python 3.14.7 (arm64 macOS) và chạy bình thường, nên lab dùng `requires-python = ">=3.14"`.

## Chạy

Cần uv và Python 3.14; lần đầu cần mạng để tải lightgbm, scikit-learn, polars. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced examples/toi-noi-python/57-sklearn-lightgbm-imbalanced/main.py
uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced examples/toi-noi-python/57-sklearn-lightgbm-imbalanced/main.py --verify
uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced pytest examples/toi-noi-python/57-sklearn-lightgbm-imbalanced -q
```

Nạp mô hình đã lưu từ code khác (đặt `model_io.py`, `features.py` cạnh code gọi):

```python
from pathlib import Path
from model_io import load_model

scoring = load_model(Path("out"))        # kiểm sha256, danh sách đặc trưng, phiên bản thư viện
p = scoring.predict_proba(df)            # df là polars.DataFrame có đủ 30 cột FEATURES (cột thừa bị bỏ)
p1 = scoring.score_one(features_dict)    # dict đủ 30 đặc trưng
```

CẢNH BÁO: `joblib` dùng pickle; chỉ nạp file do chính pipeline của mình tạo.

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 45/45` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/45` với k là số ca đạt. `pytest` báo `25 passed`.
- Lab chạy khoảng hai đến ba phút khi máy bận (CV theo thời gian và ngẫu nhiên là phần lâu nhất), nhanh hơn nhiều khi máy rảnh. `--verify` chỉ kiểm bất biến và thứ bậc lớn, không ghim số tuyệt đối hay tốc độ.
- Số trong bài (PR-AUC, chi phí) phụ thuộc phiên bản numpy/polars/scikit-learn/lightgbm. Mọi dữ liệu là tổng hợp; số đo không đại diện cho dữ liệu thật. Điểm gian lận là đầu vào cho người duyệt, không tự khóa tài khoản.

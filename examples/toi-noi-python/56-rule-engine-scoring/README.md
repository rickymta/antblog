# Lab 56 · Rule engine và điểm rủi ro: Strategy, cấu hình YAML, giải thích được

Đi kèm bài `python-rule-engine-scoring-strategy-explainability`. Lab dựng một rule engine nhỏ cho fraud-scoring của Tới Nơi: mỗi rule là một hàm `(features) -> RuleHit | None`, cấu hình bằng YAML, tổng điểm kèm lý do; rồi đo precision/recall của từng rule và của cả engine trên bộ dữ liệu gian lận tổng hợp (bài 54), và so với một mô hình học.

- `rules.py`: `RuleHit`, `Verdict`, ba aggregator (`sum_capped`, `max_points`, `noisy_or`) làm Strategy, `parse_config` kiểm cấu hình khi nạp (gom các lỗi tìm được vào `ConfigError`, mọi cấu hình hỏng đều ra `ConfigError`; đọc bằng lớp con của `SafeLoader`, từ chối khóa trùng và thẻ Python), `build_rule`, `RuleEngine`; cuối file là đường theo lô bằng polars (`rule_flags`, `score_frame`) cùng định nghĩa để đo trên hàng trăm nghìn chuyến.
- `rules.yaml`: 11 rule, `review_at: 40`, `watch_at: 15`, `aggregate: noisy_or`. Điểm mỗi rule đặt theo precision đo trên train (làm tròn xuống bội 5).
- `tests/test_rules.py`: pytest, 83 ca (biên của từng điều kiện, bảng ca vàng viết tay ghim điểm, ngưỡng, dải, khóa thiếu ở điều kiện thứ hai, NaN, aggregator, cấu hình hỏng).
- `main.py`: tám bước và `--verify`.
- `synthetic_fraud.py`, `dataset.py`, `metrics.py` copy NGUYÊN từ lab 54; `features.py` copy NGUYÊN từ lab 55; `prepare.py` dựng bảng 30 đặc trưng + nhãn, lưu `out/table.parquet` (gitignore).

## Chạy

Không cần Docker. Cần uv và Python 3.14; lần đầu cần mạng để tải numpy, polars, scikit-learn, pyyaml và dựng đặc trưng từ event lần đầu. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/56-rule-engine-scoring examples/toi-noi-python/56-rule-engine-scoring/main.py
uv run --project examples/toi-noi-python/56-rule-engine-scoring examples/toi-noi-python/56-rule-engine-scoring/main.py --verify
uv run --project examples/toi-noi-python/56-rule-engine-scoring pytest examples/toi-noi-python/56-rule-engine-scoring -q
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 50/50` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/50` với k là số ca đạt. `pytest` báo `83 passed`.
- Bước huấn luyện mô hình so sánh (HistGradientBoosting, tối đa 150 vòng, dừng sớm ở 84) là phần lâu nhất; thời gian chưa đo trên máy rảnh (lần chạy ghi trong bài: 293 giây khi load average 363).
- `--verify` chỉ kiểm bất biến và biên (không ghim số phụ thuộc seed); số trong bài là của seed 42; đổi `SEED` trong `prepare.py` thành 1 hoặc 2 cũng cho `OK 50/50`. Có ca sai thì exit 1 cả khi không có `--verify`. Số trong bài phụ thuộc phiên bản numpy/polars/scikit-learn (bộ sinh số của numpy không cam kết giữ dòng số giữa phiên bản).
- Mọi dữ liệu là tổng hợp; ngưỡng và điểm trong `rules.yaml` khớp với bộ sinh, không phải số vận hành. Điểm rủi ro là đầu vào cho người duyệt, không tự khóa tài khoản.

## Ghi chú về cấu hình YAML (chuyển từ bài viết)

- File đọc bằng lớp con của `yaml.SafeLoader` (như `yaml.safe_load`): thẻ `!!python/object/apply:...` bị từ chối. Trong PyYAML 6, `yaml.load` thiếu `Loader` ném `TypeError`, `FullLoader` cũng từ chối thẻ đó; chỉ `Loader` và `UnsafeLoader` chạy mã.
- `safe_load` âm thầm lấy khóa cuối khi mapping có khóa trùng; lớp con trong `rules.py` từ chối (`version` hai lần ra `ConstructorError: khóa trùng 'version'`).
- `reason` là chuỗi `str.format_map`, nên `{speed_kmh.real}` truy cập thuộc tính, `{x[0]}` truy cập phần tử và `{x:{y}}` lồng một trường vào định dạng. Parser chỉ nhận tên đặc trưng trần, không lồng, và chỉ đặc trưng có trong `when` của chính rule (để `None` không làm `:.0f` nổ giữa luồng chấm điểm); định dạng được thử điền thật khi nạp.
- `build_rule` lấy MỌI giá trị đặc trưng trước khi so: với `all(c.holds(features[c.feature]) ...)`, điều kiện đầu sai làm `all()` dừng và khóa thứ hai bị đổi tên không bao giờ bị phát hiện. `None` và `NaN` đều là «không biết»: rule im lặng (đường polars dùng `fill_null(False)` và loại NaN để khớp).
- `points` đặt bằng precision đo trên train (nhãn cuối cùng), làm tròn xuống bội 5, trần ở 95 (precision 1,000 vẫn chỉ 95), và `--verify` đòi không vượt 100 lần precision quá 10 điểm.

# Lab 61 · SHAP, nhật ký quyết định và hàng đợi người duyệt

Đi kèm bài `python-shap-explainability-audit-human-review`. Lab giải thích từng điểm gian lận của mô hình bài 57 bằng SHAP (`reasons[]`), đưa chuyến bị gắn cờ vào hàng đợi người duyệt trên PostgreSQL (schema `lab61`), ghi nhật ký quyết định chỉ-ghi-thêm, và đo công suất, SLA, dải điểm còn đáng duyệt. Điểm gian lận là đầu vào cho người duyệt; chỉ người duyệt RV-001 mới chốt `confirmed_fraud` hoặc `cleared` (không dùng approved/rejected để khỏi lẫn với trạng thái KYC ở bài 70). RV-001 trong lab là mô phỏng (chốt theo nhãn cuối của bộ sinh, luôn đúng).

- `explain.py`: `Explainer` (gói `shap.TreeExplainer` quanh LightGBM gốc; đối chiếu với TreeSHAP của chính LightGBM qua `ScoringModel.contributions`), câu lý do cho 30 đặc trưng, họ tín hiệu. SHAP giải thích điểm thô chưa hiệu chỉnh, không phải p.
- `review.py`: DDL, ràng buộc và trigger ép quy tắc ở cơ sở dữ liệu (superuser vẫn `DISABLE TRIGGER` được: production cần vai trò riêng và bản sao log ngoài tầm với); `ReviewStore.enqueue` (idempotent), `decide`. `pii.py`: che số giấy tờ và số điện thoại (`099*******89`, chuẩn hóa dấu phân cách trước khi quét). `schedule.py`: mô phỏng công suất người duyệt 08:00-17:00.
- `train.py`: huấn luyện lại mô hình bài 57 nếu `out/` chưa có. `rules.py`, `rules.yaml`: copy từ lab 56 (rule engine, phiên bản cấu hình ghi vào nhật ký).
- `main.py`: bảy bước và `--verify`. Hàng đợi dùng quy tắc chi phí kỳ vọng `p × (tiền + 20.000) > 8.000`; ngưỡng p chung chỉ để so sánh. Schema `lab61` được tạo lúc đầu và xóa khi xong, kể cả khi lỗi. `tests/`: pytest cho phần không cần PostgreSQL.
- `synthetic_fraud.py`, `dataset.py`, `metrics.py` copy NGUYÊN từ lab 54; `features.py` từ lab 55; `model.py`, `model_io.py`, `prepare.py` từ lab 57.

## Hạ tầng cần

PostgreSQL của hạ tầng chung (`TOINOI_PG_URL`, mặc định `postgresql://toinoi:toinoi@localhost:54329/toinoi`):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

Không nối được thì `main.py` in `FAIL: không nối được PostgreSQL tại <url> (docker compose ...)` và exit 1. Trên macOS, LightGBM cần OpenMP: `brew install libomp`. Kiểm tra ngày 07/11/2026: `shap` 0.54.1 cài và chạy được trên Python 3.14.7 (arm64 macOS), nên `requires-python = ">=3.14"`.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/61-shap-audit-human-review examples/toi-noi-python/61-shap-audit-human-review/main.py
uv run --project examples/toi-noi-python/61-shap-audit-human-review examples/toi-noi-python/61-shap-audit-human-review/main.py --verify
uv run --project examples/toi-noi-python/61-shap-audit-human-review pytest examples/toi-noi-python/61-shap-audit-human-review -q
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `Kiểm chứng: OK n/n` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/n` với k là số ca đạt. Nó kiểm bất biến và quan hệ lớn (SHAP cộng ra điểm thô, các ràng buộc cơ sở dữ liệu (cả INSERT lẫn log phải khớp hàng đợi), che số giấy tờ, khoảng tin cậy ở ba seed), không ghim số tuyệt đối hay tốc độ.
- Lần đầu lab dựng đặc trưng (seed 42) rồi sinh lại dữ liệu cho seed 7 và 123; thời gian ghi trong bài kèm load average.
- Mọi dữ liệu là tổng hợp, kèm hai tạo tác của bộ sinh (xem bài); số đo không đại diện cho dữ liệu thật. Không lưu số giấy tờ rõ trong log; số tiền, mã chuyến và đặc trưng số là dữ liệu duy nhất lab ghi.

# Lab 67 - Clean Architecture: domain, application, infrastructure, api

Bài viết: `api/seed/content/python-clean-architecture-domain-application-infrastructure.md`.

Service hồ sơ KYC tài xế của Tới Nơi chia bốn lớp trong `src/kyc_service/`:

- `domain/`: `KycApplication` (aggregate, một hồ sơ gồm đúng hai giấy tờ CCCD và GPLX), `KycStatus` và bảng chuyển của bài 41, value object `OcrResult`, `FaceMatch`, `RiskScore`. Không import gì ngoài thư viện chuẩn.
- `application/`: port (`OcrPort`, `FaceMatchPort`, `RiskScorePort`, `KycRepository`, `Clock`) là `Protocol`; use case `SubmitDriverApplicationUseCase`, `ReviewApplicationUseCase`, `GetApplicationUseCase`; DTO `ApplicationView`.
- `infrastructure/`: adapter giả (OCR, khuôn mặt, điểm rủi ro) và kho lưu trong bộ nhớ. Adapter thật nằm ở P05-P07; kho PostgreSQL ở bài 69.
- `api/`: FastAPI, Problem Details và composition root `container.py`.

Kết quả OCR, điểm khuôn mặt, điểm rủi ro chỉ là đầu vào cho người duyệt: mọi hồ sơ đi hết đường đều dừng ở `needs_review`; `approved`/`rejected` chỉ từ `needs_review` do người duyệt (`RV-001`) quyết định. Dữ liệu là tổng hợp (CCCD giả bắt đầu `099`, GPLX giả bắt đầu `99`).

Kiến trúc được `import-linter` giữ: hai hợp đồng trong `[tool.importlinter]` của `pyproject.toml` (chiều phụ thuộc giữa bốn lớp, `exhaustive = true` nên module mới ngoài bốn lớp làm hợp đồng vỡ; lõi không import `fastapi`, `starlette`, `pydantic`). `build_container()` phát `UserWarning` khi dùng adapter giả. `face_consent` chỉ nhận bool thật; điểm khuôn mặt là cosine -1..1, điểm rủi ro 0..1; `auto_approve` của bài 41 đã gỡ khỏi domain.

## Chạy

Từ thư mục gốc repository (không cần Docker, không cần mạng sau lần resolve đầu):

```bash
uv run --project examples/toi-noi-python/67-clean-architecture-layers examples/toi-noi-python/67-clean-architecture-layers/main.py
uv run --project examples/toi-noi-python/67-clean-architecture-layers examples/toi-noi-python/67-clean-architecture-layers/main.py --verify
uv run --project examples/toi-noi-python/67-clean-architecture-layers pytest examples/toi-noi-python/67-clean-architecture-layers -q
uv run --project examples/toi-noi-python/67-clean-architecture-layers lint-imports --config examples/toi-noi-python/67-clean-architecture-layers/pyproject.toml
sh examples/toi-noi-python/67-clean-architecture-layers/verify.sh
```

`--verify` in `OK n/n` (kỳ vọng `OK 23/23`) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. `verify.sh` chạy ruff, mypy strict, mypy phải bắt adapter sai chữ ký (`demo-errors/bad_adapter.py`), import-linter, pytest (97 test, gồm cả test cố ý phá kiến trúc trên bản sao trong thư mục tạm) và `--verify`, kỳ vọng `OK 7/7`.

## Giới hạn

Lab chưa có transaction: lỗi bất ngờ giữa chừng để lại hồ sơ `submitted` (có test ghi nhận); bài 68 thêm transaction. Adapter giả tra bảng cố định, không phải mô hình. Kho trong bộ nhớ không nhiều worker, không bền.

# Lab 69 - Repository và Unit of Work trên SQLAlchemy

Bài viết: `api/seed/content/python-repository-unit-of-work-sqlalchemy.md`.

Cùng bốn lớp, domain và pipeline mediator với lab 68 (tự chứa, không import từ lab khác). Khác ở chỗ lưu trữ: kho trong bộ nhớ và `TransactionManager` được thay bằng PostgreSQL thật.

- `application/ports.py`: `UnitOfWork` thay `TransactionManager`; `KycRepository` giữ nguyên các thao tác `next_id/add/get/save/list_by_status`.
- `infrastructure/sql/tables.py`: aggregate = một hàng gốc `kyc_applications` + hàng con `kyc_documents`, `kyc_ocr_results`; cột `version` cho optimistic concurrency; sequence sinh mã `KYC-0001`; ràng buộc `ck_decision_has_reviewer`.
- `infrastructure/sql/repository.py`: ánh xạ hàng <-> aggregate thuần của domain; giữ tham chiếu mạnh tới hàng đã nạp (identity map của Session chỉ giữ tham chiếu yếu).
- `domain/kyc.py`: `OcrResult` che số giấy tờ ngay khi được tạo (`mask_id_number`) và mang `model_version`, `image_sha256` làm khóa nối sang document OCR bài 47; `FaceMatch.score` là cosine -1..1; mã hồ sơ `KYC-[0-9]{4,}`.
- `application/behaviors.py`: `validation_behavior` chép từ lab 68 (handler nhận bản đã validate; `face_consent` là `StrictBool`).
- `api/container.py`: `build_container()` phát `FakeAdapterWarning` (một `UserWarning`) nêu tên port dùng adapter giả.
- `infrastructure/sql/unit_of_work.py`: `begin()` mở một `Session` + `session.begin()`, đặt vào `ContextVar` (phạm vi Context: mỗi request có Session riêng; task tạo trong `begin()` dùng chung; `Thread` trần không thấy).
- `infrastructure/sql/read_model.py`: phía đọc SELECT thẳng ra view bằng Session ngắn riêng.
- `application/behaviors.py`: `transaction_behavior(uow)` gọi `uow.begin()`.

Schema `lab69` được tạo mới khi chạy và bị xóa khi xong (trừ `--keep`).

## Chạy

Cần hạ tầng dùng chung (PostgreSQL); từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/69-repository-unit-of-work examples/toi-noi-python/69-repository-unit-of-work/main.py
uv run --project examples/toi-noi-python/69-repository-unit-of-work examples/toi-noi-python/69-repository-unit-of-work/main.py --verify
uv run --project examples/toi-noi-python/69-repository-unit-of-work pytest examples/toi-noi-python/69-repository-unit-of-work -q
uv run --project examples/toi-noi-python/69-repository-unit-of-work lint-imports --config examples/toi-noi-python/69-repository-unit-of-work/pyproject.toml
sh examples/toi-noi-python/69-repository-unit-of-work/verify.sh
```

`--verify` in `OK n/n` (kỳ vọng `OK 33/33`) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. Khi thiếu PostgreSQL: `FAIL: không nối được PostgreSQL tại <url> (docker compose ...)`, exit 1. `verify.sh` kỳ vọng `OK 6/6` (ruff, mypy strict, import-linter 4 hợp đồng, pytest 136 test, `--verify`). `--cleanup` chỉ xóa schema `lab69`.

## Giới hạn

Không có migration (lab dùng `create_all`; production dùng Alembic, bài 24). Lab gọi service ngoài (adapter giả) bên trong transaction của command như bài 67 và 68; bài 70 đo hệ quả. Không có outbox: sự kiện miền chưa tồn tại ở bài này.

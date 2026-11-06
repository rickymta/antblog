# Lab 68 - CQRS và mediator: command/query handler, pipeline

Bài viết: `api/seed/content/python-cqrs-command-query-handler-mediator.md`.

Cùng cấu trúc bốn lớp và domain với lab 67 (tự chứa, không import từ lab khác). Khác ở `application/` và phần ghép:

- `messages.py`: `Message[R]`, `Command[R]`, `Query[R]` (generic PEP 695).
- `commands.py`: `SubmitDriverApplication`, `RecordReviewDecision`. `queries.py`: `GetApplication`, `ListReviewQueue`. Kiểu chú thích của dataclass (pydantic `StringConstraints`) mô tả hình dạng hợp lệ.
- `handlers.py`: mỗi message đúng một handler; thân giữ logic use case của bài 67.
- `mediator.py`: `Mediator.register(kiểu, handler)` và `send(message)`, dispatch theo `type(message)`; pipeline dựng một lần lúc đăng ký.
- `behaviors.py`: `logging_behavior`, `validation_behavior` (pydantic `TypeAdapter` dựng một lần mỗi kiểu; handler nhận bản ĐÃ validate; `face_consent` là `StrictBool`), `transaction_behavior` (chỉ command). Thứ tự ghép trong `api/container.py`: log, validate, transaction, handler.
- `ports.py` thêm `TransactionManager` (bài 69 thay bằng Unit of Work) và `ApplicationReadModel` (phía đọc).

Kho, transaction và read model đều trong bộ nhớ. Rollback hoàn lại các hàng nhưng không hoàn lại bộ đếm mã, như sequence của PostgreSQL.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline examples/toi-noi-python/68-cqrs-mediator-pipeline/main.py
uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline examples/toi-noi-python/68-cqrs-mediator-pipeline/main.py --verify
uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline pytest examples/toi-noi-python/68-cqrs-mediator-pipeline -q
uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline lint-imports --config examples/toi-noi-python/68-cqrs-mediator-pipeline/pyproject.toml
sh examples/toi-noi-python/68-cqrs-mediator-pipeline/verify.sh
```

`--verify` in `OK n/n` (kỳ vọng `OK 31/31`) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. `verify.sh` kỳ vọng `OK 6/6` (ruff, mypy strict, import-linter 3 hợp đồng, pytest 111 test, `--verify`).

## Giới hạn

Mediator giảng giải: không notification, không streaming, không DI container, không chẩn đoán đăng ký. Một bản chạy nhiều worker cần kho và transaction thật (bài 69).

# Lab 70 - Use case nộp hồ sơ tài xế end-to-end

Bài viết: `api/seed/content/python-use-case-ho-so-tai-xe-end-to-end.md`.

Cùng bốn lớp, mediator, repository và Unit of Work PostgreSQL của lab 69 (tự chứa). Khác ở use case và domain:

- `domain/policy.py`: `ReviewPolicy` (ngưỡng OCR 0,80; khuôn mặt 0,75; rủi ro từ 0,30) chỉ chọn LÝ DO vào hàng chờ (`domain/review.py`: `ReviewReason`). Ngưỡng là ví dụ, không đo từ dữ liệu của lab.
- `domain/events.py` và `KycApplication.send_to_review(reasons, now=...)`: hồ sơ chấm điểm đủ phát domain event `KycApplicationScored`; `pull_events()` lấy và xóa.
- `application/handlers.py`: lớp `Screening` chạy chuỗi OCR, khuôn mặt, điểm rủi ro, hàng chờ theo trạng thái hiện tại, dùng chung cho `SubmitDriverApplication` và `ResumeScreening` (làm tiếp khi service ngoài sống lại). Máy luôn dừng ở `needs_review`; `approved` và `rejected` chỉ đến từ `RecordReviewDecision` của người duyệt (`RV-001`).
- `application/ports.py`: thêm `EventPublisher`. Adapter của lab chỉ ghi lại trong bộ nhớ: sự kiện giao trong transaction có thể thành «sự kiện ma» khi COMMIT lỗi (có ca verify); bài 71 sửa bằng outbox.
- `Screening._accept` dịch `DomainError` của kết quả service (ví dụ OCR trả hai CCCD) thành `InvalidServiceResponse`: hồ sơ dừng ở bước đó thay vì rollback và 422. `pending_check` suy từ trạng thái nên `GET` cũng thấy.
- Ba service OCR, khuôn mặt, rủi ro là adapter giả (`infrastructure/fake_services.py`, 5 tài xế tổng hợp `DR-001..DR-005`). Lab không đọc ảnh nào; adapter thật nằm ở bài 41 đến 61.

Schema `lab70` tạo mới khi chạy và xóa khi xong (trừ `--keep`).

## Chạy

Cần hạ tầng dùng chung (PostgreSQL); từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/main.py
uv run --project examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/main.py --verify
uv run --project examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end pytest examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end -q
sh examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/verify.sh
```

`--verify` in `OK n/n` (kỳ vọng `OK 39/39`) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. Khi thiếu PostgreSQL: `FAIL: không nối được PostgreSQL tại <url> (docker compose ...)`, exit 1. `verify.sh` kỳ vọng `OK 6/6` (ruff, mypy strict, import-linter 4 hợp đồng, pytest 185 test, `--verify`).

## Giới hạn

Use case gọi ba service bên trong transaction của command (đo bằng `pg_stat_activity` trong demo): với service thật phải tách bước gọi khỏi transaction. Ngưỡng là ví dụ. `ResumeScreening` cũng chạy cả chuỗi trong một transaction, chỉ idempotent một phần (service tắt: version tăng và service bị gọi mỗi lần; hai lần song song: gọi service hai lần, một lần ghi). Lệnh nộp chưa có khóa idempotency: nộp hai lần cho ra hai hồ sơ. Hồ sơ dừng giữa chừng chỉ được làm tiếp khi có ai gọi `ResumeScreening`; lab không có worker tự động làm việc đó.

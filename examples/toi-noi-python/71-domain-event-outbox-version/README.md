# Lab 71 - Domain event, outbox và version hợp đồng sự kiện

Bài viết: `api/seed/content/python-domain-event-outbox-integration-event-version.md`.

Cùng bốn lớp, use case và domain event `KycApplicationScored` của lab 70 (tự chứa). Khác ở đường đi của sự kiện:

- `application/integration_events.py`: hợp đồng gửi ra ngoài, tách khỏi domain event. Envelope `{event_id, event_type, occurred_at, version, payload}`, tên trường `snake_case`; version 1 gồm `application_id, driver_id, risk_score, review_reasons`; version 2 thêm `face_score`, `ocr_min_confidence` (tùy chọn).
- `infrastructure/sql/outbox.py`: `OutboxEventPublisher` ghi dòng `lab71.outbox` bằng Session của Unit of Work đang mở (cùng transaction với hồ sơ).
- `infrastructure/kafka/dispatcher.py`: `OutboxDispatcher` thuê dòng (`FOR UPDATE SKIP LOCKED` + `lease_until`), phát ra Kafka ngoài transaction, rồi đánh dấu `published_at`.
- `consumers/scored_consumer.py`: consumer Python tham chiếu (bản cũ chỉ hiểu version 1, bản mới hiểu 1 và 2), cố ý nằm ngoài package `kyc_service`.
- `csharp-consumer/`: consumer C# (`System.Text.Json`, `JsonNamingPolicy.SnakeCaseLower`, `RespectRequiredConstructorParameters`, `RespectNullableAnnotations`) đọc các thông điệp do lab ghi ra `out/messages.jsonl`; thiếu trường, sai `event_type`, `payload` null bị từ chối (`invalid_payload`, `wrong_event_type`).
- `build_container(emit_version=...)`: phiên bản hợp đồng producer phát là cấu hình bắt buộc, không có mặc định.
- `tests/golden/scored.v1.json`, `scored.v2.json`: golden file của hợp đồng.

Schema `lab71` và các topic `lab71.*` được tạo khi chạy và xóa khi xong (trừ `--keep`; `--cleanup` chỉ dọn).

## Chạy

Cần hạ tầng dùng chung (PostgreSQL và Kafka) và .NET SDK 10 cho phần consumer C#; từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/71-domain-event-outbox-version examples/toi-noi-python/71-domain-event-outbox-version/main.py
uv run --project examples/toi-noi-python/71-domain-event-outbox-version examples/toi-noi-python/71-domain-event-outbox-version/main.py --verify
uv run --project examples/toi-noi-python/71-domain-event-outbox-version pytest examples/toi-noi-python/71-domain-event-outbox-version -q
sh examples/toi-noi-python/71-domain-event-outbox-version/verify.sh
```

`--verify` in `OK n/n` (kỳ vọng `OK 21/21`, trong đó 3 ca C#) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. Không có `dotnet` thì `--verify` in lý do và exit 1; thêm `--no-csharp` để bỏ ba ca C# (khi đó `OK 18/18`). Khi thiếu PostgreSQL hoặc Kafka: `FAIL: không nối được <dịch vụ> tại <url> (docker compose ...)`, exit 1. `verify.sh` kỳ vọng `OK 6/6` (ruff, mypy strict, import-linter 5 hợp đồng, pytest 221 test gồm các test C#, `--verify`). Bỏ test C# khi không có .NET: `pytest -m "not csharp"`.

Lần chạy đầu `dotnet run` build project C# (không cần gói NuGet ngoài).

## Giới hạn

Một dispatcher mỗi lần chạy `dispatch_once` (không có vòng lặp nền, không backoff, không dead-letter cho dòng hỏng). Không có inbox: lab chỉ chứng minh thông điệp trùng sinh ra thế nào (xem bài 39 cho phía nhận). Không có Schema Registry; hợp đồng giữ bằng golden file và hai consumer mẫu.

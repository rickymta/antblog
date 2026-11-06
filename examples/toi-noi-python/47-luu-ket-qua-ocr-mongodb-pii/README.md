# Lab 47 · Lưu kết quả OCR vào MongoDB: mặt nạ PII, hash có khóa, TTL, audit, migration

Đi kèm bài `python-luu-ket-qua-ocr-mongodb-pii-retention`, nối tiếp lab 26 (bộ trường, `schema_version`). Kết quả OCR có hình dạng như job của bài 46 (`fixtures/job_result_cccd.json` là kết quả thật của lab 46 trên thẻ giả seed 7, có `image_sha256`) được lưu vào ba collection `lab47_*` với ba vòng đời khác nhau. Dãy phiên bản nối tiếp bài 26: v2 còn số rõ, v3 là dạng đã che.

- `app/pii.py`: `mask_digits` (`099*******83`: 3 số đầu, 2 số cuối), `pii_hash` (HMAC-SHA-256 theo loại giấy tờ), `load_pii_key` (khóa bí mật `TOINOI_PII_HASH_KEY` >= 32 ký tự, không có giá trị mặc định; đây là khóa HMAC, không phải salt).
- `app/documents.py`: `$jsonSchema` v2/v3 (v3 đóng kín), `reset_and_setup` (DROP rồi dựng lại collection, index, TTL), `save_ocr_result` (khóa `kyc_id, document_type, model_version, image_sha256`; raw trước, document sau; thiếu số hoặc ngày sai thì lưu `needs_review`), `normalize_to_v3` (upgrade on read), `migrate_v2_to_v3`.
- `app/audit.py`, `app/access.py`: `MongoAuditSink`, `view_document` và `lookup_by_number` (ghi audit trước khi trả dữ liệu; reveal cần scope và lý do; tra theo số cũng ghi audit).
- `app/retention.py`: tạm đặt `ttlMonitorSleepSecs` về 1 giây để quan sát TTL; chờ document biến mất.
- `main.py`: demo và `--verify`. `synthetic_card.py`: thẻ giả, copy từ lab 30 (chỉ dùng `make_card_fields`).

Khóa băm trong `main.py` là **giá trị của lab**, công khai trong mã nguồn. Production lấy khóa từ secret manager, đặt tách khỏi cơ sở dữ liệu. Cần MongoDB của hạ tầng dùng chung, từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii/main.py
uv run --project examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii/main.py --verify
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 53/53` (exit 0). Không nối được MongoDB thì in `FAIL: không nối được MongoDB tại <url> ...` và exit 1.
- **Lab đụng vào hai thứ, không chỉ ba collection:** (1) `lab47_ocr_documents`, `lab47_ocr_raw`, `lab47_audit_log`, bị DROP khi bắt đầu và khi xong; (2) tham số `ttlMonitorSleepSecs` của **cả mongod** (mặc định 60), tạm đặt về 1 giây ở phần TTL, nên mọi lab khác dùng chung container cũng bị quét nhanh trong lúc đó. Tham số này không có trong trang tham số chính thức của MongoDB; lab dùng vì `setParameter` đổi được lúc chạy. Lab trả lại bằng `finally` và bắt SIGTERM; nếu bị SIGKILL thì lần chạy sau thấy giá trị không lớn hơn 1, in cảnh báo và trả về 60. Nếu không có quyền đổi, lab chờ tới 130 giây cho mỗi lần quan sát TTL.
- Đã chạy với Python 3.14.7, pymongo 4.19.0, pydantic-settings 2.16.2, MongoDB 8.3.13, macOS.

# Lab 26 - MongoDB với PyMongo async: lưu kết quả OCR dạng document

Bài viết: `api/seed/content/python-mongodb-pymongo-motor-document-ocr.md`.

- `main.py`: dùng `pymongo.AsyncMongoClient` (API async chính thức của PyMongo, không cần Motor) với collection `lab26_ocr_documents`: `insert_one`/`insert_many`, `find` với projection/sort/limit, unique index và `$jsonSchema` validator, `create_index` + `explain()` (COLLSCAN so với IXSCAN; truy vấn «bản mới nhất» có SORT cho tới khi thêm index ghép), `datetime` UTC aware và độ chính xác mili giây, `schema_version` (upgrade on read, migrate bằng `update_many` với pipeline, đặt `field_confidence` null để mọi document v2 có đủ khóa).
- Dữ liệu tổng hợp, seed cố định: 1.000 hồ sơ KYC x (CCCD giả `099…` với bộ trường chung `full_name`, `id_number` (099 + giới tính/thế kỷ + 2 số năm + 6 số ngẫu nhiên), `date_of_birth` (lưu ISO), `sex`, `hometown_city` + GPLX giả `99…`) + 5 document schema v1. Kết quả OCR là đầu vào cho người duyệt, không phải quyết định xác minh.

## Hạ tầng cần

MongoDB trong hạ tầng dùng chung (`mongodb://localhost:27019/toinoi`, ghi đè bằng `TOINOI_MONGO_URL`):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

Lab tự tạo và xóa collection `lab26_ocr_documents`; không đụng collection khác. Không cần mạng khi chạy (lần đầu uv tải `pymongo`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/26-mongodb-pymongo-document-ocr examples/toi-noi-python/26-mongodb-pymongo-document-ocr/main.py
uv run --project examples/toi-noi-python/26-mongodb-pymongo-document-ocr examples/toi-noi-python/26-mongodb-pymongo-document-ocr/main.py --verify
```

Kết quả `--verify` kỳ vọng: `OK 37/37`, exit code 0. Khi chưa có MongoDB: `FAIL: không nối được MongoDB tại <url> (docker compose …)` và exit code 1.

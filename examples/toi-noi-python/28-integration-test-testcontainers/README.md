# Lab 28 - Integration test với PostgreSQL và MongoDB thật: testcontainers và fixture

Bài viết: `api/seed/content/python-integration-test-testcontainers-fixture.md`.

- `trips_db.py`: `TripRepository` (PostgreSQL, psycopg 3: CHECK, partial unique index, chuyển trạng thái có điều kiện, `FOR UPDATE SKIP LOCKED`) và `OcrDocumentStore` (MongoDB, PyMongo: unique index, truy vấn field lồng).
- `tests/conftest.py`: fixture container (scope `session`, đổi được bằng `LAB28_SCOPE`), schema/database riêng mỗi ca (scope `function`), `connect` để mở thêm kết nối cho ca cần hai phiên.
- `tests/test_trips_postgres.py`, `tests/test_ocr_mongodb.py`: 16 ca. `tests/test_why_not_sqlite.py`: 3 ca đặt SQLite cạnh PostgreSQL.

Cần Docker đang chạy (image `pgvector/pgvector:pg17` và `mongo:8`, lần đầu cần mạng để tải; thêm image `testcontainers/ryuk` để dọn container).

```bash
uv run --project examples/toi-noi-python/28-integration-test-testcontainers pytest examples/toi-noi-python/28-integration-test-testcontainers -q
```

Kết quả kỳ vọng: `19 passed`, exit code 0.

Dùng hạ tầng compose sẵn thay vì dựng container (fixture tự chọn khi có biến môi trường):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
TOINOI_PG_URL=postgresql://toinoi:toinoi@localhost:54329/toinoi \
TOINOI_MONGO_URL=mongodb://localhost:27019/toinoi \
uv run --project examples/toi-noi-python/28-integration-test-testcontainers pytest examples/toi-noi-python/28-integration-test-testcontainers -q
```

Mỗi ca tạo schema `t_<hex>` (PostgreSQL) hoặc database `t_<hex>` (MongoDB) và xóa khi xong, nên chạy trên compose không để lại dữ liệu. `LAB28_SCOPE=function` ép dựng container cho từng ca để thấy giá của scope sai.

Nếu pytest treo ở setup của ca đầu: testcontainers-python 4.15 mặc định dùng ảnh Ryuk `testcontainers/ryuk:0.8.1`, và lần đầu nó phải tải ảnh này. Thử `docker pull testcontainers/ryuk:0.8.1`; nếu lệnh kẹt thì kiểm credential helper của Docker. Lối thoát: đặt `RYUK_CONTAINER_IMAGE=testcontainers/ryuk:<phiên bản đã có trên máy>` trước khi chạy pytest.

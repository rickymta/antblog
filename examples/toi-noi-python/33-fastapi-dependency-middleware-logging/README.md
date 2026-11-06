# Lab 33 · FastAPI: dependency, lifespan, middleware, cấu hình và log JSON

Đi kèm bài `python-fastapi-dependency-middleware-settings-logging`. Lab tự chứa, không cần hạ tầng, không cần mạng khi chạy (lần resolve đầu cần mạng để tải `fastapi`, `uvicorn`, `pydantic-settings`, `httpx`).

- `app/settings.py`: `Settings(BaseSettings)` đọc `TOINOI_*`, kiểm tra ngay khi khởi động.
- `app/logs.py`: `JsonFormatter`, `RequestIdFilter`, `request_id_var` (contextvars), `configure_logging`.
- `app/middleware.py`: middleware `@app.middleware("http")` gán `X-Request-Id`, đo thời gian, ghi một dòng `request.completed` mỗi request.
- `app/deps.py`: dependency cấu hình, phiên kho chuyến (có `yield`), khách hiện tại (chỗ tạm, bài 34 thay), đồng hồ.
- `app/routes.py`: `POST /v1/quotes`, `GET /v1/trips/{trip_id}` và các route quan sát `/v1/diagnostics/*` (chỉ khi `TOINOI_ENVIRONMENT=dev`).
- `app/schemas.py`, `app/problems.py`: hợp đồng `POST /v1/quotes` (201 + `Location`, `quote_id`) và Problem Details copy từ lab 32, thêm `request_id`.
- `app/repo.py`, `app/domain.py`: kho chuyến giả trong bộ nhớ; miền báo giá copy từ lab 32.
- `main.py`: các demo in trong bài và chế độ `--verify`, gọi app trong tiến trình bằng `httpx.AsyncClient` + `ASGITransport`, chạy lifespan thật.

Dữ liệu là giả. Phiên bản đã chạy: Python 3.14.7, FastAPI 0.162.0, Starlette 1.10.0, uvicorn 0.54.0, pydantic 2.14.3, pydantic-settings 2.16.2, httpx 0.28.1 (xem `uv.lock`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/33-fastapi-dependency-middleware-logging examples/toi-noi-python/33-fastapi-dependency-middleware-logging/main.py
uv run --project examples/toi-noi-python/33-fastapi-dependency-middleware-logging examples/toi-noi-python/33-fastapi-dependency-middleware-logging/main.py --verify
```

Chạy server thật (log JSON ra stdout, `--no-access-log` vì middleware đã ghi một dòng mỗi request):

```bash
TOINOI_LOG_LEVEL=DEBUG uv run --project examples/toi-noi-python/33-fastapi-dependency-middleware-logging uvicorn app.main:app --app-dir examples/toi-noi-python/33-fastapi-dependency-middleware-logging --port 8033 --no-access-log
curl -i -X POST localhost:8033/v1/quotes -H 'content-type: application/json' -d '{"distance_km":"2.5","service":"ride"}'
curl -i localhost:8033/v1/trips/TN-0001 -H 'X-Customer-Id: CU-001' -H 'X-Request-Id: curl-demo-0001'
```

Cấu hình sai thì server không lên: `TOINOI_QUOTE_TTL_SECONDS=5` ném `ValidationError` ngay khi import `app.main`.

## Kết quả kỳ vọng

- Lệnh đầu in tám nhóm demo: cấu hình, cache dependency (kèm đối chứng và bẫy khác scope), hợp đồng lỗi và commit lỗi hai scope, thứ tự vào/ra của dependency có `yield`, middleware và log, contextvars, chiều đi của `ContextVar`, lỗi 500 và lifespan.
- `--verify` kết thúc bằng `OK 56/56` (exit code 0). Nếu lệch: in `SAI: …`, rồi `FAIL n/m` và exit 1.
- `--verify` tự gỡ mọi biến `TOINOI_*` đang có trên máy trong lúc chạy.

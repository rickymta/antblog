# Lab 32 - Validation, lỗi có cấu trúc (Problem Details) và hợp đồng OpenAPI

Bài viết: `api/seed/content/python-fastapi-validation-problem-details-openapi.md`. Lab tự chứa, không cần hạ tầng; lần resolve đầu cần mạng.

- `app/problems.py`: năm exception handler đưa mọi lỗi về `application/problem+json` (RFC 9457): `RequestValidationError` (422), `DistanceOutOfRange` (400), `QuoteNotFound` (404), `HTTPException` của framework (404, 405...), và `Exception` bất ngờ (500, chi tiết chỉ vào log). Có hàm đổi khóa `application/json` thành `application/problem+json` trong OpenAPI.
- `app/schemas.py`: `QuoteRequest`, `QuoteResponse`, và `ProblemDetails`, `FieldProblem` cho tài liệu lỗi.
- `app/main.py`: router `/v1`, `operation_id`, `tags`, `responses=` khai báo lỗi.
- `app/domain.py`: như bài 31, thêm hai exception có kiểu (`DistanceOutOfRange` vẫn là `ValueError`).
- `openapi.snapshot.json`: bản chụp hợp đồng đã commit. `--verify` so `app.openapi()` với file này.
- `main.py`: các demo in trong bài, `--verify`, và `--write-snapshot` để ghi lại snapshot sau khi CỐ Ý đổi hợp đồng.

Phiên bản đã chạy: Python 3.14.7, FastAPI 0.162.0, Starlette 1.10.0, pydantic 2.14.3, uvicorn 0.54.0, httpx 0.28.1 (xem `uv.lock`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details examples/toi-noi-python/32-fastapi-validation-problem-details/main.py
uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details examples/toi-noi-python/32-fastapi-validation-problem-details/main.py --verify
```

Server thật (cổng 8032), dừng bằng Ctrl+C:

```bash
uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details uvicorn app.main:app --app-dir examples/toi-noi-python/32-fastapi-validation-problem-details --port 8032
curl -i -X POST http://127.0.0.1:8032/v1/quotes -H "Content-Type: application/json" -d '{"distance_km": "150"}'
```

## Kết quả kỳ vọng

- Lệnh đầu in năm nhóm: 422 mặc định của FastAPI, 422 dạng Problem Details (bảy ca xấu), 400/404/405/500, tóm tắt OpenAPI (kể cả cái bẫy quên khai báo `responses`), và diff snapshot khi giả lập đổi 201 thành 200.
- `--verify` kết thúc bằng `OK 51/51` (exit code 0). Nếu lệch: in `SAI: <tên ca>`, rồi `FAIL n/m` và exit 1.
- Khi cố ý đổi hợp đồng (thêm route, đổi mã trạng thái), `--verify` đỏ ở ca snapshot; chạy `main.py --write-snapshot` để chấp nhận thay đổi rồi commit file cùng thay đổi code.

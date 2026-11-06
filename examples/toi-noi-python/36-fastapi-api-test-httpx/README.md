# Lab 36 · API test với httpx.AsyncClient

Đi kèm bài `python-fastapi-api-test-httpx`. Một API FastAPI nhỏ (báo giá theo hợp đồng của lab 32, xem và hủy chuyến, JWT theo scope như lab 34, lỗi `application/problem+json` như lab 32) cùng bộ 59 test kiểm chứng hợp đồng của nó mà không cần mở cổng mạng.

- `app/`: `main.py` (route, `create_app(settings)`, lifespan), `domain.py` (cước, trạng thái chuyến), `schemas.py` (model request/response/lỗi), `security.py` (JWT HS256 bằng PyJWT, `require_scope` dùng `OAuth2PasswordBearer` + `Security` nên scope hiện trong OpenAPI), `problems.py` (Problem Details và schema OpenAPI khớp với nó), `request_id.py` (mã yêu cầu như bài 33), `repo.py`, `settings.py` (không có mặc định cho môi trường và khóa ký). Tự chứa: mã chép từ các lab trước, không import từ lab khác. Lab chỉ KIỂM token, không cấp token.
- `tests/conftest.py`: `app` mới mỗi test, `client` (`httpx.AsyncClient` + `ASGITransport` + lifespan), `auth` (tự ký token bằng `jwt.encode`, có tham số làm hỏng token), đồng hồ cố định qua `dependency_overrides`, cờ `--update-snapshots`.
- `tests/test_quotes.py`, `test_auth.py`, `test_trips.py`, `test_problems.py`, `test_openapi.py` và `tests/snapshots/openapi.json` (snapshot hợp đồng, commit cùng code). Nâng FastAPI hoặc pydantic có thể làm snapshot đổi dù hợp đồng không đổi; khi đó đọc diff rồi mới ghi lại.
- Không cần Docker hay cơ sở dữ liệu. Khóa JWT của test sinh trong `conftest.py`; production dùng IdP và khóa bất đối xứng (bài 34).

## Chạy

Từ thư mục gốc repository (lần đầu uv tải gói nên cần mạng):

```bash
uv run --project examples/toi-noi-python/36-fastapi-api-test-httpx pytest examples/toi-noi-python/36-fastapi-api-test-httpx -q
```

Cố ý đổi hợp đồng rồi ghi lại snapshot:

```bash
uv run --project examples/toi-noi-python/36-fastapi-api-test-httpx pytest examples/toi-noi-python/36-fastapi-api-test-httpx -q --update-snapshots
```

Chạy service để thử tay (cổng 8036, tài liệu tại `http://localhost:8036/docs`). Cần hai biến môi trường, thiếu là service không lên. Endpoint cần token nên tự ký một token ngắn hạn với cùng khóa, issuer và audience:

```bash
export TOINOI_ENVIRONMENT=dev TOINOI_JWT_SECRET=lab36-dev-secret-not-for-production
uv run --project examples/toi-noi-python/36-fastapi-api-test-httpx uvicorn --factory app.main:create_app --app-dir examples/toi-noi-python/36-fastapi-api-test-httpx --port 8036
uv run --project examples/toi-noi-python/36-fastapi-api-test-httpx python -c "import jwt,datetime as d; n=d.datetime.now(d.UTC); print(jwt.encode({'iss':'toinoi-lab36','aud':'toinoi-api','sub':'CU-001','scope':'customer','iat':n,'exp':n+d.timedelta(minutes=15)}, 'lab36-dev-secret-not-for-production', algorithm='HS256'))"
```

## Kết quả kỳ vọng

- `59 passed` (exit code 0): 21 test báo giá, 9 xác thực, 13 chuyến, 10 hình dạng lỗi, 6 hợp đồng OpenAPI.
- Đã chạy với fastapi 0.162.0, starlette 1.10.0, httpx 0.28.1, httpx2 2.16.0, pytest 9.2.0, pytest-asyncio 1.4.1, pyjwt 2.15.1, jsonschema 4.26.0 trên Python 3.14.7. `httpx2` có mặt vì `starlette.testclient` của Starlette 1.10 yêu cầu nó; nếu chỉ có `httpx`, `TestClient` vẫn chạy nhưng in cảnh báo deprecated. Phần `AsyncClient` của bài dùng `httpx`.

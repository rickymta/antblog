# Lab 34 · FastAPI: OAuth2 password + bearer, JWT, scope

Đi kèm bài `python-fastapi-jwt-oauth2-authorization`. Lab tự chứa, không cần hạ tầng, không cần mạng khi chạy (lần resolve đầu cần mạng để tải `fastapi`, `pyjwt`, `pwdlib[argon2]`, `python-multipart`, ...).

- `app/settings.py`: `Settings(BaseSettings)`; `TOINOI_ENVIRONMENT` và `TOINOI_JWT_SECRET` **bắt buộc, không có mặc định**. Khóa ký là `SecretStr`; thiếu biến, khóa ngắn hơn 32 byte, khóa lặp ký tự, hay prod dùng khóa của lab (`lab34-dev-secret-not-for-production`) thì service không lên; `hide_input_in_errors=True` nên thông báo lỗi không in khóa.
- `app/security.py`: `TokenService` (PyJWT, HS256, kiểm `exp`, `iat`, `sub`, `iss`, `aud`), `current_principal` (401/403), `require_scope(...)` dựa trên `Security` + `SecurityScopes` nên scope hiện trong OpenAPI.
- `app/users.py`: bốn tài khoản giả `CU-001`, `CU-002`, `DR-001`, `DP-001`, mật khẩu băm argon2id bằng `pwdlib`. Mật khẩu của lab (`lab34-cu001-pass`, ...) là giá trị giả, có trong `main.py`.
- `app/routes.py`: `POST /v1/auth/token` (form OAuth2), `/v1/me`, `/v1/quotes`, `/v1/trips/{trip_id}`, `/v1/driver/availability`, `/v1/dispatch/queue`.
- `app/problems.py`, `app/schemas.py`, `app/request_id.py`: Problem Details và hợp đồng `POST /v1/quotes` copy từ lab 32 và 33 (có `request_id`), thêm lỗi OAuth2 `invalid_grant`/`invalid_scope` (HTTP 400, thành viên `error`).
- `main.py`: các demo in trong bài và chế độ `--verify`, gọi app trong tiến trình.

Phiên bản đã chạy: Python 3.14.7, FastAPI 0.162.0, PyJWT 2.15.1, pwdlib 0.3.1 (argon2-cffi 25.1.0), python-multipart 0.0.33, pydantic-settings 2.16.2 (xem `uv.lock`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/34-fastapi-jwt-authorization examples/toi-noi-python/34-fastapi-jwt-authorization/main.py
uv run --project examples/toi-noi-python/34-fastapi-jwt-authorization examples/toi-noi-python/34-fastapi-jwt-authorization/main.py --verify
```

Chạy server thật (sinh khóa ngẫu nhiên, không dùng khóa của lab) rồi lấy token bằng `curl`; `create_app` được gọi qua `--factory`, nên import không cần biến môi trường:

```bash
export TOINOI_ENVIRONMENT=dev TOINOI_JWT_SECRET=$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')
uv run --project examples/toi-noi-python/34-fastapi-jwt-authorization uvicorn --factory app.main:create_app --app-dir examples/toi-noi-python/34-fastapi-jwt-authorization --port 8034
curl -s -X POST localhost:8034/v1/auth/token -d 'username=CU-001&password=lab34-cu001-pass'
curl -i localhost:8034/v1/me -H "Authorization: Bearer <access_token>"
```

## Kết quả kỳ vọng

- Lệnh đầu in bốn nhóm demo: đăng nhập và cấu trúc token, bảng quyền (ẩn danh, khách, tài xế, điều phối), mười kiểu token xấu cùng lý do trong log, cấu hình khóa ký (không mặc định, không lộ khóa trong lỗi).
- `--verify` kết thúc bằng `OK 73/73` (exit code 0). Nếu lệch: in `SAI: …`, rồi `FAIL n/m` và exit 1.
- `--verify` tự gỡ mọi biến `TOINOI_*` đang có trên máy trong lúc chạy và dùng khóa ký sinh ngẫu nhiên cho mỗi lần chạy.

## Giới hạn

HS256 dùng chung một khóa để ký và kiểm: mọi service kiểm token đều ký được token. Production dùng nhà cung cấp danh tính (IdP) và khóa bất đối xứng (RS256/ES256) với JWKS; lab không có thu hồi token, refresh token, giới hạn số lần đăng nhập sai, TLS.

"""Bài 34: OAuth2 password + bearer, JWT HS256 (PyJWT), scope và argon2 (pwdlib) cho API Tới Nơi.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Không cần server đang chạy và không cần hạ tầng: kịch bản gọi app trong tiến trình bằng
httpx.AsyncClient + ASGITransport. Mật khẩu, khóa ký trong lab là giá trị giả, chỉ để chạy thử.
"""

import asyncio
import base64
import json
import logging
import os
import re
import secrets
import sys
import time
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from datetime import UTC, datetime, timedelta

import httpx
import jwt
from fastapi import FastAPI
from pydantic import SecretStr, ValidationError

from app.main import create_app
from app.security import TokenService
from app.settings import ENV_PREFIX, LAB_DEV_SECRET, Settings

QUIET = False
# Khóa ký của mỗi lần chạy lab: sinh ngẫu nhiên, không nằm trong mã nguồn hay mặc định của Settings.
TEST_SECRET = secrets.token_urlsafe(32)
PASSWORDS = {"CU-001": "lab34-cu001-pass", "CU-002": "lab34-cu002-pass", "DR-001": "lab34-dr001-pass", "DP-001": "lab34-dp001-pass"}


def say(*parts: object) -> None:
    if not QUIET:
        print(*parts)


class Checker:
    def __init__(self) -> None:
        self.total = 0
        self.failures: list[str] = []

    def check(self, name: str, got: object, expected: object) -> None:
        self.total += 1
        if got != expected:
            self.failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")


class ListHandler(logging.Handler):
    """Gom message log của logger `toinoi` để kiểm tra điều đã (và chưa) được ghi."""

    def __init__(self) -> None:
        super().__init__()
        self.messages: list[str] = []

    def emit(self, record: logging.LogRecord) -> None:
        self.messages.append(record.getMessage())


def lab_settings(**overrides: object) -> Settings:
    return Settings(environment="dev", jwt_secret=SecretStr(TEST_SECRET), **overrides)  # type: ignore[arg-type]


@asynccontextmanager
async def running(settings: Settings | None = None) -> AsyncIterator[tuple[FastAPI, httpx.AsyncClient, ListHandler]]:
    app = create_app(settings or lab_settings())
    handler = ListHandler()
    async with app.router.lifespan_context(app):
        logger = logging.getLogger("toinoi")
        logger.addHandler(handler)
        logger.propagate = False  # không in ra màn hình: kịch bản tự kiểm log qua ListHandler
        for h in list(logger.handlers):
            if isinstance(h, logging.StreamHandler) and not isinstance(h, ListHandler):
                logger.removeHandler(h)
        transport = httpx.ASGITransport(app=app)
        async with httpx.AsyncClient(transport=transport, base_url="http://lab34") as client:
            yield app, client, handler
        logger.removeHandler(handler)


async def login(client: httpx.AsyncClient, user: str, scope: str = "") -> httpx.Response:
    return await client.post("/v1/auth/token", data={"username": user, "password": PASSWORDS[user], "scope": scope})


def strip_rid(message: str) -> str:
    """Bỏ phần ` request_id=...` ở cuối dòng log (giá trị khác nhau mỗi lần chạy)."""
    return re.sub(r" request_id=\S+$", "", message)


def bearer(token: str) -> dict[str, str]:
    return {"Authorization": f"Bearer {token}"}


def b64url_json(part: str) -> dict[str, object]:
    return json.loads(base64.urlsafe_b64decode(part + "=" * (-len(part) % 4)))


def b64url_encode(data: dict[str, object]) -> str:
    return base64.urlsafe_b64encode(json.dumps(data, separators=(",", ":")).encode()).rstrip(b"=").decode()


# ---------------------------------------------------------------- 1. mật khẩu và token


async def demo_login(c: Checker) -> None:
    say("== 1. đăng nhập: argon2 kiểm mật khẩu, JWT chứa tối thiểu ==")
    async with running() as (app, client, logs):
        hasher = app.state.hasher
        record = app.state.users["CU-001"]
        started = time.perf_counter()
        hasher.verify(PASSWORDS["CU-001"], record.password_hash)
        verify_ms = (time.perf_counter() - started) * 1000
        say(f"hash lưu trữ : {record.password_hash[:31]}…")
        say(f"verify một mật khẩu: khoảng {verify_ms:.0f} ms (cố ý chậm; máy của bạn sẽ khác)")
        c.check("argon2: hash là argon2id", record.password_hash.startswith("$argon2id$"), True)
        c.check("argon2: cùng mật khẩu, hai lần băm khác nhau (có muối)",
                hasher.hash("lab34-cu001-pass") != hasher.hash("lab34-cu001-pass"), True)
        c.check("argon2: mật khẩu đúng / sai", (hasher.verify(PASSWORDS["CU-001"], record.password_hash),
                                                hasher.verify("sai", record.password_hash)), (True, False))

        r = await login(client, "CU-001")
        body = r.json()
        say(f"POST /v1/auth/token -> {r.status_code} {json.dumps({k: (v if k != 'access_token' else v[:24] + '…') for k, v in body.items()}, ensure_ascii=False)}")
        c.check("login: 200, bearer, 900 giây, scope customer",
                (r.status_code, body["token_type"], body["expires_in"], body["scope"]), (200, "bearer", 900, "customer"))
        say(f"header cache: Cache-Control={r.headers.get('cache-control')} Pragma={r.headers.get('pragma')}")
        c.check("login: response chứa token có Cache-Control: no-store và Pragma: no-cache (RFC 6749 mục 5.1)",
                (r.headers.get("cache-control"), r.headers.get("pragma")), ("no-store", "no-cache"))

        header_part, payload_part, _sig = body["access_token"].split(".")
        header, payload = b64url_json(header_part), b64url_json(payload_part)
        say(f"header : {header}")
        say(f"payload: {payload}")
        say("(đọc được bằng base64: chữ ký chống SỬA, không chống ĐỌC)")
        c.check("token: header HS256", header, {"alg": "HS256", "typ": "JWT"})
        c.check("token: đúng sáu claim", set(payload), {"iss", "aud", "sub", "iat", "exp", "scope"})
        c.check("token: sub, scope, hạn 900 giây",
                (payload["sub"], payload["scope"], int(payload["exp"]) - int(payload["iat"])), ("CU-001", "customer", 900))
        c.check("token: không chứa mật khẩu hay hash", "pass" in json.dumps(payload) or "argon" in json.dumps(payload), False)

        wrong = await client.post("/v1/auth/token", data={"username": "CU-001", "password": "sai-mat-khau"})
        ghost = await client.post("/v1/auth/token", data={"username": "CU-999", "password": "sai-mat-khau"})
        say(f"sai mật khẩu -> {wrong.status_code} {json.dumps(wrong.json(), ensure_ascii=False)}")
        say(f"tài khoản không có -> {ghost.status_code} {ghost.json()['error']}: {ghost.json()['detail']}")

        def oauth_shape(r: httpx.Response) -> tuple[int, str, str, str, str | None]:
            j = r.json()
            return r.status_code, r.headers["content-type"], j["error"], j["detail"], r.headers.get("cache-control")

        c.check("login: sai mật khẩu và không có tài khoản trả y hệt nhau (trừ request_id)",
                oauth_shape(wrong), oauth_shape(ghost))
        c.check("login: lỗi là 400 problem+json có error=invalid_grant, no-store",
                oauth_shape(wrong), (400, "application/problem+json", "invalid_grant", "sai tên đăng nhập hoặc mật khẩu", "no-store"))
        escalate = await login(client, "CU-001", scope="customer dispatcher")
        c.check("login: xin scope ngoài quyền bị từ chối invalid_scope",
                (escalate.status_code, escalate.json()["error"]), (400, "invalid_scope"))
        subset = await login(client, "DP-001", scope="dispatcher")
        c.check("login: xin đúng scope được phép", subset.json()["scope"], "dispatcher")
        c.check("log: ghi login thành công và thất bại, không ghi mật khẩu",
                ["auth.login.succeeded user=CU-001 scope=customer",
                 "auth.login.failed known_user=True",
                 "auth.login.failed known_user=False"],
                [strip_rid(m) for m in logs.messages if m.startswith("auth.login")][:3])
        c.check("log: không có mật khẩu hay token nào trong log",
                any(pw in m for m in logs.messages for pw in [*PASSWORDS.values(), "sai-mat-khau", body["access_token"]]), False)


# ---------------------------------------------------------------- 2. bảng quyền


async def demo_matrix(c: Checker) -> None:
    say("\n== 2. bảng quyền: ai gọi route nào ==")
    routes = [
        ("GET", "/v1/me"),
        ("POST", "/v1/quotes"),
        ("GET", "/v1/trips/TN-0001"),
        ("GET", "/v1/trips/TN-0002"),
        ("GET", "/v1/driver/availability"),
        ("GET", "/v1/dispatch/queue"),
    ]
    labels = ["me", "quotes", "TN-0001", "TN-0002", "driver", "queue"]
    expected = {
        None: [401, 401, 401, 401, 401, 401],
        "CU-001": [200, 201, 200, 404, 403, 403],
        "DR-001": [200, 403, 200, 404, 200, 403],
        "DP-001": [200, 403, 200, 200, 403, 200],
    }
    async with running() as (_, client, _logs):
        say(f"{'':8}" + "".join(f"{label:>9}" for label in labels))
        for user, want in expected.items():
            headers = bearer((await login(client, user)).json()["access_token"]) if user else {}
            got: list[int] = []
            for method, path in routes:
                kwargs = {"json": {"distance_km": "2.5"}} if method == "POST" else {}
                got.append((await client.request(method, path, headers=headers, **kwargs)).status_code)
            say(f"{user or 'ẩn danh':8}" + "".join(f"{code:>9}" for code in got))
            c.check(f"ma trận quyền: {user or 'ẩn danh'}", got, want)

        token = (await login(client, "CU-001")).json()["access_token"]
        r403 = await client.get("/v1/dispatch/queue", headers=bearer(token))
        r401 = await client.get("/v1/me")
        say(f"WWW-Authenticate của 401: {r401.headers.get('www-authenticate')}")
        say(f"WWW-Authenticate của 403: {r403.headers.get('www-authenticate')}")
        c.check("401 kèm thách thức Bearer", r401.headers.get("www-authenticate"), "Bearer")
        c.check("403 nêu scope còn thiếu", r403.headers.get("www-authenticate"), 'Bearer error="insufficient_scope", scope="dispatcher"')
        c.check("lỗi là application/problem+json", (r401.headers["content-type"], r403.headers["content-type"]),
                ("application/problem+json", "application/problem+json"))
        _logs.messages.clear()
        denied = await client.get("/v1/trips/TN-0002", headers=bearer(token))
        absent = await client.get("/v1/trips/TN-9999", headers=bearer(token))

        def trip_shape(r: httpx.Response) -> tuple[int, str, str]:
            return r.status_code, r.json()["type"], r.json()["detail"]

        c.check("404: chuyến của người khác và chuyến không tồn tại cho cùng body (trừ instance, request_id)",
                trip_shape(denied), trip_shape(absent))
        c.check("log: chuyến có thật bị từ chối thì để lại dấu vết, chuyến không có thì không",
                [strip_rid(m) for m in _logs.messages if m.startswith("auth.trip.denied")],
                ["auth.trip.denied user=CU-001 trip=TN-0002"])
        queue = await client.get("/v1/dispatch/queue", headers=bearer((await login(client, "DP-001")).json()["access_token"]))
        c.check("dispatcher thấy hàng đợi", queue.json(), {"waiting": ["TN-0003"]})
        bad = await client.post("/v1/quotes", json={"distance_km": "101"}, headers=bearer(token))
        c.check("customer báo giá 101 km: 400 từ miền, không phải từ auth", (bad.status_code, bad.json()["type"]),
                (400, "https://toinoi.example/problems/distance-out-of-range"))
        malformed = await client.post("/v1/quotes", json={"distance_km": "abc", "mau": "do"}, headers=bearer(token))
        c.check("quotes: 422 theo errors[field, code, message], không có input",
                (malformed.status_code, [e["field"] for e in malformed.json()["errors"]], "abc" in malformed.text),
                (422, ["body.distance_km", "body.mau"], False))
        made = await client.post("/v1/quotes", json={"distance_km": "2.5"}, headers=bearer(token))
        c.check("quotes: 201 + Location + quote_id, cước 14250",
                (made.status_code, made.headers["location"] == f"/v1/quotes/{made.json()['quote_id']}",
                 bool(re.fullmatch(r"QT-\d{4}", made.json()["quote_id"])), made.json()["fare"]),
                (201, True, True, "14250"))
        quote_id = made.json()["quote_id"]
        mine = await client.get(f"/v1/quotes/{quote_id}", headers=bearer(token))
        other_token = (await login(client, "CU-002")).json()["access_token"]
        theirs = await client.get(f"/v1/quotes/{quote_id}", headers=bearer(other_token))
        c.check("quotes: chủ đọc được; khách khác nhận 404 quote-not-found",
                (mine.status_code, theirs.status_code, theirs.json()["type"]),
                (200, 404, "https://toinoi.example/problems/quote-not-found"))
        r401 = await client.get("/v1/me", headers={"X-Request-Id": "auth-req-0001"})
        c.check("401: problem+json about:blank, có request_id, header trùng body",
                (r401.json()["type"], r401.json()["title"], r401.json()["request_id"], r401.headers["x-request-id"]),
                ("about:blank", "Unauthorized", "auth-req-0001", "auth-req-0001"))

        spec = (await client.get("/openapi.json")).json()
        scheme = spec["components"]["securitySchemes"]["OAuth2PasswordBearer"]
        c.check("OpenAPI: luồng password, tokenUrl và ba scope",
                (list(scheme["flows"]), scheme["flows"]["password"]["tokenUrl"], sorted(scheme["flows"]["password"]["scopes"])),
                (["password"], "/v1/auth/token", ["customer", "dispatcher", "driver"]))
        c.check("OpenAPI: route điều phối khai báo scope dispatcher",
                spec["paths"]["/v1/dispatch/queue"]["get"]["security"], [{"OAuth2PasswordBearer": ["dispatcher"]}])


# ---------------------------------------------------------------- 3. token xấu


def forged_tokens(app: FastAPI) -> dict[str, tuple[str, dict[str, str], str]]:
    """Trả về {tên ca: (token, header, reason kỳ vọng trong log)}. Mọi ca đều phải ra 401."""
    tokens: TokenService = app.state.tokens
    now = datetime.now(UTC)
    good = tokens.issue("CU-001", frozenset({"customer"}), now)
    claims = jwt.decode(good, options={"verify_signature": False})
    other_key = "khoa-khac-hoan-toan-0123456789-abcdef"
    head, payload, signature = good.split(".")
    escalated = b64url_encode({**b64url_json(payload), "scope": "dispatcher"})

    def sign(patch: dict[str, object] | None = None, drop: tuple[str, ...] = (), key: str = TEST_SECRET) -> str:
        data = {k: v for k, v in {**claims, **(patch or {})}.items() if k not in drop}
        return jwt.encode(data, key, algorithm="HS256")

    cases = {
        "hết hạn 1 giây": (tokens.issue("CU-001", frozenset({"customer"}), now - timedelta(seconds=tokens.ttl_seconds + 1)), "expired"),
        "sửa payload, giữ chữ ký": (f"{head}.{escalated}.{signature}", "bad_signature"),
        "ký bằng khóa khác": (sign(key=other_key), "bad_signature"),
        "alg=none, không chữ ký": (jwt.encode(claims, None, algorithm="none"), "bad_algorithm"),
        "sai audience": (sign({"aud": "dich-vu-khac"}), "bad_audience"),
        "sai issuer": (sign({"iss": "ai-do-khac"}), "bad_issuer"),
        "thiếu exp": (sign(drop=("exp",)), "missing_claim:exp"),
        "thiếu sub": (sign(drop=("sub",)), "missing_claim:sub"),
        "scope là danh sách": (sign({"scope": ["dispatcher"]}), "malformed"),
        "không phải JWT": ("not.a.jwt", "malformed"),
    }
    return {name: (tok, bearer(tok), reason) for name, (tok, reason) in cases.items()}


async def demo_bad_tokens(c: Checker) -> None:
    say("\n== 3. token xấu: cùng một câu trả lời 401, lý do thật chỉ nằm trong log ==")
    async with running() as (app, client, logs):
        cases = forged_tokens(app)
        for name, (token, headers, reason) in cases.items():
            logs.messages.clear()
            r = await client.get("/v1/me", headers=headers)
            logged = strip_rid(logs.messages[-1]) if logs.messages else ""
            say(f"{name:<26} -> {r.status_code} | log: {logged}")
            c.check(f"token xấu '{name}': 401 và lý do log", (r.status_code, logged), (401, f"auth.token.rejected reason={reason}"))
            c.check(f"token xấu '{name}': body không tiết lộ lý do", r.json().get("detail"), "token không hợp lệ hoặc đã hết hạn")
            c.check(f"token xấu '{name}': không ghi chính token vào log", token in " ".join(logs.messages), False)

        tokens: TokenService = app.state.tokens
        edge = tokens.issue("CU-001", frozenset({"customer"}), datetime.now(UTC) - timedelta(seconds=tokens.ttl_seconds - 30))
        r_edge = await client.get("/v1/me", headers=bearer(edge))
        say(f"{'còn 30 giây hạn':<26} -> {r_edge.status_code}")
        c.check("token còn 30 giây vẫn dùng được", r_edge.status_code, 200)
        basic = await client.get("/v1/me", headers={"Authorization": "Basic dXNlcjpwYXNz"})
        c.check("scheme Basic không được chấp nhận", basic.status_code, 401)


# ---------------------------------------------------------------- 4. cấu hình khóa


def demo_settings(c: Checker) -> None:
    say("\n== 4. cấu hình khóa ký: không mặc định, fail fast, không lộ khóa ==")
    fields = Settings.model_fields
    c.check("settings: environment và jwt_secret không có giá trị mặc định",
            (fields["environment"].is_required(), fields["jwt_secret"].is_required()), (True, True))
    messages: dict[str, str] = {}

    def attempt(label: str, **env: str) -> tuple[str, str]:
        """Dựng Settings() từ biến môi trường `env`; trả về (toàn bộ str(exc), bản tóm tắt để in)."""
        saved = {k: os.environ.pop(k, None) for k in ("TOINOI_ENVIRONMENT", "TOINOI_JWT_SECRET")}
        os.environ.update(env)
        try:
            Settings()  # type: ignore[call-arg]
            return "OK", "OK"
        except ValidationError as exc:
            short = "; ".join(f"{'.'.join(map(str, e['loc'])) or 'model'}: {e['msg']}" for e in exc.errors())
            return str(exc), short
        finally:
            for k in ("TOINOI_ENVIRONMENT", "TOINOI_JWT_SECRET"):
                os.environ.pop(k, None)
                if saved[k] is not None:
                    os.environ[k] = saved[k]  # type: ignore[assignment]

    short_key = "my-real-short-key"
    cases = {
        "thiếu cả hai biến": attempt("thiếu"),
        "khóa ngắn 17 byte": attempt("ngắn", TOINOI_ENVIRONMENT="dev", TOINOI_JWT_SECRET=short_key),
        "khóa lặp 48 ký tự": attempt("lặp", TOINOI_ENVIRONMENT="dev", TOINOI_JWT_SECRET="x" * 48),
        "prod + khóa của lab": attempt("lab", TOINOI_ENVIRONMENT="prod", TOINOI_JWT_SECRET=LAB_DEV_SECRET),
        "dev + khóa của lab": attempt("dev", TOINOI_ENVIRONMENT="dev", TOINOI_JWT_SECRET=LAB_DEV_SECRET),
    }
    for label, (text, short) in cases.items():
        say(f"{label:<20} -> {short}")
        messages[label] = text
    c.check("settings: thiếu biến thì lỗi gọi tên cả hai trường",
            ("environment" in messages["thiếu cả hai biến"], "jwt_secret" in messages["thiếu cả hai biến"]), (True, True))
    c.check("settings: khóa ngắn bị từ chối", "ngắn hơn 32 byte" in messages["khóa ngắn 17 byte"], True)
    c.check("settings: KHÔNG lộ khóa trong thông báo lỗi (hide_input_in_errors)",
            any(short_key in t or "x" * 20 in t for t in messages.values()), False)
    c.check("settings: khóa lặp bị từ chối", "quá đơn điệu" in messages["khóa lặp 48 ký tự"], True)
    c.check("settings: prod dùng khóa của lab bị từ chối", "không dùng được ở prod" in messages["prod + khóa của lab"], True)
    c.check("settings: dev với khóa của lab được phép (đặt tường minh)", messages["dev + khóa của lab"], "OK")
    generated = secrets.token_urlsafe(32)
    prod = Settings(environment="prod", jwt_secret=SecretStr(generated))
    say(f"prod + khóa sinh bằng secrets.token_urlsafe(32) ({len(generated)} ký tự) -> OK, repr={prod.jwt_secret!r}")
    c.check("settings: prod với khóa sinh ngẫu nhiên chạy được", prod.environment, "prod")
    c.check("settings: repr không in khóa", generated in repr(prod), False)


async def run() -> int:
    for noisy in ("httpx", "httpcore"):
        logging.getLogger(noisy).setLevel(logging.WARNING)
    for name in [k for k in os.environ if k.startswith(ENV_PREFIX)]:
        del os.environ[name]
    c = Checker()
    load1, load5, _ = os.getloadavg()
    say(f"(máy lúc chạy: load average {load1:.2f} / {load5:.2f}; mili giây của argon2 phụ thuộc tải này)\n")
    await demo_login(c)
    await demo_matrix(c)
    await demo_bad_tokens(c)
    demo_settings(c)
    if "--verify" in sys.argv[1:]:
        if c.failures:
            print(f"FAIL {c.total - len(c.failures)}/{c.total}")
            return 1
        print(f"OK {c.total}/{c.total}")
    return 1 if c.failures else 0


if __name__ == "__main__":
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(asyncio.run(run()))

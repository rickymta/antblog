"""Bài 33: FastAPI Depends, lifespan, middleware, pydantic-settings và log JSON có correlation id.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Không cần server đang chạy và không cần hạ tầng: kịch bản gọi app trong tiến trình bằng
httpx.AsyncClient + ASGITransport, chạy lifespan thật qua app.router.lifespan_context.
Muốn chạy server thật: xem README.md (uvicorn ... --port 8033).
"""

import asyncio
import io
import json
import logging
import os
import sys
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager, contextmanager
from contextvars import ContextVar
from datetime import UTC, datetime

import httpx
from fastapi import FastAPI, Request
from pydantic import ValidationError

from app.deps import utc_now
from app.logs import JsonFormatter
from app.main import create_app
from app.settings import ENV_PREFIX, Settings

QUIET = False
CUSTOMER = {"X-Customer-Id": "CU-001"}


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


@contextmanager
def env(**values: str):
    """Đặt biến môi trường tạm thời; trả lại nguyên trạng khi thoát."""
    saved = {k: os.environ.get(k) for k in values}
    os.environ.update(values)
    try:
        yield
    finally:
        for key, old in saved.items():
            if old is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = old


@asynccontextmanager
async def running(settings: Settings | None = None) -> AsyncIterator[tuple[FastAPI, httpx.AsyncClient, io.StringIO]]:
    """Dựng app với log ghi vào bộ đệm, chạy lifespan, trả client gọi thẳng vào app."""
    buffer = io.StringIO()
    app = create_app(settings or Settings(), buffer)
    async with app.router.lifespan_context(app):
        # raise_app_exceptions=False: lỗi chưa bắt trở thành response 500, như khi chạy sau uvicorn.
        transport = httpx.ASGITransport(app=app, raise_app_exceptions=False)
        async with httpx.AsyncClient(transport=transport, base_url="http://lab33") as client:
            yield app, client, buffer


def log_lines(buffer: io.StringIO) -> list[dict[str, object]]:
    return [json.loads(line) for line in buffer.getvalue().splitlines()]


def settings_error_report(exc: ValidationError) -> list[str]:
    return [f"{ENV_PREFIX}{str(e['loc'][0]).upper()}: {e['msg']} (nhận {e['input']!r})" for e in exc.errors()]


def compact(line: dict[str, object], keys: tuple[str, ...]) -> str:
    return " ".join(f"{k}={line[k]}" for k in keys if k in line)


# ---------------------------------------------------------------- 1. cấu hình


def demo_settings(c: Checker) -> None:
    say("== 1. pydantic-settings: đọc env, kiểm tra, fail fast ==")
    defaults = Settings()
    say(f"mặc định : {defaults}")
    c.check("settings.mặc định", (defaults.environment, defaults.log_level, defaults.quote_ttl_seconds), ("dev", "INFO", 300))
    with env(TOINOI_LOG_LEVEL="DEBUG", TOINOI_QUOTE_TTL_SECONDS="600", TOINOI_PG_URL="postgresql://khong-lien-quan"):
        from_env = Settings()
    say(f"từ env   : log_level={from_env.log_level} quote_ttl_seconds={from_env.quote_ttl_seconds} (TOINOI_PG_URL bị bỏ qua)")
    c.check("settings.env ghi đè, biến lạ bị bỏ qua", (from_env.log_level, from_env.quote_ttl_seconds), ("DEBUG", 600))
    with env(TOINOI_QUOTE_TTL_SECONDS="5", TOINOI_LOG_LEVEL="LOUD", TOINOI_ENVIRONMENT="staging"):
        try:
            Settings()
        except ValidationError as exc:
            report = settings_error_report(exc)
    say("cấu hình sai, service không lên:")
    for row in report:
        say(f"  {row}")
    c.check("settings.sai báo đủ 3 biến", len(report), 3)


# ---------------------------------------------------------------- 2. Depends


async def demo_dependencies(c: Checker) -> None:
    say("\n== 2. Depends: cache trong một request, ownership, thay thế được ==")
    async with running(Settings(log_level="DEBUG")) as (app, client, _):
        got = {}
        for name in ("dependency-cache", "dependency-cache-control", "dependency-scope-mismatch"):
            body = (await client.get(f"/v1/diagnostics/{name}")).json()
            got[name] = body
            say(f"{name:<27}: {json.dumps(body)}")
        c.check("cache: a, b cùng phiên, c (use_cache=False) là phiên khác", got["dependency-cache"]["distinct"], 2)
        c.check("cache: không use_cache=False thì ba tham số là MỘT phiên", got["dependency-cache-control"]["distinct"], 1)
        c.check("cache: khác scope thì KHÔNG dùng chung, mở hai phiên", got["dependency-scope-mismatch"]["distinct"], 2)
        c.check("cache: không phiên nào còn mở", app.state.db.open_now, 0)

        own = await client.get("/v1/trips/TN-0001", headers=CUSTOMER)
        other = await client.get("/v1/trips/TN-0002", headers=CUSTOMER)
        missing = await client.get("/v1/trips/TN-9999", headers=CUSTOMER)
        anonymous = await client.get("/v1/trips/TN-0001")
        forged = await client.get("/v1/trips/TN-0001", headers={"X-Customer-Id": "admin"})
        say(f"chuyến của mình {own.status_code}, của khách khác {other.status_code}, "
            f"không có {missing.status_code}, thiếu header {anonymous.status_code}, header sai dạng {forged.status_code}")
        c.check("trips: chủ chuyến 200", own.status_code, 200)
        def shape(r: httpx.Response) -> tuple[int, str, str]:
            return r.status_code, r.json()["type"], r.json()["detail"]

        c.check("trips: khách khác 404 như chuyến không tồn tại (cùng type và detail)", shape(other), shape(missing))
        c.check("trips: thiếu / sai header 401", (anonymous.status_code, forged.status_code), (401, 401))
        c.check("trips: 401 kèm WWW-Authenticate: Bearer",
                (anonymous.headers.get("www-authenticate"), forged.headers.get("www-authenticate")), ("Bearer", "Bearer"))
        c.check("trips: chuyến trả đủ trường", own.json()["status"], "in_transit")

        # dependency_overrides: thay thế dependency mà không sửa handler (bài 36 dùng nhiều hơn).
        fixed = datetime(2026, 1, 1, 0, 0, tzinfo=UTC)
        app.dependency_overrides[utc_now] = lambda: fixed
        quote = await client.post("/v1/quotes", json={"distance_km": "2.5", "service": "ride"})
        say(f"quote với đồng hồ cố định: {quote.status_code} {quote.text}")
        c.check("quote: 201, cước 14250 là chuỗi", (quote.status_code, quote.json()["fare"]), (201, "14250"))
        c.check("quote: hạn = now + quote_ttl_seconds", quote.json()["expires_at"], "2026-01-01T00:05:00Z")
        c.check("quote: có quote_id và Location", (quote.json()["quote_id"], quote.headers["location"]), ("QT-0001", "/v1/quotes/QT-0001"))
        app.dependency_overrides.clear()


# ---------------------------------------------------------------- 2b. hợp đồng lỗi của bài 32 + request_id


async def demo_contract(c: Checker) -> None:
    say("\n== 2b. Problem Details (bài 32) kèm request_id; hợp đồng POST /v1/quotes ==")
    async with running() as (app, client, _logs):
        rid = {"X-Request-Id": "contract-0001-ab"}
        ok = await client.post("/v1/quotes", json={"distance_km": "2.5", "service": "ride"}, headers=rid)
        c.check("quotes: 201, Location, quote_id", (ok.status_code, ok.headers["location"], ok.json()["quote_id"]),
                (201, "/v1/quotes/QT-0001", "QT-0001"))
        again = await client.get("/v1/quotes/QT-0001")
        c.check("quotes: GET báo giá vừa tạo", (again.status_code, again.json()["fare"]), (200, "14250"))

        far = await client.post("/v1/quotes", json={"distance_km": "101"}, headers=rid)
        bad = await client.post("/v1/quotes", json={"distance_km": "abc", "mau": "do"}, headers=rid)
        places = await client.post("/v1/quotes", json={"distance_km": "2.5555"})
        unknown = await client.get("/v1/quotes/QT-9999", headers=rid)
        no_route = await client.get("/v1/khong-co", headers=rid)
        wrong_method = await client.delete("/v1/quotes/QT-0001", headers=rid)
        for label, r in [("400", far), ("422", bad), ("404 báo giá", unknown), ("404 route", no_route), ("405", wrong_method)]:
            say(f"{label:<12}: {r.status_code} {r.headers['content-type']} {r.text}")

        c.check("lỗi: mọi response lỗi là application/problem+json",
                {r.headers["content-type"] for r in (far, bad, places, unknown, no_route, wrong_method)}, {"application/problem+json"})
        c.check("lỗi 400: type và extension min_km/max_km",
                (far.json()["type"], far.json()["max_km"]), ("https://toinoi.example/problems/distance-out-of-range", "100"))
        errors = bad.json()["errors"]
        c.check("lỗi 422: errors[field, code, message], không có input",
                (sorted(e["field"] for e in errors), all(set(e) == {"field", "code", "message"} for e in errors)),
                (["body.distance_km", "body.mau"], True))
        c.check("lỗi 422: quá 3 chữ số thập phân", places.json()["errors"][0]["code"], "decimal_max_places")
        c.check("lỗi 404 báo giá: type riêng", unknown.json()["type"], "https://toinoi.example/problems/quote-not-found")
        c.check("lỗi 404 route: about:blank, title chuẩn", (no_route.json()["type"], no_route.json()["title"]), ("about:blank", "Not Found"))
        c.check("lỗi 405: giữ header Allow", wrong_method.status_code == 405 and "GET" in wrong_method.headers["allow"], True)
        c.check("request_id có ở body của mọi lỗi có header",
                [r.json()["request_id"] for r in (far, bad, unknown, no_route, wrong_method)], ["contract-0001-ab"] * 5)
        c.check("lỗi: instance là đường dẫn request", far.json()["instance"], "/v1/quotes")

        boom = await client.get("/v1/diagnostics/boom", headers=rid)
        c.check("lỗi 500: type internal-error, không lộ tên lớp hay chi tiết",
                (boom.json()["type"], "RuntimeError" in boom.text, "giả lập" in boom.text),
                ("https://toinoi.example/problems/internal-error", False, False))

        from app.domain import calculate_fare
        try:
            calculate_fare(__import__("decimal").Decimal("2"), service="drone")
            lan = "không lỗi"
        except ValueError as exc:
            lan = type(exc).__name__
        c.check("miền: dịch vụ lạ là ValueError", lan, "ValueError")

        # Hai scope của dependency có yield khi commit thất bại.
        fn = await client.post("/v1/diagnostics/commit-function")
        rq = await client.post("/v1/diagnostics/commit-request")
        say(f"commit lỗi, scope function: {fn.status_code}; scope request: {rq.status_code}")
        c.check("scope function: commit lỗi thì client thấy 500", fn.status_code, 500)
        c.check("scope request: commit lỗi sau khi response 201 đã gửi", rq.status_code, 201)
        c.check("commit lỗi: phiên vẫn được đóng ở cả hai scope", app.state.db.open_now, 0)


# ---------------------------------------------------------------- 3. thứ tự vào / ra


async def demo_order(c: Checker) -> None:
    say("\n== 3. dependency có yield: scope request và scope function ==")
    async with running() as (_, client, buffer):
        await client.get("/v1/diagnostics/scope-order")
    order = [
        compact(line, ("event", "dep")) for line in log_lines(buffer)
        if line["logger"] in ("toinoi.api", "toinoi.access")
    ]
    for step in order:
        say(f"  {step}")
    c.check(
        "scope: function-scope thoát trước request.completed, request-scope thoát sau",
        order,
        [
            "event=dep.enter dep=request-scope",
            "event=dep.enter dep=function-scope",
            "event=handler.body",
            "event=dep.exit dep=function-scope",
            "event=request.completed",
            "event=dep.exit dep=request-scope",
        ],
    )


# ---------------------------------------------------------------- 4. middleware, request id, log


async def demo_middleware(c: Checker) -> None:
    say("\n== 4. middleware: request id, Server-Timing, một dòng log mỗi request ==")
    async with running(Settings(log_level="DEBUG", slow_request_ms=20)) as (_, client, buffer):
        good = await client.get("/v1/trips/TN-0001", headers={**CUSTOMER, "X-Request-Id": "web-0001-abcdef"})
        bad = await client.get("/v1/trips/TN-0001", headers={**CUSTOMER, "X-Request-Id": "x" * 200})
        spaced = await client.get("/v1/trips/TN-0001", headers={**CUSTOMER, "X-Request-Id": "co dau cach"})
        say(f"X-Request-Id hợp lệ  -> {good.headers['x-request-id']}")
        say(f"X-Request-Id 200 chữ -> {bad.headers['x-request-id']}")
        say(f"X-Request-Id có dấu cách -> {spaced.headers['x-request-id']}")
        c.check("request id: hợp lệ được giữ", good.headers["x-request-id"], "web-0001-abcdef")
        c.check("request id: quá dài bị thay bằng 32 ký tự hex", len(bad.headers["x-request-id"]), 32)
        c.check("request id: có dấu cách bị thay", spaced.headers["x-request-id"] != "co dau cach", True)
        c.check("Server-Timing có dạng app;dur=…", good.headers["server-timing"].startswith("app;dur="), True)

        await client.get("/v1/diagnostics/slow")
        await client.get("/nope")
        await client.get("/v1/trips/TN-0001", headers={**CUSTOMER, "Authorization": "Bearer bi-mat-khong-duoc-log"})

    lines = log_lines(buffer)
    one = [ln for ln in lines if ln["request_id"] == "web-0001-abcdef"]
    say("\nMọi dòng log của request đầu (cùng một request_id):")
    for ln in one:
        say("  " + json.dumps(ln, ensure_ascii=False))
    c.check("log: mọi dòng của request mang cùng request_id", [ln["event"] for ln in one],
            ["db.session.opened", "trip.read", "db.session.closed", "request.completed"])
    done = next(ln for ln in one if ln["event"] == "request.completed")
    c.check("log: route là mẫu, có customer_id từ request.state",
            (done["route"], done["status"], done["customer_id"], done["level"]),
            ("/v1/trips/{trip_id}", 200, "CU-001", "INFO"))
    slow = next(ln for ln in lines if ln["event"] == "request.completed" and ln["route"] == "/v1/diagnostics/slow")
    c.check("log: request chậm hơn ngưỡng là WARNING", slow["level"], "WARNING")
    notfound = next(ln for ln in lines if ln["event"] == "request.completed" and ln["status"] == 404)
    c.check("log: 404 không khớp route thì route là '-'", notfound["route"], "-")
    c.check("log: không dòng nào chứa Authorization", "bi-mat-khong-duoc-log" in json.dumps(lines), False)
    c.check("log: mọi dòng có đủ khóa chuẩn",
            all({"ts", "level", "logger", "service", "request_id", "event"} <= ln.keys() for ln in lines), True)

    record = logging.LogRecord("toinoi.demo", logging.INFO, __file__, 1, "Chuyến Đà Nẵng %s", ("TN-0001",), None)
    c.check("log: giữ nguyên tiếng Việt, không escape", "Đà Nẵng" in JsonFormatter("quote-api").format(record), True)


# ---------------------------------------------------------------- 5. contextvars


async def demo_context(c: Checker) -> None:
    say("\n== 5. contextvars: request id đi theo cả luồng và task ==")
    async with running() as (_, client, _buffer):
        r = await client.get("/v1/diagnostics/context", headers={"X-Request-Id": "ctx-demo-0001"})
        say(json.dumps(r.json(), indent=2))
        c.check("context: handler, dependency def, thread, task cùng id", set(r.json().values()), {"ctx-demo-0001"})

        ids = [f"req-{i:04d}-aaaa" for i in range(20)]
        replies = await asyncio.gather(*(
            client.get("/v1/diagnostics/context", headers={"X-Request-Id": rid}) for rid in ids
        ))
        mixed = [rid for rid, rep in zip(ids, replies, strict=True) if rep.json()["in_task"] != rid]
        say(f"20 request song song: {20 - len(mixed)}/20 task thấy đúng id của request mình")
        c.check("context: 20 request song song không lẫn id", mixed, [])


def demo_contextvar_direction(c: Checker) -> None:
    """ContextVar đặt trong handler KHÔNG đi ngược lên middleware @app.middleware("http")."""
    say("\n== 6. chiều đi của ContextVar qua middleware ==")
    touched: ContextVar[str] = ContextVar("touched", default="chưa đặt")
    probe = FastAPI()
    seen: dict[str, str] = {}

    @probe.middleware("http")
    async def reader(request: Request, call_next):  # noqa: ANN001, ANN202
        response = await call_next(request)
        seen["contextvar"] = touched.get()
        seen["request.state"] = getattr(request.state, "touched", "chưa đặt")
        return response

    @probe.get("/")
    async def handler(request: Request) -> dict[str, bool]:
        touched.set("đã đặt trong handler")
        request.state.touched = "đã đặt trong handler"
        return {"ok": True}

    async def call() -> None:
        async with httpx.AsyncClient(transport=httpx.ASGITransport(app=probe), base_url="http://t") as client:
            await client.get("/")

    asyncio.run(call())
    say(f"middleware đọc lại sau call_next: ContextVar -> {seen['contextvar']!r}; request.state -> {seen['request.state']!r}")
    c.check("contextvar: không đi ngược lên middleware", seen["contextvar"], "chưa đặt")
    c.check("request.state: đi ngược được", seen["request.state"], "đã đặt trong handler")


# ---------------------------------------------------------------- 7. lỗi 500 và lifespan


async def demo_failure_and_lifespan(c: Checker) -> None:
    say("\n== 7. lỗi chưa bắt và lifespan ==")
    async with running(Settings(log_level="DEBUG")) as (app, client, buffer):
        r = await client.get("/v1/diagnostics/boom", headers={"X-Request-Id": "boom-0001-xyz"})
        say(f"boom: {r.status_code} {r.headers['content-type']} X-Request-Id={r.headers.get('x-request-id')} body={r.text}")
        c.check("500: problem+json, header và body cùng request_id",
                (r.status_code, r.headers["content-type"], r.headers["x-request-id"], r.json()["request_id"]),
                (500, "application/problem+json", "boom-0001-xyz", "boom-0001-xyz"))
        c.check("500: phiên dependency vẫn được đóng (try/finally)", app.state.db.open_now, 0)
        after = await client.post("/v1/quotes", json={"distance_km": "5"})
        c.check("500: request sau vẫn phục vụ bình thường", after.status_code, 201)
        c.check("lifespan: db mở trong lúc chạy", app.state.db.closed, False)
    lines = log_lines(buffer)
    failed = next(ln for ln in lines if ln["event"] == "request.failed")
    last_exc_line = str(failed["exc"]).strip().splitlines()[-1]
    say(f"request.failed: level={failed['level']} status={failed['status']} request_id={failed['request_id']}")
    say(f"  exc (dòng cuối của traceback): {last_exc_line}")
    c.check("500: log request.failed mức ERROR kèm traceback",
            (failed["level"], failed["request_id"], last_exc_line.startswith("RuntimeError:")),
            ("ERROR", "boom-0001-xyz", True))
    events = [(ln["event"], ln["request_id"]) for ln in lines if ln["event"].startswith("service.")]
    say(f"lifespan: {events}")
    c.check("lifespan: started đầu tiên, stopping cuối cùng, ngoài request",
            (lines[0]["event"], lines[-1]["event"], lines[0]["request_id"], lines[-1]["open_sessions"]),
            ("service.started", "service.stopping", "-", 0))
    c.check("lifespan: db đóng sau khi tắt", app.state.db.closed, True)

    async with running(Settings(environment="prod")) as (_, client, _buffer):
        hidden = await client.get("/v1/diagnostics/context")
        openapi = (await client.get("/openapi.json")).json()
    say(f"môi trường prod: /v1/diagnostics/context -> {hidden.status_code}")
    c.check("prod: route diagnostics không tồn tại", hidden.status_code, 404)
    params = [p["name"] for p in openapi["paths"]["/v1/trips/{trip_id}"]["get"]["parameters"]]
    c.check("OpenAPI: tham số của dependency hiện trong tài liệu", sorted(params), ["trip_id", "x-customer-id"])


async def run() -> int:
    for noisy in ("httpx", "httpcore"):
        logging.getLogger(noisy).setLevel(logging.WARNING)
    for name in [k for k in os.environ if k.startswith(ENV_PREFIX)]:
        del os.environ[name]  # kết quả không phụ thuộc biến TOINOI_* sẵn có trên máy
    c = Checker()
    load1, load5, _ = os.getloadavg()
    say(f"(máy lúc chạy: load average {load1:.2f} / {load5:.2f}; các mili giây trong log phụ thuộc tải này)\n")
    demo_settings(c)
    await demo_dependencies(c)
    await demo_contract(c)
    await demo_order(c)
    await demo_middleware(c)
    await demo_context(c)
    await asyncio.to_thread(demo_contextvar_direction, c)
    await demo_failure_and_lifespan(c)
    if "--verify" in sys.argv[1:]:
        if c.failures:
            print(f"FAIL {c.total - len(c.failures)}/{c.total}")
            return 1
        print(f"OK {c.total}/{c.total}")
    return 1 if c.failures else 0


if __name__ == "__main__":
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(asyncio.run(run()))

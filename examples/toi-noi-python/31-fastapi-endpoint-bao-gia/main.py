"""Bài 31: HTTP và FastAPI - xuất bản endpoint báo giá.

Chạy:   uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia examples/toi-noi-python/31-fastapi-endpoint-bao-gia/main.py
Verify: thêm --verify ở cuối lệnh trên.
Server: xem README (uvicorn app.main:app --port 8031).

Mọi lời gọi đi qua httpx + ASGITransport: app chạy TRONG tiến trình này, không cần server,
không cần mở cổng. Con số giây ở phần cuối phụ thuộc máy; --verify chỉ kiểm bất biến.
"""

import asyncio
import inspect
import os
import sys
import threading
import time
from datetime import UTC, datetime

import httpx
from fastapi import FastAPI

from app.main import create_app
from app.raw import asgi_app, wsgi_app

VERIFY = "--verify" in sys.argv
results: list[tuple[str, bool]] = []


def say(*parts: object) -> None:
    if not VERIFY:
        print(*parts)


def check(name: str, ok: bool) -> None:
    results.append((name, bool(ok)))


def client_for(app) -> httpx.AsyncClient:
    return httpx.AsyncClient(transport=httpx.ASGITransport(app=app), base_url="http://tinoi.test")


# --- 1. WSGI và ASGI ở dạng trần ----------------------------------------------------------
async def demo_raw() -> None:
    say("== 1. WSGI và ASGI trần ==")
    captured: dict = {}

    def start_response(status: str, headers: list) -> None:
        captured["status"] = status

    body = b"".join(wsgi_app({"REQUEST_METHOD": "GET", "PATH_INFO": "/hello"}, start_response))
    say("WSGI :", captured["status"], body.decode())
    check("wsgi trả 200", captured["status"] == "200 OK")

    async with client_for(asgi_app) as c:
        r = await c.post("/hello", content=b"x" * 3000)
    say("ASGI :", r.status_code, r.text)
    check("asgi trần trả 200 và đếm đủ 3000 byte", r.status_code == 200 and r.json()["bytes_in"] == 3000)

    app = create_app()
    say("FastAPI cũng là một ASGI app: coroutine __call__ =", inspect.iscoroutinefunction(app.__call__))
    say("tham số của __call__:", list(inspect.signature(app.__call__).parameters))
    check("FastAPI.__call__ là coroutine (scope, receive, send)", inspect.iscoroutinefunction(app.__call__))
    seen: dict = {}

    async def spy(scope, receive, send):
        seen.update(type=scope["type"], method=scope.get("method"), path=scope.get("path"))
        await app(scope, receive, send)

    async with client_for(spy) as c:
        await c.get("/v1/quotes/QT-0001")
    say("scope mà FastAPI nhận:", seen)
    check("scope có type/method/path", seen == {"type": "http", "method": "GET", "path": "/v1/quotes/QT-0001"})


# --- 2. Endpoint báo giá ------------------------------------------------------------------
FARE_CASES = [("0", "12000"), ("2", "12000"), ("2.5", "14250"), ("5", "25500"), ("100", "453000")]


async def demo_quotes() -> None:
    say("\n== 2. POST /v1/quotes ==")
    app = create_app()  # app mới: bộ đếm quote_id bắt đầu lại từ QT-0001
    async with client_for(app) as c:
        for km, expected in FARE_CASES:
            r = await c.post("/v1/quotes", json={"distance_km": km, "service": "ride"})
            body = r.json()
            say(f"{km:>5} km -> {r.status_code} fare={body['fare']} Location={r.headers['location']}")
            check(f"{km} km -> 201, fare {expected}", r.status_code == 201 and body["fare"] == expected)
        r = await c.post("/v1/quotes", json={"distance_km": "2.5"})
        say("\nbody đầy đủ của một báo giá:")
        say(r.text)
        body = r.json()
        remain = (datetime.fromisoformat(body["expires_at"]) - datetime.now(UTC)).total_seconds()
        check("service mặc định ride, currency VND", body["service"] == "ride" and body["currency"] == "VND")
        check("fare và distance_km là chuỗi JSON", isinstance(body["fare"], str) and isinstance(body["distance_km"], str))
        check("expires_at còn trong (0, 300] giây", 0 < remain <= 300)
        check("content-type application/json", r.headers["content-type"] == "application/json")

        r = await c.post("/v1/quotes", json={"distance_km": 2.5})
        check("số JSON 2.5 cũng được nhận, ra cùng cước", r.status_code == 201 and r.json()["fare"] == "14250")

        r = await c.post("/v1/quotes", json={"distance_km": "1e2"})
        say("\nký pháp khoa học: \"1e2\" ->", r.status_code, "distance_km", r.json()["distance_km"], "fare", r.json()["fare"])
        check('"1e2" nhận được, trả distance_km "100" (không phải "1E+2"), fare 453000', r.status_code == 201 and r.json()["distance_km"] == "100" and r.json()["fare"] == "453000")
        r = await c.post("/v1/quotes", json={"distance_km": "\u0662"})  # chữ số Ả Rập-Ấn Độ ٢
        say("chữ số Unicode: \"\u0662\" ->", r.status_code, "distance_km", r.json()["distance_km"])
        check("GIỚI HẠN ĐÃ BIẾT: chữ số Unicode '\u0662' vẫn được nhận thành 2 km", r.status_code == 201 and r.json()["distance_km"] == "2")

        loc = r.headers["location"]
        r = await c.get(loc)
        say(f"\nGET {loc} -> {r.status_code}")
        check("GET theo Location trả 200", r.status_code == 200 and r.json()["quote_id"] == loc.rsplit("/", 1)[-1])
        r = await c.get("/v1/quotes/QT-9999")
        say("GET /v1/quotes/QT-9999 ->", r.status_code, r.text)
        check("GET id lạ -> 404", r.status_code == 404)

    # `uvicorn --workers 2` là hai tiến trình, mỗi tiến trình một dict VÀ một bộ đếm. Mô phỏng bằng hai app.
    app_a, app_b = create_app(), create_app()
    async with client_for(app_a) as ca, client_for(app_b) as cb:
        post_a = await ca.post("/v1/quotes", json={"distance_km": "6"})
        post_b = await cb.post("/v1/quotes", json={"distance_km": "5"})
        loc = post_a.headers["location"]
        on_a, on_b = await ca.get(loc), await cb.get(loc)
        only_b = await cb.post("/v1/quotes", json={"distance_km": "3"})
        not_on_a = await ca.get(only_b.headers["location"])
    say(f"\nhai worker: cả hai cấp {post_a.json()['quote_id']} và {post_b.json()['quote_id']} cho hai báo giá khác nhau")
    say(f"  GET {loc} ở worker A -> {on_a.json()['distance_km']} km, fare {on_a.json()['fare']}")
    say(f"  GET {loc} ở worker B -> {on_b.json()['distance_km']} km, fare {on_b.json()['fare']}")
    say(f"  báo giá {only_b.json()['quote_id']} chỉ có ở B: hỏi A -> {not_on_a.status_code}")
    check("hai worker cấp TRÙNG mã QT-0001", post_a.json()["quote_id"] == post_b.json()["quote_id"] == "QT-0001")
    check("trùng mã: cùng GET QT-0001 ra hai báo giá khác nhau (sai dữ liệu, không lỗi)", on_a.json()["fare"] != on_b.json()["fare"])
    check("báo giá tạo ở một worker, worker kia trả 404", not_on_a.status_code == 404)


# --- 3. Bốn loại lỗi ----------------------------------------------------------------------
async def demo_errors() -> None:
    say("\n== 3. Lỗi: 400, 422, 404, 405 ==")
    app = create_app()
    cases = [
        ("miền: 150 km", dict(json={"distance_km": "150"}), 400),
        ("miền: -1 km", dict(json={"distance_km": "-1"}), 400),
        ("hình dạng: 2.1234 km", dict(json={"distance_km": "2.1234"}), 422),
        ("hình dạng: thiếu distance_km", dict(json={}), 422),
        ("hình dạng: service lạ", dict(json={"distance_km": "2", "service": "bike"}), 422),
        ("hình dạng: NaN", dict(json={"distance_km": "NaN"}), 422),
        ("JSON hỏng", dict(content="{bad", headers={"content-type": "application/json"}), 422),
    ]
    async with client_for(app) as c:
        for name, kwargs, want in cases:
            r = await c.post("/v1/quotes", **kwargs)
            say(f"{name:<32} -> {r.status_code} {r.text}")
            check(f"{name} -> {want}", r.status_code == want)
        r = await c.get("/nope")
        say(f"{'route không có':<32} -> {r.status_code} {r.text}")
        check("route không có -> 404", r.status_code == 404)
        r = await c.put("/v1/quotes")
        say(f"{'PUT /v1/quotes':<32} -> {r.status_code} {r.text} Allow={r.headers['allow']}")
        check("PUT -> 405 kèm Allow: POST", r.status_code == 405 and r.headers["allow"] == "POST")


# --- 4. OpenAPI và /docs ------------------------------------------------------------------
async def demo_openapi() -> None:
    say("\n== 4. OpenAPI ==")
    app = create_app()
    async with client_for(app) as c:
        spec = (await c.get("/openapi.json")).json()
        docs = await c.get("/docs")
        redoc = await c.get("/redoc")
    post = spec["paths"]["/v1/quotes"]["post"]
    say("openapi:", spec["openapi"], "| title:", spec["info"]["title"])
    say("paths:", list(spec["paths"]))
    say("POST /v1/quotes responses:", list(post["responses"]))
    say("operationId:", post["operationId"])
    say("schemas:", sorted(spec["components"]["schemas"]))
    say("/docs:", docs.status_code, docs.headers["content-type"], "| /redoc:", redoc.status_code)
    check("openapi 3.x", spec["openapi"].startswith("3."))
    check("hai route /v1/quotes và /v1/quotes/{quote_id}", set(spec["paths"]) == {"/v1/quotes", "/v1/quotes/{quote_id}"})
    check("POST khai báo 201 (không phải 200)", "201" in post["responses"] and "200" not in post["responses"])
    check("/docs và /redoc trả HTML", docs.status_code == 200 and "text/html" in docs.headers["content-type"] and redoc.status_code == 200)


# --- 5. async def hay def: ai chạy ở đâu --------------------------------------------------
def make_probe_app() -> FastAPI:
    probe = FastAPI()

    @probe.get("/async-block")
    async def async_block() -> dict:
        time.sleep(0.2)  # SAI có chủ ý: chặn nguyên event loop
        return {"thread": threading.current_thread().name}

    @probe.get("/async-await")
    async def async_await() -> dict:
        await asyncio.sleep(0.2)  # nhường event loop trong lúc chờ
        return {"thread": threading.current_thread().name}

    @probe.get("/sync-def")
    def sync_def() -> dict:
        time.sleep(0.2)  # chặn, nhưng chỉ chặn một luồng trong thread pool
        return {"thread": threading.current_thread().name}

    return probe


async def timed_burst(path: str, n: int = 5) -> tuple[float, set[str]]:
    async with client_for(make_probe_app()) as c:
        start = time.perf_counter()
        responses = await asyncio.gather(*(c.get(path) for _ in range(n)))
        elapsed = time.perf_counter() - start
    return elapsed, {r.json()["thread"] for r in responses}


async def demo_threads() -> None:
    load1, load5, load15 = os.getloadavg()
    say("\n== 5. 5 request đồng thời, mỗi handler ngủ 0,2 giây ==")
    say(f"load average lúc đo (1/5/15 phút): {load1:.2f} {load5:.2f} {load15:.2f}")
    measured = {}
    for path in ("/async-block", "/async-await", "/sync-def"):
        elapsed, threads = await timed_burst(path)
        measured[path] = (elapsed, threads)
        say(f"{path:<14} {elapsed:5.2f} s  luồng: {sorted(threads)}")
    block, await_, sync = (measured[p][0] for p in ("/async-block", "/async-await", "/sync-def"))
    check("async def + time.sleep: tuần tự, >= 1,0 s", block >= 1.0)
    check("async def + await sleep: đồng thời, nhanh hơn chặn ít nhất 2 lần", block > 2 * await_)
    check("def thường: đồng thời nhờ thread pool, nhanh hơn chặn ít nhất 2 lần", block > 2 * sync)
    check("async def chạy ở luồng của event loop", measured["/async-block"][1] == {"MainThread"})
    check("def thường chạy ở luồng khác", "MainThread" not in measured["/sync-def"][1])


async def main() -> int:
    for step in (demo_raw, demo_quotes, demo_errors, demo_openapi, demo_threads):
        try:
            await step()
        except Exception as exc:  # --verify phải báo SAI/FAIL có tên, không được chết bằng traceback
            if not VERIFY:
                raise
            check(f"{step.__name__} dừng giữa chừng: {type(exc).__name__}: {exc}", False)
    failed = [name for name, ok in results if not ok]
    if VERIFY:
        for name in failed:
            print(f"SAI: {name}")
        if failed:
            print(f"FAIL {len(results) - len(failed)}/{len(results)}")
            return 1
        print(f"OK {len(results)}/{len(results)}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))

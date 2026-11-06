"""Bài 32: validation, lỗi có cấu trúc (Problem Details, RFC 9457) và hợp đồng OpenAPI.

Chạy:     uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details examples/toi-noi-python/32-fastapi-validation-problem-details/main.py
Verify:   thêm --verify ở cuối lệnh trên.
Snapshot: thêm --write-snapshot để ghi lại openapi.snapshot.json sau khi CỐ Ý đổi hợp đồng.

Mọi lời gọi đi qua httpx + ASGITransport: app chạy trong tiến trình này, không cần server.
"""

import asyncio
import difflib
import json
import logging
import sys
from pathlib import Path

import httpx
from fastapi import FastAPI

from app import problems
from app.main import create_app
from app.schemas import ProblemDetails, QuoteRequest

SNAPSHOT = Path(__file__).with_name("openapi.snapshot.json")
VERIFY = "--verify" in sys.argv
results: list[tuple[str, bool]] = []


def say(*parts: object) -> None:
    if not VERIFY:
        print(*parts)


def check(name: str, ok: bool) -> None:
    results.append((name, bool(ok)))


def client_for(app) -> httpx.AsyncClient:
    # raise_app_exceptions=False: exception không bắt được trở thành response 500 như server thật
    transport = httpx.ASGITransport(app=app, raise_app_exceptions=False)
    return httpx.AsyncClient(transport=transport, base_url="http://tinoi.test")


class ListHandler(logging.Handler):
    """Gom log record để kiểm tra lỗi 500 có vào log hay không."""

    def __init__(self) -> None:
        super().__init__()
        self.records: list[logging.LogRecord] = []

    def emit(self, record: logging.LogRecord) -> None:
        self.records.append(record)


BAD_REQUESTS = [
    ("thiếu distance_km", dict(json={})),
    ("distance_km = abc", dict(json={"distance_km": "abc"})),
    ("distance_km = 2.1234", dict(json={"distance_km": "2.1234"})),
    ("service = bike", dict(json={"distance_km": "2", "service": "bike"})),
    ("trường lạ", dict(json={"distance_km": "2", "x": 1})),
    ("JSON hỏng", dict(content="{bad", headers={"content-type": "application/json"})),
    ("nhiều lỗi cùng lúc", dict(json={"distance_km": "2.1234", "service": "bike", "x": 1})),
]


# --- 1. Trước: 422 mặc định của FastAPI --------------------------------------------------
async def demo_default_422() -> None:
    say("== 1. 422 mặc định của FastAPI ==")
    async with client_for(create_app(problem_details=False)) as c:
        r = await c.post("/v1/quotes", json={"distance_km": "2.1234", "service": "bike"})
    say(r.status_code, r.headers["content-type"])
    say(json.dumps(r.json(), ensure_ascii=False, indent=2))
    check("422 mặc định: content-type application/json", r.headers["content-type"] == "application/json")
    check("422 mặc định: detail là list có loc/msg/type/input", set(r.json()["detail"][0]) >= {"loc", "msg", "type", "input"})
    check("422 mặc định: echo lại input của client", r.json()["detail"][0]["input"] == "2.1234")
    check("422 mặc định: không có type/title/status", "title" not in r.json() and "status" not in r.json())


# --- 2. Sau: Problem Details cho lỗi validation -----------------------------------------
async def demo_validation() -> None:
    say("\n== 2. 422 dạng Problem Details ==")
    async with client_for(create_app()) as c:
        for name, kwargs in BAD_REQUESTS:
            r = await c.post("/v1/quotes", **kwargs)
            body = r.json()
            say(f"-- {name}: {r.status_code} {r.headers['content-type']}")
            say(json.dumps(body, ensure_ascii=False))
            check(f"{name}: 422 problem+json", r.status_code == 422 and r.headers["content-type"] == "application/problem+json")
            check(f"{name}: có type/title/status/instance/errors", {"type", "title", "status", "instance", "errors"} <= set(body))
            check(f"{name}: không echo input", "input" not in json.dumps(body))
        r = await c.post("/v1/quotes", json={"distance_km": "2.1234", "service": "bike", "x": 1})
        codes = [(e["field"], e["code"]) for e in r.json()["errors"]]
        say("\nba lỗi cùng lúc, theo (field, code):", codes)
        check("ba lỗi cùng lúc được báo đủ", codes == [("body.distance_km", "decimal_max_places"), ("body.service", "enum"), ("body.x", "extra_forbidden")])
        r = await c.post("/v1/quotes", content="{bad", headers={"content-type": "application/json"})
        check("JSON hỏng: field là 'body', không phải vị trí ký tự", r.json()["errors"][0]["field"] == "body")


# --- 3. Lỗi miền, không tìm thấy, lỗi framework, lỗi bất ngờ ----------------------------
async def demo_domain_and_others() -> None:
    say("\n== 3. 400, 404, 405, 500 ==")
    app = create_app()

    async def boom() -> dict:
        raise ValueError("bug-bi-mat: lỗi lập trình, không phải lỗi người dùng")

    app.add_api_route("/v1/_boom", boom, include_in_schema=False)
    handler = ListHandler()
    logging.getLogger("toinoi.problems").addHandler(handler)
    logging.getLogger("toinoi.problems").propagate = False  # demo: không in traceback ra terminal
    try:
        async with client_for(app) as c:
            r400 = await c.post("/v1/quotes", json={"distance_km": "150"})
            r404 = await c.get("/v1/quotes/QT-9999")
            r404r = await c.get("/nope")
            r405 = await c.put("/v1/quotes")
            r500 = await c.get("/v1/_boom")
            r201 = await c.post("/v1/quotes", json={"distance_km": "2.5"})
            r_sci = await c.post("/v1/quotes", json={"distance_km": "1e2"})
    finally:
        logging.getLogger("toinoi.problems").removeHandler(handler)
    for name, r in [("150 km (miền)", r400), ("QT-9999", r404), ("/nope", r404r), ("PUT /v1/quotes", r405), ("ValueError ở chỗ khác", r500)]:
        say(f"-- {name}: {r.status_code} {r.headers['content-type']}")
        say(json.dumps(r.json(), ensure_ascii=False))
    p400 = r400.json()
    check("400 miền: problem+json, type distance-out-of-range", r400.status_code == 400 and p400["type"] == "https://toinoi.example/problems/distance-out-of-range")
    check("400 miền: extension min_km/max_km là chuỗi", (p400["min_km"], p400["max_km"]) == ("0", "100"))
    check("400 miền: instance là đường dẫn request", p400["instance"] == "/v1/quotes")
    check("404 id lạ: type quote-not-found", r404.status_code == 404 and r404.json()["type"].endswith("/quote-not-found"))
    check("404 route lạ: about:blank, title Not Found, không lặp detail", r404r.status_code == 404 and r404r.json()["type"] == "about:blank" and r404r.json()["title"] == "Not Found" and "detail" not in r404r.json())
    check("405: giữ header Allow", r405.status_code == 405 and r405.headers["allow"] == "POST" and r405.json()["title"] == "Method Not Allowed")
    check("500: ValueError ngoài miền KHÔNG bị nhầm thành 400", r500.status_code == 500 and r500.json()["type"].endswith("/internal-error"))
    check("500: không lộ nội dung exception ra client", "bug-bi-mat" not in r500.text and "ValueError" not in r500.text)
    check("500: exception vào log kèm traceback", len(handler.records) == 1 and handler.records[0].exc_info is not None and "bug-bi-mat" in str(handler.records[0].exc_info[1]))
    check("201 vẫn là application/json, không phải problem+json", r201.status_code == 201 and r201.headers["content-type"] == "application/json")
    check('"1e2" nhận được, phản hồi distance_km là "100" (không phải "1E+2")', r_sci.status_code == 201 and r_sci.json()["distance_km"] == "100")


# --- 4. OpenAPI: hợp đồng khớp với hành vi ------------------------------------------------
async def demo_openapi() -> None:
    say("\n== 4. OpenAPI ==")
    app = create_app()
    spec = app.openapi()
    post = spec["paths"]["/v1/quotes"]["post"]
    get = spec["paths"]["/v1/quotes/{quote_id}"]["get"]
    say("info.version:", spec["info"]["version"], "| tags:", [t["name"] for t in spec["tags"]])
    say("operationId:", post["operationId"], get["operationId"])
    for name, op in (("POST /v1/quotes", post), ("GET /v1/quotes/{quote_id}", get)):
        say(f"{name}:")
        for status, resp in op["responses"].items():
            ref = next(iter(resp["content"].values()))["schema"].get("$ref", "(inline)").rsplit("/", 1)[-1]
            say(f"  {status} {list(resp['content'])[0]:<26} {ref}")
    say("schemas:", sorted(spec["components"]["schemas"]))
    check("operationId đặt tay", (post["operationId"], get["operationId"]) == ("createQuote", "getQuote"))
    check("POST khai báo 201, 400, 422, 500", set(post["responses"]) == {"201", "400", "422", "500"})
    check("GET khai báo 200, 404, 422, 500", set(get["responses"]) == {"200", "404", "422", "500"})
    check("mọi lỗi khai báo application/problem+json", all(list(post["responses"][s]["content"]) == ["application/problem+json"] for s in ("400", "422", "500")))
    check("HTTPValidationError không còn trong OpenAPI", "HTTPValidationError" not in spec["components"]["schemas"])
    check("thành công khai báo application/json", list(post["responses"].get("201", {}).get("content", {})) == ["application/json"])
    check("tag có mô tả", spec["tags"][0]["name"] == "quotes" and spec["tags"][0]["description"])
    check("info.version là 1.0.0", spec["info"]["version"] == "1.0.0")
    paths = set(spec["paths"])
    check("mọi route nằm dưới /v1", all(p.startswith("/v1/") for p in paths))

    # Đối chiếu hành vi với đặc tả: mỗi phản hồi quan sát được phải nằm trong những gì đã khai báo
    async with client_for(app) as c:
        observed = [
            (post, await c.post("/v1/quotes", json={"distance_km": "2.5"})),
            (post, await c.post("/v1/quotes", json={"distance_km": "150"})),
            (post, await c.post("/v1/quotes", json={})),
            (get, await c.get("/v1/quotes/QT-0001")),
            (get, await c.get("/v1/quotes/QT-9999")),
        ]
    mismatches = []
    for operation, r in observed:
        declared = operation["responses"].get(str(r.status_code))
        media = r.headers["content-type"].split(";")[0]
        if declared is None or media not in declared["content"]:
            mismatches.append((r.request.url.path, r.status_code, media))
        elif r.status_code >= 400:
            ProblemDetails.model_validate(r.json())  # body lỗi đúng schema ProblemDetails
    say("\nđối chiếu 5 phản hồi thật với đặc tả: lệch", len(mismatches), "ca")
    check("mọi phản hồi quan sát được (201, 400, 422, 200, 404) khớp status và media type đã khai báo", not mismatches)

    # Cái bẫy: đã đăng ký handler problem+json nhưng quên khai báo responses -> tài liệu nói dối.
    lying = FastAPI()
    problems.install(lying)

    @lying.post("/q")
    async def q(body: QuoteRequest) -> dict:
        return {}

    async with client_for(lying) as c:
        runtime = await c.post("/q", json={})
    declared = lying.openapi()["paths"]["/q"]["post"]["responses"]["422"]["content"]
    say("\nquên khai báo responses:")
    say("  runtime  :", runtime.headers["content-type"])
    say("  OpenAPI  :", {k: v["schema"]["$ref"].rsplit("/", 1)[-1] for k, v in declared.items()})
    check("quên responses: runtime problem+json nhưng OpenAPI vẫn khai HTTPValidationError", runtime.headers["content-type"] == "application/problem+json" and list(declared) == ["application/json"] and declared["application/json"]["schema"]["$ref"].endswith("HTTPValidationError"))


# --- 5. Snapshot: đổi hợp đồng phải hiện thành diff --------------------------------------
def to_text(spec: dict) -> str:
    return json.dumps(spec, indent=2, ensure_ascii=False) + "\n"


async def demo_snapshot() -> None:
    say("\n== 5. Snapshot OpenAPI ==")
    live = create_app().openapi()
    if "--write-snapshot" in sys.argv:
        SNAPSHOT.write_text(to_text(live), encoding="utf-8")
        say("đã ghi", SNAPSHOT.name)
    saved = json.loads(SNAPSHOT.read_text(encoding="utf-8"))
    say(f"{SNAPSHOT.name}: {len(SNAPSHOT.read_text(encoding='utf-8').splitlines())} dòng; app hiện tại khớp snapshot: {live == saved}")
    check("openapi của app khớp openapi.snapshot.json", live == saved)

    # Giả lập một thay đổi: ai đó đổi POST từ 201 sang 200. Snapshot phải báo ngay.
    changed = json.loads(to_text(live))
    post = changed["paths"]["/v1/quotes"]["post"]
    post["responses"] = {("200" if k == "201" else k): v for k, v in post["responses"].items()}
    diff = [
        line
        for line in difflib.unified_diff(to_text(saved).splitlines(), to_text(changed).splitlines(), "snapshot", "sau-khi-đổi", n=0, lineterm="")
        if not line.startswith(("---", "+++", "@@"))
    ]
    say("diff khi đổi 201 thành 200:")
    for line in diff:
        say("  " + line)
    check("đổi 201 -> 200 làm snapshot lệch (đúng một dòng khác)", changed != saved and len(diff) == 2)


async def main() -> int:
    for step in (demo_default_422, demo_validation, demo_domain_and_others, demo_openapi, demo_snapshot):
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

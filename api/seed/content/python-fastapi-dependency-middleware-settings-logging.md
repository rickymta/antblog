---
slug: python-fastapi-dependency-middleware-settings-logging
title: "Dependency, middleware, cấu hình và structured logging"
summary: "Bài 33: dựng khung vận hành cho FastAPI: Depends có cache và yield, lifespan, middleware gán request id, pydantic-settings fail fast, log JSON mang correlation id nhờ contextvars. Có lab chạy thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, fastapi, dependency-injection, logging]
language: vi
---

Một khách của [Tới Nơi](/articles/du-an-mau-toi-noi) nhắn cho bộ phận hỗ trợ: «19 giờ tối qua tôi bấm báo giá, ứng dụng quay mãi rồi báo lỗi». Thứ duy nhất hỗ trợ cầm được là mã yêu cầu hiện trên màn hình lỗi. Giờ cao điểm có khoảng 70 chuyến mỗi giây; nếu không có một mã đi xuyên suốt, dòng log «mở phiên kho chuyến» và dòng «lỗi» không ai nối lại được. Bài này dựng khung đó quanh API báo giá: cấu hình kiểm ngay lúc khởi động, thứ handler cần được đưa vào qua `Depends`, một middleware gán mã cho mọi request, và log JSON mà dòng nào cũng mang mã ấy.

Đây là bài 33 của lộ trình **Python trung bình qua Tới Nơi**. Bạn nên đọc [bài 31](/articles/python-fastapi-http-endpoint-bao-gia) (endpoint báo giá, uvicorn), [bài 15](/articles/python-pydantic-validation-settings) (`BaseSettings`) và [bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation) (`async`/`await`). Lab của bài này tự chứa: nó mang theo hợp đồng `POST /v1/quotes` và Problem Details của [bài 32](/articles/python-fastapi-validation-problem-details-openapi) (201 kèm `Location` và `quote_id`, lỗi `application/problem+json`) và không cần chạy lab 31 hay 32.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một handler khai ba tham số: <code>a</code> và <code>b</code> đều là <code>Depends(get_session)</code>, còn <code>c</code> là <code>Depends(get_session, use_cache=False)</code>, cả ba cùng scope. Hàm <code>get_session</code> mở một phiên kho chuyến mới mỗi lần được gọi. Một request làm mở bao nhiêu phiên? Và nếu bỏ <code>use_cache=False</code>?</p>
<details><summary>Xem đáp án</summary><p><b>Hai phiên</b> với <code>use_cache=False</code>, và <b>một phiên</b> nếu bỏ nó. FastAPI nhớ kết quả của một dependency trong phạm vi một request, nên <code>a</code> và <code>b</code> là cùng một đối tượng; <code>use_cache=False</code> buộc <code>c</code> gọi lại <code>get_session</code>. Bài chạy cả hai ca, và thêm một bẫy: cùng hàm nhưng <b>khác scope</b> thì cache không dùng chung.</p></details>
</div>

## Một request đi qua những lớp nào

Ứng dụng FastAPI là một ứng dụng ASGI (bài 31) bọc trong vài lớp: request qua middleware, router khớp route, FastAPI dựng dependency của route, handler chạy, rồi response đi ngược lại. Ngoài chuỗi này có hai việc chỉ chạy một lần: đọc cấu hình và `lifespan`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Request đi từ client qua middleware, router, dependency tới handler; response quay ngược qua middleware, nơi ghi dòng log request.completed. Bên dưới, cấu hình Settings và lifespan chạy một lần ngoài chuỗi request."><defs><marker id="py33-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="50" width="110" height="90" rx="10" class="f-bg s-line"/><text x="36" y="86" class="t-b">Client</text><text x="36" y="110" class="t-small">curl, ứng dụng</text><path d="M130 95 L150 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py33-0-arrow)"/><rect x="150" y="50" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="166" y="86" class="t-b">Middleware</text><text x="166" y="110" class="t-small">gán request_id, đo giờ</text><path d="M360 95 L380 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py33-0-arrow)"/><rect x="380" y="50" width="120" height="90" rx="10" class="f-pk-soft s-pk"/><text x="396" y="86" class="t-b">Router</text><text x="396" y="110" class="t-small">khớp route</text><path d="M500 95 L520 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py33-0-arrow)"/><rect x="520" y="50" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="536" y="86" class="t-b">Depends</text><text x="536" y="110" class="t-small">Session, Customer</text><path d="M700 95 L720 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py33-0-arrow)"/><rect x="720" y="50" width="140" height="90" rx="10" class="f-pk-soft s-pk"/><text x="736" y="86" class="t-b">Handler</text><text x="736" y="110" class="t-mono">get_trip</text><path d="M790 140 L790 172 L255 172 L255 142" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py33-0-arrow)"/><text x="290" y="192" class="t-small">response đi ngược: middleware ghi request.completed, thêm X-Request-Id</text><text x="20" y="226" class="t-small">Chạy một lần, ngoài chuỗi request</text><rect x="20" y="236" width="410" height="52" rx="10" class="f-phys-soft s-phys"/><text x="36" y="257" class="t-b">Settings</text><text x="36" y="276" class="t-small">đọc TOINOI_* ở create_app, sai thì service không lên</text><rect x="450" y="236" width="410" height="52" rx="10" class="f-phys-soft s-phys"/><text x="466" y="257" class="t-b">lifespan</text><text x="466" y="276" class="t-small">mở Database khi lên, đóng khi tắt, log ngoài request</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một request qua khung FastAPI. Nét đứt là chiều response; hai khối dưới chạy một lần, không thuộc request nào.</figcaption>
</figure>

Quy tắc chọn chỗ đặt code: việc áp cho mọi request mà handler không cần biết (mã yêu cầu, đo giờ, một dòng log) thuộc **middleware**; thứ riêng handler cần để làm việc (phiên kho chuyến, cấu hình, khách hiện tại) thuộc **`Depends`**. Phía C# xem [bài middleware, DI, options và logging](/articles/dotnet10-middleware-di-options-logging); khác biệt là `Depends` không có container đăng ký trước. «Cache trong một request» gần với scoped, `app.state` gần với singleton.

Lab (`examples/toi-noi-python/33-fastapi-dependency-middleware-logging/`) gồm `app/settings.py`, `logs.py`, `middleware.py`, `deps.py`, `routes.py`, `schemas.py` và `problems.py` (copy từ bài 32, thêm `request_id`), `app/main.py`; `main.py` ở gốc lab là kịch bản chạy và `--verify`.

## Cấu hình: đọc một lần, kiểm ngay

Cấu hình cũng là dữ liệu từ ngoài, nên dùng cơ chế của bài 15. Lớp `Settings` của lab (trích `app/settings.py`):

```python
class Settings(BaseSettings):
    # Trường quote_ttl_seconds đọc từ TOINOI_QUOTE_TTL_SECONDS; biến lạ như TOINOI_PG_URL bị bỏ qua.
    model_config = SettingsConfigDict(env_prefix=ENV_PREFIX, extra="ignore")

    service_name: str = "quote-api"
    environment: Literal["dev", "prod"] = "dev"
    log_level: Literal["DEBUG", "INFO", "WARNING", "ERROR"] = "INFO"
    quote_ttl_seconds: int = Field(300, ge=30, le=3600)
    # Request chậm hơn ngưỡng này được log mức WARNING thay vì INFO.
    slow_request_ms: int = Field(500, ge=1)
```

`create_app()` gọi `Settings()` đúng một lần, ở dòng đầu tiên, rồi cất vào `app.state.settings`. Dependency `get_settings` chỉ đọc lại từ đó:

```python
def get_settings(request: Request) -> Settings:
    return request.app.state.settings
```

Tài liệu FastAPI hay dùng `@lru_cache` bọc `get_settings()`, nhưng test muốn đổi cấu hình phải sửa env rồi `cache_clear()`. Lab truyền `Settings` vào `create_app(settings)` để test dựng app với cấu hình tường minh. Demo cấu hình (trích output của `main.py`):

```text
mặc định : service_name='quote-api' environment='dev' log_level='INFO' quote_ttl_seconds=300 slow_request_ms=500
từ env   : log_level=DEBUG quote_ttl_seconds=600 (TOINOI_PG_URL bị bỏ qua)
cấu hình sai, service không lên:
  TOINOI_ENVIRONMENT: Input should be 'dev' or 'prod' (nhận 'staging')
  TOINOI_LOG_LEVEL: Input should be 'DEBUG', 'INFO', 'WARNING' or 'ERROR' (nhận 'LOUD')
  TOINOI_QUOTE_TTL_SECONDS: Input should be greater than or equal to 30 (nhận '5')
```

Với server thật, `TOINOI_QUOTE_TTL_SECONDS=5 uv run … uvicorn app.main:app …` dừng ngay lúc import `app.main` (trích dòng cuối):

```text
pydantic_core._pydantic_core.ValidationError: 1 validation error for Settings
quote_ttl_seconds
  Input should be greater than or equal to 30 [type=greater_than_equal, input_value='5', input_type=str]
```

Chết ngay khi khởi động tốt hơn chạy với TTL báo giá 5 giây. Lab chưa có bí mật nào. Khi có (bài 34 có khoá ký), `SecretStr` che giá trị trong `repr` và log, nhưng **không** che trong thông báo `ValidationError` khi cấu hình sai; bài 34 xử lý chỗ đó.

## Depends: cache, yield và scope

Handler `get_trip` (trích `app/routes.py`) khai ba thứ nó cần và không biết chúng đến từ đâu:

```python
@router.get("/trips/{trip_id}")
async def get_trip(trip_id: str, customer: CustomerDep, session: SessionDep) -> TripResponse:
    trip = session.get_trip(trip_id)
    # Chuyến của khách khác trả 404 giống chuyến không tồn tại: không lộ chuyến nào có thật.
    if trip is None or trip.customer_id != customer:
        raise HTTPException(status_code=404, detail="không tìm thấy chuyến")
```

`CustomerDep` và `SessionDep` là bí danh `Annotated[…, Depends(…)]` ở `app/deps.py`. Handler chỉ phụ thuộc kiểu, nên nguồn của kiểu thay được bằng `app.dependency_overrides[utc_now] = lambda: fixed`; lab dùng đúng cách đó để kiểm `expires_at` của báo giá là `2026-01-01T00:05:00Z` (bài 36 đi sâu vào test).

**Cache theo request.** Câu hỏi đầu bài có đáp án qua ba route quan sát `/v1/diagnostics/...` (chỉ có khi `TOINOI_ENVIRONMENT=dev`). Cả ba đều khai tham số kiểu `SessionDep`; `session_nos` là số thứ tự phiên mà mỗi tham số nhận được:

```text
dependency-cache           : {"session_nos": [1, 1, 2], "distinct": 2, "opened_total": 2}
dependency-cache-control   : {"session_nos": [3, 3, 3], "distinct": 1, "opened_total": 3}
dependency-scope-mismatch  : {"session_nos": [4, 5], "distinct": 2, "opened_total": 5}
```

Dòng đầu là ca có `use_cache=False` ở tham số `c`: hai phiên. Dòng hai là đối chứng, cùng ba tham số nhưng không có `use_cache=False`: một phiên; thiếu ca này thì chưa chứng minh được `use_cache=False` tạo ra khác biệt. Dòng ba là **bẫy**: một tham số là `Depends(get_session, scope="function")`, tham số kia là `Depends(get_session)` (scope mặc định). Cùng hàm nhưng khác scope thì FastAPI không dùng chung kết quả, nên mở hai phiên mà không ai đặt `use_cache=False`. Vì vậy hãy dùng một bí danh duy nhất như `SessionDep`. Cache chỉ tồn tại trong một request; request sau dựng lại từ đầu. Mặc định có lợi vì nhiều dependency con cùng cần «khách hiện tại» thì chỉ kiểm một lần.

**Dependency có `yield`** là cách mở rồi dọn tài nguyên theo request (trích `app/deps.py`):

```python
async def get_session(request: Request) -> AsyncIterator[TripSession]:
    session = request.app.state.db.open_session()
    log.debug("db.session.opened", extra={"session_no": session.session_no})
    try:
        yield session
    finally:
        session.close()
        log.debug("db.session.closed", extra={"session_no": session.session_no})
```

`try/finally` là bắt buộc: nếu handler ném lỗi, exception được ném vào đúng chỗ `yield`, và đoạn dọn dẹp không có `finally` sẽ bị bỏ qua. Để chắc chắn lab thật sự kiểm điều này, ta bỏ `try/finally` trên một bản sao và chạy `--verify`; nó báo (trích, còn hai dòng `SAI` khác cùng nguyên nhân rồi `FAIL 53/56`):

```text
SAI: commit lỗi: phiên vẫn được đóng ở cả hai scope: nhận 1, kỳ vọng 0
SAI: 500: phiên dependency vẫn được đóng (try/finally): nhận 1, kỳ vọng 0
```

Mỗi lỗi 500 rò một phiên là kiểu sự cố làm cạn pool kết nối sau vài giờ.

**Scope quyết định phần dọn chạy lúc nào.** Mặc định (`scope="request"`), phần sau `yield` chạy **sau khi response đã gửi**. `Depends(…, scope="function")` cho nó chạy ngay khi handler trả về, **trước khi response được gửi**. Route `/v1/diagnostics/scope-order` khai một dependency mỗi loại và ghi log lúc vào, lúc ra:

```text
  event=dep.enter dep=request-scope
  event=dep.enter dep=function-scope
  event=handler.body
  event=dep.exit dep=function-scope
  event=request.completed
  event=dep.exit dep=request-scope
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Sáu sự kiện theo thứ tự thời gian. Dependency scope request mở đầu tiên và thoát cuối cùng, sau khi response bắt đầu gửi. Dependency scope function mở thứ hai và thoát ngay sau handler, trước khi response bắt đầu gửi."><defs><marker id="py33-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M40 128 L40 38" class="f-none s-line dash" stroke-width="1"/><path d="M190 128 L190 38" class="f-none s-line dash" stroke-width="1"/><path d="M340 128 L340 38" class="f-none s-line dash" stroke-width="1"/><path d="M490 128 L490 38" class="f-none s-line dash" stroke-width="1"/><path d="M640 128 L640 38" class="f-none s-warn dash" stroke-width="2"/><path d="M770 128 L770 38" class="f-none s-line dash" stroke-width="1"/><text x="580" y="26" class="t-small t-warn">response bắt đầu gửi</text><rect x="40" y="48" width="730" height="28" rx="6" class="f-pk-soft s-pk"/><text x="52" y="67" class="t-small">scope request (mặc định): sống tới khi response gửi xong</text><rect x="190" y="86" width="300" height="28" rx="6" class="f-phys-soft s-phys"/><text x="202" y="105" class="t-small">scope function: thoát ngay sau handler</text><text x="36" y="154" class="t-b">1</text><text x="50" y="154" class="t-mono">dep.enter</text><text x="50" y="170" class="t-mono">request-scope</text><text x="186" y="154" class="t-b">2</text><text x="200" y="154" class="t-mono">dep.enter</text><text x="200" y="170" class="t-mono">function-scope</text><text x="336" y="154" class="t-b">3</text><text x="350" y="154" class="t-mono">handler.body</text><text x="486" y="154" class="t-b">4</text><text x="500" y="154" class="t-mono">dep.exit</text><text x="500" y="170" class="t-mono">function-scope</text><text x="636" y="154" class="t-b">5</text><text x="650" y="154" class="t-mono">request.</text><text x="650" y="170" class="t-mono">completed</text><text x="766" y="154" class="t-b">6</text><text x="780" y="154" class="t-mono">dep.exit</text><text x="780" y="170" class="t-mono">request-scope</text><text x="20" y="214" class="t-small">Thời gian chảy từ trái sang phải. Dòng 4 nằm trước dòng 5, dòng 6 nằm sau: đó là khác biệt giữa hai scope.</text></svg>
</div>
<figcaption>Hình 2. Thứ tự vào và ra của hai dependency có yield, đọc từ log của lab. Dòng 5 là lúc middleware ghi xong request.completed.</figcaption>
</figure>

Hệ quả thực tế có hai. Một là phần dọn chạy trước response thì mới an toàn để đặt việc **commit** vào đó. Lab dựng hai route giả lập cùng một commit luôn thất bại, chỉ khác scope:

```text
commit lỗi, scope function: 500; scope request: 201
```

Với scope `function`, client thấy 500. Với scope `request`, response 201 đã đi rồi mới tới commit: khách tưởng thành công trong khi dữ liệu không được lưu, và lỗi **không để lại dòng nào** trong log JSON của lab (chỉ có `request.completed` 201). Lab đặt `SessionDep` là `scope="function"` vì lý do này. Hai là phần dọn chạy sau cùng không nằm trong thời gian middleware đo. Đổi lại, response stream dữ liệu có dùng dependency đó thì cần scope mặc định.

Còn `current_customer` trong `app/deps.py` là chỗ **tạm**: nó tin header `X-Customer-Id` do client gửi, nên không phải xác thực (thiếu hoặc sai thì 401 kèm `WWW-Authenticate: Bearer`). Giá trị của nó ở đây là cho thấy chỗ cắm: bài 34 đổi nguồn của dependency thành token đã ký, và handler chỉ bổ sung kiểm theo vai trò.

## Lifespan: việc chạy một lần

`lifespan` là một async context manager: phần trước `yield` chạy khi service lên, phần sau chạy khi service tắt. Đây là chỗ đúng để mở `Database` (ở lab là kho giả, ở bài 24 là engine SQLAlchemy) và gắn cấu hình log. Cách cũ `@app.on_event("startup")` đã bị FastAPI đánh dấu deprecated. Trích `create_app`:

```python
    @asynccontextmanager
    async def lifespan(app: FastAPI) -> AsyncIterator[None]:
        restore_logging = configure_logging(settings, log_stream)
        app.state.db = Database(seed_trips())
        log.info("service.started", extra={"environment": settings.environment,
                                           "quote_ttl_seconds": settings.quote_ttl_seconds})
        try:
            yield
        finally:
            log.info("service.stopping", extra={"open_sessions": app.state.db.open_now})
            app.state.db.close()
            restore_logging()
```

Hai dòng log vòng đời có `request_id` là `-` vì ngoài mọi request; `open_sessions` làm rò rỉ phiên hiện ngay lúc tắt. Gửi SIGINT (Ctrl+C) cho uvicorn đang chạy thật:

```text
INFO:     Shutting down
INFO:     Waiting for application shutdown.
{"ts": "2026-11-06T11:49:33.396+00:00", "level": "INFO", "logger": "toinoi.app", "service": "quote-api", "request_id": "-", "event": "service.stopping", "open_sessions": 0}
INFO:     Application shutdown complete.
```

Bẫy khi kiểm thử: `httpx.ASGITransport` không chạy lifespan, nên không có `app.state.db`; lab bọc client trong `async with app.router.lifespan_context(app)`. Lab dùng `httpx` trực tiếp vì `starlette.testclient.TestClient` ở bản này cảnh báo deprecation khi chạy trên `httpx`; bài 36 quay lại test.

## Middleware: request id, thời gian và lỗi

Middleware của lab (trích `app/middleware.py`) làm bốn việc: chọn request id, đặt vào contextvar, đo giờ, ghi một dòng log.

```python
    @app.middleware("http")
    async def request_context(request, call_next):
        request_id = pick_request_id(request.headers.get("x-request-id"))
        request.state.request_id = request_id  # exception handler (ngoài middleware) đọc từ đây
        token = request_id_var.set(request_id)
        started = perf_counter()
        try:
            try:
                response = await call_next(request)
            except Exception:
                access_log.exception("request.failed", extra=_fields(request, 500, started))
                raise
            fields = _fields(request, response.status_code, started)
            ...
            access_log.log(level, "request.completed", extra=fields)
            response.headers["X-Request-Id"] = request_id
            response.headers["Server-Timing"] = f"app;dur={fields['duration_ms']}"
            return response
        finally:
            request_id_var.reset(token)
```

**Đừng phản chiếu bừa header của client.** Gateway phía trước thường gửi sẵn `X-Request-Id` để cả chuỗi dịch vụ dùng chung một mã, nên ta nhận nó, nhưng chỉ khi khớp mẫu `[A-Za-z0-9._-]{8,64}`; không thì tự sinh `uuid4().hex`. Chuỗi 200 ký tự hay có dấu cách đưa thẳng vào log và header có thể chèn dòng log giả hoặc làm phình header. Kết quả với ba đầu vào:

```text
X-Request-Id hợp lệ  -> web-0001-abcdef
X-Request-Id 200 chữ -> 8323fbaa13214b30b55d282742bab06c
X-Request-Id có dấu cách -> 8b87b48c67c742e1b17936109b27fb11
```

`Server-Timing` là header chuẩn của W3C để công cụ trình duyệt hiển thị thời gian phía server. Với server thật, `curl -i` trên chuyến TN-0001 trả (trích):

```text
HTTP/1.1 200 OK
content-type: application/json
x-request-id: curl-demo-0001
server-timing: app;dur=5.78
```

**Đường lỗi.** Mọi lỗi của lab đi theo Problem Details của bài 32 (`app/problems.py`), chỉ thêm một thành viên `request_id` lấy từ `request.state`. Lỗi chưa bắt đi lên qua `call_next`; middleware ghi `request.failed` kèm traceback rồi ném lại. Response 500 không do middleware tạo ra mà do một lớp ngoài cùng của Starlette, nên header `X-Request-Id` của middleware không tới được nó; exception handler của `Exception` tự đặt header từ `request.state`. Câu cho client là câu chung, tên lớp và traceback chỉ ở log:

```text
boom: 500 application/problem+json X-Request-Id=boom-0001-xyz body={"type":"https://toinoi.example/problems/internal-error","title":"Lỗi nội bộ","status":500,"detail":"Có lỗi ở phía máy chủ; hãy thử lại sau, kèm mã yêu cầu khi liên hệ hỗ trợ.","instance":"/v1/diagnostics/boom","request_id":"boom-0001-xyz"}
request.failed: level=ERROR status=500 request_id=boom-0001-xyz
  exc (dòng cuối của traceback): RuntimeError: lỗi giả lập khi phiên 1 đang mở
```

Khách đọc mã `boom-0001-xyz` trên màn hình lỗi, hỗ trợ tìm đúng mã đó trong log. Dưới uvicorn, lớp ngoài cùng ném lại lỗi sau khi đã gửi 500, nên uvicorn tự in thêm `Exception in ASGI application`: một sự cố thành hai bản ghi, một JSON và một văn bản; hãy chọn giữ cái nào trước khi lên production. Lỗi 400, 422, 404 và 405 cũng mang `request_id` trùng header, kiểm bằng `--verify`.

**Giới hạn của `@app.middleware("http")`.** Decorator này dựng trên `BaseHTTPMiddleware`: `call_next` chạy phần còn lại của ứng dụng trong một task con, với bản sao của ngữ cảnh hiện tại. Giá trị `set` trước `call_next` được task con thừa hưởng; giá trị `set` trong task con ở lại đó. Tài liệu Starlette nêu giới hạn này và đề nghị pure ASGI middleware nếu cần. Lab chứng minh bằng một app riêng (output của `main.py`):

```text
middleware đọc lại sau call_next: ContextVar -> 'chưa đặt'; request.state -> 'đã đặt trong handler'
```

Do đó, muốn `request.completed` có `customer_id` mà dependency mới biết, ta ghi vào `request.state`, như `current_customer` làm. Thời gian đo cũng chỉ tới lúc response bắt đầu gửi, chưa gồm truyền thân response; đo đủ cần pure ASGI middleware, còn lab giữ dạng decorator cho dễ đọc.

## Structured logging và correlation id

**Structured logging** ghi mỗi dòng log là một object JSON, nên hệ thống thu log lọc theo `request_id`, `route` hay `status` mà không cần regex. Lab dùng `logging` chuẩn với formatter tự viết khoảng 20 dòng thay vì thêm `structlog` (trích `app/logs.py`):

```python
request_id_var: ContextVar[str] = ContextVar("request_id", default="-")

class RequestIdFilter(logging.Filter):
    def filter(self, record: logging.LogRecord) -> bool:
        record.request_id = request_id_var.get()
        return True
```

```python
        for key, value in record.__dict__.items():
            if key not in _STANDARD_ATTRS and key != "request_id":
                data[key] = value
        if record.exc_info:
            data["exc"] = self.formatException(record.exc_info)
        return json.dumps(data, ensure_ascii=False, default=str)
```

Thuộc tính `LogRecord` ngoài tập chuẩn là trường do `extra={...}` thêm vào, nên `extra={"trip_id": …}` sinh khoá `trip_id`. `ensure_ascii=False` giữ tiếng Việt; `default=str` để `Decimal`, `datetime` không làm vỡ dòng log.

**Correlation id đi bằng `contextvars`.** Ta không muốn truyền `request_id` qua tham số của mọi hàm. `ContextVar` là biến mà mỗi ngữ cảnh thực thi (mỗi task asyncio có một ngữ cảnh, sao chép từ nơi tạo task) có giá trị riêng. Middleware `set` ở đầu request và `reset` bằng token trong `finally`; filter đọc giá trị đó cho mọi dòng log. Cả bốn chỗ dưới đây đều thấy đúng mã (route `/v1/diagnostics/context`):

```text
{
  "in_handler": "ctx-demo-0001",
  "in_sync_dependency": "ctx-demo-0001",
  "in_thread": "ctx-demo-0001",
  "in_task": "ctx-demo-0001"
}
20 request song song: 20/20 task thấy đúng id của request mình
```

Dependency `def` thường chạy trong threadpool nhưng ngữ cảnh vẫn được sao chép; `asyncio.to_thread` và `asyncio.create_task` cũng vậy, và 20 request đồng thời không lẫn mã. Bẫy ở chỗ khác: `loop.run_in_executor(None, hàm)` **không** sao chép ngữ cảnh, nên log trong hàm đó mang `request_id` là `-` (bài tập bên dưới).

Một request đọc chuyến trong log (đủ bốn dòng, `TOINOI_LOG_LEVEL=DEBUG` để thấy hai dòng phiên):

```text
{"ts": "2026-11-06T11:50:20.064+00:00", "level": "DEBUG", "logger": "toinoi.deps", "service": "quote-api", "request_id": "web-0001-abcdef", "event": "db.session.opened", "session_no": 1}
{"ts": "2026-11-06T11:50:20.064+00:00", "level": "INFO", "logger": "toinoi.api", "service": "quote-api", "request_id": "web-0001-abcdef", "event": "trip.read", "trip_id": "TN-0001"}
{"ts": "2026-11-06T11:50:20.064+00:00", "level": "DEBUG", "logger": "toinoi.deps", "service": "quote-api", "request_id": "web-0001-abcdef", "event": "db.session.closed", "session_no": 1}
{"ts": "2026-11-06T11:50:20.064+00:00", "level": "INFO", "logger": "toinoi.access", "service": "quote-api", "request_id": "web-0001-abcdef", "event": "request.completed", "method": "GET", "route": "/v1/trips/{trip_id}", "status": 200, "duration_ms": 13.47, "customer_id": "CU-001"}
```

Vài quyết định nằm trong dòng cuối. `route` là **mẫu** `/v1/trips/{trip_id}`, không phải đường dẫn thật, nên số giá trị có hạn và làm nhãn dashboard được (404 không khớp route thì là `-`). Mức log thay đổi theo kết quả: `INFO` bình thường, `WARNING` khi chậm hơn `slow_request_ms`, `ERROR` khi 5xx. `ts` luôn UTC (bài 04). Và thứ **không** được ghi: header `Authorization`, thân request, số giấy tờ. Lab gửi một request có `Authorization: Bearer bi-mat-khong-duoc-log` rồi kiểm không dòng log nào chứa chuỗi đó.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Dòng log của request mang `request_id` là `-` | Log phát sinh trong thread tạo bằng `run_in_executor` hoặc executor tự dựng, không sao chép ngữ cảnh; dùng `asyncio.to_thread` |
| Phiên kho chuyến tăng dần, không về 0 | Dependency có `yield` đã bọc phần dọn trong `try/finally` chưa |
| Giá trị đặt bằng `ContextVar.set` trong handler không thấy ở `request.completed` | Middleware là `BaseHTTPMiddleware`; chuyển sang `request.state` hoặc pure ASGI middleware |
| Test báo `AttributeError` trên `app.state.db` | Client `ASGITransport` không chạy lifespan; bọc bằng `app.router.lifespan_context(app)` |
| Mỗi sự cố ra hai traceback | Uvicorn log thêm `Exception in ASGI application`; chọn giữ log nào |

## Chạy và kiểm chứng

Không cần hạ tầng. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/33-fastapi-dependency-middleware-logging examples/toi-noi-python/33-fastapi-dependency-middleware-logging/main.py
uv run --project examples/toi-noi-python/33-fastapi-dependency-middleware-logging examples/toi-noi-python/33-fastapi-dependency-middleware-logging/main.py --verify
```

Lệnh đầu in các khối output ở trên (trích từng phần); `--verify` chạy cùng kịch bản và so 56 kết quả kỳ vọng (cache và bẫy khác scope, thứ tự vào/ra, commit lỗi hai scope, Problem Details, request id, không rò rỉ `Authorization`, 20 request song song không lẫn mã, 500 vẫn đóng phiên, thứ tự lifespan, route quan sát biến mất ở `prod`):

```text
OK 56/56
```

Server thật: `TOINOI_LOG_LEVEL=DEBUG uv run --project examples/toi-noi-python/33-fastapi-dependency-middleware-logging uvicorn app.main:app --app-dir examples/toi-noi-python/33-fastapi-dependency-middleware-logging --port 8033 --no-access-log` (`--no-access-log` vì middleware đã ghi một dòng mỗi request), rồi `curl` như README. Môi trường: Python 3.14.7, FastAPI 0.162.0, Starlette 1.10.0, uvicorn 0.54.0, pydantic-settings 2.16.2, httpx 0.28.1 (`uv.lock`). Các mili giây trong log phụ thuộc máy; lần chạy cuối in `load average 89.80 / 58.69` (máy Apple M1 Max đang bận nhiều tiến trình khác), nên đừng so tuyệt đối. Thứ tự dòng, mã và số đếm lặp lại được.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Kho chuyến, kho báo giá và bộ đếm <code>quote_id</code> nằm trong bộ nhớ một tiến trình; chạy nhiều worker thì mỗi worker có bản riêng và có thể cùng sinh <code>QT-0001</code>. Log đi ra stdout, chưa có bộ thu gom hay khử thông tin cá nhân. <code>X-Request-Id</code> là quy ước tự đặt; truy vết xuyên service dùng W3C <code>traceparent</code> qua OpenTelemetry (bài 77). <code>X-Customer-Id</code> không phải xác thực. Route <code>/v1/diagnostics/*</code> chỉ nên có ở môi trường phát triển.</p>
</div>

**Bài tập:** (1) Trong `/v1/diagnostics/context`, thêm một giá trị đọc `request_id_var.get()` bên trong `await asyncio.get_running_loop().run_in_executor(None, request_id_var.get)`. Kết quả là gì và sửa thế nào? (2) Đổi `SessionDep` sang scope mặc định (bỏ `scope="function"`) rồi chạy lại `/v1/trips/TN-0001` với `TOINOI_LOG_LEVEL=DEBUG`. Dòng `db.session.closed` nằm ở đâu so với `request.completed`, và vì sao điều đó đáng quan tâm với commit?

<details><summary>Xem lời giải</summary><p>(1) Kết quả là <code>"-"</code>: <code>run_in_executor</code> không sao chép ngữ cảnh nên ContextVar về giá trị mặc định (đã chạy thử trên Python 3.14.7: <code>run_in_executor</code> trả <code>-</code>, <code>asyncio.to_thread</code> trả mã đã đặt). Sửa bằng <code>asyncio.to_thread(request_id_var.get)</code>, hoặc <code>run_in_executor(None, contextvars.copy_context().run, hàm)</code>. (2) <code>db.session.closed</code> chuyển xuống <b>sau</b> <code>request.completed</code>, giống dòng 6 ở Hình 2: phần dọn chạy khi response đã đi. Nếu đó là commit và commit lỗi, khách đã nhận thành công; scope <code>function</code> còn trả được 500.</p></details>

Bài trước: [bài 32](/articles/python-fastapi-validation-problem-details-openapi). Bài tiếp: [bài 34](/articles/python-fastapi-jwt-oauth2-authorization).

## Tài liệu gốc

- [FastAPI: Dependencies with yield (gồm `scope`)](https://fastapi.tiangolo.com/tutorial/dependencies/dependencies-with-yield/).
- [FastAPI: Lifespan Events](https://fastapi.tiangolo.com/advanced/events/).
- [FastAPI: Settings and Environment Variables](https://fastapi.tiangolo.com/advanced/settings/).
- [Starlette: Middleware (giới hạn của BaseHTTPMiddleware)](https://starlette.dev/middleware/).
- [Python: contextvars](https://docs.python.org/3.14/library/contextvars.html).
- [Python: Logging Cookbook](https://docs.python.org/3.14/howto/logging-cookbook.html).

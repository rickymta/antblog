---
slug: python-fastapi-http-endpoint-bao-gia
title: "HTTP và FastAPI: xuất bản endpoint báo giá"
summary: "Bài 31: đưa calculate_fare lên POST /v1/quotes bằng FastAPI và uvicorn: ASGI, model pydantic, 201/400/422, OpenAPI tự sinh, async def và def. Lab chạy thật, kiểm bằng httpx ngay trong tiến trình."
category: python
level: intermediate
tags: [python, toi-noi, fastapi, http, openapi]
language: vi
---

Ứng dụng khách của [Tới Nơi](/articles/du-an-mau-toi-noi) cho xem giá trước khi đặt xe: người dùng nhập điểm đến, màn hình hiện «14.250 VND». Con số do service `pricing` tính, và Dispatch (viết bằng C#) cũng gọi nó để kiểm tra giá trước khi ghép chuyến. Từ bài 01 ta có `calculate_fare` chạy trong terminal, không biết gì về mạng; muốn người khác gọi được, hàm cần một địa chỉ: `POST /v1/quotes`.

Bài này đưa hàm lên HTTP bằng **FastAPI** chạy trên **uvicorn**. Điều đáng hiểu là ba thứ giữa socket và hàm của ta: giao diện ASGI, kiểm tra body bằng pydantic, và cách chọn mã trạng thái. Đây là bài 31 của lộ trình **Python trung bình qua Tới Nơi**, mở đầu phần 04. Bài dùng `async`/`await` (bài 21) và model pydantic (bài 15). Lab không cần hạ tầng: phần lớn lời gọi đi qua `httpx` vào thẳng app trong tiến trình, chỉ phần `curl` chạy server thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một handler khai báo <code>async def</code> nhưng gọi <code>time.sleep(0.2)</code>. Năm request đến cùng lúc: tổng thời gian khoảng 0,2 giây, 1 giây hay 5 giây?</p>
<details><summary>Xem đáp án</summary><p><b>Khoảng 1 giây.</b> <code>async def</code> chạy thẳng trên event loop, và <code>time.sleep</code> giữ cả loop, nên năm handler xếp hàng nối đuôi nhau (5 × 0,2). Nếu đổi thành <code>def</code> thường, FastAPI chạy handler trong thread pool và cả năm xong sau cỡ 0,3 giây. Bài đo cả ba trường hợp ở mục «async def hay def».</p></details>
</div>

## Giữa socket và hàm của ta: WSGI và ASGI

HTTP là chữ chạy trên TCP. Ai đó phải đọc byte, tách method, đường dẫn, header, body, gọi code ứng dụng rồi viết response trở lại. Python chuẩn hóa chỗ nối «server» và «ứng dụng» bằng một giao diện để mỗi bên thay được độc lập. Có hai thế hệ:

| | WSGI (PEP 3333) | ASGI |
| --- | --- | --- |
| Dạng hàm | `def app(environ, start_response)` | `async def app(scope, receive, send)` |
| Request | dict `environ` có sẵn mọi thứ | `scope` (metadata) và `receive()` lấy body theo từng mảnh |
| Response | trả về iterable các `bytes` | gọi `send()` hai lần: bắt đầu response, rồi body |
| Chờ I/O | chặn luồng đang chạy | `await` nhường event loop |
| Giao thức | HTTP | HTTP, WebSocket, lifespan (khởi động, tắt) |
| Server ví dụ | gunicorn, waitress | uvicorn, hypercorn |

Lab có hai hàm viết tay, không framework nào, để thấy tận mặt (trích `app/raw.py`):

```python
# WSGI: đồng bộ, không có chỗ nào để `await`
def wsgi_app(environ: dict, start_response: Callable) -> Iterable[bytes]:
    payload = json.dumps({"method": environ["REQUEST_METHOD"], "path": environ["PATH_INFO"]}).encode()
    start_response("200 OK", [("Content-Type", "application/json")])
    return [payload]


# ASGI: coroutine với ba đối số
async def asgi_app(scope: dict, receive: Callable, send: Callable) -> None:
    if scope["type"] != "http":
        return  # lifespan và websocket cũng đi qua cổng này; hàm trần chỉ lo http
    body = b""
    while True:  # body đến theo từng mảnh: `more_body` cho biết còn nữa không
        message = await receive()
        body += message.get("body", b"")
        if not message.get("more_body"):
            break
    ...  # dựng payload rồi send({"type": "http.response.start", ...}) và send({"type": "http.response.body", ...})
```

Chạy hai hàm đó và hàm của FastAPI (phần đầu output của `main.py`):

```text
== 1. WSGI và ASGI trần ==
WSGI : 200 OK {"method": "GET", "path": "/hello"}
ASGI : 200 {"method": "POST", "path": "/hello", "bytes_in": 3000}
FastAPI cũng là một ASGI app: coroutine __call__ = True
tham số của __call__: ['scope', 'receive', 'send']
scope mà FastAPI nhận: {'type': 'http', 'method': 'GET', 'path': '/v1/quotes/QT-0001'}
```

Đối tượng `FastAPI` là lớp con của Starlette (`FastAPI.__mro__` bắt đầu bằng `FastAPI`, `Starlette`), và nó tự là một ASGI app: `__call__` là coroutine nhận đúng `scope, receive, send`. Route, kiểm tra body, OpenAPI đều nằm bên trong lời gọi đó. **uvicorn** là server ASGI: mở socket, đọc HTTP, dựng `scope`, gọi `await app(scope, receive, send)` rồi gửi những gì `send` nhận được. `uvicorn app.main:app` nghĩa là «import module `app.main`, lấy biến `app`».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Một request đi từ client qua uvicorn, FastAPI, handler create_quote rồi tới calculate_fare. Uvicorn và FastAPI là thư viện; create_quote và calculate_fare là code của Tới Nơi."><defs><marker id="py31-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="140" height="90" rx="10" class="f-bg s-line"/><text x="34" y="62" class="t-b">Client</text><text x="34" y="86" class="t-small">curl, app di động</text><text x="34" y="104" class="t-small">POST + JSON</text><path d="M160 75 L195 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-0-arrow)"/><rect x="195" y="30" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="209" y="62" class="t-b">uvicorn</text><text x="209" y="86" class="t-small">đọc HTTP, dựng</text><text x="209" y="104" class="t-small">scope/receive/send</text><path d="M335 75 L370 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-0-arrow)"/><rect x="370" y="30" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="384" y="62" class="t-b">FastAPI</text><text x="384" y="86" class="t-small">route + pydantic</text><text x="384" y="104" class="t-small">kiểm tra body</text><path d="M510 75 L545 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-0-arrow)"/><rect x="545" y="30" width="140" height="90" rx="10" class="f-pk-soft s-pk"/><text x="559" y="62" class="t-b">create_quote</text><text x="559" y="86" class="t-small">handler của ta</text><text x="559" y="104" class="t-small">async def</text><path d="M685 75 L720 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-0-arrow)"/><rect x="720" y="30" width="140" height="90" rx="10" class="f-pk-soft s-pk"/><text x="734" y="62" class="t-b">calculate_fare</text><text x="734" y="86" class="t-small">quy tắc miền,</text><text x="734" y="104" class="t-small">không biết HTTP</text><path d="M195 135 L195 145 L510 145 L510 135" class="f-none s-muted" stroke-width="1.5"/><text x="195" y="168" class="t-small">thư viện: không viết lại cho mỗi endpoint</text><path d="M545 135 L545 145 L860 145 L860 135" class="f-none s-muted" stroke-width="1.5"/><text x="545" y="168" class="t-small">code của Tới Nơi, chạy trong lab</text><text x="20" y="208" class="t-small">Response đi ngược lại cùng đường: handler trả QuoteResponse, FastAPI dựng JSON, uvicorn gửi byte.</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một request. Chỉ hai khối bên phải là code ta viết; hai khối giữa là thư viện, nói chuyện với nhau qua giao diện ASGI.</figcaption>
</figure>

## Ba lớp trong một endpoint

Lab tách ba thứ mà ví dụ «hello world» hay dồn vào một chỗ:

- `app/domain.py`: `calculate_fare`, `Quote`, `Money` (sao chép rút gọn từ bài 13). Không import FastAPI.
- `app/schemas.py`: hợp đồng HTTP, hai model pydantic.
- `app/main.py`: route, nối hai lớp trên với nhau.

Hợp đồng của yêu cầu và phản hồi (trích `app/schemas.py`):

```python
class QuoteRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    distance_km: Decimal = Field(decimal_places=3, allow_inf_nan=False, examples=["2.5"])
    service: ServiceKind = ServiceKind.RIDE


class QuoteResponse(BaseModel):
    quote_id: str = Field(examples=["QT-0001"])
    distance_km: Decimal
    service: ServiceKind
    fare: Decimal = Field(description="Cước VND, số nguyên dạng chuỗi.", examples=["14250"])
    currency: str = "VND"
    expires_at: datetime = Field(description="UTC, hết hạn sau 5 phút.")
```

`QuoteRequest` từ chối trường lạ (`extra="forbid"`), nhận `distance_km` dạng chuỗi `"2.5"` hoặc số `2.5` (kiểu `Decimal` ép kiểu ở chế độ lax như bài 15), tối đa 3 chữ số thập phân. Hợp đồng khuyên gửi chuỗi, cùng quy ước với `fare`: tiền không nên đi qua số thực. `Decimal` còn nhận ký pháp khoa học và chữ số Unicode: `"1e2"` vào thì phản hồi chuẩn hóa `distance_km` thành `"100"` (không để lộ `"1E+2"`), còn `"٢"` vẫn được nhận thành 2 km và `--verify` ghi lại đó là giới hạn đã biết; production nên kiểm chuỗi đầu vào bằng biểu thức chính quy chỉ nhận chữ số ASCII. Phản hồi có `Decimal` ra JSON là **chuỗi** (`"14250"`), nên client không phải đoán số nguyên hay số thực.

Route nối các lớp (trích `app/main.py`):

```python
@app.post("/v1/quotes", status_code=201, response_model=QuoteResponse, tags=["quotes"], summary="Tạo báo giá")
async def create_quote(body: QuoteRequest, request: Request, response: Response) -> QuoteResponse:
    try:
        fare = calculate_fare(body.distance_km, service=body.service)
    except ValueError as exc:  # quy tắc nghiệp vụ bị vi phạm -> 400, không phải 422
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    quote = Quote(body.distance_km, Money(fare), datetime.now(UTC) + QUOTE_TTL)
    quote_id = f"QT-{next(request.app.state.counter):04d}"
    ...  # dựng QuoteResponse, lưu vào request.app.state.quotes
    response.headers["Location"] = f"/v1/quotes/{quote_id}"
    return out
```

FastAPI đọc chữ ký hàm: tham số có kiểu `BaseModel` là **body JSON**, nên trước khi `create_quote` chạy, body đã được parse và kiểm bằng `QuoteRequest`; hàm chỉ chạy khi body hợp lệ. `response_model` quyết định hình dạng JSON trả ra, và `status_code=201` đổi mã thành công mặc định (200).

Vì sao 201 mà không phải 200? Bài C# cùng chủ đề, [HTTP và Minimal API](/articles/dotnet10-http-minimal-api), chọn 200 vì endpoint chỉ tính rồi trả, không tạo gì. Ở lab này, mỗi báo giá được cấp `quote_id`, lưu tạm và đọc lại được qua `GET /v1/quotes/{quote_id}`: một tài nguyên mới đã xuất hiện và `Location` trỏ tới nó. Không lưu gì thì 200 trung thực hơn; hai bài khác nhau vì hợp đồng khác nhau, không vì khác ngôn ngữ.

## 422 hay 400: sai hình dạng hay sai quy tắc

Một yêu cầu có thể hỏng ở ba chỗ khác nhau, và mỗi chỗ cho mã khác nhau:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Bốn bước xử lý một yêu cầu: đọc body JSON, kiểm QuoteRequest, gọi calculate_fare, trả 201. Ba bước đầu mỗi bước có một lối ra lỗi: JSON hỏng cho 422, sai hình dạng cho 422, ngoài miền cho 400."><defs><marker id="py31-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="76" rx="10" class="f-pk-soft s-pk"/><text x="36" y="62" class="t-b">1. Đọc body JSON</text><text x="36" y="86" class="t-small">Content-Type, cú pháp</text><path d="M210 68 L236 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-1-arrow)"/><rect x="236" y="30" width="190" height="76" rx="10" class="f-pk-soft s-pk"/><text x="252" y="62" class="t-b">2. Kiểm QuoteRequest</text><text x="252" y="86" class="t-small">thiếu, sai kiểu, trường lạ</text><path d="M426 68 L452 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-1-arrow)"/><rect x="452" y="30" width="190" height="76" rx="10" class="f-pk-soft s-pk"/><text x="468" y="62" class="t-b">3. calculate_fare</text><text x="468" y="86" class="t-small">quy tắc miền 0..100 km</text><path d="M642 68 L668 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py31-1-arrow)"/><rect x="668" y="30" width="192" height="76" rx="10" class="f-phys-soft s-phys"/><text x="684" y="62" class="t-b">4. Trả báo giá</text><text x="684" y="86" class="t-small">201 + Location</text><path d="M115 106 L115 150" class="f-none s-warn" stroke-width="2" marker-end="url(#py31-1-arrow)"/><path d="M331 106 L331 150" class="f-none s-warn" stroke-width="2" marker-end="url(#py31-1-arrow)"/><path d="M547 106 L547 150" class="f-none s-warn" stroke-width="2" marker-end="url(#py31-1-arrow)"/><rect x="20" y="150" width="190" height="90" rx="10" class="f-warn-soft s-warn"/><text x="36" y="180" class="t-b">422</text><text x="36" y="204" class="t-small">JSON hỏng hoặc</text><text x="36" y="222" class="t-small">sai Content-Type</text><rect x="236" y="150" width="190" height="90" rx="10" class="f-warn-soft s-warn"/><text x="252" y="180" class="t-b">422</text><text x="252" y="204" class="t-small">sai hình dạng</text><text x="252" y="222" class="t-small">(detail là danh sách)</text><rect x="452" y="150" width="190" height="90" rx="10" class="f-warn-soft s-warn"/><text x="468" y="180" class="t-b">400</text><text x="468" y="204" class="t-small">ngoài miền 0..100 km</text><text x="468" y="222" class="t-small">(detail là chuỗi)</text><text x="668" y="176" class="t-small">Quy ước của lab, không phải</text><text x="668" y="194" class="t-small">luật của HTTP: nhiều API</text><text x="668" y="212" class="t-small">dùng 422 cho cả hai.</text></svg>
</div>
<figcaption>Hình 2. Ba lối ra lỗi trước khi tới 201. Hai lối đầu do FastAPI và pydantic xử lý trước khi hàm của ta chạy; lối thứ ba do chính miền nghiệp vụ.</figcaption>
</figure>

Bảy ca lỗi chạy qua app, kèm một route không tồn tại và một method sai (trích section 3 của output):

```text
miền: 150 km                     -> 400 {"detail":"khoảng cách ngoài miền 0..100 km: 150"}
hình dạng: 2.1234 km             -> 422 {"detail":[{"type":"decimal_max_places","loc":["body","distance_km"],"msg":"Decimal input should have no more than 3 decimal places","input":"2.1234","ctx":{"decimal_places":3}}]}
hình dạng: thiếu distance_km     -> 422 {"detail":[{"type":"missing","loc":["body","distance_km"],"msg":"Field required","input":{}}]}
hình dạng: service lạ            -> 422 {"detail":[{"type":"enum","loc":["body","service"],"msg":"Input should be 'ride', 'food' or 'parcel'","input":"bike","ctx":{"expected":"'ride', 'food' or 'parcel'"}}]}
JSON hỏng                        -> 422 {"detail":[{"type":"json_invalid","loc":["body",1],"msg":"JSON decode error","input":{},"ctx":{"error":"Expecting property name enclosed in double quotes"}}]}
route không có                   -> 404 {"detail":"Not Found"}
PUT /v1/quotes                   -> 405 {"detail":"Method Not Allowed"} Allow=POST
```

Quy ước của lab: **sai hình dạng thì 422, đúng hình dạng nhưng vi phạm quy tắc nghiệp vụ thì 400**. Hình dạng là việc của pydantic: có trường không, đúng kiểu không, bao nhiêu chữ số thập phân. Miền 0 đến 100 km là quy tắc của `calculate_fare`, nên nó ném `ValueError` và handler đổi thành 400. Đây là chọn lựa thiết kế và nói thẳng thì nó **ngược chiều chữ RFC 9110** ở cả hai phía: 422 dành cho nội dung đúng cú pháp nhưng không xử lý được, nên 150 km mới là ca điển hình của 422, còn JSON hỏng cú pháp lẽ ra thuộc 400 thì FastAPI trả 422. Nhiều API vì thế dùng 422 cho cả hai nhóm. Lab giữ quy ước trên vì nó cho client phân biệt «sửa hình dạng» với «chọn giá trị khác» chỉ bằng mã HTTP; điều quan trọng là chọn một quy ước, ghi vào hợp đồng và giữ nhất quán.

Hai chi tiết đáng nhớ. Hai nhóm lỗi có hai hình dạng JSON khác nhau: `detail` lúc là chuỗi (400), lúc là danh sách (422), nên client phải viết hai nhánh; bài 32 đưa mọi lỗi về một dạng. Và `input` trong 422: FastAPI mặc định chép lại nguyên giá trị client gửi, nên với trường nhạy cảm (số giấy tờ, mật khẩu) phản hồi lỗi thành bản sao của chúng, có thể lọt vào log của proxy.

## OpenAPI tự sinh

FastAPI dựng đặc tả OpenAPI từ chính chữ ký hàm và model, không ai viết tay. Đặc tả nằm ở `/openapi.json`; `/docs` (Swagger UI) và `/redoc` đọc cùng file đó để vẽ trang tài liệu thử gọi được. Tóm tắt do lab in:

```text
== 4. OpenAPI ==
openapi: 3.1.0 | title: Tới Nơi - Báo giá
paths: ['/v1/quotes', '/v1/quotes/{quote_id}']
POST /v1/quotes responses: ['201', '422']
operationId: create_quote_v1_quotes_post
schemas: ['HTTPValidationError', 'QuoteRequest', 'QuoteResponse', 'ServiceKind', 'ValidationError']
/docs: 200 text/html; charset=utf-8 | /redoc: 200
```

Đối chiếu với code, nhiều thứ ta không viết mà đã có, và ba chỗ đặc tả nói chưa đủ:

- Có sẵn: mã `201`, schema `QuoteRequest` và `QuoteResponse`, mô tả trường, nhóm `quotes`.
- `distance_km` trong đặc tả là `anyOf: [number, string]`, đúng với những gì app nhận, nhưng `decimal_places=3` **không** hiện ra ở đó: client sinh từ đặc tả sẽ không biết giới hạn.
- Đặc tả chỉ có `201` và `422`. Lỗi `400` có trong code nhưng chưa có trong tài liệu, và `422` được mô tả bằng schema `HTTPValidationError` mặc định.
- `operationId` là `create_quote_v1_quotes_post`, sinh từ tên hàm và đường dẫn. Công cụ sinh client dùng nó làm tên hàm, nên đổi tên hàm Python vô tình đổi tên trong SDK.

Hai chỗ cuối thuộc bài 32; giới hạn thập phân quay lại ở bài tập. Lưu ý: Swagger UI mặc định nạp JavaScript từ CDN, nên máy không có mạng thấy trang `/docs` trắng dù server chạy tốt.

## async def hay def: ai chạy ở đâu

Quay lại câu hỏi đầu bài. FastAPI chạy handler theo cách tùy vào chữ `async`: **`async def`** chạy thẳng trên event loop, **`def` thường** được đẩy sang thread pool. Lab dựng một app thử nhỏ với ba route cùng ngủ 0,2 giây (trích `main.py`):

```python
@probe.get("/async-block")
async def async_block() -> dict:
    time.sleep(0.2)  # SAI có chủ ý: chặn nguyên event loop
    return {"thread": threading.current_thread().name}

@probe.get("/async-await")
async def async_await() -> dict:
    await asyncio.sleep(0.2)  # nhường event loop trong lúc chờ
    ...

@probe.get("/sync-def")
def sync_def() -> dict:
    time.sleep(0.2)  # chặn, nhưng chỉ chặn một luồng trong thread pool
    ...
```

Gửi năm request đồng thời tới từng route và đo tổng thời gian:

```text
== 5. 5 request đồng thời, mỗi handler ngủ 0,2 giây ==
load average lúc đo (1/5/15 phút): 52.06 54.10 47.86
/async-block    1.05 s  luồng: ['MainThread']
/async-await    0.20 s  luồng: ['MainThread']
/sync-def       0.25 s  luồng: ['AnyIO worker thread']
```

Kết quả nói rõ ba trường hợp:

| Handler | Tổng thời gian | Vì sao |
| --- | --- | --- |
| `async def` + `time.sleep` | 1,05 s | Chặn event loop: năm lời gọi tuần tự, mọi request khác cũng đứng chờ |
| `async def` + `await asyncio.sleep` | 0,20 s | `await` nhường loop, năm handler chờ chồng lên nhau |
| `def` + `time.sleep` | 0,25 s | Mỗi handler một luồng của thread pool (mặc định 40 luồng trong AnyIO của bản đã chạy), loop vẫn rảnh |

Quy tắc rút ra: handler gọi thư viện async (redis asyncio, `AsyncMongoClient` của PyMongo, httpx) thì dùng `async def`; gọi thư viện chặn (driver đồng bộ, `requests`) thì dùng `def` để FastAPI tự cách ly; còn **`async def` mà gọi hàm chặn là kết hợp tệ nhất**, vì không báo lỗi mà chỉ làm cả service chậm lại. `create_quote` chỉ tính toán thuần, không I/O, nên `async def` hợp lý. Việc nặng CPU thì chữ `async` nào cũng không cứu: trong `async def` nó giữ cả event loop, trong `def` nó chạy ở luồng của pool nhưng trên bản Python thường vẫn tranh GIL với loop. Hướng đi là tiến trình riêng hoặc worker nền (bài 22 và bài 40); cách nhường loop đúng chỗ ở bài 21 ([asyncio: coroutine, Task, timeout và hủy](/articles/python-asyncio-coroutine-task-timeout-cancellation)).

Năm request chạy in-process bằng `ASGITransport`, không qua mạng, nên phép đo chỉ phản ánh event loop, không phải độ trễ HTTP thật.

## Chạy và kiểm chứng

Mở server thật ở cổng 8031 (giữ terminal này mở):

```bash
uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia uvicorn app.main:app --app-dir examples/toi-noi-python/31-fastapi-endpoint-bao-gia --port 8031
```

`--app-dir` cho uvicorn biết tìm `app.main` ở đâu khi lệnh chạy từ gốc repository. Từ terminal khác, gọi bằng `curl` (trích, bỏ dòng header `date`):

```bash
curl -i -s -X POST http://127.0.0.1:8031/v1/quotes -H "Content-Type: application/json" -d '{"distance_km": "2.5", "service": "ride"}'
```

```text
HTTP/1.1 201 Created
server: uvicorn
content-length: 134
content-type: application/json
location: /v1/quotes/QT-0001

{"quote_id":"QT-0001","distance_km":"2.5","service":"ride","fare":"14250","currency":"VND","expires_at":"2026-11-06T11:52:44.840792Z"}
```

Gửi 150 km, rồi đọc lại báo giá đầu tiên theo `Location`:

```text
HTTP/1.1 400 Bad Request
content-type: application/json

{"detail":"khoảng cách ngoài miền 0..100 km: 150"}
```

```text
{"quote_id":"QT-0001","distance_km":"2.5","service":"ride","fare":"14250","currency":"VND","expires_at":"2026-11-06T11:52:44.840792Z"}
```

Server in từng request ra log, ví dụ `"POST /v1/quotes HTTP/1.1" 201 Created`. Kiểm toàn bộ bằng `--verify`, không cần server:

```bash
uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia examples/toi-noi-python/31-fastapi-endpoint-bao-gia/main.py --verify
```

```text
OK 39/39
```

39 ca gồm: cước 0, 2, 2,5, 5, 100 km ra 12000, 12000, 14250, 25500, 453000; 201 kèm `Location`; số JSON và chuỗi cho cùng kết quả, `"1e2"` ra `"100"`; từng nhóm lỗi; đặc tả khai `201` thay vì `200`; hai worker cấp trùng mã; và bất biến của phép đo luồng (chặn thì từ 1,0 giây trở lên, không chặn thì nhanh hơn chặn ít nhất 2 lần), không kiểm số tuyệt đối. Môi trường: Python 3.14.7, FastAPI 0.162.0, Starlette 1.10.0, pydantic 2.14.3, uvicorn 0.54.0, httpx 0.28.1, máy Apple M1 Max đang rất bận lúc đo (`load average` 1 phút là 52,06, lab in ở đầu phần đo), nên con số giây có thể khác trên máy bạn.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `ModuleNotFoundError: No module named 'app'` khi chạy uvicorn | `--app-dir` có trỏ đúng thư mục lab không; chạy từ gốc repository hay từ trong lab |
| `Error loading ASGI app. Attribute "app" not found` | Phần sau dấu hai chấm trong `app.main:app` là tên biến; `create_app()` chỉ là hàm, biến `app` phải tồn tại |
| `curl -d` ra 422 `model_attributes_type` dù body đúng | Thiếu `-H "Content-Type: application/json"`; curl mặc định gửi dạng form, FastAPI không thấy một đối tượng JSON |
| Trang `/docs` trắng | Swagger UI nạp JavaScript từ CDN; máy offline thì không vẽ được, `/openapi.json` vẫn dùng được |
| Service chậm dần khi nhiều request đồng thời | Có handler `async def` nào gọi hàm chặn (`time.sleep`, `requests`) không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Báo giá và bộ đếm <code>quote_id</code> nằm trong bộ nhớ của tiến trình. Mất khi restart; <code>uvicorn --workers 2</code> là hai tiến trình với hai dict và hai bộ đếm riêng, nên <b>mã bị trùng</b> chứ không chỉ thiếu: hai worker cùng cấp <code>QT-0001</code> cho hai báo giá khác nhau, và GET trả lúc báo giá này lúc báo giá kia mà không lỗi nào báo. Lab mô phỏng bằng hai app (có ca trong <code>--verify</code>) và chạy thật bằng <code>workers-demo.sh</code>, xem output bên dưới. Production cần kho dùng chung và id toàn cục (UUID hoặc sequence của database, bài 24, 27). Endpoint chưa có xác thực (bài 34), chưa giới hạn tần suất, và <code>POST</code> không idempotent: gọi hai lần sinh hai báo giá. <code>expires_at</code> chưa ai kiểm khi đặt chuyến. Không TLS, không reverse proxy, <code>/docs</code> mở công khai.</p>
</div>

Chạy thật với hai worker: mở server bằng `--workers 2` (cùng lệnh uvicorn ở trên, thêm `--workers 2`), rồi chạy `examples/toi-noi-python/31-fastapi-endpoint-bao-gia/workers-demo.sh` từ terminal khác. Kết quả của một lần chạy (số báo giá mỗi worker nhận phụ thuộc cách hệ điều hành chia kết nối, nên lần của bạn có thể khác):

```text
QT-0001 3 km, fare 16500
QT-0001 4 km, fare 21000
QT-0002 13 km, fare 61500
QT-0002 6 km, fare 30000
QT-0003 9 km, fare 43500
...
--- GET QT-0001, 6 lần
QT-0001 4 km, fare 21000
QT-0001 3 km, fare 16500
QT-0001 3 km, fare 16500
```

(Trích: giữ năm dòng POST và ba dòng GET đầu.) `QT-0001` và `QT-0002` mỗi mã xuất hiện hai lần cho hai quãng đường khác nhau, và GET `QT-0001` trả 4 km rồi 3 km tùy kết nối rơi vào worker nào: một khách có thể nhận báo giá của người khác khi đọc lại theo mã.

**Bài tập:** (1) Bỏ `decimal_places=3` khỏi `QuoteRequest`. Gửi `{"distance_km": "2.1234"}`: status là gì và `fare` bao nhiêu? Việc này cho thấy quy tắc «tối đa 3 chữ số thập phân» nên đặt ở lớp nào? (2) Đổi `async def create_quote` thành `def create_quote`. Kết quả HTTP có đổi không? Giải thích vì sao tên luồng chạy handler đổi, và nêu một điều kiện mà việc đổi này sẽ có lợi.

<details><summary>Xem lời giải</summary><p>(1) Không còn gì chặn 4 chữ số thập phân nên request qua pydantic, <code>calculate_fare(Decimal("2.1234"))</code> tính phần vượt 0,1234 km × 4.500 = 555,3, cước thô 12.555,3 làm tròn <b>12555</b> với status 201. Mất giới hạn này, hai quãng 2,1234 và 2,1236 km có thể ra hai cước khác nhau mà khách không phân biệt được. Quy tắc về <b>định dạng đầu vào</b> thuộc lớp schema (422, rẻ, báo sớm cho client); quy tắc về <b>nghiệp vụ</b> thuộc miền. (2) Kết quả HTTP y hệt, vì hàm không I/O. Chỉ thay đổi nơi chạy: <code>def</code> được FastAPI đẩy sang thread pool nên tên luồng là <code>AnyIO worker thread</code> thay vì <code>MainThread</code>. Có lợi khi handler gọi thư viện chặn (driver đồng bộ, <code>requests</code>): nó chỉ chặn một luồng của pool thay vì cả event loop. Với handler tính toán ngắn như này, chuyển luồng chỉ thêm chi phí.</p></details>

Bài trước: [bài 30](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh). Bài tiếp: [bài 32](/articles/python-fastapi-validation-problem-details-openapi).

## Tài liệu gốc

- [FastAPI: First Steps](https://fastapi.tiangolo.com/tutorial/first-steps/).
- [FastAPI: Concurrency and async / await](https://fastapi.tiangolo.com/async/).
- [FastAPI: Response Model](https://fastapi.tiangolo.com/tutorial/response-model/).
- [ASGI specification](https://asgi.readthedocs.io/en/latest/specs/main.html).
- [PEP 3333: Python Web Server Gateway Interface v1.0.1](https://peps.python.org/pep-3333/).
- [Uvicorn: deployment](https://uvicorn.dev/deployment/).

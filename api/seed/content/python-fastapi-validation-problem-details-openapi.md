---
slug: python-fastapi-validation-problem-details-openapi
title: "Validation, lỗi có cấu trúc (Problem Details) và OpenAPI: hợp đồng API"
summary: "Bài 32: đưa mọi lỗi của API báo giá về application/problem+json (RFC 9457) bằng exception handler, tách 400 và 422, khai báo lỗi trong OpenAPI, version /v1 và snapshot hợp đồng. Lab chạy thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, fastapi, openapi, problem-details]
language: vi
---

Service Dispatch của [Tới Nơi](/articles/du-an-mau-toi-noi), viết bằng C#, gọi `POST /v1/quotes` mà bài 31 vừa đưa lên HTTP. Khi yêu cầu sai, nó nhận lúc thì `{"detail": "khoảng cách ngoài miền 0..100 km: 150"}`, một chuỗi, lúc thì `{"detail": [...]}`, một danh sách, kèm thông điệp tiếng Anh và bản sao nguyên giá trị đã gửi. Người viết client phải đoán xem đó là dạng nào. Đội ứng dụng di động sinh SDK từ `openapi.json` thì không thấy mã 400 ở đâu cả. Hai vấn đề cùng một gốc: API chưa có **hợp đồng lỗi**.

Bài này biến lỗi thành dữ liệu có cấu trúc theo RFC 9457 (Problem Details), đăng ký handler để mọi lỗi có cùng một dạng, khai báo lỗi trong OpenAPI cho khớp hành vi, rồi chụp hợp đồng thành file để mỗi thay đổi hiện ra thành một diff. Đây là bài 32 của lộ trình **Python trung bình qua Tới Nơi**. Lab tiếp nối lab bài 31, nên hãy đọc bài 31 trước.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn đã đăng ký handler đổi mọi lỗi 422 thành <code>application/problem+json</code>. Mở <code>/docs</code>, mục 422 của <code>POST /v1/quotes</code> sẽ mô tả body lỗi theo dạng nào: Problem Details mới, hay dạng <code>HTTPValidationError</code> cũ?</p>
<details><summary>Xem đáp án</summary><p><b>Dạng cũ.</b> Handler chỉ đổi hành vi lúc chạy; tài liệu OpenAPI sinh từ khai báo của route, và FastAPI tự thêm 422 với schema mặc định nếu route không tự khai báo. Kết quả là tài liệu nói một đằng, server làm một nẻo. Bài chạy đúng tình huống này và sửa bằng <code>responses=</code>.</p></details>
</div>

## Lỗi mặc định của FastAPI thiếu gì

Test case xấu có hai lỗi cùng lúc, gửi tới app chưa đăng ký handler nào (trích output của `main.py`):

```text
422 application/json
{"detail": [
  {"type": "decimal_max_places", "loc": ["body", "distance_km"],
   "msg": "Decimal input should have no more than 3 decimal places",
   "input": "2.1234", "ctx": {"decimal_places": 3}},
  {"type": "enum", "loc": ["body", "service"],
   "msg": "Input should be 'ride', 'food' or 'parcel'",
   "input": "bike", "ctx": {"expected": "'ride', 'food' or 'parcel'"}}]}
```

(Trích; kết quả chạy thật in thụt dòng từng khoá, ở đây nén lại cho gọn.) Cấu trúc `detail[].loc/msg/type` đủ dùng khi phát triển, nhưng chưa thành hợp đồng:

- **Hai hình dạng.** Lỗi validation có `detail` là danh sách; lỗi `HTTPException` (như 400 ở bài 31, 404) có `detail` là chuỗi. Client phải viết hai nhánh.
- **Không có định danh ổn định của ta.** `type` như `decimal_max_places` là mã của pydantic. Mã này thuộc về thư viện, và khi nâng phiên bản thì tên có thể đổi. `msg` là tiếng Anh, so sánh chuỗi thì mong manh.
- **Chép lại `input`.** Giá trị client gửi quay ngược trong phản hồi, và có thể nằm lại trong log của proxy. Với số giấy tờ hay mật khẩu, đó là rò rỉ.
- **Cùng media type `application/json`** với phản hồi thành công, nên phần mềm trung gian không phân biệt được body lỗi chuẩn.

## Problem Details: một hình dạng cho mọi lỗi

RFC 9457 (thay RFC 7807) định nghĩa một đối tượng JSON mô tả lỗi, gửi với media type `application/problem+json`. Năm thành viên chuẩn, đều tuỳ chọn, cộng với thành viên mở rộng tuỳ ý:

| Thành viên | Ý nghĩa | Cách lab dùng |
| --- | --- | --- |
| `type` | URI định danh **loại** lỗi; client dùng nó làm định danh chính. Mặc định `about:blank` | `https://toinoi.example/problems/distance-out-of-range` |
| `title` | Tóm tắt ngắn, không đổi giữa các lần lỗi cùng `type` | «Khoảng cách ngoài miền cho phép» |
| `status` | Mã HTTP, chỉ để tiện đọc; phải trùng mã HTTP thật | `400` |
| `detail` | Giải thích riêng cho lần lỗi này, để người đọc; RFC khuyên client không phân tích chuỗi này | «khoảng cách 150 km nằm ngoài miền 0..100 km» |
| `instance` | Định danh một lần xuất hiện của lỗi | đường dẫn request, `/v1/quotes` |
| mở rộng | Dữ liệu máy đọc được, thay cho việc phân tích `detail` | `errors[]`, `min_km`, `max_km` |

Vài điểm trong RFC đáng nhớ khi thiết kế. `type` là chỗ client rẽ nhánh, nên đổi `type` là thay đổi phá vỡ client. URI không bắt buộc phân giải được, nhưng RFC khuyến khích URI phân giải được vì đổi sang URI mới về sau đồng nghĩa đổi định danh; lab dùng `toinoi.example` (tên miền `.example` dành riêng, không bao giờ phân giải thật), production nên trỏ tới trang tài liệu thật. Khi chỉ cần mã HTTP, dùng `about:blank` và để `title` là tên chuẩn của mã (`Not Found`). Lab dùng `instance` là đường dẫn request: biết lỗi ở route nào, nhưng chưa phân biệt được hai lần gọi; định danh riêng từng lần để tra log thì cần một mã gắn với từng request. Bài 33 sẽ gán `request_id` ở middleware và đưa nó vào extension của Problem Details, để bên nhận lỗi đưa mã đó cho bên vận hành tra log.

FastAPI không tự sinh dạng này, nên ta viết handler. Bài C# cùng chủ đề, [Validation, Problem Details và OpenAPI](/articles/dotnet10-validation-problem-details-openapi), dùng `AddProblemDetails` có sẵn của ASP.NET Core: cùng RFC, khác công sức. Lỗi validation ở bài C# là 400, ở FastAPI mặc định là 422; hai framework có mặc định khác nhau nên phải ghi vào hợp đồng chứ đừng suy ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Năm nguồn lỗi gồm RequestValidationError, DistanceOutOfRange, QuoteNotFound, HTTPException và Exception bất ngờ cùng đi qua hàm problem_response, và ra thành năm loại problem+json với mã 422, 400, 404, 404 hoặc 405, và 500."><defs><marker id="py32-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="34" y="45" class="t-mono">RequestValidationError</text><rect x="20" y="72" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="34" y="97" class="t-mono">DistanceOutOfRange</text><rect x="20" y="124" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="34" y="149" class="t-mono">QuoteNotFound</text><rect x="20" y="176" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="34" y="201" class="t-mono">HTTPException (404, 405)</text><rect x="20" y="228" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="34" y="253" class="t-mono">Exception bất ngờ</text><path d="M270 40 L330 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M270 92 L330 110" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M270 144 L330 144" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M270 196 L330 178" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M270 248 L330 198" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><rect x="330" y="20" width="160" height="248" rx="10" class="f-phys-soft s-phys"/><text x="346" y="130" class="t-b">problem_response</text><text x="346" y="154" class="t-small">một hàm dựng body,</text><text x="346" y="172" class="t-small">cùng media type</text><path d="M490 40 L550 40" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M490 92 L550 92" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M490 144 L550 144" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M490 196 L550 196" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><path d="M490 248 L550 248" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-0-arrow)"/><rect x="550" y="20" width="310" height="40" rx="8" class="f-warn-soft s-warn"/><text x="562" y="45" class="t-small">422  validation-error  (+ errors[])</text><rect x="550" y="72" width="310" height="40" rx="8" class="f-warn-soft s-warn"/><text x="562" y="97" class="t-small">400  distance-out-of-range  (+ min_km, max_km)</text><rect x="550" y="124" width="310" height="40" rx="8" class="f-warn-soft s-warn"/><text x="562" y="149" class="t-small">404  quote-not-found</text><rect x="550" y="176" width="310" height="40" rx="8" class="f-warn-soft s-warn"/><text x="562" y="201" class="t-small">404, 405  about:blank  (title là tên mã HTTP)</text><rect x="550" y="228" width="310" height="40" rx="8" class="f-warn-soft s-warn"/><text x="562" y="253" class="t-small">500  internal-error  (chi tiết chỉ vào log)</text></svg>
</div>
<figcaption>Hình 1. Năm nguồn lỗi, một cách dựng phản hồi. Client chỉ còn một dạng body để đọc, bất kể lỗi đến từ pydantic, từ miền nghiệp vụ hay từ framework.</figcaption>
</figure>

## Năm handler, một cách dựng lỗi

Mọi handler gọi cùng một hàm (trích `app/problems.py`):

```python
def problem_response(request, *, status, type_, title, detail=None, headers=None, **extensions) -> JSONResponse:
    problem = ProblemDetails(
        type=type_, title=title, status=status, detail=detail, instance=request.url.path, **extensions
    )
    return JSONResponse(
        jsonable_encoder(problem.model_dump(exclude_none=True)),
        status_code=status,
        media_type="application/problem+json",
        headers=headers,
    )
```

`ProblemDetails` là một model pydantic với `extra="allow"` để nhận thành viên mở rộng; `exclude_none` bỏ `detail` hay `errors` khi không có. Dựng qua model thay vì dict viết tay thì cùng model đó mô tả lỗi trong OpenAPI, hai nơi không lệch nhau.

Lỗi validation đổi thành danh sách phẳng, bỏ `input`, dịch thông điệp cho các mã đã gặp:

```python
def field_problems(exc: RequestValidationError) -> list[FieldProblem]:
    out = []
    for err in exc.errors():
        code = err["type"]
        # json_invalid có loc ["body", <vị trí lỗi>]: vị trí ký tự không phải tên trường
        field = "body" if code == "json_invalid" else ".".join(str(part) for part in err["loc"])
        out.append(FieldProblem(field=field, code=code, message=MESSAGES_VI.get(code, err["msg"])))
    return out
```

Chạy ba lỗi cùng lúc (`distance_km` 4 chữ số thập phân, `service` lạ, trường thừa `x`) cho ra một phản hồi, mỗi lỗi một dòng trong `errors` (output rút từ lab, đã xuống dòng cho dễ đọc):

```text
422 application/problem+json
{"type": "https://toinoi.example/problems/validation-error",
 "title": "Yêu cầu không hợp lệ", "status": 422, "detail": "3 lỗi trong yêu cầu", "instance": "/v1/quotes",
 "errors": [
  {"field": "body.distance_km", "code": "decimal_max_places", "message": "tối đa 3 chữ số thập phân"},
  {"field": "body.service", "code": "enum", "message": "giá trị không thuộc danh sách cho phép"},
  {"field": "body.x", "code": "extra_forbidden", "message": "trường không có trong hợp đồng"}]}
```

Client rẽ nhánh theo `type`, tô đỏ theo `errors[].field`, hiển thị `message`. `code` vẫn là mã của pydantic; muốn tách hẳn khỏi thư viện thì cần bảng đổi riêng. Lab giữ nguyên các mã đó và verify ghim cứng ba mã của test case ba lỗi, nên nâng pydantic mà đổi mã thì đỏ ngay.

Lỗi miền khác lỗi hình dạng. Ở bài 31, route tự `try/except ValueError`. Ở đây miền ném một exception **có kiểu**, vẫn là `ValueError` với thông điệp cũ của loạt bài:

```python
class DistanceOutOfRange(ValueError):
    def __init__(self, distance_km: Decimal) -> None:
        super().__init__(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
        self.distance_km = distance_km
```

và handler đăng ký riêng cho kiểu đó:

```python
@app.exception_handler(DistanceOutOfRange)
async def on_distance(request: Request, exc: DistanceOutOfRange) -> JSONResponse:
    return problem_response(
        request, status=400, type_=TYPE_BASE + "distance-out-of-range",
        title="Khoảng cách ngoài miền cho phép",
        detail=f"khoảng cách {exc.distance_km} km nằm ngoài miền 0..{MAX_KM} km",
        min_km="0", max_km=str(MAX_KM),
    )
```

Vì sao không đăng ký handler cho `ValueError` chung cho gọn? Vì `ValueError` bay ra từ **bất cứ đâu**, kể cả lỗi lập trình, và mọi lỗi đó sẽ bị báo là «khách gửi sai». Lab thêm một route gây lỗi có chủ ý, route này ném `ValueError("bug-bi-mat: ...")` để kiểm:

```text
-- 150 km (miền): 400 application/problem+json
{"type": "https://toinoi.example/problems/distance-out-of-range", "title": "Khoảng cách ngoài miền cho phép", "status": 400, "detail": "khoảng cách 150 km nằm ngoài miền 0..100 km", "instance": "/v1/quotes", "min_km": "0", "max_km": "100"}
-- /nope: 404 application/problem+json
{"type": "about:blank", "title": "Not Found", "status": 404, "instance": "/nope"}
-- PUT /v1/quotes: 405 application/problem+json
{"type": "about:blank", "title": "Method Not Allowed", "status": 405, "instance": "/v1/quotes"}
-- ValueError ở chỗ khác: 500 application/problem+json
{"type": "https://toinoi.example/problems/internal-error", "title": "Lỗi nội bộ", "status": 500, "detail": "Có lỗi ở phía máy chủ; hãy thử lại sau.", "instance": "/v1/_boom"}
```

(Trích.) `ValueError` lạc chỗ ra 500, và phản hồi **không** chứa chữ `bug-bi-mat` hay tên lớp exception: handler `Exception` ghi chi tiết và traceback vào log, client chỉ thấy câu chung chung. Verify kiểm tra cả hai: body không lộ, và log có đúng một bản ghi kèm traceback. Hai chi tiết kỹ thuật: 404 và 405 do Starlette ném, nên handler phải đăng ký cho `starlette.exceptions.HTTPException` (lớp cha của `HTTPException` của FastAPI); còn handler `Exception` chạy ở tầng ngoài cùng và Starlette vẫn ném lại sau khi gửi phản hồi để server ghi log, nên test client phải đặt `raise_app_exceptions=False` mới nhận 500 thay vì exception.

## OpenAPI nói đúng về lỗi

Handler chỉ sửa hành vi. Để tài liệu khớp, route phải khai báo lỗi mà nó có thể trả (trích `app/main.py`):

```python
COMMON_ERRORS = {
    422: {"model": ProblemDetails, "description": "Yêu cầu sai hình dạng (thiếu trường, sai kiểu, trường lạ)."},
    500: {"model": ProblemDetails, "description": "Lỗi nội bộ; chi tiết chỉ có trong log."},
}
v1 = APIRouter(prefix="/v1", tags=["quotes"], responses=COMMON_ERRORS)


@v1.post(
    "/quotes",
    operation_id="createQuote",
    summary="Tạo báo giá",
    status_code=201,
    response_model=QuoteResponse,
    responses={400: {"model": ProblemDetails, "description": "Khoảng cách ngoài miền 0..100 km."}},
)
async def create_quote(body: QuoteRequest, request: Request, response: Response) -> QuoteResponse: ...
```

`responses` đặt ở router thì áp dụng cho mọi route, nên 422 và 500 khai báo một lần. Riêng 422 có lý do thêm: FastAPI tự gắn 422 mặc định (`HTTPValidationError`) vào mọi route có tham số, kể cả `GET` với `quote_id: str` hiện chưa thể gây 422; khai báo lại để tài liệu không rơi về schema cũ, và không đổi nghĩa khi kiểu tham số đổi sau này; 400 và 404 khai báo ở từng route vì chỉ route đó có. `operation_id` đặt bằng tay để có tên ổn định (`createQuote`) thay vì `create_quote_v1_quotes_post` sinh từ tên hàm và đường dẫn. `tags` kèm mô tả gom route thành nhóm trong `/docs`.

Có một vướng mắc: FastAPI gắn schema của `model` vào `application/json`, trong khi lỗi của ta là `application/problem+json`. Lab đổi tên khoá sau khi sinh, theo mẫu «extending OpenAPI» của FastAPI:

```python
def problem_media_type_in_openapi(app: FastAPI) -> None:
    generate = app.openapi

    def openapi() -> dict[str, Any]:
        if app.openapi_schema:
            return app.openapi_schema
        spec = generate()  # tự lưu vào app.openapi_schema; ta sửa tại chỗ
        ref = {"$ref": "#/components/schemas/ProblemDetails"}
        for path_item in spec["paths"].values():
            for operation in path_item.values():
                for response in operation["responses"].values():
                    content = response.get("content", {})
                    if content.get("application/json", {}).get("schema") == ref:
                        content[PROBLEM_JSON] = content.pop("application/json")
        return spec

    app.openapi = openapi
```

Kết quả (output của lab; thứ tự mã là thứ tự FastAPI sinh):

```text
POST /v1/quotes:
  201 application/json           QuoteResponse
  422 application/problem+json   ProblemDetails
  500 application/problem+json   ProblemDetails
  400 application/problem+json   ProblemDetails
GET /v1/quotes/{quote_id}:
  200 application/json           QuoteResponse
  422 application/problem+json   ProblemDetails
  500 application/problem+json   ProblemDetails
  404 application/problem+json   ProblemDetails
schemas: ['FieldProblem', 'ProblemDetails', 'QuoteRequest', 'QuoteResponse', 'ServiceKind']
```

`HTTPValidationError` đã biến mất khỏi danh sách schema, và mã 400, 404 đã có mặt. Nhưng tài liệu đúng chưa chứng minh server đúng, nên lab đối chiếu hai phía: gửi năm yêu cầu thật (201, 400, 422, 200, 404), tra mã trả về và media type trong đặc tả, rồi dựng lại body lỗi bằng chính model `ProblemDetails`. Kết quả `lệch 0 ca`. Và đây là cái bẫy trong câu hỏi đầu bài, dựng lại trong lab: app đăng ký handler nhưng route không khai báo `responses`:

```text
quên khai báo responses:
  runtime  : application/problem+json
  OpenAPI  : {'application/json': 'HTTPValidationError'}
```

Đặc tả nói body là `HTTPValidationError` còn server trả `problem+json`. SDK sinh tự động sẽ cố đọc lỗi theo schema sai, và không test đặc tả nào cảnh báo, vì tài liệu vẫn «hợp lệ».

## Version và snapshot: đổi hợp đồng có chủ ý

Tất cả route nằm dưới tiền tố `/v1` của `APIRouter`. `info.version` (`1.0.0`) là phiên bản hợp đồng theo semver, còn `/v1` chỉ là số major trong URL: một thay đổi tương thích, như thêm trường tuỳ chọn, tăng `1.0.0` lên `1.1.0` mà URL không đổi. Khi cần đổi phá vỡ, ta thêm router `/v2` chạy song song và để `/v1` sống thêm một thời gian cho client cũ. Thế nào là thay đổi phá vỡ:

| Thay đổi | Phá client cũ? | Cách làm |
| --- | --- | --- |
| Thêm route, thêm trường tuỳ chọn vào response | Thường không, nếu client bỏ qua trường lạ | Cùng `/v1`, tăng minor |
| Đổi `fare` từ chuỗi sang số | Có | `/v2` hoặc trường mới |
| Đổi `type` URI của một lỗi | Có, vì client rẽ nhánh theo `type` | Giữ `type` cũ, thêm `type` mới |
| Siết validation (từ chối thứ trước đây nhận) | Có | Báo trước, `/v2` |
| Nới validation | Thường không | Cùng `/v1` |

Biết thế nào là phá vỡ chưa đủ, ta cần phát hiện lúc nó xảy ra. Lab giữ file `openapi.snapshot.json` (328 dòng, đã commit), bản chụp hợp đồng hiện hành; `--verify` so `app.openapi()` với file. Muốn chứng minh cơ chế bắt được thay đổi, lab giả lập một lần sửa: ai đó đổi `POST` từ 201 sang 200, rồi in đúng dòng khác biệt:

```text
openapi.snapshot.json: 328 dòng; app hiện tại khớp snapshot: True
diff khi đổi 201 thành 200:
  -          "201": {
  +          "200": {
```

Khi chủ động đổi hợp đồng, chạy `main.py --write-snapshot`, xem `git diff` của file rồi commit cùng thay đổi code. Một diff mà không ai chủ ý tạo ra là tín hiệu có người vừa phá hợp đồng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Quy trình snapshot: đổi code, sinh openapi bằng app.openapi, so với snapshot đã commit. Khớp thì verify đạt. Lệch thì verify đỏ và in diff, người xem diff quyết định ghi lại snapshot và commit cùng code nếu cố ý, hoặc sửa code nếu vô ý."><defs><marker id="py32-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="200" height="80" rx="10" class="f-pk-soft s-pk"/><text x="36" y="62" class="t-b">1. Đổi code</text><text x="36" y="86" class="t-small">route, model, status</text><path d="M220 70 L260 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-1-arrow)"/><rect x="260" y="30" width="200" height="80" rx="10" class="f-pk-soft s-pk"/><text x="276" y="62" class="t-b">2. app.openapi()</text><text x="276" y="86" class="t-small">hợp đồng hiện tại</text><path d="M460 70 L500 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-1-arrow)"/><rect x="500" y="30" width="200" height="80" rx="10" class="f-pk-soft s-pk"/><text x="516" y="62" class="t-b">3. So với snapshot</text><text x="516" y="86" class="t-small">file đã commit</text><path d="M700 70 L740 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py32-1-arrow)"/><rect x="740" y="30" width="120" height="80" rx="10" class="f-phys-soft s-phys"/><text x="756" y="62" class="t-b">Khớp</text><text x="756" y="86" class="t-small">verify đạt</text><path d="M600 110 L600 160" class="f-none s-warn" stroke-width="2" marker-end="url(#py32-1-arrow)"/><rect x="500" y="160" width="360" height="80" rx="10" class="f-warn-soft s-warn"/><text x="516" y="192" class="t-b">Lệch</text><text x="516" y="216" class="t-small">verify đỏ, in đúng dòng khác biệt</text><path d="M500 200 L460 200" class="f-none s-warn" stroke-width="2" marker-end="url(#py32-1-arrow)"/><rect x="20" y="160" width="440" height="80" rx="10" class="f-bg s-line"/><text x="36" y="192" class="t-b">4. Người xem diff quyết định</text><text x="36" y="214" class="t-small">Cố ý: --write-snapshot, commit cùng thay đổi code</text><text x="36" y="230" class="t-small">Vô ý: sửa code cho khớp lại hợp đồng</text></svg>
</div>
<figcaption>Hình 2. Snapshot biến «ai đó vừa đổi hợp đồng» thành một diff có người xem. Snapshot không thay thế test hành vi; bài 36 đưa phép so này vào pytest.</figcaption>
</figure>

## Chạy và kiểm chứng

Từ thư mục gốc repository, chạy toàn bộ kịch bản và chế độ kiểm chứng (không cần server, không cần hạ tầng):

```bash
uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details examples/toi-noi-python/32-fastapi-validation-problem-details/main.py
uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details examples/toi-noi-python/32-fastapi-validation-problem-details/main.py --verify
```

```text
OK 51/51
```

51 test case gồm: 422 mặc định; bảy test case xấu cho 422 `problem+json` có `errors`, không chép lại input; ba lỗi cùng lúc; 400 kèm `min_km`, `max_km`; 404 hai loại; 405 giữ `Allow`; 500 không lộ nội dung và có log; 201 vẫn là `application/json`, `"1e2"` chuẩn hoá thành `"100"`; đặc tả khai đúng mã và media type; đối chiếu năm phản hồi thật; cái bẫy quên `responses`; snapshot. Môi trường: Python 3.14.7, FastAPI 0.162.0, Starlette 1.10.0, pydantic 2.14.3, httpx 0.28.1.

Server thật ở cổng 8032 (giữ terminal này mở, dừng bằng Ctrl+C):

```bash
uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details uvicorn app.main:app --app-dir examples/toi-noi-python/32-fastapi-validation-problem-details --port 8032
curl -i -s -X POST http://127.0.0.1:8032/v1/quotes -H "Content-Type: application/json" -d '{"distance_km": "150"}'
```

```text
HTTP/1.1 400 Bad Request
server: uvicorn
content-length: 241
content-type: application/problem+json

{"type":"https://toinoi.example/problems/distance-out-of-range","title":"Khoảng cách ngoài miền cho phép","status":400,"detail":"khoảng cách 150 km nằm ngoài miền 0..100 km","instance":"/v1/quotes","min_km":"0","max_km":"100"}
```

(Trích, bỏ dòng header `date`.) Thử thêm `-X PUT` cho 405 có header `allow: POST`, và `{"distance_km": "2.1234", "service": "bike"}` cho 422 có hai dòng trong `errors`.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| 404 của route không tồn tại vẫn ra `{"detail":"Not Found"}` | Handler có đăng ký cho `starlette.exceptions.HTTPException` không; đăng ký cho `fastapi.HTTPException` thì không bắt được lỗi do Starlette ném |
| `/docs` hiện 422 là `HTTPValidationError` dù server trả problem+json | Route hoặc router có khai báo `responses={422: {"model": ProblemDetails}}` không |
| Test client ném exception thay vì nhận 500 | `httpx.ASGITransport(app=app, raise_app_exceptions=False)` |
| `--verify` đỏ ở test case snapshot sau khi sửa model | Thay đổi có chủ ý không; nếu có, `--write-snapshot` rồi đọc `git diff` của `openapi.snapshot.json` |
| Client mới nhận `type` lạ và không biết xử lý | `type` có đổi so với bản đã công bố không; thêm `type` mới thay vì đổi tên `type` cũ |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>URI <code>toinoi.example/problems/...</code> không phân giải thật; production cần trang tài liệu cho từng loại lỗi. Thông điệp tiếng Việt chỉ có cho bảy mã pydantic mà API này gặp, mã khác rơi về tiếng Anh, và chưa có <code>Accept-Language</code>. <code>instance</code> là đường dẫn chứ chưa phải mã riêng từng lần lỗi, và 500 chỉ ghi log vào bộ nhớ của demo; log có cấu trúc và <code>request_id</code> trong Problem Details là phần của bài 33. Snapshot chỉ so cấu trúc đặc tả, không biết server có làm đúng như tài liệu không (đó là phần đối chiếu và bài 36). OpenAPI vẫn chưa diễn đạt giới hạn 3 chữ số thập phân của <code>distance_km</code>; lab chỉ ghi vào <code>description</code>. Chưa có xác thực, giới hạn tần suất hay <code>/v2</code> thật.</p>
</div>

**Bài tập:** (1) Trong `app/main.py`, đổi `status_code=201` thành `status_code=200` rồi chạy `--verify`. Chạy trước khi đọc đáp án: bao nhiêu test case đỏ, và vì sao phép đối chiếu năm phản hồi thật với đặc tả vẫn xanh? (2) Thêm lỗi miền mới: dịch vụ `parcel` chỉ nhận tối đa 30 km, ném `ParcelTooFar` rồi đăng ký handler 400 với `type` riêng, nhưng quên khai báo gì thêm ở route. Test case nào của `--verify` bắt được lỗi tài liệu?

<details><summary>Xem lời giải</summary><p>(1) Lab chạy cho <code>FAIL 46/51</code>: năm test case đỏ là «201 vẫn là application/json», «1e2 nhận được» (test case này cũng đòi 201), «POST khai báo 201, 400, 422, 500», «thành công khai báo application/json» và so snapshot. Phép đối chiếu năm phản hồi vẫn xanh vì hành vi lẫn tài liệu cùng đổi sang 200 nên chúng khớp nhau; cái bắt được là những thứ giữ bản hợp đồng <b>cũ</b>: snapshot đã commit và các test case ghim cứng mã 201. Bài học: đối chiếu tài liệu với hành vi và snapshot trả lời hai câu khác nhau («có nhất quán không» và «có đổi không»), cần cả hai. (2) Không test case nào. Snapshot chỉ ghi những gì đã được khai báo, nên tài liệu thiếu thì snapshot cũng thiếu; ma trận đối chiếu chỉ gồm năm phản hồi đã liệt kê, trong đó không có test case <code>parcel</code> quá 30 km. Muốn bắt, phải thêm đúng test case đó vào ma trận: gửi <code>parcel</code> 31 km và kiểm mã trả về có trong <code>responses</code>. Ở route này mã 400 đã được khai báo cho lỗi khoảng cách, nên tài liệu vẫn đúng về mã và media type; cái thiếu là mô tả riêng của loại lỗi mới.</p></details>

Bài trước: [bài 31](/articles/python-fastapi-http-endpoint-bao-gia). Bài tiếp: [bài 33](/articles/python-fastapi-dependency-middleware-settings-logging).

## Tài liệu gốc

- [RFC 9457: Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html).
- [FastAPI: Handling Errors](https://fastapi.tiangolo.com/tutorial/handling-errors/).
- [FastAPI: Additional Responses in OpenAPI](https://fastapi.tiangolo.com/advanced/additional-responses/).
- [FastAPI: Path Operation Advanced Configuration](https://fastapi.tiangolo.com/advanced/path-operation-advanced-configuration/).
- [FastAPI: Extending OpenAPI](https://fastapi.tiangolo.com/how-to/extending-openapi/).
- [OpenAPI Specification 3.1.0](https://spec.openapis.org/oas/v3.1.0.html).

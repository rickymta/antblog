---
slug: python-fastapi-api-test-httpx
title: "API test với httpx AsyncClient: kiểm chứng hợp đồng"
summary: "Bài 36: kiểm chứng API FastAPI không mở cổng mạng bằng httpx.AsyncClient và ASGITransport: lifespan, dependency_overrides, token JWT, lỗi Problem Details, snapshot OpenAPI. Lab 59 test chạy thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, fastapi, pytest, httpx]
language: vi
---

Một lập trình viên của [Tới Nơi](/articles/du-an-mau-toi-noi) thêm trường `rating` vào response của `GET /v1/trips/{trip_id}`. Mọi unit test của hàm tính cước vẫn xanh, nhưng ứng dụng khách và service Dispatch viết bằng C# đọc response này, và một trường mới có thể làm vỡ bộ giải mã chặt. Điều cần bảo vệ là **hợp đồng HTTP**: đường dẫn, mã trạng thái, hình dạng JSON, quy tắc xác thực, dạng lỗi. Unit test (bài 10) không chạm tới nó, còn dựng PostgreSQL và server thật cho mỗi ca thì chậm.

Đây là bài 36 của lộ trình **Python trung bình qua Tới Nơi**. Bài dùng lại FastAPI từ bài 31 đến 35 (route, Problem Details, `Depends`, JWT) và pytest ở bài 10; lab tự chứa, có một API nhỏ gom đủ thứ cần kiểm. Bên .NET, cùng việc này là [WebApplicationFactory](/articles/dotnet10-webapplicationfactory-integration-test).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một test gọi <code>GET /v1/trips/TN-0001</code> kèm token hợp lệ qua <code>httpx.AsyncClient(transport=httpx.ASGITransport(app=app))</code>. Kho chuyến của app dựng trong <code>lifespan</code> (<code>app.state.repo</code>). Test nhận 200, 500, hay gì khác?</p>
<details><summary>Xem đáp án</summary><p><b>Một ngoại lệ <code>AttributeError</code> ném thẳng vào test</b> (<code>'State' object has no attribute 'repo'</code>), không phải response 500. <code>ASGITransport</code> không chạy lifespan nên kho chưa được dựng; và mặc định nó ném lại ngoại lệ của app thay vì biến thành 500. (Không có token thì test nhận 401 trước, vì xác thực chạy trước <code>get_repo</code>.) Bài chạy đúng ca này rồi cách mở lifespan trong fixture.</p></details>
</div>

## Test API nằm ở đâu

Unit test không biết HTTP; integration test với database thật (bài 28) cần Docker và chạy lâu. Test API gọi **cả đường ống HTTP** của app (middleware, routing, kiểm dữ liệu, dependency, handler, exception handler, serialize) trong cùng tiến trình, không mở socket.

| Kiểm | Unit test | Test API (bài này) | Integration test (bài 28) |
| --- | --- | --- | --- |
| Mã trạng thái, JSON, header | Không | **Có, đây là việc chính** | Gián tiếp |
| Token, scope, dạng lỗi | Không | Có | Không |
| Truy vấn SQL, ràng buộc DB | Không | Chỉ khi kho là thật | **Có** |
| TCP, TLS, proxy, nhiều tiến trình | Không | Không | Không (cần thử trên server thật) |

Dòng cuối là giới hạn thật: test trong tiến trình không bắt được lỗi chỉ có trên mạng, như bài 35 thấy với `--workers 2`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Test async gọi httpx.AsyncClient, qua ASGITransport, đi thẳng vào app FastAPI gồm routing, dependencies và handler, trong cùng một tiến trình. Hộp dependency_overrides thay các dependency như kho chuyến và đồng hồ bằng bản giả. Chú thích nói rõ không có socket, cổng hay uvicorn."><defs><marker id="py36-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="15" y="50" width="120" height="80" rx="10" class="f-pk-soft s-pk"/><text x="29" y="82" class="t-b">Test</text><text x="29" y="106" class="t-small">async def test_*</text><path d="M135 90 L155 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py36-0-arrow)"/><rect x="155" y="50" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="169" y="82" class="t-b">AsyncClient</text><text x="169" y="106" class="t-small">get, post, headers</text><path d="M285 90 L305 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py36-0-arrow)"/><rect x="305" y="50" width="140" height="80" rx="10" class="f-pk-soft s-pk"/><text x="319" y="82" class="t-b">ASGITransport</text><text x="319" y="106" class="t-small">gọi app(scope,</text><text x="319" y="122" class="t-small">receive, send)</text><path d="M445 90 L465 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py36-0-arrow)"/><rect x="465" y="20" width="400" height="140" rx="10" class="f-bg s-line"/><text x="479" y="44" class="t-b">App FastAPI, cùng tiến trình</text><rect x="479" y="58" width="116" height="86" rx="8" class="f-phys-soft s-phys"/><text x="491" y="84" class="t-b">Routing</text><text x="491" y="106" class="t-small">kiểm dữ liệu,</text><text x="491" y="122" class="t-small">ép kiểu</text><rect x="607" y="58" width="116" height="86" rx="8" class="f-phys-soft s-phys"/><text x="619" y="84" class="t-b">Dependencies</text><text x="619" y="106" class="t-small">token, kho,</text><text x="619" y="122" class="t-small">đồng hồ</text><rect x="735" y="58" width="116" height="86" rx="8" class="f-phys-soft s-phys"/><text x="747" y="84" class="t-b">Handler</text><text x="747" y="106" class="t-small">nghiệp vụ,</text><text x="747" y="122" class="t-small">ngoại lệ</text><rect x="565" y="190" width="300" height="80" rx="10" class="f-warn-soft s-warn"/><text x="579" y="216" class="t-b">dependency_overrides</text><text x="579" y="238" class="t-small">get_repo trả kho giả luôn lỗi,</text><text x="579" y="254" class="t-small">get_clock trả giờ cố định</text><path d="M665 190 L665 146" class="f-none s-warn" stroke-width="2" marker-end="url(#py36-0-arrow)"/><rect x="15" y="190" width="520" height="80" rx="10" class="f-bg s-line dash"/><text x="29" y="216" class="t-b">Không có socket, cổng hay uvicorn</text><text x="29" y="238" class="t-small">Ngoại lệ trong app được ném lại vào test (raise_app_exceptions),</text><text x="29" y="254" class="t-small">lifespan không tự chạy, và không lỗi mạng nào xuất hiện ở đây.</text></svg>
</div>
<figcaption>Hình 1. Test API đi vào app qua ASGITransport. Chỗ thay bản giả chỉ là các dependency; phần còn lại là code thật.</figcaption>
</figure>

## Dựng client: AsyncClient, ASGITransport và lifespan

`httpx.AsyncClient` bình thường gửi request qua mạng; với `transport=httpx.ASGITransport(app=app)` nó gọi thẳng hàm ASGI của app, URL chỉ là chỗ điền cho đủ. Nền tảng nằm ở `tests/conftest.py` (trích):

```python
@pytest.fixture
def app() -> FastAPI:
    """App MỚI cho mỗi test: dependency_overrides và app.state không rò rỉ giữa các test."""
    application = create_app(make_settings())
    application.dependency_overrides[get_clock] = lambda: lambda: FIXED_NOW
    return application


@pytest.fixture
async def client(app: FastAPI) -> AsyncIterator[httpx.AsyncClient]:
    # ASGITransport KHÔNG chạy lifespan; tự mở bằng router.lifespan_context để app.state.repo tồn tại.
    async with app.router.lifespan_context(app):
        transport = httpx.ASGITransport(app=app)
        async with httpx.AsyncClient(transport=transport, base_url="http://test") as http:
            yield http
```

Ba quyết định trong 13 dòng này đều có lý do. **App mới cho mỗi test**: `dependency_overrides`, `app.state` và cả route thêm bằng tay đều gắn vào đối tượng app; dùng chung thì cái của test này sống sang test sau (phần kiểm chứng cho thấy bằng số). **Mở lifespan bằng tay**: `ASGITransport` chỉ gửi sự kiện `http`, không gửi `lifespan.startup`, nên mọi thứ dựng trong lifespan chưa tồn tại; fixture dùng `app.router.lifespan_context(app)`, cùng context manager mà router chạy khi nhận `lifespan.startup` từ uvicorn. **Đồng hồ qua override, cấu hình qua tham số**: `get_clock` là dependency nên test thay được bằng giờ cố định; `Settings` truyền thẳng vào `create_app`, không cần patch module hay biến môi trường.

Cấu hình pytest-asyncio nằm trong `pyproject.toml`: `asyncio_mode = "auto"` để hàm test và fixture `async def` chạy không cần `@pytest.mark.asyncio`. `asyncio_default_fixture_loop_scope = "function"` điều khiển vòng lặp sự kiện của fixture, còn vòng lặp của hàm test do `asyncio_default_test_loop_scope` quyết định (mặc định cũng `function`). Chế độ `strict` đòi đánh dấu từng test, hợp với dự án trộn nhiều framework async.

Hai hành vi của `ASGITransport` cần biết trước khi viết test. Một: nó ném lại ngoại lệ chưa xử lý của app vào test; `raise_app_exceptions=False` đổi thành response 500 như server thật, dùng khi chính nội dung 500 cần kiểm. Cả hai chế độ có test ở lab này. Hai: nó chờ app hoàn tất lời gọi ASGI và gom body, nên không trả response từng phần và chờ cả background task; API của lab này không có background task hay SSE nên điều đó được kiểm ở lab 35. Test SSE hay độ trễ 202 cần server thật.

Còn `TestClient` của Starlette? Nó là cầu nối đồng bộ: chạy app trong thread riêng, test viết bằng hàm thường (không `asyncio.gather` trực tiếp được), và chỉ chạy lifespan khi bọc bằng `with TestClient(app)`. Lab kiểm cả hai ca trên `GET /v1/trips/TN-0001`: không `with` thì cùng lỗi `'State' object has no attribute 'repo'`, có `with` thì 200 (`tests/test_trips.py`).

Chi tiết phiên bản: Starlette 1.10 đòi gói `httpx2` cho `starlette.testclient`; chỉ có `httpx` thì `TestClient` vẫn chạy nhưng in `StarletteDeprecationWarning`. Lab khai báo cả hai (`httpx` cho `AsyncClient`, `httpx2` cho `TestClient`) để bộ test sạch cảnh báo.

## Test hợp đồng: giá, tiền và lỗi

Hợp đồng báo giá copy nguyên từ lab 32: 201 kèm `Location`, có `quote_id`, body cấm trường lạ, tối đa 3 chữ số thập phân. Test đi qua API và kiểm hình dạng response, không chỉ con số (`tests/test_quotes.py`, trích):

```python
@pytest.mark.parametrize(
    ("distance", "fare"),
    [("0", "12000"), ("2", "12000"), ("2.001", "12005"), ("2.5", "14250"), ("5", "25500"), ("100", "453000")],
)
async def test_quote_prices_follow_the_spec(client, auth, distance, fare):
    response = await client.post("/v1/quotes", json={"distance_km": distance, "service": "ride"},
                                 headers=auth())

    assert response.status_code == 201
    body = response.json()
    assert body["fare"] == fare  # chuỗi, không phải số: so bằng "14250", không phải 14250
    assert body["currency"] == "VND"
```

Kỳ vọng viết sẵn từ đặc tả như bài 10. Điều mới là dòng chú thích: Decimal đi ra JSON dưới dạng **chuỗi**, nên không có `double` nào xen giữa để làm mất chính xác; ai đổi sang `float` sẽ thấy đỏ. Các test khác kiểm `Location: /v1/quotes/QT-0001`, đọc lại báo giá, khách khác đọc báo giá của mình ra 404 giống hệt báo giá không tồn tại, và `expires_at` đúng `2026-11-06T03:05:00Z` nhờ `get_clock`.

Phần lỗi quan trọng ngang phần thành công, vì client rẽ nhánh theo mã và `type`. Với 422, test kiểm đúng cặp `field` và `code` (ví dụ `body.distance_km`, `decimal_max_places`) và mỗi lỗi chỉ có ba khóa `field`, `code`, `message`, không có `input`. Thay vì lặp assertion, `tests/test_problems.py` giữ một bảng ca lỗi và một test kiểm chung:

```python
@pytest.mark.parametrize(("method", "url", "kwargs", "role", "status"),
                         [case[1:] for case in ERROR_CASES], ids=[case[0] for case in ERROR_CASES])
async def test_every_listed_error_is_problem_json(client, auth, method, url, kwargs, role, status):
    headers = auth(*role) if role else {}
    response = await client.request(method, url, headers=headers, **kwargs)

    assert response.status_code == status
    assert response.headers["content-type"] == "application/problem+json"
    body = response.json()
    assert REQUIRED <= body.keys()
    assert body["status"] == status  # trường status trong body phải khớp mã HTTP
    assert body["type"] == "about:blank" or body["type"].startswith(BASE)
    assert body["request_id"] == response.headers["x-request-id"]  # bài 33: nối được với log
```

Tám ca (thiếu token 401, thiếu quyền 403, ngoài miền 400, body sai 422, không có chuyến 404, sai trạng thái 409, route không có 404, sai phương thức 405) cùng phải ra `application/problem+json`. Hai ca cuối là lỗi của framework: nếu không có handler cho `StarletteHTTPException`, 404 và 405 ra `application/json` với `{"detail": ...}`; lab có handler đưa chúng về `type: about:blank`, `title` là cụm từ chuẩn của mã (`Not Found`, `Method Not Allowed`) và giữ header `Allow`, như lab 32. Giới hạn: bảng chỉ bảo vệ các ca đã liệt kê; thêm route hay exception mới mà quên thêm ca thì không test nào đỏ.

Những ca không thể dựng bằng kho thật mà không phá hạ tầng thì dùng kho giả qua override (`tests/test_trips.py`, trích):

```python
class DownRepo:
    async def get(self, trip_id: str) -> Trip | None:
        raise RepositoryUnavailable

async def test_repository_outage_becomes_503_with_retry_after(app, client, auth):
    app.dependency_overrides[get_repo] = lambda: DownRepo()
    response = await client.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))

    assert response.status_code == 503
    assert response.headers["retry-after"] == "5"
```

Cùng mẫu cho lỗi không lường trước: kho giả ném `RuntimeError("mật khẩu db = hunter2")`. Với `raise_app_exceptions=False`, test thấy 500 generic không chứa `hunter2`; với mặc định, test thấy chính `RuntimeError` bị ném lại.

## Xác thực: token thật và thay bộ kiểm token

Fixture `auth` ký token bằng `jwt.encode` với khóa của riêng test, không gọi hàm nào của app, nên bắt được cả lỗi phía app (đọc sai claim, quên kiểm chữ ký). Token đủ claim như lab 34 (`iss`, `aud`, `sub`, `scope`, `iat`, `exp`); tham số của fixture làm hỏng từng thứ một (`tests/conftest.py`, trích):

```python
def make(sub="CU-001", scope="customer", *, expires_in=timedelta(minutes=15), secret=TEST_SECRET,
         algorithm="HS256", issuer=ISSUER, audience=AUDIENCE, omit=()) -> dict[str, str]:
    now = datetime.now(UTC)
    claims = {"iss": issuer, "aud": audience, "sub": sub, "scope": scope, "iat": now, "exp": now + expires_in}
    for name in omit:
        del claims[name]
    return {"Authorization": f"Bearer {jwt.encode(claims, secret, algorithm=algorithm)}"}
```

Bảy kiểu token xấu (hết hạn, sai chữ ký, ký HS512, sai `aud`, sai `iss`, thiếu `iat`, `scope` là list) phải nhận **cùng một** câu trả lời 401 với `WWW-Authenticate: Bearer error="invalid_token"`. Đây là quyết định của bài 34: chỉ cho client biết token sai ở đâu là giúp kẻ dò token. Test so các body với nhau (bỏ `request_id`) và khẳng định giống hệt; lý do thật nằm trong log, và test đọc `caplog` để kiểm `reason=expired`, `reason=bad_algorithm`, `reason=missing_claim:iat`... Không gửi token thì 401 với `WWW-Authenticate: Bearer` trơn (RFC 6750). Token tài xế gọi báo giá nhận 403 kèm `Bearer error="insufficient_scope", scope="customer"`; `scope` là chuỗi cách nhau bởi dấu cách theo OAuth2.

`require_scope` theo lab 34 yêu cầu **đủ mọi** scope liệt kê, và dùng `Security` với `OAuth2PasswordBearer` nên scope hiện trong OpenAPI. Một test thêm route thử `require_scope("customer", "driver")`: token chỉ có `customer` ra 403, có cả hai ra 200. Cần «ít nhất một» thì đó là hàm khác tên khác (`require_any_scope`), không phải tham số của hàm này.

`dependency_overrides` đưa ra lựa chọn khác: bỏ việc ký và giải token. Chọn chỗ ghi đè quyết định test còn kiểm được gì. Lab 34 kiểm scope **bên trong** `current_principal`, nên hai cách ghi đè cho hai kết quả trái ngược (`tests/test_auth.py`, trích):

```python
async def test_fake_token_service_skips_signing_but_scope_rules_still_run(app, client):
    bearer = {"Authorization": "Bearer bat-ky-chuoi-nao"}  # oauth2_scheme vẫn cần một chuỗi để đọc
    app.dependency_overrides[get_token_service] = lambda: FakeTokens(Principal("DR-001", frozenset({"driver"})))
    forbidden = await client.post("/v1/quotes", json=QUOTE, headers=bearer)
    app.dependency_overrides[get_token_service] = lambda: FakeTokens(Principal("CU-001", frozenset({"customer"})))
    allowed = await client.post("/v1/quotes", json=QUOTE, headers=bearer)

    assert (forbidden.status_code, allowed.status_code) == (403, 201)


async def test_overriding_current_principal_removes_the_scope_check_too(app, client):
    app.dependency_overrides[current_principal] = lambda: Principal("DR-001", frozenset({"driver"}))
    response = await client.post("/v1/quotes", json=QUOTE)  # không header, scope driver, vẫn qua

    assert response.status_code == 201
```

Test đầu thay `get_token_service`, tầng chỉ giải token: không cần ký token thật mà `require_scope` vẫn chạy (driver ra 403). Test sau thay thẳng `current_principal`, tức tháo luôn hàng rào phân quyền: tài khoản `driver` không gửi header nào cũng tạo được báo giá. Lab giữ test thứ hai để ghi lại bẫy này. Quy tắc: mỗi route có ít nhất một test với token thật, ghi đè chỉ cho test nghiệp vụ không quan tâm danh tính, và ghi đè ở tầng thấp nhất còn giữ được quy tắc cần kiểm.

## Snapshot OpenAPI và hợp đồng schema

FastAPI sinh `/openapi.json` từ code, nên một thay đổi vô tình ở model sẽ đổi hợp đồng mà không ai quyết định. **Snapshot test** lưu bản chấp nhận được vào git (`tests/snapshots/openapi.json`), mỗi lần chạy so bản hiện tại với nó, và khi lệch thì in diff (`tests/test_openapi.py`, trích):

```python
async def test_openapi_matches_the_committed_snapshot(client, update_snapshots):
    actual = dump((await client.get("/openapi.json")).json())
    if update_snapshots:
        SNAPSHOT.write_text(actual, encoding="utf-8")
    expected = SNAPSHOT.read_text(encoding="utf-8")
    if actual != expected:
        diff = "".join(difflib.unified_diff(expected.splitlines(True), actual.splitlines(True),
                                            "snapshot", "hiện tại", n=1))
        pytest.fail(f"hợp đồng OpenAPI đã đổi; chạy lại với --update-snapshots nếu cố ý:\n{diff}", pytrace=False)
```

Cờ `--update-snapshots` do `conftest.py` đăng ký. Quy trình: test đỏ kèm diff, người sửa đọc diff, nếu cố ý thì chạy lại với cờ và **commit snapshot cùng thay đổi code**, để reviewer thấy hợp đồng đổi gì ngay trong pull request. Snapshot không nói thay đổi đó tốt hay xấu, chỉ buộc ai đó nhìn vào nó. Nâng FastAPI hoặc pydantic cũng có thể đổi cách sinh schema và làm snapshot đỏ dù hợp đồng không đổi; hãy đọc diff, đừng ghi đè theo phản xạ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Sơ đồ snapshot OpenAPI: đổi code rồi sinh OpenAPI hiện tại, so với snapshot lưu trong git. Nếu khớp thì test xanh. Nếu lệch thì test đỏ kèm diff, người review quyết định, và nếu cố ý thì chạy update-snapshots và commit snapshot cùng thay đổi code."><defs><marker id="py36-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="15" y="100" width="160" height="70" rx="10" class="f-pk-soft s-pk"/><text x="29" y="128" class="t-b">Đổi code</text><text x="29" y="150" class="t-small">model, route, response</text><path d="M175 135 L205 135" class="f-none s-phys" stroke-width="2" marker-end="url(#py36-1-arrow)"/><rect x="205" y="100" width="170" height="70" rx="10" class="f-pk-soft s-pk"/><text x="219" y="128" class="t-b">OpenAPI hiện tại</text><text x="219" y="150" class="t-small">app.openapi()</text><path d="M375 135 L405 135" class="f-none s-phys" stroke-width="2" marker-end="url(#py36-1-arrow)"/><rect x="405" y="100" width="170" height="70" rx="10" class="f-phys-soft s-phys"/><text x="419" y="128" class="t-b">So với snapshot</text><text x="419" y="150" class="t-small">file trong git</text><path d="M575 120 L615 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py36-1-arrow)"/><rect x="615" y="30" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="629" y="56" class="t-b">Khớp</text><text x="629" y="76" class="t-small">test xanh, hợp đồng không đổi</text><path d="M575 150 L615 200" class="f-none s-warn" stroke-width="2" marker-end="url(#py36-1-arrow)"/><rect x="615" y="150" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="629" y="176" class="t-b">Lệch: test đỏ kèm diff</text><text x="629" y="198" class="t-small">cố ý: --update-snapshots,</text><text x="629" y="216" class="t-small">commit snapshot cùng code</text><text x="629" y="238" class="t-small">vô tình: sửa code cho khớp</text></svg>
</div>
<figcaption>Hình 2. Snapshot không phán đúng sai; nó buộc mọi thay đổi của hợp đồng đi qua một lần review có diff.</figcaption>
</figure>

Snapshot bắt thay đổi, nhưng không nói app có tuân theo tài liệu hay không. Năm test còn lại trong `test_openapi.py` lấp chỗ đó: tập `operationId` đúng bốn tên (client sinh mã từ đây nên đổi tên là đổi hợp đồng); scope hiện trong tài liệu (`createQuote` đòi `customer`, `cancelTrip` đòi `dispatcher`, `getTrip` chỉ đòi token); mọi operation khai báo 401, operation có scope khai báo thêm 403; mọi lỗi 4xx và 5xx khai báo `application/problem+json` (FastAPI mặc định tả 422 bằng schema riêng và tự thêm 422 cho cả route chỉ có path `str`; `problems.py` sửa cho khớp handler). Và quan trọng nhất, response thật khớp schema đã tài liệu hóa:

```python
root = {"$ref": f"#/components/schemas/{schema_name}", "components": spec["components"]}
jsonschema.validate(body, root)
```

Gắn `components` vào gốc để `$ref` tự phân giải, rồi dùng `jsonschema` kiểm body thật của báo giá, chuyến và hai loại lỗi. Nếu tài liệu nói «`fare` là chuỗi» mà code trả số, test này đỏ ngay cả khi snapshot không đổi. Kiểm thử theo schema (property-based) làm điều này tự động trên mọi route; lab chỉ làm thủ công bốn ca để thấy cơ chế.

## Chạy và kiểm chứng

Từ gốc repository (lần đầu cần mạng để tải gói; không cần Docker):

```bash
uv run --project examples/toi-noi-python/36-fastapi-api-test-httpx pytest examples/toi-noi-python/36-fastapi-api-test-httpx -q
```

```text
...........................................................              [100%]
59 passed in 1.89s
```

59 test = 21 báo giá + 9 xác thực + 13 chuyến + 10 hình dạng lỗi + 6 hợp đồng OpenAPI. Chạy `-v` để xem tên (trích, bỏ đường dẫn lab ở đầu dòng):

```text
tests/test_auth.py::test_every_bad_token_gets_the_same_401_and_the_reason_goes_to_the_log PASSED
tests/test_auth.py::test_overriding_current_principal_removes_the_scope_check_too PASSED
tests/test_openapi.py::test_real_responses_conform_to_their_documented_schemas PASSED
tests/test_trips.py::test_unexpected_error_becomes_a_generic_500_without_leaking PASSED
tests/test_trips.py::test_testclient_without_with_has_the_same_trap PASSED
```

Test xanh chỉ đáng tin nếu nó có thể đỏ, nên ta phá có chủ ý ba lần rồi khôi phục. **Một:** thêm `rating: float | None = None` vào `TripOut`. Snapshot đỏ và diff chỉ ra đúng chỗ (trích; `snapshot` và `hiện tại` là tên do test đặt):

```text
hợp đồng OpenAPI đã đổi; chạy lại với --update-snapshots nếu cố ý:
--- snapshot
+++ hiện tại
@@ -200,2 +200,13 @@
             "title": "Driver Id"
+          },
+          "rating": {
+            "anyOf": [
+              {
+                "type": "number"
+              },
+              {
+                "type": "null"
+              }
+            ],
+            "title": "Rating"
```

Cùng lúc `test_customer_sees_own_trip` đỏ vì so cả dict response (`Left contains 1 more item: {'rating': None}`): `2 failed, 57 passed`. Hai tầng test bắt cùng một thay đổi theo hai cách. **Hai:** đổi route hủy từ `require_scope("dispatcher")` sang `Authenticated`. Bốn test đỏ, ba trong số đó thuộc tầng hợp đồng:

```text
FAILED tests/test_openapi.py::test_openapi_matches_the_committed_snapshot
FAILED tests/test_openapi.py::test_security_scopes_show_up_in_the_spec
FAILED tests/test_openapi.py::test_every_operation_documents_401_and_scoped_ones_also_403
FAILED tests/test_trips.py::test_customer_cannot_cancel - assert 200 == 403
4 failed, 55 passed in 0.59s
```

**Ba:** fixture `app` dùng chung một app dựng sẵn. Override và route thêm bằng tay của một test lọt sang các test sau (trích):

```text
FAILED tests/test_openapi.py::test_operation_ids_are_unique_and_stable
FAILED tests/test_problems.py::test_every_listed_error_is_problem_json[thi\u1ebfu token]
FAILED tests/test_quotes.py::test_created_quote_has_an_id_a_location_and_can_be_read_back
FAILED tests/test_trips.py::test_dispatcher_sees_every_trip - assert 404 == 200
8 failed, 51 passed in 0.37s
```

pytest in tên ca tiếng Việt dưới dạng escape Unicode (`\u1ebf` là «ế»). Ca `[thiếu token]` đỏ dù không đụng tới override: một test trong `test_auth.py` đã đặt override trên app dùng chung và không ai gỡ. `test_operation_ids_...` đỏ vì route `/_test/both` do test khác thêm vẫn còn, và `QT-0001` đã bị bộ đếm của test trước dùng mất. Triệu chứng xa nguyên nhân là lý do app mới mỗi test đáng vài mili giây.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `AttributeError: 'State' object has no attribute ...` | Client chưa mở lifespan: `app.router.lifespan_context(app)` hoặc `with TestClient(app)` |
| Test thấy ngoại lệ thay vì response 500 | `ASGITransport` mặc định ném lại; đặt `raise_app_exceptions=False` |
| Đỏ khi chạy cả bộ, xanh khi chạy riêng | `dependency_overrides` hoặc route thêm tay rò rỉ qua app dùng chung |
| `async def functions are not natively supported`, hoặc `requested an async fixture 'client', with no plugin or hook that handled it` | pytest-asyncio chưa bật hoặc không nạp: kiểm `asyncio_mode = "auto"` và plugin có được nạp (chạy với `-p no:asyncio` tái hiện đúng hai thông báo này) |
| `StarletteDeprecationWarning` về httpx | Thêm `httpx2` vào nhóm dev cho `TestClient` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Kho chuyến là một <code>dict</code> trong bộ nhớ, kèm kho giả để dựng lỗi; không test nào chạm PostgreSQL (bài 28). Test trong tiến trình không thấy TLS, timeout của proxy, nhiều tiến trình hay độ trễ mạng. Lab chỉ kiểm token chứ không cấp token (bài 34); production dùng IdP và khóa bất đối xứng. Snapshot chỉ bắt thay đổi tài liệu, không chứng minh mọi nhánh của route đã được test, và bốn ca kiểm schema chạy bằng tay không thay được kiểm thử theo schema tự động. Chạy trên một máy macOS, Python 3.14.7, fastapi 0.162.0, starlette 1.10.0, httpx 0.28.1, httpx2 2.16.0, pytest 9.2.0, pytest-asyncio 1.4.1; lúc đó load average khoảng 48 đến 52 nên số giây chỉ để tham khảo.</p>
</div>

**Bài tập 1:** `test_every_bad_token_gets_the_same_401...` có ca token hết hạn bằng `expires_in=-1s`. Ca này có thể chập chờn trên máy chậm không? Nếu đổi sang `+1s` rồi `await asyncio.sleep(1.1)` thì sao, và đồng hồ cố định `get_clock` có giúp được không?

**Bài tập 2:** Thêm route `GET /v1/trips/{trip_id}/receipt` (chỉ dispatcher). Bạn phải sửa những gì để toàn bộ bộ test vẫn xanh, và lần chạy đầu tiên test nào đỏ?

<details><summary>Xem lời giải</summary><p><b>1.</b> Ca <code>-1s</code> không chập chờn: fixture ký <code>exp = now + expires_in</code> với <code>now</code> là giờ thật, nên <code>exp</code> nằm ở quá khứ ngay khi ký và hết hạn không phụ thuộc tốc độ máy. Ca <code>+1s</code> rồi ngủ chậm và vẫn dựa vào đồng hồ thật. <code>get_clock</code> không giúp: nó chỉ chi phối <code>expires_at</code> của báo giá, còn <code>jwt.decode</code> đọc giờ hệ thống của chính nó. Muốn điều khiển thời gian của việc kiểm token phải đóng băng đồng hồ hệ thống bằng thư viện chuyên dụng, hoặc truyền <code>now</code> vào chỗ ký và kiểm như <code>TokenService.issue</code> của lab 34. <b>2.</b> Hai test đỏ ngay: <code>test_operation_ids_are_unique_and_stable</code> (tập operationId có thêm phần tử) và <code>test_openapi_matches_the_committed_snapshot</code> (diff thêm đường dẫn). Sửa: thêm <code>operation_id</code> và <code>responses</code> cho route, thêm tên mới vào tập kỳ vọng, chạy lại với <code>--update-snapshots</code> và commit snapshot cùng code. Quên khai 401 hoặc 403 thì <code>test_every_operation_documents_401_and_scoped_ones_also_403</code> đỏ. Nên viết thêm test hành vi (đúng quyền, sai quyền, không có chuyến), đừng chỉ trông vào snapshot.</p></details>

Bài trước: [bài 35](/articles/python-fastapi-upload-background-task-streaming). Bài tiếp: [bài 37](/articles/python-grpc-fraud-scoring-service).

## Tài liệu gốc

- [FastAPI: Testing](https://fastapi.tiangolo.com/tutorial/testing/).
- [FastAPI: Async Tests](https://fastapi.tiangolo.com/advanced/async-tests/).
- [FastAPI: Testing Dependencies with Overrides](https://fastapi.tiangolo.com/advanced/testing-dependencies/).
- [FastAPI: Testing Events (lifespan)](https://fastapi.tiangolo.com/advanced/testing-events/).
- [HTTPX: Async Support, gọi thẳng vào ứng dụng ASGI](https://www.python-httpx.org/async/).
- [pytest-asyncio](https://pytest-asyncio.readthedocs.io/en/stable/).

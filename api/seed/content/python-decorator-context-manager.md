---
slug: python-decorator-context-manager
title: "Decorator và context manager: retry, đo thời gian, transaction"
summary: "Bài 17: closure, functools.wraps, decorator có tham số retry và timed, thứ tự chồng decorator; @contextmanager, class và ExitStack cho transaction giả. Kiểm thử bằng FakeTimer, không ngủ thật."
category: programming
level: beginner
tags: [python, toi-noi, decorator, context-manager]
language: vi
---

Dịch vụ báo giá mà [Tới Nơi](/articles/du-an-mau-toi-noi) gọi qua mạng thỉnh thoảng trả lỗi 503, tức là «tạm thời không phục vụ»,. Điều phối muốn thử lại vài lần trước khi báo lỗi cho khách, và muốn ghi lại mỗi lần gọi mất bao lâu. Ở bước gán tài xế còn một việc khác: ghi `driver_id` vào chuyến, đổi trạng thái sang `accepted` và tăng số chuyến đang mở của tài xế. Ba thao tác này phải xong cả ba hoặc không đổi gì; thao tác thứ ba lỗi mà hai thao tác đầu đã ghi thì dữ liệu hỏng âm thầm.

Bài này dùng hai công cụ của Python để tách «hành vi bao quanh» khỏi nghiệp vụ thay vì chép `try/except` vào mọi nơi gọi: **decorator** bọc cả một hàm, **context manager** bọc một khối lệnh.

Đây là bài 17 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần quen với hàm là một giá trị, exception và logging (bài 08), và đồng hồ thay thế được để kiểm thử (bài 14). `Timer` của bài này là Protocol nhỏ riêng, không phải `Timer.now()` (giờ thực aware) ở bài 14, vì ở đây cần đọc đồng hồ đo (`perf_counter`) và chờ (`sleep`).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hàm gọi dịch vụ báo giá ném <code>ValueError</code> vì khoảng cách 150 km nằm ngoài miền 0..100 km. Decorator <code>retry(times=3)</code> có nên gọi lại hàm đó đủ 3 lần không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Lỗi nghiệp vụ lặp lại sẽ cho cùng kết quả ở lần thứ hai và thứ ba, chỉ tốn thời gian và làm log rối. Retry chỉ nên áp dụng cho lỗi tạm thời (dịch vụ bận, mất kết nối), nên decorator cần một danh sách loại lỗi được phép thử lại, không phải «mọi Exception».</p></details>
</div>

## Hàm bọc hàm, khối `with` bọc khối lệnh

Decorator là một hàm nhận một hàm và trả về một hàm. Dòng `@deco` đặt trên `def f` chỉ là cú pháp rút gọn cho `f = deco(f)` viết ngay sau khi `def` chạy xong; nó khả thi vì hàm là đối tượng bình thường (PEP 318 đưa cú pháp `@` vào ngôn ngữ).

Context manager là đối tượng có hai phương thức `__enter__` và `__exit__`. Câu lệnh `with` gọi `__enter__` trước khối lệnh và **luôn** gọi `__exit__` khi rời khối (miễn `__enter__` đã chạy xong), dù khối kết thúc bình thường, `return`, `break` hay có exception.

Hai công cụ khác nhau ở phạm vi: decorator gắn vào **định nghĩa** hàm nên mọi lời gọi đều đi qua lớp bọc; `with` gắn vào **chỗ viết** nên chỉ khối đó được bọc. Ai đã đọc loạt C# sẽ thấy dáng quen: `using` tương ứng `with`, còn middleware cũng là «bọc handler bằng handler». Decorator của Python không phải pattern Decorator của GoF: GoF bọc *đối tượng* cùng interface lúc chạy (xem [nhóm cấu trúc](/articles/nhom-cau-truc-structural)), còn `@` là cú pháp áp lên hàm hoặc class lúc định nghĩa.

## Closure và `functools.wraps`

Muốn viết decorator, trước hết cần hiểu **closure**: hàm nhớ biến của nơi nó được định nghĩa, kể cả sau khi hàm bên ngoài đã trả về. Code của bài nằm ở `examples/toi-noi-python/17-decorator-context-manager/main.py`.

```python
def make_fare_quoter(policy: FarePolicy) -> Callable[[Decimal], Decimal]:
    def quote(distance_km: Decimal) -> Decimal:
        return calculate_fare(distance_km, policy)   # `policy` là biến tự do, lấy từ hàm ngoài
    return quote

quote_5000 = make_fare_quoter(FarePolicy(price_per_extra_km=Decimal("5000")))
quote_5000(Decimal("5"))              # 27000: nhớ chính sách 5.000 VND/km vượt
quote_5000.__code__.co_freevars       # ('policy',)
quote_5000.__closure__[0].cell_contents   # đúng FarePolicy vừa truyền vào
```

`policy` không phải tham số hay biến cục bộ của `quote`; Python gọi nó là **biến tự do** và giữ trong một «ô» (cell) gắn vào hàm. Với chuyến 5 km, chính sách chuẩn cho 25.500 VND còn chính sách 5.000 VND cho 27.000 VND. Closure giữ **biến**, không chép giá trị lúc tạo, và đó là nguồn của một lỗi kinh điển:

```python
late = [lambda: i for i in range(3)]
fixed = [lambda i=i: i for i in range(3)]   # i=i chốt giá trị ngay lúc định nghĩa lambda
[f() for f in late]    # [2, 2, 2]
[f() for f in fixed]   # [0, 1, 2]
```

Ba lambda của `late` dùng chung một ô `i`; lúc gọi, vòng lặp đã xong và `i` bằng 2. Giá trị mặc định của tham số được tính ngay lúc `lambda` được tạo, nên `fixed` giữ riêng từng số. Bài 18 có cách sạch hơn là `functools.partial`.

Bây giờ là decorator đầu tiên: ghi lại tên mỗi hàm được gọi. Phiên bản đầu quên một chi tiết:

```python
def record_naive[**P, R](func: Callable[P, R]) -> Callable[P, R]:
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        CALL_LOG.append(func.__name__)
        return func(*args, **kwargs)
    return wrapper        # thiếu functools.wraps

def record[**P, R](func: Callable[P, R]) -> Callable[P, R]:
    @functools.wraps(func)    # sao chép tên, docstring... từ func sang wrapper
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        CALL_LOG.append(func.__name__)
        return func(*args, **kwargs)
    return wrapper
```

Phần `[**P, R]` là cú pháp generic của PEP 695 (Python 3.12 trở lên): `P` là `ParamSpec` giúp trình kiểm kiểu biết hàm sau khi bọc vẫn có chữ ký gốc.

Áp hai decorator lên cùng hàm `price_trip` (có docstring «Báo giá một chuyến theo chính sách chuẩn.»), ta thấy khác biệt:

```text
naive_name              : wrapper
naive_doc               : None
good_name               : price_trip
good_doc                : Báo giá một chuyến theo chính sách chuẩn.
good_wrapped_is_original: True
```

Thiếu `wraps`, hàm sau khi bọc tự xưng là `wrapper` và mất docstring, nên mọi dòng log hay công cụ dựa vào `__name__` và `__doc__` (như `help()`) đều đọc nhầm. `functools.wraps` tự nó là một decorator có tham số. Nó sao chép `__module__`, `__name__`, `__qualname__`, `__doc__`, `__type_params__` và `__annotate__` (hàm tính annotation lười của Python 3.14; danh sách in được từ `functools.WRAPPER_ASSIGNMENTS`), cập nhật `__dict__`, và gắn `__wrapped__` trỏ về hàm gốc; `inspect.signature` đi theo `__wrapped__` để báo đúng chữ ký ban đầu. Thói quen nên có: mọi decorator bọc hàm đều dùng `@functools.wraps(func)`.

## `retry` và `timed`: decorator có tham số

`@retry(times=3, delay=0.5)` khác `@record` ở chỗ có ngoặc và đối số. Python tính `retry(times=3, delay=0.5)` trước; kết quả phải là một decorator thật sự, rồi mới áp lên hàm. Vì vậy decorator có tham số có **ba tầng hàm**: tầng ngoài nhận cấu hình, tầng giữa nhận hàm, tầng trong là `wrapper` chạy mỗi lần gọi.

```python
def retry[**P, R](times: int = 3, delay: float = 0.0, backoff: float = 1.0, *,
                  on: tuple[type[BaseException], ...] = (PricingUnavailable,),
                  timer: Timer = SYSTEM_TIMER) -> Callable[[Callable[P, R]], Callable[P, R]]:
    if isinstance(times, bool) or not isinstance(times, int):
        raise TypeError("retry cần ngoặc: viết @retry(...) hoặc @retry(), không viết @retry trần")

    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            for attempt in range(1, times + 1):
                try:
                    return func(*args, **kwargs)
                except on as exc:
                    if attempt == times:
                        raise                       # hết lượt: ném lại đúng lỗi gốc
                    wait = delay * backoff ** (attempt - 1)
                    LOGGER.warning("%s lần %d/%d lỗi (%s), chờ %.1f s", func.__name__, attempt, times, exc, wait)
                    timer.sleep(wait)
            raise AssertionError("không tới được đây")
        return wrapper
    return decorator
```

(Bản rút gọn; bản lab có docstring và kiểm `times >= 1`.) Bốn quyết định đáng nói. Một, `times` là **tổng số lần gọi** tính cả lần đầu, nên `times=3` là một lần gọi và tối đa hai lần thử lại. Hai, chỉ lỗi trong `on` mới được thử lại; mặc định là `PricingUnavailable`, lớp ta đặt cho lỗi tạm thời của dịch vụ báo giá. Ba, hết lượt thì `raise` trần, giữ nguyên exception và traceback gốc. Bốn, quên ngoặc là lỗi dễ mắc: viết `@retry` trần thì hàm bị truyền vào chỗ của `times`. Không có dòng kiểm tra đầu hàm, lỗi chỉ bung ra xa chỗ gây ra; có dòng đó, nó bung ngay lúc import.

Để kiểm thử mà không chờ thật, `retry` nhận `timer`. Lab có `SystemTimer` (dùng `time.sleep`) và `FakeTimer`, nơi `sleep` chỉ ghi lại thời lượng và cộng vào thời gian đã trôi. `FlakyPricing` là dịch vụ giả: `fail_first` lần gọi đầu ném `PricingUnavailable`, mỗi lần gọi tốn 0,2 giây trên đồng hồ giả. Kết quả thật của ba tình huống:

```text
ok_result               : 14250
ok_calls                : 3
ok_sleeps               : [0.5, 1.0]
ok_logs                 : ['WARNING price lần 1/3 lỗi (503 lần 1), chờ 0.5 s', 'WARNING price lần 2/3 lỗi (503 lần 2), chờ 1.0 s']
exhausted_error         : 503 lần 3
exhausted_calls         : 3
exhausted_sleeps        : [0.5, 0.5]
domain_error            : khoảng cách ngoài miền 0..100 km: 150
domain_calls            : 1
domain_sleeps           : []
bare_error              : retry cần ngoặc: viết @retry(...) hoặc @retry(), không viết @retry trần
```

Với `delay=0.5, backoff=2.0`, hai lần chờ là 0,5 giây rồi 1,0 giây. Lần gọi thứ ba thành công, trả 14.250 VND cho chuyến 2,5 km. Khi dịch vụ hỏng cả ba lần (`backoff` mặc định bằng 1), `PricingUnavailable` lần ba tới tay người gọi nguyên vẹn sau hai lần chờ 0,5 giây. Còn `ValueError` của khoảng cách 150 km không nằm trong `on`, nên hàm chỉ được gọi **một** lần, không chờ gì. Ba ca chạy tức thì vì `FakeTimer` không ngủ; với `time.sleep` thật, riêng hai ca đầu đã tốn 2,5 giây mỗi lần chạy test.

Decorator thứ hai, `timed`, ghi thời gian mỗi lần gọi. Nó dùng `finally` để vẫn ghi khi hàm ném lỗi:

```python
def timed(*, timer: Timer = SYSTEM_TIMER, label: str | None = None):
    def decorator(func):
        name = label or func.__name__
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            start = timer.perf_counter()
            try:
                return func(*args, **kwargs)
            finally:
                LOGGER.info("%s mất %.3f s", name, timer.perf_counter() - start)
        return wrapper
    return decorator
```

(Bỏ chú thích kiểu và cả `[**P, R]` cho vừa khổ; bản lab có đủ.) Khi chồng nhiều decorator, **thứ tự quyết định điều được đo**. `@a` trên `@b` trên `def f` nghĩa là `f = a(b(f))`: lời gọi đi vào wrapper của `a`, rồi của `b`, rồi mới tới `f`. Lab dựng hai thứ tự bằng lời gọi hàm thường (nhắc lại `@` chỉ là cú pháp) trên cùng dịch vụ lỗi hai lần:

```python
quote = timing(retrying(backend.price))   # timed ngoài, retry trong
quote = retrying(timing(backend.price))   # retry ngoài, timed trong
```

```text
timed_outside : ['INFO price mất 2.100 s']
timed_inside  : ['INFO price mất 0.200 s', 'INFO price mất 0.200 s', 'INFO price mất 0.200 s']
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hai thứ tự chồng decorator trên cùng một hàm price. Hàng trên: timed bọc ngoài retry, ghi một dòng log 2,100 giây gồm ba lần gọi và hai lần chờ. Hàng dưới: retry bọc ngoài timed, ghi ba dòng log, mỗi dòng 0,200 giây, không tính thời gian chờ."><defs><marker id="py17-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="22" class="t-small">lời gọi đi từ lớp ngoài vào trong; lớp ngoài nhìn thấy cả thời gian chờ của lớp trong</text><rect x="20" y="34" width="540" height="104" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">timed</text><text x="36" y="76" class="t-small">đo trước và sau</text><rect x="190" y="46" width="360" height="80" rx="10" class="f-pk-soft s-pk"/><text x="204" y="70" class="t-b">retry</text><text x="204" y="88" class="t-small">gọi lại khi lỗi tạm thời</text><rect x="360" y="58" width="180" height="56" rx="8" class="f-warn-soft s-warn"/><text x="374" y="82" class="t-mono">price()</text><text x="374" y="100" class="t-small">tốn 0,2 s mỗi lần</text><path d="M565 86 L585 86" class="f-none s-phys" stroke-width="2" marker-end="url(#py17-0-arrow)"/><text x="595" y="68" class="t-b">1 dòng log</text><text x="595" y="88" class="t-small">price mất 2,100 s</text><text x="595" y="106" class="t-small">gồm 3 lần gọi và 2 lần chờ</text><rect x="20" y="158" width="540" height="104" rx="10" class="f-pk-soft s-pk"/><text x="36" y="182" class="t-b">retry</text><text x="36" y="200" class="t-small">gọi lại khi lỗi</text><rect x="190" y="170" width="360" height="80" rx="10" class="f-phys-soft s-phys"/><text x="204" y="194" class="t-b">timed</text><text x="204" y="212" class="t-small">đo từng lần gọi</text><rect x="360" y="182" width="180" height="56" rx="8" class="f-warn-soft s-warn"/><text x="374" y="206" class="t-mono">price()</text><text x="374" y="224" class="t-small">tốn 0,2 s mỗi lần</text><path d="M565 210 L585 210" class="f-none s-phys" stroke-width="2" marker-end="url(#py17-0-arrow)"/><text x="595" y="192" class="t-b">3 dòng log</text><text x="595" y="212" class="t-small">mỗi dòng price mất 0,200 s</text><text x="595" y="230" class="t-small">không tính thời gian chờ giữa các lần</text></svg>
</div>
<figcaption>Hình 1. Cùng ba thành phần, hai thứ tự bọc cho hai con số khác nhau: lớp đo thời gian ở ngoài thấy tổng thời gian người dùng chờ, ở trong thấy từng lần gọi dịch vụ.</figcaption>
</figure>

Con số nào đúng tùy câu hỏi: khách chờ báo giá bao lâu thì đo ở ngoài (2,100 giây = 3 × 0,2 + 0,5 + 1,0); dịch vụ chậm bao nhiêu mỗi lần gọi thì đo ở trong, không lẫn thời gian ta tự chờ.

Về retry: nó chỉ an toàn khi thao tác **idempotent**, tức gọi thêm lần nữa không đổi kết quả. Báo giá là đọc nên an toàn; tạo chuyến hay thu tiền thì không, vì lần gọi «lỗi» có thể đã thực hiện xong trước khi mạng đứt. Loạt C# bàn chuyện này trong bài [cache, timeout, retry và rate limiting](/articles/dotnet10-cache-timeout-retry-rate-limit); phía Python là chủ đề bài 25.

## Context manager: transaction giả bằng `@contextmanager`

Quay lại bài toán gán tài xế. Cách ngây thơ ghi thẳng vào dữ liệu theo từng bước:

```python
def assign_driver_no_tx(db, trip_id, driver_id):
    trip = db["trips"][trip_id]
    trip["driver_id"] = driver_id
    trip["status"] = TripStatus.ACCEPTED.value
    db["drivers"][driver_id]["open_trips"] += 1     # KeyError nếu tài xế không tồn tại
```

Gọi với tài xế `DR-999` không tồn tại, hai dòng đầu đã ghi xong khi dòng ba ném `KeyError`: chuyến `TN-0002` nằm lại ở `accepted` với tài xế không có thật. Trạng thái dở dang như vậy là loại hỏng dữ liệu khó lần ra nhất.

`contextlib.contextmanager` cho phép viết context manager bằng một **generator có đúng một `yield`**: đoạn trước `yield` đóng vai `__enter__`, giá trị được `yield` gán cho biến sau `as`, đoạn sau `yield` đóng vai `__exit__`. Nếu thân `with` ném exception, nó được ném lại ngay tại dòng `yield`, nên `try/except` bao quanh `yield` bắt được.

```python
@contextmanager
def transaction(db: TripDb) -> Iterator[TripDb]:
    working = copy.deepcopy(db)          # làm việc trên bản sao
    try:
        yield working                    # thân khối with chạy ở đây
    except Exception:
        LOGGER.info("rollback")          # bỏ working: db gốc chưa bị đụng
        raise                            # vẫn ném lên: người gọi phải biết đã thất bại
    else:
        db.clear()
        db.update(working)               # commit: thay db bằng bản đã sửa
        LOGGER.info("commit")

def assign_driver(db, trip_id, driver_id):
    with transaction(db) as tx:
        trip = tx["trips"][trip_id]
        trip["driver_id"] = driver_id
        trip["status"] = TripStatus.ACCEPTED.value
        tx["drivers"][driver_id]["open_trips"] += 1
```

Dùng `deepcopy` vì `db` là dict lồng dict: bản sao nông chỉ chép dict ngoài cùng, các dict chuyến bên trong vẫn dùng chung và thay đổi lọt vào bản gốc, làm rollback vô nghĩa. Nhánh `else` chỉ chạy khi không có exception.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Luồng của context manager transaction: vào khối with thì tạo bản sao working bằng deepcopy, thân khối sửa trên working, rồi rẽ hai hướng. Không có lỗi thì commit bằng cách thay db bằng working. Có Exception thì rollback bằng cách bỏ working, db giữ nguyên, và ném lỗi lên cho người gọi."><defs><marker id="py17-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py17-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="90" width="200" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="116" class="t-b">trước yield</text><text x="34" y="138" class="t-mono">working = deepcopy(db)</text><text x="34" y="156" class="t-small">tạo bản sao để sửa</text><rect x="270" y="90" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="284" y="116" class="t-b">thân khối with</text><text x="284" y="138" class="t-mono">tx = working</text><text x="284" y="156" class="t-small">sửa trên bản sao</text><rect x="520" y="26" width="340" height="76" rx="10" class="f-phys-soft s-phys"/><text x="534" y="50" class="t-b">không lỗi: nhánh else</text><text x="534" y="72" class="t-mono">db.clear(); db.update(working)</text><text x="534" y="90" class="t-small">commit: db nhận trạng thái mới</text><rect x="520" y="150" width="340" height="94" rx="10" class="f-warn-soft s-warn"/><text x="534" y="174" class="t-b">có Exception: nhánh except</text><text x="534" y="196" class="t-small">bỏ working, db giữ nguyên (rollback)</text><text x="534" y="216" class="t-small">raise: người gọi biết việc thất bại</text><path d="M220 130 L268 130" class="f-none s-phys" stroke-width="2" marker-end="url(#py17-1-arrow)"/><path d="M460 114 C 490 114, 490 64, 518 64" class="f-none s-phys" stroke-width="2" marker-end="url(#py17-1-arrow)"/><path d="M460 146 C 490 146, 490 196, 518 196" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py17-1-warn)"/><text x="20" y="232" class="t-small">yield trao working cho biến sau as; db gốc chỉ đổi ở nhánh commit.</text></svg>
</div>
<figcaption>Hình 2. Hai lối ra của transaction giả: đường liền là kết thúc êm và commit, đường đứt là có lỗi nên bỏ bản sao và ném lỗi tiếp.</figcaption>
</figure>

Kết quả thật khi gán `TN-0001` cho `DR-001` (thành công), rồi `TN-0002` cho `DR-999` (lỗi), và cùng việc đó không có transaction:

```text
after_commit                : ({'status': 'accepted', 'driver_id': 'DR-001'}, {'open_trips': 1})
rollback_error              : KeyError('DR-999')
db_unchanged_after_rollback : True
trip2_after_rollback        : {'status': 'requested', 'driver_id': None}
tx_logs                     : ['INFO commit', 'INFO rollback']
trip2_without_tx            : {'status': 'accepted', 'driver_id': 'DR-999'}
```

Bản có `transaction` để `TN-0002` nguyên trạng `requested` và `db` bằng đúng bản chụp trước đó; bản không có để lại trạng thái dở dang. Transaction chỉ bảo đảm «cả khối hoặc không gì», không tự kiểm tra việc có nên làm khối đó không (xem bài tập 2).

## Class, `ExitStack` và các bẫy

Khi cần giữ trạng thái, ta viết context manager dạng class. `DriverLock` giữ chỗ một tài xế trong tập `busy` để hai lượt điều phối chạy nối tiếp nhau không gán cùng một người:

```python
class DriverLock:
    def __init__(self, busy: set[str], driver_id: str) -> None:
        self.busy, self.driver_id = busy, driver_id

    def __enter__(self) -> str:
        if self.driver_id in self.busy:
            raise DriverBusy(self.driver_id)
        self.busy.add(self.driver_id)
        return self.driver_id

    def __exit__(self, exc_type, exc, tb) -> None:
        self.busy.discard(self.driver_id)    # trả chỗ dù thân with có lỗi hay không
```

`__exit__` nhận ba đối số mô tả exception đang bay (cả ba là `None` nếu khối kết thúc êm). Giá trị trả về quyết định số phận exception: trả **giá trị đúng** (như `True`) là «đã xử lý» và exception bị nuốt; trả `None` hoặc `False` thì nó tiếp tục bay. `DriverLock` trả `None` vì ta chỉ dọn dẹp, không che lỗi. Muốn nuốt lỗi có chủ ý, dùng `contextlib.suppress(KeyError)`: chỉ đúng loại chỉ định bị nuốt, dòng sau chỗ lỗi trong khối không chạy, còn `ValueError` vẫn bay lên.

Khi số tài nguyên chỉ biết lúc chạy (giữ chỗ cho cả danh sách tài xế của một lô), ta không viết sẵn được số `with` lồng nhau. `ExitStack` gom chúng lại:

```python
busy = {"DR-002"}                  # DR-002 đang bận ở nơi khác
with ExitStack() as stack:
    for driver in ("DR-001", "DR-002", "DR-003"):
        stack.enter_context(DriverLock(busy, driver))   # DR-002 làm DriverBusy nổ ra ở đây
```

Kết quả thật: `stack_error: DriverBusy(DR-002)` và `busy_after_failed_stack: ['DR-002']`. `DR-001` đã giữ được trước đó nhưng `ExitStack` thoát theo thứ tự ngược và trả lại, nên chỉ còn `DR-002` của người khác. Thứ tự thoát luôn ngược thứ tự vào, giống ngăn xếp, kể cả với `with a, b:`; lab ghi `['enter A', 'enter B', 'body', 'exit B', 'exit A']`.

Một bẫy riêng của `@contextmanager`: đoạn sau `yield` **không** tự chạy khi thân `with` lỗi, vì exception được ném tại `yield`. Muốn dọn dẹp chắc chắn phải đặt trong `finally`. Lab chạy hai generator, `leaky` (dọn dẹp ngay sau `yield`) và `safe` (có `try/finally`), cùng một thân `with` ném lỗi:

```text
leaky   : ['open']
safe    : ['open', 'close']
```

Dòng `close` của `leaky` bị bỏ qua: tài nguyên rò.

Chọn dạng nào? Generator với `@contextmanager` gọn khi logic là «chuẩn bị, trao giá trị, dọn dẹp». Class hợp khi đối tượng cần thêm phương thức ngoài `with`, cần xét loại exception trong `__exit__`, hoặc cần kế thừa.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Log và traceback hiện tên `wrapper`, `help()` mất docstring | Decorator thiếu `@functools.wraps(func)` |
| `TypeError` ngay lúc import module khi dùng `@retry` | Viết `@retry` trần; cần `@retry()` hoặc `@retry(times=...)` |
| Hàm bị gọi 3 lần với lỗi nghiệp vụ (`ValueError`), log đầy cảnh báo | Tham số `on` quá rộng (`Exception`); chỉ liệt kê lỗi tạm thời |
| Tài nguyên không được trả khi thân `with` lỗi, hoặc rollback không chạy | Generator `@contextmanager` thiếu `try/finally` hoặc `except` quanh `yield` |
| Lỗi biến mất không dấu vết | `__exit__` trả giá trị đúng, hoặc `suppress` bắt loại lỗi quá rộng |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn so với production</span>
<p>Transaction trong bài chỉ là sao chép cả dict rồi ghi đè: một process, một luồng, không cách ly giữa hai người ghi, không bền khi tiến trình chết, và chi phí sao chép tăng theo dữ liệu. Transaction thật nằm ở database (bài 24, 25). <code>retry</code> chưa có jitter (ngẫu nhiên hóa thời gian chờ để nhiều client không thử lại cùng lúc), chưa giới hạn tổng thời gian, và chỉ hợp với thao tác idempotent; <code>FakeTimer</code> không mô phỏng timeout giữa chừng một lời gọi. <code>DriverLock</code> chỉ an toàn khi dùng tuần tự trong một luồng, vì <code>in busy</code> rồi <code>add</code> không nguyên tử; nhiều luồng cần <code>threading.Lock</code> (bài 23), nhiều server cần khóa phân tán hoặc ràng buộc ở database. Số tài xế, số chuyến và độ trễ 0,2 giây đều là số giả.</p>
</div>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/17-decorator-context-manager examples/toi-noi-python/17-decorator-context-manager/main.py
uv run --project examples/toi-noi-python/17-decorator-context-manager examples/toi-noi-python/17-decorator-context-manager/main.py --verify
```

Lệnh đầu in từng nhóm kết quả mà bài đã trích (closure, `wraps`, retry, thứ tự decorator, transaction, lock và `ExitStack`, thứ tự thoát), chạy dưới một giây và không phụ thuộc máy vì không đo thời gian thật. Lệnh thứ hai kiểm 21 ca gồm đúng những hành vi trên: từ late binding, `wraps`, ba tình huống retry và hai thứ tự chồng, đến commit, rollback, trả chỗ khi lỗi, `ExitStack`, `suppress` và bẫy thiếu `finally`. Kết quả cuối:

```text
OK 21/21
```

Sai ca nào, chương trình in tên ca đó cùng `FAIL n/21` và thoát với mã 1.

**Bài tập 1:** Viết `timed_block(label, timer)` bằng `@contextmanager` để đo thời gian một khối lệnh (thay vì cả hàm), ghi log kể cả khi khối ném lỗi. Vì sao phải có `finally`?

**Bài tập 2:** Gọi `assign_driver(db, "TN-0001", "DR-001")` hai lần liên tiếp thì `open_trips` của tài xế là bao nhiêu, và đó có phải điều ta muốn? Sửa thế nào trong khuôn khổ bài này?

<details><summary>Xem lời giải</summary><p>(1) Dùng khung của <code>timed</code> ở dạng generator: <code>start = timer.perf_counter()</code>, rồi <code>try: yield</code>, và <code>finally: LOGGER.info("%s mất %.3f s", label, timer.perf_counter() - start)</code>. Với <code>FakeTimer</code>, khối <code>with timed_block("assign", c): c.advance(0.4); raise RuntimeError</code> vẫn ghi <code>INFO assign mất 0.400 s</code>. Cần <code>finally</code> vì exception của thân <code>with</code> được ném lại tại <code>yield</code>; dòng log đứng sau <code>yield</code> không có <code>finally</code> sẽ bị bỏ qua đúng lúc ta cần số đo nhất. (2) <code>open_trips</code> thành <b>2</b>: transaction chỉ bảo đảm khối chạy trọn vẹn hoặc không chạy, không kiểm tra khối có hợp lệ. Thêm vào đầu khối <code>with</code> một kiểm tra: nếu chuyến không còn <code>requested</code> thì <code>raise ValueError</code>, và lỗi đó làm transaction rollback. Với dữ liệu thật, kiểm tra phải nằm ở database (ràng buộc hoặc cột version) vì hai process có thể cùng thấy <code>requested</code>; bài 25 sẽ làm.</p></details>

Bài trước: [bài 16](/articles/python-match-pattern-matching-none). Bài tiếp: [bài 18](/articles/python-functools-itertools-pipeline).

## Tài liệu gốc

- [functools: higher-order functions and operations on callable objects](https://docs.python.org/3.14/library/functools.html) (có `wraps`, `partial`, `lru_cache`).
- [contextlib: utilities for with-statement contexts](https://docs.python.org/3.14/library/contextlib.html) (`contextmanager`, `ExitStack`, `suppress`).
- [Data model: with statement context managers](https://docs.python.org/3.14/reference/datamodel.html#with-statement-context-managers).
- [PEP 318: Decorators for Functions and Methods](https://peps.python.org/pep-0318/).
- [PEP 343: The "with" Statement](https://peps.python.org/pep-0343/).
- [typing.ParamSpec](https://docs.python.org/3.14/library/typing.html#typing.ParamSpec).

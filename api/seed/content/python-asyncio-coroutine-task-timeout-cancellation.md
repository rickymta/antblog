---
slug: python-asyncio-coroutine-task-timeout-cancellation
title: "asyncio: coroutine, Task, timeout và hủy: gọi dịch vụ báo giá"
summary: "Bài 21: gọi mười báo giá đồng thời bằng gather và TaskGroup, giới hạn chờ bằng asyncio.timeout, hủy an toàn với CancelledError, không chặn event loop bằng to_thread. Có số đo thật và lab --verify."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, asyncio, concurrency]
language: vi
---

Màn hình xác nhận chuyến của [Tới Nơi](/articles/du-an-mau-toi-noi) hiện báo giá cho mười điểm đến gợi ý cùng lúc. Mỗi báo giá là một lời gọi sang service `pricing`, mất khoảng 0,15 đến 0,25 giây qua mạng. Gọi lần lượt thì mười lời gọi mất gần 2 giây, đủ để khách thấy vòng quay rồi thoát app. Gọi cả mười cùng lúc thì chỉ mất bằng lời gọi chậm nhất. Nhưng «cùng lúc» kéo theo những câu hỏi mới: một lời gọi lỗi thì chín lời còn lại ra sao, service treo thì chờ tới bao giờ, khách đóng màn hình giữa chừng thì ai dọn phần đang giữ? Và nguy hiểm nhất, một dòng code chặn có thể làm đứng cả service, không riêng một yêu cầu.

Đây là bài 21 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần quen [exception](/articles/python-exception-debug-logging) (bài 08) và khối `with` (bài 09). Bài này mở đầu phần 03; bài trước, [đóng gói và cấu trúc dự án](/articles/python-dong-goi-ruff-pre-commit-cau-truc-du-an), kết thúc phần 02.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta chạy <code>asyncio.gather(a, b, c)</code>. Lời gọi <code>a</code> ném lỗi sau 0,05 giây, còn <code>b</code> và <code>c</code> cần 0,3 giây. Khi <code>gather</code> ném lỗi ra cho người gọi, <code>b</code> và <code>c</code> đã bị hủy chưa?</p>
<details><summary>Xem đáp án</summary><p><b>Chưa.</b> <code>gather</code> chỉ chuyển lỗi đầu tiên cho người gọi; các Task còn lại vẫn chạy tới hết 0,3 giây, và kết quả của chúng bị bỏ đi. <code>TaskGroup</code> thì hủy chúng ngay. Mục «Một lời gọi lỗi» chạy cả hai và in ra sự khác biệt.</p></details>
</div>

## Coroutine, event loop và Task

`async def` định nghĩa một **coroutine function**. Gọi nó không chạy thân hàm mà trả về một **coroutine object**, thứ chỉ chạy khi có ai đó `await` nó hoặc đưa nó vào event loop. **Event loop** là vòng lặp chạy trong một thread: nó lấy một Task sẵn sàng, chạy tới khi Task gặp `await` một thứ chưa xong, rồi chuyển sang Task khác. Vì vậy asyncio cho **đồng thời** (nhiều việc có tiến độ chồng lấp) chứ không **song song** (nhiều CPU chạy cùng lúc), và các coroutine trong một loop không bao giờ chạy cùng một thời điểm.

Dịch vụ báo giá trong lab là mô phỏng: `asyncio.sleep` thay cho độ trễ mạng, jitter sinh từ `random.Random(42)` nên con số lặp lại được (`main.py`, rút gọn: bỏ docstring):

```python
def make_latencies(count: int, *, seed: int = 42) -> list[float]:
    """Độ trễ mỗi lần gọi: 0,15 s cộng jitter 0..0,10 s, cố định theo seed."""
    rng = random.Random(seed)
    return [0.15 + rng.random() * 0.10 for _ in range(count)]


async def fetch_quote(distance_km: Decimal, latency: float) -> Quote:
    await asyncio.sleep(latency)
    fare = calculate_fare(distance_km)  # ValueError nếu ngoài miền
    return Quote(distance_km, Money(fare), datetime.now(UTC) + timedelta(minutes=5))
```

`await asyncio.sleep(latency)` trả quyền điều khiển cho loop trong lúc chờ, nên loop chạy được các coroutine khác. Điểm vào chuẩn của chương trình là `asyncio.run(main())`: nó tạo loop, chạy `main()` tới hết rồi đóng loop. Ta không cần tới `asyncio.get_event_loop()` kiểu cũ.

**Task** là coroutine đã được loop nhận và chạy nền. `asyncio.create_task(coro)` tạo Task; tài liệu nhắc phải giữ tham chiếu tới nó, vì loop chỉ giữ tham chiếu yếu và một Task không ai giữ có thể bị thu gom giữa chừng. Cách thường dùng không cần tự giữ là `gather` hoặc `TaskGroup`.

## gather và TaskGroup: mười lời gọi cùng lúc

Ba cách gọi mười báo giá, cùng danh sách độ trễ (`main.py`, rút gọn: bỏ type hint và docstring):

```python
async def quotes_sequential(latencies):
    return [await fetch_quote(d, lat) for d, lat in zip(DISTANCES, latencies)]


async def quotes_gather(latencies):
    return list(await asyncio.gather(*(fetch_quote(d, lat) for d, lat in zip(DISTANCES, latencies))))


async def quotes_taskgroup(latencies):
    async with asyncio.TaskGroup() as tg:
        tasks = [tg.create_task(fetch_quote(d, lat)) for d, lat in zip(DISTANCES, latencies)]
    return [t.result() for t in tasks]
```

Bản đầu `await` từng lời gọi một nên thời gian xấp xỉ tổng độ trễ. `gather` bọc mỗi coroutine thành Task rồi chờ tất cả, trả kết quả theo thứ tự đầu vào. `TaskGroup` (Python 3.11 trở lên) làm cùng việc nhưng ràng vòng đời các Task vào khối `async with`: ra khỏi khối là mọi Task đã xong. Số đo thật từ lab (một lần chạy, giây dao động theo máy):

```text
độ trễ giả: tổng 1.90 s, lớn nhất 0.24 s
tuần tự   : 1.92 s
gather    : 0.24 s
TaskGroup : 0.24 s
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Mười lời gọi báo giá nối đuôi nhau mất gần hai giây khi await tuần tự, và xếp chồng lên nhau chỉ mất bằng lời gọi chậm nhất khi chạy đồng thời."><path d="M20 34 L20 262" class="f-none s-line dash"/><path d="M220 34 L220 262" class="f-none s-line dash"/><path d="M420 34 L420 262" class="f-none s-line dash"/><path d="M620 34 L620 262" class="f-none s-line dash"/><path d="M820 34 L820 262" class="f-none s-line dash"/><text x="20" y="24" class="t-b">Tuần tự (await từng lời gọi): 1,92 s</text><rect x="20.0" y="34" width="85.6" height="26" class="f-phys-soft s-phys"/><text x="62.8" y="51" text-anchor="middle" class="t-small">1</text><rect x="105.6" y="34" width="61.0" height="26" class="f-pk-soft s-pk"/><text x="136.1" y="51" text-anchor="middle" class="t-small">2</text><rect x="166.6" y="34" width="71.0" height="26" class="f-phys-soft s-phys"/><text x="202.1" y="51" text-anchor="middle" class="t-small">3</text><rect x="237.6" y="34" width="68.9" height="26" class="f-pk-soft s-pk"/><text x="272.0" y="51" text-anchor="middle" class="t-small">4</text><rect x="306.5" y="34" width="89.5" height="26" class="f-phys-soft s-phys"/><text x="351.2" y="51" text-anchor="middle" class="t-small">5</text><rect x="396.0" y="34" width="87.1" height="26" class="f-pk-soft s-pk"/><text x="439.5" y="51" text-anchor="middle" class="t-small">6</text><rect x="483.0" y="34" width="95.7" height="26" class="f-phys-soft s-phys"/><text x="530.9" y="51" text-anchor="middle" class="t-small">7</text><rect x="578.7" y="34" width="63.5" height="26" class="f-pk-soft s-pk"/><text x="610.5" y="51" text-anchor="middle" class="t-small">8</text><rect x="642.2" y="34" width="76.9" height="26" class="f-phys-soft s-phys"/><text x="680.6" y="51" text-anchor="middle" class="t-small">9</text><rect x="719.1" y="34" width="61.2" height="26" class="f-pk-soft s-pk"/><text x="749.7" y="51" text-anchor="middle" class="t-small">10</text><text x="20" y="100" class="t-b">gather hoặc TaskGroup: 0,24 s</text><rect x="20" y="112" width="85.6" height="10" class="f-phys-soft s-phys"/><rect x="20" y="125" width="61.0" height="10" class="f-pk-soft s-pk"/><rect x="20" y="138" width="71.0" height="10" class="f-phys-soft s-phys"/><rect x="20" y="151" width="68.9" height="10" class="f-pk-soft s-pk"/><rect x="20" y="164" width="89.5" height="10" class="f-phys-soft s-phys"/><rect x="20" y="177" width="87.1" height="10" class="f-pk-soft s-pk"/><rect x="20" y="190" width="95.7" height="10" class="f-phys-soft s-phys"/><rect x="20" y="203" width="63.5" height="10" class="f-pk-soft s-pk"/><rect x="20" y="216" width="76.9" height="10" class="f-phys-soft s-phys"/><rect x="20" y="229" width="61.2" height="10" class="f-pk-soft s-pk"/><path d="M20 268 L820 268" class="f-none s-muted"/><text x="20" y="288" text-anchor="middle" class="t-small">0 s</text><text x="220" y="288" text-anchor="middle" class="t-small">0,5 s</text><text x="420" y="288" text-anchor="middle" class="t-small">1 s</text><text x="620" y="288" text-anchor="middle" class="t-small">1,5 s</text><text x="820" y="288" text-anchor="middle" class="t-small">2 s</text></svg>
</div>
<figcaption>Hình 1. Mười lời gọi với đúng các độ trễ của lab. Chờ nối đuôi thì cộng dồn độ trễ; chờ chồng lên nhau thì tổng thời gian chỉ còn bằng lời gọi lâu nhất.</figcaption>
</figure>

Tổng độ trễ là 1,90 giây, tuần tự mất 1,92 giây (phần dư là chi phí của loop). Đồng thời mất 0,24 giây, đúng bằng độ trễ lớn nhất. Cả ba cách trả cùng danh sách cước, từ 12.000 cho 0 km tới 57.000 cho 12 km. Chênh lệch thời gian không đến từ việc tính cước nhanh hơn mà từ việc **chờ** chồng lên nhau: loop không có gì làm trong lúc một Task chờ thì chuyển sang Task khác.

## Một lời gọi lỗi: gather khác TaskGroup thế nào

Thêm vào hai lời gọi chậm 0,3 giây và một lời gọi hỏng: khoảng cách 150 km nằm ngoài miền 0 đến 100 km của bài 03 nên `calculate_fare` ném `ValueError` sau 0,05 giây. Hai lời gọi chậm ghi lại chúng chạy xong hay bị hủy (`main.py`, rút gọn: bỏ docstring):

```python
async def slow_sibling(events: list[str], name: str) -> str:
    try:
        await asyncio.sleep(0.3)
        events.append(f"{name}: chạy xong")
        return name
    except asyncio.CancelledError:
        events.append(f"{name}: bị hủy")
        raise  # luôn ném lại, xem mục «Hủy» trong bài
```

```python
async def gather_failure() -> list[str]:
    sibling_events: list[str] = []
    lines: list[str] = []
    try:
        await asyncio.gather(
            failing_quote(), slow_sibling(sibling_events, "A"), slow_sibling(sibling_events, "B")
        )
    except ValueError as exc:
        lines.append(f"gather ném ValueError: {exc}")
    # gather không hủy các Task còn lại: chờ thêm để thấy chúng vẫn chạy.
    await asyncio.sleep(0.4)
    # Thứ tự A/B kết thúc hoặc bị hủy không được asyncio bảo đảm, nên sắp xếp trước khi in.
    return lines + sorted(sibling_events)


async def taskgroup_failure() -> list[str]:
    sibling_events: list[str] = []
    lines: list[str] = []
    try:
        async with asyncio.TaskGroup() as tg:
            tg.create_task(failing_quote())
            tg.create_task(slow_sibling(sibling_events, "A"))
            tg.create_task(slow_sibling(sibling_events, "B"))
    except* ValueError as group:  # except* bắt từng loại trong ExceptionGroup
        lines.append(f"TaskGroup ném ExceptionGroup: {[str(e) for e in group.exceptions]}")
    await asyncio.sleep(0.4)
    return lines + sorted(sibling_events)  # sắp xếp: thứ tự hủy A, B không được bảo đảm
```

Thứ tự hai Task anh em kết thúc hoặc bị hủy **không** được asyncio bảo đảm (khi review, thứ tự bị hủy đã ra «B trước A»), nên lab sắp xếp các dòng này trước khi in; nội dung từng dòng mới là điều cần so sánh. Kết quả (trích):

```text
gather:
   gather ném ValueError: khoảng cách ngoài miền 0..100 km: 150
   A: chạy xong
   B: chạy xong
TaskGroup:
   TaskGroup ném ExceptionGroup: ['khoảng cách ngoài miền 0..100 km: 150']
   A: bị hủy
   B: bị hủy
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Với gather, lời gọi hỏng kết thúc lúc 0,05 giây còn hai lời gọi chậm vẫn chạy tới 0,3 giây. Với TaskGroup, hai lời gọi chậm bị hủy ngay lúc 0,05 giây."><rect x="20" y="20" width="410" height="260" rx="10" class="f-surface s-line"/><text x="36" y="46" class="t-b">gather</text><text x="36" y="70" class="t-small">lời gọi hỏng</text><rect x="60" y="78" width="50" height="18" class="f-warn-soft s-warn"/><text x="120" y="91" class="t-small">ValueError ở 0,05 s</text><text x="36" y="120" class="t-small">A</text><rect x="60" y="128" width="300" height="18" class="f-phys-soft s-phys"/><text x="70" y="141" class="t-small">vẫn chạy, xong ở 0,30 s</text><text x="36" y="170" class="t-small">B</text><rect x="60" y="178" width="300" height="18" class="f-phys-soft s-phys"/><text x="70" y="191" class="t-small">vẫn chạy, xong ở 0,30 s</text><path d="M110 60 L110 215" class="f-none s-warn dash"/><text x="36" y="250" class="t-small">Lỗi tới người gọi lúc 0,05 s, kết quả A và B bị bỏ</text><rect x="450" y="20" width="410" height="260" rx="10" class="f-surface s-line"/><text x="466" y="46" class="t-b">TaskGroup</text><text x="466" y="70" class="t-small">lời gọi hỏng</text><rect x="490" y="78" width="50" height="18" class="f-warn-soft s-warn"/><text x="550" y="91" class="t-small">ValueError ở 0,05 s</text><text x="466" y="120" class="t-small">A</text><rect x="490" y="128" width="50" height="18" class="f-dead s-dead"/><text x="548" y="141" class="t-small">bị hủy ở 0,05 s</text><rect x="540" y="128" width="250" height="18" class="f-none s-dead dash"/><text x="466" y="170" class="t-small">B</text><rect x="490" y="178" width="50" height="18" class="f-dead s-dead"/><text x="548" y="191" class="t-small">bị hủy ở 0,05 s</text><rect x="540" y="178" width="250" height="18" class="f-none s-dead dash"/><path d="M540 60 L540 215" class="f-none s-warn dash"/><text x="466" y="250" class="t-small">Khối async with kết thúc, lỗi gói trong ExceptionGroup</text></svg>
</div>
<figcaption>Hình 2. Cùng một lời gọi hỏng, hai cách xử lý khác nhau với các Task anh em: để chúng chạy tiếp hoặc hủy ngay khi nhóm đã thất bại.</figcaption>
</figure>

Hành vi này đúng theo tài liệu: khi một Task trong `TaskGroup` thất bại bằng exception khác `CancelledError`, các Task còn lại bị hủy; còn `gather` mặc định để chúng chạy tiếp. Với màn hình báo giá, `gather` mặc định có nghĩa là chín lời gọi vẫn tốn kết nối và CPU của service `pricing` cho một kết quả không ai dùng. Vì lỗi có thể đến từ nhiều Task cùng lúc, `TaskGroup` gom chúng thành `ExceptionGroup` ([PEP 654](https://peps.python.org/pep-0654/)); cú pháp `except*` bắt từng loại trong nhóm, lỗi không khớp tiếp tục bay lên. Nếu muốn mọi kết quả kể cả lỗi, dùng `gather(..., return_exceptions=True)` rồi tự phân loại.

| | `gather` | `TaskGroup` |
|---|---|---|
| Một Task lỗi | Lỗi đầu tiên tới người gọi, Task khác chạy tiếp | Task còn lại bị hủy, lỗi gói trong `ExceptionGroup` |
| Vòng đời Task | Khi có lỗi, trả lỗi ngay không chờ Task còn lại; Task chưa xong bị hủy khi `asyncio.run` kết thúc | Mọi Task xong hoặc bị hủy trước khi thoát `async with` |
| Muốn lấy cả lỗi lẫn kết quả | `return_exceptions=True` | Tự bắt lỗi trong từng coroutine |

## Timeout: không chờ vô hạn

Service `pricing` có thể treo. Màn hình báo giá thà hiện cước ước tính cục bộ còn hơn quay mãi. `asyncio.timeout()` (3.11 trở lên) là context manager đặt hạn chót cho khối bên trong (`main.py`):

```python
async def quote_with_timeout(distance_km: Decimal, latency: float, limit: float) -> tuple[str, Decimal]:
    try:
        async with asyncio.timeout(limit):
            quote = await fetch_quote(distance_km, latency)
            return "service", quote.fare.amount
    except TimeoutError:
        # asyncio.timeout hủy đoạn bên trong, rồi đổi CancelledError thành TimeoutError ở đây.
        return "fallback", calculate_fare(distance_km)
```

```text
dịch vụ nhanh (0,10 s): nguồn=service, cước=25500, mất 0.10 s
dịch vụ chậm  (2,00 s): nguồn=fallback, cước=25500, mất 0.30 s
```

Cơ chế: hết hạn, `timeout` hủy Task đang chạy khối bên trong; Task nhận `CancelledError` tại `await` đang treo, và khi `CancelledError` ra tới `async with`, `timeout` đổi nó thành `TimeoutError`. Hệ quả cần nhớ: `TimeoutError` chỉ bắt được **bên ngoài** khối `async with`, không bắt được ở bên trong. Lời gọi chậm 2 giây mất đúng 0,30 giây, không phải 2 giây. Hai nguồn cho cùng cước 25.500 vì cả hai dùng chung công thức; trong thực tế fallback nên được đánh dấu là «ước tính» để người dùng và hệ thống biết.

Thời hạn nên đặt ở **biên** của một thao tác trọn vẹn (một yêu cầu của khách), không rải từng lời gọi nhỏ mà quên tổng. Ngoài ra `asyncio.timeout` chỉ có tác dụng nếu đoạn bên trong thực sự nhường quyền ở `await`: một vòng lặp tính toán không có `await` không bị ngắt giữa chừng (mục «Không chặn event loop»).

## Hủy: CancelledError, dọn dẹp và điều không được làm

Hủy trong asyncio là **hợp tác**: `task.cancel()` chỉ đặt yêu cầu, và Task nhận `CancelledError` tại `await` kế tiếp. Timeout, `TaskGroup` và việc khách đóng màn hình đều đi qua cơ chế này. `CancelledError` kế thừa `BaseException` chứ không phải `Exception` (đã kiểm trên Python 3.14), nên `except Exception` không bắt nhầm nó.

Bài toán của Tới Nơi: khi báo giá được tạo, hệ thống **giữ chỗ giá** cho chuyến trong lúc chờ khách bấm đặt. Nếu khách bỏ đi, chỗ giữ phải được nhả (`main.py`):

```python
async def hold_quote(trip_id: str, holds: set[str], log: list[str]) -> None:
    holds.add(trip_id)
    try:
        await asyncio.sleep(10)  # chờ khách bấm đặt
    except asyncio.CancelledError:
        log.append(f"{trip_id}: nhận CancelledError, đang dọn")
        raise  # ném lại để Task kết thúc ở trạng thái cancelled
    finally:
        holds.discard(trip_id)  # chạy cả khi xong bình thường, lỗi hay bị hủy
        log.append(f"{trip_id}: đã nhả giữ chỗ")
```

```text
đang giữ chỗ trước khi hủy: ['TN-0001']
   TN-0001: nhận CancelledError, đang dọn
   TN-0001: đã nhả giữ chỗ
   người gọi thấy CancelledError khi await task
đang giữ chỗ sau khi hủy  : []
task.cancelled() = True
nuốt CancelledError -> vẫn trả kết quả | cancelled() = False
timeout 0,10 s bọc coroutine nuốt hủy -> TimeoutError: False, expired(): True, kết quả: chạy tiếp sau hạn, mất 0.20 s
```

Phần dọn dẹp thuộc `finally`, vì nó chạy trong mọi trường hợp. Quy tắc quan trọng nhất: **bắt `CancelledError` để dọn thì phải ném lại** (`raise`). Dòng áp chót của output là phản ví dụ trong lab: một coroutine nuốt `CancelledError` rồi chạy tiếp thì Task vẫn trả kết quả và `cancelled()` là `False`. `timeout` và `TaskGroup` đều giả định hủy có hiệu lực; một coroutine nuốt nó thì việc vẫn tiếp tục chạy sau hạn chót. Dòng cuối của output là thí nghiệm trong lab (`stubborn_quote`, bọc trong `asyncio.timeout(0.1)`): không có `TimeoutError` dù `expired()` là `True`, và khối chạy 0,20 s thay vì 0,10 s. Tài liệu cũng nói: nếu thực sự cần chặn hủy thì phải gọi thêm `uncancel()`, và việc này không khuyến khích. Đoạn dọn trong `finally` cũng không nên kéo dài hay chờ mạng vô hạn, vì Task đang bị hủy mà vẫn chờ lâu thì cả nhóm chờ theo.

Thí nghiệm trong lab (`main.py`, rút gọn: bỏ docstring và phần đo giờ):

```python
async def stubborn_quote() -> str:
    try:
        await asyncio.sleep(1)
    except asyncio.CancelledError:
        pass  # nuốt: timeout tưởng đã cắt xong
    await asyncio.sleep(0.1)
    return "chạy tiếp sau hạn"


async def timeout_vs_swallow() -> dict[str, object]:
    raised = False
    result = ""
    try:
        async with asyncio.timeout(0.1) as scope:
            result = await stubborn_quote()
    except TimeoutError:
        raised = True
    return {"raised": raised, "expired": scope.expired(), "result": result}
```

Người đã đọc [bài C# về async/await và cancellation](/articles/csharp-async-await-cancellation) sẽ thấy cùng ý: C# truyền `CancellationToken` tường minh qua từng lời gọi, còn asyncio hủy bằng exception bắn vào Task tại điểm `await`, nên không có tham số nào để quên truyền nhưng có exception không được nuốt.

## Không chặn event loop

Vì mọi coroutine chia sẻ một thread, một lời gọi **đồng bộ** chậm trong coroutine làm đứng tất cả. Lab đo bằng một Task «nhịp tim» tick mỗi 0,05 giây và ghi khoảng cách thật giữa hai tick, rồi chạy một lời gọi chặn 0,5 giây theo hai cách (`main.py`, rút gọn):

```python
def legacy_blocking_call(seconds: float) -> None:
    """Giả lập SDK đồng bộ (hoặc tính toán nặng): time.sleep giữ nguyên thread."""
    time.sleep(seconds)


async def blocking_in_coroutine(seconds: float) -> None:
    legacy_blocking_call(seconds)  # SAI: gọi thẳng trong coroutine, event loop đứng


async def offloaded_to_thread(seconds: float) -> None:
    await asyncio.to_thread(legacy_blocking_call, seconds)  # chạy ở thread khác
```

```text
gọi sync trực tiếp 0,5 s : khoảng cách tick lớn nhất 0.55 s (danh định 0,05 s)
asyncio.to_thread 0,5 s  : khoảng cách tick lớn nhất 0.05 s
cảnh báo slow callback (debug): trực tiếp [0.503], to_thread []
```

Gọi trực tiếp, nhịp tim đứng 0,55 giây: trong khoảng đó mọi yêu cầu khác của service đều chờ, kể cả những yêu cầu chỉ cần 1 mili giây. `asyncio.to_thread` đưa hàm sang thread của thread pool, loop tự do nên nhịp tim giữ nguyên 0,05 giây. Lab chạy phần này riêng với `asyncio.run(..., debug=True)`; ở chế độ debug, asyncio ghi cảnh báo mức WARNING dạng `Executing <Task ...> took 0.503 seconds` cho callback chạy quá 0,1 giây, một cách tìm ra chỗ chặn trong code thật mà không cần đoán.

`to_thread` chỉ hợp với việc **chờ** (I/O đồng bộ, thư viện chưa có bản async). Theo tài liệu, do GIL nó thường chỉ giúp hàm I/O-bound không chặn loop; việc tính toán CPU nặng bằng Python thuần trong thread vẫn tranh GIL. Chọn thread hay process cho việc CPU-bound là chủ đề của bài 22.

## Chạy và kiểm chứng

Từ thư mục gốc repository (cần uv và Python 3.14, lab chỉ dùng thư viện chuẩn):

```bash
uv run --project examples/toi-noi-python/21-asyncio-task-timeout examples/toi-noi-python/21-asyncio-task-timeout/main.py --verify
```

Output đầy đủ của lần chạy cuối cùng, Python 3.14.7 trên macOS Apple Silicon:

```text
== 1. Tuần tự và đồng thời (10 lời gọi báo giá) ==
độ trễ giả: tổng 1.90 s, lớn nhất 0.24 s
tuần tự   : 1.92 s
gather    : 0.24 s
TaskGroup : 0.24 s
cước theo thứ tự đầu vào: 12000, 12000, 14250, 16500, 21000, 25500, 32250, 39000, 48000, 57000

== 2. Một lời gọi lỗi: gather và TaskGroup ==
gather:
   gather ném ValueError: khoảng cách ngoài miền 0..100 km: 150
   A: chạy xong
   B: chạy xong
TaskGroup:
   TaskGroup ném ExceptionGroup: ['khoảng cách ngoài miền 0..100 km: 150']
   A: bị hủy
   B: bị hủy

== 3. Timeout 0,30 s ==
dịch vụ nhanh (0,10 s): nguồn=service, cước=25500, mất 0.10 s
dịch vụ chậm  (2,00 s): nguồn=fallback, cước=25500, mất 0.30 s

== 4. Hủy và dọn dẹp ==
đang giữ chỗ trước khi hủy: ['TN-0001']
   TN-0001: nhận CancelledError, đang dọn
   TN-0001: đã nhả giữ chỗ
   người gọi thấy CancelledError khi await task
đang giữ chỗ sau khi hủy  : []
task.cancelled() = True
nuốt CancelledError -> vẫn trả kết quả | cancelled() = False
timeout 0,10 s bọc coroutine nuốt hủy -> TimeoutError: False, expired(): True, kết quả: chạy tiếp sau hạn, mất 0.20 s

== 5. Chặn event loop ==
gọi sync trực tiếp 0,5 s : khoảng cách tick lớn nhất 0.55 s (danh định 0,05 s)
asyncio.to_thread 0,5 s  : khoảng cách tick lớn nhất 0.05 s
cảnh báo slow callback (debug): trực tiếp [0.503], to_thread []

OK 20/20
```

20 ca của `--verify` chia hai nhóm. Nhóm logic lặp lại tuyệt đối: cước 0, 2, 2,5 và 5 km là 12.000, 12.000, 14.250 và 25.500; ba cách gọi cho cùng kết quả; `gather` để Task còn lại chạy xong, `TaskGroup` hủy chúng; timeout rơi vào fallback; giữ chỗ được nhả; nuốt `CancelledError` làm Task không còn `cancelled`, và `asyncio.timeout` bọc coroutine nuốt hủy không ném `TimeoutError`. Nhóm thời gian so theo tỉ lệ hoặc ngưỡng rộng, không so số giây tuyệt đối: `gather` nhanh hơn tuần tự ít nhất 3 lần, timeout dừng sau 0,28 giây và xa độ trễ 2 giây của dịch vụ, lời gọi chặn làm tick lớn nhất từ 0,45 giây trở lên, `to_thread` giữ tick lớn nhất dưới nửa lúc bị chặn. Ngưỡng vẫn có thể trượt trên máy rất bận; khi đó chạy lại.

| Hiện tượng | Kiểm tra trước |
|---|---|
| `RuntimeWarning: coroutine ... was never awaited` | Gọi `async def` mà quên `await` hoặc `create_task` |
| Đồng thời mà vẫn chậm bằng tuần tự | Có `await` từng lời gọi trong vòng `for` không, hay đã `gather` |
| Service đứng im khi có một yêu cầu nặng | Có hàm đồng bộ chậm trong coroutine không; bật `debug=True` xem cảnh báo `took N seconds` |
| `Task exception was never retrieved` | Task tạo bằng `create_task` mà không ai `await` hoặc giữ tham chiếu; dùng `TaskGroup` |
| `TimeoutError` không bị bắt, hoặc `timeout`/`TaskGroup` không cắt được việc | `except TimeoutError` phải nằm ngoài khối `async with asyncio.timeout(...)`; có coroutine nuốt `CancelledError` không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>«Service báo giá» ở đây chỉ là <code>asyncio.sleep</code>, chưa có kết nối mạng, retry, circuit breaker hay giới hạn số lời gọi đồng thời (bài 23 bàn tới <code>Semaphore</code>). Mười lời gọi cùng lúc không phải bài học chung: 10.000 lời gọi cùng lúc có thể làm sập chính dịch vụ phía sau. Số giây là của một máy, một lần chạy; chỉ tỉ lệ giữa các cách gọi mới đáng tin. Fallback ở đây dùng cùng công thức nên cho cùng giá; trong thực tế fallback thường kém chính xác hơn và phải được ghi nhận rõ.</p>
</div>

**Bài tập 1:** Trong `quotes_taskgroup`, thêm tham số `limit` và bọc cả khối `async with asyncio.TaskGroup()` trong `asyncio.timeout(limit)`. Với `limit = 0.2`, kết quả là gì khi độ trễ lớn nhất là 0,24 giây, và Task nào bị hủy?

**Bài tập 2:** Trong `hold_quote`, xóa dòng `raise` sau `log.append(...)`. `task.cancelled()` đổi thế nào, và vì sao đó là lỗi?

<details><summary>Xem lời giải</summary><p>(1) Khối <code>asyncio.timeout(0.2)</code> hết hạn khi vẫn còn các Task có độ trễ 0,214 đến 0,239 giây (bốn trong mười lời gọi: 0,214; 0,224; 0,218; 0,239). Timeout hủy Task đang chạy khối <code>async with</code>, <code>TaskGroup</code> hủy tiếp các Task con, và <code>TimeoutError</code> ra ngoài; không có kết quả nào được trả vì <code>t.result()</code> nằm sau khối. Muốn lấy kết quả một phần phải thu thập trong từng Task thay vì sau khối. (2) <code>cancelled()</code> thành <code>False</code> vì Task kết thúc bình thường, và <code>await task</code> không còn ném <code>CancelledError</code>. Đó là lỗi vì người hủy (timeout, TaskGroup, người dùng) tưởng Task đã dừng do bị hủy trong khi nó vẫn chạy tiếp những gì sau <code>except</code>.</p></details>

Bài trước: [bài 20](/articles/python-dong-goi-ruff-pre-commit-cau-truc-du-an). Bài tiếp: [bài 22](/articles/python-gil-threading-multiprocessing-free-threaded).

## Tài liệu gốc

- [asyncio: Coroutines and Tasks (Python 3.14)](https://docs.python.org/3.14/library/asyncio-task.html).
- [asyncio: Developing with asyncio, debug mode và blocking code](https://docs.python.org/3.14/library/asyncio-dev.html).
- [asyncio: Runner, asyncio.run](https://docs.python.org/3.14/library/asyncio-runner.html).
- [asyncio: Exceptions, CancelledError](https://docs.python.org/3.14/library/asyncio-exceptions.html).
- [PEP 654: Exception Groups and except*](https://peps.python.org/pep-0654/).

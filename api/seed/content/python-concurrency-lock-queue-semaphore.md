---
slug: python-concurrency-lock-queue-semaphore
title: "Lock, Queue, Semaphore: tránh nhận trùng chuyến"
summary: "Bài 23: tái hiện race hai tài xế cùng nhận một chuyến bằng asyncio và thread, sửa bằng asyncio.Lock, threading.Lock, giới hạn bằng Semaphore, điều tiết bằng Queue có maxsize. Có số đo thật và lab --verify."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, concurrency, asyncio]
language: vi
---

Lúc cao điểm [Tới Nơi](/articles/du-an-mau-toi-noi) tạo khoảng 70 chuyến mỗi giây, và một chuyến vừa vào trạng thái `searching` có thể hiện trên điện thoại của nhiều tài xế trong vùng. Nhiều người bấm «Nhận» trong cùng một giây, nhưng chỉ một người được nhận. Nếu code kiểm tra «chuyến còn trống không» rồi mới ghi tên tài xế, hai lời gọi đến sát nhau có thể cùng thấy «còn trống», cùng ghi, cùng nhận được câu trả lời «thành công». Hai tài xế cùng chạy tới đón một khách, hệ thống chỉ còn tên người ghi sau.

Đây là bài 23 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần quen asyncio ([bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation)) và biết thread và GIL ([bài 22](/articles/python-gil-threading-multiprocessing-free-threaded)). Bài C# cùng chủ đề là [concurrency và thread safety](/articles/csharp-concurrency-thread-safety); ở đây ta làm với coroutine và thread của Python, cộng thêm `Semaphore` và `Queue`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong asyncio, mọi coroutine chạy trên một thread và không bao giờ có hai coroutine chạy cùng lúc. Vậy hàm «kiểm tra chuyến còn trống rồi ghi tài xế» có cần <code>Lock</code> không?</p>
<details><summary>Xem đáp án</summary><p><b>Có, nếu giữa bước kiểm tra và bước ghi có một <code>await</code>.</b> Tại <code>await</code> coroutine nhường quyền, coroutine khác chen vào và thấy trạng thái cũ. Hai coroutine không chạy cùng thời điểm vẫn có thể xen kẽ. Lab chạy thử và in ra 20 «người thắng» cho một chuyến.</p></details>
</div>

## Hai câu trả lời thành công cho một chỗ

Lab dùng `Trip` rất gọn: mã chuyến, `driver_id` (ban đầu `None`) và trạng thái (`TripStatus`, 8 giá trị của Tới Nơi). Hàm nhận chuyến sai (`main.py`):

```python
async def accept_unsafe(trip: Trip, driver_id: str) -> bool:
    """SAI: kiểm tra, nhường quyền ở `await`, rồi mới ghi. Giữa hai bước, trạng thái có thể đã đổi."""
    if trip.driver_id is not None:  # check
        return False
    await asyncio.sleep(0)  # giả lập việc chờ I/O (xác minh tài xế, ghi log, gọi service khác)
    trip.driver_id = driver_id  # act: có thể ghi đè người đã nhận
    trip.status = TripStatus.ACCEPTED
    return True
```

Đây là mẫu **check-then-act**: quyết định dựa trên trạng thái đọc ở bước 1, còn bước 2 ghi như thể trạng thái đó vẫn đúng. `await asyncio.sleep(0)` đại diện cho việc chờ I/O thật; nó trả quyền cho loop mà không chờ thêm giây nào. Hai tài xế cùng nhận `TN-0001`, rồi hai mươi tài xế (`gather` như bài 21):

```text
không khóa, 2 tài xế : 2 người báo thắng, driver cuối = DR-002
không khóa, 20 tài xế: 20 người báo thắng, driver cuối = DR-020
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hai coroutine lần lượt đọc driver_id còn None, sau đó cả hai ghi: coroutine thứ hai ghi đè coroutine thứ nhất và cả hai đều trả True."><defs><marker id="py23-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="28" class="t-small">Bước 1</text><text x="230" y="28" class="t-small">Bước 2</text><text x="440" y="28" class="t-small">Bước 3</text><text x="650" y="28" class="t-small">Bước 4</text><rect x="20" y="42" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">DR-001 đọc</text><text x="34" y="92" class="t-small">driver_id = None</text><rect x="230" y="150" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="244" y="178" class="t-b">DR-002 đọc</text><text x="244" y="200" class="t-small">driver_id = None</text><rect x="440" y="42" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="454" y="70" class="t-b">DR-001 ghi</text><text x="454" y="92" class="t-small">trả True</text><rect x="650" y="150" width="200" height="70" rx="10" class="f-warn-soft s-warn"/><text x="664" y="178" class="t-b">DR-002 ghi đè</text><text x="664" y="200" class="t-small">cũng trả True</text><path d="M210 77 L230 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-0-arrow)"/><path d="M420 185 L440 112" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-0-arrow)"/><path d="M630 77 L650 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-0-arrow)"/><rect x="20" y="238" width="400" height="36" rx="8" class="f-none s-warn dash"/><text x="34" y="261" class="t-small">cả hai đã đọc «còn trống» trước khi ai ghi</text></svg>
</div>
<figcaption>Hình 1. Chỉ cần một điểm await giữa kiểm tra và ghi là hai coroutine xen kẽ được. Không cần hai CPU để race xảy ra.</figcaption>
</figure>

Hai mươi người báo thắng cho một chỗ; `driver_id` còn lại là của coroutine ghi sau cùng. Đừng viết test khẳng định «DR-020 thắng»: đó là hệ quả của lịch chạy, không phải điều ta muốn bảo vệ. Điều cần chứng minh là **số người thắng bằng 1**.

## asyncio.Lock: kiểm tra và ghi trong cùng vùng bảo vệ

`asyncio.Lock` bảo đảm tại mỗi thời điểm chỉ một coroutine ở trong khối `async with lock`. Sửa lỗi là đưa cả kiểm tra lẫn ghi vào cùng khối, kể cả đoạn `await` ở giữa (`main.py`):

```python
async def accept_locked(trip: Trip, driver_id: str, lock: asyncio.Lock) -> bool:
    """Đúng: kiểm tra VÀ ghi nằm trong cùng một vùng lock, kể cả đoạn await ở giữa."""
    async with lock:
        if trip.driver_id is not None:
            return False
        await asyncio.sleep(0)
        trip.driver_id = driver_id
        trip.status = TripStatus.ACCEPTED
        return True
```

```text
asyncio.Lock, 20 tài xế: 1 người thắng, driver = DR-001, status = accepted
```

Đúng một người thắng, trạng thái chuyển sang `accepted`, 19 người còn lại nhận `False` và có thể hiện «chuyến đã có người nhận». Theo tài liệu, lock của asyncio **công bằng**: coroutine được vào tiếp theo là coroutine chờ lâu nhất, nên ở đây `DR-001` thắng; ta vẫn chỉ nên khẳng định số người thắng, vì thứ tự đến thực tế do mạng quyết định.

Vài điều cần nhớ khi dùng:

- **Cùng một lock cho cùng một dữ liệu.** Hai hàm cùng sửa `Trip` mà dùng hai lock khác nhau thì không bảo vệ gì.
- **Không tái nhập.** Giữ lock rồi xin lại trong cùng coroutine sẽ chờ mãi. Lab bọc lần xin thứ hai trong `asyncio.timeout` của bài 21 để thấy `TimeoutError` thay vì treo.
- **Giữ ngắn.** Mọi coroutine khác xếp hàng chờ; đừng gọi mạng chậm trong khối lock nếu không bắt buộc.
- **Không dùng cho thread.** `asyncio.Lock`, `Semaphore` và `Queue` đều ghi rõ là không an toàn khi dùng từ nhiều thread.

## Thread cũng bị: Barrier để tái hiện, threading.Lock để sửa

Với thread, check-then-act cũng hỏng, nhưng khó tái hiện vì thread bị ngắt ở những thời điểm khó đoán. Thay vì cầu may, lab điều khiển lịch bằng `threading.Barrier`: mọi thread đọc xong rồi mới cùng ghi (`main.py`):

```python
def accept_thread_unsafe(trip: Trip, driver_id: str, barrier: threading.Barrier) -> bool:
    if trip.driver_id is not None:
        return False
    barrier.wait(timeout=5)  # điểm đồng bộ: thay cho việc cầu may bộ lập lịch
    trip.driver_id = driver_id
    trip.status = TripStatus.ACCEPTED
    return True


def accept_thread_locked(trip: Trip, driver_id: str, lock: threading.Lock) -> bool:
    with lock:
        if trip.driver_id is not None:
            return False
        time.sleep(0.005)  # giữ lock lâu hơn để cửa sổ race rộng; kết quả vẫn đúng
        trip.driver_id = driver_id
        trip.status = TripStatus.ACCEPTED
        return True
```

Pool phải có đủ 8 worker cho 8 tác vụ (`ThreadPoolExecutor(max_workers=count)`), nếu ít hơn, các thread đầu chờ ở Barrier tới hết `timeout=5` rồi nhận `BrokenBarrierError` sau 5 giây. Kết quả:

```text
không khóa, 8 thread (có Barrier): 8 người báo thắng, driver cuối = DR-007
threading.Lock, 8 thread          : 1 người thắng, driver = DR-001
```

Dòng đầu: tám người cùng thắng; tên «driver cuối» đổi theo lần chạy. Dòng hai: `threading.Lock` đưa về một người thắng dù ta cố ý ngủ 5 mili giây trong khối lock.

**Race thật, không có Barrier.** Barrier là cách của test để *ép* lịch xen kẽ. Bỏ nó đi thì code sai có tự lộ ra không? Lab đo hai phiên bản hàm sai, khác nhau đúng một lời gọi hàm giữa kiểm tra và ghi (`main.py`, rút gọn: bỏ docstring):

```python
def audit(driver_id: str) -> None: ...  # lời gọi hàm Python rỗng


def accept_thread_no_sync(trip: Trip, driver_id: str) -> bool:
    if trip.driver_id is not None:
        return False
    trip.driver_id = driver_id
    return True


def accept_thread_no_sync_call(trip: Trip, driver_id: str) -> bool:
    if trip.driver_id is not None:
        return False
    audit(driver_id)  # chỉ khác bản trên đúng dòng này
    trip.driver_id = driver_id
    return True
```

Mỗi lượt chạy 2.000 chuyến, mỗi chuyến 4 thread cùng nhận, đếm chuyến có hơn một người thắng; lặp 5 lượt, với `sys.setswitchinterval` mặc định (5 mili giây) và `1e-6` giây (yêu cầu chuyển thread dày đặc nhất). Kết quả trên build có GIL (một phần output của lần chạy `--verify` cuối; lab cũng in bytecode của hai hàm):

```text
Python 3.14.7, GIL bật, load average đầu: 16.7
bytecode khi chuyến còn trống:
  không có lời gọi: LOAD_ATTR POP_JUMP_IF_NONE LOAD_FAST_BORROW_LOAD_FAST_BORROW STORE_ATTR
  có một lời gọi  : LOAD_ATTR POP_JUMP_IF_NONE LOAD_GLOBAL LOAD_FAST_BORROW CALL POP_TOP LOAD_FAST_BORROW_LOAD_FAST_BORROW STORE_ATTR
biến thể (2000 chuyến x 4 thread)         số chuyến hỏng qua 5 lượt
không có lời gọi, interval mặc định       [0, 0, 0, 0, 0]
không có lời gọi, interval 1e-6 s         [0, 0, 0, 0, 0]
có một lời gọi hàm, interval mặc định     [0, 0, 0, 0, 0]
có một lời gọi hàm, interval 1e-6 s       [25, 15, 20, 29, 15]
```

Cùng bảng trên build free-threaded 3.14t (`uv run --no-project --python 3.14t …/main.py --race-table`, ngay sau lượt trên):

```text
Python 3.14.7, GIL tắt (free-threaded), load average đầu: 16.0
bytecode khi chuyến còn trống:
  không có lời gọi: LOAD_ATTR POP_JUMP_IF_NONE LOAD_FAST_BORROW_LOAD_FAST_BORROW STORE_ATTR
  có một lời gọi  : LOAD_ATTR POP_JUMP_IF_NONE LOAD_GLOBAL LOAD_FAST_BORROW CALL POP_TOP LOAD_FAST_BORROW_LOAD_FAST_BORROW STORE_ATTR
biến thể (2000 chuyến x 4 thread)         số chuyến hỏng qua 5 lượt
không có lời gọi, interval mặc định       [14, 17, 9, 11, 23]
không có lời gọi, interval 1e-6 s         [9, 13, 13, 11, 8]
có một lời gọi hàm, interval mặc định     [20, 27, 51, 30, 19]
có một lời gọi hàm, interval 1e-6 s       [25, 20, 16, 32, 35]
```

Đọc cụ thể từng dòng:

- **Có GIL, không có lời gọi: 0 trên cả 10 lượt**, kể cả khi yêu cầu chuyển thread dày đặc (`1e-6`). Hai dòng bytecode lab in ra cho thấy lý do: trên đường «chuyến còn trống», giữa `LOAD_ATTR` (đọc `driver_id`) và `STORE_ATTR` (ghi) không có lệnh `CALL`. Theo cách CPython đang cài, một thread chỉ nhường GIL khi chờ I/O hoặc ở điểm kiểm tra như vào hàm, gọi hàm, nhảy lùi trong vòng lặp; đoạn này không có điểm nào, nên thread khác không chen vào được. Đây là **chi tiết cài đặt**, không phải bảo đảm của ngôn ngữ.
- **Có GIL, thêm một lời gọi hàm, interval mặc định: vẫn 0.** Đã có chỗ để nhường GIL, nhưng yêu cầu chuyển thread chỉ xuất hiện sau khoảng 5 mili giây, còn đoạn giữa hai dòng chỉ dài cỡ vài chục lệnh. Đó là cách ta hiểu kết quả; lab không đo riêng cơ chế này.
- **Có GIL, thêm một lời gọi hàm, interval `1e-6`: 15 đến 29 chuyến hỏng trên 2.000.** Cùng logic sai, chỉ thêm một lời gọi và một tham số, race lộ ra ở mọi lượt.
- **3.14t (không có GIL): cả bốn dòng đều hỏng**, từ 8 đến 51 chuyến trên 2.000, kể cả bản không có lời gọi với interval mặc định. Số dao động mạnh giữa các lượt, nên chỉ đọc là «có lỗi, cỡ vài chục trên 2.000».

Bài học cho test: code sai chạy 2.000 chuyến trên build có GIL vẫn xanh, nhưng thêm một dòng log, đổi tham số chuyển thread hoặc chuyển sang build free-threaded (bài 22) là lỗi lộ ra. Test xanh nhờ **may mắn của một cài đặt** không chứng minh gì. Hãy tái hiện race bằng lịch xen kẽ điều khiển được (Barrier ở trên, `await` có chủ ý ở phần asyncio) và sửa bằng khóa, không dựa vào việc «hiện giờ chưa thấy lỗi». Đo trên Apple Silicon 10 lõi (8 hiệu năng, 2 tiết kiệm điện), load average 16 đến 18 lúc đo; số cụ thể sẽ khác ở máy khác.

Lưu ý: `threading.Lock` đặt trong coroutine sẽ chặn cả event loop (bài 21); `asyncio.Lock` chỉ có ý nghĩa giữa các coroutine của một loop.

## Semaphore: giới hạn số lời gọi đồng thời

Nhận chuyến cần loại trừ lẫn nhau (một người vào một lúc). Một nhu cầu khác là **giới hạn**: service `pricing` chỉ chịu được 10 lời gọi đồng thời. `asyncio.Semaphore(n)` giữ một bộ đếm n «giấy phép»; `async with sem` lấy một giấy phép và trả lại khi ra khỏi khối, hết giấy thì chờ (`main.py`, rút gọn):

```python
async def call_pricing(gauge: Gauge, limiter: asyncio.Semaphore | None) -> None:
    async def work() -> None:
        gauge.in_flight += 1
        gauge.peak = max(gauge.peak, gauge.in_flight)
        await asyncio.sleep(0.05)  # một lời gọi sang dịch vụ báo giá
        gauge.in_flight -= 1

    if limiter is None:
        await work()
    else:
        async with limiter:  # vào được khi còn "giấy phép"; ra khỏi khối là trả lại
            await work()
```

`Gauge` đếm số lời gọi đang chạy và đỉnh của nó. Một trăm lời gọi, mỗi lời 0,05 giây:

```text
không giới hạn  : đỉnh đồng thời 100, mất 0.05 s
Semaphore(10)   : đỉnh đồng thời  10, mất 0.51 s
```

Không giới hạn thì cả 100 lời gọi cùng bay tới service, mất 0,05 giây. Với `Semaphore(10)` đỉnh đúng bằng 10 và tổng thời gian xấp xỉ 10 lượt nối nhau (100 chia 10, nhân 0,05 giây). Ta đổi thời gian lấy việc phía sau không bị quá tải. Về ý nghĩa, `Lock` là trường hợp số giấy phép bằng 1. `asyncio.BoundedSemaphore` thêm bảo vệ: báo lỗi nếu `release()` nhiều hơn `acquire()`, hữu ích khi tự gọi `acquire`/`release` thay vì dùng `async with`.

## Queue: producer, consumer và backpressure

Cách thứ ba để tránh race là không chia sẻ dữ liệu: đưa yêu cầu vào hàng đợi và để các worker lấy ra xử lý. Với `asyncio.Queue` có hai bài toán nữa: **backpressure** (producer nhanh hơn consumer thì làm gì) và **dừng sạch** (báo consumer hết việc). Hàm `run_pipeline` của lab chạy cả ba cấu hình (`main.py`, rút gọn):

```python
queue: asyncio.Queue[str | None] = asyncio.Queue(maxsize=maxsize)

async def producer() -> None:
    for n in range(1, items + 1):
        await queue.put(f"TN-{n:04d}")  # đầy thì chờ ở đây
        peak = max(peak, queue.qsize())
    if use_shutdown:
        queue.shutdown()  # consumer xử lý nốt phần còn lại rồi nhận QueueShutDown
    else:
        for _ in range(consumers):
            await queue.put(None)  # sentinel: "hết việc"

async def consumer() -> None:
    while True:
        try:
            item = await queue.get()
        except asyncio.QueueShutDown:
            break
        try:
            if item is None:  # nhận sentinel thì thoát vòng lặp
                break
            await asyncio.sleep(0.02)  # xử lý một yêu cầu
            processed += 1
        finally:
            queue.task_done()  # mỗi get() phải có đúng một task_done()
```

(Trong lab `peak`, `processed` là biến `nonlocal` của hàm `run_pipeline`; phần trích bỏ khai báo đó cho gọn.) Ba điều trong code:

- **`maxsize`.** Queue đầy thì `await queue.put()` treo producer cho tới khi có chỗ. Đó là backpressure: tốc độ vào bị kéo về tốc độ ra. `maxsize=0` là không giới hạn, producer không bao giờ chờ.
- **`task_done` và `join`.** Mỗi `get()` phải được cân bằng bằng một `task_done()` (nên đặt trong `finally`); `await queue.join()` chờ tới khi mọi phần tử đã được đánh dấu xong. Sau `join` ta mới `gather` các consumer.
- **Dừng.** Cách cổ điển là **sentinel**: producer đặt vào mỗi consumer một giá trị `None` làm tín hiệu thoát. Từ Python 3.13 có `Queue.shutdown()`: producer gọi một lần, consumer lấy hết phần còn trong queue rồi `get()` ném `QueueShutDown`. Hai chi tiết theo tài liệu: một producer đang treo ở `put()` cũng nhận `QueueShutDown`, và `shutdown(immediate=True)` bỏ luôn phần còn lại trong queue (kể cả mở khóa `join()` dù việc chưa làm), nên chỉ dùng khi chấp nhận mất yêu cầu chưa xử lý.

30 yêu cầu, 3 consumer, mỗi yêu cầu 0,02 giây (bảng do lab in):

```text
cấu hình                   xử lý  peak qsize  producer xong  tất cả xong
maxsize=0 (không giới hạn)    30          30         0.00 s       0.20 s
maxsize=5 + sentinel          30           5         0.19 s       0.21 s
maxsize=5 + shutdown()        30           5         0.16 s       0.21 s
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Producer đẩy yêu cầu vào queue có năm chỗ, ba consumer lấy ra xử lý; khi queue đầy, lời gọi put của producer bị treo cho tới khi consumer lấy bớt."><defs><marker id="py23-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="85" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="115" class="t-b">producer</text><text x="36" y="139" class="t-small">await put(item)</text><path d="M210 125 L285 125" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-1-arrow)"/><text x="290" y="90" class="t-b">Queue(maxsize=5)</text><rect x="290" y="105" width="60" height="40" class="f-pk-soft s-pk"/><rect x="350" y="105" width="60" height="40" class="f-pk-soft s-pk"/><rect x="410" y="105" width="60" height="40" class="f-pk-soft s-pk"/><rect x="470" y="105" width="60" height="40" class="f-pk-soft s-pk"/><rect x="530" y="105" width="60" height="40" class="f-pk-soft s-pk"/><text x="296" y="130" class="t-mono">TN-0006</text><text x="356" y="130" class="t-mono">TN-0007</text><text x="416" y="130" class="t-mono">TN-0008</text><text x="476" y="130" class="t-mono">TN-0009</text><text x="536" y="130" class="t-mono">TN-0010</text><path d="M590 115 L690 55" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-1-arrow)"/><path d="M590 125 L690 125" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-1-arrow)"/><path d="M590 135 L690 195" class="f-none s-phys" stroke-width="2" marker-end="url(#py23-1-arrow)"/><rect x="690" y="30" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="706" y="60" class="t-b">consumer 1</text><rect x="690" y="100" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="706" y="130" class="t-b">consumer 2</text><rect x="690" y="170" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="706" y="200" class="t-b">consumer 3</text><rect x="290" y="170" width="300" height="60" rx="10" class="f-warn-soft s-warn"/><text x="304" y="196" class="t-b">Queue đầy: put() treo producer</text><text x="304" y="216" class="t-small">tốc độ vào bị kéo về tốc độ ra</text></svg>
</div>
<figcaption>Hình 2. Hàng đợi có giới hạn làm cầu nối: khi consumer chậm hơn, áp lực dồn ngược về producer thay vì tích tụ trong bộ nhớ.</figcaption>
</figure>

Cột `peak qsize` là bằng chứng. Không giới hạn, producer đẩy cả 30 yêu cầu trước khi consumer kịp lấy: đỉnh 30 và producer xong gần như tức thì (0,00 giây), tức toàn bộ backlog nằm trong bộ nhớ. Với `maxsize=5` đỉnh không vượt 5 và producer bị giữ lại tới 0,19 giây, gần bằng thời gian xử lý toàn bộ (0,21 giây). Tổng thời gian xong gần như không đổi (0,20 và 0,21 giây) vì nút thắt là ba consumer. Backpressure không làm hệ thống nhanh hơn, nó làm hệ thống **không phình**. Cả ba cách đều xử lý đủ 30 yêu cầu và dừng cả 3 consumer.

Queue còn giúp tránh khóa theo một cách khác: nếu mọi thay đổi của một chuyến đều đi qua đúng một consumer (ví dụ chia consumer theo mã chuyến), thì không còn hai bên cùng sửa dữ liệu đó và không cần lock cho nó. Điều kiện là mọi đường sửa chuyến đều phải đi qua queue; một đường tắt bên ngoài là đủ để race quay lại. Với thread thì dùng `queue.Queue` của thư viện chuẩn, không dùng `asyncio.Queue`.

## Chạy và kiểm chứng

Từ thư mục gốc repository (lab chỉ dùng thư viện chuẩn):

```bash
uv run --project examples/toi-noi-python/23-lock-queue-semaphore examples/toi-noi-python/23-lock-queue-semaphore/main.py --verify
```

Output đầy đủ của lần chạy cuối cùng, Python 3.14.7 trên macOS Apple Silicon:

```text
== 1. Race với asyncio: hai tài xế cùng nhận TN-0001 ==
không khóa, 2 tài xế : 2 người báo thắng, driver cuối = DR-002
không khóa, 20 tài xế: 20 người báo thắng, driver cuối = DR-020
asyncio.Lock, 20 tài xế: 1 người thắng, driver = DR-001, status = accepted
xin lock hai lần trong cùng coroutine: TimeoutError: lần xin thứ hai chờ mãi

== 2. Race với thread ==
không khóa, 8 thread (có Barrier): 8 người báo thắng, driver cuối = DR-007
threading.Lock, 8 thread          : 1 người thắng, driver = DR-001

Python 3.14.7, GIL bật, load average đầu: 16.7
bytecode khi chuyến còn trống:
  không có lời gọi: LOAD_ATTR POP_JUMP_IF_NONE LOAD_FAST_BORROW_LOAD_FAST_BORROW STORE_ATTR
  có một lời gọi  : LOAD_ATTR POP_JUMP_IF_NONE LOAD_GLOBAL LOAD_FAST_BORROW CALL POP_TOP LOAD_FAST_BORROW_LOAD_FAST_BORROW STORE_ATTR
biến thể (2000 chuyến x 4 thread)         số chuyến hỏng qua 5 lượt
không có lời gọi, interval mặc định       [0, 0, 0, 0, 0]
không có lời gọi, interval 1e-6 s         [0, 0, 0, 0, 0]
có một lời gọi hàm, interval mặc định     [0, 0, 0, 0, 0]
có một lời gọi hàm, interval 1e-6 s       [25, 15, 20, 29, 15]

== 3. Semaphore: 100 lời gọi báo giá, mỗi lời 0,05 s ==
không giới hạn  : đỉnh đồng thời 100, mất 0.05 s
Semaphore(10)   : đỉnh đồng thời  10, mất 0.51 s

== 4. Queue: 30 yêu cầu, 3 consumer, mỗi yêu cầu 0,02 s ==
cấu hình                   xử lý  peak qsize  producer xong  tất cả xong
maxsize=0 (không giới hạn)    30          30         0.00 s       0.20 s
maxsize=5 + sentinel          30           5         0.19 s       0.21 s
maxsize=5 + shutdown()        30           5         0.16 s       0.21 s

OK 20/20
```

20 ca `--verify` kiểm các số logic lặp lại được: 2, 20 và 8 người thắng khi không khóa, đúng 1 với cả hai loại Lock; Lock không tái nhập; đỉnh đồng thời 100 so với 10; ba cấu hình queue xử lý đủ 30 yêu cầu và dừng đủ 3 consumer; `maxsize=5` không vượt 5. Các ca thời gian dùng ngưỡng rộng (`Semaphore(10)` mất ít nhất 0,45 giây, producer có backpressure bị giữ từ 0,05 giây). Bảng race (số chuyến hỏng) không được kiểm vì phụ thuộc máy và build Python; nó chỉ để quan sát.

| Hiện tượng | Kiểm tra trước |
|---|---|
| Hai người cùng báo «đã nhận» một chuyến, hoặc test race xanh mà vẫn nghi ngờ | Giữa kiểm tra và ghi có `await` hay lời gọi hàm mà không nằm trong cùng khối lock; test có ép được lịch xen kẽ không, hay chỉ chạy nhiều lần và hy vọng |
| Chương trình treo ở `async with lock` | Giữ lock rồi xin lại cùng lock trong cùng coroutine; hoặc quên `release` khi dùng `acquire` thủ công |
| `queue.join()` không bao giờ xong | Có `get()` nào thiếu `task_done()` (quên trong nhánh lỗi hoặc nhánh sentinel) |
| Bộ nhớ tăng dần khi tải cao | Queue không giới hạn (`maxsize=0`) trong khi producer nhanh hơn consumer |
| `BrokenBarrierError` sau 5 giây khi test race | Số thread thực sự chạy ít hơn số bên của `Barrier` (pool nhỏ hơn số tác vụ) |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Mọi khóa trong bài chỉ có hiệu lực <b>trong một process</b>. Tới Nơi chạy nhiều bản của cùng service: hai server tải cùng một chuyến thành hai đối tượng <code>Trip</code>, mỗi đối tượng có lock riêng, cả hai vẫn nhận được. Race giữa nhiều server phải được chặn ở nơi sở hữu dữ liệu, bằng giao dịch, khóa dòng hoặc cập nhật có điều kiện trong PostgreSQL (bài 25), không bằng <code>asyncio.Lock</code>. <code>Semaphore</code> và <code>Queue</code> cũng chỉ giới hạn trong một process; giới hạn toàn hệ thống cần rate limit dùng chung (bài 27). Queue trong bộ nhớ mất hết khi process chết; việc không được mất cần hàng đợi bền như Kafka hoặc job queue trên Redis.</p>
</div>

**Bài tập 1:** Trong `accept_locked`, chuyển dòng `if trip.driver_id is not None: return False` ra **trước** khối `async with lock`. Với 20 tài xế có còn đúng một người thắng không? Vì sao? (Ta đã chạy thử biến thể này.)

**Bài tập 2:** Trong `consumer`, bỏ `finally` và đặt `queue.task_done()` ngay sau dòng `processed += 1` (cuối thân `try`), để nhánh `if item is None: break` thoát vòng lặp mà không gọi `task_done()`. Chuyện gì xảy ra với `await queue.join()`? Nếu thay vào đó đặt `task_done()` ngay sau dòng `item = await queue.get()` thì sao?

<details><summary>Xem lời giải</summary><p>(1) <b>Không còn đúng.</b> Kiểm tra ngoài lock thì cả 20 coroutine đều đọc <code>driver_id is None</code> trước khi ai ghi, rồi lần lượt vào lock và cùng ghi; biến thể này cho 20 người thắng. Lock chỉ có tác dụng khi bao cả kiểm tra lẫn ghi. (2) Bộ đếm «chưa xong» của queue còn dư 3 phần tử (3 sentinel không bao giờ được <code>task_done</code>), nên <code>join()</code> chờ mãi và chương trình treo; ta đã thử bằng một bản consumer như vậy với <code>asyncio.timeout(1)</code> bọc <code>join()</code>: hết giờ với 3 phần tử chưa xong. Còn đặt <code>task_done()</code> ngay sau <code>get()</code> thì sentinel cũng được tính, <code>join()</code> xong bình thường; đặt trong <code>finally</code> thì còn đúng cả khi xử lý ném lỗi, đó là lý do lab chọn cách đó.</p></details>

Bài trước: [bài 22](/articles/python-gil-threading-multiprocessing-free-threaded). Bài tiếp: [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration).

## Tài liệu gốc

- [asyncio: Synchronization Primitives (Lock, Semaphore, Event)](https://docs.python.org/3.14/library/asyncio-sync.html).
- [asyncio: Queues, join, task_done và shutdown](https://docs.python.org/3.14/library/asyncio-queue.html).
- [threading: Lock, Barrier và các primitive đồng bộ cho thread](https://docs.python.org/3.14/library/threading.html).
- [queue: hàng đợi đồng bộ cho thread](https://docs.python.org/3.14/library/queue.html).
- [asyncio: Concurrency and Multithreading](https://docs.python.org/3.14/library/asyncio-dev.html).
